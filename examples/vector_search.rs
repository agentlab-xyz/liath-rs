//! Vector similarity search example
//!
//! This example demonstrates how to use Liath's vector search capabilities
//! for semantic document search and similarity matching.

use liath::{EmbeddedLiath, Config};
use usearch::{MetricKind, ScalarKind};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Liath Vector Search Example ===\n");

    // Create database with temporary storage
    let config = Config {
        data_dir: std::env::temp_dir().join("liath_vector_example"),
        ..Default::default()
    };

    let db = EmbeddedLiath::new(config)?;
    println!("Database initialized.\n");

    // Create a namespace with 384-dimensional vectors (matches BGE-small model)
    // Using cosine similarity for semantic search
    db.create_namespace("documents", 384, MetricKind::Cos, ScalarKind::F32)?;
    println!("Created 'documents' namespace (384 dims, cosine similarity)\n");

    // Store some documents with their embeddings
    let documents = vec![
        (1, "doc:ai", "Artificial intelligence is transforming how we work and live"),
        (2, "doc:ml", "Machine learning models can learn patterns from data"),
        (3, "doc:rust", "Rust is a systems programming language focused on safety"),
        (4, "doc:db", "Databases store and retrieve data efficiently"),
        (5, "doc:vec", "Vector databases enable semantic search using embeddings"),
    ];

    println!("Storing documents with embeddings...");
    for (id, key, text) in &documents {
        db.store_with_embedding("documents", *id, key.as_bytes(), text)?;
        println!("  Stored: {} -> \"{}\"", key, text);
    }
    println!();

    // Perform semantic search
    println!("=== Semantic Search Results ===\n");

    let queries = vec![
        "How does AI affect our daily lives?",
        "What programming languages are safe?",
        "How do search engines work?",
    ];

    for query in queries {
        println!("Query: \"{}\"\n", query);

        let results = db.semantic_search("documents", query, 3)?;

        for (i, (id, content, distance)) in results.iter().enumerate() {
            println!("  {}. [id={}, dist={:.3}] {}", i + 1, id, distance, content);
        }
        println!();
    }

    // Demonstrate direct vector operations
    println!("=== Direct Vector Operations ===\n");

    // Generate an embedding for a new query
    let query_embedding = db.generate_embedding("neural networks and deep learning")?;
    println!("Generated embedding for query (dim={})\n", query_embedding.len());

    // Search using the raw embedding
    let results = db.search_vectors("documents", &query_embedding, 2)?;
    println!("Raw vector search results:");
    for (id, distance) in results {
        println!("  ID: {}, Distance: {:.4}", id, distance);
    }

    println!("\nVector search example completed!");
    Ok(())
}
