//! # Liath: A Fast Embedded Database for Running Agents
//!
//! Liath is a high-performance database designed for efficiently running AI agents.
//! It combines key-value storage, vector search, and Lua scripting into a single
//! embeddable package.
//!
//! ## Features
//!
//! - **Key-Value Storage**: Fast persistent KV store built on [Fjall](https://crates.io/crates/fjall)
//! - **Vector Search**: Semantic similarity search using [USearch](https://crates.io/crates/usearch)
//! - **Embeddings**: Built-in text embeddings via [FastEmbed](https://crates.io/crates/fastembed)
//! - **Lua Scripting**: Flexible query interface with Lua
//! - **Agent API**: First-class support for AI agent memory, conversations, and tool state
//! - **MCP Server**: Model Context Protocol support for AI assistant integration
//! - **HTTP API**: RESTful API for remote access
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use liath::{EmbeddedLiath, Config};
//! use usearch::{MetricKind, ScalarKind};
//!
//! // Create database
//! let config = Config::default();
//! let mut db = EmbeddedLiath::new(config)?;
//!
//! // Create a namespace with vector support
//! db.create_namespace("documents", 384, MetricKind::Cos, ScalarKind::F32)?;
//!
//! // Store a document with embedding
//! db.store_with_embedding("documents", 1, b"doc1", "Hello, world!")?;
//!
//! // Semantic search
//! let results = db.semantic_search("documents", "greeting", 5)?;
//! ```
//!
//! ## Agent Usage
//!
//! ```rust,ignore
//! use liath::{EmbeddedLiath, Config};
//! use liath::agent::{Agent, Role};
//! use std::sync::Arc;
//!
//! let db = Arc::new(EmbeddedLiath::new(Config::default())?);
//! let agent = Agent::new("my-agent", db);
//!
//! // Store memories
//! let memory = agent.memory()?;
//! memory.store("Important fact", &["tag1", "tag2"])?;
//!
//! // Create conversations
//! let conv = agent.conversation(None)?;
//! conv.add_message(Role::User, "Hello!")?;
//!
//! // Tool state persistence
//! let state = agent.tool_state("my-tool")?;
//! state.set("last_result", &42)?;
//! ```
//!
//! ## Lua Queries
//!
//! ```rust,ignore
//! let executor = db.query_executor();
//!
//! // Execute Lua code
//! let result = executor.execute("return 1 + 1", "user").await?;
//!
//! // Use built-in functions
//! executor.execute(r#"insert("ns", "key", "value")"#, "user").await?;
//! let value = executor.execute(r#"return select("ns", "key")"#, "user").await?;
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
pub mod agent;
pub mod error;
#[cfg(feature = "server")]
pub mod server;
#[cfg(feature = "mcp")]
pub mod mcp;
#[cfg(feature = "python")]
pub mod python;

// Re-export key types
pub use crate::core::{FjallWrapper, NamespaceManager};
pub use crate::vector::UsearchWrapper;
pub use crate::ai::EmbeddingWrapper;
pub use crate::lua::LuaVM;
pub use crate::file::FileStorage;
pub use crate::query::executor::QueryExecutor;
pub use crate::auth::AuthManager;
pub use crate::agent::Agent;
pub use crate::error::{LiathError, LiathResult};

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
    current_namespace: String,
}

impl EmbeddedLiath {
    /// Create a new embedded Liath instance
    pub fn new(config: Config) -> Result<Self> {
        std::fs::create_dir_all(&config.data_dir)?;
        let namespace_manager = NamespaceManager::new(config.data_dir.clone())?;
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

        Ok(Self {
            query_executor,
            current_namespace: String::from("default"),
        })
    }

    /// Execute a Lua query and return the result as JSON
    /// Uses "admin" user for authorization
    pub async fn execute_lua(&self, query: &str) -> Result<serde_json::Value> {
        self.execute_lua_as(query, "admin").await
    }

    /// Execute a Lua query as a specific user and return the result as JSON
    pub async fn execute_lua_as(&self, query: &str, user_id: &str) -> Result<serde_json::Value> {
        let result = self.query_executor.execute(query, user_id).await?;
        // Try to parse the result as JSON, otherwise return as string
        match serde_json::from_str(&result) {
            Ok(json) => Ok(json),
            Err(_) => Ok(serde_json::Value::String(result)),
        }
    }

    /// Set the current namespace for operations that don't specify one
    pub fn set_namespace(&mut self, namespace: &str) {
        self.current_namespace = namespace.to_string();
    }

    /// Get the current namespace
    pub fn current_namespace(&self) -> &str {
        &self.current_namespace
    }

    /// Save all data to disk
    pub fn save(&self) -> Result<()> {
        self.query_executor.save_all()
    }

    /// Close the database connection and save all data
    pub fn close(&self) -> Result<()> {
        self.save()?;
        tracing::info!("Liath database closed successfully");
        Ok(())
    }

    /// Access the underlying query executor (cloned)
    pub fn query_executor(&self) -> QueryExecutor {
        self.query_executor.clone()
    }

    // Convenience APIs
    #[cfg(feature = "vector")]
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

    #[cfg(not(feature = "vector"))]
    pub fn create_namespace_basic(&self, name: &str) -> anyhow::Result<()> {
        use crate::core::{MetricKind, ScalarKind};
        self.query_executor.create_namespace(name, 128, MetricKind::Cos, ScalarKind::F32)
    }

    // ========== Phase 3: Low-Level Vector API ==========

    /// Add a vector to a namespace
    pub fn add_vector(&self, namespace: &str, id: u64, vector: &[f32]) -> Result<()> {
        self.query_executor.add_vector(namespace, id, vector)
    }

    /// Search for similar vectors in a namespace
    pub fn search_vectors(&self, namespace: &str, query: &[f32], k: usize) -> Result<Vec<(u64, f32)>> {
        self.query_executor.similarity_search(namespace, query, k)
    }

    /// Generate embedding for a single text
    pub fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let embeddings = self.query_executor.generate_embedding(vec![text])?;
        embeddings.into_iter().next()
            .ok_or_else(|| anyhow::anyhow!("Failed to generate embedding"))
    }

    /// Generate embeddings for multiple texts
    pub fn generate_embeddings(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        self.query_executor.generate_embedding(texts.to_vec())
    }

    /// Check if a namespace exists
    pub fn namespace_exists(&self, name: &str) -> bool {
        self.query_executor.namespace_exists(name)
    }

    /// List all namespaces
    pub fn list_namespaces(&self) -> Vec<String> {
        self.query_executor.list_namespaces()
    }

    /// Store text with auto-generated embedding
    /// Stores the text in KV store and its embedding in the vector index
    /// Also stores a mapping from vector ID to KV key for semantic search
    pub fn store_with_embedding(&self, namespace: &str, id: u64, key: &[u8], text: &str) -> Result<()> {
        let embedding = self.generate_embedding(text)?;
        self.put(namespace, key, text.as_bytes())?;
        self.add_vector(namespace, id, &embedding)?;
        // Store ID -> key mapping for semantic search lookup
        let mapping_key = format!("_vidx:{}", id);
        self.put(namespace, mapping_key.as_bytes(), key)?;
        Ok(())
    }

    /// Semantic search - search by text query and return matching content
    /// Returns (id, content, distance) tuples
    pub fn semantic_search(&self, namespace: &str, query: &str, k: usize) -> Result<Vec<(u64, String, f32)>> {
        let query_embedding = self.generate_embedding(query)?;
        let results = self.search_vectors(namespace, &query_embedding, k)?;

        // Look up content for each result using ID -> key mapping
        let mut output = Vec::with_capacity(results.len());
        for (id, distance) in results {
            let mapping_key = format!("_vidx:{}", id);
            let content = if let Some(key) = self.get(namespace, mapping_key.as_bytes())? {
                // Found the key, now get the content
                if let Some(data) = self.get(namespace, &key)? {
                    String::from_utf8_lossy(&data).into_owned()
                } else {
                    String::new()
                }
            } else {
                String::new()
            };
            output.push((id, content, distance));
        }
        Ok(output)
    }

    // ========== Convenience methods using current namespace ==========

    /// Put a value in the current namespace
    pub fn put_current(&self, key: &[u8], value: &[u8]) -> Result<()> {
        self.put(&self.current_namespace, key, value)
    }

    /// Get a value from the current namespace
    pub fn get_current(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        self.get(&self.current_namespace, key)
    }

    /// Delete a value from the current namespace
    pub fn delete_current(&self, key: &[u8]) -> Result<()> {
        self.delete(&self.current_namespace, key)
    }

    /// Store with embedding in the current namespace
    pub fn store_with_embedding_current(&self, id: u64, key: &[u8], text: &str) -> Result<()> {
        self.store_with_embedding(&self.current_namespace, id, key, text)
    }

    /// Semantic search in the current namespace
    pub fn semantic_search_current(&self, query: &str, k: usize) -> Result<Vec<(u64, String, f32)>> {
        self.semantic_search(&self.current_namespace, query, k)
    }
}
