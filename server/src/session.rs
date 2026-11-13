//! Sessions (HttpOnly cookie with a random token, only its SHA-256 is
//! stored), the client address and a light in-memory rate limit.
//!
//! The native app (ADR 0026) has no cookie: it gets the same token in
//! [`TOKEN_HEADER`] and sends it back as `Authorization: Bearer <token>`.

use argon2::password_hash::rand_core::{OsRng, RngCore};
use axum::extract::{ConnectInfo, FromRequestParts};
use axum::http::header::{AUTHORIZATION, COOKIE, SET_COOKIE};
use axum::http::request::Parts;
use axum::http::{HeaderMap, HeaderValue};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use crate::AppState;
use crate::db;
use crate::error::{ApiError, ApiResult};

/// Name of the session cookie.
pub const COOKIE_NAME: &str = "stammtisch_session";
/// Response header that hands the native app its session token where a
/// browser would get the cookie (ADR 0026); empty when the session ended.
pub const TOKEN_HEADER: &str = "x-session-token";
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

/// The session token of a request: `Authorization: Bearer <token>` (the
/// native app, ADR 0026), else the cookie.
pub fn token_of(headers: &HeaderMap) -> Option<String> {
    if let Some(auth) = headers.get(AUTHORIZATION).and_then(|v| v.to_str().ok()) {
        let token = auth
            .strip_prefix("Bearer ")
            .map(str::trim)
            .filter(|t| !t.is_empty() && t.len() <= 128);
        return token.map(str::to_owned);
    }
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

/// A session just started: the headers that set its cookie and the hash
/// of its token (to keep it when other sessions are ended).
pub struct Started {
    /// `Set-Cookie` for the new token.
    pub headers: HeaderMap,
    /// Hash of the new token as stored.
    pub token_hash: String,
}

/// Start a session for a user.
pub fn start(state: &AppState, user_id: &str) -> ApiResult<Started> {
    let token = new_token();
    let hash = token_hash(&token);
    let expires = db::now_ms() + SESSION_DAYS * DAY_MS;
    db::insert_session(&state.db(), &hash, user_id, expires)?;
    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, set_cookie(&token, state.config.secure_cookie));
    Ok(Started {
        headers,
        token_hash: hash,
    })
}

/// The signed-in user of a request.
#[derive(Debug, Clone)]
pub struct Session {
    /// User id.
    pub user_id: String,
    /// Hash of the token (to end the session).
    pub token_hash: String,
}

/// Find the session of a request. Extending it is the job of
/// [`refresh`], which runs for every request before the handler.
pub fn current(state: &AppState, headers: &HeaderMap) -> ApiResult<Option<Session>> {
    let Some(token) = token_of(headers) else {
        return Ok(None);
    };
    let hash = token_hash(&token);
    let Some((user_id, _)) = db::session_user(&state.db(), &hash)? else {
        return Ok(None);
    };
    Ok(Some(Session {
        user_id,
        token_hash: hash,
    }))
}

/// Extend the request's session when half of it has passed. Returns the
/// `Set-Cookie` that gives the browser the new lifetime too: the cookie's
/// `Max-Age` was set at sign-in and would otherwise run out while the
/// session in the database still lives.
pub fn refresh(state: &AppState, headers: &HeaderMap) -> ApiResult<Option<HeaderValue>> {
    let Some(token) = token_of(headers) else {
        return Ok(None);
    };
    let hash = token_hash(&token);
    let conn = state.db();
    let Some((_, expires)) = db::session_user(&conn, &hash)? else {
        return Ok(None);
    };
    let now = db::now_ms();
    if expires - now >= SESSION_DAYS * DAY_MS / 2 {
        return Ok(None);
    }
    db::extend_session(&conn, &hash, now + SESSION_DAYS * DAY_MS)?;
    Ok(Some(set_cookie(&token, state.config.secure_cookie)))
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
    /// Writes: save, name, board results.
    Write,
    /// Reads.
    Read,
    /// New accounts (each one may store a save of up to 1 MB).
    SignUp,
    /// Bug reports (up to 256 KB each, ADR 0027).
    Report,
}

impl Bucket {
    /// Requests per window and address.
    pub const fn limit(self) -> u32 {
        match self {
            Bucket::Auth => 30,
            Bucket::Recover => 10,
            Bucket::Write => 120,
            Bucket::Read => 300,
            Bucket::SignUp => 5,
            Bucket::Report => 5,
        }
    }

    /// Length of the window: a minute, an hour for new accounts and reports.
    pub const fn window(self) -> Duration {
        match self {
            Bucket::SignUp | Bucket::Report => Duration::from_secs(60 * 60),
            _ => Duration::from_secs(60),
        }
    }
}

/// The key an address is counted under. An IPv6 client usually holds a
/// whole /64 (and can pick any address in it), so IPv6 counts per /64;
/// an IPv4-mapped IPv6 address counts as the IPv4 address.
pub fn client_key(ip: IpAddr) -> IpAddr {
    match ip {
        IpAddr::V4(_) => ip,
        IpAddr::V6(v6) => match v6.to_ipv4_mapped() {
            Some(v4) => IpAddr::V4(v4),
            None => {
                let prefix = v6.to_bits() & (u128::MAX << 64);
                IpAddr::V6(Ipv6Addr::from_bits(prefix))
            }
        },
    }
}

/// Fixed windows per address (see [`client_key`]) and bucket.
#[derive(Debug, Default)]
pub struct RateLimiter {
    windows: Mutex<HashMap<(IpAddr, Bucket), (Instant, u32)>>,
}

impl RateLimiter {
    /// Count a request; error if the address used up its window.
    pub fn check(&self, ip: IpAddr, bucket: Bucket) -> ApiResult<()> {
        let now = Instant::now();
        let mut map = self
            .windows
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if map.len() > 50_000 {
            map.retain(|(_, b), (start, _)| now.duration_since(*start) < b.window());
        }
        let entry = map.entry((client_key(ip), bucket)).or_insert((now, 0));
        if now.duration_since(entry.0) >= bucket.window() {
            *entry = (now, 0);
        }
        entry.1 += 1;
        if entry.1 > bucket.limit() {
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
    fn bearer_token_comes_first() {
        let mut headers = HeaderMap::new();
        headers.insert(COOKIE, HeaderValue::from_static("stammtisch_session=abc"));
        headers.insert(AUTHORIZATION, HeaderValue::from_static("Bearer xyz"));
        assert_eq!(token_of(&headers).as_deref(), Some("xyz"));
        // A broken Authorization header is no session, not the cookie's.
        headers.insert(AUTHORIZATION, HeaderValue::from_static("Basic xyz"));
        assert_eq!(token_of(&headers), None);
        headers.insert(AUTHORIZATION, HeaderValue::from_static("Bearer "));
        assert_eq!(token_of(&headers), None);
    }

    #[test]
    fn limiter_stops_after_the_budget() {
        let limiter = RateLimiter::default();
        let ip = IpAddr::V4(Ipv4Addr::LOCALHOST);
        for _ in 0..Bucket::Recover.limit() {
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

    #[test]
    fn ipv6_counts_per_64() {
        let limiter = RateLimiter::default();
        let a: IpAddr = "2001:db8:1:2::1".parse().unwrap();
        let b: IpAddr = "2001:db8:1:2:ffff:1234:5678:9abc".parse().unwrap();
        let other: IpAddr = "2001:db8:1:3::1".parse().unwrap();
        for _ in 0..Bucket::SignUp.limit() {
            assert!(limiter.check(a, Bucket::SignUp).is_ok());
        }
        // Another address in the same /64 shares the budget.
        assert!(limiter.check(b, Bucket::SignUp).is_err());
        assert!(limiter.check(other, Bucket::SignUp).is_ok());
        // An IPv4-mapped address is the IPv4 address.
        let mapped: IpAddr = "::ffff:10.0.0.7".parse().unwrap();
        assert_eq!(client_key(mapped), IpAddr::V4(Ipv4Addr::new(10, 0, 0, 7)));
    }
}
