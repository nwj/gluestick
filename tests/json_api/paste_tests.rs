use crate::common::mocks::mock_pagination::{MockPaginationParams, MockPaginationResponse};
use crate::common::mocks::mock_paste::MockPaste;
use crate::common::mocks::mock_user::MockUser;
use crate::common::rand_helper::{random_alphanumeric_string, random_filename, random_string};
use crate::common::test_app::TestApp;
use crate::common::test_client::TestClient;
use crate::prelude::*;
use jiff::Timestamp as JiffTimestamp;
use serde::Deserialize;
use std::collections::HashSet;
use uuid::{NoContext, Timestamp, Uuid};

#[derive(Debug, Deserialize)]
struct IndexResponse {
    pastes: Vec<MockPaste>,
    pagination: MockPaginationResponse,
}

#[tokio::test]
async fn index_happy_path() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (user, api_key) = MockUser::builder()
        .random()?
        .build()
        .seed_with_api_key(&app)
        .await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
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
    let pastes = HashSet::from([paste1, paste2]);

    let response = client.api_pastes().get(None).await?;
    assert_eq!(response.status(), 200);
    let response_data: IndexResponse = response.json().await?;
    let response_pastes: HashSet<MockPaste> = response_data.pastes.into_iter().collect();
    assert_eq!(pastes, response_pastes);
    Ok(())
}

#[tokio::test]
async fn index_requires_an_api_key() -> Result<()> {
    let app = TestApp::spawn().await?;
    let client = TestClient::new(app.address, None)?;

    let response = client.api_pastes().get(None).await?;
    assert_eq!(response.status(), 401);
    Ok(())
}

#[tokio::test]
async fn index_does_not_include_secret_pastes() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (user, api_key) = MockUser::builder()
        .random()?
        .build()
        .seed_with_api_key(&app)
        .await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
    let paste1 = MockPaste::builder()
        .random()?
        .build()
        .seed(&app, &user)
        .await?;
    MockPaste::builder()
        .random()?
        .visibility("secret")
        .build()
        .seed(&app, &user)
        .await?;

    let response = client.api_pastes().get(None).await?;
    let response_data: IndexResponse = response.json().await?;
    assert_eq!(vec![paste1], response_data.pastes);
    Ok(())
}

#[tokio::test]
async fn index_has_per_page_default() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (user, api_key) = MockUser::builder()
        .random()?
        .build()
        .seed_with_api_key(&app)
        .await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
    for _ in 0..11 {
        MockPaste::builder()
            .random()?
            .build()
            .seed(&app, &user)
            .await?;
    }

    let response = client.api_pastes().get(None).await?;
    assert_eq!(response.status(), 200);
    let response_data: IndexResponse = response.json().await?;
    assert_eq!(response_data.pastes.len(), 10);
    Ok(())
}

#[tokio::test]
async fn index_uses_per_page_when_provided() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (user, api_key) = MockUser::builder()
        .random()?
        .build()
        .seed_with_api_key(&app)
        .await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
    let per_page = 3;
    for _ in 0..per_page + 1 {
        MockPaste::builder()
            .random()?
            .build()
            .seed(&app, &user)
            .await?;
    }

    let params = MockPaginationParams::builder().per_page(per_page).build();
    let response = client.api_pastes().get(Some(params)).await?;
    assert_eq!(response.status(), 200);
    let response_data: IndexResponse = response.json().await?;
    assert_eq!(response_data.pastes.len(), per_page);
    Ok(())
}

#[tokio::test]
async fn index_falls_back_to_default_if_per_page_more_than_100() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (user, api_key) = MockUser::builder()
        .random()?
        .build()
        .seed_with_api_key(&app)
        .await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
    let per_page = 101;
    for _ in 0..11 {
        MockPaste::builder()
            .random()?
            .build()
            .seed(&app, &user)
            .await?;
    }

    let params = MockPaginationParams::builder().per_page(per_page).build();
    let response = client.api_pastes().get(Some(params)).await?;
    assert_eq!(response.status(), 200);
    let response_data: IndexResponse = response.json().await?;
    assert_eq!(response_data.pastes.len(), 10);

    Ok(())
}

#[tokio::test]
async fn index_paginates_correctly() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (user, api_key) = MockUser::builder()
        .random()?
        .build()
        .seed_with_api_key(&app)
        .await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
    let now = (JiffTimestamp::now().as_millisecond()) as u64;

    let mut pastes = Vec::new();
    for i in 0..8 {
        let paste = MockPaste::builder()
            // This is necessary because we assert below on the order of items within and across pages.
            // That order is based on uuid v7 ordering, which has millisecond precision. Our tests are
            // so fast at creating these pastes that without this sleep, we can get multiple pastes
            // with the same millisecond of creation, which then fails the ordering assertions.
            .id(Uuid::new_v7(Timestamp::from_unix(NoContext, now + i, 0)))
            .filename(i.to_string())
            .description(i.to_string())
            .body(i.to_string())
            .build()
            .seed(&app, &user)
            .await?;
        pastes.push(paste);
    }

    // First page
    let params = MockPaginationParams::builder().per_page(3).build();
    let response = client.api_pastes().get(Some(params)).await?;
    let response_data: IndexResponse = response.json().await?;
    let expected: Vec<MockPaste> = pastes[5..8].into_iter().cloned().rev().collect();
    assert!(response_data.pagination.prev_page.is_none());
    assert!(response_data.pagination.next_page.is_some());
    assert_eq!(expected, response_data.pastes);

    // Second page (forward)
    let next_cursor = response_data.pagination.next_page.unwrap();
    let params = MockPaginationParams::builder()
        .per_page(3)
        .next_page(&next_cursor)
        .build();
    let response = client.api_pastes().get(Some(params)).await?;
    let response_data: IndexResponse = response.json().await?;
    let expected: Vec<MockPaste> = pastes[2..5].into_iter().cloned().rev().collect();
    assert!(response_data.pagination.prev_page.is_some());
    assert!(response_data.pagination.next_page.is_some());
    assert_eq!(expected, response_data.pastes);

    // Third page (forward)
    let next_cursor = response_data.pagination.next_page.unwrap();
    let params = MockPaginationParams::builder()
        .per_page(3)
        .next_page(&next_cursor)
        .build();
    let response = client.api_pastes().get(Some(params)).await?;
    let response_data: IndexResponse = response.json().await?;
    let expected: Vec<MockPaste> = pastes[0..2].into_iter().cloned().rev().collect();
    assert!(response_data.pagination.prev_page.is_some());
    assert!(response_data.pagination.next_page.is_none());
    assert_eq!(expected, response_data.pastes);

    // Second page (backward)
    let prev_cursor = response_data.pagination.prev_page.unwrap();
    let params = MockPaginationParams::builder()
        .per_page(3)
        .prev_page(&prev_cursor)
        .build();
    let response = client.api_pastes().get(Some(params)).await?;
    let response_data: IndexResponse = response.json().await?;
    let expected: Vec<MockPaste> = pastes[2..5].into_iter().cloned().rev().collect();
    assert!(response_data.pagination.prev_page.is_some());
    assert!(response_data.pagination.next_page.is_some());
    assert_eq!(expected, response_data.pastes);

    // First page (backward)
    let prev_cursor = response_data.pagination.prev_page.unwrap();
    let params = MockPaginationParams::builder()
        .per_page(3)
        .prev_page(&prev_cursor)
        .build();
    let response = client.api_pastes().get(Some(params)).await?;
    let response_data: IndexResponse = response.json().await?;
    let expected: Vec<MockPaste> = pastes[5..8].into_iter().cloned().rev().collect();
    assert!(response_data.pagination.prev_page.is_none());
    assert!(response_data.pagination.next_page.is_some());
    assert_eq!(expected, response_data.pastes);
    Ok(())
}

#[tokio::test]
async fn create_show_update_destroy_happy_path() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (_user, api_key) = MockUser::builder()
        .random()?
        .build()
        .seed_with_api_key(&app)
        .await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
    let mut paste = MockPaste::builder().random()?.build();

    // Create
    let response = client.api_pastes().post(&paste).await?;
    assert_eq!(response.status(), 200);
    paste.id = response.json().await?;

    // Show
    let response = client.api_pastes().get_by_id(&paste).await?;
    assert_eq!(response.status(), 200);
    let persisted_paste: MockPaste = response.json().await?;
    assert_eq!(paste, persisted_paste);

    // Update
    paste.filename = random_filename(1..=30)?;
    paste.description = random_string(1..=30)?;
    paste.body = random_string(1..=30)?;
    let response = client.api_pastes().patch_by_id(&paste).await?;
    assert_eq!(response.status(), 200);

    // Show
    let response = client.api_pastes().get_by_id(&paste).await?;
    assert_eq!(response.status(), 200);
    let persisted_paste: MockPaste = response.json().await?;
    assert_eq!(paste, persisted_paste);

    // Delete
    let response = client.api_pastes().delete_by_id(&paste).await?;
    assert_eq!(response.status(), 200);

    // Show
    let response = client.api_pastes().get_by_id(&paste).await?;
    assert_eq!(response.status(), 404);
    Ok(())
}

#[tokio::test]
async fn create_requires_an_api_key() -> Result<()> {
    let app = TestApp::spawn().await?;
    let client = TestClient::new(app.address, None)?;
    let paste = MockPaste::builder().random()?.build();

    let response = client.api_pastes().post(&paste).await?;
    assert_eq!(response.status(), 401);
    Ok(())
}

#[tokio::test]
async fn create_responds_with_400_when_missing_required_fields() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (_user, api_key) = MockUser::builder()
        .random()?
        .build()
        .seed_with_api_key(&app)
        .await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
    let bad_pastes = vec![
        MockPaste::builder().filename("").build(),
        MockPaste::builder().body("").build(),
    ];

    for bad_paste in bad_pastes {
        let response = client.api_pastes().post(&bad_paste).await?;
        assert_eq!(response.status(), 422)
    }
    Ok(())
}

#[tokio::test]
async fn create_responds_with_400_when_invalid_fields() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (_user, api_key) = MockUser::builder()
        .random()?
        .build()
        .seed_with_api_key(&app)
        .await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
    let bad_pastes = vec![
        MockPaste::builder()
            .filename(random_filename(257..=257)?)
            .build(),
        MockPaste::builder()
            .filename("illegal/characters.md")
            .build(),
        MockPaste::builder()
            .description(random_alphanumeric_string(257..=257)?)
            .build(),
    ];

    for bad_paste in bad_pastes {
        let response = client.api_pastes().post(&bad_paste).await?;
        assert_eq!(response.status(), 422)
    }
    Ok(())
}

#[tokio::test]
async fn show_requires_an_api_key() -> Result<()> {
    let app = TestApp::spawn().await?;
    let user = MockUser::builder().random()?.build().seed(&app).await?;
    let client = TestClient::new(app.address, None)?;
    let paste = MockPaste::builder()
        .random()?
        .build()
        .seed(&app, &user)
        .await?;

    let response = client.api_pastes().get_by_id(&paste).await?;
    assert_eq!(response.status(), 401);
    Ok(())
}

#[tokio::test]
async fn show_responds_with_404_when_paste_doesnt_exist() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (_user, api_key) = MockUser::builder()
        .random()?
        .build()
        .seed_with_api_key(&app)
        .await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
    let paste = MockPaste::builder().random()?.random_id().build();

    let response = client.api_pastes().get_by_id(&paste).await?;
    assert_eq!(response.status(), 404);
    Ok(())
}

#[tokio::test]
async fn show_responds_with_404_when_invalid_id() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (_user, api_key) = MockUser::builder()
        .random()?
        .build()
        .seed_with_api_key(&app)
        .await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
    let paste = MockPaste::builder().random()?.id("garbage").build();

    let response = client.api_pastes().get_by_id(&paste).await?;
    assert_eq!(response.status(), 404);
    Ok(())
}

#[tokio::test]
async fn update_requires_an_api_key() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (user, api_key) = MockUser::builder()
        .random()?
        .build()
        .seed_with_api_key(&app)
        .await?;
    let authed_client = TestClient::new(app.address, Some(&api_key))?;
    let client = TestClient::new(app.address, None)?;
    let paste = MockPaste::builder()
        .random()?
        .build()
        .seed(&app, &user)
        .await?;
    let mut modified_paste = paste.clone();
    modified_paste.filename = random_filename(1..=30)?;

    let response = client.api_pastes().patch_by_id(&modified_paste).await?;
    assert_eq!(response.status(), 401);

    let response = authed_client.api_pastes().get_by_id(&paste).await?;
    assert_eq!(response.status(), 200);
    let persisted_paste: MockPaste = response.json().await?;
    assert_eq!(paste, persisted_paste);
    Ok(())
}

#[tokio::test]
async fn update_responds_with_404_when_paste_doesnt_exist() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (_user, api_key) = MockUser::builder()
        .random()?
        .build()
        .seed_with_api_key(&app)
        .await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
    let paste = MockPaste::builder().random()?.random_id().build();

    let response = client.api_pastes().patch_by_id(&paste).await?;
    assert_eq!(response.status(), 404);
    Ok(())
}

#[tokio::test]
async fn update_responds_with_400_when_invalid_input() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (_user, api_key) = MockUser::builder()
        .random()?
        .build()
        .seed_with_api_key(&app)
        .await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
    let paste = MockPaste::builder()
        .random()?
        .filename("illegal/filename.")
        .build();

    let response = client.api_pastes().patch_by_id(&paste).await?;
    assert_eq!(response.status(), 422);
    Ok(())
}

#[tokio::test]
async fn update_responds_with_400_when_invalid_fields() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (user, api_key) = MockUser::builder()
        .random()?
        .build()
        .seed_with_api_key(&app)
        .await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
    let paste = MockPaste::builder()
        .random()?
        .build()
        .seed(&app, &user)
        .await?;
    let bad_pastes = vec![
        MockPaste::builder()
            .id(paste.id.clone().unwrap())
            .filename(random_filename(257..=257)?)
            .build(),
        MockPaste::builder()
            .id(paste.id.clone().unwrap())
            .filename("illegal/characters.md")
            .build(),
        MockPaste::builder()
            .id(paste.id.clone().unwrap())
            .description(random_alphanumeric_string(257..=257)?)
            .build(),
    ];

    for bad_paste in bad_pastes {
        let response = client.api_pastes().patch_by_id(&bad_paste).await?;
        assert_eq!(response.status(), 422)
    }
    Ok(())
}

#[tokio::test]
async fn cannot_update_other_users_pastes() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (user1, api_key1) = MockUser::builder()
        .random()?
        .build()
        .seed_with_api_key(&app)
        .await?;
    let (_user2, api_key2) = MockUser::builder()
        .random()?
        .build()
        .seed_with_api_key(&app)
        .await?;
    let client1 = TestClient::new(app.address, Some(&api_key1))?;
    let client2 = TestClient::new(app.address, Some(&api_key2))?;
    let paste = MockPaste::builder()
        .random()?
        .build()
        .seed(&app, &user1)
        .await?;
    let mut modified_paste = paste.clone();
    modified_paste.filename = random_filename(1..=30)?;

    let response = client2.api_pastes().patch_by_id(&modified_paste).await?;
    assert_eq!(response.status(), 403);

    let response = client1.api_pastes().get_by_id(&paste).await?;
    assert_eq!(response.status(), 200);
    let persisted_paste: MockPaste = response.json().await?;
    assert_eq!(paste, persisted_paste);
    Ok(())
}

#[tokio::test]
async fn cannot_update_public_paste_to_secret() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (user, api_key) = MockUser::builder()
        .random()?
        .build()
        .seed_with_api_key(&app)
        .await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
    let mut paste = MockPaste::builder()
        .random()?
        .visibility("public")
        .build()
        .seed(&app, &user)
        .await?;

    paste.visibility = "secret".into();
    let response = client.api_pastes().patch_by_id(&paste).await?;
    assert_eq!(response.status(), 422);
    Ok(())
}

#[tokio::test]
async fn can_update_secret_paste_to_public() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (user, api_key) = MockUser::builder()
        .random()?
        .build()
        .seed_with_api_key(&app)
        .await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
    let mut paste = MockPaste::builder()
        .random()?
        .visibility("secret")
        .build()
        .seed(&app, &user)
        .await?;

    paste.visibility = "public".into();
    let response = client.api_pastes().patch_by_id(&paste).await?;
    assert_eq!(response.status(), 200);
    Ok(())
}

#[tokio::test]
async fn destroy_requires_an_api_key() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (user, api_key) = MockUser::builder()
        .random()?
        .build()
        .seed_with_api_key(&app)
        .await?;
    let authed_client = TestClient::new(app.address, Some(&api_key))?;
    let client = TestClient::new(app.address, None)?;
    let paste = MockPaste::builder()
        .random()?
        .build()
        .seed(&app, &user)
        .await?;

    let response = client.api_pastes().delete_by_id(&paste).await?;
    assert_eq!(response.status(), 401);

    let response = authed_client.api_pastes().get_by_id(&paste).await?;
    assert_eq!(response.status(), 200);
    Ok(())
}

#[tokio::test]
async fn destroy_responds_with_404_when_paste_doesnt_exist() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (_user, api_key) = MockUser::builder()
        .random()?
        .build()
        .seed_with_api_key(&app)
        .await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
    let paste = MockPaste::builder().random()?.random_id().build();

    let response = client.api_pastes().delete_by_id(&paste).await?;
    assert_eq!(response.status(), 404);
    Ok(())
}

#[tokio::test]
async fn destroy_responds_with_404_when_invalid_id() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (_user, api_key) = MockUser::builder()
        .random()?
        .build()
        .seed_with_api_key(&app)
        .await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
    let paste = MockPaste::builder().random()?.id("garbage").build();

    let response = client.api_pastes().delete_by_id(&paste).await?;
    assert_eq!(response.status(), 404);
    Ok(())
}

#[tokio::test]
async fn cannot_destroy_other_users_pastes() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (user1, api_key1) = MockUser::builder()
        .random()?
        .build()
        .seed_with_api_key(&app)
        .await?;
    let (_user2, api_key2) = MockUser::builder()
        .random()?
        .build()
        .seed_with_api_key(&app)
        .await?;
    let client1 = TestClient::new(app.address, Some(&api_key1))?;
    let client2 = TestClient::new(app.address, Some(&api_key2))?;
    let paste = MockPaste::builder()
        .random()?
        .build()
        .seed(&app, &user1)
        .await?;

    let response = client2.api_pastes().delete_by_id(&paste).await?;
    assert_eq!(response.status(), 403);

    let response = client1.api_pastes().get_by_id(&paste).await?;
    assert_eq!(response.status(), 200);
    Ok(())
}
