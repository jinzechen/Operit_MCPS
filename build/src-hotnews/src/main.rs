use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{BufRead, Write};

#[derive(Deserialize)]
struct JsonRpcRequest {
    #[allow(dead_code)]
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

const PLATFORM_MAP: &[(&str, &str)] = &[
    ("zhihu", "知乎"),
    ("36kr", "36氪"),
    ("baidu", "百度"),
    ("bilibili", "B站"),
    ("weibo", "微博"),
    ("douyin", "抖音"),
    ("hupu", "虎扑"),
    ("douban", "豆瓣"),
    ("it", "IT新闻"),
];

fn server_info() -> Value {
    json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {
            "tools": {}
        },
        "serverInfo": {
            "name": "hotnews",
            "version": "1.0.0"
        }
    })
}

fn tools_list() -> Value {
    json!({
        "tools": [{
            "name": "get_hot_news",
            "description": "获取多平台实时热搜",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "sources": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "平台key列表: zhihu,36kr,baidu,bilibili,weibo,douyin,hupu,douban,it"
                    }
                },
                "required": ["sources"]
            }
        }]
    })
}

async fn get_hot_news(sources: Vec<String>) -> String {
    let client = reqwest::Client::new();
    let mut handles = vec![];

    for key in &sources {
        if let Some((_, name)) = PLATFORM_MAP.iter().find(|(k, _)| k == key) {
            let url = format!("https://api.vvhan.com/api/hotlist/{}", key);
            let name = name.to_string();
            let client = client.clone();
            handles.push(tokio::spawn(async move {
                match client.get(&url).send().await {
                    Ok(resp) => match resp.json::<Value>().await {
                        Ok(data) => {
                            let items = data["data"].as_array().cloned().unwrap_or_default();
                            Ok::<(String, Vec<Value>), String>((name, items))
                        }
                        Err(e) => Err(format!("JSON parse error for {}: {}", name, e)),
                    },
                    Err(e) => Err(format!("Request error for {}: {}", name, e)),
                }
            }));
        }
    }

    let mut md = String::new();
    for handle in handles {
        match handle.await.unwrap_or(Err("task failed".into())) {
            Ok((name, items)) => {
                md.push_str(&format!("## {}\n\n", name));
                for (i, item) in items.iter().take(10).enumerate() {
                    let title = item["title"].as_str().unwrap_or("");
                    let url = item["url"].as_str().unwrap_or("");
                    let hot = item["hot"].as_str().unwrap_or("N/A");
                    md.push_str(&format!("{}. [{}]({})  🔥 {}\n", i + 1, title, url, hot));
                }
                md.push('\n');
            }
            Err(e) => md.push_str(&format!("Error: {}\n\n", e)),
        }
    }

    if md.is_empty() { "No results.".to_string() } else { md }
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
                    "get_hot_news" => {
                        let sources: Vec<String> = args["sources"]
                            .as_array()
                            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                            .unwrap_or_else(|| {
                                PLATFORM_MAP.iter().map(|(k, _)| k.to_string()).collect()
                            });
                        let content = get_hot_news(sources).await;
                        Some(json!({
                            "content": [{
                                "type": "text",
                                "text": content
                            }]
                        }))
                    }
                    _ => Some(json!({
                        "content": [{
                            "type": "text",
                            "text": format!("Unknown tool: {}", tool_name)
                        }],
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
