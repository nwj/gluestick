use crate::common::test_app::TestApp;
use crate::prelude::*;

#[tokio::test]
async fn fallback_responds_with_404() -> Result<()> {
    let app = TestApp::spawn().await?;
    let response = reqwest::get(format!("http://{}/api/doesnt_exist", &app.address))
        .await
        .expect("Failed to send test request.");

    assert_eq!(response.status(), 404);
    Ok(())
}
