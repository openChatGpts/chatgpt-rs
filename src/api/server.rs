use std::net::SocketAddr;

use axum::{
    Json, Router,
    extract::State,
    http::{Method, header::CONTENT_TYPE},
    response::IntoResponse,
    routing::{delete, get, post},
};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

use crate::utils::{ChatGptError, Result as ChatGptResult};
use super::{handlers, state::AppState};

pub fn router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::POST, Method::GET, Method::OPTIONS, Method::DELETE])
        .allow_headers([CONTENT_TYPE, axum::http::header::AUTHORIZATION]);

    Router::new()
        // Threads endpoints
        .route("/v1/threads", post(handlers::create_thread))
        .route("/v1/threads", get(handlers::list_threads))
        .route("/v1/threads/{thread_id}", get(handlers::get_thread))
        .route("/v1/threads/{thread_id}", delete(handlers::delete_thread))
        // Messages endpoints
        .route("/v1/threads/{thread_id}/messages", post(handlers::add_message))
        .route("/v1/threads/{thread_id}/messages", get(handlers::list_messages))
        // Responses endpoint
        .route("/v1/responses", post(handlers::create_response))
        // Health and models
        .route("/health", get(health_check))
        .route("/v1/models", get(list_models))
        .with_state(state)
        .layer(cors)
}

/// Health check endpoint
async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let threads = state.list_threads().await;
    let proxy_info = state.get_default_proxy().unwrap_or("none");
    
    Json(serde_json::json!({
        "status": "ok",
        "default_proxy": proxy_info,
        "active_threads": threads.len(),
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// List models endpoint (OpenAI compatibility)
async fn list_models() -> impl IntoResponse {
    Json(serde_json::json!({
        "object": "list",
        "data": [
            {
                "id": "gpt-4",
                "object": "model",
                "created": 1677610602,
                "owned_by": "openai"
            },
            {
                "id": "gpt-4o",
                "object": "model",
                "created": 1715367049,
                "owned_by": "openai"
            }
        ]
    }))
}

/// Run the API server with the provided host and port.
pub async fn run(host: &str, port: u16, default_proxy: Option<String>) -> ChatGptResult<()> {
    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .map_err(|err| ChatGptError::configuration(format!("invalid address: {}", err)))?;

    let state = AppState::new(default_proxy);
    let app = router(state);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    let local_addr = listener.local_addr()?;

    info!("🚀 API server listening on http://{}", local_addr);
    info!("📚 API Endpoints:");
    info!("  Health: GET /health");
    info!("  Models: GET /v1/models");
    info!("  Threads: POST /v1/threads, GET /v1/threads");
    info!("  Thread: GET/DELETE /v1/threads/:thread_id");
    info!("  Messages: POST/GET /v1/threads/:thread_id/messages");
    info!("  Response: POST /v1/responses");

    axum::serve(listener, app).await?;

    Ok(())
}
