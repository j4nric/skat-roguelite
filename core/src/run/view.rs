//! Read-only view of a run for frontends (the web app via `skat_wasm`).
//!
//! Everything the table, the Ansage dialog, the Tresen and the score
//! screen show comes from here, already decided by the core: the hand in
//! display order, which cards are legal, which declarations are allowed,
//! prices, what can be bought, sold or used. A frontend only renders it.
//!
//! The view never changes the run. Whether an action is allowed is found
//! out by trying it on a copy of the run, so the view can never disagree
//! with [`Run::apply`].

use std::collections::BTreeMap;

use serde::Serialize;

use super::guests::GuestInfo;
use super::{
    EncounterChoice, Kontra, PUBS_PER_DISTRICT, Run, RunAction, RunPhase, RunStats, sell_price,
    series_target,
};
use crate::cards::{Card, PlayingCard, Suit};
use crate::content::{
    BossId, CardFilter, CardModId, CardModStatus, Character, ConsumableAction, ConsumableCategory,
    ConsumableId, Content, EncounterId, Factor, HookContext, ModKind, NextGameEffect,
    OPPONENT_BACK, Rarity, RelicId, RelicKind, RunLength, ScoreBreakdown, ScoreStep,
    card_mod_status,
};
use crate::deck::{DealOdds, DealtCard, DeckCard, Owner};
use crate::game::{Phase, Position, Seat, seat_at};
use crate::rules::{
    Declaration, GameType, NullRisk, NullScoring, RiskLevel, discardable_of, suit_strength,
    trump_order_key,
};

/// Everything a frontend shows of a run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RunView {
    /// The seed as decimal text (JavaScript numbers cannot hold every u64).
    pub seed: String,
    /// Who plays the run.
    pub character: CharacterInfo,
    /// What the run waits for.
    pub phase: RunPhase,
    /// The defender who thinks next: while set, the run waits for the AI,
    /// not for the player, and every action fails with `AiToMove`; the
    /// frontend calls [`Run::advance_ai`] (one card) until it is `None`.
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
    /// The Kneipentour: lives, the city, the route choice (ADR 0016).
    pub tour: super::tour::TourView,
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
    /// Slots for relics.
    pub relic_slots: u32,
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
    pub next_pub: super::NextPubPlan,
    /// Groschen owed at the Tresen ("Deckel anschreiben"), paid from the
    /// next income.
    pub debt: u32,
    /// How far the Tresen would sell on credit now ("Deckel anschreiben"):
    /// the limit minus the debt, 0 without credit or once used in this
    /// district.
    pub credit_left: u32,
    /// The bet waiting for the next declared game ("Die Wette").
    pub wager: Option<super::Wager>,
    /// Route choices still to come whose exact stops are shown ("Kiebitz am
    /// Fenster").
    pub scout_legs: u32,
    /// The last scored game.
    pub last_score: Option<ScoreView>,
    /// The player's edition of the 32 cards (ADR 0015), in id order.
    pub edition: Vec<DeckCard>,
    /// Statistics of the run.
    pub stats: RunStats,
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
    /// Bier"), [`Run::redeals_left`]. Back to the full allowance with every
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

/// An option of the open encounter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EncounterOptionView {
    /// Index for [`RunAction::ChooseEncounter`].
    pub index: usize,
    /// What it does.
    pub choice: EncounterChoice,
    /// Groschen it costs.
    pub cost: u32,
    /// The relic it names (house rule on offer, relic given away).
    pub relic: Option<RelicInfo>,
    /// The relic received by a swap ("Alter Bekannter"), known before the
    /// choice.
    pub new_relic: Option<RelicInfo>,
    /// The modification a card gets ("Der Kartengeber" and the like),
    /// known before the choice.
    pub card_mod: Option<CardModInfo>,
    /// The curse that comes with it ("Alte Karten im Schrank", "Verlorenes
    /// Portemonnaie"), known before the choice.
    pub curse: Option<CardModInfo>,
    /// The consumable it gives ("Der Wirt spendiert").
    pub consumable: Option<ConsumableInfo>,
    /// Whether what the option gives is only drawn at random when it is
    /// chosen: a swap or a modification of a save from before they were
    /// drawn in advance. Then `new_relic` or `card_mod` is `None`.
    pub random_gain: bool,
    /// Whether it can be chosen now (with one of `targets` if it needs
    /// one).
    pub can_choose: bool,
    /// Whether it needs a target card of the deck.
    pub needs_target: bool,
    /// Deck card ids it can be used on.
    pub targets: Vec<u32>,
    /// What the option costs on each of `targets`, in the same order: a
    /// Zinke on a card may cost less than `cost` ("Bruderherz": half on
    /// Rot, [`Run::zinken_price`]). Empty without targets.
    pub target_costs: Vec<u32>,
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
    /// Its options; the last one is always "Weitergehen".
    pub options: Vec<EncounterOptionView>,
}

/// The count of Augen as "Der Zähler" shows it. Only what the player knows:
/// the tricks of both sides, the skat once he knows it, and the Augen of
/// every card he has not seen yet (the talon included, since nobody knows
/// which cards lie there).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct CountView {
    /// Augen the player took in tricks, plus the skat once he knows it.
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

/// The relic pick of a won boss pub (ADR 0017).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BossRewardView {
    /// The relics on offer, with the index for
    /// [`RunAction::ChooseBossRelic`].
    pub offers: Vec<BossRelicOffer>,
    /// Whether every relic slot is taken: then a relic can only be taken
    /// for an owned one (sold for half its price, `replace`).
    pub slots_full: bool,
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
    /// Einmischen per game of the character, without relics.
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
}

/// An owned relic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OwnedRelic {
    /// Index for [`RunAction::SellRelic`].
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
    /// Slot for [`RunAction::UseConsumable`] and [`RunAction::SellConsumable`].
    pub slot: usize,
    /// The consumable.
    pub consumable: ConsumableInfo,
    /// Groschen when sold.
    pub sell_price: u32,
    /// Whether it can be sold now.
    pub can_sell: bool,
    /// Whether it can be used now (with one of `targets` if it needs one).
    pub can_use: bool,
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
    /// Price of a Neuauslage.
    pub reroll_cost: u32,
    /// Whether a Neuauslage is possible now.
    pub can_reroll: bool,
    /// Neuauslagen bought during this visit.
    pub rerolls: u32,
}

/// A relic on offer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RelicOffer {
    /// Slot for [`RunAction::BuyRelic`].
    pub slot: usize,
    /// The relic.
    pub relic: RelicInfo,
    /// What it costs now (the content price minus discounts).
    pub price: u32,
    /// Whether it can be bought now (Groschen and free slot).
    pub can_buy: bool,
}

/// A consumable on offer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConsumableOffer {
    /// Slot for [`RunAction::BuyConsumable`].
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
    /// on this card works in this game ([`card_mod_status`]). Empty before
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
    /// ([`Run::can_rise`]): it comes back once per game in a suit game or
    /// Grand, not as the last card. Left out of the JSON when `false`.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub can_rise: bool,
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
    /// Whether the skat can be picked up ([`RunAction::PickUpSkat`]): only
    /// before the skat, not against "Der Schieber" (every game is a hand
    /// game).
    pub can_pick_up_skat: bool,
    /// Whether "Der Angeber" demands Ouvert.
    pub ouvert_required: bool,
    /// Whether the game can be dealt again now (Einmischen,
    /// [`RunAction::Redeal`]).
    pub can_redeal: bool,
    /// Einmischen left now ([`Run::redeals_left`]).
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
    /// the skat now (with [`RunAction::TakeFromSkat`]): only if one lies
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
    /// [`RunAction::SettleGame`] ("Rest abkürzen"): it is his turn and the
    /// remaining cards can no longer change the score, however anyone
    /// plays them ([`Run::can_settle`]). Says nothing about the
    /// defenders' cards beyond that.
    pub can_settle: bool,
    /// Cards to discard now: 2, or 3 after drawing a card with the skat
    /// ("Drei drücken", [`RunAction::DiscardThree`]); 0 outside
    /// [`TablePhase::Discarding`].
    pub discard_count: usize,
    /// Cards of the untouched skat the player was shown after declaring a
    /// hand game ("Spickzettel"), as long as they lie in the skat.
    pub skat_shown: Vec<CardView>,
    /// Whether "Drei drücken" draws a card at the next pickup of the skat
    /// (once per series; [`Run::discard_three_ready`]).
    pub discard_three_ready: bool,
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

/// The trump games of the Ansage dialog in dialog order; Null follows.
const GAMES: [GameType; 5] = [
    GameType::Suit(Suit::Clubs),
    GameType::Suit(Suit::Spades),
    GameType::Suit(Suit::Hearts),
    GameType::Suit(Suit::Diamonds),
    GameType::Grand,
];

/// Display order of a card in a hand: trumps by strength, then the suits
/// Kreuz, Pik, Herz, Karo by rank, blanks last. Uses the printed card, so a
/// "Gezinkt" card stays in its place; a "Bube ehrenhalber" sits below the
/// jacks in a suit game.
fn display_key(game: GameType, card: &DealtCard) -> (u8, u8, u32) {
    let Some(face) = card.face() else {
        return (9, 0, card.id);
    };
    let printed = DealtCard {
        rank_bonus: 0,
        ..*card
    };
    if let Some((strength, exact)) = trump_order_key(game, printed) {
        return (0, u8::MAX - (2 * strength + exact), card.id);
    }
    let suit_rank = match face.suit {
        Suit::Clubs => 1,
        Suit::Spades => 2,
        Suit::Hearts => 3,
        Suit::Diamonds => 4,
    };
    (
        suit_rank,
        u8::MAX - suit_strength::<Card>(game, face),
        card.id,
    )
}

impl Run {
    /// Whether the run would accept `action` now. Tried on a copy.
    #[must_use]
    pub fn accepts(&self, content: &Content, action: RunAction) -> bool {
        let mut probe = self.clone();
        probe.dispatch(content, action, &mut Vec::new()).is_ok()
    }

    /// The card back of a seat (ADR 0015): `None` for the player, the
    /// boss's in his seat, a Stammgast's otherwise.
    fn back_of_seat(&self, content: &Content, seat: Seat) -> Option<String> {
        match self.owner_of_seat(seat) {
            Owner::Player => None,
            Owner::Boss => Some(self.boss_back(content, 0)),
            Owner::BossPartner => Some(self.boss_back(content, 1)),
            Owner::Guest => Some(
                self.guests(content)
                    .iter()
                    .zip(Seat::DEFENDERS)
                    .find(|(_, s)| *s == seat)
                    .and_then(|(id, _)| content.guest(id))
                    .map_or_else(|| OPPONENT_BACK.to_owned(), |g| g.back.clone()),
            ),
        }
    }

    /// The back of a boss player (figure 0 left, 1 right): the figure's
    /// own, else the boss's (for "Der Alte" always his, not that of the
    /// boss whose rule is on).
    fn boss_back(&self, content: &Content, figure: usize) -> String {
        let back = |id: &BossId| {
            content.boss(id).and_then(|b| {
                b.figures
                    .get(figure)
                    .and_then(|f| f.back.clone())
                    .or_else(|| b.back.clone())
            })
        };
        self.series
            .boss
            .as_ref()
            .and_then(back)
            .or_else(|| self.game.as_ref()?.boss_rule.as_ref().and_then(back))
            .unwrap_or_else(|| OPPONENT_BACK.to_owned())
    }

    /// The card back of a card's owner: `None` for the player's own.
    fn back_of(&self, content: &Content, owner: Owner) -> Option<String> {
        match owner {
            Owner::Player => None,
            Owner::Boss => Some(self.boss_back(content, 0)),
            Owner::BossPartner => Some(self.boss_back(content, 1)),
            // The Stammgast's seat: right of the player (seat 2) holds one
            // in every series.
            Owner::Guest => self.back_of_seat(content, Seat::DEFENDERS[1]),
        }
    }

    fn card_view(&self, content: &Content, card: DealtCard) -> CardView {
        let chalked = self.game.as_ref().and_then(|a| {
            a.chalked
                .iter()
                .find(|(id, _)| *id == card.id)
                .map(|&(_, suit)| suit)
        });
        let (mods, mod_values) = self
            .editions()
            .of(card.owner)
            .and_then(|e| e.get(card.id))
            .map(|d| (d.mods.clone(), d.mod_values.clone()))
            .unwrap_or_default();
        let mod_status = self.mod_status(content, card, &mods);
        let named = self.game.as_ref().and_then(|a| {
            a.named
                .iter()
                .find(|(id, _)| *id == card.id)
                .map(|&(_, suit)| suit)
        });
        CardView {
            chalked,
            card,
            mods,
            back: self.back_of(content, card.owner),
            mod_values,
            legal: false,
            from_skat: false,
            drawn_from: None,
            discardable: false,
            mod_status,
            unsettled: card.wild
                && self
                    .game
                    .as_ref()
                    .is_some_and(|a| a.game.declaration().is_none()),
            named,
            can_rise: false,
        }
    }

    /// Whether the conditional modifications on `card` work in the game in
    /// progress, from what the player knows ([`card_mod_status`]).
    fn mod_status(
        &self,
        content: &Content,
        card: DealtCard,
        mods: &[CardModId],
    ) -> BTreeMap<CardModId, CardModStatus> {
        let Some(active) = &self.game else {
            return BTreeMap::new();
        };
        let ctx = HookContext {
            flags: &self.flags,
            counters: &self.counters,
            game: Some(&active.game),
            played: None,
            result: None,
            games_left: self.series.games_left(),
            transport: None,
        };
        mods.iter()
            .filter_map(|id| {
                let def = content.card_mod(id)?;
                card_mod_status(def, &ctx, card).map(|status| (id.clone(), status))
            })
            .collect()
    }

    fn relic_info(content: &Content, id: &RelicId) -> RelicInfo {
        match content.relic(id) {
            Some(def) => RelicInfo {
                id: id.clone(),
                kind: def.kind,
                name: def.name.clone(),
                text: def.text.clone(),
                rarity: def.rarity,
                price: def.price,
                group: def.group,
            },
            None => RelicInfo {
                id: id.clone(),
                kind: RelicKind::HouseRule,
                name: id.to_string(),
                text: String::new(),
                rarity: Rarity::Common,
                price: 0,
                group: crate::content::RelicGroup::Value,
            },
        }
    }

    fn card_mod_info(content: &Content, id: &CardModId, value: Option<i32>) -> CardModInfo {
        match content.card_mod(id) {
            Some(def) => {
                let value = def
                    .has_value()
                    .then(|| value.unwrap_or_else(|| def.default_value()));
                CardModInfo {
                    id: id.clone(),
                    name: def.name.clone(),
                    text: def.text_for(value.unwrap_or(1)),
                    rarity: def.rarity,
                    curse: def.is_curse(),
                    kind: def.kind,
                    value,
                }
            }
            None => CardModInfo {
                id: id.clone(),
                name: id.to_string(),
                text: String::new(),
                rarity: Rarity::Common,
                curse: false,
                kind: ModKind::Edge,
                value,
            },
        }
    }

    fn consumable_info(content: &Content, id: &ConsumableId) -> ConsumableInfo {
        match content.consumable(id) {
            Some(def) => ConsumableInfo {
                id: id.clone(),
                name: def.name.clone(),
                text: def.text.clone(),
                rarity: def.rarity,
                price: def.price,
                needs_target: matches!(
                    def.action,
                    ConsumableAction::CleanseCard
                        | ConsumableAction::AddCardMod(_)
                        | ConsumableAction::ApplyCardMod { .. }
                ),
                needs_suit: def.action == ConsumableAction::ChalkCard,
                category: def.category,
            },
            None => ConsumableInfo {
                id: id.clone(),
                name: id.to_string(),
                text: String::new(),
                rarity: Rarity::Common,
                price: 0,
                needs_target: false,
                needs_suit: false,
                category: ConsumableCategory::Item,
            },
        }
    }

    /// A boss as the run's character meets it (with its Zielwert factor
    /// for that character).
    pub(super) fn boss_info(&self, content: &Content, id: &BossId) -> BossInfo {
        match content.boss(id) {
            Some(def) => {
                let back = def
                    .back
                    .clone()
                    .unwrap_or_else(|| crate::content::OPPONENT_BACK.to_owned());
                BossInfo {
                    id: id.clone(),
                    name: def.name.clone(),
                    text: def.text.clone(),
                    target_factor: def.target_factor_for(self.character),
                    figures: def
                        .figures
                        .iter()
                        .map(|f| BossFigureInfo {
                            name: f.name.clone(),
                            edition: f
                                .edition
                                .iter()
                                .map(|(filter, card_mod, value)| EditionEntry {
                                    filter: *filter,
                                    card_mod: Run::card_mod_info(content, card_mod, Some(*value)),
                                })
                                .collect(),
                            back: f.back.clone().unwrap_or_else(|| back.clone()),
                        })
                        .collect(),
                    back,
                }
            }
            None => BossInfo {
                id: id.clone(),
                name: id.to_string(),
                text: String::new(),
                target_factor: Factor::ONE,
                figures: Vec::new(),
                back: crate::content::OPPONENT_BACK.to_owned(),
            },
        }
    }

    /// The pub after the current one: where the way leads (a Spelunke's
    /// target with its factor), else the regular next pub. A new
    /// district's boss is not drawn yet, so its first pub shows none.
    fn next_series_view(&self, content: &Content) -> Option<NextSeriesView> {
        if matches!(self.phase, RunPhase::Won | RunPhase::Lost) {
            return None;
        }
        let (section, index, spelunke) = match (&self.trip, &self.route) {
            (Some(trip), _) => (trip.section, trip.index, trip.spelunke.is_some()),
            (None, Some(route)) => (route.section, route.index, false),
            (None, None) => {
                let (s, i) = self.next_series_position();
                (s, i, false)
            }
        };
        if section >= content.run.sections() {
            return None;
        }
        let is_boss = index + 1 == PUBS_PER_DISTRICT;
        let boss = if is_boss && section == self.series.section {
            self.section_boss.as_ref()
        } else {
            None
        };
        let mut target = series_target(content, self.character, section, index, boss)?;
        if spelunke {
            target = content.run.spelunke.target_factor.apply(target);
        }
        Some(NextSeriesView {
            section,
            index,
            is_boss,
            target,
            game_limit: content.run.game_limit,
            redeals: content.redeals(self.character),
            boss: boss.map(|id| self.boss_info(content, id)),
        })
    }

    /// The read-only view of the run for a frontend.
    #[must_use]
    pub fn view(&self, content: &Content) -> RunView {
        let series = &self.series;
        let boss = series.boss.as_ref().map(|id| self.boss_info(content, id));
        let boss_ahead = match self.phase {
            RunPhase::Game => true,
            RunPhase::Shop | RunPhase::Encounter | RunPhase::Route => !series.is_boss_pub(),
            RunPhase::BossReward | RunPhase::Won | RunPhase::Lost => false,
        };
        let upcoming_boss = self
            .section_boss
            .as_ref()
            .filter(|_| boss_ahead)
            .map(|id| self.boss_info(content, id));
        let relics = self
            .relics
            .iter()
            .enumerate()
            .map(|(index, id)| {
                let relic = Run::relic_info(content, id);
                OwnedRelic {
                    index,
                    sell_price: sell_price(relic.price),
                    relic,
                    can_sell: self.accepts(content, RunAction::SellRelic(index)),
                }
            })
            .collect();
        let consumables = self
            .consumables
            .iter()
            .enumerate()
            .map(|(slot, id)| self.owned_consumable(content, slot, id))
            .collect();
        let character = content.character(self.character);
        RunView {
            seed: self.seed.to_string(),
            character: CharacterInfo {
                character: self.character,
                name: character.map_or_else(|| self.character.to_string(), |c| c.name.clone()),
                text: character.map_or_else(String::new, |c| c.text.clone()),
                back: character
                    .map_or(crate::content::PLAYER_BACK, |c| c.back())
                    .to_owned(),
            },
            phase: self.phase,
            ai_to_move: self.ai_to_move(),
            series: SeriesView {
                section: series.section,
                index: series.index,
                is_boss: series.is_boss_pub(),
                target: series.target,
                score: series.score,
                games_played: series.games_played,
                game_limit: series.game_limit,
                games_left: series.games_left(),
                redeals_left: self.redeals_left(content),
                redeals_used: series.redeals_used,
                boss,
                guests: self.guest_infos(content),
            },
            upcoming_boss,
            next_series: self.next_series_view(content),
            deal_odds: crate::deck::deal_odds(),
            sections: self.sections(content),
            length: self.length,
            tour: self.tour_view(content),
            stake: self.stake,
            win_threshold: crate::rules::WIN_THRESHOLD.saturating_add(self.stake),
            coins: self.coins,
            relics,
            relic_slots: content.relic_slots(self.character),
            consumables,
            consumable_slots: content.run.consumable_slots,
            flags: self.flags.iter().cloned().collect(),
            counters: self.counters.clone(),
            game: self.game_view(content),
            shop: self.shop_view(content),
            encounter: self.encounter_view(content),
            boss_reward: self.boss_reward_view(content),
            evening_relics: self
                .evening_relics
                .iter()
                .map(|id| Run::relic_info(content, id))
                .collect(),
            pending_factor: self.pending_factor(),
            pending_next_game: self
                .pending_next_game
                .iter()
                .map(|(consumable, effect)| PendingNextGame {
                    consumable: consumable.clone(),
                    effect: *effect,
                })
                .collect(),
            next_pub: self.next_pub,
            debt: self.debt,
            credit_left: self.credit_left(content),
            wager: self.wager.clone(),
            scout_legs: self.scout_legs,
            last_score: self.last_score.as_ref().map(|b| ScoreView {
                breakdown: b.clone(),
                steps: b.steps(),
            }),
            edition: self.edition.cards().to_vec(),
            stats: self.stats,
        }
    }

    /// The open relic pick of a won boss pub.
    fn boss_reward_view(&self, content: &Content) -> Option<BossRewardView> {
        let reward = self.boss_reward.as_ref()?;
        let slots = usize::try_from(content.relic_slots(self.character)).unwrap_or(0);
        Some(BossRewardView {
            offers: reward
                .relics
                .iter()
                .enumerate()
                .map(|(index, id)| BossRelicOffer {
                    index,
                    relic: Run::relic_info(content, id),
                })
                .collect(),
            slots_full: self.relics.len() >= slots,
        })
    }

    fn owned_consumable(
        &self,
        content: &Content,
        slot: usize,
        id: &ConsumableId,
    ) -> OwnedConsumable {
        let consumable = Run::consumable_info(content, id);
        let (can_use, targets) = if consumable.needs_suit {
            let hand: Vec<DealtCard> = self
                .game
                .as_ref()
                .map(|a| a.game.hand(Seat::DECLARER).to_vec())
                .unwrap_or_default();
            let targets: Vec<u32> = hand
                .iter()
                .filter(|card| {
                    // Every other suit is as good as any: try one.
                    let own = card.face().map(|f| f.suit);
                    let other = Suit::ALL.into_iter().find(|&s| Some(s) != own);
                    self.accepts(
                        content,
                        RunAction::UseConsumable {
                            slot,
                            target: Some(card.id),
                            suit: other,
                        },
                    )
                })
                .map(|c| c.id)
                .collect();
            (!targets.is_empty(), targets)
        } else if consumable.needs_target {
            let targets: Vec<u32> = self
                .edition
                .cards()
                .iter()
                .map(|c| c.id)
                .filter(|&card| {
                    self.accepts(
                        content,
                        RunAction::UseConsumable {
                            slot,
                            target: Some(card),
                            suit: None,
                        },
                    )
                })
                .collect();
            (!targets.is_empty(), targets)
        } else {
            let action = RunAction::UseConsumable {
                slot,
                target: None,
                suit: None,
            };
            (self.accepts(content, action), Vec::new())
        };
        OwnedConsumable {
            slot,
            sell_price: sell_price(consumable.price),
            consumable,
            can_sell: self.accepts(content, RunAction::SellConsumable(slot)),
            can_use,
            targets,
        }
    }

    /// How far the Tresen would sell on credit now: the limit minus the
    /// debt, 0 once the credit of this district is used up.
    fn credit_left(&self, content: &Content) -> u32 {
        let limit = self.credit_limit(content);
        let open = self.shop.as_ref().is_some_and(|s| s.on_credit)
            || self.credit_section != Some(self.way_section());
        if open {
            limit.saturating_sub(self.debt)
        } else {
            0
        }
    }

    fn shop_view(&self, content: &Content) -> Option<ShopView> {
        let shop = self.shop.as_ref()?;
        Some(ShopView {
            relics: shop
                .relics
                .iter()
                .enumerate()
                .map(|(slot, id)| {
                    id.as_ref().map(|id| {
                        let relic = Run::relic_info(content, id);
                        RelicOffer {
                            slot,
                            price: self.shop_price(content, relic.price),
                            relic,
                            can_buy: self.accepts(content, RunAction::BuyRelic(slot)),
                        }
                    })
                })
                .collect(),
            consumables: shop
                .consumables
                .iter()
                .enumerate()
                .map(|(slot, id)| {
                    id.as_ref().map(|id| {
                        let consumable = Run::consumable_info(content, id);
                        ConsumableOffer {
                            slot,
                            price: self.shop_price(content, consumable.price),
                            consumable,
                            can_buy: self.accepts(content, RunAction::BuyConsumable(slot)),
                        }
                    })
                })
                .collect(),
            reroll_cost: self.shop_price(content, content.run.economy.reroll_cost),
            can_reroll: self.accepts(content, RunAction::Reroll),
            rerolls: shop.rerolls,
        })
    }

    /// Whether the skat is picked up and discarded: the Ansage without hand
    /// levels.
    fn after_skat(&self) -> bool {
        self.game
            .as_ref()
            .is_some_and(|a| a.game.phase() == Phase::DeclaringAfterSkat)
    }

    /// The Ansage dialog (ADR 0011, 0012, 0013): every character may
    /// declare the suit games, the grand and Null, each with the levels of
    /// this phase (before the skat the hand levels, after it the levels
    /// without Hand: suit and grand `Normal`, Schneider and Schwarz
    /// angesagt, Ouvert; Null `Normal` and `Ouvert`).
    fn declare_options(&self, content: &Content) -> DeclareOptions {
        let after_skat = self.after_skat();
        let skipped = if after_skat {
            RiskLevel::Hand
        } else {
            RiskLevel::Normal
        };
        let risks: Vec<RiskOption> = RiskLevel::ALL
            .into_iter()
            .filter(|&risk| risk != skipped)
            .map(|risk| RiskOption {
                risk,
                allowed: self.accepts(content, RunAction::Declare(Declaration::grand(risk))),
            })
            .collect();
        let lowest = risks
            .iter()
            .find(|r| r.allowed)
            .or_else(|| risks.first())
            .map_or(RiskLevel::Normal, |r| r.risk);
        let scoring = self.null_scoring(content);
        let null_risks: Vec<NullRiskOption> = NullRisk::ALL
            .into_iter()
            .filter(|risk| risk.hand() != after_skat)
            .map(|risk| NullRiskOption {
                risk,
                allowed: self.accepts(content, RunAction::Declare(Declaration::null(risk))),
                value: match &scoring {
                    NullScoring::Fixed => {
                        u64::from(crate::rules::null_value(risk.hand(), risk.ouvert()))
                    }
                    NullScoring::Formula(f) => u64::from(f.base_value) * u64::from(risk.step()),
                },
            })
            .collect();
        let null_base = match &scoring {
            NullScoring::Fixed => crate::rules::null_value(false, false),
            NullScoring::Formula(f) => f.base_value,
        };
        let games = GAMES
            .into_iter()
            .map(|game| GameOption {
                game,
                base_value: game.base_value().unwrap_or(0),
                allowed: Declaration::new(game, lowest)
                    .is_some_and(|d| self.accepts(content, RunAction::Declare(d))),
            })
            .chain(std::iter::once(GameOption {
                game: GameType::Null,
                base_value: null_base,
                allowed: null_risks.iter().any(|r| r.allowed),
            }))
            .collect();
        let can_redeal = self.accepts(content, RunAction::Redeal);
        DeclareOptions {
            games,
            risks,
            null_risks,
            null_style: matches!(scoring, NullScoring::Formula(_)),
            can_pick_up_skat: self.accepts(content, RunAction::PickUpSkat),
            ouvert_required: self.game.as_ref().is_some_and(|a| a.rules.require_ouvert),
            can_redeal,
            redeals_left: self.redeals_left(content),
            hand_required: self.requires_hand(),
        }
    }

    /// The count of "Der Zähler" from what the player knows.
    fn count_view(&self) -> Option<CountView> {
        let active = self.game.as_ref()?;
        if !active.rules.show_count {
            return None;
        }
        let game = &active.game;
        let points = game.trick_card_points();
        let known_skat = game.skat_picked_up() && game.phase() != Phase::Discarding;
        let skat_points: u32 = game.skat().iter().map(|c| c.card_points()).sum();
        let mut seen: Vec<DealtCard> = game.hand(Seat::DECLARER).to_vec();
        seen.extend(game.tricks().iter().flat_map(|t| t.cards));
        seen.extend(game.current_trick().iter().copied());
        if known_skat {
            seen.extend(game.skat());
        }
        // A skat card shown by "Spickzettel", a card given back ("Drei
        // drücken").
        seen.extend(active.skat_shown.iter().copied());
        seen.extend(game.returned().map(|(_, c)| c));
        let unseen: u32 = game
            .deck_cards()
            .iter()
            .filter(|c| !seen.contains(c))
            .map(|c| c.card_points())
            .sum();
        let own = points[0] + if known_skat { skat_points } else { 0 };
        Some(CountView {
            declarer: own,
            defenders: points[1] + points[2],
            unseen,
            seats: [own, points[1], points[2]],
        })
    }

    fn encounter_view(&self, content: &Content) -> Option<EncounterView> {
        let encounter = self.encounter.as_ref()?;
        let def = content.encounter(&encounter.id);
        let options = encounter
            .options
            .iter()
            .enumerate()
            .map(|(index, option)| {
                let needs_target = matches!(
                    option.choice,
                    EncounterChoice::MarkChosenCard
                        | EncounterChoice::CleanseChosenCard
                        | EncounterChoice::Gamble
                        | EncounterChoice::Lesson { .. }
                        | EncounterChoice::OldCards { .. }
                );
                let relic = match &option.choice {
                    EncounterChoice::TakeRule(id)
                    | EncounterChoice::TakeRelic(id)
                    | EncounterChoice::SwapRelic { relic: id, .. } => {
                        Some(Run::relic_info(content, id))
                    }
                    _ => None,
                };
                let new_relic = match &option.choice {
                    EncounterChoice::SwapRelic { new: Some(id), .. } => {
                        Some(Run::relic_info(content, id))
                    }
                    _ => None,
                };
                let marks = matches!(
                    option.choice,
                    EncounterChoice::MarkRandomCard | EncounterChoice::MarkChosenCard
                );
                let names_mod = marks
                    || matches!(
                        option.choice,
                        EncounterChoice::Gamble
                            | EncounterChoice::Lesson { .. }
                            | EncounterChoice::OldCards { .. }
                    );
                let card_mod = option
                    .card_mod
                    .as_ref()
                    .filter(|_| names_mod)
                    .map(|id| Run::card_mod_info(content, id, option.value));
                let curse = match &option.choice {
                    EncounterChoice::OldCards { curse, .. }
                    | EncounterChoice::KeepWallet { curse, .. } => {
                        Some(Run::card_mod_info(content, curse, None))
                    }
                    _ => None,
                };
                let consumable = match &option.choice {
                    EncounterChoice::TakeConsumable(id) => Some(Run::consumable_info(content, id)),
                    _ => None,
                };
                let random_gain = match &option.choice {
                    EncounterChoice::SwapRelic { new, .. } => new.is_none(),
                    _ => marks && card_mod.is_none(),
                };
                let targets: Vec<u32> = if needs_target {
                    self.edition
                        .cards()
                        .iter()
                        .map(|c| c.id)
                        .filter(|&card| {
                            self.accepts(
                                content,
                                RunAction::ChooseEncounter {
                                    option: index,
                                    target: Some(card),
                                },
                            )
                        })
                        .collect()
                } else {
                    Vec::new()
                };
                let target_costs: Vec<u32> = targets
                    .iter()
                    .map(|&card| match option.choice {
                        EncounterChoice::MarkChosenCard | EncounterChoice::Gamble => {
                            self.zinken_price(content, card, option.cost)
                        }
                        _ => option.cost,
                    })
                    .collect();
                let can_choose = if needs_target {
                    !targets.is_empty()
                } else {
                    self.accepts(
                        content,
                        RunAction::ChooseEncounter {
                            option: index,
                            target: None,
                        },
                    )
                };
                EncounterOptionView {
                    index,
                    choice: option.choice.clone(),
                    cost: option.cost,
                    relic,
                    new_relic,
                    card_mod,
                    curse,
                    consumable,
                    random_gain,
                    can_choose,
                    needs_target,
                    targets,
                    target_costs,
                }
            })
            .collect();
        Some(EncounterView {
            id: encounter.id.clone(),
            name: def.map_or_else(|| encounter.id.to_string(), |d| d.name.clone()),
            text: def.map_or_else(String::new, |d| d.text.clone()),
            options,
        })
    }

    fn game_view(&self, content: &Content) -> Option<GameView> {
        if self.phase != RunPhase::Game {
            return None;
        }
        let active = self.game.as_ref()?;
        let game = &active.game;
        let declaration = game.declaration();
        let phase = match game.phase() {
            Phase::Declaring => TablePhase::Declaring,
            Phase::Discarding => TablePhase::Discarding,
            Phase::DeclaringAfterSkat => TablePhase::DeclaringAfterSkat,
            Phase::Playing => TablePhase::Playing,
            Phase::Finished(_) => TablePhase::Finished,
        };
        let order_game = declaration.map_or(GameType::Grand, |d| d.game());
        let legal = if active.awaiting_re {
            Vec::new()
        } else {
            game.legal_moves(Seat::DECLARER)
        };
        let skat = game.skat();
        let held = game.hand(Seat::DECLARER);
        let discarding = phase == TablePhase::Discarding && !active.awaiting_re;
        let discard_count = game.discard_count();
        let drawn = game.drawn().filter(|_| phase == TablePhase::Discarding);
        let mut hand: Vec<DealtCard> = held.to_vec();
        hand.sort_by_key(|c| display_key(order_game, c));
        let hand = hand
            .into_iter()
            .map(|card| CardView {
                legal: legal.contains(&card),
                from_skat: phase == TablePhase::Discarding && skat.contains(&card),
                drawn_from: drawn.filter(|(_, c)| c.id == card.id).map(|(seat, _)| seat),
                discardable: discarding && discardable_of(held, card, discard_count),
                can_rise: self.can_rise(content, card),
                ..self.card_view(content, card)
            })
            .collect();
        let trick = game
            .current_trick()
            .iter()
            .enumerate()
            .map(|(i, &card)| PlayedCard {
                seat: seat_at(game.trick_lead(), i),
                card: self.card_view(content, card),
            })
            .collect();
        let last_trick = game.tricks().last().map(|t| TrickView {
            cards: t
                .cards
                .iter()
                .enumerate()
                .map(|(i, &card)| PlayedCard {
                    seat: t.seat_of(i),
                    card: self.card_view(content, card),
                })
                .collect(),
            winner: t.winner,
        });
        let declarer_tricks = game
            .tricks()
            .iter()
            .filter(|t| t.winner.is_declarer())
            .count();
        let known_skat = (game.skat_picked_up() && phase != TablePhase::Discarding)
            .then(|| skat.iter().map(|&c| self.card_view(content, c)).collect());
        let mut revealed: Vec<RevealedCard> = self
            .revealed_holders()
            .into_iter()
            .map(|(card, holder)| RevealedCard {
                source: RevealSource::CardMod(self.reveal_mod(content, card)),
                card: self.card_view(content, card),
                holder,
            })
            .collect();
        // The hand `RevealDefenderHand` shows, in display order, like the player's.
        let mut peeked = active.peeked.clone();
        peeked.sort_by_key(|(card, seat)| {
            if active.shown_hand == Some(*seat) {
                display_key(order_game, card)
            } else {
                (0, 0, 0)
            }
        });
        for &(card, seat) in &peeked {
            // The copy of its holder now (a swap changes the edition).
            let card = game.current(card);
            if !revealed.iter().any(|r| r.card.card.id == card.id) {
                revealed.push(RevealedCard {
                    card: self.card_view(content, card),
                    holder: game.holder_of(card),
                    source: if active.shown_hand == Some(seat) {
                        RevealSource::Onlooker
                    } else {
                        RevealSource::Kibitz
                    },
                });
            }
        }
        // The card given back ("Drei drücken"): the player knows who holds
        // it until it is played.
        if let Some((_, card)) = game.returned() {
            let card = game.current(card);
            if !revealed.iter().any(|r| r.card.card.id == card.id) {
                revealed.push(RevealedCard {
                    card: self.card_view(content, card),
                    holder: game.holder_of(card),
                    source: RevealSource::Returned,
                });
            }
        }
        let skat_swap_cards: Vec<CardView> = self
            .skat_swap_cards()
            .into_iter()
            .map(|c| self.card_view(content, c))
            .collect();
        let can_take_from_skat = !skat_swap_cards.is_empty()
            && game.hand(Seat::DECLARER).iter().any(|c| {
                self.accepts(
                    content,
                    RunAction::TakeFromSkat {
                        give: c.id,
                        take: None,
                    },
                )
            });
        let skat_swap_cards = if can_take_from_skat {
            skat_swap_cards
        } else {
            Vec::new()
        };
        Some(GameView {
            number: self.series.games_played + 1,
            phase,
            forehand: game.forehand(),
            positions: Seat::ALL.iter().map(|&s| game.position(s)).collect(),
            declaration,
            hand,
            hand_sizes: Seat::ALL.iter().map(|&s| game.hand(s).len()).collect(),
            trick_lead: game.trick_lead(),
            trick,
            last_trick,
            tricks_played: game.tricks().len(),
            declarer_tricks,
            defender_tricks: game.tricks().len() - declarer_tricks,
            skat: known_skat,
            your_turn: !active.awaiting_re && game.current_seat() == Some(Seat::DECLARER),
            kontra: active.kontra,
            awaiting_kontra_answer: active.awaiting_re,
            kontra_possible: active.rules.kontra_source.is_some(),
            win_threshold: active.rules.thresholds.win,
            declare: matches!(
                phase,
                TablePhase::Declaring | TablePhase::DeclaringAfterSkat
            )
            .then(|| self.declare_options(content)),
            revealed,
            wild_settled: active
                .named
                .iter()
                .filter_map(|&(card, printed)| {
                    let kind = game.deck_cards().iter().find(|c| c.id == card)?.kind;
                    Some(SettledWild {
                        card,
                        kind,
                        printed,
                    })
                })
                .collect(),
            boss_rule: active
                .boss_rule
                .as_ref()
                .map(|id| self.boss_info(content, id)),
            backs: Seat::ALL.map(|seat| self.back_of_seat(content, seat)),
            count: self.count_view(),
            can_take_from_skat,
            skat_swap_cards,
            skat_peek: (active.skat_peeked && !game.skat_picked_up())
                .then(|| skat.iter().map(|&c| self.card_view(content, c)).collect()),
            seat_tricks: game.seat_tricks(),
            value_preview: self.value_preview(content),
            can_settle: self.can_settle(content),
            discard_count,
            skat_shown: active
                .skat_shown
                .iter()
                .filter(|c| skat.contains(c))
                .map(|&c| self.card_view(content, c))
                .collect(),
            discard_three_ready: self.discard_three_ready(),
        })
    }

    /// The modification that shows the holder of `card` ("Durchsichtig"):
    /// on the copy's own edition, else on the player's edition.
    fn reveal_mod(&self, content: &Content, card: DealtCard) -> CardModId {
        let editions = self.editions();
        let reveals = |m: &&CardModId| {
            content
                .card_mod(m)
                .is_some_and(|def| def.passive.contains(&crate::content::Effect::RevealHolder))
        };
        [editions.of(card.owner), Some(editions.player)]
            .into_iter()
            .flatten()
            .filter_map(|e| e.get(card.id))
            .find_map(|d| d.mods.iter().find(reveals))
            .cloned()
            .unwrap_or_else(|| CardModId::new("transparent"))
    }
}

#[cfg(test)]
mod tests {
    use super::super::bot::RunBot;
    use super::*;

    fn content() -> Content {
        Content::load().expect("embedded content is valid")
    }

    #[test]
    fn fresh_run_offers_every_declaration_and_a_sorted_hand() {
        let content = content();
        let (run, _) = Run::new(&content, 3).expect("start");
        let view = run.view(&content);
        assert_eq!(view.phase, RunPhase::Game);
        assert_eq!(view.seed, "3");
        assert_eq!(view.series.target, run.series().target);
        let game = view.game.expect("game");
        assert_eq!(game.phase, TablePhase::Declaring);
        assert_eq!(game.hand.len(), 10);
        assert_eq!(game.hand_sizes, vec![10, 10, 10]);
        assert!(!game.your_turn);
        assert!(game.hand.iter().all(|c| !c.legal && !c.from_skat));
        let keys: Vec<_> = game
            .hand
            .iter()
            .map(|c| display_key(GameType::Grand, &c.card))
            .collect();
        let mut sorted = keys.clone();
        sorted.sort();
        assert_eq!(keys, sorted, "hand in display order");
        let options = game.declare.expect("options");
        // Every Spielart for every character (ADR 0012): Null last. The
        // Alleinspieler counts Null like the Skatordnung, without style
        // bonus.
        assert_eq!(options.games.len(), 6);
        assert!(options.games.iter().all(|g| g.allowed));
        assert!(options.risks.iter().all(|r| r.allowed));
        assert_eq!(options.games[0].base_value, 12);
        assert_eq!(options.games[4].base_value, 24);
        assert_eq!(options.games[5].game, GameType::Null);
        assert_eq!(options.games[5].base_value, 23);
        let null_risks: Vec<(NullRisk, u64)> = options
            .null_risks
            .iter()
            .map(|r| (r.risk, r.value))
            .collect();
        assert_eq!(
            null_risks,
            vec![(NullRisk::Hand, 46), (NullRisk::OuvertHand, 92)]
        );
        assert!(options.null_style);
        assert!(options.can_redeal);
        assert_eq!(options.redeals_left, content.run.redeals);
        assert!(options.can_pick_up_skat);
        assert!(!options.hand_required);
        // Before the skat only the hand levels (ADR 0011).
        assert!(options.risks.iter().all(|r| r.risk != RiskLevel::Normal));
        assert_eq!(options.risks.len(), 4);
    }

    #[test]
    fn view_agrees_with_the_run_through_a_whole_run() {
        let content = content();
        let (mut run, _) = Run::new(&content, 21).expect("start");
        let mut saw_discard = false;
        let mut saw_declare = [false; 2];
        let mut saw_play = false;
        let mut saw_shop = false;
        let mut saw_score = false;
        for _ in 0..2_000 {
            let view = run.view(&content);
            if let Some(game) = &view.game {
                let legal: Vec<u32> = game
                    .hand
                    .iter()
                    .filter(|c| c.legal)
                    .map(|c| c.card.id)
                    .collect();
                if game.phase == TablePhase::Discarding {
                    saw_discard = true;
                    assert_eq!(game.hand.len(), 12);
                    assert_eq!(game.hand.iter().filter(|c| c.from_skat).count(), 2);
                }
                if let Some(options) = &game.declare {
                    saw_declare[usize::from(game.phase == TablePhase::DeclaringAfterSkat)] = true;
                    assert_eq!(
                        options.can_pick_up_skat,
                        run.accepts(&content, RunAction::PickUpSkat)
                    );
                    for risk in &options.risks {
                        let grand = Declaration::grand(risk.risk);
                        assert_eq!(
                            risk.allowed,
                            run.accepts(&content, RunAction::Declare(grand))
                        );
                    }
                    for risk in &options.null_risks {
                        let null = Declaration::null(risk.risk);
                        assert_eq!(
                            risk.allowed,
                            run.accepts(&content, RunAction::Declare(null))
                        );
                    }
                }
                let previewed = game.phase == TablePhase::Playing
                    && game.declaration.is_some_and(|d| !d.is_ramsch());
                assert_eq!(game.value_preview.is_some(), previewed);
                if let Some(p) = game.value_preview {
                    assert_eq!(p.game_value, p.multiplier.scale(p.base_value * p.levels));
                }
                if game.your_turn {
                    saw_play = true;
                    assert!(!legal.is_empty());
                    for card in &game.hand {
                        assert_eq!(
                            card.legal,
                            run.accepts(&content, RunAction::Play(card.card.id)),
                            "legal flag matches the run"
                        );
                    }
                } else {
                    assert!(legal.is_empty());
                }
            }
            if let Some(shop) = &view.shop {
                saw_shop = true;
                for offer in shop.relics.iter().flatten() {
                    // The price after discounts ("Der Wirt").
                    let affordable = offer.price <= view.coins;
                    let free = view.relics.len() < view.relic_slots as usize;
                    assert_eq!(offer.can_buy, affordable && free);
                }
            }
            if let Some(score) = &view.last_score {
                saw_score = true;
                let last = score.steps.last().expect("steps");
                assert_eq!(last.base_value, score.breakdown.base_value);
                assert_eq!(last.levels, score.breakdown.levels);
                assert_eq!(last.multiplier, score.breakdown.multiplier);
            }
            let Some(action) = RunBot.next_action(&run, &content) else {
                break;
            };
            run.apply(&content, action).expect("bot action");
        }
        assert!(saw_discard && saw_play && saw_score, "covered the table");
        assert_eq!(saw_declare, [true, true], "both Ansage dialogs");
        assert!(saw_shop || run.phase() == RunPhase::Lost);
        let stats = run.stats();
        assert!(stats.games_won + stats.games_lost + stats.redeals > 0);
    }

    #[test]
    fn consumables_report_their_targets() {
        let content = content();
        let (mut run, _) = Run::new(&content, 5).expect("start");
        run.consumables = vec![ConsumableId::new("fresh_deck"), ConsumableId::new("herbal")];
        let view = run.view(&content);
        // In a game: Kräuter works, the fresh deck only at the Tresen.
        assert!(!view.consumables[0].can_use);
        assert!(view.consumables[0].consumable.needs_target);
        assert!(view.consumables[1].can_use);
        assert!(!view.consumables[1].can_sell, "selling only at the Tresen");
        run.game = None;
        run.phase = RunPhase::Shop;
        run.shop = Some(run.draw_offer(&content, 0));
        // Frisches Deck cleanses a card that carries a modification.
        let view = run.view(&content);
        assert!(!view.consumables[0].can_use);
        run.edition
            .add_mod(&content, 7, &CardModId::new("points"), 3)
            .expect("Augen");
        let view = run.view(&content);
        assert!(view.consumables[0].can_use);
        assert_eq!(view.consumables[0].targets, vec![7]);
        assert!(!view.consumables[1].can_use);
        assert!(view.consumables[1].can_sell);
        assert_eq!(view.consumables[1].sell_price, 1);
        assert!(view.game.is_none());
        assert!(view.shop.is_some());
    }
}
