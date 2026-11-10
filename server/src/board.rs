//! The leaderboards (ADR 0021): the Tafel des Tages and the Tafel der
//! Woche (DESIGN 5.6). Both work the same: a signed-in player reports the
//! result of the period's run; the first result per player and period
//! counts, every further one is practice and stays off the board. The
//! result is what the app reports; the server does not replay or check it
//! (ADR 0021: no game logic on the server). What differs per board is the
//! period: its key (`JJJJMMTT`, `JJJJWW`), how far a result may lie from
//! the server's current period, and its table ([`db::Board`]).

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::db::Board;
use crate::error::{ApiError, ApiResult};
use crate::session::{self, Bucket, ClientIp};
use crate::{AppState, daily, db, weekly};

/// Entries of a board by default and at most.
const DEFAULT_LIMIT: i64 = 10;
const MAX_LIMIT: i64 = 50;

impl Board {
    /// The key's name in requests and answers.
    const fn json_key(self) -> &'static str {
        match self {
            Board::Daily => "day",
            Board::Weekly => "week",
        }
    }

    /// The period of a key as an ordinal (day or week number), if the key
    /// names a real period.
    fn ordinal(self, key: &str) -> Option<i64> {
        match self {
            Board::Daily => daily::parse_day(key),
            Board::Weekly => weekly::parse_week(key),
        }
    }

    /// The server's current period (UTC), as an ordinal.
    fn current(self) -> i64 {
        match self {
            Board::Daily => daily::today(),
            Board::Weekly => weekly::this_week(),
        }
    }

    /// How many periods a result may lie from the server's (time zones,
    /// a run over midnight or into the next week).
    const fn slack(self) -> i64 {
        match self {
            Board::Daily => 2,
            Board::Weekly => 1,
        }
    }
}

/// Body of `POST /api/daily` and `POST /api/weekly`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Submit {
    /// The period: the daily seed (`JJJJMMTT`, as `day`) or the week
    /// (`JJJJWW`, as `week`).
    #[serde(alias = "day", alias = "week")]
    pub period: String,
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

fn entry_json(row: &db::BoardRow, me: Option<&str>) -> Value {
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

/// The checked result of a submit.
fn checked(board: Board, body: Submit) -> ApiResult<db::BoardResult> {
    let period = board
        .ordinal(&body.period)
        .ok_or(ApiError::bad_request(board.json_key()))?;
    if (period - board.current()).abs() > board.slack() {
        return Err(ApiError::bad_request(board.json_key()));
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
    Ok(db::BoardResult {
        period: body.period,
        character: body.character,
        won: body.won,
        section: body.section,
        series: body.series,
        stake: body.stake,
        best_game: body.best_game,
    })
}

/// Report the result of the period's run on `board`.
pub async fn submit(
    board: Board,
    state: AppState,
    ip: std::net::IpAddr,
    headers: &HeaderMap,
    body: Submit,
) -> ApiResult<(StatusCode, Json<Value>)> {
    state.limiter.check(ip, Bucket::Write)?;
    let session = session::require(&state, headers)?;
    let result = checked(board, body)?;
    let conn = state.db();
    let stored = db::insert_result(&conn, board, &session.user_id, &result)?;
    let mine = db::result_of(&conn, board, &result.period, &session.user_id)?;
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

/// Query of `GET /api/daily/{day}` and `GET /api/weekly/{week}`.
#[derive(Debug, Deserialize)]
pub struct BoardQuery {
    /// How many entries (1 to 50, default 10).
    pub limit: Option<i64>,
}

/// The best of a period and, signed in, the own place.
pub async fn show(
    board: Board,
    state: AppState,
    ip: std::net::IpAddr,
    headers: &HeaderMap,
    period: String,
    q: BoardQuery,
) -> ApiResult<Json<Value>> {
    state.limiter.check(ip, Bucket::Read)?;
    board
        .ordinal(&period)
        .ok_or(ApiError::bad_request(board.json_key()))?;
    let limit = q.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let me = session::current(&state, headers)?.map(|s| s.user_id);
    let conn = state.db();
    let top = db::board_top(&conn, board, &period, limit)?;
    let total = db::board_count(&conn, board, &period)?;
    let mine = match &me {
        Some(id) => db::result_of(&conn, board, &period, id)?,
        None => None,
    };
    let mut out = json!({
        "total": total,
        "entries": top.iter().map(|r| entry_json(r, me.as_deref())).collect::<Vec<_>>(),
        "me": mine.as_ref().map(|r| entry_json(r, me.as_deref())),
        "signedIn": me.is_some(),
    });
    out[board.json_key()] = Value::String(period);
    Ok(Json(out))
}

/// `POST /api/daily`: report the result of today's daily run.
pub async fn submit_daily(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
    Json(body): Json<Submit>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    submit(Board::Daily, state, ip, &headers, body).await
}

/// `GET /api/daily/{day}`: the best of the day and, signed in, the own place.
pub async fn show_daily(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
    Path(day): Path<String>,
    Query(q): Query<BoardQuery>,
) -> ApiResult<Json<Value>> {
    show(Board::Daily, state, ip, &headers, day, q).await
}

/// `POST /api/weekly`: report the result of this week's run.
pub async fn submit_weekly(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
    Json(body): Json<Submit>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    submit(Board::Weekly, state, ip, &headers, body).await
}

/// `GET /api/weekly/{week}`: the best of the week and, signed in, the own
/// place.
pub async fn show_weekly(
    State(state): State<AppState>,
    ClientIp(ip): ClientIp,
    headers: HeaderMap,
    Path(week): Path<String>,
    Query(q): Query<BoardQuery>,
) -> ApiResult<Json<Value>> {
    show(Board::Weekly, state, ip, &headers, week, q).await
}
