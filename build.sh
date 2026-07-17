#!/bin/bash
# MCP Plugin Builder — Build all 9 plugins for aarch64-unknown-linux-musl
# Prerequisites: Docker installed
# Usage: bash build.sh

set -e

BUILD_DIR="$(cd "$(dirname "$0")" && pwd)/build"
OUTPUT_DIR="$(cd "$(dirname "$0")" && pwd)"

PLUGINS=(
    "obscura:src-obscura:obscura:"
    "agentic_vision:src-agentic_vision:agentic-vision-mcp:--bin agentic-vision-mcp"
    "rust_mcp_server:src-rust_mcp_server:rust-mcp-server:-p rust-mcp-server"
    "mcp_proxy:src-mcp_proxy:mcp-proxy:-p mcp-proxy"
    "rust_mcp_filesystem:src-rust-mcp-filesystem:rust-mcp-filesystem:"
    "rust_docs_mcp:src-rust_docs_mcp:rust-docs-mcp:"
    "typemill:src-typemill:mill:-p mill"
    "hotnews:src-hotnews:hotnews:"
    "mcp_research_router:src-mcp_research_router:mcp_research_router:"
)

echo "=== MCP Plugin Builder ==="
echo "Building 9 plugins for aarch64-unknown-linux-musl"
echo ""

# Check cross or docker
if command -v cross &>/dev/null; then
    CMD="cross"
    echo "Using: cross"
elif command -v docker &>/dev/null; then
    echo "Installing cross via cargo..."
    cargo install cross
    CMD="cross"
else
    echo "ERROR: Neither cross nor Docker found."
    echo "Install Docker or run: cargo install cross"
    exit 1
fi

for plugin in "${PLUGINS[@]}"; do
    IFS=':' read -r name dir bin args <<< "$plugin"
    echo ""
    echo "=== Building $name ==="
    
    cd "$BUILD_DIR/$dir"
    
    # Build
    $CMD build --target aarch64-unknown-linux-musl --release $args
    
    # Copy binary
    cp "target/aarch64-unknown-linux-musl/release/$bin" "$BUILD_DIR/"
    cd "$BUILD_DIR"
    chmod +x "$bin"
    
    # Generate index.js
    if [ "$name" = "obscura" ]; then
        TEMPLATE="$BUILD_DIR/templates/index_obscura_base.js"
    else
        TEMPLATE="$BUILD_DIR/templates/index_base.js"
    fi
    sed "s/__BIN__/$bin/g; s/__NAME__/$name/g" "$TEMPLATE" > "$OUTPUT_DIR/${name}_index.js"
    
    # Generate package.json
    echo "{\"name\":\"$name\",\"version\":\"1.0.0\",\"main\":\"index.js\",\"dependencies\":{}}" > "$OUTPUT_DIR/${name}_package.json"
    
    # Create ZIP
    cd "$OUTPUT_DIR"
    zip -j "$name.zip" "$BUILD_DIR/$bin" "${name}_index.js" "${name}_package.json"
    
    echo "Created: $name.zip"
done

echo ""
echo "=== Done! ==="
ls -la "$OUTPUT_DIR"/*.zip
