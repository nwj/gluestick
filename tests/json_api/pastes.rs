use crate::common::app::TestApp;
use crate::common::client::TestClient;
use crate::common::pagination_helper::{PaginationParams, PaginationResponse};
use crate::common::paste_helper::TestPaste;
use crate::prelude::*;
use serde::Deserialize;
use std::collections::HashSet;

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
async fn create_and_show_happy_path() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (_, api_key) = app.seed_random_user_and_api_key().await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
    let mut paste = TestPaste::builder().random()?.build();

    let response = client.api_pastes().post(&paste).await?;
    assert_eq!(response.status(), 200);
    paste.id = response.json().await?;

    let response = client.api_pastes().get_by_id(&paste).await?;
    assert_eq!(response.status(), 200);
    let persisted_paste: TestPaste = response.json().await?;
    assert_eq!(paste, persisted_paste);

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
async fn destroy_happy_path() -> Result<()> {
    let app = TestApp::spawn().await?;
    let (_, api_key) = app.seed_random_user_and_api_key().await?;
    let client = TestClient::new(app.address, Some(&api_key))?;
    let paste = TestPaste::builder()
        .random()?
        .build()
        .persist(&client)
        .await?;

    let response = client.api_pastes().delete_by_id(&paste).await?;
    assert_eq!(response.status(), 200);

    // Call show to confirm the paste is now gone
    let response = client.api_pastes().get_by_id(&paste).await?;
    assert_eq!(response.status(), 404);
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
