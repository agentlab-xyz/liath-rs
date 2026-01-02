# Agent Patterns

Examples of common patterns for building AI agents with Liath.

## Basic Agent Setup

```rust
use liath::{EmbeddedLiath, Config};
use liath::agent::{Agent, Role};
use std::sync::Arc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Arc::new(EmbeddedLiath::new(Config::default())?);

    // Create agent with description
    let agent = Agent::new_with_description(
        "assistant",
        "A helpful AI assistant for programming tasks",
        db.clone()
    );

    // Save metadata
    agent.save()?;

    // Check agent exists
    println!("Agent exists: {}", Agent::exists("assistant", &db)?);

    // List all agents
    for meta in Agent::list_agents(&db)? {
        println!("Agent: {} - {:?}", meta.id, meta.description);
    }

    Ok(())
}
```

## Memory-Augmented Agent

```rust
use liath::{EmbeddedLiath, Config};
use liath::agent::{Agent, Role};
use std::sync::Arc;

pub struct MemoryAgent {
    agent: Agent,
    db: Arc<EmbeddedLiath>,
}

impl MemoryAgent {
    pub fn new(id: &str, db: Arc<EmbeddedLiath>) -> Result<Self, Box<dyn std::error::Error>> {
        let agent = if Agent::exists(id, &db)? {
            Agent::load(id, db.clone())?.unwrap()
        } else {
            let agent = Agent::new(id, db.clone());
            agent.save()?;
            agent
        };

        Ok(Self { agent, db })
    }

    pub fn learn(&self, fact: &str, tags: &[&str]) -> Result<u64, Box<dyn std::error::Error>> {
        let memory = self.agent.memory()?;
        Ok(memory.store(fact, tags)?)
    }

    pub fn recall(&self, query: &str, limit: usize) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let memory = self.agent.memory()?;
        let results = memory.recall(query, limit)?;
        Ok(results.into_iter().map(|e| e.content).collect())
    }

    pub fn process_message(&self, message: &str) -> Result<String, Box<dyn std::error::Error>> {
        // Store the interaction
        self.learn(&format!("User said: {}", message), &["interaction"])?;

        // Get relevant context
        let context = self.recall(message, 5)?;

        // Build response context
        let context_str = context.join("\n- ");

        Ok(format!(
            "Based on context:\n- {}\n\nUser message: {}",
            context_str, message
        ))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Arc::new(EmbeddedLiath::new(Config::default())?);
    let agent = MemoryAgent::new("memory-agent", db)?;

    // Teach the agent
    agent.learn("User's favorite language is Rust", &["preferences", "programming"])?;
    agent.learn("User prefers dark mode", &["preferences", "ui"])?;
    agent.learn("User is working on a web project", &["context", "projects"])?;

    // Process a message
    let response = agent.process_message("What should I use for my project?")?;
    println!("{}", response);

    Ok(())
}
```

## Conversational Agent

```rust
use liath::{EmbeddedLiath, Config};
use liath::agent::{Agent, Role, Conversation};
use std::sync::Arc;

pub struct ChatAgent {
    agent: Agent,
    current_conv: Option<String>,
}

impl ChatAgent {
    pub fn new(db: Arc<EmbeddedLiath>) -> Result<Self, Box<dyn std::error::Error>> {
        let agent = Agent::new_with_description(
            "chat-agent",
            "Conversational AI assistant",
            db
        );
        agent.save()?;

        Ok(Self {
            agent,
            current_conv: None,
        })
    }

    pub fn start_conversation(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        let conv = self.agent.conversation(None)?;
        let id = conv.id().to_string();

        // Add system prompt
        conv.add_message(Role::System, r#"
            You are a helpful assistant.
            Be concise and friendly.
            Ask clarifying questions when needed.
        "#)?;

        self.current_conv = Some(id.clone());
        Ok(id)
    }

    pub fn send_message(&self, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conv_id = self.current_conv.as_ref()
            .ok_or("No active conversation")?;

        let conv = self.agent.conversation(Some(conv_id))?;
        conv.add_message(Role::User, content)?;

        Ok(())
    }

    pub fn add_response(&self, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        let conv_id = self.current_conv.as_ref()
            .ok_or("No active conversation")?;

        let conv = self.agent.conversation(Some(conv_id))?;
        conv.add_message(Role::Assistant, content)?;

        Ok(())
    }

    pub fn get_history(&self, limit: usize) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
        let conv_id = self.current_conv.as_ref()
            .ok_or("No active conversation")?;

        let conv = self.agent.conversation(Some(conv_id))?;
        let messages = conv.last_n(limit)?;

        Ok(messages.into_iter().map(|m| {
            let role = match m.role {
                Role::User => "user".to_string(),
                Role::Assistant => "assistant".to_string(),
                Role::System => "system".to_string(),
                Role::Tool(name) => format!("tool:{}", name),
            };
            (role, m.content)
        }).collect())
    }
}
```

## Tool-Using Agent

```rust
use liath::{EmbeddedLiath, Config};
use liath::agent::{Agent, ToolState};
use std::sync::Arc;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Default)]
struct CalculatorState {
    last_result: Option<f64>,
    history: Vec<String>,
}

pub struct ToolAgent {
    agent: Agent,
}

impl ToolAgent {
    pub fn new(db: Arc<EmbeddedLiath>) -> Result<Self, Box<dyn std::error::Error>> {
        let agent = Agent::new("tool-agent", db);
        Ok(Self { agent })
    }

    pub fn use_calculator(&self, expression: &str) -> Result<f64, Box<dyn std::error::Error>> {
        let state = self.agent.tool_state("calculator")?;

        // Load state
        let mut calc_state: CalculatorState = state.get("state")?.unwrap_or_default();

        // Evaluate (simplified)
        let result = self.evaluate(expression)?;

        // Update state
        calc_state.last_result = Some(result);
        calc_state.history.push(format!("{} = {}", expression, result));

        // Keep history bounded
        if calc_state.history.len() > 100 {
            calc_state.history = calc_state.history.split_off(calc_state.history.len() - 100);
        }

        state.set("state", &calc_state)?;

        // Store in memory
        let memory = self.agent.memory()?;
        memory.store(
            &format!("Calculated: {} = {}", expression, result),
            &["tool_usage", "calculator"]
        )?;

        Ok(result)
    }

    fn evaluate(&self, expr: &str) -> Result<f64, Box<dyn std::error::Error>> {
        // Simplified evaluation
        Ok(42.0)  // Replace with actual evaluation
    }

    pub fn get_calculator_history(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let state = self.agent.tool_state("calculator")?;
        let calc_state: CalculatorState = state.get("state")?.unwrap_or_default();
        Ok(calc_state.history)
    }
}
```

## Multi-Agent System

```rust
use liath::{EmbeddedLiath, Config};
use liath::agent::Agent;
use std::sync::Arc;
use std::collections::HashMap;

pub struct AgentTeam {
    db: Arc<EmbeddedLiath>,
    agents: HashMap<String, Agent>,
}

impl AgentTeam {
    pub fn new(db: Arc<EmbeddedLiath>) -> Self {
        Self {
            db,
            agents: HashMap::new(),
        }
    }

    pub fn create_agent(&mut self, role: &str, description: &str) -> Result<(), Box<dyn std::error::Error>> {
        let agent = Agent::new_with_description(
            &format!("team:{}", role),
            description,
            self.db.clone()
        );
        agent.save()?;

        self.agents.insert(role.to_string(), agent);
        Ok(())
    }

    pub fn assign_knowledge(&self, role: &str, knowledge: &str, tags: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
        let agent = self.agents.get(role)
            .ok_or("Agent not found")?;

        let memory = agent.memory()?;
        memory.store(knowledge, tags)?;

        Ok(())
    }

    pub fn share_knowledge(&self, from: &str, to: &str, query: &str) -> Result<(), Box<dyn std::error::Error>> {
        let from_agent = self.agents.get(from).ok_or("Source agent not found")?;
        let to_agent = self.agents.get(to).ok_or("Target agent not found")?;

        // Get knowledge from source
        let from_memory = from_agent.memory()?;
        let knowledge = from_memory.recall(query, 5)?;

        // Share to target
        let to_memory = to_agent.memory()?;
        for entry in knowledge {
            to_memory.store(
                &format!("[From {}] {}", from, entry.content),
                &["shared", from]
            )?;
        }

        Ok(())
    }

    pub fn coordinate(&self, task: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut results = Vec::new();

        for (role, agent) in &self.agents {
            let memory = agent.memory()?;
            let relevant = memory.recall(task, 3)?;

            if !relevant.is_empty() {
                results.push(format!(
                    "{}: {}",
                    role,
                    relevant.iter().map(|r| r.content.clone()).collect::<Vec<_>>().join("; ")
                ));
            }
        }

        Ok(results.join("\n"))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = Arc::new(EmbeddedLiath::new(Config::default())?);
    let mut team = AgentTeam::new(db);

    // Create specialists
    team.create_agent("researcher", "Finds and organizes information")?;
    team.create_agent("coder", "Writes and reviews code")?;
    team.create_agent("reviewer", "Reviews work and provides feedback")?;

    // Assign knowledge
    team.assign_knowledge("researcher", "Latest ML papers focus on transformers", &["ml", "research"])?;
    team.assign_knowledge("coder", "Use Rust for performance-critical code", &["best-practices"])?;
    team.assign_knowledge("reviewer", "Code should have 80% test coverage", &["standards"])?;

    // Share knowledge
    team.share_knowledge("researcher", "coder", "ML techniques")?;

    // Coordinate on a task
    let response = team.coordinate("Build a machine learning system")?;
    println!("Team response:\n{}", response);

    Ok(())
}
```

## Programmable Memory Agent

```lua
-- Agent with programmable memory retrieval
local Agent = {}

function Agent:new(agent_id, db)
    local agent = {
        id = agent_id,
        memory_ns = "agent:" .. agent_id .. ":memory"
    }
    setmetatable(agent, {__index = Agent})
    return agent
end

function Agent:remember(content, importance, tags)
    local id = id()
    store_with_embedding(self.memory_ns, id, content)
    put(self.memory_ns .. ":meta", id, json.encode({
        importance = importance or 0.5,
        tags = tags or {},
        timestamp = now()
    }))
    return id
end

function Agent:recall_smart(query, limit)
    local results = semantic_search(self.memory_ns, query, limit * 3)

    local scored = {}
    for _, r in ipairs(results) do
        local meta = json.decode(get(self.memory_ns .. ":meta", r.id) or '{}')

        local relevance = 1 - r.distance
        local importance = meta.importance or 0.5

        -- Time decay
        local age_seconds = now() - (meta.timestamp or 0)
        local age_days = age_seconds / 86400
        local recency = math.exp(-age_days / 7)  -- Week half-life

        local score = relevance * importance * recency

        table.insert(scored, {
            content = r.content,
            score = score,
            metadata = meta
        })
    end

    table.sort(scored, function(a, b) return a.score > b.score end)

    local top = {}
    for i = 1, math.min(limit, #scored) do
        table.insert(top, scored[i])
    end

    return top
end

function Agent:process_turn(user_message)
    -- Remember this interaction
    self:remember("User: " .. user_message, 0.7, {"interaction"})

    -- Get context
    local context = self:recall_smart(user_message, 5)

    -- Build response data
    return json.encode({
        user_message = user_message,
        context = map(context, function(c) return c.content end),
        context_scores = map(context, function(c) return c.score end)
    })
end

-- Usage
local agent = Agent:new("assistant")
agent:remember("User prefers concise explanations", 0.9, {"preferences"})
agent:remember("User is working on a Rust project", 0.8, {"context"})

local response = agent:process_turn("How should I structure my code?")
return response
```

## Next Steps

- [RAG Applications](rag.md) - Retrieval-augmented generation
- [Multi-Agent Systems](multi-agent.md) - Complex agent architectures
- [Building Agents Guide](../guides/building-agents.md) - Best practices
