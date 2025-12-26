# Guide

This guide covers building, running interfaces, and using Liath's features.

## Build

```bash
# Default build (embedding + vector + tui)
cargo build

# With HTTP server
cargo build --features server

# With MCP server
cargo build --features mcp

# All features
cargo build --all-features

# Minimal build (no optional features)
cargo build --no-default-features
```

### Feature Flags

| Flag | Default | Description |
|------|---------|-------------|
| `embedding` | on | FastEmbed/ONNX Runtime for text embeddings |
| `vector` | on | USearch for vector similarity search |
| `tui` | on | Ratatui terminal interface |
| `server` | off | Axum HTTP API server |
| `mcp` | off | MCP server for AI assistants |

## TUI Console (Default)

Start the interactive TUI:
```bash
cargo run --bin liath
```

### Controls

| Key | Mode | Action |
|-----|------|--------|
| `i` | Normal | Enter insert mode |
| `Esc` | Insert | Return to normal mode |
| `Enter` | Insert | Execute query |
| `Up/Down` | Insert | Navigate command history |
| `PageUp/PageDown` | Normal | Navigate result pages |
| `q` | Normal | Quit |

### Features

- **Command History**: Persisted across sessions in `.liath_history`
- **Result Pagination**: Large results are paginated automatically
- **Vim-style Modes**: Normal mode for navigation, Insert mode for input

## Simple CLI

For a basic readline interface:
```bash
cargo run --bin liath -- cli --simple
```

### Helper Commands

- `:ns list` - List all namespaces
- `:ns create <name> <dims> <cosine|euclidean> <f32|f16>` - Create namespace
- `:put <namespace> <key> <value...>` - Store a value
- `:get <namespace> <key>` - Retrieve a value
- `:del <namespace> <key>` - Delete a key

Any other input is executed as Lua.

## HTTP Server

Enable the `server` feature:
```bash
cargo run --features server --bin liath -- server --port 3000
```

### Endpoints

#### Health Check
```bash
curl localhost:3000/health
```

#### Namespaces
```bash
# List namespaces
curl localhost:3000/namespaces

# Create namespace
curl -X POST localhost:3000/namespaces \
  -H 'content-type: application/json' \
  -d '{"name":"docs","dimensions":384,"metric":"cosine"}'

# Delete namespace
curl -X DELETE localhost:3000/namespaces/docs
```

#### Key-Value Operations
```bash
# Store value
curl -X PUT localhost:3000/kv/docs/mykey \
  -H 'content-type: application/json' \
  -d '{"value":"Hello World"}'

# Get value
curl localhost:3000/kv/docs/mykey

# Delete key
curl -X DELETE localhost:3000/kv/docs/mykey
```

#### Vector Search
```bash
# Search by vector
curl -X POST localhost:3000/search/docs \
  -H 'content-type: application/json' \
  -d '{"vector":[0.1,0.2,...],"limit":10}'

# Semantic search (text query)
curl -X POST localhost:3000/semantic/docs \
  -H 'content-type: application/json' \
  -d '{"query":"programming languages","limit":10}'
```

#### Embeddings
```bash
curl -X POST localhost:3000/embed \
  -H 'content-type: application/json' \
  -d '{"text":"Hello world"}'
```

#### Execute Lua
```bash
curl -X POST localhost:3000/execute \
  -H 'content-type: application/json' \
  -d '{"query":"return 1+1","user_id":"admin"}'
```

## MCP Server

For AI assistant integration (Claude, etc.):
```bash
cargo run --features mcp --bin liath -- mcp
```

### Available Tools

- `liath_put` - Store a key-value pair
- `liath_get` - Retrieve a value
- `liath_delete` - Delete a key
- `liath_list_namespaces` - List all namespaces
- `liath_create_namespace` - Create a namespace
- `liath_semantic_search` - Semantic similarity search
- `liath_execute` - Execute Lua code

## Lua Scripting

Liath provides a Lua interface for data operations:

```lua
-- Namespace operations
create_namespace("docs", 384, "cosine", "f32")
list_namespaces()
delete_namespace("docs")

-- Key-value operations
put("docs", "key1", "value1")
get("docs", "key1")
delete("docs", "key1")

-- Vector operations (if enabled)
store_with_embedding("docs", "id1", "Some text content")
semantic_search("docs", "query text", 10)

-- Embedding (if enabled)
embed("text to embed")
```

## Namespace Management CLI

```bash
# List namespaces
cargo run --bin liath -- namespace list

# Create namespace
cargo run --bin liath -- namespace create myns --dimensions 384 --metric cosine

# Delete namespace (with confirmation)
cargo run --bin liath -- namespace delete myns

# Delete namespace (no confirmation)
cargo run --bin liath -- namespace delete myns --force
```

## Execute Scripts

Run Lua code directly:
```bash
# Inline code
cargo run --bin liath -- execute "return 1 + 1"

# From file
cargo run --bin liath -- execute --file script.lua
```

## Library Usage

See `examples/` directory for comprehensive examples:
- `embedded.rs` - Basic embedded usage
- `vector_search.rs` - Vector similarity search
- `agent_usage.rs` - Agent memory and conversations
- `lua_scripting.rs` - Lua query interface

Run examples:
```bash
cargo run --example embedded
cargo run --example vector_search
cargo run --example agent_usage
cargo run --example lua_scripting
```
