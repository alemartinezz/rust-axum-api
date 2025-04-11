//! tests/global_errors/404.rs

#[path = "../mod.rs"]
mod common;

use reqwest::StatusCode;
use serde_json::Value;

#[tokio::test]
async fn returns_404_for_nonexistent_route() {
    let base_url: String = common::spawn_app();

    // Cualquier ruta que no exista en el Router (ej. /does-not-exist)
    let resp: reqwest::Response = reqwest::Client::new()
        .get(format!("{}/does-not-exist", base_url))
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    let body: String = resp.text().await.unwrap();
    let json: Value = serde_json::from_str(&body).unwrap();

    assert_eq!(json["status"], "NOT_FOUND");
    assert_eq!(json["code"], 404);
}
