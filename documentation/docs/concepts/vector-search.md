# Vector Search

Liath uses vector search to enable semantic similarity queries across your data. This guide explains how vector search works and how to use it effectively.

## Overview

Vector search finds items that are semantically similar to a query, even if they don't share exact keywords. It works by:

1. Converting text to dense vectors (embeddings)
2. Storing vectors in an efficient index
3. Finding nearest neighbors to a query vector

```mermaid
graph LR
    A[Query Text] --> B[Embedding Model]
    B --> C[Query Vector]
    C --> D[HNSW Index]
    D --> E[Nearest Neighbors]
    E --> F[Results + Content]
```

## HNSW Index

Liath uses the HNSW (Hierarchical Navigable Small World) algorithm via the USearch library.

### How HNSW Works

HNSW builds a multi-layer graph structure:

```
Layer 2:  o -------- o
          |          |
Layer 1:  o -- o --- o -- o
          |    |     |    |
Layer 0:  o-o--o-o-o-o-o--o-o
```

- Higher layers have fewer, well-connected nodes for fast navigation
- Lower layers have more nodes for precise search
- Search starts at the top and descends to find nearest neighbors

### Performance Characteristics

| Operation | Complexity | Typical Time |
|-----------|------------|--------------|
| Search | O(log N) | < 1ms for 100K vectors |
| Insert | O(log N) | ~1ms per vector |
| Memory | O(N * D) | ~1.5KB per 384-dim vector (F32) |

## Using Vector Search

### Basic Semantic Search

```rust
use liath::EmbeddedLiath;

let db = EmbeddedLiath::new(Config::default())?;

// Store documents with embeddings
db.store_with_embedding("docs", 1, b"doc:1", "Rust is a systems programming language")?;
db.store_with_embedding("docs", 2, b"doc:2", "Python is great for data science")?;
db.store_with_embedding("docs", 3, b"doc:3", "JavaScript runs in browsers")?;

// Semantic search
let results = db.semantic_search("docs", "low-level programming", 2)?;

for (id, content, distance) in results {
    println!("ID: {}, Distance: {:.4}", id, distance);
    println!("Content: {}", content);
}
```

### Raw Vector Search

For more control, work with vectors directly:

```rust
// Generate embedding
let query_vec = db.generate_embedding("systems programming")?;

// Search with raw vector
let results = db.search_vectors("docs", &query_vec, 5)?;

for (id, distance) in results {
    println!("ID: {}, Distance: {:.4}", id, distance);
}
```

### Via Lua

```lua
-- Store with embedding
store_with_embedding("docs", "doc1", "Rust is a systems programming language")
store_with_embedding("docs", "doc2", "Python is great for data science")

-- Semantic search
local results = semantic_search("docs", "low-level programming", 2)

for _, r in ipairs(results) do
    print(r.id, r.content, r.distance)
end

-- Raw vector search
local query_vec = embed("systems programming")
local raw_results = vector_search("docs", query_vec, 5)
```

## Distance Metrics

### Cosine Similarity

**Best for:** Text embeddings, normalized vectors

Measures the angle between two vectors:

```
cos(θ) = (A · B) / (||A|| * ||B||)
```

- Range: 0 to 2 (as distance), 0 means identical
- Ignores magnitude, focuses on direction
- Recommended for most text applications

```rust
db.create_namespace("text", 384, MetricKind::Cos, ScalarKind::F32)?;
```

### Euclidean Distance (L2)

**Best for:** Image features, absolute positions

Measures straight-line distance:

```
d = sqrt(Σ(a_i - b_i)²)
```

- Range: 0 to infinity
- Considers both direction and magnitude
- Better for spatial/geometric data

```rust
db.create_namespace("images", 512, MetricKind::L2sq, ScalarKind::F32)?;
```

### Inner Product (IP)

**Best for:** Pre-normalized vectors, maximum inner product search

```
IP = Σ(a_i * b_i)
```

- Faster than cosine (no normalization)
- Vectors should be unit-normalized
- Used in some recommendation systems

```rust
db.create_namespace("recommendations", 384, MetricKind::IP, ScalarKind::F32)?;
```

## Optimizing Search Quality

### 1. Choose the Right Metric

| Data Type | Recommended Metric |
|-----------|-------------------|
| Text embeddings | Cosine |
| Image features | L2sq (Euclidean) |
| Pre-normalized | Inner Product |
| Audio fingerprints | L2sq |

### 2. Use Appropriate k

```lua
-- Too small: might miss relevant results
local results = semantic_search("docs", query, 1)

-- Too large: includes irrelevant results
local results = semantic_search("docs", query, 100)

-- Just right: balance relevance and coverage
local results = semantic_search("docs", query, 10)

-- Then filter/rank
local filtered = filter(results, function(r) return r.distance < 0.5 end)
```

### 3. Filter by Distance

Not all results are relevant. Filter by distance threshold:

```lua
local function get_relevant(query, threshold)
    local results = semantic_search("docs", query, 20)

    return filter(results, function(r)
        return r.distance < threshold
    end)
end

-- Cosine distance thresholds:
-- < 0.3: Very similar
-- < 0.5: Similar
-- < 0.7: Somewhat related
-- > 0.7: Likely unrelated
```

### 4. Combine with Metadata

Use vector search as a first pass, then refine:

```lua
local function smart_search(query, filters)
    -- Get candidate results from vector search
    local candidates = semantic_search("docs", query, 50)

    -- Filter by metadata
    local filtered = filter(candidates, function(r)
        local meta = json.decode(get("meta", r.id) or '{}')

        -- Apply filters
        if filters.category and meta.category ~= filters.category then
            return false
        end
        if filters.min_date and meta.date < filters.min_date then
            return false
        end
        return true
    end)

    -- Return top results
    return slice(filtered, 1, 10)
end
```

## Vector Index Management

### Index Capacity

Pre-allocate for better performance:

```rust
// Get namespace and reserve capacity
let namespace = db.get_namespace("docs")?;
namespace.vector_db.reserve(100_000)?;
```

### Index Statistics

```rust
let namespace = db.get_namespace("docs")?;
let index = &namespace.vector_db;

println!("Size: {} vectors", index.size());
println!("Capacity: {}", index.capacity());
println!("Dimensions: {}", index.dimensions());
println!("Connectivity: {}", index.connectivity());
```

### Persistence

Indices are automatically persisted, but you can force a save:

```rust
// Save specific namespace
db.save_namespace("docs")?;

// Save all
db.save()?;
```

## Advanced Patterns

### Hybrid Search

Combine vector search with keyword matching:

```lua
local function hybrid_search(query, k)
    -- Vector search
    local semantic_results = semantic_search("docs", query, k * 2)

    -- Keyword search (simple contains)
    local keywords = split(query, " ")
    local keyword_boost = {}

    for _, r in ipairs(semantic_results) do
        local boost = 0
        for _, kw in ipairs(keywords) do
            if string.find(r.content:lower(), kw:lower()) then
                boost = boost + 0.1
            end
        end
        keyword_boost[r.id] = boost
    end

    -- Combine scores
    local combined = map(semantic_results, function(r)
        return {
            id = r.id,
            content = r.content,
            score = (1 - r.distance) + (keyword_boost[r.id] or 0)
        }
    end)

    table.sort(combined, function(a, b) return a.score > b.score end)
    return slice(combined, 1, k)
end
```

### Multi-Vector Queries

Search with multiple query vectors and combine:

```lua
local function multi_query_search(queries, k)
    local all_results = {}
    local seen = {}

    for _, query in ipairs(queries) do
        local results = semantic_search("docs", query, k)
        for _, r in ipairs(results) do
            if not seen[r.id] then
                seen[r.id] = true
                table.insert(all_results, r)
            end
        end
    end

    -- Sort by minimum distance across all queries
    table.sort(all_results, function(a, b) return a.distance < b.distance end)
    return slice(all_results, 1, k)
end

-- Usage
local results = multi_query_search({
    "machine learning",
    "artificial intelligence",
    "neural networks"
}, 10)
```

### Clustering

Group similar documents:

```lua
local function find_clusters(namespace, k_clusters, samples_per_cluster)
    local all_keys = keys(namespace)
    local clusters = {}

    for i = 1, k_clusters do
        -- Random seed document
        local seed_idx = math.random(#all_keys)
        local seed_key = all_keys[seed_idx]
        local seed_content = get(namespace, seed_key)

        -- Find similar documents
        local cluster = semantic_search(namespace, seed_content, samples_per_cluster)

        -- Mark as used
        for _, doc in ipairs(cluster) do
            for j, key in ipairs(all_keys) do
                if key == doc.id then
                    table.remove(all_keys, j)
                    break
                end
            end
        end

        clusters[i] = cluster
    end

    return clusters
end
```

## Troubleshooting

### Poor Search Quality

**Symptoms:** Irrelevant results, wrong ordering

**Solutions:**

1. Check embedding model matches your domain
2. Verify namespace dimensions match model output
3. Use appropriate distance metric for your data type
4. Increase k and filter results

### Slow Searches

**Symptoms:** Queries take > 10ms

**Solutions:**

1. Pre-allocate index capacity
2. Use F16 quantization for large indices
3. Reduce k if possible
4. Consider separate indices for different data types

### Memory Usage

**Symptoms:** High memory consumption

**Solutions:**

1. Use F16 instead of F32 (50% memory reduction)
2. Split data across multiple namespaces
3. Archive old data to separate indices

## Next Steps

- [Embeddings](embeddings.md) - Configure embedding models
- [Lua Scripting](../guides/lua-scripting.md) - Advanced search patterns
- [Performance](../guides/performance.md) - Optimization guide
