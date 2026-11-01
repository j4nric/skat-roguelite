//! The value preview of the game in progress ([`ValuePreview`]): what the
//! game is worth if the player wins it, as far as it is certain now.
//!
//! Scored with the functions of a finished game (rules, `OnScore` hooks,
//! [`score_parts`]) on the smallest win the declaration allows, with only
//! what the player knows.

use std::collections::BTreeSet;

use super::view::ValuePreview;
use super::{ActiveGame, Run, kontra_lines, score_line};
use crate::cards::PlayingCard;
use crate::content::{
    Content, HookContext, ScoreLine, Source, Stage, evaluate_if_won, score_parts,
};
use crate::game::{Event, Phase, Seat, TRICKS_PER_GAME};
use crate::rules::{
    Declaration, GameType, Matadors, NullStyle, Outcome, game_value_parts, null_value_parts,
};

impl Run {
    /// The value preview while playing; `None` before the declaration,
    /// after the game and in a Ramsch. See [`ValuePreview`] for what it
    /// includes.
    pub(super) fn value_preview(&self, content: &Content) -> Option<ValuePreview> {
        let active = self.game.as_ref()?;
        let game = &active.game;
        if game.phase() != Phase::Playing {
            return None;
        }
        let declaration = game.declaration()?;
        let knowledge = Knowledge::new(active);
        let mut thresholds = active.rules.thresholds.clone();
        let possible = match declaration {
            Declaration::Ramsch => return None,
            Declaration::Null { .. } => vec![Matadors {
                with: false,
                count: 0,
            }],
            Declaration::Suit { .. } | Declaration::Grand { .. } => {
                knowledge.possible_matadors(active, declaration.game())
            }
        };
        // The smallest count the player cannot rule out.
        let matadors = possible
            .iter()
            .copied()
            .min_by_key(|m| m.count)
            .unwrap_or(Matadors {
                with: false,
                count: 0,
            });
        let tricks = u32::try_from(TRICKS_PER_GAME).unwrap_or(10);
        let (parts, outcome) = if let Declaration::Null { risk } = declaration {
            // A won Null: the declarer takes no trick; no style yet.
            let outcome = Outcome {
                defender_tricks: tricks,
                ..Outcome::default()
            };
            let scoring = self.null_scoring(content);
            let parts = null_value_parts(risk, &scoring, &outcome, &NullStyle::default());
            (parts, outcome)
        } else {
            // The smallest win: just enough Augen, Schneider and Schwarz
            // only if declared (then winning needs them). Augen ranges
            // that make a game Schneider depend on the Augen played.
            thresholds.declarer_schneider.clear();
            let schneider = declaration.schneider_declared();
            let schwarz = declaration.schwarz_declared();
            let outcome = Outcome {
                declarer_card_points: thresholds.win,
                defender_card_points: if schneider {
                    thresholds.schneider_limit
                } else {
                    thresholds.schneider_limit.saturating_add(1)
                },
                declarer_tricks: if schwarz { tricks } else { tricks - 1 },
                defender_tricks: u32::from(!schwarz),
            };
            let parts = game_value_parts(declaration, matadors, &outcome, &thresholds);
            (parts, outcome)
        };

        // Lines collected so far, then the OnScore hooks certain for a win.
        let mut lines: Vec<ScoreLine> = active
            .lines
            .iter()
            .filter(|l| knowledge.counts(&l.source))
            .cloned()
            .collect();
        let ctx = HookContext {
            flags: &self.flags,
            counters: &self.counters,
            game: Some(game),
            played: None,
            result: Some(&parts),
            games_left: self.series.games_left(),
        };
        for source in self.sources(content, true) {
            if !knowledge.counts(&source.source) {
                continue;
            }
            for hook in source.hooks.iter().filter(|h| h.stage() == Stage::OnScore) {
                if evaluate_if_won(hook.when(), &ctx, source.this_card, &possible) == Some(true) {
                    lines.extend(
                        hook.then()
                            .iter()
                            .filter_map(|e| score_line(&source.source, e, &self.counters)),
                    );
                }
            }
        }
        lines.extend(kontra_lines(active.kontra));
        let b = score_parts(declaration, matadors, &outcome, &thresholds, parts, lines);
        Some(ValuePreview {
            base_value: b.base_value,
            levels: b.levels,
            multiplier: b.multiplier,
            game_value: b.value,
        })
    }
}

/// What the player knows about where the cards of the game are.
struct Knowledge {
    /// Ids of the cards whose place the player knows: his own cards, the
    /// skat once picked up, played cards, revealed and peeked cards. All
    /// of them are in play.
    known: BTreeSet<u32>,
    /// Whether the player has seen the skat.
    skat_known: bool,
    /// Whether cards lie in the talon (deck larger than 32 cards).
    talon: bool,
}

/// Where a trump may be, as far as the player knows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Place {
    /// Among the declarer's twelve cards (hand plus skat).
    held: bool,
    /// In play with a defender.
    not_held: bool,
    /// Not in play (every copy in the talon).
    absent: bool,
}

impl Knowledge {
    fn new(active: &ActiveGame) -> Knowledge {
        let game = &active.game;
        let skat_known = game.skat_picked_up();
        let mut known: BTreeSet<u32> = game.hand(Seat::DECLARER).iter().map(|c| c.id).collect();
        known.extend(game.tricks().iter().flat_map(|t| t.cards).map(|c| c.id));
        known.extend(game.current_trick().iter().map(|c| c.id));
        if skat_known {
            known.extend(game.skat().iter().map(|c| c.id));
        }
        // A card the player put into the untouched skat ("Die Oma").
        for event in game.log() {
            if let Event::SkatSwapped { gave, .. } = event {
                known.insert(gave.id);
            }
        }
        known.extend(
            game.cards_in_play()
                .iter()
                .filter(|c| c.revealed)
                .map(|c| c.id),
        );
        known.extend(active.peeked.iter().map(|(c, _)| c.id));
        Knowledge {
            known,
            skat_known,
            talon: !game.talon().is_empty(),
        }
    }

    /// Whether the player knows the card is in play: always without a
    /// talon.
    fn in_play(&self, card: u32) -> bool {
        !self.talon || self.known.contains(&card)
    }

    /// Whether lines and hooks of `source` count for the preview: card
    /// modifications only on cards the player knows to be in play, so the
    /// preview does not tell which cards lie in the talon.
    fn counts(&self, source: &Source) -> bool {
        match source {
            Source::CardMod { card, .. } => self.in_play(*card),
            _ => true,
        }
    }

    /// Every count of Spitzen that fits what the player knows, counted
    /// along the trumps in play as [`crate::rules::matadors_in`] does.
    /// One entry once the count is certain.
    fn possible_matadors(&self, active: &ActiveGame, game_type: GameType) -> Vec<Matadors> {
        let game = &active.game;
        let declarer = game.declarer_cards();
        let deck = game.deck_cards();
        let places: Vec<Place> = game_type
            .trump_order()
            .into_iter()
            .filter_map(|top| {
                let copies: Vec<_> = deck.iter().filter(|c| c.face() == Some(top)).collect();
                if copies.is_empty() {
                    return None;
                }
                let known_held = copies
                    .iter()
                    .any(|c| self.known.contains(&c.id) && declarer.iter().any(|d| d.id == c.id));
                if known_held {
                    return Some(Place {
                        held: true,
                        not_held: false,
                        absent: false,
                    });
                }
                let unknown = copies.iter().any(|c| !self.known.contains(&c.id));
                let known_in_play = copies.iter().any(|c| self.known.contains(&c.id));
                Some(Place {
                    held: unknown && !self.skat_known,
                    not_held: known_in_play || unknown,
                    absent: !known_in_play && self.talon,
                })
            })
            .collect();
        let mut out = BTreeSet::new();
        walk(&places, None, 0, &mut out);
        out.into_iter()
            .map(|(with, count)| Matadors { with, count })
            .collect()
    }
}

/// Every (mit, count) of Spitzen over the possible places of the trumps,
/// strongest first: the first trump in play decides mit or ohne, the count
/// runs while the next trump in play agrees.
fn walk(places: &[Place], first: Option<bool>, count: u32, out: &mut BTreeSet<(bool, u32)>) {
    let Some((place, rest)) = places.split_first() else {
        out.insert((first.unwrap_or(false), count));
        return;
    };
    if place.absent {
        walk(rest, first, count, out);
    }
    for (possible, held) in [(place.held, true), (place.not_held, false)] {
        if !possible {
            continue;
        }
        match first {
            None => walk(rest, Some(held), 1, out),
            Some(with) if with == held => walk(rest, first, count + 1, out),
            Some(with) => {
                out.insert((with, count));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::content_tests::{content, declare, fixed_deal, relic_run};
    use super::super::{Character, RelicId, RunAction, RunStart};
    use super::*;
    use crate::cards::Suit;
    use crate::content::BossId;
    use crate::rules::{NullRisk, RiskLevel};

    /// The declarer holds CJ, HJ and the diamonds; SJ and HA lie in the
    /// skat, DJ with seat 2: mit 3 in a diamonds game.
    const MIT_3: [&str; 3] = [
        "CJ HJ DA DT DK DQ D9 D8 D7 H7",
        "HT HK HQ H9 H8 SA ST SK SQ S9",
        "S8 S7 CA CT CK CQ C9 C8 C7 DJ",
    ];
    const MIT_3_SKAT: &str = "SJ HA";

    /// The declarer holds all four jacks and the diamonds but the D8.
    const STRONG: [&str; 3] = [
        "CJ SJ HJ DJ DA DT DK DQ D9 D7",
        "HA HT HK HQ H9 H8 H7 SA ST SK",
        "SQ S9 S8 S7 CA CT CK CQ C9 C8",
    ];
    const STRONG_SKAT: &str = "D8 C7";

    /// A Null deal the declarer cannot lose (all eight sevens).
    const SAFE_NULL: [&str; 3] = [
        "D7 D7 H7 H7 S7 S7 C7 C7 D8 H8",
        "D9 DT DQ DK DA H9 HT HQ HK HA",
        "S8 S9 ST SQ SK SA C8 C9 CT CQ",
    ];
    const SAFE_NULL_SKAT: &str = "CK CA";

    fn preview(run: &Run, content: &Content) -> ValuePreview {
        run.view(content)
            .game
            .expect("game")
            .value_preview
            .expect("preview")
    }

    fn diamonds(risk: RiskLevel) -> Declaration {
        Declaration::suit(Suit::Diamonds, risk)
    }

    #[test]
    fn suit_game_after_the_skat_counts_the_spitzen_of_hand_and_skat() {
        let content = content();
        let mut run = relic_run(&content, &[]);
        fixed_deal(&mut run, &content, MIT_3, MIT_3_SKAT);
        assert!(
            run.view(&content)
                .game
                .expect("game")
                .value_preview
                .is_none()
        );
        declare(&mut run, &content, diamonds(RiskLevel::Normal)).expect("declare");
        let matadors = run.game().expect("game").game.matadors().expect("declared");
        assert_eq!(matadors.count, 3);
        // Diamonds 9 × (mit 3 + Spiel).
        let p = preview(&run, &content);
        assert_eq!((p.base_value, p.levels, p.multiplier), (9, 4, 1));
        assert_eq!(p.game_value, 36);
    }

    #[test]
    fn a_hand_game_does_not_tell_what_lies_in_the_skat() {
        let content = content();
        let mut run = relic_run(&content, &[]);
        fixed_deal(&mut run, &content, MIT_3, MIT_3_SKAT);
        run.apply(&content, RunAction::Declare(diamonds(RiskLevel::Hand)))
            .expect("declare");
        // Mit 3 with the SJ in the skat, but the player only knows CJ and
        // HJ: the SJ may sit with a defender, so mit 1 is all he knows.
        // 9 × (mit 1 + Spiel + Hand).
        let p = preview(&run, &content);
        assert_eq!((p.base_value, p.levels), (9, 3));
        assert_eq!(p.game_value, 27);
        // Schneider angesagt: + Schneider + angesagt.
        let mut run = relic_run(&content, &[]);
        fixed_deal(&mut run, &content, MIT_3, MIT_3_SKAT);
        run.apply(
            &content,
            RunAction::Declare(diamonds(RiskLevel::SchneiderDeclared)),
        )
        .expect("declare");
        assert_eq!(preview(&run, &content).levels, 5);
    }

    #[test]
    fn grand_counts_certain_relic_levels_and_base_value() {
        let content = content();
        // Mit 4, Spiel, Hand: 24 × 6.
        let mut run = relic_run(&content, &[]);
        fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
        run.apply(
            &content,
            RunAction::Declare(Declaration::grand(RiskLevel::Hand)),
        )
        .expect("declare");
        let p = preview(&run, &content);
        assert_eq!((p.base_value, p.levels, p.game_value), (24, 6, 144));

        // "Mit Vieren" (+1 level for certain mit 4), "Tournee" with two
        // strokes (+2 Grundwert), "Durchmarsch" only pays for Schneider.
        let mut run = relic_run(&content, &["with_four", "tour", "march_through"]);
        run.counters.insert("tour".into(), 2);
        fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
        run.apply(
            &content,
            RunAction::Declare(Declaration::grand(RiskLevel::Hand)),
        )
        .expect("declare");
        let p = preview(&run, &content);
        assert_eq!((p.base_value, p.levels, p.multiplier), (26, 7, 1));
        assert_eq!(p.game_value, 182);

        // Declared Schneider: "Durchmarsch" is certain for a win.
        let mut run = relic_run(&content, &["march_through"]);
        fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
        run.apply(
            &content,
            RunAction::Declare(Declaration::grand(RiskLevel::SchneiderDeclared)),
        )
        .expect("declare");
        let p = preview(&run, &content);
        // Mit 4, Spiel, Hand, Schneider, angesagt: 24 × 8 × 2.
        assert_eq!((p.base_value, p.levels, p.multiplier), (24, 8, 2));
        assert_eq!(p.game_value, 384);
    }

    #[test]
    fn ouvert_bonus_and_kontra_count_at_once() {
        let content = content();
        let mut run = relic_run(&content, &["ouvert_bonus"]);
        fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
        run.apply(
            &content,
            RunAction::Declare(Declaration::grand(RiskLevel::Ouvert)),
        )
        .expect("declare");
        // Mit 4, Spiel, Hand, Schneider, angesagt, Schwarz, angesagt,
        // Ouvert, plus the relic: 24 × 12.
        let p = preview(&run, &content);
        assert_eq!((p.base_value, p.levels, p.game_value), (24, 12, 288));
        // Kontra doubles what is certain.
        let active = run.game.as_mut().expect("game");
        active.kontra = super::super::Kontra::Kontra(Seat::ALL[1]);
        let p = preview(&run, &content);
        assert_eq!((p.multiplier, p.game_value), (2, 576));
    }

    #[test]
    fn null_counts_the_risk_and_certain_relics_without_style() {
        let content = content();
        let start = RunStart {
            character: Character::NullPlayer,
            ..RunStart::default()
        };
        let (mut run, _) = Run::new_with(&content, 11, &start).expect("null run");
        run.relics = ["blind_flight", "seven_sleeper"]
            .iter()
            .map(|r| RelicId::new(r))
            .collect();
        fixed_deal(&mut run, &content, SAFE_NULL, SAFE_NULL_SKAT);
        run.apply(
            &content,
            RunAction::Declare(Declaration::null(NullRisk::Hand)),
        )
        .expect("declare");
        // Formula: 23 × (Spiel + Hand) + "Blindflug" (won Null Hand). The
        // style relic "Siebenschläfer" waits for a ducked seven.
        let p = preview(&run, &content);
        assert_eq!((p.base_value, p.levels, p.game_value), (23, 3, 69));
    }

    #[test]
    fn a_ramsch_has_no_preview() {
        let content = content();
        let mut run = relic_run(&content, &[]);
        run.series.boss = Some(BossId::new("ramscher"));
        let mut events = Vec::new();
        run.deal(&content, true, &mut events).expect("deal");
        let game = run.view(&content).game.expect("game");
        assert_eq!(game.declaration, Some(Declaration::Ramsch));
        assert!(game.value_preview.is_none());
    }

    #[test]
    fn the_walk_agrees_with_the_rules_when_everything_is_known() {
        let place = |held| Place {
            held,
            not_held: !held,
            absent: false,
        };
        let mut out = BTreeSet::new();
        walk(
            &[place(true), place(true), place(false), place(true)],
            None,
            0,
            &mut out,
        );
        assert_eq!(out.into_iter().collect::<Vec<_>>(), vec![(true, 2)]);
        let mut out = BTreeSet::new();
        walk(
            &[place(false), place(false), place(true)],
            None,
            0,
            &mut out,
        );
        assert_eq!(out.into_iter().collect::<Vec<_>>(), vec![(false, 2)]);
        // A trump that may lie in the talon is skipped or counted.
        let maybe = Place {
            held: false,
            not_held: true,
            absent: true,
        };
        let mut out = BTreeSet::new();
        walk(&[place(true), maybe, place(true)], None, 0, &mut out);
        assert_eq!(
            out.into_iter().collect::<Vec<_>>(),
            vec![(true, 1), (true, 2)]
        );
    }
}
