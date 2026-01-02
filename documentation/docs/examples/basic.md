# Basic Usage Examples

Fundamental examples for getting started with Liath.

## Key-Value Operations

### Simple Storage

```rust
use liath::{EmbeddedLiath, Config};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = EmbeddedLiath::new(Config::default())?;

    // Store values
    db.put("users", b"user:1", b"Alice")?;
    db.put("users", b"user:2", b"Bob")?;
    db.put("settings", b"theme", b"dark")?;

    // Retrieve values
    if let Some(value) = db.get("users", b"user:1")? {
        println!("User 1: {}", String::from_utf8_lossy(&value));
    }

    // Delete values
    db.delete("users", b"user:2")?;

    // Check if deleted
    match db.get("users", b"user:2")? {
        Some(_) => println!("Still exists"),
        None => println!("Successfully deleted"),
    }

    Ok(())
}
```

### JSON Data

```rust
use liath::{EmbeddedLiath, Config};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct User {
    name: String,
    email: String,
    age: u32,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = EmbeddedLiath::new(Config::default())?;

    // Store JSON
    let user = User {
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        age: 30,
    };

    let json = serde_json::to_vec(&user)?;
    db.put("users", b"user:alice", &json)?;

    // Retrieve and parse
    if let Some(data) = db.get("users", b"user:alice")? {
        let user: User = serde_json::from_slice(&data)?;
        println!("User: {:?}", user);
    }

    Ok(())
}
```

## Namespace Management

```rust
use liath::{EmbeddedLiath, Config};
use usearch::{MetricKind, ScalarKind};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = EmbeddedLiath::new(Config::default())?;

    // Create namespaces
    db.create_namespace("documents", 384, MetricKind::Cos, ScalarKind::F32)?;
    db.create_namespace("images", 512, MetricKind::L2sq, ScalarKind::F32)?;

    // List namespaces
    for ns in db.list_namespaces() {
        println!("Namespace: {}", ns);
    }

    // Check existence
    if db.namespace_exists("documents") {
        println!("Documents namespace exists");
    }

    // Delete namespace
    // db.delete_namespace("images")?;

    Ok(())
}
```

## Embedding Generation

```rust
use liath::{EmbeddedLiath, Config};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = EmbeddedLiath::new(Config::default())?;

    // Generate single embedding
    let embedding = db.generate_embedding("Hello, world!")?;
    println!("Embedding dimensions: {}", embedding.len());
    println!("First 5 values: {:?}", &embedding[..5]);

    // Generate batch embeddings
    let texts = vec![
        "First document about programming",
        "Second document about cooking",
        "Third document about travel",
    ];

    let embeddings = db.generate_embeddings(&texts)?;
    for (i, emb) in embeddings.iter().enumerate() {
        println!("Text {}: {} dimensions", i, emb.len());
    }

    Ok(())
}
```

## Storing with Embeddings

```rust
use liath::{EmbeddedLiath, Config};
use usearch::{MetricKind, ScalarKind};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = EmbeddedLiath::new(Config::default())?;

    // Create namespace
    db.create_namespace("docs", 384, MetricKind::Cos, ScalarKind::F32)?;

    // Store documents with automatic embedding
    db.store_with_embedding("docs", 1, b"doc:1", "Rust is a systems programming language")?;
    db.store_with_embedding("docs", 2, b"doc:2", "Python is great for data science")?;
    db.store_with_embedding("docs", 3, b"doc:3", "JavaScript runs in the browser")?;
    db.store_with_embedding("docs", 4, b"doc:4", "Go is designed for simplicity")?;

    println!("Stored 4 documents with embeddings");

    Ok(())
}
```

## Basic Semantic Search

```rust
use liath::{EmbeddedLiath, Config};
use usearch::{MetricKind, ScalarKind};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = EmbeddedLiath::new(Config::default())?;

    // Setup
    db.create_namespace("docs", 384, MetricKind::Cos, ScalarKind::F32)?;

    db.store_with_embedding("docs", 1, b"doc:1", "Machine learning is a subset of AI")?;
    db.store_with_embedding("docs", 2, b"doc:2", "Neural networks power deep learning")?;
    db.store_with_embedding("docs", 3, b"doc:3", "Cooking pasta requires boiling water")?;
    db.store_with_embedding("docs", 4, b"doc:4", "Natural language processing uses ML")?;

    // Search
    let results = db.semantic_search("docs", "artificial intelligence", 3)?;

    println!("Search results for 'artificial intelligence':");
    for (id, content, distance) in results {
        println!("  [{:.3}] {}: {}", distance, id, content);
    }

    Ok(())
}
```

## Using Lua Queries

```rust
use liath::{EmbeddedLiath, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = EmbeddedLiath::new(Config::default())?;
    let executor = db.query_executor();

    // Simple calculation
    let result = executor.execute("return 1 + 2 + 3", "user").await?;
    println!("1 + 2 + 3 = {}", result);

    // String operations
    let result = executor.execute(r#"return "Hello, " .. "World!""#, "user").await?;
    println!("{}", result);

    // JSON encoding
    let result = executor.execute(r#"
        local data = {
            name = "Alice",
            scores = {95, 87, 92},
            active = true
        }
        return json.encode(data)
    "#, "user").await?;
    println!("JSON: {}", result);

    // Database operations
    executor.execute(r#"
        put("test", "greeting", "Hello from Lua!")
    "#, "user").await?;

    let result = executor.execute(r#"
        return get("test", "greeting")
    "#, "user").await?;
    println!("Retrieved: {}", result);

    Ok(())
}
```

## Error Handling

```rust
use liath::{EmbeddedLiath, Config, LiathError};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = EmbeddedLiath::new(Config::default())?;

    // Handle namespace not found
    match db.get("nonexistent", b"key") {
        Ok(Some(value)) => println!("Found: {:?}", value),
        Ok(None) => println!("Key not found"),
        Err(LiathError::NamespaceNotFound(ns)) => {
            println!("Namespace '{}' doesn't exist", ns);
        }
        Err(e) => println!("Other error: {}", e),
    }

    // Handle with Result combinators
    let value = db.get("test", b"maybe_exists")?
        .map(|v| String::from_utf8_lossy(&v).to_string())
        .unwrap_or_else(|| "default".to_string());

    println!("Value: {}", value);

    Ok(())
}
```

## Persistence

```rust
use liath::{EmbeddedLiath, Config};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Specify data directory
    let config = Config {
        data_dir: PathBuf::from("./my_database"),
        ..Default::default()
    };

    {
        let db = EmbeddedLiath::new(config.clone())?;

        // Store data
        db.put("persistent", b"key1", b"value1")?;
        db.put("persistent", b"key2", b"value2")?;

        // Explicitly save
        db.save()?;

        // Close gracefully
        db.close()?;
    }

    // Reopen and verify
    {
        let db = EmbeddedLiath::new(config)?;

        let value = db.get("persistent", b"key1")?;
        println!("After reopen: {:?}", value.map(|v| String::from_utf8_lossy(&v).to_string()));
    }

    Ok(())
}
```

## Next Steps

- [Vector Search Examples](vector-search.md) - Advanced search patterns
- [Agent Patterns](agent-patterns.md) - Building AI agents
- [Lua Scripting Guide](../guides/lua-scripting.md) - Full Lua reference
