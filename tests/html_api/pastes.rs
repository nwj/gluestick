use crate::common::app::TestApp;
use crate::common::client::TestClient;
use crate::common::paste_helper::TestPaste;
use crate::common::user_helper::TestUser;
use crate::prelude::*;
use reqwest::StatusCode;

#[tokio::test]
async fn create_persists_when_valid_form_data() -> Result<()> {
    let app = TestApp::spawn().await?;
    let user = TestUser::builder().random()?.build().seed(&app).await?;
    let client = TestClient::new(app.address, None)?;
    client.login().post(&user).await?;
    let paste = TestPaste::builder().random()?.build();

    let response = client.pastes().post(&paste).await?;

    assert_eq!(response.status(), StatusCode::OK);
    let persisted = app
        .db
        .conn
        .call(|conn| {
            Ok(conn
                .prepare("SELECT filename, description, body FROM pastes")?
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?
                .collect::<std::result::Result<Vec<(String, String, String)>, _>>())
        })
        .await??;
    assert_eq!(persisted.len(), 1);
    assert_eq!(persisted[0].0, paste.filename);
    assert_eq!(persisted[0].1, paste.description);
    assert_eq!(persisted[0].2, paste.body);
    Ok(())
}

#[tokio::test]
async fn create_responds_with_400_when_data_missing() -> Result<()> {
    let app = TestApp::spawn().await?;
    let user = TestUser::builder().random()?.build().seed(&app).await?;
    let client = TestClient::new(app.address, None)?;
    client.login().post(&user).await?;
    let bad_pastes = vec![
        TestPaste::builder().filename("").build(),
        TestPaste::builder().body("").build(),
    ];

    for bad_paste in bad_pastes {
        let response = client.pastes().post(&bad_paste).await?;
        assert_eq!(response.status(), 400);
    }
    Ok(())
}
