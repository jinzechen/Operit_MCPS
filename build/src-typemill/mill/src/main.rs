use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
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
        "serverInfo": { "name": "typemill", "version": "1.0.0" }
    })
}

fn tools_list() -> Value {
    json!({
        "tools": [
            {
                "name": "format_markdown",
                "description": "格式化 Markdown 文本，规范化标题、列表和代码块",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "text": { "type": "string", "description": "要格式化的 Markdown 文本" },
                        "style": {
                            "type": "string",
                            "enum": ["prettier", "compact", "gfm"],
                            "description": "格式化风格"
                        }
                    },
                    "required": ["text"]
                }
            },
            {
                "name": "generate_toc",
                "description": "为 Markdown 文档生成目录",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "text": { "type": "string", "description": "Markdown 文档内容" },
                        "min_level": { "type": "integer", "description": "最小标题级别(默认2)", "default": 2 }
                    },
                    "required": ["text"]
                }
            },
            {
                "name": "count_words",
                "description": "统计文档的字数、行数和段落数",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "text": { "type": "string", "description": "要统计的文本" }
                    },
                    "required": ["text"]
                }
            }
        ]
    })
}

fn format_markdown(text: &str, style: &str) -> String {
    let mut result = String::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            if let Some(pos) = trimmed.find(' ') {
                result.push_str(&format!("#{} {}\n", &trimmed[..pos], trimmed[pos..].trim()));
            } else {
                result.push_str(&format!("{}\n", trimmed));
            }
        } else if trimmed.starts_with('-') || trimmed.starts_with('*') {
            match style {
                "compact" => result.push_str(&format!("{}\n", trimmed)),
                _ => result.push_str(&format!("  {}\n", trimmed)),
            }
        } else if trimmed.is_empty() {
            if !result.ends_with("\n\n") {
                result.push('\n');
            }
        } else {
            result.push_str(&format!("{}\n", trimmed));
        }
    }
    result.trim().to_string()
}

fn generate_toc(text: &str, min_level: usize) -> String {
    let mut toc = String::from("## Table of Contents\n\n");
    let mut count = 0;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            let level = trimmed.chars().take_while(|c| *c == '#').count();
            if level >= min_level && level <= 6 {
                let title = trimmed[level..].trim();
                let anchor = title.to_lowercase()
                    .replace(' ', "-")
                    .replace(|c: char| !c.is_alphanumeric() && c != '-', "");
                let indent = "  ".repeat(level - min_level);
                toc.push_str(&format!("{}- [{}](#{})\n", indent, title, anchor));
                count += 1;
            }
        }
    }
    if count == 0 {
        "No headings found at the specified level.".to_string()
    } else {
        toc
    }
}

fn count_words(text: &str) -> String {
    let lines = text.lines().count();
    let chars = text.chars().count();
    let words = text.split_whitespace().count();
    let paragraphs = text.split("\n\n").filter(|p| !p.trim().is_empty()).count();
    format!(
        "## Document Statistics\n\n| Metric | Count |\n|--------|-------|\n| Characters | {} |\n| Words | {} |\n| Lines | {} |\n| Paragraphs | {} |\n",
        chars, words, lines, paragraphs
    )
}

fn main() -> anyhow::Result<()> {
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
                    "format_markdown" => {
                        let text = args["text"].as_str().unwrap_or("");
                        let style = args["style"].as_str().unwrap_or("gfm");
                        let content = format_markdown(text, style);
                        Some(json!({ "content": [{ "type": "text", "text": content }] }))
                    }
                    "generate_toc" => {
                        let text = args["text"].as_str().unwrap_or("");
                        let min_level = args["min_level"].as_u64().unwrap_or(2) as usize;
                        let content = generate_toc(text, min_level);
                        Some(json!({ "content": [{ "type": "text", "text": content }] }))
                    }
                    "count_words" => {
                        let text = args["text"].as_str().unwrap_or("");
                        let content = count_words(text);
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
