#!/usr/bin/env python3
"""Build MCP plugin ZIPs: replace template placeholders and package."""
import os, shutil, json

BUILD_DIR = r"D:\Hermes_Agent_Desktop\Hermes_Download\build"
OUTPUT_DIR = r"D:\Hermes_Agent_Desktop\Hermes_Download"
INDEX_BASE = os.path.join(BUILD_DIR, "templates", "index_base.js")
INDEX_OBSCURA_BASE = os.path.join(BUILD_DIR, "templates", "index_obscura_base.js")

PLUGINS = [
    # (name, bin_name, use_obscura_template, extra_build_args)
    ("obscura",            "obscura",            True,  []),
    ("agentic_vision",     "agentic-vision-mcp", False, ["--bin", "agentic-vision-mcp"]),
    ("rust_mcp_server",    "rust-mcp-server",    False, []),
    ("mcp_proxy",          "mcp-proxy",          False, []),
    ("rust_mcp_filesystem","rust-mcp-filesystem",False, []),
    ("rust_docs_mcp",      "rust-docs-mcp",      False, []),
    ("typemill",           "mill",               False, ["-p", "mill"]),
    ("hotnews",            "hotnews",            False, []),
    ("mcp_research_router","mcp_research_router",False, []),
]

SRC_MAP = {
    "obscura": "src-obscura",
    "agentic_vision": "src-agentic_vision",
    "rust_mcp_server": "src-rust-mcp-server",
    "mcp_proxy": "src-mcp-proxy",
    "rust_mcp_filesystem": "src-rust-mcp-filesystem",
    "rust_docs_mcp": "src-rust-docs-mcp",
    "typemill": "src-typemill",
    "hotnews": "src-hotnews",
    "mcp_research_router": "src-mcp_research_router",
}

def create_index_js(name, bin_name, obscura_template=False):
    template_file = INDEX_OBSCURA_BASE if obscura_template else INDEX_BASE
    with open(template_file, 'r', encoding='utf-8') as f:
        content = f.read()
    content = content.replace('__BIN__', bin_name)
    content = content.replace('__NAME__', name)
    return content

def create_package_json(name):
    return json.dumps({
        "name": name,
        "version": "1.0.0",
        "main": "index.js",
        "dependencies": {}
    }, indent=2)

def build_zip(name, bin_name, obscura_template):
    """Create ZIP from binary + index.js + package.json"""
    import zipfile
    
    # Find binary
    src_dir = os.path.join(BUILD_DIR, SRC_MAP[name])
    target_dir = os.path.join(src_dir, "target", "aarch64-unknown-linux-musl", "release")
    bin_path = os.path.join(target_dir, bin_name)
    
    if not os.path.exists(bin_path):
        print(f"  WARNING: binary not found at {bin_path}")
        return False
    
    tmp_dir = os.path.join(BUILD_DIR, f"pkg-{name}")
    os.makedirs(tmp_dir, exist_ok=True)
    
    # Copy binary
    shutil.copy2(bin_path, os.path.join(tmp_dir, bin_name))
    
    # Create index.js
    with open(os.path.join(tmp_dir, "index.js"), 'w', encoding='utf-8') as f:
        f.write(create_index_js(name, bin_name, obscura_template))
    
    # Create package.json
    with open(os.path.join(tmp_dir, "package.json"), 'w', encoding='utf-8') as f:
        f.write(create_package_json(name))
    
    # Create ZIP
    zip_path = os.path.join(OUTPUT_DIR, f"{name}.zip")
    with zipfile.ZipFile(zip_path, 'w', zipfile.ZIP_DEFLATED) as zf:
        zf.write(os.path.join(tmp_dir, bin_name), bin_name)
        zf.write(os.path.join(tmp_dir, "index.js"), "index.js")
        zf.write(os.path.join(tmp_dir, "package.json"), "package.json")
    
    print(f"  Created: {zip_path}")
    shutil.rmtree(tmp_dir)
    return True

if __name__ == "__main__":
    print("MCP Plugin Packager")
    print("=" * 50)
    for name, bin_name, obscura_tmpl, _ in PLUGINS:
        print(f"Packaging {name}...")
        build_zip(name, bin_name, obscura_tmpl)
    print("Done.")
