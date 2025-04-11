//! tests/global_errors/500.rs

#[path = "../mod.rs"]
mod common;

use reqwest::StatusCode;
use serde_json::Value;

#[tokio::test]
async fn returns_500_on_internal_error() {
    let base_url: String = common::spawn_app();

    // Para forzar 500, llamamos GET /error-500 que hace panic
    let url: String = format!("{}/error-500", base_url); // <-- Ajustado

    let resp: reqwest::Response = reqwest::Client::new()
        .get(url)
        .send()
        .await
        .expect("Failed to make request.");

    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);

    let body: String = resp.text().await.unwrap();
    let json: Value = serde_json::from_str(&body).unwrap();
    
    assert_eq!(json["status"], "INTERNAL_SERVER_ERROR");
    assert_eq!(json["code"], 500);
}
