# EmbeddedLiath API

`EmbeddedLiath` is the main entry point for using Liath as an embedded database.

## Creating an Instance

### new

Create a new EmbeddedLiath instance with configuration:

```rust
use liath::{EmbeddedLiath, Config};

let config = Config {
    data_dir: "./my_data".into(),
    ..Default::default()
};

let db = EmbeddedLiath::new(config)?;
```

## Configuration

```rust
pub struct Config {
    /// Directory for storing data files
    pub data_dir: PathBuf,

    /// Vector dimensions for embeddings (default: 384)
    pub vector_dimensions: usize,

    /// Maximum concurrent Lua executions (default: 10)
    pub max_concurrent_queries: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            data_dir: PathBuf::from("./liath_data"),
            vector_dimensions: 384,
            max_concurrent_queries: 10,
        }
    }
}
```

## Key-Value Operations

### put

Store a value:

```rust
fn put(&self, namespace: &str, key: &[u8], value: &[u8]) -> Result<(), Error>
```

**Example:**

```rust
db.put("users", b"user:1", b"Alice")?;
db.put("config", b"theme", b"dark")?;
```

### get

Retrieve a value:

```rust
fn get(&self, namespace: &str, key: &[u8]) -> Result<Option<Vec<u8>>, Error>
```

**Example:**

```rust
if let Some(value) = db.get("users", b"user:1")? {
    let name = String::from_utf8_lossy(&value);
    println!("User: {}", name);
}
```

### delete

Remove a value:

```rust
fn delete(&self, namespace: &str, key: &[u8]) -> Result<(), Error>
```

**Example:**

```rust
db.delete("users", b"user:1")?;
```

## Namespace Management

### create_namespace

Create a namespace with optional vector index:

```rust
fn create_namespace(
    &self,
    name: &str,
    dimensions: usize,
    metric: MetricKind,
    scalar: ScalarKind
) -> Result<(), Error>
```

**Example:**

```rust
use usearch::{MetricKind, ScalarKind};

db.create_namespace("documents", 384, MetricKind::Cos, ScalarKind::F32)?;
```

### list_namespaces

List all namespaces:

```rust
fn list_namespaces(&self) -> Result<Vec<String>, Error>
```

**Example:**

```rust
let namespaces = db.list_namespaces()?;
for ns in namespaces {
    println!("Namespace: {}", ns);
}
```

## Vector Operations

### add_vector

Add a vector to a namespace's index:

```rust
fn add_vector(&self, namespace: &str, id: &str, vector: &[f32]) -> Result<(), Error>
```

**Example:**

```rust
let vector = db.generate_embedding("Hello, world!")?;
db.add_vector("docs", "doc:1", &vector)?;
```

### search_vectors

Search for similar vectors:

```rust
fn search_vectors(
    &self,
    namespace: &str,
    query: &[f32],
    k: usize
) -> Result<Vec<SearchResult>, Error>
```

**Example:**

```rust
let query_vec = db.generate_embedding("greeting")?;
let results = db.search_vectors("docs", &query_vec, 5)?;

for result in results {
    println!("ID: {}, Distance: {:.4}", result.id, result.distance);
}
```

## Embedding Operations

### generate_embedding

Generate embedding for single text:

```rust
fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, Error>
```

**Example:**

```rust
let embedding = db.generate_embedding("Machine learning concepts")?;
println!("Dimensions: {}", embedding.len());
```

### generate_embeddings

Generate embeddings for multiple texts:

```rust
fn generate_embeddings(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, Error>
```

**Example:**

```rust
let texts = vec!["First document", "Second document", "Third document"];
let embeddings = db.generate_embeddings(&texts)?;

for (i, emb) in embeddings.iter().enumerate() {
    println!("Text {}: {} dimensions", i, emb.len());
}
```

## Semantic Search

### semantic_search

Search using text query:

```rust
fn semantic_search(
    &self,
    namespace: &str,
    query: &str,
    k: usize
) -> Result<Vec<SemanticResult>, Error>
```

**Example:**

```rust
let results = db.semantic_search("docs", "programming languages", 5)?;

for result in results {
    println!("Content: {}", result.content);
    println!("Distance: {:.4}", result.distance);
}
```

## Query Executor

### query_executor

Get the query executor for Lua execution:

```rust
fn query_executor(&self) -> &QueryExecutor
```

**Example:**

```rust
let executor = db.query_executor();

let code = r#"
    store_with_embedding("docs", "d1", "Hello, world!")
    return semantic_search("docs", "greeting", 1)
"#;

let result = executor.execute(code, "agent").await?;
```

## QueryExecutor API

### execute

Execute Lua code:

```rust
async fn execute(&self, code: &str, agent_id: &str) -> Result<String, Error>
```

**Example:**

```rust
let result = executor.execute(r#"
    local x = 1 + 2
    return json.encode({ result = x })
"#, "agent").await?;
```

### validate

Validate Lua code without executing:

```rust
fn validate(&self, code: &str) -> Result<(), ValidationError>
```

**Example:**

```rust
match executor.validate("return 1 + 2") {
    Ok(()) => println!("Valid Lua code"),
    Err(e) => println!("Invalid: {}", e),
}
```

## Error Handling

All operations return `Result<T, liath::Error>`:

```rust
use liath::Error;

match db.get("namespace", b"key") {
    Ok(Some(value)) => println!("Found: {:?}", value),
    Ok(None) => println!("Not found"),
    Err(Error::NamespaceNotFound(ns)) => println!("Namespace {} not found", ns),
    Err(e) => println!("Error: {}", e),
}
```

## Thread Safety

`EmbeddedLiath` is thread-safe and can be shared across threads using `Arc`:

```rust
use std::sync::Arc;

let db = Arc::new(EmbeddedLiath::new(Config::default())?);

let db_clone = db.clone();
tokio::spawn(async move {
    db_clone.put("ns", b"key", b"value").unwrap();
});
```

## Next Steps

- [Agent API](agent-api.md) - High-level agent abstractions
- [Lua Standard Library](lua-stdlib.md) - Available Lua functions
- [Examples](../examples/index.md) - Working code examples
