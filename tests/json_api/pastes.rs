use crate::common::{spawn_app, test_paste::TestPaste};
use serde::Deserialize;
use std::collections::HashSet;
use uuid::Uuid;

#[tokio::test]
async fn pastes_index_responds_with_200() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("http://{}/api/pastes", app.address))
        .header("X-GLUESTICK-API-KEY", &app.user.api_key)
        .send()
        .await
        .expect("Failed to send test request.");

    assert_eq!(response.status(), 200);
}

#[derive(Debug, Deserialize)]
struct IndexResponse {
    pastes: Vec<TestPaste>,
}

#[tokio::test]
async fn pastes_index_responds_with_all_pastes() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let mut pastes = HashSet::new();
    pastes.insert(TestPaste::default().without_id());
    pastes.insert(TestPaste::default().without_id());
    for paste in &pastes {
        client
            .post(format!("http://{}/api/pastes", app.address))
            .header("X-GLUESTICK-API-KEY", &app.user.api_key)
            .json(paste)
            .send()
            .await
            .expect("Failed to setup test state.");
    }

    let response = client
        .get(format!("http://{}/api/pastes", app.address))
        .header("X-GLUESTICK-API-KEY", &app.user.api_key)
        .header("Content-Type", "application/json")
        .send()
        .await
        .expect("Failed to send test request.");

    let response_data: IndexResponse = response
        .json()
        .await
        .expect("Failed to parse test response.");

    let response_pastes: HashSet<TestPaste> = response_data.pastes.into_iter().collect();

    assert_eq!(
        pastes,
        response_pastes.iter().map(|p| p.without_id()).collect()
    );
}

#[tokio::test]
async fn pastes_create_responds_with_200_when_valid_input() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let paste = TestPaste::default();

    let response = client
        .post(format!("http://{}/api/pastes", app.address))
        .header("X-GLUESTICK-API-KEY", &app.user.api_key)
        .json(&paste)
        .send()
        .await
        .expect("Failed to send test request.");

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn pastes_create_persists_when_valid_input() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let paste = TestPaste::default_without_id();

    let response = client
        .post(format!("http://{}/api/pastes", app.address))
        .header("X-GLUESTICK-API-KEY", &app.user.api_key)
        .json(&paste)
        .send()
        .await
        .expect("Failed to send test request.");

    let id: Uuid = response
        .json()
        .await
        .expect("Failed to parse test response.");

    let response = client
        .get(format!("http://{}/api/pastes/{}", app.address, id))
        .header("X-GLUESTICK-API-KEY", &app.user.api_key)
        .send()
        .await
        .expect("Failed to send test request");

    let persisted_paste: TestPaste = response
        .json()
        .await
        .expect("Failed to parse test request.");

    assert_eq!(paste, persisted_paste.without_id())
}

#[tokio::test]
async fn pastes_create_responds_with_422_when_missing_input() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let bad_pastes = vec![
        (
            ("filename", Uuid::now_v7()),
            ("description", Uuid::now_v7()),
        ),
        (("description", Uuid::now_v7()), ("body", Uuid::now_v7())),
        (("filename", Uuid::now_v7()), ("body", Uuid::now_v7())),
    ];

    for bad_paste in bad_pastes {
        let response = client
            .post(format!("http://{}/api/pastes", app.address))
            .header("X-GLUESTICK-API-KEY", &app.user.api_key)
            .json(&bad_paste)
            .send()
            .await
            .expect("Failed to send test request.");
        assert_eq!(response.status(), 422)
    }
}

#[tokio::test]
async fn pastes_create_responds_with_400_when_invalid_input() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let bad_pastes = vec![
        "{\"filename\":\"\",\"description\":\"A description.\",\"body\":\"A body.\",\"visibility\":\"public\"}",
        "{\"filename\":\"A filename\",\"description\":\"A description.\",\"body\":\"\",\"visibility\":\"public\"}",
        "{\"filename\":\" \",\"description\":\"A description.\",\"body\":\"A body.\",\"visibility\":\"public\"}",
        "{\"filename\":\"A filename\",\"description\":\"A description.\",\"body\":\" \",\"visibility\":\"public\"}",
    ];

    // Reqwest's .json strips out empty fields, so we set the json header manually and pass in raw
    // json strings for the payload.
    for bad_paste in bad_pastes {
        let response = client
            .post(format!("http://{}/api/pastes", app.address))
            .header("Content-Type", "application/json")
            .header("X-GLUESTICK-API-KEY", &app.user.api_key)
            .body(bad_paste)
            .send()
            .await
            .expect("Failed to send test request.");
        assert_eq!(response.status(), 400)
    }
}

#[tokio::test]
async fn pastes_show_responds_with_200_when_valid_input() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let paste = TestPaste::default_without_id();
    let response = client
        .post(format!("http://{}/api/pastes", app.address))
        .header("X-GLUESTICK-API-KEY", &app.user.api_key)
        .json(&paste)
        .send()
        .await
        .unwrap();
    let paste_id: Uuid = response.json().await.unwrap();

    let response = client
        .get(format!("http://{}/api/pastes/{}", app.address, paste_id))
        .header("X-GLUESTICK-API-KEY", &app.user.api_key)
        .send()
        .await
        .expect("Failed to send test request.");

    assert_eq!(response.status(), 200)
}

#[tokio::test]
async fn pastes_show_responds_with_requested_paste_when_valid_input() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let paste = TestPaste::default_without_id();
    let response = client
        .post(format!("http://{}/api/pastes", app.address))
        .header("X-GLUESTICK-API-KEY", &app.user.api_key)
        .json(&paste)
        .send()
        .await
        .unwrap();
    let paste_id: Uuid = response.json().await.unwrap();

    let response = client
        .get(format!("http://{}/api/pastes/{}", app.address, paste_id))
        .header("X-GLUESTICK-API-KEY", &app.user.api_key)
        .send()
        .await
        .expect("Failed to send test request.");

    let response_paste: TestPaste = response
        .json()
        .await
        .expect("Failed to parse test request.");

    assert_eq!(paste, response_paste.without_id())
}

#[tokio::test]
async fn pastes_show_responds_with_404_when_requested_paste_doesnt_exist() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!(
            "http://{}/api/pastes/{}",
            app.address,
            Uuid::now_v7()
        ))
        .header("X-GLUESTICK-API-KEY", &app.user.api_key)
        .send()
        .await
        .expect("Failed to send test request.");

    assert_eq!(response.status(), 404)
}

#[tokio::test]
async fn pastes_show_responds_with_400_when_invalid_input() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("http://{}/api/pastes/some-nonsense", app.address))
        .header("X-GLUESTICK-API-KEY", &app.user.api_key)
        .send()
        .await
        .expect("Failed to send test request.");

    assert_eq!(response.status(), 400)
}

#[tokio::test]
async fn pastes_destroy_responds_with_200_when_valid_input() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let paste = TestPaste::default_without_id();
    let response = client
        .post(format!("http://{}/api/pastes", app.address))
        .header("X-GLUESTICK-API-KEY", &app.user.api_key)
        .json(&paste)
        .send()
        .await
        .unwrap();
    let paste_id: Uuid = response.json().await.unwrap();

    let response = client
        .delete(format!("http://{}/api/pastes/{}", app.address, paste_id))
        .header("X-GLUESTICK-API-KEY", &app.user.api_key)
        .send()
        .await
        .expect("failed to send test request.");

    assert_eq!(response.status(), 200)
}

#[tokio::test]
async fn pastes_destroy_deletes_requested_paste() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let paste = TestPaste::default_without_id();
    let response = client
        .post(format!("http://{}/api/pastes", app.address))
        .header("X-GLUESTICK-API-KEY", &app.user.api_key)
        .json(&paste)
        .send()
        .await
        .unwrap();
    let paste_id: Uuid = response.json().await.unwrap();

    client
        .delete(format!("http://{}/api/pastes/{}", app.address, paste_id))
        .header("X-GLUESTICK-API-KEY", &app.user.api_key)
        .send()
        .await
        .expect("failed to send test request.");

    let response = client
        .get(format!("http://{}/api/pastes/{}", app.address, paste_id))
        .header("X-GLUESTICK-API-KEY", &app.user.api_key)
        .send()
        .await
        .expect("failed to send test request.");

    assert_eq!(response.status(), 404)
}

#[tokio::test]
async fn pastes_destroy_responds_with_404_when_requested_paste_doesnt_exist() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .delete(format!(
            "http://{}/api/pastes/{}",
            app.address,
            Uuid::now_v7(),
        ))
        .header("X-GLUESTICK-API-KEY", &app.user.api_key)
        .send()
        .await
        .expect("failed to send test request.");

    assert_eq!(response.status(), 404)
}

#[tokio::test]
async fn pastes_destroy_responds_with_400_when_invalid_input() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .delete(format!("http://{}/api/pastes/some-nonsense", app.address,))
        .header("X-GLUESTICK-API-KEY", &app.user.api_key)
        .send()
        .await
        .expect("Failed to send test request.");

    assert_eq!(response.status(), 400)
}
