use crate::common::mocks::mock_api_key::MockApiKey;
use crate::common::mocks::mock_pagination::MockPaginationParams;
use crate::common::mocks::mock_paste::MockPaste;
use crate::common::mocks::mock_user::MockUser;
use crate::common::rand_helper::{random_alphanumeric_string, random_filename, random_string};
use crate::common::test_app::TestApp;
use crate::common::test_client::TestClient;
use crate::prelude::*;
use uuid::Uuid;

#[tokio::test]
async fn signup_happy_path() -> Result<()> {
    let app = TestApp::spawn().await?;
    let client = TestClient::new(app.address, None)?;
    let invite = app.seed_random_invite_code().await?;
    let user = MockUser::builder().random()?.build();

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
    let user = MockUser::builder().random()?.build();
    let bad_invites = &["doesnt-exist", ""];

    for bad_invite in bad_invites {
        let response = client.signup().post(bad_invite.to_string(), &user).await?;
        assert_eq!(response.status(), 422);

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

    let user = MockUser::builder().random()?.build();
    let response = client.signup().post(invite.clone(), &user).await?;
    assert_eq!(response.status(), 200);

    // Since settings is session gated, we can use it to check for a session
    let response = client.settings().get().await?;
    assert_eq!(response.status(), 200);

    client.logout().delete().await?;

    let user2 = MockUser::builder().random()?.build();
    let response = client.signup().post(invite, &user2).await?;
    assert_eq!(response.status(), 422);

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
        MockUser::builder().username("").build(),
        MockUser::builder().email("").build(),
        MockUser::builder().password("").build(),
    ];

    for bad_user in bad_users {
        let invite = app.seed_random_invite_code().await?;
        let response = client.signup().post(invite, &bad_user).await?;
        assert_eq!(response.status(), 422);

        // Since settings is session gated, we can use it to check for a session
        let response = client.settings().get().await?;
        assert_eq!(response.status(), 401);
    }

    Ok(())
}

#[tokio::test]
async fn signup_requires_valid_username() -> Result<()> {
    let app = TestApp::spawn().await?;
    let client = TestClient::new(app.address, None)?;
    let bad_users = &[
        MockUser::builder().username("").build(),
        MockUser::builder()
            .username(random_alphanumeric_string(33..=33)?)
            .build(),
        MockUser::builder().username("-starts-with-hyphen").build(),
        MockUser::builder().username("ends-with-hyphen-").build(),
        MockUser::builder()
            .username("two--hyphens-in-a-row")
            .build(),
        MockUser::builder().username(random_string(3..=32)?).build(),
    ];

    for bad_user in bad_users {
        let invite = app.seed_random_invite_code().await?;
        let response = client.signup().post(invite, &bad_user).await?;
        assert_eq!(response.status(), 422);

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
    let random = random_alphanumeric_string(1..=30)?;
    let bad_users = &[
        // Missing @ symbol
        MockUser::builder().email(&random).build(),
        // Missing domain part
        MockUser::builder().email(format!("{random}@")).build(),
        // Missing username part
        MockUser::builder().email(format!("@{random}")).build(),
    ];

    for bad_user in bad_users {
        let invite = app.seed_random_invite_code().await?;
        let response = client.signup().post(invite, &bad_user).await?;
        assert_eq!(response.status(), 422);

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
        MockUser::builder()
            .password(random_alphanumeric_string(7..=7)?)
            .build(),
        MockUser::builder()
            .password(random_alphanumeric_string(257..=257)?)
            .build(),
    ];

    for bad_user in bad_users {
        let invite = app.seed_random_invite_code().await?;
        let response = client.signup().post(invite, &bad_user).await?;
        assert_eq!(response.status(), 422);

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
    let user = MockUser::builder().random()?.username(username).build();
    let dup_user = MockUser::builder().random()?.username(username).build();

    let invite = app.seed_random_invite_code().await?;
    let response = client.signup().post(invite, &user).await?;
    assert_eq!(response.status(), 200);

    // Since settings is session gated, we can use it to check for a session
    let response = client.settings().get().await?;
    assert_eq!(response.status(), 200);

    client.logout().delete().await?;

    let invite = app.seed_random_invite_code().await?;
    let response = client.signup().post(invite, &dup_user).await?;
    assert_eq!(response.status(), 422);

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
    let user = MockUser::builder().random()?.email(email).build();
    let dup_user = MockUser::builder().random()?.email(email).build();

    let invite = app.seed_random_invite_code().await?;
    let response = client.signup().post(invite, &user).await?;
    assert_eq!(response.status(), 200);

    // Since settings is session gated, we can use it to check for a session
    let response = client.settings().get().await?;
    assert_eq!(response.status(), 200);

    client.logout().delete().await?;

    let invite = app.seed_random_invite_code().await?;
    let response = client.signup().post(invite, &dup_user).await?;
    assert_eq!(response.status(), 422);

    // Since settings is session gated, we can use it to check for a session
    let response = client.settings().get().await?;
    assert_eq!(response.status(), 401);

    Ok(())
}

#[tokio::test]
async fn login_logout_happy_path() -> Result<()> {
    let app = TestApp::spawn().await?;
    let client = TestClient::new(app.address, None)?;
    let user = MockUser::builder().random()?.build().seed(&app).await?;

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
    let mut no_email = MockUser::builder().random()?.build().seed(&app).await?;
    no_email.email = "".into();
    let mut no_password = MockUser::builder().random()?.build().seed(&app).await?;
    no_password.password = "".into();
    let mut no_nothing = MockUser::builder().random()?.build().seed(&app).await?;
    no_nothing.email = "".into();
    no_nothing.password = "".into();
    let bad_users = &[no_email, no_password, no_nothing];

    for bad_user in bad_users {
        let response = client.login().post(&bad_user).await?;
        assert_eq!(response.status(), 401);

        let response = client.settings().get().await?;
        assert_eq!(response.status(), 401);
    }
    Ok(())
}

#[tokio::test]
async fn cant_login_with_your_email_but_someone_elses_password() -> Result<()> {
    let app = TestApp::spawn().await?;
    let client = TestClient::new(app.address, None)?;
    let user1 = MockUser::builder().random()?.build().seed(&app).await?;
    let mut user2 = MockUser::builder().random()?.build().seed(&app).await?;
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
    let user1 = MockUser::builder().random()?.build().seed(&app).await?;
    let mut user2 = MockUser::builder().random()?.build().seed(&app).await?;
    user2.email = user1.email;

    let response = client.login().post(&user2).await?;
    assert_eq!(response.status(), 401);

    let response = client.settings().get().await?;
    assert_eq!(response.status(), 401);
    Ok(())
}

#[tokio::test]
async fn show_happpy_path() -> Result<()> {
    let app = TestApp::spawn().await?;
    let user = MockUser::builder().random()?.build().seed(&app).await?;
    let client = TestClient::new(app.address, None)?;
    let paste1 = MockPaste::builder()
        .random()?
        .build()
        .seed(&app, &user)
        .await?;
    let paste2 = MockPaste::builder()
        .random()?
        .build()
        .seed(&app, &user)
        .await?;

    let response = client.username(&user.username).get(None).await?;
    assert_eq!(response.status(), 200);
    let html = response.text().await.unwrap();
    assert!(html.contains(&paste1.filename));
    assert!(html.contains(&paste2.filename));
    Ok(())
}

#[tokio::test]
async fn show_does_not_include_secret_pastes() -> Result<()> {
    let app = TestApp::spawn().await?;
    let user = MockUser::builder().random()?.build().seed(&app).await?;
    let client = TestClient::new(app.address, None)?;
    let paste1 = MockPaste::builder()
        .random()?
        .filename(random_filename(64..=64)?)
        .build()
        .seed(&app, &user)
        .await?;
    let paste2 = MockPaste::builder()
        .random()?
        .filename(random_filename(64..=64)?)
        .visibility("secret")
        .build()
        .seed(&app, &user)
        .await?;

    let response = client.username(&user.username).get(None).await?;
    assert_eq!(response.status(), 200);
    let html = response.text().await.unwrap();
    assert!(html.contains(&paste1.filename));
    assert!(!html.contains(&paste2.filename));
    Ok(())
}

#[tokio::test]
async fn show_does_not_include_other_users_pastes() -> Result<()> {
    let app = TestApp::spawn().await?;
    let user = MockUser::builder().random()?.build().seed(&app).await?;
    let user2 = MockUser::builder().random()?.build().seed(&app).await?;
    let client = TestClient::new(app.address, None)?;
    let paste1 = MockPaste::builder()
        .random()?
        .filename(random_filename(64..=64)?)
        .build()
        .seed(&app, &user)
        .await?;
    let paste2 = MockPaste::builder()
        .random()?
        .filename(random_filename(64..=64)?)
        .build()
        .seed(&app, &user2)
        .await?;

    let response = client.username(&user.username).get(None).await?;
    assert_eq!(response.status(), 200);
    let html = response.text().await.unwrap();
    assert!(html.contains(&paste1.filename));
    assert!(!html.contains(&paste2.filename));
    Ok(())
}

#[tokio::test]
async fn show_includes_your_own_secret_pastes_when_logged_in() -> Result<()> {
    let app = TestApp::spawn().await?;
    let user = MockUser::builder().random()?.build().seed(&app).await?;
    let client = TestClient::new(app.address, None)?;
    let paste1 = MockPaste::builder()
        .random()?
        .filename(random_filename(64..=64)?)
        .build()
        .seed(&app, &user)
        .await?;
    let paste2 = MockPaste::builder()
        .random()?
        .filename(random_filename(64..=64)?)
        .visibility("secret")
        .build()
        .seed(&app, &user)
        .await?;
    client.login().post(&user).await?;

    let response = client.username(&user.username).get(None).await?;
    assert_eq!(response.status(), 200);
    let html = response.text().await.unwrap();
    assert!(html.contains(&paste1.filename));
    assert!(html.contains(&paste2.filename));
    Ok(())
}

#[tokio::test]
async fn show_does_not_include_other_users_secret_pastes_when_logged_in() -> Result<()> {
    let app = TestApp::spawn().await?;
    let user = MockUser::builder().random()?.build().seed(&app).await?;
    let user2 = MockUser::builder().random()?.build().seed(&app).await?;
    let client = TestClient::new(app.address, None)?;
    let paste1 = MockPaste::builder()
        .random()?
        .filename(random_filename(64..=64)?)
        .build()
        .seed(&app, &user2)
        .await?;
    let paste2 = MockPaste::builder()
        .random()?
        .filename(random_filename(64..=64)?)
        .visibility("secret")
        .build()
        .seed(&app, &user2)
        .await?;
    client.login().post(&user).await?;

    let response = client.username(&user2.username).get(None).await?;
    assert_eq!(response.status(), 200);
    let html = response.text().await.unwrap();
    assert!(html.contains(&paste1.filename));
    assert!(!html.contains(&paste2.filename));
    Ok(())
}

#[tokio::test]
async fn show_has_per_page_default() -> Result<()> {
    let app = TestApp::spawn().await?;
    let user = MockUser::builder().random()?.build().seed(&app).await?;
    let client = TestClient::new(app.address, None)?;
    for _ in 0..11 {
        MockPaste::builder()
            .random()?
            .build()
            .seed(&app, &user)
            .await?;
    }

    let response = client.username(&user.username).get(None).await?;
    assert_eq!(response.status(), 200);
    let html = response.text().await?;
    assert_eq!(html.matches("<li class=\"paste\">").count(), 10);
    Ok(())
}

#[tokio::test]
async fn show_uses_per_page_when_provided() -> Result<()> {
    let app = TestApp::spawn().await?;
    let user = MockUser::builder().random()?.build().seed(&app).await?;
    let client = TestClient::new(app.address, None)?;
    let per_page = 3;
    for _ in 0..per_page + 1 {
        MockPaste::builder()
            .random()?
            .build()
            .seed(&app, &user)
            .await?;
    }

    let params = MockPaginationParams::builder().per_page(per_page).build();
    let response = client.username(&user.username).get(Some(params)).await?;
    assert_eq!(response.status(), 200);
    let html = response.text().await?;
    assert_eq!(html.matches("<li class=\"paste\">").count(), per_page);
    Ok(())
}

#[tokio::test]
async fn show_falls_back_to_default_if_per_page_more_than_100() -> Result<()> {
    let app = TestApp::spawn().await?;
    let user = MockUser::builder().random()?.build().seed(&app).await?;
    let client = TestClient::new(app.address, None)?;
    let per_page = 101;
    for _ in 0..11 {
        MockPaste::builder()
            .random()?
            .build()
            .seed(&app, &user)
            .await?;
    }

    let params = MockPaginationParams::builder().per_page(per_page).build();
    let response = client.username(&user.username).get(Some(params)).await?;
    assert_eq!(response.status(), 200);
    let html = response.text().await?;
    assert_eq!(html.matches("<li class=\"paste\">").count(), 10);
    Ok(())
}

#[tokio::test]
async fn show_paginates_correctly() -> Result<()> {
    fn extract_next_cursor(html: &str) -> Result<&str> {
        html.split("next_page=")
            .nth(1)
            .and_then(|s| s.split('"').next())
            .ok_or_else(|| "Failed to find next cursor".into())
    }

    fn extract_prev_cursor(html: &str) -> Result<&str> {
        html.split("prev_page=")
            .nth(1)
            .and_then(|s| s.split('"').next())
            .ok_or_else(|| "Failed to find prev cursor".into())
    }

    let app = TestApp::spawn().await?;
    let user = MockUser::builder().random()?.build().seed(&app).await?;
    let client = TestClient::new(app.address, None)?;

    let mut pastes = Vec::new();
    for _ in 0..8 {
        let paste = MockPaste::builder()
            .random()?
            .id(Uuid::now_v7())
            .build()
            .seed(&app, &user)
            .await?;
        pastes.push(paste);
    }

    // First page
    let params = MockPaginationParams::builder().per_page(3).build();
    let response = client.username(&user.username).get(Some(params)).await?;
    let html = response.text().await?;
    for paste in &pastes[5..8] {
        assert!(html.contains(&paste.filename));
    }
    assert!(html.contains("<span>Newer</span>"));
    assert!(html.contains("Older</a>"));

    // Second page (forward)
    let next_cursor = extract_next_cursor(&html)?;
    let params = MockPaginationParams::builder()
        .per_page(3)
        .next_page(next_cursor)
        .build();
    let response = client.username(&user.username).get(Some(params)).await?;
    let html = response.text().await?;
    for paste in &pastes[2..5] {
        assert!(html.contains(&paste.filename));
    }
    assert!(html.contains("Newer</a>"));
    assert!(html.contains("Older</a>"));

    // Third page (forward)
    let next_cursor = extract_next_cursor(&html)?;
    let params = MockPaginationParams::builder()
        .per_page(3)
        .next_page(next_cursor)
        .build();
    let response = client.username(&user.username).get(Some(params)).await?;
    let html = response.text().await?;
    for paste in &pastes[0..2] {
        assert!(html.contains(&paste.filename));
    }
    assert!(html.contains("Newer</a>"));
    assert!(html.contains("<span>Older</span>"));

    // Second page (backward)
    let prev_cursor = extract_prev_cursor(&html)?;
    let params = MockPaginationParams::builder()
        .per_page(3)
        .prev_page(prev_cursor)
        .build();
    let response = client.username(&user.username).get(Some(params)).await?;
    let html = response.text().await?;
    for paste in &pastes[2..5] {
        assert!(html.contains(&paste.filename));
    }
    assert!(html.contains("Newer</a>"));
    assert!(html.contains("Older</a>"));

    // First page (backward)
    let prev_cursor = extract_prev_cursor(&html)?;
    let params = MockPaginationParams::builder()
        .per_page(3)
        .prev_page(prev_cursor)
        .build();
    let response = client.username(&user.username).get(Some(params)).await?;
    let html = response.text().await?;
    for paste in &pastes[5..8] {
        assert!(html.contains(&paste.filename));
    }
    assert!(html.contains("<span>Newer</span>"));
    assert!(html.contains("Older</a>"));
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
async fn settings_lists_users_api_keys() -> Result<()> {
    let app = TestApp::spawn().await?;
    let user = MockUser::builder().random()?.build().seed(&app).await?;
    let client = TestClient::new(app.address, None)?;
    let api_key1 = MockApiKey::builder()
        .random()?
        .build()
        .seed(&app, &user)
        .await?;
    let api_key2 = MockApiKey::builder()
        .random()?
        .build()
        .seed(&app, &user)
        .await?;
    client.login().post(&user).await?;

    let response = client.settings().get().await?;
    assert_eq!(response.status(), 200);
    let html = response.text().await?;
    assert!(html.contains(&api_key1.name));
    assert!(html.contains(&api_key2.name));

    Ok(())
}

#[tokio::test]
async fn settings_does_not_list_other_users_api_keys() -> Result<()> {
    let app = TestApp::spawn().await?;
    let user = MockUser::builder().random()?.build().seed(&app).await?;
    let other_user = MockUser::builder().random()?.build().seed(&app).await?;
    let client = TestClient::new(app.address, None)?;
    let api_key1 = MockApiKey::builder()
        .random()?
        .build()
        .seed(&app, &other_user)
        .await?;
    let api_key2 = MockApiKey::builder()
        .random()?
        .build()
        .seed(&app, &other_user)
        .await?;
    client.login().post(&user).await?;

    let response = client.settings().get().await?;
    assert_eq!(response.status(), 200);
    let html = response.text().await?;
    assert!(!html.contains(&api_key1.name));
    assert!(!html.contains(&api_key2.name));

    Ok(())
}

#[tokio::test]
async fn can_generate_new_api_keys() -> Result<()> {
    let app = TestApp::spawn().await?;
    let user = MockUser::builder().random()?.build().seed(&app).await?;
    let client = TestClient::new(app.address, None)?;
    client.login().post(&user).await?;

    let response = client.settings().get().await?;
    assert_eq!(response.status(), 200);
    let html = response.text().await?;
    assert!(!html.contains("<li class=\"key\">"));

    let response = client.api_sessions().post().await?;
    assert_eq!(response.status(), 200);

    let response = client.settings().get().await?;
    assert_eq!(response.status(), 200);
    let html = response.text().await?;
    assert!(html.contains("<li class=\"key\">"));

    Ok(())
}

#[tokio::test]
async fn can_delete_api_keys() -> Result<()> {
    let app = TestApp::spawn().await?;
    let user = MockUser::builder().random()?.build().seed(&app).await?;
    let client = TestClient::new(app.address, None)?;
    let api_key = MockApiKey::builder()
        .random()?
        .build()
        .seed(&app, &user)
        .await?;
    client.login().post(&user).await?;

    let response = client.settings().get().await?;
    assert_eq!(response.status(), 200);
    let html = response.text().await?;
    assert!(html.contains(&api_key.name));

    let response = client.api_sessions().delete_by_id(&api_key).await?;
    assert_eq!(response.status(), 200);

    let response = client.settings().get().await?;
    assert_eq!(response.status(), 200);
    let html = response.text().await?;
    assert!(!html.contains(&api_key.name));

    Ok(())
}

#[tokio::test]
async fn cannot_delete_other_users_api_keys() -> Result<()> {
    let app = TestApp::spawn().await?;
    let user = MockUser::builder().random()?.build().seed(&app).await?;
    let client = TestClient::new(app.address, None)?;
    let other_user = MockUser::builder().random()?.build().seed(&app).await?;
    let other_client = TestClient::new(app.address, None)?;
    let api_key = MockApiKey::builder()
        .random()?
        .build()
        .seed(&app, &other_user)
        .await?;
    client.login().post(&user).await?;
    other_client.login().post(&other_user).await?;

    let response = other_client.settings().get().await?;
    assert_eq!(response.status(), 200);
    let html = response.text().await?;
    assert!(html.contains(&api_key.name));

    let response = client.api_sessions().delete_by_id(&api_key).await?;
    assert_eq!(response.status(), 404);

    let response = other_client.settings().get().await?;
    assert_eq!(response.status(), 200);
    let html = response.text().await?;
    assert!(html.contains(&api_key.name));

    Ok(())
}
