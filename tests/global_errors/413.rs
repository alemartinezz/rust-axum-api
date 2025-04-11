//! tests/global_errors/413.rs
//! Ensures that sending a large payload (> 2MB by default) triggers 413.

#[path = "../mod.rs"]
mod common;

use reqwest::StatusCode;
use serde_json::Value;

#[tokio::test]
async fn returns_413_when_payload_exceeds_global_limit() {
    let base_url: String = common::spawn_app();

    // Generate a payload slightly larger than 2MB.
    let oversized_payload: Vec<u8> = vec![b'X'; 2_097_152 + 100];

    let client: reqwest::Client = reqwest::Client::new();
    let resp: reqwest::Response = client
        .post(format!("{}/hello", base_url))
        .body(oversized_payload)
        .send()
        .await
        .expect("Failed to send large request.");

    // Expect a 413 Payload Too Large response.
    assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);

    // Optionally, verify the JSON.
    let body: String = resp.text().await.unwrap();
    let json: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["status"], "PAYLOAD_TOO_LARGE");
    assert_eq!(json["code"], 413);
}
