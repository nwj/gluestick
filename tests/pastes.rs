use reqwest::StatusCode;

mod common;

#[tokio::test]
async fn pastes_new_responds_with_200() {
    let app = common::spawn_app().await;
    let response = reqwest::get(format!("http://{}/", app.address))
        .await
        .expect("Failed to send test request.");

    assert!(response.status().is_success());
}

#[tokio::test]
async fn pastes_create_responds_with_200_for_valid_form_data() {
    let app = common::spawn_app().await;
    let client = reqwest::Client::new();
    let response = client
        .post(format!("http://{}/pastes", app.address))
        .form(&[
            ("title", "Paste"),
            ("description", "description"),
            ("body", "body"),
        ])
        .send()
        .await
        .expect("Failed to send test request.");

    assert_eq!(response.status(), StatusCode::OK)
}

#[tokio::test]
async fn pastes_create_responds_with_422_when_data_missing() {
    let app = common::spawn_app().await;
    let client = reqwest::Client::new();
    let cases = vec![
        [("title", "Paste"), ("description", "A paste.")],
        [("description", "A paste."), ("body", "A paste body.")],
        [("title", "Paste"), ("body", "A paste body.")],
    ];

    for invalid_body in cases {
        let response = client
            .post(format!("http://{}/pastes", app.address))
            .form(&invalid_body)
            .send()
            .await
            .expect("Failed to send test request.");

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY)
    }
}

#[tokio::test]
async fn pastes_index_responds_with_200() {
    let app = common::spawn_app().await;
    let response = reqwest::get(format!("http://{}/pastes", app.address))
        .await
        .expect("Failed to send test request.");

    assert!(response.status().is_success());
}

#[tokio::test]
async fn pastes_index_lists_all_pastes() {
    let app = common::spawn_app().await;
    let client = reqwest::Client::new();

    let paste1 = "Paste 1";
    let paste2 = "Paste 2";

    client
        .post(format!("http://{}/pastes", app.address))
        .form(&[
            ("title", paste1),
            ("description", "description"),
            ("body", "body"),
        ])
        .send()
        .await
        .expect("Failed to send test request.");

    client
        .post(format!("http://{}/pastes", app.address))
        .form(&[
            ("title", paste2),
            ("description", "description"),
            ("body", "body"),
        ])
        .send()
        .await
        .expect("Failed to send test request.");

    let response = client
        .get(format!("http://{}/pastes", app.address))
        .send()
        .await
        .expect("Failed to send test request.");
    let body = response.text().await.unwrap();

    assert!(body.contains(paste1));
    assert!(body.contains(paste2));
}
