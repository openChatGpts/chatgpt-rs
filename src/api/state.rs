use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};

use crate::client::ChatGptClient;
use super::error::ApiError;
use super::types::ThreadMessage;

/// Thread state - manages conversation context
#[derive(Clone)]
pub struct ThreadState {
    pub client: Arc<RwLock<ChatGptClient>>,
    pub messages: Vec<ThreadMessage>,
    pub created_at: u64,
    pub metadata: Option<serde_json::Value>,
}

impl ThreadState {
    pub fn new(
        client: Arc<RwLock<ChatGptClient>>,
        metadata: Option<serde_json::Value>,
    ) -> Self {
        Self {
            client,
            messages: Vec::new(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metadata,
        }
    }

    pub fn add_message(&mut self, role: String, content: String) {
        let message = ThreadMessage {
            role,
            content,
            created_at: Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            ),
        };
        self.messages.push(message);
    }

    pub fn is_new(&self) -> bool {
        // 检查是否有 assistant 的回复
        !self.messages.iter().any(|m| m.role == "assistant")
    }

    pub fn get_messages(&self) -> &[ThreadMessage] {
        &self.messages
    }
}

/// App state for managing threads (conversations)
#[derive(Clone)]
pub struct AppState {
    threads: Arc<RwLock<HashMap<String, ThreadState>>>,
    default_proxy: Option<String>,
}

impl AppState {
    pub fn new(default_proxy: Option<String>) -> Self {
        Self {
            threads: Arc::new(RwLock::new(HashMap::new())),
            default_proxy,
        }
    }

    /// Create a new thread
    pub async fn create_thread(
        &self,
        initial_messages: Vec<ThreadMessage>,
        metadata: Option<serde_json::Value>,
        proxy: Option<&str>,
    ) -> Result<(String, ThreadState), ApiError> {
        // Use request-specific proxy if provided, otherwise use default
        let proxy_to_use = proxy.or(self.default_proxy.as_deref());

        let client = ChatGptClient::new(proxy_to_use).await.map_err(|err| {
            error!("Failed to create ChatGPT client: {}", err);
            ApiError::from(err)
        })?;

        let client_arc = Arc::new(RwLock::new(client));
        let thread_id = uuid::Uuid::new_v4().to_string();

        let mut state = ThreadState::new(client_arc, metadata);
        
        // Add initial messages
        for msg in initial_messages {
            state.add_message(msg.role, msg.content);
        }

        let mut threads = self.threads.write().await;
        threads.insert(thread_id.clone(), state.clone());

        info!("Created new thread: {}", thread_id);
        Ok((thread_id, state))
    }

    /// Get an existing thread
    pub async fn get_thread(&self, thread_id: &str) -> Result<ThreadState, ApiError> {
        let threads = self.threads.read().await;
        threads
            .get(thread_id)
            .cloned()
            .ok_or_else(|| ApiError::not_found(format!("Thread {} not found", thread_id)))
    }

    /// Update thread state
    pub async fn update_thread(&self, thread_id: &str, state: ThreadState) -> Result<(), ApiError> {
        let mut threads = self.threads.write().await;
        if threads.contains_key(thread_id) {
            threads.insert(thread_id.to_string(), state);
            Ok(())
        } else {
            Err(ApiError::not_found(format!("Thread {} not found", thread_id)))
        }
    }

    /// Add a message to a thread
    pub async fn add_message_to_thread(
        &self,
        thread_id: &str,
        role: String,
        content: String,
    ) -> Result<(), ApiError> {
        let mut threads = self.threads.write().await;
        let thread = threads
            .get_mut(thread_id)
            .ok_or_else(|| ApiError::not_found(format!("Thread {} not found", thread_id)))?;

        thread.add_message(role, content);
        Ok(())
    }

    /// List all threads
    pub async fn list_threads(&self) -> Vec<(String, ThreadState)> {
        let threads = self.threads.read().await;
        threads
            .iter()
            .map(|(id, state)| (id.clone(), state.clone()))
            .collect()
    }

    /// Delete a thread
    pub async fn delete_thread(&self, thread_id: &str) -> Result<(), ApiError> {
        let mut threads = self.threads.write().await;
        threads
            .remove(thread_id)
            .ok_or_else(|| ApiError::not_found(format!("Thread {} not found", thread_id)))?;
        info!("Deleted thread: {}", thread_id);
        Ok(())
    }

    /// Get the default proxy setting
    pub fn get_default_proxy(&self) -> Option<&str> {
        self.default_proxy.as_deref()
    }
}
