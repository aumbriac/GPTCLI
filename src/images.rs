use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct OpenAiDalleRequestBody {
    pub model: String,
    pub prompt: String,
    pub n: u8,
    pub size: String,
    pub quality: String,
}

#[derive(Debug, Deserialize)]
pub struct DalleImageGeneration {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct DalleApiResponse {
    pub data: Vec<DalleImageGeneration>,
}
