use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};
use anyhow::Result;

fn main() -> Result<()> {
    // Initialize the model
    let model = TextEmbedding::try_new(Default::default())?;
    
    // Get model metadata
    let metadata = model.get_metadata();
    println!("Model: {:?}", metadata.model_name);
    println!("Dimensions: {}", metadata.dimensions);
    
    // Generate embeddings
    let texts = vec![
        "Hello, world!",
        "This is a test sentence.",
        "FastEmbed is awesome!"
    ];
    
    let embeddings = model.embed(texts, None)?;
    
    for (i, embedding) in embeddings.iter().enumerate() {
        println!("Embedding {}: {:?}", i, &embedding[..5]); // Print first 5 values
    }
    
    Ok(())
}