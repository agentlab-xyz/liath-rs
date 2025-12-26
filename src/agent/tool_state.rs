//! Tool state persistence for agents

use std::sync::Arc;
use anyhow::{Result, Context};
use serde::{de::DeserializeOwned, Serialize};
use crate::EmbeddedLiath;

/// Persistent state storage for a tool
///
/// ToolState provides key-value storage for tools to persist their state
/// across invocations. Each tool gets its own isolated namespace.
pub struct ToolState {
    agent_id: String,
    tool_name: String,
    namespace: String,
    db: Arc<EmbeddedLiath>,
}

impl ToolState {
    /// Create a new ToolState instance
    pub fn new(agent_id: &str, tool_name: &str, db: Arc<EmbeddedLiath>) -> Result<Self> {
        let namespace = format!("agent_{}_tool_{}", agent_id, tool_name);

        // Create namespace if it doesn't exist (using minimal dimensions since we don't need vectors)
        #[cfg(feature = "vector")]
        if !db.namespace_exists(&namespace) {
            db.create_namespace(&namespace, 1, usearch::MetricKind::Cos, usearch::ScalarKind::F32)?;
        }

        Ok(Self {
            agent_id: agent_id.to_string(),
            tool_name: tool_name.to_string(),
            namespace,
            db,
        })
    }

    /// Get a value by key, deserializing from JSON
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>> {
        let full_key = format!("state:{}", key);
        match self.db.get(&self.namespace, full_key.as_bytes())? {
            Some(data) => {
                let value: T = serde_json::from_slice(&data)
                    .context(format!("Failed to deserialize tool state for key '{}'", key))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// Set a value by key, serializing to JSON
    pub fn set<T: Serialize>(&self, key: &str, value: &T) -> Result<()> {
        let full_key = format!("state:{}", key);
        let data = serde_json::to_vec(value)
            .context(format!("Failed to serialize tool state for key '{}'", key))?;
        self.db.put(&self.namespace, full_key.as_bytes(), &data)?;
        Ok(())
    }

    /// Delete a value by key
    pub fn delete(&self, key: &str) -> Result<()> {
        let full_key = format!("state:{}", key);
        self.db.delete(&self.namespace, full_key.as_bytes())?;
        Ok(())
    }

    /// Check if a key exists
    pub fn exists(&self, key: &str) -> Result<bool> {
        let full_key = format!("state:{}", key);
        Ok(self.db.get(&self.namespace, full_key.as_bytes())?.is_some())
    }

    /// Get the agent ID
    pub fn agent_id(&self) -> &str {
        &self.agent_id
    }

    /// Get the tool name
    pub fn tool_name(&self) -> &str {
        &self.tool_name
    }
}

/// Context provides access to agent capabilities within a tool
///
/// This is a simplified interface that tools can use to access
/// memory, conversation history, and other agent capabilities.
pub struct ToolContext {
    agent_id: String,
    db: Arc<EmbeddedLiath>,
}

impl ToolContext {
    /// Create a new tool context
    pub fn new(agent_id: &str, db: Arc<EmbeddedLiath>) -> Self {
        Self {
            agent_id: agent_id.to_string(),
            db,
        }
    }

    /// Get state storage for a specific tool
    pub fn state(&self, tool_name: &str) -> Result<ToolState> {
        ToolState::new(&self.agent_id, tool_name, self.db.clone())
    }

    /// Generate an embedding for text
    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        self.db.generate_embedding(text)
    }

    /// Access the underlying database
    pub fn db(&self) -> &Arc<EmbeddedLiath> {
        &self.db
    }
}
