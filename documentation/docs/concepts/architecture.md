# Architecture

Liath is designed as a modular, embedded database optimized for AI agent workloads. This document explains the key architectural components and how they work together.

## High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Application                              │
├─────────────────────────────────────────────────────────────────┤
│                       Agent API Layer                            │
│  ┌──────────┐  ┌──────────────┐  ┌───────────┐  ┌────────────┐ │
│  │  Agent   │  │   Memory     │  │   Conv    │  │ ToolState  │ │
│  └──────────┘  └──────────────┘  └───────────┘  └────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                      Query Executor                              │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                    Lua Runtime (Sandboxed)                │   │
│  └──────────────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────────────┤
│                       Core Services                              │
│  ┌────────────┐  ┌──────────────┐  ┌──────────────────────┐    │
│  │  Storage   │  │   Vector     │  │    Embeddings        │    │
│  │  (Fjall)   │  │   (USearch)  │  │    (FastEmbed)       │    │
│  └────────────┘  └──────────────┘  └──────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

## Core Components

### 1. EmbeddedLiath

The main entry point that coordinates all components:

```rust
pub struct EmbeddedLiath {
    namespace_manager: NamespaceManager,    // Namespace isolation
    embedding_wrapper: EmbeddingWrapper,    // Text embeddings
    query_executor: QueryExecutor,          // Lua execution
}
```

**Responsibilities:**

- Initialize and configure all subsystems
- Provide the public API for storage operations
- Coordinate between embedding, storage, and vector components

### 2. Storage Layer (Fjall)

A high-performance key-value store based on LSM trees:

```rust
pub struct FjallWrapper {
    keyspace: Keyspace,
    partitions: HashMap<String, PartitionHandle>,
}
```

**Features:**

- Persistent, crash-safe storage
- Atomic operations
- Partition-based namespace isolation
- Efficient range scans

### 3. Vector Index (USearch)

HNSW-based vector similarity search:

```rust
pub struct UsearchWrapper {
    index: Index,
    id_map: HashMap<u64, String>,
}
```

**Features:**

- Fast approximate nearest neighbor search
- Multiple distance metrics (Cosine, Euclidean, etc.)
- Configurable index parameters
- Persistent index storage

### 4. Embedding Engine (FastEmbed)

ONNX-based text embedding generation:

```rust
pub struct EmbeddingWrapper {
    model: TextEmbedding,
}
```

**Features:**

- CPU-optimized inference
- Batch embedding support
- Multiple model options
- No external API dependencies

### 5. Lua Runtime

Sandboxed Lua 5.4 execution environment:

```rust
pub struct LuaVM {
    lua: Lua,
    validator: LuaValidator,
}
```

**Features:**

- Safe, sandboxed execution
- Pre-registered database functions
- Input validation
- Async execution support

### 6. Query Executor

Orchestrates Lua execution with database access:

```rust
pub struct QueryExecutor {
    namespace_manager: Arc<NamespaceManager>,
    embedding_wrapper: Arc<EmbeddingWrapper>,
    lua_vm: LuaVM,
    semaphore: Semaphore,
}
```

**Features:**

- Registers Lua database functions
- Controls concurrency
- Manages execution context
- Handles async operations

## Agent API Layer

High-level abstractions for building AI agents:

### Agent

```rust
pub struct Agent {
    id: String,
    db: Arc<EmbeddedLiath>,
    description: Option<String>,
}
```

The central coordination point for an agent's state.

### Memory

```rust
pub struct Memory {
    agent_id: String,
    db: Arc<EmbeddedLiath>,
}
```

Long-term semantic memory with vector search capabilities.

### Conversation

```rust
pub struct Conversation {
    id: String,
    agent_id: String,
    db: Arc<EmbeddedLiath>,
}
```

Message history management with threading support.

### ToolState

```rust
pub struct ToolState {
    tool_name: String,
    agent_id: String,
    db: Arc<EmbeddedLiath>,
}
```

Persistent state for agent tools.

## Namespace Isolation

Namespaces provide logical data isolation:

```
┌─────────────────────────────────────────────────┐
│                    Namespace: "agent-1"          │
│  ┌───────────────┐  ┌───────────────────────┐   │
│  │   KV Store    │  │    Vector Index       │   │
│  │  (Partition)  │  │    (Optional)         │   │
│  └───────────────┘  └───────────────────────┘   │
├─────────────────────────────────────────────────┤
│                    Namespace: "agent-2"          │
│  ┌───────────────┐  ┌───────────────────────┐   │
│  │   KV Store    │  │    Vector Index       │   │
│  │  (Partition)  │  │    (Optional)         │   │
│  └───────────────┘  └───────────────────────┘   │
└─────────────────────────────────────────────────┘
```

Each namespace has:

- Isolated key-value partition
- Optional vector index with configurable dimensions
- Independent lifecycle management

## Data Flow

### Storing with Embeddings

```
1. User calls store_with_embedding("ns", "id", "text")
2. EmbeddingWrapper generates vector from text
3. FjallWrapper stores text in KV partition
4. UsearchWrapper adds vector to index with ID mapping
5. Transaction commits atomically
```

### Semantic Search

```
1. User calls semantic_search("ns", "query", k)
2. EmbeddingWrapper generates query vector
3. UsearchWrapper finds k nearest neighbors
4. FjallWrapper retrieves content for matched IDs
5. Results returned with content and distances
```

### Lua Execution

```
1. User submits Lua code via execute()
2. LuaValidator checks code safety
3. QueryExecutor acquires semaphore
4. Lua code runs with registered functions
5. Functions access DB through closures
6. Results returned as JSON string
```

## Optional Components

### HTTP Server (Axum)

REST API for external access:

- KV operations: `PUT/GET/DELETE /kv/:namespace/:key`
- Semantic search: `POST /search/:namespace`
- Lua execution: `POST /execute`

### MCP Server

Model Context Protocol for AI assistant integration:

- Tools exposed: `liath_put`, `liath_get`, `liath_semantic_search`, etc.
- Direct integration with Claude, ChatGPT, etc.

### Python Bindings (PyO3)

Python API for data science workflows:

```python
from liath import Liath

db = Liath()
db.put("ns", "key", "value")
results = db.semantic_search("ns", "query", k=5)
```

## Performance Considerations

### Storage

- LSM-tree provides excellent write throughput
- Bloom filters accelerate point lookups
- Compaction happens in background

### Vector Search

- HNSW provides O(log N) search complexity
- Index builds incrementally
- Memory-mapped for large indices

### Embeddings

- ONNX runtime with CPU optimizations
- Batch processing reduces overhead
- Model loaded once at startup

### Lua Execution

- Semaphore limits concurrent executions
- Compiled bytecode cached
- Minimal per-execution overhead

## Next Steps

- [EmbeddedLiath API](../api/embedded-liath.md) - Core API reference
- [Lua Standard Library](../api/lua-stdlib.md) - Available Lua functions
- [Building AI Agents](../guides/building-agents.md) - Agent development patterns
