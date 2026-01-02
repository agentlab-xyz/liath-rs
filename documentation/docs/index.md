# Liath

**The SQLite for AI Agents - Programmable Memory Database**

Liath is a high-performance, embedded database designed specifically for AI agents. It enables agents to write Lua code to intelligently query their memory, rather than being limited to fixed APIs.

## Why Liath?

Traditional vector databases offer fixed APIs:

```python
# Traditional approach
results = semantic_search("query", limit=5)
```

Liath offers **programmable memory** - agents generate code to implement custom retrieval strategies:

```lua
-- Agent-generated code
local relevant = semantic_search("memory", query, 10)
local enriched = {}

for _, r in ipairs(relevant) do
    local meta = json.decode(get("meta", r.id))
    local recency_boost = meta.age_days <= 7 and 2.0 or 1.0
    local score = (1 - r.distance) * meta.importance * recency_boost
    table.insert(enriched, {content = r.content, score = score})
end

table.sort(enriched, function(a, b) return a.score > b.score end)
return json.encode(enriched)
```

The agent can combine semantic similarity, recency weighting, importance scoring, and cross-referencing - all safely sandboxed within Lua.

## Key Features

- **Programmable Memory**: Agents write Lua code to query their own memory
- **Safe Execution**: Sandboxed Lua runtime blocks all system access
- **Embedded Database**: Zero-config, single dependency, no server required
- **Vector Search**: Built-in semantic search with HNSW indices
- **Agent API**: First-class support for memory, conversations, and tool state
- **Fast Embeddings**: Built-in ONNX-based text embedding generation

## Quick Example

```rust
use liath::{EmbeddedLiath, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = EmbeddedLiath::new(Config::default())?;

    // Store data
    db.put("notes", b"key1", b"Hello, Liath!")?;

    // Execute agent-generated Lua code
    let code = r#"
        store_with_embedding("docs", "d1", "Important meeting notes")
        local results = semantic_search("docs", "meeting", 5)
        return json.encode(results)
    "#;

    let result = db.query_executor().execute(code, "agent").await?;
    println!("{}", result);

    Ok(())
}
```

## Use Cases

- **AI Agent Memory Systems**: Long-term learning for agent interactions
- **Retrieval-Augmented Generation (RAG)**: Store and retrieve context documents
- **Semantic Search**: Find similar content across stored documents
- **Conversation Management**: Maintain dialogue history with context
- **Tool State Persistence**: Remember tool states across invocations

## Getting Started

<div class="grid cards" markdown>

-   :material-download:{ .lg .middle } **Installation**

    ---

    Add Liath to your Rust project

    [:octicons-arrow-right-24: Installation Guide](getting-started/installation.md)

-   :material-clock-fast:{ .lg .middle } **Quick Start**

    ---

    Get up and running in 5 minutes

    [:octicons-arrow-right-24: Quick Start](getting-started/quick-start.md)

-   :material-brain:{ .lg .middle } **Programmable Memory**

    ---

    Understand the core concept

    [:octicons-arrow-right-24: Learn More](concepts/programmable-memory.md)

-   :material-code-braces:{ .lg .middle } **Examples**

    ---

    Explore practical examples

    [:octicons-arrow-right-24: Examples](examples/index.md)

</div>
