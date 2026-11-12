//! Endpoint tests against an in-memory database. Passkey ceremonies need a
//! real authenticator (checked in the browser with Chromium's virtual
//! authenticator); here accounts are created directly in the database.

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use stammtisch_server::{AppState, Config, daily, db, recovery, router, session, weekly};
use tower::ServiceExt;

const ORIGIN: &str = "https://stammtisch.example";

fn app() -> (Router, AppState) {
    let config = Config::new(&[ORIGIN], None).expect("config");
    let state = AppState::new(config, db::open_in_memory().expect("db")).expect("state");
    (router(state.clone()), state)
}

/// A user with a known recovery code and a session; returns the cookie.
fn user(state: &AppState, id: &str, name: Option<&str>) -> (String, String) {
    let code = recovery::generate();
    let hash = recovery::hash(&code.secret).expect("hash");
    let token = session::new_token();
    let conn = state.db();
    db::insert_user(&conn, id, name, &code.id, &hash).expect("user");
    db::insert_session(
        &conn,
        &session::token_hash(&token),
        id,
        db::now_ms() + 3_600_000,
    )
    .expect("session");
    (format!("{}={token}", session::COOKIE_NAME), code.display)
}

struct Res {
    status: StatusCode,
    headers: axum::http::HeaderMap,
    body: Value,
}

async fn call(
    app: &Router,
    method: &str,
    uri: &str,
    cookie: Option<&str>,
    body: Option<Value>,
) -> Res {
    let mut req = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::ORIGIN, ORIGIN);
    if let Some(c) = cookie {
        req = req.header(header::COOKIE, c);
    }
    let req = match body {
        Some(b) => req
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(b.to_string())),
        None => req.body(Body::empty()),
    }
    .expect("request");
    let res = app.clone().oneshot(req).await.expect("response");
    let status = res.status();
    let headers = res.headers().clone();
    let bytes = res.into_body().collect().await.expect("body").to_bytes();
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    Res {
        status,
        headers,
        body,
    }
}

fn cookie_of(res: &Res) -> String {
    let set = res
        .headers
        .get(header::SET_COOKIE)
        .and_then(|v| v.to_str().ok())
        .expect("set-cookie");
    assert!(set.contains("HttpOnly"));
    assert!(set.contains("SameSite=Lax"));
    assert!(set.contains("Secure"));
    set.split(';').next().expect("pair").to_owned()
}

#[tokio::test]
async fn health_and_no_session() {
    let (app, _) = app();
    let res = call(&app, "GET", "/api/health", None, None).await;
    assert_eq!(res.status, StatusCode::OK);
    // Nobody signed in is an answer, not an error.
    let res = call(&app, "GET", "/api/me", None, None).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.body, json!({ "user": null }));
    let res = call(
        &app,
        "GET",
        "/api/save",
        Some("stammtisch_session=nope"),
        None,
    )
    .await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn register_start_asks_for_a_discoverable_passkey() {
    let (app, _) = app();
    let res = call(
        &app,
        "POST",
        "/api/register/start",
        None,
        Some(json!({ "name": "  Jan " })),
    )
    .await;
    assert_eq!(res.status, StatusCode::OK);
    let pk = &res.body["options"]["publicKey"];
    assert_eq!(pk["rp"]["id"], "stammtisch.example");
    assert_eq!(pk["user"]["name"], "Jan");
    assert_eq!(pk["authenticatorSelection"]["residentKey"], "required");
    assert_eq!(pk["authenticatorSelection"]["userVerification"], "required");
    assert!(res.body["ceremony"].as_str().is_some_and(|c| c.len() > 20));
    // A bad name is refused before anything starts.
    let res = call(
        &app,
        "POST",
        "/api/register/start",
        None,
        Some(json!({ "name": "x".repeat(30) })),
    )
    .await;
    assert_eq!(res.status, StatusCode::BAD_REQUEST);
    assert_eq!(res.body["error"], "name");
}

#[tokio::test]
async fn finish_without_ceremony_is_refused() {
    let (app, _) = app();
    let credential = json!({
        "id": "AAAA", "rawId": "AAAA", "type": "public-key", "extensions": {},
        "response": { "attestationObject": "AAAA", "clientDataJSON": "AAAA" }
    });
    let res = call(
        &app,
        "POST",
        "/api/register/finish",
        None,
        Some(json!({ "ceremony": "unknown", "credential": credential })),
    )
    .await;
    assert_eq!(res.status, StatusCode::BAD_REQUEST);
    assert_eq!(res.body["error"], "expired");
}

#[tokio::test]
async fn login_start_is_modal_and_discoverable() {
    let (app, _) = app();
    let res = call(&app, "POST", "/api/login/start", None, None).await;
    assert_eq!(res.status, StatusCode::OK);
    assert!(res.body["options"].get("mediation").is_none());
    let pk = &res.body["options"]["publicKey"];
    assert_eq!(pk["rpId"], "stammtisch.example");
    assert!(pk["challenge"].as_str().is_some());
    assert_eq!(
        pk["allowCredentials"].as_array().map(Vec::len).unwrap_or(0),
        0
    );
}

#[tokio::test]
async fn foreign_origin_cannot_change_anything() {
    let (app, state) = app();
    let (cookie, _) = user(&state, "u1", None);
    let req = Request::builder()
        .method("PUT")
        .uri("/api/me/name")
        .header(header::ORIGIN, "https://evil.example")
        .header(header::COOKIE, &cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"name":"Mallory"}"#))
        .expect("request");
    let res = app.clone().oneshot(req).await.expect("response");
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn name_is_set_cleaned_and_removed() {
    let (app, state) = app();
    let (cookie, _) = user(&state, "u1", None);
    let res = call(&app, "GET", "/api/me", Some(&cookie), None).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.body["user"]["name"], Value::Null);
    assert_eq!(res.body["user"]["passkeys"], 0);
    assert_eq!(
        res.headers
            .get(header::CACHE_CONTROL)
            .and_then(|v| v.to_str().ok()),
        Some("no-store")
    );

    let res = call(
        &app,
        "PUT",
        "/api/me/name",
        Some(&cookie),
        Some(json!({ "name": "  Der  alte\u{0007} Fritz " })),
    )
    .await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.body["user"]["name"], "Der alte Fritz");
    let res = call(
        &app,
        "PUT",
        "/api/me/name",
        Some(&cookie),
        Some(json!({ "name": "y".repeat(25) })),
    )
    .await;
    assert_eq!(res.status, StatusCode::BAD_REQUEST);
    let res = call(
        &app,
        "PUT",
        "/api/me/name",
        Some(&cookie),
        Some(json!({ "name": "  " })),
    )
    .await;
    assert_eq!(res.body["user"]["name"], Value::Null);
}

#[tokio::test]
async fn save_uses_optimistic_versions() {
    let (app, state) = app();
    let (cookie, _) = user(&state, "u1", None);
    let res = call(&app, "GET", "/api/save", Some(&cookie), None).await;
    assert_eq!(res.body, json!({ "save": null }));

    let put = |data: &str, if_version: Value, updated_at: i64| json!({ "data": data, "ifVersion": if_version, "updatedAt": updated_at });
    let res = call(
        &app,
        "PUT",
        "/api/save",
        Some(&cookie),
        Some(put("STAMMTISCH1:a", Value::Null, 1000)),
    )
    .await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.body["version"], 1);

    // A second device that has not seen version 1 must not overwrite it.
    let res = call(
        &app,
        "PUT",
        "/api/save",
        Some(&cookie),
        Some(put("STAMMTISCH1:b", Value::Null, 2000)),
    )
    .await;
    assert_eq!(res.status, StatusCode::CONFLICT);
    assert_eq!(res.body["error"], "conflict");
    assert_eq!(res.body["save"]["version"], 1);
    assert_eq!(res.body["save"]["updatedAt"], 1000);
    let res = call(
        &app,
        "PUT",
        "/api/save",
        Some(&cookie),
        Some(put("STAMMTISCH1:b", json!(7), 2000)),
    )
    .await;
    assert_eq!(res.status, StatusCode::CONFLICT);

    let res = call(
        &app,
        "PUT",
        "/api/save",
        Some(&cookie),
        Some(put("STAMMTISCH1:c", json!(1), 3000)),
    )
    .await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.body["version"], 2);
    let res = call(&app, "GET", "/api/save", Some(&cookie), None).await;
    assert_eq!(
        res.body["save"],
        json!({ "version": 2, "updatedAt": 3000, "data": "STAMMTISCH1:c" })
    );

    // Too large and empty saves are refused; nothing changes.
    let big = "x".repeat(1_000_001);
    let res = call(
        &app,
        "PUT",
        "/api/save",
        Some(&cookie),
        Some(put(&big, json!(2), 4000)),
    )
    .await;
    assert_eq!(res.status, StatusCode::PAYLOAD_TOO_LARGE);
    let res = call(
        &app,
        "PUT",
        "/api/save",
        Some(&cookie),
        Some(put("", json!(2), 4000)),
    )
    .await;
    assert_eq!(res.status, StatusCode::BAD_REQUEST);

    // Another player sees only their own save.
    let (other, _) = user(&state, "u2", None);
    let res = call(&app, "GET", "/api/save", Some(&other), None).await;
    assert_eq!(res.body, json!({ "save": null }));
}

#[tokio::test]
async fn recovery_code_signs_in_once_and_is_replaced() {
    let (app, state) = app();
    let (_, code) = user(&state, "u1", Some("Jan"));
    let res = call(
        &app,
        "POST",
        "/api/recover",
        None,
        Some(json!({ "code": code.to_lowercase() })),
    )
    .await;
    assert_eq!(res.status, StatusCode::OK, "{:?}", res.body);
    assert_eq!(res.body["user"]["name"], "Jan");
    let cookie = cookie_of(&res);
    let fresh = res.body["recoveryCode"]
        .as_str()
        .expect("new code")
        .to_owned();
    assert_ne!(fresh, code);

    let res = call(&app, "GET", "/api/me", Some(&cookie), None).await;
    assert_eq!(res.status, StatusCode::OK);

    // The used code is gone, the new one works.
    let res = call(
        &app,
        "POST",
        "/api/recover",
        None,
        Some(json!({ "code": code })),
    )
    .await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
    assert_eq!(res.body["error"], "code");
    let res = call(
        &app,
        "POST",
        "/api/recover",
        None,
        Some(json!({ "code": fresh })),
    )
    .await;
    assert_eq!(res.status, StatusCode::OK);
    // Signing in with the code ends every other session: whoever used the
    // code before is out.
    let old_cookie = cookie;
    let cookie = cookie_of(&res);
    let res = call(&app, "GET", "/api/me", Some(&old_cookie), None).await;
    assert_eq!(res.body["user"], Value::Null);

    let res = call(
        &app,
        "POST",
        "/api/recover",
        None,
        Some(json!({ "code": "kein code" })),
    )
    .await;
    assert_eq!(res.status, StatusCode::BAD_REQUEST);

    // Renewing the code while signed in ends the other sessions, not this one.
    let other = second_session(&state, "u1");
    let res = call(&app, "POST", "/api/recovery-code", Some(&cookie), None).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.body["recoveryCode"].as_str().map(str::len), Some(29));
    let res = call(&app, "GET", "/api/me", Some(&cookie), None).await;
    assert_eq!(res.body["user"]["name"], "Jan");
    let res = call(&app, "GET", "/api/me", Some(&other), None).await;
    assert_eq!(res.body["user"], Value::Null);
}

/// Another session of a user (a second device); returns its cookie.
fn second_session(state: &AppState, id: &str) -> String {
    let token = session::new_token();
    db::insert_session(
        &state.db(),
        &session::token_hash(&token),
        id,
        db::now_ms() + 100 * 24 * 3_600_000,
    )
    .expect("session");
    format!("{}={token}", session::COOKIE_NAME)
}

#[tokio::test]
async fn session_cookie_is_refreshed_with_the_session() {
    let (app, state) = app();
    // `user` makes a session that ends in an hour: past half its life.
    let (cookie, _) = user(&state, "u1", None);
    let res = call(&app, "GET", "/api/me", Some(&cookie), None).await;
    assert_eq!(res.status, StatusCode::OK);
    let set = res
        .headers
        .get(header::SET_COOKIE)
        .and_then(|v| v.to_str().ok())
        .expect("refreshed cookie");
    assert!(set.starts_with(&format!("{cookie};")), "{set}");
    assert!(set.contains(&format!("Max-Age={}", session::SESSION_DAYS * 24 * 60 * 60)));
    let token = cookie.split_once('=').expect("pair").1;
    let (_, expires) = db::session_user(&state.db(), &session::token_hash(token))
        .expect("query")
        .expect("session");
    assert!(expires > db::now_ms() + (session::SESSION_DAYS - 1) * 24 * 3_600_000);

    // A session with most of its life ahead is left alone.
    let res = call(&app, "GET", "/api/me", Some(&cookie), None).await;
    assert!(res.headers.get(header::SET_COOKIE).is_none());
    // Logging out keeps its own cookie (removal), not a refresh.
    let (old, _) = user(&state, "u2", None);
    let res = call(&app, "POST", "/api/logout", Some(&old), None).await;
    let set = res
        .headers
        .get(header::SET_COOKIE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(set.contains("Max-Age=0"), "{set}");
}

#[tokio::test]
async fn passkeys_are_listed_and_removed_but_not_the_last() {
    let (app, state) = app();
    let (cookie, _) = user(&state, "u1", None);
    {
        let conn = state.db();
        db::insert_passkey(
            &conn,
            "credA",
            "u1",
            r#"{"cred":{"backup_state":true}}"#,
            "stammtisch.example",
        )
        .expect("a");
        db::insert_passkey(&conn, "credB", "u1", "{}", "stammtisch.example").expect("b");
    }
    let (other, _) = user(&state, "u2", None);
    db::insert_passkey(&state.db(), "credC", "u2", "{}", "stammtisch.example").expect("c");

    let res = call(&app, "GET", "/api/passkeys", Some(&cookie), None).await;
    assert_eq!(res.status, StatusCode::OK);
    let list = res.body["passkeys"].as_array().expect("list");
    assert_eq!(list.len(), 2);
    assert_eq!(list[0]["id"], "credA");
    assert_eq!(list[0]["synced"], true);
    assert!(list[0]["createdAt"].as_i64().is_some());
    assert_eq!(list[0]["lastUsedAt"], Value::Null);
    assert_eq!(list[1]["synced"], Value::Null);

    // Somebody else's passkey is not found.
    let res = call(&app, "DELETE", "/api/passkeys/credC", Some(&cookie), None).await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);
    assert_eq!(res.body["error"], "not_found");
    let res = call(&app, "DELETE", "/api/passkeys/credA", Some(&cookie), None).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.body["user"]["passkeys"], 1);
    let res = call(&app, "DELETE", "/api/passkeys/credB", Some(&cookie), None).await;
    assert_eq!(res.status, StatusCode::CONFLICT);
    assert_eq!(res.body["error"], "last_passkey");
    let res = call(&app, "GET", "/api/passkeys", Some(&other), None).await;
    assert_eq!(res.body["passkeys"].as_array().map(Vec::len), Some(1));
    let res = call(&app, "GET", "/api/passkeys", None, None).await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn logout_all_ends_every_session() {
    let (app, state) = app();
    let (cookie, _) = user(&state, "u1", None);
    let other = second_session(&state, "u1");
    let (stranger, _) = user(&state, "u2", None);
    let res = call(&app, "POST", "/api/logout-all", Some(&cookie), None).await;
    assert_eq!(res.status, StatusCode::NO_CONTENT);
    let set = res
        .headers
        .get(header::SET_COOKIE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(set.contains("Max-Age=0"));
    for c in [&cookie, &other] {
        let res = call(&app, "GET", "/api/me", Some(c), None).await;
        assert_eq!(res.body["user"], Value::Null);
    }
    // Other accounts are untouched.
    let res = call(&app, "GET", "/api/me", Some(&stranger), None).await;
    assert!(res.body["user"].is_object());
    let res = call(&app, "POST", "/api/logout-all", None, None).await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
}

async fn raw(app: &Router, req: Request<Body>) -> Res {
    let res = app.clone().oneshot(req).await.expect("response");
    let status = res.status();
    let headers = res.headers().clone();
    let bytes = res.into_body().collect().await.expect("body").to_bytes();
    Res {
        status,
        headers,
        body: serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    }
}

#[tokio::test]
async fn changes_need_our_origin_and_no_foreign_fetch_site() {
    let (app, state) = app();
    let (cookie, _) = user(&state, "u1", None);
    let put = |origin: Option<&str>, site: Option<&str>| {
        let mut req = Request::builder()
            .method("PUT")
            .uri("/api/me/name")
            .header(header::COOKIE, &cookie)
            .header(header::CONTENT_TYPE, "application/json");
        if let Some(o) = origin {
            req = req.header(header::ORIGIN, o);
        }
        if let Some(s) = site {
            req = req.header("sec-fetch-site", s);
        }
        req.body(Body::from(r#"{"name":"Jan"}"#)).expect("request")
    };
    // No Origin on a change: refused (browsers always send one).
    let res = raw(&app, put(None, None)).await;
    assert_eq!(res.status, StatusCode::FORBIDDEN);
    assert_eq!(res.body["error"], "origin");
    // Our Origin, but the browser says another site (or a sibling host
    // under sslip.io) made the request.
    for site in ["cross-site", "same-site"] {
        let res = raw(&app, put(Some(ORIGIN), Some(site))).await;
        assert_eq!(res.status, StatusCode::FORBIDDEN, "{site}");
    }
    let res = raw(&app, put(Some(ORIGIN), Some("same-origin"))).await;
    assert_eq!(res.status, StatusCode::OK);
    // Reads need no Origin.
    let req = Request::builder()
        .uri("/api/me")
        .header(header::COOKIE, &cookie)
        .header("sec-fetch-site", "cross-site")
        .body(Body::empty())
        .expect("request");
    assert_eq!(raw(&app, req).await.status, StatusCode::OK);
}

#[tokio::test]
async fn bodies_are_limited_and_errors_are_json() {
    let (app, state) = app();
    let (cookie, _) = user(&state, "u1", None);
    // 16 KiB for small requests.
    let res = call(
        &app,
        "PUT",
        "/api/me/name",
        Some(&cookie),
        Some(json!({ "name": "x".repeat(20_000) })),
    )
    .await;
    assert_eq!(res.status, StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(res.body["error"], "too_large");
    // A save may be much larger.
    let data = format!("STAMMTISCH1:{}", "x".repeat(500_000));
    let res = call(
        &app,
        "PUT",
        "/api/save",
        Some(&cookie),
        Some(json!({ "data": data, "ifVersion": null, "updatedAt": 1 })),
    )
    .await;
    assert_eq!(res.status, StatusCode::OK, "{:?}", res.body);
    // Broken JSON, missing fields, a wrong content type, unknown paths:
    // all JSON errors the app can read.
    let req = Request::builder()
        .method("PUT")
        .uri("/api/me/name")
        .header(header::ORIGIN, ORIGIN)
        .header(header::COOKIE, &cookie)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from("{nope"))
        .expect("request");
    let res = raw(&app, req).await;
    assert_eq!(res.status, StatusCode::BAD_REQUEST);
    assert_eq!(res.body["error"], "bad_request");
    let res = call(&app, "PUT", "/api/me/name", Some(&cookie), Some(json!({}))).await;
    assert_eq!(res.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(res.body["error"], "invalid");
    let req = Request::builder()
        .method("PUT")
        .uri("/api/me/name")
        .header(header::ORIGIN, ORIGIN)
        .header(header::COOKIE, &cookie)
        .body(Body::from(r#"{"name":"Jan"}"#))
        .expect("request");
    let res = raw(&app, req).await;
    assert_eq!(res.status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert_eq!(res.body["error"], "content_type");
    let res = call(&app, "GET", "/api/nothing", None, None).await;
    assert_eq!(res.status, StatusCode::NOT_FOUND);
    assert_eq!(res.body["error"], "not_found");
}

#[tokio::test]
async fn saves_must_look_like_save_codes() {
    let (app, state) = app();
    let (cookie, _) = user(&state, "u1", None);
    let res = call(
        &app,
        "PUT",
        "/api/save",
        Some(&cookie),
        Some(json!({ "data": "just some data", "ifVersion": null, "updatedAt": 1 })),
    )
    .await;
    assert_eq!(res.status, StatusCode::BAD_REQUEST);
    assert_eq!(res.body["error"], "not_a_save");
    // A newer format is kept as it is (an older app must not lose it).
    let res = call(
        &app,
        "PUT",
        "/api/save",
        Some(&cookie),
        Some(json!({ "data": "STAMMTISCH2:z:abc", "ifVersion": null, "updatedAt": 1 })),
    )
    .await;
    assert_eq!(res.status, StatusCode::OK);
}

#[tokio::test]
async fn a_full_database_answers_507() {
    let (app, state) = app();
    let (cookie, _) = user(&state, "u1", None);
    db::set_max_bytes(&state.db(), 128 * 1024).expect("cap");
    let data = format!("STAMMTISCH1:{}", "x".repeat(400_000));
    let res = call(
        &app,
        "PUT",
        "/api/save",
        Some(&cookie),
        Some(json!({ "data": data, "ifVersion": null, "updatedAt": 1 })),
    )
    .await;
    assert_eq!(res.status, StatusCode::INSUFFICIENT_STORAGE);
    assert_eq!(res.body["error"], "storage_full");
    // Small things still work.
    let res = call(&app, "GET", "/api/me", Some(&cookie), None).await;
    assert_eq!(res.status, StatusCode::OK);
}

#[tokio::test]
async fn one_client_cannot_hold_all_ceremonies() {
    let (app, _) = app();
    let start = |ip: &str| {
        Request::builder()
            .method("POST")
            .uri("/api/login/start")
            .header(header::ORIGIN, ORIGIN)
            .header("x-client-ip", ip)
            .body(Body::empty())
            .expect("request")
    };
    for _ in 0..10 {
        assert_eq!(raw(&app, start("2001:db8::1")).await.status, StatusCode::OK);
    }
    // The same /64 is the same client.
    let res = raw(&app, start("2001:db8::2")).await;
    assert_eq!(res.status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(res.body["error"], "rate_limited");
    assert_eq!(
        raw(&app, start("2001:db8:0:1::1")).await.status,
        StatusCode::OK
    );
}

#[tokio::test]
async fn recovery_is_rate_limited() {
    let (app, _) = app();
    let mut last = StatusCode::OK;
    for _ in 0..12 {
        last = call(
            &app,
            "POST",
            "/api/recover",
            None,
            Some(json!({ "code": "0000-0000-0000-0000-0000-0000" })),
        )
        .await
        .status;
    }
    assert_eq!(last, StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn logout_ends_the_session() {
    let (app, state) = app();
    let (cookie, _) = user(&state, "u1", None);
    let res = call(&app, "POST", "/api/logout", Some(&cookie), None).await;
    assert_eq!(res.status, StatusCode::NO_CONTENT);
    let set = res
        .headers
        .get(header::SET_COOKIE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(set.contains("Max-Age=0"));
    let res = call(&app, "GET", "/api/me", Some(&cookie), None).await;
    assert_eq!(res.body["user"], Value::Null);
    let res = call(&app, "GET", "/api/save", Some(&cookie), None).await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
}

fn result(won: bool, section: i64, series: i64, best: i64) -> Value {
    json!({ "day": daily::today_seed(), "character": "Declarer", "won": won, "section": section, "series": series, "stake": 0, "bestGame": best })
}

#[tokio::test]
async fn daily_board_ranks_results() {
    let (app, state) = app();
    let (a, _) = user(&state, "a", Some("Anna"));
    let (b, _) = user(&state, "b", Some("Bert"));
    let (c, _) = user(&state, "c", None);
    let day = daily::today_seed();

    assert_eq!(
        call(
            &app,
            "POST",
            "/api/daily",
            Some(&a),
            Some(result(false, 2, 3, 48))
        )
        .await
        .status,
        StatusCode::CREATED
    );
    assert_eq!(
        call(
            &app,
            "POST",
            "/api/daily",
            Some(&b),
            Some(result(true, 4, 5, 96))
        )
        .await
        .status,
        StatusCode::CREATED
    );
    let res = call(
        &app,
        "POST",
        "/api/daily",
        Some(&c),
        Some(result(false, 2, 3, 72)),
    )
    .await;
    assert_eq!(res.status, StatusCode::CREATED);
    assert_eq!(res.body["me"]["rank"], 2);

    // Only the first result of the day counts.
    let res = call(
        &app,
        "POST",
        "/api/daily",
        Some(&a),
        Some(result(true, 4, 5, 200)),
    )
    .await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.body["stored"], false);
    assert_eq!(res.body["me"]["rank"], 3);

    let res = call(&app, "GET", &format!("/api/daily/{day}"), Some(&a), None).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.body["total"], 3);
    let names: Vec<Value> = res.body["entries"]
        .as_array()
        .expect("entries")
        .iter()
        .map(|e| e["name"].clone())
        .collect();
    assert_eq!(names, vec![json!("Bert"), Value::Null, json!("Anna")]);
    assert_eq!(res.body["entries"][2]["me"], true);
    assert_eq!(res.body["me"]["rank"], 3);
    assert_eq!(res.body["signedIn"], true);

    // Read-only without a session, limited.
    let res = call(
        &app,
        "GET",
        &format!("/api/daily/{day}?limit=1"),
        None,
        None,
    )
    .await;
    assert_eq!(res.body["entries"].as_array().map(Vec::len), Some(1));
    assert_eq!(res.body["me"], Value::Null);
    assert_eq!(res.body["signedIn"], false);

    // Bad days and results.
    let mut far = result(true, 1, 1, 1);
    far["day"] = json!("20200101");
    assert_eq!(
        call(&app, "POST", "/api/daily", Some(&a), Some(far))
            .await
            .status,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        call(
            &app,
            "POST",
            "/api/daily",
            Some(&b),
            Some(result(true, 0, 1, 1))
        )
        .await
        .status,
        StatusCode::BAD_REQUEST
    );
    let mut odd = result(true, 1, 1, 1);
    odd["character"] = json!("<script>");
    assert_eq!(
        call(&app, "POST", "/api/daily", Some(&a), Some(odd))
            .await
            .status,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        call(&app, "GET", "/api/daily/2026-11-07", None, None)
            .await
            .status,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        call(
            &app,
            "POST",
            "/api/daily",
            None,
            Some(result(true, 1, 1, 1))
        )
        .await
        .status,
        StatusCode::UNAUTHORIZED
    );
}

fn weekly_result(won: bool, section: i64, series: i64, best: i64) -> Value {
    json!({ "week": weekly::this_week_key(), "character": "Declarer", "won": won, "section": section, "series": series, "stake": 0, "bestGame": best })
}

#[tokio::test]
async fn weekly_board_ranks_results_like_the_daily() {
    let (app, state) = app();
    let (a, _) = user(&state, "a", Some("Anna"));
    let (b, _) = user(&state, "b", Some("Bert"));
    let week = weekly::this_week_key();
    let post = |cookie: &str, body: Value| {
        let app = app.clone();
        let cookie = cookie.to_owned();
        async move { call(&app, "POST", "/api/weekly", Some(&cookie), Some(body)).await }
    };
    assert_eq!(
        post(&a, weekly_result(false, 2, 3, 48)).await.status,
        StatusCode::CREATED
    );
    let res = post(&b, weekly_result(true, 3, 5, 96)).await;
    assert_eq!(res.status, StatusCode::CREATED);
    assert_eq!(res.body["me"]["rank"], 1);
    // The first attempt of the week counts; a better one later is practice.
    let res = post(&a, weekly_result(true, 3, 5, 500)).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.body["stored"], false);
    assert_eq!(res.body["me"]["rank"], 2);

    let res = call(&app, "GET", &format!("/api/weekly/{week}"), Some(&a), None).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.body["week"], json!(week));
    assert_eq!(res.body["total"], 2);
    assert_eq!(res.body["entries"][0]["name"], json!("Bert"));
    assert_eq!(res.body["entries"][1]["me"], true);
    // The daily board of today knows nothing of it.
    let day = daily::today_seed();
    let res = call(&app, "GET", &format!("/api/daily/{day}"), Some(&a), None).await;
    assert_eq!(res.body["total"], 0);

    // A week far away, no week, a daily key, a bad result.
    let mut far = weekly_result(true, 1, 1, 1);
    far["week"] = json!("202001");
    assert_eq!(post(&a, far).await.status, StatusCode::BAD_REQUEST);
    let mut odd = weekly_result(true, 1, 1, 1);
    odd["week"] = json!("202699");
    assert_eq!(post(&a, odd).await.status, StatusCode::BAD_REQUEST);
    assert_eq!(
        post(&a, result(true, 1, 1, 1)).await.status,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        post(&b, weekly_result(true, 0, 1, 1)).await.status,
        StatusCode::BAD_REQUEST
    );
    for bad in ["2026-46", "20261110", "202600"] {
        assert_eq!(
            call(&app, "GET", &format!("/api/weekly/{bad}"), None, None)
                .await
                .status,
            StatusCode::BAD_REQUEST,
            "{bad}"
        );
    }
    // Read-only without a session; writing needs one.
    let res = call(
        &app,
        "GET",
        &format!("/api/weekly/{week}?limit=1"),
        None,
        None,
    )
    .await;
    assert_eq!(res.body["entries"].as_array().map(Vec::len), Some(1));
    assert_eq!(res.body["signedIn"], false);
    assert_eq!(
        call(
            &app,
            "POST",
            "/api/weekly",
            None,
            Some(weekly_result(true, 1, 1, 1))
        )
        .await
        .status,
        StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn deleting_the_account_removes_everything() {
    let (app, state) = app();
    let (cookie, code) = user(&state, "u1", Some("Jan"));
    let save = json!({ "data": "STAMMTISCH1:x", "ifVersion": null, "updatedAt": 1 });
    assert_eq!(
        call(&app, "PUT", "/api/save", Some(&cookie), Some(save))
            .await
            .status,
        StatusCode::OK
    );
    assert_eq!(
        call(
            &app,
            "POST",
            "/api/daily",
            Some(&cookie),
            Some(result(true, 4, 5, 96))
        )
        .await
        .status,
        StatusCode::CREATED
    );

    assert_eq!(
        call(
            &app,
            "POST",
            "/api/weekly",
            Some(&cookie),
            Some(weekly_result(true, 4, 5, 96))
        )
        .await
        .status,
        StatusCode::CREATED
    );

    let res = call(&app, "DELETE", "/api/account", Some(&cookie), None).await;
    assert_eq!(res.status, StatusCode::NO_CONTENT);
    assert_eq!(
        call(&app, "GET", "/api/me", Some(&cookie), None).await.body["user"],
        Value::Null
    );
    assert_eq!(
        call(
            &app,
            "POST",
            "/api/recover",
            None,
            Some(json!({ "code": code }))
        )
        .await
        .status,
        StatusCode::UNAUTHORIZED
    );
    let conn = state.db();
    assert!(db::save(&conn, "u1").expect("query").is_none());
    assert_eq!(
        db::board_count(&conn, db::Board::Daily, &daily::today_seed()).expect("count"),
        0
    );
    assert_eq!(
        db::board_count(&conn, db::Board::Weekly, &weekly::this_week_key()).expect("count"),
        0
    );
}

const OLD_ORIGIN: &str = "https://old.example";

/// The server on the new and the old address (ADR 0025).
fn moved_app() -> (Router, AppState) {
    let config = Config::new(&[ORIGIN, OLD_ORIGIN], None).expect("config");
    let state = AppState::new(config, db::open_in_memory().expect("db")).expect("state");
    (router(state.clone()), state)
}

async fn call_from(
    app: &Router,
    origin: &str,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> Res {
    let mut req = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::ORIGIN, origin);
    let req = match body {
        Some(b) => {
            req = req.header(header::CONTENT_TYPE, "application/json");
            req.body(Body::from(b.to_string()))
        }
        None => req.body(Body::empty()),
    }
    .expect("request");
    let res = app.clone().oneshot(req).await.expect("response");
    let status = res.status();
    let headers = res.headers().clone();
    let bytes = res.into_body().collect().await.expect("body").to_bytes();
    Res {
        status,
        headers,
        body: serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    }
}

#[tokio::test]
async fn each_address_is_its_own_relying_party() {
    let (app, state) = moved_app();
    assert_eq!(state.config.rp_id, "stammtisch.example");
    let ids: Vec<&str> = state.config.rps.iter().map(|rp| rp.id.as_str()).collect();
    assert_eq!(ids, ["stammtisch.example", "old.example"]);
    // Passkeys of the old address keep working there, new ones are made
    // for the address the player is on.
    for (origin, rp) in [(ORIGIN, "stammtisch.example"), (OLD_ORIGIN, "old.example")] {
        let res = call_from(&app, origin, "POST", "/api/register/start", Some(json!({}))).await;
        assert_eq!(res.status, StatusCode::OK, "{origin}");
        assert_eq!(res.body["options"]["publicKey"]["rp"]["id"], rp);
        let res = call_from(&app, origin, "POST", "/api/login/start", None).await;
        assert_eq!(res.status, StatusCode::OK, "{origin}");
        assert_eq!(res.body["options"]["publicKey"]["rpId"], rp);
    }
    // A foreign origin is still refused.
    let res = call_from(
        &app,
        "https://evil.example",
        "POST",
        "/api/login/start",
        None,
    )
    .await;
    assert_eq!(res.status, StatusCode::FORBIDDEN);
    // One RP id given: it holds every origin, as before.
    let config = Config::new(
        &["https://a.example", "https://b.a.example"],
        Some("a.example"),
    )
    .expect("config");
    assert_eq!(config.rps.len(), 1);
    assert!(config.origin_allowed("https://b.a.example"));
}

#[tokio::test]
async fn move_code_signs_the_same_account_in_once() {
    let (app, state) = moved_app();
    let (cookie, _) = user(&state, "u1", Some("Jan"));
    // Only signed in.
    let res = call_from(&app, OLD_ORIGIN, "POST", "/api/move/start", None).await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
    let make = |cookie: String| {
        let app = app.clone();
        async move {
            let req = Request::builder()
                .method("POST")
                .uri("/api/move/start")
                .header(header::ORIGIN, OLD_ORIGIN)
                .header(header::COOKIE, cookie)
                .body(Body::empty())
                .expect("request");
            let res = app.oneshot(req).await.expect("response");
            assert_eq!(res.status(), StatusCode::OK);
            let bytes = res.into_body().collect().await.expect("body").to_bytes();
            serde_json::from_slice::<Value>(&bytes).expect("json")
        }
    };
    let first = make(cookie.clone()).await;
    let first_code = first["code"].as_str().expect("code").to_owned();
    assert_eq!(first_code.len(), 43);
    let left = first["expiresAt"].as_i64().expect("expiry") - db::now_ms();
    assert!(left > 9 * 60 * 1000 && left <= 10 * 60 * 1000, "{left}");
    // A new code replaces the old one.
    let second = make(cookie.clone()).await;
    let code = second["code"].as_str().expect("code").to_owned();
    assert_ne!(code, first_code);
    let res = call_from(
        &app,
        ORIGIN,
        "POST",
        "/api/move/redeem",
        Some(json!({ "code": first_code })),
    )
    .await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
    assert_eq!(res.body["error"], "move_code");
    // Only its hash is stored.
    let stored: String = state
        .db()
        .query_row(
            "SELECT code_hash FROM move_codes WHERE user_id = 'u1'",
            [],
            |r| r.get(0),
        )
        .expect("stored");
    assert_eq!(stored, session::token_hash(&code));

    let res = call_from(
        &app,
        ORIGIN,
        "POST",
        "/api/move/redeem",
        Some(json!({ "code": code })),
    )
    .await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.body["user"]["name"], "Jan");
    let new_cookie = cookie_of(&res);
    let res = call(&app, "GET", "/api/me", Some(&new_cookie), None).await;
    assert_eq!(res.body["user"]["name"], "Jan");
    // The old address stays signed in: moving is no takeover.
    let res = call(&app, "GET", "/api/me", Some(&cookie), None).await;
    assert_eq!(res.body["user"]["name"], "Jan");
    // Used up.
    let res = call_from(
        &app,
        ORIGIN,
        "POST",
        "/api/move/redeem",
        Some(json!({ "code": code })),
    )
    .await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
    for bad in ["", "nope", &"x".repeat(200)] {
        let res = call_from(
            &app,
            ORIGIN,
            "POST",
            "/api/move/redeem",
            Some(json!({ "code": bad })),
        )
        .await;
        assert_eq!(res.status, StatusCode::UNAUTHORIZED, "{bad}");
    }
}

#[tokio::test]
async fn move_code_expires_and_goes_with_the_account() {
    let (app, state) = moved_app();
    let (cookie, _) = user(&state, "u1", None);
    db::put_move_code(
        &state.db(),
        "u1",
        &session::token_hash("late"),
        db::now_ms() - 1,
    )
    .expect("code");
    let res = call_from(
        &app,
        ORIGIN,
        "POST",
        "/api/move/redeem",
        Some(json!({ "code": "late" })),
    )
    .await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
    db::put_move_code(
        &state.db(),
        "u1",
        &session::token_hash("soon"),
        db::now_ms() + 60_000,
    )
    .expect("code");
    let res = call(&app, "DELETE", "/api/account", Some(&cookie), None).await;
    assert!(res.status.is_success(), "{}", res.status);
    let left: i64 = state
        .db()
        .query_row("SELECT COUNT(*) FROM move_codes", [], |r| r.get(0))
        .expect("count");
    assert_eq!(left, 0);
}

#[tokio::test]
async fn passkeys_say_which_address_they_open() {
    let (app, state) = moved_app();
    let (cookie, _) = user(&state, "u1", None);
    {
        let conn = state.db();
        db::insert_passkey(&conn, "new", "u1", "{}", "stammtisch.example").expect("new");
        // One from before the RP id was stored.
        conn.execute(
            "INSERT INTO passkeys (cred_id, user_id, passkey, created_at) VALUES ('legacy', 'u1', '{}', 0)",
            [],
        )
        .expect("legacy");
    }
    let res = call(&app, "GET", "/api/passkeys", Some(&cookie), None).await;
    let list = res.body["passkeys"].as_array().expect("list");
    assert_eq!(list[0]["id"], "legacy");
    assert_eq!(list[0]["rpId"], Value::Null);
    assert_eq!(list[1]["rpId"], "stammtisch.example");
    db::note_passkey_rp(&state.db(), "legacy", "old.example").expect("note");
    db::note_passkey_rp(&state.db(), "new", "old.example").expect("kept");
    let res = call(&app, "GET", "/api/passkeys", Some(&cookie), None).await;
    let list = res.body["passkeys"].as_array().expect("list");
    assert_eq!(list[0]["rpId"], "old.example");
    assert_eq!(list[1]["rpId"], "stammtisch.example");
}

const APP_ORIGIN: &str = "capacitor://localhost";

/// The server with the native app's origin (ADR 0026).
fn app_with_native() -> (Router, AppState) {
    let config = Config::new(&[ORIGIN, OLD_ORIGIN], None)
        .and_then(|c| c.with_app_origins(&[APP_ORIGIN, " "]))
        .expect("config");
    let state = AppState::new(config, db::open_in_memory().expect("db")).expect("state");
    (router(state.clone()), state)
}

/// A request as the app's web view sends it: our app origin, always
/// cross-site, the session (if any) as a bearer token.
fn from_app(method: &str, uri: &str, token: Option<&str>, body: Option<Value>) -> Request<Body> {
    let mut req = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::ORIGIN, APP_ORIGIN)
        .header("sec-fetch-site", "cross-site");
    if let Some(t) = token {
        req = req.header(header::AUTHORIZATION, format!("Bearer {t}"));
    }
    match body {
        Some(b) => req
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(b.to_string())),
        None => req.body(Body::empty()),
    }
    .expect("request")
}

fn token_header(res: &Res) -> Option<String> {
    res.headers
        .get(session::TOKEN_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned)
}

#[test]
fn app_origins_are_checked() {
    let config = || Config::new(&[ORIGIN], None).expect("config");
    assert!(config().with_app_origins(&[ORIGIN]).is_err());
    assert!(
        config()
            .with_app_origins(&["capacitor://localhost/"])
            .is_err()
    );
    assert!(
        config()
            .with_app_origins(&["capacitor://localhost/app"])
            .is_err()
    );
    assert!(config().with_app_origins(&["kein origin"]).is_err());
    let c = config().with_app_origins(&[APP_ORIGIN]).expect("app");
    assert!(c.is_app_origin(APP_ORIGIN));
    assert!(!c.is_app_origin("capacitor://localhost.evil"));
    assert!(!c.origin_allowed(APP_ORIGIN), "the app is no relying party");
}

#[tokio::test]
async fn the_app_gets_cors_and_a_preflight() {
    let (app, _) = app_with_native();
    let res = raw(&app, from_app("OPTIONS", "/api/save", None, None)).await;
    assert_eq!(res.status, StatusCode::NO_CONTENT);
    let h = |name: &str| {
        res.headers
            .get(name)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_owned()
    };
    assert_eq!(h("access-control-allow-origin"), APP_ORIGIN);
    assert!(h("access-control-allow-headers").contains("Authorization"));
    assert!(h("access-control-allow-methods").contains("PUT"));
    assert!(h("access-control-allow-credentials").is_empty());
    // A foreign page gets neither a preflight nor CORS headers.
    let req = Request::builder()
        .method("OPTIONS")
        .uri("/api/save")
        .header(header::ORIGIN, "https://evil.example")
        .body(Body::empty())
        .expect("request");
    let res = raw(&app, req).await;
    assert_ne!(res.status, StatusCode::NO_CONTENT);
    assert!(!res.headers.contains_key("access-control-allow-origin"));
    let res = call(&app, "GET", "/api/health", None, None).await;
    assert!(!res.headers.contains_key("access-control-allow-origin"));
    // Errors carry CORS too, so the app can read them.
    let res = raw(&app, from_app("GET", "/api/save", None, None)).await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
    assert_eq!(res.body["error"], "unauthorized");
    assert!(res.headers.contains_key("access-control-allow-origin"));
}

#[tokio::test]
async fn the_app_signs_in_with_a_bearer_token() {
    let (app, state) = app_with_native();
    let (cookie, code) = user(&state, "u1", Some("Jan"));
    // Signing in from the app: the token comes in a header, no cookie.
    let res = raw(
        &app,
        from_app("POST", "/api/recover", None, Some(json!({ "code": code }))),
    )
    .await;
    assert_eq!(res.status, StatusCode::OK, "{:?}", res.body);
    assert!(!res.headers.contains_key(header::SET_COOKIE));
    let token = token_header(&res).expect("token");
    assert!(token.len() > 20);
    let exposed = res
        .headers
        .get("access-control-expose-headers")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(exposed.eq_ignore_ascii_case(session::TOKEN_HEADER));

    // The token opens the account, also for changes (always cross-site).
    let res = raw(&app, from_app("GET", "/api/me", Some(&token), None)).await;
    assert_eq!(res.body["user"]["name"], "Jan");
    let res = raw(
        &app,
        from_app(
            "PUT",
            "/api/me/name",
            Some(&token),
            Some(json!({ "name": "Uwe" })),
        ),
    )
    .await;
    assert_eq!(res.status, StatusCode::OK, "{:?}", res.body);
    assert_eq!(res.body["user"]["name"], "Uwe");

    // A cookie never counts for the app's origin.
    let mut req = from_app("GET", "/api/me", None, None);
    req.headers_mut()
        .insert(header::COOKIE, cookie.parse().expect("cookie"));
    let res = raw(&app, req).await;
    assert_eq!(res.body["user"], Value::Null);

    // The web pages keep their cookie and get no token.
    let res = call(&app, "POST", "/api/recovery-code", None, None).await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
    assert!(token_header(&res).is_none());

    // Signing out ends the session and says so with an empty token.
    let res = raw(&app, from_app("POST", "/api/logout", Some(&token), None)).await;
    assert!(res.status.is_success(), "{:?}", res.status);
    assert_eq!(token_header(&res).as_deref(), Some(""));
    let res = raw(&app, from_app("GET", "/api/me", Some(&token), None)).await;
    assert_eq!(res.body["user"], Value::Null);
}

#[tokio::test]
async fn the_app_makes_passkeys_for_the_main_address() {
    let (app, _) = app_with_native();
    for path in ["/api/register/start", "/api/login/start"] {
        let res = raw(&app, from_app("POST", path, None, Some(json!({})))).await;
        assert_eq!(res.status, StatusCode::OK, "{path}: {:?}", res.body);
        let pk = &res.body["options"]["publicKey"];
        let rp = pk["rp"]["id"].as_str().or(pk["rpId"].as_str());
        assert_eq!(rp, Some("stammtisch.example"), "{path}");
    }
    // Without the app origin configured, the app cannot change anything.
    let (plain, _) = moved_app();
    let res = raw(&plain, from_app("POST", "/api/login/start", None, None)).await;
    assert_eq!(res.status, StatusCode::FORBIDDEN);
}
