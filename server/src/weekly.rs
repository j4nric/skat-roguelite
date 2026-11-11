//! The weeks of the weekly leaderboard (Tafel der Woche, DESIGN 5.6): an
//! ISO week (weeks start on Monday, week 1 holds the first Thursday of
//! the year) as `JJJJWW`. Calendar only: the week's seed and Sonderregel
//! are the core's business, the server never knows them. The board itself
//! is [`crate::board`].

use crate::daily::{days_from_civil, today};

/// Weekday of a day number, 0 = Monday (1970-01-01 was a Thursday).
const fn weekday(days: i64) -> i64 {
    (days + 3).rem_euclid(7)
}

/// The Monday of ISO week 1 of `year`, as a day number.
const fn week_one_monday(year: i64) -> i64 {
    let jan4 = days_from_civil(year, 1, 4);
    jan4 - weekday(jan4)
}

/// ISO weeks of `year`: 52 or 53.
const fn weeks_in_year(year: i64) -> i64 {
    (week_one_monday(year + 1) - week_one_monday(year)) / 7
}

/// The week number (weeks since ISO week 1 of 2000) of a key `JJJJWW`,
/// if it names a real ISO week from 2000 to 2999.
pub fn parse_week(key: &str) -> Option<i64> {
    if key.len() != 6 || !key.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let year: i64 = key[..4].parse().ok()?;
    let week: i64 = key[4..].parse().ok()?;
    if !(2000..=2999).contains(&year) || !(1..=weeks_in_year(year)).contains(&week) {
        return None;
    }
    Some((week_one_monday(year) + (week - 1) * 7 - week_one_monday(2000)) / 7)
}

/// The week number of a day number.
fn week_of_day(days: i64) -> i64 {
    (days - weekday(days) - week_one_monday(2000)).div_euclid(7)
}

/// The key `JJJJWW` of a week number (inverse of [`parse_week`]).
pub fn key_of_week(week: i64) -> String {
    let monday = week_one_monday(2000) + week * 7;
    // The Thursday names the ISO year.
    let thursday = monday + 3;
    let mut year = 2000 + (thursday - week_one_monday(2000)).div_euclid(365);
    while week_one_monday(year + 1) <= monday {
        year += 1;
    }
    while week_one_monday(year) > monday {
        year -= 1;
    }
    let n = (monday - week_one_monday(year)) / 7 + 1;
    format!("{year:04}{n:02}")
}

/// This week's number by the server's clock (UTC).
pub fn this_week() -> i64 {
    week_of_day(today())
}

/// This week's key `JJJJWW` by the server's clock (UTC).
pub fn this_week_key() -> String {
    key_of_week(this_week())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weeks_parse() {
        assert_eq!(parse_week("200001"), Some(0));
        assert_eq!(parse_week("200002"), Some(1));
        assert_eq!(parse_week("202053"), parse_week("202101").map(|w| w - 1));
        assert_eq!(parse_week("202553"), None);
        assert_eq!(parse_week("202654"), None);
        assert_eq!(parse_week("202600"), None);
        assert_eq!(parse_week("199952"), None);
        assert_eq!(parse_week("2026-46"), None);
        assert_eq!(parse_week("20261"), None);
        // 2026-11-10 lies in week 46.
        assert_eq!(
            Some(week_of_day(days_from_civil(2026, 11, 10))),
            parse_week("202646")
        );
        // 2021-01-03 (Sunday) still belongs to 2020's week 53.
        assert_eq!(
            Some(week_of_day(days_from_civil(2021, 1, 3))),
            parse_week("202053")
        );
        for key in ["200001", "202053", "202101", "202646", "202653", "299952"] {
            assert_eq!(key_of_week(parse_week(key).expect(key)), key);
        }
        for w in 0..2000 {
            assert_eq!(parse_week(&key_of_week(w)), Some(w));
        }
    }
}
