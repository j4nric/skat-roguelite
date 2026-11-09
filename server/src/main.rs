//! `stammtisch-server`: the account and save API (ADR 0021).
//!
//! Configuration by environment:
//!
//! | Variable | Default | Meaning |
//! |---|---|---|
//! | `STAMMTISCH_ORIGIN` | `http://localhost:5173,http://localhost:4173` | Site origin(s), comma-separated; the first is the main one |
//! | `STAMMTISCH_RP_ID` | host of the first origin | WebAuthn relying party id |
//! | `STAMMTISCH_DB` | `stammtisch.db` | SQLite file (`:memory:` for a throwaway) |
//! | `STAMMTISCH_BACKUP_DIR` | `backup` next to the database | Daily copies (`VACUUM INTO`), the last 7 stay; `off` disables |
//! | `STAMMTISCH_LISTEN` | `127.0.0.1:8787` | Address to listen on |
//! | `RUST_LOG` | `info` | Log filter |

use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use stammtisch_server::{AppState, Backup, Config, auth, db, router, spawn_housekeeping};

/// Daily backups kept.
const BACKUPS_KEPT: usize = 7;

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

/// Where the daily backup goes: `STAMMTISCH_BACKUP_DIR`, else `backup/`
/// next to the database file; none for an in-memory database or `off`.
fn backup_config(db_path: &str) -> Option<Backup> {
    if db_path == ":memory:" {
        return None;
    }
    let db = PathBuf::from(db_path);
    let dir = match std::env::var("STAMMTISCH_BACKUP_DIR") {
        Ok(v) if v.trim() == "off" => return None,
        Ok(v) if !v.trim().is_empty() => PathBuf::from(v.trim()),
        _ => db
            .parent()
            .map_or_else(|| PathBuf::from("backup"), |p| p.join("backup")),
    };
    Some(Backup {
        db,
        dir,
        keep: BACKUPS_KEPT,
    })
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
    let backup = backup_config(&db_path);
    match &backup {
        Some(b) => tracing::info!(dir = %b.dir.display(), keep = b.keep, "daily backup"),
        None => tracing::info!("no daily backup"),
    }
    let state = AppState::new(config, conn)?;
    // The dummy hash of a wrong recovery id is argon2 work: do it now, on
    // a blocking thread, not in the first request.
    tokio::task::spawn_blocking(auth::warm_up)
        .await
        .map_err(|e| format!("warm up: {e}"))?;
    spawn_housekeeping(state.clone(), backup);
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
