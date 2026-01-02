# Security

This guide covers security considerations and best practices for Liath deployments.

## Lua Sandbox Security

### Blocked Operations

The Lua runtime blocks all system access:

| Category | Blocked Functions | Risk Mitigated |
|----------|-------------------|----------------|
| File System | `io.*`, `file.*` | Data exfiltration, modification |
| OS Access | `os.execute`, `os.exit`, `os.remove` | Command injection |
| Code Loading | `loadfile`, `dofile`, `loadstring` | Remote code execution |
| Debugging | `debug.*` | Sandbox escape |
| Modules | `require`, `package.*` | Arbitrary code loading |

### Testing Sandbox

```lua
-- All these attempts will fail safely
local tests = {
    function() os.execute("whoami") end,
    function() io.open("/etc/passwd", "r") end,
    function() require("socket") end,
    function() loadfile("malicious.lua")() end,
    function() debug.debug() end,
}

for i, test in ipairs(tests) do
    local ok, err = pcall(test)
    if ok then
        print("SECURITY ISSUE: Test " .. i .. " succeeded!")
    else
        print("Test " .. i .. " blocked: " .. tostring(err))
    end
end
```

### Sandbox Guarantees

1. **No file system access**: Cannot read/write files
2. **No network access**: Cannot make HTTP requests
3. **No process execution**: Cannot run shell commands
4. **No code injection**: Cannot load external Lua code
5. **Resource limits**: Memory and CPU constraints

## Authentication

### AuthManager

```rust
use liath::AuthManager;

// In-memory (testing)
let mut auth = AuthManager::new();

// Persistent (production)
let mut auth = AuthManager::with_persistence(Path::new("/var/lib/liath/auth"))?;

// Add users with permissions
auth.add_user("admin", vec![
    "select".into(),
    "insert".into(),
    "update".into(),
    "delete".into(),
    "create_namespace".into(),
    "delete_namespace".into(),
    "generate_embedding".into(),
    "similarity_search".into(),
]);

auth.add_user("reader", vec!["select".into(), "similarity_search".into()]);

// Check authorization
if !auth.is_authorized("reader", "delete") {
    return Err(LiathError::Unauthorized("Insufficient permissions".into()));
}
```

### Available Permissions

| Permission | Description | Risk Level |
|------------|-------------|------------|
| `select` | Read from KV store | Low |
| `insert` | Write new values | Medium |
| `update` | Modify existing values | Medium |
| `delete` | Remove values | High |
| `create_namespace` | Create namespaces | Medium |
| `delete_namespace` | Remove namespaces | High |
| `upload_file` | Upload files | High |
| `process_file` | Process files | Medium |
| `generate_embedding` | Generate embeddings | Low |
| `similarity_search` | Vector search | Low |
| `install_package` | Install Lua packages | Critical |
| `list_packages` | List packages | Low |

### Permission Patterns

```rust
// Minimal permissions for read-only agent
auth.add_user("read_agent", vec![
    "select".into(),
    "similarity_search".into(),
    "generate_embedding".into(),
]);

// Full permissions for admin
auth.add_user("admin", vec![
    "select".into(),
    "insert".into(),
    "update".into(),
    "delete".into(),
    "create_namespace".into(),
    "delete_namespace".into(),
    "upload_file".into(),
    "process_file".into(),
    "generate_embedding".into(),
    "similarity_search".into(),
    "install_package".into(),
    "list_packages".into(),
]);

// Application-specific permissions
auth.add_user("app_service", vec![
    "select".into(),
    "insert".into(),
    "similarity_search".into(),
    "generate_embedding".into(),
]);
```

## Input Validation

### Validate Lua Code

```rust
// Always validate before execution
let validation = executor.validate(user_code);

if !validation.valid {
    // Log the attempt
    warn!(
        user = %user_id,
        errors = ?validation.errors,
        "Invalid Lua code submitted"
    );

    return Err(LiathError::InvalidInput("Invalid code".into()));
}

// Check for suspicious patterns
fn is_suspicious(code: &str) -> bool {
    let suspicious_patterns = [
        "while true",
        "repeat until false",
        "for i=1,1e10",
        "string.rep",
    ];

    suspicious_patterns.iter().any(|p| code.contains(p))
}

if is_suspicious(user_code) {
    warn!(user = %user_id, "Suspicious code pattern detected");
    return Err(LiathError::InvalidInput("Suspicious code pattern".into()));
}
```

### Sanitize User Input

```lua
local function sanitize_key(key)
    -- Remove potentially dangerous characters
    return key:gsub("[^%w%-%_]", "")
end

local function sanitize_content(content)
    -- Limit length
    if #content > 10000 then
        return content:sub(1, 10000)
    end
    return content
end

-- Use in operations
local safe_key = sanitize_key(user_provided_key)
local safe_content = sanitize_content(user_provided_content)
put("namespace", safe_key, safe_content)
```

### Validate Namespaces

```rust
fn validate_namespace_name(name: &str) -> Result<(), LiathError> {
    // Length limits
    if name.is_empty() || name.len() > 64 {
        return Err(LiathError::InvalidInput("Namespace name must be 1-64 characters".into()));
    }

    // Character restrictions
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        return Err(LiathError::InvalidInput("Invalid namespace characters".into()));
    }

    // Reserved names
    let reserved = ["system", "admin", "root", "internal"];
    if reserved.contains(&name) {
        return Err(LiathError::InvalidInput("Reserved namespace name".into()));
    }

    Ok(())
}
```

## Data Protection

### Sensitive Data Handling

```rust
// Don't store sensitive data in plain text
// Instead, use a separate secrets manager or encryption

fn store_sensitive(db: &EmbeddedLiath, key: &str, value: &str) -> LiathResult<()> {
    // Hash or encrypt before storage
    let encrypted = encrypt(value)?;  // Use actual encryption
    db.put("secrets", key.as_bytes(), &encrypted)
}

// Mask sensitive data in logs
fn log_safe(data: &str) -> String {
    if data.len() > 4 {
        format!("{}...{}", &data[..2], &data[data.len()-2..])
    } else {
        "****".to_string()
    }
}
```

### Namespace Isolation

```rust
// Ensure agents can only access their own namespaces
fn agent_namespace(agent_id: &str) -> String {
    format!("agent:{}", agent_id)
}

fn validate_access(agent_id: &str, namespace: &str) -> bool {
    let allowed_prefix = agent_namespace(agent_id);
    namespace.starts_with(&allowed_prefix) || namespace == "shared"
}
```

### Memory Safety

```lua
-- Prevent memory exhaustion
local function safe_search(namespace, query, k)
    -- Limit k to prevent memory issues
    local safe_k = math.min(k, 100)
    return semantic_search(namespace, query, safe_k)
end

-- Limit stored data size
local function safe_store(namespace, key, value)
    if #value > 1000000 then  -- 1MB limit
        return nil, "Value too large"
    end
    return put(namespace, key, value)
end
```

## HTTP Server Security

### Enable HTTPS

Always use HTTPS in production:

```rust
// Configure with TLS
use axum_server::tls_rustls::RustlsConfig;

let config = RustlsConfig::from_pem_file(
    "cert.pem",
    "key.pem"
).await?;

axum_server::bind_rustls(addr, config)
    .serve(app.into_make_service())
    .await?;
```

### Rate Limiting

```rust
use tower::ServiceBuilder;
use tower_governor::{GovernorConfigBuilder, GovernorLayer};

let governor_conf = GovernorConfigBuilder::default()
    .per_second(10)
    .burst_size(50)
    .finish()
    .unwrap();

let app = Router::new()
    .route("/query", post(handle_query))
    .layer(
        ServiceBuilder::new()
            .layer(GovernorLayer {
                config: Box::leak(Box::new(governor_conf)),
            })
    );
```

### CORS Configuration

```rust
use tower_http::cors::{CorsLayer, Any};

let cors = CorsLayer::new()
    .allow_origin(["https://yourdomain.com".parse().unwrap()])
    .allow_methods([Method::GET, Method::POST])
    .allow_headers([CONTENT_TYPE, AUTHORIZATION]);

let app = Router::new()
    .route("/api/query", post(handle_query))
    .layer(cors);
```

## Audit Logging

```rust
use tracing::{info, warn, error};

async fn handle_query(
    user: User,
    Query(params): Query<QueryParams>,
) -> Result<Json<Response>, Error> {
    // Log the attempt
    info!(
        user = %user.id,
        query_hash = %hash(&params.code),
        "Query attempt"
    );

    // Execute
    let result = executor.execute(&params.code, &user.id).await;

    // Log result
    match &result {
        Ok(_) => info!(user = %user.id, "Query successful"),
        Err(e) => warn!(user = %user.id, error = %e, "Query failed"),
    }

    result
}
```

## Security Checklist

### Development

- [ ] Validate all Lua code before execution
- [ ] Use typed parameters, not string concatenation
- [ ] Implement proper error handling without exposing internals
- [ ] Test sandbox with attempted escapes

### Deployment

- [ ] Enable HTTPS
- [ ] Configure authentication
- [ ] Set up rate limiting
- [ ] Restrict network access
- [ ] Use non-root user
- [ ] Encrypt data at rest

### Operations

- [ ] Monitor for unusual patterns
- [ ] Rotate credentials regularly
- [ ] Keep dependencies updated
- [ ] Back up data securely
- [ ] Have incident response plan

## Next Steps

- [Error Handling](error-handling.md) - Secure error messages
- [Performance](performance.md) - Security vs performance trade-offs
- [HTTP Server](../integrations/http-server.md) - Server security details
