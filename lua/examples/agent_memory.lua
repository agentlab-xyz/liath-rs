-- Agent Memory System
-- Demonstrates semantic memory storage and recall for AI agents
--
-- Run with: liath execute --file lua/examples/agent_memory.lua

print("=== Agent Memory System ===\n")

-- Initialize namespace for agent memories
-- create_namespace("agent:memory", 384, "cosine", "f32")

-- Helper: Store a memory with metadata
function remember(content, importance, tags)
    local mem_id = id()

    -- Store the memory content with embedding
    store_with_embedding("agent:memory", mem_id, content)

    -- Store metadata
    put("agent:meta", mem_id, json.encode({
        content = content,
        importance = importance or 0.5,
        tags = tags or {},
        created_at = now()
    }))

    -- Index by tags
    if tags then
        for _, tag in ipairs(tags) do
            local tag_key = "tag:" .. tag .. ":" .. mem_id
            put("agent:tags", tag_key, mem_id)
        end
    end

    print("Stored: " .. content:sub(1, 50) .. "...")
    return mem_id
end

-- Helper: Recall memories by semantic similarity
function recall(query, limit)
    local results = semantic_search("agent:memory", query, limit or 5)

    local memories = {}
    for _, r in ipairs(results) do
        local meta = json.decode(get("agent:meta", r.id) or "{}")
        table.insert(memories, {
            id = r.id,
            content = r.content,
            importance = meta.importance,
            tags = meta.tags,
            relevance = 1 - r.distance
        })
    end

    return memories
end

-- Helper: Recall by tags
function recall_by_tag(tag, limit)
    local memories = {}
    local prefix = "tag:" .. tag .. ":"

    -- In a full implementation, we'd iterate keys with prefix
    -- For now, we do semantic search and filter
    local all = semantic_search("agent:memory", tag, limit * 2)

    for _, r in ipairs(all) do
        local meta = json.decode(get("agent:meta", r.id) or "{}")
        if meta.tags then
            for _, t in ipairs(meta.tags) do
                if t == tag then
                    table.insert(memories, {
                        id = r.id,
                        content = r.content,
                        importance = meta.importance,
                        tags = meta.tags
                    })
                    break
                end
            end
        end
        if #memories >= limit then break end
    end

    return memories
end

-- Store some memories
print("\n--- Storing Memories ---\n")

remember("User prefers dark mode in all applications", 0.9, {"preference", "ui"})
remember("User is a software engineer working on backend systems", 0.8, {"profile", "work"})
remember("User mentioned they have a cat named Whiskers", 0.4, {"personal", "pets"})
remember("User prefers Python for scripting but uses Rust for performance", 0.7, {"preference", "programming"})
remember("User's timezone is Pacific Standard Time", 0.6, {"profile", "location"})
remember("User asked about machine learning last Tuesday", 0.5, {"topic", "ml"})
remember("User prefers concise technical explanations over verbose ones", 0.9, {"preference", "communication"})

-- Semantic recall
print("\n--- Semantic Recall ---\n")

print("Query: 'programming language preferences'")
local results = recall("programming language preferences", 3)
for i, mem in ipairs(results) do
    print(string.format("  %d. [%.0f%%] %s", i, mem.relevance * 100, mem.content))
end

print("\nQuery: 'how should I communicate with this user'")
results = recall("how should I communicate with this user", 3)
for i, mem in ipairs(results) do
    print(string.format("  %d. [%.0f%%] %s", i, mem.relevance * 100, mem.content))
end

-- Build context for LLM
print("\n--- Building LLM Context ---\n")

function build_context(user_query, max_memories)
    local memories = recall(user_query, max_memories)

    local context_parts = {"Relevant information about the user:"}
    for _, mem in ipairs(memories) do
        if mem.relevance > 0.3 then  -- Threshold
            table.insert(context_parts, "- " .. mem.content)
        end
    end

    return table.concat(context_parts, "\n")
end

local context = build_context("What programming tools does this user prefer?", 5)
print(context)

print("\n=== Done ===")
