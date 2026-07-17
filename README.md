# Operit MCP Plugins / Operit MCP 插件集

[![Build MCP Plugins](https://github.com/jinzechen/Operit_MCPS/actions/workflows/build-mcp-plugins.yml/badge.svg)](https://github.com/jinzechen/Operit_MCPS/actions/workflows/build-mcp-plugins.yml)

> 9 cross-compiled MCP (Model Context Protocol) plugins for **Operit** on Android. All binaries are statically linked `aarch64-unknown-linux-musl` executables — zero runtime dependencies, ready to deploy.
>
> 9 个为 Android 平台 **Operit** 客户端交叉编译的 MCP 插件。全部静态链接为 `aarch64-unknown-linux-musl` 二进制，无运行时依赖，即装即用。

---

## Quick Start / 快速开始

1. Download the ZIP for the plugin you need from [Releases](https://github.com/jinzechen/Operit_MCPS/releases) or [Actions artifacts](https://github.com/jinzechen/Operit_MCPS/actions).
2. Import into Operit on your Android device.
3. Each ZIP contains: `binary` + `index.js` (auto-chmod) + `package.json`.

---

## Plugin Catalog / 插件目录

### 1. obscura — Headless Browser MCP / 无头浏览器

| | |
|---|---|
| **Description / 描述** | Full MCP server for the Obscura headless browser. Supports page navigation, DOM extraction, screenshots, network interception, and JavaScript execution — all via MCP tools. |
| | 完整的 Obscura 无头浏览器 MCP 服务器。支持页面导航、DOM 提取、截图、网络拦截和 JS 执行。 |
| **Original / 原项目** | [h4ckf0r0day/obscura](https://github.com/h4ckf0r0day/obscura) |
| **Binary / 二进制** | Prebuilt from official release (官方预编译) |
| **Special / 特殊** | `index.js` uses `spawn(bin, ['mcp'])` to launch in MCP mode |

---

### 2. agentic_vision — Vision Analysis MCP / 视觉分析

| | |
|---|---|
| **Description / 描述** | Lightweight vision MCP server. Tools: `analyze_image` (image file analysis), `ocr_text` (OCR via Tesseract), `image_info` (metadata extraction). |
| | 轻量级视觉 MCP 服务器。工具包括图片分析、OCR 文字识别、图片元信息提取。 |
| **Original / 原项目** | [agentralabs/agentic-vision](https://github.com/agentralabs/agentic-vision) (simplified Rust rewrite / 简化 Rust 重写) |
| **Binary / 二进制** | `agentic-vision-mcp` |

---

### 3. rust_mcp_server — Rust Dev Tools MCP / Rust 开发工具

| | |
|---|---|
| **Description / 描述** | Rich set of Rust development tools for LLM coding agents. Includes cargo wrappers (build, check, clippy, test, doc, update, add), rustup toolchain management, and workspace introspection. |
| | 面向 LLM 编程代理的 Rust 开发工具集。包含 cargo 封装、rustup 工具链管理、工作区检查等。 |
| **Original / 原项目** | [Vaiz/rust-mcp-server](https://github.com/Vaiz/rust-mcp-server) |
| **Binary / 二进制** | `rust-mcp-server` |

---

### 4. mcp_proxy — MCP Proxy Gateway / MCP 代理网关

| | |
|---|---|
| **Description / 描述** | Simple MCP proxy that forwards tool calls to an upstream MCP server. Tools: `proxy_list_tools` (list upstream tools), `proxy_call_tool` (forward tool calls), `proxy_health` (health check). Set `UPSTREAM_MCP_URL` env var to configure. |
| | 简洁的 MCP 代理，将工具调用转发到上游 MCP 服务。通过环境变量 `UPSTREAM_MCP_URL` 配置上游地址。 |
| **Original / 原项目** | [nuwax-ai/mcp-proxy](https://github.com/nuwax-ai/mcp-proxy) (simplified Rust rewrite / 简化 Rust 重写) |
| **Binary / 二进制** | `mcp-proxy` |

---

### 5. rust_mcp_filesystem — Filesystem MCP / 文件系统

| | |
|---|---|
| **Description / 描述** | Blazing-fast, feature-rich filesystem MCP server. 25+ tools: read/write/edit files, directory operations, zip/unzip, search, diff, media file handling, file info, and more. |
| | 极速功能丰富的文件系统 MCP 服务器。25+ 工具：文件读写编辑、目录操作、压缩解压、搜索比对、媒体文件处理等。 |
| **Original / 原项目** | [rust-mcp-stack/rust-mcp-filesystem](https://github.com/rust-mcp-stack/rust-mcp-filesystem) |
| **Binary / 二进制** | `rust-mcp-filesystem` |

---

### 6. rust_docs_mcp — Rust Documentation MCP / Rust 文档查询

| | |
|---|---|
| **Description / 描述** | Search and retrieve Rust crate documentation. Queries crates.io API and serves focused, up-to-date documentation to prevent AI coding agents from suggesting outdated APIs. |
| | 搜索和获取 Rust crate 文档。通过 crates.io API 提供最新文档，防止 AI 编程代理推荐过时的 API。 |
| **Original / 原项目** | [Govcraft/rust-docs-mcp-server](https://github.com/Govcraft/rust-docs-mcp-server) |
| **Binary / 二进制** | `rust-docs-mcp` |

---

### 7. typemill — Document Tools MCP / 文档处理

| | |
|---|---|
| **Description / 描述** | Markdown document processing tools. `format_markdown` (beautify/compact/GFM styles), `generate_toc` (table of contents), `count_words` (word/line/paragraph stats). |
| | Markdown 文档处理工具。支持格式化美化、目录生成、字数统计。 |
| **Original / 原项目** | Custom Rust implementation / 自定义 Rust 实现 |
| **Binary / 二进制** | `mill` |

---

### 8. hotnews — Hot News Aggregator MCP / 热搜聚合

| | |
|---|---|
| **Description / 描述** | Real-time multi-platform hot news aggregator. Fetches trending topics from 9 Chinese platforms: Zhihu, 36Kr, Baidu, Bilibili, Weibo, Douyin, Hupu, Douban, IT News. Outputs formatted Markdown. |
| | 实时多平台热搜聚合器。并发抓取 9 个中文平台热搜：知乎、36氪、百度、B站、微博、抖音、虎扑、豆瓣、IT新闻。 |
| **Original / 原项目** | [wopal-cn/mcp-hotnews-server](https://github.com/wopal-cn/mcp-hotnews-server) (Rust rewrite / Rust 重写) |
| **Binary / 二进制** | `hotnews` |

---

### 9. mcp_research_router — Research Router MCP / 研究策略路由

| | |
|---|---|
| **Description / 描述** | Research strategy router gateway. `get_tool_list` queries downstream MCP services for available tools with optional filtering. `execute_tools` concurrently calls multiple downstream tools and aggregates results. Configure via `MCP_SERVER_URL` env var. |
| | 研究策略路由网关。获取下游工具列表（支持查询过滤），并发调用多个下游工具并整合结果。通过环境变量 `MCP_SERVER_URL` 配置。 |
| **Original / 原项目** | [SpiritHerb/mcp-research-router](https://github.com/SpiritHerb/mcp-research-router) (Rust rewrite / Rust 重写) |
| **Binary / 二进制** | `mcp_research_router` |

---

## ZIP Structure / ZIP 结构

```
plugin-name.zip
├── <binary>        # aarch64-unknown-linux-musl static binary / 静态二进制
├── index.js        # Auto-chmod + stdio bridge / 自动提权 + 标准 IO 桥接
└── package.json    # Operit package manifest / Operit 包清单
```

## Build / 构建

```bash
# Local build with Docker (recommended)
bash build.sh

# CI: Push to main branch, GitHub Actions auto-builds all 9 plugins
# CI 自动构建：推送到 main 分支即可触发全部 9 个插件编译
git push origin main
```

## Requirements for Operit / Operit 运行要求

- Android device with Operit client installed
- All binaries are `aarch64-unknown-linux-musl` (static, no glibc dependency)
- Some plugins require environment variables (see individual descriptions)
- 部分插件需要设置环境变量（详见各插件说明）

## License / 许可证

Each plugin retains the license of its original project. Custom rewrites are MIT.
各插件保留原项目许可证，自定义重写部分使用 MIT 许可证。

---

**Built with / 构建于** [cross](https://github.com/cross-rs/cross) + [GitHub Actions](https://github.com/jinzechen/Operit_MCPS/actions) | **Target / 目标架构** `aarch64-unknown-linux-musl`
