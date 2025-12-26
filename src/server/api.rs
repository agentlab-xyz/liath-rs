use axum::{
    extract::{Path, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, oneshot};
use crate::query::QueryExecutor;

// ========== Request/Response Types ==========

#[derive(Deserialize)]
struct QueryRequest {
    query: String,
    user_id: String,
}

#[derive(Serialize)]
struct QueryResponse {
    result: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
    uptime_secs: u64,
}

#[derive(Serialize)]
struct MetricsResponse {
    namespaces: usize,
    requests_total: u64,
    uptime_secs: u64,
}

#[derive(Serialize)]
struct NamespacesResponse {
    namespaces: Vec<String>,
}

#[derive(Deserialize)]
struct CreateNamespaceRequest {
    name: String,
    #[serde(default = "default_dimensions")]
    dimensions: usize,
    #[serde(default = "default_metric")]
    metric: String,
}

fn default_dimensions() -> usize { 384 }
fn default_metric() -> String { "cosine".to_string() }

#[derive(Serialize)]
struct SuccessResponse {
    success: bool,
    message: String,
}

#[derive(Deserialize)]
struct KvPutRequest {
    value: String,
}

#[derive(Serialize)]
struct KvGetResponse {
    key: String,
    value: Option<String>,
}

#[derive(Deserialize)]
struct SemanticSearchRequest {
    query: String,
    #[serde(default = "default_k")]
    k: usize,
}

fn default_k() -> usize { 5 }

#[derive(Serialize)]
struct SemanticSearchResult {
    id: u64,
    content: String,
    distance: f32,
}

#[derive(Serialize)]
struct SemanticSearchResponse {
    results: Vec<SemanticSearchResult>,
}

#[derive(Deserialize)]
struct EmbedRequest {
    texts: Vec<String>,
}

#[derive(Serialize)]
struct EmbedResponse {
    embeddings: Vec<Vec<f32>>,
}

// ========== Worker Message ==========

enum WorkerMsg {
    Execute {
        query: String,
        user_id: String,
        resp: oneshot::Sender<String>,
    },
    GetNamespaceCount {
        resp: oneshot::Sender<usize>,
    },
    ListNamespaces {
        resp: oneshot::Sender<Vec<String>>,
    },
    CreateNamespace {
        name: String,
        dimensions: usize,
        metric: String,
        resp: oneshot::Sender<Result<(), String>>,
    },
    DeleteNamespace {
        name: String,
        resp: oneshot::Sender<Result<(), String>>,
    },
    KvGet {
        namespace: String,
        key: String,
        resp: oneshot::Sender<Result<Option<String>, String>>,
    },
    KvPut {
        namespace: String,
        key: String,
        value: String,
        resp: oneshot::Sender<Result<(), String>>,
    },
    KvDelete {
        namespace: String,
        key: String,
        resp: oneshot::Sender<Result<(), String>>,
    },
    SemanticSearch {
        namespace: String,
        query: String,
        k: usize,
        resp: oneshot::Sender<Result<Vec<(u64, String, f32)>, String>>,
    },
    GenerateEmbeddings {
        texts: Vec<String>,
        resp: oneshot::Sender<Result<Vec<Vec<f32>>, String>>,
    },
}

// ========== App State ==========

#[derive(Clone)]
struct AppState {
    tx: mpsc::Sender<WorkerMsg>,
    start_time: u64,
    requests: Arc<std::sync::atomic::AtomicU64>,
}

impl AppState {
    fn new(tx: mpsc::Sender<WorkerMsg>) -> Self {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            tx,
            start_time,
            requests: Arc::new(std::sync::atomic::AtomicU64::new(0)),
        }
    }

    fn uptime(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now.saturating_sub(self.start_time)
    }
}

// ========== Handlers ==========

async fn execute_query(State(state): State<AppState>, Json(payload): Json<QueryRequest>) -> Json<QueryResponse> {
    state.requests.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    let (tx, rx) = oneshot::channel();
    let _ = state
        .tx
        .send(WorkerMsg::Execute {
            query: payload.query,
            user_id: payload.user_id,
            resp: tx,
        })
        .await;

    let result = rx.await.unwrap_or_else(|e| format!("Recv error: {}", e));
    Json(QueryResponse { result })
}

async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_secs: state.uptime(),
    })
}

async fn metrics(State(state): State<AppState>) -> Json<MetricsResponse> {
    // Get namespace count from worker
    let (tx, rx) = oneshot::channel();
    let namespace_count = if state.tx.send(WorkerMsg::GetNamespaceCount { resp: tx }).await.is_ok() {
        rx.await.unwrap_or(0)
    } else {
        0
    };

    Json(MetricsResponse {
        namespaces: namespace_count,
        requests_total: state.requests.load(std::sync::atomic::Ordering::Relaxed),
        uptime_secs: state.uptime(),
    })
}

async fn list_namespaces(State(state): State<AppState>) -> Json<NamespacesResponse> {
    let (tx, rx) = oneshot::channel();
    let namespaces = if state.tx.send(WorkerMsg::ListNamespaces { resp: tx }).await.is_ok() {
        rx.await.unwrap_or_default()
    } else {
        Vec::new()
    };
    Json(NamespacesResponse { namespaces })
}

async fn create_namespace(
    State(state): State<AppState>,
    Json(payload): Json<CreateNamespaceRequest>,
) -> Json<SuccessResponse> {
    let (tx, rx) = oneshot::channel();
    let _ = state.tx.send(WorkerMsg::CreateNamespace {
        name: payload.name.clone(),
        dimensions: payload.dimensions,
        metric: payload.metric,
        resp: tx,
    }).await;

    match rx.await {
        Ok(Ok(())) => Json(SuccessResponse {
            success: true,
            message: format!("Created namespace '{}'", payload.name),
        }),
        Ok(Err(e)) => Json(SuccessResponse {
            success: false,
            message: e,
        }),
        Err(_) => Json(SuccessResponse {
            success: false,
            message: "Worker communication error".to_string(),
        }),
    }
}

async fn delete_namespace_handler(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Json<SuccessResponse> {
    let (tx, rx) = oneshot::channel();
    let _ = state.tx.send(WorkerMsg::DeleteNamespace {
        name: name.clone(),
        resp: tx,
    }).await;

    match rx.await {
        Ok(Ok(())) => Json(SuccessResponse {
            success: true,
            message: format!("Deleted namespace '{}'", name),
        }),
        Ok(Err(e)) => Json(SuccessResponse {
            success: false,
            message: e,
        }),
        Err(_) => Json(SuccessResponse {
            success: false,
            message: "Worker communication error".to_string(),
        }),
    }
}

async fn kv_get(
    State(state): State<AppState>,
    Path((namespace, key)): Path<(String, String)>,
) -> Json<KvGetResponse> {
    let (tx, rx) = oneshot::channel();
    let _ = state.tx.send(WorkerMsg::KvGet {
        namespace,
        key: key.clone(),
        resp: tx,
    }).await;

    match rx.await {
        Ok(Ok(value)) => Json(KvGetResponse { key, value }),
        Ok(Err(_)) | Err(_) => Json(KvGetResponse { key, value: None }),
    }
}

async fn kv_put(
    State(state): State<AppState>,
    Path((namespace, key)): Path<(String, String)>,
    Json(payload): Json<KvPutRequest>,
) -> Json<SuccessResponse> {
    let (tx, rx) = oneshot::channel();
    let _ = state.tx.send(WorkerMsg::KvPut {
        namespace,
        key: key.clone(),
        value: payload.value,
        resp: tx,
    }).await;

    match rx.await {
        Ok(Ok(())) => Json(SuccessResponse {
            success: true,
            message: format!("Stored key '{}'", key),
        }),
        Ok(Err(e)) => Json(SuccessResponse {
            success: false,
            message: e,
        }),
        Err(_) => Json(SuccessResponse {
            success: false,
            message: "Worker communication error".to_string(),
        }),
    }
}

async fn kv_delete(
    State(state): State<AppState>,
    Path((namespace, key)): Path<(String, String)>,
) -> Json<SuccessResponse> {
    let (tx, rx) = oneshot::channel();
    let _ = state.tx.send(WorkerMsg::KvDelete {
        namespace,
        key: key.clone(),
        resp: tx,
    }).await;

    match rx.await {
        Ok(Ok(())) => Json(SuccessResponse {
            success: true,
            message: format!("Deleted key '{}'", key),
        }),
        Ok(Err(e)) => Json(SuccessResponse {
            success: false,
            message: e,
        }),
        Err(_) => Json(SuccessResponse {
            success: false,
            message: "Worker communication error".to_string(),
        }),
    }
}

async fn semantic_search_handler(
    State(state): State<AppState>,
    Path(namespace): Path<String>,
    Json(payload): Json<SemanticSearchRequest>,
) -> Json<SemanticSearchResponse> {
    let (tx, rx) = oneshot::channel();
    let _ = state.tx.send(WorkerMsg::SemanticSearch {
        namespace,
        query: payload.query,
        k: payload.k,
        resp: tx,
    }).await;

    match rx.await {
        Ok(Ok(results)) => Json(SemanticSearchResponse {
            results: results.into_iter().map(|(id, content, distance)| {
                SemanticSearchResult { id, content, distance }
            }).collect(),
        }),
        _ => Json(SemanticSearchResponse { results: Vec::new() }),
    }
}

async fn embed_handler(
    State(state): State<AppState>,
    Json(payload): Json<EmbedRequest>,
) -> Json<EmbedResponse> {
    let (tx, rx) = oneshot::channel();
    let _ = state.tx.send(WorkerMsg::GenerateEmbeddings {
        texts: payload.texts,
        resp: tx,
    }).await;

    match rx.await {
        Ok(Ok(embeddings)) => Json(EmbedResponse { embeddings }),
        _ => Json(EmbedResponse { embeddings: Vec::new() }),
    }
}

// ========== Server ==========

pub async fn run_server(port: u16, query_executor: QueryExecutor) -> anyhow::Result<()> {
    // channel between axum handlers and the worker
    let (tx, mut rx) = mpsc::channel::<WorkerMsg>(64);

    // spawn a local task for the worker on the current runtime
    let local = tokio::task::LocalSet::new();
    local.spawn_local(async move {
        while let Some(msg) = rx.recv().await {
            match msg {
                WorkerMsg::Execute { query, user_id, resp } => {
                    let out = query_executor
                        .execute(&query, &user_id)
                        .await
                        .unwrap_or_else(|e| format!("Error: {}", e));
                    let _ = resp.send(out);
                }
                WorkerMsg::GetNamespaceCount { resp } => {
                    let count = query_executor.list_namespaces().len();
                    let _ = resp.send(count);
                }
                WorkerMsg::ListNamespaces { resp } => {
                    let namespaces = query_executor.list_namespaces();
                    let _ = resp.send(namespaces);
                }
                WorkerMsg::CreateNamespace { name, dimensions, metric, resp } => {
                    #[cfg(feature = "vector")]
                    {
                        use usearch::{MetricKind, ScalarKind};
                        let metric_kind = match metric.to_lowercase().as_str() {
                            "euclidean" | "l2" => MetricKind::L2sq,
                            _ => MetricKind::Cos,
                        };
                        let result = query_executor.create_namespace(&name, dimensions, metric_kind, ScalarKind::F32)
                            .map_err(|e| e.to_string());
                        let _ = resp.send(result);
                    }
                    #[cfg(not(feature = "vector"))]
                    {
                        let _ = (dimensions, metric);
                        let _ = resp.send(Err("Vector feature not enabled".to_string()));
                    }
                }
                WorkerMsg::DeleteNamespace { name, resp } => {
                    let result = query_executor.delete_namespace(&name)
                        .map_err(|e| e.to_string());
                    let _ = resp.send(result);
                }
                WorkerMsg::KvGet { namespace, key, resp } => {
                    let result = query_executor.get(&namespace, key.as_bytes())
                        .map(|opt| opt.map(|v| String::from_utf8_lossy(&v).to_string()))
                        .map_err(|e| e.to_string());
                    let _ = resp.send(result);
                }
                WorkerMsg::KvPut { namespace, key, value, resp } => {
                    let result = query_executor.put(&namespace, key.as_bytes(), value.as_bytes())
                        .map_err(|e| e.to_string());
                    let _ = resp.send(result);
                }
                WorkerMsg::KvDelete { namespace, key, resp } => {
                    let result = query_executor.delete(&namespace, key.as_bytes())
                        .map_err(|e| e.to_string());
                    let _ = resp.send(result);
                }
                WorkerMsg::SemanticSearch { namespace, query, k, resp } => {
                    // Generate embedding
                    let result = match query_executor.generate_embedding(vec![query.as_str()]) {
                        Ok(embeddings) => {
                            match embeddings.into_iter().next() {
                                Some(query_vec) => {
                                    match query_executor.similarity_search(&namespace, &query_vec, k) {
                                        Ok(results) => {
                                            // Get content for each result using ID mapping
                                            let mut output = Vec::new();
                                            for (id, distance) in results {
                                                let mapping_key = format!("_vidx:{}", id);
                                                let content = if let Ok(Some(key)) = query_executor.get(&namespace, mapping_key.as_bytes()) {
                                                    if let Ok(Some(data)) = query_executor.get(&namespace, &key) {
                                                        String::from_utf8_lossy(&data).to_string()
                                                    } else {
                                                        String::new()
                                                    }
                                                } else {
                                                    String::new()
                                                };
                                                output.push((id, content, distance));
                                            }
                                            Ok(output)
                                        }
                                        Err(e) => Err(e.to_string()),
                                    }
                                }
                                None => Err("Failed to generate embedding".to_string()),
                            }
                        }
                        Err(e) => Err(e.to_string()),
                    };
                    let _ = resp.send(result);
                }
                WorkerMsg::GenerateEmbeddings { texts, resp } => {
                    let text_refs: Vec<&str> = texts.iter().map(|s| s.as_str()).collect();
                    let result = query_executor.generate_embedding(text_refs)
                        .map_err(|e| e.to_string());
                    let _ = resp.send(result);
                }
            }
        }
    });

    let app_state = AppState::new(tx);

    let app = Router::new()
        .route("/query", post(execute_query))
        .route("/health", get(health))
        .route("/metrics", get(metrics))
        .route("/namespaces", get(list_namespaces))
        .route("/namespaces", post(create_namespace))
        .route("/namespaces/{name}", delete(delete_namespace_handler))
        .route("/kv/{namespace}/{key}", get(kv_get))
        .route("/kv/{namespace}/{key}", put(kv_put))
        .route("/kv/{namespace}/{key}", delete(kv_delete))
        .route("/semantic/{namespace}", post(semantic_search_handler))
        .route("/embed", post(embed_handler))
        .with_state(app_state);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    println!("Liath DB Server listening on {}", addr);

    // axum 0.7 style: use TcpListener and axum::serve
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    // Run the server within the LocalSet so both the server and the local worker can make progress
    local
        .run_until(async move { axum::serve(listener, app).await })
        .await?;

    Ok(())
}
