//! The signed-in player: name, save, deleting the account.
//!
//! The save is an opaque text (the app's save code, `STAMMTISCH1:…`): the
//! server stores it with a version and the time the player's state
//! changed, and never reads it. A write names the version it builds on
//! (`ifVersion`); if another device wrote in between, the answer is 409
//! and the app asks the player which state to keep.

use axum::Json;
use axum::extract::State;
use axum::http::header::SET_COOKIE;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::error::{ApiError, ApiResult};
use crate::session::{self, Bucket, ClientIp};
use crate::{AppState, db};

/// Largest save blob in bytes.
pub const MAX_SAVE_BYTES: usize = 1_000_000;
/// Longest display name in characters.
pub const MAX_NAME_CHARS: usize = 24;

/// Clean a display name: control and format characters out, runs of
/// whitespace become one space, trimmed; 1 to 24 characters. Names need
/// not be unique.
pub fn clean_name(raw: &str) -> Option<String> {
    let mut out = String::new();
    let mut space = false;
    for c in raw.chars() {
        if c.is_whitespace() {
            space = !out.is_empty();
            continue;
        }
        // Control characters and the invisible ones that could hide or
        // turn text around (zero-width, bidi overrides, BOM).
        if c.is_control()
            || matches!(c, '\u{200B}'..='\u{200F}' | '\u{202A}'..='\u{202E}' | '\u{2060}'..='\u{206F}' | '\u{FEFF}')
        {
            continue;
        }
        if space {
            out.push(' ');
            space = false;
        }
        out.push(c);
    }
    let n = out.chars().count();
    (1..=MAX_NAME_CHARS).contains(&n).then_some(out)
}

/// Whether a text starts like a save code (`STAMMTISCH<format>:`, see
/// `web/src/savecode.ts`). A cheap check, not a parse: the server never
/// reads the save, but it need not store arbitrary data either. Any
/// format number passes, so a newer app's save is kept as it is.
pub fn looks_like_save(data: &str) -> bool {
    data.strip_prefix("STAMMTISCH")
        .and_then(|rest| rest.split_once(':'))
        .is_some_and(|(format, _)| {
            (1..=4).contains(&format.len()) && format.bytes().all(|b| b.is_ascii_digit())
        })
}

/// The user as JSON: `{name, passkeys, createdAt}`.
pub fn user_json(state: &AppState, user_id: &str) -> ApiResult<Value> {
    let conn = state.db();
    let user = db::user(&conn, user_id)?.ok_or_else(ApiError::unauthorized)?;
    let passkeys = db::passkey_count(&conn, user_id)?;
    Ok(json!({
        "name": user.name,
        "passkeys": passkeys,
        "createdAt": user.created_at,
    }))
}

/// `GET /api/me`: who is signed in; `{"user": null}` if nobody (not an
/// error: the app asks this without knowing).
pub async fn me(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
) -> ApiResult<Json<Value>> {
    state.limiter.check(ip, Bucket::Read)?;
    let user = match session::current(&state, &headers)? {
        Some(session) => user_json(&state, &session.user_id)?,
        None => Value::Null,
    };
    Ok(Json(json!({ "user": user })))
}

/// Body of `PUT /api/me/name`.
#[derive(Debug, Deserialize)]
pub struct SetName {
    /// The new name; empty removes it.
    pub name: String,
}

/// `PUT /api/me/name`: set or remove the display name.
pub async fn set_name(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
    Json(body): Json<SetName>,
) -> ApiResult<Json<Value>> {
    state.limiter.check(ip, Bucket::Write)?;
    let session = session::require(&state, &headers)?;
    let name = if body.name.trim().is_empty() {
        None
    } else {
        Some(clean_name(&body.name).ok_or(ApiError::bad_request("name"))?)
    };
    db::set_name(&state.db(), &session.user_id, name.as_deref())?;
    Ok(Json(
        json!({ "user": user_json(&state, &session.user_id)? }),
    ))
}

/// `GET /api/save`: the stored save, `{"save": null}` if there is none.
pub async fn get_save(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
) -> ApiResult<Json<Value>> {
    state.limiter.check(ip, Bucket::Read)?;
    let session = session::require(&state, &headers)?;
    let save = db::save(&state.db(), &session.user_id)?;
    Ok(Json(json!({
        "save": save.map(|s| json!({ "version": s.version, "updatedAt": s.updated_at, "data": s.data })),
    })))
}

/// Body of `PUT /api/save`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PutSave {
    /// The save code.
    pub data: String,
    /// When the player's state changed (ms since epoch, client clock).
    pub updated_at: i64,
    /// The version this write builds on; null for the first save.
    pub if_version: Option<i64>,
}

/// `PUT /api/save`: store the save if nothing newer is there.
pub async fn put_save(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
    Json(body): Json<PutSave>,
) -> ApiResult<impl IntoResponse> {
    state.limiter.check(ip, Bucket::Write)?;
    let session = session::require(&state, &headers)?;
    if body.data.len() > MAX_SAVE_BYTES {
        return Err(ApiError::new(StatusCode::PAYLOAD_TOO_LARGE, "too_large"));
    }
    if body.data.is_empty() {
        return Err(ApiError::bad_request("empty"));
    }
    if !looks_like_save(&body.data) {
        return Err(ApiError::bad_request("not_a_save"));
    }
    // A clock far off is clamped: the time only orders states for the player.
    let now = db::now_ms();
    let updated_at = body.updated_at.clamp(0, now + 24 * 60 * 60 * 1000);
    let result = db::put_save(
        &mut state.db(),
        &session.user_id,
        body.if_version,
        updated_at,
        &body.data,
    )?;
    Ok(match result {
        db::PutSave::Stored(version) => (
            StatusCode::OK,
            Json(json!({ "version": version, "updatedAt": updated_at })),
        ),
        db::PutSave::Conflict(current) => (
            StatusCode::CONFLICT,
            Json(json!({
                "error": "conflict",
                "save": current.map(|(version, updated_at)| json!({ "version": version, "updatedAt": updated_at })),
            })),
        ),
    })
}

/// `DELETE /api/account`: delete the account with passkeys, sessions,
/// save and daily results. The passkey stays in the player's password
/// manager; it no longer opens anything.
pub async fn delete_account(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
) -> ApiResult<impl IntoResponse> {
    state.limiter.check(ip, Bucket::Write)?;
    let session = session::require(&state, &headers)?;
    db::delete_user(&state.db(), &session.user_id)?;
    tracing::info!("account deleted");
    let mut out = HeaderMap::new();
    out.insert(
        SET_COOKIE,
        session::clear_cookie(state.config.secure_cookie),
    );
    Ok((StatusCode::NO_CONTENT, out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_cleaned() {
        assert_eq!(clean_name("  Jan  "), Some("Jan".into()));
        assert_eq!(
            clean_name("Der   alte\tFritz"),
            Some("Der alte Fritz".into())
        );
        assert_eq!(clean_name("A\u{0007}b\u{202E}c"), Some("Abc".into()));
        assert_eq!(clean_name("   "), None);
        assert_eq!(clean_name("\u{200B}"), None);
        assert_eq!(clean_name(&"x".repeat(24)), Some("x".repeat(24)));
        assert_eq!(clean_name(&"x".repeat(25)), None);
        assert_eq!(clean_name("Jürgen 🍺"), Some("Jürgen 🍺".into()));
    }

    #[test]
    fn saves_are_recognised_by_their_prefix() {
        assert!(looks_like_save("STAMMTISCH1:d:abc:0011aabb"));
        assert!(looks_like_save("STAMMTISCH12:x"));
        assert!(!looks_like_save("STAMMTISCH:x"));
        assert!(!looks_like_save("STAMMTISCHx:1"));
        assert!(!looks_like_save("hello"));
        assert!(!looks_like_save(" STAMMTISCH1:x"));
    }
}
