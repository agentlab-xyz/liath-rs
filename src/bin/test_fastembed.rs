use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};
use anyhow::Result;

fn main() -> Result<()> {
    // Initialize the model
    let model = TextEmbedding::try_new(Default::default())?;
    
    // Generate embeddings
    let texts = vec![
        "Hello, world!",
        "This is a test sentence.",
        "FastEmbed is awesome!"
    ];
    
    let embeddings = model.embed(texts, None)?;
    
    for (i, embedding) in embeddings.iter().enumerate() {
        println!("Embedding {}: length = {}, first 5 values = {:?}", i, embedding.len(), &embedding[..5]);
    }
    
    Ok(())
}