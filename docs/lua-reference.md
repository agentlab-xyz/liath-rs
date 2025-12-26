# Lua Reference

Liath provides a sandboxed Lua 5.4 runtime with built-in functions for storage, vectors, embeddings, and agent workflows.

## Safety Model

Liath's Lua runtime is **sandboxed**:
- No file system access (`io`, `os.execute`, `os.remove` disabled)
- No network access
- No system calls
- No loading external modules (`require` disabled for arbitrary paths)
- Memory and execution time limits

This makes it safe for AI agents to execute Lua code without risk of system compromise.

## Core Storage

### put(namespace, key, value)

Store a value in the key-value store.

```lua
put("config", "theme", "dark")
put("users", "user:123", '{"name": "Alice", "email": "alice@example.com"}')
```

**Parameters:**
- `namespace` (string): Storage namespace
- `key` (string): Key identifier
- `value` (string): Value to store

**Returns:** `nil`

---

### get(namespace, key)

Retrieve a value from the key-value store.

```lua
local theme = get("config", "theme")
local user_json = get("users", "user:123")
```

**Parameters:**
- `namespace` (string): Storage namespace
- `key` (string): Key identifier

**Returns:** `string` or `nil` if key doesn't exist

---

### delete(namespace, key)

Delete a key from the store.

```lua
delete("cache", "temp-data")
```

**Parameters:**
- `namespace` (string): Storage namespace
- `key` (string): Key identifier

**Returns:** `nil`

---

### keys(namespace, prefix)

List keys matching a prefix.

```lua
local user_keys = keys("users", "user:")
-- Returns: {"user:1", "user:2", "user:3"}
```

**Parameters:**
- `namespace` (string): Storage namespace
- `prefix` (string): Key prefix to match

**Returns:** `table` (array of key strings)

## Vector Operations

### embed(text)

Generate an embedding vector from text using FastEmbed.

```lua
local vector = embed("Hello, world!")
-- Returns: {0.123, -0.456, 0.789, ...} (384 dimensions)
```

**Parameters:**
- `text` (string): Text to embed

**Returns:** `table` (array of floats)

---

### store_vector(namespace, id, vector)

Store a vector in the index.

```lua
local vec = embed("Some text")
store_vector("documents", "doc-1", vec)
```

**Parameters:**
- `namespace` (string): Storage namespace
- `id` (string): Vector identifier
- `vector` (table): Array of floats

**Returns:** `nil`

---

### vector_search(namespace, vector, limit)

Find similar vectors by cosine similarity.

```lua
local query_vec = embed("search query")
local results = vector_search("documents", query_vec, 10)

for _, result in ipairs(results) do
    print(result.id, result.distance)
end
```

**Parameters:**
- `namespace` (string): Storage namespace
- `vector` (table): Query vector
- `limit` (number): Maximum results

**Returns:** `table` (array of `{id, distance}`)

---

### store_with_embedding(namespace, id, text)

Store text with automatic embedding generation.

```lua
store_with_embedding("notes", "note-1", "Meeting notes from Monday standup")
```

**Parameters:**
- `namespace` (string): Storage namespace
- `id` (string): Document identifier
- `text` (string): Text content (will be embedded automatically)

**Returns:** `nil`

---

### semantic_search(namespace, query, limit)

Search by text similarity (embeds query automatically).

```lua
local results = semantic_search("notes", "weekly meetings", 5)

for _, result in ipairs(results) do
    print(result.id, result.content, result.distance)
end
```

**Parameters:**
- `namespace` (string): Storage namespace
- `query` (string): Search query text
- `limit` (number): Maximum results

**Returns:** `table` (array of `{id, content, distance}`)

## Agent Memory

### store_memory(agent_id, entry)

Store a memory entry for an agent.

```lua
store_memory("agent-1", {
    content = "User mentioned they prefer dark mode",
    tags = {"preference", "ui"},
    importance = 0.8
})
```

**Parameters:**
- `agent_id` (string): Agent identifier
- `entry` (table): Memory entry with fields:
  - `content` (string): Memory content
  - `tags` (table, optional): Array of tag strings
  - `importance` (number, optional): 0.0 to 1.0, default 0.5

**Returns:** `string` (memory ID)

---

### recall(agent_id, query, limit)

Recall memories by semantic similarity.

```lua
local memories = recall("agent-1", "user interface preferences", 5)

for _, mem in ipairs(memories) do
    print(mem.content, mem.importance, mem.timestamp)
end
```

**Parameters:**
- `agent_id` (string): Agent identifier
- `query` (string): Search query
- `limit` (number): Maximum results

**Returns:** `table` (array of memory entries)

---

### recall_by_tags(agent_id, tags, limit)

Recall memories by tag.

```lua
local memories = recall_by_tags("agent-1", {"preference", "ui"}, 10)
```

**Parameters:**
- `agent_id` (string): Agent identifier
- `tags` (table): Array of tags to match
- `limit` (number): Maximum results

**Returns:** `table` (array of memory entries)

## Conversations

### add_message(conversation_id, role, content)

Add a message to a conversation.

```lua
add_message("conv-123", "user", "Hello!")
add_message("conv-123", "assistant", "Hi there! How can I help?")
add_message("conv-123", "user", "What's the weather like?")
```

**Parameters:**
- `conversation_id` (string): Conversation identifier
- `role` (string): Message role ("user", "assistant", "system")
- `content` (string): Message content

**Returns:** `nil`

---

### get_messages(conversation_id, limit)

Get recent messages from a conversation.

```lua
local history = get_messages("conv-123", 10)

for _, msg in ipairs(history) do
    print(msg.role .. ": " .. msg.content)
end
```

**Parameters:**
- `conversation_id` (string): Conversation identifier
- `limit` (number): Maximum messages to return

**Returns:** `table` (array of `{role, content, timestamp}`)

---

### clear_conversation(conversation_id)

Clear all messages in a conversation.

```lua
clear_conversation("conv-123")
```

**Parameters:**
- `conversation_id` (string): Conversation identifier

**Returns:** `nil`

## Tool State

### set_tool_state(tool_name, key, value)

Store state for a tool.

```lua
set_tool_state("browser", "current_url", "https://example.com")
set_tool_state("browser", "tabs", json.encode({"tab1", "tab2"}))
```

**Parameters:**
- `tool_name` (string): Tool identifier
- `key` (string): State key
- `value` (string): State value

**Returns:** `nil`

---

### get_tool_state(tool_name, key)

Retrieve tool state.

```lua
local url = get_tool_state("browser", "current_url")
local tabs = json.decode(get_tool_state("browser", "tabs") or "[]")
```

**Parameters:**
- `tool_name` (string): Tool identifier
- `key` (string): State key

**Returns:** `string` or `nil`

## Utilities

### now()

Get current Unix timestamp in seconds.

```lua
local timestamp = now()
local one_day_ago = now() - 86400
```

**Returns:** `number` (Unix timestamp)

---

### id()

Generate a unique identifier.

```lua
local unique_id = id()
-- Returns something like: "01HQXK5M3N..."
```

**Returns:** `string` (ULID)

---

### json.encode(value)

Serialize a Lua value to JSON.

```lua
local json_str = json.encode({
    name = "Alice",
    age = 30,
    tags = {"admin", "user"}
})
-- Returns: '{"name":"Alice","age":30,"tags":["admin","user"]}'
```

**Parameters:**
- `value` (any): Lua value (table, string, number, boolean, nil)

**Returns:** `string` (JSON)

---

### json.decode(str)

Parse JSON into a Lua value.

```lua
local data = json.decode('{"name":"Alice","age":30}')
print(data.name)  -- "Alice"
```

**Parameters:**
- `str` (string): JSON string

**Returns:** Lua value (table, string, number, boolean, nil)

## Functional Utilities

### map(list, fn)

Transform each element in a list.

```lua
local numbers = {1, 2, 3, 4, 5}
local doubled = map(numbers, function(n) return n * 2 end)
-- Returns: {2, 4, 6, 8, 10}
```

**Parameters:**
- `list` (table): Array to transform
- `fn` (function): Transformation function

**Returns:** `table` (new array)

---

### filter(list, fn)

Keep elements that pass a predicate.

```lua
local numbers = {1, 2, 3, 4, 5}
local evens = filter(numbers, function(n) return n % 2 == 0 end)
-- Returns: {2, 4}
```

**Parameters:**
- `list` (table): Array to filter
- `fn` (function): Predicate function returning boolean

**Returns:** `table` (new array)

---

### reduce(list, fn, initial)

Reduce a list to a single value.

```lua
local numbers = {1, 2, 3, 4, 5}
local sum = reduce(numbers, function(acc, n) return acc + n end, 0)
-- Returns: 15
```

**Parameters:**
- `list` (table): Array to reduce
- `fn` (function): Reducer function `(accumulator, element) -> new_accumulator`
- `initial` (any): Initial accumulator value

**Returns:** Final accumulator value

---

### inspect(value)

Pretty-print a Lua value for debugging.

```lua
local data = {name = "Alice", tags = {"a", "b"}}
print(inspect(data))
-- {
--   name = "Alice",
--   tags = {"a", "b"}
-- }
```

**Parameters:**
- `value` (any): Value to inspect

**Returns:** `string` (formatted representation)

## Namespace Management

### create_namespace(name, dimensions, metric, scalar)

Create a new namespace with vector support.

```lua
create_namespace("documents", 384, "cosine", "f32")
```

**Parameters:**
- `name` (string): Namespace name
- `dimensions` (number): Vector dimensions (384 for default model)
- `metric` (string): "cosine" or "euclidean"
- `scalar` (string): "f32" or "f16"

**Returns:** `nil`

---

### list_namespaces()

List all namespaces.

```lua
local namespaces = list_namespaces()
-- Returns: {"default", "documents", "memories"}
```

**Returns:** `table` (array of namespace names)

---

### delete_namespace(name)

Delete a namespace and all its data.

```lua
delete_namespace("temp-data")
```

**Parameters:**
- `name` (string): Namespace to delete

**Returns:** `nil`

## Complete Examples

### RAG (Retrieval-Augmented Generation)

```lua
-- Index documents
function index_document(doc_id, content)
    store_with_embedding("docs", doc_id, content)
    put("docs:meta", doc_id, json.encode({
        indexed_at = now(),
        length = #content
    }))
end

-- Retrieve context for a query
function get_context(query, k)
    local results = semantic_search("docs", query, k)
    local context = {}
    for _, r in ipairs(results) do
        table.insert(context, {
            content = r.content,
            relevance = 1 - r.distance
        })
    end
    return context
end

-- Usage
index_document("doc1", "Lua is a lightweight scripting language")
index_document("doc2", "Rust is a systems programming language")

local context = get_context("scripting languages", 2)
return json.encode(context)
```

### Agent Memory System

```lua
-- Store an experience
function remember(agent_id, experience, importance)
    return store_memory(agent_id, {
        content = experience,
        tags = extract_keywords(experience),
        importance = importance or 0.5
    })
end

-- Get relevant context for a task
function get_relevant_memories(agent_id, task, limit)
    local semantic = recall(agent_id, task, limit)

    -- Filter by recency (last 7 days)
    local recent = filter(semantic, function(m)
        return m.timestamp > now() - (7 * 86400)
    end)

    -- Sort by combined score
    table.sort(recent, function(a, b)
        local score_a = (1 - a.distance) * a.importance
        local score_b = (1 - b.distance) * b.importance
        return score_a > score_b
    end)

    return recent
end

-- Extract keywords (simple implementation)
function extract_keywords(text)
    local keywords = {}
    for word in text:gmatch("%w+") do
        if #word > 4 then
            table.insert(keywords, word:lower())
        end
    end
    return keywords
end
```

### Conversation with Memory

```lua
function chat_turn(agent_id, conv_id, user_message)
    -- Store user message
    add_message(conv_id, "user", user_message)

    -- Remember this interaction
    remember(agent_id, "User said: " .. user_message, 0.6)

    -- Get conversation history
    local history = get_messages(conv_id, 10)

    -- Get relevant memories
    local memories = get_relevant_memories(agent_id, user_message, 5)

    -- Return context for LLM
    return json.encode({
        history = history,
        memories = map(memories, function(m) return m.content end),
        user_message = user_message
    })
end
```

### Data Processing Pipeline

```lua
-- Safe data analysis (no system access)
function analyze_sales(data_json)
    local data = json.decode(data_json)

    -- Calculate totals by category
    local by_category = {}
    for _, sale in ipairs(data) do
        local cat = sale.category
        by_category[cat] = (by_category[cat] or 0) + sale.amount
    end

    -- Find top category
    local top_cat, top_amount = nil, 0
    for cat, amount in pairs(by_category) do
        if amount > top_amount then
            top_cat, top_amount = cat, amount
        end
    end

    return json.encode({
        total_sales = reduce(data, function(acc, s) return acc + s.amount end, 0),
        by_category = by_category,
        top_category = top_cat,
        record_count = #data
    })
end
```
