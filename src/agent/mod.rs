//! High-level agent API for building AI agents
//!
//! This module provides abstractions for building stateful AI agents including:
//! - **Memory**: Long-term semantic memory with vector search
//! - **Conversation**: Message history management
//! - **ToolState**: Persistent state for tools
//! - **Agent**: Entry point combining all capabilities
//!
//! # Example
//!
//! ```rust,ignore
//! use liath::{EmbeddedLiath, Config};
//! use liath::agent::Agent;
//! use std::sync::Arc;
//!
//! let db = Arc::new(EmbeddedLiath::new(Config::default())?);
//! let agent = Agent::new("my-agent", db);
//!
//! // Store and recall memories
//! let memory = agent.memory()?;
//! memory.store("The capital of France is Paris", &["geography", "facts"])?;
//! let results = memory.recall("What is the capital of France?", 3)?;
//!
//! // Manage conversations
//! let conv = agent.conversation(None)?;
//! conv.add_message(Role::User, "Hello!")?;
//! conv.add_message(Role::Assistant, "Hi there! How can I help?")?;
//!
//! // Tool state persistence
//! let tool_state = agent.tool_state("calculator")?;
//! tool_state.set("last_result", &42.0)?;
//! ```

pub mod types;
pub mod memory;
pub mod conversation;
pub mod tool_state;

pub use types::{Role, Message, MemoryEntry, AgentId, MemoryId, MessageId, ConversationId};
pub use memory::Memory;
pub use conversation::Conversation;
pub use tool_state::{ToolState, ToolContext};

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use crate::EmbeddedLiath;

const AGENTS_NAMESPACE: &str = "_agents";

/// Metadata for a registered agent
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentMetadata {
    pub id: String,
    pub created_at: u64,
    pub description: Option<String>,
}

/// High-level agent interface
///
/// Agent provides a unified entry point for accessing all agent capabilities:
/// memory, conversations, and tool state.
pub struct Agent {
    id: AgentId,
    db: Arc<EmbeddedLiath>,
}

impl Agent {
    /// Create a new agent with the given ID
    /// This also registers the agent for persistence
    pub fn new(id: &str, db: Arc<EmbeddedLiath>) -> Self {
        let agent = Self {
            id: id.to_string(),
            db,
        };
        // Register the agent (ignore errors for now)
        let _ = agent.register(None);
        agent
    }

    /// Create a new agent with description
    pub fn new_with_description(id: &str, description: &str, db: Arc<EmbeddedLiath>) -> Self {
        let agent = Self {
            id: id.to_string(),
            db,
        };
        let _ = agent.register(Some(description));
        agent
    }

    /// Register this agent in the agents registry
    fn register(&self, description: Option<&str>) -> Result<()> {
        Self::ensure_agents_namespace(&self.db)?;

        // Check if already registered
        let key = format!("agent:{}", self.id);
        if self.db.get(AGENTS_NAMESPACE, key.as_bytes())?.is_some() {
            return Ok(()); // Already registered
        }

        let metadata = AgentMetadata {
            id: self.id.clone(),
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            description: description.map(String::from),
        };

        let metadata_bytes = serde_json::to_vec(&metadata)
            .context("Failed to serialize agent metadata")?;
        self.db.put(AGENTS_NAMESPACE, key.as_bytes(), &metadata_bytes)?;

        // Add to agent index
        let mut index: Vec<String> = if let Some(index_data) = self.db.get(AGENTS_NAMESPACE, b"_agent_index")? {
            serde_json::from_slice(&index_data).unwrap_or_default()
        } else {
            Vec::new()
        };

        if !index.contains(&self.id) {
            index.push(self.id.clone());
            let index_bytes = serde_json::to_vec(&index)?;
            self.db.put(AGENTS_NAMESPACE, b"_agent_index", &index_bytes)?;
        }

        Ok(())
    }

    /// Ensure the agents namespace exists
    fn ensure_agents_namespace(db: &EmbeddedLiath) -> Result<()> {
        #[cfg(feature = "vector")]
        if !db.namespace_exists(AGENTS_NAMESPACE) {
            db.create_namespace(AGENTS_NAMESPACE, 1, usearch::MetricKind::Cos, usearch::ScalarKind::F32)?;
        }
        Ok(())
    }

    /// List all registered agents
    pub fn list_agents(db: &Arc<EmbeddedLiath>) -> Result<Vec<AgentMetadata>> {
        Self::ensure_agents_namespace(db)?;

        let mut agents = Vec::new();

        // Scan for agent entries (they start with "agent:")
        // Since we don't have prefix iteration, we'll use the query executor's scan via Lua
        // For now, we'll iterate through known patterns
        // A better implementation would add prefix scanning to FjallWrapper

        // Use a scan approach - iterate all keys and filter
        // Since we can't easily iterate, we'll check for agents by trying common patterns
        // This is a limitation - in production you'd want proper iteration support

        // For now, let's read the _next_agent_id to know how many to scan
        // Actually, a simpler approach: store an index of agent IDs

        // Read the agent index
        if let Some(index_data) = db.get(AGENTS_NAMESPACE, b"_agent_index")? {
            let index: Vec<String> = serde_json::from_slice(&index_data)
                .unwrap_or_default();

            for agent_id in index {
                let key = format!("agent:{}", agent_id);
                if let Some(data) = db.get(AGENTS_NAMESPACE, key.as_bytes())? {
                    if let Ok(metadata) = serde_json::from_slice::<AgentMetadata>(&data) {
                        agents.push(metadata);
                    }
                }
            }
        }

        Ok(agents)
    }

    /// Load an existing agent by ID
    /// Returns None if the agent is not registered
    pub fn load(id: &str, db: Arc<EmbeddedLiath>) -> Result<Option<Self>> {
        Self::ensure_agents_namespace(&db)?;

        let key = format!("agent:{}", id);
        if db.get(AGENTS_NAMESPACE, key.as_bytes())?.is_some() {
            Ok(Some(Self {
                id: id.to_string(),
                db,
            }))
        } else {
            Ok(None)
        }
    }

    /// Check if an agent exists
    pub fn exists(id: &str, db: &Arc<EmbeddedLiath>) -> Result<bool> {
        Self::ensure_agents_namespace(db)?;
        let key = format!("agent:{}", id);
        Ok(db.get(AGENTS_NAMESPACE, key.as_bytes())?.is_some())
    }

    /// Delete an agent and all its data
    pub fn delete(id: &str, db: &Arc<EmbeddedLiath>) -> Result<()> {
        Self::ensure_agents_namespace(db)?;

        // Remove from registry
        let key = format!("agent:{}", id);
        db.delete(AGENTS_NAMESPACE, key.as_bytes())?;

        // Remove from index
        if let Some(index_data) = db.get(AGENTS_NAMESPACE, b"_agent_index")? {
            let mut index: Vec<String> = serde_json::from_slice(&index_data)
                .unwrap_or_default();
            index.retain(|i| i != id);
            let index_bytes = serde_json::to_vec(&index)?;
            db.put(AGENTS_NAMESPACE, b"_agent_index", &index_bytes)?;
        }

        // Note: This doesn't delete the agent's namespaces (memory, conversations, tool state)
        // Those would need to be deleted separately if desired

        Ok(())
    }

    /// Get the agent's metadata
    pub fn metadata(&self) -> Result<Option<AgentMetadata>> {
        let key = format!("agent:{}", self.id);
        if let Some(data) = self.db.get(AGENTS_NAMESPACE, key.as_bytes())? {
            Ok(serde_json::from_slice(&data).ok())
        } else {
            Ok(None)
        }
    }

    /// Get the agent's ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Access the agent's long-term memory
    pub fn memory(&self) -> Result<Memory> {
        Memory::new(&self.id, self.db.clone())
    }

    /// Create a new conversation or load an existing one
    ///
    /// If `id` is None, creates a new conversation.
    /// If `id` is Some, loads the existing conversation.
    pub fn conversation(&self, id: Option<&str>) -> Result<Conversation> {
        match id {
            Some(conv_id) => Conversation::load(conv_id, &self.id, self.db.clone()),
            None => Conversation::new(&self.id, self.db.clone()),
        }
    }

    /// Get tool state storage for a specific tool
    pub fn tool_state(&self, tool_name: &str) -> Result<ToolState> {
        ToolState::new(&self.id, tool_name, self.db.clone())
    }

    /// Get a tool context for accessing agent capabilities from within a tool
    pub fn tool_context(&self) -> ToolContext {
        ToolContext::new(&self.id, self.db.clone())
    }

    /// Access the underlying database
    pub fn db(&self) -> &Arc<EmbeddedLiath> {
        &self.db
    }

    /// Save all agent data to disk
    pub fn save(&self) -> Result<()> {
        self.db.save()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::Config;

    // Note: These tests require the embedding model to be available
    // They are marked as ignore by default

    #[test]
    #[ignore = "requires embedding model"]
    fn test_agent_memory() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config {
            data_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        let db = Arc::new(EmbeddedLiath::new(config).unwrap());
        let agent = Agent::new("test-agent", db);

        let memory = agent.memory().unwrap();
        let id = memory.store("Test content", &["test"]).unwrap();
        assert!(id > 0);
    }

    #[test]
    #[ignore = "requires embedding model"]
    fn test_agent_conversation() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config {
            data_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        let db = Arc::new(EmbeddedLiath::new(config).unwrap());
        let agent = Agent::new("test-agent", db);

        let conv = agent.conversation(None).unwrap();
        conv.add_message(Role::User, "Hello").unwrap();
        conv.add_message(Role::Assistant, "Hi there!").unwrap();

        assert_eq!(conv.message_count(), 2);
    }

    #[test]
    #[ignore = "requires embedding model"]
    fn test_agent_tool_state() {
        let temp_dir = TempDir::new().unwrap();
        let config = Config {
            data_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };
        let db = Arc::new(EmbeddedLiath::new(config).unwrap());
        let agent = Agent::new("test-agent", db);

        let state = agent.tool_state("calculator").unwrap();
        state.set("result", &42i32).unwrap();

        let result: Option<i32> = state.get("result").unwrap();
        assert_eq!(result, Some(42));
    }
}
