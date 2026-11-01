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

use super::{
    EncounterChoice, Kontra, Run, RunAction, RunPhase, RunStats, SERIES_PER_SECTION, sell_price,
    series_target,
};
use crate::cards::{Card, PlayingCard, Suit};
use crate::content::{
    BossId, CardModId, Character, ConsumableAction, ConsumableId, Content, EncounterId, Factor,
    Rarity, RelicId, RelicKind, ScoreBreakdown, ScoreStep,
};
use crate::deck::{DealOdds, DealtCard, DeckCard};
use crate::game::{Phase, Position, Seat, seat_at};
use crate::rules::{
    Declaration, GameType, NullRisk, NullScoring, RiskLevel, discardable, suit_strength,
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
    /// The series after the current one, for the Tresen and the run
    /// overview. `None` after the last series and once the run is over.
    pub next_series: Option<NextSeriesView>,
    /// Where any one card of the deck lands when a game is dealt, for the
    /// current deck (DESIGN 7.4). For a deck after a change use
    /// [`crate::deck::deal_odds`] with the new size.
    pub deal_odds: DealOdds,
    /// Sections (Abschnitte) in a run.
    pub sections: u32,
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
    /// House rules of the evening (encounter "Stammtisch-Streit"), active
    /// until the section ends.
    pub evening_relics: Vec<RelicInfo>,
    /// Factor waiting for the next declared game ("Letzte Runde").
    pub pending_factor: Option<u32>,
    /// The last scored game.
    pub last_score: Option<ScoreView>,
    /// The shared deck, in id order.
    pub deck: Vec<DeckCard>,
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
    /// Games played or passed.
    pub games_played: u32,
    /// Spiellimit.
    pub game_limit: u32,
    /// Games left before the limit.
    pub games_left: u32,
    /// Passes left.
    pub passes_left: u32,
    /// Passes used.
    pub passes_used: u32,
    /// The boss of a boss series.
    pub boss: Option<BossInfo>,
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
    /// Passes at the start of the series.
    pub passes: u32,
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
    /// Whether using it needs a target card of the deck.
    pub needs_target: bool,
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
    /// Deck card ids it can be used on now.
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
    /// says what is allowed), pass or redeal.
    Declaring,
    /// Twelve cards: two to discard.
    Discarding,
    /// Skat picked up and discarded: the Ansage without hand levels.
    DeclaringAfterSkat,
    /// Tricks.
    Playing,
    /// Scored.
    Finished,
}

/// A card as shown to the player.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CardView {
    /// The dealt card with its effective Augen.
    #[serde(flatten)]
    pub card: DealtCard,
    /// Card modifications on this instance.
    pub mods: Vec<CardModId>,
    /// Whether the player may play it now.
    pub legal: bool,
    /// Whether it came from the skat (while discarding).
    pub from_skat: bool,
    /// Whether the player may put it into the skat now: only for hand
    /// cards while discarding. A card that must not be discarded
    /// ("Ramponiert") only when fewer than two other cards are left
    /// ([`crate::rules::discardable`]).
    pub discardable: bool,
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

/// A Null-Risikostufe in the Ansage dialog of the Nullspieler.
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
/// levels without Hand (suit and grand: `Normal`; Null: `Normal` and
/// `Ouvert`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeclareOptions {
    /// Spielarten (the Nullspieler: only Null). `allowed` with the lowest
    /// allowed Risikostufe of this phase.
    pub games: Vec<GameOption>,
    /// Risikostufen of suit games and grand allowed in this phase, lowest
    /// first (empty for the Nullspieler).
    pub risks: Vec<RiskOption>,
    /// Null-Risikostufen allowed in this phase (only the Nullspieler).
    pub null_risks: Vec<NullRiskOption>,
    /// Whether the skat can be picked up ([`RunAction::PickUpSkat`]): only
    /// before the skat, not against "Der Schieber" (every game is a hand
    /// game).
    pub can_pick_up_skat: bool,
    /// Whether "Der Angeber" demands Ouvert.
    pub ouvert_required: bool,
    /// Whether Einpassen is possible.
    pub can_pass: bool,
    /// Whether the game can be dealt again ("Schieben").
    pub can_redeal: bool,
    /// Redeals ("Schieben") left in this series.
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
    /// Augen needed to win.
    pub win_threshold: u32,
    /// Ansage options while declaring, before and after the skat.
    pub declare: Option<DeclareOptions>,
    /// "Durchsichtig" cards and the cards "Der Kiebitz" showed, with their
    /// holders.
    pub revealed: Vec<RevealedCard>,
    /// The boss rule in force in this game: the series' boss, for the
    /// Endboss "Der Alte" the rule of this game.
    pub boss_rule: Option<BossInfo>,
    /// The count of "Der Zähler", if a relic shows it.
    pub count: Option<CountView>,
    /// Whether "Die Oma" (or "Der Opa") lets the player take a card from
    /// the skat now (with [`RunAction::TakeFromSkat`]): only if one lies
    /// there.
    pub can_take_from_skat: bool,
    /// The skat cards a skat swap may take now, the one taken by default
    /// first (`take: None`): "Die Oma" names the jack that comes out.
    pub skat_swap_cards: Vec<CardView>,
    /// Tricks per seat so far (index = seat), as they lie on the table.
    pub seat_tricks: [u32; 3],
    /// What the game is worth if the player wins it, as far as it is
    /// certain now: only while playing a suit game, grand or Null
    /// ([`ValuePreview`]).
    pub value_preview: Option<ValuePreview>,
}

/// The score line of the table while playing: what the game in progress
/// is worth if the player wins it, as far as it is certain now. Computed
/// with the scoring of a finished game ([`crate::content::score_parts`]),
/// so `game_value = base_value × levels × multiplier` and the numbers are
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
    /// Product of the certain Spielwert factors (1 without any).
    pub multiplier: u64,
    /// `base_value × levels × multiplier`.
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

/// The Spielarten the Alleinspieler may choose, in dialog order.
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

    fn card_view(&self, card: DealtCard) -> CardView {
        CardView {
            card,
            mods: self
                .deck
                .get(card.id)
                .map(|d| d.mods.clone())
                .unwrap_or_default(),
            legal: false,
            from_skat: false,
            discardable: false,
        }
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
            },
            None => RelicInfo {
                id: id.clone(),
                kind: RelicKind::HouseRule,
                name: id.to_string(),
                text: String::new(),
                rarity: Rarity::Common,
                price: 0,
            },
        }
    }

    fn card_mod_info(content: &Content, id: &CardModId) -> CardModInfo {
        match content.card_mod(id) {
            Some(def) => CardModInfo {
                id: id.clone(),
                name: def.name.clone(),
                text: def.text.clone(),
                rarity: def.rarity,
                curse: def.curse,
            },
            None => CardModInfo {
                id: id.clone(),
                name: id.to_string(),
                text: String::new(),
                rarity: Rarity::Common,
                curse: false,
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
                    ConsumableAction::RemoveCard | ConsumableAction::AddCardMod(_)
                ),
            },
            None => ConsumableInfo {
                id: id.clone(),
                name: id.to_string(),
                text: String::new(),
                rarity: Rarity::Common,
                price: 0,
                needs_target: false,
            },
        }
    }

    fn boss_info(content: &Content, id: &BossId) -> BossInfo {
        match content.boss(id) {
            Some(def) => BossInfo {
                id: id.clone(),
                name: def.name.clone(),
                text: def.text.clone(),
                target_factor: def.target_factor,
            },
            None => BossInfo {
                id: id.clone(),
                name: id.to_string(),
                text: String::new(),
                target_factor: Factor::ONE,
            },
        }
    }

    /// The series after the current one. A new section's boss is not drawn
    /// yet, so its first series shows none.
    fn next_series_view(&self, content: &Content) -> Option<NextSeriesView> {
        if matches!(self.phase, RunPhase::Won | RunPhase::Lost) {
            return None;
        }
        let (section, index) = self.next_series_position();
        let is_boss = index + 1 == SERIES_PER_SECTION;
        let boss = if is_boss {
            self.section_boss.as_ref()
        } else {
            None
        };
        let target = series_target(content, self.character, section, index, boss)?;
        Some(NextSeriesView {
            section,
            index,
            is_boss,
            target,
            game_limit: content.run.game_limit,
            passes: content.run.passes,
            boss: boss.map(|id| Run::boss_info(content, id)),
        })
    }

    /// The read-only view of the run for a frontend.
    #[must_use]
    pub fn view(&self, content: &Content) -> RunView {
        let series = &self.series;
        let boss = series.boss.as_ref().map(|id| Run::boss_info(content, id));
        let boss_ahead = match self.phase {
            RunPhase::Game => true,
            RunPhase::Shop | RunPhase::Encounter => series.index + 1 < SERIES_PER_SECTION,
            RunPhase::Won | RunPhase::Lost => false,
        };
        let upcoming_boss = self
            .section_boss
            .as_ref()
            .filter(|_| boss_ahead)
            .map(|id| Run::boss_info(content, id));
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
            },
            phase: self.phase,
            ai_to_move: self.ai_to_move(),
            series: SeriesView {
                section: series.section,
                index: series.index,
                is_boss: series.index + 1 == SERIES_PER_SECTION,
                target: series.target,
                score: series.score,
                games_played: series.games_played,
                game_limit: series.game_limit,
                games_left: series.games_left(),
                passes_left: series.passes_left,
                passes_used: series.passes_used,
                boss,
            },
            upcoming_boss,
            next_series: self.next_series_view(content),
            deal_odds: self.deck.deal_odds(),
            sections: u32::try_from(content.run.targets.len()).unwrap_or(0),
            coins: self.coins,
            relics,
            relic_slots: content.run.relic_slots,
            consumables,
            consumable_slots: content.run.consumable_slots,
            flags: self.flags.iter().cloned().collect(),
            counters: self.counters.clone(),
            game: self.game_view(content),
            shop: self.shop_view(content),
            encounter: self.encounter_view(content),
            evening_relics: self
                .evening_relics
                .iter()
                .map(|id| Run::relic_info(content, id))
                .collect(),
            pending_factor: self.pending_factor(),
            last_score: self.last_score.as_ref().map(|b| ScoreView {
                breakdown: b.clone(),
                steps: b.steps(),
            }),
            deck: self.deck.cards().to_vec(),
            stats: self.stats,
        }
    }

    fn owned_consumable(
        &self,
        content: &Content,
        slot: usize,
        id: &ConsumableId,
    ) -> OwnedConsumable {
        let consumable = Run::consumable_info(content, id);
        let (can_use, targets) = if consumable.needs_target {
            let targets: Vec<u32> = self
                .deck
                .cards()
                .iter()
                .map(|c| c.id)
                .filter(|&card| {
                    self.accepts(
                        content,
                        RunAction::UseConsumable {
                            slot,
                            target: Some(card),
                        },
                    )
                })
                .collect();
            (!targets.is_empty(), targets)
        } else {
            let usable = self.accepts(content, RunAction::UseConsumable { slot, target: None });
            (usable, Vec::new())
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

    fn declare_options(&self, content: &Content) -> DeclareOptions {
        if self.character.plays_null() {
            return self.null_declare_options(content);
        }
        let after_skat = self.after_skat();
        let risks: Vec<RiskOption> = RiskLevel::ALL
            .into_iter()
            .filter(|&risk| (risk == RiskLevel::Normal) == after_skat)
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
        let games = GAMES
            .into_iter()
            .map(|game| GameOption {
                game,
                base_value: game.base_value().unwrap_or(0),
                allowed: Declaration::new(game, lowest)
                    .is_some_and(|d| self.accepts(content, RunAction::Declare(d))),
            })
            .collect();
        DeclareOptions {
            games,
            risks,
            null_risks: Vec::new(),
            can_pick_up_skat: self.accepts(content, RunAction::PickUpSkat),
            ouvert_required: self.game.as_ref().is_some_and(|a| a.rules.require_ouvert),
            can_pass: self.accepts(content, RunAction::Pass),
            can_redeal: self.accepts(content, RunAction::Redeal),
            redeals_left: self.redeals_left(),
            hand_required: self.requires_hand(),
        }
    }

    /// The Ansage dialog of the Nullspieler: the Null levels of this phase
    /// (before the skat Null Hand and Null Ouvert Hand, after it Null and
    /// Null Ouvert).
    fn null_declare_options(&self, content: &Content) -> DeclareOptions {
        let scoring = self.null_scoring(content);
        let after_skat = self.after_skat();
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
        let base_value = match &scoring {
            NullScoring::Fixed => crate::rules::null_value(false, false),
            NullScoring::Formula(f) => f.base_value,
        };
        DeclareOptions {
            games: vec![GameOption {
                game: GameType::Null,
                base_value,
                allowed: null_risks.iter().any(|r| r.allowed),
            }],
            risks: Vec::new(),
            null_risks,
            can_pick_up_skat: self.accepts(content, RunAction::PickUpSkat),
            ouvert_required: self.game.as_ref().is_some_and(|a| a.rules.require_ouvert),
            can_pass: self.accepts(content, RunAction::Pass),
            can_redeal: self.accepts(content, RunAction::Redeal),
            redeals_left: self.redeals_left(),
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
                let needs_target = option.choice == EncounterChoice::MarkChosenCard;
                let relic = match &option.choice {
                    EncounterChoice::TakeRule(id)
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
                let card_mod = option
                    .card_mod
                    .as_ref()
                    .filter(|_| marks)
                    .map(|id| Run::card_mod_info(content, id));
                let random_gain = match &option.choice {
                    EncounterChoice::SwapRelic { new, .. } => new.is_none(),
                    _ => marks && card_mod.is_none(),
                };
                let targets: Vec<u32> = if needs_target {
                    self.deck
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
                    random_gain,
                    can_choose,
                    needs_target,
                    targets,
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
        let mut hand: Vec<DealtCard> = held.to_vec();
        hand.sort_by_key(|c| display_key(order_game, c));
        let hand = hand
            .into_iter()
            .map(|card| CardView {
                legal: legal.contains(&card),
                from_skat: phase == TablePhase::Discarding && skat.contains(&card),
                discardable: discarding && discardable(held, card),
                ..self.card_view(card)
            })
            .collect();
        let trick = game
            .current_trick()
            .iter()
            .enumerate()
            .map(|(i, &card)| PlayedCard {
                seat: seat_at(game.trick_lead(), i),
                card: self.card_view(card),
            })
            .collect();
        let last_trick = game.tricks().last().map(|t| TrickView {
            cards: t
                .cards
                .iter()
                .enumerate()
                .map(|(i, &card)| PlayedCard {
                    seat: t.seat_of(i),
                    card: self.card_view(card),
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
            .then(|| skat.iter().map(|&c| self.card_view(c)).collect());
        let mut revealed: Vec<RevealedCard> = self
            .revealed_holders()
            .into_iter()
            .map(|(card, holder)| RevealedCard {
                source: RevealSource::CardMod(self.reveal_mod(content, card.id)),
                card: self.card_view(card),
                holder,
            })
            .collect();
        for &(card, _) in &active.peeked {
            if !revealed.iter().any(|r| r.card.card.id == card.id) {
                revealed.push(RevealedCard {
                    card: self.card_view(card),
                    holder: game.holder_of(card),
                    source: RevealSource::Kibitz,
                });
            }
        }
        let skat_swap_cards: Vec<CardView> = self
            .skat_swap_cards()
            .into_iter()
            .map(|c| self.card_view(c))
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
            boss_rule: active
                .boss_rule
                .as_ref()
                .map(|id| Run::boss_info(content, id)),
            count: self.count_view(),
            can_take_from_skat,
            skat_swap_cards,
            seat_tricks: game.seat_tricks(),
            value_preview: self.value_preview(content),
        })
    }

    /// The first modification on a deck card that shows its holder.
    fn reveal_mod(&self, content: &Content, card: u32) -> CardModId {
        let mods = self
            .deck
            .get(card)
            .map(|d| d.mods.as_slice())
            .unwrap_or(&[]);
        mods.iter()
            .find(|m| {
                content
                    .card_mod(m)
                    .is_some_and(|def| def.passive.contains(&crate::content::Effect::RevealHolder))
            })
            .or_else(|| mods.first())
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
        assert_eq!(options.games.len(), 5);
        assert!(options.games.iter().all(|g| g.allowed));
        assert!(options.risks.iter().all(|r| r.allowed));
        assert_eq!(options.games[0].base_value, 12);
        assert_eq!(options.games[4].base_value, 24);
        assert!(options.can_pass);
        assert!(options.can_pick_up_skat);
        assert!(!options.can_redeal);
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
                }
                let previewed = game.phase == TablePhase::Playing
                    && game.declaration.is_some_and(|d| !d.is_ramsch());
                assert_eq!(game.value_preview.is_some(), previewed);
                if let Some(p) = game.value_preview {
                    assert_eq!(p.game_value, p.base_value * p.levels * p.multiplier);
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
                    let affordable = offer.relic.price <= view.coins;
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
        assert!(stats.games_won + stats.games_lost + stats.games_passed > 0);
    }

    #[test]
    fn consumables_report_their_targets() {
        let content = content();
        let (mut run, _) = Run::new(&content, 5).expect("start");
        run.consumables = vec![ConsumableId::new("fresh_deck"), ConsumableId::new("beer")];
        let view = run.view(&content);
        // In a game: the beer works, the fresh deck only at the Tresen.
        assert!(!view.consumables[0].can_use);
        assert!(view.consumables[0].consumable.needs_target);
        assert!(view.consumables[1].can_use);
        assert!(!view.consumables[1].can_sell, "selling only at the Tresen");
        run.game = None;
        run.phase = RunPhase::Shop;
        run.shop = Some(run.draw_offer(&content, 0));
        let view = run.view(&content);
        assert!(view.consumables[0].can_use);
        assert_eq!(view.consumables[0].targets.len(), 32);
        assert!(!view.consumables[1].can_use);
        assert!(view.consumables[1].can_sell);
        assert_eq!(view.consumables[1].sell_price, 1);
        assert!(view.game.is_none());
        assert!(view.shop.is_some());
    }
}
