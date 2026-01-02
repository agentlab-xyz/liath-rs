# RAG Applications

Examples for building Retrieval-Augmented Generation systems with Liath.

## Basic RAG System

```rust
use liath::{EmbeddedLiath, Config};
use usearch::{MetricKind, ScalarKind};

pub struct RAGSystem {
    db: EmbeddedLiath,
}

impl RAGSystem {
    pub fn new(data_dir: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config = Config {
            data_dir: data_dir.into(),
            ..Default::default()
        };
        let db = EmbeddedLiath::new(config)?;
        db.create_namespace("knowledge", 384, MetricKind::Cos, ScalarKind::F32)?;

        Ok(Self { db })
    }

    pub fn add_document(&self, doc_id: &str, content: &str, metadata: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Store with embedding
        self.db.store_with_embedding(
            "knowledge",
            doc_id.parse()?,
            doc_id.as_bytes(),
            content
        )?;

        // Store metadata
        self.db.put("knowledge:meta", doc_id.as_bytes(), metadata.as_bytes())?;

        Ok(())
    }

    pub fn retrieve(&self, query: &str, k: usize) -> Result<Vec<(String, f32)>, Box<dyn std::error::Error>> {
        let results = self.db.semantic_search("knowledge", query, k)?;
        Ok(results.into_iter().map(|(_, content, dist)| (content, dist)).collect())
    }

    pub fn build_context(&self, query: &str, k: usize) -> Result<String, Box<dyn std::error::Error>> {
        let results = self.retrieve(query, k)?;

        let context = results.iter()
            .enumerate()
            .map(|(i, (content, _))| format!("[{}] {}", i + 1, content))
            .collect::<Vec<_>>()
            .join("\n\n");

        Ok(context)
    }

    pub fn build_prompt(&self, query: &str, k: usize) -> Result<String, Box<dyn std::error::Error>> {
        let context = self.build_context(query, k)?;

        Ok(format!(r#"
Use the following context to answer the question. If the context doesn't contain the answer, say "I don't have information about that."

Context:
{}

Question: {}

Answer:"#, context, query))
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rag = RAGSystem::new("./rag_data")?;

    // Add documents
    rag.add_document("doc:1", "Rust is a systems programming language focused on safety, concurrency, and performance.", r#"{"source": "docs"}"#)?;
    rag.add_document("doc:2", "Python is a high-level programming language known for its simplicity and readability.", r#"{"source": "docs"}"#)?;
    rag.add_document("doc:3", "Machine learning is a subset of artificial intelligence that enables systems to learn from data.", r#"{"source": "wiki"}"#)?;

    // Build prompt
    let prompt = rag.build_prompt("What is Rust?", 3)?;
    println!("{}", prompt);

    Ok(())
}
```

## Chunked Document RAG

```rust
use liath::{EmbeddedLiath, Config};

pub struct ChunkedRAG {
    db: EmbeddedLiath,
    chunk_size: usize,
    chunk_overlap: usize,
}

impl ChunkedRAG {
    pub fn new(data_dir: &str, chunk_size: usize, chunk_overlap: usize) -> Result<Self, Box<dyn std::error::Error>> {
        let db = EmbeddedLiath::new(Config {
            data_dir: data_dir.into(),
            ..Default::default()
        })?;

        Ok(Self { db, chunk_size, chunk_overlap })
    }

    fn chunk_text(&self, text: &str) -> Vec<String> {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut chunks = Vec::new();

        let mut i = 0;
        while i < words.len() {
            let end = std::cmp::min(i + self.chunk_size, words.len());
            let chunk = words[i..end].join(" ");
            chunks.push(chunk);

            if end >= words.len() {
                break;
            }

            i += self.chunk_size - self.chunk_overlap;
        }

        chunks
    }

    pub fn add_document(&self, doc_id: &str, content: &str) -> Result<usize, Box<dyn std::error::Error>> {
        let chunks = self.chunk_text(content);

        for (i, chunk) in chunks.iter().enumerate() {
            let chunk_id = format!("{}:chunk:{}", doc_id, i);
            self.db.store_with_embedding(
                "knowledge",
                (doc_id.parse::<u64>()? * 1000 + i as u64),
                chunk_id.as_bytes(),
                chunk
            )?;

            // Store chunk metadata
            self.db.put(
                "knowledge:meta",
                chunk_id.as_bytes(),
                serde_json::to_vec(&serde_json::json!({
                    "doc_id": doc_id,
                    "chunk_index": i,
                    "total_chunks": chunks.len()
                }))?.as_slice()
            )?;
        }

        Ok(chunks.len())
    }

    pub fn retrieve_with_context(&self, query: &str, k: usize) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let results = self.db.semantic_search("knowledge", query, k)?;

        // Group by document and get surrounding chunks
        let mut enriched = Vec::new();
        for (_, content, _) in results {
            enriched.push(content);
        }

        Ok(enriched)
    }
}
```

## RAG with Metadata Filtering

```lua
-- RAG with metadata-based filtering
local RAG = {}

function RAG:new()
    local rag = {}
    setmetatable(rag, {__index = RAG})
    return rag
end

function RAG:add_document(doc_id, content, metadata)
    store_with_embedding("rag:docs", doc_id, content)
    put("rag:meta", doc_id, json.encode(metadata))
end

function RAG:retrieve(query, k, filters)
    -- Get more results to filter
    local results = semantic_search("rag:docs", query, k * 3)

    local filtered = {}
    for _, r in ipairs(results) do
        local meta = json.decode(get("rag:meta", r.id) or '{}')

        local matches = true
        if filters then
            -- Apply filters
            if filters.source and meta.source ~= filters.source then
                matches = false
            end
            if filters.min_date and meta.date < filters.min_date then
                matches = false
            end
            if filters.category and meta.category ~= filters.category then
                matches = false
            end
        end

        if matches then
            r.metadata = meta
            table.insert(filtered, r)
        end

        if #filtered >= k then
            break
        end
    end

    return filtered
end

function RAG:build_context(query, k, filters)
    local results = self:retrieve(query, k, filters)

    local context_parts = {}
    for i, r in ipairs(results) do
        local source = r.metadata.source or "unknown"
        table.insert(context_parts, string.format(
            "[%d] (Source: %s)\n%s",
            i, source, r.content
        ))
    end

    return table.concat(context_parts, "\n\n")
end

function RAG:build_prompt(query, k, filters, system_prompt)
    local context = self:build_context(query, k, filters)

    system_prompt = system_prompt or "Answer based on the provided context."

    return json.encode({
        system = system_prompt,
        context = context,
        query = query
    })
end

-- Usage
local rag = RAG:new()

-- Add documents
rag:add_document("doc:1", "Rust memory safety prevents common bugs", {
    source = "documentation",
    category = "programming",
    date = "2024-01-15"
})

rag:add_document("doc:2", "Python GIL affects multi-threading performance", {
    source = "blog",
    category = "programming",
    date = "2024-02-01"
})

-- Retrieve with filters
local prompt = rag:build_prompt(
    "memory safety",
    5,
    {source = "documentation"},
    "You are a programming expert. Answer concisely."
)

return prompt
```

## Conversational RAG

```rust
use liath::{EmbeddedLiath, Config};
use liath::agent::{Agent, Role};
use std::sync::Arc;

pub struct ConversationalRAG {
    db: Arc<EmbeddedLiath>,
    agent: Agent,
}

impl ConversationalRAG {
    pub fn new(data_dir: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let db = Arc::new(EmbeddedLiath::new(Config {
            data_dir: data_dir.into(),
            ..Default::default()
        })?);

        let agent = Agent::new("rag-agent", db.clone());

        Ok(Self { db, agent })
    }

    pub fn add_document(&self, doc_id: &str, content: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.db.store_with_embedding(
            "knowledge",
            doc_id.parse()?,
            doc_id.as_bytes(),
            content
        )?;
        Ok(())
    }

    pub fn chat(&self, conv_id: Option<&str>, message: &str) -> Result<String, Box<dyn std::error::Error>> {
        let conv = self.agent.conversation(conv_id)?;

        // Add user message
        conv.add_message(Role::User, message)?;

        // Retrieve relevant context
        let results = self.db.semantic_search("knowledge", message, 3)?;
        let context: Vec<String> = results.iter()
            .map(|(_, content, _)| content.clone())
            .collect();

        // Get conversation history
        let history = conv.last_n(5)?;
        let history_text: Vec<String> = history.iter()
            .map(|m| format!("{:?}: {}", m.role, m.content))
            .collect();

        // Build response data
        let response_data = serde_json::json!({
            "context": context,
            "history": history_text,
            "query": message
        });

        Ok(serde_json::to_string_pretty(&response_data)?)
    }
}
```

## RAG with Re-ranking

```lua
-- RAG with two-stage retrieval and re-ranking
local function rag_with_reranking(query, k)
    -- Stage 1: Broad retrieval
    local initial_k = k * 5
    local candidates = semantic_search("rag:docs", query, initial_k)

    -- Stage 2: Re-rank with multiple signals
    local scored = {}
    for _, r in ipairs(candidates) do
        local meta = json.decode(get("rag:meta", r.id) or '{}')

        -- Semantic relevance (primary signal)
        local relevance = 1 - r.distance

        -- Keyword overlap
        local keywords = {}
        for word in query:lower():gmatch("%w+") do
            keywords[word] = true
        end
        local keyword_score = 0
        for word in r.content:lower():gmatch("%w+") do
            if keywords[word] then
                keyword_score = keyword_score + 0.05
            end
        end

        -- Source quality
        local source_quality = {
            documentation = 1.0,
            wiki = 0.9,
            blog = 0.7,
            forum = 0.5
        }
        local quality = source_quality[meta.source] or 0.6

        -- Recency
        local recency = 1.0
        if meta.date then
            -- Simplified recency calculation
            recency = 0.8 + math.random() * 0.2
        end

        -- Combined score
        local score = (relevance * 0.5) +
                      (keyword_score * 0.2) +
                      (quality * 0.2) +
                      (recency * 0.1)

        table.insert(scored, {
            id = r.id,
            content = r.content,
            score = score,
            factors = {
                relevance = relevance,
                keyword = keyword_score,
                quality = quality,
                recency = recency
            }
        })
    end

    -- Sort by score
    table.sort(scored, function(a, b) return a.score > b.score end)

    -- Return top k
    local results = {}
    for i = 1, math.min(k, #scored) do
        table.insert(results, scored[i])
    end

    return results
end

-- Build context with re-ranked results
local function build_context(query, k)
    local results = rag_with_reranking(query, k)

    local parts = {}
    for i, r in ipairs(results) do
        table.insert(parts, string.format(
            "[%d] (Score: %.2f)\n%s",
            i, r.score, r.content
        ))
    end

    return table.concat(parts, "\n\n")
end

return build_context("machine learning models", 5)
```

## Streaming RAG Response

```rust
use liath::{EmbeddedLiath, Config};
use tokio::sync::mpsc;

pub struct StreamingRAG {
    db: EmbeddedLiath,
}

impl StreamingRAG {
    pub async fn stream_response(
        &self,
        query: &str,
        tx: mpsc::Sender<String>
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Send "retrieving" status
        tx.send("Retrieving relevant documents...".to_string()).await?;

        // Retrieve
        let results = self.db.semantic_search("knowledge", query, 5)?;

        // Send context
        tx.send(format!("Found {} relevant documents", results.len())).await?;

        for (i, (_, content, distance)) in results.iter().enumerate() {
            tx.send(format!(
                "\n[{}] (relevance: {:.0}%)\n{}",
                i + 1,
                (1.0 - distance) * 100.0,
                content
            )).await?;
        }

        // Send completion
        tx.send("\n---\nRetrieval complete. Generating response...".to_string()).await?;

        Ok(())
    }
}
```

## Next Steps

- [Multi-Agent Systems](multi-agent.md) - Complex agent architectures
- [Vector Search](vector-search.md) - Advanced search patterns
- [Performance Guide](../guides/performance.md) - Optimize RAG
