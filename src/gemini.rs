use crate::models::{Content, GeminiRequest, GeminiResponse, Part};
use futures_util::StreamExt;
use regex::Regex;
use reqwest::{Client, StatusCode};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io::{self, Write};

type HandlerError = Box<dyn Error + Send + Sync>;

pub trait GeminiResponseHandler {
    fn on_text(&mut self, text: &str) -> Result<(), HandlerError>;
}

pub struct StdoutResponseHandler;

impl GeminiResponseHandler for StdoutResponseHandler {
    fn on_text(&mut self, text: &str) -> Result<(), HandlerError> {
        print!("{text}");
        io::stdout()
            .flush()
            .map_err(|e| Box::new(e) as HandlerError)
    }
}

#[derive(Debug)]
pub enum GeminiClientError {
    Request(reqwest::Error),
    ApiStatus(StatusCode),
    Output(HandlerError),
}

impl Display for GeminiClientError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Request(err) => write!(f, "{err}"),
            Self::ApiStatus(status) => write!(f, "Помилка API: {status}"),
            Self::Output(err) => write!(f, "{err}"),
        }
    }
}

impl Error for GeminiClientError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Request(err) => Some(err),
            Self::ApiStatus(_) => None,
            Self::Output(err) => Some(err.as_ref()),
        }
    }
}

pub struct GeminiClient {
    api_key: String,
    http_client: Client,
    api_base_url: String,
}

impl GeminiClient {
    const DEFAULT_API_BASE_URL: &str = "https://generativelanguage.googleapis.com/v1beta/models";

    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            http_client: Client::new(),
            api_base_url: Self::DEFAULT_API_BASE_URL.to_string(),
        }
    }

    fn sanitize_for_gemini(input: &str) -> String {
        let mut sanitized = input.to_string();

        // 1. Патерн для довгих Hex-хешів (від 32 до 64 символів).
        // \b гарантує, що це окреме слово. Це покриває MD5, SHA-1, SHA-256.
        let hex_hash_regex = Regex::new(r"\b[a-fA-F0-9]{32,64}\b").unwrap();
        sanitized = hex_hash_regex
            .replace_all(&sanitized, "[REDACTED_HASH]")
            .to_string();

        // 2. Патерн для JWT токенів (завжди починаються з eyJ і мають дві крапки)
        let jwt_regex = Regex::new(r"eyJ[a-zA-Z0-9_-]+\.[a-zA-Z0-9_-]+\.[a-zA-Z0-9_-]+").unwrap();
        sanitized = jwt_regex
            .replace_all(&sanitized, "[REDACTED_JWT]")
            .to_string();

        // 3. Патерн для ключів Google API (починаються з AIza)
        let google_key_regex = Regex::new(r"AIza[0-9A-Za-z-_]{35}").unwrap();
        sanitized = google_key_regex
            .replace_all(&sanitized, "[REDACTED_GOOGLE_KEY]")
            .to_string();

        // 4. Патерн для стандартних UUID (напр. 123e4567-e89b-12d3-a456-426614174000)
        let uuid_regex = Regex::new(
            r"\b[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}\b",
        )
        .unwrap();
        sanitized = uuid_regex
            .replace_all(&sanitized, "[REDACTED_UUID]")
            .to_string();

        // 5. Патерн для AWS Access Key ID (зазвичай 20 символів, великі літери та цифри, починається з AKIA)
        let aws_key_regex = Regex::new(r"\bAKIA[0-9A-Z]{16}\b").unwrap();
        sanitized = aws_key_regex
            .replace_all(&sanitized, "[REDACTED_AWS_KEY]")
            .to_string();
        print!("Sanitised promt {}", sanitized);
        sanitized
    }

    pub async fn stream_generate<H>(
        &self,
        model: &str,
        prompt: &str,
        handler: &mut H,
    ) -> Result<String, GeminiClientError>
    where
        H: GeminiResponseHandler,
    {
        let request = GeminiRequest {
            contents: vec![Content {
                parts: vec![Part {
                    text: Self::sanitize_for_gemini(prompt),
                }],
            }],
        };

        let response = self
            .http_client
            .post(self.build_url(model))
            .json(&request)
            .send()
            .await
            .map_err(GeminiClientError::Request)?;

        if !response.status().is_success() {
            return Err(GeminiClientError::ApiStatus(response.status()));
        }

        let mut response_stream = response.bytes_stream();
        let mut buffer = String::new();
        let mut full_response = String::new();

        while let Some(item) = response_stream.next().await {
            let bytes = item.map_err(GeminiClientError::Request)?;
            buffer.push_str(&String::from_utf8_lossy(&bytes));
            Self::handle_sse_buffer(&mut buffer, handler, &mut full_response)?;
        }

        Ok(full_response)
    }

    fn build_url(&self, model: &str) -> String {
        format!(
            "{}/{}:streamGenerateContent?alt=sse&key={}",
            self.api_base_url, model, self.api_key
        )
    }

    fn handle_sse_buffer<H>(
        buffer: &mut String,
        handler: &mut H,
        full_response: &mut String,
    ) -> Result<(), GeminiClientError>
    where
        H: GeminiResponseHandler,
    {
        while let Some(line_end) = buffer.find('\n') {
            let line = buffer[..line_end].to_string();
            buffer.drain(..line_end + 1);

            match Self::parse_sse_line(&line) {
                Ok(Some(text)) => {
                    handler.on_text(&text).map_err(GeminiClientError::Output)?;
                    full_response.push_str(&text);
                }
                Ok(None) => {}
                Err(_) => eprintln!("Помилка парсингу: {}", &line[6..]),
            }
        }

        Ok(())
    }

    fn parse_sse_line(line: &str) -> Result<Option<String>, serde_json::Error> {
        if !line.starts_with("data: ") {
            return Ok(None);
        }

        let payload = &line[6..];
        if payload.trim().is_empty() || matches!(payload.trim(), "[" | "]") {
            return Ok(None);
        }

        let response = serde_json::from_str::<GeminiResponse>(payload)?;
        Ok(Self::extract_text(response))
    }

    fn extract_text(response: GeminiResponse) -> Option<String> {
        response
            .candidates
            .into_iter()
            .next()?
            .content
            .parts
            .into_iter()
            .next()
            .map(|part| part.text)
    }
}

#[cfg(test)]
mod tests {
    use super::{GeminiClient, GeminiResponseHandler};
    use std::error::Error;

    #[derive(Default)]
    struct CollectingHandler {
        chunks: Vec<String>,
    }

    impl GeminiResponseHandler for CollectingHandler {
        fn on_text(&mut self, text: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
            self.chunks.push(text.to_string());
            Ok(())
        }
    }

    #[test]
    fn parses_text_from_sse_data_line() {
        let line = r#"data: {"candidates":[{"content":{"parts":[{"text":"Привіт"}]}}]}"#;

        let parsed = GeminiClient::parse_sse_line(line).unwrap();

        assert_eq!(parsed.as_deref(), Some("Привіт"));
    }

    #[test]
    fn ignores_non_content_sse_lines() {
        assert_eq!(
            GeminiClient::parse_sse_line("event: message").unwrap(),
            None
        );
        assert_eq!(GeminiClient::parse_sse_line("data: [").unwrap(), None);
        assert_eq!(GeminiClient::parse_sse_line("data: ]").unwrap(), None);
        assert_eq!(GeminiClient::parse_sse_line("data: ").unwrap(), None);
    }

    #[test]
    fn keeps_partial_buffer_for_next_chunk() {
        let mut buffer =
            String::from("data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"Hi\"}]}}]}");
        let mut handler = CollectingHandler::default();
        let mut full_response = String::new();

        GeminiClient::handle_sse_buffer(&mut buffer, &mut handler, &mut full_response).unwrap();

        assert!(handler.chunks.is_empty());
        assert!(full_response.is_empty());
        assert!(buffer.starts_with("data: "));
    }

    #[test]
    fn streams_all_complete_sse_lines_to_handler() {
        let mut buffer = concat!(
            "data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"Hel\"}]}}]}\n",
            "data: {\"candidates\":[{\"content\":{\"parts\":[{\"text\":\"lo\"}]}}]}\n"
        )
        .to_string();
        let mut handler = CollectingHandler::default();
        let mut full_response = String::new();

        GeminiClient::handle_sse_buffer(&mut buffer, &mut handler, &mut full_response).unwrap();

        assert_eq!(handler.chunks, vec!["Hel", "lo"]);
        assert_eq!(full_response, "Hello");
        assert!(buffer.is_empty());
    }
}
