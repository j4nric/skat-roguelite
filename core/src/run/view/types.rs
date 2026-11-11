//! The data types of the view: what a frontend receives, serialized as
//! JSON by `skat_wasm`. Field names and order are the JSON the web app
//! reads; they only change together with the web app.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::cards::Suit;
use crate::content::{
    BossId, CardFilter, CardModId, CardModStatus, Character, ConsumableCategory, ConsumableId,
    ConsumableTarget, EncounterEffect, EncounterId, Factor, ModKind, NextGameEffect, Rarity,
    RelicId, RelicKind, RunLength, ScoreBreakdown, ScoreStep, StopCategory,
};
use crate::deck::{DealOdds, DealtCard, DeckCard};
use crate::game::{Position, Seat};
use crate::rules::{Declaration, GameType, NullRisk, RiskLevel};

use super::{GuestInfo, Kontra, OptionBlock, RunPhase, RunStats};

/// Everything a frontend shows of a run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RunView {
    /// The seed as decimal text (JavaScript numbers cannot hold every u64).
    pub seed: String,
    /// The Einführung (DESIGN 16.1): absent for every other run.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub tutorial: bool,
    /// Who plays the run.
    pub character: CharacterInfo,
    /// What the run waits for.
    pub phase: RunPhase,
    /// The defender who thinks next: while set, the run waits for the AI,
    /// not for the player, and every action fails with `AiToMove`; the
    /// frontend calls [`Run::advance_ai`](super::Run::advance_ai) (one card) until it is `None`.
    pub ai_to_move: Option<Seat>,
    /// The current series.
    pub series: SeriesView,
    /// The boss of the current section while its boss series is still
    /// ahead or running: known from the start of the section (DESIGN 11).
    /// `None` after the boss series and once the run is over.
    pub upcoming_boss: Option<BossInfo>,
    /// The pub the way leads to (on the way and at the route choice the
    /// regular next pub), for the Tresen and the run overview. `None` once
    /// the run is over.
    pub next_series: Option<NextSeriesView>,
    /// Where any one card lands when a game is dealt (DESIGN 7.4): always
    /// 12 of 32 for the player's hand or the skat since ADR 0015.
    pub deal_odds: DealOdds,
    /// Districts (Stadtteile) in this run.
    pub sections: u32,
    /// Always the ganzer Abend since ADR 0016.
    pub length: RunLength,
    /// The Kneipentour: the city and the route choice (ADR 0016).
    pub tour: super::super::tour::TourView,
    /// Stake of the Einsatz-Leiter (DESIGN 9.3), 0 for the normal game.
    pub stake: u32,
    /// Augen a suit game or Grand needs to win in this run before relics
    /// lower it: `61 + stake`. The game in progress has its own
    /// [`GameView::win_threshold`] with relics applied.
    pub win_threshold: u32,
    /// Groschen.
    pub coins: u32,
    /// Owned relics in purchase order.
    pub relics: Vec<OwnedRelic>,
    /// Slots for relics; `None`: any number (DESIGN 8.0).
    pub relic_slots: Option<u32>,
    /// Owned consumables.
    pub consumables: Vec<OwnedConsumable>,
    /// Slots for consumables.
    pub consumable_slots: u32,
    /// Run flags set by content (e.g. `bock`).
    pub flags: Vec<String>,
    /// Run counters raised by content (e.g. `tour`).
    pub counters: BTreeMap<String, u32>,
    /// The game in progress.
    pub game: Option<GameView>,
    /// The Tresen, while open.
    pub shop: Option<ShopView>,
    /// The encounter (Ereignis), while open.
    pub encounter: Option<EncounterView>,
    /// The relic pick of a won boss pub, while open (ADR 0017).
    pub boss_reward: Option<BossRewardView>,
    /// The card reward of a won normal pub or Spelunke, while open
    /// ([`RunPhase::CardReward`], DESIGN 5.0).
    pub card_reward: Option<CardRewardView>,
    /// The Kartenmaler's choice, waiting for the player (Tresen or
    /// Werkstatt): nothing but [`RunAction::ChoosePaint`](super::RunAction::ChoosePaint) goes now.
    pub painter: Option<PainterView>,
    /// The Schnaps of a won pub while every slot is full (DESIGN 8.9):
    /// nothing but [`RunAction::ChooseSchnappsReward`](super::RunAction::ChooseSchnappsReward) goes now. It comes
    /// on top of whatever phase follows the pub (route, loot, boss
    /// reward).
    pub schnapps_reward: Option<SchnappsRewardView>,
    /// House rules of the evening (encounter "Stammtisch-Streit"), active
    /// until the section ends.
    pub evening_relics: Vec<RelicInfo>,
    /// Factor waiting for the next declared game ("Letzte Runde").
    pub pending_factor: Option<u32>,
    /// Consumables used for the next declared game ("Doppelkorn",
    /// "Magenbitter"), in the order used.
    pub pending_next_game: Vec<PendingNextGame>,
    /// Changes waiting for the next pub ("Wegbier", "Der Skatlehrer",
    /// "Kiebitz am Fenster").
    pub next_pub: super::super::NextPubPlan,
    /// Groschen owed at the Tresen ("Deckel anschreiben"), paid from the
    /// next income.
    pub debt: u32,
    /// How far the Tresen would sell on credit now ("Deckel anschreiben"):
    /// the limit minus the debt, 0 without credit or once used in this
    /// district.
    pub credit_left: u32,
    /// The bet waiting for the next declared game ("Die Wette").
    pub wager: Option<super::super::Wager>,
    /// Route choices still to come whose exact stops are shown ("Kiebitz am
    /// Fenster").
    pub scout_legs: u32,
    /// The last scored game.
    pub last_score: Option<ScoreView>,
    /// Whether the player may give up the pub now ("Kneipe verlassen",
    /// [`RunAction::GiveUpPub`](super::RunAction::GiveUpPub)): between two games, from the second game
    /// of a pub on. In a boss pub it ends the run (ADR 0017).
    pub can_give_up: bool,
    /// "Noch nötig" before the last game of a pub; `None` otherwise.
    pub needed: Option<NeededView>,
    /// The player's edition of the 32 cards (ADR 0015), in id order.
    pub edition: Vec<DeckCard>,
    /// Statistics of the run.
    pub stats: RunStats,
    /// The long night (ADR 0020), once the player continued after der
    /// Alte.
    pub night: Option<super::super::night::NightView>,
    /// Die Wochen-Herausforderung (DESIGN 5.6): the Sonderregel in force;
    /// absent for every other run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weekly: Option<super::super::weekly::WeeklyView>,
    /// Whether the player may continue into the long night now
    /// ([`RunAction::EnterNight`](super::RunAction::EnterNight)): the run is won and the night has not
    /// begun.
    pub can_enter_night: bool,
}

/// "Noch nötig": what the last game of a pub has to bring (DESIGN 5.1).
/// The frontend makes the German line of it ("Du brauchst noch 87
/// Punkte", "Nicht mehr zu schaffen").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct NeededView {
    /// Zielwert minus account: what a won game must be worth. 0 or less
    /// once the target is reached (a boss pub plays all its games).
    pub points: i64,
    /// The smallest Spielwert of the plain formula (no relics, no factors)
    /// that is at least [`NeededView::points`]; `None` when nothing is
    /// needed or no plain game is worth that much.
    pub smallest_game: Option<u64>,
    /// Upper bound of a won game's value now, without knowing the hand
    /// ([`Run::max_game_value`](super::Run::max_game_value)).
    pub max_value: u64,
    /// Whether a won game can still reach the target:
    /// `points <= max_value`.
    pub reachable: bool,
}

/// The current series.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SeriesView {
    /// Section, from 0.
    pub section: u32,
    /// Series within the section, from 0.
    pub index: u32,
    /// Whether this is the boss series.
    pub is_boss: bool,
    /// Zielwert.
    pub target: u64,
    /// Series account.
    pub score: i64,
    /// Games played.
    pub games_played: u32,
    /// Spiellimit.
    pub game_limit: u32,
    /// Games left before the limit.
    pub games_left: u32,
    /// Einmischen left now: the game's own plus the series' extras ("Ein
    /// Bier"), [`Run::redeals_left`](super::Run::redeals_left). Back to the full allowance with every
    /// new game.
    pub redeals_left: u32,
    /// Einmischen used in this series.
    pub redeals_used: u32,
    /// The boss of a boss series.
    pub boss: Option<BossInfo>,
    /// The Stammgäste at the table, seat 1 (left) and seat 2 (right). In a
    /// boss pub the boss's two figures take both seats in the frontend
    /// ([`BossInfo::figures`], ADR 0017).
    pub guests: Vec<GuestInfo>,
}

/// The character of the run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CharacterInfo {
    /// Which character.
    pub character: Character,
    /// German name from the content.
    pub name: String,
    /// German text from the content.
    pub text: String,
    /// Card back of the player's edition (DESIGN 9.3): the character's,
    /// the Distel by default.
    pub back: String,
}

/// What an option of the open encounter needs as target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum OptionTarget {
    /// Nothing.
    None,
    /// A card of the player's edition (its id).
    Card,
    /// An owned relic (its index).
    Relic,
}

/// One effect of an encounter option as the player sees it, with what
/// was drawn for it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EncounterEffectView {
    /// The effect (data form, every draw filled in).
    pub effect: EncounterEffect,
    /// The relic it names: the house rule, the relic received, the relic
    /// a swap brings.
    pub relic: Option<RelicInfo>,
    /// The modification a card gets (with its curse mark).
    pub card_mod: Option<CardModInfo>,
    /// The Schnaps it gives.
    pub consumable: Option<ConsumableInfo>,
    /// A Schnaps with every slot full: these Groschen instead.
    pub coins_instead: Option<u32>,
}

/// An option of the open encounter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EncounterOptionView {
    /// Index for [`RunAction::ChooseEncounter`](super::RunAction::ChooseEncounter).
    pub index: usize,
    /// German title from the content.
    pub label: String,
    /// Groschen it costs (the stake of a bet or a throw of the dice).
    pub cost: u32,
    /// What it does, in order.
    pub effects: Vec<EncounterEffectView>,
    /// Offered only when no other option can be taken.
    pub fallback: bool,
    /// Whether it can be chosen now (with one of `targets` if it needs
    /// one).
    pub can_choose: bool,
    /// Why it cannot be chosen now.
    pub blocked: Option<OptionBlock>,
    /// What it needs as target.
    pub target: OptionTarget,
    /// Deck card ids or relic indices it can be used on.
    pub targets: Vec<u32>,
    /// What the option costs on each of `targets`, in the same order: a
    /// Zinke on a card may cost less than `cost` ("Bruderherz": half on
    /// Rot, [`Run::zinken_price`](super::Run::zinken_price)). Empty without targets.
    pub target_costs: Vec<u32>,
    /// The Marken of a Marke option ([`EncounterEffect::Marke`],
    /// 04.11.2026): with `marke: Chosen` the choice for
    /// [`RunAction::ChooseEncounter`](super::RunAction::ChooseEncounter) `marke`, with `Random` what may come.
    /// Empty for every other option.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub marken: Vec<MarkeChoiceView>,
}

/// One Marke of a Marke option of an encounter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MarkeChoiceView {
    /// The Marke with its value and text.
    pub card_mod: CardModInfo,
    /// Whether it can be taken now (some card fits; for a Toten- or
    /// Narrenmarke the edition may still get its special card).
    pub can_choose: bool,
    /// For a chosen Marke on a chosen card: the deck card ids it may go on
    /// now; empty otherwise.
    pub targets: Vec<u32>,
    /// What the option costs on each of `targets`, in the same order.
    pub target_costs: Vec<u32>,
    /// Chance in percent to be drawn for a random Marke on any card
    /// (rarity weights among those that fit some card); 0 for a chosen
    /// Marke or one that cannot come.
    pub percent: u32,
}

/// The open encounter (Ereignis).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EncounterView {
    /// Id, also the key for the frontend's texts.
    pub id: EncounterId,
    /// German name from the content.
    pub name: String,
    /// German text from the content.
    pub text: String,
    /// Its stop category (Begegnung, Werkstatt, Fundgroschen, ...).
    pub category: StopCategory,
    /// Its options; one must be taken.
    pub options: Vec<EncounterOptionView>,
}

/// The count of Augen as "Der Zähler" shows it. Only what the player knows:
/// the tricks of both sides, the skat once he knows it, and the Augen of
/// every card he has not seen yet (the talon included, since nobody knows
/// which cards lie there).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct CountView {
    /// Augen the player took in tricks, plus the skat once he knows it.
    /// Augen below zero ("Ramponiert") count against the seat that took
    /// them; no count is below 0.
    pub declarer: u32,
    /// Augen the defenders took in tricks.
    pub defenders: u32,
    /// Augen of all cards of the deck the player has not seen.
    pub unseen: u32,
    /// Augen per seat (index = seat) from the tricks, the player's with
    /// the skat once he knows it: who has the most in a Ramsch.
    pub seats: [u32; 3],
}

/// A boss with its rule text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BossInfo {
    /// Id, also the key for the frontend's texts.
    pub id: BossId,
    /// German name from the content.
    pub name: String,
    /// German rule text from the content.
    pub text: String,
    /// Factor on the Zielwert of the boss series (1.0 = unchanged).
    pub target_factor: Factor,
    /// The two boss players (ADR 0017), left and right of the player.
    pub figures: Vec<BossFigureInfo>,
    /// The card back of the boss's cards (a figure may have its own).
    pub back: String,
}

/// One of the two boss players.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BossFigureInfo {
    /// German name from the content.
    pub name: String,
    /// The figure's edition (ADR 0015, DESIGN 11): the modifications its
    /// cards carry. Empty for the figures of "Der Alte" (they play the
    /// editions of the rule in force, see [`GameView::boss_rule`]).
    pub edition: Vec<EditionEntry>,
    /// The card back of the figure's cards.
    pub back: String,
}

/// The Schnaps of a won pub with every slot full (DESIGN 8.9): take it
/// for an owned one ([`RunAction::ChooseSchnappsReward`](super::RunAction::ChooseSchnappsReward) with `give_up`,
/// any slot of [`RunView::consumables`]) or sell it for `sell_price`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SchnappsRewardView {
    /// The Schnaps.
    pub consumable: ConsumableInfo,
    /// Groschen when sold instead (half its price).
    pub sell_price: u32,
}

/// The relic pick of a won boss pub (ADR 0017).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BossRewardView {
    /// The relics on offer, with the index for
    /// [`RunAction::ChooseBossRelic`](super::RunAction::ChooseBossRelic).
    pub offers: Vec<BossRelicOffer>,
    /// Whether every relic slot is taken: then a relic can only be taken
    /// for an owned one (sold for half its price, `replace`). Never with
    /// unlimited slots (the default since 03.11.2026).
    pub slots_full: bool,
}

/// The card reward of a won pub (DESIGN 5.0): take one card with its
/// Zinke for free ([`RunAction::TakeCardReward`](super::RunAction::TakeCardReward)) or skip for Groschen
/// ([`RunAction::SkipCardReward`](super::RunAction::SkipCardReward)).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CardRewardView {
    /// The offers: a card with a Zinke each.
    pub offers: Vec<CardRewardOfferView>,
    /// Groschen for "Überspringen".
    pub skip_coins: u32,
}

/// One offer of the card reward: a card of the edition with a Zinke
/// (since 05.11.2026), shown like an offer of the Auslage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CardRewardOfferView {
    /// Index for [`RunAction::TakeCardReward`](super::RunAction::TakeCardReward) `offer`.
    pub index: usize,
    /// The card as it is now.
    pub card: DeckCard,
    /// The card as it would be after taking it.
    pub result: DeckCard,
    /// The Zinke with its value and text.
    pub card_mod: CardModInfo,
    /// Whether it can be taken now.
    pub can_take: bool,
    /// The modification on the card it would replace (ask first).
    pub replaces: Option<CardModId>,
    /// Whether it would raise the same modification on the card.
    pub raises: bool,
}

/// One relic of the boss reward.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BossRelicOffer {
    /// Index into the offers.
    pub index: usize,
    /// The relic.
    pub relic: RelicInfo,
}

/// One line of a boss edition: every card that matches the filter carries
/// the modification with its value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EditionEntry {
    /// The cards: a rank, a suit or one card.
    pub filter: CardFilter,
    /// The modification with name, text and value.
    pub card_mod: CardModInfo,
}

/// Preview of the next series.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NextSeriesView {
    /// Section, from 0.
    pub section: u32,
    /// Series within the section, from 0.
    pub index: u32,
    /// Whether it is the boss series.
    pub is_boss: bool,
    /// Zielwert, with the boss's factor applied.
    pub target: u64,
    /// Spiellimit.
    pub game_limit: u32,
    /// Einmischen per game of the character in its district, without
    /// relics.
    pub redeals: u32,
    /// Its boss, if it is the boss series.
    pub boss: Option<BossInfo>,
}

/// A relic as the player sees it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RelicInfo {
    /// Id.
    pub id: RelicId,
    /// House rule or figure.
    pub kind: RelicKind,
    /// German name from the content.
    pub name: String,
    /// German text from the content.
    pub text: String,
    /// Rarity.
    pub rarity: Rarity,
    /// Price at the Tresen.
    pub price: u32,
    /// Display group (shape of the beer mat).
    pub group: crate::content::RelicGroup,
}

/// A card modification as the player sees it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CardModInfo {
    /// Id.
    pub id: CardModId,
    /// German name from the content.
    pub name: String,
    /// German text from the content.
    pub text: String,
    /// Rarity.
    pub rarity: Rarity,
    /// Whether it is a curse (Fluch).
    pub curse: bool,
    /// Rand, Moment, Gutschein, Fluch or Spezialkarte (effects v2).
    pub kind: ModKind,
    /// The value it comes with, if it has one (the text has it filled in).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<i32>,
}

/// A consumable as the player sees it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConsumableInfo {
    /// Id.
    pub id: ConsumableId,
    /// German name from the content.
    pub name: String,
    /// German text from the content.
    pub text: String,
    /// Rarity.
    pub rarity: Rarity,
    /// Price at the Tresen.
    pub price: u32,
    /// Whether using it needs a target card of the deck (at the Tresen).
    pub needs_target: bool,
    /// Whether using it needs a card of the hand and a suit ("Blue Curaçao"):
    /// one of [`OwnedConsumable::targets`] and any other suit than the
    /// card's own.
    pub needs_suit: bool,
    /// Group for labels ("Schnäpse").
    pub category: ConsumableCategory,
    /// What the frontend pours it on (drag and drop, DESIGN 8.9): `Skat`,
    /// `Table`, `Card`, `Self`, `Map`. Whether it goes now is
    /// [`OwnedConsumable::usable_now`].
    pub target: ConsumableTarget,
}

/// An owned relic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OwnedRelic {
    /// Index for [`RunAction::SellRelic`](super::RunAction::SellRelic).
    pub index: usize,
    /// The relic.
    pub relic: RelicInfo,
    /// Groschen when sold.
    pub sell_price: u32,
    /// Whether it can be sold now.
    pub can_sell: bool,
}

/// A consumable waiting for the next declared game.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PendingNextGame {
    /// The consumable.
    pub consumable: ConsumableId,
    /// What it will do.
    pub effect: NextGameEffect,
}

/// An owned consumable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OwnedConsumable {
    /// Slot for [`RunAction::UseConsumable`](super::RunAction::UseConsumable) and [`RunAction::SellConsumable`](super::RunAction::SellConsumable).
    pub slot: usize,
    /// The consumable.
    pub consumable: ConsumableInfo,
    /// Groschen when sold.
    pub sell_price: u32,
    /// Whether it can be sold now.
    pub can_sell: bool,
    /// Whether the core would accept drinking it now (with one of
    /// `targets` if it needs one): phase, conditions ("Klarer" only in
    /// the minus), a Spelunke's Schnapsverbot, a choice that waits.
    /// Tried with the real action on a copy of the run ([`Run::accepts`](super::Run::accepts)),
    /// so it is exactly what [`RunAction::UseConsumable`](super::RunAction::UseConsumable) would do. Until
    /// 03.11.2026 `can_use`.
    pub usable_now: bool,
    /// Card ids it can be used on now: deck cards, or for "Blue Curaçao" the
    /// cards of the hand.
    pub targets: Vec<u32>,
}

/// The Tresen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ShopView {
    /// Relic slots; `None` once bought.
    pub relics: Vec<Option<RelicOffer>>,
    /// Consumable slots; `None` once bought.
    pub consumables: Vec<Option<ConsumableOffer>>,
    /// Price of "Neu auslosen" now (relics, Schnäpse and the Auslage, once
    /// per visit since 05.11.2026; 0 with "Der Stammkunde").
    pub reroll_cost: u32,
    /// Whether "Neu auslosen" is possible now.
    pub can_reroll: bool,
    /// "Neu auslosen" during this visit (0 or 1).
    pub rerolls: u32,
    /// "Neu auslosen" was used at this visit.
    pub reroll_used: bool,
    /// The Auslage (Zinken, DESIGN 8.6); `None` once bought.
    pub card_offers: Vec<Option<CardOfferView>>,
    /// The Kartenmaler ([`RunAction::HirePainter`](super::RunAction::HirePainter)).
    pub painter: ServiceView,
    /// "Karte reinigen" ([`RunAction::CleanseCard`](super::RunAction::CleanseCard)).
    pub cleanse: ServiceView,
    /// A Kiosk (03.11.2026), not the Tresen: only Schnäpse, one may be
    /// bought; no Neuauslage, Zinken or selling.
    pub kiosk: bool,
    /// At a Kiosk: Groschen off each Schnaps against the Tresen.
    pub kiosk_discount: u32,
}

/// An offer of the Auslage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CardOfferView {
    /// Slot for [`RunAction::BuyCardOffer`](super::RunAction::BuyCardOffer).
    pub slot: usize,
    /// The card as it is now.
    pub card: DeckCard,
    /// The card as it would be after buying.
    pub result: DeckCard,
    /// The modification with its value.
    pub card_mod: CardModInfo,
    /// What it costs now (after discounts).
    pub price: u32,
    /// Whether it can be bought now.
    pub can_buy: bool,
    /// The modification on the card it would replace (ask first).
    pub replaces: Option<CardModId>,
    /// Whether it would raise the same modification on the card.
    pub raises: bool,
    /// A "Schnäppchen" (curses as trade, DESIGN 8.6): half price, and
    /// buying it curses another card of the edition.
    pub bargain: bool,
    /// For a "Schnäppchen": what it would cost without the trade (after
    /// discounts).
    pub was: Option<u32>,
}

/// A service of the Tresen that works on a card of the player's choice
/// (Kartenmaler, "Karte reinigen"), once per visit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ServiceView {
    /// What it costs now (after discounts).
    pub price: u32,
    /// Used at this visit already.
    pub used: bool,
    /// Whether it can be used now on one of `targets`.
    pub can_use: bool,
    /// Cards of the edition it can work on now.
    pub targets: Vec<u32>,
    /// What it costs on each of `targets`, in the same order: the
    /// character's price for Zinken applied ([`Run::painter_cost`](super::Run::painter_cost),
    /// "Bruderherz": half on Rot); `price` for "Karte reinigen".
    pub target_prices: Vec<u32>,
}

/// The Kartenmaler's choice, waiting for the player.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PainterView {
    /// The card as it is now.
    pub card: DeckCard,
    /// The modifications to take one of.
    pub options: Vec<PainterOptionView>,
    /// Hired at the Werkstatt (an encounter) rather than the Tresen.
    pub encounter: bool,
}

/// One modification of the Kartenmaler's choice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PainterOptionView {
    /// Index for [`RunAction::ChoosePaint`](super::RunAction::ChoosePaint).
    pub index: usize,
    /// The modification with its value.
    pub card_mod: CardModInfo,
    /// The card as it would be after taking it.
    pub result: DeckCard,
    /// The modification on the card it would replace (ask first).
    pub replaces: Option<CardModId>,
    /// Whether it would raise the same modification on the card.
    pub raises: bool,
}

/// A relic on offer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RelicOffer {
    /// Slot for [`RunAction::BuyRelic`](super::RunAction::BuyRelic).
    pub slot: usize,
    /// The relic.
    pub relic: RelicInfo,
    /// What it costs now (the content price minus discounts, half for
    /// "Hehlerware").
    pub price: u32,
    /// Whether it can be bought now (Groschen and free slot).
    pub can_buy: bool,
    /// With every relic slot full (DESIGN 8.0): the owned relics (indices)
    /// it can replace now ([`RunAction::ReplaceRelic`](super::RunAction::ReplaceRelic), the old one sold
    /// for half its price). Empty while a slot is free.
    pub replace: Vec<usize>,
    /// "Hehlerware" (curses as trade, DESIGN 8.6): half price, and buying
    /// it curses a random card of the edition.
    pub fence: bool,
    /// For "Hehlerware": what it would cost without the trade (after
    /// discounts).
    pub was: Option<u32>,
}

/// A consumable on offer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConsumableOffer {
    /// Slot for [`RunAction::BuyConsumable`](super::RunAction::BuyConsumable).
    pub slot: usize,
    /// The consumable.
    pub consumable: ConsumableInfo,
    /// What it costs now (the content price minus discounts).
    pub price: u32,
    /// Whether it can be bought now.
    pub can_buy: bool,
}

/// Phase of the game in progress, in the order of the Skatordnung (ADR
/// 0011).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TablePhase {
    /// Ten cards: declare a hand game, pick up the skat ([`DeclareOptions`]
    /// says what is allowed) or redeal (einmischen).
    Declaring,
    /// Twelve cards: two to discard ("Drei drücken": thirteen and three,
    /// [`GameView::discard_count`]).
    Discarding,
    /// Skat picked up and discarded: the Ansage without the Hand level.
    DeclaringAfterSkat,
    /// Tricks.
    Playing,
    /// Scored.
    Finished,
}

/// A card as shown to the player.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CardView {
    /// The dealt card with its effective Augen, in the edition of its
    /// holder (`owner`, ADR 0015).
    #[serde(flatten)]
    pub card: DealtCard,
    /// Card modifications on this instance in its owner's edition.
    pub mods: Vec<CardModId>,
    /// The card back of the owner: `None` for the player's own cards
    /// (the back he chose), the opponent's back otherwise.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub back: Option<String>,
    /// Their values, for those that have one ([`DeckCard::mod_values`]).
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub mod_values: BTreeMap<CardModId, i32>,
    /// Whether the player may play it now.
    pub legal: bool,
    /// Whether it came from the skat (while discarding).
    pub from_skat: bool,
    /// The defender it was drawn from with the skat ("Drei drücken"),
    /// while discarding.
    pub drawn_from: Option<Seat>,
    /// The printed suit of a card "Blue Curaçao" repainted for this game; the
    /// card's `kind` is the suit it has now.
    pub chalked: Option<Suit>,
    /// Whether the player may put it into the skat now: only for hand
    /// cards while discarding. A card that must not be discarded
    /// ("Ramponiert") only when fewer than two other cards are left
    /// ([`crate::rules::discardable`]).
    pub discardable: bool,
    /// Once the game is declared: whether each conditional modification
    /// on this card works in this game ([`card_mod_status`](crate::content::card_mod_status)). Empty before
    /// the declaration; modifications without a condition ("Fett") are
    /// left out.
    pub mod_status: BTreeMap<CardModId, CardModStatus>,
    /// A Wandelkarte of the player whose suit is still open: before the
    /// game starts (the card is its printed card until then, DESIGN 7.2).
    /// Left out of the JSON when `false`.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub unsettled: bool,
    /// The printed suit of a Wandelkarte settled as this game started; the
    /// card's `kind` has its settled suit ([`ActiveGame::named`]).
    pub named: Option<Suit>,
    /// An "Untot" card of the hand that would rise if played now
    /// ([`Run::can_rise`](super::Run::can_rise)): it comes back once per game in a suit game or
    /// Grand, not as the last card. Left out of the JSON when `false`.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub can_rise: bool,
    /// A "Doppelgänger" of the hand whose mirror is still to come in this
    /// game ([`Run::can_mirror`](super::Run::can_mirror)): played as the last card of a trick it
    /// becomes a copy of the highest card and wins the trick. Left out of
    /// the JSON when `false`.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub can_mirror: bool,
    /// A card of the player that rose in this game ("Untot",
    /// [`ActiveGame::risen`]): it came back into the hand as the card that
    /// won the trick. The web app draws only this one with the zombie
    /// picture (playtest 05.11.2026). Left out of the JSON when `false`.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub risen: bool,
}

/// A card on the table and who played it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PlayedCard {
    /// Who played it.
    pub seat: Seat,
    /// The card.
    pub card: CardView,
}

/// A completed trick.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TrickView {
    /// Cards in play order.
    pub cards: Vec<PlayedCard>,
    /// Who took it.
    pub winner: Seat,
}

/// Why the player knows where a card is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum RevealSource {
    /// A card modification that shows its holder ("Durchsichtig",
    /// "Bierfleck", "Attrappe"), by id.
    CardMod(CardModId),
    /// "Der Kiebitz" showed it before the declaration.
    Kibitz,
    /// A character rule (`RevealDefenderHand`) shows the whole hand of this defender.
    Onlooker,
    /// The player gave it back to this defender ("Drei drücken").
    Returned,
}

/// A Wandelkarte settled in the game in progress.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct SettledWild {
    /// Instance id.
    pub card: u32,
    /// The card it is until the end of the game.
    pub kind: crate::cards::CardKind,
    /// Its printed suit.
    pub printed: Suit,
}

/// A card whose holder the player knows, and why.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RevealedCard {
    /// The card.
    pub card: CardView,
    /// Holder; `None` if in the skat or already played.
    pub holder: Option<Seat>,
    /// Why it is known: a modification or "Der Kiebitz". A card known
    /// both ways names its modification.
    pub source: RevealSource,
}

/// A Spielart in the Ansage dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct GameOption {
    /// The Spielart.
    pub game: GameType,
    /// Its Grundwert by the rules.
    pub base_value: u32,
    /// Whether it may be declared.
    pub allowed: bool,
}

/// A Risikostufe in the Ansage dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct RiskOption {
    /// The Risikostufe.
    pub risk: RiskLevel,
    /// Whether it may be declared.
    pub allowed: bool,
}

/// A Null-Risikostufe in the Ansage dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct NullRiskOption {
    /// The Null-Risikostufe.
    pub risk: NullRisk,
    /// Whether it may be declared.
    pub allowed: bool,
    /// Grundwert × Gewinnstufen of a won game without style bonus and
    /// relics: 23, 35, 46, 59 (Fixed) or 23 × Risikostufe (Formula).
    pub value: u64,
}

/// What the player may do before the declaration. Before the skat
/// ([`TablePhase::Declaring`]) the lists hold the hand levels only, the
/// other way is [`DeclareOptions::can_pick_up_skat`]; after picking up the
/// skat and discarding ([`TablePhase::DeclaringAfterSkat`]) they hold the
/// levels without Hand (suit and grand: `Normal`, Schneider and Schwarz
/// angesagt, Ouvert since ADR 0013; Null: `Normal` and `Ouvert`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeclareOptions {
    /// Spielarten for every character (ADR 0012): Kreuz, Pik, Herz, Karo,
    /// Grand, then Null. `allowed` with the lowest allowed Risikostufe of
    /// this phase (Null: any allowed Null-Risikostufe); the Grundwert of
    /// Null is that of the character's Null scoring.
    pub games: Vec<GameOption>,
    /// Risikostufen of suit games and grand in this phase, lowest first.
    pub risks: Vec<RiskOption>,
    /// Null-Risikostufen in this phase, lowest first.
    pub null_risks: Vec<NullRiskOption>,
    /// Whether a won Null earns a style bonus (the character's Null
    /// scoring is the formula of DESIGN 9.2, not the Skatordnung).
    pub null_style: bool,
    /// Whether the skat can be picked up ([`RunAction::PickUpSkat`](super::RunAction::PickUpSkat)): only
    /// before the skat, not against "Der Schieber" (every game is a hand
    /// game).
    pub can_pick_up_skat: bool,
    /// Whether "Der Angeber" demands Ouvert.
    pub ouvert_required: bool,
    /// Whether the game can be dealt again now (Einmischen,
    /// [`RunAction::Redeal`](super::RunAction::Redeal)).
    pub can_redeal: bool,
    /// Einmischen left now ([`Run::redeals_left`](super::Run::redeals_left)).
    pub redeals_left: u32,
    /// Whether the boss demands a hand game.
    pub hand_required: bool,
}

/// The game in progress.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GameView {
    /// Number of the game in the series, from 1.
    pub number: u32,
    /// Phase.
    pub phase: TablePhase,
    /// Seat that led the first trick.
    pub forehand: Seat,
    /// Table position per seat (index = seat).
    pub positions: Vec<Position>,
    /// The declaration, once made.
    pub declaration: Option<Declaration>,
    /// The player's hand in display order: trumps first, then the suits.
    pub hand: Vec<CardView>,
    /// Cards in each seat's hand (index = seat).
    pub hand_sizes: Vec<usize>,
    /// Seat that led (or leads) the current trick.
    pub trick_lead: Seat,
    /// Cards of the current trick in play order.
    pub trick: Vec<PlayedCard>,
    /// The last completed trick.
    pub last_trick: Option<TrickView>,
    /// Completed tricks.
    pub tricks_played: usize,
    /// Tricks taken by the player.
    pub declarer_tricks: usize,
    /// Tricks taken by the defenders.
    pub defender_tricks: usize,
    /// The skat as the player knows it: the discards once discarded (also
    /// while declaring after the skat), `None` in hand games and before.
    pub skat: Option<Vec<CardView>>,
    /// Whether the player has to play a card now.
    pub your_turn: bool,
    /// Kontra state.
    pub kontra: Kontra,
    /// Whether Kontra waits for the player's answer (Re or not).
    pub awaiting_kontra_answer: bool,
    /// Whether the defenders may say Kontra in this game.
    pub kontra_possible: bool,
    /// Augen a suit game or Grand needs to win: `61 + stake`, lowered by
    /// relics in force (once declared, also by declaration hooks such as
    /// Hufeisen).
    pub win_threshold: u32,
    /// Ansage options while declaring, before and after the skat.
    pub declare: Option<DeclareOptions>,
    /// "Durchsichtig" cards and the cards "Der Kiebitz" showed, with their
    /// holders.
    pub revealed: Vec<RevealedCard>,
    /// The Wandelkarten settled as this game started (DESIGN 7.2), also
    /// those already played: for the edition shown during the game.
    pub wild_settled: Vec<SettledWild>,
    /// The boss rule in force in this game: the series' boss, for the
    /// Endboss "Der Alte" the rule of this game (with its edition).
    pub boss_rule: Option<BossInfo>,
    /// Card back per seat (index = seat): `None` for the player (his own
    /// choice), the back of the boss or the Stammgast otherwise (ADR
    /// 0015).
    pub backs: [Option<String>; 3],
    /// The count of "Der Zähler", if a relic shows it.
    pub count: Option<CountView>,
    /// Whether "Die Oma" (or "Der Opa") lets the player take a card from
    /// the skat now (with [`RunAction::TakeFromSkat`](super::RunAction::TakeFromSkat)): only if one lies
    /// there.
    pub can_take_from_skat: bool,
    /// The skat cards a skat swap may take now, the one taken by default
    /// first (`take: None`): "Die Oma" names the jack that comes out.
    pub skat_swap_cards: Vec<CardView>,
    /// The untouched skat the player looked at ("Kurzer"); `None` if he
    /// did not, and once the skat is picked up.
    pub skat_peek: Option<Vec<CardView>>,
    /// Tricks per seat so far (index = seat), as they lie on the table.
    pub seat_tricks: [u32; 3],
    /// What the game is worth if the player wins it, as far as it is
    /// certain now: only while playing a suit game, grand or Null
    /// ([`ValuePreview`]).
    pub value_preview: Option<ValuePreview>,
    /// Whether the player may end the game now with
    /// [`RunAction::SettleGame`](super::RunAction::SettleGame) ("Rest abkürzen"): it is his turn and the
    /// remaining cards can no longer change the score, however anyone
    /// plays them ([`Run::can_settle`](super::Run::can_settle)). Says nothing about the
    /// defenders' cards beyond that.
    pub can_settle: bool,
    /// Cards to discard now: 2, or 3 after drawing a card with the skat
    /// ("Drei drücken", [`RunAction::DiscardThree`](super::RunAction::DiscardThree)); 0 outside
    /// [`TablePhase::Discarding`].
    pub discard_count: usize,
    /// Cards of the untouched skat the player was shown after declaring a
    /// hand game ("Spickzettel"), as long as they lie in the skat.
    pub skat_shown: Vec<CardView>,
    /// Whether "Drei drücken" draws a card at the next pickup of the skat
    /// (once per series; [`Run::discard_three_ready`](super::Run::discard_three_ready)).
    pub discard_three_ready: bool,
    /// An "Untot" card waiting for the player's three cards
    /// ([`RunAction::ChooseRiseCards`](super::RunAction::ChooseRiseCards)); every card of the hand may be
    /// picked. `None` otherwise.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rise_choice: Option<RiseChoiceView>,
}

/// "Untot" (DESIGN 7.2): the player picks [`super::super::UNDEAD_PICK`] cards of
/// his hand, one of them is drawn and goes into the trick in place of the
/// rising card.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RiseChoiceView {
    /// The card as it lies in the trick.
    pub card: CardView,
    /// The card it comes back as: a copy of the trick's winning card (the
    /// card itself if it won the trick).
    pub becomes: CardView,
    /// How many cards to pick: always [`super::super::UNDEAD_PICK`].
    pub pick: usize,
}

/// The score line of the table while playing: what the game in progress
/// is worth if the player wins it, as far as it is certain now. Computed
/// with the scoring of a finished game ([`crate::content::score_parts`]),
/// so `game_value = base_value × levels × multiplier` (rounded half up)
/// and the numbers are
/// the ones the score screen would start from.
///
/// Included, for suit games and grand:
/// - the Grundwert of the Spielart with every Grundwert line collected so
///   far (relics, boss, Bock, counters such as "Tournee", card
///   modifications whose card the player knows to be in play);
/// - the Gewinnstufen of the Skatordnung known now: Spitzen, Spiel, Hand,
///   declared Schneider and Schwarz (each with the level for reaching it,
///   since winning requires it) and Ouvert. Spitzen are counted along the
///   trumps in play (DESIGN 4.4) from what the player knows: his own
///   cards, the skat once picked up, played and revealed cards. Where that
///   leaves the count open (an untouched skat in a hand game, cards that
///   may lie in the talon), the smallest possible count is taken, so the
///   preview never tells more than the player knows;
/// - Gewinnstufen lines collected so far (e.g. "Auf dem Trockenen" after
///   the third trick) and Spielwert factors (Kontra, Re, "Letzte Runde");
/// - `OnScore` hooks whose condition is already certain for a won game
///   ([`crate::content::evaluate_if_won`]): e.g. "Ouvert-Bonus", "Mit
///   Vieren" once the Spitzen are certain, "Durchmarsch" for declared
///   Schneider. Flags and counters count as they are now.
///
/// Not included: everything that depends on how the game goes on:
/// Schneider or Schwarz played but not declared, Augen ranges such as
/// "Jungfrau", the last trick, hooks that are still to fire during play,
/// and a boss rule that only pays for Schneider.
///
/// Null: the Null scoring of the character with the risk known (Formula:
/// 23 × Risikostufe; Fixed: 23, 35, 46, 59) plus the certain relic lines,
/// without any style bonus (sevens ducked, never followed, Kurzschluss),
/// which only counts at the end.
///
/// Ramsch: no preview (`None`). Its value is the Augen of the loser,
/// unknown until the last trick, and the player does not win a Ramsch by
/// playing on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ValuePreview {
    /// Grundwert after the certain modifications.
    pub base_value: u64,
    /// Gewinnstufen after the certain modifications.
    pub levels: u64,
    /// Product of the certain Spielwert factors (1 without any; a decimal
    /// number in JSON, e.g. 1.5 for "Stammfarbe").
    pub multiplier: Factor,
    /// `base_value × levels × multiplier`, rounded half up.
    pub game_value: u64,
}

/// The last scored game.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScoreView {
    /// Every term and modification.
    pub breakdown: ScoreBreakdown,
    /// The breakdown step by step with running totals.
    pub steps: Vec<ScoreStep>,
}
