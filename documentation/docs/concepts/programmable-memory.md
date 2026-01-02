# Programmable Memory

Programmable Memory is the core innovation of Liath. Instead of providing fixed APIs that agents must work within, Liath allows agents to **write programs** to query their own memory.

## The Problem with Traditional Approaches

Traditional vector databases provide fixed APIs:

```python
# Traditional approach - limited to pre-defined operations
results = db.semantic_search(query="user preferences", limit=5)
```

This approach has fundamental limitations:

1. **Fixed ranking logic** - You can only use the similarity scores provided
2. **No cross-referencing** - Can't combine data from multiple sources
3. **Static filtering** - Limited to pre-built filter options
4. **One-size-fits-all** - Same retrieval for every use case

## The Liath Solution

Liath gives agents the ability to write Lua code that implements custom retrieval strategies:

```lua
-- Agent-generated code for smart retrieval
local function smart_recall(query)
    -- 1. Get semantically relevant memories
    local results = semantic_search("memory", query, 10)

    -- 2. Enrich with metadata
    local enriched = {}
    for _, r in ipairs(results) do
        local meta = json.decode(get("meta", r.id) or '{}')

        -- 3. Calculate composite score
        local recency_boost = meta.age_days <= 7 and 2.0 or 1.0
        local relevance = 1 - r.distance
        local score = relevance * (meta.importance or 0.5) * recency_boost

        table.insert(enriched, {
            content = r.content,
            score = score,
            reasoning = string.format(
                "relevance=%.0f%%, importance=%.0f%%, age=%d days",
                relevance * 100,
                (meta.importance or 0.5) * 100,
                meta.age_days or 0
            )
        })
    end

    -- 4. Sort by composite score
    table.sort(enriched, function(a, b) return a.score > b.score end)

    -- 5. Return top results
    return enriched
end

return json.encode(smart_recall("programming preferences"))
```

## What Agents Can Do

With programmable memory, agents can implement:

### Multi-Factor Ranking

Combine multiple signals for better relevance:

```lua
local score = semantic_similarity
            * importance_weight
            * recency_boost
            * source_trust_factor
```

### Cross-Referencing

Query multiple data sources and combine results:

```lua
local user_prefs = semantic_search("preferences", query, 5)
local past_actions = semantic_search("actions", query, 5)
local context = semantic_search("context", query, 5)

-- Combine and deduplicate
local combined = merge_and_rank(user_prefs, past_actions, context)
```

### Dynamic Context Building

Build context tailored to the current task:

```lua
function build_context(task_type, query)
    if task_type == "coding" then
        return semantic_search("code_examples", query, 10)
    elseif task_type == "conversation" then
        local memories = semantic_search("memories", query, 5)
        local history = get_messages("conv", 10)
        return { memories = memories, history = history }
    else
        return semantic_search("general", query, 5)
    end
end
```

### Custom Filtering

Apply complex business logic:

```lua
local results = filter(all_memories, function(m)
    local meta = json.decode(get("meta", m.id) or '{}')
    return meta.verified == true
       and meta.source ~= "deprecated"
       and (meta.expires_at or math.huge) > now()
end)
```

## Safety: The Lua Sandbox

A critical feature of programmable memory is **safety**. The Lua runtime is fully sandboxed:

### Blocked Operations

```lua
-- These will all fail safely
os.execute("rm -rf /")        -- No system commands
io.open("/etc/passwd", "r")   -- No file system access
require("socket")             -- No network access
loadfile("malicious.lua")     -- No loading external code
```

### Available Operations

The sandbox allows only safe, database-focused operations:

- **Storage**: `put()`, `get()`, `delete()`, `keys()`
- **Vectors**: `embed()`, `store_vector()`, `vector_search()`, `semantic_search()`
- **Memory**: `store_memory()`, `recall()`, `recall_by_tags()`
- **Conversations**: `add_message()`, `get_messages()`, `clear_conversation()`
- **Utilities**: `id()`, `now()`, `json.encode()`, `json.decode()`, `map()`, `filter()`, `reduce()`

## Real-World Pattern

Here's how programmable memory works in a typical agent loop:

```lua
function agent_turn(user_message)
    -- 1. Store this interaction
    local mem_id = id()
    store_with_embedding("memory", mem_id, "User: " .. user_message)
    put("memory:meta", mem_id, json.encode({
        importance = 0.7,
        timestamp = now(),
        type = "interaction"
    }))

    -- 2. Build smart context (agent decides strategy)
    local context = smart_recall(user_message)

    -- 3. Add to conversation
    add_message("conv", "user", user_message)
    local history = get_messages("conv", 10)

    -- 4. Return everything the LLM needs
    return json.encode({
        user_message = user_message,
        relevant_context = context,
        conversation_history = history
    })
end
```

## Comparison

| Feature | Traditional Vector DB | Liath |
|---------|----------------------|-------|
| Query interface | Fixed API | Programmable |
| Ranking | Single metric | Multi-factor |
| Cross-referencing | Manual | Built-in |
| Custom logic | External code | Inline |
| Safety | N/A | Sandboxed |
| Flexibility | Limited | Unlimited |

## When to Use Programmable Memory

Programmable memory shines when:

- You need **custom retrieval strategies** beyond simple similarity
- You want to **combine multiple signals** (recency, importance, source, etc.)
- Your agent needs to **adapt its retrieval** based on context
- You're building **autonomous agents** that improve over time

## Next Steps

- [Lua Scripting Guide](../guides/lua-scripting.md) - Learn the full Lua API
- [Building AI Agents](../guides/building-agents.md) - Complete agent development guide
- [Examples](../examples/index.md) - Practical code examples
