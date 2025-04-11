//! tests/global_errors/404.rs
//! Ensures that hitting an unknown route returns HTTP 404.

// Include the helper module defined in tests/mod.rs.
#[path = "../mod.rs"]
mod common;

use reqwest::StatusCode;
use serde_json::Value;

#[tokio::test]
async fn returns_404_for_nonexistent_route() {
    // Use the helper function to spawn the app.
    let base_url: String = common::spawn_app();

    // Send a GET request to a route that does not exist.
    let resp: reqwest::Response = reqwest::Client::new()
        .get(format!("{}/does-not-exist", base_url))
        .send()
        .await
        .expect("Failed to execute request.");

    // Verify the status is 404.
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // Optionally, parse the response JSON.
    let body: String = resp.text().await.unwrap();
    let json: Value = serde_json::from_str(&body).unwrap();

    // Assert the JSON has the expected structure.
    assert_eq!(json["status"], "NOT_FOUND");
    assert_eq!(json["code"], 404);
}
