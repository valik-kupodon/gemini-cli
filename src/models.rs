use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct Content {
    pub parts: Vec<Part>,
}

#[derive(Serialize)]
pub struct Part {
    pub text: String,
}

#[derive(Serialize)]
pub struct GeminiRequest {
    pub contents: Vec<Content>,
}

#[derive(Deserialize)]
pub struct GeminiResponse {
    pub candidates: Vec<Candidate>,
}

#[derive(Deserialize)]
pub struct Candidate {
    pub content: ResponseContent,
}

#[derive(Deserialize)]
pub struct ResponseContent {
    pub parts: Vec<ResponsePart>,
}

#[derive(Deserialize)]
pub struct ResponsePart {
    pub text: String,
}
