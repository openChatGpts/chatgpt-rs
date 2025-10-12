use serde::{Deserialize, Serialize};

// Request types for Responses API
#[derive(Debug, Deserialize)]
pub struct CreateResponseRequest {
    /// The thread ID to create a response for
    pub thread_id: String,
    /// The model to use
    #[serde(default = "default_model")]
    pub model: String,
    /// Optional instructions for the assistant
    #[serde(default)]
    pub instructions: Option<String>,
    /// Whether to stream the response
    #[serde(default)]
    pub stream: bool,
    /// Optional proxy configuration
    #[serde(default)]
    pub proxy: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateThreadRequest {
    /// Initial messages for the thread
    #[serde(default)]
    pub messages: Vec<ThreadMessage>,
    /// Optional metadata
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
    /// Optional proxy configuration
    #[serde(default)]
    pub proxy: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AddMessageRequest {
    /// The role of the message sender
    pub role: String,
    /// The content of the message
    pub content: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ThreadMessage {
    /// The role of the message sender (user or assistant)
    pub role: String,
    /// The content of the message
    pub content: String,
    /// When the message was created
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<u64>,
}

// Response types
#[derive(Debug, Serialize)]
pub struct Thread {
    pub id: String,
    pub object: String,
    pub created_at: u64,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct Message {
    pub id: String,
    pub object: String,
    pub created_at: u64,
    pub thread_id: String,
    pub role: String,
    pub content: Vec<ContentPart>,
}

#[derive(Debug, Serialize)]
pub struct ContentPart {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: TextContent,
}

#[derive(Debug, Serialize)]
pub struct TextContent {
    pub value: String,
    pub annotations: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct Response {
    pub id: String,
    pub object: String,
    pub created_at: u64,
    pub thread_id: String,
    pub status: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

#[derive(Debug, Serialize)]
pub struct ResponseChunk {
    pub id: String,
    pub object: String,
    pub created_at: u64,
    pub thread_id: String,
    pub delta: Delta,
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
pub struct ErrorResponse {
    pub status: &'static str,
    pub detail: String,
}

#[derive(Debug, Serialize)]
pub struct ListThreadsResponse {
    pub object: String,
    pub data: Vec<Thread>,
    pub has_more: bool,
}

#[derive(Debug, Serialize)]
pub struct ListMessagesResponse {
    pub object: String,
    pub data: Vec<Message>,
    pub has_more: bool,
}

fn default_model() -> String {
    "gpt-4".to_string()
}
