// mod llm;  // Commenting out LLM module for now to reduce dependencies

#[cfg(feature = "embedding")]
mod embedding;

// pub use llm::LLMWrapper;
#[cfg(feature = "embedding")]
pub use embedding::EmbeddingWrapper;

#[cfg(not(feature = "embedding"))]
pub struct EmbeddingWrapper;

#[cfg(not(feature = "embedding"))]
impl EmbeddingWrapper {
    pub fn new() -> anyhow::Result<Self> { Ok(Self) }
    pub fn generate(&self, _texts: Vec<&str>) -> anyhow::Result<Vec<Vec<f32>>> {
        anyhow::bail!("embedding feature is disabled")
    }
}
