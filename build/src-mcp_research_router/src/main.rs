use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::env;
use std::io::{BufRead, Write};

#[derive(Deserialize)]
struct JsonRpcRequest {
    jsonrpc: Option<String>,
    id: Option<Value>,
    method: Option<String>,
    params: Option<Value>,
}

#[derive(Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Value>,
}

fn server_info() -> Value {
    json!({
        "protocolVersion": "2024-11-05",
        "capabilities": { "tools": {} },
        "serverInfo": { "name": "mcp_research_router", "version": "1.0.0" }
    })
}

fn tools_list() -> Value {
    json!({
        "tools": [
            {
                "name": "get_tool_list",
                "description": "获取下游MCP服务的工具列表，支持查询过滤",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "user_query": { "type": "string", "description": "可选的查询过滤字符串" },
                        "max_tools": { "type": "integer", "description": "最大返回工具数" }
                    }
                }
            },
            {
                "name": "execute_tools",
                "description": "并发调用下游MCP工具并返回整合结果",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "tools": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "tool_name": { "type": "string" },
                                    "arguments": { "type": "object" }
                                },
                                "required": ["tool_name", "arguments"]
                            }
                        }
                    },
                    "required": ["tools"]
                }
            }
        ]
    })
}

fn server_url() -> String {
    env::var("MCP_SERVER_URL").unwrap_or_else(|_| "http://localhost:8000".into())
}

async fn get_tool_list(user_query: Option<String>, max_tools: Option<usize>) -> String {
    let url = server_url();
    let client = reqwest::Client::new();
    match client.get(&format!("{}/tools/list", url)).send().await {
        Ok(resp) => match resp.json::<Value>().await {
            Ok(data) => {
                let tools = data["tools"].as_array().cloned().unwrap_or_default();
                let mut out = String::from("Available tools:\n\n");
                let max = max_tools.unwrap_or(tools.len());
                let filtered: Vec<&Value> = if let Some(ref q) = user_query {
                    tools.iter().filter(|t| {
                        let name = t["name"].as_str().unwrap_or("");
                        let desc = t["description"].as_str().unwrap_or("");
                        name.to_lowercase().contains(&q.to_lowercase())
                            || desc.to_lowercase().contains(&q.to_lowercase())
                    }).collect()
                } else {
                    tools.iter().collect()
                };
                for tool in filtered.iter().take(max) {
                    let name = tool["name"].as_str().unwrap_or("");
                    let desc = tool["description"].as_str().unwrap_or("");
                    out.push_str(&format!("- **{}**: {}\n", name, desc));
                }
                if out == "Available tools:\n\n" {
                    out.push_str("(no tools found)");
                }
                out
            }
            Err(e) => format!("Failed to parse tool list: {}", e),
        },
        Err(e) => format!("Failed to fetch tool list: {}", e),
    }
}

async fn execute_tools(tools: Vec<Value>) -> String {
    let url = server_url();
    let client = reqwest::Client::new();
    let mut results = String::new();

    for tool in &tools {
        let tool_name = tool["tool_name"].as_str().unwrap_or("unknown");
        let arguments = tool["arguments"].clone();

        match client
            .post(&format!("{}/tools/call", url))
            .json(&json!({ "name": tool_name, "arguments": arguments }))
            .send()
            .await
        {
            Ok(resp) => match resp.text().await {
                Ok(text) => {
                    results.push_str(&format!("**{}** result:\n```json\n{}\n```\n\n", tool_name, text));
                }
                Err(e) => {
                    results.push_str(&format!("**{}** error reading response: {}\n\n", tool_name, e));
                }
            },
            Err(e) => {
                results.push_str(&format!("**{}** request failed: {}\n\n", tool_name, e));
            }
        }
    }

    if results.is_empty() {
        "No results.".to_string()
    } else {
        results
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let req: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(_) => continue,
        };

        let method = req.method.as_deref().unwrap_or("");
        let id = req.id.unwrap_or(Value::Null);

        let result = match method {
            "initialize" => Some(server_info()),
            "tools/list" => Some(tools_list()),
            "tools/call" => {
                let params = req.params.unwrap_or(Value::Null);
                let tool_name = params["name"].as_str().unwrap_or("");
                let args = &params["arguments"];

                match tool_name {
                    "get_tool_list" => {
                        let user_query = args["user_query"].as_str().map(String::from);
                        let max_tools = args["max_tools"].as_u64().map(|v| v as usize);
                        let content = get_tool_list(user_query, max_tools).await;
                        Some(json!({ "content": [{ "type": "text", "text": content }] }))
                    }
                    "execute_tools" => {
                        let tools: Vec<Value> = args["tools"]
                            .as_array()
                            .cloned()
                            .unwrap_or_default();
                        let content = execute_tools(tools).await;
                        Some(json!({ "content": [{ "type": "text", "text": content }] }))
                    }
                    _ => Some(json!({
                        "content": [{ "type": "text", "text": format!("Unknown tool: {}", tool_name) }],
                        "isError": true
                    })),
                }
            }
            _ => None,
        };

        let response = JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id,
            result,
            error: None,
        };

        let resp_str = serde_json::to_string(&response)?;
        writeln!(stdout, "{}", resp_str)?;
        stdout.flush()?;
    }

    Ok(())
}
