use crate::common::app::TestApp;
use crate::common::client::TestClient;
use crate::prelude::*;

#[tokio::test]
async fn fallback_responds_with_404() -> Result<()> {
    let app = TestApp::spawn().await?;
    let response = reqwest::get(format!("http://{}/doesnt_exist", &app.address))
        .await
        .expect("Failed to send test request.");

    assert_eq!(response.status(), 404);
    Ok(())
}

#[tokio::test]
async fn health_check_responds_with_200_and_zero_content() -> Result<()> {
    let app = TestApp::spawn().await?;
    let client = TestClient::new(app.address, None)?;
    let response = client.health().get().await?;

    assert_eq!(response.status(), 200);
    assert_eq!(Some(0), response.content_length());
    Ok(())
}
