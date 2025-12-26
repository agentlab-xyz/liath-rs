//! Programmable Memory - The Core Liath Concept
//!
//! This example demonstrates the key differentiator of Liath:
//! **Agents can write programs to query their own memory safely.**
//!
//! Traditional vector databases offer fixed APIs:
//!   semantic_search(query, limit) -> results
//!
//! Liath offers programmable memory:
//!   execute(agent_generated_code) -> results
//!
//! The agent (LLM) can generate Lua code to:
//! - Implement complex retrieval strategies
//! - Filter and rank memories by custom criteria
//! - Cross-reference multiple data sources
//! - Build context dynamically for the current task
//!
//! And it's SAFE - the Lua sandbox prevents system access.
//!
//! Run with: cargo run --example programmable_memory

use liath::{Config, EmbeddedLiath};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config {
        data_dir: "./data/programmable_memory".into(),
        ..Default::default()
    };
    let liath = EmbeddedLiath::new(config)?;
    let executor = liath.query_executor();

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║         PROGRAMMABLE MEMORY - The Liath Difference           ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Step 1: Populate memory with agent experiences
    println!("📝 Step 1: Agent accumulates memories over time\n");

    let setup_code = r#"
        -- Agent stores memories as it interacts with the user
        store_with_embedding("memory", "m1", "User is a software engineer at a startup")
        store_with_embedding("memory", "m2", "User prefers Rust over C++ for systems programming")
        store_with_embedding("memory", "m3", "User asked about async/await patterns yesterday")
        store_with_embedding("memory", "m4", "User mentioned deadline pressure for Q4 release")
        store_with_embedding("memory", "m5", "User likes concise explanations with code examples")
        store_with_embedding("memory", "m6", "User's project involves real-time data processing")
        store_with_embedding("memory", "m7", "User previously worked with Python but switched to Rust")
        store_with_embedding("memory", "m8", "User mentioned interest in WebAssembly")

        -- Store with timestamps (simulated as metadata)
        put("memory:meta", "m1", '{"importance": 0.8, "age_days": 30}')
        put("memory:meta", "m2", '{"importance": 0.9, "age_days": 7}')
        put("memory:meta", "m3", '{"importance": 0.7, "age_days": 1}')
        put("memory:meta", "m4", '{"importance": 0.95, "age_days": 2}')
        put("memory:meta", "m5", '{"importance": 0.85, "age_days": 14}')
        put("memory:meta", "m6", '{"importance": 0.8, "age_days": 5}')
        put("memory:meta", "m7", '{"importance": 0.6, "age_days": 21}')
        put("memory:meta", "m8", '{"importance": 0.5, "age_days": 10}')

        return "Memories stored"
    "#;

    executor.execute(setup_code, "agent").await?;
    println!("   ✓ 8 memories stored with importance scores and timestamps\n");

    // Step 2: Traditional approach - fixed API
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 Step 2: Traditional Approach (Fixed API)\n");
    println!("   Query: 'How should I help with their Rust project?'\n");

    let traditional_code = r#"
        -- Traditional: Just semantic search, no logic
        local results = semantic_search("memory", "Rust project help", 3)

        local output = {}
        for _, r in ipairs(results) do
            table.insert(output, r.content)
        end
        return json.encode(output)
    "#;

    let result = executor.execute(traditional_code, "agent").await?;
    println!("   Results (semantic similarity only):");
    println!("   {}\n", result);

    // Step 3: Programmable approach - agent writes retrieval logic
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🧠 Step 3: Programmable Approach (Agent-Generated Code)\n");
    println!("   The agent WRITES CODE to implement smart retrieval:\n");

    // This is code an LLM would generate based on the current context
    let agent_generated_code = r#"
        -- AGENT-GENERATED CODE
        -- Task: Build context to help with user's Rust project
        -- Strategy: Combine relevance, recency, and importance

        -- 1. Get semantically relevant memories
        local relevant = semantic_search("memory", "Rust programming project", 10)

        -- 2. Enrich with metadata
        local enriched = {}
        for _, r in ipairs(relevant) do
            local meta_json = get("memory:meta", r.id)
            local meta = meta_json and json.decode(meta_json) or {importance = 0.5, age_days = 30}

            -- 3. Calculate composite score
            -- Recency boost: memories from last 7 days get 2x weight
            local recency_boost = meta.age_days <= 7 and 2.0 or 1.0

            -- Relevance from semantic search (lower distance = more relevant)
            local relevance = 1 - r.distance

            -- Combined score
            local score = relevance * meta.importance * recency_boost

            table.insert(enriched, {
                content = r.content,
                relevance = relevance,
                importance = meta.importance,
                age_days = meta.age_days,
                score = score
            })
        end

        -- 4. Sort by composite score
        table.sort(enriched, function(a, b) return a.score > b.score end)

        -- 5. Take top 5 and format for context
        local context = {}
        for i = 1, math.min(5, #enriched) do
            local m = enriched[i]
            table.insert(context, {
                memory = m.content,
                why = string.format("relevance=%.0f%%, importance=%.0f%%, %d days ago",
                    m.relevance * 100, m.importance * 100, m.age_days)
            })
        end

        return json.encode({
            strategy = "relevance × importance × recency_boost",
            context = context
        })
    "#;

    println!("   ```lua");
    for line in agent_generated_code.lines().take(25) {
        if !line.trim().is_empty() {
            println!("   {}", line);
        }
    }
    println!("   ... (complex scoring logic)");
    println!("   ```\n");

    let result = executor.execute(agent_generated_code, "agent").await?;
    println!("   Results (smart retrieval):");

    // Pretty print the JSON
    let parsed: serde_json::Value = serde_json::from_str(&result)?;
    println!("   {}\n", serde_json::to_string_pretty(&parsed)?);

    // Step 4: Show safety
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔒 Step 4: Safety - Agent Code Cannot Access System\n");

    let unsafe_attempts = vec![
        ("os.execute('rm -rf /')", "Execute system command"),
        ("io.open('/etc/passwd', 'r')", "Read system file"),
        ("require('socket')", "Load network library"),
    ];

    for (code, description) in unsafe_attempts {
        let test_code = format!(
            r#"
            local ok, err = pcall(function()
                {}
            end)
            if not ok then
                return "BLOCKED: " .. tostring(err):sub(1, 50)
            else
                return "DANGER: Code executed!"
            end
            "#,
            code
        );

        let result = executor.execute(&test_code, "agent").await?;
        println!("   {} {}: {}", "🛡️", description, result);
    }

    // Step 5: Real-world pattern
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔄 Step 5: Real-World Pattern - Agent Conversation Loop\n");

    let conversation_loop = r#"
        -- This is how an agent uses programmable memory in practice

        function agent_turn(user_message)
            -- 1. Store this interaction as a memory
            local mem_id = id()
            store_with_embedding("memory", mem_id, "User asked: " .. user_message)
            put("memory:meta", mem_id, json.encode({
                importance = 0.7,
                age_days = 0,
                type = "interaction"
            }))

            -- 2. Build smart context (agent decides the strategy)
            local context = semantic_search("memory", user_message, 5)

            -- 3. Add conversation history
            add_message("conv", "user", user_message)
            local history = get_messages("conv", 5)

            -- 4. Return everything the LLM needs
            return json.encode({
                user_message = user_message,
                relevant_memories = map(context, function(c) return c.content end),
                conversation_history = history,
                memory_count = 9  -- Agent knows it has 9 memories now
            })
        end

        return agent_turn("Can you help me optimize my Rust async code?")
    "#;

    let result = executor.execute(conversation_loop, "agent").await?;
    let parsed: serde_json::Value = serde_json::from_str(&result)?;
    println!("   Agent turn result:");
    println!("   {}\n", serde_json::to_string_pretty(&parsed)?);

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║                        KEY TAKEAWAY                          ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  Traditional: Agent calls fixed API                          ║");
    println!("║  Liath:       Agent WRITES CODE to query memory              ║");
    println!("║                                                              ║");
    println!("║  The agent can implement ANY retrieval strategy:             ║");
    println!("║  • Recency-weighted search                                   ║");
    println!("║  • Multi-factor ranking                                      ║");
    println!("║  • Cross-referencing data sources                            ║");
    println!("║  • Custom filtering logic                                    ║");
    println!("║                                                              ║");
    println!("║  And it's SAFE - Lua sandbox blocks system access.           ║");
    println!("╚══════════════════════════════════════════════════════════════╝");

    Ok(())
}
