//! Stammgäste at the table (DESIGN 10, decided on 01.11.2026): the two
//! normal opponents get a name and a face, without rules of their own.
//!
//! At the start of every series two distinct guests sit down, drawn from
//! `guests.ron`. The draw comes from its own generator, derived from the
//! run's seed and the series alone ([`seat_guests`]), so it never advances
//! any other random stream: deals, Tresen, bosses and encounters are the
//! same as without guests. The guests of the previous series sit out the
//! next one when there are enough guests, so the same pair never comes
//! twice in a row.
//!
//! Pure presentation: the guests change nothing in play or AI. In a boss
//! series the frontend seats the boss on the left; the guest drawn for
//! that seat is not shown.

use serde::Serialize;

use super::Run;
use crate::content::{Content, GuestId};
use crate::game::Seat;
use crate::rng::Rng;

/// Salt of the guest stream ("GUESTS" + series), so it differs from every
/// other generator derived from the seed.
const GUEST_STREAM: u64 = 0x4755_4553_5453_0000;

/// The two guests of series `index` in `section`: distinct, drawn from
/// every guest except those in `previous` (all guests when fewer than two
/// would remain). Seat 1 first, then seat 2. Fewer than two only when the
/// content has fewer guests.
#[must_use]
pub fn seat_guests(
    content: &Content,
    seed: u64,
    section: u32,
    index: u32,
    previous: &[GuestId],
) -> Vec<GuestId> {
    let all: Vec<&GuestId> = content.guests.keys().collect();
    let fresh: Vec<&GuestId> = all
        .iter()
        .copied()
        .filter(|id| !previous.contains(id))
        .collect();
    let mut pool = if fresh.len() >= 2 { fresh } else { all };
    let series = (u64::from(section) << 8) | u64::from(index);
    let mut rng = Rng::from_seed(seed ^ GUEST_STREAM ^ series);
    let mut seated = Vec::with_capacity(2);
    for _ in 0..2 {
        if pool.is_empty() {
            break;
        }
        let pick = rng.gen_range(0..pool.len() as u64);
        let id = pool.remove(usize::try_from(pick).unwrap_or(0));
        seated.push(id.clone());
    }
    seated
}

/// A guest at the table, for the view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GuestInfo {
    /// Where the guest sits (seat 1 left, seat 2 right).
    pub seat: Seat,
    /// Content id.
    pub id: GuestId,
    /// German name.
    pub name: String,
    /// Id of the face the frontend draws.
    pub look: String,
    /// Speech bubble line when the guest sits down.
    pub greet: String,
    /// Speech bubble line when the guest takes a trick.
    pub win: String,
    /// Speech bubble line when the player takes a trick.
    pub lose: String,
}

impl Run {
    /// The guests of the current series, seat 1 first. Saves from before
    /// guests existed hold none; they get the draw of this series without
    /// a previous pair, which is deterministic too.
    #[must_use]
    pub fn guests(&self, content: &Content) -> Vec<GuestId> {
        if self.guests.is_empty() {
            seat_guests(
                content,
                self.seed,
                self.series.section,
                self.series.index,
                &[],
            )
        } else {
            self.guests.clone()
        }
    }

    /// Seat the guests of a new series (called when it starts).
    pub(super) fn seat_guests(&mut self, content: &Content, section: u32, index: u32) {
        self.guests = seat_guests(content, self.seed, section, index, &self.guests);
    }

    /// The guests of the current series for the view.
    pub(super) fn guest_infos(&self, content: &Content) -> Vec<GuestInfo> {
        self.guests(content)
            .iter()
            .zip(Seat::DEFENDERS)
            .filter_map(|(id, seat)| {
                let def = content.guest(id)?;
                Some(GuestInfo {
                    seat,
                    id: id.clone(),
                    name: def.name.clone(),
                    look: def.look.clone(),
                    greet: def.quips.greet.clone(),
                    win: def.quips.win.clone(),
                    lose: def.quips.lose.clone(),
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use crate::content::GuestDef;
    use crate::run::bot::RunBot;

    fn content() -> Content {
        Content::load().expect("content loads")
    }

    #[test]
    fn guests_have_unique_ids_and_text() {
        let content = content();
        assert!(
            (12..=16).contains(&content.guests.len()),
            "12 to 16 Stammgäste, got {}",
            content.guests.len()
        );
        let mut looks = BTreeSet::new();
        for (id, guest) in &content.guests {
            assert_eq!(id, &guest.id);
            assert!(!id.as_str().is_empty());
            assert!(!guest.name.trim().is_empty(), "{id}: name");
            assert!(!guest.look.trim().is_empty(), "{id}: look");
            assert!(looks.insert(guest.look.clone()), "{id}: look used twice");
            for line in [&guest.quips.greet, &guest.quips.win, &guest.quips.lose] {
                assert!(!line.trim().is_empty(), "{id}: empty quip");
                assert!(line.chars().count() <= 44, "{id}: quip too long: {line}");
            }
        }
        let names: BTreeSet<&str> = content.guests.values().map(|g| g.name.as_str()).collect();
        assert_eq!(names.len(), content.guests.len(), "names are unique");
    }

    #[test]
    fn duplicate_or_empty_guests_are_rejected() {
        let one = r#"(id: "a", name: "A", look: "a", quips: (greet: "x", win: "y", lose: "z"))"#;
        let twice = format!("[{one}, {one}]");
        let err = Content::from_sources(&crate::content::Sources {
            guests: &twice,
            ..crate::content::Sources::embedded()
        })
        .expect_err("duplicate id");
        assert!(err.0.contains("duplicate id a"), "{err}");
        let empty =
            r#"[(id: "a", name: " ", look: "a", quips: (greet: "x", win: "y", lose: "z"))]"#;
        assert!(
            Content::from_sources(&crate::content::Sources {
                guests: empty,
                ..crate::content::Sources::embedded()
            })
            .is_err()
        );
    }

    #[test]
    fn draw_is_deterministic_distinct_and_avoids_the_last_pair() {
        let content = content();
        for seed in 0..200 {
            let mut previous: Vec<GuestId> = Vec::new();
            for section in 0..5 {
                for index in 0..3 {
                    let a = seat_guests(&content, seed, section, index, &previous);
                    let b = seat_guests(&content, seed, section, index, &previous);
                    assert_eq!(a, b, "deterministic");
                    assert_eq!(a.len(), 2);
                    assert_ne!(a[0], a[1], "two different guests");
                    assert!(
                        a.iter().all(|g| !previous.contains(g)),
                        "seed {seed}: {a:?} after {previous:?}"
                    );
                    previous = a;
                }
            }
        }
    }

    #[test]
    fn every_guest_comes_to_the_table() {
        let content = content();
        let mut seen = BTreeSet::new();
        for seed in 0..100 {
            seen.extend(seat_guests(&content, seed, 0, 0, &[]));
        }
        assert_eq!(seen.len(), content.guests.len());
    }

    #[test]
    fn few_guests_still_seat_two() {
        let mut content = content();
        let keep: Vec<GuestDef> = content.guests.values().take(2).cloned().collect();
        content.guests = keep.into_iter().map(|g| (g.id.clone(), g)).collect();
        let first = seat_guests(&content, 1, 0, 0, &[]);
        let next = seat_guests(&content, 1, 0, 1, &first);
        assert_eq!(
            next.len(),
            2,
            "the same pair again rather than an empty seat"
        );
        content.guests.clear();
        assert!(seat_guests(&content, 1, 0, 0, &[]).is_empty());
    }

    #[test]
    fn run_seats_new_guests_each_series_and_shows_them() {
        let content = content();
        let mut changes = 0;
        for seed in 1_u64..=8 {
            let (mut run, _) = Run::new(&content, seed).expect("run");
            let view = run.view(&content);
            let seats: Vec<Seat> = view.series.guests.iter().map(|g| g.seat).collect();
            assert_eq!(seats, Seat::DEFENDERS.to_vec());
            assert_eq!(view.series.guests[0].id, run.guests(&content)[0]);
            assert!(!view.series.guests[1].name.is_empty());
            let mut series = (run.series().section, run.series().index);
            let mut previous = run.guests(&content);
            while let Some(action) = RunBot.next_action(&run, &content) {
                run.apply(&content, action).expect("bot action");
                let now = (run.series().section, run.series().index);
                if now != series {
                    let guests = run.guests(&content);
                    assert_eq!(guests, seat_guests(&content, seed, now.0, now.1, &previous));
                    assert!(guests.iter().all(|g| !previous.contains(g)));
                    series = now;
                    previous = guests;
                    changes += 1;
                } else {
                    assert_eq!(run.guests(&content), previous, "same guests all series");
                }
            }
        }
        assert!(changes >= 4, "the bot plays past a few series: {changes}");
    }

    #[test]
    fn saves_without_guests_load_and_get_a_fallback() {
        let content = content();
        let (run, _) = Run::new(&content, 7).expect("run");
        let text = ron::to_string(&run).expect("ron");
        let seated = run.guests(&content);
        let field = format!(",guests:[\"{}\",\"{}\"]", seated[0], seated[1]);
        assert!(text.contains(&field), "guests are saved");
        let old: Run = ron::from_str(&text.replace(&field, "")).expect("old save loads");
        let fallback = old.guests(&content);
        assert_eq!(fallback, seat_guests(&content, 7, 0, 0, &[]));
        assert_eq!(old.view(&content).series.guests.len(), 2);
    }
}
