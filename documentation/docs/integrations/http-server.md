# HTTP Server

Liath includes an optional HTTP server for REST API access.

## Enabling the Server

Add the `server` feature to your `Cargo.toml`:

```toml
[dependencies]
liath = { version = "0.1", features = ["server"] }
```

## Starting the Server

### Via CLI

```bash
# Default (localhost:8080)
liath server

# Custom host and port
liath server --host 0.0.0.0 --port 3000

# With data directory
liath server --data-dir /var/lib/liath
```

### Programmatically

```rust
use liath::{EmbeddedLiath, Config};
use liath::server::run_server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db = EmbeddedLiath::new(Config::default())?;
    let executor = db.query_executor();

    run_server(executor, "0.0.0.0:8080").await?;

    Ok(())
}
```

## API Endpoints

### Health Check

```http
GET /health
```

**Response:**

```json
{
    "status": "healthy",
    "version": "0.1.0",
    "uptime_secs": 3600
}
```

### Metrics

```http
GET /metrics
```

**Response:**

```json
{
    "namespaces": 5,
    "requests_total": 1234,
    "uptime_secs": 3600
}
```

### Namespaces

#### List Namespaces

```http
GET /namespaces
```

**Response:**

```json
{
    "namespaces": ["default", "documents", "memories"]
}
```

#### Create Namespace

```http
POST /namespaces
Content-Type: application/json

{
    "name": "documents",
    "dimensions": 384,
    "metric": "cosine"
}
```

**Response:**

```json
{
    "status": "created",
    "name": "documents"
}
```

#### Delete Namespace

```http
DELETE /namespaces/{name}
```

**Response:**

```json
{
    "status": "deleted"
}
```

### Key-Value Operations

#### Get Value

```http
GET /kv/{namespace}/{key}
```

**Response:**

```json
{
    "key": "user:1",
    "value": "{\"name\": \"Alice\"}"
}
```

#### Put Value

```http
POST /kv/{namespace}/{key}
Content-Type: application/json

{
    "value": "{\"name\": \"Alice\"}"
}
```

**Response:**

```json
{
    "status": "stored"
}
```

#### Delete Value

```http
DELETE /kv/{namespace}/{key}
```

**Response:**

```json
{
    "status": "deleted"
}
```

### Semantic Search

```http
POST /search/{namespace}
Content-Type: application/json

{
    "query": "machine learning algorithms",
    "k": 10
}
```

**Response:**

```json
{
    "results": [
        {
            "id": "doc:1",
            "content": "Introduction to neural networks...",
            "distance": 0.123
        },
        {
            "id": "doc:2",
            "content": "Deep learning fundamentals...",
            "distance": 0.234
        }
    ]
}
```

### Embeddings

```http
POST /embed
Content-Type: application/json

{
    "texts": ["Hello world", "Goodbye world"]
}
```

**Response:**

```json
{
    "embeddings": [
        [0.123, -0.456, ...],
        [0.789, -0.012, ...]
    ],
    "dimensions": 384
}
```

### Lua Query Execution

```http
POST /query
Content-Type: application/json

{
    "query": "local x = semantic_search('docs', 'AI', 5)\nreturn json.encode(x)",
    "user_id": "api_user"
}
```

**Response:**

```json
{
    "result": "[{\"id\":\"doc:1\",\"content\":\"...\",\"distance\":0.1}]"
}
```

## Request/Response Types

### QueryRequest

```rust
struct QueryRequest {
    query: String,
    user_id: String,
}
```

### CreateNamespaceRequest

```rust
struct CreateNamespaceRequest {
    name: String,
    dimensions: usize,
    metric: String,  // "cosine", "euclidean", "ip"
}
```

### SemanticSearchRequest

```rust
struct SemanticSearchRequest {
    query: String,
    k: usize,
}
```

### KvPutRequest

```rust
struct KvPutRequest {
    value: String,
}
```

## Authentication

### API Key Authentication

```rust
use axum::middleware;

async fn auth_middleware(
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Response {
    let api_key = headers
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok());

    match api_key {
        Some(key) if is_valid_key(key) => next.run(request).await,
        _ => StatusCode::UNAUTHORIZED.into_response(),
    }
}

let app = Router::new()
    .route("/query", post(handle_query))
    .layer(middleware::from_fn(auth_middleware));
```

### Bearer Token

```rust
use axum_extra::headers::{Authorization, authorization::Bearer};

async fn handle_query(
    TypedHeader(auth): TypedHeader<Authorization<Bearer>>,
    Json(request): Json<QueryRequest>,
) -> impl IntoResponse {
    let token = auth.token();
    // Validate token...
}
```

## Error Responses

### 400 Bad Request

```json
{
    "error": "Invalid input",
    "message": "Namespace name cannot be empty"
}
```

### 401 Unauthorized

```json
{
    "error": "Unauthorized",
    "message": "Invalid or missing API key"
}
```

### 404 Not Found

```json
{
    "error": "Not found",
    "message": "Namespace 'unknown' does not exist"
}
```

### 500 Internal Server Error

```json
{
    "error": "Internal error",
    "message": "An unexpected error occurred"
}
```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `LIATH_HOST` | Bind address | `127.0.0.1` |
| `LIATH_PORT` | Port number | `8080` |
| `LIATH_DATA_DIR` | Data directory | `./liath_data` |
| `LIATH_LOG_LEVEL` | Log level | `info` |

### TLS/HTTPS

```rust
use axum_server::tls_rustls::RustlsConfig;

let tls_config = RustlsConfig::from_pem_file(
    "cert.pem",
    "key.pem"
).await?;

axum_server::bind_rustls("0.0.0.0:443".parse()?, tls_config)
    .serve(app.into_make_service())
    .await?;
```

### CORS

```rust
use tower_http::cors::{CorsLayer, Any};

let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods([Method::GET, Method::POST, Method::DELETE])
    .allow_headers([CONTENT_TYPE, AUTHORIZATION]);

let app = Router::new()
    .route("/query", post(handle_query))
    .layer(cors);
```

### Rate Limiting

```rust
use tower_governor::{GovernorConfigBuilder, GovernorLayer};

let governor_conf = GovernorConfigBuilder::default()
    .per_second(10)
    .burst_size(50)
    .finish()
    .unwrap();

let app = Router::new()
    .route("/query", post(handle_query))
    .layer(GovernorLayer {
        config: Box::leak(Box::new(governor_conf)),
    });
```

## Client Examples

### curl

```bash
# Create namespace
curl -X POST http://localhost:8080/namespaces \
  -H "Content-Type: application/json" \
  -d '{"name": "docs", "dimensions": 384, "metric": "cosine"}'

# Store document
curl -X POST http://localhost:8080/kv/docs/doc:1 \
  -H "Content-Type: application/json" \
  -d '{"value": "Hello, world!"}'

# Semantic search
curl -X POST http://localhost:8080/search/docs \
  -H "Content-Type: application/json" \
  -d '{"query": "greeting", "k": 5}'

# Execute Lua
curl -X POST http://localhost:8080/query \
  -H "Content-Type: application/json" \
  -d '{"query": "return 1 + 2", "user_id": "test"}'
```

### JavaScript/TypeScript

```typescript
const LIATH_URL = 'http://localhost:8080';

async function semanticSearch(namespace: string, query: string, k: number) {
    const response = await fetch(`${LIATH_URL}/search/${namespace}`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ query, k }),
    });
    return response.json();
}

async function executeQuery(query: string, userId: string) {
    const response = await fetch(`${LIATH_URL}/query`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ query, user_id: userId }),
    });
    return response.json();
}
```

### Python

```python
import requests

LIATH_URL = 'http://localhost:8080'

def semantic_search(namespace: str, query: str, k: int = 10):
    response = requests.post(
        f'{LIATH_URL}/search/{namespace}',
        json={'query': query, 'k': k}
    )
    return response.json()

def execute_query(query: str, user_id: str):
    response = requests.post(
        f'{LIATH_URL}/query',
        json={'query': query, 'user_id': user_id}
    )
    return response.json()
```

## See Also

- [Security Guide](../guides/security.md) - Server security
- [Performance Guide](../guides/performance.md) - Optimization
- [MCP Server](mcp-server.md) - AI assistant integration
