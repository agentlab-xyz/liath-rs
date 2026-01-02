# Memory API

The Memory API provides long-term semantic memory storage and retrieval for agents.

## Overview

```rust
use liath::agent::{Agent, Memory, MemoryEntry};

let agent = Agent::new("my-agent", db.clone());
let memory = agent.memory()?;
```

## Creating Memory

Memory is accessed through an Agent instance:

```rust
let memory = agent.memory()?;
```

## Methods

### store

Store a memory with optional tags.

```rust
fn store(&self, content: &str, tags: &[&str]) -> Result<MemoryId, Error>
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `content` | `&str` | The memory content to store |
| `tags` | `&[&str]` | Optional tags for categorization |

**Returns:** `MemoryId` (u64) - Unique identifier for the stored memory

**Example:**

```rust
// Store with tags
let id = memory.store(
    "User prefers dark mode in applications",
    &["preferences", "ui", "settings"]
)?;

// Store without tags
let id = memory.store("Important fact to remember", &[])?;

// Store multiple related memories
memory.store("User's name is Alice", &["user", "profile"])?;
memory.store("User works at TechCorp", &["user", "work"])?;
memory.store("User prefers Rust", &["user", "programming"])?;
```

---

### recall

Recall memories using semantic search.

```rust
fn recall(&self, query: &str, k: usize) -> Result<Vec<MemoryEntry>, Error>
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `query` | `&str` | Search query text |
| `k` | `usize` | Maximum number of results |

**Returns:** `Vec<MemoryEntry>` - Memories sorted by relevance (closest first)

**Example:**

```rust
let results = memory.recall("What programming language does the user prefer?", 5)?;

for entry in results {
    println!("Memory: {}", entry.content);
    println!("Distance: {:.4}", entry.distance);
    println!("Tags: {:?}", entry.tags);
    println!("---");
}
```

---

### recall_by_tags

Recall memories filtered by tags.

```rust
fn recall_by_tags(&self, tags: &[&str], limit: usize) -> Result<Vec<MemoryEntry>, Error>
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `tags` | `&[&str]` | Tags to filter by (AND logic) |
| `limit` | `usize` | Maximum number of results |

**Returns:** `Vec<MemoryEntry>` - Memories matching all specified tags

**Example:**

```rust
// Get all preferences
let prefs = memory.recall_by_tags(&["preferences"], 10)?;

// Get programming-related user info
let prog = memory.recall_by_tags(&["user", "programming"], 5)?;

// Combine with semantic search
let prefs = memory.recall_by_tags(&["preferences"], 20)?;
let relevant: Vec<_> = prefs.into_iter()
    .filter(|e| e.content.contains("dark"))
    .collect();
```

---

### delete

Delete a memory by ID.

```rust
fn delete(&self, id: MemoryId) -> Result<(), Error>
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `id` | `MemoryId` | ID of the memory to delete |

**Example:**

```rust
let id = memory.store("Temporary note", &["temp"])?;

// Later...
memory.delete(id)?;
```

---

### update

Update an existing memory's content.

```rust
fn update(&self, id: MemoryId, content: &str) -> Result<(), Error>
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `id` | `MemoryId` | ID of the memory to update |
| `content` | `&str` | New content |

**Example:**

```rust
let id = memory.store("User's favorite color is blue", &["preferences"])?;

// Correct the information
memory.update(id, "User's favorite color is green")?;
```

## Types

### MemoryEntry

Returned by recall operations:

```rust
pub struct MemoryEntry {
    /// Unique identifier
    pub id: MemoryId,

    /// Memory content
    pub content: String,

    /// Associated tags
    pub tags: Vec<String>,

    /// Semantic distance from query (lower = more relevant)
    pub distance: f32,

    /// Unix timestamp of creation
    pub created_at: u64,
}
```

### MemoryId

Type alias for memory identifiers:

```rust
pub type MemoryId = u64;
```

## Storage Details

### Namespace Structure

Memories are stored in agent-specific namespaces:

```
agent:{agent_id}:memory          -- Content + embeddings
agent:{agent_id}:memory:meta     -- Metadata (tags, timestamps)
agent:{agent_id}:memory:vectors  -- Vector index
```

### Data Format

```rust
// Content storage
key: "{memory_id}"
value: "{content}"

// Metadata storage
key: "{memory_id}"
value: {
    "tags": ["tag1", "tag2"],
    "created_at": 1699876543
}
```

## Usage Patterns

### Categorized Storage

```rust
// User profile information
memory.store("Name: Alice", &["profile", "identity"])?;
memory.store("Role: Senior Engineer", &["profile", "work"])?;
memory.store("Location: San Francisco", &["profile", "location"])?;

// Preferences
memory.store("Prefers dark mode", &["preferences", "ui"])?;
memory.store("Uses Vim keybindings", &["preferences", "editor"])?;

// Learned facts
memory.store("Project deadline is March 15", &["facts", "project"])?;
memory.store("Team uses Rust and Python", &["facts", "tech-stack"])?;
```

### Smart Retrieval

```rust
// Combine semantic and tag-based recall
fn smart_recall(
    memory: &Memory,
    query: &str,
    required_tags: &[&str],
    limit: usize,
) -> Result<Vec<MemoryEntry>, Error> {
    if required_tags.is_empty() {
        // Pure semantic search
        memory.recall(query, limit)
    } else {
        // Tag filter then semantic sort
        let tagged = memory.recall_by_tags(required_tags, limit * 2)?;

        // Re-rank by query relevance (would need embeddings)
        Ok(tagged.into_iter().take(limit).collect())
    }
}
```

### Memory Maintenance

```rust
// Archive old memories
fn archive_old_memories(memory: &Memory, max_age_days: u64) -> Result<(), Error> {
    let cutoff = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs() - (max_age_days * 24 * 3600);

    let old = memory.recall_by_tags(&["temporary"], 100)?
        .into_iter()
        .filter(|e| e.created_at < cutoff)
        .collect::<Vec<_>>();

    for entry in old {
        memory.delete(entry.id)?;
    }

    Ok(())
}
```

## Error Handling

```rust
use liath::LiathError;

match memory.recall("query", 10) {
    Ok(results) => {
        println!("Found {} memories", results.len());
    }
    Err(LiathError::NamespaceNotFound(ns)) => {
        println!("Memory namespace '{}' not initialized", ns);
    }
    Err(LiathError::Embedding(e)) => {
        println!("Embedding generation failed: {}", e);
    }
    Err(e) => {
        println!("Error: {}", e);
    }
}
```

## See Also

- [Agent API](agent-api.md) - Parent agent interface
- [Memory Patterns Guide](../guides/memory-patterns.md) - Best practices
- [Lua Stdlib](lua-stdlib.md) - Lua memory functions
