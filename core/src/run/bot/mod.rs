//! A simple run policy for the simulator and for tests.
//!
//! Games: the declarer heuristic of M1 for the card play and the suit game
//! or grand, plus run decisions on top. Every character may declare every
//! game (ADR 0012), so the bot chooses between suit game, grand and Null
//! for both:
//!
//! - **Hand or skat** (Skatordnung, ADR 0011): with ten cards the bot
//!   declares a hand game only for a very strong hand: Null Ouvert Hand
//!   without an exposed card (classic Null rule, see
//!   [`crate::ai::null_exposure`]), a suit game or grand with eight trumps
//!   or an announcement below. Otherwise it picks up the skat and chooses
//!   with twelve cards (`RunBot::after_skat`): the suit game or grand of
//!   [`DeclarerHeuristic::choose_game_after_skat`](crate::ai::DeclarerHeuristic::choose_game_after_skat) or Null (the two most
//!   exposed cards discarded, Null Ouvert if none is left), whichever has
//!   the better expected score ([`suit_expected_score`],
//!   [`null_expected_score`], win rates measured with the PIMC player). Null
//!   is left out where a won Null would count nothing (no boss does that
//!   since "Der Schwarze Peter" pays half, but a rule could).
//!   A hand game redeems a random voucher of the hand (04.11.2026, DESIGN
//!   7.2); the bot does not count it in its hand rule: a few Groschen
//!   weigh little against the risk of the hand game, to be measured once
//!   the simulations run again.
//! - **Einmischen** ([`RunBot::is_weak`]): a suit game needs six trumps,
//!   or five trumps and two aces; a grand needs two jacks and two aces;
//!   Null at most [`NULL_MAX_EXPOSED`] exposed cards. A hand weak for both
//!   is thrown in, cheapest first: an Einmischen left in this game (or from
//!   the series' extras), the consumable "Neues Blatt", and with none left
//!   "Ein Bier" for one more. With nothing left it is played. The balancing
//!   measurement of 01.11.2026 showed that the weaker hands lose points on
//!   average (`docs/balance/2026-11-01-m2-zielwerte.md`). "Reue" is never
//!   used. Since 05.11.2026 a hand that is not weak is thrown in too while
//!   the game has an Einmischen of its own left and its
//!   [`ten_card_score`] is below [`RunBot::redeal_threshold`].
//! - **Rescue Schnäpse** (`RunBot::rescue_schnapps`, before Skat oder
//!   Hand): "Klarer" with the account below 0, "Absacker" before the last
//!   game short of the target, "Eierlikör" whenever it is accepted. The
//!   bot never gives a pub up ("Kneipe verlassen"): a lost pub ends the
//!   run either way (ADR 0019), and played a miracle may still help.
//! - **Forced hand** ("Der Schieber", `RunBot::forced_hand`): Null Hand
//!   only with at most [`NULL_HAND_MAX_EXPOSED`] exposed cards, else the
//!   suit game or grand with Hand; the better expected score if both fit.
//! - **Always open** ("Der Angeber"): an open suit game or grand only with
//!   an Ouvert hand, otherwise Null Ouvert (Hand without exposed card).
//! - **Announcements** ([`RunBot::announcement`]): Schneider angesagt or
//!   Ouvert only for hands that make it almost surely (conservative rules,
//!   measured against the defender heuristic, see the function), from the
//!   hand and, with the same rules for the ten cards kept, after the skat
//!   (ADR 0013). Re on Kontra only with a very strong hand
//!   ([`RunBot::says_re`]).
//! - **Relics that pay for announcements** (relics v4, 07.11.2026): with
//!   "Große Klappe" or "Offenes Visier" the bot announces Schneider or
//!   plays Ouvert after the skat when double-dummy deals of the unseen
//!   cards say Schneider is near certain ([`schneider_percent`],
//!   `RunBot::bold_announcement`). With "Kahlschlag" it discards for void
//!   side suits (`RunBot::void_discard`). Without these relics nothing
//!   changes.
//!
//! Tresen ([`RunBot::shop_action`]): one action per call, in this order:
//! take the Kartenmaler's best modification while he waits, "Karte
//! reinigen" on a cursed card, sell the weakest relic for a clearly better
//! one when all slots are full, buy the most valuable relic, buy the most
//! valuable consumable, buy the best offer of the Auslage on a card that
//! carries nothing else (never replacing), hire the Kartenmaler for an ace
//! without modification when Groschen are plenty, draw a new offer when
//! nothing sensible can be bought and Groschen are plenty, leave. Values
//! come from [`relic_value`], [`consumable_value`] and [`zinken_value`].
//! The bot never pays on credit for Zinken. A trade offer ("Hehlerware",
//! "Schnäppchen") counts [`TRADE_PENALTY`] less at its half price; with
//! "Pechvogel" it counts in full and the bot keeps its curses.
//!
//! Encounters ([`RunBot::encounter_action`], since 03.11.2026 one option
//! must be taken): the option with the best value ([`RunBot::option_value`],
//! simple rules in rough Groschen, not tuned), among those that can be
//! taken. A swap only for a relic worth at least [`SWAP_MARGIN`] more than
//! the weakest owned one; double stakes are never sought. At a Kiosk the
//! most valuable Schnaps, if any.
//!
//! Kneipentour (ADR 0016, simple rules, not tuned, not measured): at a
//! Rast the cursed card cleansed, else one Einmischen more in the next pub
//! (ADR 0019), at the Werkstatt the cursed card cleansed (else the free
//! Kartenmaler on an ace, his best modification taken), from a Spelunke's loot the relic into a free
//! slot (else the Groschen). The route ([`RunBot::route_action`]) scores
//! each way by the category it shows: the Tresen when rich, an Upgrade
//! with a cursed card, a Rast a little; a Spelunke only when strong
//! (since ADR 0019 a lost Spelunke ends the run: since 05.11.2026 five
//! relics, or four and the "Schutzengel"; before four, or three). After a won boss pub (ADR 0017)
//! the most valuable relic of the pick into a free slot (slots are
//! unlimited since 03.11.2026), with a cap and every slot full in place of
//! the weakest owned one if it is worth at least [`SWAP_MARGIN`] more, else
//! none ([`RunBot::boss_reward_action`]).
//!
//! The Schnaps of a won pub with every slot full (DESIGN 8.9): taken for
//! the weakest owned Schnaps if it is worth more ([`consumable_value`]),
//! else sold ([`RunBot::schnapps_reward_action`]).
//!
//! Schnäpse (simple rules, not tuned): "Kurzer" right before picking up the
//! skat (the bot does not use what it sees); after the skat, "Blue Curaçao"
//! paints the cheapest side card (never a jack) into the trump suit of a
//! suit game, then "Doppelkorn" before a game that is not weak
//! ([`RunBot::is_weak`]) and "Magenbitter" before a weak one, never before
//! a Null. "Kräuter" counts as "Ein Bier". "Fusel" before Skat oder Hand
//! with a hand that is not weak, when no other Schnaps waits for the game.
//!
//! Figures "Die Oma" and "Der Opa": in a hand game the bot takes the card
//! the run offers from the skat whenever it may (never in a Null), giving
//! away its cheapest side card. It never discards a card that must not be discarded
//! ("Ramponiert") while it has others.
//!
//! "Drei drücken": with thirteen cards the bot chooses its game as with
//! twelve and discards three: the two it would discard, then the one it
//! would discard next (the run gives the lowest of them back).
//!
//! The bot never looks at the Zielwert or the series account. The
//! simulator's success curve (`--curve`) relies on that.
//!
//! It lives in the core (not in `sim/`) so that core tests can drive whole
//! runs with it; a frontend could use it for an "auto play" button.

mod estimate;
mod table;
mod tresen;
mod values;

pub use estimate::{
    BOLD_MIN_PERCENT, BOLD_SAMPLES, GRAND_WIN_PERCENT, NULL_HAND_MAX_EXPOSED, NULL_MAX_EXPOSED,
    NULL_WIN_PERCENT, SUIT_WIN_PERCENT, null_expected_score, schneider_percent,
    suit_expected_score, ten_card_score,
};
pub use table::REDEAL_STOP;
pub use values::{
    CONSUMABLE_VALUES, DEFAULT_RELIC_VALUE, LEGENDARY_VALUES, NIGHT_VALUES, PAINTER_RESERVE,
    RELIC_DIRECTION_BONUS, RELIC_VALUES, REROLL_RESERVE, RICH, SWAP_MARGIN, TEETOTALLER_SKIP_WORTH,
    TRADE_PENALTY, ZINKE_DIRECTION_BONUS_PERCENT, ZINKEN_VALUES, consumable_value, relic_score,
    relic_value, zinke_score, zinken_value,
};

use table::count_rank;

use super::{EncounterOption, Run, RunAction, RunPhase};
use crate::content::Content;

/// Whether the player holds the relic `id`.
fn holds_relic(run: &Run, id: &str) -> bool {
    run.relics().iter().any(|r| r.as_str() == id)
}

/// The simple run policy.
#[derive(Debug, Clone, Copy, Default)]
pub struct RunBot;

impl RunBot {
    /// The long night (ADR 0020), for measurement: after der Alte the bot
    /// always continues ([`RunAction::EnterNight`]). [`RunBot::next_action`]
    /// stops at the won run as before; only a caller that wants the night
    /// (the simulator's `--night`) asks here.
    #[must_use]
    pub fn night_action(&self, run: &Run) -> Option<RunAction> {
        (run.phase() == RunPhase::Won && !run.in_night()).then_some(RunAction::EnterNight)
    }

    /// The next action, or `None` once the run is over.
    #[must_use]
    pub fn next_action(&self, run: &Run, content: &Content) -> Option<RunAction> {
        if let Some(action) = Self::schnapps_reward_action(run) {
            return Some(action);
        }
        match run.phase() {
            RunPhase::Won | RunPhase::Lost => None,
            RunPhase::Shop => Some(self.shop_action(run, content)),
            RunPhase::Encounter => Some(self.encounter_action(run, content)),
            RunPhase::Route => Some(self.route_action(run, content)),
            RunPhase::BossReward => Some(Self::boss_reward_action(run, content)),
            RunPhase::CardReward => Some(Self::card_reward_action(run, content)),
            RunPhase::Game => self.game_action(run, content),
        }
    }
}

#[cfg(test)]
mod tests;
