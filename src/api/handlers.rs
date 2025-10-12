use axum::{
    extract::State,
    response::{IntoResponse, Response as AxumResponse, Sse, sse::Event},
    Json,
};
use std::convert::Infallible;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{error, info};

use super::error::ApiError;
use super::state::AppState;
use super::types::*;

/// Create a new thread
pub async fn create_thread(
    State(state): State<AppState>,
    Json(payload): Json<CreateThreadRequest>,
) -> std::result::Result<AxumResponse, ApiError> {
    info!("Creating new thread with {} initial messages", payload.messages.len());

    let (thread_id, thread_state) = state
        .create_thread(
            payload.messages,
            payload.metadata,
            payload.proxy.as_deref(),
        )
        .await?;

    let response = Thread {
        id: thread_id,
        object: "thread".to_string(),
        created_at: thread_state.created_at,
        metadata: thread_state.metadata,
    };

    Ok(Json(response).into_response())
}

/// Get a thread by ID
pub async fn get_thread(
    State(state): State<AppState>,
    axum::extract::Path(thread_id): axum::extract::Path<String>,
) -> std::result::Result<AxumResponse, ApiError> {
    let thread_state = state.get_thread(&thread_id).await?;

    let response = Thread {
        id: thread_id,
        object: "thread".to_string(),
        created_at: thread_state.created_at,
        metadata: thread_state.metadata,
    };

    Ok(Json(response).into_response())
}

/// List all threads
pub async fn list_threads(
    State(state): State<AppState>,
) -> std::result::Result<AxumResponse, ApiError> {
    let threads = state.list_threads().await;

    let data: Vec<Thread> = threads
        .into_iter()
        .map(|(id, state)| Thread {
            id,
            object: "thread".to_string(),
            created_at: state.created_at,
            metadata: state.metadata,
        })
        .collect();

    let response = ListThreadsResponse {
        object: "list".to_string(),
        data,
        has_more: false,
    };

    Ok(Json(response).into_response())
}

/// Delete a thread
pub async fn delete_thread(
    State(state): State<AppState>,
    axum::extract::Path(thread_id): axum::extract::Path<String>,
) -> std::result::Result<AxumResponse, ApiError> {
    state.delete_thread(&thread_id).await?;

    Ok(Json(serde_json::json!({
        "id": thread_id,
        "object": "thread.deleted",
        "deleted": true
    }))
    .into_response())
}

/// Add a message to a thread
pub async fn add_message(
    State(state): State<AppState>,
    axum::extract::Path(thread_id): axum::extract::Path<String>,
    Json(payload): Json<AddMessageRequest>,
) -> std::result::Result<AxumResponse, ApiError> {
    if payload.content.trim().is_empty() {
        return Err(ApiError::bad_request("Message content cannot be empty"));
    }

    state
        .add_message_to_thread(&thread_id, payload.role.clone(), payload.content.clone())
        .await?;

    let message_id = uuid::Uuid::new_v4().to_string();
    let created_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let response = Message {
        id: message_id,
        object: "thread.message".to_string(),
        created_at,
        thread_id,
        role: payload.role,
        content: vec![ContentPart {
            content_type: "text".to_string(),
            text: TextContent {
                value: payload.content,
                annotations: vec![],
            },
        }],
    };

    Ok(Json(response).into_response())
}

/// List messages in a thread
pub async fn list_messages(
    State(state): State<AppState>,
    axum::extract::Path(thread_id): axum::extract::Path<String>,
) -> std::result::Result<AxumResponse, ApiError> {
    let thread_state = state.get_thread(&thread_id).await?;

    let data: Vec<Message> = thread_state
        .get_messages()
        .iter()
        .enumerate()
        .map(|(idx, msg)| Message {
            id: format!("msg_{}_{}", thread_id, idx),
            object: "thread.message".to_string(),
            created_at: msg.created_at.unwrap_or(thread_state.created_at),
            thread_id: thread_id.clone(),
            role: msg.role.clone(),
            content: vec![ContentPart {
                content_type: "text".to_string(),
                text: TextContent {
                    value: msg.content.clone(),
                    annotations: vec![],
                },
            }],
        })
        .collect();

    let response = ListMessagesResponse {
        object: "list".to_string(),
        data,
        has_more: false,
    };

    Ok(Json(response).into_response())
}

/// Create a response (run the assistant)
pub async fn create_response(
    State(state): State<AppState>,
    Json(payload): Json<CreateResponseRequest>,
) -> std::result::Result<AxumResponse, ApiError> {
    let thread_id = payload.thread_id.clone();
    
    info!("Creating response for thread: {}, stream: {}", thread_id, payload.stream);

    let thread_state = state.get_thread(&thread_id).await?;

    // Get the last user message
    let last_user_message = thread_state
        .get_messages()
        .iter()
        .rev()
        .find(|m| m.role == "user")
        .ok_or_else(|| ApiError::bad_request("No user message found in thread"))?;

    let message_content = last_user_message.content.clone();
    if message_content.trim().is_empty() {
        return Err(ApiError::bad_request("Last user message content is empty"));
    }

    let is_new = thread_state.is_new();
    let client_arc = thread_state.client.clone();

    if payload.stream {
        handle_stream_response(state, thread_state, client_arc, &message_content, is_new, thread_id).await
    } else {
        handle_non_stream_response(state, thread_state, client_arc, &message_content, is_new, thread_id).await
    }
}

async fn handle_non_stream_response(
    state: AppState,
    mut thread_state: super::state::ThreadState,
    client_arc: std::sync::Arc<tokio::sync::RwLock<crate::client::ChatGptClient>>,
    message: &str,
    is_new: bool,
    thread_id: String,
) -> std::result::Result<AxumResponse, ApiError> {
    let mut client = client_arc.write().await;

    let answer = if is_new {
        client.start_conversation(message).await.map_err(|err| {
            error!("ChatGPT start_conversation failed: {:?}", err);
            ApiError::from(err)
        })?
    } else {
        client.hold_conversation(message, false).await.map_err(|err| {
            error!("ChatGPT hold_conversation failed: {:?}", err);
            ApiError::from(err)
        })?
    };

    drop(client);

    // Add assistant's response to thread
    thread_state.add_message("assistant".to_string(), answer.clone());
    state.update_thread(&thread_id, thread_state).await?;

    let response_id = uuid::Uuid::new_v4().to_string();
    let created_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let response = Response {
        id: response_id,
        object: "thread.response".to_string(),
        created_at,
        thread_id,
        status: "completed".to_string(),
        model: "gpt-4".to_string(),
        usage: Some(Usage {
            prompt_tokens: 0,
            completion_tokens: 0,
            total_tokens: 0,
        }),
    };

    Ok(Json(response).into_response())
}

async fn handle_stream_response(
    state: AppState,
    mut thread_state: super::state::ThreadState,
    client_arc: std::sync::Arc<tokio::sync::RwLock<crate::client::ChatGptClient>>,
    message: &str,
    is_new: bool,
    thread_id: String,
) -> std::result::Result<AxumResponse, ApiError> {
    let mut client = client_arc.write().await;

    let answer = if is_new {
        client.start_conversation(message).await.map_err(|err| {
            error!("ChatGPT start_conversation failed: {:?}", err);
            ApiError::from(err)
        })?
    } else {
        client.hold_conversation(message, false).await.map_err(|err| {
            error!("ChatGPT hold_conversation failed: {:?}", err);
            ApiError::from(err)
        })?
    };

    drop(client);

    // Add assistant's response to thread
    let full_answer = answer.clone();
    thread_state.add_message("assistant".to_string(), full_answer);
    state.update_thread(&thread_id, thread_state).await?;

    // Split response into chunks for streaming
    let chunks: Vec<String> = answer
        .chars()
        .collect::<Vec<char>>()
        .chunks(10)
        .map(|chunk| chunk.iter().collect::<String>())
        .collect();

    let created_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let response_id = uuid::Uuid::new_v4().to_string();

    let (tx, rx) = tokio::sync::mpsc::channel(100);

    // Spawn a task to send chunks
    tokio::spawn(async move {
        // Send content chunks
        for (i, chunk) in chunks.into_iter().enumerate() {
            let chunk_data = ResponseChunk {
                id: response_id.clone(),
                object: "thread.response.chunk".to_string(),
                created_at,
                thread_id: thread_id.clone(),
                delta: Delta {
                    role: if i == 0 { Some("assistant".to_string()) } else { None },
                    content: Some(chunk),
                },
            };

            if tx
                .send(Ok::<_, Infallible>(
                    Event::default().json_data(chunk_data).unwrap(),
                ))
                .await
                .is_err()
            {
                break;
            }
        }

        // Send final chunk
        let final_chunk = ResponseChunk {
            id: response_id.clone(),
            object: "thread.response.chunk".to_string(),
            created_at,
            thread_id: thread_id.clone(),
            delta: Delta {
                role: None,
                content: None,
            },
        };

        let _ = tx
            .send(Ok(Event::default().json_data(final_chunk).unwrap()))
            .await;
    });

    let stream = ReceiverStream::new(rx);
    Ok(Sse::new(stream).into_response())
}
