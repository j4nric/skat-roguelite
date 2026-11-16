//! Account and save server of Stammtisch (ADR 0021).
//!
//! A small HTTP API under `/api` behind our Caddy: sign-up and sign-in
//! with passkeys (WebAuthn), a recovery code as the way back in, a display
//! name, one opaque save blob per player, the daily and the weekly
//! leaderboard ([`board`]), and a one-time move code to take the account
//! to the new address ([`moving`], ADR 0025), and bug reports from the app
//! ([`report`], ADR 0027). The server never looks into the save and never
//! checks game state: all game logic stays in the browser (the Rust core
//! as WebAssembly). It knows nothing of `skat_core`.

pub mod account;
pub mod auth;
pub mod board;
pub mod daily;
pub mod db;
pub mod error;
pub mod moving;
pub mod names;
pub mod recovery;
pub mod report;
pub mod session;
pub mod weekly;

use axum::Router;
use axum::extract::{DefaultBodyLimit, Request, State};
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode, header};
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
/// Largest body of every other request: names, codes, board results.
pub const SMALL_BODY_BYTES: usize = 16 * 1024;
/// Largest body of the passkey registration finish calls: an attestation
/// object can carry a certificate chain (we ask for none, but some
/// authenticators send one anyway).
pub const PASSKEY_BODY_BYTES: usize = 64 * 1024;
/// At most this many argon2 hashes run at once (each takes ~19 MiB and a
/// core for a moment); further requests wait for a permit.
pub const ARGON2_PERMITS: usize = 2;

/// One WebAuthn relying party: an RP id and the origins that belong to it.
#[derive(Debug, Clone)]
pub struct RelyingParty {
    /// The RP id passkeys are bound to (a host name).
    pub id: String,
    /// Origins of this RP id (scheme, host, port).
    pub origins: Vec<Url>,
}

/// Settings of the server.
///
/// The site can live under several host names at once: the game moved
/// from `stammtisch-62-83-23-90.sslip.io` to `one-more-skat.de` (ADR 0025),
/// and the passkeys made on the old address are bound to its RP id. Every
/// host is then its own relying party, and a ceremony is run by the one
/// the request's `Origin` belongs to.
#[derive(Debug, Clone)]
pub struct Config {
    /// RP id of the main (first) origin.
    pub rp_id: String,
    /// Origins the browser may send from (the first is the main one).
    pub origins: Vec<Url>,
    /// The relying parties, the main one first.
    pub rps: Vec<RelyingParty>,
    /// Set `Secure` on the cookie (when the site is served over HTTPS).
    pub secure_cookie: bool,
    /// Origins of the native app (ADR 0026), compared as exact strings,
    /// e.g. `capacitor://localhost`. They are no relying party: the app
    /// makes its passkeys natively for the main RP id. A request from such
    /// an origin gets CORS headers, never uses a cookie and carries its
    /// session as a bearer token instead ([`session::TOKEN_HEADER`]).
    pub app_origins: Vec<String>,
}

impl Config {
    /// Config for these origins. With `rp_id`, one relying party with that
    /// id holds every origin (they must be the id or below it); without,
    /// every host of the origins is a relying party of its own.
    pub fn new(origins: &[&str], rp_id: Option<&str>) -> Result<Self, String> {
        let origins = origins
            .iter()
            .map(|o| Url::parse(o.trim()).map_err(|e| format!("bad origin {o}: {e}")))
            .collect::<Result<Vec<_>, _>>()?;
        if origins.is_empty() {
            return Err("no origin".into());
        }
        let mut rps: Vec<RelyingParty> = Vec::new();
        match rp_id {
            Some(id) if !id.is_empty() => rps.push(RelyingParty {
                id: id.to_owned(),
                origins: origins.clone(),
            }),
            _ => {
                for origin in &origins {
                    let host = origin.host_str().ok_or("origin without host")?;
                    match rps.iter_mut().find(|rp| rp.id == host) {
                        Some(rp) => rp.origins.push(origin.clone()),
                        None => rps.push(RelyingParty {
                            id: host.to_owned(),
                            origins: vec![origin.clone()],
                        }),
                    }
                }
            }
        }
        let rp_id = rps[0].id.clone();
        let secure_cookie = origins.iter().all(|o| o.scheme() == "https");
        Ok(Self {
            rp_id,
            origins,
            rps,
            secure_cookie,
            app_origins: Vec::new(),
        })
    }

    /// The same config with these origins of the native app (ADR 0026).
    /// Empty entries are dropped; an app origin must not be a site origin.
    pub fn with_app_origins(mut self, origins: &[&str]) -> Result<Self, String> {
        let mut list = Vec::new();
        for origin in origins.iter().map(|o| o.trim()).filter(|o| !o.is_empty()) {
            let url = Url::parse(origin).map_err(|e| format!("bad app origin {origin}: {e}"))?;
            if self.rp_of(origin).is_some() {
                return Err(format!("app origin {origin} is a site origin"));
            }
            // What a browser sends in `Origin`: no trailing slash, no path.
            let bare = url.path().is_empty() || url.path() == "/";
            if !bare || origin.ends_with('/') || url.query().is_some() {
                return Err(format!("app origin {origin} has a path"));
            }
            list.push(origin.to_owned());
        }
        self.app_origins = list;
        Ok(self)
    }

    /// Whether an `Origin` header is one of the native app's.
    pub fn is_app_origin(&self, origin: &str) -> bool {
        self.app_origins.iter().any(|o| o == origin)
    }

    fn webauthn(rp: &RelyingParty) -> Result<Webauthn, String> {
        let first = rp.origins.first().ok_or("no origin")?;
        let mut builder = WebauthnBuilder::new(&rp.id, first)
            .map_err(|e| format!("webauthn config {}: {e}", rp.id))?
            .rp_name("Stammtisch");
        for origin in &rp.origins[1..] {
            builder = builder.append_allowed_origin(origin);
        }
        // Local development: vite dev and preview run on different ports.
        if rp.id == "localhost" {
            builder = builder.allow_any_port(true);
        }
        builder
            .build()
            .map_err(|e| format!("webauthn config {}: {e}", rp.id))
    }

    /// Index of the relying party an `Origin` header belongs to.
    pub fn rp_of(&self, origin: &str) -> Option<usize> {
        let url = Url::parse(origin).ok()?;
        self.rps.iter().position(|rp| {
            rp.origins.iter().any(|o| {
                o.scheme() == url.scheme()
                    && o.host_str() == url.host_str()
                    && (o.port_or_known_default() == url.port_or_known_default()
                        || rp.id == "localhost")
            })
        })
    }

    /// Whether a request with this `Origin` header may change something.
    pub fn origin_allowed(&self, origin: &str) -> bool {
        self.rp_of(origin).is_some()
    }
}

/// A passkey ceremony between its start and finish.
pub enum Ceremony {
    /// Sign-up: the new user's id and optional name.
    Register {
        /// Relying party that started it ([`Config::rps`]).
        rp: usize,
        /// Id of the user to create.
        user_id: Uuid,
        /// Name given at the start, already checked ([`names`]).
        name: Option<String>,
        /// WebAuthn state.
        state: PasskeyRegistration,
    },
    /// A further passkey for a signed-in user.
    AddPasskey {
        /// Relying party that started it.
        rp: usize,
        /// The user.
        user_id: Uuid,
        /// WebAuthn state.
        state: PasskeyRegistration,
    },
    /// Sign-in with a discoverable passkey.
    Login {
        /// Relying party that started it.
        rp: usize,
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
    /// WebAuthn, one per relying party (same order as [`Config::rps`]).
    webauthn: Arc<Vec<Webauthn>>,
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
        let webauthn = config
            .rps
            .iter()
            .map(Config::webauthn)
            .collect::<Result<Vec<_>, _>>()?;
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

    /// The relying party a request comes from (by its `Origin`), with its
    /// index for the ceremony. Ceremonies are POSTs, which always carry an
    /// allowed `Origin` ([`api_layer`]); without one the main RP answers.
    /// So does the native app (ADR 0026): its origin is no relying party,
    /// it makes and uses passkeys of the main RP id through iOS.
    pub fn rp(&self, headers: &axum::http::HeaderMap) -> (usize, &Webauthn) {
        let index = headers
            .get(header::ORIGIN)
            .and_then(|o| o.to_str().ok())
            .and_then(|o| self.config.rp_of(o))
            .unwrap_or(0);
        self.rp_at(index)
    }

    /// The relying party with this index (the main one if out of range).
    pub fn rp_at(&self, index: usize) -> (usize, &Webauthn) {
        match self.webauthn.get(index) {
            Some(w) => (index, w),
            None => (0, &self.webauthn[0]),
        }
    }

    /// RP id of the relying party with this index.
    pub fn rp_id(&self, index: usize) -> &str {
        self.config
            .rps
            .get(index)
            .map_or(self.config.rp_id.as_str(), |rp| rp.id.as_str())
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
/// - The native app (ADR 0026) is always cross-site to us. Its requests
///   are let through by their `Origin` alone: they never carry a cookie
///   ([`api_layer`] drops one), only a bearer token no other page has, so
///   there is nothing a forged request could ride on.
fn source_allowed(config: &Config, req: &Request) -> bool {
    if matches!(*req.method(), Method::GET | Method::HEAD | Method::OPTIONS) {
        return true;
    }
    let headers = req.headers();
    let origin = headers.get(header::ORIGIN).and_then(|o| o.to_str().ok());
    if origin.is_some_and(|o| config.is_app_origin(o)) {
        return true;
    }
    let fetch_site = headers
        .get("sec-fetch-site")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if fetch_site.eq_ignore_ascii_case("cross-site") || fetch_site.eq_ignore_ascii_case("same-site")
    {
        return false;
    }
    origin.is_some_and(|o| config.origin_allowed(o))
}

/// The `Origin` of a request from the native app, if it is one.
fn app_origin(config: &Config, req: &Request) -> Option<HeaderValue> {
    req.headers()
        .get(header::ORIGIN)
        .filter(|o| o.to_str().is_ok_and(|o| config.is_app_origin(o)))
        .cloned()
}

/// CORS headers for the native app's origin (no credentials: the app
/// sends its session as a bearer token, never a cookie).
fn add_cors(headers: &mut HeaderMap, origin: HeaderValue) {
    headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);
    headers.insert(
        header::ACCESS_CONTROL_EXPOSE_HEADERS,
        HeaderValue::from_static(session::TOKEN_HEADER),
    );
    headers.append(header::VARY, HeaderValue::from_static("Origin"));
}

/// The answer to the native app's CORS preflight.
fn preflight(origin: HeaderValue) -> Response {
    let mut res = StatusCode::NO_CONTENT.into_response();
    let headers = res.headers_mut();
    add_cors(headers, origin);
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        HeaderValue::from_static("GET, POST, PUT, DELETE"),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        HeaderValue::from_static("Authorization, Content-Type"),
    );
    headers.insert(
        header::ACCESS_CONTROL_MAX_AGE,
        HeaderValue::from_static("600"),
    );
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    res
}

/// For the native app: the session a handler set as a cookie goes into
/// [`session::TOKEN_HEADER`] instead (empty when it was cleared), and no
/// cookie is set.
fn cookie_to_token(headers: &mut HeaderMap) {
    let token = headers
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .find_map(|v| {
            let pair = v.split(';').next()?.trim();
            let (name, value) = pair.split_once('=')?;
            (name == session::COOKIE_NAME).then(|| value.to_owned())
        });
    headers.remove(header::SET_COOKIE);
    if let Some(token) = token
        && let Ok(value) = HeaderValue::from_str(&token)
    {
        headers.insert(session::TOKEN_HEADER, value);
    }
}

/// Every request under `/api`: refuse changes from foreign sources, extend
/// the session (and its cookie) while it is used, and make sure every error
/// is the JSON the app reads (`{"error": code}`), also those axum answers
/// on its own (bad JSON, body too large, unknown path).
///
/// Requests of the native app (ADR 0026) get CORS headers and a preflight
/// answer, are never read with a cookie, and see their session as a
/// bearer token in [`session::TOKEN_HEADER`] instead of `Set-Cookie`.
async fn api_layer(State(state): State<AppState>, mut req: Request, next: Next) -> Response {
    let app = app_origin(&state.config, &req);
    if let Some(origin) = &app {
        if req.method() == Method::OPTIONS {
            return preflight(origin.clone());
        }
        req.headers_mut().remove(header::COOKIE);
    }
    if !source_allowed(&state.config, &req) {
        let mut res = ApiError::new(StatusCode::FORBIDDEN, "origin").into_response();
        if let Some(origin) = app {
            add_cors(res.headers_mut(), origin);
        }
        return res;
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
    if let Some(origin) = app {
        cookie_to_token(res.headers_mut());
        add_cors(res.headers_mut(), origin);
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
        .route("/move/start", post(moving::move_start))
        .route("/move/redeem", post(moving::move_redeem))
        .route("/me", get(account::me))
        .route("/me/name", put(account::set_name))
        .route(
            "/save",
            get(account::get_save)
                .put(account::put_save)
                .layer(DefaultBodyLimit::max(MAX_BODY_BYTES)),
        )
        .route("/account", delete(account::delete_account))
        .route("/daily", post(board::submit_daily))
        .route("/daily/{day}", get(board::show_daily))
        .route("/weekly", post(board::submit_weekly))
        .route("/weekly/{week}", get(board::show_weekly))
        .route(
            "/reports",
            post(report::submit).layer(DefaultBodyLimit::max(report::REPORT_BODY_BYTES)),
        );
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

/// Housekeeping every hour: drop expired sessions and bug reports older
/// than [`report::KEEP_DAYS`] days, and once a day write a backup of the
/// database (`VACUUM INTO`, the last `keep` days stay).
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
            let purged = report::purge(&state.db(), db::now_ms());
            if let Ok(n) = purged
                && n > 0
            {
                tracing::info!(n, "old reports removed");
            }
            if let Some(backup) = &backup {
                run_backup(backup.clone()).await;
            }
        }
    });
}
