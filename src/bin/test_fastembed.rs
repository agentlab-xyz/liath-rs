#[cfg(feature = "embedding")]
use fastembed::TextEmbedding;
#[cfg(feature = "embedding")]
use anyhow::Result;

#[cfg(feature = "embedding")]
fn main() -> Result<()> {
    let model = TextEmbedding::try_new(Default::default())?;
    let texts = vec![
        "Hello, world!",
        "This is a test sentence.",
        "FastEmbed is awesome!",
    ];
    let embeddings = model.embed(texts, None)?;
    for (i, embedding) in embeddings.iter().enumerate() {
        println!(
            "Embedding {}: length = {}, first 5 values = {:?}",
            i,
            embedding.len(),
            &embedding[..5]
        );
    }
    Ok(())
}

#[cfg(not(feature = "embedding"))]
fn main() {
    println!("Rebuild with `--features embedding` to run this example.");
}
