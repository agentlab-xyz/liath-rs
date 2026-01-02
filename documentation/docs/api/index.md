# API Reference

Complete API documentation for Liath.

## Core APIs

### EmbeddedLiath

The main entry point for using Liath as an embedded database.

[:octicons-arrow-right-24: EmbeddedLiath API](embedded-liath.md)

### QueryExecutor

Execute Lua queries against the database.

[:octicons-arrow-right-24: QueryExecutor API](query-executor.md)

## Agent APIs

### Agent

High-level agent management interface.

[:octicons-arrow-right-24: Agent API](agent-api.md)

### Memory

Long-term semantic memory for agents.

[:octicons-arrow-right-24: Memory API](memory.md)

### Conversation

Message history and conversation management.

[:octicons-arrow-right-24: Conversation API](conversation.md)

### ToolState

Persistent state for agent tools.

[:octicons-arrow-right-24: ToolState API](tool-state.md)

## Lua APIs

### Lua Standard Library

All functions available in the Lua runtime.

[:octicons-arrow-right-24: Lua Stdlib](lua-stdlib.md)

## Error Types

### LiathError

Error types and handling patterns.

[:octicons-arrow-right-24: Error Types](errors.md)

## Quick Reference

### Type Aliases

```rust
pub type AgentId = String;
pub type MemoryId = u64;
pub type ConversationId = String;
pub type MessageId = u64;
pub type LiathResult<T> = Result<T, LiathError>;
```

### Common Imports

```rust
use liath::{
    // Core
    EmbeddedLiath,
    Config,
    LiathError,
    LiathResult,

    // Components
    QueryExecutor,
    EmbeddingWrapper,
    AuthManager,

    // Agent
    agent::{Agent, Memory, Conversation, ToolState, Role, Message},
};

// Vector features
#[cfg(feature = "vector")]
use usearch::{MetricKind, ScalarKind};
```

### Feature Gates

```rust
// Vector operations
#[cfg(feature = "vector")]
db.create_namespace("docs", 384, MetricKind::Cos, ScalarKind::F32)?;

// Embedding operations
#[cfg(feature = "embedding")]
let vec = db.generate_embedding("text")?;

// Server
#[cfg(feature = "server")]
liath::server::run_server(executor, addr).await?;

// MCP
#[cfg(feature = "mcp")]
liath::mcp::run_mcp_server(executor, user).await?;
```
