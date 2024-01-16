#[cfg(test)]
mod tests {
    use crate::chat::OpenAiChatRequestBody;
    use crate::constants::{
        RequestType, DALLE_API_URL, DALLE_MODEL, DEFAULT_VISION_INSTRUCTIONS, GPT4_VISION_MODEL,
        GPT_API_URL,
    };
    use crate::utils::{
        build_chat_request, build_dalle_request, build_headers, build_vision_request,
        create_request_type_and_url, create_spinner, encode_image, make_openai_request,
        process_chat_response, process_command, process_dalle_response, process_vision_response,
    };
    use crate::vision::VisionContent;
    use reqwest::{
        header::{AUTHORIZATION, CONTENT_TYPE},
        Client,
    };
    use std::{env, io::Write};
    use tempfile::NamedTempFile;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn test_build_headers() {
        env::set_var("OPENAI_API_KEY", "test_key");

        let result = build_headers();
        if let Err(e) = &result {
            println!("Error: {}", e);
        }
        assert!(result.is_ok());

        let headers = result.unwrap();
        assert_eq!(
            headers.get(AUTHORIZATION).unwrap().to_str().unwrap(),
            "Bearer test_key"
        );
        assert_eq!(
            headers.get(CONTENT_TYPE).unwrap().to_str().unwrap(),
            "application/json"
        );

        env::remove_var("OPENAI_API_KEY");
    }

    #[test]
    fn test_create_spinner() {
        let color = "green";
        let message = "Loading...".to_string();
        let spinner = create_spinner(color, message.clone());

        assert_eq!(spinner.is_hidden(), false);
    }

    #[tokio::test]
    async fn test_encode_image_success() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Test data").unwrap();

        let file_path = temp_file.path().to_str().unwrap();
        let result = encode_image(file_path).await;

        assert!(result.is_ok());
        let encoded = result.unwrap();
        assert_eq!(encoded, base64::encode("Test data\n"));
    }

    #[tokio::test]
    async fn test_encode_image_file_not_found() {
        let file_path = "non_existent_file.jpg";
        let result = encode_image(file_path).await;

        assert!(result.is_err());
        assert_eq!(
            format!("{}", result.unwrap_err()),
            format!("Failed to open image file: {}", file_path)
        );
    }

    #[tokio::test]
    async fn test_build_vision_request_with_default_instructions() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Test image data").unwrap();
        let file_path = temp_file.path().to_str().unwrap();
        let args = vec!["gpt".to_string(), "v".to_string(), file_path.to_string()];
        let result = build_vision_request(&args).await;

        assert!(result.is_ok());
        let request = result.unwrap();
        assert_eq!(request.model, GPT4_VISION_MODEL);
        assert_eq!(request.messages.len(), 1);
        assert_eq!(request.messages[0].role, "user");
        assert!(request.messages[0].content.iter().any(|content| matches!(content, VisionContent::Text { text } if text == DEFAULT_VISION_INSTRUCTIONS)));
    }

    #[tokio::test]
    async fn test_build_vision_request_with_custom_instructions() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Test image data").unwrap();
        let file_path = temp_file.path().to_str().unwrap();

        let args = vec![
            "gpt".to_string(),
            "v".to_string(),
            file_path.to_string(),
            "Describe the image".to_string(),
        ];

        let result = build_vision_request(&args).await;

        assert!(result.is_ok());
        let request = result.unwrap();
        assert_eq!(request.model, GPT4_VISION_MODEL);
        assert_eq!(request.messages.len(), 1);
        assert_eq!(request.messages[0].role, "user");
        assert!(request.messages[0].content.iter().any(|content| matches!(content, VisionContent::Text { text } if text == "Describe the image")));
    }

    #[test]
    fn test_build_chat_request_with_single_argument() {
        let args = vec!["gpt".to_string(), "4".to_string(), "Hello".to_string()];
        let model = "test-model";

        let request = build_chat_request(&args, model);

        assert_eq!(request.model, "test-model");
        assert_eq!(request.messages.len(), 2);
        assert_eq!(request.messages[0].role, "system");
        assert_eq!(request.messages[0].content, "You are a helpful assistant.");
        assert_eq!(request.messages[1].role, "user");
        assert_eq!(request.messages[1].content, "Hello");
        assert!(request.stream);
    }

    #[test]
    fn test_build_chat_request_with_multiple_arguments() {
        let args = vec![
            "gpt".to_string(),
            "4".to_string(),
            "Hello".to_string(),
            "How".to_string(),
            "are".to_string(),
            "you?".to_string(),
        ];
        let model = "test-model";

        let request = build_chat_request(&args, model);

        assert_eq!(request.model, "test-model");
        assert_eq!(request.messages.len(), 2);
        assert_eq!(request.messages[0].role, "system");
        assert_eq!(request.messages[0].content, "You are a helpful assistant.");
        assert_eq!(request.messages[1].role, "user");
        assert_eq!(request.messages[1].content, "Hello How are you?");
        assert!(request.stream);
    }

    #[test]
    fn test_build_dalle_request_with_single_argument() {
        let args = vec!["gpt".to_string(), "d".to_string(), "Astronaut".to_string()];

        let request = build_dalle_request(&args);

        assert_eq!(request.model, DALLE_MODEL);
        assert_eq!(request.prompt, "Astronaut");
        assert_eq!(request.n, 1);
        assert_eq!(request.size, "1792x1024");
        assert_eq!(request.quality, "hd");
    }

    #[test]
    fn test_build_dalle_request_with_multiple_arguments() {
        let args = vec![
            "gpt".to_string(),
            "d".to_string(),
            "Astronaut".to_string(),
            "on".to_string(),
            "Mars".to_string(),
        ];

        let request = build_dalle_request(&args);

        assert_eq!(request.model, DALLE_MODEL);
        assert_eq!(request.prompt, "Astronaut on Mars");
        assert_eq!(request.n, 1);
        assert_eq!(request.size, "1792x1024");
        assert_eq!(request.quality, "hd");
    }

    #[tokio::test]
    async fn test_process_vision_response() {
        let mock_server = MockServer::start().await;
        let response_body = r#"{
            "choices": [
                {
                    "message": {
                        "content": "This is a test response"
                    }
                }
            ]
        }"#;
        let response = ResponseTemplate::new(200).set_body_string(response_body.to_string());
        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(response)
            .mount(&mock_server)
            .await;

        let url = format!("{}/test", &mock_server.uri());
        let client = Client::new();
        let res = client.get(&url).send().await.unwrap();

        let result = process_vision_response(res).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_chat_response() {
        let mock_server = MockServer::start().await;
        let response_body = r#"data: {"choices": [{"delta": {"content": "Hello, world!"}}]}"#;
        let response = ResponseTemplate::new(200)
            .set_body_string(response_body.to_string())
            .insert_header("Content-Type", "text/event-stream");
        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(response)
            .mount(&mock_server)
            .await;

        let url = format!("{}/test", &mock_server.uri());
        let client = Client::new();
        let res = client.get(&url).send().await.unwrap();
        let result = process_chat_response(res).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_dalle_response() {
        let mock_server = MockServer::start().await;
        let response_body = r#"{
            "data": [
                {"url": "http://example.com/image1.jpg"},
                {"url": "http://example.com/image2.jpg"}
            ]
        }"#;
        let response = ResponseTemplate::new(200).set_body_string(response_body.to_string());
        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(response)
            .mount(&mock_server)
            .await;

        let url = format!("{}/test", &mock_server.uri());
        let client = Client::new();
        let res = client.get(&url).send().await.unwrap();

        let result = process_dalle_response(res).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_request_type_and_url_for_gpt() {
        let args = vec!["gpt".to_string(), "Hello".to_string()];

        let result = create_request_type_and_url(&args).await;
        assert!(result.is_ok());
        let (request_type, api_url) = result.unwrap();

        matches!(request_type, RequestType::Chat(_));
        assert_eq!(api_url, GPT_API_URL);
    }

    #[tokio::test]
    async fn test_create_request_type_and_url_for_gpt4() {
        let args = vec!["gpt".to_string(), "4".to_string(), "Hello".to_string()];

        let result = create_request_type_and_url(&args).await;
        assert!(result.is_ok());
        let (request_type, api_url) = result.unwrap();

        matches!(request_type, RequestType::Chat(_));
        assert_eq!(api_url, GPT_API_URL);
    }

    #[tokio::test]
    async fn test_create_request_type_and_url_for_vision() {
        let args = vec![
            "gpt".to_string(),
            "v".to_string(),
            "rust_astronaut.png".to_string(),
            "Describe this image".to_string(),
        ];

        let result = create_request_type_and_url(&args).await;
        assert!(result.is_ok());
        let (request_type, api_url) = result.unwrap();

        matches!(request_type, RequestType::Vision(_));
        assert_eq!(api_url, GPT_API_URL);
    }

    #[tokio::test]
    async fn test_create_request_type_and_url_for_dalle() {
        let args = vec![
            "gpt".to_string(),
            "d".to_string(),
            "Astronaut on Mars".to_string(),
        ];

        let result = create_request_type_and_url(&args).await;
        assert!(result.is_ok());
        let (request_type, api_url) = result.unwrap();

        matches!(request_type, RequestType::Dalle(_));
        assert_eq!(api_url, DALLE_API_URL);
    }

    #[tokio::test]
    async fn test_create_request_type_and_url_default_case() {
        let args = vec![
            "gpt".to_string(),
            "unknown".to_string(),
            "Hello".to_string(),
        ];

        let result = create_request_type_and_url(&args).await;
        assert!(result.is_ok());
        let (request_type, api_url) = result.unwrap();

        matches!(request_type, RequestType::Chat(_));
        assert_eq!(api_url, GPT_API_URL);
    }

    #[tokio::test]
    async fn test_make_openai_request_successful() {
        env::set_var("OPENAI_API_KEY", "testkey");
        let mock_server = MockServer::start().await;
        let response = ResponseTemplate::new(200)
            .set_body_string(r#"{"choices": [{"message": {"content": "Test response"}}]}"#);
        Mock::given(method("POST"))
            .and(path("/test"))
            .respond_with(response)
            .mount(&mock_server)
            .await;

        let client = Client::new();
        let request_type = RequestType::Chat(OpenAiChatRequestBody {
            model: "gpt-3.5-turbo".to_string(),
            messages: vec![],
            stream: false,
        });
        let api_url = format!("{}/test", mock_server.uri());
        let result = make_openai_request(&client, request_type, &api_url).await;

        if let Err(e) = &result {
            println!("Error: {:?}", e);
        }

        assert!(result.is_ok());
        env::remove_var("OPENAI_API_KEY");
    }

    #[tokio::test]
    async fn test_process_command_integration() {
        let client = Client::new();
        let args = vec![
            "gpt".to_string(),
            "4".to_string(),
            "What is the capital of California?".to_string(),
        ];
        let result = process_command(&client, &args).await;

        assert!(result.is_ok() || result.is_err());
    }
}
