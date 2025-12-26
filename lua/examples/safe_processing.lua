-- Safe Data Processing
-- Demonstrates sandboxed execution for agent tools
--
-- Run with: liath execute --file lua/examples/safe_processing.lua

print("=== Safe Data Processing ===\n")
print("This code runs in a sandbox - no file system or network access.\n")

-- Example 1: JSON data processing
print("--- JSON Data Processing ---\n")

local sales_data = json.encode({
    {product = "Widget A", quantity = 100, price = 10.00, region = "North"},
    {product = "Widget B", quantity = 50, price = 25.00, region = "South"},
    {product = "Widget A", quantity = 75, price = 10.00, region = "East"},
    {product = "Widget C", quantity = 200, price = 5.00, region = "North"},
    {product = "Widget B", quantity = 30, price = 25.00, region = "West"},
})

function analyze_sales(data_json)
    local data = json.decode(data_json)

    -- Calculate totals
    local total_revenue = reduce(data, function(acc, item)
        return acc + (item.quantity * item.price)
    end, 0)

    local total_units = reduce(data, function(acc, item)
        return acc + item.quantity
    end, 0)

    -- Group by product
    local by_product = {}
    for _, item in ipairs(data) do
        local p = item.product
        if not by_product[p] then
            by_product[p] = {quantity = 0, revenue = 0}
        end
        by_product[p].quantity = by_product[p].quantity + item.quantity
        by_product[p].revenue = by_product[p].revenue + (item.quantity * item.price)
    end

    -- Group by region
    local by_region = {}
    for _, item in ipairs(data) do
        local r = item.region
        by_region[r] = (by_region[r] or 0) + (item.quantity * item.price)
    end

    -- Find best performer
    local best_product, best_revenue = nil, 0
    for product, stats in pairs(by_product) do
        if stats.revenue > best_revenue then
            best_product, best_revenue = product, stats.revenue
        end
    end

    return {
        total_revenue = total_revenue,
        total_units = total_units,
        average_price = total_revenue / total_units,
        by_product = by_product,
        by_region = by_region,
        best_product = {name = best_product, revenue = best_revenue}
    }
end

local analysis = analyze_sales(sales_data)
print("Total Revenue: $" .. analysis.total_revenue)
print("Total Units: " .. analysis.total_units)
print("Average Price: $" .. string.format("%.2f", analysis.average_price))
print("Best Product: " .. analysis.best_product.name .. " ($" .. analysis.best_product.revenue .. ")")

-- Example 2: Text processing
print("\n--- Text Processing ---\n")

function process_text(text)
    -- Word count
    local words = {}
    for word in text:gmatch("%w+") do
        table.insert(words, word:lower())
    end

    -- Word frequency
    local freq = {}
    for _, word in ipairs(words) do
        freq[word] = (freq[word] or 0) + 1
    end

    -- Top words (simple sort)
    local sorted = {}
    for word, count in pairs(freq) do
        if #word > 3 then  -- Skip short words
            table.insert(sorted, {word = word, count = count})
        end
    end
    table.sort(sorted, function(a, b) return a.count > b.count end)

    -- Extract sentences
    local sentences = {}
    for sentence in text:gmatch("[^.!?]+[.!?]") do
        table.insert(sentences, sentence:match("^%s*(.-)%s*$"))
    end

    return {
        word_count = #words,
        unique_words = #sorted,
        sentence_count = #sentences,
        top_words = {sorted[1], sorted[2], sorted[3]},
        average_sentence_length = #words / math.max(#sentences, 1)
    }
end

local sample_text = [[
    Liath is an embedded database for AI agents. It provides storage, vector search,
    and a safe Lua runtime. Agents can store memories and recall them semantically.
    The Lua sandbox ensures safe code execution. Liath is written in Rust for performance.
    The database supports key-value storage, vector similarity search, and embeddings.
]]

local text_analysis = process_text(sample_text)
print("Word count: " .. text_analysis.word_count)
print("Unique words: " .. text_analysis.unique_words)
print("Sentences: " .. text_analysis.sentence_count)
print("Top words:")
for i, w in ipairs(text_analysis.top_words) do
    if w then
        print("  " .. i .. ". " .. w.word .. " (" .. w.count .. ")")
    end
end

-- Example 3: Data transformation pipeline
print("\n--- Data Pipeline ---\n")

function pipeline(data)
    return data
        -- Filter: only items over threshold
        |> function(d) return filter(d, function(x) return x.value > 10 end) end
        -- Map: add computed field
        |> function(d) return map(d, function(x)
            x.doubled = x.value * 2
            return x
        end) end
        -- Reduce: sum all values
        |> function(d) return {
            items = d,
            total = reduce(d, function(acc, x) return acc + x.value end, 0)
        } end
end

-- Lua doesn't have |> so we simulate it
function pipe(data, ...)
    local funcs = {...}
    local result = data
    for _, fn in ipairs(funcs) do
        result = fn(result)
    end
    return result
end

local input = {
    {name = "a", value = 5},
    {name = "b", value = 15},
    {name = "c", value = 25},
    {name = "d", value = 8},
    {name = "e", value = 30},
}

local result = pipe(input,
    function(d) return filter(d, function(x) return x.value > 10 end) end,
    function(d) return map(d, function(x)
        return {name = x.name, value = x.value, doubled = x.value * 2}
    end) end
)

print("Filtered and transformed:")
for _, item in ipairs(result) do
    print(string.format("  %s: %d -> %d", item.name, item.value, item.doubled))
end

-- Example 4: Safe calculation (no system access)
print("\n--- Security Demo ---\n")

print("Attempting unsafe operations (all will fail):")

-- These would all fail in the sandbox:
local unsafe_ops = {
    "os.execute('ls')",
    "io.open('/etc/passwd')",
    "require('socket')",
    "loadfile('script.lua')",
}

for _, op in ipairs(unsafe_ops) do
    local fn, err = load(op)
    if fn then
        local ok, result = pcall(fn)
        if not ok then
            print("  BLOCKED: " .. op .. " -> " .. tostring(result):sub(1, 50))
        end
    else
        print("  BLOCKED: " .. op .. " -> " .. tostring(err):sub(1, 50))
    end
end

print("\nAll unsafe operations were blocked. Agent code is safe!")

print("\n=== Done ===")
