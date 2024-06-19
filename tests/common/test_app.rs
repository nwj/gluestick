use crate::common::rand_helper;
use crate::common::user_helper::TestUser;
use crate::prelude::*;
use core::net::SocketAddr;
use gluestick::{db::migrations, db::Database, router};
use once_cell::sync::Lazy;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use tokio::net::TcpListener;
use tokio_rusqlite::{named_params, Connection};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

static INIT_TRACING: Lazy<()> = Lazy::new(|| {
    if std::env::var("GLUESTICK_TEST_LOG").is_ok() {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_env("GLUESTICK_TEST_LOG")
                    .unwrap_or_else(|_| "info".into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();
    }
});

pub struct TestApp {
    pub address: SocketAddr,
    pub db: Database,
}

impl TestApp {
    pub async fn spawn() -> Self {
        Lazy::force(&INIT_TRACING);

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
        let address = listener.local_addr().unwrap();

        let db_clone = db.clone();
        tokio::spawn(async move {
            axum::serve(listener, router(db_clone))
                .await
                .expect("Failed to serve test server.")
        });

        Self { address, db }
    }

    pub async fn session_and_api_authenticated_client(&self) -> Result<Client> {
        let client = Client::builder().cookie_store(true).build()?;
        let invite_code = self.seed_random_invite_code().await?;
        let user = TestUser::builder().random()?.build();
        user.signup(self, &client, invite_code).await?;

        let response = user.generate_api_key(self, &client).await?;
        let body = response.text().await?;
        let api_key = body
            .find("<pre><code>")
            .and_then(|start| {
                body[start + 11..]
                    .find("</code></pre>")
                    .map(|end| (start, end))
            })
            .map(|(start, end)| body[start + 11..start + 11 + end].to_string())
            .ok_or("Failed to parse api key")?;

        let mut headers = HeaderMap::new();
        headers.insert("X-GLUESTICK-API-KEY", HeaderValue::from_str(&api_key)?);
        let client = Client::builder()
            .cookie_store(true)
            .default_headers(headers)
            .build()?;
        user.login(self, &client).await?;
        Ok(client)
    }

    pub async fn session_authenticated_client(&self) -> Result<Client> {
        let client = Client::builder().cookie_store(true).build()?;
        let invite_code = self.seed_random_invite_code().await?;
        let user = TestUser::builder().random()?.build();
        user.signup(self, &client, invite_code).await?;
        Ok(client)
    }

    pub async fn seed_invite_code(&self, invite_code: String) -> Result<()> {
        self.db
            .conn
            .call(move |conn| {
                let mut stmt = conn.prepare("INSERT INTO invite_codes VALUES(:invite_code);")?;
                stmt.execute(named_params! {":invite_code": invite_code})?;
                Ok(())
            })
            .await?;
        Ok(())
    }

    pub async fn seed_random_invite_code(&self) -> Result<String> {
        let invite_code = rand_helper::random_string(8..=8)?;
        self.seed_invite_code(invite_code.clone()).await?;
        Ok(invite_code)
    }
}
