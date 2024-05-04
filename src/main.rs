use dotenvy::dotenv;
use gluestick::{app, config, db};
use tokio::net::TcpListener;
use tracing::debug;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    // We won't use .env files in production, so only compile this in non-release builds
    #[cfg(debug_assertions)]
    dotenv()
        .map_err(|e| {
            if e.not_found() {
                debug!("no .env file found, continuing with normal execution");
            } else {
                debug!(
                    "error with .env file: {}, continuing with normal execution",
                    e
                );
            }
            e
        })
        .ok();

    let config = config::Config::parse()?;

    let mut db = db::Database::new(&config).await?;

    // Pragmas should be applied immediately after connecting to the database and outside of
    // the context of migrations, because some (e.g. `foreign keys`) need to be executed per
    // connection and some (e.g. `journal_mode`) need to be executed outside of transactions,
    // which migrations are run in.
    db.conn
        .call(|conn| {
            conn.pragma_update(None, "journal_mode", "WAL")?;
            conn.pragma_update(None, "synchronous", "NORMAL")?;
            conn.pragma_update(None, "busy_timeout", "5000")?;
            conn.pragma_update(None, "foreign_keys", "true")?;
            Ok(())
        })
        .await?;

    db::migrations().to_latest(&mut db.conn).await?;

    let app = app(db);

    let listener = TcpListener::bind(("127.0.0.1", config.port())).await?;

    debug!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler.");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler.")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}
