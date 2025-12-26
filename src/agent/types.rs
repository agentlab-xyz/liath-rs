//! Core types for the agent module

use serde::{Serialize, Deserialize};

/// Unique identifier for an agent
pub type AgentId = String;

/// Unique identifier for a memory entry
pub type MemoryId = u64;

/// Unique identifier for a conversation
pub type ConversationId = String;

/// Unique identifier for a message
pub type MessageId = u64;

/// Role of a message sender
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Role {
    /// User message
    User,
    /// Assistant/AI response
    Assistant,
    /// System message
    System,
    /// Tool/function call result
    Tool(String),
}

impl Role {
    pub fn as_str(&self) -> &str {
        match self {
            Role::User => "user",
            Role::Assistant => "assistant",
            Role::System => "system",
            Role::Tool(_) => "tool",
        }
    }
}

/// A message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub role: Role,
    pub content: String,
    pub timestamp: u64,
}

/// An entry in long-term memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    pub id: MemoryId,
    pub content: String,
    pub tags: Vec<String>,
    pub distance: f32,
    pub created_at: u64,
}

/// Metadata for a stored memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct MemoryMetadata {
    pub id: MemoryId,
    pub tags: Vec<String>,
    pub created_at: u64,
}

/// Metadata for a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ConversationMetadata {
    pub id: ConversationId,
    pub agent_id: AgentId,
    pub created_at: u64,
    pub message_count: u64,
}
