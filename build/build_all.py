#!/usr/bin/env python3
"""Batch build and package all 9 MCP plugins for Operit platform.
Usage: python build_all.py [--native] [--skip-build]
"""
import os, sys, json, shutil, zipfile, subprocess

BUILD_DIR = r"D:\Hermes_Agent_Desktop\Hermes_Download\build"
OUTPUT_DIR = r"D:\Hermes_Agent_Desktop\Hermes_Download"

TOOLCHAIN = "+stable-x86_64-pc-windows-gnu"
TARGET = "aarch64-unknown-linux-musl"

INDEX_BASE = os.path.join(BUILD_DIR, "templates", "index_base.js")
INDEX_OBSCURA_BASE = os.path.join(BUILD_DIR, "templates", "index_obscura_base.js")

PLUGINS = [
    # (name, bin_name, src_dir, obscura_template, build_args)
    ("obscura",             "obscura",             "src-obscura",             True,  []),
    ("agentic_vision",      "agentic-vision-mcp",  "src-agentic_vision",      False, ["--bin", "agentic-vision-mcp"]),
    ("rust_mcp_server",     "rust-mcp-server",     "src-rust_mcp_server",     False, ["-p", "rust-mcp-server"]),
    ("mcp_proxy",           "mcp-proxy",           "src-mcp_proxy",           False, ["-p", "mcp-proxy"]),
    ("rust_mcp_filesystem", "rust-mcp-filesystem", "src-rust-mcp-filesystem", False, []),
    ("rust_docs_mcp",       "rust-docs-mcp",       "src-rust_docs_mcp",       False, []),
    ("typemill",            "mill",                "src-typemill",            False, ["-p", "mill"]),
    ("hotnews",             "hotnews",             "src-hotnews",             False, []),
    ("mcp_research_router", "mcp_research_router", "src-mcp_research_router", False, []),
]

def create_index_js(name, bin_name, obscura_template=False):
    template_file = INDEX_OBSCURA_BASE if obscura_template else INDEX_BASE
    with open(template_file, 'r', encoding='utf-8') as f:
        content = f.read()
    content = content.replace('__BIN__', bin_name).replace('__NAME__', name)
    return content

def create_package_json(name):
    return json.dumps({"name": name, "version": "1.0.0", "main": "index.js", "dependencies": {}}, indent=2)

def build_project(src_dir, extra_args):
    """Run cargo build for the project"""
    src_path = os.path.join(BUILD_DIR, src_dir)
    cmd = ["cargo", TOOLCHAIN, "build", "--target", TARGET, "--release"]
    cmd.extend(extra_args)
    print(f"  Building: {' '.join(cmd)}")
    result = subprocess.run(cmd, cwd=src_path, capture_output=True, text=True, timeout=600)
    if result.returncode != 0:
        print(f"  BUILD FAILED:\n{result.stderr[-500:]}")
        return False
    print(f"  BUILD OK")
    return True

def package_plugin(name, bin_name, src_dir, obscura_template):
    """Package binary + index.js + package.json into ZIP"""
    src_path = os.path.join(BUILD_DIR, src_dir)
    bin_path = os.path.join(src_path, "target", TARGET, "release", bin_name)
    if not os.path.exists(bin_path):
        # Try without .exe extension
        if os.path.exists(bin_path + ".exe"):
            bin_path = bin_path + ".exe"
        else:
            print(f"  WARNING: binary not found at {bin_path}")
            return False

    tmp_dir = os.path.join(BUILD_DIR, f"pkg-{name}")
    os.makedirs(tmp_dir, exist_ok=True)

    shutil.copy2(bin_path, os.path.join(tmp_dir, bin_name))
    
    with open(os.path.join(tmp_dir, "index.js"), 'w', encoding='utf-8') as f:
        f.write(create_index_js(name, bin_name, obscura_template))
    
    with open(os.path.join(tmp_dir, "package.json"), 'w', encoding='utf-8') as f:
        f.write(create_package_json(name))
    
    zip_path = os.path.join(OUTPUT_DIR, f"{name}.zip")
    with zipfile.ZipFile(zip_path, 'w', zipfile.ZIP_DEFLATED) as zf:
        zf.write(os.path.join(tmp_dir, bin_name), bin_name)
        zf.write(os.path.join(tmp_dir, "index.js"), "index.js")
        zf.write(os.path.join(tmp_dir, "package.json"), "package.json")
    
    print(f"  Created: {zip_path} ({os.path.getsize(zip_path)} bytes)")
    shutil.rmtree(tmp_dir)
    return True

def main():
    skip_build = "--skip-build" in sys.argv
    
    print("=" * 60)
    print("MCP Plugin Builder - 9 plugins for Operit/Android")
    print(f"Target: {TARGET}")
    print("=" * 60)
    
    success = 0
    failed = []
    
    for name, bin_name, src_dir, obscura_tmpl, extra_args in PLUGINS:
        print(f"\n[{name}]")
        
        if not skip_build:
            if not build_project(src_dir, extra_args):
                failed.append(name)
                continue
        
        if package_plugin(name, bin_name, src_dir, obscura_tmpl):
            success += 1
        else:
            failed.append(name)
    
    print(f"\n{'=' * 60}")
    print(f"Results: {success}/{len(PLUGINS)} packaged successfully")
    if failed:
        print(f"Failed: {', '.join(failed)}")

if __name__ == "__main__":
    main()
