# Error Handling

This guide covers error types in Liath and best practices for handling them.

## Error Types

### LiathError

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

    /// User not authorized
    Unauthorized(String),

    /// Embedding generation failed
    Embedding(String),

    /// Vector search failed
    VectorSearch(String),

    /// Serialization/deserialization error
    Serialization(String),

    /// Configuration error
    Configuration(String),

    /// Agent operation error
    Agent(String),

    /// Conversation not found
    ConversationNotFound(String),

    /// I/O error
    Io(std::io::Error),

    /// Invalid input
    InvalidInput(String),
}
```

### Result Type

```rust
pub type LiathResult<T> = Result<T, LiathError>;
```

## Handling Errors

### Basic Pattern

```rust
use liath::{EmbeddedLiath, Config, LiathError};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = EmbeddedLiath::new(Config::default())?;

    match db.get("my_namespace", b"key") {
        Ok(Some(value)) => {
            println!("Found: {:?}", value);
        }
        Ok(None) => {
            println!("Key not found");
        }
        Err(LiathError::NamespaceNotFound(ns)) => {
            println!("Namespace '{}' doesn't exist", ns);
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }

    Ok(())
}
```

### Propagating Errors

```rust
fn process_document(db: &EmbeddedLiath, doc_id: &str) -> LiathResult<String> {
    // Errors automatically propagate with ?
    let content = db.get("documents", doc_id.as_bytes())?
        .ok_or_else(|| LiathError::KeyNotFound(doc_id.to_string()))?;

    let text = String::from_utf8(content)
        .map_err(|e| LiathError::Serialization(e.to_string()))?;

    Ok(text)
}
```

### Error Context

Add context to errors:

```rust
use anyhow::Context;

fn load_agent(db: &EmbeddedLiath, id: &str) -> anyhow::Result<Agent> {
    Agent::load(id, db.clone())?
        .context(format!("Failed to load agent '{}'", id))?
        .ok_or_else(|| anyhow::anyhow!("Agent '{}' not found", id))
}
```

## Lua Error Handling

### Validation Errors

Liath validates Lua code before execution:

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
    SyntaxError,
    ForbiddenFunction,
    UndefinedVariable,
    TypeMismatch,
    MissingReturn,
    ComplexityExceeded,
}
```

### Validation Results

```rust
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub available_functions: Vec<FunctionInfo>,
}

// Check before executing
let result = executor.validate(lua_code);

if !result.valid {
    for error in &result.errors {
        println!("Error at line {}: {}", error.line.unwrap_or(0), error.message);
        println!("Suggestion: {}", error.suggestion);
    }
}
```

### Runtime Errors

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

### Handling Lua Errors

```rust
async fn execute_safe(executor: &QueryExecutor, code: &str, user: &str) -> Result<String, String> {
    // Validate first
    let validation = executor.validate(code);
    if !validation.valid {
        let errors: Vec<String> = validation.errors.iter()
            .map(|e| format!("Line {}: {} - {}", e.line.unwrap_or(0), e.message, e.suggestion))
            .collect();
        return Err(format!("Validation failed:\n{}", errors.join("\n")));
    }

    // Execute
    match executor.execute(code, user).await {
        Ok(result) => Ok(result),
        Err(e) => Err(format!("Execution failed: {}", e)),
    }
}
```

### In Lua Code

```lua
-- Use pcall for safe execution
local function safe_get(namespace, key)
    local ok, result = pcall(function()
        return get(namespace, key)
    end)

    if not ok then
        print("Error getting key:", result)
        return nil
    end
    return result
end

-- Handle missing data
local function get_or_default(namespace, key, default)
    local value = get(namespace, key)
    if value == nil then
        return default
    end
    return json.decode(value)
end

-- Validate before operations
local function safe_semantic_search(namespace, query, k)
    if not namespace or namespace == "" then
        return {error = "Namespace required"}
    end
    if not query or query == "" then
        return {error = "Query required"}
    end
    if k <= 0 then
        return {error = "k must be positive"}
    end

    local ok, results = pcall(function()
        return semantic_search(namespace, query, k)
    end)

    if not ok then
        return {error = tostring(results)}
    end
    return {results = results}
end
```

## Common Errors and Solutions

### NamespaceNotFound

**Cause**: Accessing a namespace that doesn't exist

**Solution**:
```rust
// Check before access
if !db.namespace_exists("my_namespace") {
    db.create_namespace("my_namespace", 384, MetricKind::Cos, ScalarKind::F32)?;
}

// Or handle gracefully
match db.get("my_namespace", key) {
    Err(LiathError::NamespaceNotFound(_)) => {
        // Create and retry, or return default
    }
    other => other,
}
```

### EmbeddingError

**Cause**: Embedding model failed to load or process text

**Solutions**:
```rust
// 1. Check model availability at startup
let test = db.generate_embedding("test");
if test.is_err() {
    eprintln!("Embedding model not available");
}

// 2. Handle empty text
if text.is_empty() {
    return Err(LiathError::InvalidInput("Empty text".into()));
}

// 3. Truncate long text
let text = if text.len() > 10000 {
    &text[..10000]
} else {
    text
};
```

### VectorSearchError

**Cause**: Vector index issues or dimension mismatch

**Solutions**:
```rust
// Verify dimensions match
let embedding = db.generate_embedding("test")?;
assert_eq!(embedding.len(), 384, "Dimension mismatch");

// Create namespace with correct dimensions
db.create_namespace("docs", embedding.len(), MetricKind::Cos, ScalarKind::F32)?;
```

### SerializationError

**Cause**: JSON encoding/decoding failed

**Solutions**:
```rust
// Use proper types
#[derive(Serialize, Deserialize)]
struct MyData {
    field: String,
}

// Handle parse errors
match serde_json::from_slice::<MyData>(&bytes) {
    Ok(data) => data,
    Err(e) => {
        eprintln!("Failed to parse: {}", e);
        MyData::default()
    }
}
```

### Unauthorized

**Cause**: User lacks required permissions

**Solutions**:
```rust
// Check permissions
if !auth_manager.is_authorized(user_id, "select") {
    return Err(LiathError::Unauthorized("select permission required".into()));
}

// Grant permissions
auth_manager.add_permission(user_id, "select".into())?;
```

## Error Recovery Patterns

### Retry with Backoff

```rust
async fn with_retry<T, F, Fut>(
    f: F,
    max_retries: u32,
) -> Result<T, LiathError>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, LiathError>>,
{
    let mut attempts = 0;
    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                attempts += 1;
                if attempts >= max_retries {
                    return Err(e);
                }

                // Exponential backoff
                let delay = std::time::Duration::from_millis(100 * 2u64.pow(attempts));
                tokio::time::sleep(delay).await;
            }
        }
    }
}
```

### Fallback Values

```rust
fn get_with_fallback<T: Default + DeserializeOwned>(
    db: &EmbeddedLiath,
    namespace: &str,
    key: &[u8],
) -> T {
    db.get(namespace, key)
        .ok()
        .flatten()
        .and_then(|bytes| serde_json::from_slice(&bytes).ok())
        .unwrap_or_default()
}
```

### Transaction-like Operations

```rust
fn update_atomically(
    db: &EmbeddedLiath,
    namespace: &str,
    key: &[u8],
    f: impl FnOnce(Option<Vec<u8>>) -> Vec<u8>,
) -> LiathResult<()> {
    // Read
    let current = db.get(namespace, key)?;

    // Transform
    let new_value = f(current);

    // Write
    db.put(namespace, key, &new_value)?;

    Ok(())
}
```

## Logging Errors

```rust
use tracing::{error, warn, info};

async fn process_request(executor: &QueryExecutor, code: &str, user: &str) -> Result<String, ()> {
    match executor.execute(code, user).await {
        Ok(result) => {
            info!(user = %user, "Query executed successfully");
            Ok(result)
        }
        Err(e) => {
            error!(
                user = %user,
                error = %e,
                code_preview = %code.chars().take(100).collect::<String>(),
                "Query execution failed"
            );
            Err(())
        }
    }
}
```

## Next Steps

- [Security](security.md) - Secure error handling
- [Performance](performance.md) - Error impact on performance
- [API Reference](../api/errors.md) - Complete error type reference
