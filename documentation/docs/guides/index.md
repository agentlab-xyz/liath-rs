# Guides

Practical guides for building with Liath. These guides cover common patterns, best practices, and advanced techniques.

## Getting Productive

<div class="grid cards" markdown>

-   :material-code-braces:{ .lg .middle } **Lua Scripting**

    ---

    Master the Lua API for programmable queries

    [:octicons-arrow-right-24: Lua Guide](lua-scripting.md)

-   :material-robot:{ .lg .middle } **Building AI Agents**

    ---

    Create agents with memory and conversations

    [:octicons-arrow-right-24: Agent Guide](building-agents.md)

-   :material-brain:{ .lg .middle } **Memory Patterns**

    ---

    Organize and retrieve agent memories effectively

    [:octicons-arrow-right-24: Memory Patterns](memory-patterns.md)

-   :material-forum:{ .lg .middle } **Conversation Management**

    ---

    Handle multi-turn conversations

    [:octicons-arrow-right-24: Conversations](conversations.md)

</div>

## Advanced Topics

<div class="grid cards" markdown>

-   :material-tools:{ .lg .middle } **Tool State**

    ---

    Build stateful tools for your agents

    [:octicons-arrow-right-24: Tool State](tool-state.md)

-   :material-alert-circle:{ .lg .middle } **Error Handling**

    ---

    Handle errors gracefully

    [:octicons-arrow-right-24: Error Handling](error-handling.md)

-   :material-shield-lock:{ .lg .middle } **Security**

    ---

    Secure your Liath deployment

    [:octicons-arrow-right-24: Security](security.md)

-   :material-speedometer:{ .lg .middle } **Performance**

    ---

    Optimize for speed and scale

    [:octicons-arrow-right-24: Performance](performance.md)

</div>

## Quick Reference

### Common Tasks

| Task | Guide | API |
|------|-------|-----|
| Store with embedding | [Lua Scripting](lua-scripting.md#store_with_embedding) | [Lua Stdlib](../api/lua-stdlib.md) |
| Semantic search | [Lua Scripting](lua-scripting.md#semantic_search) | [Lua Stdlib](../api/lua-stdlib.md) |
| Agent memory | [Building Agents](building-agents.md#memory) | [Memory API](../api/memory.md) |
| Conversations | [Conversations](conversations.md) | [Conversation API](../api/conversation.md) |
| Tool state | [Tool State](tool-state.md) | [ToolState API](../api/tool-state.md) |

### Common Patterns

```lua
-- Smart retrieval with ranking
local function smart_recall(query, k)
    local results = semantic_search("memory", query, k * 2)
    local scored = {}

    for _, r in ipairs(results) do
        local meta = json.decode(get("meta", r.id) or '{}')
        local score = (1 - r.distance) * (meta.importance or 0.5)
        table.insert(scored, {content = r.content, score = score})
    end

    table.sort(scored, function(a, b) return a.score > b.score end)
    return slice(scored, 1, k)
end
```

```lua
-- Conversation with context
local function process_message(conv_id, message)
    -- Get context
    local context = semantic_search("memory", message, 5)

    -- Add message
    add_message(conv_id, "user", message)

    -- Return context for LLM
    return json.encode({
        context = map(context, function(c) return c.content end),
        history = get_messages(conv_id, 10)
    })
end
```

## Recommended Reading Order

1. **Start here**: [Lua Scripting](lua-scripting.md) - Foundation for all queries
2. **Build agents**: [Building AI Agents](building-agents.md) - Agent architecture
3. **Organize memory**: [Memory Patterns](memory-patterns.md) - Effective memory use
4. **Handle conversations**: [Conversations](conversations.md) - Multi-turn dialogue
5. **Add tools**: [Tool State](tool-state.md) - Stateful capabilities
6. **Go to production**: [Security](security.md) + [Performance](performance.md)
