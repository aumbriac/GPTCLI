use crate::chat::OpenAiChatRequestBody;
use crate::images::OpenAiDalleRequestBody;
use crate::vision::OpenAiVisionRequestBody;
use serde::Serialize;

pub const GPT_API_URL: &str = "https://api.openai.com/v1/chat/completions";
pub const DEFAULT_MODEL: &str = "gpt-3.5-turbo";
pub const GPT4_MODEL: &str = "gpt-4";
pub const GPT4_VISION_MODEL: &str = "gpt-4-vision-preview";
pub const DEFAULT_VISION_INSTRUCTIONS: &str = "What's in the image?";
pub const DALLE_API_URL: &str = "https://api.openai.com/v1/images/generations";
pub const DALLE_MODEL: &str = "dall-e-3";
pub const CMD_VISION: &str = "v";
pub const CMD_GPT4: &str = "4";
pub const CMD_DALLE: &str = "d";

#[derive(Serialize)]
pub enum RequestType {
    Chat(OpenAiChatRequestBody),
    Vision(OpenAiVisionRequestBody),
    Dalle(OpenAiDalleRequestBody),
}
