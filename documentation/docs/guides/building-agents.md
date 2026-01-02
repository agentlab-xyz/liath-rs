# Building AI Agents

This guide covers patterns and best practices for building AI agents with Liath.

## Agent Basics

### Creating an Agent

```rust
use liath::{EmbeddedLiath, Config};
use liath::agent::{Agent, Role};
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Arc::new(EmbeddedLiath::new(Config::default())?);

    // Simple agent
    let agent = Agent::new("my-agent", db.clone());

    // Agent with description
    let agent = Agent::new_with_description(
        "assistant-1",
        "A helpful AI assistant for coding tasks",
        db.clone()
    );

    Ok(())
}
```

### Agent Lifecycle

```rust
// Create and register
let agent = Agent::new("agent-1", db.clone());

// Save metadata
agent.save()?;

// Check if exists
let exists = Agent::exists("agent-1", &db)?;

// List all agents
let agents = Agent::list_agents(&db)?;

// Load metadata
if let Some(meta) = agent.metadata()? {
    println!("Created at: {}", meta.created_at);
}
```

## Working with Memory

### Storing Memories

```rust
let memory = agent.memory()?;

// Store with tags for organization
memory.store(
    "User prefers functional programming style",
    &["preferences", "coding"]
)?;

memory.store(
    "User works at a startup in fintech",
    &["background", "work"]
)?;

memory.store(
    "Previous conversation about async/await",
    &["history", "topics"]
)?;
```

### Semantic Recall

```rust
// Find relevant memories
let results = memory.recall("What programming style does the user prefer?", 3)?;

for entry in results {
    println!("Memory: {} (distance: {:.3})", entry.content, entry.distance);
}
```

### Tag-Based Recall

```rust
// Recall by specific tags
let preferences = memory.recall_by_tags(&["preferences"], 10)?;
let work_context = memory.recall_by_tags(&["work", "background"], 5)?;
```

### Memory Patterns

**Categorize memories effectively:**

```rust
// User information
memory.store("Name: Alice", &["user", "profile"])?;
memory.store("Timezone: PST", &["user", "profile"])?;

// Preferences
memory.store("Prefers concise answers", &["preferences", "communication"])?;
memory.store("Uses VS Code", &["preferences", "tools"])?;

// Task history
memory.store("Built REST API last week", &["history", "projects"])?;

// Knowledge
memory.store("User's project uses PostgreSQL", &["context", "tech-stack"])?;
```

## Managing Conversations

### Creating Conversations

```rust
// New conversation (auto-generated ID)
let conv = agent.conversation(None)?;

// Named conversation
let conv = agent.conversation(Some("support-ticket-123"))?;
```

### Adding Messages

```rust
use liath::agent::Role;

conv.add_message(Role::User, "Hello! Can you help me with Rust?")?;
conv.add_message(Role::Assistant, "Of course! What would you like to know?")?;
conv.add_message(Role::User, "How do I handle errors?")?;

// System messages for context
conv.add_message(Role::System, "User is a beginner Rust programmer")?;

// Tool results
conv.add_message(
    Role::Tool("code_executor".to_string()),
    "Execution successful: output = 42"
)?;
```

### Retrieving History

```rust
// Get message count
let count = conv.message_count();

// Get all messages
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

// Clear conversation
conv.clear()?;
```

## Tool State Management

### Storing State

```rust
let calculator = agent.tool_state("calculator")?;

// Store various types
calculator.set("last_result", &42.5)?;
calculator.set("operation_count", &100u32)?;
calculator.set("history", &vec!["1+1", "2*3", "sqrt(16)"])?;
calculator.set("settings", &serde_json::json!({
    "precision": 10,
    "mode": "scientific"
}))?;
```

### Retrieving State

```rust
// Get with type inference
let result: Option<f64> = calculator.get("last_result")?;
let count: Option<u32> = calculator.get("operation_count")?;
let history: Option<Vec<String>> = calculator.get("history")?;

// Check existence and delete
if calculator.exists("old_key")? {
    calculator.delete("old_key")?;
}
```

### Tool State Patterns

```rust
// Initialize with defaults
let state = agent.tool_state("web_browser")?;

if !state.exists("initialized")? {
    state.set("initialized", &true)?;
    state.set("tabs", &Vec::<String>::new())?;
    state.set("history", &Vec::<String>::new())?;
}

// Update state
let mut history: Vec<String> = state.get("history")?.unwrap_or_default();
history.push("https://example.com".to_string());
state.set("history", &history)?;
```

## Programmable Memory with Lua

### Using Query Executor

```rust
let executor = db.query_executor();

let code = r#"
    -- Store memories with metadata
    local mem_id = id()
    store_with_embedding("memory", mem_id, "Important fact")
    put("memory:meta", mem_id, json.encode({
        importance = 0.9,
        source = "user",
        timestamp = now()
    }))

    return mem_id
"#;

let result = executor.execute(code, agent.id()).await?;
```

### Smart Retrieval

```rust
let code = r#"
    function smart_recall(query, limit)
        local results = semantic_search("memory", query, limit * 2)

        local enriched = {}
        for _, r in ipairs(results) do
            local meta = json.decode(get("memory:meta", r.id) or '{}')

            local score = (1 - r.distance) * (meta.importance or 0.5)

            table.insert(enriched, {
                content = r.content,
                score = score,
                source = meta.source or "unknown"
            })
        end

        table.sort(enriched, function(a, b) return a.score > b.score end)

        local top = {}
        for i = 1, math.min(limit, #enriched) do
            table.insert(top, enriched[i])
        end

        return top
    end

    return json.encode(smart_recall("user preferences", 5))
"#;

let results = executor.execute(code, agent.id()).await?;
```

## Agent Architecture Patterns

### Stateful Agent Loop

```rust
pub struct ChatAgent {
    agent: Agent,
    executor: QueryExecutor,
}

impl ChatAgent {
    pub async fn process_message(&self, message: &str) -> Result<String, Error> {
        // 1. Store interaction
        let memory = self.agent.memory()?;
        memory.store(
            &format!("User said: {}", message),
            &["interaction", "user-message"]
        )?;

        // 2. Get relevant context
        let context = memory.recall(message, 5)?;

        // 3. Update conversation
        let conv = self.agent.conversation(None)?;
        conv.add_message(Role::User, message)?;

        // 4. Generate response (placeholder - use your LLM)
        let response = self.generate_response(message, &context).await?;

        // 5. Store assistant response
        conv.add_message(Role::Assistant, &response)?;

        Ok(response)
    }
}
```

### Multi-Agent System

```rust
pub struct AgentTeam {
    coordinator: Agent,
    specialists: HashMap<String, Agent>,
    db: Arc<EmbeddedLiath>,
}

impl AgentTeam {
    pub fn create_specialist(&mut self, role: &str) -> Result<(), Error> {
        let agent = Agent::new_with_description(
            &format!("specialist-{}", role),
            &format!("Specialist agent for {}", role),
            self.db.clone()
        );
        agent.save()?;
        self.specialists.insert(role.to_string(), agent);
        Ok(())
    }

    pub fn share_memory(&self, from: &str, to: &str, query: &str) -> Result<(), Error> {
        let source = self.specialists.get(from).ok_or("Agent not found")?;
        let target = self.specialists.get(to).ok_or("Agent not found")?;

        let memories = source.memory()?.recall(query, 5)?;

        let target_mem = target.memory()?;
        for mem in memories {
            target_mem.store(
                &format!("[From {}] {}", from, mem.content),
                &["shared", from]
            )?;
        }

        Ok(())
    }
}
```

### Tool-Using Agent

```rust
pub struct ToolAgent {
    agent: Agent,
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolAgent {
    pub async fn execute_tool(&self, name: &str, input: &str) -> Result<String, Error> {
        let tool = self.tools.get(name).ok_or("Tool not found")?;

        // Get tool state
        let state = self.agent.tool_state(name)?;

        // Execute with state
        let result = tool.execute(input, &state).await?;

        // Store tool usage in memory
        let memory = self.agent.memory()?;
        memory.store(
            &format!("Used {} with input: {}, result: {}", name, input, result),
            &["tool-usage", name]
        )?;

        Ok(result)
    }
}
```

## Best Practices

### Memory Organization

1. **Use consistent tagging** - Define a tag taxonomy and stick to it
2. **Store metadata separately** - Use `namespace:meta` for importance, timestamps
3. **Summarize periodically** - Compress old memories into summaries
4. **Clean up old data** - Remove outdated or irrelevant memories

### Conversation Management

1. **Limit history size** - Keep recent context, summarize older messages
2. **Use system messages** - Inject context at conversation start
3. **Thread related conversations** - Use consistent IDs for related topics

### Performance

1. **Batch operations** - Use Lua scripts to combine multiple operations
2. **Limit search results** - Only retrieve what you need
3. **Cache frequently accessed data** - Store computed values in tool state

### Safety

1. **Validate LLM-generated code** - Use the built-in validator
2. **Monitor memory growth** - Set limits on stored data
3. **Sanitize inputs** - Don't trust user input directly

## Next Steps

- [Agent API Reference](../api/agent-api.md) - Complete API documentation
- [Lua Scripting Guide](lua-scripting.md) - Advanced Lua patterns
- [Examples](../examples/index.md) - Working code examples
