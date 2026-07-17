use serde::Deserialize;
use serde_json::{json, Value};
use std::io::{BufRead, Write};

#[derive(Deserialize)]
struct RpcReq { jsonrpc: Option<String>, id: Option<Value>, method: Option<String>, params: Option<Value> }

fn info() -> Value { json!({"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"serverInfo":{"name":"rust_docs_mcp","version":"2.0.1"}}) }

fn tools() -> Value { json!({"tools":[
    {"name":"search_crates","description":"在 crates.io 搜索 Rust crate","inputSchema":{"type":"object","properties":{"query":{"type":"string","description":"搜索关键词"},"limit":{"type":"integer","description":"最大结果数(默认10)"}},"required":["query"]}},
    {"name":"get_crate_info","description":"获取指定 crate 的详细信息","inputSchema":{"type":"object","properties":{"name":{"type":"string","description":"crate 名称"}},"required":["name"]}},
    {"name":"get_rustdoc","description":"获取标准库文档链接","inputSchema":{"type":"object","properties":{"item":{"type":"string","description":"标准库项名称(如 Vec,String)"}},"required":["item"]}}
]})}

fn search_crates(query: &str, limit: usize) -> String {
    let url = format!("https://crates.io/api/v1/crates?q={}&per_page={}&sort=downloads", urlencoding(query), limit.min(20));
    match ureq::get(&url).call() {
        Ok(r) => match r.into_json::<Value>() {
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

fn get_crate_info(name: &str) -> String {
    let url = format!("https://crates.io/api/v1/crates/{}", urlencoding(name));
    match ureq::get(&url).call() {
        Ok(r) => match r.into_json::<Value>() {
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
                format!("## {} v{}\n\n{}\n\n| Field | Value |\n|-------|-------|\n| Downloads | {} |\n| License | {} |\n| Repository | {} |\n| Documentation | {} |\n| Homepage | {} |\n| Created | {} |", name, ver, desc, dl, license, repo, doc, homepage, created)
            }
            Err(e) => format!("JSON error: {}", e),
        },
        Err(e) => format!("Network error: {}", e),
    }
}

fn get_rustdoc(item: &str) -> String {
    let base = format!("https://doc.rust-lang.org/std/?search={}", urlencoding(item));
    let direct = match item.to_lowercase().replace(" ", "-").as_str() {
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
        "clone" => Some("https://doc.rust-lang.org/std/clone/trait.Clone.html"),
        "default" => Some("https://doc.rust-lang.org/std/default/trait.Default.html"),
        _ => None,
    };
    match direct {
        Some(url) => format!("## {}\n\nDirect: {}\n\nSearch: {}", item, url, base),
        None => format!("## {}\n\nSearch: {}", item, base),
    }
}

fn urlencoding(s: &str) -> String {
    s.chars().map(|c| match c {
        ' ' => "%20".into(),
        c if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' => c.to_string(),
        _ => format!("%{:02X}", c as u8),
    }).collect()
}

fn main() {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let line = match line { Ok(l) => l, Err(_) => break };
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
                    "search_crates" => search_crates(args["query"].as_str().unwrap_or(""), args["limit"].as_u64().unwrap_or(10) as usize),
                    "get_crate_info" => get_crate_info(args["name"].as_str().unwrap_or("")),
                    "get_rustdoc" => get_rustdoc(args["item"].as_str().unwrap_or("")),
                    _ => format!("Unknown tool: {}", name),
                };
                Some(json!({"content":[{"type":"text","text":content}]}))
            }
            _ => None,
        };
        let resp = json!({"jsonrpc":"2.0","id":id,"result":result});
        let _ = writeln!(stdout, "{}", serde_json::to_string(&resp).unwrap());
        let _ = stdout.flush();
    }
}
