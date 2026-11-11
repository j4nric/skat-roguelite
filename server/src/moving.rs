//! Moving to the new address (ADR 0025): the game moved from
//! `stammtisch-62-83-23-90.sslip.io` to `one-more-skat.de`, and passkeys
//! are bound to the address they were made on. A player signed in on the
//! old address makes a move code there; on the new address the code signs
//! the same account in once, and the app offers to add a passkey for it.
//!
//! - The code is 32 random bytes (base64url, 43 characters); only its
//!   SHA-256 is stored. It lives [`MOVE_CODE_MINUTES`] minutes, works
//!   once, and a new one replaces the old one.
//! - It travels in the link's fragment (`#umzug=…`), which the browser
//!   never sends, and to the server only in the body of
//!   `POST /api/move/redeem`: it is in no request line and no log.
//! - Redeeming does not touch other sessions or the recovery code: it is
//!   the player moving, not somebody taking the account over. It counts
//!   against the recovery-code rate limit, so it cannot be guessed (256
//!   bits would not be guessed anyway).

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::account::user_json;
use crate::error::{ApiError, ApiResult};
use crate::session::{self, Bucket, ClientIp};
use crate::{AppState, db};

/// A move code is good for this long.
pub const MOVE_CODE_MINUTES: i64 = 10;
/// Longest code accepted (the real ones have 43 characters).
const MAX_CODE_LEN: usize = 128;

/// `POST /api/move/start`: a fresh move code for the signed-in account:
/// `{"code": "…", "expiresAt": ms}`.
pub async fn move_start(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
) -> ApiResult<Json<Value>> {
    state.limiter.check(ip, Bucket::Auth)?;
    let session = session::require(&state, &headers)?;
    let code = session::new_token();
    let expires_at = db::now_ms() + MOVE_CODE_MINUTES * 60 * 1000;
    db::put_move_code(
        &state.db(),
        &session.user_id,
        &session::token_hash(&code),
        expires_at,
    )?;
    tracing::info!("move code made");
    Ok(Json(json!({ "code": code, "expiresAt": expires_at })))
}

/// Body of `POST /api/move/redeem`.
#[derive(Debug, Deserialize)]
pub struct Redeem {
    /// The code from the link.
    pub code: String,
}

/// `POST /api/move/redeem`: sign in with a move code, which is used up.
/// 401 `move_code` for an unknown, used or expired one.
pub async fn move_redeem(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    Json(body): Json<Redeem>,
) -> ApiResult<impl IntoResponse> {
    state.limiter.check(ip, Bucket::Recover)?;
    let code = body.code.trim();
    if code.is_empty() || code.len() > MAX_CODE_LEN {
        return Err(ApiError::new(StatusCode::UNAUTHORIZED, "move_code"));
    }
    let user_id = db::take_move_code(&state.db(), &session::token_hash(code))?
        .ok_or(ApiError::new(StatusCode::UNAUTHORIZED, "move_code"))?;
    let started = session::start(&state, &user_id)?;
    let user = user_json(&state, &user_id)?;
    tracing::info!("move code used");
    Ok((started.headers, Json(json!({ "user": user }))))
}
