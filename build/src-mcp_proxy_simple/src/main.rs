use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::io::{BufRead, Write};

#[derive(Deserialize)]
struct JsonRpcRequest { jsonrpc: Option<String>, id: Option<Value>, method: Option<String>, params: Option<Value> }

#[derive(Serialize)]
struct JsonRpcResponse { jsonrpc: String, id: Value, result: Option<Value>, error: Option<Value> }

fn upstream_url() -> String {
    env::var("UPSTREAM_MCP_URL").unwrap_or_else(|_| "http://localhost:8000".into())
}

fn server_info() -> Value {
    json!({"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"serverInfo":{"name":"mcp_proxy","version":"1.0.0"}})
}

fn tools_list() -> Value {
    json!({"tools":[
        {"name":"proxy_list_tools","description":"列出上游MCP服务的所有可用工具","inputSchema":{"type":"object","properties":{"query":{"type":"string","description":"可选的名称过滤关键字"}}}},
        {"name":"proxy_call_tool","description":"代理调用上游MCP工具","inputSchema":{"type":"object","properties":{"tool_name":{"type":"string","description":"工具名称"},"arguments":{"type":"object","description":"工具参数"}},"required":["tool_name"]}},
        {"name":"proxy_health","description":"检查上游MCP服务健康状态","inputSchema":{"type":"object","properties":{}}}
    ]})
}

async fn proxy_list_tools(query: Option<String>) -> String {
    let url = upstream_url();
    match reqwest::get(&format!("{}/tools/list", url)).await {
        Ok(resp) => match resp.json::<Value>().await {
            Ok(data) => {
                let tools = data["tools"].as_array().cloned().unwrap_or_default();
                let mut out = String::from("Upstream tools:\n\n");
                for tool in tools {
                    let name = tool["name"].as_str().unwrap_or("");
                    let desc = tool["description"].as_str().unwrap_or("");
                    if let Some(ref q) = query {
                        if !name.to_lowercase().contains(&q.to_lowercase()) { continue; }
                    }
                    out.push_str(&format!("- **{}**: {}\n", name, desc));
                }
                if out == "Upstream tools:\n\n" { out.push_str("(no tools found)") }
                out
            }
            Err(e) => format!("Failed to parse response: {}", e),
        },
        Err(e) => format!("Failed to connect to upstream ({}): {}\nSet UPSTREAM_MCP_URL env var.", url, e),
    }
}

async fn proxy_call_tool(tool_name: &str, arguments: &Value) -> String {
    let url = upstream_url();
    let client = reqwest::Client::new();
    match client.post(&format!("{}/tools/call", url)).json(&json!({"name":tool_name,"arguments":arguments})).send().await {
        Ok(resp) => match resp.text().await {
            Ok(text) => format!("**{}** result:\n```json\n{}\n```", tool_name, text),
            Err(e) => format!("Error reading response: {}", e),
        },
        Err(e) => format!("Request failed: {}", e),
    }
}

async fn proxy_health() -> String {
    let url = upstream_url();
    match reqwest::get(&url).await {
        Ok(_) => format!("Upstream MCP server at {} is reachable ✓", url),
        Err(e) => format!("Upstream MCP server at {} is NOT reachable: {}", url, e),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() { continue; }
        let req: JsonRpcRequest = match serde_json::from_str(&line) { Ok(r) => r, Err(_) => continue };
        let method = req.method.as_deref().unwrap_or("");
        let id = req.id.unwrap_or(Value::Null);
        let result = match method {
            "initialize" => Some(server_info()),
            "tools/list" => Some(tools_list()),
            "tools/call" => {
                let params = req.params.unwrap_or(Value::Null);
                let name = params["name"].as_str().unwrap_or("");
                let args = &params["arguments"];
                let content = match name {
                    "proxy_list_tools" => proxy_list_tools(args["query"].as_str().map(String::from)).await,
                    "proxy_call_tool" => proxy_call_tool(args["tool_name"].as_str().unwrap_or(""), args).await,
                    "proxy_health" => proxy_health().await,
                    _ => format!("Unknown tool: {}", name),
                };
                Some(json!({"content":[{"type":"text","text":content}]}))
            }
            _ => None,
        };
        let resp = JsonRpcResponse { jsonrpc: "2.0".into(), id, result, error: None };
        writeln!(stdout, "{}", serde_json::to_string(&resp)?)?;
        stdout.flush()?;
    }
    Ok(())
}
