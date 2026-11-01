//! A simple run policy for the simulator and for tests: the declarer
//! heuristic of M1 for games, passing weak hands while passes are left, and
//! at the Tresen buying the most expensive relic it can afford.
//!
//! It lives in the core (not in `sim/`) so that core tests can drive whole
//! runs with it; a frontend could use it for an "auto play" button.

use super::{Run, RunAction, RunPhase};
use crate::ai::{DeclarerHeuristic, Policy, trump_count};
use crate::content::Content;
use crate::deck::DealtCard;
use crate::game::{Phase, Seat};
use crate::rules::{Declaration, GameType, RiskLevel};

/// The simple run policy.
#[derive(Debug, Clone, Copy, Default)]
pub struct RunBot;

impl RunBot {
    /// Whether a hand is too weak to play: a suit game with fewer than five
    /// trumps, or a grand with fewer than two jacks.
    #[must_use]
    pub fn is_weak(hand: &[DealtCard], declaration: Declaration) -> bool {
        let trumps = trump_count(declaration.game, hand);
        match declaration.game {
            GameType::Grand => trumps < 2,
            _ => trumps < 5,
        }
    }

    /// The next action, or `None` once the run is over.
    #[must_use]
    pub fn next_action(&self, run: &Run, content: &Content) -> Option<RunAction> {
        match run.phase() {
            RunPhase::Won | RunPhase::Lost => None,
            RunPhase::Shop => Some(self.shop_action(run, content)),
            RunPhase::Game => self.game_action(run),
        }
    }

    fn shop_action(&self, run: &Run, content: &Content) -> RunAction {
        let Some(shop) = run.shop() else {
            return RunAction::LeaveShop;
        };
        let free_slot = run.relics().len() < usize::try_from(content.run.relic_slots).unwrap_or(0);
        let best = shop
            .relics
            .iter()
            .enumerate()
            .filter_map(|(slot, id)| {
                let def = content.relic(id.as_ref()?)?;
                (def.price <= run.coins()).then_some((def.price, std::cmp::Reverse(slot)))
            })
            .max();
        match best {
            Some((_, std::cmp::Reverse(slot))) if free_slot => RunAction::BuyRelic(slot),
            _ => RunAction::LeaveShop,
        }
    }

    fn game_action(&self, run: &Run) -> Option<RunAction> {
        let active = run.game()?;
        let game = &active.game;
        let declarer = DeclarerHeuristic;
        if active.awaiting_re {
            let declaration = game.declaration()?;
            let re = trump_count(declaration.game, game.hand(Seat::DECLARER)) >= 7;
            return Some(RunAction::AnswerKontra { re });
        }
        match game.phase() {
            Phase::Declaring => {
                let hand = game.hand(Seat::DECLARER);
                let mut declaration = declarer.choose_declaration(hand);
                if RunBot::is_weak(hand, declaration) {
                    if active.rules.redraws_left > 0 {
                        return Some(RunAction::RedrawSkat);
                    }
                    if run.series().passes_left > 0 {
                        return Some(RunAction::Pass);
                    }
                }
                if active.rules.require_hand && declaration.risk < RiskLevel::Hand {
                    declaration.risk = RiskLevel::Hand;
                }
                Some(RunAction::Declare(declaration))
            }
            Phase::Discarding => {
                let declaration = game.declaration()?;
                let [a, b] =
                    declarer.choose_discard(game.hand(Seat::DECLARER), declaration.game)?;
                Some(RunAction::Discard([a.id, b.id]))
            }
            Phase::Playing => {
                let view = game.view(Seat::DECLARER)?;
                let card = declarer.choose(&view)?;
                Some(RunAction::Play(card.id))
            }
            Phase::Finished(_) => None,
        }
    }
}
