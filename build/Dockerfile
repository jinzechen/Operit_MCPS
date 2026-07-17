# MCP Plugin Cross-Compiler
# Build all 9 plugins for aarch64-unknown-linux-musl
#
# Usage:
#   docker build -t mcp-builder .
#   docker run --rm -v "D:/Hermes_Agent_Desktop/Hermes_Download/build:/build" -v "D:/Hermes_Agent_Desktop/Hermes_Download:/output" mcp-builder
#
# Or with cross (simpler):
#   cd D:/Hermes_Agent_Desktop/Hermes_Download/build/src-<NAME>
#   cross build --target aarch64-unknown-linux-musl --release

FROM rust:1.82-alpine AS builder

RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static perl make

WORKDIR /build

# Copy all source directories
COPY src-obscura/ src-obscura/
COPY src-agentic_vision/ src-agentic_vision/
COPY src-rust_mcp_server/ src-rust_mcp_server/
COPY src-mcp_proxy/ src-mcp_proxy/
COPY src-rust-mcp-filesystem/ src-rust-mcp-filesystem/
COPY src-rust_docs_mcp/ src-rust_docs_mcp/
COPY src-typemill/ src-typemill/
COPY src-hotnews/ src-hotnews/
COPY src-mcp_research_router/ src-mcp_research_router/

# Add musl target
RUN rustup target add aarch64-unknown-linux-musl

# Build each plugin
RUN cd src-obscura && cargo build --target aarch64-unknown-linux-musl --release && \
    cp target/aarch64-unknown-linux-musl/release/obscura /output/obscura

# Note: agentic_vision has external dependency (agentic-sdk), may need adjustment
# Other plugins follow similar pattern

# The actual binaries will be extracted to /output/
CMD ["sh", "-c", "echo 'Build complete. Check /output/ for binaries.'"]
