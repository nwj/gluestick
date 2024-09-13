use crate::common::mocks::mock_user::MockUser;
use crate::common::test_app::TestApp;
use crate::common::test_client::TestClient;
use crate::prelude::*;

#[tokio::test]
async fn fallback_responds_with_404() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (_, api_key) = MockUser::builder()
        .random()?
        .build()
        .seed_with_api_key(&app)
        .await?;
    let client = TestClient::new(app.address, Some(&api_key))?;

    let response = client.get_arbitrary("api/v1/doesnt_exist").await?;

    assert_eq!(response.status(), 404);
    Ok(())
}
