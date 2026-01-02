# QueryExecutor API

The QueryExecutor handles Lua code execution with database access.

## Overview

```rust
use liath::EmbeddedLiath;

let db = EmbeddedLiath::new(Config::default())?;
let executor = db.query_executor();
```

## Getting the Executor

```rust
// From EmbeddedLiath
let executor = db.query_executor();

// The executor is cloneable
let executor2 = executor.clone();
```

## Methods

### execute

Execute Lua code and return the result.

```rust
async fn execute(&self, code: &str, user_id: &str) -> Result<String, Error>
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `code` | `&str` | Lua code to execute |
| `user_id` | `&str` | User ID for permission checking |

**Returns:** `String` - The Lua return value as a string

**Example:**

```rust
// Simple calculation
let result = executor.execute("return 1 + 2", "user").await?;
assert_eq!(result, "3");

// With database operations
let result = executor.execute(r#"
    put("test", "key", "value")
    return get("test", "key")
"#, "user").await?;

// Complex query
let result = executor.execute(r#"
    store_with_embedding("docs", "d1", "Rust programming guide")
    local results = semantic_search("docs", "programming", 5)
    return json.encode(results)
"#, "user").await?;
```

---

### validate

Validate Lua code without executing.

```rust
fn validate(&self, code: &str) -> ValidationResult
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| `code` | `&str` | Lua code to validate |

**Returns:** `ValidationResult` - Validation result with errors/warnings

**Example:**

```rust
let result = executor.validate(r#"
    local x = 1
    return x + y  -- y is undefined
"#);

if !result.valid {
    for error in &result.errors {
        println!("Line {}: {} - {}",
            error.line.unwrap_or(0),
            error.message,
            error.suggestion
        );
    }
}
```

## Validation Types

### ValidationResult

```rust
pub struct ValidationResult {
    /// Whether the code is valid
    pub valid: bool,

    /// List of errors
    pub errors: Vec<ValidationError>,

    /// List of warnings
    pub warnings: Vec<ValidationWarning>,

    /// Available functions in scope
    pub available_functions: Vec<FunctionInfo>,
}
```

### ValidationError

```rust
pub struct ValidationError {
    /// Type of error
    pub error_type: ErrorType,

    /// Error message
    pub message: String,

    /// Line number (if available)
    pub line: Option<usize>,

    /// Column number (if available)
    pub column: Option<usize>,

    /// Suggested fix
    pub suggestion: String,

    /// Code snippet showing error
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

### ValidationWarning

```rust
pub struct ValidationWarning {
    pub warning_type: WarningType,
    pub message: String,
    pub line: Option<usize>,
    pub suggestion: String,
}

pub enum WarningType {
    MissingReturn,
    UnusedVariable,
    DeprecatedFunction,
}
```

## Namespace Operations

The executor provides direct namespace operations:

### create_namespace

```rust
executor.create_namespace("docs", 384, MetricKind::Cos, ScalarKind::F32)?;
```

### list_namespaces

```rust
let namespaces = executor.list_namespaces();
for ns in namespaces {
    println!("Namespace: {}", ns);
}
```

### namespace_exists

```rust
if executor.namespace_exists("docs") {
    println!("Namespace exists");
}
```

### delete_namespace

```rust
executor.delete_namespace("old_namespace")?;
```

## Storage Operations

### put

```rust
executor.put("namespace", b"key", b"value")?;
```

### get

```rust
if let Some(value) = executor.get("namespace", b"key")? {
    println!("Value: {:?}", value);
}
```

### delete

```rust
executor.delete("namespace", b"key")?;
```

## Embedding Operations

### generate_embedding

```rust
let texts = vec!["Hello", "World"];
let embeddings = executor.generate_embedding(texts)?;
```

## Vector Operations

### similarity_search

```rust
let results = executor.similarity_search("namespace", &query_vec, 10)?;
for (id, distance) in results {
    println!("ID: {}, Distance: {}", id, distance);
}
```

### add_vector

```rust
executor.add_vector("namespace", 123, &vector)?;
```

## Persistence

### save_all

Save all namespaces to disk:

```rust
executor.save_all()?;
```

## Concurrency

The executor uses a semaphore to limit concurrent operations:

```rust
// Constructor parameter controls concurrency
let executor = QueryExecutor::new(
    namespace_manager,
    embedding_wrapper,
    lua_vm,
    file_storage,
    auth_manager,
    10  // max_concurrent_embedding
);
```

## Usage Patterns

### Safe Execution

```rust
async fn safe_execute(
    executor: &QueryExecutor,
    code: &str,
    user: &str,
) -> Result<String, String> {
    // Validate first
    let validation = executor.validate(code);
    if !validation.valid {
        let errors: Vec<String> = validation.errors.iter()
            .map(|e| format!("Line {}: {}", e.line.unwrap_or(0), e.message))
            .collect();
        return Err(errors.join("\n"));
    }

    // Execute
    executor.execute(code, user).await
        .map_err(|e| e.to_string())
}
```

### Batch Operations

```rust
async fn batch_store(
    executor: &QueryExecutor,
    documents: Vec<(&str, &str)>,  // (id, content)
) -> Result<(), Error> {
    let code = documents.iter()
        .map(|(id, content)| {
            format!(r#"store_with_embedding("docs", "{}", "{}")"#,
                id.replace('"', r#"\""#),
                content.replace('"', r#"\""#))
        })
        .collect::<Vec<_>>()
        .join("\n");

    executor.execute(&format!("{}\nreturn 'done'", code), "system").await?;
    Ok(())
}
```

### With Timeout

```rust
use tokio::time::{timeout, Duration};

async fn execute_with_timeout(
    executor: &QueryExecutor,
    code: &str,
    user: &str,
    timeout_ms: u64,
) -> Result<String, Error> {
    timeout(
        Duration::from_millis(timeout_ms),
        executor.execute(code, user)
    ).await
    .map_err(|_| LiathError::InvalidInput("Execution timeout".into()))?
}
```

## Error Handling

```rust
use liath::LiathError;

match executor.execute(code, user).await {
    Ok(result) => {
        println!("Result: {}", result);
    }
    Err(LiathError::Unauthorized(msg)) => {
        println!("Permission denied: {}", msg);
    }
    Err(LiathError::InvalidInput(msg)) => {
        println!("Invalid input: {}", msg);
    }
    Err(e) => {
        println!("Execution error: {}", e);
    }
}
```

## See Also

- [Lua Standard Library](lua-stdlib.md) - Available Lua functions
- [EmbeddedLiath API](embedded-liath.md) - Parent database interface
- [Security Guide](../guides/security.md) - Safe execution practices
