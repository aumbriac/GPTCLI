use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE, AUTHORIZATION};
use serde::{Serialize, Deserialize};
use indicatif::{ProgressStyle, ProgressBar};
use futures::stream::StreamExt;
use std::{env, error::Error, fs, io::{Read, Write, self}};
use base64;
use colored::*;

const OPENAI_API_URL: &str = "https://api.openai.com/v1/chat/completions";
const DEFAULT_MODEL: &str = "gpt-3.5-turbo";
const GPT4_MODEL: &str = "gpt-4";
const GPT4_VISION_MODEL: &str = "gpt-4-vision-preview";
const DEFAULT_VISION_INSTRUCTIONS: &str = "What's in the image?";

#[derive(Deserialize, Debug)]
struct ChatApiResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct VisionApiResponse {
    choices: Vec<VisionChoice>,
}

#[derive(Deserialize, Debug)]
struct ChatChoice {
    delta: Delta,
}

#[derive(Debug, Deserialize)]
struct VisionChoice {
    message: Message,
}

#[derive(Debug, Deserialize)]
struct Message {
    content: String,
}

#[derive(Deserialize, Debug)]
struct Delta {
    content: Option<String>,
}

#[derive(Serialize)]
struct OpenAiChatRequestBody {
    model: String,
    messages: Vec<ChatMessageRole>,
    stream: bool
}

#[derive(Debug, Serialize)]
struct OpenAiVisionRequestBody {
    model: String,
    messages: Vec<VisionMessageRole>,
    max_tokens: u32,
}

#[derive(Debug, Serialize)]
struct ChatMessageRole {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct VisionMessageRole {
    role: String,
    content: Vec<Content>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum Content {
    #[serde(rename = "text")]
    Text { text: String },

    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrl },
}

#[derive(Debug, Serialize)]
struct ImageUrl {
    url: String,
}

fn build_headers() -> Result<HeaderMap, Box<dyn Error>> {
    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&format!("Bearer {}", env::var("OPENAI_API_KEY")?))?);
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    Ok(headers)
}

async fn make_openai_request(client: &reqwest::Client, model: &str, prompt: &str) -> Result<(), Box<dyn Error>> {
    let headers = build_headers()?;
    let spinner = ProgressBar::new_spinner();
    let color = match model {
        GPT4_MODEL => "cyan",
        _ => "green",
    };
    spinner.set_style(ProgressStyle::default_spinner()
        .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
        .template(&format!("{{spinner:.{}}} {{msg}}", color)));
    spinner.enable_steady_tick(100);
    spinner.set_message("Generating response...");

    let request_body = OpenAiChatRequestBody {
        model: model.to_string(),        
        messages: vec![
            ChatMessageRole { role: "system".to_string(), content: "You are a helpful assistant.".to_string() },
            ChatMessageRole { role: "user".to_string(), content: prompt.to_string() },
        ],
        stream: true
    };

    let response = client.post(OPENAI_API_URL)
        .headers(headers)
        .json(&request_body)
        .send()
        .await?;
        
    spinner.finish_and_clear();
    if response.status().is_success() {
        let mut stream = response.bytes_stream();

        while let Some(item) = stream.next().await {
            let chunk = item?;
            let chunk_str = String::from_utf8(chunk.to_vec())?;
        
            for line in chunk_str.split('\n') {
                let line = line.trim_start_matches("data: ").trim();
                if !line.is_empty() {
                    match serde_json::from_str::<ChatApiResponse>(line) {
                        Ok(api_response) => {
                            for choice in api_response.choices {
                                if let Some(content) = choice.delta.content {
                                    print!("{}", content);
                                    io::stdout().flush().unwrap();
                                }
                            }
                        },
                        Err(_) => {}
                    }
                }
            }
        }                
        
    } else {
        spinner.finish_with_message("Error!");
        return Err("An unexpected error has occurred".into());
    }
    println!();

    Ok(())
}

async fn make_openai_vision_request(client: &reqwest::Client, image_base64: &str, instructions: &str) -> Result<(), Box<dyn Error>> {
    let headers = build_headers()?;
    let request_body = OpenAiVisionRequestBody {
        model: GPT4_VISION_MODEL.to_string(),
        messages: vec![
            VisionMessageRole {
                role: "user".to_string(),
                content: vec![
                    Content::Text {
                        text: instructions.to_string(),
                    },
                    Content::ImageUrl {
                        image_url: ImageUrl {
                            url: format!("data:image/jpeg;base64,{}", image_base64),
                        },
                    },
                ],
            },
        ],
        max_tokens: 300,
    };

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(ProgressStyle::default_spinner()
        .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
        .template("{spinner:.magenta} {msg}"));
    spinner.enable_steady_tick(100);
    spinner.set_message("Analyzing image...");

    let response = client.post(OPENAI_API_URL)
        .headers(headers)
        .json(&request_body)
        .send()
        .await;

    spinner.finish_and_clear();

    match response {
        Ok(res) => {
            if res.status().is_success() {
                spinner.finish_with_message("Done!");
                let api_response = res.json::<VisionApiResponse>().await?;
                println!("{}", api_response.choices.get(0).map_or("No content in response", |c| &c.message.content));
            } else {
                eprintln!("Failed with status code: {}", res.status());
                if let Ok(error_message) = res.text().await {
                    eprintln!("Response error message: {}", error_message);
                }
                spinner.finish_with_message("Failed to get a valid response");
                return Err("Failed to get a valid response".into());
            }
        },
        Err(e) => {
            eprintln!("Request error: {}", e);
            spinner.finish_with_message("Error in sending request");
            return Err("An error occurred while sending the request".into());
        }
    }    

    Ok(())
}

async fn encode_image(image_path: &str) -> Result<String, Box<dyn Error>> {
    let mut file = fs::File::open(image_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(base64::encode(buffer))
}

fn print_help() {
    println!("{:━^60}", " gpt CLI Tool ".yellow());
    println!("Usage:");
    println!("  {} [option] <argument>", "gpt".bold().green());
    println!("\nOptions:");
    println!("  {}   Use the GPT-4 model for text prompts.", "4".bold().cyan());
    println!("  {}   Use the GPT-4 Vision model for image analysis.", "v".bold().magenta());
    println!("  {}     Display this help message.", "-help".bold().blue());
    println!("\nArguments:");
    println!("  {}  A text prompt for the GPT-4 model.", "<prompt>".bold().cyan());
    println!("  {}  A path to an image file for the GPT-4 Vision model.", "<image_path>".bold().magenta());
    println!("\nExamples:");
    println!("  {} What is the capital of California?", "gpt".bold().green());
    println!("  {} What is the meaning of life?", "gpt 4".bold().cyan());
    println!("  {} ./path/to/image.jpg Describe this image", "gpt v".bold().magenta());
    println!("{:━^60}", "".yellow());
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    env_logger::init();

    let args: Vec<String> = env::args().collect();

    if args.contains(&"-help".to_string()) {
        print_help();
        return Ok(());
    }

    if args.len() < 3 {
        eprintln!("Usage: gpt [4|v] <prompt|image_path>");
        eprintln!("For more information, try 'gpt -help'");
        return Err("Insufficient arguments".into());
    }
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    match args[1].as_str() {
        "4" => {
            let prompt = args[2..].join(" ");
            make_openai_request(&client, GPT4_MODEL, &prompt).await?
        },
        "v" => {
            let instructions = if args.len() > 3 {
                args[2..].join(" ")
            } else {
                DEFAULT_VISION_INSTRUCTIONS.to_string()
            };
            let image_base64 = encode_image(&args[2]).await?;
            make_openai_vision_request(&client, &image_base64, &instructions).await?
        },
        _ => {
            let prompt = args[1..].join(" ");
            make_openai_request(&client, DEFAULT_MODEL, &prompt).await?
        }
    }

    Ok(())
}
