use crate::common::app::TestApp;
use crate::common::client::TestClient;
use crate::common::rand_helper;
use crate::common::user_helper::TestUser;
use crate::prelude::*;

#[tokio::test]
async fn signup_happy_path() -> Result<()> {
    let app = TestApp::spawn().await?;
    let client = TestClient::new(app.address, None)?;
    let invite = app.seed_random_invite_code().await?;
    let user = TestUser::builder().random()?.build();

    let response = client.signup().post(invite, &user).await?;

    assert_eq!(response.status(), 200);
    Ok(())
}

#[tokio::test]
async fn signup_requires_valid_invite_code() -> Result<()> {
    let app = TestApp::spawn().await?;
    let client = TestClient::new(app.address, None)?;
    let user = TestUser::builder().random()?.build();
    let bad_invites = &["doesnt-exist", ""];

    for bad_invite in bad_invites {
        let response = client.signup().post(bad_invite.to_string(), &user).await?;
        assert_eq!(response.status(), 401);
    }

    Ok(())
}

#[tokio::test]
async fn signup_requires_all_fields() -> Result<()> {
    let app = TestApp::spawn().await?;
    let client = TestClient::new(app.address, None)?;
    let bad_users = &[
        TestUser::builder().username("").build(),
        TestUser::builder().email("").build(),
        TestUser::builder().password("").build(),
    ];

    for bad_user in bad_users {
        let invite = app.seed_random_invite_code().await?;
        let response = client.signup().post(invite, &bad_user).await?;
        assert_eq!(response.status(), 400);
    }

    Ok(())
}

#[tokio::test]
async fn signup_requires_alphanumeric_username_between_3_and_32_chars() -> Result<()> {
    let app = TestApp::spawn().await?;
    let client = TestClient::new(app.address, None)?;
    let bad_users = &[
        TestUser::builder()
            .username(rand_helper::random_alphanumeric_string(2..=2)?)
            .build(),
        TestUser::builder()
            .username(rand_helper::random_alphanumeric_string(33..=33)?)
            .build(),
        TestUser::builder()
            .username(rand_helper::random_string(3..=32)?)
            .build(),
    ];

    for bad_user in bad_users {
        let invite = app.seed_random_invite_code().await?;
        let response = client.signup().post(invite, &bad_user).await?;
        assert_eq!(response.status(), 400);
    }

    Ok(())
}

#[tokio::test]
async fn signup_requires_valid_email_address() -> Result<()> {
    let app = TestApp::spawn().await?;
    let client = TestClient::new(app.address, None)?;
    let random = rand_helper::random_alphanumeric_string(1..=30)?;
    let bad_users = &[
        // Missing @ symbol
        TestUser::builder().email(&random).build(),
        // Missing domain part
        TestUser::builder().email(format!("{random}@")).build(),
        // Missing username part
        TestUser::builder().email(format!("@{random}")).build(),
    ];

    for bad_user in bad_users {
        let invite = app.seed_random_invite_code().await?;
        let response = client.signup().post(invite, &bad_user).await?;
        assert_eq!(response.status(), 400);
    }

    Ok(())
}

#[tokio::test]
async fn signup_requires_password_between_8_and_256_chars() -> Result<()> {
    let app = TestApp::spawn().await?;
    let client = TestClient::new(app.address, None)?;
    let bad_users = &[
        TestUser::builder()
            .password(rand_helper::random_alphanumeric_string(7..=7)?)
            .build(),
        TestUser::builder()
            .password(rand_helper::random_alphanumeric_string(257..=257)?)
            .build(),
    ];

    for bad_user in bad_users {
        let invite = app.seed_random_invite_code().await?;
        let response = client.signup().post(invite, &bad_user).await?;
        assert_eq!(response.status(), 400);
    }

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
