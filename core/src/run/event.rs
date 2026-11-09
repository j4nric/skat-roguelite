//! What happened: [`RunEvent`], the output of [`Run::apply`].

use serde::{Deserialize, Serialize};

use crate::content::{
    BossId, CardModId, Character, ConsumableId, EncounterId, RelicId, RunLength, ScoreBreakdown,
    Source,
};
use crate::deck::{DealtCard, DeckChange};
use crate::game::{Event, Seat};

use super::{EncounterOption, Payout, card_reward, is_zero, tour, trade, zinken};

/// Something that happened in the run, in order of occurrence.
///
/// `G` is the type of the game events: the full [`Event`] for the run
/// itself and the simulator, [`player::PlayerGameEvent`] for what the
/// player may see (see [`player::PlayerEvent`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunEvent<G = Event<DealtCard>> {
    /// A new section (Skatabend) began and its boss was drawn. The boss is
    /// known from here on, so the player can prepare for it (DESIGN 11).
    BossAnnounced {
        /// Section, from 0.
        section: u32,
        /// The boss of this section's boss series.
        boss: BossId,
    },
    /// A new pub started (the event keeps its name from the series).
    SeriesStarted {
        /// Section, from 0.
        section: u32,
        /// Series within the section (2 = boss series).
        index: u32,
        /// Zielwert.
        target: u64,
        /// Spiellimit.
        game_limit: u32,
        /// Einmischen per game of the character, without relics. Also read
        /// under its old name `passes` (passes per series).
        #[serde(alias = "passes")]
        redeals: u32,
        /// Boss of a boss series.
        boss: Option<BossId>,
        /// The malus of a Spelunke.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        malus: Option<crate::content::MalusId>,
    },
    /// A game was dealt.
    GameDealt {
        /// Number of the game in the series, from 1.
        number: u32,
        /// Seat that leads.
        forehand: Seat,
    },
    /// An event of the game in progress.
    Game(G),
    /// "Untot" (DESIGN 7.2): the player's card rises from the trick just
    /// completed as soon as he picks `pick` cards of his hand
    /// ([`RunAction::ChooseRiseCards`]); one of them is drawn and takes its
    /// place. Then [`Event::CardRose`] follows.
    RiseChoice {
        /// The card as it lies in the trick.
        card: DealtCard,
        /// The card it comes back as: a copy of the trick's winning card.
        becomes: DealtCard,
        /// How many cards to pick.
        pick: usize,
    },
    /// A defender said Kontra.
    Kontra {
        /// Who said it.
        seat: Seat,
    },
    /// The declarer answered Re.
    Re,
    /// A flag was set by content.
    FlagSet(String),
    /// A flag was cleared by content.
    FlagCleared(String),
    /// A counter was raised by content.
    CounterRaised {
        /// Counter name.
        counter: String,
        /// New value.
        value: u32,
    },
    /// A counter went back to zero ("Altes Blatt" after a won series).
    CounterReset {
        /// Counter name.
        counter: String,
    },
    /// Content made the discarded skat count more Augen for the declarer
    /// ("Skatpfleger").
    SkatPointsRaised {
        /// Who did it.
        source: Source,
        /// Augen the declarer counts more (fewer if negative).
        card_points: i32,
    },
    /// A card effect gave a seat extra Augen ("Erster Stich": the Augen of
    /// the first trick once more, DESIGN 7.2).
    BonusPoints {
        /// Who did it.
        source: Source,
        /// The seat that gets them.
        seat: Seat,
        /// Extra Augen.
        card_points: u32,
    },
    /// Content paid (or took) Groschen (a curse when attached, a voucher).
    CoinsGained {
        /// Who paid.
        source: Source,
        /// Groschen gained; negative when taken.
        amount: i32,
        /// Groschen after the change.
        coins: u32,
    },
    /// A finished game was scored.
    GameScored {
        /// Every term and modification.
        breakdown: ScoreBreakdown,
        /// Series account after this game.
        series_score: i64,
        /// Augen of the cards the player discarded into the skat (with
        /// their modifications); 0 in a hand game.
        #[serde(default, skip_serializing_if = "is_zero")]
        discarded_points: u32,
    },
    /// The series was won.
    SeriesWon {
        /// Section.
        section: u32,
        /// Series within the section.
        index: u32,
        /// Groschen earned.
        payout: Payout,
        /// Groschen after the payout.
        coins: u32,
        /// The boss of a boss series.
        #[serde(default)]
        boss: Option<BossId>,
        /// Einmischen in the series. Also read under its old name
        /// `passes_used`.
        #[serde(default, alias = "passes_used")]
        redeals_used: u32,
        /// Games left when the series was won.
        #[serde(default)]
        games_left: u32,
    },
    /// A Kiosk opened on the way (03.11.2026).
    KioskOpened {
        /// Schnäpse on offer.
        consumables: Vec<Option<ConsumableId>>,
    },
    /// The Tresen opened.
    ShopOpened {
        /// Relics on offer.
        relics: Vec<Option<RelicId>>,
        /// Consumables on offer.
        consumables: Vec<Option<ConsumableId>>,
        /// The slot of the relic offered as "Hehlerware", if any.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fence: Option<usize>,
    },
    /// A relic was bought.
    RelicBought {
        /// The relic.
        relic: RelicId,
        /// Price paid.
        price: u32,
    },
    /// A consumable was bought.
    ConsumableBought {
        /// The consumable.
        consumable: ConsumableId,
        /// Price paid.
        price: u32,
    },
    /// A relic was sold.
    RelicSold {
        /// The relic.
        relic: RelicId,
        /// Groschen received.
        price: u32,
    },
    /// A consumable was sold.
    ConsumableSold {
        /// The consumable.
        consumable: ConsumableId,
        /// Groschen received.
        price: u32,
    },
    /// "Neu auslosen": a new offer of relics and consumables was drawn
    /// (`ShopOpened` follows), then the Auslage (`CardOffersDrawn`).
    ShopRerolled {
        /// Price paid.
        cost: u32,
    },
    /// The Auslage was drawn (Zinken, DESIGN 8.6): when the Tresen opens
    /// and after "Neu auslosen" ([`RunEvent::ShopRerolled`]).
    CardOffersDrawn {
        /// The offers, by slot.
        offers: Vec<Option<zinken::CardOffer>>,
    },
    /// An offer of the Auslage was bought. The `DeckChanged` that follows
    /// shows the card.
    CardOfferBought {
        /// Slot of the Auslage.
        slot: usize,
        /// The card (instance id).
        card: u32,
        /// The modification.
        card_mod: CardModId,
        /// Its value.
        value: i32,
        /// Price paid.
        price: u32,
    },
    /// The Kartenmaler took a card and drew the modifications to choose
    /// from (at the Tresen, or free at the Werkstatt).
    PainterHired {
        /// The card (instance id).
        card: u32,
        /// The modifications with their values.
        options: Vec<(CardModId, i32)>,
        /// Price paid.
        price: u32,
    },
    /// The player took a modification of the Kartenmaler. The
    /// `DeckChanged` that follows shows the card.
    PainterChosen {
        /// The card (instance id).
        card: u32,
        /// The modification.
        card_mod: CardModId,
        /// Its value.
        value: i32,
    },
    /// A trade put a permanent curse on a card of the player's edition
    /// (curses as trade, DESIGN 8.6): a "Schnäppchen" of the Auslage, a
    /// relic of the "Hehlerware", the Schnaps "Fusel". The `DeckChanged`
    /// that follows shows the card; a curse that pays when attached says
    /// so after it.
    TradeCurse {
        /// The card (instance id).
        card: u32,
        /// The curse.
        curse: CardModId,
        /// What the curse was the price of.
        cause: trade::CurseCause,
    },
    /// "Karte reinigen" at the Tresen was paid. The `DeckChanged` that
    /// follows shows the card.
    CardCleansed {
        /// The card (instance id).
        card: u32,
        /// Price paid.
        price: u32,
    },
    /// The player left the Tresen.
    ShopLeft,
    /// A consumable was used.
    ConsumableUsed {
        /// The consumable.
        consumable: ConsumableId,
    },
    /// The deck changed.
    DeckChanged(DeckChange),
    /// Einmischen were added to the series ("Ein Bier", "Kräuter"). Also
    /// read under its old name `PassesAdded`.
    #[serde(alias = "PassesAdded")]
    RedealsAdded {
        /// Einmischen added.
        #[serde(alias = "passes")]
        redeals: u32,
        /// Einmischen left now ([`Run::redeals_left`]).
        #[serde(alias = "passes_left")]
        redeals_left: u32,
    },
    /// The game was thrown in and dealt again: Einmischen, or the
    /// consumable "Neues Blatt".
    Redealt,
    /// "Reue": the last Einmischen was taken back; the deal before it is
    /// on the table again.
    RedealUndone,
    /// A voucher was redeemed: its card went into the skat by discarding
    /// ("wenn gedrückt", DESIGN 7.2). Its effects follow as their own
    /// events (`CoinsGained`, `DeckChanged`), then it falls off the card.
    VoucherRedeemed {
        /// Instance id of the card.
        card: u32,
        /// The voucher.
        card_mod: CardModId,
        /// Its value.
        value: i32,
        /// Redeemed by declaring a hand game (decided on 04.11.2026,
        /// DESIGN 7.2): the card stays in the hand. Left out of the JSON
        /// when `false`.
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        via_hand: bool,
    },
    /// "Der Drückeberger" (DESIGN 9.4): a voucher redeemed in the pub
    /// that just ended is back on its card.
    VoucherReturned {
        /// Instance id of the card.
        card: u32,
        /// The voucher.
        card_mod: CardModId,
        /// Its value.
        value: i32,
    },
    /// An encounter put a Marke on a card of the player's edition
    /// ([`EncounterEffect::Marke`], 04.11.2026); followed by the
    /// `DeckChanged` of the card.
    MarkeGiven {
        /// Deck card id.
        card: u32,
        /// The Marke.
        card_mod: CardModId,
        /// Whether the Marke was drawn (not chosen).
        drawn: bool,
        /// Whether the card was drawn (not chosen).
        random_card: bool,
    },
    /// "Glasauge": a defender's card is "Durchsichtig" for this game; the
    /// player sees it with its holder until it is played.
    CardMadeTransparent {
        /// Who granted it.
        source: Source,
        /// The card, in the defender's edition.
        card: DealtCard,
        /// Who holds it.
        seat: Seat,
    },
    /// "Der Kiebitz" showed the player a defender's card.
    CardPeeked {
        /// The card.
        card: DealtCard,
        /// Who holds it.
        seat: Seat,
    },
    /// The game is the Ramsch of the series ("Der Ramscher"); nobody
    /// declares, it is played at once.
    RamschDealt,
    /// An encounter opened instead of the Tresen.
    EncounterOpened {
        /// Which encounter.
        encounter: EncounterId,
        /// Its options.
        options: Vec<EncounterOption>,
    },
    /// The player chose an option of the encounter.
    EncounterResolved {
        /// Which encounter.
        encounter: EncounterId,
        /// Index of the option taken.
        option: usize,
    },
    /// The dice fell at the Würfelbude.
    DiceThrown {
        /// Groschen staked (already paid).
        stake: u32,
        /// Won.
        won: bool,
        /// Groschen paid out (0 if lost).
        payout: u32,
    },
    /// The player will see both skat cards in the next game.
    SkatPeekPlanned,
    /// A house rule applies until the end of the section.
    EveningRuleAdded(RelicId),
    /// The house rules of the evening ended with the section.
    EveningRulesEnded,
    /// An owned relic was swapped for another.
    RelicSwapped {
        /// The relic given away.
        old: RelicId,
        /// The relic received.
        new: RelicId,
    },
    /// The next game declared counts `factor` times ("Letzte Runde").
    NextGameDoubled {
        /// Factor on the Spielwert.
        factor: u32,
    },
    /// The player looked at the untouched skat ("Kurzer").
    SkatPeeked {
        /// The two skat cards.
        skat: [DealtCard; 2],
    },
    /// After a hand declaration the player was shown a card of the
    /// untouched skat ("Spickzettel").
    SkatCardShown {
        /// Who showed it.
        source: Source,
        /// The skat card.
        card: DealtCard,
    },
    /// The route choice to the next pub opened (ADR 0016).
    RouteOpened {
        /// District of the next pub.
        section: u32,
        /// The next pub within it.
        index: u32,
    },
    /// The player took a route.
    RouteChosen {
        /// Index of the route.
        option: usize,
        /// How he goes.
        transport: tour::Transport,
        /// Whether it leads into a Spelunke.
        spelunke: bool,
    },
    /// A pub ended below its target after its last game (ADR 0016).
    PubLost {
        /// District.
        section: u32,
        /// Pub within the district.
        index: u32,
        /// The boss of a boss pub (it is played again).
        #[serde(default)]
        boss: Option<BossId>,
    },
    /// The player gave up the pub ("Kneipe verlassen",
    /// [`RunAction::GiveUpPub`]). It counts as lost: `PubLost` and what a
    /// lost pub brings follow (the end of the run, or the second chance of
    /// the "Schutzengel").
    PubGivenUp {
        /// District.
        section: u32,
        /// Pub within the district.
        index: u32,
        /// Account when given up.
        score: i64,
    },
    /// A pub account below 0 was set to 0 ("Klarer").
    AccountCleared {
        /// Account before.
        before: i64,
        /// Account now (0).
        score: i64,
    },
    /// The Spiellimit of the pub rose ("Absacker").
    GamesAdded {
        /// Games added.
        games: u32,
        /// Spiellimit now.
        game_limit: u32,
        /// Games left now.
        games_left: u32,
    },
    /// A relic was gained without paying (loot of a Spelunke).
    RelicGained {
        /// The relic.
        relic: RelicId,
    },
    /// The Groschen owed at the Tresen changed ("Deckel anschreiben"):
    /// credit taken, or paid back from income.
    DebtChanged {
        /// Groschen owed now.
        debt: u32,
    },
    /// Second chance (ADR 0019): the "Schutzengel" has the lost pub
    /// played again from the start and is used up; `SeriesStarted` of the
    /// same pub follows.
    SecondChance {
        /// The relic used up.
        relic: RelicId,
        /// District of the pub played again.
        section: u32,
        /// Pub within the district.
        index: u32,
    },
    /// Content put Groschen into a pot ("Stammtischkasse").
    CoinsStashed {
        /// Who did it.
        source: Source,
        /// Groschen moved.
        amount: u32,
        /// Groschen in the pot now.
        pot: u32,
        /// Groschen left to the player.
        coins: u32,
    },
    /// A relic that stashes Groschen ("Stammtischkasse") left the player
    /// (sold, swapped, replaced or an evening rule ended): its pot was paid
    /// out first, as it was, and emptied (DESIGN 8.7).
    StashPaidOut {
        /// The relic that left.
        relic: RelicId,
        /// Groschen paid out.
        amount: u32,
        /// Groschen of the player now.
        coins: u32,
    },
    /// The stops of the open route choice were drawn again ("Schwenker").
    RoutesRerolled,
    /// The plan for the next pub changed ("Wegbier", "Der Skatlehrer",
    /// "Kiebitz am Fenster"); the whole plan now.
    NextPubPlanned {
        /// Einmischen more (fewer if negative).
        redeals: i32,
        /// Change of the Zielwert.
        target: i32,
    },
    /// The plan for the next pub took effect as it started.
    NextPubApplied {
        /// Einmischen more (fewer if negative: in the first game).
        redeals: i32,
        /// Change of the Zielwert.
        target: i32,
    },
    /// A bet on the next declared game was placed ("Die Wette").
    WagerPlaced {
        /// Groschen staked.
        stake: u32,
    },
    /// The bet was decided by a scored game.
    WagerSettled {
        /// Groschen staked.
        stake: u32,
        /// Won with Schneider.
        won: bool,
        /// Groschen paid out (0 if lost).
        payout: u32,
    },
    /// The exact stops of the next route choices are shown ("Kiebitz am
    /// Fenster").
    StopsRevealed {
        /// Route choices still shown.
        legs: u32,
    },
    /// The malus of a Spelunke was chosen ("Spelunkenkenner").
    MalusChosen {
        /// Index of the route.
        option: usize,
        /// The malus.
        malus: crate::content::MalusId,
    },
    /// "Falschspieler am Nebentisch": whether the trick was a curse.
    GambleResolved {
        /// A curse landed instead of the offer.
        curse: bool,
    },
    /// A consumable was gained without paying ("Der Wirt spendiert").
    ConsumableGained {
        /// The consumable.
        consumable: ConsumableId,
    },
    /// The run was won.
    RunWon {
        /// Who won it.
        #[serde(default)]
        character: Character,
        /// Ganzer or kurzer Abend.
        #[serde(default)]
        length: RunLength,
        /// Stake of the run (Einsatz-Leiter).
        #[serde(default)]
        stake: u32,
    },
    /// The run was lost: a pub was lost (ADR 0019).
    RunLost {
        /// District of the pub that was lost.
        section: u32,
        /// Pub within the district.
        index: u32,
        /// Who lost it.
        #[serde(default)]
        character: Character,
        /// Ganzer or kurzer Abend.
        #[serde(default)]
        length: RunLength,
        /// Stake of the run (Einsatz-Leiter).
        #[serde(default)]
        stake: u32,
        /// The boss whose pub was lost (ADR 0017). `None` for a normal pub
        /// or a Spelunke.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        boss: Option<BossId>,
        /// Lost in the long night (ADR 0020): the run was won already and
        /// stays won; only the night ends.
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        night: bool,
    },
    /// The player continued into the long night after der Alte (ADR
    /// 0020).
    NightStarted {
        /// Who plays it.
        character: Character,
    },
    /// A Verschärfung of the long night was drawn for the night district
    /// ahead; it stays for the rest of the night (ADR 0020).
    VerschaerfungDrawn {
        /// The night district it is drawn for, from 0 (3 = Stadtteil 4).
        section: u32,
        /// The Verschärfung.
        verschaerfung: crate::content::VerschaerfungId,
    },
    /// "Doppelte Bosse": further boss rules of a night boss pub, known
    /// from the start of the district like its boss.
    ExtraBossRules {
        /// The night district, from 0.
        section: u32,
        /// The further bosses whose rules apply too.
        bosses: Vec<BossId>,
    },
    /// "Verhexte Karte": cards of the player's edition carry a curse until
    /// the pub ends.
    NightCursesLaid {
        /// The cards (ids) with their curse, in id order.
        cards: Vec<(u32, CardModId)>,
    },
    /// "Die Oma" (ADR 0017): cards of the player's edition carry a curse
    /// until the pub ends; each in place of its normal modification.
    BossCursesLaid {
        /// The boss whose rule laid them.
        boss: BossId,
        /// The cards (ids) with their curse, in id order.
        cards: Vec<(u32, CardModId)>,
    },
    /// The pub ended: the cards a boss cursed are as before again.
    BossCursesLifted {
        /// The cards (ids), in id order.
        cards: Vec<u32>,
    },
    /// A won boss pub (not the last): relics to pick one from (ADR 0017),
    /// see [`BossReward`].
    BossRewardOffered {
        /// The relics on offer.
        relics: Vec<RelicId>,
    },
    /// The relic pick of a won boss pub was passed without a relic.
    BossRewardDeclined,
    /// A won pub gave a random Schnaps (DESIGN 8.9). With a free slot it
    /// is in the slots now; with every slot full the player chooses
    /// ([`RunAction::ChooseSchnappsReward`]).
    SchnappsRewarded {
        /// The Schnaps.
        consumable: ConsumableId,
        /// Every slot was full: the choice is open.
        slots_full: bool,
    },
    /// The Schnaps of a won pub took the slot of an owned one, which is
    /// gone. Sold instead: [`RunEvent::ConsumableSold`].
    SchnappsRewardTaken {
        /// The new Schnaps.
        consumable: ConsumableId,
        /// The Schnaps given up for it.
        given_up: ConsumableId,
    },
    /// A won normal pub or Spelunke: the innkeeper offers these cards
    /// with a Zinke each (DESIGN 5.0, [`RunPhase::CardReward`]).
    CardRewardOffered {
        /// The offers, in order.
        offers: Vec<card_reward::RewardOffer>,
    },
    /// A Zinke of the card reward was taken; [`RunEvent::DeckChanged`]
    /// follows.
    CardRewardTaken {
        /// Index into the offers.
        offer: usize,
        /// The deck card (instance id).
        card: u32,
        /// The Zinke.
        card_mod: CardModId,
        /// Its value.
        value: i32,
    },
    /// The card reward was skipped for Groschen.
    CardRewardSkipped {
        /// Groschen paid.
        coins: u32,
    },
}
