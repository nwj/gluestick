use gluestick::{app, db::migrations, db::Database};
use tokio::net::TcpListener;
use tokio_rusqlite::Connection;

#[tokio::test]
async fn health_check_works() {
    let port = spawn_server().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("http://127.0.0.1:{}/health_check", port))
        .send()
        .await
        .expect("Failed to execute test request.");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

async fn spawn_server() -> u16 {
    let mut db = Database {
        conn: Connection::open_in_memory()
            .await
            .expect("Failed to establish connection with test database."),
    };

    db.conn
        .call(|conn| {
            conn.pragma_update(None, "journal_mode", "WAL")?;
            conn.pragma_update(None, "synchronous", "NORMAL")?;
            conn.pragma_update(None, "busy_timeout", "5000")?;
            conn.pragma_update(None, "foreign_keys", "true")?;
            Ok(())
        })
        .await
        .expect("Failed to set pragmas on the test database.");

    migrations()
        .to_latest(&mut db.conn)
        .await
        .expect("Failed to migrate the test database.");

    // Binding to port 0 will cause the OS to scan for an available port which will then be used
    // for the bind. So this effectively runs the test server on a random, open port.
    let listener = TcpListener::bind(("127.0.0.1", 0))
        .await
        .expect("Failed to bind test server to address.");
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        axum::serve(listener, app(db))
            .await
            .expect("Failed to serve test server.")
    });

    port
}
