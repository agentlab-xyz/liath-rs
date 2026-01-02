# Multi-Agent Systems

Examples for building multi-agent systems with Liath.

## Agent Team Architecture

```rust
use liath::{EmbeddedLiath, Config};
use liath::agent::Agent;
use std::sync::Arc;
use std::collections::HashMap;

pub struct AgentTeam {
    db: Arc<EmbeddedLiath>,
    coordinator: Agent,
    specialists: HashMap<String, Agent>,
}

impl AgentTeam {
    pub fn new(data_dir: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let db = Arc::new(EmbeddedLiath::new(Config {
            data_dir: data_dir.into(),
            ..Default::default()
        })?);

        let coordinator = Agent::new_with_description(
            "coordinator",
            "Orchestrates tasks across specialist agents",
            db.clone()
        );
        coordinator.save()?;

        Ok(Self {
            db,
            coordinator,
            specialists: HashMap::new(),
        })
    }

    pub fn add_specialist(&mut self, role: &str, description: &str) -> Result<(), Box<dyn std::error::Error>> {
        let agent = Agent::new_with_description(
            &format!("specialist:{}", role),
            description,
            self.db.clone()
        );
        agent.save()?;

        self.specialists.insert(role.to_string(), agent);
        Ok(())
    }

    pub fn assign_task(&self, role: &str, task: &str) -> Result<(), Box<dyn std::error::Error>> {
        let agent = self.specialists.get(role)
            .ok_or("Specialist not found")?;

        let memory = agent.memory()?;
        memory.store(
            &format!("Task assigned: {}", task),
            &["tasks", "pending"]
        )?;

        // Log in coordinator
        let coord_memory = self.coordinator.memory()?;
        coord_memory.store(
            &format!("Assigned to {}: {}", role, task),
            &["delegation", role]
        )?;

        Ok(())
    }

    pub fn share_knowledge(&self, from: &str, to: &str, topic: &str) -> Result<usize, Box<dyn std::error::Error>> {
        let from_agent = self.specialists.get(from)
            .ok_or("Source agent not found")?;
        let to_agent = self.specialists.get(to)
            .ok_or("Target agent not found")?;

        let from_memory = from_agent.memory()?;
        let knowledge = from_memory.recall(topic, 5)?;

        let to_memory = to_agent.memory()?;
        let mut shared = 0;

        for entry in knowledge {
            to_memory.store(
                &format!("[Shared from {}] {}", from, entry.content),
                &["shared", from]
            )?;
            shared += 1;
        }

        Ok(shared)
    }

    pub fn coordinate(&self, task: &str) -> Result<Vec<(String, Vec<String>)>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();

        for (role, agent) in &self.specialists {
            let memory = agent.memory()?;
            let relevant = memory.recall(task, 3)?;

            if !relevant.is_empty() {
                results.push((
                    role.clone(),
                    relevant.into_iter().map(|r| r.content).collect()
                ));
            }
        }

        Ok(results)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut team = AgentTeam::new("./team_data")?;

    // Create specialists
    team.add_specialist("researcher", "Gathers and analyzes information")?;
    team.add_specialist("developer", "Writes and reviews code")?;
    team.add_specialist("reviewer", "Quality assurance and feedback")?;

    // Assign knowledge
    if let Some(researcher) = team.specialists.get("researcher") {
        let mem = researcher.memory()?;
        mem.store("Transformer architecture dominates NLP", &["ml", "research"])?;
        mem.store("RAG improves factual accuracy", &["ml", "techniques"])?;
    }

    // Coordinate on task
    let responses = team.coordinate("Build an NLP system")?;
    for (role, knowledge) in responses {
        println!("{}: {:?}", role, knowledge);
    }

    Ok(())
}
```

## Agent Communication Protocol

```lua
-- Message-based agent communication
local MessageBus = {}

function MessageBus:new()
    local bus = {
        queues = {}
    }
    setmetatable(bus, {__index = MessageBus})
    return bus
end

function MessageBus:send(from, to, message_type, content)
    local msg_id = id()
    local message = {
        id = msg_id,
        from = from,
        to = to,
        type = message_type,
        content = content,
        timestamp = now(),
        status = "pending"
    }

    -- Store in recipient's queue
    local queue_key = "agent:" .. to .. ":inbox"
    local queue = json.decode(get("messages", queue_key) or "[]")
    table.insert(queue, message)
    put("messages", queue_key, json.encode(queue))

    return msg_id
end

function MessageBus:receive(agent_id, count)
    local queue_key = "agent:" .. agent_id .. ":inbox"
    local queue = json.decode(get("messages", queue_key) or "[]")

    local messages = {}
    for i = 1, math.min(count, #queue) do
        table.insert(messages, queue[i])
    end

    return messages
end

function MessageBus:acknowledge(agent_id, msg_id)
    local queue_key = "agent:" .. agent_id .. ":inbox"
    local queue = json.decode(get("messages", queue_key) or "[]")

    local new_queue = filter(queue, function(m)
        return m.id ~= msg_id
    end)

    put("messages", queue_key, json.encode(new_queue))
end

-- Agent with message handling
local Agent = {}

function Agent:new(agent_id, bus)
    local agent = {
        id = agent_id,
        bus = bus,
        handlers = {}
    }
    setmetatable(agent, {__index = Agent})
    return agent
end

function Agent:on(message_type, handler)
    self.handlers[message_type] = handler
end

function Agent:send(to, message_type, content)
    return self.bus:send(self.id, to, message_type, content)
end

function Agent:process_messages()
    local messages = self.bus:receive(self.id, 10)

    for _, msg in ipairs(messages) do
        local handler = self.handlers[msg.type]
        if handler then
            handler(msg)
        end
        self.bus:acknowledge(self.id, msg.id)
    end

    return #messages
end

-- Usage
local bus = MessageBus:new()

local researcher = Agent:new("researcher", bus)
local developer = Agent:new("developer", bus)

-- Set up handlers
developer:on("task", function(msg)
    print("Developer received task:", msg.content)
end)

developer:on("review_request", function(msg)
    print("Developer reviewing:", msg.content)
end)

-- Send messages
researcher:send("developer", "task", "Implement search algorithm")
researcher:send("developer", "review_request", "Check ML model accuracy")

-- Process
local processed = developer:process_messages()
return "Processed " .. processed .. " messages"
```

## Hierarchical Agent System

```rust
use liath::{EmbeddedLiath, Config};
use liath::agent::Agent;
use std::sync::Arc;

pub struct HierarchicalAgents {
    db: Arc<EmbeddedLiath>,
    root: Agent,
    children: Vec<HierarchicalAgents>,
}

impl HierarchicalAgents {
    pub fn new_root(db: Arc<EmbeddedLiath>, id: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let root = Agent::new(id, db.clone());
        root.save()?;

        Ok(Self {
            db,
            root,
            children: Vec::new(),
        })
    }

    pub fn add_child(&mut self, id: &str) -> Result<&mut HierarchicalAgents, Box<dyn std::error::Error>> {
        let child = HierarchicalAgents::new_root(self.db.clone(), id)?;
        self.children.push(child);
        Ok(self.children.last_mut().unwrap())
    }

    pub fn propagate_down(&self, knowledge: &str, tags: &[&str]) -> Result<usize, Box<dyn std::error::Error>> {
        let mut count = 0;

        // Store in self
        let memory = self.root.memory()?;
        memory.store(knowledge, tags)?;
        count += 1;

        // Propagate to children
        for child in &self.children {
            count += child.propagate_down(knowledge, tags)?;
        }

        Ok(count)
    }

    pub fn aggregate_up(&self, query: &str) -> Result<Vec<(String, Vec<String>)>, Box<dyn std::error::Error>> {
        let mut results = Vec::new();

        // Get from self
        let memory = self.root.memory()?;
        let own_results = memory.recall(query, 3)?;
        if !own_results.is_empty() {
            results.push((
                self.root.id().to_string(),
                own_results.into_iter().map(|r| r.content).collect()
            ));
        }

        // Aggregate from children
        for child in &self.children {
            let child_results = child.aggregate_up(query)?;
            results.extend(child_results);
        }

        Ok(results)
    }
}
```

## Collaborative Task Execution

```lua
-- Collaborative task execution with multiple agents
local TaskManager = {}

function TaskManager:new()
    local tm = {
        tasks = {},
        agents = {}
    }
    setmetatable(tm, {__index = TaskManager})
    return tm
end

function TaskManager:register_agent(agent_id, capabilities)
    self.agents[agent_id] = {
        id = agent_id,
        capabilities = capabilities,
        current_task = nil
    }

    -- Store capabilities in agent memory
    for _, cap in ipairs(capabilities) do
        store_memory("agent:" .. agent_id .. ":memory",
            "I am capable of: " .. cap,
            {"capability", cap})
    end
end

function TaskManager:create_task(task_id, description, required_capabilities)
    self.tasks[task_id] = {
        id = task_id,
        description = description,
        required = required_capabilities,
        assigned = {},
        status = "pending",
        results = {}
    }
end

function TaskManager:assign_task(task_id)
    local task = self.tasks[task_id]
    if not task then return nil, "Task not found" end

    -- Find agents for each capability
    for _, cap in ipairs(task.required) do
        for agent_id, agent in pairs(self.agents) do
            if not agent.current_task then
                for _, agent_cap in ipairs(agent.capabilities) do
                    if agent_cap == cap then
                        agent.current_task = task_id
                        table.insert(task.assigned, {
                            agent = agent_id,
                            capability = cap
                        })
                        break
                    end
                end
            end
        end
    end

    task.status = "in_progress"
    return task
end

function TaskManager:submit_result(task_id, agent_id, result)
    local task = self.tasks[task_id]
    if not task then return nil, "Task not found" end

    table.insert(task.results, {
        agent = agent_id,
        result = result,
        timestamp = now()
    })

    -- Free up agent
    local agent = self.agents[agent_id]
    if agent then
        agent.current_task = nil
    end

    -- Check if all assigned agents have submitted
    if #task.results >= #task.assigned then
        task.status = "completed"
    end

    return task
end

function TaskManager:get_combined_result(task_id)
    local task = self.tasks[task_id]
    if not task then return nil end

    local combined = {}
    for _, r in ipairs(task.results) do
        table.insert(combined, {
            agent = r.agent,
            contribution = r.result
        })
    end

    return {
        task = task.description,
        status = task.status,
        contributions = combined
    }
end

-- Usage
local tm = TaskManager:new()

-- Register agents
tm:register_agent("researcher", {"research", "analysis"})
tm:register_agent("developer", {"coding", "testing"})
tm:register_agent("writer", {"documentation", "review"})

-- Create task
tm:create_task("build-feature",
    "Build a new search feature",
    {"research", "coding", "documentation"})

-- Assign
tm:assign_task("build-feature")

-- Submit results
tm:submit_result("build-feature", "researcher", "Found best algorithm is BM25 + vector hybrid")
tm:submit_result("build-feature", "developer", "Implemented in Rust with async support")
tm:submit_result("build-feature", "writer", "Created API documentation")

-- Get combined result
return json.encode(tm:get_combined_result("build-feature"))
```

## Agent Consensus Protocol

```lua
-- Agents reach consensus through voting
local ConsensusProtocol = {}

function ConsensusProtocol:new(agents)
    local cp = {
        agents = agents,
        proposals = {}
    }
    setmetatable(cp, {__index = ConsensusProtocol})
    return cp
end

function ConsensusProtocol:propose(proposal_id, content)
    self.proposals[proposal_id] = {
        id = proposal_id,
        content = content,
        votes = {},
        status = "voting"
    }
end

function ConsensusProtocol:vote(proposal_id, agent_id, vote, reason)
    local proposal = self.proposals[proposal_id]
    if not proposal then return nil, "Proposal not found" end

    proposal.votes[agent_id] = {
        vote = vote,  -- "approve", "reject", "abstain"
        reason = reason
    }
end

function ConsensusProtocol:tally(proposal_id)
    local proposal = self.proposals[proposal_id]
    if not proposal then return nil end

    local approve = 0
    local reject = 0
    local abstain = 0

    for _, v in pairs(proposal.votes) do
        if v.vote == "approve" then
            approve = approve + 1
        elseif v.vote == "reject" then
            reject = reject + 1
        else
            abstain = abstain + 1
        end
    end

    local total = approve + reject + abstain
    local threshold = total / 2

    if approve > threshold then
        proposal.status = "approved"
    elseif reject > threshold then
        proposal.status = "rejected"
    else
        proposal.status = "no_consensus"
    end

    return {
        proposal = proposal.content,
        status = proposal.status,
        votes = {
            approve = approve,
            reject = reject,
            abstain = abstain
        }
    }
end

-- Usage
local agents = {"agent1", "agent2", "agent3", "agent4", "agent5"}
local consensus = ConsensusProtocol:new(agents)

-- Create proposal
consensus:propose("use-rust", "Should we use Rust for the new service?")

-- Agents vote
consensus:vote("use-rust", "agent1", "approve", "Memory safety is crucial")
consensus:vote("use-rust", "agent2", "approve", "Performance benefits")
consensus:vote("use-rust", "agent3", "reject", "Team lacks experience")
consensus:vote("use-rust", "agent4", "approve", "Good ecosystem")
consensus:vote("use-rust", "agent5", "abstain", "Need more information")

-- Tally
return json.encode(consensus:tally("use-rust"))
```

## Next Steps

- [Agent Patterns](agent-patterns.md) - Single agent patterns
- [Building Agents Guide](../guides/building-agents.md) - Best practices
- [Memory Patterns](../guides/memory-patterns.md) - Memory organization
