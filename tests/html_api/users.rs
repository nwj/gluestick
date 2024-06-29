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

    // Since settings is session gated, we can use it to check for a session
    let response = client.settings().get().await?;
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
        assert_eq!(response.status(), 200);

        // Since settings is session gated, we can use it to check for a session
        let response = client.settings().get().await?;
        assert_eq!(response.status(), 401);
    }

    Ok(())
}

#[tokio::test]
async fn invite_codes_are_consumed_on_signup() -> Result<()> {
    let app = TestApp::spawn().await?;
    let client = TestClient::new(app.address, None)?;
    let invite = app.seed_random_invite_code().await?;

    let user = TestUser::builder().random()?.build();
    let response = client.signup().post(invite.clone(), &user).await?;
    assert_eq!(response.status(), 200);

    // Since settings is session gated, we can use it to check for a session
    let response = client.settings().get().await?;
    assert_eq!(response.status(), 200);

    client.logout().delete().await?;

    let user2 = TestUser::builder().random()?.build();
    let response = client.signup().post(invite, &user2).await?;
    assert_eq!(response.status(), 200);

    // Since settings is session gated, we can use it to check for a session
    let response = client.settings().get().await?;
    assert_eq!(response.status(), 401);

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
        assert_eq!(response.status(), 200);

        // Since settings is session gated, we can use it to check for a session
        let response = client.settings().get().await?;
        assert_eq!(response.status(), 401);
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
        assert_eq!(response.status(), 200);

        // Since settings is session gated, we can use it to check for a session
        let response = client.settings().get().await?;
        assert_eq!(response.status(), 401);
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
        assert_eq!(response.status(), 200);

        // Since settings is session gated, we can use it to check for a session
        let response = client.settings().get().await?;
        assert_eq!(response.status(), 401);
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
        assert_eq!(response.status(), 200);

        // Since settings is session gated, we can use it to check for a session
        let response = client.settings().get().await?;
        assert_eq!(response.status(), 401);
    }

    Ok(())
}

#[tokio::test]
async fn cant_signup_twice_with_the_same_username() -> Result<()> {
    let app = TestApp::spawn().await?;
    let client = TestClient::new(app.address, None)?;
    let username = "POOPFEAST420";
    let user = TestUser::builder().random()?.username(username).build();
    let dup_user = TestUser::builder().random()?.username(username).build();

    let invite = app.seed_random_invite_code().await?;
    let response = client.signup().post(invite, &user).await?;
    assert_eq!(response.status(), 200);

    // Since settings is session gated, we can use it to check for a session
    let response = client.settings().get().await?;
    assert_eq!(response.status(), 200);

    client.logout().delete().await?;

    let invite = app.seed_random_invite_code().await?;
    let response = client.signup().post(invite, &dup_user).await?;
    assert_eq!(response.status(), 200);

    // Since settings is session gated, we can use it to check for a session
    let response = client.settings().get().await?;
    assert_eq!(response.status(), 401);

    Ok(())
}

#[tokio::test]
async fn cant_signup_twice_with_the_same_email() -> Result<()> {
    let app = TestApp::spawn().await?;
    let client = TestClient::new(app.address, None)?;
    let email = "john.malkovich@johnmalkovich.com";
    let user = TestUser::builder().random()?.email(email).build();
    let dup_user = TestUser::builder().random()?.email(email).build();

    let invite = app.seed_random_invite_code().await?;
    let response = client.signup().post(invite, &user).await?;
    assert_eq!(response.status(), 200);

    // Since settings is session gated, we can use it to check for a session
    let response = client.settings().get().await?;
    assert_eq!(response.status(), 200);

    client.logout().delete().await?;

    let invite = app.seed_random_invite_code().await?;
    let response = client.signup().post(invite, &dup_user).await?;
    assert_eq!(response.status(), 200);

    // Since settings is session gated, we can use it to check for a session
    let response = client.settings().get().await?;
    assert_eq!(response.status(), 401);

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
async fn login_logout_happy_path() -> Result<()> {
    let app = TestApp::spawn().await?;
    let client = TestClient::new(app.address, None)?;
    let user = TestUser::builder().random()?.build().seed(&app).await?;

    let response = client.login().post(&user).await?;
    assert_eq!(response.status(), 200);

    // Check for a session
    let response = client.settings().get().await?;
    assert_eq!(response.status(), 200);

    let response = client.logout().delete().await?;
    assert_eq!(response.status(), 200);

    // Check that session is gone
    let response = client.settings().get().await?;
    assert_eq!(response.status(), 401);
    Ok(())
}

#[tokio::test]
async fn login_requires_email_and_password() -> Result<()> {
    let app = TestApp::spawn().await?;
    let client = TestClient::new(app.address, None)?;
    let mut no_email = TestUser::builder().random()?.build().seed(&app).await?;
    no_email.email = "".into();
    let mut no_password = TestUser::builder().random()?.build().seed(&app).await?;
    no_password.password = "".into();
    let mut no_nothing = TestUser::builder().random()?.build().seed(&app).await?;
    no_nothing.email = "".into();
    no_nothing.password = "".into();
    let bad_users = &[no_email, no_password, no_nothing];

    for bad_user in bad_users {
        let response = client.login().post(&bad_user).await?;
        assert_eq!(response.status(), 401);
    }
    Ok(())
}

#[tokio::test]
async fn cant_login_with_your_email_but_someone_elses_password() -> Result<()> {
    let app = TestApp::spawn().await?;
    let client = TestClient::new(app.address, None)?;
    let user1 = TestUser::builder().random()?.build().seed(&app).await?;
    let mut user2 = TestUser::builder().random()?.build().seed(&app).await?;
    user2.password = user1.password;

    let response = client.login().post(&user2).await?;
    assert_eq!(response.status(), 401);

    let response = client.settings().get().await?;
    assert_eq!(response.status(), 401);
    Ok(())
}

#[tokio::test]
async fn cant_login_with_your_password_but_someone_elses_email() -> Result<()> {
    let app = TestApp::spawn().await?;
    let client = TestClient::new(app.address, None)?;
    let user1 = TestUser::builder().random()?.build().seed(&app).await?;
    let mut user2 = TestUser::builder().random()?.build().seed(&app).await?;
    user2.email = user1.email;

    let response = client.login().post(&user2).await?;
    assert_eq!(response.status(), 401);

    let response = client.settings().get().await?;
    assert_eq!(response.status(), 401);
    Ok(())
}
