use crate::common::{spawn_app, test_paste::TestPaste};
use serde_rusqlite::to_params_named;
use std::collections::HashSet;

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
