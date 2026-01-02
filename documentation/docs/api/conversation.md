# Conversation API

The Conversation API manages message history for agent interactions.

## Overview

```rust
use liath::agent::{Agent, Conversation, Role, Message};

let agent = Agent::new("my-agent", db.clone());
let conv = agent.conversation(None)?;  // New conversation
```

## Creating Conversations

### New Conversation

```rust
// Auto-generated ID
let conv = agent.conversation(None)?;
println!("Conversation ID: {}", conv.id());
```

### Specific Conversation

```rust
// Create or load by ID
let conv = agent.conversation(Some("support-ticket-123"))?;
```

## Methods

### add_message

Add a message to the conversation.

```rust
fn add_message(&self, role: Role, content: &str) -> Result<MessageId, Error>
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `role` | `Role` | Message sender role |
| `content` | `&str` | Message content |

**Returns:** `MessageId` (u64) - Unique message identifier

**Example:**

```rust
// User message
conv.add_message(Role::User, "Hello, can you help me?")?;

// Assistant response
conv.add_message(Role::Assistant, "Of course! What do you need?")?;

// System context
conv.add_message(Role::System, "Be concise and helpful")?;

// Tool result
conv.add_message(Role::Tool("calculator".into()), "Result: 42")?;
```

---

### messages

Get all messages in the conversation.

```rust
fn messages(&self) -> Result<Vec<Message>, Error>
```

**Returns:** `Vec<Message>` - All messages in chronological order

**Example:**

```rust
let messages = conv.messages()?;

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

---

### last_n

Get the last N messages.

```rust
fn last_n(&self, n: usize) -> Result<Vec<Message>, Error>
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `n` | `usize` | Number of messages to retrieve |

**Returns:** `Vec<Message>` - Last N messages

**Example:**

```rust
// Get recent context for LLM
let recent = conv.last_n(10)?;
```

---

### search

Search messages semantically.

```rust
fn search(&self, query: &str, k: usize) -> Result<Vec<Message>, Error>
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `query` | `&str` | Search query |
| `k` | `usize` | Maximum results |

**Returns:** `Vec<Message>` - Messages sorted by relevance

**Example:**

```rust
// Find messages about a topic
let about_rust = conv.search("Rust programming", 5)?;

for msg in about_rust {
    println!("[{}] {}", msg.role, msg.content);
}
```

---

### clear

Clear all messages from the conversation.

```rust
fn clear(&self) -> Result<(), Error>
```

**Example:**

```rust
// Start fresh
conv.clear()?;
conv.add_message(Role::System, "New conversation started")?;
```

---

### id

Get the conversation ID.

```rust
fn id(&self) -> &str
```

**Example:**

```rust
println!("Conversation: {}", conv.id());
```

---

### agent_id

Get the parent agent ID.

```rust
fn agent_id(&self) -> &str
```

---

### message_count

Get the total number of messages.

```rust
fn message_count(&self) -> u64
```

**Example:**

```rust
println!("Messages in conversation: {}", conv.message_count());
```

## Types

### Role

Message sender role:

```rust
pub enum Role {
    /// Human user
    User,

    /// AI assistant
    Assistant,

    /// System instructions
    System,

    /// Tool execution result
    Tool(String),
}
```

### Message

Individual message in a conversation:

```rust
pub struct Message {
    /// Unique identifier
    pub id: MessageId,

    /// Sender role
    pub role: Role,

    /// Message content
    pub content: String,

    /// Unix timestamp
    pub timestamp: u64,
}
```

### MessageId

Type alias:

```rust
pub type MessageId = u64;
```

### ConversationId

Type alias:

```rust
pub type ConversationId = String;
```

## Storage Details

### Namespace Structure

```
agent:{agent_id}:conv:{conv_id}:messages  -- Message list
agent:{agent_id}:conv:{conv_id}:meta      -- Metadata
agent:{agent_id}:conv:{conv_id}:vectors   -- Message embeddings
```

### Message Format

```json
{
    "id": 1,
    "role": "user",
    "content": "Hello!",
    "timestamp": 1699876543
}
```

## Usage Patterns

### Standard Conversation Flow

```rust
// Initialize with system prompt
conv.add_message(Role::System, r#"
You are a helpful coding assistant.
- Be concise
- Provide code examples
- Ask clarifying questions when needed
"#)?;

// User interaction
conv.add_message(Role::User, "How do I read a file in Rust?")?;

// Assistant response
conv.add_message(Role::Assistant, r#"
Use `std::fs::read_to_string`:

```rust
let content = std::fs::read_to_string("file.txt")?;
```
"#)?;

// Follow-up
conv.add_message(Role::User, "What about async?")?;
```

### Tool Usage

```rust
conv.add_message(Role::User, "Calculate 15 * 7")?;

// Record tool call
conv.add_message(
    Role::Tool("calculator".into()),
    r#"{"expression": "15 * 7", "result": 105}"#
)?;

// Assistant incorporates result
conv.add_message(Role::Assistant, "15 * 7 = 105")?;
```

### Context Window Management

```rust
fn get_context(conv: &Conversation, max_tokens: usize) -> Result<Vec<Message>, Error> {
    let mut messages = Vec::new();
    let mut total_chars = 0;
    let char_per_token = 4;  // Rough estimate

    // Always include system messages
    let all = conv.messages()?;
    for msg in &all {
        if matches!(msg.role, Role::System) {
            total_chars += msg.content.len();
            messages.push(msg.clone());
        }
    }

    // Add recent messages until limit
    for msg in all.iter().rev() {
        if !matches!(msg.role, Role::System) {
            if total_chars + msg.content.len() > max_tokens * char_per_token {
                break;
            }
            total_chars += msg.content.len();
            messages.insert(1, msg.clone());  // After system
        }
    }

    Ok(messages)
}
```

### Multiple Conversations

```rust
// Main conversation
let main_conv = agent.conversation(Some("main"))?;

// Topic-specific threads
let rust_conv = agent.conversation(Some("topic:rust"))?;
let python_conv = agent.conversation(Some("topic:python"))?;

// Support tickets
let ticket_conv = agent.conversation(Some("ticket:12345"))?;
```

## Error Handling

```rust
use liath::LiathError;

match conv.messages() {
    Ok(msgs) => {
        println!("Retrieved {} messages", msgs.len());
    }
    Err(LiathError::ConversationNotFound(id)) => {
        println!("Conversation '{}' not found", id);
    }
    Err(e) => {
        println!("Error: {}", e);
    }
}
```

## See Also

- [Agent API](agent-api.md) - Parent agent interface
- [Conversation Management Guide](../guides/conversations.md) - Best practices
- [Lua Stdlib](lua-stdlib.md) - Lua conversation functions
