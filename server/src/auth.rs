//! Sign-up, sign-in and the recovery code.
//!
//! - Sign-up creates a discoverable passkey (resident key), so signing in
//!   needs no user name: the authenticator offers the passkeys it has for
//!   this site and returns the user handle (our user id).
//! - The recovery code is shown once at sign-up and replaced by a new one
//!   whenever it is used or renewed.

use axum::Json;
use axum::extract::State;
use axum::http::header::SET_COOKIE;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::Deserialize;
use serde_json::{Value, json};
use webauthn_rs::prelude::{Passkey, PublicKeyCredential, RegisterPublicKeyCredential, Uuid};

use crate::account::{clean_name, user_json};
use crate::error::{ApiError, ApiResult};
use crate::session::{self, Bucket, ClientIp};
use crate::{AppState, Ceremony, db, recovery};

/// Label of a passkey in the password manager when no name is set.
const DEFAULT_LABEL: &str = "Stammtisch-Spieler";

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

/// Make a recovery code and hash its secret (off the async threads).
async fn new_code() -> ApiResult<FreshCode> {
    tokio::task::spawn_blocking(|| {
        let code = recovery::generate();
        recovery::hash(&code.secret).map(|hash| FreshCode {
            display: code.display,
            id: code.id,
            hash,
        })
    })
    .await
    .ok()
    .flatten()
    .ok_or_else(ApiError::internal)
}

/// Body of `POST /api/register/start`.
#[derive(Debug, Default, Deserialize)]
pub struct RegisterStart {
    /// Optional display name.
    #[serde(default)]
    pub name: Option<String>,
}

/// `POST /api/register/start`: options for `navigator.credentials.create`.
pub async fn register_start(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    body: Option<Json<RegisterStart>>,
) -> ApiResult<Json<Value>> {
    state.limiter.check(ip, Bucket::Auth)?;
    let name = match body.and_then(|Json(b)| b.name) {
        Some(raw) if !raw.trim().is_empty() => {
            Some(clean_name(&raw).ok_or_else(|| ApiError::bad_request("name"))?)
        }
        _ => None,
    };
    let user_id = Uuid::new_v4();
    let label = name.clone().unwrap_or_else(|| DEFAULT_LABEL.to_owned());
    let (options, reg) = state
        .webauthn
        .start_passkey_registration(user_id, &label, &label, None)
        .map_err(passkey_error)?;
    let mut options = to_json(&options)?;
    require_resident_key(&mut options);
    let ceremony = state.put_ceremony(Ceremony::Register {
        user_id,
        name,
        state: reg,
    })?;
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
        user_id,
        name,
        state: reg,
    }) = state.take_ceremony(&body.ceremony)
    else {
        return Err(ApiError::bad_request("expired"));
    };
    let passkey = state
        .webauthn
        .finish_passkey_registration(&body.credential, &reg)
        .map_err(passkey_error)?;
    let code = new_code().await?;
    let id = user_id.to_string();
    {
        let mut conn = state.db();
        let tx = conn.transaction()?;
        db::insert_user(&tx, &id, name.as_deref(), &code.id, &code.hash)?;
        store_passkey(&tx, &id, &passkey)?;
        tx.commit()?;
    }
    let headers = session::start(&state, &id)?;
    let user = user_json(&state, &id)?;
    Ok((
        StatusCode::CREATED,
        headers,
        Json(json!({ "user": user, "recoveryCode": code.display })),
    ))
}

fn store_passkey(conn: &rusqlite::Connection, user_id: &str, passkey: &Passkey) -> ApiResult<()> {
    let cred_id = URL_SAFE_NO_PAD.encode(passkey.cred_id());
    let json = serde_json::to_string(passkey).map_err(|_| ApiError::internal())?;
    db::insert_passkey(conn, &cred_id, user_id, &json).map_err(|err| {
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
) -> ApiResult<Json<Value>> {
    state.limiter.check(ip, Bucket::Auth)?;
    let (options, auth) = state
        .webauthn
        .start_discoverable_authentication()
        .map_err(passkey_error)?;
    let mut options = to_json(&options)?;
    // A button press, not the autofill flow: no conditional mediation.
    if let Some(o) = options.as_object_mut() {
        o.remove("mediation");
    }
    let ceremony = state.put_ceremony(Ceremony::Login { state: auth })?;
    Ok(Json(json!({ "ceremony": ceremony, "options": options })))
}

/// `POST /api/login/finish`: check the assertion, sign in.
pub async fn login_finish(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    Json(body): Json<Finish<PublicKeyCredential>>,
) -> ApiResult<impl IntoResponse> {
    state.limiter.check(ip, Bucket::Auth)?;
    let Some(Ceremony::Login { state: auth }) = state.take_ceremony(&body.ceremony) else {
        return Err(ApiError::bad_request("expired"));
    };
    let (user_id, cred_id) = state
        .webauthn
        .identify_discoverable_authentication(&body.credential)
        .map_err(passkey_error)?;
    let user_id = user_id.to_string();
    let cred_id = URL_SAFE_NO_PAD.encode(cred_id);
    let stored = db::passkey(&state.db(), &user_id, &cred_id)?
        .ok_or(ApiError::new(StatusCode::UNAUTHORIZED, "unknown_passkey"))?;
    let mut passkey: Passkey = serde_json::from_str(&stored).map_err(|_| ApiError::internal())?;
    let result = state
        .webauthn
        .finish_discoverable_authentication_v2(&body.credential, auth, &passkey)
        .map_err(passkey_error)?;
    if passkey.update_credential(&result) == Some(true) {
        let json = serde_json::to_string(&passkey).map_err(|_| ApiError::internal())?;
        db::update_passkey(&state.db(), &cred_id, &json)?;
    }
    let headers = session::start(&state, &user_id)?;
    let user = user_json(&state, &user_id)?;
    Ok((headers, Json(json!({ "user": user }))))
}

/// Body of `POST /api/recover`.
#[derive(Debug, Deserialize)]
pub struct Recover {
    /// The recovery code as typed.
    pub code: String,
}

/// A hash to verify against when the code's id is unknown, so a wrong id
/// takes as long as a wrong secret.
fn dummy_hash() -> &'static str {
    static HASH: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    HASH.get_or_init(|| recovery::hash("0000000000000000").unwrap_or_default())
}

/// `POST /api/recover`: sign in with the recovery code. The code is used
/// up; the answer carries the new one. The app then offers to add a passkey.
pub async fn recover(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    Json(body): Json<Recover>,
) -> ApiResult<impl IntoResponse> {
    state.limiter.check(ip, Bucket::Recover)?;
    let (id, secret) = recovery::parse(&body.code).ok_or(ApiError::bad_request("code"))?;
    let found = db::user_by_recovery_id(&state.db(), &id)?;
    let (user_id, hash) = match found {
        Some((user, hash)) => (Some(user), hash),
        None => (None, dummy_hash().to_owned()),
    };
    let ok = tokio::task::spawn_blocking(move || recovery::verify(&secret, &hash))
        .await
        .unwrap_or(false);
    let Some(user_id) = user_id.filter(|_| ok) else {
        return Err(ApiError::new(StatusCode::UNAUTHORIZED, "code"));
    };
    let code = new_code().await?;
    db::set_recovery(&state.db(), &user_id, &code.id, &code.hash)?;
    let headers = session::start(&state, &user_id)?;
    let user = user_json(&state, &user_id)?;
    Ok((
        headers,
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
    let (name, existing) = {
        let conn = state.db();
        let user = db::user(&conn, &session.user_id)?.ok_or_else(ApiError::unauthorized)?;
        (user.name, db::passkeys(&conn, &session.user_id)?)
    };
    let exclude = existing
        .iter()
        .filter_map(|j| serde_json::from_str::<Passkey>(j).ok())
        .map(|p| p.cred_id().clone())
        .collect::<Vec<_>>();
    let label = name.unwrap_or_else(|| DEFAULT_LABEL.to_owned());
    let (options, reg) = state
        .webauthn
        .start_passkey_registration(user_id, &label, &label, Some(exclude))
        .map_err(passkey_error)?;
    let mut options = to_json(&options)?;
    require_resident_key(&mut options);
    let ceremony = state.put_ceremony(Ceremony::AddPasskey {
        user_id,
        state: reg,
    })?;
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
        user_id,
        state: reg,
    }) = state.take_ceremony(&body.ceremony)
    else {
        return Err(ApiError::bad_request("expired"));
    };
    if user_id.to_string() != session.user_id {
        return Err(ApiError::unauthorized());
    }
    let passkey = state
        .webauthn
        .finish_passkey_registration(&body.credential, &reg)
        .map_err(passkey_error)?;
    store_passkey(&state.db(), &session.user_id, &passkey)?;
    Ok(Json(
        json!({ "user": user_json(&state, &session.user_id)? }),
    ))
}

/// `POST /api/recovery-code`: replace the recovery code, show the new one.
pub async fn new_recovery_code(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
) -> ApiResult<Json<Value>> {
    state.limiter.check(ip, Bucket::Auth)?;
    let session = session::require(&state, &headers)?;
    let code = new_code().await?;
    db::set_recovery(&state.db(), &session.user_id, &code.id, &code.hash)?;
    Ok(Json(json!({ "recoveryCode": code.display })))
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
