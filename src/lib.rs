//! Liath: An AI-powered database system with key-value storage, vector search, and AI capabilities.
//!
//! This library provides:
//! - A database system built on Fjall
//! - Lua as the query language
//! - AI integration capabilities (embeddings)
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
pub mod cli;
#[cfg(feature = "server")]
pub mod server;

// Re-export key types
pub use crate::core::{FjallWrapper, NamespaceManager};
pub use crate::vector::UsearchWrapper;
pub use crate::ai::EmbeddingWrapper;
pub use crate::lua::LuaVM;
pub use crate::file::FileStorage;
pub use crate::query::executor::QueryExecutor;
pub use crate::auth::AuthManager;

use anyhow::Result;
use std::path::PathBuf;

/// Configuration for the Liath database
#[derive(Debug, Clone)]
pub struct Config {
    pub data_dir: PathBuf,
    pub luarocks_path: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("./data"),
            luarocks_path: None,
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
        let embedding = EmbeddingWrapper::new()?;
        let lua_vm = LuaVM::new(config.luarocks_path.clone().unwrap_or_else(|| std::path::PathBuf::from("luarocks")))?; // Uses `luarocks` from PATH by default
        let file_storage_path = config.data_dir.join("files");
        let file_storage = FileStorage::new(file_storage_path)?;
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
        ]);

        let query_executor = QueryExecutor::new(
            namespace_manager,
            embedding,
            lua_vm,
            file_storage,
            auth_manager,
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

    /// Access the underlying query executor (cloned)
    pub fn query_executor(&self) -> QueryExecutor {
        self.query_executor.clone()
    }

    // Convenience APIs
    pub fn create_namespace(
        &self,
        name: &str,
        dimensions: usize,
        metric: usearch::MetricKind,
        scalar: usearch::ScalarKind,
    ) -> Result<()> {
        self.query_executor.create_namespace(name, dimensions, metric, scalar)
    }

    pub fn put(&self, namespace: &str, key: &[u8], value: &[u8]) -> Result<()> {
        self.query_executor.put(namespace, key, value)
    }

    pub fn get(&self, namespace: &str, key: &[u8]) -> Result<Option<Vec<u8>>> {
        self.query_executor.get(namespace, key)
    }

    pub fn delete(&self, namespace: &str, key: &[u8]) -> Result<()> {
        self.query_executor.delete(namespace, key)
    }
}
