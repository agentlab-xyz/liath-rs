use fastembed::TextEmbedding;
use anyhow::Result;

fn main() -> Result<()> {
    // Initialize the model with defaults
    let model = TextEmbedding::try_new(Default::default())?;
    println!("Model initialized successfully");

    // Generate embeddings
    let texts = vec![
        "Hello, world!",
        "This is a test sentence.",
        "FastEmbed is awesome!",
    ];

    let embeddings = model.embed(texts, None)?;

    println!("Generated {} embeddings", embeddings.len());
    for (i, embedding) in embeddings.iter().enumerate() {
        println!(
            "Embedding {}: {} dimensions, first 5 values: {:?}",
            i,
            embedding.len(),
            &embedding[..5.min(embedding.len())]
        );
    }

    Ok(())
}