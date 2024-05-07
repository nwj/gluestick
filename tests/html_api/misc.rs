use crate::common;
use reqwest::StatusCode;

#[tokio::test]
async fn health_check_responds_with_200() {
    let app = common::spawn_app().await;
    let response = reqwest::get(format!("http://{}/health_check", &app.address))
        .await
        .expect("Failed to send test request.");

    assert!(response.status().is_success());
}

#[tokio::test]
async fn health_check_responds_with_zero_content() {
    let app = common::spawn_app().await;
    let response = reqwest::get(format!("http://{}/health_check", &app.address))
        .await
        .expect("Failed to send test request.");

    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn fallback_responds_with_404() {
    let app = common::spawn_app().await;
    let response = reqwest::get(format!("http://{}/doesnt_exist", &app.address))
        .await
        .expect("Failed to send test request.");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
