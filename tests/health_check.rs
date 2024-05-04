mod common;

#[tokio::test]
async fn health_check_responds_with_200() {
    let address = common::spawn_server().await;
    let response = common::get(address, "/health_check").await;

    assert!(response.status().is_success());
}

#[tokio::test]
async fn health_check_responds_with_zero_content() {
    let address = common::spawn_server().await;
    let response = common::get(address, "/health_check").await;

    assert_eq!(Some(0), response.content_length());
}
