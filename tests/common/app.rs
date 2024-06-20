use crate::common::rand_helper;
use crate::common::user_helper::TestUser;
use crate::prelude::*;
use core::net::SocketAddr;
use gluestick::{db::migrations, db::Database, router};
use once_cell::sync::Lazy;
use tokio::net::TcpListener;
use tokio_rusqlite::{named_params, Connection};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

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
    pub async fn spawn() -> Result<Self> {
        Lazy::force(&INIT_TRACING);

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

    pub async fn seed_user(&self, user_id: Uuid, user: TestUser) -> Result<()> {
        let hashed_password = rand_helper::hash_password(user.password)?;
        self.db.conn.call(move |conn| {
                let mut stmt = conn.prepare("INSERT INTO users VALUES(:id, :username, :email, :password);")?;
                stmt.execute(named_params! {":id": user_id, ":username": user.username, ":email": user.email.to_lowercase(), ":password": hashed_password})?;
                Ok(())
        }).await?;
        Ok(())
    }

    pub async fn seed_random_user(&self) -> Result<TestUser> {
        let user_id = Uuid::now_v7();
        let user = TestUser::builder().random()?.build();
        self.seed_user(user_id, user.clone()).await?;
        Ok(user)
    }

    pub async fn seed_api_key(&self, api_key: String, user_id: Uuid) -> Result<()> {
        let hashed_api_key = rand_helper::hash_api_key(api_key);
        self.db
            .conn
            .call(move |conn| {
                let mut stmt = conn
                    .prepare("INSERT INTO api_sessions VALUES(:api_key, :user_id, unixepoch());")?;
                stmt.execute(named_params! {":api_key": hashed_api_key, ":user_id": user_id})?;
                Ok(())
            })
            .await?;
        Ok(())
    }

    pub async fn seed_random_user_and_api_key(&self) -> Result<(TestUser, String)> {
        let user_id = Uuid::now_v7();
        let user = TestUser::builder().random()?.build();
        let api_key = rand_helper::random_api_key();
        self.seed_user(user_id, user.clone()).await?;
        self.seed_api_key(api_key.clone(), user_id).await?;
        Ok((user, api_key))
    }
}
