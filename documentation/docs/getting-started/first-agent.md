# Building Your First Agent

This tutorial walks you through building a complete AI agent with persistent memory, conversation management, and tool state using Liath.

## Overview

By the end of this tutorial, you'll have built an agent that can:

- Store and recall semantic memories
- Maintain conversation history
- Persist tool state across sessions
- Use programmable memory for smart retrieval

## Prerequisites

- Rust 1.70+ installed
- Basic Rust knowledge
- Liath added to your project (see [Installation](installation.md))

## Step 1: Project Setup

Create a new Rust project:

```bash
cargo new my_agent
cd my_agent
```

Add dependencies to `Cargo.toml`:

```toml
[dependencies]
liath = "0.1"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
```

## Step 2: Initialize the Database

Create `src/main.rs`:

```rust
use liath::{EmbeddedLiath, Config};
use liath::agent::{Agent, Role};
use std::sync::Arc;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Configure the database
    let config = Config {
        data_dir: PathBuf::from("./agent_data"),
        ..Default::default()
    };

    // Create the database instance
    let db = Arc::new(EmbeddedLiath::new(config)?);
    println!("Database initialized at ./agent_data");

    // Create our agent
    let agent = Agent::new_with_description(
        "my-assistant",
        "A helpful AI assistant that remembers user preferences",
        db.clone()
    );

    // Save agent metadata
    agent.save()?;
    println!("Agent '{}' created", agent.id());

    Ok(())
}
```

Run to verify setup:

```bash
cargo run
```

## Step 3: Add Long-Term Memory

The memory system allows your agent to store and recall information semantically:

```rust
use liath::{EmbeddedLiath, Config};
use liath::agent::{Agent, Role};
use std::sync::Arc;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config {
        data_dir: PathBuf::from("./agent_data"),
        ..Default::default()
    };
    let db = Arc::new(EmbeddedLiath::new(config)?);

    let agent = Agent::new_with_description(
        "my-assistant",
        "A helpful AI assistant",
        db.clone()
    );

    // Access the memory system
    let memory = agent.memory()?;

    // Store memories with semantic tags
    println!("\n=== Storing Memories ===");

    let id1 = memory.store(
        "User's name is Alice and she works as a software engineer",
        &["user-profile", "personal"]
    )?;
    println!("Stored memory {}: user profile", id1);

    let id2 = memory.store(
        "User prefers Rust for systems programming and Python for data analysis",
        &["preferences", "programming"]
    )?;
    println!("Stored memory {}: programming preferences", id2);

    let id3 = memory.store(
        "User is working on a machine learning project using PyTorch",
        &["projects", "current-work"]
    )?;
    println!("Stored memory {}: current project", id3);

    let id4 = memory.store(
        "User prefers concise explanations with code examples",
        &["preferences", "communication"]
    )?;
    println!("Stored memory {}: communication style", id4);

    let id5 = memory.store(
        "User's timezone is PST and prefers morning meetings",
        &["preferences", "scheduling"]
    )?;
    println!("Stored memory {}: scheduling preferences", id5);

    // Semantic recall - find relevant memories
    println!("\n=== Semantic Recall ===");

    let query = "What programming languages does the user like?";
    println!("Query: {}", query);

    let results = memory.recall(query, 3)?;
    for (i, entry) in results.iter().enumerate() {
        println!("\n  {}. {} (distance: {:.3})", i + 1, entry.content, entry.distance);
        println!("     Tags: {:?}", entry.tags);
    }

    // Tag-based recall
    println!("\n=== Tag-Based Recall ===");

    let tag_results = memory.recall_by_tags(&["preferences"], 10)?;
    println!("Memories tagged with 'preferences':");
    for entry in &tag_results {
        println!("  - {}", entry.content);
    }

    Ok(())
}
```

## Step 4: Add Conversation Management

Track conversation history with the agent:

```rust
use liath::{EmbeddedLiath, Config};
use liath::agent::{Agent, Role};
use std::sync::Arc;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config {
        data_dir: PathBuf::from("./agent_data"),
        ..Default::default()
    };
    let db = Arc::new(EmbeddedLiath::new(config)?);

    let agent = Agent::new("my-assistant", db.clone());

    // Create a new conversation
    let conv = agent.conversation(None)?;
    println!("Started conversation: {}", conv.id());

    // Add system context
    conv.add_message(
        Role::System,
        "You are a helpful programming assistant. Be concise and provide code examples."
    )?;

    // Simulate a conversation
    conv.add_message(Role::User, "How do I read a file in Rust?")?;

    conv.add_message(
        Role::Assistant,
        r#"Use `std::fs::read_to_string`:

```rust
use std::fs;

fn main() -> std::io::Result<()> {
    let content = fs::read_to_string("file.txt")?;
    println!("{}", content);
    Ok(())
}
```"#
    )?;

    conv.add_message(Role::User, "What about async file reading?")?;

    conv.add_message(
        Role::Assistant,
        r#"Use `tokio::fs`:

```rust
use tokio::fs;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let content = fs::read_to_string("file.txt").await?;
    println!("{}", content);
    Ok(())
}
```"#
    )?;

    // Display conversation history
    println!("\n=== Conversation History ===");
    let messages = conv.messages()?;

    for msg in &messages {
        let role_str = match &msg.role {
            Role::User => "User",
            Role::Assistant => "Assistant",
            Role::System => "System",
            Role::Tool(name) => name.as_str(),
        };
        println!("\n[{}]", role_str);
        println!("{}", msg.content);
    }

    println!("\nTotal messages: {}", conv.message_count());

    // Search within conversation
    println!("\n=== Searching Conversation ===");
    let search_results = conv.search("async", 2)?;
    println!("Messages about 'async':");
    for msg in search_results {
        println!("  - {}", msg.content.chars().take(100).collect::<String>());
    }

    Ok(())
}
```

## Step 5: Add Tool State

Persist state for agent tools:

```rust
use liath::{EmbeddedLiath, Config};
use liath::agent::Agent;
use std::sync::Arc;
use std::path::PathBuf;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct CalculatorHistory {
    expressions: Vec<String>,
    results: Vec<f64>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config {
        data_dir: PathBuf::from("./agent_data"),
        ..Default::default()
    };
    let db = Arc::new(EmbeddedLiath::new(config)?);
    let agent = Agent::new("my-assistant", db.clone());

    // Get tool state for a calculator tool
    let calc_state = agent.tool_state("calculator")?;

    // Initialize if first run
    if !calc_state.exists("initialized")? {
        println!("Initializing calculator tool state...");

        calc_state.set("initialized", &true)?;
        calc_state.set("precision", &10u32)?;
        calc_state.set("mode", &"scientific")?;
        calc_state.set("history", &CalculatorHistory {
            expressions: vec![],
            results: vec![],
        })?;
    }

    // Simulate using the calculator
    let mut history: CalculatorHistory = calc_state.get("history")?.unwrap();

    // Add some calculations
    let expressions = vec![
        ("2 + 2", 4.0),
        ("sqrt(16)", 4.0),
        ("sin(pi/2)", 1.0),
        ("2^10", 1024.0),
    ];

    for (expr, result) in expressions {
        history.expressions.push(expr.to_string());
        history.results.push(result);
        println!("Calculated: {} = {}", expr, result);
    }

    // Save updated history
    calc_state.set("history", &history)?;
    calc_state.set("last_result", &history.results.last().unwrap())?;

    // Retrieve state
    println!("\n=== Calculator State ===");
    let mode: String = calc_state.get("mode")?.unwrap();
    let precision: u32 = calc_state.get("precision")?.unwrap();
    let last: f64 = calc_state.get("last_result")?.unwrap();

    println!("Mode: {}", mode);
    println!("Precision: {}", precision);
    println!("Last result: {}", last);
    println!("History: {} calculations", history.expressions.len());

    Ok(())
}
```

## Step 6: Programmable Memory with Lua

Use Lua scripts for advanced retrieval:

```rust
use liath::{EmbeddedLiath, Config};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config {
        data_dir: PathBuf::from("./agent_data"),
        ..Default::default()
    };
    let db = EmbeddedLiath::new(config)?;
    let executor = db.query_executor();

    // Setup: Store memories with metadata
    let setup_code = r#"
        -- Store memories
        store_with_embedding("memory", "m1", "User is a senior software engineer")
        store_with_embedding("memory", "m2", "User prefers Rust for backend development")
        store_with_embedding("memory", "m3", "User asked about async patterns today")
        store_with_embedding("memory", "m4", "User has deadline next week")
        store_with_embedding("memory", "m5", "User enjoys functional programming")

        -- Store metadata
        put("memory:meta", "m1", json.encode({importance = 0.9, age_days = 30}))
        put("memory:meta", "m2", json.encode({importance = 0.95, age_days = 7}))
        put("memory:meta", "m3", json.encode({importance = 0.7, age_days = 0}))
        put("memory:meta", "m4", json.encode({importance = 1.0, age_days = 1}))
        put("memory:meta", "m5", json.encode({importance = 0.6, age_days = 14}))

        return "Setup complete"
    "#;

    executor.execute(setup_code, "my-assistant").await?;
    println!("Memory setup complete");

    // Smart retrieval with multi-factor ranking
    let smart_query = r#"
        -- Agent-generated smart retrieval function
        local function smart_recall(query, limit)
            -- Get semantically relevant memories
            local results = semantic_search("memory", query, limit * 2)

            local scored = {}
            for _, r in ipairs(results) do
                -- Get metadata
                local meta_json = get("memory:meta", r.id)
                local meta = meta_json and json.decode(meta_json) or {
                    importance = 0.5,
                    age_days = 30
                }

                -- Calculate composite score
                local relevance = 1 - r.distance  -- Convert distance to similarity
                local importance = meta.importance or 0.5

                -- Recency boost: recent memories get higher weight
                local recency_boost = 1.0
                if meta.age_days <= 1 then
                    recency_boost = 2.0  -- Today's memories
                elseif meta.age_days <= 7 then
                    recency_boost = 1.5  -- This week
                end

                local score = relevance * importance * recency_boost

                table.insert(scored, {
                    id = r.id,
                    content = r.content,
                    score = score,
                    factors = {
                        relevance = string.format("%.0f%%", relevance * 100),
                        importance = string.format("%.0f%%", importance * 100),
                        recency = string.format("%.1fx", recency_boost),
                        age_days = meta.age_days
                    }
                })
            end

            -- Sort by composite score
            table.sort(scored, function(a, b) return a.score > b.score end)

            -- Return top results
            local top = {}
            for i = 1, math.min(limit, #scored) do
                table.insert(top, scored[i])
            end
            return top
        end

        -- Execute smart recall
        local results = smart_recall("What should I know about helping with their Rust project?", 3)

        return json.encode({
            query = "Rust project context",
            results = results
        })
    "#;

    let result = executor.execute(smart_query, "my-assistant").await?;

    // Pretty print results
    let parsed: serde_json::Value = serde_json::from_str(&result)?;
    println!("\n=== Smart Retrieval Results ===");
    println!("{}", serde_json::to_string_pretty(&parsed)?);

    Ok(())
}
```

## Step 7: Complete Agent Implementation

Here's a complete agent combining all features:

```rust
use liath::{EmbeddedLiath, Config};
use liath::agent::{Agent, Role};
use std::sync::Arc;
use std::path::PathBuf;

pub struct MyAssistant {
    agent: Agent,
    db: Arc<EmbeddedLiath>,
}

impl MyAssistant {
    pub fn new(data_dir: &str) -> anyhow::Result<Self> {
        let config = Config {
            data_dir: PathBuf::from(data_dir),
            ..Default::default()
        };
        let db = Arc::new(EmbeddedLiath::new(config)?);

        // Load or create agent
        let agent = if Agent::exists("assistant", &db)? {
            Agent::load("assistant", db.clone())?.unwrap()
        } else {
            let agent = Agent::new_with_description(
                "assistant",
                "Personal AI assistant with persistent memory",
                db.clone()
            );
            agent.save()?;
            agent
        };

        Ok(Self { agent, db })
    }

    pub fn learn(&self, fact: &str, tags: &[&str]) -> anyhow::Result<u64> {
        let memory = self.agent.memory()?;
        Ok(memory.store(fact, tags)?)
    }

    pub fn recall(&self, query: &str, limit: usize) -> anyhow::Result<Vec<String>> {
        let memory = self.agent.memory()?;
        let results = memory.recall(query, limit)?;
        Ok(results.into_iter().map(|e| e.content).collect())
    }

    pub fn chat(&self, conv_id: Option<&str>, message: &str) -> anyhow::Result<String> {
        let conv = self.agent.conversation(conv_id)?;

        // Get relevant context
        let memory = self.agent.memory()?;
        let context = memory.recall(message, 3)?;

        // Add user message
        conv.add_message(Role::User, message)?;

        // In a real implementation, you'd call your LLM here
        // For now, we'll return a placeholder
        let response = format!(
            "I found {} relevant memories for your question. [LLM response would go here]",
            context.len()
        );

        conv.add_message(Role::Assistant, &response)?;

        Ok(response)
    }

    pub async fn smart_recall(&self, query: &str) -> anyhow::Result<String> {
        let executor = self.db.query_executor();

        let code = format!(r#"
            local results = semantic_search("agent:assistant:memory", "{}", 5)
            local enriched = {{}}

            for _, r in ipairs(results) do
                table.insert(enriched, {{
                    content = r.content,
                    relevance = 1 - r.distance
                }})
            end

            return json.encode(enriched)
        "#, query.replace('"', r#"\""#));

        Ok(executor.execute(&code, "assistant").await?)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let assistant = MyAssistant::new("./my_assistant_data")?;

    // Teach the assistant
    assistant.learn("User's favorite color is blue", &["preferences"])?;
    assistant.learn("User is learning Rust", &["learning", "programming"])?;
    assistant.learn("User works at a tech startup", &["work", "background"])?;

    // Recall memories
    let memories = assistant.recall("What is the user learning?", 2)?;
    println!("Relevant memories:");
    for m in memories {
        println!("  - {}", m);
    }

    // Chat
    let response = assistant.chat(None, "Can you help me with Rust?")?;
    println!("\nAssistant: {}", response);

    // Smart recall
    let smart_results = assistant.smart_recall("programming interests").await?;
    println!("\nSmart recall: {}", smart_results);

    Ok(())
}
```

## Next Steps

Now that you've built your first agent, explore:

- [Memory Patterns](../guides/memory-patterns.md) - Advanced memory organization
- [Lua Scripting](../guides/lua-scripting.md) - Custom retrieval logic
- [Conversation Management](../guides/conversations.md) - Thread management
- [Tool State](../guides/tool-state.md) - Stateful tool development

## Troubleshooting

### Common Issues

**"Failed to create embedded instance"**

This usually means the embedding model failed to load. Ensure you have enough memory (512MB+) and the ONNX runtime is available.

**"Namespace not found"**

Namespaces are created automatically when using the Agent API. If you're using low-level APIs, create the namespace first:

```rust
db.create_namespace("my_namespace", 384, MetricKind::Cos, ScalarKind::F32)?;
```

**"Permission denied" for data directory**

Ensure the data directory exists and is writable:

```rust
std::fs::create_dir_all("./agent_data")?;
```
