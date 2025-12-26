//! Example of using Liath as an embedded database

use liath::{EmbeddedLiath, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a configuration with default settings
    let config = Config::default();

    // Create an embedded database instance
    match EmbeddedLiath::new(config) {
        Ok(db) => {
            println!("Successfully created embedded Liath instance");

            // Create a namespace and perform basic operations
            #[cfg(feature = "vector")]
            {
                use usearch::{MetricKind, ScalarKind};
                if let Err(e) = db.create_namespace("example", 384, MetricKind::Cos, ScalarKind::F32) {
                    println!("Note: {}", e);
                }
            }

            // Put and get a value
            db.put("example", b"greeting", b"Hello, Liath!")?;
            if let Some(value) = db.get("example", b"greeting")? {
                println!("Retrieved: {}", String::from_utf8_lossy(&value));
            }

            // Clean up
            db.delete("example", b"greeting")?;
            println!("Example completed successfully!");
        }
        Err(e) => {
            println!("Failed to create embedded Liath instance: {}", e);
            println!("This may happen if embedding models fail to load.");
        }
    }

    Ok(())
}