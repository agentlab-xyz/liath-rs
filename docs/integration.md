# Integration Guide

How to use Liath's programmable memory from Python and Rust.

## The Core Pattern

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│      LLM        │────▶│  Generate Lua   │────▶│     Liath       │
│  (Claude/GPT)   │     │   Memory Query  │     │  Execute Safe   │
└─────────────────┘     └─────────────────┘     └─────────────────┘
         │                                              │
         │              ┌─────────────────┐             │
         └──────────────│    Results      │◀────────────┘
                        │  Back to LLM    │
                        └─────────────────┘
```

**The key insight**: The LLM generates Lua code to query memory, and Liath executes it safely.

## Rust Integration

### Basic Usage

```rust
use liath::{EmbeddedLiath, Config};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize Liath
    let liath = EmbeddedLiath::new(Config::default())?;
    let executor = liath.query_executor();

    // Store memories
    executor.execute(r#"
        store_with_embedding("mem", "m1", "User likes Rust")
        store_with_embedding("mem", "m2", "User is building an API")
    "#, "agent").await?;

    // Agent-generated query (this would come from an LLM)
    let agent_code = r#"
        local results = semantic_search("mem", "programming preferences", 5)
        return json.encode(results)
    "#;

    let result = executor.execute(agent_code, "agent").await?;
    println!("Memories: {}", result);

    Ok(())
}
```

### With an LLM (Claude/OpenAI)

```rust
use liath::{EmbeddedLiath, Config};
use anthropic::Anthropic;  // or openai

struct Agent {
    liath: EmbeddedLiath,
    llm: Anthropic,
}

impl Agent {
    async fn think(&self, user_message: &str) -> anyhow::Result<String> {
        let executor = self.liath.query_executor();

        // Step 1: Ask LLM to generate memory query
        let query_prompt = format!(r#"
            Generate Lua code to retrieve relevant memories for this user message.
            Available functions: semantic_search(namespace, query, limit), get(ns, key),
            filter(list, fn), map(list, fn), json.encode(value)

            User message: "{}"

            Return ONLY the Lua code, no explanation.
        "#, user_message);

        let lua_code = self.llm.complete(&query_prompt).await?;

        // Step 2: Execute the agent-generated code safely
        let memories = executor.execute(&lua_code, "agent").await?;

        // Step 3: Build context and get response
        let response_prompt = format!(r#"
            Relevant memories: {}

            User message: {}

            Respond helpfully based on context.
        "#, memories, user_message);

        let response = self.llm.complete(&response_prompt).await?;

        // Step 4: Store this interaction as a new memory
        executor.execute(&format!(r#"
            store_with_embedding("mem", id(), "User asked about: {}")
        "#, user_message.replace('"', r#"\""#)), "agent").await?;

        Ok(response)
    }
}
```

### Full Agent Loop

```rust
use liath::{EmbeddedLiath, Config};
use tokio::io::{AsyncBufReadExt, BufReader};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let liath = EmbeddedLiath::new(Config::default())?;
    let executor = liath.query_executor();

    // Initialize agent memory system
    executor.execute(r#"
        -- Define agent memory functions
        function remember(content, importance)
            local id = id()
            store_with_embedding("agent:mem", id, content)
            put("agent:meta", id, json.encode({
                importance = importance or 0.5,
                timestamp = now()
            }))
            return id
        end

        function recall(query, limit)
            local results = semantic_search("agent:mem", query, limit or 10)
            local enriched = {}
            for _, r in ipairs(results) do
                local meta = json.decode(get("agent:meta", r.id) or "{}")
                table.insert(enriched, {
                    content = r.content,
                    relevance = 1 - r.distance,
                    importance = meta.importance or 0.5
                })
            end
            -- Sort by relevance * importance
            table.sort(enriched, function(a, b)
                return (a.relevance * a.importance) > (b.relevance * b.importance)
            end)
            return enriched
        end

        return "Agent initialized"
    "#, "system").await?;

    println!("Agent ready. Type messages:");

    let stdin = BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();

    while let Some(line) = lines.next_line().await? {
        // Agent processes the message using programmable memory
        let agent_turn = format!(r#"
            -- Store user message
            remember("User said: {}", 0.6)

            -- Recall relevant context
            local context = recall("{}", 5)

            -- Build response context
            return json.encode({{
                memories = map(context, function(m) return m.content end),
                user_input = "{}"
            }})
        "#, line.replace('"', r#"\""#), line.replace('"', r#"\""#), line.replace('"', r#"\""#));

        let result = executor.execute(&agent_turn, "agent").await?;
        println!("Context: {}", result);

        // Here you'd send to LLM and get response...
    }

    Ok(())
}
```

## Python Integration

> **Note**: Python bindings are planned. For now, use the HTTP server or MCP.

### Future PyO3 Bindings (Planned API)

```python
# pip install liath (coming soon)
from liath import Liath

# Initialize
db = Liath("./data")

# Store memories
db.execute("""
    store_with_embedding("mem", "m1", "User prefers Python")
    store_with_embedding("mem", "m2", "User is building ML models")
""")

# Agent generates query (from LLM)
agent_code = """
    local results = semantic_search("mem", "programming", 5)
    -- Custom filtering logic
    local filtered = filter(results, function(r)
        return r.distance < 0.5  -- High relevance only
    end)
    return json.encode(filtered)
"""

result = db.execute(agent_code)
memories = json.loads(result)
```

### LangChain Integration (Planned)

```python
from langchain.memory import LiathMemory
from langchain.llms import Claude

# Liath as LangChain memory backend
memory = LiathMemory(
    data_dir="./agent_data",
    # Key feature: LLM can generate memory queries
    programmable=True
)

llm = Claude()

# The agent can write Lua to query its own memory
agent = ConversationalAgent(
    llm=llm,
    memory=memory,
    # Agent generates Lua for complex retrieval
    memory_query_prompt="""
        Generate Lua code to find relevant memories.
        Available: semantic_search, filter, map, json.encode
        Query: {query}
    """
)

response = agent.run("Help me with my Rust project")
```

### Via HTTP Server (Available Now)

```python
import requests
import json

LIATH_URL = "http://localhost:3000"

def execute_lua(code: str, user_id: str = "agent") -> str:
    """Execute Lua code on Liath server."""
    response = requests.post(
        f"{LIATH_URL}/execute",
        json={"query": code, "user_id": user_id}
    )
    return response.json()["result"]

# Store memories
execute_lua("""
    store_with_embedding("mem", "m1", "User is a data scientist")
    store_with_embedding("mem", "m2", "User uses PyTorch for deep learning")
""")

# Agent-generated query (from LLM)
agent_code = """
    -- Complex retrieval logic generated by LLM
    local results = semantic_search("mem", "machine learning tools", 10)

    -- Filter and rank
    local relevant = filter(results, function(r)
        return r.distance < 0.6
    end)

    -- Format for LLM consumption
    local memories = map(relevant, function(r)
        return r.content
    end)

    return json.encode(memories)
"""

memories = json.loads(execute_lua(agent_code))
print(f"Retrieved memories: {memories}")
```

### Via MCP (For Claude Desktop)

```json
// claude_desktop_config.json
{
  "mcpServers": {
    "liath": {
      "command": "liath",
      "args": ["mcp"]
    }
  }
}
```

Then Claude can use tools like:
- `liath_execute` - Run Lua code (the key feature!)
- `liath_semantic_search` - Direct search
- `liath_store_memory` - Store memories

**Example Claude interaction:**
```
User: Remember that I prefer functional programming

Claude: *uses liath_execute tool*
Lua code: store_with_embedding("mem", id(), "User prefers functional programming")

User: What do you know about my coding preferences?

Claude: *uses liath_execute tool*
Lua code:
  local results = semantic_search("mem", "coding preferences", 10)
  local prefs = filter(results, function(r) return r.distance < 0.5 end)
  return json.encode(map(prefs, function(r) return r.content end))

Result: ["User prefers functional programming"]

Claude: Based on my memory, you prefer functional programming...
```

## The Power of Programmable Memory

### Why This Matters

Traditional vector databases:
```python
# Fixed API - same query structure every time
results = db.search(query="coding preferences", limit=5)
```

Liath programmable memory:
```python
# Agent generates the query logic based on context
agent_code = llm.generate(f"""
    Task: Find relevant memories for "{user_message}"
    Consider: recency, importance, semantic relevance
    Generate Lua code using: semantic_search, filter, map, get
""")

# Agent's custom logic runs safely
results = db.execute(agent_code)
```

### Example: Adaptive Retrieval

The agent can implement different strategies based on the situation:

```lua
-- Strategy 1: Recent + Relevant (for ongoing tasks)
function recent_relevant(query)
    local results = semantic_search("mem", query, 20)
    return filter(results, function(r)
        local meta = json.decode(get("mem:meta", r.id) or "{}")
        return meta.age_days < 7 and r.distance < 0.5
    end)
end

-- Strategy 2: Important memories only (for key decisions)
function important_only(query)
    local results = semantic_search("mem", query, 20)
    return filter(results, function(r)
        local meta = json.decode(get("mem:meta", r.id) or "{}")
        return meta.importance > 0.8
    end)
end

-- Strategy 3: Cross-reference (for complex queries)
function cross_reference(query1, query2)
    local set1 = semantic_search("mem", query1, 10)
    local set2 = semantic_search("mem", query2, 10)

    -- Find memories that match both
    local ids2 = {}
    for _, r in ipairs(set2) do ids2[r.id] = true end

    return filter(set1, function(r) return ids2[r.id] end)
end
```

**The LLM chooses which strategy to use based on the current context.** This is impossible with fixed APIs.

## Summary

| Approach | Query Logic | Safety | Flexibility |
|----------|-------------|--------|-------------|
| Traditional DB | Fixed API | N/A | Low |
| Raw Code Exec | User-defined | ❌ Dangerous | High |
| **Liath** | Agent-generated | ✅ Sandboxed | High |

Liath gives you the flexibility of code execution with the safety of a sandbox.
