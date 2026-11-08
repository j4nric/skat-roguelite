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
use axum::http::{Method, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, post, put};
use rusqlite::Connection;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};
use url::Url;
use webauthn_rs::prelude::{DiscoverableAuthentication, PasskeyRegistration, Uuid};
use webauthn_rs::{Webauthn, WebauthnBuilder};

use crate::error::ApiError;

/// Largest request body (the save blob is at most [`account::MAX_SAVE_BYTES`]).
pub const MAX_BODY_BYTES: usize = 1_200_000;

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

/// Ceremonies older than this are dropped.
const CEREMONY_TTL: Duration = Duration::from_secs(300);
/// At most this many open ceremonies.
const CEREMONY_MAX: usize = 10_000;

/// Shared state of the handlers.
#[derive(Clone)]
pub struct AppState {
    /// Settings.
    pub config: Arc<Config>,
    /// WebAuthn relying party.
    pub webauthn: Arc<Webauthn>,
    db: Arc<Mutex<Connection>>,
    ceremonies: Arc<Mutex<HashMap<String, (Instant, Ceremony)>>>,
    /// Requests per address.
    pub limiter: Arc<session::RateLimiter>,
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
        })
    }

    /// The database connection (held only for short statements).
    pub fn db(&self) -> MutexGuard<'_, Connection> {
        self.db
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Keep a ceremony; returns its id for the client.
    pub fn put_ceremony(&self, ceremony: Ceremony) -> Result<String, ApiError> {
        let id = session::new_token();
        let now = Instant::now();
        let mut map = self
            .ceremonies
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        map.retain(|_, (at, _)| now.duration_since(*at) < CEREMONY_TTL);
        if map.len() >= CEREMONY_MAX {
            return Err(ApiError::too_many());
        }
        map.insert(id.clone(), (now, ceremony));
        Ok(id)
    }

    /// Take a ceremony out (each one is finished at most once).
    pub fn take_ceremony(&self, id: &str) -> Option<Ceremony> {
        let mut map = self
            .ceremonies
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        map.remove(id)
            .filter(|(at, _)| at.elapsed() < CEREMONY_TTL)
            .map(|(_, c)| c)
    }
}

/// Changing requests from a foreign origin are refused (on top of the
/// SameSite cookie and the JSON content type).
async fn check_origin(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let changes = !matches!(*req.method(), Method::GET | Method::HEAD | Method::OPTIONS);
    if changes
        && let Some(origin) = req.headers().get(header::ORIGIN)
        && !origin
            .to_str()
            .map(|o| state.config.origin_allowed(o))
            .unwrap_or(false)
    {
        return ApiError::new(axum::http::StatusCode::FORBIDDEN, "origin").into_response();
    }
    let mut res = next.run(req).await;
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
        .route("/register/finish", post(auth::register_finish))
        .route("/login/start", post(auth::login_start))
        .route("/login/finish", post(auth::login_finish))
        .route("/recover", post(auth::recover))
        .route("/passkey/start", post(auth::passkey_start))
        .route("/passkey/finish", post(auth::passkey_finish))
        .route("/recovery-code", post(auth::new_recovery_code))
        .route("/logout", post(auth::logout))
        .route("/me", get(account::me))
        .route("/me/name", put(account::set_name))
        .route("/save", get(account::get_save).put(account::put_save))
        .route("/account", delete(account::delete_account))
        .route("/daily", post(daily::submit))
        .route("/daily/{day}", get(daily::board));
    Router::new()
        .nest("/api", api)
        .layer(middleware::from_fn_with_state(state.clone(), check_origin))
        .layer(DefaultBodyLimit::max(MAX_BODY_BYTES))
        .with_state(state)
}

/// Drop expired sessions now and then.
pub fn spawn_housekeeping(state: AppState) {
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(6 * 60 * 60));
        loop {
            tick.tick().await;
            let purged = db::purge_sessions(&state.db());
            if let Ok(n) = purged
                && n > 0
            {
                tracing::info!(n, "expired sessions removed");
            }
        }
    });
}
