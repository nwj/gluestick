use reqwest::StatusCode;

mod common;

#[tokio::test]
async fn health_check_responds_with_200() {
    let address = common::spawn_server().await;
    let response = reqwest::get(format!("http://{}/health_check", &address))
        .await
        .expect("Failed to send test request.");

    assert!(response.status().is_success());
}

#[tokio::test]
async fn health_check_responds_with_zero_content() {
    let address = common::spawn_server().await;
    let response = reqwest::get(format!("http://{}/health_check", &address))
        .await
        .expect("Failed to send test request.");

    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn fallback_responds_with_404() {
    let address = common::spawn_server().await;
    let response = reqwest::get(format!("http://{}/doesnt_exist", &address))
        .await
        .expect("Failed to send test request.");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
