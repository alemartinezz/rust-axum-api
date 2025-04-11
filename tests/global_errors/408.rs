//! tests/global_errors/408.rs

#[path = "../mod.rs"]
mod common;

use reqwest::StatusCode;
use serde_json::Value;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn returns_408_when_request_times_out() {
    let base_url: String = common::spawn_app();

    // Intentamos GET /timeout, que duerme 5s
    let resp_result: Result<Result<reqwest::Response, reqwest::Error>, tokio::time::error::Elapsed> = timeout(
        Duration::from_secs(5),
        
        async {
            reqwest::Client::new()
                .get(format!("{}/timeout", base_url)) // <-- Ajustado
                .send()
                .await
        }
    )
    .await;

    assert!(resp_result.is_ok(), "Client timed out waiting for server.");

    let resp: reqwest::Response = resp_result.unwrap().expect("Request failed unexpectedly.");

    assert_eq!(resp.status(), StatusCode::REQUEST_TIMEOUT);

    let body: String = resp.text().await.unwrap();
    let json: Value = serde_json::from_str(&body).unwrap();
    
    assert_eq!(json["status"], "REQUEST_TIMEOUT");
    assert_eq!(json["code"], 408);
}
