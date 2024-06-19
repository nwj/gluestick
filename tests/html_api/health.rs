use crate::common::test_app::TestApp;
use crate::prelude::*;

#[tokio::test]
async fn health_check_responds_with_200() -> Result<()> {
    let app = TestApp::spawn().await?;
    let response = reqwest::get(format!("http://{}/health", &app.address))
        .await
        .expect("Failed to send test request.");

    assert!(response.status().is_success());
    Ok(())
}

#[tokio::test]
async fn health_check_responds_with_zero_content() -> Result<()> {
    let app = TestApp::spawn().await?;
    let response = reqwest::get(format!("http://{}/health", &app.address))
        .await
        .expect("Failed to send test request.");

    assert_eq!(Some(0), response.content_length());
    Ok(())
}
