# Examples

This section provides practical examples demonstrating Liath's capabilities.

## Basic Examples

### Embedded Database

Basic usage of Liath as an embedded database:

```rust
use liath::{EmbeddedLiath, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::default();
    let db = EmbeddedLiath::new(config)?;

    // Create namespace with vector index
    #[cfg(feature = "vector")]
    {
        use usearch::{MetricKind, ScalarKind};
        db.create_namespace("example", 384, MetricKind::Cos, ScalarKind::F32)?;
    }

    // Put and get values
    db.put("example", b"greeting", b"Hello, Liath!")?;

    if let Some(value) = db.get("example", b"greeting")? {
        println!("Retrieved: {}", String::from_utf8_lossy(&value));
    }

    // Clean up
    db.delete("example", b"greeting")?;

    Ok(())
}
```

### Key-Value Operations

```rust
use liath::{EmbeddedLiath, Config};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = EmbeddedLiath::new(Config::default())?;

    // Store JSON data
    let user = serde_json::json!({
        "name": "Alice",
        "email": "alice@example.com",
        "role": "admin"
    });
    db.put("users", b"user:1", user.to_string().as_bytes())?;

    // Retrieve and parse
    if let Some(data) = db.get("users", b"user:1")? {
        let user: serde_json::Value = serde_json::from_slice(&data)?;
        println!("User: {}", user["name"]);
    }

    Ok(())
}
```

---

## Vector Search Examples

### Semantic Search

```rust
use liath::{EmbeddedLiath, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = EmbeddedLiath::new(Config::default())?;
    let executor = db.query_executor();

    // Store documents with embeddings
    let code = r#"
        store_with_embedding("docs", "rust", "Rust is a systems programming language focused on safety and performance")
        store_with_embedding("docs", "python", "Python is a high-level language great for data science and scripting")
        store_with_embedding("docs", "javascript", "JavaScript is the language of the web, running in browsers")
        store_with_embedding("docs", "go", "Go is a statically typed language designed for simplicity and efficiency")

        -- Search for systems programming
        local results = semantic_search("docs", "low-level systems programming", 2)

        return json.encode(map(results, function(r)
            return { doc = r.id, content = r.content, relevance = 1 - r.distance }
        end))
    "#;

    let result = executor.execute(code, "agent").await?;
    println!("{}", result);

    Ok(())
}
```

---

## Agent Examples

### Complete Agent Usage

```rust
use liath::{EmbeddedLiath, Config};
use liath::agent::{Agent, Role};
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Arc::new(EmbeddedLiath::new(Config::default())?);

    // Create agent
    let agent = Agent::new_with_description(
        "assistant-1",
        "A helpful AI assistant",
        db.clone()
    );

    // === Memory ===
    let memory = agent.memory()?;

    memory.store("User prefers Rust", &["preferences", "programming"])?;
    memory.store("User works in fintech", &["background", "work"])?;
    memory.store("User likes concise explanations", &["preferences", "communication"])?;

    // Semantic recall
    let results = memory.recall("What programming language?", 2)?;
    for entry in &results {
        println!("Memory: {} (distance: {:.3})", entry.content, entry.distance);
    }

    // Tag-based recall
    let prefs = memory.recall_by_tags(&["preferences"], 5)?;
    println!("Found {} preference memories", prefs.len());

    // === Conversations ===
    let conv = agent.conversation(None)?;

    conv.add_message(Role::User, "Hello!")?;
    conv.add_message(Role::Assistant, "Hi! How can I help?")?;

    let messages = conv.messages()?;
    println!("Conversation has {} messages", messages.len());

    // === Tool State ===
    let calc_state = agent.tool_state("calculator")?;
    calc_state.set("last_result", &42.5)?;

    let result: Option<f64> = calc_state.get("last_result")?;
    println!("Last result: {:?}", result);

    // Save agent
    agent.save()?;

    Ok(())
}
```

---

## Programmable Memory Examples

### Smart Retrieval

```rust
use liath::{EmbeddedLiath, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = EmbeddedLiath::new(Config::default())?;
    let executor = db.query_executor();

    // Setup memories with metadata
    let setup = r#"
        store_with_embedding("memory", "m1", "User is a software engineer")
        store_with_embedding("memory", "m2", "User prefers Rust over C++")
        store_with_embedding("memory", "m3", "User asked about async patterns yesterday")
        store_with_embedding("memory", "m4", "User has deadline pressure for Q4")

        put("memory:meta", "m1", '{"importance": 0.8, "age_days": 30}')
        put("memory:meta", "m2", '{"importance": 0.9, "age_days": 7}')
        put("memory:meta", "m3", '{"importance": 0.7, "age_days": 1}')
        put("memory:meta", "m4", '{"importance": 0.95, "age_days": 2}')

        return "setup complete"
    "#;
    executor.execute(setup, "agent").await?;

    // Smart retrieval with multi-factor ranking
    let smart_query = r#"
        local function smart_recall(query)
            local results = semantic_search("memory", query, 10)

            local enriched = {}
            for _, r in ipairs(results) do
                local meta = json.decode(get("memory:meta", r.id) or '{}')

                -- Multi-factor scoring
                local relevance = 1 - r.distance
                local importance = meta.importance or 0.5
                local recency = meta.age_days <= 7 and 2.0 or 1.0

                local score = relevance * importance * recency

                table.insert(enriched, {
                    content = r.content,
                    score = score,
                    factors = {
                        relevance = relevance,
                        importance = importance,
                        recency = recency
                    }
                })
            end

            table.sort(enriched, function(a, b) return a.score > b.score end)

            -- Return top 3
            local top = {}
            for i = 1, math.min(3, #enriched) do
                table.insert(top, enriched[i])
            end
            return top
        end

        return json.encode(smart_recall("How should I help with their Rust project?"))
    "#;

    let result = executor.execute(smart_query, "agent").await?;
    let parsed: serde_json::Value = serde_json::from_str(&result)?;
    println!("{}", serde_json::to_string_pretty(&parsed)?);

    Ok(())
}
```

### Conversation with Context

```rust
use liath::{EmbeddedLiath, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = EmbeddedLiath::new(Config::default())?;
    let executor = db.query_executor();

    let agent_turn = r#"
        -- Simulate an agent turn
        local user_message = "Can you help me optimize my Rust async code?"

        -- 1. Store this interaction
        local mem_id = id()
        store_with_embedding("memory", mem_id, "User asked: " .. user_message)
        put("memory:meta", mem_id, json.encode({
            importance = 0.7,
            age_days = 0,
            type = "interaction"
        }))

        -- 2. Get relevant context
        local context = semantic_search("memory", user_message, 5)

        -- 3. Add to conversation
        add_message("main", "user", user_message)
        local history = get_messages("main", 10)

        -- 4. Return context for LLM
        return json.encode({
            user_message = user_message,
            relevant_context = map(context, function(c) return c.content end),
            conversation_history = history
        })
    "#;

    let result = executor.execute(agent_turn, "agent").await?;
    let parsed: serde_json::Value = serde_json::from_str(&result)?;
    println!("{}", serde_json::to_string_pretty(&parsed)?);

    Ok(())
}
```

---

## Safety Demonstration

```rust
use liath::{EmbeddedLiath, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = EmbeddedLiath::new(Config::default())?;
    let executor = db.query_executor();

    // These dangerous operations are safely blocked
    let test_cases = vec![
        ("os.execute('ls')", "System command"),
        ("io.open('/etc/passwd')", "File access"),
        ("require('socket')", "Network library"),
    ];

    for (code, description) in test_cases {
        let test = format!(r#"
            local ok, err = pcall(function() {} end)
            if not ok then
                return "BLOCKED: " .. tostring(err):sub(1, 50)
            else
                return "EXECUTED"
            end
        "#, code);

        let result = executor.execute(&test, "agent").await?;
        println!("{}: {}", description, result);
    }

    Ok(())
}
```

---

## Running Examples

The repository includes runnable examples:

```bash
# Basic embedded usage
cargo run --example embedded

# Vector search
cargo run --example vector_search

# Agent API
cargo run --example agent_usage

# Programmable memory
cargo run --example programmable_memory

# Lua scripting
cargo run --example lua_scripting
```

---

## Next Steps

- [Lua Scripting Guide](../guides/lua-scripting.md) - Full Lua reference
- [Agent API](../api/agent-api.md) - Complete API documentation
- [Architecture](../concepts/architecture.md) - System design overview
