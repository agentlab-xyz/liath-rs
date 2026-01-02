# Error Types

Complete reference for Liath error types and handling.

## LiathError

The main error enum for all Liath operations:

```rust
pub enum LiathError {
    /// Namespace does not exist
    NamespaceNotFound(String),

    /// Namespace already exists
    NamespaceExists(String),

    /// Key not found in namespace
    KeyNotFound(String),

    /// Storage layer error
    Storage(anyhow::Error),

    /// User not authorized for operation
    Unauthorized(String),

    /// Embedding generation failed
    Embedding(String),

    /// Vector search operation failed
    VectorSearch(String),

    /// JSON serialization/deserialization error
    Serialization(String),

    /// Configuration error
    Configuration(String),

    /// Agent operation error
    Agent(String),

    /// Conversation not found
    ConversationNotFound(String),

    /// I/O error
    Io(std::io::Error),

    /// Invalid input provided
    InvalidInput(String),
}
```

## Result Type

```rust
pub type LiathResult<T> = Result<T, LiathError>;
```

## Error Descriptions

### NamespaceNotFound

Raised when accessing a namespace that doesn't exist.

```rust
// Cause
db.get("nonexistent_namespace", b"key")?;

// Handling
match db.get("namespace", b"key") {
    Err(LiathError::NamespaceNotFound(ns)) => {
        println!("Namespace '{}' not found", ns);
        // Create it
        db.create_namespace(&ns, 384, MetricKind::Cos, ScalarKind::F32)?;
    }
    _ => {}
}
```

### NamespaceExists

Raised when creating a namespace that already exists.

```rust
// Cause
db.create_namespace("existing", 384, MetricKind::Cos, ScalarKind::F32)?;
db.create_namespace("existing", 384, MetricKind::Cos, ScalarKind::F32)?; // Error!

// Handling
match db.create_namespace("ns", 384, MetricKind::Cos, ScalarKind::F32) {
    Err(LiathError::NamespaceExists(_)) => {
        // Already exists, continue
    }
    Err(e) => return Err(e),
    Ok(()) => {}
}

// Or check first
if !db.namespace_exists("ns") {
    db.create_namespace("ns", 384, MetricKind::Cos, ScalarKind::F32)?;
}
```

### KeyNotFound

Raised when a required key doesn't exist.

```rust
// Cause (explicit)
let value = db.get("ns", b"key")?
    .ok_or_else(|| LiathError::KeyNotFound("key".into()))?;

// Handling
match db.get("ns", b"key")? {
    Some(value) => process(value),
    None => use_default(),
}
```

### Storage

Raised for underlying storage layer errors.

```rust
// Cause
// - Disk full
// - Permission denied
// - Corruption

// Handling
match db.put("ns", b"key", b"value") {
    Err(LiathError::Storage(e)) => {
        eprintln!("Storage error: {}", e);
        // Log, alert, retry
    }
    _ => {}
}
```

### Unauthorized

Raised when user lacks required permissions.

```rust
// Cause
// User without 'insert' permission tries to write
executor.execute("put('ns', 'key', 'value')", "restricted_user").await?;

// Handling
match executor.execute(code, user).await {
    Err(LiathError::Unauthorized(msg)) => {
        eprintln!("Access denied: {}", msg);
        // Return 403, request permission
    }
    _ => {}
}
```

### Embedding

Raised when embedding generation fails.

```rust
// Cause
// - Model not loaded
// - Invalid text
// - OOM

// Handling
match db.generate_embedding("text") {
    Err(LiathError::Embedding(msg)) => {
        eprintln!("Embedding failed: {}", msg);
        // Use fallback, skip document
    }
    Ok(vec) => store_vector(vec),
}
```

### VectorSearch

Raised when vector search operations fail.

```rust
// Cause
// - Dimension mismatch
// - Index corruption
// - Invalid k

// Handling
match db.semantic_search("ns", "query", 10) {
    Err(LiathError::VectorSearch(msg)) => {
        eprintln!("Search failed: {}", msg);
        // Fall back to keyword search
    }
    Ok(results) => process(results),
}
```

### Serialization

Raised for JSON encoding/decoding errors.

```rust
// Cause
// - Invalid JSON
// - Type mismatch
// - Missing fields

// Handling
match state.get::<MyStruct>("key") {
    Err(LiathError::Serialization(msg)) => {
        eprintln!("Deserialization failed: {}", msg);
        // Use default, migrate data
    }
    Ok(Some(data)) => use_data(data),
    Ok(None) => use_default(),
}
```

### Configuration

Raised for configuration errors.

```rust
// Cause
// - Invalid path
// - Missing required config
// - Invalid values

// Handling
match EmbeddedLiath::new(config) {
    Err(LiathError::Configuration(msg)) => {
        eprintln!("Configuration error: {}", msg);
        // Use defaults, exit
    }
    Ok(db) => use_db(db),
}
```

### Agent

Raised for agent-specific errors.

```rust
// Cause
// - Agent not found
// - Invalid agent ID
// - State corruption

// Handling
match Agent::load("agent-id", db.clone()) {
    Err(LiathError::Agent(msg)) => {
        eprintln!("Agent error: {}", msg);
        // Create new agent
    }
    Ok(Some(agent)) => use_agent(agent),
    Ok(None) => create_new_agent(),
}
```

### ConversationNotFound

Raised when loading a non-existent conversation.

```rust
// Cause
let conv = Conversation::load("nonexistent", "agent", db.clone())?;

// Handling
match Conversation::load(conv_id, agent_id, db.clone()) {
    Err(LiathError::ConversationNotFound(id)) => {
        // Create new conversation
        let conv = agent.conversation(Some(&id))?;
    }
    Ok(conv) => use_conv(conv),
}
```

### Io

Raised for I/O errors.

```rust
// Cause
// - File not found
// - Permission denied
// - Network error

// Handling
match operation() {
    Err(LiathError::Io(e)) => {
        eprintln!("I/O error: {}", e);
        match e.kind() {
            std::io::ErrorKind::NotFound => create_file(),
            std::io::ErrorKind::PermissionDenied => request_permission(),
            _ => return Err(e.into()),
        }
    }
    _ => {}
}
```

### InvalidInput

Raised for invalid input data.

```rust
// Cause
// - Empty namespace name
// - Invalid characters
// - Out of range values

// Handling
match db.create_namespace("", 384, MetricKind::Cos, ScalarKind::F32) {
    Err(LiathError::InvalidInput(msg)) => {
        eprintln!("Invalid input: {}", msg);
        // Show user error, request valid input
    }
    _ => {}
}
```

## Lua Validation Errors

### ValidationError

```rust
pub struct ValidationError {
    pub error_type: ErrorType,
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub suggestion: String,
    pub code_snippet: Option<String>,
}

pub enum ErrorType {
    /// Lua syntax error
    SyntaxError,

    /// Blocked function (os.execute, io.open, etc.)
    ForbiddenFunction,

    /// Variable used without definition
    UndefinedVariable,

    /// Type mismatch in operation
    TypeMismatch,

    /// No return statement
    MissingReturn,

    /// Code too complex
    ComplexityExceeded,
}
```

### RuntimeError

```rust
pub struct RuntimeError {
    pub error_type: RuntimeErrorType,
    pub message: String,
    pub lua_traceback: Option<String>,
    pub suggestion: String,
}

pub enum RuntimeErrorType {
    NamespaceNotFound,
    KeyNotFound,
    DeserializationError,
    EmbeddingError,
    VectorSearchError,
    TimeoutExceeded,
    MemoryLimitExceeded,
}
```

## Error Handling Patterns

### Comprehensive Handler

```rust
fn handle_liath_error(error: LiathError) -> Response {
    match error {
        LiathError::NamespaceNotFound(ns) => {
            Response::not_found(format!("Namespace '{}' not found", ns))
        }
        LiathError::Unauthorized(msg) => {
            Response::forbidden(msg)
        }
        LiathError::InvalidInput(msg) => {
            Response::bad_request(msg)
        }
        LiathError::Embedding(_) | LiathError::VectorSearch(_) => {
            Response::service_unavailable("Search temporarily unavailable")
        }
        _ => {
            Response::internal_error("Internal server error")
        }
    }
}
```

### With Context

```rust
use anyhow::Context;

fn process_document(db: &EmbeddedLiath, id: &str) -> anyhow::Result<String> {
    let content = db.get("docs", id.as_bytes())
        .context("Failed to retrieve document")?
        .ok_or_else(|| anyhow::anyhow!("Document not found"))?;

    let text = String::from_utf8(content)
        .context("Document contains invalid UTF-8")?;

    Ok(text)
}
```

### Retry Pattern

```rust
async fn with_retry<T, F, Fut>(
    f: F,
    max_retries: u32,
) -> LiathResult<T>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = LiathResult<T>>,
{
    let mut last_error = None;

    for attempt in 0..max_retries {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                // Only retry transient errors
                match &e {
                    LiathError::Storage(_) | LiathError::Io(_) => {
                        last_error = Some(e);
                        tokio::time::sleep(Duration::from_millis(100 * 2u64.pow(attempt))).await;
                    }
                    _ => return Err(e),
                }
            }
        }
    }

    Err(last_error.unwrap())
}
```

## See Also

- [Error Handling Guide](../guides/error-handling.md) - Best practices
- [Security Guide](../guides/security.md) - Secure error messages
