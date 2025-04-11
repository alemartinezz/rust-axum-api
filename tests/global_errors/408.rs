//! tests/global_errors/408.rs
//! Ensures that requests taking too long result in a 408 timeout.

#[path = "../mod.rs"]
mod common;

use reqwest::StatusCode;
use serde_json::Value;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn returns_408_when_request_times_out() {
    let base_url: String = common::spawn_app();

    // In this test, we expect the route to take longer than the timeout.
    // Make sure your handler is configured to sleep (for example, temporarily
    // add a sleep in the hello_handler when a query parameter is present).
    let resp_result: Result<Result<reqwest::Response, reqwest::Error>, tokio::time::error::Elapsed> = timeout(
        Duration::from_secs(5), // client-side timeout duration
        async {
            reqwest::Client::new()
                .get(format!("{}/hello", base_url))
                .send()
                .await
        }
    )
    .await;

    // Ensure the client did not timeout waiting for a response.
    assert!(resp_result.is_ok(), "Client timed out waiting for server.");

    let resp: reqwest::Response = resp_result.unwrap().expect("Request failed unexpectedly.");

    // Check that the response is HTTP 408.
    assert_eq!(resp.status(), StatusCode::REQUEST_TIMEOUT);

    // Parse and verify the JSON output.
    let body: String = resp.text().await.unwrap();
    let json: Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["status"], "REQUEST_TIMEOUT");
    assert_eq!(json["code"], 408);
}
