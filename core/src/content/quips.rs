//! Speech bubble lines (Sprüche) of the people at the table: the
//! Stammgäste (`guests.ron`) and the two figures of every boss
//! (`bosses.ron`), DESIGN 10.4 and 11.
//!
//! Pure presentation: the core only loads, checks and passes the lines on
//! in the view. Which situation shows a line, and which of its variants,
//! the frontend decides (decoration, not game logic); nothing here touches
//! a random stream or the result of a game.

use std::fmt;

use serde::de::{self, Deserializer, SeqAccess, Visitor};
use serde::{Deserialize, Serialize};

/// A situation a person at the table has lines for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum QuipSituation {
    /// Sitting down: the first game of a series (for a boss figure: the
    /// entrance of the boss pub).
    Greet,
    /// The speaker's side takes a trick worth something.
    Win,
    /// The player takes a trick worth something.
    Lose,
    /// The speaker drops a Daus or a Zehn on a trick his partner takes
    /// (schmieren).
    Smear,
    /// The player declares a Grand.
    Grand,
    /// The player declares a Null.
    Null,
    /// The player declares Hand, Schneider or Schwarz angesagt, or Ouvert.
    Bold,
    /// A Ramsch starts.
    Ramsch,
    /// The speaker's side wins the game (the player loses it).
    Won,
    /// The player wins the game.
    Lost,
    /// The player wins with Schneider or Schwarz.
    Schneider,
    /// A boss figure: the player wins the boss pub.
    Beaten,
    /// A boss figure: the boss wins the boss pub.
    Victory,
}

impl QuipSituation {
    /// Every situation a Stammgast has lines for.
    pub const GUEST: [Self; 11] = [
        Self::Greet,
        Self::Win,
        Self::Lose,
        Self::Smear,
        Self::Grand,
        Self::Null,
        Self::Bold,
        Self::Ramsch,
        Self::Won,
        Self::Lost,
        Self::Schneider,
    ];

    /// Every situation a boss figure has lines for.
    pub const FIGURE: [Self; 7] = [
        Self::Greet,
        Self::Win,
        Self::Lose,
        Self::Won,
        Self::Lost,
        Self::Beaten,
        Self::Victory,
    ];
}

/// What a person at the table says, per situation several short lines
/// (variants, so they do not repeat). Every list may be empty: then the
/// person keeps quiet in that situation. In the content a single line may
/// stand without brackets (`greet: "Moin."`), as before 11.11.2026.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Quips {
    /// [`QuipSituation::Greet`].
    #[serde(default, deserialize_with = "lines")]
    pub greet: Vec<String>,
    /// [`QuipSituation::Win`].
    #[serde(default, deserialize_with = "lines")]
    pub win: Vec<String>,
    /// [`QuipSituation::Lose`].
    #[serde(default, deserialize_with = "lines")]
    pub lose: Vec<String>,
    /// [`QuipSituation::Smear`].
    #[serde(default, deserialize_with = "lines")]
    pub smear: Vec<String>,
    /// [`QuipSituation::Grand`].
    #[serde(default, deserialize_with = "lines")]
    pub grand: Vec<String>,
    /// [`QuipSituation::Null`].
    #[serde(default, deserialize_with = "lines")]
    pub null: Vec<String>,
    /// [`QuipSituation::Bold`].
    #[serde(default, deserialize_with = "lines")]
    pub bold: Vec<String>,
    /// [`QuipSituation::Ramsch`].
    #[serde(default, deserialize_with = "lines")]
    pub ramsch: Vec<String>,
    /// [`QuipSituation::Won`].
    #[serde(default, deserialize_with = "lines")]
    pub won: Vec<String>,
    /// [`QuipSituation::Lost`].
    #[serde(default, deserialize_with = "lines")]
    pub lost: Vec<String>,
    /// [`QuipSituation::Schneider`].
    #[serde(default, deserialize_with = "lines")]
    pub schneider: Vec<String>,
    /// [`QuipSituation::Beaten`].
    #[serde(default, deserialize_with = "lines")]
    pub beaten: Vec<String>,
    /// [`QuipSituation::Victory`].
    #[serde(default, deserialize_with = "lines")]
    pub victory: Vec<String>,
}

impl Quips {
    /// The lines of one situation.
    #[must_use]
    pub fn lines(&self, situation: QuipSituation) -> &[String] {
        match situation {
            QuipSituation::Greet => &self.greet,
            QuipSituation::Win => &self.win,
            QuipSituation::Lose => &self.lose,
            QuipSituation::Smear => &self.smear,
            QuipSituation::Grand => &self.grand,
            QuipSituation::Null => &self.null,
            QuipSituation::Bold => &self.bold,
            QuipSituation::Ramsch => &self.ramsch,
            QuipSituation::Won => &self.won,
            QuipSituation::Lost => &self.lost,
            QuipSituation::Schneider => &self.schneider,
            QuipSituation::Beaten => &self.beaten,
            QuipSituation::Victory => &self.victory,
        }
    }

    /// Every line of every situation.
    pub fn all(&self) -> impl Iterator<Item = &String> {
        [
            &self.greet,
            &self.win,
            &self.lose,
            &self.smear,
            &self.grand,
            &self.null,
            &self.bold,
            &self.ramsch,
            &self.won,
            &self.lost,
            &self.schneider,
            &self.beaten,
            &self.victory,
        ]
        .into_iter()
        .flatten()
    }
}

/// One line (`"…"`) or a list of lines (`["…", "…"]`).
fn lines<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<String>, D::Error> {
    struct Lines;
    impl<'de> Visitor<'de> for Lines {
        type Value = Vec<String>;
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("a line or a list of lines")
        }
        fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
            Ok(vec![v.to_owned()])
        }
        fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
            Ok(vec![v])
        }
        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let mut out = Vec::new();
            while let Some(line) = seq.next_element::<String>()? {
                out.push(line);
            }
            Ok(out)
        }
    }
    deserializer.deserialize_any(Lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_single_line_or_a_list() {
        let q: Quips = ron::from_str(r#"(greet: "Moin.", win: ["A", "B"])"#).expect("parses");
        assert_eq!(q.greet, ["Moin."]);
        assert_eq!(q.win, ["A", "B"]);
        assert!(q.lose.is_empty());
        assert_eq!(q.lines(QuipSituation::Win).len(), 2);
        assert_eq!(q.all().count(), 3);
    }
}
