//! Errors of the API: a status code and a short machine-readable code in
//! JSON (`{"error":"conflict"}`). The web app maps the code to German text.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

/// An error answer of the API.
#[derive(Debug)]
pub struct ApiError {
    /// HTTP status.
    pub status: StatusCode,
    /// Stable code for the client.
    pub code: &'static str,
}

impl ApiError {
    /// An error with status and code.
    pub const fn new(status: StatusCode, code: &'static str) -> Self {
        Self { status, code }
    }

    /// 400: the request is malformed or a value is out of range.
    pub const fn bad_request(code: &'static str) -> Self {
        Self::new(StatusCode::BAD_REQUEST, code)
    }

    /// 401: no valid session.
    pub const fn unauthorized() -> Self {
        Self::new(StatusCode::UNAUTHORIZED, "unauthorized")
    }

    /// 404: nothing there.
    pub const fn not_found() -> Self {
        Self::new(StatusCode::NOT_FOUND, "not_found")
    }

    /// 429: too many requests from this address.
    pub const fn too_many() -> Self {
        Self::new(StatusCode::TOO_MANY_REQUESTS, "rate_limited")
    }

    /// 503: the server is too busy right now (all open passkey ceremonies
    /// are taken); not the client's fault, try again soon.
    pub const fn busy() -> Self {
        Self::new(StatusCode::SERVICE_UNAVAILABLE, "busy")
    }

    /// 507: the database reached its size cap (`db::MAX_DB_BYTES`).
    pub const fn storage_full() -> Self {
        Self::new(StatusCode::INSUFFICIENT_STORAGE, "storage_full")
    }

    /// The JSON error for a status that no handler named: what axum's
    /// extractors and the router answer on their own (bad JSON, body too
    /// large, wrong content type, unknown path or method).
    pub fn for_status(status: StatusCode) -> Self {
        let code = match status.as_u16() {
            400 => "bad_request",
            401 => "unauthorized",
            403 => "forbidden",
            404 => "not_found",
            405 => "method",
            413 => "too_large",
            415 => "content_type",
            422 => "invalid",
            429 => "rate_limited",
            500..=599 => "internal",
            _ => "error",
        };
        Self::new(status, code)
    }

    /// 500: something broke on our side (details go to the log only).
    pub const fn internal() -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, "internal")
    }
}

impl From<rusqlite::Error> for ApiError {
    fn from(err: rusqlite::Error) -> Self {
        if let rusqlite::Error::SqliteFailure(e, _) = &err
            && e.code == rusqlite::ErrorCode::DiskFull
        {
            tracing::error!(%err, "database full");
            return Self::storage_full();
        }
        tracing::error!(%err, "database error");
        Self::internal()
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(json!({ "error": self.code }))).into_response()
    }
}

/// Result type of the handlers.
pub type ApiResult<T> = Result<T, ApiError>;
