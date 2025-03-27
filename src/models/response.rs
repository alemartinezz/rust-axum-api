use serde::Serialize;
use serde_json::Value;

#[derive(Serialize)]
pub struct ResponseFormat {
    pub status: String,
    pub code: u16,
    pub data: Value,
    pub messages: Vec<String>,
    pub errors: Vec<String>,
}
