use crate::common::app::TestApp;
use crate::common::client::TestClient;
use crate::common::pagination_helper::PaginationParams;
use crate::common::paste_helper::TestPaste;
use crate::common::rand_helper::{random_alphanumeric_string, random_filename, random_string};
use crate::common::user_helper::TestUser;
use crate::prelude::*;
use jiff::Timestamp as JiffTimestamp;
use reqwest::header::HeaderValue;
use uuid::{NoContext, Timestamp, Uuid};

#[tokio::test]
async fn index_happy_path() -> Result<()> {
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

    let response = client.pastes().get(None).await?;
    assert_eq!(response.status(), 200);
    let html = response.text().await.unwrap();
    assert!(html.contains(&paste1.filename));
    assert!(html.contains(&paste2.filename));
    Ok(())
}

#[tokio::test]
async fn index_does_not_include_secret_pastes() -> Result<()> {
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

    let response = client.pastes().get(None).await?;
    assert_eq!(response.status(), 200);
    let html = response.text().await.unwrap();
    assert!(html.contains(&paste1.filename));
    assert!(!html.contains(&paste2.filename));
    Ok(())
}

#[tokio::test]
async fn index_has_per_page_default() -> Result<()> {
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

    let response = client.pastes().get(None).await?;
    assert_eq!(response.status(), 200);
    let html = response.text().await?;
    assert_eq!(html.matches("<li class=\"paste\">").count(), 10);
    Ok(())
}

#[tokio::test]
async fn index_uses_per_page_when_provided() -> Result<()> {
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
    let response = client.pastes().get(Some(params)).await?;
    assert_eq!(response.status(), 200);
    let html = response.text().await?;
    assert_eq!(html.matches("<li class=\"paste\">").count(), per_page);
    Ok(())
}

#[tokio::test]
async fn index_400s_if_per_page_more_than_100() -> Result<()> {
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
    let response = client.pastes().get(Some(params)).await?;
    assert_eq!(response.status(), 400);
    Ok(())
}

#[tokio::test]
async fn index_paginates_correctly() -> Result<()> {
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
    let response = client.pastes().get(Some(params)).await?;
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
    let response = client.pastes().get(Some(params)).await?;
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
    let response = client.pastes().get(Some(params)).await?;
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
    let response = client.pastes().get(Some(params)).await?;
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
    let response = client.pastes().get(Some(params)).await?;
    let html = response.text().await?;
    for paste in &pastes[5..8] {
        assert!(html.contains(&paste.filename));
    }
    assert!(html.contains("<span>Newer</span>"));
    assert!(html.contains("Older</a>"));
    Ok(())
}

#[tokio::test]
async fn create_show_update_destroy_happy_path() -> Result<()> {
    let app = TestApp::spawn().await?;
    let user = TestUser::builder().random()?.build().seed(&app).await?;
    let client = TestClient::new(app.address, None)?;
    client.login().post(&user).await?;
    let mut paste = TestPaste::builder().random()?.build();

    // Create
    let response = client.pastes().post(&paste).await?;
    assert_eq!(response.status(), 200);
    paste.id = response.url().path().split("/").nth(2).map(String::from);

    // Show
    let response = client
        .username(&user.username)
        .get_by_paste_id(&paste)
        .await?;
    assert_eq!(response.status(), 200);
    let html = response.text().await?;
    assert!(html.contains(&paste.filename));
    assert!(html.contains(&paste.description));
    assert!(html.contains(&paste.body));

    // Update
    paste.filename = random_filename(1..=30)?;
    paste.description = random_string(1..=30)?;
    paste.body = random_string(1..=30)?;
    let response = client
        .username(&user.username)
        .put_by_paste_id(&paste)
        .await?;
    assert_eq!(response.status(), 200);

    // Show
    let response = client
        .username(&user.username)
        .get_by_paste_id(&paste)
        .await?;
    assert_eq!(response.status(), 200);
    let html = response.text().await?;
    assert!(html.contains(&paste.filename));
    assert!(html.contains(&paste.description));
    assert!(html.contains(&paste.body));

    // Delete
    let response = client
        .username(&user.username)
        .delete_by_paste_id(&paste)
        .await?;
    assert_eq!(response.status(), 200);

    // Show
    let response = client
        .username(&user.username)
        .get_by_paste_id(&paste)
        .await?;
    assert_eq!(response.status(), 404);
    Ok(())
}

#[tokio::test]
async fn create_requires_session() -> Result<()> {
    let app = TestApp::spawn().await?;
    let _user = TestUser::builder().random()?.build().seed(&app).await?;
    let client = TestClient::new(app.address, None)?;
    let paste = TestPaste::builder().random()?.build();

    let response = client.pastes().post(&paste).await?;
    assert_eq!(response.status(), 401);
    Ok(())
}

#[tokio::test]
async fn create_does_not_persist_paste_when_missing_required_fields() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (user, api_key) = TestUser::builder()
        .random()?
        .build()
        .seed_with_api_key(&app)
        .await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
    client.login().post(&user).await?;
    let bad_pastes = vec![
        TestPaste::builder().filename("").build(),
        TestPaste::builder().body("").build(),
    ];

    for bad_paste in bad_pastes {
        let response = client.pastes().post(&bad_paste).await?;
        assert_eq!(response.status(), 200);

        let response_data = client.api_pastes().get_and_deserialize(None).await?;
        assert_eq!(response_data.pastes.len(), 0);
    }
    Ok(())
}

#[tokio::test]
async fn create_does_not_persist_paste_when_invalid_fields() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (user, api_key) = TestUser::builder()
        .random()?
        .build()
        .seed_with_api_key(&app)
        .await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
    client.login().post(&user).await?;
    let bad_pastes = vec![
        TestPaste::builder()
            .filename(random_filename(257..=257)?)
            .build(),
        TestPaste::builder()
            .filename("illegal/characters.md")
            .build(),
        TestPaste::builder()
            .description(random_alphanumeric_string(257..=257)?)
            .build(),
    ];

    for bad_paste in bad_pastes {
        let response = client.pastes().post(&bad_paste).await?;
        assert_eq!(response.status(), 200);

        let response_data = client.api_pastes().get_and_deserialize(None).await?;
        assert_eq!(response_data.pastes.len(), 0);
    }
    Ok(())
}

#[tokio::test]
async fn show_responds_with_404_when_paste_doesnt_exist() -> Result<()> {
    let app = TestApp::spawn().await?;
    let user = TestUser::builder().random()?.build().seed(&app).await?;
    let client = TestClient::new(app.address, None)?;
    let unpersisted_paste = TestPaste::builder().random()?.random_id().build();

    let response = client
        .username(&user.username)
        .get_by_paste_id(&unpersisted_paste)
        .await?;
    assert_eq!(response.status(), 404);
    Ok(())
}

#[tokio::test]
async fn show_and_edit_include_no_index_header_when_paste_is_secret() -> Result<()> {
    let app = TestApp::spawn().await?;
    let user = TestUser::builder().random()?.build().seed(&app).await?;
    let client = TestClient::new(app.address, None)?;
    client.login().post(&user).await?;
    let paste = TestPaste::builder()
        .random()?
        .visibility("secret")
        .build()
        .seed(&app, &user)
        .await?;

    let response = client
        .username(&user.username)
        .get_by_paste_id(&paste)
        .await?;
    let maybe_x_robots_tag = response.headers().get("X-Robots-Tag");
    assert!(maybe_x_robots_tag.is_some(),);
    if let Some(x_robots_tag) = maybe_x_robots_tag {
        assert_eq!(x_robots_tag, HeaderValue::from_static("noindex"),);
    }

    let response = client
        .username(&user.username)
        .get_raw_by_paste_id(&paste)
        .await?;
    let maybe_x_robots_tag = response.headers().get("X-Robots-Tag");
    assert!(maybe_x_robots_tag.is_some(),);
    if let Some(x_robots_tag) = maybe_x_robots_tag {
        assert_eq!(x_robots_tag, HeaderValue::from_static("noindex"),);
    }

    let response = client
        .username(&user.username)
        .get_download_by_paste_id(&paste)
        .await?;
    let maybe_x_robots_tag = response.headers().get("X-Robots-Tag");
    assert!(maybe_x_robots_tag.is_some(),);
    if let Some(x_robots_tag) = maybe_x_robots_tag {
        assert_eq!(x_robots_tag, HeaderValue::from_static("noindex"),);
    }

    let response = client
        .username(&user.username)
        .get_edit_by_paste_id(&paste)
        .await?;
    let maybe_x_robots_tag = response.headers().get("X-Robots-Tag");
    assert!(maybe_x_robots_tag.is_some(),);
    if let Some(x_robots_tag) = maybe_x_robots_tag {
        assert_eq!(x_robots_tag, HeaderValue::from_static("noindex"),);
    }

    Ok(())
}

#[tokio::test]
async fn show_and_edit_do_not_include_no_index_header_when_paste_is_public() -> Result<()> {
    let app = TestApp::spawn().await?;
    let user = TestUser::builder().random()?.build().seed(&app).await?;
    let client = TestClient::new(app.address, None)?;
    client.login().post(&user).await?;
    let paste = TestPaste::builder()
        .random()?
        .visibility("public")
        .build()
        .seed(&app, &user)
        .await?;

    let response = client
        .username(&user.username)
        .get_by_paste_id(&paste)
        .await?;
    let maybe_x_robots_tag = response.headers().get("X-Robots-Tag");
    assert!(maybe_x_robots_tag.is_none(),);

    let response = client
        .username(&user.username)
        .get_raw_by_paste_id(&paste)
        .await?;
    let maybe_x_robots_tag = response.headers().get("X-Robots-Tag");
    assert!(maybe_x_robots_tag.is_none(),);

    let response = client
        .username(&user.username)
        .get_download_by_paste_id(&paste)
        .await?;
    let maybe_x_robots_tag = response.headers().get("X-Robots-Tag");
    assert!(maybe_x_robots_tag.is_none(),);

    let response = client
        .username(&user.username)
        .get_edit_by_paste_id(&paste)
        .await?;
    let maybe_x_robots_tag = response.headers().get("X-Robots-Tag");
    assert!(maybe_x_robots_tag.is_none(),);

    Ok(())
}

#[tokio::test]
async fn show_responds_with_404_when_paste_exists_but_username_is_wrong() -> Result<()> {
    let app = TestApp::spawn().await?;
    let user1 = TestUser::builder().random()?.build().seed(&app).await?;
    let user2 = TestUser::builder().random()?.build().seed(&app).await?;
    let client = TestClient::new(app.address, None)?;
    let paste = TestPaste::builder()
        .random()?
        .build()
        .seed(&app, &user2)
        .await?;

    let response = client
        .username(&String::from("garbage"))
        .get_by_paste_id(&paste)
        .await?;
    assert_eq!(response.status(), 404);

    let response = client
        .username(&user1.username)
        .get_by_paste_id(&paste)
        .await?;
    assert_eq!(response.status(), 404);

    Ok(())
}

#[tokio::test]
async fn update_requires_a_session() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (user, api_key) = TestUser::builder()
        .random()?
        .build()
        .seed_with_api_key(&app)
        .await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
    let paste = TestPaste::builder()
        .random()?
        .build()
        .seed(&app, &user)
        .await?;
    let mut modified_paste = paste.clone();
    modified_paste.filename = random_filename(1..=30)?;

    let response = client
        .username(&user.username)
        .put_by_paste_id(&modified_paste)
        .await?;
    assert_eq!(response.status(), 401);

    let response = client.api_pastes().get_by_id(&paste).await?;
    assert_eq!(response.status(), 200);
    let persisted_paste: TestPaste = response.json().await?;
    assert_eq!(paste, persisted_paste);
    Ok(())
}

#[tokio::test]
async fn update_responds_with_404_when_paste_doesnt_exist() -> Result<()> {
    let app = TestApp::spawn().await?;
    let user = TestUser::builder().random()?.build().seed(&app).await?;
    let client = TestClient::new(app.address, None)?;
    client.login().post(&user).await?;
    let paste = TestPaste::builder().random()?.random_id().build();

    let response = client
        .username(&user.username)
        .put_by_paste_id(&paste)
        .await?;
    assert_eq!(response.status(), 404);
    Ok(())
}

#[tokio::test]
async fn update_does_not_persist_paste_when_invalid_fields() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (user, api_key) = TestUser::builder()
        .random()?
        .build()
        .seed_with_api_key(&app)
        .await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
    client.login().post(&user).await?;
    let paste = TestPaste::builder()
        .random()?
        .build()
        .seed(&app, &user)
        .await?;
    let bad_pastes = vec![
        TestPaste::builder()
            .id(paste.id.clone().unwrap())
            .filename(random_filename(257..=257)?)
            .build(),
        TestPaste::builder()
            .id(paste.id.clone().unwrap())
            .filename("illegal/characters.md")
            .build(),
        TestPaste::builder()
            .id(paste.id.clone().unwrap())
            .description(random_alphanumeric_string(257..=257)?)
            .build(),
    ];

    for bad_paste in bad_pastes {
        let response = client
            .username(&user.username)
            .put_by_paste_id(&bad_paste)
            .await?;
        assert_eq!(response.status(), 200);

        let response = client.api_pastes().get_by_id(&bad_paste).await?;
        let persisted_paste: TestPaste = response.json().await?;
        assert_eq!(persisted_paste, paste);
    }
    Ok(())
}

#[tokio::test]
async fn cannot_update_other_users_pastes() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (user1, api_key1) = TestUser::builder()
        .random()?
        .build()
        .seed_with_api_key(&app)
        .await?;
    let client1 = TestClient::new(app.address, Some(&api_key1))?;
    let paste = TestPaste::builder()
        .random()?
        .build()
        .seed(&app, &user1)
        .await?;
    let (user2, api_key2) = TestUser::builder()
        .random()?
        .build()
        .seed_with_api_key(&app)
        .await?;
    let client2 = TestClient::new(app.address, Some(&api_key2))?;
    client2.login().post(&user2).await?;
    let mut modified_paste = paste.clone();
    modified_paste.filename = random_filename(1..=30)?;

    let response = client2
        .username(&user1.username)
        .put_by_paste_id(&modified_paste)
        .await?;
    assert_eq!(response.status(), 403);

    let response = client1.api_pastes().get_by_id(&paste).await?;
    assert_eq!(response.status(), 200);
    let persisted_paste: TestPaste = response.json().await?;
    assert_eq!(paste, persisted_paste);
    Ok(())
}

#[tokio::test]
async fn destroy_requires_a_session() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (user, api_key) = TestUser::builder()
        .random()?
        .build()
        .seed_with_api_key(&app)
        .await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
    let paste = TestPaste::builder()
        .random()?
        .build()
        .seed(&app, &user)
        .await?;

    let response = client
        .username(&user.username)
        .delete_by_paste_id(&paste)
        .await?;
    assert_eq!(response.status(), 401);

    let response = client.api_pastes().get_by_id(&paste).await?;
    assert_eq!(response.status(), 200);
    Ok(())
}

#[tokio::test]
async fn destroy_responds_with_404_when_paste_doesnt_exist() -> Result<()> {
    let app = TestApp::spawn().await?;
    let user = TestUser::builder().random()?.build().seed(&app).await?;
    let client = TestClient::new(app.address, None)?;
    client.login().post(&user).await?;
    let paste = TestPaste::builder().random()?.random_id().build();

    let response = client
        .username(&user.username)
        .delete_by_paste_id(&paste)
        .await?;
    assert_eq!(response.status(), 404);
    Ok(())
}

#[tokio::test]
async fn cannot_destroy_other_users_pastes() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (user1, api_key1) = TestUser::builder()
        .random()?
        .build()
        .seed_with_api_key(&app)
        .await?;
    let (user2, api_key2) = TestUser::builder()
        .random()?
        .build()
        .seed_with_api_key(&app)
        .await?;
    let client1 = TestClient::new(app.address, Some(&api_key1))?;
    let client2 = TestClient::new(app.address, Some(&api_key2))?;
    client2.login().post(&user2).await?;
    let paste = TestPaste::builder()
        .random()?
        .build()
        .seed(&app, &user1)
        .await?;

    let response = client2
        .username(&user1.username)
        .delete_by_paste_id(&paste)
        .await?;
    assert_eq!(response.status(), 403);

    let response = client1.api_pastes().get_by_id(&paste).await?;
    assert_eq!(response.status(), 200);
    Ok(())
}
