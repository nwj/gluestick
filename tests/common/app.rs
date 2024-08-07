use crate::common::paste_helper::TestPaste;
use crate::common::rand_helper;
use crate::common::user_helper::TestUser;
use crate::prelude::*;
use core::net::SocketAddr;
use gluestick::{db::migrations, db::Database, router};
use jiff::Timestamp;
use std::sync::LazyLock;
use tokio::net::TcpListener;
use tokio_rusqlite::{named_params, Connection};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

static INIT_TRACING: LazyLock<()> = LazyLock::new(|| {
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
    pub async fn spawn() -> Result<Self> {
        LazyLock::force(&INIT_TRACING);

        let mut db = Database {
            conn: Connection::open_in_memory().await?,
        };

        db.conn
            .call(|conn| {
                conn.pragma_update(None, "journal_mode", "WAL")?;
                conn.pragma_update(None, "synchronous", "NORMAL")?;
                conn.pragma_update(None, "busy_timeout", "5000")?;
                conn.pragma_update(None, "foreign_keys", "true")?;
                Ok(())
            })
            .await?;

        migrations().to_latest(&mut db.conn).await?;

        // Binding to port 0 will cause the OS to scan for an available port which will then be used
        // for the bind. So this effectively runs the test server on a random, open port.
        let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
        let address = listener.local_addr().unwrap();

        let db_clone = db.clone();
        tokio::spawn(async move {
            axum::serve(listener, router(db_clone))
                .await
                .expect("Failed to serve test server.")
        });

        Ok(Self { address, db })
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

    pub async fn seed_user(&self, user: TestUser) -> Result<()> {
        let id = Uuid::try_parse(
            &user
                .id
                .clone()
                .unwrap_or("can't seed user without an id".into()),
        )?;
        let hashed_password = rand_helper::hash_password(user.password)?;
        let now = Timestamp::now().as_millisecond();
        self.db
            .conn
            .call(move |conn| {
                let mut stmt =
                    conn.prepare("INSERT INTO users VALUES(:id, :username, :email, :password, :created_at, :updated_at);")?;
                stmt.execute(named_params! {
                    ":id": id,
                    ":username": user.username,
                    ":email": user.email.to_lowercase(),
                    ":password": hashed_password,
                    ":created_at": now,
                    ":updated_at": now,
                })?;
                Ok(())
            })
            .await?;
        Ok(())
    }

    pub async fn seed_api_key(&self, api_key: String, user: &TestUser) -> Result<()> {
        let user_id = Uuid::try_parse(
            &user
                .id
                .clone()
                .unwrap_or("can't seed api key without a user id".into()),
        )?;
        let hashed_api_key = rand_helper::hash_api_key(api_key);
        let now = Timestamp::now().as_millisecond();
        self.db
            .conn
            .call(move |conn| {
                let mut stmt = conn.prepare(
                    "INSERT INTO api_keys VALUES(:id, :name, :key, :user_id, :created_at, :last_used_at);",
                )?;
                stmt.execute(named_params! {
                    ":id": Uuid::now_v7(),
                    ":name": "Unnamed API Key",
                    ":key": hashed_api_key,
                    ":user_id": user_id,
                    ":created_at": now,
                    ":last_used_at": now,
                })?;
                Ok(())
            })
            .await?;
        Ok(())
    }

    pub async fn seed_paste(&self, paste: TestPaste, user: &TestUser) -> Result<()> {
        let id = Uuid::try_parse(
            &paste
                .id
                .clone()
                .unwrap_or("can't seed paste without an id".into()),
        )?;
        let user_id = Uuid::try_parse(
            &user
                .id
                .clone()
                .unwrap_or("can't seed paste without a user id".into()),
        )?;
        let now = Timestamp::now().as_millisecond();
        self.db
            .conn
            .call(move |conn| {
                let mut stmt = conn
                    .prepare("INSERT INTO pastes VALUES(:id, :user_id, :filename, :description, :body, :visibility, :created_at, :updated_at);")?;
                stmt.execute(named_params! {
                    ":id": id,
                    ":user_id": user_id,
                    ":filename": paste.filename,
                    ":description": paste.description,
                    ":body": paste.body,
                    ":visibility": paste.visibility,
                    ":created_at": now,
                    ":updated_at": now,
                })?;
                Ok(())
            })
            .await?;
        Ok(())
    }
}
