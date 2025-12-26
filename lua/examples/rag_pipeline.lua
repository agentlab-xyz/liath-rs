-- RAG (Retrieval-Augmented Generation) Pipeline
-- Demonstrates building a complete RAG system in Liath
--
-- Run with: liath execute --file lua/examples/rag_pipeline.lua

print("=== RAG Pipeline Example ===\n")

-- Document indexing
function index_document(doc_id, content, metadata)
    -- Store content with embedding
    store_with_embedding("rag:docs", doc_id, content)

    -- Store metadata
    put("rag:meta", doc_id, json.encode({
        id = doc_id,
        content = content,
        metadata = metadata or {},
        indexed_at = now(),
        chunk_length = #content
    }))

    return doc_id
end

-- Chunk text into smaller pieces
function chunk_text(text, chunk_size, overlap)
    chunk_size = chunk_size or 500
    overlap = overlap or 50

    local chunks = {}
    local pos = 1

    while pos <= #text do
        local chunk_end = math.min(pos + chunk_size - 1, #text)

        -- Try to break at sentence boundary
        if chunk_end < #text then
            local last_period = text:sub(pos, chunk_end):match(".*()%. ")
            if last_period and last_period > chunk_size / 2 then
                chunk_end = pos + last_period
            end
        end

        table.insert(chunks, text:sub(pos, chunk_end))
        pos = chunk_end - overlap + 1
    end

    return chunks
end

-- Index a document with chunking
function index_document_chunked(doc_id, content, metadata)
    local chunks = chunk_text(content, 300, 30)

    for i, chunk in ipairs(chunks) do
        local chunk_id = doc_id .. ":chunk:" .. i
        index_document(chunk_id, chunk, {
            parent_doc = doc_id,
            chunk_index = i,
            total_chunks = #chunks,
            original_metadata = metadata
        })
    end

    print(string.format("Indexed '%s' as %d chunks", doc_id, #chunks))
    return #chunks
end

-- Retrieve relevant chunks
function retrieve(query, k)
    local results = semantic_search("rag:docs", query, k)

    local retrieved = {}
    for _, r in ipairs(results) do
        local meta = json.decode(get("rag:meta", r.id) or "{}")
        table.insert(retrieved, {
            id = r.id,
            content = r.content,
            relevance = 1 - r.distance,
            metadata = meta.metadata or {}
        })
    end

    return retrieved
end

-- Build context string for LLM
function build_rag_context(query, k)
    local chunks = retrieve(query, k)

    local context_parts = {}
    for i, chunk in ipairs(chunks) do
        local header = string.format("[Source %d - %.0f%% relevant]",
            i, chunk.relevance * 100)
        table.insert(context_parts, header)
        table.insert(context_parts, chunk.content)
        table.insert(context_parts, "")
    end

    return table.concat(context_parts, "\n")
end

-- Full RAG query function
function rag_query(question)
    print("Question: " .. question)
    print(string.rep("-", 50))

    -- Retrieve relevant context
    local context = build_rag_context(question, 3)

    -- Build prompt for LLM
    local prompt = string.format([[
Based on the following context, answer the question.

Context:
%s

Question: %s

Answer:]], context, question)

    return {
        question = question,
        context = context,
        prompt = prompt,
        -- In real usage, you'd send 'prompt' to an LLM
    }
end

-- Example documents
print("--- Indexing Documents ---\n")

local doc1 = [[
Rust is a systems programming language focused on safety, speed, and concurrency.
It achieves memory safety without garbage collection through its ownership system.
The borrow checker ensures references are always valid. Rust is commonly used for
systems programming, WebAssembly, CLI tools, and embedded systems.
]]

local doc2 = [[
Python is a high-level programming language known for its simplicity and readability.
It has a large ecosystem of libraries for data science, machine learning, and web
development. Python uses dynamic typing and automatic memory management through
garbage collection. Popular frameworks include Django, Flask, NumPy, and PyTorch.
]]

local doc3 = [[
JavaScript is the language of the web, running in browsers and on servers via Node.js.
It supports event-driven, functional, and object-oriented programming styles.
Modern JavaScript (ES6+) includes features like arrow functions, async/await, and
modules. Popular frameworks include React, Vue, Angular, and Express.
]]

index_document_chunked("rust-overview", doc1, {topic = "programming", language = "rust"})
index_document_chunked("python-overview", doc2, {topic = "programming", language = "python"})
index_document_chunked("javascript-overview", doc3, {topic = "programming", language = "javascript"})

-- Test retrieval
print("\n--- RAG Queries ---\n")

local result1 = rag_query("What language is best for memory safety?")
print("\nContext retrieved:")
print(result1.context)

print("\n" .. string.rep("=", 50) .. "\n")

local result2 = rag_query("Which language has the best machine learning libraries?")
print("\nContext retrieved:")
print(result2.context)

print("\n=== Done ===")
