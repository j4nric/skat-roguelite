//! `stammtisch-server`: the account and save API (ADR 0021).
//!
//! Configuration by environment:
//!
//! | Variable | Default | Meaning |
//! |---|---|---|
//! | `STAMMTISCH_ORIGIN` | `http://localhost:5173,http://localhost:4173` | Site origin(s), comma-separated; the first is the main one |
//! | `STAMMTISCH_APP_ORIGIN` | none | Origins of the native app, comma-separated (`capacitor://localhost`, ADR 0026): CORS, bearer token instead of cookie |
//! | `STAMMTISCH_RP_ID` | one per host | WebAuthn relying party id for every origin; unset, each host of the origins is its own (old and new address, ADR 0025) |
//! | `STAMMTISCH_DB` | `stammtisch.db` | SQLite file (`:memory:` for a throwaway) |
//! | `STAMMTISCH_BACKUP_DIR` | `backup` next to the database | Daily copies (`VACUUM INTO`), the last 7 stay; `off` disables |
//! | `STAMMTISCH_LISTEN` | `127.0.0.1:8787` | Address to listen on |
//! | `RUST_LOG` | `info` | Log filter |
//!
//! `stammtisch-server reports [--since T] [--id B-XXXX] [--limit N]` does
//! not serve: it prints the stored bug reports (ADR 0027) as JSON to
//! stdout, read-only from `STAMMTISCH_DB`, the oldest first. `T` is
//! `JJJJ-MM-TT`, `JJJJ-MM-TTTHH:MM` (UTC), `7d` or `24h`; without
//! `--since` and `--id` the last 7 days.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use stammtisch_server::{AppState, Backup, Config, auth, db, report, router, spawn_housekeeping};

/// Daily backups kept.
const BACKUPS_KEPT: usize = 7;

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key)
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| default.to_owned())
}

/// Default number of reports the export prints at most.
const REPORTS_LIMIT: usize = 200;

/// `stammtisch-server reports …`: print the bug reports as JSON.
fn reports_command(args: &[String]) -> Result<(), String> {
    let mut since = None;
    let mut id = None;
    let mut limit = REPORTS_LIMIT;
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        let mut value = || it.next().map(String::as_str).filter(|v| !v.is_empty());
        match arg.as_str() {
            "--since" => {
                if let Some(v) = value() {
                    since = Some(
                        report::parse_since(v, db::now_ms())
                            .ok_or_else(|| format!("bad --since {v}"))?,
                    );
                }
            }
            "--id" => {
                if let Some(v) = value() {
                    let v = v.trim().to_ascii_uppercase();
                    if !report::is_id(&v) {
                        return Err(format!("bad --id {v}"));
                    }
                    id = Some(v);
                }
            }
            "--limit" => {
                let v = value().unwrap_or_default();
                limit = v.parse().map_err(|_| format!("bad --limit {v}"))?;
            }
            other => return Err(format!("unknown argument {other}")),
        }
    }
    if since.is_none() && id.is_none() {
        since = report::parse_since("7d", db::now_ms());
    }
    let db_path = env_or("STAMMTISCH_DB", "stammtisch.db");
    let conn =
        db::open_read_only(Path::new(&db_path)).map_err(|e| format!("database {db_path}: {e}"))?;
    let json =
        report::export(&conn, since, id.as_deref(), limit).map_err(|e| format!("reports: {e}"))?;
    let text = serde_json::to_string_pretty(&json).map_err(|e| format!("json: {e}"))?;
    println!("{text}");
    Ok(())
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("reports") {
        // No tracing here: stdout is the JSON.
        if let Err(err) = reports_command(&args[1..]) {
            eprintln!("{err}");
            std::process::exit(2);
        }
        return;
    }
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
    let app_origins = env_or("STAMMTISCH_APP_ORIGIN", "");
    let app_origins: Vec<&str> = app_origins.split(',').collect();
    let config = Config::new(&origins, rp_id.as_deref())?.with_app_origins(&app_origins)?;
    let db_path = env_or("STAMMTISCH_DB", "stammtisch.db");
    let conn = if db_path == ":memory:" {
        db::open_in_memory()
    } else {
        db::open(Path::new(&db_path))
    }
    .map_err(|e| format!("database {db_path}: {e}"))?;
    tracing::info!(
        rp_ids = ?config.rps.iter().map(|rp| rp.id.as_str()).collect::<Vec<_>>(),
        origins = ?config.origins.iter().map(|o| o.as_str()).collect::<Vec<_>>(),
        app_origins = ?config.app_origins,
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
