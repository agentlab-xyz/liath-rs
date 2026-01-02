# Lua Scripting Guide

Liath uses Lua as its programmable query language. This guide covers all available functions and patterns for writing effective Lua scripts.

## Basics

### Executing Lua Code

```rust
let db = EmbeddedLiath::new(Config::default())?;
let executor = db.query_executor();

let result = executor.execute(r#"
    return "Hello from Lua!"
"#, "agent-id").await?;
```

### Return Values

All Lua scripts must return a value. For complex data, use JSON encoding:

```lua
-- Simple return
return 42

-- String return
return "Hello"

-- Complex data (use JSON)
local data = { name = "Alice", age = 30 }
return json.encode(data)
```

## Storage Functions

### put(namespace, key, value)

Store a key-value pair:

```lua
put("users", "user:1", '{"name": "Alice", "email": "alice@example.com"}')
put("config", "theme", "dark")
```

### get(namespace, key)

Retrieve a value by key:

```lua
local user = get("users", "user:1")
if user then
    local data = json.decode(user)
    return data.name
end
return nil
```

### delete(namespace, key)

Remove a key-value pair:

```lua
delete("users", "user:1")
return "deleted"
```

### keys(namespace, prefix)

List keys with optional prefix:

```lua
-- All keys in namespace
local all = keys("users")

-- Keys with prefix
local user_keys = keys("users", "user:")

return json.encode(user_keys)
```

## Vector Operations

### embed(text)

Generate an embedding vector from text:

```lua
local vector = embed("Hello, world!")
return json.encode({ dimensions = #vector })
```

### store_vector(namespace, id, vector)

Store a vector directly:

```lua
local vec = embed("Some text")
store_vector("embeddings", "doc:1", vec)
```

### store_with_embedding(namespace, id, text)

Store text and automatically generate embedding:

```lua
store_with_embedding("docs", "doc:1", "Introduction to Rust programming")
store_with_embedding("docs", "doc:2", "Python for data science")
store_with_embedding("docs", "doc:3", "JavaScript and web development")
```

### vector_search(namespace, vector, k)

Search using a raw vector:

```lua
local query_vec = embed("programming languages")
local results = vector_search("docs", query_vec, 5)

for _, r in ipairs(results) do
    print(r.id, r.distance)
end
```

### semantic_search(namespace, query, k)

Search using text (embedding generated automatically):

```lua
local results = semantic_search("docs", "systems programming", 3)

local output = {}
for _, r in ipairs(results) do
    table.insert(output, {
        id = r.id,
        content = r.content,
        distance = r.distance
    })
end

return json.encode(output)
```

## Agent Memory Functions

### store_memory(namespace, content, tags)

Store a memory with tags:

```lua
store_memory("agent:memory", "User prefers dark mode", {"preferences", "ui"})
store_memory("agent:memory", "User works with Rust", {"skills", "programming"})
```

### recall(namespace, query, k)

Recall memories semantically:

```lua
local memories = recall("agent:memory", "What does the user program in?", 3)

for _, m in ipairs(memories) do
    print(m.content, m.distance)
end
```

### recall_by_tags(namespace, tags, limit)

Recall memories by tags:

```lua
local prefs = recall_by_tags("agent:memory", {"preferences"}, 10)
return json.encode(prefs)
```

## Conversation Functions

### add_message(conversation_id, role, content)

Add a message to a conversation:

```lua
add_message("conv:1", "user", "Hello!")
add_message("conv:1", "assistant", "Hi! How can I help?")
add_message("conv:1", "user", "Tell me about Rust")
```

### get_messages(conversation_id, limit)

Retrieve recent messages:

```lua
local messages = get_messages("conv:1", 10)

for _, msg in ipairs(messages) do
    print(msg.role .. ": " .. msg.content)
end

return json.encode(messages)
```

### clear_conversation(conversation_id)

Clear all messages:

```lua
clear_conversation("conv:1")
return "cleared"
```

## Utility Functions

### id()

Generate a unique ID:

```lua
local unique_id = id()
store_with_embedding("docs", unique_id, "New document")
```

### now()

Get current Unix timestamp:

```lua
local timestamp = now()
put("meta", "last_access", tostring(timestamp))
```

### json.encode(value)

Encode Lua table to JSON:

```lua
local data = {
    name = "Alice",
    scores = {95, 87, 92},
    active = true
}
return json.encode(data)
```

### json.decode(string)

Decode JSON to Lua table:

```lua
local json_str = get("users", "user:1")
local user = json.decode(json_str)
print(user.name)
```

## Functional Helpers

### map(list, fn)

Transform each element:

```lua
local numbers = {1, 2, 3, 4, 5}
local doubled = map(numbers, function(n) return n * 2 end)
-- {2, 4, 6, 8, 10}

-- With search results
local results = semantic_search("docs", "query", 5)
local contents = map(results, function(r) return r.content end)
```

### filter(list, fn)

Keep elements matching predicate:

```lua
local numbers = {1, 2, 3, 4, 5}
local evens = filter(numbers, function(n) return n % 2 == 0 end)
-- {2, 4}

-- Filter search results
local results = semantic_search("docs", "query", 10)
local relevant = filter(results, function(r) return r.distance < 0.5 end)
```

### reduce(list, fn, initial)

Reduce list to single value:

```lua
local numbers = {1, 2, 3, 4, 5}
local sum = reduce(numbers, function(acc, n) return acc + n end, 0)
-- 15
```

## Advanced Patterns

### Multi-Factor Ranking

```lua
function smart_search(query)
    local results = semantic_search("memory", query, 20)

    local scored = {}
    for _, r in ipairs(results) do
        local meta = json.decode(get("memory:meta", r.id) or '{}')

        -- Calculate composite score
        local relevance = 1 - r.distance
        local importance = meta.importance or 0.5
        local recency = meta.age_days and (meta.age_days <= 7 and 1.5 or 1.0) or 1.0

        local score = relevance * importance * recency

        table.insert(scored, {
            content = r.content,
            score = score
        })
    end

    table.sort(scored, function(a, b) return a.score > b.score end)

    -- Return top 5
    local top = {}
    for i = 1, math.min(5, #scored) do
        table.insert(top, scored[i])
    end

    return top
end

return json.encode(smart_search("user preferences"))
```

### Cross-Referencing

```lua
function build_context(query)
    -- Search multiple sources
    local memories = semantic_search("memories", query, 5)
    local facts = semantic_search("facts", query, 5)
    local examples = semantic_search("examples", query, 3)

    -- Merge results
    local all = {}
    for _, m in ipairs(memories) do
        table.insert(all, {source = "memory", content = m.content, distance = m.distance})
    end
    for _, f in ipairs(facts) do
        table.insert(all, {source = "fact", content = f.content, distance = f.distance})
    end
    for _, e in ipairs(examples) do
        table.insert(all, {source = "example", content = e.content, distance = e.distance})
    end

    -- Sort by relevance
    table.sort(all, function(a, b) return a.distance < b.distance end)

    return all
end

return json.encode(build_context("Rust programming"))
```

### Conversation Loop

```lua
function agent_turn(user_message)
    -- Store interaction as memory
    local mem_id = id()
    store_with_embedding("memory", mem_id, "User: " .. user_message)
    put("memory:meta", mem_id, json.encode({
        timestamp = now(),
        importance = 0.7,
        type = "interaction"
    }))

    -- Get relevant context
    local context = semantic_search("memory", user_message, 5)

    -- Update conversation
    add_message("main", "user", user_message)
    local history = get_messages("main", 10)

    return json.encode({
        context = map(context, function(c) return c.content end),
        history = history
    })
end

return agent_turn("How do I handle errors in Rust?")
```

## Safety Notes

The Lua sandbox blocks all system access:

```lua
-- These will fail safely
os.execute("ls")          -- BLOCKED
io.open("file.txt", "r")  -- BLOCKED
require("socket")         -- BLOCKED
loadfile("script.lua")    -- BLOCKED
```

Only database functions are available, ensuring agent-generated code cannot harm the system.

## Next Steps

- [Lua Standard Library Reference](../api/lua-stdlib.md) - Complete function reference
- [Building AI Agents](building-agents.md) - Agent development patterns
- [Examples](../examples/index.md) - More code examples
