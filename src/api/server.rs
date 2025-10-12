use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    http::{Method, StatusCode, header::CONTENT_TYPE},
    response::{IntoResponse, Response, Sse, sse::Event},
    routing::post,
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tokio_stream::wrappers::ReceiverStream;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};

use crate::{
    client::ChatGptClient,
    utils::{ChatGptError, Result as ChatGptResult},
};

// OpenAI compatible structures
#[derive(Debug, Deserialize)]
pub struct ChatCompletionRequest {
    #[serde(default)]
    pub model: Option<String>,
    pub messages: Vec<ChatMessage>,
    #[serde(default)]
    pub stream: bool,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub max_tokens: Option<u32>,
    #[serde(default)]
    pub conversation_id: Option<String>,
    #[serde(default)]
    pub proxy: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChatChoice>,
    pub usage: Usage,
}

#[derive(Debug, Serialize)]
pub struct ChatChoice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: String,
}

#[derive(Debug, Serialize)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChatChunkChoice>,
}

#[derive(Debug, Serialize)]
pub struct ChatChunkChoice {
    pub index: u32,
    pub delta: Delta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Delta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
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

// App state for managing ChatGPT clients
#[derive(Clone)]
pub struct AppState {
    clients: Arc<RwLock<HashMap<String, Arc<RwLock<ChatGptClient>>>>>,
    default_proxy: Option<String>,
}

impl AppState {
    pub fn new(default_proxy: Option<String>) -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            default_proxy,
        }
    }

    async fn get_or_create_client(
        &self,
        conversation_id: Option<&str>,
        proxy: Option<&str>,
    ) -> Result<(String, Arc<RwLock<ChatGptClient>>), ApiError> {
        let proxy_to_use = proxy.or(self.default_proxy.as_deref());
        
        // Generate a unique key for this client
        let client_key = conversation_id
            .map(|id| id.to_string())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        let mut clients = self.clients.write().await;

        if let Some(client) = clients.get(&client_key) {
            // Return existing client
            Ok((client_key.clone(), Arc::clone(client)))
        } else {
            // Create a new client
            let client = ChatGptClient::new(proxy_to_use).await.map_err(|err| {
                error!("Failed to create ChatGPT client: {}", err);
                ApiError::from(err)
            })?;

            let client_arc = Arc::new(RwLock::new(client));
            clients.insert(client_key.clone(), Arc::clone(&client_arc));
            Ok((client_key, client_arc))
        }
    }
}

pub fn router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::POST])
        .allow_headers([CONTENT_TYPE]);

    Router::new()
        .route("/v1/chat/completions", post(chat_completions))
        .with_state(state)
        .layer(cors)
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

    info!("API server listening on http://{}", local_addr);

    axum::serve(listener, app).await?;

    Ok(())
}

/// OpenAI-compatible chat completions endpoint
async fn chat_completions(
    State(state): State<AppState>,
    Json(payload): Json<ChatCompletionRequest>,
) -> std::result::Result<Response, ApiError> {
    let ChatCompletionRequest {
        messages,
        stream,
        conversation_id,
        proxy,
        ..
    } = payload;

    if messages.is_empty() {
        return Err(ApiError::bad_request("Messages cannot be empty"));
    }

    // Get the last user message
    let last_message = messages
        .iter()
        .rev()
        .find(|m| m.role == "user")
        .ok_or_else(|| ApiError::bad_request("No user message found"))?;

    let message_content = last_message.content.trim();
    if message_content.is_empty() {
        return Err(ApiError::bad_request("Message content cannot be empty"));
    }

    info!(
        "Handling chat completion request (stream: {}, conversation_id: {:?})",
        stream, conversation_id
    );

    // Get or create client
    let (conv_id, client_arc) = state
        .get_or_create_client(conversation_id.as_deref(), proxy.as_deref())
        .await?;

    let is_new_conversation = conversation_id.is_none() || messages.len() <= 1;

    if stream {
        // Stream response
        handle_stream_response(client_arc, message_content, is_new_conversation, conv_id).await
    } else {
        // Non-stream response
        handle_non_stream_response(client_arc, message_content, is_new_conversation, conv_id).await
    }
}

async fn handle_non_stream_response(
    client_arc: Arc<RwLock<ChatGptClient>>,
    message: &str,
    is_new: bool,
    conversation_id: String,
) -> std::result::Result<Response, ApiError> {
    let mut client = client_arc.write().await;

    let answer = if is_new {
        client.start_conversation(message).await.map_err(|err| {
            error!("ChatGPT conversation failed: {}", err);
            ApiError::from(err)
        })?
    } else {
        client.hold_conversation(message, false).await.map_err(|err| {
            error!("ChatGPT hold_conversation failed: {}", err);
            ApiError::from(err)
        })?
    };

    let response = ChatCompletionResponse {
        id: conversation_id,
        object: "chat.completion".to_string(),
        created: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        model: "gpt-4".to_string(),
        choices: vec![ChatChoice {
            index: 0,
            message: ChatMessage {
                role: "assistant".to_string(),
                content: answer,
            },
            finish_reason: "stop".to_string(),
        }],
        usage: Usage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        },
    };

    Ok(Json(response).into_response())
}

async fn handle_stream_response(
    client_arc: Arc<RwLock<ChatGptClient>>,
    message: &str,
    is_new: bool,
    conversation_id: String,
) -> std::result::Result<Response, ApiError> {
    let mut client = client_arc.write().await;

    let answer = if is_new {
        client.start_conversation(message).await.map_err(|err| {
            error!("ChatGPT conversation failed: {}", err);
            ApiError::from(err)
        })?
    } else {
        client.hold_conversation(message, false).await.map_err(|err| {
            error!("ChatGPT hold_conversation failed: {}", err);
            ApiError::from(err)
        })?
    };

    // Split response into chunks for streaming
    let chunks: Vec<String> = answer
        .chars()
        .collect::<Vec<char>>()
        .chunks(10)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect();

    let created = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let (tx, rx) = tokio::sync::mpsc::channel(100);

    // Spawn a task to send chunks
    tokio::spawn(async move {
        // Send content chunks
        for (i, chunk) in chunks.into_iter().enumerate() {
            let chunk_data = ChatCompletionChunk {
                id: conversation_id.clone(),
                object: "chat.completion.chunk".to_string(),
                created,
                model: "gpt-4".to_string(),
                choices: vec![ChatChunkChoice {
                    index: 0,
                    delta: Delta {
                        role: if i == 0 { Some("assistant".to_string()) } else { None },
                        content: Some(chunk),
                    },
                    finish_reason: None,
                }],
            };

            if tx.send(Ok::<_, Infallible>(Event::default().json_data(chunk_data).unwrap())).await.is_err() {
                break;
            }
        }

        // Send final chunk with finish_reason
        let final_chunk = ChatCompletionChunk {
            id: conversation_id.clone(),
            object: "chat.completion.chunk".to_string(),
            created,
            model: "gpt-4".to_string(),
            choices: vec![ChatChunkChoice {
                index: 0,
                delta: Delta {
                    role: None,
                    content: None,
                },
                finish_reason: Some("stop".to_string()),
            }],
        };

        let _ = tx.send(Ok(Event::default().json_data(final_chunk).unwrap())).await;
    });

    let stream = ReceiverStream::new(rx);
    Ok(Sse::new(stream).into_response())
}
