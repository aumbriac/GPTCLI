use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub struct ChatApiResponse {
    pub choices: Vec<ChatChoice>,
}

#[derive(Deserialize, Debug)]
pub struct ChatChoice {
    pub delta: ChatDelta,
}

#[derive(Deserialize, Debug)]
pub struct ChatDelta {
    pub content: Option<String>,
}

#[derive(Serialize)]
pub struct OpenAiChatRequestBody {
    pub model: String,
    pub messages: Vec<ChatMessageRole>,
    pub stream: bool,
}

#[derive(Debug, Serialize)]
pub struct ChatMessageRole {
    pub role: String,
    pub content: String,
}
