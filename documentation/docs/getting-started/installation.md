# Installation

## Requirements

- Rust 1.70 or later
- A C compiler (for native dependencies)

## Adding Liath to Your Project

Add Liath to your `Cargo.toml`:

```toml
[dependencies]
liath = "0.1"
```

### Feature Flags

Liath provides several optional features:

| Flag | Default | Description |
|------|---------|-------------|
| `embedding` | Yes | FastEmbed/ONNX for text embeddings |
| `vector` | Yes | USearch for vector similarity search |
| `tui` | Yes | Interactive terminal UI |
| `server` | No | HTTP API server (Axum) |
| `mcp` | No | MCP server for AI assistants |
| `python` | No | Python bindings (PyO3) |

#### Minimal Installation

For a minimal installation without embedding or vector features:

```toml
[dependencies]
liath = { version = "0.1", default-features = false }
```

#### Full Installation

To enable all features:

```toml
[dependencies]
liath = { version = "0.1", features = ["server", "mcp", "python"] }
```

## Building from Source

Clone the repository and build:

```bash
git clone https://github.com/skelf-research/liath-rs.git
cd liath-rs
cargo build --release
```

### Running Tests

```bash
cargo test
```

### Building with Specific Features

```bash
# Build with server support
cargo build --release --features server

# Build with MCP support
cargo build --release --features mcp

# Build minimal version
cargo build --release --no-default-features
```

## CLI Installation

To install the Liath CLI globally:

```bash
cargo install --path .
```

Then you can use:

```bash
# Interactive TUI
liath tui

# Command-line REPL
liath cli

# Start HTTP server
liath server

# Execute Lua code
liath execute 'return 1 + 1'
```

## Verifying Installation

Create a simple test program:

```rust
use liath::{EmbeddedLiath, Config};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::default();
    let db = EmbeddedLiath::new(config)?;

    db.put("test", b"hello", b"world")?;

    if let Some(value) = db.get("test", b"hello")? {
        println!("Success! Retrieved: {}", String::from_utf8_lossy(&value));
    }

    Ok(())
}
```

Run with:

```bash
cargo run
```

You should see: `Success! Retrieved: world`

## Next Steps

- [Quick Start Guide](quick-start.md) - Get started with basic operations
- [Programmable Memory](../concepts/programmable-memory.md) - Understand the core concept
- [Lua Scripting Guide](../guides/lua-scripting.md) - Learn the Lua API
