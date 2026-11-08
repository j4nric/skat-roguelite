//! `stammtisch-server`: the account and save API (ADR 0021).
//!
//! Configuration by environment:
//!
//! | Variable | Default | Meaning |
//! |---|---|---|
//! | `STAMMTISCH_ORIGIN` | `http://localhost:5173,http://localhost:4173` | Site origin(s), comma-separated; the first is the main one |
//! | `STAMMTISCH_RP_ID` | host of the first origin | WebAuthn relying party id |
//! | `STAMMTISCH_DB` | `stammtisch.db` | SQLite file (`:memory:` for a throwaway) |
//! | `STAMMTISCH_LISTEN` | `127.0.0.1:8787` | Address to listen on |
//! | `RUST_LOG` | `info` | Log filter |

use std::net::SocketAddr;
use std::path::Path;

use stammtisch_server::{AppState, Config, db, router, spawn_housekeeping};

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key)
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| default.to_owned())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        // Plain text in `docker compose logs`, colours only on a terminal.
        .with_ansi(std::io::IsTerminal::is_terminal(&std::io::stdout()))
        .init();
    if let Err(err) = run().await {
        tracing::error!("{err}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), String> {
    let origins = env_or(
        "STAMMTISCH_ORIGIN",
        "http://localhost:5173,http://localhost:4173",
    );
    let origins: Vec<&str> = origins
        .split(',')
        .filter(|o| !o.trim().is_empty())
        .collect();
    let rp_id = std::env::var("STAMMTISCH_RP_ID").ok();
    let config = Config::new(&origins, rp_id.as_deref())?;
    let db_path = env_or("STAMMTISCH_DB", "stammtisch.db");
    let conn = if db_path == ":memory:" {
        db::open_in_memory()
    } else {
        db::open(Path::new(&db_path))
    }
    .map_err(|e| format!("database {db_path}: {e}"))?;
    tracing::info!(
        rp_id = %config.rp_id,
        origins = ?config.origins.iter().map(|o| o.as_str()).collect::<Vec<_>>(),
        db = %db_path,
        "starting"
    );
    let state = AppState::new(config, conn)?;
    spawn_housekeeping(state.clone());
    let listen = env_or("STAMMTISCH_LISTEN", "127.0.0.1:8787");
    let listener = tokio::net::TcpListener::bind(&listen)
        .await
        .map_err(|e| format!("listen {listen}: {e}"))?;
    tracing::info!(%listen, "listening");
    axum::serve(
        listener,
        router(state).into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(async {
        let ctrl_c = tokio::signal::ctrl_c();
        #[cfg(unix)]
        {
            let mut term =
                tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                    .expect("signal handler");
            tokio::select! {
                _ = ctrl_c => {},
                _ = term.recv() => {},
            }
        }
        #[cfg(not(unix))]
        {
            let _ = ctrl_c.await;
        }
    })
    .await
    .map_err(|e| format!("serve: {e}"))
}
