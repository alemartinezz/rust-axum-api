// Start of file: /src/models/response_format.rs

use serde::{Serialize, Deserialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
pub struct ResponseFormat {
    pub status: String,
    pub code: u16,
    pub data: Value,
    pub messages: Vec<String>,
    pub errors: Vec<String>,
}

// End of file: /src/models/response_format.rs
