use crate::common::app::TestApp;
use crate::common::client::TestClient;
use crate::common::pagination_helper::{PaginationParams, PaginationResponse};
use crate::common::paste_helper::TestPaste;
use crate::common::rand_helper::{random_alphanumeric_string, random_filename, random_string};
use crate::prelude::*;
use serde::Deserialize;
use std::collections::HashSet;
use std::thread::sleep;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct IndexResponse {
    pastes: Vec<TestPaste>,
    pagination: PaginationResponse,
}

#[tokio::test]
async fn index_happy_path() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (_, api_key) = app.seed_random_user_and_api_key().await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
    let paste1 = TestPaste::builder().build().persist(&client).await?;
    let paste2 = TestPaste::builder().build().persist(&client).await?;
    let pastes = HashSet::from([paste1, paste2]);

    let response = client.api_pastes().get(None).await?;
    assert_eq!(response.status(), 200);
    let response_data: IndexResponse = response.json().await?;
    let response_pastes: HashSet<TestPaste> = response_data.pastes.into_iter().collect();
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
    let (_, api_key) = app.seed_random_user_and_api_key().await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
    let paste1 = TestPaste::builder().build().persist(&client).await?;
    TestPaste::builder()
        .visibility("secret")
        .build()
        .persist(&client)
        .await?;

    let response = client.api_pastes().get(None).await?;
    let response_data: IndexResponse = response.json().await?;
    assert_eq!(vec![paste1], response_data.pastes);
    Ok(())
}

#[tokio::test]
async fn index_has_per_page_default() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (_, api_key) = app.seed_random_user_and_api_key().await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
    for _ in 0..11 {
        TestPaste::builder()
            .random()?
            .build()
            .persist(&client)
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
    let (_, api_key) = app.seed_random_user_and_api_key().await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
    let per_page = 3;
    for _ in 0..per_page + 1 {
        TestPaste::builder()
            .random()?
            .build()
            .persist(&client)
            .await?;
    }

    let params = PaginationParams::builder().per_page(per_page).build();
    let response = client.api_pastes().get(Some(params)).await?;
    assert_eq!(response.status(), 200);
    let response_data: IndexResponse = response.json().await?;
    assert_eq!(response_data.pastes.len(), per_page);

    Ok(())
}

#[tokio::test]
async fn index_uses_default_if_per_page_more_than_100() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (_, api_key) = app.seed_random_user_and_api_key().await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
    let per_page = 101;
    for _ in 0..11 {
        TestPaste::builder()
            .random()?
            .build()
            .persist(&client)
            .await?;
    }

    let params = PaginationParams::builder().per_page(per_page).build();
    let response = client.api_pastes().get(Some(params)).await?;
    assert_eq!(response.status(), 200);
    let response_data: IndexResponse = response.json().await?;
    assert_eq!(response_data.pastes.len(), 10);

    Ok(())
}

#[tokio::test]
async fn index_paginates_correctly() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (_, api_key) = app.seed_random_user_and_api_key().await?;
    let client = TestClient::new(app.address, Some(&api_key))?;

    let mut pastes = Vec::new();
    for i in 0..8 {
        let paste = TestPaste::builder()
            .filename(i.to_string())
            .description(i.to_string())
            .body(i.to_string())
            .build()
            .persist(&client)
            .await?;
        pastes.push(paste);

        // This is necessary because we assert below on the order of items within and across pages.
        // That order is based on uuid v7 ordering, which has millisecond precision. Our tests are
        // so fast at creating these pastes that without this sleep, we can get multiple pastes
        // with the same millisecond of creation, which then fails the ordering assertions.
        sleep(Duration::from_millis(1));
    }

    // First page
    let params = PaginationParams::builder().per_page(3).build();
    let response = client.api_pastes().get(Some(params)).await?;
    let response_data: IndexResponse = response.json().await?;
    let expected: Vec<TestPaste> = pastes[5..8].into_iter().cloned().rev().collect();
    assert!(response_data.pagination.prev_page.is_none());
    assert!(response_data.pagination.next_page.is_some());
    assert_eq!(expected, response_data.pastes);

    // Second page (forward)
    let next_cursor = response_data.pagination.next_page.unwrap();
    let params = PaginationParams::builder()
        .per_page(3)
        .next_page(&next_cursor)
        .build();
    let response = client.api_pastes().get(Some(params)).await?;
    let response_data: IndexResponse = response.json().await?;
    let expected: Vec<TestPaste> = pastes[2..5].into_iter().cloned().rev().collect();
    assert!(response_data.pagination.prev_page.is_some());
    assert!(response_data.pagination.next_page.is_some());
    assert_eq!(expected, response_data.pastes);

    // Third page (forward)
    let next_cursor = response_data.pagination.next_page.unwrap();
    let params = PaginationParams::builder()
        .per_page(3)
        .next_page(&next_cursor)
        .build();
    let response = client.api_pastes().get(Some(params)).await?;
    let response_data: IndexResponse = response.json().await?;
    let expected: Vec<TestPaste> = pastes[0..2].into_iter().cloned().rev().collect();
    assert!(response_data.pagination.prev_page.is_some());
    assert!(response_data.pagination.next_page.is_none());
    assert_eq!(expected, response_data.pastes);

    // Second page (backward)
    let prev_cursor = response_data.pagination.prev_page.unwrap();
    let params = PaginationParams::builder()
        .per_page(3)
        .prev_page(&prev_cursor)
        .build();
    let response = client.api_pastes().get(Some(params)).await?;
    let response_data: IndexResponse = response.json().await?;
    let expected: Vec<TestPaste> = pastes[2..5].into_iter().cloned().rev().collect();
    assert!(response_data.pagination.prev_page.is_some());
    assert!(response_data.pagination.next_page.is_some());
    assert_eq!(expected, response_data.pastes);

    // First page (backward)
    let prev_cursor = response_data.pagination.prev_page.unwrap();
    let params = PaginationParams::builder()
        .per_page(3)
        .prev_page(&prev_cursor)
        .build();
    let response = client.api_pastes().get(Some(params)).await?;
    let response_data: IndexResponse = response.json().await?;
    let expected: Vec<TestPaste> = pastes[5..8].into_iter().cloned().rev().collect();
    assert!(response_data.pagination.prev_page.is_none());
    assert!(response_data.pagination.next_page.is_some());
    assert_eq!(expected, response_data.pastes);
    Ok(())
}

#[tokio::test]
async fn create_show_update_destroy_happy_path() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (_, api_key) = app.seed_random_user_and_api_key().await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
    let mut paste = TestPaste::builder().random()?.build();

    // Create
    let response = client.api_pastes().post(&paste).await?;
    assert_eq!(response.status(), 200);
    paste.id = response.json().await?;

    // Show
    let response = client.api_pastes().get_by_id(&paste).await?;
    assert_eq!(response.status(), 200);
    let persisted_paste: TestPaste = response.json().await?;
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
    let persisted_paste: TestPaste = response.json().await?;
    assert_eq!(paste, persisted_paste);

    // Delete
    let response = client.api_pastes().delete_by_id(&paste).await?;
    assert_eq!(response.status(), 200);

    // SHow
    let response = client.api_pastes().get_by_id(&paste).await?;
    assert_eq!(response.status(), 404);
    Ok(())
}

#[tokio::test]
async fn create_requires_an_api_key() -> Result<()> {
    let app = TestApp::spawn().await?;
    let client = TestClient::new(app.address, None)?;
    let paste = TestPaste::builder().random()?.build();

    let response = client.api_pastes().post(&paste).await?;
    assert_eq!(response.status(), 401);
    Ok(())
}

#[tokio::test]
async fn create_responds_with_400_when_missing_required_fields() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (_, api_key) = app.seed_random_user_and_api_key().await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
    let bad_pastes = vec![
        TestPaste::builder().filename("").build(),
        TestPaste::builder().body("").build(),
    ];

    for bad_paste in bad_pastes {
        let response = client.api_pastes().post(&bad_paste).await?;
        assert_eq!(response.status(), 400)
    }
    Ok(())
}

#[tokio::test]
async fn create_responds_with_400_when_invalid_fields() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (_, api_key) = app.seed_random_user_and_api_key().await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
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
        let response = client.api_pastes().post(&bad_paste).await?;
        assert_eq!(response.status(), 400)
    }
    Ok(())
}

#[tokio::test]
async fn show_requires_an_api_key() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (_, api_key) = app.seed_random_user_and_api_key().await?;
    let authed_client = TestClient::new(app.address, Some(&api_key))?;
    let client = TestClient::new(app.address, None)?;
    let paste = TestPaste::builder()
        .random()?
        .build()
        .persist(&authed_client)
        .await?;

    let response = client.api_pastes().get_by_id(&paste).await?;
    assert_eq!(response.status(), 401);
    Ok(())
}

#[tokio::test]
async fn show_responds_with_404_when_paste_doesnt_exist() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (_, api_key) = app.seed_random_user_and_api_key().await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
    let paste = TestPaste::builder().random()?.random_id().build();

    let response = client.api_pastes().get_by_id(&paste).await?;
    assert_eq!(response.status(), 404);
    Ok(())
}

#[tokio::test]
async fn show_responds_with_400_when_invalid_input() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (_, api_key) = app.seed_random_user_and_api_key().await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
    let paste = TestPaste::builder().random()?.id("garbage").build();

    let response = client.api_pastes().get_by_id(&paste).await?;
    assert_eq!(response.status(), 400);
    Ok(())
}

#[tokio::test]
async fn update_requires_an_api_key() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (_, api_key) = app.seed_random_user_and_api_key().await?;
    let authed_client = TestClient::new(app.address, Some(&api_key))?;
    let client = TestClient::new(app.address, None)?;
    let paste = TestPaste::builder()
        .random()?
        .build()
        .persist(&authed_client)
        .await?;
    let mut modified_paste = paste.clone();
    modified_paste.filename = random_filename(1..=30)?;

    let response = client.api_pastes().patch_by_id(&modified_paste).await?;
    assert_eq!(response.status(), 401);

    let response = authed_client.api_pastes().get_by_id(&paste).await?;
    assert_eq!(response.status(), 200);
    let persisted_paste: TestPaste = response.json().await?;
    assert_eq!(paste, persisted_paste);
    Ok(())
}

#[tokio::test]
async fn update_responds_with_404_when_paste_doesnt_exist() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (_, api_key) = app.seed_random_user_and_api_key().await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
    let paste = TestPaste::builder().random()?.random_id().build();

    let response = client.api_pastes().patch_by_id(&paste).await?;
    assert_eq!(response.status(), 404);
    Ok(())
}

#[tokio::test]
async fn update_responds_with_400_when_invalid_input() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (_, api_key) = app.seed_random_user_and_api_key().await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
    let paste = TestPaste::builder().random()?.id("garbage").build();

    let response = client.api_pastes().patch_by_id(&paste).await?;
    assert_eq!(response.status(), 400);
    Ok(())
}

#[tokio::test]
async fn update_responds_with_400_when_invalid_fields() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (_, api_key) = app.seed_random_user_and_api_key().await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
    let paste = TestPaste::builder()
        .random()?
        .build()
        .persist(&client)
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
        let response = client.api_pastes().patch_by_id(&bad_paste).await?;
        assert_eq!(response.status(), 400)
    }
    Ok(())
}

#[tokio::test]
async fn cannot_update_other_users_pastes() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (_, api_key1) = app.seed_random_user_and_api_key().await?;
    let (_, api_key2) = app.seed_random_user_and_api_key().await?;
    let client1 = TestClient::new(app.address, Some(&api_key1))?;
    let client2 = TestClient::new(app.address, Some(&api_key2))?;
    let paste = TestPaste::builder()
        .random()?
        .build()
        .persist(&client1)
        .await?;
    let mut modified_paste = paste.clone();
    modified_paste.filename = random_filename(1..=30)?;

    let response = client2.api_pastes().patch_by_id(&modified_paste).await?;
    assert_eq!(response.status(), 403);

    let response = client1.api_pastes().get_by_id(&paste).await?;
    assert_eq!(response.status(), 200);
    let persisted_paste: TestPaste = response.json().await?;
    assert_eq!(paste, persisted_paste);
    Ok(())
}

#[tokio::test]
async fn destroy_requires_an_api_key() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (_, api_key) = app.seed_random_user_and_api_key().await?;
    let authed_client = TestClient::new(app.address, Some(&api_key))?;
    let client = TestClient::new(app.address, None)?;
    let paste = TestPaste::builder()
        .random()?
        .build()
        .persist(&authed_client)
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
    let (_, api_key) = app.seed_random_user_and_api_key().await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
    let paste = TestPaste::builder().random()?.random_id().build();

    let response = client.api_pastes().delete_by_id(&paste).await?;
    assert_eq!(response.status(), 404);
    Ok(())
}

#[tokio::test]
async fn destroy_responds_with_400_when_invalid_input() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (_, api_key) = app.seed_random_user_and_api_key().await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
    let paste = TestPaste::builder().random()?.id("garbage").build();

    let response = client.api_pastes().delete_by_id(&paste).await?;
    assert_eq!(response.status(), 400);
    Ok(())
}

#[tokio::test]
async fn cannot_destroy_other_users_pastes() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (_, api_key1) = app.seed_random_user_and_api_key().await?;
    let (_, api_key2) = app.seed_random_user_and_api_key().await?;
    let client1 = TestClient::new(app.address, Some(&api_key1))?;
    let client2 = TestClient::new(app.address, Some(&api_key2))?;
    let paste = TestPaste::builder()
        .random()?
        .build()
        .persist(&client1)
        .await?;

    let response = client2.api_pastes().delete_by_id(&paste).await?;
    assert_eq!(response.status(), 403);

    let response = client1.api_pastes().get_by_id(&paste).await?;
    assert_eq!(response.status(), 200);
    Ok(())
}
