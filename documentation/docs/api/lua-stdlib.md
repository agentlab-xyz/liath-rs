# Lua Standard Library

Complete reference for all Lua functions available in Liath's sandboxed runtime.

## Storage Functions

### put(namespace, key, value)

Store a key-value pair.

| Parameter | Type | Description |
|-----------|------|-------------|
| namespace | string | Target namespace |
| key | string | Key identifier |
| value | string | Value to store |

**Returns:** `nil`

```lua
put("users", "user:1", '{"name": "Alice"}')
put("config", "theme", "dark")
```

---

### get(namespace, key)

Retrieve a value by key.

| Parameter | Type | Description |
|-----------|------|-------------|
| namespace | string | Target namespace |
| key | string | Key identifier |

**Returns:** `string | nil`

```lua
local value = get("users", "user:1")
if value then
    local user = json.decode(value)
    print(user.name)
end
```

---

### delete(namespace, key)

Delete a key-value pair.

| Parameter | Type | Description |
|-----------|------|-------------|
| namespace | string | Target namespace |
| key | string | Key identifier |

**Returns:** `nil`

```lua
delete("users", "user:1")
```

---

### keys(namespace, prefix?)

List keys in a namespace.

| Parameter | Type | Description |
|-----------|------|-------------|
| namespace | string | Target namespace |
| prefix | string? | Optional key prefix filter |

**Returns:** `table` (array of strings)

```lua
-- All keys
local all_keys = keys("users")

-- Keys with prefix
local user_keys = keys("users", "user:")
```

---

## Vector Functions

### embed(text)

Generate an embedding vector from text.

| Parameter | Type | Description |
|-----------|------|-------------|
| text | string | Text to embed |

**Returns:** `table` (array of floats)

```lua
local vector = embed("Hello, world!")
print("Dimensions:", #vector)
```

---

### store_vector(namespace, id, vector)

Store a vector in the index.

| Parameter | Type | Description |
|-----------|------|-------------|
| namespace | string | Target namespace |
| id | string | Vector identifier |
| vector | table | Vector (array of floats) |

**Returns:** `nil`

```lua
local vec = embed("Some text")
store_vector("docs", "doc:1", vec)
```

---

### store_with_embedding(namespace, id, text)

Store text with auto-generated embedding.

| Parameter | Type | Description |
|-----------|------|-------------|
| namespace | string | Target namespace |
| id | string | Document identifier |
| text | string | Text content |

**Returns:** `nil`

```lua
store_with_embedding("docs", "doc:1", "Introduction to Rust")
store_with_embedding("docs", "doc:2", "Python for data science")
```

---

### vector_search(namespace, vector, k)

Search using a raw vector.

| Parameter | Type | Description |
|-----------|------|-------------|
| namespace | string | Target namespace |
| vector | table | Query vector |
| k | number | Number of results |

**Returns:** `table` (array of results)

```lua
local query_vec = embed("programming")
local results = vector_search("docs", query_vec, 5)

for _, r in ipairs(results) do
    print(r.id, r.distance)
end
```

**Result structure:**
```lua
{
    id = "doc:1",
    distance = 0.123
}
```

---

### semantic_search(namespace, query, k)

Search using text query.

| Parameter | Type | Description |
|-----------|------|-------------|
| namespace | string | Target namespace |
| query | string | Search query |
| k | number | Number of results |

**Returns:** `table` (array of results)

```lua
local results = semantic_search("docs", "systems programming", 3)

for _, r in ipairs(results) do
    print(r.content, r.distance)
end
```

**Result structure:**
```lua
{
    id = "doc:1",
    content = "Introduction to Rust",
    distance = 0.123
}
```

---

## Memory Functions

### store_memory(namespace, content, tags)

Store a tagged memory.

| Parameter | Type | Description |
|-----------|------|-------------|
| namespace | string | Target namespace |
| content | string | Memory content |
| tags | table | Array of tag strings |

**Returns:** `string` (memory ID)

```lua
local id = store_memory("agent:memory", "User likes dark mode", {"preferences", "ui"})
```

---

### recall(namespace, query, k)

Recall memories semantically.

| Parameter | Type | Description |
|-----------|------|-------------|
| namespace | string | Target namespace |
| query | string | Search query |
| k | number | Number of results |

**Returns:** `table` (array of memories)

```lua
local memories = recall("agent:memory", "UI preferences", 5)

for _, m in ipairs(memories) do
    print(m.content, m.distance)
end
```

---

### recall_by_tags(namespace, tags, limit)

Recall memories by tags.

| Parameter | Type | Description |
|-----------|------|-------------|
| namespace | string | Target namespace |
| tags | table | Array of required tags |
| limit | number | Maximum results |

**Returns:** `table` (array of memories)

```lua
local prefs = recall_by_tags("agent:memory", {"preferences"}, 10)
```

---

## Conversation Functions

### add_message(conversation_id, role, content)

Add a message to a conversation.

| Parameter | Type | Description |
|-----------|------|-------------|
| conversation_id | string | Conversation identifier |
| role | string | "user", "assistant", or "system" |
| content | string | Message content |

**Returns:** `nil`

```lua
add_message("conv:main", "user", "Hello!")
add_message("conv:main", "assistant", "Hi! How can I help?")
add_message("conv:main", "system", "Be helpful and concise")
```

---

### get_messages(conversation_id, limit)

Get recent messages.

| Parameter | Type | Description |
|-----------|------|-------------|
| conversation_id | string | Conversation identifier |
| limit | number | Maximum messages |

**Returns:** `table` (array of messages)

```lua
local messages = get_messages("conv:main", 10)

for _, msg in ipairs(messages) do
    print(msg.role .. ": " .. msg.content)
end
```

**Message structure:**
```lua
{
    role = "user",
    content = "Hello!",
    timestamp = 1699876543
}
```

---

### clear_conversation(conversation_id)

Clear all messages.

| Parameter | Type | Description |
|-----------|------|-------------|
| conversation_id | string | Conversation identifier |

**Returns:** `nil`

```lua
clear_conversation("conv:main")
```

---

## Utility Functions

### id()

Generate a unique identifier.

**Returns:** `string` (UUID-like string)

```lua
local unique_id = id()
store_with_embedding("docs", unique_id, "New document")
```

---

### now()

Get current Unix timestamp.

**Returns:** `number` (Unix timestamp in seconds)

```lua
local timestamp = now()
put("meta", "last_access", tostring(timestamp))
```

---

### json.encode(value)

Encode Lua value to JSON string.

| Parameter | Type | Description |
|-----------|------|-------------|
| value | any | Value to encode |

**Returns:** `string`

```lua
local data = {
    name = "Alice",
    scores = {95, 87, 92},
    active = true
}
return json.encode(data)
-- '{"name":"Alice","scores":[95,87,92],"active":true}'
```

---

### json.decode(str)

Decode JSON string to Lua value.

| Parameter | Type | Description |
|-----------|------|-------------|
| str | string | JSON string |

**Returns:** `table | string | number | boolean | nil`

```lua
local json_str = '{"name": "Alice", "age": 30}'
local data = json.decode(json_str)
print(data.name)  -- "Alice"
```

---

## Functional Helpers

### map(list, fn)

Transform each element.

| Parameter | Type | Description |
|-----------|------|-------------|
| list | table | Input array |
| fn | function | Transform function |

**Returns:** `table` (transformed array)

```lua
local numbers = {1, 2, 3, 4, 5}
local doubled = map(numbers, function(n) return n * 2 end)
-- {2, 4, 6, 8, 10}

-- Extract content from search results
local results = semantic_search("docs", "query", 5)
local contents = map(results, function(r) return r.content end)
```

---

### filter(list, fn)

Keep elements matching predicate.

| Parameter | Type | Description |
|-----------|------|-------------|
| list | table | Input array |
| fn | function | Predicate function |

**Returns:** `table` (filtered array)

```lua
local numbers = {1, 2, 3, 4, 5}
local evens = filter(numbers, function(n) return n % 2 == 0 end)
-- {2, 4}

-- Filter by distance threshold
local results = semantic_search("docs", "query", 10)
local close = filter(results, function(r) return r.distance < 0.5 end)
```

---

### reduce(list, fn, initial)

Reduce list to single value.

| Parameter | Type | Description |
|-----------|------|-------------|
| list | table | Input array |
| fn | function | Reducer function(acc, item) |
| initial | any | Initial accumulator value |

**Returns:** `any` (reduced value)

```lua
local numbers = {1, 2, 3, 4, 5}
local sum = reduce(numbers, function(acc, n) return acc + n end, 0)
-- 15

-- Concatenate strings
local words = {"hello", "world"}
local sentence = reduce(words, function(acc, w) return acc .. " " .. w end, "")
```

---

## Blocked Functions

The following are **not available** in the sandbox:

| Category | Blocked |
|----------|---------|
| System | `os.execute`, `os.exit`, `os.remove`, `os.rename` |
| Files | `io.open`, `io.read`, `io.write`, `io.close` |
| Loading | `loadfile`, `dofile`, `require`, `load` |
| Debug | `debug.*` |
| Package | `package.*` |

Attempting to use these will raise an error:

```lua
-- These will all fail
os.execute("ls")  -- Error: attempt to call a nil value
io.open("file")   -- Error: attempt to call a nil value
require("http")   -- Error: attempt to call a nil value
```

---

## Complete Example

```lua
-- Initialize data
store_with_embedding("memory", "m1", "User prefers Rust programming")
store_with_embedding("memory", "m2", "User works on backend systems")
store_with_embedding("memory", "m3", "User asked about async patterns")

put("memory:meta", "m1", json.encode({importance = 0.9, age = 7}))
put("memory:meta", "m2", json.encode({importance = 0.8, age = 14}))
put("memory:meta", "m3", json.encode({importance = 0.7, age = 1}))

-- Smart retrieval function
local function smart_search(query, limit)
    local results = semantic_search("memory", query, limit * 2)

    local scored = {}
    for _, r in ipairs(results) do
        local meta = json.decode(get("memory:meta", r.id) or '{}')
        local recency = meta.age <= 7 and 1.5 or 1.0
        local score = (1 - r.distance) * (meta.importance or 0.5) * recency

        table.insert(scored, {
            content = r.content,
            score = score
        })
    end

    table.sort(scored, function(a, b) return a.score > b.score end)

    return map(
        filter(scored, function(_, i) return i <= limit end),
        function(s) return s end
    )
end

-- Add to conversation
add_message("main", "user", "What should I learn next?")

-- Get context
local context = smart_search("learning recommendations", 3)

return json.encode({
    query = "learning recommendations",
    context = context,
    timestamp = now()
})
```
