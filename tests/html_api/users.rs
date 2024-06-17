use crate::common;
use crate::common::user_helper::TestUser;
use crate::prelude::*;
use reqwest::{Client, StatusCode};

#[tokio::test]
async fn can_signup_with_valid_invite_code() -> Result<()> {
    let app = common::spawn_app().await;
    let client = Client::builder().cookie_store(true).build()?;
    let user = TestUser::builder().full_random()?.build();
    let invite_code = app.seed_random_invite_code().await?;

    let response = user.signup(&app, &client, invite_code).await?;

    assert_eq!(response.status(), StatusCode::OK);
    Ok(())
}

#[tokio::test]
async fn settings_inaccessible_when_logged_out() -> Result<()> {
    let app = common::spawn_app().await;
    let client = Client::builder().cookie_store(true).build()?;
    let user = TestUser::builder().full_random()?.build();

    let response = user.settings(&app, &client).await?;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    Ok(())
}

#[tokio::test]
async fn has_session_after_signup() -> Result<()> {
    let app = common::spawn_app().await;
    let client = Client::builder().cookie_store(true).build()?;
    let user = TestUser::builder().full_random()?.build();
    let invite_code = app.seed_random_invite_code().await?;

    user.signup(&app, &client, invite_code).await?;
    let response = user.settings(&app, &client).await?;

    assert_eq!(response.status(), StatusCode::OK);
    Ok(())
}

#[tokio::test]
async fn logout_ends_session() -> Result<()> {
    let app = common::spawn_app().await;
    let client = Client::builder().cookie_store(true).build()?;
    let user = TestUser::builder().full_random()?.build();
    let invite_code = app.seed_random_invite_code().await?;
    user.signup(&app, &client, invite_code).await?;

    user.logout(&app, &client).await?;
    let response = user.settings(&app, &client).await?;

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    Ok(())
}

#[tokio::test]
async fn can_login_with_valid_credentials() -> Result<()> {
    let app = common::spawn_app().await;
    let client = Client::builder().cookie_store(true).build()?;
    let user = TestUser::builder().full_random()?.build();
    let invite_code = app.seed_random_invite_code().await?;
    user.signup(&app, &client, invite_code).await?;
    user.logout(&app, &client).await?;

    let response = user.login(&app, &client).await?;

    assert_eq!(response.status(), StatusCode::OK);
    Ok(())
}
#[tokio::test]
async fn has_session_after_login() -> Result<()> {
    let app = common::spawn_app().await;
    let client = Client::builder().cookie_store(true).build()?;
    let user = TestUser::builder().full_random()?.build();
    let invite_code = app.seed_random_invite_code().await?;
    user.signup(&app, &client, invite_code).await?;
    user.logout(&app, &client).await?;

    user.login(&app, &client).await?;
    let response = user.settings(&app, &client).await?;

    assert_eq!(response.status(), StatusCode::OK);
    Ok(())
}
