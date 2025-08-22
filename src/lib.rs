//! Liath: An AI-powered database system with key-value storage, vector search, and AI capabilities.
//!
//! This library provides:
//! - A database system built on RocksDB
//! - Lua as the query language
//! - AI integration capabilities (LLM inference, embeddings)
//! - Vector search functionality
//! - File storage operations
//! - Authentication and authorization
//! - CLI and HTTP API interfaces
//!
//! # Example
//!
//! ```rust
//! // TODO: Add example usage when API is stabilized
//! ```

// Re-export core modules
pub mod core;
pub mod vector;
pub mod ai;
pub mod lua;
pub mod file;
pub mod query;
pub mod auth;

// Re-export key types
pub use crate::core::{RocksDBWrapper, NamespaceManager};
pub use crate::vector::UsearchWrapper;
pub use crate::ai::{LLMWrapper, EmbeddingWrapper};
pub use crate::lua::LuaVM;
pub use crate::file::FileStorage;
pub use crate::query::executor::QueryExecutor;
pub use crate::auth::AuthManager;

use anyhow::Result;
use candle_core::Device;
use std::path::PathBuf;

/// Configuration for the Liath database
#[derive(Debug, Clone)]
pub struct Config {
    pub device: Device,
    pub model_path: PathBuf,
    pub tokenizer_path: PathBuf,
    pub data_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            device: Device::Cpu,
            model_path: PathBuf::from("model.gguf"),
            tokenizer_path: PathBuf::from("tokenizer.json"),
            data_dir: PathBuf::from("./data"),
        }
    }
}

/// Embedded Liath database interface
/// 
/// This struct provides a simplified interface for embedding Liath directly into Rust applications.
pub struct EmbeddedLiath {
    query_executor: QueryExecutor,
}

impl EmbeddedLiath {
    /// Create a new embedded Liath instance
    pub fn new(config: Config) -> Result<Self> {
        let namespace_manager = NamespaceManager::new();
        let llm = LLMWrapper::new(config.model_path, config.tokenizer_path, config.device)?;
        let embedding = EmbeddingWrapper::new()?;
        let lua_vm = LuaVM::new(std::path::PathBuf::from("path/to/luarocks"))?; // TODO: Make configurable
        let file_storage = FileStorage::new("path/to/file/storage")?; // TODO: Use config.data_dir
        let mut auth_manager = AuthManager::new();

        // Add a default admin user
        auth_manager.add_user("admin", vec![
            "select".to_string(),
            "insert".to_string(),
            "update".to_string(),
            "delete".to_string(),
            "create_namespace".to_string(),
            "delete_namespace".to_string(),
            "upload_file".to_string(),
            "process_file".to_string(),
            "generate_embedding".to_string(),
            "similarity_search".to_string(),
            "llm_query".to_string(),
        ]);

        let query_executor = QueryExecutor::new(
            namespace_manager,
            llm,
            embedding,
            lua_vm,
            file_storage,
            auth_manager,
            5,  // max_concurrent_llm
            10, // max_concurrent_embedding
        );

        Ok(Self { query_executor })
    }

    /// Execute a Lua query
    pub async fn execute_lua(&self, query: &str) -> Result<serde_json::Value> {
        // TODO: Implement proper query execution with namespace and authentication
        self.query_executor.execute(query, "default").await?;
        // For now, we're just returning a dummy value
        Ok(serde_json::Value::String("Query executed".to_string()))
    }

    /// Set the current namespace
    pub fn set_namespace(&mut self, _namespace: &str) {
        // TODO: Implement namespace switching
    }

    /// Close the database connection
    pub fn close(&self) {
        // TODO: Implement proper cleanup
    }
}