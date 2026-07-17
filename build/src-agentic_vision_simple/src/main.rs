use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{BufRead, Write};
use std::process::Command;

#[derive(Deserialize)]
struct JsonRpcRequest { jsonrpc: Option<String>, id: Option<Value>, method: Option<String>, params: Option<Value> }

#[derive(Serialize)]
struct JsonRpcResponse { jsonrpc: String, id: Value, result: Option<Value>, error: Option<Value> }

fn server_info() -> Value { json!({"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"serverInfo":{"name":"agentic_vision","version":"1.0.0"}}) }

fn tools_list() -> Value {
    json!({"tools":[
        {"name":"analyze_image","description":"分析图片文件(base64/路径)返回描述信息","inputSchema":{"type":"object","properties":{"image_path":{"type":"string","description":"图片文件路径或base64数据"}},"required":["image_path"]}},
        {"name":"ocr_text","description":"对图片进行OCR文字识别(需系统安装tesseract)","inputSchema":{"type":"object","properties":{"image_path":{"type":"string","description":"图片路径"}},"required":["image_path"]}},
        {"name":"image_info","description":"获取图片元信息(尺寸/格式等)","inputSchema":{"type":"object","properties":{"image_path":{"type":"string","description":"图片路径"}},"required":["image_path"]}}
    ]})
}

fn analyze_image(path: &str) -> String {
    if path.starts_with("data:") || path.starts_with("/9j/") || path.len() > 100 {
        return "Image analysis: base64 data detected. Size estimation and format analysis would be performed by the AI client.".into();
    }
    match std::fs::metadata(path) {
        Ok(m) => format!("Image file found: {} ({} bytes). Use OCR or image analysis tools on the client side for detailed analysis.", path, m.len()),
        Err(e) => format!("Cannot access image: {}", e),
    }
}

fn image_info(path: &str) -> String {
    match std::fs::metadata(path) {
        Ok(m) => {
            let ext = std::path::Path::new(path).extension().and_then(|e| e.to_str()).unwrap_or("unknown");
            format!("File: {}\nSize: {} bytes\nExtension: {}\nType hint: {}", path, m.len(), ext,
                match ext.to_lowercase().as_str() {
                    "png" => "PNG image", "jpg"|"jpeg" => "JPEG image",
                    "gif" => "GIF image", "webp" => "WebP image",
                    "bmp" => "BMP image", "svg" => "SVG vector",
                    _ => "Unknown format"
                })
        }
        Err(e) => format!("Cannot access file: {}", e),
    }
}

fn ocr_text(path: &str) -> String {
    match Command::new("tesseract").arg(path).arg("stdout").arg("-l").arg("chi_sim+eng").output() {
        Ok(out) if out.status.success() => {
            String::from_utf8_lossy(&out.stdout).into_owned()
        }
        Ok(out) => format!("Tesseract exited with error:\n{}", String::from_utf8_lossy(&out.stderr)),
        Err(_) => "Tesseract OCR not installed on this system. Install: apt-get install tesseract-ocr tesseract-ocr-chi-sim".into(),
    }
}

fn main() -> anyhow::Result<()> {
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
                    "analyze_image" => analyze_image(args["image_path"].as_str().unwrap_or("")),
                    "ocr_text" => ocr_text(args["image_path"].as_str().unwrap_or("")),
                    "image_info" => image_info(args["image_path"].as_str().unwrap_or("")),
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
