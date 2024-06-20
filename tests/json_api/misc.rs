use crate::common::app::TestApp;
use crate::common::client::TestClient;
use crate::prelude::*;

#[tokio::test]
async fn fallback_responds_with_404() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (_, api_key) = app.seed_random_user_and_api_key().await?;
    let client = TestClient::new(app.address, Some(&api_key))?;

    let response = client.get_arbitrary("api/doesnt_exist").await?;

    assert_eq!(response.status(), 404);
    Ok(())
}
