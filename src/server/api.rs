use axum::{extract::State, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use crate::query::QueryExecutor;

#[derive(Deserialize)]
struct QueryRequest {
    query: String,
    user_id: String,
}

#[derive(Serialize)]
struct QueryResponse {
    result: String,
}

enum WorkerMsg {
    Execute {
        query: String,
        user_id: String,
        resp: oneshot::Sender<String>,
    },
}

#[derive(Clone)]
struct AppState {
    tx: mpsc::Sender<WorkerMsg>,
}

async fn execute_query(State(state): State<AppState>, Json(payload): Json<QueryRequest>) -> Json<QueryResponse> {
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
            }
        }
    });

    let app_state = AppState { tx };

    let app = Router::new()
        .route("/query", post(execute_query))
        .with_state(app_state);

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    println!("AI-First DB Server listening on {}", addr);
    // Run the server within the LocalSet so both the server and the local worker can make progress
    local
        .run_until(async move { axum::Server::bind(&addr).serve(app.into_make_service()).await })
        .await?;
    Ok(())
}
