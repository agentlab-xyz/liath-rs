# Performance

This guide covers optimization strategies for Liath deployments.

## Performance Characteristics

### Operation Costs

| Operation | Typical Time | Notes |
|-----------|--------------|-------|
| KV Get | < 1ms | Very fast |
| KV Put | 1-5ms | Includes durability |
| Embedding (single) | 10-50ms | CPU bound |
| Embedding (batch) | 5-20ms per item | Batching recommended |
| Vector Search (100K) | < 5ms | HNSW efficiency |
| Lua Execution | 1-100ms | Depends on complexity |

### Scaling Factors

| Factor | Impact | Mitigation |
|--------|--------|------------|
| Vector count | O(log N) search | Index sharding |
| Vector dimensions | Linear memory | Use F16 if possible |
| Embedding text length | Longer = slower | Truncate, chunk |
| Lua complexity | Varies | Simplify, cache |

## Embedding Optimization

### Batch Processing

Always batch embedding operations:

```rust
// Slow: individual calls
for text in texts {
    let emb = db.generate_embedding(&text)?;  // 50ms each
}

// Fast: batch call
let embeddings = db.generate_embeddings(&texts)?;  // ~15ms each
```

### Caching Embeddings

Avoid regenerating embeddings:

```lua
local function get_cached_embedding(text)
    local cache_key = "emb_cache:" .. md5(text)
    local cached = get("cache", cache_key)

    if cached then
        return json.decode(cached)
    end

    local embedding = embed(text)
    put("cache", cache_key, json.encode(embedding))
    return embedding
end
```

### Concurrency Control

Limit concurrent embedding operations:

```rust
let executor = QueryExecutor::new(
    namespace_manager,
    embedding_wrapper,
    lua_vm,
    file_storage,
    auth_manager,
    10  // Limit to 10 concurrent embeddings
);
```

## Vector Search Optimization

### Index Pre-allocation

Reserve capacity for expected data:

```rust
let namespace = db.get_namespace("documents")?;
namespace.vector_db.reserve(100_000)?;  // Pre-allocate for 100K vectors
```

### Quantization

Use smaller scalar types for large indices:

```rust
// High precision (4 bytes per dimension)
db.create_namespace("critical", 384, MetricKind::Cos, ScalarKind::F32)?;

// Memory efficient (2 bytes per dimension)
db.create_namespace("bulk", 384, MetricKind::Cos, ScalarKind::F16)?;

// Compact (1 byte per dimension)
db.create_namespace("huge", 384, MetricKind::Cos, ScalarKind::I8)?;
```

### Memory Estimates

| Vectors | Dimensions | Type | Memory |
|---------|------------|------|--------|
| 100K | 384 | F32 | ~150MB |
| 100K | 384 | F16 | ~75MB |
| 1M | 384 | F32 | ~1.5GB |
| 1M | 384 | F16 | ~750MB |

### Search Optimization

```lua
-- Only retrieve what you need
local function efficient_search(query, needed)
    -- Fetch 2x to allow filtering
    local results = semantic_search("docs", query, needed * 2)

    -- Filter in Lua (fast)
    local filtered = filter(results, function(r)
        return r.distance < 0.5
    end)

    return slice(filtered, 1, needed)
end
```

## Lua Performance

### Avoid Repeated Lookups

```lua
-- Slow: repeated function lookups
for i = 1, 1000 do
    local result = semantic_search("ns", query, 1)
end

-- Fast: cache results
local results = semantic_search("ns", query, 1000)
for i, r in ipairs(results) do
    -- process
end
```

### Use Local Variables

```lua
-- Slow: global access
for i = 1, 1000 do
    table.insert(results, process(data[i]))
end

-- Fast: local reference
local insert = table.insert
local local_results = {}
for i = 1, 1000 do
    insert(local_results, process(data[i]))
end
```

### Minimize JSON Encoding

```lua
-- Slow: encode in loop
for _, item in ipairs(items) do
    put("ns", item.id, json.encode(item))
end

-- Fast: batch if possible
local batch = json.encode(items)
put("ns", "batch", batch)
```

### Efficient String Building

```lua
-- Slow: string concatenation
local result = ""
for _, s in ipairs(strings) do
    result = result .. s
end

-- Fast: table.concat
local parts = {}
for _, s in ipairs(strings) do
    table.insert(parts, s)
end
local result = table.concat(parts)
```

## Storage Optimization

### Key Design

```lua
-- Good: short, structured keys
put("u", "123:name", "Alice")
put("u", "123:email", "alice@example.com")

-- Avoid: long, repetitive keys
put("users_namespace", "user_id_123_field_name", "Alice")
```

### Value Compression

```lua
-- For large values, consider compression
local function compress_store(ns, key, value)
    -- Implement compression if needed
    local compressed = compress(json.encode(value))
    put(ns, key, compressed)
end
```

### Batch Operations

```rust
// Slow: individual puts
for (key, value) in items {
    db.put("ns", key, value)?;
}

// Fast: use batch when available
db.batch_put("ns", items)?;
```

## Memory Management

### Limit Search Results

```lua
-- Always limit k
local MAX_RESULTS = 100

local function safe_search(query, k)
    local safe_k = math.min(k, MAX_RESULTS)
    return semantic_search("docs", query, safe_k)
end
```

### Clear Unused Data

```lua
-- Periodically clean up
local function cleanup_old_data(namespace, max_age_days)
    local cutoff = now() - (max_age_days * 24 * 3600)
    local all_keys = keys(namespace)

    for _, key in ipairs(all_keys) do
        local meta = json.decode(get(namespace .. ":meta", key) or '{}')
        if meta.created_at and meta.created_at < cutoff then
            delete(namespace, key)
            delete(namespace .. ":meta", key)
        end
    end
end
```

### Monitor Memory Usage

```rust
// Check index size periodically
let namespace = db.get_namespace("documents")?;
let index = &namespace.vector_db;

info!(
    namespace = "documents",
    vectors = index.size(),
    capacity = index.capacity(),
    memory_estimate = index.size() * 384 * 4,  // F32
    "Index statistics"
);
```

## Query Patterns

### Efficient Filtering

```lua
-- Slow: filter after retrieving all
local all = semantic_search("docs", query, 1000)
local filtered = filter(all, function(r) return r.distance < 0.3 end)

-- Fast: use distance threshold in design
local results = semantic_search("docs", query, 50)  -- Smaller k
local good = filter(results, function(r) return r.distance < 0.3 end)
```

### Parallel Searches

```rust
use tokio::join;

async fn multi_namespace_search(
    db: &EmbeddedLiath,
    query: &str,
    namespaces: &[&str],
) -> Vec<SearchResult> {
    let futures: Vec<_> = namespaces.iter()
        .map(|ns| db.semantic_search(ns, query, 10))
        .collect();

    let results = futures::future::join_all(futures).await;

    // Merge and sort
    results.into_iter()
        .flat_map(|r| r.unwrap_or_default())
        .sorted_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap())
        .take(20)
        .collect()
}
```

## Benchmarking

### Simple Benchmark

```rust
use std::time::Instant;

async fn benchmark_search(db: &EmbeddedLiath, iterations: usize) {
    let start = Instant::now();

    for _ in 0..iterations {
        db.semantic_search("docs", "test query", 10)?;
    }

    let elapsed = start.elapsed();
    let per_op = elapsed / iterations as u32;

    println!("Search: {:?} per operation", per_op);
}
```

### Lua Benchmark

```lua
local function benchmark(name, iterations, f)
    local start = os.clock()
    for i = 1, iterations do
        f()
    end
    local elapsed = os.clock() - start
    print(name .. ": " .. (elapsed / iterations * 1000) .. "ms per op")
end

benchmark("get", 1000, function()
    get("test", "key")
end)

benchmark("put", 1000, function()
    put("test", "key" .. id(), "value")
end)

benchmark("search", 100, function()
    semantic_search("docs", "query", 10)
end)
```

## Production Recommendations

### Configuration

```rust
let config = Config {
    data_dir: PathBuf::from("/var/lib/liath"),  // Fast SSD
    ..Default::default()
};

// High concurrency for embeddings
let executor = QueryExecutor::new(
    namespace_manager,
    embedding_wrapper,
    lua_vm,
    file_storage,
    auth_manager,
    20  // Higher limit for production
);
```

### Monitoring

```rust
// Export metrics
struct Metrics {
    query_latency: Histogram,
    embedding_latency: Histogram,
    search_latency: Histogram,
    vectors_count: Gauge,
}

impl Metrics {
    fn record_query(&self, duration: Duration) {
        self.query_latency.observe(duration.as_secs_f64());
    }
}
```

### Scaling Strategies

1. **Vertical**: Faster CPU, more RAM, NVMe storage
2. **Read replicas**: Clone data for read-heavy workloads
3. **Namespace sharding**: Distribute data across instances
4. **Caching layer**: Redis/memcached for hot queries

## Next Steps

- [Architecture](../concepts/architecture.md) - System design
- [Configuration](../getting-started/configuration.md) - Tuning options
- [HTTP Server](../integrations/http-server.md) - Server optimization
