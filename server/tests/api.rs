//! Endpoint tests against an in-memory database. Passkey ceremonies need a
//! real authenticator (checked in the browser with Chromium's virtual
//! authenticator); here accounts are created directly in the database.

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use stammtisch_server::{AppState, Config, daily, db, recovery, router, session};
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
    let res = call(&app, "GET", "/api/me", None, None).await;
    assert_eq!(res.status, StatusCode::UNAUTHORIZED);
    assert_eq!(res.body["error"], "unauthorized");
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

    let res = call(
        &app,
        "POST",
        "/api/recover",
        None,
        Some(json!({ "code": "kein code" })),
    )
    .await;
    assert_eq!(res.status, StatusCode::BAD_REQUEST);

    // Renewing the code while signed in.
    let res = call(&app, "POST", "/api/recovery-code", Some(&cookie), None).await;
    assert_eq!(res.status, StatusCode::OK);
    assert_eq!(res.body["recoveryCode"].as_str().map(str::len), Some(29));
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

    let res = call(&app, "DELETE", "/api/account", Some(&cookie), None).await;
    assert_eq!(res.status, StatusCode::NO_CONTENT);
    assert_eq!(
        call(&app, "GET", "/api/me", Some(&cookie), None)
            .await
            .status,
        StatusCode::UNAUTHORIZED
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
        db::daily_count(&conn, &daily::today_seed()).expect("count"),
        0
    );
}
