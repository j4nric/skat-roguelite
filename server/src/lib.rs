//! Account and save server of Stammtisch (ADR 0021).
//!
//! A small HTTP API under `/api` behind our Caddy: sign-up and sign-in
//! with passkeys (WebAuthn), a recovery code as the way back in, a display
//! name, one opaque save blob per player and the daily leaderboard. The
//! server never looks into the save and never checks game state: all game
//! logic stays in the browser (the Rust core as WebAssembly). It knows
//! nothing of `skat_core`.

pub mod account;
pub mod auth;
pub mod daily;
pub mod db;
pub mod error;
pub mod recovery;
pub mod session;

use axum::Router;
use axum::extract::{DefaultBodyLimit, Request, State};
use axum::http::{Method, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post, put};
use rusqlite::Connection;
use std::collections::HashMap;
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use url::Url;
use webauthn_rs::prelude::{DiscoverableAuthentication, PasskeyRegistration, Uuid};
use webauthn_rs::{Webauthn, WebauthnBuilder};

use crate::error::ApiError;

/// Largest body of `PUT /api/save` (the save blob is at most
/// [`account::MAX_SAVE_BYTES`], plus the JSON around it).
pub const MAX_BODY_BYTES: usize = 1_200_000;
/// Largest body of every other request: names, codes, daily results.
pub const SMALL_BODY_BYTES: usize = 16 * 1024;
/// Largest body of the passkey registration finish calls: an attestation
/// object can carry a certificate chain (we ask for none, but some
/// authenticators send one anyway).
pub const PASSKEY_BODY_BYTES: usize = 64 * 1024;
/// At most this many argon2 hashes run at once (each takes ~19 MiB and a
/// core for a moment); further requests wait for a permit.
pub const ARGON2_PERMITS: usize = 2;

/// Settings of the server.
#[derive(Debug, Clone)]
pub struct Config {
    /// WebAuthn relying party id: the host of the site.
    pub rp_id: String,
    /// Origins the browser may send from (the first is the main one).
    pub origins: Vec<Url>,
    /// Set `Secure` on the cookie (when the site is served over HTTPS).
    pub secure_cookie: bool,
}

impl Config {
    /// Config for these origins; the RP id defaults to the first origin's host.
    pub fn new(origins: &[&str], rp_id: Option<&str>) -> Result<Self, String> {
        let origins = origins
            .iter()
            .map(|o| Url::parse(o.trim()).map_err(|e| format!("bad origin {o}: {e}")))
            .collect::<Result<Vec<_>, _>>()?;
        let first = origins.first().ok_or("no origin")?;
        let rp_id = match rp_id {
            Some(id) if !id.is_empty() => id.to_owned(),
            _ => first.host_str().ok_or("origin without host")?.to_owned(),
        };
        let secure_cookie = origins.iter().all(|o| o.scheme() == "https");
        Ok(Self {
            rp_id,
            origins,
            secure_cookie,
        })
    }

    fn webauthn(&self) -> Result<Webauthn, String> {
        let first = self.origins.first().ok_or("no origin")?;
        let mut builder = WebauthnBuilder::new(&self.rp_id, first)
            .map_err(|e| format!("webauthn config: {e}"))?
            .rp_name("Stammtisch");
        for origin in &self.origins[1..] {
            builder = builder.append_allowed_origin(origin);
        }
        // Local development: vite dev and preview run on different ports.
        if self.rp_id == "localhost" {
            builder = builder.allow_any_port(true);
        }
        builder.build().map_err(|e| format!("webauthn config: {e}"))
    }

    /// Whether a request with this `Origin` header may change something.
    pub fn origin_allowed(&self, origin: &str) -> bool {
        let Ok(url) = Url::parse(origin) else {
            return false;
        };
        self.origins.iter().any(|o| {
            o.scheme() == url.scheme()
                && o.host_str() == url.host_str()
                && (o.port_or_known_default() == url.port_or_known_default()
                    || self.rp_id == "localhost")
        })
    }
}

/// A passkey ceremony between its start and finish.
pub enum Ceremony {
    /// Sign-up: the new user's id and optional name.
    Register {
        /// Id of the user to create.
        user_id: Uuid,
        /// Name given at the start, already cleaned.
        name: Option<String>,
        /// WebAuthn state.
        state: PasskeyRegistration,
    },
    /// A further passkey for a signed-in user.
    AddPasskey {
        /// The user.
        user_id: Uuid,
        /// WebAuthn state.
        state: PasskeyRegistration,
    },
    /// Sign-in with a discoverable passkey.
    Login {
        /// WebAuthn state.
        state: DiscoverableAuthentication,
    },
}

/// Ceremonies older than this are dropped (a passkey dialog takes seconds).
const CEREMONY_TTL: Duration = Duration::from_secs(120);
/// At most this many open ceremonies.
const CEREMONY_MAX: usize = 10_000;
/// At most this many open ceremonies per address (IPv6: per /64), so one
/// client cannot take all of them.
const CEREMONY_PER_CLIENT: usize = 10;

/// Shared state of the handlers.
#[derive(Clone)]
pub struct AppState {
    /// Settings.
    pub config: Arc<Config>,
    /// WebAuthn relying party.
    pub webauthn: Arc<Webauthn>,
    db: Arc<Mutex<Connection>>,
    ceremonies: Arc<Mutex<HashMap<String, OpenCeremony>>>,
    /// Requests per address.
    pub limiter: Arc<session::RateLimiter>,
    argon2: Arc<Semaphore>,
}

/// A ceremony with when and for whom it started.
struct OpenCeremony {
    at: Instant,
    client: IpAddr,
    ceremony: Ceremony,
}

impl AppState {
    /// State over an open database.
    pub fn new(config: Config, conn: Connection) -> Result<Self, String> {
        let webauthn = config.webauthn()?;
        Ok(Self {
            config: Arc::new(config),
            webauthn: Arc::new(webauthn),
            db: Arc::new(Mutex::new(conn)),
            ceremonies: Arc::new(Mutex::new(HashMap::new())),
            limiter: Arc::new(session::RateLimiter::default()),
            argon2: Arc::new(Semaphore::new(ARGON2_PERMITS)),
        })
    }

    /// Run argon2 work off the async threads, at most [`ARGON2_PERMITS`]
    /// at once.
    pub async fn argon2<T: Send + 'static>(
        &self,
        work: impl FnOnce() -> T + Send + 'static,
    ) -> Result<T, ApiError> {
        let _permit = self
            .argon2
            .acquire()
            .await
            .map_err(|_| ApiError::internal())?;
        tokio::task::spawn_blocking(work)
            .await
            .map_err(|_| ApiError::internal())
    }

    /// The database connection (held only for short statements).
    pub fn db(&self) -> MutexGuard<'_, Connection> {
        self.db
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Keep a ceremony a client started; returns its id for the client.
    /// One client may hold [`CEREMONY_PER_CLIENT`] (else 429); when all
    /// are taken the answer is 503 `busy`.
    pub fn put_ceremony(&self, client: IpAddr, ceremony: Ceremony) -> Result<String, ApiError> {
        let id = session::new_token();
        let now = Instant::now();
        let client = session::client_key(client);
        let mut map = self
            .ceremonies
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        map.retain(|_, c| now.duration_since(c.at) < CEREMONY_TTL);
        if map.values().filter(|c| c.client == client).count() >= CEREMONY_PER_CLIENT {
            return Err(ApiError::too_many());
        }
        if map.len() >= CEREMONY_MAX {
            return Err(ApiError::busy());
        }
        map.insert(
            id.clone(),
            OpenCeremony {
                at: now,
                client,
                ceremony,
            },
        );
        Ok(id)
    }

    /// Take a ceremony out (each one is finished at most once).
    pub fn take_ceremony(&self, id: &str) -> Option<Ceremony> {
        let mut map = self
            .ceremonies
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        map.remove(id)
            .filter(|c| c.at.elapsed() < CEREMONY_TTL)
            .map(|c| c.ceremony)
    }
}

/// Whether a request may change something, judged by where it comes from
/// (on top of the SameSite cookie and the JSON content type):
///
/// - `Origin` must be one of ours. Browsers send it with every POST, PUT
///   and DELETE, so a changing request without it is refused too; `curl`
///   needs `-H 'Origin: …'`.
/// - `Sec-Fetch-Site`, where the browser sends it, must not be
///   `cross-site` or `same-site`. The latter matters here: `sslip.io` is
///   not a public suffix, so every other `*.sslip.io` host is "same site"
///   for the browser and would get our SameSite=Lax cookie.
fn source_allowed(config: &Config, req: &Request) -> bool {
    if matches!(*req.method(), Method::GET | Method::HEAD | Method::OPTIONS) {
        return true;
    }
    let headers = req.headers();
    let fetch_site = headers
        .get("sec-fetch-site")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if fetch_site.eq_ignore_ascii_case("cross-site") || fetch_site.eq_ignore_ascii_case("same-site")
    {
        return false;
    }
    headers
        .get(header::ORIGIN)
        .and_then(|o| o.to_str().ok())
        .is_some_and(|o| config.origin_allowed(o))
}

/// Every request under `/api`: refuse changes from foreign sources, extend
/// the session (and its cookie) while it is used, and make sure every error
/// is the JSON the app reads (`{"error": code}`), also those axum answers
/// on its own (bad JSON, body too large, unknown path).
async fn api_layer(State(state): State<AppState>, req: Request, next: Next) -> Response {
    if !source_allowed(&state.config, &req) {
        return ApiError::new(StatusCode::FORBIDDEN, "origin").into_response();
    }
    let refreshed = match session::refresh(&state, req.headers()) {
        Ok(cookie) => cookie,
        Err(err) => return err.into_response(),
    };
    let mut res = next.run(req).await;
    let is_json = res
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.starts_with("application/json"));
    if res.status().as_u16() >= 400 && !is_json {
        res = ApiError::for_status(res.status()).into_response();
    }
    // A handler that set a cookie (sign-in, sign-out) has the last word.
    if let Some(cookie) = refreshed
        && !res.headers().contains_key(header::SET_COOKIE)
    {
        res.headers_mut().insert(header::SET_COOKIE, cookie);
    }
    // Answers of the API are never cached by the browser or a proxy.
    res.headers_mut().insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("no-store"),
    );
    res
}

async fn health() -> &'static str {
    "ok"
}

/// The router with every endpoint under `/api`.
pub fn router(state: AppState) -> Router {
    let api = Router::new()
        .route("/health", get(health))
        .route("/register/start", post(auth::register_start))
        .route(
            "/register/finish",
            post(auth::register_finish).layer(DefaultBodyLimit::max(PASSKEY_BODY_BYTES)),
        )
        .route("/login/start", post(auth::login_start))
        .route("/login/finish", post(auth::login_finish))
        .route("/recover", post(auth::recover))
        .route("/passkey/start", post(auth::passkey_start))
        .route(
            "/passkey/finish",
            post(auth::passkey_finish).layer(DefaultBodyLimit::max(PASSKEY_BODY_BYTES)),
        )
        .route("/passkeys", get(auth::list_passkeys))
        .route("/passkeys/{id}", delete(auth::remove_passkey))
        .route("/recovery-code", post(auth::new_recovery_code))
        .route("/logout", post(auth::logout))
        .route("/logout-all", post(auth::logout_all))
        .route("/me", get(account::me))
        .route("/me/name", put(account::set_name))
        .route(
            "/save",
            get(account::get_save)
                .put(account::put_save)
                .layer(DefaultBodyLimit::max(MAX_BODY_BYTES)),
        )
        .route("/account", delete(account::delete_account))
        .route("/daily", post(daily::submit))
        .route("/daily/{day}", get(daily::board));
    Router::new()
        .nest("/api", api)
        .layer(middleware::from_fn_with_state(state.clone(), api_layer))
        .layer(DefaultBodyLimit::max(SMALL_BODY_BYTES))
        .with_state(state)
}

/// Where and how the daily backup goes ([`spawn_housekeeping`]).
#[derive(Debug, Clone)]
pub struct Backup {
    /// The live database file.
    pub db: PathBuf,
    /// Folder of the copies (`/data/backup` on the vServer).
    pub dir: PathBuf,
    /// How many daily copies stay.
    pub keep: usize,
}

/// Write today's backup (by the server's clock, UTC) if there is none yet.
pub async fn run_backup(backup: Backup) {
    let day = daily::today_seed();
    let result =
        tokio::task::spawn_blocking(move || db::backup(&backup.db, &backup.dir, &day, backup.keep))
            .await;
    match result {
        Ok(Ok(Some(path))) => tracing::info!(path = %path.display(), "backup written"),
        Ok(Ok(None)) => {}
        Ok(Err(err)) => tracing::error!(%err, "backup failed"),
        Err(err) => tracing::error!(%err, "backup task failed"),
    }
}

/// Housekeeping every hour: drop expired sessions, and once a day write a
/// backup of the database (`VACUUM INTO`, the last `keep` days stay).
pub fn spawn_housekeeping(state: AppState, backup: Option<Backup>) {
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(60 * 60));
        loop {
            tick.tick().await;
            let purged = db::purge_sessions(&state.db());
            if let Ok(n) = purged
                && n > 0
            {
                tracing::info!(n, "expired sessions removed");
            }
            if let Some(backup) = &backup {
                run_backup(backup.clone()).await;
            }
        }
    });
}
