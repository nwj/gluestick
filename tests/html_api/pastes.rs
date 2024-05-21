use crate::common;
use reqwest::{
    header::{HeaderMap, HeaderValue, COOKIE},
    Client, StatusCode,
};

#[tokio::test]
async fn pastes_create_responds_with_200_for_valid_form_data() {
    let app = common::spawn_app().await;
    let cookie_str = format!("session_token={}", &app.user.session_token);
    let mut headers = HeaderMap::new();
    headers.insert(COOKIE, HeaderValue::from_str(&cookie_str).unwrap());
    let client = Client::builder().default_headers(headers).build().unwrap();

    let response = client
        .post(format!("http://{}/pastes", app.address))
        .form(&[
            ("title", "Paste"),
            ("description", "description"),
            ("body", "body"),
            ("visibility", "public"),
        ])
        .send()
        .await
        .expect("Failed to send test request.");

    assert_eq!(response.status(), StatusCode::OK)
}

#[tokio::test]
async fn pastes_create_persists_when_valid_form_data() {
    let app = common::spawn_app().await;
    let cookie_str = format!("session_token={}", &app.user.session_token);
    let mut headers = HeaderMap::new();
    headers.insert(COOKIE, HeaderValue::from_str(&cookie_str).unwrap());
    let client = Client::builder().default_headers(headers).build().unwrap();

    client
        .post(format!("http://{}/pastes", app.address))
        .form(&[
            ("title", "Paste"),
            ("description", "description"),
            ("body", "body"),
            ("visibility", "public"),
        ])
        .send()
        .await
        .expect("Failed to send test request.");

    let persisted = app
        .db
        .conn
        .call(|conn| {
            Ok(conn
                .prepare("SELECT title, description, body FROM pastes")?
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
                .collect::<Result<Vec<(String, String, String)>, _>>())
        })
        .await
        .unwrap()
        .unwrap();

    assert_eq!(persisted.len(), 1);
    assert_eq!(persisted[0].0, "Paste");
    assert_eq!(persisted[0].1, "description");
    assert_eq!(persisted[0].2, "body");
}

#[tokio::test]
async fn pastes_create_responds_with_422_when_data_missing() {
    let app = common::spawn_app().await;
    let cookie_str = format!("session_token={}", &app.user.session_token);
    let mut headers = HeaderMap::new();
    headers.insert(COOKIE, HeaderValue::from_str(&cookie_str).unwrap());
    let client = Client::builder().default_headers(headers).build().unwrap();
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
    let cookie_str = format!("session_token={}", &app.user.session_token);
    let mut headers = HeaderMap::new();
    headers.insert(COOKIE, HeaderValue::from_str(&cookie_str).unwrap());
    let client = Client::builder().default_headers(headers).build().unwrap();
    let paste1 = "Paste 1";
    let paste2 = "Paste 2";

    client
        .post(format!("http://{}/pastes", app.address))
        .form(&[
            ("title", paste1),
            ("description", "description"),
            ("body", "body"),
            ("visibility", "public"),
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
            ("visibility", "public"),
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
