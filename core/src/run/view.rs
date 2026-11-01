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
    Kontra, Run, RunAction, RunPhase, RunStats, SERIES_PER_SECTION, sell_price, series_target,
};
use crate::cards::{Card, PlayingCard, Suit};
use crate::content::{
    BossId, CardModId, ConsumableAction, ConsumableId, Content, Factor, Rarity, RelicId, RelicKind,
    ScoreBreakdown, ScoreStep,
};
use crate::deck::{DealOdds, DealtCard, DeckCard};
use crate::game::{Phase, Position, Seat, seat_at};
use crate::rules::{Declaration, GameType, RiskLevel, suit_strength, trump_strength};

/// Everything a frontend shows of a run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RunView {
    /// The seed as decimal text (JavaScript numbers cannot hold every u64).
    pub seed: String,
    /// What the run waits for.
    pub phase: RunPhase,
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
    /// The boss of a boss series.
    pub boss: Option<BossInfo>,
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
    /// Whether it can be bought now.
    pub can_buy: bool,
}

/// Phase of the game in progress.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TablePhase {
    /// Ansage.
    Declaring,
    /// Two cards to discard.
    Discarding,
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

/// A card marked "Durchsichtig" and who holds it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RevealedCard {
    /// The card.
    pub card: CardView,
    /// Holder; `None` if in the skat or already played.
    pub holder: Option<Seat>,
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

/// What the player may do before the declaration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DeclareOptions {
    /// Spielarten.
    pub games: Vec<GameOption>,
    /// Risikostufen.
    pub risks: Vec<RiskOption>,
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
    /// The skat as the player knows it: the discards after discarding,
    /// `None` in hand games and before.
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
    /// Ansage options while declaring.
    pub declare: Option<DeclareOptions>,
    /// "Durchsichtig" cards and their holders.
    pub revealed: Vec<RevealedCard>,
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
/// "Gezinkt" card stays in its place.
fn display_key(game: GameType, card: &DealtCard) -> (u8, u8, u32) {
    let Some(face) = card.face() else {
        return (9, 0, card.id);
    };
    if let Some(strength) = trump_strength(game, face) {
        return (0, u8::MAX - strength, card.id);
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
        let (section, index) = if self.series.index + 1 >= SERIES_PER_SECTION {
            (self.series.section + 1, 0)
        } else {
            (self.series.section, self.series.index + 1)
        };
        let is_boss = index + 1 == SERIES_PER_SECTION;
        let boss = if is_boss {
            self.section_boss.as_ref()
        } else {
            None
        };
        let target = series_target(content, section, index, boss)?;
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
            RunPhase::Shop => series.index + 1 < SERIES_PER_SECTION,
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
        RunView {
            seed: self.seed.to_string(),
            phase: self.phase,
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
                    id.as_ref().map(|id| RelicOffer {
                        slot,
                        relic: Run::relic_info(content, id),
                        can_buy: self.accepts(content, RunAction::BuyRelic(slot)),
                    })
                })
                .collect(),
            consumables: shop
                .consumables
                .iter()
                .enumerate()
                .map(|(slot, id)| {
                    id.as_ref().map(|id| ConsumableOffer {
                        slot,
                        consumable: Run::consumable_info(content, id),
                        can_buy: self.accepts(content, RunAction::BuyConsumable(slot)),
                    })
                })
                .collect(),
            reroll_cost: content.run.economy.reroll_cost,
            can_reroll: self.accepts(content, RunAction::Reroll),
            rerolls: shop.rerolls,
        })
    }

    fn declare_options(&self, content: &Content) -> DeclareOptions {
        let risks: Vec<RiskOption> = RiskLevel::ALL
            .into_iter()
            .map(|risk| RiskOption {
                risk,
                allowed: self.accepts(
                    content,
                    RunAction::Declare(Declaration::new(GameType::Grand, risk)),
                ),
            })
            .collect();
        let lowest = risks
            .iter()
            .find(|r| r.allowed)
            .map_or(RiskLevel::Normal, |r| r.risk);
        let games = GAMES
            .into_iter()
            .map(|game| GameOption {
                game,
                base_value: game.base_value().unwrap_or(0),
                allowed: self.accepts(content, RunAction::Declare(Declaration::new(game, lowest))),
            })
            .collect();
        DeclareOptions {
            games,
            risks,
            can_pass: self.accepts(content, RunAction::Pass),
            can_redeal: self.accepts(content, RunAction::Redeal),
            redeals_left: self.redeals_left(),
            hand_required: self.requires_hand(),
        }
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
            Phase::Playing => TablePhase::Playing,
            Phase::Finished(_) => TablePhase::Finished,
        };
        let order_game = declaration.map_or(GameType::Grand, |d| d.game);
        let legal = if active.awaiting_re {
            Vec::new()
        } else {
            game.legal_moves(Seat::DECLARER)
        };
        let skat = game.skat();
        let mut hand: Vec<DealtCard> = game.hand(Seat::DECLARER).to_vec();
        hand.sort_by_key(|c| display_key(order_game, c));
        let hand = hand
            .into_iter()
            .map(|card| CardView {
                legal: legal.contains(&card),
                from_skat: phase == TablePhase::Discarding && skat.contains(&card),
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
        let known_skat = match (declaration, phase) {
            (Some(d), TablePhase::Playing | TablePhase::Finished) if !d.hand() => {
                Some(skat.iter().map(|&c| self.card_view(c)).collect())
            }
            _ => None,
        };
        let revealed = self
            .revealed_holders()
            .into_iter()
            .map(|(card, holder)| RevealedCard {
                card: self.card_view(card),
                holder,
            })
            .collect();
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
            declare: (phase == TablePhase::Declaring).then(|| self.declare_options(content)),
            revealed,
        })
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
        assert!(!options.can_redeal);
        assert!(!options.hand_required);
    }

    #[test]
    fn view_agrees_with_the_run_through_a_whole_run() {
        let content = content();
        let (mut run, _) = Run::new(&content, 21).expect("start");
        let mut saw_discard = false;
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
