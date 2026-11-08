//! SQLite storage. One file (`stammtisch.db`), one connection behind a
//! mutex: the load is a handful of players, every statement is short.
//!
//! Tables:
//! - `users`: id (UUID, also the WebAuthn user handle), display name,
//!   the public part of the recovery code and the argon2 hash of its secret.
//! - `passkeys`: the stored WebAuthn credential (public key, counter) per user.
//! - `sessions`: SHA-256 of the session token, user, expiry.
//! - `saves`: one opaque save blob per user with version and time.
//! - `daily_results`: one result per user and daily seed.
//!
//! Deleting a user deletes everything else of that user (foreign keys
//! with `ON DELETE CASCADE`).

use rusqlite::{Connection, OptionalExtension, params};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const SCHEMA: &str = r#"
PRAGMA foreign_keys = ON;
CREATE TABLE IF NOT EXISTS users (
    id            TEXT PRIMARY KEY,
    name          TEXT,
    created_at    INTEGER NOT NULL,
    recovery_id   TEXT NOT NULL UNIQUE,
    recovery_hash TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS passkeys (
    cred_id      TEXT PRIMARY KEY,
    user_id      TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    passkey      TEXT NOT NULL,
    created_at   INTEGER NOT NULL,
    last_used_at INTEGER
);
CREATE INDEX IF NOT EXISTS passkeys_user ON passkeys(user_id);
CREATE TABLE IF NOT EXISTS sessions (
    token_hash TEXT PRIMARY KEY,
    user_id    TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS sessions_user ON sessions(user_id);
CREATE TABLE IF NOT EXISTS saves (
    user_id    TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    version    INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    stored_at  INTEGER NOT NULL,
    data       TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS daily_results (
    day          TEXT NOT NULL,
    user_id      TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    character    TEXT NOT NULL,
    won          INTEGER NOT NULL,
    section      INTEGER NOT NULL,
    series       INTEGER NOT NULL,
    stake        INTEGER NOT NULL,
    best_game    INTEGER NOT NULL,
    submitted_at INTEGER NOT NULL,
    PRIMARY KEY (day, user_id)
);
"#;

/// Milliseconds since the Unix epoch.
pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

/// Open (or create) the database file and apply the schema.
pub fn open(path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "busy_timeout", 5000)?;
    init(&conn)?;
    Ok(conn)
}

/// An in-memory database (tests, `--memory`).
pub fn open_in_memory() -> rusqlite::Result<Connection> {
    let conn = Connection::open_in_memory()?;
    init(&conn)?;
    Ok(conn)
}

fn init(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(SCHEMA)
}

/// A user as the API shows it.
#[derive(Debug, Clone)]
pub struct User {
    /// UUID as text.
    pub id: String,
    /// Display name, if set.
    pub name: Option<String>,
    /// Creation time (ms).
    pub created_at: i64,
}

/// Create a user with the recovery code's public id and hash.
pub fn insert_user(
    conn: &Connection,
    id: &str,
    name: Option<&str>,
    recovery_id: &str,
    recovery_hash: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO users (id, name, created_at, recovery_id, recovery_hash) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, name, now_ms(), recovery_id, recovery_hash],
    )?;
    Ok(())
}

/// The user with this id.
pub fn user(conn: &Connection, id: &str) -> rusqlite::Result<Option<User>> {
    conn.query_row(
        "SELECT id, name, created_at FROM users WHERE id = ?1",
        params![id],
        |r| {
            Ok(User {
                id: r.get(0)?,
                name: r.get(1)?,
                created_at: r.get(2)?,
            })
        },
    )
    .optional()
}

/// Set or clear the display name.
pub fn set_name(conn: &Connection, id: &str, name: Option<&str>) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE users SET name = ?2 WHERE id = ?1",
        params![id, name],
    )?;
    Ok(())
}

/// The user and recovery hash for the public part of a recovery code.
pub fn user_by_recovery_id(
    conn: &Connection,
    recovery_id: &str,
) -> rusqlite::Result<Option<(String, String)>> {
    conn.query_row(
        "SELECT id, recovery_hash FROM users WHERE recovery_id = ?1",
        params![recovery_id],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
    .optional()
}

/// Replace the recovery code of a user.
pub fn set_recovery(
    conn: &Connection,
    id: &str,
    recovery_id: &str,
    recovery_hash: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE users SET recovery_id = ?2, recovery_hash = ?3 WHERE id = ?1",
        params![id, recovery_id, recovery_hash],
    )?;
    Ok(())
}

/// Delete a user and, through the foreign keys, all of their data.
pub fn delete_user(conn: &Connection, id: &str) -> rusqlite::Result<bool> {
    Ok(conn.execute("DELETE FROM users WHERE id = ?1", params![id])? > 0)
}

/// Store a new passkey (JSON of `webauthn_rs::prelude::Passkey`).
pub fn insert_passkey(
    conn: &Connection,
    cred_id: &str,
    user_id: &str,
    passkey: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO passkeys (cred_id, user_id, passkey, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![cred_id, user_id, passkey, now_ms()],
    )?;
    Ok(())
}

/// The passkey with this credential id of this user.
pub fn passkey(
    conn: &Connection,
    user_id: &str,
    cred_id: &str,
) -> rusqlite::Result<Option<String>> {
    conn.query_row(
        "SELECT passkey FROM passkeys WHERE user_id = ?1 AND cred_id = ?2",
        params![user_id, cred_id],
        |r| r.get(0),
    )
    .optional()
}

/// All passkeys of a user (JSON).
pub fn passkeys(conn: &Connection, user_id: &str) -> rusqlite::Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT passkey FROM passkeys WHERE user_id = ?1")?;
    let rows = stmt.query_map(params![user_id], |r| r.get(0))?;
    rows.collect()
}

/// Write back a passkey after a login (counter, backup state).
pub fn update_passkey(conn: &Connection, cred_id: &str, passkey: &str) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE passkeys SET passkey = ?2, last_used_at = ?3 WHERE cred_id = ?1",
        params![cred_id, passkey, now_ms()],
    )?;
    Ok(())
}

/// Number of passkeys of a user.
pub fn passkey_count(conn: &Connection, user_id: &str) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COUNT(*) FROM passkeys WHERE user_id = ?1",
        params![user_id],
        |r| r.get(0),
    )
}

/// Store a session (token hash) until `expires_at`.
pub fn insert_session(
    conn: &Connection,
    token_hash: &str,
    user_id: &str,
    expires_at: i64,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO sessions (token_hash, user_id, created_at, expires_at) VALUES (?1, ?2, ?3, ?4)",
        params![token_hash, user_id, now_ms(), expires_at],
    )?;
    Ok(())
}

/// The user of a valid session and its expiry.
pub fn session_user(
    conn: &Connection,
    token_hash: &str,
) -> rusqlite::Result<Option<(String, i64)>> {
    conn.query_row(
        "SELECT user_id, expires_at FROM sessions WHERE token_hash = ?1 AND expires_at > ?2",
        params![token_hash, now_ms()],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )
    .optional()
}

/// Push the expiry of a session.
pub fn extend_session(
    conn: &Connection,
    token_hash: &str,
    expires_at: i64,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE sessions SET expires_at = ?2 WHERE token_hash = ?1",
        params![token_hash, expires_at],
    )?;
    Ok(())
}

/// End a session.
pub fn delete_session(conn: &Connection, token_hash: &str) -> rusqlite::Result<()> {
    conn.execute(
        "DELETE FROM sessions WHERE token_hash = ?1",
        params![token_hash],
    )?;
    Ok(())
}

/// Drop expired sessions.
pub fn purge_sessions(conn: &Connection) -> rusqlite::Result<usize> {
    conn.execute(
        "DELETE FROM sessions WHERE expires_at <= ?1",
        params![now_ms()],
    )
}

/// The stored save of a user.
#[derive(Debug, Clone)]
pub struct Save {
    /// Increases by one with every write.
    pub version: i64,
    /// When the player's state changed, as the client said (ms).
    pub updated_at: i64,
    /// The blob (the save code, opaque to the server).
    pub data: String,
}

/// The save of a user.
pub fn save(conn: &Connection, user_id: &str) -> rusqlite::Result<Option<Save>> {
    conn.query_row(
        "SELECT version, updated_at, data FROM saves WHERE user_id = ?1",
        params![user_id],
        |r| {
            Ok(Save {
                version: r.get(0)?,
                updated_at: r.get(1)?,
                data: r.get(2)?,
            })
        },
    )
    .optional()
}

/// Outcome of writing a save.
#[derive(Debug)]
pub enum PutSave {
    /// Written; the new version.
    Stored(i64),
    /// The stored version is not the expected one: version and time of what is there.
    Conflict(Option<(i64, i64)>),
}

/// Write a save if the stored version is `if_version` (None: there must be none).
pub fn put_save(
    conn: &mut Connection,
    user_id: &str,
    if_version: Option<i64>,
    updated_at: i64,
    data: &str,
) -> rusqlite::Result<PutSave> {
    let tx = conn.transaction()?;
    let current: Option<(i64, i64)> = tx
        .query_row(
            "SELECT version, updated_at FROM saves WHERE user_id = ?1",
            params![user_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    if current.map(|c| c.0) != if_version {
        return Ok(PutSave::Conflict(current));
    }
    let version = if_version.unwrap_or(0) + 1;
    tx.execute(
        "INSERT INTO saves (user_id, version, updated_at, stored_at, data) VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(user_id) DO UPDATE SET version = ?2, updated_at = ?3, stored_at = ?4, data = ?5",
        params![user_id, version, updated_at, now_ms(), data],
    )?;
    tx.commit()?;
    Ok(PutSave::Stored(version))
}

/// A daily result to store.
#[derive(Debug, Clone)]
pub struct DailyResult {
    /// The daily seed (JJJJMMTT).
    pub day: String,
    /// Character id of the core (`Declarer`, …).
    pub character: String,
    /// Run won.
    pub won: bool,
    /// District reached, from 1.
    pub section: i64,
    /// Pub reached in that district, from 1.
    pub series: i64,
    /// Stake of the Einsatz-Leiter.
    pub stake: i64,
    /// Best game value of the run.
    pub best_game: i64,
}

/// Store a daily result; the first one per user and day counts. Returns
/// whether it was stored (false: there was one already).
pub fn insert_daily(conn: &Connection, user_id: &str, r: &DailyResult) -> rusqlite::Result<bool> {
    let n = conn.execute(
        "INSERT OR IGNORE INTO daily_results (day, user_id, character, won, section, series, stake, best_game, submitted_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            r.day,
            user_id,
            r.character,
            r.won,
            r.section,
            r.series,
            r.stake,
            r.best_game,
            now_ms()
        ],
    )?;
    Ok(n > 0)
}

/// A row of the daily leaderboard.
#[derive(Debug, Clone)]
pub struct DailyRow {
    /// Place, from 1.
    pub rank: i64,
    /// User id (not sent to clients).
    pub user_id: String,
    /// Display name, if the player set one.
    pub name: Option<String>,
    /// The result.
    pub result: DailyResult,
}

/// Order of the board: won runs first, then farther, then higher stake,
/// then the better best game, then who was first.
const RANKED: &str = "SELECT ROW_NUMBER() OVER (ORDER BY d.won DESC, d.section DESC, d.series DESC, d.stake DESC, d.best_game DESC, d.submitted_at ASC) AS rank,
       d.user_id, u.name, d.day, d.character, d.won, d.section, d.series, d.stake, d.best_game
  FROM daily_results d JOIN users u ON u.id = d.user_id
 WHERE d.day = ?1";

fn daily_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<DailyRow> {
    Ok(DailyRow {
        rank: r.get(0)?,
        user_id: r.get(1)?,
        name: r.get(2)?,
        result: DailyResult {
            day: r.get(3)?,
            character: r.get(4)?,
            won: r.get(5)?,
            section: r.get(6)?,
            series: r.get(7)?,
            stake: r.get(8)?,
            best_game: r.get(9)?,
        },
    })
}

/// The best `limit` results of a day.
pub fn daily_top(conn: &Connection, day: &str, limit: i64) -> rusqlite::Result<Vec<DailyRow>> {
    let mut stmt = conn.prepare(&format!("{RANKED} ORDER BY rank LIMIT ?2"))?;
    let rows = stmt.query_map(params![day, limit], daily_row)?;
    rows.collect()
}

/// The result of one user on a day, with its place.
pub fn daily_of(conn: &Connection, day: &str, user_id: &str) -> rusqlite::Result<Option<DailyRow>> {
    conn.query_row(
        &format!("SELECT * FROM ({RANKED}) WHERE user_id = ?2"),
        params![day, user_id],
        daily_row,
    )
    .optional()
}

/// How many results a day has.
pub fn daily_count(conn: &Connection, day: &str) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COUNT(*) FROM daily_results WHERE day = ?1",
        params![day],
        |r| r.get(0),
    )
}
