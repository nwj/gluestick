use crate::prelude::*;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use core::net::SocketAddr;
use gluestick::{db::migrations, db::Database, router};
use once_cell::sync::Lazy;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use sha2::{Digest, Sha256};
use tokio::net::TcpListener;
use tokio_rusqlite::{named_params, Connection};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

pub mod paste_helper;
pub mod rand_helper;
pub mod test_paste;
pub mod user_helper;

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
    pub user: AuthenticatedTestUser,
}

impl TestApp {
    pub fn api_authenticated_client(&self) -> Result<Client> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "X-GLUESTICK-API-KEY",
            HeaderValue::from_str(&self.user.api_key)?,
        );
        let client = Client::builder().default_headers(headers).build()?;
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

pub async fn spawn_app() -> TestApp {
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

    let user = AuthenticatedTestUser::default();
    user.persist(db.clone())
        .await
        .expect("Failed to persist authenticated test user.");

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

    TestApp { address, db, user }
}

pub struct AuthenticatedTestUser {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password: String,
    pub session_token: String,
    pub api_key: String,
}

impl Default for AuthenticatedTestUser {
    fn default() -> Self {
        Self {
            id: Uuid::now_v7(),
            username: "jmanderley".to_string(),
            email: "jmanderley@unatco.gov".to_string(),
            password: "knight_killer".to_string(),
            session_token: "498e5daeec9bbaf74031926881e25a32".to_string(),
            api_key: "671778711fd3128b87d230296e29ae6e".to_string(),
        }
    }
}

impl AuthenticatedTestUser {
    async fn persist(&self, db: Database) -> std::result::Result<(), tokio_rusqlite::Error> {
        let id = self.id.clone();
        let username = self.username.clone();
        let email = self.email.clone();

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hashed_password = argon2
            .hash_password(self.password.as_bytes(), &salt)
            .unwrap()
            .to_string();
        let hashed_token = Sha256::digest(&self.session_token.as_bytes()).to_vec();
        let hashed_key = Sha256::digest(&self.api_key.as_bytes()).to_vec();

        db.conn
            .call(move |conn| {
                let mut statement = conn
                    .prepare("INSERT INTO users VALUES (:id, :username, :email, :password);")
                    .expect("Failed to persist test user");
                statement
                    .execute(named_params! {
                        ":id": id,
                        ":username": username,
                        ":email": email,
                        ":password": hashed_password
                    })
                    .expect("Failed to persist test user");

                statement = conn
                    .prepare("INSERT INTO sessions VALUES (:session_token, :user_id);")
                    .expect("Failed to persist test user session");
                statement
                    .execute(named_params! {
                        ":session_token": hashed_token,
                        ":user_id": id
                    })
                    .expect("Failed to persist test user session");

                statement = conn
                    .prepare("INSERT INTO api_sessions VALUES (:api_key, :user_id, unixepoch());")
                    .expect("Failed to persist test user api session");
                statement
                    .execute(named_params! {
                        ":api_key": hashed_key,
                        ":user_id": id
                    })
                    .expect("Failed to persist test user api session");
                Ok(())
            })
            .await
            .expect("Failed to persist test user, session, and/or api session");
        Ok(())
    }
}
