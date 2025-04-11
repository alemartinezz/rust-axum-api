//! tests/global_errors/500.rs
//! Ensures that an internal error maps to an HTTP 500 status.

#[path = "../mod.rs"]
mod common;

use reqwest::StatusCode;
use serde_json::Value;

#[tokio::test]
async fn returns_500_on_internal_error() {
    let base_url: String = common::spawn_app();

    // To trigger a 500, assume you have configured your handler to panic when a certain
    // query parameter is present (e.g., ?force_error=1). Adjust your handler accordingly.
    let url: String = format!("{}/hello?force_error=1", base_url);

    let resp: reqwest::Response = reqwest::Client::new()
        .get(url)
        .send()
        .await
        .expect("Failed to make request.");

    // Expect a 500 Internal Server Error.
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);

    // Optionally, check the JSON output.
    let body: String = resp.text().await.unwrap();
    let json: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["status"], "INTERNAL_SERVER_ERROR");
    assert_eq!(json["code"], 500);
}
