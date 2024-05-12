use crate::common;

#[tokio::test]
async fn health_check_responds_with_200() {
    let app = common::spawn_app().await;
    let response = reqwest::get(format!("http://{}/health", &app.address))
        .await
        .expect("Failed to send test request.");

    assert!(response.status().is_success());
}

#[tokio::test]
async fn health_check_responds_with_zero_content() {
    let app = common::spawn_app().await;
    let response = reqwest::get(format!("http://{}/health", &app.address))
        .await
        .expect("Failed to send test request.");

    assert_eq!(Some(0), response.content_length());
}