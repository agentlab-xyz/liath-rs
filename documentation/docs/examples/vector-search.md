# Vector Search Examples

Advanced examples for semantic search and vector operations.

## Basic Semantic Search

```rust
use liath::{EmbeddedLiath, Config};
use usearch::{MetricKind, ScalarKind};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = EmbeddedLiath::new(Config::default())?;
    db.create_namespace("docs", 384, MetricKind::Cos, ScalarKind::F32)?;

    // Store documents
    let documents = vec![
        (1, "Machine learning algorithms for data analysis"),
        (2, "Introduction to neural network architectures"),
        (3, "Natural language processing with transformers"),
        (4, "Computer vision and image recognition"),
        (5, "Reinforcement learning in robotics"),
        (6, "Statistical analysis and data visualization"),
        (7, "Database design and optimization"),
        (8, "Cloud computing infrastructure"),
    ];

    for (id, content) in &documents {
        db.store_with_embedding("docs", *id, format!("doc:{}", id).as_bytes(), content)?;
    }

    // Search
    let query = "deep learning neural networks";
    let results = db.semantic_search("docs", query, 3)?;

    println!("Query: {}\n", query);
    for (id, content, distance) in results {
        println!("[{:.3}] {}", distance, content);
    }

    Ok(())
}
```

## Multi-Namespace Search

```lua
-- Search across multiple namespaces
local function multi_search(namespaces, query, k_per_ns)
    local all_results = {}

    for _, ns in ipairs(namespaces) do
        local results = semantic_search(ns, query, k_per_ns)
        for _, r in ipairs(results) do
            r.namespace = ns
            table.insert(all_results, r)
        end
    end

    -- Sort by distance
    table.sort(all_results, function(a, b)
        return a.distance < b.distance
    end)

    return all_results
end

-- Usage
local results = multi_search(
    {"documents", "memories", "notes"},
    "machine learning",
    5
)

return json.encode(results)
```

## Filtered Search

```lua
-- Search with metadata filtering
local function filtered_search(namespace, query, k, filter_fn)
    -- Get more results than needed for filtering
    local results = semantic_search(namespace, query, k * 3)

    -- Apply filter
    local filtered = {}
    for _, r in ipairs(results) do
        local meta_json = get(namespace .. ":meta", r.id)
        local meta = meta_json and json.decode(meta_json) or {}

        if filter_fn(r, meta) then
            r.metadata = meta
            table.insert(filtered, r)
        end

        if #filtered >= k then
            break
        end
    end

    return filtered
end

-- Example: Filter by category
local results = filtered_search("docs", "programming", 5, function(r, meta)
    return meta.category == "technology"
end)

-- Example: Filter by date
local results = filtered_search("docs", "news", 10, function(r, meta)
    return meta.date and meta.date > "2024-01-01"
end)
```

## Hybrid Search

```lua
-- Combine semantic and keyword search
local function hybrid_search(namespace, query, k)
    -- Semantic search
    local semantic_results = semantic_search(namespace, query, k * 2)

    -- Extract keywords
    local keywords = {}
    for word in query:gmatch("%w+") do
        table.insert(keywords, word:lower())
    end

    -- Score results
    local scored = {}
    for _, r in ipairs(semantic_results) do
        local semantic_score = 1 - r.distance
        local keyword_score = 0

        -- Count keyword matches
        local content_lower = r.content:lower()
        for _, kw in ipairs(keywords) do
            if content_lower:find(kw, 1, true) then
                keyword_score = keyword_score + 0.1
            end
        end

        -- Combined score (70% semantic, 30% keyword)
        local combined_score = (semantic_score * 0.7) + (keyword_score * 0.3)

        table.insert(scored, {
            id = r.id,
            content = r.content,
            score = combined_score,
            semantic_score = semantic_score,
            keyword_score = keyword_score
        })
    end

    -- Sort by combined score
    table.sort(scored, function(a, b) return a.score > b.score end)

    return {unpack(scored, 1, k)}
end

return json.encode(hybrid_search("docs", "machine learning python", 5))
```

## Similarity Clustering

```lua
-- Find clusters of similar documents
local function find_similar_clusters(namespace, seed_ids, similarity_threshold)
    local clusters = {}

    for _, seed_id in ipairs(seed_ids) do
        local seed_content = get(namespace, seed_id)
        if seed_content then
            local cluster = {seed_id}
            local similar = semantic_search(namespace, seed_content, 20)

            for _, r in ipairs(similar) do
                if r.id ~= seed_id and r.distance < similarity_threshold then
                    table.insert(cluster, r.id)
                end
            end

            table.insert(clusters, {
                seed = seed_id,
                members = cluster,
                size = #cluster
            })
        end
    end

    return clusters
end

return json.encode(find_similar_clusters("docs", {"doc:1", "doc:5", "doc:10"}, 0.3))
```

## Query Expansion

```lua
-- Expand query with related terms
local function expanded_search(namespace, query, k)
    -- Generate query variations
    local queries = {
        query,
        "what is " .. query,
        query .. " definition",
        query .. " explanation",
        "how does " .. query .. " work"
    }

    local all_results = {}
    local seen_ids = {}

    for _, q in ipairs(queries) do
        local results = semantic_search(namespace, q, k)
        for _, r in ipairs(results) do
            if not seen_ids[r.id] then
                seen_ids[r.id] = true
                table.insert(all_results, r)
            end
        end
    end

    -- Sort by distance
    table.sort(all_results, function(a, b)
        return a.distance < b.distance
    end)

    return {unpack(all_results, 1, k)}
end

return json.encode(expanded_search("docs", "neural networks", 5))
```

## Re-ranking Results

```lua
-- Re-rank results using multiple signals
local function rerank_search(namespace, query, k)
    local results = semantic_search(namespace, query, k * 3)

    local reranked = {}
    for _, r in ipairs(results) do
        local meta_json = get(namespace .. ":meta", r.id)
        local meta = meta_json and json.decode(meta_json) or {}

        -- Calculate scores
        local relevance = 1 - r.distance

        -- Recency score (newer = higher)
        local age_days = meta.age_days or 30
        local recency = math.exp(-age_days / 30)

        -- Quality score
        local quality = meta.quality or 0.5

        -- Popularity score
        local views = meta.views or 0
        local popularity = math.min(1, views / 1000)

        -- Combined score
        local score = (relevance * 0.5) +
                      (recency * 0.2) +
                      (quality * 0.2) +
                      (popularity * 0.1)

        table.insert(reranked, {
            id = r.id,
            content = r.content,
            score = score,
            factors = {
                relevance = relevance,
                recency = recency,
                quality = quality,
                popularity = popularity
            }
        })
    end

    table.sort(reranked, function(a, b) return a.score > b.score end)

    return {unpack(reranked, 1, k)}
end

return json.encode(rerank_search("articles", "machine learning", 10))
```

## Batch Vector Operations

```rust
use liath::{EmbeddedLiath, Config};
use usearch::{MetricKind, ScalarKind};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = EmbeddedLiath::new(Config::default())?;
    db.create_namespace("batch_test", 384, MetricKind::Cos, ScalarKind::F32)?;

    // Batch generate embeddings
    let texts: Vec<&str> = (0..100)
        .map(|i| format!("Document number {} about various topics", i))
        .collect::<Vec<_>>()
        .iter()
        .map(|s| s.as_str())
        .collect();

    let embeddings = db.generate_embeddings(&texts)?;

    // Batch store
    for (i, embedding) in embeddings.iter().enumerate() {
        db.add_vector("batch_test", i as u64, embedding)?;
        db.put("batch_test", format!("doc:{}", i).as_bytes(),
               texts[i].as_bytes())?;
    }

    println!("Stored {} documents", texts.len());

    // Search
    let query_vec = db.generate_embedding("document topics")?;
    let results = db.search_vectors("batch_test", &query_vec, 5)?;

    for (id, distance) in results {
        println!("ID: {}, Distance: {:.4}", id, distance);
    }

    Ok(())
}
```

## Nearest Neighbor Chains

```lua
-- Find chain of related documents
local function find_chain(namespace, start_id, length)
    local chain = {start_id}
    local visited = {[start_id] = true}
    local current = start_id

    for i = 2, length do
        local content = get(namespace, current)
        if not content then break end

        local neighbors = semantic_search(namespace, content, 10)

        -- Find nearest unvisited neighbor
        local next_id = nil
        for _, n in ipairs(neighbors) do
            if not visited[n.id] then
                next_id = n.id
                break
            end
        end

        if not next_id then break end

        table.insert(chain, next_id)
        visited[next_id] = true
        current = next_id
    end

    return chain
end

return json.encode(find_chain("docs", "doc:1", 5))
```

## Distance Threshold Search

```lua
-- Search with distance threshold instead of fixed k
local function threshold_search(namespace, query, max_distance, max_results)
    max_results = max_results or 100

    local results = semantic_search(namespace, query, max_results)
    local filtered = {}

    for _, r in ipairs(results) do
        if r.distance <= max_distance then
            table.insert(filtered, r)
        else
            break  -- Results are sorted, so we can stop
        end
    end

    return filtered
end

-- Find very similar documents (distance < 0.3)
local similar = threshold_search("docs", "machine learning", 0.3)

-- Find somewhat related (distance < 0.5)
local related = threshold_search("docs", "machine learning", 0.5)

return json.encode({
    very_similar = #similar,
    somewhat_related = #related
})
```

## Next Steps

- [Agent Patterns](agent-patterns.md) - Use search in agents
- [RAG Applications](rag.md) - Build RAG systems
- [Performance Guide](../guides/performance.md) - Optimize search
