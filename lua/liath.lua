-- Liath Standard Library
-- High-level Lua API for AI-powered database operations
--
-- This library provides convenient abstractions on top of the core Liath functions.

local liath = {}

-- ============================================================
-- NAMESPACE HELPERS
-- ============================================================

--- Create a namespace with sensible defaults for AI/RAG workloads
-- @param name Namespace name
-- @param opts Options table (optional): { dimensions=384, metric="cosine", scalar="f32" }
function liath.create(name, opts)
    opts = opts or {}
    local dimensions = opts.dimensions or 384  -- Default for all-MiniLM-L6-v2
    local metric = opts.metric or "cosine"
    local scalar = opts.scalar or "f32"
    return create_namespace(name, dimensions, metric, scalar)
end

--- Get or create a namespace
function liath.ensure(name, opts)
    if not namespace_exists(name) then
        liath.create(name, opts)
    end
    return name
end

-- ============================================================
-- DOCUMENT STORE (High-level RAG interface)
-- ============================================================

liath.docs = {}

--- Store a document with automatic embedding
-- @param ns Namespace
-- @param doc Document table: { id=number, key=string, text=string, metadata={} }
function liath.docs.store(ns, doc)
    local id = doc.id or timestamp() * 1000000 + math.random(1000000)
    local key = doc.key or ("doc:" .. id)

    -- Store the text with embedding
    store_document(ns, id, key, doc.text)

    -- Store metadata if provided
    if doc.metadata then
        local meta_key = key .. ":meta"
        insert_json(ns, meta_key, doc.metadata)
    end

    return { id = id, key = key }
end

--- Store multiple documents
function liath.docs.store_many(ns, docs)
    local results = {}
    for _, doc in ipairs(docs) do
        table.insert(results, liath.docs.store(ns, doc))
    end
    return results
end

--- Search documents by semantic similarity
-- @param ns Namespace
-- @param query Query text
-- @param k Number of results (default 5)
-- @return Array of { id, distance, text, metadata }
function liath.docs.search(ns, query, k)
    k = k or 5
    local results = semantic_search(ns, query, k)

    -- Enrich results with content
    for _, r in ipairs(results) do
        local key = "doc:" .. r.id
        r.text = select(ns, key)

        local meta_key = key .. ":meta"
        r.metadata = select_json(ns, meta_key)
    end

    return results
end

--- Get a document by key
function liath.docs.get(ns, key)
    local text = select(ns, key)
    if not text then return nil end

    local meta = select_json(ns, key .. ":meta")
    return { key = key, text = text, metadata = meta }
end

--- Delete a document
function liath.docs.delete(ns, key)
    delete(ns, key)
    delete(ns, key .. ":meta")
end

-- ============================================================
-- MEMORY (Semantic memory for agents)
-- ============================================================

liath.memory = {}

--- Store a memory with optional tags
-- @param ns Namespace (typically "agent_<id>_memory")
-- @param content Text content
-- @param tags Array of tags (optional)
-- @return memory id
function liath.memory.store(ns, content, tags)
    return memory_store(ns, content, tags)
end

--- Recall memories by semantic similarity
-- @param ns Namespace
-- @param query Query text
-- @param k Number of results (default 5)
function liath.memory.recall(ns, query, k)
    return memory_recall(ns, query, k or 5)
end

--- Store a memory and immediately associate it with tags
function liath.memory.remember(ns, content, tags)
    local id = liath.memory.store(ns, content, tags)
    return { id = id, content = content, tags = tags }
end

-- ============================================================
-- KEY-VALUE HELPERS
-- ============================================================

liath.kv = {}

--- Set a value (alias for insert)
function liath.kv.set(ns, key, value)
    if type(value) == "table" then
        insert_json(ns, key, value)
    else
        insert(ns, key, tostring(value))
    end
end

--- Get a value
function liath.kv.get(ns, key, as_json)
    if as_json then
        return select_json(ns, key)
    else
        return select(ns, key)
    end
end

--- Delete a value
function liath.kv.del(ns, key)
    delete(ns, key)
end

--- Check if key exists
function liath.kv.exists(ns, key)
    return select(ns, key) ~= nil
end

--- Increment a numeric value
function liath.kv.incr(ns, key, amount)
    amount = amount or 1
    local current = tonumber(select(ns, key)) or 0
    local new_value = current + amount
    insert(ns, key, tostring(new_value))
    return new_value
end

--- Get multiple keys
function liath.kv.mget(ns, keys)
    return batch_select(ns, keys)
end

--- Set multiple key-value pairs
function liath.kv.mset(ns, items)
    local batch = {}
    for k, v in pairs(items) do
        table.insert(batch, { key = k, value = tostring(v) })
    end
    return batch_insert(ns, batch)
end

-- ============================================================
-- CONVERSATION HELPERS
-- ============================================================

liath.conversation = {}

--- Create a new conversation
-- @param ns Namespace
-- @param conv_id Conversation ID (optional, will generate if not provided)
function liath.conversation.new(ns, conv_id)
    conv_id = conv_id or uuid()
    local meta_key = "conv:" .. conv_id .. ":meta"
    insert_json(ns, meta_key, {
        id = conv_id,
        created_at = timestamp(),
        message_count = 0
    })
    return conv_id
end

--- Add a message to a conversation
-- @param ns Namespace
-- @param conv_id Conversation ID
-- @param role "user", "assistant", "system", or "tool:<name>"
-- @param content Message content
function liath.conversation.add(ns, conv_id, role, content)
    -- Get and update metadata
    local meta_key = "conv:" .. conv_id .. ":meta"
    local meta = select_json(ns, meta_key)
    if not meta then
        meta = { id = conv_id, created_at = timestamp(), message_count = 0 }
    end

    meta.message_count = meta.message_count + 1
    local msg_id = meta.message_count

    -- Store message
    local msg_key = string.format("conv:%s:msg:%08d", conv_id, msg_id)
    local message = {
        id = msg_id,
        role = role,
        content = content,
        timestamp = timestamp()
    }
    insert_json(ns, msg_key, message)

    -- Update metadata
    insert_json(ns, meta_key, meta)

    return message
end

--- Get conversation history
-- @param ns Namespace
-- @param conv_id Conversation ID
-- @param limit Max messages to return (from end)
function liath.conversation.history(ns, conv_id, limit)
    local meta_key = "conv:" .. conv_id .. ":meta"
    local meta = select_json(ns, meta_key)
    if not meta then return {} end

    limit = limit or meta.message_count
    local start_id = math.max(1, meta.message_count - limit + 1)

    local messages = {}
    for i = start_id, meta.message_count do
        local msg_key = string.format("conv:%s:msg:%08d", conv_id, i)
        local msg = select_json(ns, msg_key)
        if msg then
            table.insert(messages, msg)
        end
    end

    return messages
end

--- Format conversation as a string (for LLM context)
function liath.conversation.format(messages, format)
    format = format or "simple"
    local result = {}

    for _, msg in ipairs(messages) do
        if format == "simple" then
            table.insert(result, msg.role .. ": " .. msg.content)
        elseif format == "markdown" then
            table.insert(result, "**" .. msg.role .. "**: " .. msg.content)
        elseif format == "chat" then
            table.insert(result, "<|" .. msg.role .. "|>\n" .. msg.content)
        end
    end

    return table.concat(result, "\n\n")
end

-- ============================================================
-- UTILITY FUNCTIONS
-- ============================================================

liath.util = {}

--- Generate a unique ID
function liath.util.id()
    return uuid()
end

--- Get current timestamp
function liath.util.now()
    return timestamp()
end

--- Sleep for milliseconds
function liath.util.sleep(ms)
    sleep(ms)
end

--- Pretty print a table
function liath.util.inspect(t, indent)
    indent = indent or 0
    local prefix = string.rep("  ", indent)

    if type(t) ~= "table" then
        return tostring(t)
    end

    local result = "{\n"
    for k, v in pairs(t) do
        local key = type(k) == "string" and k or "[" .. tostring(k) .. "]"
        if type(v) == "table" then
            result = result .. prefix .. "  " .. key .. " = " .. liath.util.inspect(v, indent + 1) .. ",\n"
        else
            local val = type(v) == "string" and ('"' .. v .. '"') or tostring(v)
            result = result .. prefix .. "  " .. key .. " = " .. val .. ",\n"
        end
    end
    return result .. prefix .. "}"
end

--- Map function over array
function liath.util.map(arr, fn)
    local result = {}
    for i, v in ipairs(arr) do
        result[i] = fn(v, i)
    end
    return result
end

--- Filter array by predicate
function liath.util.filter(arr, fn)
    local result = {}
    for _, v in ipairs(arr) do
        if fn(v) then
            table.insert(result, v)
        end
    end
    return result
end

--- Reduce array
function liath.util.reduce(arr, fn, initial)
    local acc = initial
    for _, v in ipairs(arr) do
        acc = fn(acc, v)
    end
    return acc
end

-- ============================================================
-- AGENT BUILDER
-- ============================================================

liath.agent = {}

--- Create a new agent instance
-- @param id Agent ID
-- @param ns_prefix Namespace prefix (optional, defaults to "agent_")
function liath.agent.new(id, ns_prefix)
    ns_prefix = ns_prefix or "agent_"

    local agent = {
        id = id,
        memory_ns = ns_prefix .. id .. "_memory",
        conv_ns = ns_prefix .. id .. "_conv",
        state_ns = ns_prefix .. id .. "_state"
    }

    -- Ensure namespaces exist
    liath.ensure(agent.memory_ns)
    liath.ensure(agent.conv_ns)
    liath.ensure(agent.state_ns)

    --- Store a memory
    function agent:remember(content, tags)
        return liath.memory.store(self.memory_ns, content, tags)
    end

    --- Recall memories
    function agent:recall(query, k)
        return liath.memory.recall(self.memory_ns, query, k or 5)
    end

    --- Start a new conversation
    function agent:start_conversation()
        return liath.conversation.new(self.conv_ns)
    end

    --- Add message to conversation
    function agent:say(conv_id, role, content)
        return liath.conversation.add(self.conv_ns, conv_id, role, content)
    end

    --- Get conversation history
    function agent:get_history(conv_id, limit)
        return liath.conversation.history(self.conv_ns, conv_id, limit)
    end

    --- Get state value
    function agent:get_state(key)
        return liath.kv.get(self.state_ns, key, true)
    end

    --- Set state value
    function agent:set_state(key, value)
        return liath.kv.set(self.state_ns, key, value)
    end

    return agent
end

-- ============================================================
-- RAG PIPELINE HELPERS
-- ============================================================

liath.rag = {}

--- Create a simple RAG context from query
-- @param ns Namespace with documents
-- @param query User query
-- @param opts Options: { k=5, max_tokens=2000 }
function liath.rag.context(ns, query, opts)
    opts = opts or {}
    local k = opts.k or 5
    local max_tokens = opts.max_tokens or 2000

    local results = liath.docs.search(ns, query, k)

    local context_parts = {}
    local total_len = 0

    for _, doc in ipairs(results) do
        if doc.text then
            local text_len = #doc.text
            if total_len + text_len > max_tokens * 4 then  -- Rough char estimate
                break
            end
            table.insert(context_parts, doc.text)
            total_len = total_len + text_len
        end
    end

    return {
        context = table.concat(context_parts, "\n\n---\n\n"),
        sources = results,
        query = query
    }
end

--- Format RAG prompt
function liath.rag.prompt(context, query, template)
    template = template or [[
Based on the following context, answer the question.

Context:
{context}

Question: {query}

Answer:]]

    return template:gsub("{context}", context):gsub("{query}", query)
end

-- ============================================================
-- HTTP CLIENT (if available via LuaRocks)
-- ============================================================

-- Note: HTTP requires external Lua libraries like lua-requests or luasocket
-- These can be installed via: install_package("luasocket")

liath.http = {}

--- Check if HTTP is available
function liath.http.available()
    local ok, _ = pcall(require, "socket.http")
    return ok
end

-- Return the module
return liath
