# Conversation Management

This guide covers patterns for managing multi-turn conversations in Liath, including message handling, context management, and conversation organization.

## Conversation Basics

### Creating Conversations

```rust
use liath::agent::{Agent, Role};

let agent = Agent::new("my-agent", db.clone());

// Create new conversation with auto-generated ID
let conv = agent.conversation(None)?;
println!("Conversation ID: {}", conv.id());

// Create or load specific conversation
let conv = agent.conversation(Some("support-ticket-123"))?;
```

### Adding Messages

```rust
// User messages
conv.add_message(Role::User, "Hello, I need help with Rust")?;

// Assistant responses
conv.add_message(Role::Assistant, "Hi! I'd be happy to help. What specific aspect of Rust are you working with?")?;

// System context
conv.add_message(Role::System, "The user is a beginner programmer")?;

// Tool results
conv.add_message(Role::Tool("code_runner".to_string()), "Execution output: 42")?;
```

### Retrieving Messages

```rust
// Get all messages
let messages = conv.messages()?;

// Get last N messages
let recent = conv.last_n(10)?;

// Get message count
let count = conv.message_count();

// Search within conversation
let search_results = conv.search("error handling", 5)?;
```

## Message Roles

### Role Types

| Role | Description | Use Case |
|------|-------------|----------|
| `User` | Human user input | Questions, commands, responses |
| `Assistant` | AI responses | Answers, explanations |
| `System` | System instructions | Context, rules, persona |
| `Tool(name)` | Tool execution results | Code output, API responses |

### Role Usage

```rust
use liath::agent::Role;

// Standard conversation flow
conv.add_message(Role::System, "You are a helpful coding assistant.")?;
conv.add_message(Role::User, "How do I read a file?")?;
conv.add_message(Role::Assistant, "Use std::fs::read_to_string...")?;

// With tool usage
conv.add_message(Role::User, "Run this code: println!(\"hello\")")?;
conv.add_message(Role::Tool("rust_executor".to_string()), "Output: hello")?;
conv.add_message(Role::Assistant, "The code executed successfully!")?;
```

## Conversation Patterns

### Context Window Management

Keep conversations within token limits:

```lua
local function get_context_window(conv_id, max_messages, max_chars)
    local messages = get_messages(conv_id, max_messages * 2)

    local selected = {}
    local total_chars = 0

    -- Always include system messages
    for _, msg in ipairs(messages) do
        if msg.role == "system" then
            table.insert(selected, msg)
            total_chars = total_chars + #msg.content
        end
    end

    -- Add recent messages until limit
    for i = #messages, 1, -1 do
        local msg = messages[i]
        if msg.role ~= "system" then
            if total_chars + #msg.content > max_chars then
                break
            end
            table.insert(selected, 2, msg)  -- Insert after system
            total_chars = total_chars + #msg.content
        end
    end

    return selected
end
```

### Conversation Threading

Handle multiple conversation threads:

```rust
pub struct ThreadedConversation {
    agent: Agent,
    main_thread: String,
    sub_threads: HashMap<String, String>,
}

impl ThreadedConversation {
    pub fn new(agent: Agent) -> Result<Self, Error> {
        let main = agent.conversation(Some("main"))?;
        Ok(Self {
            agent,
            main_thread: main.id().to_string(),
            sub_threads: HashMap::new(),
        })
    }

    pub fn create_thread(&mut self, topic: &str) -> Result<String, Error> {
        let thread_id = format!("thread:{}", topic);
        let conv = self.agent.conversation(Some(&thread_id))?;

        // Link to main thread
        conv.add_message(Role::System, &format!("Sub-thread for: {}", topic))?;

        self.sub_threads.insert(topic.to_string(), thread_id.clone());
        Ok(thread_id)
    }

    pub fn add_to_thread(&self, topic: &str, role: Role, content: &str) -> Result<(), Error> {
        let thread_id = self.sub_threads.get(topic)
            .ok_or("Thread not found")?;
        let conv = self.agent.conversation(Some(thread_id))?;
        conv.add_message(role, content)?;
        Ok(())
    }
}
```

### Conversation Summarization

Summarize long conversations:

```lua
local function summarize_conversation(conv_id)
    local messages = get_messages(conv_id, 100)

    -- Group by topic (simplified)
    local summary_parts = {}

    -- Extract key points (in practice, use LLM)
    for _, msg in ipairs(messages) do
        if msg.role == "user" then
            table.insert(summary_parts, "User asked: " .. msg.content:sub(1, 100))
        end
    end

    return table.concat(summary_parts, "\n")
end

-- Store summary as memory
local function archive_conversation(conv_id)
    local summary = summarize_conversation(conv_id)

    store_memory("memory",
        "Conversation summary: " .. summary,
        {"conversation", "archived", conv_id}
    )

    -- Optionally clear original
    -- clear_conversation(conv_id)
end
```

### Memory-Augmented Conversations

Inject relevant memories into conversation context:

```lua
local function process_with_memory(conv_id, user_message)
    -- Store user message
    add_message(conv_id, "user", user_message)

    -- Get relevant memories
    local memories = recall("memory", user_message, 5)
    local memory_context = map(memories, function(m)
        return "- " .. m.content
    end)

    -- Get conversation history
    local history = get_messages(conv_id, 10)

    -- Build context for LLM
    return json.encode({
        system = "You are a helpful assistant with access to the following memories:\n" ..
                 table.concat(memory_context, "\n"),
        messages = history,
        user_message = user_message
    })
end
```

## Conversation Storage

### Structure

Conversations are stored with the following schema:

```
agent:{agent_id}:conv:{conv_id}:messages    -- Message list
agent:{agent_id}:conv:{conv_id}:meta        -- Metadata
agent:{agent_id}:conv:{conv_id}:vectors     -- Message embeddings
```

### Metadata

```lua
-- Store conversation metadata
local function set_conv_metadata(conv_id, metadata)
    put("conv:meta", conv_id, json.encode({
        created_at = metadata.created_at or now(),
        updated_at = now(),
        title = metadata.title,
        tags = metadata.tags,
        participants = metadata.participants,
        status = metadata.status  -- "active", "archived", "resolved"
    }))
end

-- Get metadata
local function get_conv_metadata(conv_id)
    local meta = get("conv:meta", conv_id)
    return meta and json.decode(meta) or {}
end
```

## Conversation Search

### Search Within Conversation

```rust
let conv = agent.conversation(Some("conv-123"))?;

// Semantic search in conversation
let results = conv.search("error handling", 5)?;

for msg in results {
    println!("[{}] {}", msg.role, msg.content);
}
```

### Search Across Conversations

```lua
local function search_all_conversations(query, limit)
    -- Get all conversation IDs
    local conv_keys = keys("conv:meta")
    local all_results = {}

    for _, conv_id in ipairs(conv_keys) do
        local messages = get_messages(conv_id, 50)

        for _, msg in ipairs(messages) do
            -- Simple keyword search
            if string.find(msg.content:lower(), query:lower()) then
                table.insert(all_results, {
                    conv_id = conv_id,
                    message = msg
                })
            end
        end
    end

    return slice(all_results, 1, limit)
end
```

## Best Practices

### 1. Include System Context

Always set the agent's persona and instructions:

```rust
conv.add_message(Role::System, r#"
You are a helpful coding assistant.
- Provide concise explanations
- Include code examples when relevant
- Ask clarifying questions if needed
"#)?;
```

### 2. Preserve Context Efficiently

```lua
-- Summarize long exchanges
local function compact_history(conv_id)
    local messages = get_messages(conv_id, 100)

    if #messages > 50 then
        -- Summarize older messages
        local old_messages = slice(messages, 1, #messages - 20)
        local summary = "Previous discussion covered: " ..
            extract_topics(old_messages)

        -- Replace with summary
        clear_conversation(conv_id)
        add_message(conv_id, "system", summary)

        -- Re-add recent messages
        local recent = slice(messages, #messages - 19, #messages)
        for _, msg in ipairs(recent) do
            add_message(conv_id, msg.role, msg.content)
        end
    end
end
```

### 3. Handle Long Responses

```rust
// Split long assistant responses
fn add_long_response(conv: &Conversation, response: &str) -> Result<(), Error> {
    const MAX_CHUNK: usize = 4000;

    if response.len() <= MAX_CHUNK {
        conv.add_message(Role::Assistant, response)?;
    } else {
        let chunks: Vec<&str> = response
            .chars()
            .collect::<Vec<_>>()
            .chunks(MAX_CHUNK)
            .map(|c| c.iter().collect::<String>())
            .collect();

        for (i, chunk) in chunks.iter().enumerate() {
            let prefix = if i > 0 { "(continued) " } else { "" };
            conv.add_message(Role::Assistant, &format!("{}{}", prefix, chunk))?;
        }
    }
    Ok(())
}
```

### 4. Track Conversation State

```lua
local STATES = {
    ACTIVE = "active",
    WAITING_USER = "waiting_user",
    WAITING_TOOL = "waiting_tool",
    RESOLVED = "resolved",
    ARCHIVED = "archived"
}

local function set_state(conv_id, state)
    local meta = get_conv_metadata(conv_id)
    meta.status = state
    meta.updated_at = now()
    set_conv_metadata(conv_id, meta)
end
```

### 5. Clean Up Old Conversations

```lua
local function cleanup_old_conversations(max_age_days)
    local conv_keys = keys("conv:meta")
    local cutoff = now() - (max_age_days * 24 * 3600)

    for _, conv_id in ipairs(conv_keys) do
        local meta = get_conv_metadata(conv_id)

        if meta.updated_at and meta.updated_at < cutoff then
            -- Archive first
            archive_conversation(conv_id)

            -- Then delete
            clear_conversation(conv_id)
            delete("conv:meta", conv_id)
        end
    end
end
```

## Next Steps

- [Memory Patterns](memory-patterns.md) - Link memories to conversations
- [Building AI Agents](building-agents.md) - Full agent implementation
- [Tool State](tool-state.md) - Stateful tools in conversations
