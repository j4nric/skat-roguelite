//! Sign-up, sign-in and the recovery code.
//!
//! - Sign-up creates a discoverable passkey (resident key), so signing in
//!   needs no user name: the authenticator offers the passkeys it has for
//!   this site and returns the user handle (our user id).
//! - The recovery code is shown once at sign-up and replaced by a new one
//!   whenever it is used or renewed.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::header::SET_COOKIE;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::Deserialize;
use serde_json::{Value, json};
use webauthn_rs::prelude::{Passkey, PublicKeyCredential, RegisterPublicKeyCredential, Uuid};

use crate::account::user_json;
use crate::error::{ApiError, ApiResult};
use crate::session::{self, Bucket, ClientIp};
use crate::{AppState, Ceremony, db, names, recovery};

/// Label of every passkey in the password manager. The display name is
/// only building-block ids (ADR 0029) whose words the server does not
/// know, so it is not used here.
const LABEL: &str = "Stammtisch-Spieler";

fn passkey_error(err: impl std::fmt::Display) -> ApiError {
    tracing::info!(%err, "passkey ceremony failed");
    ApiError::bad_request("passkey")
}

fn to_json<T: serde::Serialize>(value: &T) -> ApiResult<Value> {
    serde_json::to_value(value).map_err(|err| {
        tracing::error!(%err, "serialize");
        ApiError::internal()
    })
}

/// Ask for a discoverable credential (resident key): sign-in has no user
/// name. The server side of the ceremony does not depend on it.
fn require_resident_key(options: &mut Value) {
    if let Some(sel) = options
        .get_mut("publicKey")
        .and_then(|p| p.get_mut("authenticatorSelection"))
        .and_then(Value::as_object_mut)
    {
        sel.insert("residentKey".into(), json!("required"));
        sel.insert("requireResidentKey".into(), json!(true));
    }
}

/// A fresh recovery code: what the player sees and what is stored.
struct FreshCode {
    /// Formatted code, shown once.
    display: String,
    /// Public lookup id.
    id: String,
    /// argon2 hash of the secret part.
    hash: String,
}

/// Make a recovery code and hash its secret (off the async threads, with
/// an argon2 permit).
async fn new_code(state: &AppState) -> ApiResult<FreshCode> {
    state
        .argon2(|| {
            let code = recovery::generate();
            recovery::hash(&code.secret).map(|hash| FreshCode {
                display: code.display,
                id: code.id,
                hash,
            })
        })
        .await?
        .ok_or_else(ApiError::internal)
}

/// Body of `POST /api/register/start`.
#[derive(Debug, Default, Deserialize)]
pub struct RegisterStart {
    /// Optional display name as building-block ids ([`names`]).
    #[serde(default)]
    pub name: Option<String>,
}

/// `POST /api/register/start`: options for `navigator.credentials.create`.
pub async fn register_start(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
    body: Option<Json<RegisterStart>>,
) -> ApiResult<Json<Value>> {
    state.limiter.check(ip, Bucket::Auth)?;
    let name = match body.and_then(|Json(b)| b.name) {
        Some(raw) => names::parse(&raw)?,
        None => None,
    };
    let user_id = Uuid::new_v4();
    let (rp, webauthn) = state.rp(&headers);
    let (options, reg) = webauthn
        .start_passkey_registration(user_id, LABEL, LABEL, None)
        .map_err(passkey_error)?;
    let mut options = to_json(&options)?;
    require_resident_key(&mut options);
    let ceremony = state.put_ceremony(
        ip,
        Ceremony::Register {
            rp,
            user_id,
            name,
            state: reg,
        },
    )?;
    Ok(Json(json!({ "ceremony": ceremony, "options": options })))
}

/// Body of the finish calls.
#[derive(Debug, Deserialize)]
pub struct Finish<T> {
    /// Id from the start call.
    pub ceremony: String,
    /// What the browser returned.
    pub credential: T,
}

/// `POST /api/register/finish`: create the account, sign in, return the
/// recovery code (the only time it is shown).
pub async fn register_finish(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    Json(body): Json<Finish<RegisterPublicKeyCredential>>,
) -> ApiResult<impl IntoResponse> {
    state.limiter.check(ip, Bucket::Auth)?;
    let Some(Ceremony::Register {
        rp,
        user_id,
        name,
        state: reg,
    }) = state.take_ceremony(&body.ceremony)
    else {
        return Err(ApiError::bad_request("expired"));
    };
    let (rp, webauthn) = state.rp_at(rp);
    let passkey = webauthn
        .finish_passkey_registration(&body.credential, &reg)
        .map_err(passkey_error)?;
    // Each account may store a save of 1 MB: only a few new ones per
    // address and hour (counted when one is really about to be created).
    state.limiter.check(ip, Bucket::SignUp)?;
    let code = new_code(&state).await?;
    let id = user_id.to_string();
    {
        let mut conn = state.db();
        let tx = conn.transaction()?;
        db::insert_user(&tx, &id, name.as_deref(), &code.id, &code.hash)?;
        store_passkey(&tx, &id, &passkey, state.rp_id(rp))?;
        tx.commit()?;
    }
    let started = session::start(&state, &id)?;
    let user = user_json(&state, &id)?;
    Ok((
        StatusCode::CREATED,
        started.headers,
        Json(json!({ "user": user, "recoveryCode": code.display })),
    ))
}

fn store_passkey(
    conn: &rusqlite::Connection,
    user_id: &str,
    passkey: &Passkey,
    rp_id: &str,
) -> ApiResult<()> {
    let cred_id = URL_SAFE_NO_PAD.encode(passkey.cred_id());
    let json = serde_json::to_string(passkey).map_err(|_| ApiError::internal())?;
    db::insert_passkey(conn, &cred_id, user_id, &json, rp_id).map_err(|err| {
        if let rusqlite::Error::SqliteFailure(e, _) = &err
            && e.code == rusqlite::ErrorCode::ConstraintViolation
        {
            return ApiError::new(StatusCode::CONFLICT, "passkey_exists");
        }
        ApiError::from(err)
    })
}

/// `POST /api/login/start`: options for `navigator.credentials.get`.
pub async fn login_start(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
) -> ApiResult<Json<Value>> {
    state.limiter.check(ip, Bucket::Auth)?;
    let (rp, webauthn) = state.rp(&headers);
    let (options, auth) = webauthn
        .start_discoverable_authentication()
        .map_err(passkey_error)?;
    let mut options = to_json(&options)?;
    // A button press, not the autofill flow: no conditional mediation.
    if let Some(o) = options.as_object_mut() {
        o.remove("mediation");
    }
    let ceremony = state.put_ceremony(ip, Ceremony::Login { rp, state: auth })?;
    Ok(Json(json!({ "ceremony": ceremony, "options": options })))
}

/// `POST /api/login/finish`: check the assertion, sign in.
pub async fn login_finish(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    Json(body): Json<Finish<PublicKeyCredential>>,
) -> ApiResult<impl IntoResponse> {
    state.limiter.check(ip, Bucket::Auth)?;
    let Some(Ceremony::Login { rp, state: auth }) = state.take_ceremony(&body.ceremony) else {
        return Err(ApiError::bad_request("expired"));
    };
    let (rp, webauthn) = state.rp_at(rp);
    let (user_id, cred_id) = webauthn
        .identify_discoverable_authentication(&body.credential)
        .map_err(passkey_error)?;
    let user_id = user_id.to_string();
    let cred_id = URL_SAFE_NO_PAD.encode(cred_id);
    let stored = db::passkey(&state.db(), &user_id, &cred_id)?
        .ok_or(ApiError::new(StatusCode::UNAUTHORIZED, "unknown_passkey"))?;
    let mut passkey: Passkey = serde_json::from_str(&stored).map_err(|_| ApiError::internal())?;
    let result = webauthn
        .finish_discoverable_authentication_v2(&body.credential, auth, &passkey)
        .map_err(passkey_error)?;
    if passkey.update_credential(&result) == Some(true) {
        let json = serde_json::to_string(&passkey).map_err(|_| ApiError::internal())?;
        db::update_passkey(&state.db(), &cred_id, &json)?;
    }
    // Passkeys from before the RP id was stored learn it on their next use.
    db::note_passkey_rp(&state.db(), &cred_id, state.rp_id(rp))?;
    let started = session::start(&state, &user_id)?;
    let user = user_json(&state, &user_id)?;
    Ok((started.headers, Json(json!({ "user": user }))))
}

/// Body of `POST /api/recover`.
#[derive(Debug, Deserialize)]
pub struct Recover {
    /// The recovery code as typed.
    pub code: String,
}

/// A hash to verify against when the code's id is unknown, so a wrong id
/// takes as long as a wrong secret. Computing it is argon2 work: call it
/// only on a blocking thread ([`warm_up`] does it once at startup).
fn dummy_hash() -> &'static str {
    static HASH: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    HASH.get_or_init(|| recovery::hash("0000000000000000").unwrap_or_default())
}

/// Compute the dummy hash before the first request needs it. Blocking:
/// run it with `spawn_blocking`.
pub fn warm_up() {
    let _ = dummy_hash();
}

/// `POST /api/recover`: sign in with the recovery code. The code is used
/// up; the answer carries the new one. The app then offers to add a
/// passkey. Every other session of the account ends: whoever had the
/// code before (a thief, too) is signed out.
pub async fn recover(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    Json(body): Json<Recover>,
) -> ApiResult<impl IntoResponse> {
    state.limiter.check(ip, Bucket::Recover)?;
    let (id, secret) = recovery::parse(&body.code).ok_or(ApiError::bad_request("code"))?;
    let found = db::user_by_recovery_id(&state.db(), &id)?;
    let (user_id, hash) = match found {
        Some((user, hash)) => (Some(user), Some(hash)),
        None => (None, None),
    };
    let checked = hash.clone();
    let ok = state
        .argon2(move || {
            let hash = checked.as_deref().unwrap_or_else(|| dummy_hash());
            recovery::verify(&secret, hash)
        })
        .await?;
    let (Some(user_id), Some(old_hash), true) = (user_id, hash, ok) else {
        return Err(ApiError::new(StatusCode::UNAUTHORIZED, "code"));
    };
    let code = new_code(&state).await?;
    // Only one of two requests with the same code at once gets through.
    if !db::swap_recovery(&state.db(), &user_id, &old_hash, &code.id, &code.hash)? {
        return Err(ApiError::new(StatusCode::UNAUTHORIZED, "code"));
    }
    let started = session::start(&state, &user_id)?;
    db::delete_other_sessions(&state.db(), &user_id, Some(&started.token_hash))?;
    let user = user_json(&state, &user_id)?;
    Ok((
        started.headers,
        Json(json!({ "user": user, "recoveryCode": code.display })),
    ))
}

/// `POST /api/passkey/start`: options to add a passkey to the signed-in account.
pub async fn passkey_start(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
) -> ApiResult<Json<Value>> {
    state.limiter.check(ip, Bucket::Auth)?;
    let session = session::require(&state, &headers)?;
    let user_id = Uuid::parse_str(&session.user_id).map_err(|_| ApiError::internal())?;
    let existing = {
        let conn = state.db();
        db::user(&conn, &session.user_id)?.ok_or_else(ApiError::unauthorized)?;
        db::passkeys(&conn, &session.user_id)?
    };
    let exclude = existing
        .iter()
        .filter_map(|j| serde_json::from_str::<Passkey>(j).ok())
        .map(|p| p.cred_id().clone())
        .collect::<Vec<_>>();
    let (rp, webauthn) = state.rp(&headers);
    let (options, reg) = webauthn
        .start_passkey_registration(user_id, LABEL, LABEL, Some(exclude))
        .map_err(passkey_error)?;
    let mut options = to_json(&options)?;
    require_resident_key(&mut options);
    let ceremony = state.put_ceremony(
        ip,
        Ceremony::AddPasskey {
            rp,
            user_id,
            state: reg,
        },
    )?;
    Ok(Json(json!({ "ceremony": ceremony, "options": options })))
}

/// `POST /api/passkey/finish`: store the new passkey.
pub async fn passkey_finish(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
    Json(body): Json<Finish<RegisterPublicKeyCredential>>,
) -> ApiResult<Json<Value>> {
    state.limiter.check(ip, Bucket::Auth)?;
    let session = session::require(&state, &headers)?;
    let Some(Ceremony::AddPasskey {
        rp,
        user_id,
        state: reg,
    }) = state.take_ceremony(&body.ceremony)
    else {
        return Err(ApiError::bad_request("expired"));
    };
    if user_id.to_string() != session.user_id {
        return Err(ApiError::unauthorized());
    }
    let (rp, webauthn) = state.rp_at(rp);
    let passkey = webauthn
        .finish_passkey_registration(&body.credential, &reg)
        .map_err(passkey_error)?;
    store_passkey(&state.db(), &session.user_id, &passkey, state.rp_id(rp))?;
    Ok(Json(
        json!({ "user": user_json(&state, &session.user_id)? }),
    ))
}

/// `POST /api/recovery-code`: replace the recovery code, show the new
/// one. Every other session of the account ends (a renewed code is what a
/// player does who fears the old one is known).
pub async fn new_recovery_code(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
) -> ApiResult<Json<Value>> {
    state.limiter.check(ip, Bucket::Auth)?;
    let session = session::require(&state, &headers)?;
    let old_hash =
        db::recovery_hash(&state.db(), &session.user_id)?.ok_or_else(ApiError::unauthorized)?;
    let code = new_code(&state).await?;
    if !db::swap_recovery(
        &state.db(),
        &session.user_id,
        &old_hash,
        &code.id,
        &code.hash,
    )? {
        // Renewed or redeemed elsewhere in the meantime.
        return Err(ApiError::new(StatusCode::CONFLICT, "code_changed"));
    }
    db::delete_other_sessions(&state.db(), &session.user_id, Some(&session.token_hash))?;
    Ok(Json(json!({ "recoveryCode": code.display })))
}

/// `GET /api/passkeys`: the account's passkeys, oldest first:
/// `{"passkeys": [{id, createdAt, lastUsedAt, synced, rpId}]}`. `synced`
/// says whether the authenticator backs it up (iCloud keychain, Google
/// password manager). `rpId` is the address it opens (null for one made
/// before that was stored and not used since). There is no name: the
/// authenticator does not tell one.
pub async fn list_passkeys(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
) -> ApiResult<Json<Value>> {
    state.limiter.check(ip, Bucket::Read)?;
    let session = session::require(&state, &headers)?;
    let rows = db::passkey_rows(&state.db(), &session.user_id)?;
    let list = rows
        .iter()
        .map(|row| {
            // The stored JSON of webauthn-rs' Passkey: {"cred": {…, "backup_state": bool}}.
            let synced = serde_json::from_str::<Value>(&row.passkey)
                .ok()
                .and_then(|v| v["cred"]["backup_state"].as_bool());
            json!({
                "id": row.cred_id,
                "createdAt": row.created_at,
                "lastUsedAt": row.last_used_at,
                "synced": synced,
                "rpId": row.rp_id,
            })
        })
        .collect::<Vec<_>>();
    Ok(Json(json!({ "passkeys": list })))
}

/// `DELETE /api/passkeys/{id}`: remove a passkey of the account (a lost
/// phone, one somebody else added). The last one stays (409
/// `last_passkey`): without it only the recovery code would open the
/// account. Sessions are not touched; "sign out everywhere" ends them.
pub async fn remove_passkey(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> ApiResult<Json<Value>> {
    state.limiter.check(ip, Bucket::Write)?;
    let session = session::require(&state, &headers)?;
    match db::remove_passkey(&state.db(), &session.user_id, &id)? {
        db::RemovePasskey::Removed => {}
        db::RemovePasskey::NotFound => return Err(ApiError::not_found()),
        db::RemovePasskey::Last => {
            return Err(ApiError::new(StatusCode::CONFLICT, "last_passkey"));
        }
    }
    Ok(Json(
        json!({ "user": user_json(&state, &session.user_id)? }),
    ))
}

/// `POST /api/logout-all`: end every session of the account, this one
/// too, and remove the cookie.
pub async fn logout_all(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
) -> ApiResult<impl IntoResponse> {
    state.limiter.check(ip, Bucket::Write)?;
    let session = session::require(&state, &headers)?;
    let n = db::delete_other_sessions(&state.db(), &session.user_id, None)?;
    tracing::info!(n, "signed out everywhere");
    let mut out = HeaderMap::new();
    out.insert(
        SET_COOKIE,
        session::clear_cookie(state.config.secure_cookie),
    );
    Ok((StatusCode::NO_CONTENT, out))
}

/// `POST /api/logout`: end the session, remove the cookie.
pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<impl IntoResponse> {
    if let Some(token) = session::token_of(&headers) {
        db::delete_session(&state.db(), &session::token_hash(&token))?;
    }
    let mut out = HeaderMap::new();
    out.insert(
        SET_COOKIE,
        session::clear_cookie(state.config.secure_cookie),
    );
    Ok((StatusCode::NO_CONTENT, out))
}
