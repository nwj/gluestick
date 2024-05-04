mod common;

#[tokio::test]
async fn pastes_index_responds_with_200() {
    let address = common::spawn_server().await;
    let response = common::get(address, "/pastes").await;

    assert!(response.status().is_success());
}

#[tokio::test]
async fn pastes_new_responds_with_200() {
    let address = common::spawn_server().await;
    let response = common::get(address, "/").await;

    assert!(response.status().is_success());
}
