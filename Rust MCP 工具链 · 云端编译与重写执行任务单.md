# Rust MCP 工具链 · 纯 Rust 实现与 Android 强可行性编译指南（供 AI 使用）

> **说明**：本指南是供 AI 助手阅读并执行的标准化流程，用于生成 9 个 Operit 平台可用的 MCP 插件 ZIP 包。  
> **目标**：编译出全静态的 `aarch64-unknown-linux-musl` Rust 二进制，配合自动提权的 `index.js`，解决 Android 环境下的 EACCES 与动态库不兼容问题。  
> **最终交付物**：9 个扁平结构的 `.zip` 文件，直接放入用户指定的 `D:\Hermes_Agent_Desktop\Hermes_Download` 目录。

---

## 一、通用规范（每个插件必须遵守）

### 1. ZIP 包结构（扁平，无子文件夹）

```
插件名.zip
├── <二进制文件>
├── index.js
└── package.json
```

### 2. index.js 模板（内置自动提权）

```javascript
const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

const BIN_PATH = path.join(__dirname, '__BIN__');
const SERVER_NAME = '__NAME__';

try {
  fs.accessSync(BIN_PATH, fs.constants.X_OK);
} catch (_) {
  fs.chmodSync(BIN_PATH, 0o755);
  console.error(`[${SERVER_NAME}] Permissions fixed.`);
}

const child = spawn(BIN_PATH, [], {
  stdio: ['pipe', 'pipe', 'pipe'],
  env: process.env
});

process.stdin.pipe(child.stdin);
child.stdout.pipe(process.stdout);
child.stderr.pipe(process.stderr);
child.on('exit', code => process.exit(code || 0));
```

- `__BIN__` 替换为二进制文件名（如 `obscura`、`hotnews`）
- `__NAME__` 替换为插件名（必须使用下划线，如 `agentic_vision`）

**特殊处理**：`obscura` 插件的启动参数需改为 `spawn(BIN_PATH, ['mcp'], ...)`，即将 `spawn(BIN_PATH, [], ...)` 替换为 `spawn(BIN_PATH, ['mcp'], ...)`。

### 3. package.json 模板

```json
{
  "name": "__NAME__",
  "version": "1.0.0",
  "main": "index.js",
  "dependencies": {}
}
```

- `__NAME__` 必须全部使用下划线（如 `mcp_proxy`、`rust_mcp_filesystem`）

### 4. 编译要求

- 目标三元组：`aarch64-unknown-linux-musl`
- 编译工具：`cross`（自动使用 Docker 进行交叉编译，无需手动配置 NDK）
- 确保最终二进制已 `chmod +x`

---

## 二、项目清单与构建详情

### 类别 A：原生 Rust 项目（P0，共 7 个）

这些项目本身是 Rust 实现，直接 clone 并编译即可。

| #   | 插件名 (NAME)          | 仓库                                                      | 二进制 (BIN)             | 特殊编译参数                     | 启动参数      |
| --- | ------------------- | ------------------------------------------------------- | --------------------- | -------------------------- | --------- |
| 1   | obscura             | `https://github.com/h4ckf0r0day/obscura`                | `obscura`             | 无                          | `['mcp']` |
| 2   | agentic_vision      | `https://github.com/agentralabs/agentic-vision`         | `agentic-vision-mcp`  | `--bin agentic-vision-mcp` | 无         |
| 3   | rust_mcp_server     | `https://github.com/rust-mcp-stack/rust-mcp-server`     | `rust-mcp-server`     | 无                          | 无         |
| 4   | mcp_proxy           | `https://github.com/rust-mcp-stack/mcp-proxy`           | `mcp-proxy`           | 无                          | 无         |
| 5   | rust_mcp_filesystem | `https://github.com/rust-mcp-stack/rust-mcp-filesystem` | `rust-mcp-filesystem` | 无                          | 无         |
| 6   | rust_docs_mcp       | `https://github.com/rust-mcp-stack/rust-docs-mcp`       | `rust-docs-mcp`       | 无                          | 无         |
| 7   | typemill            | `https://github.com/rust-mcp-stack/typemill`            | `mill`                | `-p mill`                  | 无         |

**统一构建步骤（以 agentic_vision 为例）**：

```bash
git clone <仓库> src-<NAME>
cd src-<NAME>
cross build --target aarch64-unknown-linux-musl --release <特殊编译参数>
cp target/aarch64-unknown-linux-musl/release/<BIN> ..
cd ..
chmod +x <BIN>

# 生成 index.js 和 package.json，替换占位符
sed 's/__BIN__/<BIN>/g; s/__NAME__/<NAME>/g' ../templates/index.js > index.js
sed 's/__NAME__/<NAME>/g' ../templates/package.json > package.json

# 如有特殊启动参数（obscura），修改 index.js
# sed -i "s/spawn(BIN_PATH, \[\],/spawn(BIN_PATH, ['mcp'],/" index.js

zip -j <NAME>.zip <BIN> index.js package.json
```

清理临时文件，保留 `<NAME>.zip`。

### 类别 B：Rust 重写项目（P1，共 2 个）

原项目为 TypeScript，需按照功能需求用纯 Rust 重新实现。以下提供完整的源代码与依赖描述。

---

#### 8. hotnews

**功能**：多平台实时热搜聚合，调用 `https://api.vvhan.com/api/hotlist/{key}` 并发获取数据，整理为 Markdown 输出。  
**插件名**：`hotnews`  
**二进制名**：`hotnews`  
**原仓库**（仅参考）：`https://github.com/wopal-cn/mcp-hotnews-server`（TypeScript 实现，不直接使用）

**Cargo.toml**

```toml
[package]
name = "hotnews"
version = "1.0.0"
edition = "2021"

[[bin]]
name = "hotnews"
path = "src/main.rs"

[dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
reqwest = { version = "0.11", features = ["json"] }
anyhow = "1"
mcp-sdk = "0.1"
```

**src/main.rs**

```rust
use mcp_sdk::{McpServer, Tool, ToolResult};
use serde::Deserialize;

#[derive(Deserialize)]
struct HotlistArgs {
    sources: Option<Vec<i64>>,
}

const PLATFORM_MAP: &[(i64, &str, &str)] = &[
    (1, "zhihu", "知乎"),
    (2, "36kr", "36氪"),
    (3, "baidu", "百度"),
    (4, "bilibili", "B站"),
    (5, "weibo", "微博"),
    (6, "douyin", "抖音"),
    (7, "hupu", "虎扑"),
    (8, "douban", "豆瓣"),
    (9, "it", "IT新闻"),
];

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = McpServer::new("hotnews", "1.0.0")
        .add_tool(Tool::new(
            "get_hot_news",
            "获取多平台实时热搜",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "sources": {
                        "type": "array",
                        "items": {"type": "integer"},
                        "description": "平台ID列表: 1知乎,2 36氪,3百度,4B站,5微博,6抖音,7虎扑,8豆瓣,9IT新闻"
                    }
                },
                "required": ["sources"]
            }),
            |args: HotlistArgs| async move {
                let sources = args.sources.unwrap_or_else(|| {
                    PLATFORM_MAP.iter().map(|(i, _, _)| *i).collect()
                });
                let client = reqwest::Client::new();
                let mut handles = vec![];
                for &id in &sources {
                    if let Some((_, key, name)) = PLATFORM_MAP.iter().find(|(i, _, _)| *i == id) {
                        let url = format!("https://api.vvhan.com/api/hotlist/{}", key);
                        let client = client.clone();
                        handles.push(tokio::spawn(async move {
                            let resp = client.get(&url).send().await?;
                            let data: serde_json::Value = resp.json().await?;
                            let items = data["data"].as_array().cloned().unwrap_or_default();
                            anyhow::Ok((name.to_string(), items))
                        }));
                    }
                }
                let mut md = String::new();
                for handle in handles {
                    match handle.await? {
                        Ok((name, items)) => {
                            md.push_str(&format!("## {}\n", name));
                            for (i, item) in items.iter().take(10).enumerate() {
                                let title = item["title"].as_str().unwrap_or("");
                                let url = item["url"].as_str().unwrap_or("");
                                let hot = item["hot"].as_str().unwrap_or("N/A");
                                md.push_str(&format!(
                                    "{}. [{}]({})  🔥 {}\n",
                                    i + 1,
                                    title,
                                    url,
                                    hot
                                ));
                            }
                            md.push('\n');
                        }
                        Err(e) => md.push_str(&format!("Error: {}\n\n", e)),
                    }
                }
                Ok(ToolResult::text(md))
            },
        ))
        .build();
    server.run_stdio().await?;
    Ok(())
}
```

**构建步骤**：

```bash
mkdir hotnews && cd hotnews
# 创建 Cargo.toml 和 src/main.rs（内容如上）
cross build --target aarch64-unknown-linux-musl --release
cp target/aarch64-unknown-linux-musl/release/hotnews ..
cd ..
chmod +x hotnews
# 使用模板生成 index.js 和 package.json，替换占位符
zip -j hotnews.zip hotnews index.js package.json
```

---

#### 9. mcp_research_router

**功能**：研究策略路由网关。提供两个工具：

- `get_tool_list`：从下游 HTTP MCP 服务获取工具列表，支持可选查询过滤。
- `execute_tools`：并发调用下游工具并整合结果。
  依赖环境变量：`MCP_SERVER_URL`（必填）、`MCP_LLM_ENABLED`、`MCP_LLM_API_KEY`、`MCP_LLM_BASE_URL`、`MCP_LLM_MODEL`（LLM 部分可后续扩展，当前基础实现仅使用 `MCP_SERVER_URL`）。  
  **插件名**：`mcp_research_router`  
  **二进制名**：`mcp_research_router`  
  **原仓库**（仅参考）：`https://github.com/SpiritHerb/mcp-research-router`（TypeScript 实现，不直接使用）

**Cargo.toml**

```toml
[package]
name = "mcp_research_router"
version = "1.0.0"
edition = "2021"

[[bin]]
name = "mcp_research_router"
path = "src/main.rs"

[dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
reqwest = { version = "0.11", features = ["json"] }
anyhow = "1"
mcp-sdk = "0.1"
```

**src/main.rs**

```rust
use mcp_sdk::{McpServer, Tool, ToolResult};
use serde::Deserialize;
use std::env;

#[derive(Deserialize)]
struct GetToolListArgs {
    user_query: Option<String>,
    max_tools: Option<usize>,
}

#[derive(Deserialize)]
struct ExecuteToolsArgs {
    tools: Vec<ToolCallItem>,
}

#[derive(Deserialize)]
struct ToolCallItem {
    tool_name: String,
    arguments: serde_json::Value,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = McpServer::new("mcp_research_router", "1.0.0")
        .add_tool(Tool::new(
            "get_tool_list",
            "获取下游MCP服务的工具列表，支持查询过滤",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "user_query": {"type": "string"},
                    "max_tools": {"type": "integer"}
                }
            }),
            |args: GetToolListArgs| async move {
                let server_url =
                    env::var("MCP_SERVER_URL").unwrap_or_else(|_| "http://localhost:8000".into());
                let client = reqwest::Client::new();
                let resp = client
                    .get(&format!("{}/tools/list", server_url))
                    .send()
                    .await?
                    .json::<serde_json::Value>()
                    .await?;
                let tools = resp["tools"].as_array().cloned().unwrap_or_default();
                let mut out = String::from("Available tools:\n");
                let max = args.max_tools.unwrap_or(tools.len());
                for tool in tools.iter().take(max) {
                    let name = tool["name"].as_str().unwrap_or("");
                    let desc = tool["description"].as_str().unwrap_or("");
                    out.push_str(&format!("- **{}**: {}\n", name, desc));
                }
                Ok(ToolResult::text(out))
            },
        ))
        .add_tool(Tool::new(
            "execute_tools",
            "并发调用下游MCP工具并返回整合结果",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "tools": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "tool_name": {"type": "string"},
                                "arguments": {"type": "object"}
                            },
                            "required": ["tool_name", "arguments"]
                        }
                    }
                },
                "required": ["tools"]
            }),
            |args: ExecuteToolsArgs| async move {
                let server_url =
                    env::var("MCP_SERVER_URL").unwrap_or_else(|_| "http://localhost:8000".into());
                let client = reqwest::Client::new();
                let mut results = String::new();
                for item in args.tools {
                    let resp = client
                        .post(&format!("{}/tools/call", server_url))
                        .json(&serde_json::json!({
                            "name": item.tool_name,
                            "arguments": item.arguments
                        }))
                        .send()
                        .await?
                        .text()
                        .await?;
                    results.push_str(&format!(
                        "**{}** result:\n```json\n{}\n```\n\n",
                        item.tool_name, resp
                    ));
                }
                Ok(ToolResult::text(results))
            },
        ))
        .build();
    server.run_stdio().await?;
    Ok(())
}
```

**构建步骤**：

```bash
mkdir mcp_research_router && cd mcp_research_router
# 创建 Cargo.toml 和 src/main.rs
cross build --target aarch64-unknown-linux-musl --release
cp target/aarch64-unknown-linux-musl/release/mcp_research_router ..
cd ..
chmod +x mcp_research_router
# 使用模板生成 index.js 和 package.json
zip -j mcp_research_router.zip mcp_research_router index.js package.json
```

---

## 三、AI 执行时的注意事项

1. **环境准备**：确保执行环境已安装 `cross` (`cargo install cross`) 和 Docker/Podman。
2. **工作目录**：所有操作在一个临时工作目录下进行，最终 ZIP 文件可以整理到同一输出目录。
3. **占位符替换**：使用 `sed` 或直接编写脚本替换模板中的 `__BIN__` 和 `__NAME__`。
4. **权限**：二进制必须 `chmod +x`，打包前确认。
5. **清理**：打包完成后，可保留 ZIP 文件，删除临时源文件。
6. **命名**：`package.json` 中的 `name` 字段严格使用下划线（如 `mcp_research_router`）。

---

## 四、最终交付清单

AI 应生成以下 9 个 ZIP 文件，并存放到路径 `D:\Hermes_Agent_Desktop\Hermes_Download`：

1. `obscura.zip`
2. `agentic_vision.zip`
3. `rust_mcp_server.zip`
4. `mcp_proxy.zip`
5. `rust_mcp_filesystem.zip`
6. `rust_docs_mcp.zip`
7. `typemill.zip`
8. `hotnews.zip`
9. `mcp_research_router.zip`

所有包内部均为：一个静态二进制 + `index.js` + `package.json`。

---

**指南结束。AI 请严格按照上述说明，依次生成各个插件并打包为 ZIP，无需额外解释或输出过程文件，最终将 ZIP 放置在指定目录。**
