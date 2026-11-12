//! Bug reports from the app ("Fehler melden", ADR 0027).
//!
//! `POST /api/reports` takes a report without sign-in; with a session the
//! account id is noted. The report is an opaque text the app puts
//! together (what the player wrote, the diagnosis buffer, optionally the
//! save code): the server stores it with the time, the platform, the
//! app's version and a short id (`B-7K3Q`) and never reads it, just like
//! the save (ADR 0021). There is no API to read reports: Claude fetches
//! them on the server with `stammtisch-server reports` (a GitHub workflow
//! over SSH, `.github/workflows/bug-reports.yml`). Reports older than
//! [`KEEP_DAYS`] days are deleted by the hourly housekeeping.

use argon2::password_hash::rand_core::{OsRng, RngCore};
use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use rusqlite::Connection;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::error::{ApiError, ApiResult};
use crate::session::{self, Bucket, ClientIp};
use crate::{AppState, daily, db};

/// Largest body of `POST /api/reports`.
pub const REPORT_BODY_BYTES: usize = 256 * 1024;
/// Largest report text in bytes (the body minus the JSON around it).
pub const MAX_REPORT_BYTES: usize = 250_000;
/// Longest platform name (`web`, `ios`, `pwa`).
pub const MAX_PLATFORM_CHARS: usize = 16;
/// Longest version text (`iOS · 1a2b3c4 · 2026-11-12T10:00Z`).
pub const MAX_VERSION_CHARS: usize = 80;
/// Reports are kept this many days.
pub const KEEP_DAYS: i64 = 90;
/// All reports together take at most this much room; beyond it a new one
/// is refused with 507 until old ones have expired.
pub const MAX_TOTAL_BYTES: i64 = 256 * 1024 * 1024;
/// Characters of a report id after `B-` (Crockford base32, 20 bits).
pub const ID_CHARS: usize = 4;

const DAY_MS: i64 = 24 * 60 * 60 * 1000;
const CROCKFORD: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// A new random report id, `B-` and four Crockford base32 characters.
pub fn new_id() -> String {
    let mut bytes = [0u8; ID_CHARS];
    OsRng.fill_bytes(&mut bytes);
    let tail: String = bytes
        .iter()
        .map(|b| char::from(CROCKFORD[usize::from(b & 31)]))
        .collect();
    format!("B-{tail}")
}

/// Whether a text is a report id as [`new_id`] makes them.
pub fn is_id(text: &str) -> bool {
    text.strip_prefix("B-")
        .is_some_and(|tail| tail.len() == ID_CHARS && tail.bytes().all(|b| CROCKFORD.contains(&b)))
}

/// The platform as stored: lowercase letters, digits and `-`, 1 to 16.
fn clean_platform(raw: &str) -> Option<String> {
    let p = raw.trim().to_ascii_lowercase();
    let ok = (1..=MAX_PLATFORM_CHARS).contains(&p.len())
        && p.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-');
    ok.then_some(p)
}

/// The version as stored: printable, without control characters, trimmed,
/// 1 to 80 characters.
fn clean_version(raw: &str) -> Option<String> {
    let v: String = raw.chars().filter(|c| !c.is_control()).collect();
    let v = v.trim().to_owned();
    (1..=MAX_VERSION_CHARS)
        .contains(&v.chars().count())
        .then_some(v)
}

/// Body of `POST /api/reports`.
#[derive(Debug, Deserialize)]
pub struct NewReport {
    /// `web`, `ios`, … (what the app says it runs on).
    pub platform: String,
    /// The app's build (commit and date).
    pub version: String,
    /// The report itself, opaque to the server.
    pub report: String,
}

/// `POST /api/reports`: store a report, answer `{"id": "B-7K3Q"}` (201).
/// No sign-in needed; with a session the account id is noted.
pub async fn submit(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
    Json(body): Json<NewReport>,
) -> ApiResult<impl IntoResponse> {
    state.limiter.check(ip, Bucket::Report)?;
    let platform = clean_platform(&body.platform).ok_or(ApiError::bad_request("platform"))?;
    let version = clean_version(&body.version).ok_or(ApiError::bad_request("version"))?;
    if body.report.trim().is_empty() || body.report.len() > MAX_REPORT_BYTES {
        return Err(ApiError::bad_request("report"));
    }
    let user = session::current(&state, &headers)?.map(|s| s.user_id);
    let conn = state.db();
    if db::reports_bytes(&conn)? + i64::try_from(body.report.len()).unwrap_or(i64::MAX)
        > MAX_TOTAL_BYTES
    {
        tracing::warn!("report store full");
        return Err(ApiError::storage_full());
    }
    let now = db::now_ms();
    // 20 bits: a clash is rare while a few hundred reports are kept;
    // another id then.
    for _ in 0..8 {
        let id = new_id();
        if db::insert_report(
            &conn,
            &id,
            now,
            &platform,
            &version,
            user.as_deref(),
            &body.report,
        )? {
            tracing::info!(%id, %platform, bytes = body.report.len(), "report stored");
            return Ok((StatusCode::CREATED, Json(json!({ "id": id }))));
        }
    }
    Err(ApiError::busy())
}

/// Delete reports older than [`KEEP_DAYS`] days (by `now`, ms).
pub fn purge(conn: &Connection, now: i64) -> rusqlite::Result<usize> {
    db::purge_reports(conn, now - KEEP_DAYS * DAY_MS)
}

/// `JJJJ-MM-TTTHH:MM:SSZ` of a time in ms (UTC).
pub fn iso(ms: i64) -> String {
    let day = ms.div_euclid(DAY_MS);
    let seed = daily::seed_of_day(day);
    let secs = ms.rem_euclid(DAY_MS) / 1000;
    format!(
        "{}-{}-{}T{:02}:{:02}:{:02}Z",
        &seed[..4],
        &seed[4..6],
        &seed[6..],
        secs / 3600,
        secs / 60 % 60,
        secs % 60
    )
}

/// A `--since` for the export, as ms: `JJJJ-MM-TT`, `JJJJ-MM-TTTHH:MM`
/// (UTC), `<n>d` or `<n>h` back from `now`, or plain milliseconds.
pub fn parse_since(text: &str, now: i64) -> Option<i64> {
    let text = text.trim();
    if let Some(n) = text.strip_suffix('d').and_then(|n| n.parse::<i64>().ok()) {
        return (0..=3650).contains(&n).then(|| now - n * DAY_MS);
    }
    if let Some(n) = text.strip_suffix('h').and_then(|n| n.parse::<i64>().ok()) {
        return (0..=87_600).contains(&n).then(|| now - n * 3_600_000);
    }
    if text.len() >= 10 && text.bytes().all(|b| b.is_ascii_digit()) {
        return text.parse().ok();
    }
    let (date, time) = match text.split_once('T') {
        Some((d, t)) => (d, Some(t.trim_end_matches('Z'))),
        None => (text, None),
    };
    let parts: Vec<&str> = date.split('-').collect();
    let [y, m, d] = parts.as_slice() else {
        return None;
    };
    if y.len() != 4 || m.len() != 2 || d.len() != 2 {
        return None;
    }
    let day = daily::parse_day(&format!("{y}{m}{d}"))?;
    let mut ms = day * DAY_MS;
    if let Some(time) = time {
        let (h, min) = time.split_once(':')?;
        let h: i64 = h.parse().ok()?;
        let min: i64 = min.get(..2)?.parse().ok()?;
        if !(0..24).contains(&h) || !(0..60).contains(&min) {
            return None;
        }
        ms += h * 3_600_000 + min * 60_000;
    }
    Some(ms)
}

/// The reports as JSON for the export (`stammtisch-server reports`), the
/// oldest first: one report by `id`, or all since `since` (ms), at most
/// `limit`. `report` stays the text the app sent.
pub fn export(
    conn: &Connection,
    since: Option<i64>,
    id: Option<&str>,
    limit: usize,
) -> rusqlite::Result<Value> {
    let rows = db::reports(conn, since.unwrap_or(0), id, limit)?;
    Ok(Value::Array(
        rows.into_iter()
            .map(|r| {
                json!({
                    "id": r.id,
                    "createdAt": r.created_at,
                    "created": iso(r.created_at),
                    "platform": r.platform,
                    "version": r.version,
                    "account": r.user_id,
                    "bytes": r.data.len(),
                    "report": r.data,
                })
            })
            .collect(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_look_right() {
        for _ in 0..200 {
            let id = new_id();
            assert!(is_id(&id), "{id}");
            assert_eq!(id.len(), 6);
        }
        assert!(!is_id("B-7K3"));
        assert!(!is_id("B-7K3I")); // no I in Crockford base32
        assert!(!is_id("b-7K3Q"));
        assert!(!is_id("B-7K3Q1"));
        assert!(is_id("B-7K3Q"));
    }

    #[test]
    fn platform_and_version_are_cleaned() {
        assert_eq!(clean_platform(" iOS ").as_deref(), Some("ios"));
        assert_eq!(clean_platform("web"), Some("web".into()));
        assert_eq!(clean_platform(""), None);
        assert_eq!(clean_platform("web; drop"), None);
        assert_eq!(clean_platform(&"x".repeat(17)), None);
        assert_eq!(
            clean_version(" iOS · 1a2b3c4\n").as_deref(),
            Some("iOS · 1a2b3c4")
        );
        assert_eq!(clean_version("  "), None);
        assert_eq!(clean_version(&"v".repeat(81)), None);
    }

    #[test]
    fn times_are_read_and_written() {
        let now = 1_762_905_600_000; // 2025-11-12T00:00:00Z
        assert_eq!(iso(now), "2025-11-12T00:00:00Z");
        assert_eq!(iso(now + 3_723_000), "2025-11-12T01:02:03Z");
        assert_eq!(parse_since("2025-11-12", 0), Some(now));
        assert_eq!(parse_since("2025-11-12T01:02", 0), Some(now + 3_720_000));
        assert_eq!(parse_since("7d", now), Some(now - 7 * DAY_MS));
        assert_eq!(parse_since("24h", now), Some(now - DAY_MS));
        assert_eq!(parse_since("1762905600000", 0), Some(now));
        assert_eq!(parse_since("2025-13-01", 0), None);
        assert_eq!(parse_since("gestern", 0), None);
        assert_eq!(parse_since("2025-11-12T25:00", 0), None);
    }
}
