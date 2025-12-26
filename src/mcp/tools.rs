//! MCP tool definitions for Liath operations

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

use crate::query::QueryExecutor;
use crate::EmbeddedLiath;
use crate::agent::{Agent, Role};

/// Tool definition for MCP
#[derive(Debug, Clone, Serialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

impl Tool {
    pub fn new(name: &str, description: &str, input_schema: Value) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            input_schema,
        }
    }
}

/// Content item for tool results
#[derive(Debug, Clone, Serialize)]
pub struct Content {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

impl Content {
    pub fn text(s: impl Into<String>) -> Self {
        Self {
            content_type: "text".to_string(),
            text: s.into(),
        }
    }

    pub fn as_text(&self) -> Option<&Content> {
        if self.content_type == "text" {
            Some(self)
        } else {
            None
        }
    }
}

/// Result of a tool call
#[derive(Debug, Clone, Serialize)]
pub struct CallToolResult {
    pub content: Vec<Content>,
    #[serde(rename = "isError")]
    pub is_error: Option<bool>,
}

impl CallToolResult {
    pub fn success(content: Vec<Content>) -> Self {
        Self {
            content,
            is_error: Some(false),
        }
    }

    pub fn error(content: Vec<Content>) -> Self {
        Self {
            content,
            is_error: Some(true),
        }
    }
}

/// Liath MCP service that provides database tools
pub struct LiathService {
    pub query_executor: Arc<QueryExecutor>,
    pub db: Option<Arc<EmbeddedLiath>>,
    pub user_id: String,
}

impl LiathService {
    pub fn new(query_executor: QueryExecutor, user_id: String) -> Self {
        Self {
            query_executor: Arc::new(query_executor),
            db: None,
            user_id,
        }
    }

    /// Create service with full EmbeddedLiath for agent support
    pub fn with_db(db: Arc<EmbeddedLiath>, user_id: String) -> Self {
        Self {
            query_executor: Arc::new(db.query_executor()),
            db: Some(db),
            user_id,
        }
    }
}

// ============================================================
// Tool Input Types
// ============================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteLuaInput {
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KvGetInput {
    pub namespace: String,
    pub key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KvPutInput {
    pub namespace: String,
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KvDeleteInput {
    pub namespace: String,
    pub key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateNamespaceInput {
    pub name: String,
    #[serde(default)]
    pub dimensions: Option<usize>,
    #[serde(default)]
    pub metric: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteNamespaceInput {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SemanticSearchInput {
    pub namespace: String,
    pub query: String,
    #[serde(default)]
    pub k: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StoreDocumentInput {
    pub namespace: String,
    pub key: String,
    pub text: String,
    pub id: u64,
}

// ============================================================
// Agent Tool Input Types
// ============================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentStoreMemoryInput {
    pub agent_id: String,
    pub content: String,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentRecallMemoryInput {
    pub agent_id: String,
    pub query: String,
    #[serde(default)]
    pub k: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentRecallByTagsInput {
    pub agent_id: String,
    pub tags: Vec<String>,
    #[serde(default)]
    pub k: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentAddMessageInput {
    pub agent_id: String,
    pub conversation_id: String,
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentGetMessagesInput {
    pub agent_id: String,
    pub conversation_id: String,
    #[serde(default)]
    pub last_n: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentCreateInput {
    pub id: String,
    #[serde(default)]
    pub description: Option<String>,
}

// ============================================================
// Tool Definitions
// ============================================================

pub fn get_tools() -> Vec<Tool> {
    vec![
        Tool::new(
            "liath_execute_lua",
            "Execute Lua code against the Liath database. Use for complex queries or custom operations.",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "code": {
                        "type": "string",
                        "description": "Lua code to execute"
                    }
                },
                "required": ["code"]
            }),
        ),
        Tool::new(
            "liath_kv_get",
            "Get a value from Liath key-value storage by namespace and key",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "namespace": { "type": "string", "description": "Namespace name" },
                    "key": { "type": "string", "description": "Key to retrieve" }
                },
                "required": ["namespace", "key"]
            }),
        ),
        Tool::new(
            "liath_kv_put",
            "Store a value in Liath key-value storage",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "namespace": { "type": "string", "description": "Namespace name" },
                    "key": { "type": "string", "description": "Key to store" },
                    "value": { "type": "string", "description": "Value to store" }
                },
                "required": ["namespace", "key", "value"]
            }),
        ),
        Tool::new(
            "liath_kv_delete",
            "Delete a key from Liath storage",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "namespace": { "type": "string", "description": "Namespace name" },
                    "key": { "type": "string", "description": "Key to delete" }
                },
                "required": ["namespace", "key"]
            }),
        ),
        Tool::new(
            "liath_list_namespaces",
            "List all available namespaces in Liath",
            serde_json::json!({ "type": "object", "properties": {} }),
        ),
        Tool::new(
            "liath_create_namespace",
            "Create a new namespace in Liath for storing data and vectors",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Namespace name" },
                    "dimensions": { "type": "integer", "description": "Vector dimensions (default: 384)" },
                    "metric": { "type": "string", "description": "Distance metric: cosine or euclidean" }
                },
                "required": ["name"]
            }),
        ),
        Tool::new(
            "liath_delete_namespace",
            "Delete a namespace and all its data (WARNING: irreversible)",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string", "description": "Namespace name to delete" }
                },
                "required": ["name"]
            }),
        ),
        Tool::new(
            "liath_save",
            "Persist all Liath data to disk",
            serde_json::json!({ "type": "object", "properties": {} }),
        ),
        Tool::new(
            "liath_semantic_search",
            "Search documents using semantic similarity based on meaning",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "namespace": { "type": "string", "description": "Namespace to search" },
                    "query": { "type": "string", "description": "Search query text" },
                    "k": { "type": "integer", "description": "Number of results (default: 5)" }
                },
                "required": ["namespace", "query"]
            }),
        ),
        Tool::new(
            "liath_store_document",
            "Store a document with automatic embedding generation for semantic search",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "namespace": { "type": "string", "description": "Namespace to store in" },
                    "key": { "type": "string", "description": "Document key" },
                    "text": { "type": "string", "description": "Document text content" },
                    "id": { "type": "integer", "description": "Unique ID for vector storage" }
                },
                "required": ["namespace", "key", "text", "id"]
            }),
        ),
        // Agent Tools
        Tool::new(
            "liath_agent_create",
            "Create a new agent for managing memory and conversations",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Unique agent ID" },
                    "description": { "type": "string", "description": "Optional agent description" }
                },
                "required": ["id"]
            }),
        ),
        Tool::new(
            "liath_agent_list",
            "List all registered agents",
            serde_json::json!({ "type": "object", "properties": {} }),
        ),
        Tool::new(
            "liath_agent_store_memory",
            "Store content in agent's long-term semantic memory with optional tags",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "agent_id": { "type": "string", "description": "Agent ID" },
                    "content": { "type": "string", "description": "Content to store in memory" },
                    "tags": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Optional tags for categorization"
                    }
                },
                "required": ["agent_id", "content"]
            }),
        ),
        Tool::new(
            "liath_agent_recall_memory",
            "Recall memories similar to a query using semantic search",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "agent_id": { "type": "string", "description": "Agent ID" },
                    "query": { "type": "string", "description": "Search query" },
                    "k": { "type": "integer", "description": "Number of results (default: 5)" }
                },
                "required": ["agent_id", "query"]
            }),
        ),
        Tool::new(
            "liath_agent_recall_by_tags",
            "Recall memories that have all specified tags",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "agent_id": { "type": "string", "description": "Agent ID" },
                    "tags": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Tags to filter by (all must match)"
                    },
                    "k": { "type": "integer", "description": "Max results (default: 10)" }
                },
                "required": ["agent_id", "tags"]
            }),
        ),
        Tool::new(
            "liath_agent_add_message",
            "Add a message to an agent's conversation",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "agent_id": { "type": "string", "description": "Agent ID" },
                    "conversation_id": { "type": "string", "description": "Conversation ID" },
                    "role": { "type": "string", "description": "Message role: user, assistant, system, or tool" },
                    "content": { "type": "string", "description": "Message content" }
                },
                "required": ["agent_id", "conversation_id", "role", "content"]
            }),
        ),
        Tool::new(
            "liath_agent_get_messages",
            "Get messages from an agent's conversation",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "agent_id": { "type": "string", "description": "Agent ID" },
                    "conversation_id": { "type": "string", "description": "Conversation ID" },
                    "last_n": { "type": "integer", "description": "Get only last N messages" }
                },
                "required": ["agent_id", "conversation_id"]
            }),
        ),
    ]
}

// ============================================================
// Tool Handler
// ============================================================

impl LiathService {
    pub async fn handle_tool_call(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> CallToolResult {
        match name {
            "liath_execute_lua" => {
                match serde_json::from_value::<ExecuteLuaInput>(arguments) {
                    Ok(input) => self.execute_lua(input).await,
                    Err(e) => CallToolResult::error(vec![Content::text(format!("Invalid params: {}", e))]),
                }
            }
            "liath_kv_get" => {
                match serde_json::from_value::<KvGetInput>(arguments) {
                    Ok(input) => self.kv_get(input).await,
                    Err(e) => CallToolResult::error(vec![Content::text(format!("Invalid params: {}", e))]),
                }
            }
            "liath_kv_put" => {
                match serde_json::from_value::<KvPutInput>(arguments) {
                    Ok(input) => self.kv_put(input).await,
                    Err(e) => CallToolResult::error(vec![Content::text(format!("Invalid params: {}", e))]),
                }
            }
            "liath_kv_delete" => {
                match serde_json::from_value::<KvDeleteInput>(arguments) {
                    Ok(input) => self.kv_delete(input).await,
                    Err(e) => CallToolResult::error(vec![Content::text(format!("Invalid params: {}", e))]),
                }
            }
            "liath_list_namespaces" => self.list_namespaces().await,
            "liath_create_namespace" => {
                match serde_json::from_value::<CreateNamespaceInput>(arguments) {
                    Ok(input) => self.create_namespace(input).await,
                    Err(e) => CallToolResult::error(vec![Content::text(format!("Invalid params: {}", e))]),
                }
            }
            "liath_delete_namespace" => {
                match serde_json::from_value::<DeleteNamespaceInput>(arguments) {
                    Ok(input) => self.delete_namespace(input).await,
                    Err(e) => CallToolResult::error(vec![Content::text(format!("Invalid params: {}", e))]),
                }
            }
            "liath_save" => self.save_data().await,
            "liath_semantic_search" => {
                match serde_json::from_value::<SemanticSearchInput>(arguments) {
                    Ok(input) => self.semantic_search(input).await,
                    Err(e) => CallToolResult::error(vec![Content::text(format!("Invalid params: {}", e))]),
                }
            }
            "liath_store_document" => {
                match serde_json::from_value::<StoreDocumentInput>(arguments) {
                    Ok(input) => self.store_document(input).await,
                    Err(e) => CallToolResult::error(vec![Content::text(format!("Invalid params: {}", e))]),
                }
            }
            // Agent tools
            "liath_agent_create" => {
                match serde_json::from_value::<AgentCreateInput>(arguments) {
                    Ok(input) => self.agent_create(input).await,
                    Err(e) => CallToolResult::error(vec![Content::text(format!("Invalid params: {}", e))]),
                }
            }
            "liath_agent_list" => self.agent_list().await,
            "liath_agent_store_memory" => {
                match serde_json::from_value::<AgentStoreMemoryInput>(arguments) {
                    Ok(input) => self.agent_store_memory(input).await,
                    Err(e) => CallToolResult::error(vec![Content::text(format!("Invalid params: {}", e))]),
                }
            }
            "liath_agent_recall_memory" => {
                match serde_json::from_value::<AgentRecallMemoryInput>(arguments) {
                    Ok(input) => self.agent_recall_memory(input).await,
                    Err(e) => CallToolResult::error(vec![Content::text(format!("Invalid params: {}", e))]),
                }
            }
            "liath_agent_recall_by_tags" => {
                match serde_json::from_value::<AgentRecallByTagsInput>(arguments) {
                    Ok(input) => self.agent_recall_by_tags(input).await,
                    Err(e) => CallToolResult::error(vec![Content::text(format!("Invalid params: {}", e))]),
                }
            }
            "liath_agent_add_message" => {
                match serde_json::from_value::<AgentAddMessageInput>(arguments) {
                    Ok(input) => self.agent_add_message(input).await,
                    Err(e) => CallToolResult::error(vec![Content::text(format!("Invalid params: {}", e))]),
                }
            }
            "liath_agent_get_messages" => {
                match serde_json::from_value::<AgentGetMessagesInput>(arguments) {
                    Ok(input) => self.agent_get_messages(input).await,
                    Err(e) => CallToolResult::error(vec![Content::text(format!("Invalid params: {}", e))]),
                }
            }
            _ => CallToolResult::error(vec![Content::text(format!("Unknown tool: {}", name))]),
        }
    }

    async fn execute_lua(&self, input: ExecuteLuaInput) -> CallToolResult {
        match self.query_executor.execute(&input.code, &self.user_id).await {
            Ok(result) => CallToolResult::success(vec![Content::text(result)]),
            Err(e) => CallToolResult::error(vec![Content::text(format!("Error: {}", e))]),
        }
    }

    async fn kv_get(&self, input: KvGetInput) -> CallToolResult {
        match self.query_executor.get(&input.namespace, input.key.as_bytes()) {
            Ok(Some(value)) => {
                let text = String::from_utf8_lossy(&value).to_string();
                CallToolResult::success(vec![Content::text(text)])
            }
            Ok(None) => CallToolResult::success(vec![Content::text("(nil)")]),
            Err(e) => CallToolResult::error(vec![Content::text(format!("Error: {}", e))]),
        }
    }

    async fn kv_put(&self, input: KvPutInput) -> CallToolResult {
        match self.query_executor.put(&input.namespace, input.key.as_bytes(), input.value.as_bytes()) {
            Ok(_) => CallToolResult::success(vec![Content::text("OK")]),
            Err(e) => CallToolResult::error(vec![Content::text(format!("Error: {}", e))]),
        }
    }

    async fn kv_delete(&self, input: KvDeleteInput) -> CallToolResult {
        match self.query_executor.delete(&input.namespace, input.key.as_bytes()) {
            Ok(_) => CallToolResult::success(vec![Content::text("Deleted")]),
            Err(e) => CallToolResult::error(vec![Content::text(format!("Error: {}", e))]),
        }
    }

    async fn list_namespaces(&self) -> CallToolResult {
        let namespaces = self.query_executor.list_namespaces();
        let result = if namespaces.is_empty() {
            "No namespaces found.".to_string()
        } else {
            namespaces.join("\n")
        };
        CallToolResult::success(vec![Content::text(result)])
    }

    async fn create_namespace(&self, input: CreateNamespaceInput) -> CallToolResult {
        let dims = input.dimensions.unwrap_or(384);
        let metric = input.metric.as_deref().unwrap_or("cosine");

        #[cfg(feature = "vector")]
        {
            use usearch::{MetricKind, ScalarKind};
            let metric_kind = match metric.to_lowercase().as_str() {
                "euclidean" | "l2" => MetricKind::L2sq,
                _ => MetricKind::Cos,
            };
            match self.query_executor.create_namespace(&input.name, dims, metric_kind, ScalarKind::F32) {
                Ok(_) => CallToolResult::success(vec![Content::text(
                    format!("Created namespace '{}' ({}D, {})", input.name, dims, metric)
                )]),
                Err(e) => CallToolResult::error(vec![Content::text(format!("Error: {}", e))]),
            }
        }
        #[cfg(not(feature = "vector"))]
        {
            let _ = (dims, metric);
            CallToolResult::error(vec![Content::text("Vector feature not enabled")])
        }
    }

    async fn delete_namespace(&self, input: DeleteNamespaceInput) -> CallToolResult {
        match self.query_executor.delete_namespace(&input.name) {
            Ok(_) => CallToolResult::success(vec![Content::text(
                format!("Deleted namespace '{}'", input.name)
            )]),
            Err(e) => CallToolResult::error(vec![Content::text(format!("Error: {}", e))]),
        }
    }

    async fn save_data(&self) -> CallToolResult {
        match self.query_executor.save_all() {
            Ok(_) => CallToolResult::success(vec![Content::text("All data saved")]),
            Err(e) => CallToolResult::error(vec![Content::text(format!("Error: {}", e))]),
        }
    }

    async fn semantic_search(&self, input: SemanticSearchInput) -> CallToolResult {
        let k = input.k.unwrap_or(5);

        let embeddings = match self.query_executor.generate_embedding(vec![input.query.as_str()]) {
            Ok(e) => e,
            Err(e) => return CallToolResult::error(vec![Content::text(format!("Embedding error: {}", e))]),
        };

        let query_vector = match embeddings.into_iter().next() {
            Some(v) => v,
            None => return CallToolResult::error(vec![Content::text("Failed to generate embedding")]),
        };

        match self.query_executor.similarity_search(&input.namespace, &query_vector, k) {
            Ok(results) => {
                let output: Vec<String> = results
                    .iter()
                    .map(|(id, distance)| format!("ID: {}, Distance: {:.4}", id, distance))
                    .collect();
                let result_text = if output.is_empty() {
                    "No results found".to_string()
                } else {
                    output.join("\n")
                };
                CallToolResult::success(vec![Content::text(result_text)])
            }
            Err(e) => CallToolResult::error(vec![Content::text(format!("Search error: {}", e))]),
        }
    }

    async fn store_document(&self, input: StoreDocumentInput) -> CallToolResult {
        let embeddings = match self.query_executor.generate_embedding(vec![input.text.as_str()]) {
            Ok(e) => e,
            Err(e) => return CallToolResult::error(vec![Content::text(format!("Embedding error: {}", e))]),
        };

        let vector = match embeddings.into_iter().next() {
            Some(v) => v,
            None => return CallToolResult::error(vec![Content::text("Failed to generate embedding")]),
        };

        if let Err(e) = self.query_executor.put(&input.namespace, input.key.as_bytes(), input.text.as_bytes()) {
            return CallToolResult::error(vec![Content::text(format!("Storage error: {}", e))]);
        }

        if let Err(e) = self.query_executor.add_vector(&input.namespace, input.id, &vector) {
            return CallToolResult::error(vec![Content::text(format!("Vector error: {}", e))]);
        }

        // Store ID -> key mapping for semantic search lookup
        let mapping_key = format!("_vidx:{}", input.id);
        if let Err(e) = self.query_executor.put(&input.namespace, mapping_key.as_bytes(), input.key.as_bytes()) {
            return CallToolResult::error(vec![Content::text(format!("Mapping error: {}", e))]);
        }

        CallToolResult::success(vec![Content::text(
            format!("Stored document '{}' with ID {}", input.key, input.id)
        )])
    }

    // ============================================================
    // Agent Tool Handlers
    // ============================================================

    fn require_db(&self) -> Result<&Arc<EmbeddedLiath>, CallToolResult> {
        self.db.as_ref().ok_or_else(|| {
            CallToolResult::error(vec![Content::text(
                "Agent tools require EmbeddedLiath (use LiathService::with_db)"
            )])
        })
    }

    async fn agent_create(&self, input: AgentCreateInput) -> CallToolResult {
        let db = match self.require_db() {
            Ok(db) => db,
            Err(err) => return err,
        };

        let agent = if let Some(desc) = input.description {
            Agent::new_with_description(&input.id, &desc, db.clone())
        } else {
            Agent::new(&input.id, db.clone())
        };

        CallToolResult::success(vec![Content::text(
            format!("Created agent '{}'", agent.id())
        )])
    }

    async fn agent_list(&self) -> CallToolResult {
        let db = match self.require_db() {
            Ok(db) => db,
            Err(err) => return err,
        };

        match Agent::list_agents(db) {
            Ok(agents) => {
                if agents.is_empty() {
                    CallToolResult::success(vec![Content::text("No agents registered")])
                } else {
                    let list: Vec<String> = agents.iter().map(|a| {
                        match &a.description {
                            Some(desc) => format!("- {} ({})", a.id, desc),
                            None => format!("- {}", a.id),
                        }
                    }).collect();
                    CallToolResult::success(vec![Content::text(list.join("\n"))])
                }
            }
            Err(e) => CallToolResult::error(vec![Content::text(format!("Error: {}", e))]),
        }
    }

    async fn agent_store_memory(&self, input: AgentStoreMemoryInput) -> CallToolResult {
        let db = match self.require_db() {
            Ok(db) => db,
            Err(err) => return err,
        };

        let agent = Agent::new(&input.agent_id, db.clone());
        let memory = match agent.memory() {
            Ok(m) => m,
            Err(e) => return CallToolResult::error(vec![Content::text(format!("Memory error: {}", e))]),
        };

        let tags: Vec<&str> = input.tags
            .as_ref()
            .map(|t| t.iter().map(|s| s.as_str()).collect())
            .unwrap_or_default();

        match memory.store(&input.content, &tags) {
            Ok(id) => CallToolResult::success(vec![Content::text(
                format!("Stored memory with ID {}", id)
            )]),
            Err(e) => CallToolResult::error(vec![Content::text(format!("Error: {}", e))]),
        }
    }

    async fn agent_recall_memory(&self, input: AgentRecallMemoryInput) -> CallToolResult {
        let db = match self.require_db() {
            Ok(db) => db,
            Err(err) => return err,
        };

        let agent = Agent::new(&input.agent_id, db.clone());
        let memory = match agent.memory() {
            Ok(m) => m,
            Err(e) => return CallToolResult::error(vec![Content::text(format!("Memory error: {}", e))]),
        };

        let k = input.k.unwrap_or(5);
        match memory.recall(&input.query, k) {
            Ok(entries) => {
                if entries.is_empty() {
                    CallToolResult::success(vec![Content::text("No memories found")])
                } else {
                    let output: Vec<String> = entries.iter().map(|e| {
                        format!("[ID: {}, Distance: {:.4}] {}", e.id, e.distance, e.content)
                    }).collect();
                    CallToolResult::success(vec![Content::text(output.join("\n\n"))])
                }
            }
            Err(e) => CallToolResult::error(vec![Content::text(format!("Error: {}", e))]),
        }
    }

    async fn agent_recall_by_tags(&self, input: AgentRecallByTagsInput) -> CallToolResult {
        let db = match self.require_db() {
            Ok(db) => db,
            Err(err) => return err,
        };

        let agent = Agent::new(&input.agent_id, db.clone());
        let memory = match agent.memory() {
            Ok(m) => m,
            Err(e) => return CallToolResult::error(vec![Content::text(format!("Memory error: {}", e))]),
        };

        let tags: Vec<&str> = input.tags.iter().map(|s| s.as_str()).collect();
        let k = input.k.unwrap_or(10);

        match memory.recall_by_tags(&tags, k) {
            Ok(entries) => {
                if entries.is_empty() {
                    CallToolResult::success(vec![Content::text("No memories found with those tags")])
                } else {
                    let output: Vec<String> = entries.iter().map(|e| {
                        format!("[ID: {}] {}", e.id, e.content)
                    }).collect();
                    CallToolResult::success(vec![Content::text(output.join("\n\n"))])
                }
            }
            Err(e) => CallToolResult::error(vec![Content::text(format!("Error: {}", e))]),
        }
    }

    async fn agent_add_message(&self, input: AgentAddMessageInput) -> CallToolResult {
        let db = match self.require_db() {
            Ok(db) => db,
            Err(err) => return err,
        };

        let agent = Agent::new(&input.agent_id, db.clone());
        let conversation = match agent.conversation(Some(&input.conversation_id)) {
            Ok(c) => c,
            Err(e) => return CallToolResult::error(vec![Content::text(format!("Conversation error: {}", e))]),
        };

        let role = match input.role.to_lowercase().as_str() {
            "user" => Role::User,
            "assistant" => Role::Assistant,
            "system" => Role::System,
            other => Role::Tool(other.to_string()),
        };

        match conversation.add_message(role, &input.content) {
            Ok(_) => CallToolResult::success(vec![Content::text("Message added")]),
            Err(e) => CallToolResult::error(vec![Content::text(format!("Error: {}", e))]),
        }
    }

    async fn agent_get_messages(&self, input: AgentGetMessagesInput) -> CallToolResult {
        let db = match self.require_db() {
            Ok(db) => db,
            Err(err) => return err,
        };

        let agent = Agent::new(&input.agent_id, db.clone());
        let conversation = match agent.conversation(Some(&input.conversation_id)) {
            Ok(c) => c,
            Err(e) => return CallToolResult::error(vec![Content::text(format!("Conversation error: {}", e))]),
        };

        let messages = match input.last_n {
            Some(n) => match conversation.last_n(n) {
                Ok(m) => m,
                Err(e) => return CallToolResult::error(vec![Content::text(format!("Error: {}", e))]),
            },
            None => match conversation.messages() {
                Ok(m) => m,
                Err(e) => return CallToolResult::error(vec![Content::text(format!("Error: {}", e))]),
            },
        };

        if messages.is_empty() {
            CallToolResult::success(vec![Content::text("No messages in conversation")])
        } else {
            let output: Vec<String> = messages.iter().map(|m| {
                let role = match &m.role {
                    Role::User => "User",
                    Role::Assistant => "Assistant",
                    Role::System => "System",
                    Role::Tool(name) => name,
                };
                format!("[{}] {}", role, m.content)
            }).collect();
            CallToolResult::success(vec![Content::text(output.join("\n\n"))])
        }
    }
}
