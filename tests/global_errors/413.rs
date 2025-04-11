//! tests/global_errors/413.rs

#[path = "../mod.rs"]
mod common;

use reqwest::StatusCode;
use serde_json::Value;

#[tokio::test]
async fn returns_413_when_payload_exceeds_global_limit() {
    let base_url: String = common::spawn_app();

    let oversized_payload: Vec<u8> = vec![b'X'; 2_097_152 + 100];

    let client: reqwest::Client = reqwest::Client::new();
    
    let resp: reqwest::Response = client
        .post(format!("{}/big-payload", base_url)) // <-- Ajustado
        .body(oversized_payload)
        .send()
        .await
        .expect("Failed to send large request.");

    assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);

    let body: String = resp.text().await.unwrap();
    let json: Value = serde_json::from_str(&body).unwrap();
    
    assert_eq!(json["status"], "PAYLOAD_TOO_LARGE");
    assert_eq!(json["code"], 413);
}
