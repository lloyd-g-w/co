//! Model providers. Each provider turns a conversation into a stream of
//! [`StreamEvent`]s over HTTPS (SSE).

use futures::stream::BoxStream;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: serde_json::Value,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
}

/// A piece of a streamed model response.
#[derive(Debug, Clone)]
pub enum StreamEvent {
    TextDelta(String),
    ToolCall {
        id: String,
        name: String,
        input: serde_json::Value,
    },
    Done,
}

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("http: {0}")]
    Http(#[from] reqwest::Error),
    #[error("{0}")]
    Api(String),
}

pub trait Provider: Send + Sync {
    fn name(&self) -> &str;

    fn stream(
        &self,
        model: &str,
        messages: &[Message],
    ) -> BoxStream<'static, Result<StreamEvent, ProviderError>>;
}
