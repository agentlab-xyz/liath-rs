//! Agent Runtime Example
//!
//! Demonstrates using Liath's Lua runtime as a safe execution environment
//! for AI agent logic. Shows how agents can:
//! - Store and recall memories semantically
//! - Manage conversation history
//! - Execute complex retrieval logic safely
//! - Process data without system access
//!
//! Run with: cargo run --example agent_runtime

use liath::{Config, EmbeddedLiath};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create Liath instance
    let config = Config {
        data_dir: "./data/agent_runtime_example".into(),
        ..Default::default()
    };
    let liath = EmbeddedLiath::new(config)?;
    let executor = liath.query_executor();

    println!("=== Liath Agent Runtime Example ===\n");

    // Example 1: Safe Code Execution
    println!("1. Safe Code Execution");
    println!("   Agent can execute Lua safely - no system access\n");

    let safe_code = r#"
        -- This runs in a sandbox - no file system, no network
        local data = {10, 20, 30, 40, 50}

        local sum = reduce(data, function(acc, n) return acc + n end, 0)
        local doubled = map(data, function(n) return n * 2 end)
        local big = filter(data, function(n) return n > 25 end)

        return json.encode({
            original = data,
            sum = sum,
            average = sum / #data,
            doubled = doubled,
            over_25 = big
        })
    "#;

    let result = executor.execute(safe_code, "agent-1").await?;
    println!("   Result: {}\n", result);

    // Example 2: Memory Storage and Semantic Recall
    println!("2. Memory Storage and Semantic Recall");
    println!("   Agent stores experiences and recalls by meaning\n");

    let memory_code = r#"
        -- Store memories with semantic indexing
        store_with_embedding("agent:memories", "m1",
            "User prefers dark mode and minimalist interfaces")
        store_with_embedding("agent:memories", "m2",
            "User is working on a machine learning project in Python")
        store_with_embedding("agent:memories", "m3",
            "User mentioned they drink coffee in the morning")
        store_with_embedding("agent:memories", "m4",
            "User asked about Rust programming last week")
        store_with_embedding("agent:memories", "m5",
            "User prefers concise, technical explanations")

        -- Recall memories relevant to a programming question
        local relevant = semantic_search("agent:memories",
            "how should I explain code to this user?", 3)

        local memories = {}
        for _, r in ipairs(relevant) do
            table.insert(memories, {
                content = r.content,
                relevance = string.format("%.2f", 1 - r.distance)
            })
        end

        return json.encode(memories)
    "#;

    let result = executor.execute(memory_code, "agent-1").await?;
    println!("   Relevant memories: {}\n", result);

    // Example 3: Conversation Management
    println!("3. Conversation Management");
    println!("   Agent tracks conversation history\n");

    let conversation_code = r#"
        local conv_id = "chat-123"

        -- Simulate a conversation
        add_message(conv_id, "user", "Hi! Can you help me with Rust?")
        add_message(conv_id, "assistant", "Of course! What would you like to know about Rust?")
        add_message(conv_id, "user", "How do I handle errors?")
        add_message(conv_id, "assistant", "Rust uses Result<T, E> for error handling...")
        add_message(conv_id, "user", "Can you show me an example?")

        -- Get conversation history
        local history = get_messages(conv_id, 10)

        return json.encode({
            message_count = #history,
            messages = history
        })
    "#;

    let result = executor.execute(conversation_code, "agent-1").await?;
    println!("   Conversation: {}\n", result);

    // Example 4: Complex Retrieval Logic
    println!("4. Complex Retrieval Logic");
    println!("   Agent combines semantic search with filtering\n");

    let retrieval_code = r#"
        -- Define a RAG retrieval function
        function get_context(query, max_results)
            -- Semantic search
            local results = semantic_search("agent:memories", query, max_results * 2)

            -- Filter by relevance threshold
            local filtered = filter(results, function(r)
                return r.distance < 0.7  -- Only keep high similarity
            end)

            -- Transform to context format
            local context = map(filtered, function(r)
                return {
                    text = r.content,
                    score = 1 - r.distance
                }
            end)

            -- Sort by score (highest first)
            table.sort(context, function(a, b)
                return a.score > b.score
            end)

            -- Return top results
            local top = {}
            for i = 1, math.min(max_results, #context) do
                table.insert(top, context[i])
            end

            return top
        end

        local context = get_context("programming preferences", 3)
        return json.encode(context)
    "#;

    let result = executor.execute(retrieval_code, "agent-1").await?;
    println!("   Retrieved context: {}\n", result);

    // Example 5: Full Agent Turn
    println!("5. Full Agent Turn");
    println!("   Complete agent workflow: memory + conversation + context\n");

    let agent_turn_code = r#"
        -- Agent processes a user message
        function agent_turn(user_message)
            local agent_id = "assistant"
            local conv_id = "main"

            -- 1. Store the user message
            add_message(conv_id, "user", user_message)

            -- 2. Store this as a memory (agent learns from interactions)
            store_with_embedding("agent:memories", id(),
                "User asked: " .. user_message)

            -- 3. Get relevant memories
            local memories = semantic_search("agent:memories", user_message, 5)

            -- 4. Get conversation history
            local history = get_messages(conv_id, 10)

            -- 5. Build context for LLM
            local context = {
                user_message = user_message,
                relevant_memories = map(memories, function(m)
                    return m.content
                end),
                conversation_history = map(history, function(h)
                    return h.role .. ": " .. h.content
                end),
                timestamp = now()
            }

            return json.encode(context)
        end

        return agent_turn("What programming languages have we discussed?")
    "#;

    let result = executor.execute(agent_turn_code, "agent-1").await?;
    println!("   Agent context: {}\n", result);

    // Example 6: Tool State Management
    println!("6. Tool State Management");
    println!("   Agent tracks state across tool invocations\n");

    let tool_state_code = r#"
        -- Browser tool simulation
        function browser_tool(action, params)
            local tool = "browser"

            if action == "navigate" then
                set_tool_state(tool, "url", params.url)
                set_tool_state(tool, "title", params.title or "")

                -- Track history
                local history = json.decode(get_tool_state(tool, "history") or "[]")
                table.insert(history, {
                    url = params.url,
                    timestamp = now()
                })
                set_tool_state(tool, "history", json.encode(history))

                return {status = "navigated", url = params.url}

            elseif action == "get_state" then
                return {
                    url = get_tool_state(tool, "url"),
                    title = get_tool_state(tool, "title"),
                    history = json.decode(get_tool_state(tool, "history") or "[]")
                }
            end
        end

        -- Simulate browser usage
        browser_tool("navigate", {url = "https://rust-lang.org", title = "Rust"})
        browser_tool("navigate", {url = "https://docs.rs", title = "Docs.rs"})
        browser_tool("navigate", {url = "https://crates.io", title = "Crates.io"})

        local state = browser_tool("get_state", {})
        return json.encode(state)
    "#;

    let result = executor.execute(tool_state_code, "agent-1").await?;
    println!("   Browser state: {}\n", result);

    println!("=== Example Complete ===");
    println!("\nKey takeaways:");
    println!("- Lua runs in a sandbox (no system access)");
    println!("- Semantic search enables memory recall by meaning");
    println!("- Conversation history persists across turns");
    println!("- Complex logic can be expressed in Lua");
    println!("- Tool state survives across invocations");

    Ok(())
}
