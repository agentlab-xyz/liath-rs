# Quick Start

This guide walks you through the essential Liath operations.

## Basic Setup

```rust
use liath::{EmbeddedLiath, Config};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create configuration
    let config = Config {
        data_dir: PathBuf::from("./my_data"),
        ..Default::default()
    };

    // Initialize database
    let db = EmbeddedLiath::new(config)?;

    Ok(())
}
```

## Key-Value Operations

Store and retrieve data using simple key-value operations:

```rust
// Store a value
db.put("my_namespace", b"user:1", b"Alice")?;

// Retrieve a value
if let Some(value) = db.get("my_namespace", b"user:1")? {
    println!("User: {}", String::from_utf8_lossy(&value));
}

// Delete a value
db.delete("my_namespace", b"user:1")?;
```

## Semantic Search

Store text with embeddings and perform semantic search:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = EmbeddedLiath::new(Config::default())?;
    let executor = db.query_executor();

    // Store documents with embeddings
    let code = r#"
        store_with_embedding("docs", "d1", "Rust is a systems programming language")
        store_with_embedding("docs", "d2", "Python is great for data science")
        store_with_embedding("docs", "d3", "JavaScript runs in the browser")

        -- Search for similar documents
        local results = semantic_search("docs", "systems programming", 2)
        return json.encode(results)
    "#;

    let result = executor.execute(code, "agent").await?;
    println!("{}", result);

    Ok(())
}
```

## Using the Agent API

The Agent API provides high-level abstractions for building AI agents:

```rust
use liath::{EmbeddedLiath, Config};
use liath::agent::{Agent, Role};
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Arc::new(EmbeddedLiath::new(Config::default())?);

    // Create an agent
    let agent = Agent::new("my-agent", db.clone());

    // Store memories
    let memory = agent.memory()?;
    memory.store("User prefers dark mode", &["preferences", "ui"])?;
    memory.store("User works with Rust", &["skills", "programming"])?;

    // Recall memories semantically
    let results = memory.recall("What programming language?", 3)?;
    for entry in results {
        println!("Memory: {} (distance: {:.3})", entry.content, entry.distance);
    }

    // Manage conversations
    let conv = agent.conversation(None)?;
    conv.add_message(Role::User, "Hello!")?;
    conv.add_message(Role::Assistant, "Hi! How can I help?")?;

    // Store tool state
    let state = agent.tool_state("calculator")?;
    state.set("last_result", &42.0)?;

    Ok(())
}
```

## Programmable Memory with Lua

Execute agent-generated Lua code for flexible queries:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = EmbeddedLiath::new(Config::default())?;
    let executor = db.query_executor();

    // Agent-generated retrieval logic
    let code = r#"
        -- Get relevant memories
        local results = semantic_search("memory", "user preferences", 10)

        -- Filter by recency and importance
        local filtered = filter(results, function(r)
            local meta = json.decode(get("meta", r.id) or '{}')
            return (meta.importance or 0) > 0.5
        end)

        -- Transform and return
        return json.encode(map(filtered, function(r)
            return { content = r.content, score = 1 - r.distance }
        end))
    "#;

    let result = executor.execute(code, "agent").await?;
    println!("{}", result);

    Ok(())
}
```

## Using the CLI

Liath includes a command-line interface:

```bash
# Start interactive TUI
liath tui

# Start CLI REPL
liath cli

# Execute Lua code directly
liath execute 'return 1 + 2 + 3'

# Manage namespaces
liath namespace list
liath namespace create my_namespace
```

## Next Steps

- [Programmable Memory](../concepts/programmable-memory.md) - Deep dive into the core concept
- [Agent API Reference](../api/agent-api.md) - Complete Agent API documentation
- [Lua Scripting Guide](../guides/lua-scripting.md) - Full Lua function reference
- [Examples](../examples/index.md) - More practical examples
