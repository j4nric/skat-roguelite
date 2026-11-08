//! Sessions (HttpOnly cookie with a random token, only its SHA-256 is
//! stored), the client address and a light in-memory rate limit.

use axum::extract::{ConnectInfo, FromRequestParts};
use axum::http::header::{COOKIE, SET_COOKIE};
use axum::http::request::Parts;
use axum::http::{HeaderMap, HeaderValue};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::RngCore;
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::AppState;
use crate::db;
use crate::error::{ApiError, ApiResult};

/// Name of the session cookie.
pub const COOKIE_NAME: &str = "stammtisch_session";
/// A session lasts this long and is extended while it is used.
pub const SESSION_DAYS: i64 = 180;
const DAY_MS: i64 = 24 * 60 * 60 * 1000;

/// A random token for a new session.
pub fn new_token() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}

/// Hex SHA-256 of a token, as stored.
pub fn token_hash(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

/// The session token from the request's cookies.
pub fn token_of(headers: &HeaderMap) -> Option<String> {
    headers
        .get_all(COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(k, _)| *k == COOKIE_NAME)
        .map(|(_, v)| v.to_owned())
        .filter(|v| !v.is_empty() && v.len() <= 128)
}

/// `Set-Cookie` for a session token.
pub fn set_cookie(token: &str, secure: bool) -> HeaderValue {
    let secure = if secure { "; Secure" } else { "" };
    let max_age = SESSION_DAYS * 24 * 60 * 60;
    HeaderValue::from_str(&format!(
        "{COOKIE_NAME}={token}; Path=/api; HttpOnly; SameSite=Lax; Max-Age={max_age}{secure}"
    ))
    .unwrap_or_else(|_| HeaderValue::from_static(""))
}

/// `Set-Cookie` that removes the session cookie.
pub fn clear_cookie(secure: bool) -> HeaderValue {
    let secure = if secure { "; Secure" } else { "" };
    HeaderValue::from_str(&format!(
        "{COOKIE_NAME}=; Path=/api; HttpOnly; SameSite=Lax; Max-Age=0{secure}"
    ))
    .unwrap_or_else(|_| HeaderValue::from_static(""))
}

/// Start a session for a user; returns the headers that set the cookie.
pub fn start(state: &AppState, user_id: &str) -> ApiResult<HeaderMap> {
    let token = new_token();
    let expires = db::now_ms() + SESSION_DAYS * DAY_MS;
    {
        let conn = state.db();
        db::insert_session(&conn, &token_hash(&token), user_id, expires)?;
    }
    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, set_cookie(&token, state.config.secure_cookie));
    Ok(headers)
}

/// The signed-in user of a request.
#[derive(Debug, Clone)]
pub struct Session {
    /// User id.
    pub user_id: String,
    /// Hash of the token (to end the session).
    pub token_hash: String,
}

/// Find the session of a request, extending it when half of it has passed.
pub fn current(state: &AppState, headers: &HeaderMap) -> ApiResult<Option<Session>> {
    let Some(token) = token_of(headers) else {
        return Ok(None);
    };
    let hash = token_hash(&token);
    let conn = state.db();
    let Some((user_id, expires)) = db::session_user(&conn, &hash)? else {
        return Ok(None);
    };
    let now = db::now_ms();
    if expires - now < SESSION_DAYS * DAY_MS / 2 {
        db::extend_session(&conn, &hash, now + SESSION_DAYS * DAY_MS)?;
    }
    Ok(Some(Session {
        user_id,
        token_hash: hash,
    }))
}

/// Like [`current`], but a missing session is a 401.
pub fn require(state: &AppState, headers: &HeaderMap) -> ApiResult<Session> {
    current(state, headers)?.ok_or_else(ApiError::unauthorized)
}

/// The address of the client: `X-Client-IP` as set by our Caddy (which
/// knows the address behind the front proxy), else the TCP peer.
#[derive(Debug, Clone, Copy)]
pub struct ClientIp(pub IpAddr);

impl<S: Send + Sync> FromRequestParts<S> for ClientIp {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get("x-client-ip")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.trim().parse::<IpAddr>().ok());
        let peer = parts
            .extensions
            .get::<ConnectInfo<SocketAddr>>()
            .map(|c| c.0.ip());
        Ok(ClientIp(
            header.or(peer).unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED)),
        ))
    }
}

/// What a request does, for the rate limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Bucket {
    /// Passkey ceremonies and sign-up.
    Auth,
    /// Recovery code login (argon2, guessing).
    Recover,
    /// Writes: save, name, daily result.
    Write,
    /// Reads.
    Read,
}

impl Bucket {
    /// Requests per minute and address.
    pub const fn per_minute(self) -> u32 {
        match self {
            Bucket::Auth => 30,
            Bucket::Recover => 10,
            Bucket::Write => 120,
            Bucket::Read => 300,
        }
    }
}

/// Fixed one-minute windows per address and bucket.
#[derive(Debug, Default)]
pub struct RateLimiter {
    windows: Mutex<HashMap<(IpAddr, Bucket), (Instant, u32)>>,
}

impl RateLimiter {
    /// Count a request; error if the address used up its minute.
    pub fn check(&self, ip: IpAddr, bucket: Bucket) -> ApiResult<()> {
        let now = Instant::now();
        let mut map = self
            .windows
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if map.len() > 50_000 {
            map.retain(|_, (start, _)| now.duration_since(*start) < Duration::from_secs(60));
        }
        let entry = map.entry((ip, bucket)).or_insert((now, 0));
        if now.duration_since(entry.0) >= Duration::from_secs(60) {
            *entry = (now, 0);
        }
        entry.1 += 1;
        if entry.1 > bucket.per_minute() {
            Err(ApiError::too_many())
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cookie_is_read_among_others() {
        let mut headers = HeaderMap::new();
        headers.insert(
            COOKIE,
            HeaderValue::from_static("a=1; stammtisch_session=abc; b=2"),
        );
        assert_eq!(token_of(&headers).as_deref(), Some("abc"));
    }

    #[test]
    fn limiter_stops_after_the_budget() {
        let limiter = RateLimiter::default();
        let ip = IpAddr::V4(Ipv4Addr::LOCALHOST);
        for _ in 0..Bucket::Recover.per_minute() {
            assert!(limiter.check(ip, Bucket::Recover).is_ok());
        }
        assert!(limiter.check(ip, Bucket::Recover).is_err());
        // Other buckets and addresses are untouched.
        assert!(limiter.check(ip, Bucket::Read).is_ok());
        assert!(
            limiter
                .check(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)), Bucket::Recover)
                .is_ok()
        );
    }
}
