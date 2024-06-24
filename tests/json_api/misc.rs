use crate::common::app::TestApp;
use crate::common::client::TestClient;
use crate::common::user_helper::TestUser;
use crate::prelude::*;

#[tokio::test]
async fn fallback_responds_with_404() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (_, api_key) = TestUser::builder()
        .random()?
        .build()
        .seed_with_api_key(&app)
        .await?;
    let client = TestClient::new(app.address, Some(&api_key))?;

    let response = client.get_arbitrary("api/doesnt_exist").await?;

    assert_eq!(response.status(), 404);
    Ok(())
}
