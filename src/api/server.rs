use std::net::SocketAddr;

use axum::{
    Json, Router,
    http::{Method, StatusCode, header::CONTENT_TYPE},
    response::{IntoResponse, Response},
    routing::post,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};

use crate::{
    client::ChatGptClient,
    utils::{ChatGptError, Result as ChatGptResult},
};

#[derive(Debug, Deserialize)]
pub struct ConversationRequest {
    pub proxy: String,
    pub message: String,
    #[serde(default)]
    pub image: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ConversationResponse {
    pub status: &'static str,
    pub result: String,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    status: &'static str,
    detail: String,
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }
}

impl From<ChatGptError> for ApiError {
    fn from(err: ChatGptError) -> Self {
        let status = match err {
            ChatGptError::InvalidProxy(_) => StatusCode::BAD_REQUEST,
            ChatGptError::Authentication(_) => StatusCode::UNAUTHORIZED,
            ChatGptError::IpFlagged => StatusCode::FORBIDDEN,
            ChatGptError::Network(_) => StatusCode::BAD_GATEWAY,
            ChatGptError::Configuration(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ChatGptError::ChallengeSolve(_)
            | ChatGptError::VmExecution(_)
            | ChatGptError::InvalidResponse(_)
            | ChatGptError::Unknown(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ChatGptError::Json(_)
            | ChatGptError::Base64Decode(_)
            | ChatGptError::Io(_)
            | ChatGptError::Image(_) => StatusCode::UNPROCESSABLE_ENTITY,
        };

        Self::new(status, err.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let payload = Json(ErrorResponse {
            status: "error",
            detail: self.message,
        });

        (self.status, payload).into_response()
    }
}

pub fn router() -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::POST])
        .allow_headers([CONTENT_TYPE]);

    Router::new()
        .route("/conversation", post(create_conversation))
        .layer(cors)
}

/// Run the API server with the provided host and port.
pub async fn run(host: &str, port: u16) -> ChatGptResult<()> {
    let addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .map_err(|err| ChatGptError::configuration(format!("invalid address: {}", err)))?;

    let app = router();

    let listener = tokio::net::TcpListener::bind(addr).await?;
    let local_addr = listener.local_addr()?;

    info!("API server listening on http://{}", local_addr);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn create_conversation(
    Json(payload): Json<ConversationRequest>,
) -> std::result::Result<Json<ConversationResponse>, ApiError> {
    let ConversationRequest {
        proxy,
        message,
        image,
    } = payload;

    let proxy_trimmed = proxy.trim();
    let message_trimmed = message.trim();

    if proxy_trimmed.is_empty() || message_trimmed.is_empty() {
        return Err(ApiError::bad_request("Proxy and message are required"));
    }

    info!("Handling new conversation request");

    let mut client = ChatGptClient::new(Some(proxy_trimmed))
        .await
        .map_err(|err| {
            error!("Failed to create ChatGPT client: {}", err);
            ApiError::from(err)
        })?;

    let answer = match image {
        Some(image_data) => {
            let trimmed = image_data.trim();
            if trimmed.is_empty() {
                client.ask_question(message_trimmed).await.map_err(|err| {
                    error!("ChatGPT text conversation failed: {}", err);
                    ApiError::from(err)
                })?
            } else {
                client
                    .ask_question_with_image(message_trimmed, trimmed)
                    .await
                    .map_err(|err| {
                        error!("ChatGPT image conversation failed: {}", err);
                        ApiError::from(err)
                    })?
            }
        }
        None => client.ask_question(message_trimmed).await.map_err(|err| {
            error!("ChatGPT conversation failed: {}", err);
            ApiError::from(err)
        })?,
    };

    Ok(Json(ConversationResponse {
        status: "success",
        result: answer,
    }))
}
