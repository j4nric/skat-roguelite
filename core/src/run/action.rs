//! What the player can do: [`RunAction`], the input of [`Run::apply`](super::Run::apply).

use serde::{Deserialize, Serialize};

use crate::cards::Suit;
use crate::content::CardModId;
use crate::rules::Declaration;

/// A decision of the player.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunAction {
    /// Einmischen: throw the dealt game in and have all hands and the skat
    /// dealt again before declaring, without using up a game. Each game
    /// allows [`Content::redeals`](crate::content::Content::redeals) of them (plus "Schieben"), then the
    /// series' extras ("Ein Bier"); see [`Run::redeals_left`](super::Run::redeals_left). Also read
    /// under the old names `RedrawSkat` and `Pass` (Einpassen, which cost a
    /// game until 01.11.2026).
    #[serde(alias = "RedrawSkat", alias = "Pass")]
    Redeal,
    /// Pick up the skat instead of declaring a hand game (Skatordnung, ADR
    /// 0011). Then two cards are discarded ([`RunAction::Discard`]; with
    /// "Drei drücken" a card of the left defender joins and three go,
    /// [`RunAction::DiscardThree`]) and the game is declared
    /// ([`RunAction::Declare`]). Einmischen is no longer possible after it.
    PickUpSkat,
    /// Announce the game. With ten cards, before the skat is touched, only a
    /// hand game (Hand, Schneider or Schwarz angesagt, Ouvert; Null Hand,
    /// Null Ouvert Hand). After picking up the skat and discarding, only a
    /// game without the Hand level: `Normal`, Schneider or Schwarz angesagt,
    /// Ouvert (ADR 0013); Null or Null Ouvert. The phase decides whether an
    /// announcement is played from the hand ([`Declaration::with_hand`]).
    Declare(Declaration),
    /// Answer a Kontra: `true` = Re.
    AnswerKontra {
        /// Whether to say Re.
        re: bool,
    },
    /// Discard two cards (instance ids) after picking up the skat; the
    /// declaration follows.
    Discard([u32; 2]),
    /// Discard three cards (instance ids) after picking up the skat with
    /// "Drei drücken" ([`view::GameView::discard_count`](super::view::GameView::discard_count) is 3): the lowest
    /// goes back to the defender the third card came from
    /// ([`crate::rules::returned_discard`]), the other two into the skat.
    DiscardThree([u32; 3]),
    /// Play a card (instance id).
    Play(u32),
    /// "Untot" (DESIGN 7.2): the three cards of the hand (instance ids) of
    /// which one is drawn to take the place of the rising card in the
    /// trick, while [`Run::rise_pending`](super::Run::rise_pending) waits for them.
    ChooseRiseCards([u32; 3]),
    /// Use a consumable from the given slot, with a target card and a suit
    /// if needed.
    UseConsumable {
        /// Slot of the consumable.
        slot: usize,
        /// Target card instance id: a deck card for deck changes, a card of
        /// the hand for "Blue Curaçao".
        target: Option<u32>,
        /// The new suit of the target ("Blue Curaçao").
        #[serde(default)]
        suit: Option<Suit>,
    },
    /// "Spelunkenkenner": at the route choice, pick the malus of the
    /// Spelunke on route `option` from its offers
    /// ([`tour::Spelunke::malus_options`](super::tour::Spelunke::malus_options)).
    ChooseMalus {
        /// Index of the route.
        option: usize,
        /// Index into the malus offers.
        malus: usize,
    },
    /// Buy the relic in this Tresen slot.
    BuyRelic(usize),
    /// Buy the relic in Tresen slot `slot` in place of the owned relic
    /// `replace` (index into the owned relics), which is sold for half its
    /// price first (every relic slot full, DESIGN 8.0). All or nothing.
    ReplaceRelic {
        /// Slot of the relic on offer.
        slot: usize,
        /// Index of the owned relic that makes room.
        replace: usize,
    },
    /// Buy the consumable in this Tresen slot.
    BuyConsumable(usize),
    /// Sell an owned relic (index into the owned relics) for half its price.
    SellRelic(usize),
    /// Sell an owned consumable for half its price.
    SellConsumable(usize),
    /// "Neu auslosen" (since 05.11.2026, DESIGN 8.6): draw a new offer of
    /// relics, consumables and the Auslage, once per Tresen visit
    /// ([`RunError::AlreadyUsed`](super::RunError::AlreadyUsed)), for
    /// [`crate::content::Economy::reroll_cost`] (free with "Der
    /// Stammkunde", [`Effect::FreeReroll`](crate::content::Effect::FreeReroll)).
    Reroll,
    /// Buy the offer of the Auslage in this slot (Zinken, DESIGN 8.6): its
    /// modification with its value goes on its card. The same modification
    /// on the card is raised, another one replaced.
    BuyCardOffer(usize),
    /// Hire the Kartenmaler (once per Tresen) for this card of the player's
    /// edition (instance id): he draws modifications that fit it, the
    /// player takes one ([`RunAction::ChoosePaint`]).
    HirePainter(u32),
    /// Take this option of the Kartenmaler's choice ([`Run::painter`](super::Run::painter)), at
    /// the Tresen or the Werkstatt.
    ChoosePaint(usize),
    /// "Karte reinigen" (once per Tresen): every modification falls off
    /// this card of the player's edition (instance id), curses included.
    CleanseCard(u32),
    /// Leave the Tresen and start the next series.
    LeaveShop,
    /// "Kneipe verlassen": give up the current pub between two games, from
    /// its second game on ([`view::RunView::can_give_up`](super::view::RunView::can_give_up)). It counts as
    /// lost: nothing is paid, `OnSeriesWon` does not fire, and the run
    /// ends (ADR 0019) unless the "Schutzengel" has the pub played again.
    GiveUpPub,
    /// "Die Oma" (and every other skat swap): in a hand game before the
    /// first card, take a matching card from the skat and put this card
    /// (instance id) in its place.
    TakeFromSkat {
        /// The card from the hand that goes into the skat.
        give: u32,
        /// The skat card to take (instance id, one of
        /// [`view::GameView::skat_swap_cards`](super::view::GameView::skat_swap_cards)); `None` takes the first of
        /// them.
        #[serde(default)]
        take: Option<u32>,
    },
    /// "Rest abkürzen": end the game in progress when the remaining cards
    /// can no longer change its score ([`view::GameView::can_settle`](super::view::GameView::can_settle),
    /// [`Run::can_settle`](super::Run::can_settle)). The remaining cards are played out by a fixed
    /// rule and the game is scored as if played. Fails with
    /// [`RunError::NotDecided`](super::RunError::NotDecided) otherwise.
    SettleGame,
    /// Choose an option of the open encounter, with a target card if the
    /// option needs one; then the way goes on (next stop, the route
    /// choice or the next pub).
    ChooseEncounter {
        /// Index into [`ActiveEncounter::options`](super::ActiveEncounter::options).
        option: usize,
        /// Deck card id for an option on a card of the player's choice
        /// (a Zinke, a Marke, the Kartenmaler, Karte reinigen), or the
        /// index of an owned relic for a relic option.
        target: Option<u32>,
        /// The Marke of the player's choice for
        /// [`EncounterEffect::Marke`](crate::content::EncounterEffect::Marke) with `marke: Chosen` (04.11.2026).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        marke: Option<CardModId>,
    },
    /// Take the route at this index of the open route choice
    /// ([`tour::RouteChoice::options`](super::tour::RouteChoice::options)).
    ChooseRoute(usize),
    /// The relic pick of a won boss pub (ADR 0017): take the relic at
    /// `offer` of [`tour::BossReward::relics`](super::tour::BossReward::relics), or none (`None`). With
    /// `replace` the owned relic at that index is sold for half its price
    /// first and the new one takes a slot (needed when every slot is
    /// full). Then the route choice.
    ChooseBossRelic {
        /// Index into the relics on offer; `None` takes none.
        offer: Option<usize>,
        /// Index of an owned relic to sell for it.
        #[serde(default)]
        replace: Option<usize>,
    },
    /// The Schnaps of a won pub while every Schnaps slot is full
    /// ([`view::RunView::schnapps_reward`](super::view::RunView::schnapps_reward), DESIGN 8.9): with `give_up`
    /// it takes the slot of that owned Schnaps, which is gone without
    /// Groschen; without, it is sold for half its price. Nothing else
    /// goes before this choice ([`RunError::SchnappsRewardPending`](super::RunError::SchnappsRewardPending)).
    ChooseSchnappsReward {
        /// Slot of the owned Schnaps to give up for it; `None` sells it.
        #[serde(default)]
        give_up: Option<usize>,
    },
    /// The card reward of a won pub ([`RunPhase::CardReward`](super::RunPhase::CardReward), DESIGN
    /// 5.0): the offer at `offer` of [`card_reward::CardReward::offers`](super::card_reward::CardReward::offers),
    /// a card with a Zinke, for free (the same Zinke on it rises, another
    /// is replaced). Since 05.11.2026 the card comes with the offer. Then
    /// the way goes on.
    TakeCardReward {
        /// Index into the offers.
        offer: usize,
    },
    /// The card reward of a won pub, skipped: Groschen instead
    /// ([`crate::content::CardRewardConfig::skip_coins`]). Then the way
    /// goes on.
    SkipCardReward,
    /// After der Alte ([`RunPhase::Won`](super::RunPhase::Won)): continue into the long night
    /// (ADR 0020). The win stays recorded; the route to Stadtteil 4 opens.
    EnterNight,
}
