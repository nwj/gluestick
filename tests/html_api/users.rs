use crate::common::app::TestApp;
use crate::common::client::TestClient;
use crate::common::user_helper::TestUser;
use crate::prelude::*;

#[tokio::test]
async fn can_signup_with_valid_invite_code() -> Result<()> {
    let app = TestApp::spawn().await?;
    let client = TestClient::new(app.address, None)?;
    let invite = app.seed_random_invite_code().await?;
    let user = TestUser::builder().random()?.build();

    let response = client.signup().post(invite, &user).await?;

    assert_eq!(response.status(), 200);
    Ok(())
}

#[tokio::test]
async fn settings_inaccessible_when_logged_out() -> Result<()> {
    let app = TestApp::spawn().await?;
    let client = TestClient::new(app.address, None)?;

    let response = client.settings().get().await?;

    assert_eq!(response.status(), 401);
    Ok(())
}

#[tokio::test]
async fn has_session_after_signup() -> Result<()> {
    let app = TestApp::spawn().await?;
    let client = TestClient::new(app.address, None)?;
    let invite = app.seed_random_invite_code().await?;
    let user = TestUser::builder().random()?.build();

    client.signup().post(invite, &user).await?;

    // Since settings is session gated, we can use it to check for a session
    let response = client.settings().get().await?;
    assert_eq!(response.status(), 200);
    Ok(())
}

#[tokio::test]
async fn logout_ends_session() -> Result<()> {
    let app = TestApp::spawn().await?;
    let client = TestClient::new(app.address, None)?;
    let invite = app.seed_random_invite_code().await?;
    TestUser::builder()
        .build()
        .persist_with_session(&client, invite)
        .await?;

    client.logout().delete().await?;

    // Since settings is session gated, we can use it to check for a session
    let response = client.settings().get().await?;
    assert_eq!(response.status(), 401);
    Ok(())
}

#[tokio::test]
async fn can_login_with_valid_credentials() -> Result<()> {
    let app = TestApp::spawn().await?;
    let client = TestClient::new(app.address, None)?;
    let invite = app.seed_random_invite_code().await?;
    let user = TestUser::builder().build().persist(&client, invite).await?;

    let response = client.login().post(&user).await?;

    assert_eq!(response.status(), 200);
    Ok(())
}
#[tokio::test]
async fn has_session_after_login() -> Result<()> {
    let app = TestApp::spawn().await?;
    let client = TestClient::new(app.address, None)?;
    let invite = app.seed_random_invite_code().await?;
    let user = TestUser::builder().build().persist(&client, invite).await?;

    client.login().post(&user).await?;

    // Since settings is session gated, we can use it to check for a session
    let response = client.settings().get().await?;
    assert_eq!(response.status(), 200);
    Ok(())
}
