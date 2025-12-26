# Liath

**The SQLite for AI Agents** - Programmable memory that agents can query with code.

## The Key Idea

**Agents can write programs to query their own memory safely.**

```
┌────────────────────────────────────────────────────────────────┐
│                    Traditional Vector DB                        │
│  Agent → Fixed API → semantic_search("query", 5) → Results     │
└────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────┐
│                    Liath (Programmable Memory)                  │
│  Agent → Generates Lua Code → Complex Logic → Results          │
│                                                                 │
│  The LLM writes code like:                                      │
│    local results = semantic_search("mem", query, 20)            │
│    local recent = filter(results, function(r)                   │
│        return r.age_days < 7 and r.importance > 0.8             │
│    end)                                                         │
│    return json.encode(top(recent, 5))                           │
│                                                                 │
│  Liath executes it SAFELY (sandboxed, no system access)         │
└────────────────────────────────────────────────────────────────┘
```

**Why this matters:**
- LLMs can implement custom retrieval strategies
- Agents adapt their memory queries to the current task
- Complex filtering, ranking, cross-referencing - all possible
- But it's SAFE - Lua sandbox blocks file/network/system access

## Architecture

```
┌─────────────────────────────────────────────┐
│                   Liath                      │
│                                              │
│  ┌─────────────────────────────────────┐    │
│  │           Lua Runtime               │    │
│  │   Safe execution for agent logic    │    │
│  └─────────────────────────────────────┘    │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐    │
│  │ KV Store │ │ Vectors  │ │Embeddings│    │
│  │  (Fjall) │ │(USearch) │ │(FastEmbed│    │
│  └──────────┘ └──────────┘ └──────────┘    │
└─────────────────────────────────────────────┘
```

## Why Liath?

| Problem | Liath Solution |
|---------|----------------|
| Fixed APIs limit retrieval strategies | **Programmable** - agents write query logic |
| Agent code execution is dangerous | **Sandboxed Lua** - no system access |
| Vector DBs require servers | **Embedded** - no infrastructure |
| Memory systems are Python-only | **Rust core** - fast, portable |
| Complex setup for AI storage | **Single dependency** - zero config |

## Quick Example

```rust
use liath::{EmbeddedLiath, Config};

fn main() -> anyhow::Result<()> {
    let liath = EmbeddedLiath::new(Config::default())?;
    let executor = liath.query_executor();

    // Store memories and execute agent logic - all in Lua
    let result = executor.execute(r#"
        -- Store a memory with semantic indexing
        store_with_embedding("memories", "m1", "User prefers dark mode")
        store_with_embedding("memories", "m2", "User is learning Rust")
        store_with_embedding("memories", "m3", "User likes coffee")

        -- Semantic recall
        local memories = semantic_search("memories", "programming", 2)

        -- Transform results (agent logic runs safely in sandbox)
        local summary = {}
        for _, m in ipairs(memories) do
            table.insert(summary, m.content)
        end
        return table.concat(summary, "; ")
    "#, "agent-1").await?;

    println!("{}", result);  // "User is learning Rust; ..."
    Ok(())
}
```

## Core Concepts

### 1. Memory (Storage)

Liath provides three storage primitives:

```lua
-- Key-Value: Fast, typed storage
put("config", "theme", "dark")
local theme = get("config", "theme")

-- Vectors: Similarity search
store_vector("docs", "doc1", {0.1, 0.2, ...})
local similar = vector_search("docs", query_vec, 10)

-- Semantic: Text → embedding → vector (automatic)
store_with_embedding("notes", "n1", "Meeting notes from Monday")
local relevant = semantic_search("notes", "weekly meetings", 5)
```

### 2. Runtime (Lua Scripting)

Agents need to execute logic safely. Liath's Lua runtime is:

- **Sandboxed**: No file system, no network, no system calls
- **Fast**: Low-latency execution for real-time agents
- **Simple**: LLMs can generate Lua easily
- **Integrated**: Direct access to storage primitives

```lua
-- Complex retrieval logic that APIs can't express
function find_relevant_context(query, user_id)
    -- Semantic search
    local memories = semantic_search("memories", query, 20)

    -- Filter by recency
    local dominated = filter(memories, function(m)
        return m.timestamp > now() - 86400  -- Last 24 hours
    end)

    -- Boost by importance
    local scored = map(recent, function(m)
        m.score = m.similarity * m.importance
        return m
    end)

    -- Return top results
    return sort_by(scored, "score"):take(5)
end
```

### 3. Agent Primitives

Built-in support for agent workflows:

```lua
-- Conversation management
add_message("conv-123", "user", "Hello!")
add_message("conv-123", "assistant", "Hi there!")
local history = get_messages("conv-123", 10)

-- Tool state tracking
set_tool_state("browser", "current_url", "https://example.com")
local url = get_tool_state("browser", "current_url")

-- Memory with tags
store_memory("agent-1", {
    content = "User mentioned they have a dog named Max",
    tags = {"user-preference", "pets"},
    importance = 0.8
})
local pet_memories = recall_by_tags("agent-1", {"pets"}, 5)
```

## Features

| Category | Features |
|----------|----------|
| **Storage** | Namespaced KV, persistence, atomic operations |
| **Vectors** | HNSW index, cosine/euclidean distance, batch ops |
| **Embeddings** | Built-in FastEmbed, automatic text→vector |
| **Scripting** | Lua 5.4, sandboxed, stdlib for agents |
| **Agent API** | Memory, conversations, tool state |
| **Interfaces** | TUI, CLI, HTTP API, MCP server |

### Feature Flags

| Flag | Default | Description |
|------|---------|-------------|
| `embedding` | on | FastEmbed/ONNX for text embeddings |
| `vector` | on | USearch for vector similarity |
| `tui` | on | Interactive terminal interface |
| `server` | off | HTTP API server |
| `mcp` | off | MCP server for AI assistants |

## Installation

### As a Library

```toml
[dependencies]
liath = "0.1"
```

### As a Binary

```bash
git clone https://github.com/nudgelang/liath-rs.git
cd liath-rs
cargo install --path .
```

## Usage

### Embedded (Library)

```rust
use liath::{EmbeddedLiath, Config};

let liath = EmbeddedLiath::new(Config::default())?;

// Direct API
liath.put("ns", b"key", b"value")?;
liath.store_with_embedding("ns", "id", "text content")?;
let results = liath.semantic_search("ns", "query", 10)?;

// Lua execution
let executor = liath.query_executor();
let result = executor.execute("return get('ns', 'key')", "user").await?;
```

### TUI Console

```bash
liath              # Start TUI
liath cli --simple # Simple readline mode
```

### HTTP Server

```bash
liath server --port 3000
```

```bash
# Store with embedding
curl -X POST localhost:3000/semantic/memories \
  -H 'content-type: application/json' \
  -d '{"id": "m1", "content": "User likes Python"}'

# Semantic search
curl -X POST localhost:3000/semantic/memories/search \
  -H 'content-type: application/json' \
  -d '{"query": "programming preferences", "limit": 5}'

# Execute Lua
curl -X POST localhost:3000/execute \
  -H 'content-type: application/json' \
  -d '{"query": "return semantic_search(\"memories\", \"coding\", 3)", "user_id": "agent"}'
```

### MCP Server

For AI assistant integration (Claude, etc.):

```bash
liath mcp
```

Available tools:
- `liath_put`, `liath_get`, `liath_delete` - KV operations
- `liath_semantic_search` - Text similarity search
- `liath_execute` - Run Lua code safely
- `liath_store_memory`, `liath_recall_memory` - Agent memory

## Lua Standard Library

Liath extends Lua with agent-focused functions:

### Storage
```lua
put(namespace, key, value)           -- Store value
get(namespace, key) -> value         -- Retrieve value
delete(namespace, key)               -- Delete key
keys(namespace, prefix) -> list      -- List keys
```

### Vectors & Embeddings
```lua
embed(text) -> vector                -- Generate embedding
store_vector(ns, id, vector)         -- Store vector
vector_search(ns, vector, k) -> list -- Similarity search
store_with_embedding(ns, id, text)   -- Text → embed → store
semantic_search(ns, query, k) -> list -- Text similarity search
```

### Agent Memory
```lua
store_memory(agent, {content, tags, importance})
recall(agent, query, k) -> memories
recall_by_tags(agent, tags, k) -> memories
```

### Conversations
```lua
add_message(conv_id, role, content)
get_messages(conv_id, limit) -> messages
clear_conversation(conv_id)
```

### Utilities
```lua
now() -> timestamp                   -- Current Unix timestamp
id() -> string                       -- Generate unique ID
json.encode(table) -> string         -- Serialize to JSON
json.decode(string) -> table         -- Parse JSON
map(list, fn) -> list                -- Transform list
filter(list, fn) -> list             -- Filter list
reduce(list, fn, init) -> value      -- Reduce list
```

## Examples

### Agent with Long-Term Memory

```lua
-- Store experiences as they happen
function remember(agent_id, experience)
    store_memory(agent_id, {
        content = experience,
        tags = extract_tags(experience),
        importance = assess_importance(experience),
        timestamp = now()
    })
end

-- Recall relevant context for a task
function get_context(agent_id, task)
    local semantic = recall(agent_id, task, 10)
    local recent = recall_recent(agent_id, 5)
    return merge_and_rank(semantic, recent)
end
```

### Safe Code Execution for Tools

```lua
-- Agent can execute this safely - no system access
function analyze_data(data_json)
    local data = json.decode(data_json)
    local total = reduce(data, function(acc, item)
        return acc + item.value
    end, 0)
    return json.encode({
        count = #data,
        total = total,
        average = total / #data
    })
end
```

### Conversation with Context Injection

```lua
function chat(conv_id, user_message)
    -- Store user message
    add_message(conv_id, "user", user_message)

    -- Get relevant memories
    local context = semantic_search("knowledge", user_message, 3)

    -- Get conversation history
    local history = get_messages(conv_id, 10)

    -- Return context for LLM
    return {
        history = history,
        context = context,
        user_message = user_message
    }
end
```

## Architecture

```
liath-rs/
├── src/
│   ├── lib.rs           # Public API
│   ├── core/            # Storage engine (Fjall)
│   ├── lua/             # Lua runtime & stdlib
│   ├── agent/           # Agent primitives
│   ├── query/           # Query executor
│   ├── cli/             # TUI & console
│   ├── server/          # HTTP API
│   └── mcp/             # MCP server
├── examples/
│   ├── embedded.rs      # Basic usage
│   ├── vector_search.rs # Similarity search
│   ├── agent_usage.rs   # Agent memory
│   ├── agent_runtime.rs # Lua scripting
│   └── lua_scripting.rs # Advanced Lua
└── lua/
    └── examples/        # Lua script examples
```

## Comparison

| Feature | Liath | Chroma | Qdrant | Zep | SQLite |
|---------|-------|--------|--------|-----|--------|
| Embedded (no server) | ✅ | ⚠️ | ❌ | ❌ | ✅ |
| Vector search | ✅ | ✅ | ✅ | ✅ | ❌ |
| Built-in embeddings | ✅ | ✅ | ❌ | ✅ | ❌ |
| Agent memory API | ✅ | ❌ | ❌ | ✅ | ❌ |
| Safe code execution | ✅ Lua | ❌ | ❌ | ❌ | ❌ |
| Rust native | ✅ | ❌ | ✅ | ❌ | ❌ |

## Documentation

- [Guide](docs/guide.md) - Getting started
- [Architecture](docs/architecture.md) - System design
- [Lua Reference](docs/lua-reference.md) - Complete Lua API
- [Status](docs/status.md) - Project status

## License

MIT
