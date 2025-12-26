//! Conversation history management for agents

use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::{Result, Context};
use crate::EmbeddedLiath;
use super::types::{Role, Message, MessageId, ConversationId, ConversationMetadata};

/// Conversation history for an agent
///
/// Manages message history within a conversation, supporting both
/// ordered retrieval and semantic search through past messages.
pub struct Conversation {
    id: ConversationId,
    agent_id: String,
    namespace: String,
    db: Arc<EmbeddedLiath>,
    next_msg_id: std::sync::atomic::AtomicU64,
}

impl Conversation {
    /// Create a new conversation for an agent
    pub fn new(agent_id: &str, db: Arc<EmbeddedLiath>) -> Result<Self> {
        let id = uuid::Uuid::new_v4().to_string();
        Self::create_with_id(&id, agent_id, db)
    }

    /// Create a conversation with a specific ID
    fn create_with_id(id: &str, agent_id: &str, db: Arc<EmbeddedLiath>) -> Result<Self> {
        let namespace = format!("agent_{}_conv_{}", agent_id, id);

        // Create namespace if it doesn't exist
        #[cfg(feature = "vector")]
        if !db.namespace_exists(&namespace) {
            db.create_namespace(&namespace, 384, usearch::MetricKind::Cos, usearch::ScalarKind::F32)?;
        }

        // Store conversation metadata
        let metadata = ConversationMetadata {
            id: id.to_string(),
            agent_id: agent_id.to_string(),
            created_at: Self::current_timestamp(),
            message_count: 0,
        };
        let metadata_bytes = serde_json::to_vec(&metadata)
            .context("Failed to serialize conversation metadata")?;
        db.put(&namespace, b"_metadata", &metadata_bytes)?;

        Ok(Self {
            id: id.to_string(),
            agent_id: agent_id.to_string(),
            namespace,
            db,
            next_msg_id: std::sync::atomic::AtomicU64::new(1),
        })
    }

    /// Load an existing conversation
    pub fn load(id: &str, agent_id: &str, db: Arc<EmbeddedLiath>) -> Result<Self> {
        let namespace = format!("agent_{}_conv_{}", agent_id, id);

        // Load metadata to verify conversation exists
        let metadata_bytes = db.get(&namespace, b"_metadata")?
            .ok_or_else(|| anyhow::anyhow!("Conversation not found: {}", id))?;
        let metadata: ConversationMetadata = serde_json::from_slice(&metadata_bytes)
            .context("Failed to deserialize conversation metadata")?;

        Ok(Self {
            id: id.to_string(),
            agent_id: agent_id.to_string(),
            namespace,
            db,
            next_msg_id: std::sync::atomic::AtomicU64::new(metadata.message_count + 1),
        })
    }

    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    fn get_next_msg_id(&self) -> MessageId {
        self.next_msg_id.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }

    /// Add a message to the conversation
    pub fn add_message(&self, role: Role, content: &str) -> Result<MessageId> {
        let id = self.get_next_msg_id();
        let timestamp = Self::current_timestamp();

        let message = Message {
            id,
            role,
            content: content.to_string(),
            timestamp,
        };

        // Store message
        let msg_key = format!("msg:{:016x}", id); // Zero-padded for lexicographic ordering
        let msg_bytes = serde_json::to_vec(&message)
            .context("Failed to serialize message")?;
        self.db.put(&self.namespace, msg_key.as_bytes(), &msg_bytes)?;

        // Generate and store embedding for semantic search
        let embedding = self.db.generate_embedding(content)?;
        self.db.add_vector(&self.namespace, id, &embedding)?;

        // Update message count in metadata
        self.update_message_count(id)?;

        Ok(id)
    }

    fn update_message_count(&self, count: u64) -> Result<()> {
        if let Some(data) = self.db.get(&self.namespace, b"_metadata")? {
            let mut metadata: ConversationMetadata = serde_json::from_slice(&data)?;
            metadata.message_count = count;
            let metadata_bytes = serde_json::to_vec(&metadata)?;
            self.db.put(&self.namespace, b"_metadata", &metadata_bytes)?;
        }
        Ok(())
    }

    /// Get all messages in the conversation (ordered by ID)
    pub fn messages(&self) -> Result<Vec<Message>> {
        let mut messages = Vec::new();

        // Scan for all messages - this is a simplified implementation
        // A more efficient approach would use range queries
        let current_id = self.next_msg_id.load(std::sync::atomic::Ordering::SeqCst);
        for i in 1..current_id {
            let msg_key = format!("msg:{:016x}", i);
            if let Some(data) = self.db.get(&self.namespace, msg_key.as_bytes())? {
                let msg: Message = serde_json::from_slice(&data)?;
                messages.push(msg);
            }
        }

        Ok(messages)
    }

    /// Get the last N messages
    pub fn last_n(&self, n: usize) -> Result<Vec<Message>> {
        let current_id = self.next_msg_id.load(std::sync::atomic::Ordering::SeqCst);
        let start_id = if current_id > n as u64 { current_id - n as u64 } else { 1 };

        let mut messages = Vec::new();
        for i in start_id..current_id {
            let msg_key = format!("msg:{:016x}", i);
            if let Some(data) = self.db.get(&self.namespace, msg_key.as_bytes())? {
                let msg: Message = serde_json::from_slice(&data)?;
                messages.push(msg);
            }
        }

        Ok(messages)
    }

    /// Search messages by semantic similarity
    pub fn search(&self, query: &str, k: usize) -> Result<Vec<Message>> {
        let query_embedding = self.db.generate_embedding(query)?;
        let results = self.db.search_vectors(&self.namespace, &query_embedding, k)?;

        let mut messages = Vec::new();
        for (id, _distance) in results {
            let msg_key = format!("msg:{:016x}", id);
            if let Some(data) = self.db.get(&self.namespace, msg_key.as_bytes())? {
                let msg: Message = serde_json::from_slice(&data)?;
                messages.push(msg);
            }
        }

        Ok(messages)
    }

    /// Get the conversation ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the agent ID
    pub fn agent_id(&self) -> &str {
        &self.agent_id
    }

    /// Get the number of messages in this conversation
    pub fn message_count(&self) -> u64 {
        self.next_msg_id.load(std::sync::atomic::Ordering::SeqCst) - 1
    }
}
