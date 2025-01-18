use crate::common::test_app::TestApp;
use crate::common::test_client::TestClient;
use crate::prelude::*;

#[tokio::test]
async fn fallback_responds_with_404() -> Result<()> {
    let app = TestApp::spawn().await?;
    let client = TestClient::new(app.address, None)?;

    let response = client.get_arbitrary("doesnt_exist").await?;
    assert_eq!(response.status(), 404);
    Ok(())
}

#[tokio::test]
async fn health_check_responds_with_200_and_zero_content() -> Result<()> {
    let app = TestApp::spawn().await?;
    let client = TestClient::new(app.address, None)?;

    let response = client.health().get().await?;
    assert_eq!(response.status(), 200);
    assert_eq!(None, response.content_length());
    Ok(())
}
