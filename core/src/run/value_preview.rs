//! The value preview of the game in progress ([`ValuePreview`]): what the
//! game is worth if the player wins it, as far as it is certain now.
//!
//! Scored with the functions of a finished game (rules, `OnScore` hooks,
//! [`score_parts`]) on the smallest win the declaration allows, with only
//! what the player knows.

use std::collections::BTreeSet;

use super::view::NeededView;
use super::view::ValuePreview;
use super::{ActiveGame, Run, kontra_lines, score_line, win_factor_lines};
use crate::cards::PlayingCard;
use crate::cards::Suit;
use crate::content::{
    Change, Content, HookContext, Lever, NextGameEffect, ScoreLine, Source, Stage, evaluate_if_won,
    score_parts,
};
use crate::game::{Event, Phase, Seat, TRICKS_PER_GAME};
use crate::rules::{
    Declaration, GameType, Matadors, NullRisk, NullStyle, Outcome, RiskLevel, null_value_parts,
    ramsch_value_parts,
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
            thresholds.schneider_bonus = 0;
            let schneider = declaration.schneider_declared();
            // "Offenes Visier": Ouvert needs Schneider only.
            let schwarz = declaration.schwarz_declared()
                && !(declaration.ouvert() && thresholds.ouvert_schneider);
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
            let parts = content
                .run
                .game_value_parts(declaration, matadors, &outcome, &thresholds);
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
            transport: None,
            marked_cards: None,
            run: Some((self, content)),
        };
        for source in self.sources(content, true) {
            if !knowledge.counts(&source.source) {
                continue;
            }
            for hook in source.hooks.iter().filter(|h| h.stage() == Stage::OnScore) {
                if evaluate_if_won(hook.when(), &ctx, source.this_card, &possible) == Some(true) {
                    lines.extend(hook.then().iter().filter_map(|e| {
                        let e = e.scaled(source.scale);
                        score_line(
                            &source.source,
                            &e,
                            &self.counters,
                            self.count_for(content, &e),
                        )
                    }));
                }
            }
        }
        lines.extend(kontra_lines(active.kontra));
        lines.extend(win_factor_lines(active));
        let b = score_parts(declaration, matadors, &outcome, &thresholds, parts, lines);
        Some(ValuePreview {
            base_value: b.base_value,
            levels: b.levels,
            multiplier: b.multiplier,
            game_value: b.value,
        })
    }
}

/// Spielwerte of the plain formula (no relics, no factors) with the run's
/// Grundwerte (Grand 18), sorted: every suit game and Grand from "mit/ohne 1, Spiel 2" to the highest, and the
/// four Null values. The smallest of them a pub still needs is the hint of
/// [`NeededView::smallest_game`].
fn plain_values(run: &crate::content::RunConfig) -> Vec<u64> {
    let mut values: Vec<u64> = vec![23, 35, 46, 59];
    for game in [
        GameType::Suit(Suit::Diamonds),
        GameType::Suit(Suit::Hearts),
        GameType::Suit(Suit::Spades),
        GameType::Suit(Suit::Clubs),
        GameType::Grand,
    ] {
        let base = u64::from(run.base_value(game).unwrap_or(0));
        // Spitzen up to every trump, plus Spiel, Hand, Schneider, angesagt,
        // Schwarz, angesagt and Ouvert.
        let most = u64::try_from(game.trump_order().len()).unwrap_or(0) + 7;
        values.extend((2..=most).map(|levels| base * levels));
    }
    values.sort_unstable();
    values.dedup();
    values
}

/// How often a hook of `stage` can fire in one game at most, for the upper
/// bound of [`Run::max_game_value`]: once at the deal, the declaration and
/// the score, after each of the ten tricks and each of the 30 cards.
const fn max_firings(stage: Stage) -> usize {
    match stage {
        Stage::OnDeal | Stage::OnDeclare | Stage::OnScore => 1,
        Stage::OnTrickEnd => TRICKS_PER_GAME,
        Stage::OnCardPlayed => 3 * TRICKS_PER_GAME,
        Stage::Passive
        | Stage::OnSeriesWon
        | Stage::OnAttach
        | Stage::OnDiscard
        | Stage::OnRouteChosen
        | Stage::OnShopOpened
        | Stage::OnVoucherRedeemed
        | Stage::OnRelicSold
        | Stage::OnCardRewardSkipped => 0,
    }
}

/// Whether a score line can only raise a won game's value.
fn raises(line: &ScoreLine) -> bool {
    match (line.lever, line.change) {
        (Lever::BaseValue | Lever::Levels, Change::Add(n)) => n > 0,
        (Lever::BaseValue | Lever::GameValue, Change::Multiply(n)) => n > 1,
        (Lever::GameValue, Change::Scale(f)) => f > crate::content::Factor::ONE,
        _ => false,
    }
}

impl Run {
    /// "Noch nötig" before the last game of a pub (DESIGN 5.1): what the
    /// account still needs and whether a won game can bring it. `None`
    /// outside a game, before any other game than the last, and once the
    /// last game is declared (then the value preview takes over).
    pub(super) fn needed_view(&self, content: &Content) -> Option<NeededView> {
        let active = self.game.as_ref()?;
        if self.series.games_left() != 1 {
            return None;
        }
        let undeclared = match active.game.declaration() {
            None => true,
            Some(d) => d.is_ramsch() && active.game.tricks().is_empty(),
        };
        if !undeclared {
            return None;
        }
        let target = i64::try_from(self.series.target).unwrap_or(i64::MAX);
        let points = target.saturating_sub(self.series.score);
        let max_value = self.max_game_value(content);
        let smallest_game = u64::try_from(points)
            .ok()
            .filter(|&p| p > 0)
            .and_then(|p| plain_values(&content.run).into_iter().find(|&v| v >= p));
        Some(NeededView {
            points,
            smallest_game,
            max_value,
            reachable: u64::try_from(points).map_or(true, |p| p <= max_value),
        })
    }

    /// The most Augen the player can take in this game: 120, plus the
    /// Augen of the richest card in play if his edition holds a
    /// "Doppelgänger" (its copy counts the copied card's Augen instead of
    /// its own, DESIGN 7.2).
    fn max_declarer_points(&self, content: &Content) -> u32 {
        let mirror = self.edition.cards().iter().any(|c| {
            c.mods.iter().any(|m| {
                content
                    .card_mod(m)
                    .is_some_and(|d| d.passive.contains(&crate::content::Effect::Doppelganger))
            })
        });
        let richest = self.game.as_ref().map_or(0, |a| {
            a.game
                .deck_cards()
                .iter()
                .map(|c| c.card_points())
                .max()
                .unwrap_or(0)
        });
        120 + if mirror { richest } else { 0 }
    }

    /// An upper bound of the Spielwert a won game of the current pub can
    /// reach, without knowing the hand: the highest game of the formula
    /// (Grand and every suit Ouvert, Hand, Schwarz angesagt with every
    /// trump as Spitze; Null Ouvert Hand with every style bonus; in a
    /// Ramsch the loser with all 120 Augen and a Jungfrau) with every
    /// raising line of every relic, boss, character and card of the game
    /// as if its condition held, each as often as its stage can fire
    /// ([`max_firings`]), plus the waiting factors ("Doppelkorn",
    /// "Eierlikör", "Letzte Runde"). Kontra and Re do not count: the
    /// player cannot make a defender say Kontra.
    #[must_use]
    pub fn max_game_value(&self, content: &Content) -> u64 {
        let Some(active) = self.game.as_ref() else {
            return 0;
        };
        let mut lines: Vec<ScoreLine> = active
            .lines
            .iter()
            .chain(&win_factor_lines(active).collect::<Vec<_>>())
            .filter(|l| raises(l))
            .cloned()
            .collect();
        for source in self.sources(content, true) {
            for hook in source.hooks {
                for effect in hook.then() {
                    let effect = effect.scaled(source.scale);
                    // What the game decides counts at its most.
                    let count = match &*effect {
                        crate::content::Effect::AddLevelsPerVoidSuit { .. } => 3,
                        crate::content::Effect::AddBaseValuePerDeclarerPoints {
                            above,
                            step,
                            ..
                        } => {
                            self.max_declarer_points(content).saturating_sub(*above)
                                / (*step).max(1)
                        }
                        _ => self.count_for(content, &effect),
                    };
                    let Some(line) = score_line(&source.source, &effect, &self.counters, count)
                    else {
                        continue;
                    };
                    if raises(&line) {
                        let times = max_firings(hook.stage());
                        lines.extend(std::iter::repeat_n(line, times));
                    }
                }
            }
        }
        if let Some((encounter, factor)) = &self.pending_factor {
            lines.push(ScoreLine {
                source: Source::Encounter(encounter.clone()),
                lever: Lever::GameValue,
                change: Change::Multiply(*factor),
            });
        }
        for (id, effect) in &self.pending_next_game {
            if let NextGameEffect::Multiply(n) | NextGameEffect::MultiplyWin(n) = effect {
                lines.push(ScoreLine {
                    source: Source::Consumable(id.clone()),
                    lever: Lever::GameValue,
                    change: Change::Multiply(*n),
                });
            }
        }
        let tricks = u32::try_from(TRICKS_PER_GAME).unwrap_or(10);
        let thresholds = &active.rules.thresholds;
        let mut best = 0;
        let mut consider =
            |declaration: Declaration, matadors: Matadors, outcome: Outcome, parts| {
                let b = score_parts(
                    declaration,
                    matadors,
                    &outcome,
                    thresholds,
                    parts,
                    lines.clone(),
                );
                best = best.max(b.value);
            };
        if active.rules.ramsch {
            let outcome = Outcome {
                declarer_card_points: 0,
                defender_card_points: 120,
                declarer_tricks: 0,
                defender_tricks: tricks,
            };
            let parts = ramsch_value_parts([0, 120, 0], [0, tricks, 0]);
            consider(
                Declaration::Ramsch,
                Matadors {
                    with: false,
                    count: 0,
                },
                outcome,
                parts,
            );
            return best;
        }
        let schwarz = Outcome {
            declarer_card_points: 120,
            defender_card_points: 0,
            declarer_tricks: tricks,
            defender_tricks: 0,
        };
        for game in [
            GameType::Grand,
            GameType::Suit(Suit::Clubs),
            GameType::Suit(Suit::Spades),
            GameType::Suit(Suit::Hearts),
            GameType::Suit(Suit::Diamonds),
        ] {
            let Some(declaration) = Declaration::new(game, RiskLevel::Ouvert) else {
                continue;
            };
            let order = crate::rules::matador_order(game, active.game.grand_aces_matadors());
            let matadors = Matadors {
                with: true,
                count: u32::try_from(order.len()).unwrap_or(0),
            };
            let parts = content
                .run
                .game_value_parts(declaration, matadors, &schwarz, thresholds);
            consider(declaration, matadors, schwarz, parts);
        }
        let null = Outcome {
            declarer_card_points: 0,
            defender_card_points: 120,
            declarer_tricks: 0,
            defender_tricks: tricks,
        };
        let style = NullStyle {
            sevens_ducked: 4,
            never_followed: true,
            defender_points_after: Some(120),
        };
        let risk = NullRisk::OuvertHand;
        let parts = null_value_parts(risk, &self.null_scoring(content), &null, &style);
        consider(
            Declaration::Null { risk },
            Matadors {
                with: false,
                count: 0,
            },
            null,
            parts,
        );
        best
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
}

/// Where a trump may be, as far as the player knows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Place {
    /// Among the declarer's twelve cards (hand plus skat).
    held: bool,
    /// In play with a defender.
    not_held: bool,
    /// No Spitze in this game: a Unter the player's edition demotes
    /// ("Abgestuft") that is or may be among his twelve cards.
    absent: bool,
}

impl Knowledge {
    fn new(active: &ActiveGame) -> Knowledge {
        let game = &active.game;
        // Picked up, or looked at with "Kurzer".
        let skat_known = game.skat_picked_up() || active.skat_peeked;
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
        // Defender cards "Durchsichtig" for this game ("Glasauge").
        known.extend(active.transparent_for_game.iter().copied());
        // A skat card shown after a hand declaration ("Spickzettel") and the
        // card given back to a defender ("Drei drücken").
        known.extend(active.skat_shown.iter().map(|c| c.id));
        known.extend(game.returned().map(|(_, c)| c.id));
        Knowledge { known, skat_known }
    }

    /// Whether lines and hooks of `source` count for the preview: card
    /// modifications only on cards whose holder the player knows (ADR
    /// 0015: a card's modifications are its holder's edition, so an
    /// unknown card might sit in another edition).
    fn counts(&self, source: &Source) -> bool {
        match source {
            Source::CardMod { card, .. } => self.known.contains(card),
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
        // The player's edition: a Unter it demotes is no Unter in his
        // twelve cards (ADR 0015, DESIGN 9.2).
        let demoted_for_player = |id: u32| {
            game.editions()[Seat::DECLARER.index()]
                .iter()
                .any(|e| e.id == id && crate::rules::is_demoted_jack(*e))
        };
        let places: Vec<Place> = crate::rules::matador_order(game_type, game.grand_aces_matadors())
            .into_iter()
            .filter_map(|top| {
                let copies: Vec<_> = deck.iter().filter(|c| c.face() == Some(top)).collect();
                if copies.is_empty() {
                    return None;
                }
                let held_copy = copies
                    .iter()
                    .find(|c| self.known.contains(&c.id) && declarer.iter().any(|d| d.id == c.id));
                if let Some(c) = held_copy {
                    let demoted = crate::rules::is_demoted_jack(**c);
                    return Some(Place {
                        held: !demoted,
                        not_held: false,
                        absent: demoted,
                    });
                }
                let unknown: Vec<_> = copies
                    .iter()
                    .filter(|c| !self.known.contains(&c.id))
                    .collect();
                let known_in_play = copies.iter().any(|c| self.known.contains(&c.id));
                let maybe_held = !unknown.is_empty() && !self.skat_known;
                let demoted = unknown.iter().any(|c| demoted_for_player(c.id));
                Some(Place {
                    held: maybe_held && !demoted,
                    not_held: known_in_play || !unknown.is_empty(),
                    absent: maybe_held && demoted,
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
    use crate::content::{BossId, Factor};
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
        "D7 DJ H7 HJ S7 SJ C7 CJ D8 H8",
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
        assert_eq!((p.base_value, p.levels, p.multiplier), (9, 4, Factor::ONE));
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
        let content = crate::run::legacy_content();
        // Mit 4, Spiel, Hand: 18 × 6 (Grand 18 in the run since 04.11.2026).
        let mut run = relic_run(&content, &[]);
        fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
        run.apply(
            &content,
            RunAction::Declare(Declaration::grand(RiskLevel::Hand)),
        )
        .expect("declare");
        let p = preview(&run, &content);
        assert_eq!((p.base_value, p.levels, p.game_value), (18, 6, 108));

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
        assert_eq!((p.base_value, p.levels, p.multiplier), (20, 7, Factor::ONE));
        assert_eq!(p.game_value, 140);

        // Declared Schneider: "Durchmarsch" is certain for a win.
        let mut run = relic_run(&content, &["march_through"]);
        fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
        run.apply(
            &content,
            RunAction::Declare(Declaration::grand(RiskLevel::SchneiderDeclared)),
        )
        .expect("declare");
        let p = preview(&run, &content);
        // Mit 4, Spiel, Hand, Schneider, angesagt: 18 × 8 × 2.
        assert_eq!(
            (p.base_value, p.levels, p.multiplier),
            (18, 8, Factor::from_int(2))
        );
        assert_eq!(p.game_value, 288);
    }

    #[test]
    fn ouvert_bonus_and_kontra_count_at_once() {
        let content = crate::run::legacy_content();
        let mut run = relic_run(&content, &["ouvert_bonus"]);
        fixed_deal(&mut run, &content, STRONG, STRONG_SKAT);
        run.apply(
            &content,
            RunAction::Declare(Declaration::grand(RiskLevel::Ouvert)),
        )
        .expect("declare");
        // Mit 4, Spiel, Hand, Schneider, angesagt, Schwarz, angesagt,
        // Ouvert, plus the relic: 18 × 12.
        let p = preview(&run, &content);
        assert_eq!((p.base_value, p.levels, p.game_value), (18, 12, 216));
        // Kontra doubles what is certain.
        let active = run.game.as_mut().expect("game");
        active.kontra = super::super::Kontra::Kontra(Seat::ALL[1]);
        let p = preview(&run, &content);
        assert_eq!((p.multiplier, p.game_value), (Factor::from_int(2), 432));
    }

    #[test]
    fn null_counts_the_risk_and_certain_relics_without_style() {
        let content = crate::run::legacy_content();
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
        let content = crate::run::legacy_content();
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
        // A demoted Unter that may lie in the skat is skipped or counted.
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
