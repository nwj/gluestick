use crate::common::{spawn_app, test_paste::TestPaste};
use rusqlite::named_params;
use serde_rusqlite::to_params_named;
use std::collections::HashSet;
use uuid::Uuid;

#[tokio::test]
async fn pastes_index_responds_with_200() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("http://{}/api/pastes", app.address))
        .send()
        .await
        .expect("Failed to send test request.");

    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn pastes_index_responds_with_all_pastes() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let mut pastes = HashSet::new();
    pastes.insert(TestPaste::default());
    pastes.insert(TestPaste::default());
    pastes = app
        .db
        .conn
        .call(move |conn| {
            let mut statement =
                conn.prepare("INSERT INTO pastes VALUES (:id, :title, :description, :body);")?;
            for paste in &pastes {
                statement.execute(to_params_named(&paste).unwrap().to_slice().as_slice())?;
            }
            Ok(pastes)
        })
        .await
        .expect("Failed to write test pastes to db.");

    let response = client
        .get(format!("http://{}/api/pastes", app.address))
        .send()
        .await
        .expect("Failed to send test request.");
    let response_pastes: HashSet<TestPaste> = response
        .json()
        .await
        .expect("Failed to parse test response.");

    assert_eq!(pastes, response_pastes);
}

#[tokio::test]
async fn pastes_create_responds_with_200_when_valid_input() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let paste = TestPaste::default();

    let response = client
        .post(format!("http://{}/api/pastes", app.address))
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
        .json(&paste)
        .send()
        .await
        .expect("Failed to send test request.");

    let id: Uuid = response
        .json()
        .await
        .expect("Failed to parse test response.");

    let persisted_paste = app
        .db
        .conn
        .call(move |conn| {
            let mut statement =
                conn.prepare("SELECT id, title, description, body FROM pastes WHERE id = :id;")?;
            let mut rows = statement.query(named_params! {":id": id})?;
            match rows.next()? {
                Some(row) => Ok(Some(
                    serde_rusqlite::from_row(row)
                        .map_err(|e| tokio_rusqlite::Error::Other(Box::new(e)))?,
                )),
                None => Ok(None),
            }
        })
        .await
        .expect("Failed to read test paste from db.")
        .unwrap();

    assert!(paste.compare_without_ids(persisted_paste))
}

#[tokio::test]
async fn pastes_create_responds_with_422_when_missing_input() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let bad_pastes = vec![
        (("title", Uuid::now_v7()), ("description", Uuid::now_v7())),
        (("description", Uuid::now_v7()), ("body", Uuid::now_v7())),
        (("title", Uuid::now_v7()), ("body", Uuid::now_v7())),
    ];

    for bad_paste in bad_pastes {
        let response = client
            .post(format!("http://{}/api/pastes", app.address))
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
        "{\"title\":\"\",\"description\":\"A description.\",\"body\":\"A body.\"}",
        "{\"title\":\"A title\",\"description\":\"\",\"body\":\"A body.\"}",
        "{\"title\":\"A title\",\"description\":\"A description.\",\"body\":\"\"}",
        "{\"title\":\" \",\"description\":\"A description.\",\"body\":\"A body.\"}",
        "{\"title\":\"A title\",\"description\":\" \",\"body\":\"A body.\"}",
        "{\"title\":\"A title\",\"description\":\"A description.\",\"body\":\" \"}",
    ];

    // Reqwest's .json strips out empty fields, so we set the json header manually and pass in raw
    // json strings for the payload.
    for bad_paste in bad_pastes {
        let response = client
            .post(format!("http://{}/api/pastes", app.address))
            .header("Content-Type", "application/json")
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
        .json(&paste)
        .send()
        .await
        .unwrap();
    let paste_id: Uuid = response.json().await.unwrap();

    let response = client
        .get(format!("http://{}/api/pastes/{}", app.address, paste_id))
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
        .json(&paste)
        .send()
        .await
        .unwrap();
    let paste_id: Uuid = response.json().await.unwrap();

    let response = client
        .get(format!("http://{}/api/pastes/{}", app.address, paste_id))
        .send()
        .await
        .expect("Failed to send test request.");

    let response_paste: TestPaste = response
        .json()
        .await
        .expect("Failed to parse test request.");

    assert!(paste.compare_without_ids(response_paste))
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
        .json(&paste)
        .send()
        .await
        .unwrap();
    let paste_id: Uuid = response.json().await.unwrap();

    let response = client
        .delete(format!("http://{}/api/pastes/{}", app.address, paste_id))
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
        .json(&paste)
        .send()
        .await
        .unwrap();
    let paste_id: Uuid = response.json().await.unwrap();

    client
        .delete(format!("http://{}/api/pastes/{}", app.address, paste_id))
        .send()
        .await
        .expect("failed to send test request.");

    let response = client
        .get(format!("http://{}/api/pastes/{}", app.address, paste_id))
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
        .send()
        .await
        .expect("Failed to send test request.");

    assert_eq!(response.status(), 400)
}
