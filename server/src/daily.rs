//! The daily leaderboard: everyone plays the same run per day (the daily
//! seed is the date, `JJJJMMTT`). A signed-in player's first finished
//! daily run counts. The result is what the app reports; the server does
//! not replay or check it (ADR 0021: no game logic on the server).

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::error::{ApiError, ApiResult};
use crate::session::{self, Bucket, ClientIp};
use crate::{AppState, db};

/// Entries of the board by default and at most.
const DEFAULT_LIMIT: i64 = 10;
const MAX_LIMIT: i64 = 50;
/// Days a result may lie from the server's date (time zones, a run over midnight).
const DAY_SLACK: i64 = 2;

/// Days since 1970-01-01 of a civil date (proleptic Gregorian).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// The day number of a daily seed `JJJJMMTT`, if it is a real date.
pub fn parse_day(day: &str) -> Option<i64> {
    if day.len() != 8 || !day.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let y: i64 = day[..4].parse().ok()?;
    let m: i64 = day[4..6].parse().ok()?;
    let d: i64 = day[6..].parse().ok()?;
    if !(2000..=2999).contains(&y) || !(1..=12).contains(&m) || d < 1 {
        return None;
    }
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let len = match m {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    (d <= len).then(|| days_from_civil(y, m, d))
}

fn today() -> i64 {
    db::now_ms().div_euclid(24 * 60 * 60 * 1000)
}

/// The daily seed of a day number (inverse of [`parse_day`]).
pub fn seed_of_day(days: i64) -> String {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}{m:02}{d:02}")
}

/// Today's daily seed by the server's clock (UTC).
pub fn today_seed() -> String {
    seed_of_day(today())
}

/// Body of `POST /api/daily`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Submit {
    /// The daily seed (`JJJJMMTT`).
    pub day: String,
    /// Character id of the core.
    pub character: String,
    /// Run won.
    pub won: bool,
    /// District reached, from 1.
    pub section: i64,
    /// Pub reached in it, from 1.
    pub series: i64,
    /// Stake of the Einsatz-Leiter.
    #[serde(default)]
    pub stake: i64,
    /// Best game value of the run.
    pub best_game: i64,
}

fn entry_json(row: &db::DailyRow, me: Option<&str>) -> Value {
    let r = &row.result;
    json!({
        "rank": row.rank,
        "name": row.name,
        "character": r.character,
        "won": r.won,
        "section": r.section,
        "series": r.series,
        "stake": r.stake,
        "bestGame": r.best_game,
        "me": me == Some(row.user_id.as_str()),
    })
}

/// `POST /api/daily`: report the result of today's daily run.
pub async fn submit(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
    Json(body): Json<Submit>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    state.limiter.check(ip, Bucket::Write)?;
    let session = session::require(&state, &headers)?;
    let day = parse_day(&body.day).ok_or(ApiError::bad_request("day"))?;
    if (day - today()).abs() > DAY_SLACK {
        return Err(ApiError::bad_request("day"));
    }
    let character_ok = (1..=32).contains(&body.character.len())
        && body.character.bytes().all(|b| b.is_ascii_alphabetic());
    let in_range = (1..=99).contains(&body.section)
        && (1..=99).contains(&body.series)
        && (0..=9999).contains(&body.stake)
        && (0..=1_000_000_000).contains(&body.best_game);
    if !character_ok || !in_range {
        return Err(ApiError::bad_request("result"));
    }
    let result = db::DailyResult {
        day: body.day.clone(),
        character: body.character,
        won: body.won,
        section: body.section,
        series: body.series,
        stake: body.stake,
        best_game: body.best_game,
    };
    let conn = state.db();
    let stored = db::insert_daily(&conn, &session.user_id, &result)?;
    let mine = db::daily_of(&conn, &body.day, &session.user_id)?;
    let status = if stored {
        StatusCode::CREATED
    } else {
        StatusCode::OK
    };
    Ok((
        status,
        Json(json!({
            "stored": stored,
            "me": mine.as_ref().map(|r| entry_json(r, Some(&session.user_id))),
        })),
    ))
}

/// Query of `GET /api/daily/{day}`.
#[derive(Debug, Deserialize)]
pub struct BoardQuery {
    /// How many entries (1 to 50, default 10).
    pub limit: Option<i64>,
}

/// `GET /api/daily/{day}`: the best of the day and, signed in, the own place.
pub async fn board(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
    Path(day): Path<String>,
    Query(q): Query<BoardQuery>,
) -> ApiResult<Json<Value>> {
    state.limiter.check(ip, Bucket::Read)?;
    parse_day(&day).ok_or(ApiError::bad_request("day"))?;
    let limit = q.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let me = session::current(&state, &headers)?.map(|s| s.user_id);
    let conn = state.db();
    let top = db::daily_top(&conn, &day, limit)?;
    let total = db::daily_count(&conn, &day)?;
    let mine = match &me {
        Some(id) => db::daily_of(&conn, &day, id)?,
        None => None,
    };
    Ok(Json(json!({
        "day": day,
        "total": total,
        "entries": top.iter().map(|r| entry_json(r, me.as_deref())).collect::<Vec<_>>(),
        "me": mine.as_ref().map(|r| entry_json(r, me.as_deref())),
        "signedIn": me.is_some(),
    })))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn days_parse() {
        assert_eq!(parse_day("19700101"), None); // before 2000
        assert_eq!(parse_day("20000101"), Some(10_957));
        assert_eq!(parse_day("20261107"), Some(20_764));
        assert_eq!(parse_day("20240229"), Some(19_782));
        assert_eq!(parse_day("20250229"), None);
        assert_eq!(parse_day("20261131"), None);
        assert_eq!(parse_day("2026117"), None);
        assert_eq!(parse_day("2026-11-07"), None);
        for day in ["20000101", "20240229", "20261107", "20991231"] {
            assert_eq!(seed_of_day(parse_day(day).unwrap()), day);
        }
    }
}
