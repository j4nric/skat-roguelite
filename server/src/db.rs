//! SQLite storage. One file (`stammtisch.db`), one connection behind a
//! mutex: the load is a handful of players, every statement is short.
//!
//! Tables:
//! - `users`: id (UUID, also the WebAuthn user handle), display name
//!   (building-block ids, [`crate::names`]), the public part of the recovery code and the argon2 hash of its secret.
//! - `passkeys`: the stored WebAuthn credential (public key, counter) per
//!   user, with the RP id (host) it belongs to.
//! - `sessions`: SHA-256 of the session token, user, expiry.
//! - `saves`: one opaque save blob per user with version and time.
//! - `daily_results`: one result per user and daily seed.
//! - `weekly_results`: one result per user and ISO week (Tafel der Woche).
//! - `move_codes`: at most one open move code per user (SHA-256 of the
//!   code, expiry), to take the account to the new address (ADR 0025).
//! - `reports`: bug reports from the app (ADR 0027), opaque text with id,
//!   time, platform, version and the account if one was signed in; kept
//!   90 days ([`crate::report`]).
//!
//! Deleting a user deletes everything else of that user (foreign keys
//! with `ON DELETE CASCADE`).
//!
//! The schema is a list of migrations ([`MIGRATIONS`]); `PRAGMA
//! user_version` says how many of them a file has seen. Opening applies
//! the missing ones in one transaction. A change to the schema is a new
//! entry at the end, never an edit of an old one.

use rusqlite::{Connection, OptionalExtension, params};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// The database stops growing here (`PRAGMA max_page_count`): a write
/// that needs more fails with `SQLITE_FULL`, which the API answers with
/// 507. Some 2000 players with a full 1 MB save each; the real saves are
/// a few kilobytes.
pub const MAX_DB_BYTES: i64 = 2 * 1024 * 1024 * 1024;

/// Migration 1: the schema of the first release. `IF NOT EXISTS`, so it
/// also adopts a file made before migrations were counted (version 0
/// with these tables).
const MIGRATION_1: &str = r#"
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

/// Migration 2: the Tafel der Woche (DESIGN 5.6), shaped like the daily
/// board with the ISO week (`JJJJWW`) as key.
const MIGRATION_2: &str = r#"
CREATE TABLE weekly_results (
    week         TEXT NOT NULL,
    user_id      TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    character    TEXT NOT NULL,
    won          INTEGER NOT NULL,
    section      INTEGER NOT NULL,
    series       INTEGER NOT NULL,
    stake        INTEGER NOT NULL,
    best_game    INTEGER NOT NULL,
    submitted_at INTEGER NOT NULL,
    PRIMARY KEY (week, user_id)
);
"#;

/// Migration 3: the move to one-more-skat.de (ADR 0025). Passkeys note
/// their RP id (NULL for the old ones until their next sign-in), and a
/// signed-in player can make a short-lived move code.
const MIGRATION_3: &str = r#"
ALTER TABLE passkeys ADD COLUMN rp_id TEXT;
CREATE TABLE move_codes (
    user_id    TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    code_hash  TEXT NOT NULL UNIQUE,
    expires_at INTEGER NOT NULL
);
"#;

/// Migration 4: bug reports (ADR 0027). The text is opaque to the server;
/// the account goes with its reports when it is deleted.
const MIGRATION_4: &str = r#"
CREATE TABLE reports (
    id         TEXT PRIMARY KEY,
    created_at INTEGER NOT NULL,
    platform   TEXT NOT NULL,
    version    TEXT NOT NULL,
    user_id    TEXT REFERENCES users(id) ON DELETE CASCADE,
    data       TEXT NOT NULL
);
CREATE INDEX reports_created ON reports(created_at);
CREATE INDEX reports_user ON reports(user_id);
"#;

/// Migration 5: names from building blocks (ADR 0029). Every name that
/// is not a valid building-block id (all free-text names from before) is
/// removed; the player is shown as nameless until they pick one.
fn migration_5(conn: &Connection) -> rusqlite::Result<()> {
    let named: Vec<(String, String)> = conn
        .prepare("SELECT id, name FROM users WHERE name IS NOT NULL")?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<rusqlite::Result<_>>()?;
    let mut clear = conn.prepare("UPDATE users SET name = NULL WHERE id = ?1")?;
    for (id, name) in named {
        if !crate::names::is_valid(&name) {
            clear.execute(params![id])?;
        }
    }
    Ok(())
}

/// One step of the schema: SQL, or code where SQL alone cannot say it.
enum Migration {
    Sql(&'static str),
    Code(fn(&Connection) -> rusqlite::Result<()>),
}

/// Every migration in order; the database's `user_version` is the number
/// applied.
const MIGRATIONS: &[Migration] = &[
    Migration::Sql(MIGRATION_1),
    Migration::Sql(MIGRATION_2),
    Migration::Sql(MIGRATION_3),
    Migration::Sql(MIGRATION_4),
    Migration::Code(migration_5),
];

/// The schema version this server writes.
pub const SCHEMA_VERSION: i64 = MIGRATIONS.len() as i64;

/// Milliseconds since the Unix epoch.
pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

/// Open (or create) the database file and bring the schema up to date.
pub fn open(path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    // With WAL, NORMAL is safe against corruption; a power cut may lose
    // the last commits, not the file.
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "busy_timeout", 5000)?;
    init(&conn)?;
    Ok(conn)
}

/// Open the database file only to read (the report export), without
/// touching its schema. Works next to the running server (WAL).
pub fn open_read_only(path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open_with_flags(
        path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    conn.pragma_update(None, "busy_timeout", 5000)?;
    Ok(conn)
}

/// An in-memory database (tests, `--memory`).
pub fn open_in_memory() -> rusqlite::Result<Connection> {
    let conn = Connection::open_in_memory()?;
    init(&conn)?;
    Ok(conn)
}

fn init(conn: &Connection) -> rusqlite::Result<()> {
    // Not inside a transaction: there the pragma does nothing.
    conn.pragma_update(None, "foreign_keys", "ON")?;
    set_max_bytes(conn, MAX_DB_BYTES)?;
    migrate(conn)
}

/// Cap the size of the database (per connection, not stored in the file).
pub fn set_max_bytes(conn: &Connection, bytes: i64) -> rusqlite::Result<()> {
    let page: i64 = conn.query_row("PRAGMA page_size", [], |r| r.get(0))?;
    let pages = (bytes / page.max(512)).max(1);
    conn.query_row(&format!("PRAGMA max_page_count = {pages}"), [], |_| Ok(()))
}

/// The schema version of the open database.
pub fn schema_version(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row("PRAGMA user_version", [], |r| r.get(0))
}

/// Apply the migrations the database has not seen, all in one
/// transaction. A database from a newer server is refused, not touched.
fn migrate(conn: &Connection) -> rusqlite::Result<()> {
    let tx = conn.unchecked_transaction()?;
    let version = schema_version(&tx)?;
    if version > SCHEMA_VERSION {
        return Err(rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_MISMATCH),
            Some(format!(
                "database schema {version} is newer than this server ({SCHEMA_VERSION})"
            )),
        ));
    }
    for (n, step) in MIGRATIONS
        .iter()
        .enumerate()
        .skip(usize::try_from(version).unwrap_or(0))
    {
        match step {
            Migration::Sql(sql) => tx.execute_batch(sql)?,
            Migration::Code(run) => run(&tx)?,
        }
        tx.pragma_update(None, "user_version", n + 1)?;
    }
    tx.commit()
}

/// A user as the API shows it.
#[derive(Debug, Clone)]
pub struct User {
    /// UUID as text.
    pub id: String,
    /// Display name as building-block ids, if set.
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

/// Set or clear the display name (already checked, [`crate::names`]).
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

/// The argon2 hash of the user's current recovery code.
pub fn recovery_hash(conn: &Connection, id: &str) -> rusqlite::Result<Option<String>> {
    conn.query_row(
        "SELECT recovery_hash FROM users WHERE id = ?1",
        params![id],
        |r| r.get(0),
    )
    .optional()
}

/// Replace the recovery code of a user, but only if it still is the one
/// with `old_hash` (compare and swap): of two requests that redeemed the
/// same code at once, only one gets through. Returns whether it changed.
pub fn swap_recovery(
    conn: &Connection,
    id: &str,
    old_hash: &str,
    recovery_id: &str,
    recovery_hash: &str,
) -> rusqlite::Result<bool> {
    let n = conn.execute(
        "UPDATE users SET recovery_id = ?3, recovery_hash = ?4 WHERE id = ?1 AND recovery_hash = ?2",
        params![id, old_hash, recovery_id, recovery_hash],
    )?;
    Ok(n == 1)
}

/// Delete a user and, through the foreign keys, all of their data.
pub fn delete_user(conn: &Connection, id: &str) -> rusqlite::Result<bool> {
    Ok(conn.execute("DELETE FROM users WHERE id = ?1", params![id])? > 0)
}

/// Store a new passkey (JSON of `webauthn_rs::prelude::Passkey`) made
/// for the RP id `rp_id`.
pub fn insert_passkey(
    conn: &Connection,
    cred_id: &str,
    user_id: &str,
    passkey: &str,
    rp_id: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO passkeys (cred_id, user_id, passkey, created_at, rp_id) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![cred_id, user_id, passkey, now_ms(), rp_id],
    )?;
    Ok(())
}

/// Note the RP id of a passkey that has none yet (made before it was
/// stored; a sign-in with it shows where it belongs).
pub fn note_passkey_rp(conn: &Connection, cred_id: &str, rp_id: &str) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE passkeys SET rp_id = ?2 WHERE cred_id = ?1 AND rp_id IS NULL",
        params![cred_id, rp_id],
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

/// A passkey as the account sheet lists it.
#[derive(Debug, Clone)]
pub struct PasskeyRow {
    /// Credential id (base64url).
    pub cred_id: String,
    /// The stored credential (JSON).
    pub passkey: String,
    /// When it was added (ms).
    pub created_at: i64,
    /// Last sign-in with it (ms), if any.
    pub last_used_at: Option<i64>,
    /// The RP id (host) it belongs to, if known.
    pub rp_id: Option<String>,
}

/// The passkeys of a user, oldest first.
pub fn passkey_rows(conn: &Connection, user_id: &str) -> rusqlite::Result<Vec<PasskeyRow>> {
    let mut stmt = conn.prepare(
        "SELECT cred_id, passkey, created_at, last_used_at, rp_id FROM passkeys WHERE user_id = ?1 ORDER BY created_at, cred_id",
    )?;
    let rows = stmt.query_map(params![user_id], |r| {
        Ok(PasskeyRow {
            cred_id: r.get(0)?,
            passkey: r.get(1)?,
            created_at: r.get(2)?,
            last_used_at: r.get(3)?,
            rp_id: r.get(4)?,
        })
    })?;
    rows.collect()
}

/// Outcome of removing a passkey.
#[derive(Debug, PartialEq, Eq)]
pub enum RemovePasskey {
    /// Gone.
    Removed,
    /// The user has no passkey with this id.
    NotFound,
    /// It is the user's last passkey; it stays.
    Last,
}

/// Remove one of a user's passkeys, never the last one (one statement,
/// so two removals at once cannot take both of the last two).
pub fn remove_passkey(
    conn: &Connection,
    user_id: &str,
    cred_id: &str,
) -> rusqlite::Result<RemovePasskey> {
    let n = conn.execute(
        "DELETE FROM passkeys WHERE user_id = ?1 AND cred_id = ?2
           AND (SELECT COUNT(*) FROM passkeys WHERE user_id = ?1) > 1",
        params![user_id, cred_id],
    )?;
    if n > 0 {
        return Ok(RemovePasskey::Removed);
    }
    let exists: bool = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM passkeys WHERE user_id = ?1 AND cred_id = ?2)",
        params![user_id, cred_id],
        |r| r.get(0),
    )?;
    Ok(if exists {
        RemovePasskey::Last
    } else {
        RemovePasskey::NotFound
    })
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

/// End every session of a user except the one with `keep` (all of them
/// with `None`). Returns how many ended.
pub fn delete_other_sessions(
    conn: &Connection,
    user_id: &str,
    keep: Option<&str>,
) -> rusqlite::Result<usize> {
    conn.execute(
        "DELETE FROM sessions WHERE user_id = ?1 AND token_hash IS NOT ?2",
        params![user_id, keep],
    )
}

/// Drop expired sessions (and expired move codes with them).
pub fn purge_sessions(conn: &Connection) -> rusqlite::Result<usize> {
    let now = now_ms();
    conn.execute(
        "DELETE FROM move_codes WHERE expires_at <= ?1",
        params![now],
    )?;
    conn.execute("DELETE FROM sessions WHERE expires_at <= ?1", params![now])
}

/// Store the move code (its hash) of a user until `expires_at`; it
/// replaces one made before.
pub fn put_move_code(
    conn: &Connection,
    user_id: &str,
    code_hash: &str,
    expires_at: i64,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO move_codes (user_id, code_hash, expires_at) VALUES (?1, ?2, ?3)
         ON CONFLICT (user_id) DO UPDATE SET code_hash = excluded.code_hash, expires_at = excluded.expires_at",
        params![user_id, code_hash, expires_at],
    )?;
    Ok(())
}

/// Use up a move code: its user if it is known and not expired. One
/// statement, so two requests with the same code get it at most once.
pub fn take_move_code(conn: &Connection, code_hash: &str) -> rusqlite::Result<Option<String>> {
    let user: Option<(String, i64)> = conn
        .query_row(
            "DELETE FROM move_codes WHERE code_hash = ?1 RETURNING user_id, expires_at",
            params![code_hash],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    Ok(user.filter(|(_, exp)| *exp > now_ms()).map(|(u, _)| u))
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

/// A leaderboard: the Tafel des Tages or the Tafel der Woche. Both
/// tables have the same shape; only the table and its key differ.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Board {
    /// One result per user and daily seed (`JJJJMMTT`).
    Daily,
    /// One result per user and ISO week (`JJJJWW`).
    Weekly,
}

impl Board {
    /// The table and its key column (fixed names, never user input).
    const fn table(self) -> (&'static str, &'static str) {
        match self {
            Board::Daily => ("daily_results", "day"),
            Board::Weekly => ("weekly_results", "week"),
        }
    }
}

/// A result to store on a board.
#[derive(Debug, Clone)]
pub struct BoardResult {
    /// The period: the daily seed (`JJJJMMTT`) or the week (`JJJJWW`).
    pub period: String,
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

/// Store a result; the first one per user and period counts. Returns
/// whether it was stored (false: there was one already).
pub fn insert_result(
    conn: &Connection,
    board: Board,
    user_id: &str,
    r: &BoardResult,
) -> rusqlite::Result<bool> {
    let (table, key) = board.table();
    let n = conn.execute(
        &format!(
            "INSERT OR IGNORE INTO {table} ({key}, user_id, character, won, section, series, stake, best_game, submitted_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"
        ),
        params![
            r.period,
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

/// A row of a leaderboard.
#[derive(Debug, Clone)]
pub struct BoardRow {
    /// Place, from 1.
    pub rank: i64,
    /// User id (not sent to clients).
    pub user_id: String,
    /// Display name, if the player set one.
    pub name: Option<String>,
    /// The result.
    pub result: BoardResult,
}

/// The ranked rows of a period: won runs first, then farther, then
/// higher stake, then the better best game, then who was first.
fn ranked(board: Board) -> String {
    let (table, key) = board.table();
    format!(
        "SELECT ROW_NUMBER() OVER (ORDER BY d.won DESC, d.section DESC, d.series DESC, d.stake DESC, d.best_game DESC, d.submitted_at ASC) AS rank,
       d.user_id, u.name, d.{key}, d.character, d.won, d.section, d.series, d.stake, d.best_game
  FROM {table} d JOIN users u ON u.id = d.user_id
 WHERE d.{key} = ?1"
    )
}

fn board_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<BoardRow> {
    Ok(BoardRow {
        rank: r.get(0)?,
        user_id: r.get(1)?,
        name: r.get(2)?,
        result: BoardResult {
            period: r.get(3)?,
            character: r.get(4)?,
            won: r.get(5)?,
            section: r.get(6)?,
            series: r.get(7)?,
            stake: r.get(8)?,
            best_game: r.get(9)?,
        },
    })
}

/// The best `limit` results of a period.
pub fn board_top(
    conn: &Connection,
    board: Board,
    period: &str,
    limit: i64,
) -> rusqlite::Result<Vec<BoardRow>> {
    let mut stmt = conn.prepare(&format!("{} ORDER BY rank LIMIT ?2", ranked(board)))?;
    let rows = stmt.query_map(params![period, limit], board_row)?;
    rows.collect()
}

/// The result of one user in a period, with its place.
pub fn result_of(
    conn: &Connection,
    board: Board,
    period: &str,
    user_id: &str,
) -> rusqlite::Result<Option<BoardRow>> {
    conn.query_row(
        &format!("SELECT * FROM ({}) WHERE user_id = ?2", ranked(board)),
        params![period, user_id],
        board_row,
    )
    .optional()
}

/// How many results a period has.
pub fn board_count(conn: &Connection, board: Board, period: &str) -> rusqlite::Result<i64> {
    let (table, key) = board.table();
    conn.query_row(
        &format!("SELECT COUNT(*) FROM {table} WHERE {key} = ?1"),
        params![period],
        |r| r.get(0),
    )
}

/// A stored bug report.
#[derive(Debug, Clone)]
pub struct ReportRow {
    /// Short id (`B-7K3Q`).
    pub id: String,
    /// When it came in (ms).
    pub created_at: i64,
    /// Platform the app named.
    pub platform: String,
    /// Version the app named.
    pub version: String,
    /// The account, if one was signed in.
    pub user_id: Option<String>,
    /// The report, as the app sent it.
    pub data: String,
}

/// Store a report; false if the id is taken (the caller picks another).
pub fn insert_report(
    conn: &Connection,
    id: &str,
    created_at: i64,
    platform: &str,
    version: &str,
    user_id: Option<&str>,
    data: &str,
) -> rusqlite::Result<bool> {
    let n = conn.execute(
        "INSERT OR IGNORE INTO reports (id, created_at, platform, version, user_id, data) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, created_at, platform, version, user_id, data],
    )?;
    Ok(n == 1)
}

/// Bytes of all stored reports.
pub fn reports_bytes(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row(
        "SELECT COALESCE(SUM(LENGTH(CAST(data AS BLOB))), 0) FROM reports",
        [],
        |r| r.get(0),
    )
}

/// Delete reports that came in before `before` (ms).
pub fn purge_reports(conn: &Connection, before: i64) -> rusqlite::Result<usize> {
    conn.execute("DELETE FROM reports WHERE created_at < ?1", params![before])
}

/// Reports since `since` (ms), the oldest first, at most `limit`; with
/// `id` only that one. Empty if the database has no reports yet (a
/// schema older than migration 4).
pub fn reports(
    conn: &Connection,
    since: i64,
    id: Option<&str>,
    limit: usize,
) -> rusqlite::Result<Vec<ReportRow>> {
    let exists: bool = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'reports')",
        [],
        |r| r.get(0),
    )?;
    if !exists {
        return Ok(Vec::new());
    }
    let mut stmt = conn.prepare(
        "SELECT id, created_at, platform, version, user_id, data FROM reports
         WHERE created_at >= ?1 AND (?2 IS NULL OR id = ?2)
         ORDER BY created_at, id LIMIT ?3",
    )?;
    let limit = i64::try_from(limit).unwrap_or(i64::MAX);
    let rows = stmt.query_map(params![since, id, limit], |r| {
        Ok(ReportRow {
            id: r.get(0)?,
            created_at: r.get(1)?,
            platform: r.get(2)?,
            version: r.get(3)?,
            user_id: r.get(4)?,
            data: r.get(5)?,
        })
    })?;
    rows.collect()
}

/// Name of the backup file of a day (`JJJJMMTT`).
fn backup_name(day: &str) -> String {
    format!("stammtisch-{day}.db")
}

/// Whether a file name is one of our daily backups; returns its day.
fn backup_day(name: &str) -> Option<&str> {
    let day = name.strip_prefix("stammtisch-")?.strip_suffix(".db")?;
    (day.len() == 8 && day.bytes().all(|b| b.is_ascii_digit())).then_some(day)
}

/// Write a consistent copy of the database at `db` to
/// `dir/stammtisch-<day>.db` with `VACUUM INTO` (on its own connection, so
/// the server keeps answering; WAL lets it read meanwhile), unless that
/// day has one already. Then keep only the newest `keep` backups. Returns
/// the path of a backup written now.
pub fn backup(db: &Path, dir: &Path, day: &str, keep: usize) -> rusqlite::Result<Option<PathBuf>> {
    let io = |e: std::io::Error| {
        rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CANTOPEN),
            Some(format!("backup dir {}: {e}", dir.display())),
        )
    };
    std::fs::create_dir_all(dir).map_err(io)?;
    let target = dir.join(backup_name(day));
    let mut written = None;
    if !target.exists() {
        // VACUUM INTO refuses an existing file; a half-written one from a
        // crash is removed first, and the rename makes the copy appear whole.
        let tmp = dir.join(format!("{}.tmp", backup_name(day)));
        let _ = std::fs::remove_file(&tmp);
        let conn = Connection::open(db)?;
        conn.pragma_update(None, "busy_timeout", 5000)?;
        let tmp_text = tmp
            .to_str()
            .ok_or_else(|| io(std::io::Error::other("path is not UTF-8")))?;
        conn.execute("VACUUM INTO ?1", params![tmp_text])?;
        drop(conn);
        std::fs::rename(&tmp, &target).map_err(io)?;
        written = Some(target);
    }
    let mut days: Vec<String> = std::fs::read_dir(dir)
        .map_err(io)?
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            e.file_name()
                .to_str()
                .and_then(backup_day)
                .map(str::to_owned)
        })
        .collect();
    days.sort_unstable();
    let old = days.len().saturating_sub(keep);
    for day in &days[..old] {
        let _ = std::fs::remove_file(dir.join(backup_name(day)));
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// A fresh path under the system temp dir (no extra crate for this).
    fn temp_path(name: &str) -> PathBuf {
        static N: AtomicU32 = AtomicU32::new(0);
        let n = N.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "stammtisch-test-{}-{}-{n}-{name}",
            std::process::id(),
            now_ms()
        ))
    }

    /// The tables as the first release created them, before migrations
    /// were counted (`user_version` 0).
    const UNVERSIONED: &str = "
        CREATE TABLE users (id TEXT PRIMARY KEY, name TEXT, created_at INTEGER NOT NULL,
            recovery_id TEXT NOT NULL UNIQUE, recovery_hash TEXT NOT NULL);
        CREATE TABLE passkeys (cred_id TEXT PRIMARY KEY, user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            passkey TEXT NOT NULL, created_at INTEGER NOT NULL, last_used_at INTEGER);
        CREATE INDEX passkeys_user ON passkeys(user_id);
        CREATE TABLE sessions (token_hash TEXT PRIMARY KEY, user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            created_at INTEGER NOT NULL, expires_at INTEGER NOT NULL);
        CREATE INDEX sessions_user ON sessions(user_id);
        CREATE TABLE saves (user_id TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE, version INTEGER NOT NULL,
            updated_at INTEGER NOT NULL, stored_at INTEGER NOT NULL, data TEXT NOT NULL);
        CREATE TABLE daily_results (day TEXT NOT NULL, user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
            character TEXT NOT NULL, won INTEGER NOT NULL, section INTEGER NOT NULL, series INTEGER NOT NULL,
            stake INTEGER NOT NULL, best_game INTEGER NOT NULL, submitted_at INTEGER NOT NULL, PRIMARY KEY (day, user_id));
        INSERT INTO users (id, name, created_at, recovery_id, recovery_hash) VALUES ('u1', 'Jan', 1, 'ABCDEFGH', 'h');
    ";

    #[test]
    fn fresh_database_is_at_the_latest_version() {
        let conn = open_in_memory().expect("db");
        assert_eq!(schema_version(&conn).expect("version"), SCHEMA_VERSION);
        // Applying again changes nothing.
        migrate(&conn).expect("again");
        assert_eq!(schema_version(&conn).expect("version"), SCHEMA_VERSION);
    }

    #[test]
    fn unversioned_database_is_adopted_with_its_data() {
        let path = temp_path("old.db");
        {
            let conn = Connection::open(&path).expect("create");
            conn.execute_batch(UNVERSIONED).expect("old schema");
            assert_eq!(schema_version(&conn).expect("version"), 0);
        }
        let conn = open(&path).expect("open");
        assert_eq!(schema_version(&conn).expect("version"), SCHEMA_VERSION);
        let user = user(&conn, "u1").expect("query").expect("kept");
        // The free-text name went with migration 5 (ADR 0029).
        assert_eq!(user.name, None);
        drop(conn);
        for ext in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(format!("{}{ext}", path.display()));
        }
    }

    #[test]
    fn version_one_gets_the_weekly_board_and_keeps_the_daily() {
        let path = temp_path("v1.db");
        {
            let conn = Connection::open(&path).expect("create");
            conn.execute_batch(MIGRATION_1).expect("v1");
            conn.pragma_update(None, "user_version", 1).expect("v1");
            conn.execute_batch(
                "INSERT INTO users (id, name, created_at, recovery_id, recovery_hash) VALUES ('u1', 'Jan', 1, 'ABCDEFGH', 'h');
                 INSERT INTO daily_results VALUES ('20261109', 'u1', 'Declarer', 1, 3, 5, 0, 96, 1);",
            )
            .expect("data");
        }
        let conn = open(&path).expect("open");
        assert_eq!(schema_version(&conn).expect("version"), SCHEMA_VERSION);
        assert_eq!(board_count(&conn, Board::Daily, "20261109").expect("n"), 1);
        let week = BoardResult {
            period: "202646".into(),
            character: "Declarer".into(),
            won: false,
            section: 2,
            series: 1,
            stake: 0,
            best_game: 40,
        };
        assert!(insert_result(&conn, Board::Weekly, "u1", &week).expect("insert"));
        assert!(!insert_result(&conn, Board::Weekly, "u1", &week).expect("again"));
        assert_eq!(board_count(&conn, Board::Weekly, "202646").expect("n"), 1);
        assert_eq!(board_count(&conn, Board::Daily, "202646").expect("n"), 0);
        let mine = result_of(&conn, Board::Weekly, "202646", "u1")
            .expect("q")
            .expect("row");
        assert_eq!((mine.rank, mine.name.as_deref()), (1, None));
        // Deleting the user takes the weekly result along.
        assert!(delete_user(&conn, "u1").expect("delete"));
        assert_eq!(board_count(&conn, Board::Weekly, "202646").expect("n"), 0);
        drop(conn);
        for ext in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(format!("{}{ext}", path.display()));
        }
    }

    #[test]
    fn version_two_gets_move_codes_and_keeps_its_passkeys() {
        let conn = Connection::open_in_memory().expect("create");
        conn.execute_batch(MIGRATION_1).expect("v1");
        conn.execute_batch(MIGRATION_2).expect("v2");
        conn.pragma_update(None, "user_version", 2).expect("v2");
        conn.execute_batch(
            "INSERT INTO users (id, name, created_at, recovery_id, recovery_hash) VALUES ('u1', 'Jan', 1, 'ABCDEFGH', 'h');
             INSERT INTO passkeys (cred_id, user_id, passkey, created_at) VALUES ('old', 'u1', '{}', 1);",
        )
        .expect("data");
        migrate(&conn).expect("migrate");
        assert_eq!(schema_version(&conn).expect("version"), SCHEMA_VERSION);
        let rows = passkey_rows(&conn, "u1").expect("rows");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].rp_id, None);
        put_move_code(&conn, "u1", "h1", now_ms() + 1000).expect("code");
        assert_eq!(
            take_move_code(&conn, "h1").expect("take").as_deref(),
            Some("u1")
        );
        assert_eq!(take_move_code(&conn, "h1").expect("again"), None);
    }

    #[test]
    fn version_four_keeps_only_building_block_names() {
        let conn = Connection::open_in_memory().expect("create");
        for sql in [MIGRATION_1, MIGRATION_2, MIGRATION_3, MIGRATION_4] {
            conn.execute_batch(sql).expect("old schema");
        }
        conn.pragma_update(None, "user_version", 4).expect("v4");
        let users = [
            ("u1", Some("Jan")),
            ("u2", Some("a3.n12")),
            ("u3", Some("a27.n0.99")),
            ("u4", None),
            ("u5", Some("a28.n1")),
            ("u6", Some("a1.n2.0")),
            ("u7", Some("Der alte Fritz 🍺")),
        ];
        for (n, (id, name)) in users.iter().enumerate() {
            conn.execute(
                "INSERT INTO users (id, name, created_at, recovery_id, recovery_hash) VALUES (?1, ?2, 1, ?3, 'h')",
                params![id, name, format!("R{n}")],
            )
            .expect("user");
        }
        conn.execute_batch(
            "INSERT INTO daily_results VALUES ('20261109', 'u1', 'Declarer', 1, 3, 5, 0, 96, 1);",
        )
        .expect("result");
        migrate(&conn).expect("migrate");
        assert_eq!(schema_version(&conn).expect("version"), SCHEMA_VERSION);
        let names: Vec<Option<String>> = users
            .iter()
            .map(|(id, _)| user(&conn, id).expect("q").expect("kept").name)
            .collect();
        assert_eq!(
            names,
            [
                None,
                Some("a3.n12".into()),
                Some("a27.n0.99".into()),
                None,
                None,
                None,
                None,
            ]
        );
        // The player and the result stay; only the name went.
        assert_eq!(board_count(&conn, Board::Daily, "20261109").expect("n"), 1);
    }

    #[test]
    fn newer_database_is_refused() {
        let conn = Connection::open_in_memory().expect("db");
        conn.pragma_update(None, "user_version", SCHEMA_VERSION + 1)
            .expect("set");
        assert!(init(&conn).is_err());
    }

    #[test]
    fn file_database_settings_and_backups() {
        let path = temp_path("live.db");
        let conn = open(&path).expect("open");
        let mode: String = conn
            .query_row("PRAGMA journal_mode", [], |r| r.get(0))
            .expect("mode");
        assert_eq!(mode, "wal");
        let sync: i64 = conn
            .query_row("PRAGMA synchronous", [], |r| r.get(0))
            .expect("sync");
        assert_eq!(sync, 1, "NORMAL");
        let fk: i64 = conn
            .query_row("PRAGMA foreign_keys", [], |r| r.get(0))
            .expect("fk");
        assert_eq!(fk, 1);
        let pages: i64 = conn
            .query_row("PRAGMA max_page_count", [], |r| r.get(0))
            .expect("pages");
        let page: i64 = conn
            .query_row("PRAGMA page_size", [], |r| r.get(0))
            .expect("page");
        assert_eq!(pages * page, MAX_DB_BYTES);
        insert_user(&conn, "u1", Some("Jan"), "ABCDEFGH", "h").expect("user");

        let dir = temp_path("backup");
        for day in ["20261101", "20261102", "20261103"] {
            assert!(backup(&path, &dir, day, 2).expect("backup").is_some());
        }
        // A second run on the same day writes nothing.
        assert!(backup(&path, &dir, "20261103", 2).expect("again").is_none());
        let mut names: Vec<String> = std::fs::read_dir(&dir)
            .expect("dir")
            .map(|e| e.expect("entry").file_name().into_string().expect("utf8"))
            .collect();
        names.sort();
        assert_eq!(names, ["stammtisch-20261102.db", "stammtisch-20261103.db"]);
        let copy = Connection::open(dir.join("stammtisch-20261103.db")).expect("copy");
        assert!(user(&copy, "u1").expect("query").is_some());
        drop((conn, copy));
        let _ = std::fs::remove_dir_all(&dir);
        for ext in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(format!("{}{ext}", path.display()));
        }
    }

    #[test]
    fn full_database_fails_with_disk_full() {
        let conn = open_in_memory().expect("db");
        insert_user(&conn, "u1", None, "ABCDEFGH", "h").expect("user");
        set_max_bytes(&conn, 64 * 1024).expect("cap");
        let mut conn = conn;
        let err = put_save(&mut conn, "u1", None, 1, &"x".repeat(200_000)).expect_err("full");
        assert!(
            matches!(&err, rusqlite::Error::SqliteFailure(e, _) if e.code == rusqlite::ErrorCode::DiskFull),
            "{err}"
        );
    }

    #[test]
    fn recovery_swap_and_passkey_removal() {
        let conn = open_in_memory().expect("db");
        insert_user(&conn, "u1", None, "ABCDEFGH", "old").expect("user");
        assert!(swap_recovery(&conn, "u1", "old", "BBBBBBBB", "new").expect("swap"));
        // The second redemption of the same code finds it changed.
        assert!(!swap_recovery(&conn, "u1", "old", "CCCCCCCC", "other").expect("swap"));
        assert_eq!(
            recovery_hash(&conn, "u1").expect("q").as_deref(),
            Some("new")
        );

        insert_passkey(&conn, "a", "u1", "{}", "localhost").expect("a");
        insert_passkey(&conn, "b", "u1", "{}", "localhost").expect("b");
        assert_eq!(
            remove_passkey(&conn, "u1", "x").expect("x"),
            RemovePasskey::NotFound
        );
        assert_eq!(
            remove_passkey(&conn, "u1", "a").expect("a"),
            RemovePasskey::Removed
        );
        assert_eq!(
            remove_passkey(&conn, "u1", "b").expect("b"),
            RemovePasskey::Last
        );
        assert_eq!(passkey_count(&conn, "u1").expect("n"), 1);
    }

    #[test]
    fn reports_are_read_without_touching_the_file() {
        let path = temp_path("reports.db");
        // A database before migration 4: the export finds nothing.
        {
            let conn = Connection::open(&path).expect("open");
            conn.execute_batch(UNVERSIONED).expect("old schema");
        }
        let ro = open_read_only(&path).expect("read only");
        assert!(reports(&ro, 0, None, 10).expect("reports").is_empty());
        assert_eq!(schema_version(&ro).expect("version"), 0);
        drop(ro);
        // Migrated and written by the server, read next to it.
        let conn = open(&path).expect("open");
        assert!(insert_report(&conn, "B-1234", 5, "ios", "v", None, "text").expect("insert"));
        let ro = open_read_only(&path).expect("read only");
        let rows = reports(&ro, 0, Some("B-1234"), 10).expect("reports");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].data, "text");
        assert!(insert_report(&ro, "B-5678", 5, "ios", "v", None, "x").is_err());
        drop((ro, conn));
        for suffix in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(format!("{}{suffix}", path.display()));
        }
    }
}
