# Guide

Liath is the **SQLite for AI Agents** - an embedded database with a safe Lua runtime. This guide covers installation, basic usage, and agent patterns.

## Installation

### Build from Source

```bash
git clone https://github.com/nudgelang/liath-rs.git
cd liath-rs
cargo build --release
```

### Feature Flags

| Flag | Default | Use Case |
|------|---------|----------|
| `embedding` | on | Text → vector embedding |
| `vector` | on | Similarity search |
| `tui` | on | Interactive terminal |
| `server` | off | HTTP API |
| `mcp` | off | AI assistant integration |

```bash
# Default (embedding + vector + tui)
cargo build

# With HTTP server
cargo build --features server

# With MCP server
cargo build --features mcp

# All features
cargo build --all-features

# Minimal (just KV + Lua)
cargo build --no-default-features
```

## Quick Start

### 1. Start the TUI

```bash
cargo run --bin liath
```

You'll see an interactive terminal. Press `i` to enter insert mode, type a query, press Enter.

### 2. Try Basic Operations

```lua
-- Store and retrieve
put("test", "hello", "world")
return get("test", "hello")
```

### 3. Try Semantic Search

```lua
-- Store documents with automatic embedding
store_with_embedding("docs", "d1", "Rust is a systems programming language")
store_with_embedding("docs", "d2", "Python is great for machine learning")
store_with_embedding("docs", "d3", "JavaScript runs in browsers")

-- Search by meaning
return semantic_search("docs", "programming languages for ML", 2)
```

## Core Concepts

### Memory: Three Storage Layers

```
┌─────────────────────────────────────────────────────┐
│                    Storage Layers                    │
├─────────────────────────────────────────────────────┤
│  Key-Value    Fast, exact lookup          O(1)      │
│  Vector       Similarity search           O(log n)  │
│  Semantic     Text → embed → search       O(log n)  │
└─────────────────────────────────────────────────────┘
```

**Key-Value**: Use for configuration, state, structured data
```lua
put("config", "model", "gpt-4")
put("users", "user:123", '{"name": "Alice"}')
```

**Vector**: Use when you have pre-computed embeddings
```lua
store_vector("images", "img-1", image_embedding)
local similar = vector_search("images", query_embedding, 10)
```

**Semantic**: Use for text similarity (most common for agents)
```lua
store_with_embedding("memories", "m1", "User likes coffee")
local relevant = semantic_search("memories", "beverage preferences", 5)
```

### Runtime: Safe Lua Execution

Liath's Lua runtime lets agents execute code safely:

```lua
-- This is SAFE - runs in sandbox
function process_data(input)
    local data = json.decode(input)
    return json.encode({
        count = #data,
        sum = reduce(data, function(a, b) return a + b end, 0)
    })
end

-- These are BLOCKED - no system access
os.execute("rm -rf /")     -- ERROR: disabled
io.open("/etc/passwd")     -- ERROR: disabled
require("socket")          -- ERROR: disabled
```

**Why Lua?**
- Sandboxed by design (safe for agent code)
- Simple syntax (LLMs can generate it)
- Fast execution (low latency)
- Embeddable (fits the "embedded database" philosophy)

## Agent Patterns

### Pattern 1: Memory with Semantic Recall

```lua
-- Store memories as experiences happen
function remember(content, importance)
    store_memory("my-agent", {
        content = content,
        importance = importance or 0.5,
        tags = {}
    })
end

-- Recall relevant memories for a task
function recall_for_task(task)
    return recall("my-agent", task, 10)
end

-- Example usage
remember("User prefers concise answers", 0.9)
remember("User is working on a Rust project", 0.7)
remember("User mentioned they like coffee", 0.3)

return recall_for_task("how should I format my response?")
```

### Pattern 2: Conversation with Context

```lua
function chat(user_message)
    local conv_id = "main"

    -- Store the message
    add_message(conv_id, "user", user_message)

    -- Get relevant context from memory
    local context = semantic_search("knowledge", user_message, 3)

    -- Get recent conversation history
    local history = get_messages(conv_id, 10)

    -- Return everything the LLM needs
    return json.encode({
        context = map(context, function(c) return c.content end),
        history = history,
        query = user_message
    })
end
```

### Pattern 3: Tool State Management

```lua
-- Browser tool state
function browser_navigate(url)
    set_tool_state("browser", "url", url)
    set_tool_state("browser", "navigated_at", tostring(now()))

    -- Store in history
    local history = json.decode(get_tool_state("browser", "history") or "[]")
    table.insert(history, url)
    set_tool_state("browser", "history", json.encode(history))
end

function browser_get_state()
    return {
        url = get_tool_state("browser", "url"),
        history = json.decode(get_tool_state("browser", "history") or "[]")
    }
end
```

### Pattern 4: RAG Pipeline

```lua
-- Index documents
function index_docs(docs)
    for _, doc in ipairs(docs) do
        store_with_embedding("corpus", doc.id, doc.content)
        put("corpus:meta", doc.id, json.encode({
            title = doc.title,
            source = doc.source,
            indexed = now()
        }))
    end
end

-- Retrieve relevant chunks
function retrieve(query, k)
    local results = semantic_search("corpus", query, k)

    return map(results, function(r)
        local meta = json.decode(get("corpus:meta", r.id) or "{}")
        return {
            content = r.content,
            title = meta.title,
            relevance = 1 - r.distance
        }
    end)
end

-- Generate context string for LLM
function get_rag_context(query)
    local chunks = retrieve(query, 5)
    local context = {}
    for i, chunk in ipairs(chunks) do
        table.insert(context, string.format(
            "[%d] %s\n%s",
            i, chunk.title or "Untitled", chunk.content
        ))
    end
    return table.concat(context, "\n\n")
end
```

### Pattern 5: Multi-Agent State

```lua
-- Each agent has isolated memory
function agent_think(agent_id, observation)
    -- Store observation
    remember(agent_id, "Observed: " .. observation, 0.6)

    -- Get agent's recent memories
    local context = recall(agent_id, observation, 5)

    -- Get agent's conversation
    local conv_id = "agent:" .. agent_id
    local history = get_messages(conv_id, 10)

    return {
        agent = agent_id,
        context = context,
        history = history,
        observation = observation
    }
end

-- Share information between agents
function share_knowledge(from_agent, to_agent, content)
    store_memory(to_agent, {
        content = "From " .. from_agent .. ": " .. content,
        tags = {"shared", from_agent},
        importance = 0.7
    })
end
```

## Interfaces

### TUI Console

```bash
liath
```

| Key | Mode | Action |
|-----|------|--------|
| `i` | Normal | Enter insert mode |
| `Esc` | Insert | Exit to normal mode |
| `Enter` | Insert | Execute query |
| `Up/Down` | Insert | History navigation |
| `PageUp/Down` | Normal | Scroll results |
| `q` | Normal | Quit |

### Simple CLI

```bash
liath cli --simple
```

Helper commands:
- `:ns list` - List namespaces
- `:ns create <name> <dims> <metric> <scalar>` - Create namespace
- `:put <ns> <key> <value>` - Store value
- `:get <ns> <key>` - Get value
- `:del <ns> <key>` - Delete key

### HTTP Server

```bash
liath server --port 3000
```

**Endpoints:**

```bash
# Health
curl localhost:3000/health

# KV Operations
curl localhost:3000/kv/myns/mykey
curl -X PUT localhost:3000/kv/myns/mykey -d '{"value": "hello"}'
curl -X DELETE localhost:3000/kv/myns/mykey

# Semantic Search
curl -X POST localhost:3000/semantic/myns/search \
  -H 'content-type: application/json' \
  -d '{"query": "search text", "limit": 5}'

# Execute Lua
curl -X POST localhost:3000/execute \
  -H 'content-type: application/json' \
  -d '{"query": "return 1 + 1", "user_id": "test"}'
```

### MCP Server

```bash
liath mcp
```

Provides tools for AI assistants:
- `liath_put`, `liath_get`, `liath_delete`
- `liath_semantic_search`
- `liath_execute` (run Lua safely)
- `liath_store_memory`, `liath_recall_memory`

## Library Integration

### Basic Rust Usage

```rust
use liath::{EmbeddedLiath, Config};

fn main() -> anyhow::Result<()> {
    // Create instance
    let config = Config {
        data_dir: "./my-data".into(),
        ..Default::default()
    };
    let liath = EmbeddedLiath::new(config)?;

    // Direct API
    liath.put("ns", b"key", b"value")?;
    let value = liath.get("ns", b"key")?;

    // Lua execution
    let executor = liath.query_executor();
    let result = executor.execute(r#"
        store_with_embedding("docs", "d1", "Hello world")
        return semantic_search("docs", "greeting", 1)
    "#, "my-app").await?;

    println!("{}", result);
    Ok(())
}
```

### Agent Application

```rust
use liath::{EmbeddedLiath, Config};

async fn run_agent() -> anyhow::Result<()> {
    let liath = EmbeddedLiath::new(Config::default())?;
    let executor = liath.query_executor();

    // Define agent logic in Lua
    let agent_code = r#"
        -- Store a memory
        store_memory("agent-1", {
            content = "User asked about the weather",
            importance = 0.5
        })

        -- Process user input
        function handle_input(input)
            add_message("conv", "user", input)
            local context = recall("agent-1", input, 5)
            local history = get_messages("conv", 10)
            return json.encode({context = context, history = history})
        end

        return handle_input("What's the weather like?")
    "#;

    let result = executor.execute(agent_code, "agent-1").await?;
    println!("Agent context: {}", result);

    Ok(())
}
```

## Best Practices

### 1. Namespace Organization

```lua
-- Good: Clear namespace hierarchy
put("users:profiles", user_id, profile_json)
put("users:settings", user_id, settings_json)
store_with_embedding("users:memories", mem_id, memory_text)

-- Avoid: Flat, unclear naming
put("data", "user123profile", ...)
```

### 2. Memory Importance Scoring

```lua
-- Score by relevance to agent's goals
local importance_rules = {
    user_preference = 0.9,   -- High: affects all interactions
    task_context = 0.7,      -- Medium: relevant to current work
    casual_mention = 0.3     -- Low: nice to know
}
```

### 3. Efficient Retrieval

```lua
-- Good: Limit results, filter in Lua
local results = semantic_search("docs", query, 20)
local filtered = filter(results, function(r)
    return r.distance < 0.5  -- Only high similarity
end)
return slice(filtered, 1, 5)

-- Avoid: Fetching everything
local all = semantic_search("docs", query, 1000)  -- Too many!
```

### 4. Conversation Management

```lua
-- Summarize old conversations to save space
function maybe_summarize(conv_id)
    local messages = get_messages(conv_id, 100)
    if #messages > 50 then
        local summary = summarize_messages(messages)
        clear_conversation(conv_id)
        add_message(conv_id, "system", "Previous summary: " .. summary)
    end
end
```

## Next Steps

- [Lua Reference](lua-reference.md) - Complete API documentation
- [Architecture](architecture.md) - System internals
- [Examples](../examples/) - Rust code examples
- [Lua Examples](../lua/examples/) - Lua script examples
