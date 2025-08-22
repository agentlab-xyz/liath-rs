//! Example of using Liath as an embedded database

use liath::{EmbeddedLiath, Config};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a configuration (in a real app, you'd provide actual paths)
    let config = Config {
        model_path: PathBuf::from("model.gguf"),
        tokenizer_path: PathBuf::from("tokenizer.json"),
        ..Default::default()
    };

    // Create an embedded database instance
    // Note: This will fail if the model files don't exist at the specified paths
    match EmbeddedLiath::new(config) {
        Ok(_db) => {
            println!("Successfully created embedded Liath instance");
            // In a real application, you would:
            // 1. Execute Lua queries
            // 2. Perform database operations
            // 3. Close the database when done
        }
        Err(e) => {
            println!("Failed to create embedded Liath instance: {}", e);
            println!("This is expected if you don't have the model files installed.");
        }
    }

    Ok(())
}