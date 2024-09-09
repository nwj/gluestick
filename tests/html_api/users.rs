use crate::common::api_key_helper::TestApiKey;
use crate::common::app::TestApp;
use crate::common::client::TestClient;
use crate::common::pagination_helper::PaginationParams;
use crate::common::paste_helper::TestPaste;
use crate::common::rand_helper::{random_alphanumeric_string, random_filename, random_string};
use crate::common::user_helper::TestUser;
use crate::prelude::*;
use jiff::Timestamp as JiffTimestamp;
use uuid::{NoContext, Timestamp, Uuid};

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

    let user = TestUser::builder().random()?.build();
    let response = client.signup().post(invite.clone(), &user).await?;
    assert_eq!(response.status(), 200);

    // Since settings is session gated, we can use it to check for a session
    let response = client.settings().get().await?;
    assert_eq!(response.status(), 200);

    client.logout().delete().await?;

    let user2 = TestUser::builder().random()?.build();
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
        TestUser::builder().username("").build(),
        TestUser::builder().email("").build(),
        TestUser::builder().password("").build(),
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
        TestUser::builder().username("").build(),
        TestUser::builder()
            .username(random_alphanumeric_string(33..=33)?)
            .build(),
        TestUser::builder().username("-starts-with-hyphen").build(),
        TestUser::builder().username("ends-with-hyphen-").build(),
        TestUser::builder()
            .username("two--hyphens-in-a-row")
            .build(),
        TestUser::builder().username(random_string(3..=32)?).build(),
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
        TestUser::builder().email(&random).build(),
        // Missing domain part
        TestUser::builder().email(format!("{random}@")).build(),
        // Missing username part
        TestUser::builder().email(format!("@{random}")).build(),
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
        TestUser::builder()
            .password(random_alphanumeric_string(7..=7)?)
            .build(),
        TestUser::builder()
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

        let response = client.settings().get().await?;
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

#[tokio::test]
async fn show_happpy_path() -> Result<()> {
    let app = TestApp::spawn().await?;
    let user = TestUser::builder().random()?.build().seed(&app).await?;
    let client = TestClient::new(app.address, None)?;
    let paste1 = TestPaste::builder()
        .random()?
        .build()
        .seed(&app, &user)
        .await?;
    let paste2 = TestPaste::builder()
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
    let user = TestUser::builder().random()?.build().seed(&app).await?;
    let client = TestClient::new(app.address, None)?;
    let paste1 = TestPaste::builder()
        .random()?
        .filename(random_filename(64..=64)?)
        .build()
        .seed(&app, &user)
        .await?;
    let paste2 = TestPaste::builder()
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
    let user = TestUser::builder().random()?.build().seed(&app).await?;
    let user2 = TestUser::builder().random()?.build().seed(&app).await?;
    let client = TestClient::new(app.address, None)?;
    let paste1 = TestPaste::builder()
        .random()?
        .filename(random_filename(64..=64)?)
        .build()
        .seed(&app, &user)
        .await?;
    let paste2 = TestPaste::builder()
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
    let user = TestUser::builder().random()?.build().seed(&app).await?;
    let client = TestClient::new(app.address, None)?;
    let paste1 = TestPaste::builder()
        .random()?
        .filename(random_filename(64..=64)?)
        .build()
        .seed(&app, &user)
        .await?;
    let paste2 = TestPaste::builder()
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
    let user = TestUser::builder().random()?.build().seed(&app).await?;
    let user2 = TestUser::builder().random()?.build().seed(&app).await?;
    let client = TestClient::new(app.address, None)?;
    let paste1 = TestPaste::builder()
        .random()?
        .filename(random_filename(64..=64)?)
        .build()
        .seed(&app, &user2)
        .await?;
    let paste2 = TestPaste::builder()
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
    let user = TestUser::builder().random()?.build().seed(&app).await?;
    let client = TestClient::new(app.address, None)?;
    for _ in 0..11 {
        TestPaste::builder()
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
    let user = TestUser::builder().random()?.build().seed(&app).await?;
    let client = TestClient::new(app.address, None)?;
    let per_page = 3;
    for _ in 0..per_page + 1 {
        TestPaste::builder()
            .random()?
            .build()
            .seed(&app, &user)
            .await?;
    }

    let params = PaginationParams::builder().per_page(per_page).build();
    let response = client.username(&user.username).get(Some(params)).await?;
    assert_eq!(response.status(), 200);
    let html = response.text().await?;
    assert_eq!(html.matches("<li class=\"paste\">").count(), per_page);
    Ok(())
}

#[tokio::test]
async fn show_falls_back_to_default_if_per_page_more_than_100() -> Result<()> {
    let app = TestApp::spawn().await?;
    let user = TestUser::builder().random()?.build().seed(&app).await?;
    let client = TestClient::new(app.address, None)?;
    let per_page = 101;
    for _ in 0..11 {
        TestPaste::builder()
            .random()?
            .build()
            .seed(&app, &user)
            .await?;
    }

    let params = PaginationParams::builder().per_page(per_page).build();
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
    let user = TestUser::builder().random()?.build().seed(&app).await?;
    let client = TestClient::new(app.address, None)?;
    let now = (JiffTimestamp::now().as_millisecond()) as u64;

    let mut pastes = Vec::new();
    for i in 0..8 {
        let paste = TestPaste::builder()
            .random()?
            // This is necessary because we assert below on the order of items within and across pages.
            // That order is based on uuid v7 ordering, which has millisecond precision. Our tests are
            // so fast at creating these pastes that without this sleep, we can get multiple pastes
            // with the same millisecond of creation, which then fails the ordering assertions.
            .id(Uuid::new_v7(Timestamp::from_unix(NoContext, now + i, 0)))
            .build()
            .seed(&app, &user)
            .await?;
        pastes.push(paste);
    }

    // First page
    let params = PaginationParams::builder().per_page(3).build();
    let response = client.username(&user.username).get(Some(params)).await?;
    let html = response.text().await?;
    for paste in &pastes[5..8] {
        assert!(html.contains(&paste.filename));
    }
    assert!(html.contains("<span>Newer</span>"));
    assert!(html.contains("Older</a>"));

    // Second page (forward)
    let next_cursor = extract_next_cursor(&html)?;
    let params = PaginationParams::builder()
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
    let params = PaginationParams::builder()
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
    let params = PaginationParams::builder()
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
    let params = PaginationParams::builder()
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
    let user = TestUser::builder().random()?.build().seed(&app).await?;
    let client = TestClient::new(app.address, None)?;
    let api_key1 = TestApiKey::builder()
        .random()?
        .build()
        .seed(&app, &user)
        .await?;
    let api_key2 = TestApiKey::builder()
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
    let user = TestUser::builder().random()?.build().seed(&app).await?;
    let other_user = TestUser::builder().random()?.build().seed(&app).await?;
    let client = TestClient::new(app.address, None)?;
    let api_key1 = TestApiKey::builder()
        .random()?
        .build()
        .seed(&app, &other_user)
        .await?;
    let api_key2 = TestApiKey::builder()
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
    let user = TestUser::builder().random()?.build().seed(&app).await?;
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
    let user = TestUser::builder().random()?.build().seed(&app).await?;
    let client = TestClient::new(app.address, None)?;
    let api_key = TestApiKey::builder()
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
    let user = TestUser::builder().random()?.build().seed(&app).await?;
    let client = TestClient::new(app.address, None)?;
    let other_user = TestUser::builder().random()?.build().seed(&app).await?;
    let other_client = TestClient::new(app.address, None)?;
    let api_key = TestApiKey::builder()
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
