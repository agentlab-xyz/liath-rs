-- Conversation Manager
-- Demonstrates conversation tracking with memory integration
--
-- Run with: liath execute --file lua/examples/conversation.lua

print("=== Conversation Manager ===\n")

-- Conversation manager class-like structure
Conversation = {}

function Conversation.new(conv_id, agent_id)
    local self = {
        conv_id = conv_id,
        agent_id = agent_id,
        turn_count = 0
    }

    return setmetatable(self, {__index = Conversation})
end

function Conversation:add_user_message(content)
    add_message(self.conv_id, "user", content)
    self.turn_count = self.turn_count + 1

    -- Also store as memory
    store_with_embedding(self.agent_id .. ":memories", id(),
        "User said: " .. content)
end

function Conversation:add_assistant_message(content)
    add_message(self.conv_id, "assistant", content)

    -- Store important assistant actions as memories
    if #content > 100 then
        store_with_embedding(self.agent_id .. ":memories", id(),
            "I responded about: " .. content:sub(1, 100))
    end
end

function Conversation:get_history(limit)
    return get_messages(self.conv_id, limit or 20)
end

function Conversation:get_relevant_memories(query, limit)
    return semantic_search(self.agent_id .. ":memories", query, limit or 5)
end

function Conversation:build_context(current_message)
    -- Get conversation history
    local history = self:get_history(10)

    -- Get relevant memories
    local memories = self:get_relevant_memories(current_message, 3)

    -- Format history
    local history_text = {}
    for _, msg in ipairs(history) do
        table.insert(history_text, msg.role .. ": " .. msg.content)
    end

    -- Format memories
    local memory_text = {}
    for _, mem in ipairs(memories) do
        table.insert(memory_text, "- " .. mem.content)
    end

    return {
        history = table.concat(history_text, "\n"),
        memories = table.concat(memory_text, "\n"),
        current_message = current_message,
        turn_count = self.turn_count
    }
end

function Conversation:summarize_and_reset()
    local history = self:get_history(100)

    if #history < 10 then
        return nil  -- Not enough to summarize
    end

    -- Simple summary (in real usage, you'd use an LLM)
    local topics = {}
    for _, msg in ipairs(history) do
        -- Extract first few words as pseudo-topic
        local words = msg.content:match("^(%S+ %S+ %S+)")
        if words then
            topics[words] = true
        end
    end

    local topic_list = {}
    for topic, _ in pairs(topics) do
        table.insert(topic_list, topic)
    end

    local summary = "Previous conversation covered: " .. table.concat(topic_list, ", ")

    -- Clear and restart with summary
    clear_conversation(self.conv_id)
    add_message(self.conv_id, "system", summary)
    self.turn_count = 0

    return summary
end

-- Demo conversation
print("--- Starting Conversation ---\n")

local conv = Conversation.new("demo-chat", "demo-agent")

-- Simulate a conversation
conv:add_user_message("Hi! I'm working on a Rust project and need help with error handling.")
conv:add_assistant_message("Hello! I'd be happy to help with Rust error handling. Rust uses the Result<T, E> type for operations that can fail. Would you like me to explain the basics or do you have a specific question?")

conv:add_user_message("Can you show me how to use the ? operator?")
conv:add_assistant_message("The ? operator is syntactic sugar for error propagation. Instead of match on Result, you can use ? to automatically return the error if it's Err, or unwrap the Ok value. For example: let file = File::open('path')?; This requires your function to return Result.")

conv:add_user_message("What about custom error types?")
conv:add_assistant_message("For custom errors, you can define an enum implementing std::error::Error. The thiserror crate makes this easier with derive macros. For example: #[derive(Error, Debug)] enum MyError { #[error('IO error')] Io(#[from] std::io::Error) }")

conv:add_user_message("How do I convert between error types?")
conv:add_assistant_message("You can use From/Into traits for conversion, or the map_err method on Result. The anyhow crate is great for applications where you just need to propagate errors without defining custom types.")

-- Show conversation state
print("--- Conversation History ---\n")
local history = conv:get_history(10)
for _, msg in ipairs(history) do
    print(msg.role .. ": " .. msg.content:sub(1, 80) .. (msg.content:len() > 80 and "..." or ""))
end

-- Build context for next turn
print("\n--- Building Context for Next Turn ---\n")
local context = conv:build_context("What's the difference between anyhow and thiserror?")
print("Turn count: " .. context.turn_count)
print("\nRelevant memories from conversation:")
print(context.memories)

-- Demonstrate memory recall
print("\n--- Memory Recall Demo ---\n")
local memories = conv:get_relevant_memories("error handling in Rust", 3)
print("Searching memories for 'error handling in Rust':")
for _, mem in ipairs(memories) do
    print("  - " .. mem.content:sub(1, 60) .. "...")
end

print("\n=== Done ===")
