// Start of file: /src/models/response_format.rs

/*
    * Defines a universal JSON response format used by our custom middleware
    * to wrap responses in a consistent structure.
*/

use serde::{Serialize, Deserialize};
use serde_json::Value;

#[derive(Serialize, Deserialize)]
pub struct ResponseFormat {
    // The textual representation of HTTP status (e.g., "OK" -> "OK", "404" -> "NOT_FOUND").
    pub status: String,
    // The numeric HTTP status code (e.g., 200, 404).
    pub code: u16,
    // The actual data payload in JSON form.
    pub data: Value,
    // A list of optional messages (e.g., warnings, errors).
    pub messages: Vec<String>,
    // The time (e.g., "50 ms") that the request took to process.
    pub time: String,
    // The UTC date/time (RFC3339) when the server responded.
    pub date: String,
}

// End of file: /src/models/response_format.rs
