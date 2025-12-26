# Liath

Liath is a fast, embeddable database designed for running AI agents. It combines:
- Key-value storage built on Fjall
- Vector similarity search (USearch)
- Text embeddings (FastEmbed / ONNX Runtime)
- Agent memory and conversation management
- A Lua scripting runtime
- TUI and CLI interfaces
- Optional HTTP API and MCP server

Liath emphasizes a simple, composable core you can embed in your own projects, with opt-in features for AI agent workflows.

## Features

- **Storage**: Namespaced key-value store using Fjall with persistence
- **Vector Search**: USearch index per namespace for similarity queries
- **Embeddings**: FastEmbed models for automatic text-to-vector conversion
- **Agent API**: Memory storage, conversation tracking, and tool state management
- **Scripting**: Lua runtime for custom queries and data manipulation
- **TUI**: Interactive terminal interface with history and pagination
- **HTTP API**: RESTful server for remote access (optional)
- **MCP Server**: Model Context Protocol for AI assistant integration (optional)

### Feature Flags

| Flag | Default | Description |
|------|---------|-------------|
| `embedding` | on | FastEmbed/ONNX Runtime for text embeddings |
| `vector` | on | USearch for vector similarity search |
| `tui` | on | Ratatui-based terminal interface |
| `server` | off | Axum HTTP API server |
| `mcp` | off | MCP server for AI assistants |

## Quick Start

### Prerequisites
- Rust (stable)
- System requirements listed in `docs/system-deps.md`

### Build from Source

```bash
git clone https://github.com/nudgelang/liath-rs.git
cd liath-rs
cargo build --release
```

### Run the TUI Console

```bash
cargo run --bin liath
```

### Run the Simple CLI

```bash
cargo run --bin liath -- cli --simple
```

### Start the HTTP Server

```bash
cargo run --features server --bin liath -- server --port 3000
```

### Start the MCP Server

```bash
cargo run --features mcp --bin liath -- mcp
```

## Library Usage

### Basic Key-Value Operations

```rust
use liath::{EmbeddedLiath, Config};

fn main() -> anyhow::Result<()> {
    let liath = EmbeddedLiath::new(Config::default())?;

    // Create a namespace
    liath.create_namespace("docs", 384, usearch::MetricKind::Cos, usearch::ScalarKind::F32)?;

    // Store and retrieve data
    liath.put("docs", b"hello", b"world")?;
    let value = liath.get("docs", b"hello")?;
    assert_eq!(value.as_deref(), Some(b"world".as_ref()));

    Ok(())
}
```

### Vector Search

```rust
use liath::{EmbeddedLiath, Config};

fn main() -> anyhow::Result<()> {
    let liath = EmbeddedLiath::new(Config::default())?;
    liath.create_namespace("vectors", 384, usearch::MetricKind::Cos, usearch::ScalarKind::F32)?;

    // Store text with automatic embedding
    liath.store_with_embedding("vectors", "doc1", "Rust is a systems programming language")?;
    liath.store_with_embedding("vectors", "doc2", "Python is great for data science")?;

    // Semantic search
    let results = liath.semantic_search("vectors", "programming languages", 5)?;
    for (id, content, distance) in results {
        println!("{}: {} (distance: {})", id, content, distance);
    }

    Ok(())
}
```

### Agent Memory

```rust
use liath::agent::{Agent, MemoryEntry};

fn main() -> anyhow::Result<()> {
    let mut agent = Agent::new("my-agent", "./data")?;

    // Store memories with tags
    agent.memory.store(MemoryEntry {
        content: "User prefers dark mode".to_string(),
        tags: vec!["preference".to_string(), "ui".to_string()],
        importance: 0.8,
        ..Default::default()
    })?;

    // Recall memories semantically
    let memories = agent.memory.recall("user interface preferences", 5)?;

    // Manage conversations
    agent.conversation.add_message("user", "Hello!")?;
    agent.conversation.add_message("assistant", "Hi there!")?;

    Ok(())
}
```

### Lua Scripting

```rust
use liath::{EmbeddedLiath, Config};

fn main() -> anyhow::Result<()> {
    let liath = EmbeddedLiath::new(Config::default())?;
    let executor = liath.query_executor();

    // Execute Lua queries
    let result = executor.execute(r#"
        put("myns", "key1", "value1")
        return get("myns", "key1")
    "#, "user1").await?;

    println!("Result: {}", result);
    Ok(())
}
```

## HTTP API Endpoints

When running with `--features server`:

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/health` | Health check |
| GET | `/namespaces` | List all namespaces |
| POST | `/namespaces` | Create a namespace |
| DELETE | `/namespaces/:name` | Delete a namespace |
| GET | `/kv/:ns/:key` | Get a value |
| PUT | `/kv/:ns/:key` | Store a value |
| DELETE | `/kv/:ns/:key` | Delete a value |
| POST | `/search/:ns` | Vector similarity search |
| POST | `/semantic/:ns` | Text-to-vector semantic search |
| POST | `/embed` | Generate embeddings |
| POST | `/execute` | Execute Lua code |

## MCP Server

When running with `--features mcp`, Liath provides MCP tools for AI assistants:

- `liath_put` - Store a key-value pair
- `liath_get` - Retrieve a value
- `liath_delete` - Delete a key
- `liath_list_namespaces` - List all namespaces
- `liath_create_namespace` - Create a new namespace
- `liath_semantic_search` - Semantic similarity search
- `liath_execute` - Execute Lua code
- Agent memory tools for storing and recalling memories

## TUI Controls

| Key | Action |
|-----|--------|
| `i` | Enter insert mode |
| `Esc` | Return to normal mode |
| `Enter` | Execute query |
| `Up/Down` | Navigate history |
| `PageUp/PageDown` | Navigate result pages |
| `q` | Quit |

## Documentation

- `docs/guide.md` - Quickstart and usage guide
- `docs/architecture.md` - Module overview and data flow
- `docs/system-deps.md` - Platform-specific dependencies
- `docs/status.md` - Current status and roadmap

## Examples

See the `examples/` directory:
- `embedded.rs` - Basic embedded usage
- `vector_search.rs` - Vector similarity search
- `agent_usage.rs` - Agent memory and conversations
- `lua_scripting.rs` - Lua query interface

Run examples with:
```bash
cargo run --example embedded
cargo run --example vector_search
cargo run --example agent_usage
cargo run --example lua_scripting
```

## License

MIT
