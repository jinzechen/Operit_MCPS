use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{BufRead, Write};

#[derive(Deserialize)]
struct RpcReq { jsonrpc: Option<String>, id: Option<Value>, method: Option<String>, params: Option<Value> }

#[derive(Serialize)]
struct RpcRes { jsonrpc: String, id: Value, result: Option<Value>, error: Option<Value> }

fn info() -> Value { json!({"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"serverInfo":{"name":"rust_docs_mcp","version":"2.0.0"}}) }

fn tools() -> Value { json!({"tools":[
    {"name":"search_crates","description":"在 crates.io 搜索 Rust crate / Search crates.io","inputSchema":{"type":"object","properties":{"query":{"type":"string","description":"搜索关键词 / search keyword"},"limit":{"type":"integer","description":"最大结果数(默认10) / max results"}},"required":["query"]}},
    {"name":"get_crate_info","description":"获取指定 crate 的详细信息(版本/描述/依赖/文档链接) / Get crate details","inputSchema":{"type":"object","properties":{"name":{"type":"string","description":"crate 名称 / crate name"}},"required":["name"]}},
    {"name":"get_rustdoc","description":"获取标准库文档链接 / Get std docs link","inputSchema":{"type":"object","properties":{"item":{"type":"string","description":"标准库项名称(如 Vec,String) / std item name"}},"required":["item"]}}
]})}

async fn search_crates(query: &str, limit: usize) -> String {
    let url = format!("https://crates.io/api/v1/crates?q={}&per_page={}&sort=downloads", query, limit.min(20));
    match reqwest::get(&url).await {
        Ok(r) => match r.json::<Value>().await {
            Ok(d) => {
                let crates = d["crates"].as_array().cloned().unwrap_or_default();
                if crates.is_empty() { return format!("No crates found for \"{}\"", query); }
                let mut out = format!("## Crates matching \"{}\"\n\n", query);
                for c in crates.iter().take(limit) {
                    let name = c["name"].as_str().unwrap_or("");
                    let desc = c["description"].as_str().unwrap_or("(no description)");
                    let dl = c["downloads"].as_u64().unwrap_or(0);
                    let ver = c["max_stable_version"].as_str().unwrap_or(c["max_version"].as_str().unwrap_or("?"));
                    out.push_str(&format!("- **{}** v{} 📥{} — {}\n", name, ver, dl, desc));
                }
                out
            }
            Err(e) => format!("JSON error: {}", e),
        },
        Err(e) => format!("Network error: {}", e),
    }
}

async fn get_crate_info(name: &str) -> String {
    let url = format!("https://crates.io/api/v1/crates/{}", name);
    match reqwest::get(&url).await {
        Ok(r) => match r.json::<Value>().await {
            Ok(d) => {
                if let Some(err) = d["errors"].as_array() {
                    return format!("Crate not found: {}", err[0]["detail"].as_str().unwrap_or("unknown"));
                }
                let krate = &d["crate"];
                let name = krate["name"].as_str().unwrap_or("");
                let desc = krate["description"].as_str().unwrap_or("");
                let ver = krate["max_stable_version"].as_str().unwrap_or("?");
                let dl = krate["downloads"].as_u64().unwrap_or(0);
                let repo = krate["repository"].as_str().unwrap_or("N/A");
                let doc = krate["documentation"].as_str().unwrap_or("N/A");
                let license = krate["license"].as_str().unwrap_or("N/A");
                let homepage = krate["homepage"].as_str().unwrap_or("N/A");
                let created = krate["created_at"].as_str().unwrap_or("");
                
                // Get latest version's deps
                let deps_url = format!("https://crates.io/api/v1/crates/{}/{}/dependencies", name, ver);
                let deps_str = match reqwest::get(&deps_url).await {
                    Ok(r2) => match r2.json::<Value>().await {
                        Ok(d2) => {
                            let deps = d2["dependencies"].as_array().cloned().unwrap_or_default();
                            deps.iter().take(10).map(|d| {
                                format!("  - {} {}", d["crate_id"].as_str().unwrap_or(""), d["req"].as_str().unwrap_or(""))
                            }).collect::<Vec<_>>().join("\n")
                        }
                        Err(_) => "(could not fetch)".into(),
                    },
                    Err(_) => "(could not fetch)".into(),
                };
                
                format!(
                    "## {} v{}\n\n{}\n\n| Field | Value |\n|-------|-------|\n| Downloads | {} |\n| License | {} |\n| Repository | {} |\n| Documentation | {} |\n| Homepage | {} |\n| Created | {} |\n\n### Dependencies\n{}",
                    name, ver, desc, dl, license, repo, doc, homepage, created, deps_str
                )
            }
            Err(e) => format!("JSON error: {}", e),
        },
        Err(e) => format!("Network error: {}", e),
    }
}

fn get_rustdoc(item: &str) -> String {
    let item_lower = item.to_lowercase().replace(" ", "-");
    let base = format!("https://doc.rust-lang.org/std/?search={}", item);
    
    // Known std item links
    let direct = match item_lower.as_str() {
        "vec" | "vec!" => Some("https://doc.rust-lang.org/std/vec/struct.Vec.html"),
        "string" => Some("https://doc.rust-lang.org/std/string/struct.String.html"),
        "hashmap" => Some("https://doc.rust-lang.org/std/collections/struct.HashMap.html"),
        "option" => Some("https://doc.rust-lang.org/std/option/enum.Option.html"),
        "result" => Some("https://doc.rust-lang.org/std/result/enum.Result.html"),
        "iterator" => Some("https://doc.rust-lang.org/std/iter/trait.Iterator.html"),
        "box" => Some("https://doc.rust-lang.org/std/boxed/struct.Box.html"),
        "rc" => Some("https://doc.rust-lang.org/std/rc/struct.Rc.html"),
        "arc" => Some("https://doc.rust-lang.org/std/sync/struct.Arc.html"),
        "mutex" => Some("https://doc.rust-lang.org/std/sync/struct.Mutex.html"),
        "refcell" => Some("https://doc.rust-lang.org/std/cell/struct.RefCell.html"),
        "clone" => Some("https://doc.rust-lang.org/std/clone/trait.Clone.html"),
        "copy" => Some("https://doc.rust-lang.org/std/marker/trait.Copy.html"),
        "drop" => Some("https://doc.rust-lang.org/std/ops/trait.Drop.html"),
        "default" => Some("https://doc.rust-lang.org/std/default/trait.Default.html"),
        "display" => Some("https://doc.rust-lang.org/std/fmt/trait.Display.html"),
        "debug" => Some("https://doc.rust-lang.org/std/fmt/trait.Debug.html"),
        _ => None,
    };
    
    match direct {
        Some(url) => format!("## {} - Standard Library\n\nDirect link: {}\n\nSearch: {}", item, url, base),
        None => format!("## {} - Standard Library\n\nNo direct link found. Try searching:\n{}", item, base),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() { continue; }
        let req: RpcReq = match serde_json::from_str(&line) { Ok(r) => r, Err(_) => continue };
        let method = req.method.as_deref().unwrap_or("");
        let id = req.id.unwrap_or(Value::Null);
        let result = match method {
            "initialize" => Some(info()),
            "tools/list" => Some(tools()),
            "tools/call" => {
                let params = req.params.unwrap_or(Value::Null);
                let name = params["name"].as_str().unwrap_or("");
                let args = &params["arguments"];
                let content = match name {
                    "search_crates" => search_crates(args["query"].as_str().unwrap_or(""), args["limit"].as_u64().unwrap_or(10) as usize).await,
                    "get_crate_info" => get_crate_info(args["name"].as_str().unwrap_or("")).await,
                    "get_rustdoc" => get_rustdoc(args["item"].as_str().unwrap_or("")),
                    _ => format!("Unknown tool: {}", name),
                };
                Some(json!({"content":[{"type":"text","text":content}]}))
            }
            _ => None,
        };
        let resp = RpcRes { jsonrpc: "2.0".into(), id, result, error: None };
        writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
        stdout.flush()?;
    }
    Ok(())
}
