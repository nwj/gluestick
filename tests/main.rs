use reqwest::StatusCode;

mod common;
mod html_api;
mod json_api;

#[tokio::test]
async fn fallback_responds_with_404() {
    let app = common::spawn_app().await;
    let response = reqwest::get(format!("http://{}/doesnt_exist", &app.address))
        .await
        .expect("Failed to send test request.");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
