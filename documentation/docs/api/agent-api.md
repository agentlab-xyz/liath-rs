# Agent API

The Agent API provides high-level abstractions for building AI agents with persistent memory, conversations, and tool state.

## Agent

The central coordinator for an agent's state.

### Creating an Agent

```rust
use liath::agent::Agent;
use std::sync::Arc;

// Simple agent
let agent = Agent::new("my-agent", db.clone());

// Agent with description
let agent = Agent::new_with_description(
    "assistant-1",
    "A helpful AI assistant for coding tasks",
    db.clone()
);
```

### Properties

```rust
// Get agent ID
let id: &str = agent.id();

// Get description (if set)
let desc: Option<&str> = agent.description();
```

### Persistence

```rust
// Save agent metadata
agent.save()?;

// Check if agent exists
let exists = Agent::exists("my-agent", &db)?;

// List all registered agents
let agents = Agent::list_agents(&db)?;
for meta in agents {
    println!("Agent: {} (created: {})", meta.id, meta.created_at);
}

// Get agent metadata
if let Some(meta) = agent.metadata()? {
    println!("ID: {}", meta.id);
    println!("Description: {:?}", meta.description);
    println!("Created: {}", meta.created_at);
    println!("Updated: {}", meta.updated_at);
}
```

### Accessing Components

```rust
// Get memory manager
let memory = agent.memory()?;

// Get or create conversation
let conv = agent.conversation(None)?;  // New conversation
let conv = agent.conversation(Some("conv-123"))?;  // Specific ID

// Get tool state
let state = agent.tool_state("my-tool")?;
```

---

## Memory

Long-term semantic memory with vector search.

### Storing Memories

```rust
let memory = agent.memory()?;

// Store with tags
memory.store("User prefers dark mode", &["preferences", "ui"])?;

// Store without tags
memory.store("Important fact to remember", &[])?;

// Store with many tags for organization
memory.store(
    "Built a REST API using Axum framework",
    &["history", "projects", "rust", "web"]
)?;
```

### Semantic Recall

```rust
// Find relevant memories
let results = memory.recall("What UI preferences does the user have?", 5)?;

for entry in results {
    println!("Content: {}", entry.content);
    println!("Distance: {:.4}", entry.distance);
    println!("Tags: {:?}", entry.tags);
    println!("---");
}
```

### Tag-Based Recall

```rust
// Recall by single tag
let prefs = memory.recall_by_tags(&["preferences"], 10)?;

// Recall by multiple tags (AND logic)
let rust_projects = memory.recall_by_tags(&["projects", "rust"], 5)?;
```

### MemoryEntry Structure

```rust
pub struct MemoryEntry {
    pub id: String,
    pub content: String,
    pub tags: Vec<String>,
    pub distance: f32,
    pub created_at: u64,
}
```

---

## Conversation

Message history management with threading support.

### Creating Conversations

```rust
// New conversation (auto-generated ID)
let conv = agent.conversation(None)?;
println!("Conversation ID: {}", conv.id());

// Resume or create specific conversation
let conv = agent.conversation(Some("support-ticket-123"))?;
```

### Adding Messages

```rust
use liath::agent::Role;

// User message
conv.add_message(Role::User, "Hello! Can you help?")?;

// Assistant response
conv.add_message(Role::Assistant, "Of course! What do you need?")?;

// System context
conv.add_message(Role::System, "User is a beginner programmer")?;

// Tool result
conv.add_message(
    Role::Tool("code_executor".to_string()),
    "Output: 42"
)?;
```

### Retrieving Messages

```rust
// Get all messages
let messages = conv.messages()?;

// Get message count
let count = conv.message_count();

// Iterate messages
for msg in messages {
    let role = match &msg.role {
        Role::User => "User",
        Role::Assistant => "Assistant",
        Role::System => "System",
        Role::Tool(name) => name.as_str(),
    };
    println!("[{}] {}", role, msg.content);
}
```

### Clearing Conversation

```rust
conv.clear()?;
```

### Message Structure

```rust
pub struct Message {
    pub role: Role,
    pub content: String,
    pub timestamp: u64,
}

pub enum Role {
    User,
    Assistant,
    System,
    Tool(String),
}
```

---

## ToolState

Persistent state for agent tools.

### Storing State

```rust
let state = agent.tool_state("calculator")?;

// Store primitives
state.set("last_result", &42.5f64)?;
state.set("count", &100u32)?;
state.set("enabled", &true)?;

// Store complex types
state.set("history", &vec!["1+1", "2*3"])?;

state.set("config", &serde_json::json!({
    "precision": 10,
    "mode": "scientific"
}))?;
```

### Retrieving State

```rust
// Get with type inference
let result: Option<f64> = state.get("last_result")?;
let count: Option<u32> = state.get("count")?;
let history: Option<Vec<String>> = state.get("history")?;

// Handle missing keys
match state.get::<f64>("unknown")? {
    Some(value) => println!("Found: {}", value),
    None => println!("Key not found"),
}
```

### Managing State

```rust
// Check if key exists
if state.exists("initialized")? {
    println!("Tool was previously initialized");
}

// Delete a key
state.delete("old_key")?;

// List all keys
let keys = state.keys()?;
for key in keys {
    println!("Key: {}", key);
}
```

---

## AgentMetadata

Metadata structure returned by `agent.metadata()`:

```rust
pub struct AgentMetadata {
    pub id: String,
    pub description: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
}
```

---

## Complete Example

```rust
use liath::{EmbeddedLiath, Config};
use liath::agent::{Agent, Role};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize
    let db = Arc::new(EmbeddedLiath::new(Config::default())?);

    // Create agent
    let agent = Agent::new_with_description(
        "assistant",
        "Personal AI assistant",
        db.clone()
    );
    agent.save()?;

    // Store memories
    let memory = agent.memory()?;
    memory.store("User's name is Alice", &["user", "profile"])?;
    memory.store("User prefers Rust", &["preferences", "programming"])?;

    // Create conversation
    let conv = agent.conversation(None)?;
    conv.add_message(Role::System, "You are a helpful assistant")?;
    conv.add_message(Role::User, "What programming language should I learn?")?;

    // Recall relevant context
    let context = memory.recall("programming language recommendation", 3)?;
    println!("Relevant context:");
    for entry in context {
        println!("  - {}", entry.content);
    }

    // Store tool state
    let state = agent.tool_state("recommendation_engine")?;
    state.set("last_query", &"programming language")?;
    state.set("recommendations_made", &1u32)?;

    Ok(())
}
```

---

## Next Steps

- [EmbeddedLiath API](embedded-liath.md) - Core database operations
- [Lua Standard Library](lua-stdlib.md) - Lua functions for agents
- [Building AI Agents Guide](../guides/building-agents.md) - Architecture patterns
