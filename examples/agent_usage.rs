//! Agent usage example
//!
//! This example demonstrates how to use Liath's agent API for building
//! AI agents with persistent memory, conversations, and tool state.

use liath::{EmbeddedLiath, Config};
use liath::agent::{Agent, Role};
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Liath Agent Usage Example ===\n");

    // Create database with temporary storage
    let config = Config {
        data_dir: std::env::temp_dir().join("liath_agent_example"),
        ..Default::default()
    };

    let db = Arc::new(EmbeddedLiath::new(config)?);
    println!("Database initialized.\n");

    // Create an agent with a description
    let agent = Agent::new_with_description(
        "assistant-1",
        "A helpful AI assistant for general tasks",
        db.clone()
    );
    println!("Created agent: {}\n", agent.id());

    // === Long-Term Memory ===
    println!("=== Long-Term Memory ===\n");

    let memory = agent.memory()?;

    // Store memories with tags for organization
    println!("Storing memories...");
    memory.store(
        "The user's preferred programming language is Rust",
        &["preferences", "programming"]
    )?;
    memory.store(
        "The user works in the fintech industry",
        &["background", "work"]
    )?;
    memory.store(
        "The user prefers concise explanations",
        &["preferences", "communication"]
    )?;
    memory.store(
        "Previous project: Built a REST API using Axum",
        &["history", "programming"]
    )?;
    println!("Stored 4 memories.\n");

    // Semantic recall
    println!("Semantic recall for 'What does the user like to code in?':");
    let results = memory.recall("What does the user like to code in?", 2)?;
    for entry in &results {
        println!("  - {} (distance: {:.3})", entry.content, entry.distance);
    }
    println!();

    // Tag-based recall
    println!("Tag-based recall for 'preferences':");
    let pref_results = memory.recall_by_tags(&["preferences"], 5)?;
    for entry in &pref_results {
        println!("  - {}", entry.content);
    }
    println!();

    // === Conversations ===
    println!("=== Conversations ===\n");

    let conv = agent.conversation(None)?;
    println!("Created new conversation: {}\n", conv.id());

    // Add messages
    conv.add_message(Role::User, "Hello! Can you help me with Rust?")?;
    conv.add_message(Role::Assistant, "Of course! I'd be happy to help with Rust. What would you like to know?")?;
    conv.add_message(Role::User, "How do I handle errors properly?")?;
    conv.add_message(Role::Assistant, "In Rust, you typically use the Result type for error handling. The ? operator is great for propagating errors.")?;

    println!("Conversation history ({} messages):", conv.message_count());
    let messages = conv.messages()?;
    for msg in &messages {
        let role_str = match msg.role {
            Role::User => "User",
            Role::Assistant => "Assistant",
            Role::System => "System",
            Role::Tool(_) => "Tool",
        };
        println!("  [{}] {}", role_str, msg.content);
    }
    println!();

    // === Tool State ===
    println!("=== Tool State ===\n");

    let calculator_state = agent.tool_state("calculator")?;

    // Store tool state
    calculator_state.set("last_result", &42.5f64)?;
    calculator_state.set("operation_count", &10u32)?;
    calculator_state.set("history", &vec!["1+1", "2*3", "sqrt(16)"])?;

    // Retrieve tool state
    let last: Option<f64> = calculator_state.get("last_result")?;
    let count: Option<u32> = calculator_state.get("operation_count")?;
    let history: Option<Vec<String>> = calculator_state.get("history")?;

    println!("Calculator tool state:");
    println!("  Last result: {:?}", last);
    println!("  Operation count: {:?}", count);
    println!("  History: {:?}", history);
    println!();

    // === Agent Persistence ===
    println!("=== Agent Persistence ===\n");

    // Save the agent data
    agent.save()?;
    println!("Agent data saved.");

    // List all agents
    let agents = Agent::list_agents(&db)?;
    println!("Registered agents: {:?}", agents.iter().map(|a| &a.id).collect::<Vec<_>>());

    // Check if agent exists
    let exists = Agent::exists("assistant-1", &db)?;
    println!("Agent 'assistant-1' exists: {}", exists);

    // Load agent metadata
    if let Some(metadata) = agent.metadata()? {
        println!("Agent metadata:");
        println!("  ID: {}", metadata.id);
        println!("  Created: {} (unix timestamp)", metadata.created_at);
        println!("  Description: {:?}", metadata.description);
    }

    println!("\nAgent usage example completed!");
    Ok(())
}
