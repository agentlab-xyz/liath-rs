use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};
use anyhow::{Result, anyhow};
use std::sync::Arc;

/// A wrapper around fastembed TextEmbedding for generating text embeddings
pub struct EmbeddingWrapper {
    model: Arc<TextEmbedding>,
}

impl EmbeddingWrapper {
    /// Create a new EmbeddingWrapper with default options
    pub fn new() -> Result<Self> {
        let model = TextEmbedding::try_new(Default::default())
            .map_err(|e| anyhow!("Failed to create TextEmbedding with default options: {}", e))?;
        
        Ok(Self { 
            model: Arc::new(model),
        })
    }

    /// Create a new EmbeddingWrapper with custom options
    pub fn with_options(options: InitOptions) -> Result<Self> {
        let model = TextEmbedding::try_new(options)
            .map_err(|e| anyhow!("Failed to create TextEmbedding with custom options: {}", e))?;
        
        Ok(Self { 
            model: Arc::new(model),
        })
    }

    /// Create a new EmbeddingWrapper with a specific model
    pub fn with_model(model: EmbeddingModel) -> Result<Self> {
        let mut options = InitOptions::default();
        options.model_name = model;
        Self::with_options(options)
    }

    /// Generate embeddings for a list of texts
    pub fn generate(&self, texts: Vec<&str>) -> Result<Vec<Vec<f32>>> {
        self.model.embed(texts, None)
            .map_err(|e| anyhow!("Failed to generate embeddings: {}", e))
    }

    /// Generate embeddings for a single text
    pub fn generate_one(&self, text: &str) -> Result<Vec<f32>> {
        let embeddings = self.model.embed(vec![text], None)
            .map_err(|e| anyhow!("Failed to generate embedding: {}", e))?;
        Ok(embeddings.into_iter().next().unwrap_or_default())
    }

    /// Get a reference to the underlying model
    pub fn model(&self) -> &TextEmbedding {
        &self.model
    }
}

impl Default for EmbeddingWrapper {
    fn default() -> Self {
        Self::new().expect("Failed to create default EmbeddingWrapper")
    }
}

impl Clone for EmbeddingWrapper {
    fn clone(&self) -> Self {
        Self {
            model: Arc::clone(&self.model),
        }
    }
}
