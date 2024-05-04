mod common;

#[tokio::test]
async fn pastes_index_responds_with_200() {
    let address = common::spawn_server().await;
    let response = reqwest::get(format!("http://{}/pastes", &address))
        .await
        .expect("Failed to send test request.");

    assert!(response.status().is_success());
}

#[tokio::test]
async fn pastes_index_lists_all_pastes() {
    let address = common::spawn_server().await;
    let client = reqwest::Client::new();

    let paste1 = "Paste 1";
    let paste2 = "Paste 2";

    client
        .post(format!("http://{}/pastes", &address))
        .form(&[
            ("title", paste1),
            ("description", "description"),
            ("body", "body"),
        ])
        .send()
        .await
        .expect("Failed to send test request.");

    client
        .post(format!("http://{}/pastes", &address))
        .form(&[
            ("title", paste2),
            ("description", "description"),
            ("body", "body"),
        ])
        .send()
        .await
        .expect("Failed to send test request.");

    let response = client
        .get(format!("http://{}/pastes", &address))
        .send()
        .await
        .expect("Failed to send test request.");
    let body = response.text().await.unwrap();

    assert!(body.contains(paste1));
    assert!(body.contains(paste2));
}

#[tokio::test]
async fn pastes_new_responds_with_200() {
    let address = common::spawn_server().await;
    let response = reqwest::get(format!("http://{}/", &address))
        .await
        .expect("Failed to send test request.");

    assert!(response.status().is_success());
}
