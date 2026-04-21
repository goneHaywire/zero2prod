use crate::helpers::{TestApp, spawn_app};

#[tokio::test]
async fn health_check_works() {
    // arrange
    let TestApp { address, .. } = spawn_app().await;
    let client = reqwest::Client::new();

    // act
    let response = client
        .get(format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to send request");

    // assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
