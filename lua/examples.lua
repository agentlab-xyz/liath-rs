-- ============================================================
-- Liath Lua Examples
-- ============================================================
-- These examples demonstrate the full capabilities of Liath's Lua API
-- Run with: liath --execute "dofile('examples.lua')"

print("=== Liath Lua API Examples ===\n")

-- ============================================================
-- 1. BASIC KEY-VALUE OPERATIONS
-- ============================================================
print("1. Basic Key-Value Operations")
print("-----------------------------")

-- Create a namespace for our data
create_namespace("myapp", 384, "cosine", "f32")

-- Simple string storage
insert("myapp", "greeting", "Hello, Liath!")
local greeting = select("myapp", "greeting")
print("Stored greeting: " .. greeting)

-- Using the high-level liath.kv API
liath.kv.set("myapp", "counter", 0)
liath.kv.incr("myapp", "counter")
liath.kv.incr("myapp", "counter", 5)
print("Counter value: " .. liath.kv.get("myapp", "counter"))

print("")

-- ============================================================
-- 2. JSON DATA STORAGE
-- ============================================================
print("2. JSON Data Storage")
print("--------------------")

-- Store structured data as JSON
local user = {
    name = "Alice",
    email = "alice@example.com",
    roles = {"admin", "user"},
    settings = {
        theme = "dark",
        notifications = true
    }
}

insert_json("myapp", "user:1", user)
local loaded_user = select_json("myapp", "user:1")
print("Loaded user: " .. loaded_user.name .. " (" .. loaded_user.email .. ")")
print("  Theme: " .. loaded_user.settings.theme)
print("  Roles: " .. table.concat(loaded_user.roles, ", "))

print("")

-- ============================================================
-- 3. DOCUMENT STORAGE & SEMANTIC SEARCH
-- ============================================================
print("3. Document Storage & Semantic Search")
print("--------------------------------------")

-- Create a namespace for documents
liath.ensure("knowledge")

-- Store documents with automatic embedding
liath.docs.store("knowledge", {
    text = "The Eiffel Tower is located in Paris, France. It was built in 1889.",
    metadata = { source = "wikipedia", topic = "landmarks" }
})

liath.docs.store("knowledge", {
    text = "The Great Wall of China is over 13,000 miles long.",
    metadata = { source = "encyclopedia", topic = "landmarks" }
})

liath.docs.store("knowledge", {
    text = "Python is a popular programming language created by Guido van Rossum.",
    metadata = { source = "docs", topic = "programming" }
})

-- Search for relevant documents
print("\nSearching for 'famous French monument'...")
local results = liath.docs.search("knowledge", "famous French monument", 2)
for i, doc in ipairs(results) do
    print(string.format("  %d. (score: %.3f) %s", i, doc.distance, doc.text or "(no text)"))
end

print("")

-- ============================================================
-- 4. AGENT MEMORY
-- ============================================================
print("4. Agent Memory")
print("---------------")

-- Create an agent namespace
liath.ensure("agent_assistant_memory")

-- Store memories with tags
memory_store("agent_assistant_memory",
    "User prefers dark mode and minimal notifications",
    {"preferences", "ui"})

memory_store("agent_assistant_memory",
    "User is working on a machine learning project",
    {"context", "work"})

memory_store("agent_assistant_memory",
    "User's timezone is UTC-5 (Eastern)",
    {"preferences", "time"})

-- Recall relevant memories
print("\nRecalling memories about 'user preferences'...")
local memories = memory_recall("agent_assistant_memory", "user preferences settings", 2)
for i, mem in ipairs(memories) do
    print(string.format("  %d. %s", i, mem.content or "(no content)"))
end

print("")

-- ============================================================
-- 5. CONVERSATION MANAGEMENT
-- ============================================================
print("5. Conversation Management")
print("--------------------------")

liath.ensure("chatbot_conv")

-- Start a new conversation
local conv_id = liath.conversation.new("chatbot_conv")
print("Started conversation: " .. conv_id)

-- Add messages
liath.conversation.add("chatbot_conv", conv_id, "user", "Hello! How are you?")
liath.conversation.add("chatbot_conv", conv_id, "assistant", "I'm doing well, thank you! How can I help you today?")
liath.conversation.add("chatbot_conv", conv_id, "user", "What's the weather like?")
liath.conversation.add("chatbot_conv", conv_id, "assistant", "I don't have access to real-time weather data, but I can help you find a weather service!")

-- Get history
print("\nConversation history:")
local history = liath.conversation.history("chatbot_conv", conv_id)
for _, msg in ipairs(history) do
    print(string.format("  [%s]: %s", msg.role, msg.content))
end

-- Format for LLM context
print("\nFormatted for LLM:")
print(liath.conversation.format(history, "simple"))

print("")

-- ============================================================
-- 6. AGENT BUILDER
-- ============================================================
print("6. Agent Builder (High-Level API)")
print("----------------------------------")

-- Create an agent with all capabilities
local agent = liath.agent.new("support_bot")

-- Remember things
agent:remember("Customer prefers email communication", {"preferences"})
agent:remember("Account was created on 2024-01-15", {"account"})

-- Start a conversation
local chat_id = agent:start_conversation()
agent:say(chat_id, "user", "I need help with my account")
agent:say(chat_id, "assistant", "I'd be happy to help! What seems to be the issue?")

-- Store agent state
agent:set_state("last_topic", "account_help")
agent:set_state("interaction_count", 1)

print("Agent state: " .. liath.util.inspect(agent:get_state("last_topic")))

-- Recall relevant context
print("\nRecalling agent memories about 'customer preferences':")
local relevant = agent:recall("customer preferences", 2)
for i, mem in ipairs(relevant) do
    print(string.format("  %d. %s", i, mem.content or "(no content)"))
end

print("")

-- ============================================================
-- 7. RAG PIPELINE
-- ============================================================
print("7. RAG Pipeline")
print("---------------")

-- Build context from documents
local rag_context = liath.rag.context("knowledge", "Tell me about the Eiffel Tower", { k = 2 })
print("RAG Context (first 200 chars):")
print("  " .. string.sub(rag_context.context, 1, 200) .. "...")

-- Generate prompt
local prompt = liath.rag.prompt(rag_context.context, rag_context.query)
print("\nGenerated prompt template ready for LLM")

print("")

-- ============================================================
-- 8. BATCH OPERATIONS
-- ============================================================
print("8. Batch Operations")
print("-------------------")

-- Batch insert
local items = {
    { key = "item:1", value = "First item" },
    { key = "item:2", value = "Second item" },
    { key = "item:3", value = "Third item" }
}
local count = batch_insert("myapp", items)
print("Inserted " .. count .. " items")

-- Batch select
local values = batch_select("myapp", {"item:1", "item:2", "item:3"})
print("Retrieved: " .. liath.util.inspect(values))

-- Scan with prefix
print("\nScanning 'item:' prefix:")
local scanned = scan("myapp", "item:", 10)
for _, entry in ipairs(scanned) do
    print("  " .. entry.key .. " = " .. entry.value)
end

print("")

-- ============================================================
-- 9. UTILITY FUNCTIONS
-- ============================================================
print("9. Utility Functions")
print("--------------------")

print("UUID: " .. uuid())
print("Timestamp: " .. timestamp())

local data = { a = 1, b = "hello", c = { nested = true } }
print("Inspected table: " .. liath.util.inspect(data))

-- Functional utilities
local numbers = {1, 2, 3, 4, 5}
local doubled = liath.util.map(numbers, function(n) return n * 2 end)
local evens = liath.util.filter(numbers, function(n) return n % 2 == 0 end)
local sum = liath.util.reduce(numbers, function(acc, n) return acc + n end, 0)

print("Doubled: " .. table.concat(doubled, ", "))
print("Evens: " .. table.concat(evens, ", "))
print("Sum: " .. sum)

print("")

-- ============================================================
-- 10. PERSISTENCE
-- ============================================================
print("10. Persistence")
print("---------------")

-- Save all data to disk
save()
print("All data saved to disk!")

print("\n=== Examples Complete ===")
