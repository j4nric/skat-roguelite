//! The days of the daily leaderboard: everyone plays the same run per day
//! (the daily seed is the date, `JJJJMMTT`). The board itself is
//! [`crate::board`].

use crate::db;

/// Days since 1970-01-01 of a civil date (proleptic Gregorian).
pub(crate) const fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
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

/// Today's day number by the server's clock (UTC).
pub fn today() -> i64 {
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
