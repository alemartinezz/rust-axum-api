//! tests/healthcheck.rs

#[path = "./mod.rs"]
mod common;

use chrono::DateTime;
use reqwest::StatusCode;
use serde_json::Value;

#[tokio::test]
async fn healthcheck_returns_valid_response_format() {
    let base_url: String = common::spawn_app();
    let client: reqwest::Client = reqwest::Client::new();
    
    // Make request to healthcheck endpoint
    let response: reqwest::Response = client
        .get(&format!("{}/healthcheck", base_url))
        .send()
        .await
        .expect("Failed to execute request");

    // Verify HTTP status code and headers
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("Content-Type").unwrap(),
        "application/json"
    );

    // Parse JSON response
    let body = response.text().await.unwrap();
    let json: Value = serde_json::from_str(&body).unwrap();

    // Validate top-level response structure
    assert!(json.get("status").is_some(), "Missing 'status' field");
    assert!(json.get("code").is_some(), "Missing 'code' field");
    assert!(json.get("data").is_some(), "Missing 'data' field");
    assert!(json.get("messages").is_some(), "Missing 'messages' field");
    assert!(json.get("date").is_some(), "Missing 'date' field");

    // Extract values from JSON
    let status: &str = json["status"].as_str().unwrap();
    let code: u64 = json["code"].as_u64().unwrap();
    let data: &Value = &json["data"];
    let messages: &Value = &json["messages"];
    let date_str: &str = json["date"].as_str().unwrap();

    // Verify status code mapping
    assert_eq!(status, "OK");
    assert_eq!(code, 200);

    // Validate data payload structure
    assert!(
        data.get("version").is_some(),
        "Missing 'version' in data field"
    );
    let version = data["version"].as_str().unwrap();
    assert!(!version.is_empty(), "Version should not be empty");

    // Verify messages array content
    assert_eq!(
        messages.as_array().unwrap().len(),
        1,
        "Should contain exactly one message"
    );
    assert_eq!(
        messages[0].as_str().unwrap(),
        "Hello from Axum!",
        "Incorrect message content"
    );

    // Validate timestamp format
    DateTime::parse_from_rfc3339(date_str)
        .expect("Date field is not RFC3339 compliant");
}