# Configuration

This guide covers all configuration options available in Liath.

## Basic Configuration

### Config Struct

The primary configuration is done through the `Config` struct:

```rust
use liath::Config;
use std::path::PathBuf;

let config = Config {
    data_dir: PathBuf::from("./my_data"),
    luarocks_path: Some(PathBuf::from("/usr/local/bin/luarocks")),
};
```

### Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `data_dir` | `PathBuf` | `./liath_data` | Directory for persistent storage |
| `luarocks_path` | `Option<PathBuf>` | `None` | Path to LuaRocks for package management |

### Default Configuration

```rust
use liath::Config;

// Uses all defaults
let config = Config::default();

// Equivalent to:
let config = Config {
    data_dir: PathBuf::from("./liath_data"),
    luarocks_path: None,
};
```

## Data Directory Structure

When you initialize Liath, it creates the following directory structure:

```
data_dir/
├── namespaces/           # Namespace data
│   ├── default/          # Default namespace
│   │   ├── kv/           # Key-value store (Fjall)
│   │   └── vectors.idx   # Vector index (USearch)
│   └── custom_ns/        # Custom namespaces
├── agents/               # Agent metadata
├── files/                # File storage
├── auth/                 # Authentication data
└── lua_packages/         # Installed Lua packages
```

## Feature Flags

Liath uses Cargo feature flags to enable optional functionality:

### Available Features

| Feature | Default | Description |
|---------|---------|-------------|
| `embedding` | Yes | FastEmbed/ONNX for text embeddings |
| `vector` | Yes | USearch for vector similarity search |
| `tui` | Yes | Interactive terminal UI (Ratatui) |
| `server` | No | HTTP API server (Axum) |
| `mcp` | No | MCP server for AI assistants |
| `python` | No | Python bindings (PyO3) |

### Feature Configuration

In your `Cargo.toml`:

```toml
# Default features (embedding, vector, tui)
[dependencies]
liath = "0.1"

# Minimal installation
[dependencies]
liath = { version = "0.1", default-features = false }

# With HTTP server
[dependencies]
liath = { version = "0.1", features = ["server"] }

# Full installation
[dependencies]
liath = { version = "0.1", features = ["server", "mcp", "python"] }

# Specific combination
[dependencies]
liath = { version = "0.1", default-features = false, features = ["embedding", "vector"] }
```

## Namespace Configuration

### Creating Namespaces

Namespaces are created with specific vector index parameters:

```rust
use liath::EmbeddedLiath;
use usearch::{MetricKind, ScalarKind};

let db = EmbeddedLiath::new(Config::default())?;

// Create with cosine similarity (recommended for text)
db.create_namespace(
    "documents",
    384,              // Vector dimensions (must match embedding model)
    MetricKind::Cos,  // Distance metric
    ScalarKind::F32   // Scalar type
)?;

// Create with Euclidean distance
db.create_namespace(
    "images",
    512,
    MetricKind::L2sq,
    ScalarKind::F32
)?;
```

### Metric Types

| Metric | Use Case | Description |
|--------|----------|-------------|
| `MetricKind::Cos` | Text embeddings | Cosine similarity (direction) |
| `MetricKind::L2sq` | Image features | Euclidean distance (magnitude) |
| `MetricKind::IP` | Normalized vectors | Inner product |

### Scalar Types

| Type | Memory | Precision | Use Case |
|------|--------|-----------|----------|
| `ScalarKind::F32` | 4 bytes | High | Default, best accuracy |
| `ScalarKind::F16` | 2 bytes | Medium | Memory constrained |
| `ScalarKind::I8` | 1 byte | Low | Large-scale, approximate |

## Embedding Configuration

### Default Model

Liath uses the `BAAI/bge-small-en-v1.5` model by default:

- **Dimensions**: 384
- **Language**: English
- **Size**: ~130MB

### Custom Embedding Models

```rust
use liath::EmbeddingWrapper;
use fastembed::{EmbeddingModel, InitOptions};

// Use a different model
let embedding = EmbeddingWrapper::with_model(EmbeddingModel::BGEBaseENV15)?;

// Or with full options
let options = InitOptions {
    model_name: EmbeddingModel::AllMiniLML6V2,
    show_download_progress: true,
    ..Default::default()
};
let embedding = EmbeddingWrapper::with_options(options)?;
```

### Available Models

| Model | Dimensions | Size | Language |
|-------|------------|------|----------|
| `BGESmallENV15` | 384 | 130MB | English |
| `BGEBaseENV15` | 768 | 440MB | English |
| `AllMiniLML6V2` | 384 | 91MB | English |
| `ParaphraseMLMiniLML12V2` | 384 | 120MB | Multilingual |

!!! warning "Dimension Matching"
    The namespace dimension must match your embedding model's output dimension.
    Using mismatched dimensions will cause runtime errors.

## Authentication Configuration

### In-Memory Auth (Default)

```rust
use liath::AuthManager;

let mut auth = AuthManager::new();
auth.add_user("admin", vec![
    "select".into(),
    "insert".into(),
    "update".into(),
    "delete".into(),
    "create_namespace".into(),
]);
```

### Persistent Auth

```rust
use liath::AuthManager;
use std::path::Path;

let mut auth = AuthManager::with_persistence(Path::new("./auth_data"))?;
auth.add_user("api_user", vec!["select".into(), "insert".into()]);
auth.flush()?;  // Persist to disk
```

### Available Permissions

| Permission | Description |
|------------|-------------|
| `select` | Read from KV store |
| `insert` | Write new values |
| `update` | Modify existing values |
| `delete` | Remove values |
| `create_namespace` | Create namespaces |
| `delete_namespace` | Remove namespaces |
| `upload_file` | Upload files |
| `process_file` | Process files |
| `generate_embedding` | Generate embeddings |
| `similarity_search` | Vector search |
| `install_package` | Install Lua packages |
| `list_packages` | List packages |

## Query Executor Configuration

### Concurrency Control

The query executor uses a semaphore to limit concurrent embedding operations:

```rust
use liath::QueryExecutor;

// Default: 10 concurrent embedding operations
let executor = QueryExecutor::new(
    namespace_manager,
    embedding_wrapper,
    lua_vm,
    file_storage,
    auth_manager,
    10  // max_concurrent_embedding
);
```

## Server Configuration

### HTTP Server

When using the `server` feature:

```rust
use liath::server::run_server;

// Environment variables or command-line args
// LIATH_HOST=0.0.0.0
// LIATH_PORT=8080

run_server(query_executor, "0.0.0.0:8080").await?;
```

### MCP Server

When using the `mcp` feature:

```rust
use liath::mcp::run_mcp_server;

// MCP uses stdio for communication
run_mcp_server(query_executor, "default_user".to_string()).await?;
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `LIATH_DATA_DIR` | Data directory path | `./liath_data` |
| `LIATH_LOG_LEVEL` | Logging level | `info` |
| `LIATH_HOST` | Server bind address | `127.0.0.1` |
| `LIATH_PORT` | Server port | `8080` |

## Production Configuration

### Recommended Settings

```rust
use liath::{EmbeddedLiath, Config};
use std::path::PathBuf;

let config = Config {
    // Use absolute path in production
    data_dir: PathBuf::from("/var/lib/liath/data"),
    luarocks_path: Some(PathBuf::from("/usr/bin/luarocks")),
};

let db = EmbeddedLiath::new(config)?;

// Create namespaces with appropriate settings
#[cfg(feature = "vector")]
{
    use usearch::{MetricKind, ScalarKind};

    // High-precision for critical data
    db.create_namespace("critical", 384, MetricKind::Cos, ScalarKind::F32)?;

    // Memory-efficient for large datasets
    db.create_namespace("logs", 384, MetricKind::Cos, ScalarKind::F16)?;
}
```

### Performance Tuning

```rust
// For high-throughput scenarios
let executor = QueryExecutor::new(
    namespace_manager,
    embedding_wrapper,
    lua_vm,
    file_storage,
    auth_manager,
    20  // Increase concurrent embeddings
);

// Pre-allocate vector index capacity
#[cfg(feature = "vector")]
{
    let ns = db.get_namespace("documents")?;
    ns.vector_db.reserve(100_000)?;  // Reserve space for 100k vectors
}
```

## Next Steps

- [Quick Start](quick-start.md) - Basic usage examples
- [Architecture](../concepts/architecture.md) - System design
- [Performance Guide](../guides/performance.md) - Optimization tips
