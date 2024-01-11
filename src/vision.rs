use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct VisionMessage {
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct VisionChoice {
    pub message: VisionMessage,
}

#[derive(Debug, Deserialize)]
pub struct VisionApiResponse {
    pub choices: Vec<VisionChoice>,
}

#[derive(Debug, Serialize)]
pub struct ImageUrl {
    pub url: String,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum VisionContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrl },
}

#[derive(Debug, Serialize)]
pub struct VisionMessageRole {
    pub role: String,
    pub content: Vec<VisionContent>,
}

#[derive(Debug, Serialize)]
pub struct OpenAiVisionRequestBody {
    pub model: String,
    pub messages: Vec<VisionMessageRole>,
    pub max_tokens: u32,
}
