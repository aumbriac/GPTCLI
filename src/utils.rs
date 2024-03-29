use crate::chat::{ChatApiResponse, ChatMessageRole, OpenAiChatRequestBody};
use crate::constants::{
    RequestType, CMD_DALLE, CMD_GPT4, CMD_VISION, DALLE_API_URL, DALLE_MODEL, DEFAULT_MODEL,
    DEFAULT_VISION_INSTRUCTIONS, GPT4_MODEL, GPT4_VISION_MODEL, GPT_API_URL,
};
use crate::images::{DalleApiResponse, OpenAiDalleRequestBody};
use crate::vision::{
    ImageUrl, OpenAiVisionRequestBody, VisionApiResponse, VisionContent, VisionMessageRole,
};
use base64;
use futures::stream::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE},
    Client,
};
use std::{
    env,
    error::Error,
    fs,
    io::{self, Read, Write},
};

pub fn build_headers() -> Result<HeaderMap, Box<dyn Error>> {
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!(
            "Bearer {}",
            env::var("OPENAI_API_KEY")
                .map_err(|_| "OPENAI_API_KEY environment variable not found or invalid")?
        ))?,
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    Ok(headers)
}

pub fn create_spinner(color: &str, message: String) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template(&format!("{{spinner:.{}}} {{msg}}", color)),
    );
    spinner.enable_steady_tick(100);
    spinner.set_message(message);

    spinner
}

pub async fn encode_image(image_path: &str) -> Result<String, Box<dyn Error>> {
    let mut file = fs::File::open(image_path)
        .map_err(|_| format!("Failed to open image file: {}", image_path))?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer).map_err(|_| {
        Box::new(io::Error::new(
            io::ErrorKind::Other,
            "Failed to read image file",
        ))
    })?;
    Ok(base64::encode(buffer))
}

pub async fn build_vision_request(
    args: &[String],
) -> Result<OpenAiVisionRequestBody, Box<dyn Error>> {
    let instructions = if args.len() > 3 {
        args[3..].join(" ")
    } else {
        DEFAULT_VISION_INSTRUCTIONS.to_string()
    };
    let image_base64 = encode_image(&args[2])
        .await
        .map_err(|e| format!("Failed to encode image: {}", e))?;
    Ok(OpenAiVisionRequestBody {
        model: GPT4_VISION_MODEL.to_string(),
        messages: vec![VisionMessageRole {
            role: "user".to_string(),
            content: vec![
                VisionContent::Text { text: instructions },
                VisionContent::ImageUrl {
                    image_url: ImageUrl {
                        url: format!("data:image/jpeg;base64,{}", image_base64),
                    },
                },
            ],
        }],
        max_tokens: 300,
    })
}

pub fn build_chat_request(args: &[String], model: &str) -> OpenAiChatRequestBody {
    OpenAiChatRequestBody {
        model: model.to_string(),
        messages: vec![
            ChatMessageRole {
                role: "system".to_string(),
                content: "You are a helpful assistant.".to_string(),
            },
            ChatMessageRole {
                role: "user".to_string(),
                content: args[2..].join(" "),
            },
        ],
        stream: true,
    }
}

pub fn build_dalle_request(args: &[String]) -> OpenAiDalleRequestBody {
    OpenAiDalleRequestBody {
        model: DALLE_MODEL.to_string(),
        prompt: args[2..].join(" "),
        n: 1,
        size: "1792x1024".to_string(),
        quality: "hd".to_string(),
    }
}

pub async fn process_vision_response(response: reqwest::Response) -> Result<(), Box<dyn Error>> {
    let api_response = response.json::<VisionApiResponse>().await?;
    println!(
        "{}",
        api_response
            .choices
            .get(0)
            .map_or("No content in response", |c| &c.message.content)
    );
    Ok(())
}

pub async fn process_chat_response(response: reqwest::Response) -> Result<(), Box<dyn Error>> {
    let mut stream = response.bytes_stream();
    let mut buffer = Vec::with_capacity(1024);

    while let Some(item) = stream.next().await {
        let chunk = item?;
        buffer.extend(chunk);

        let mut start = 0;
        while let Some(end) = buffer[start..].iter().position(|&b| b == b'\n') {
            let line = &buffer[start..start + end];
            start += end + 1;

            let line_str = std::str::from_utf8(line).map_err(|_| "Invalid UTF-8 in response")?;
            let line = line_str.trim_start_matches("data: ").trim();
            if !line.is_empty() {
                match serde_json::from_str::<ChatApiResponse>(line) {
                    Ok(api_response) => {
                        for choice in api_response.choices {
                            if let Some(content) = choice.delta.content {
                                print!("{}", content);
                                io::stdout().flush().map_err(|e| {
                                    io::Error::new(
                                        io::ErrorKind::Other,
                                        format!("Failed to flush stdout: {}", e),
                                    )
                                })?;
                            }
                        }
                    }
                    Err(_) => {}
                }
            }
        }
        buffer.drain(0..start);
    }
    println!();
    Ok(())
}

pub async fn process_dalle_response(response: reqwest::Response) -> Result<(), Box<dyn Error>> {
    let response_body = response.text().await?;
    let api_response: DalleApiResponse = serde_json::from_str(&response_body)?;
    for image_gen in api_response.data.iter() {
        println!("Generated image URL: {:#?}", image_gen.url);
    }
    Ok(())
}

pub async fn create_request_type_and_url(
    args: &[String],
) -> Result<(RequestType, &str), Box<dyn Error>> {
    let request_type = match args[1].as_str() {
        CMD_GPT4 => RequestType::Chat(build_chat_request(args, GPT4_MODEL)),
        CMD_VISION => {
            let vision_request = build_vision_request(args).await?;
            RequestType::Vision(vision_request)
        }
        CMD_DALLE => RequestType::Dalle(build_dalle_request(args)),
        _ => RequestType::Chat(build_chat_request(args, DEFAULT_MODEL)),
    };

    let api_url = match &request_type {
        RequestType::Chat(_) | RequestType::Vision(_) => GPT_API_URL,
        RequestType::Dalle(_) => DALLE_API_URL,
    };

    Ok((request_type, api_url))
}

pub async fn make_openai_request(
    client: &Client,
    request_type: RequestType,
    api_url: &str,
) -> Result<(), Box<dyn Error>> {
    let headers = build_headers()?;
    let spinner_color = match &request_type {
        RequestType::Chat(_) => "green",
        RequestType::Vision(_) => "magenta",
        RequestType::Dalle(_) => "red",
    };
    let spinner = create_spinner(spinner_color, "Processing request...".to_string());

    let request_body = match &request_type {
        RequestType::Chat(body) => serde_json::to_value(body)?,
        RequestType::Vision(body) => serde_json::to_value(body)?,
        RequestType::Dalle(body) => serde_json::to_value(body)?,
    };

    let response = client
        .post(api_url)
        .headers(headers)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Failed to send request to OpenAI: {}", e))?;

    spinner.finish_and_clear();

    if response.status().is_success() {
        match &request_type {
            RequestType::Chat(_) => process_chat_response(response).await?,
            RequestType::Vision(_) => process_vision_response(response).await?,
            RequestType::Dalle(_) => process_dalle_response(response).await?,
        }
    } else {
        eprintln!("Failed with status code: {}", response.status());
        if let Ok(error_message) = response.text().await {
            eprintln!("Response error message: {}", error_message);
        }
        return Err("Failed to get a valid response".into());
    }

    Ok(())
}

pub async fn process_command(
    client: &reqwest::Client,
    args: &[String],
) -> Result<(), Box<dyn Error>> {
    let (request_type, api_url) = create_request_type_and_url(args)
        .await
        .map_err(|e| format!("Failed to create request: {}", e))?;

    make_openai_request(client, request_type, api_url).await
}
