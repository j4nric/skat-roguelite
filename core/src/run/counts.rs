//! Counting the player's edition for conditions and effects: cursed
//! cards, cards that match a [`crate::content::CardCount`], void suits,
//! marked cards, and how a Null is scored with the relics held.

use std::collections::BTreeSet;

use crate::cards::{PlayingCard, Suit};
use crate::content::{CardModDef, Content, Counted, Effect};
use crate::game::Game;
use crate::rules::{Declaration, GameType, NullScoring};

use super::{Run, player_cards};

impl Run {
    /// How many cards of the player's edition carry a curse now (a special
    /// curse such as "Abgestuft" and a curse laid for one pub included):
    /// what "Pechvogel" counts ([`Effect::AddBaseValuePerCursedCard`]).
    #[must_use]
    pub fn cursed_cards(&self, content: &Content) -> u32 {
        let count = self
            .edition
            .cards()
            .iter()
            .filter(|c| self.card_is_cursed(content, c.id))
            .count();
        u32::try_from(count).unwrap_or(u32::MAX)
    }

    /// What an effect that counts cards counts now: [`Run::cursed_cards`]
    /// for `AddBaseValuePerCursedCard`, [`Run::count_cards`] for
    /// `AddBaseValuePerCard` and `AddLevelsPerCard`; 0 for every other
    /// effect (not counted).
    pub(super) fn count_for(&self, content: &Content, effect: &Effect) -> u32 {
        match effect.counted() {
            Some(Counted::CursedCards) => self.cursed_cards(content),
            Some(Counted::Cards(count)) => self.count_cards(content, count),
            Some(Counted::VoidSuits) => self.void_suits(),
            Some(Counted::DeclarerPointsAbove { above, step }) => {
                let points = self
                    .game
                    .as_ref()
                    .map_or(0, |a| a.game.outcome().declarer_card_points);
                points.saturating_sub(above) / step.max(1)
            }
            Some(Counted::FreeRelicSlots { every }) => {
                self.free_relic_slots(content) / every.max(1)
            }
            None => 0,
        }
    }

    /// The suits besides the trump suit of the declared suit game of which
    /// the player holds no card in the ten he plays ("Kahlschlag"): his hand
    /// and the cards he played, as he holds them (repainted by "Blue
    /// Curaçao", a settled Wandelkarte). Trumps (the Unter, "Bube
    /// ehrenhalber") and blanks belong to no suit. 0 outside a suit game.
    #[must_use]
    pub fn void_suits(&self) -> u32 {
        let Some(game) = self.game.as_ref().map(|a| &a.game) else {
            return 0;
        };
        let Some(GameType::Suit(trump)) = game
            .declaration()
            .filter(|d| !d.is_ramsch())
            .map(Declaration::game)
        else {
            return 0;
        };
        let game_type = GameType::Suit(trump);
        let held = player_cards(game);
        let n = Suit::ALL
            .iter()
            .filter(|&&suit| suit != trump)
            .filter(|&&suit| {
                !held.iter().any(|c| {
                    !crate::rules::is_trump(game_type, *c)
                        && c.face().is_some_and(|f| f.suit == suit)
                })
            })
            .count();
        u32::try_from(n).unwrap_or(0)
    }

    /// How many cards of the player's edition `count` counts now (the
    /// connector relics of the build directions, 04.11.2026): those that
    /// carry a matching modification, narrowed to the trump suit of the
    /// declared suit game and to the cards he holds in the game in
    /// progress if asked.
    #[must_use]
    pub fn count_cards(&self, content: &Content, count: &crate::content::CardCount) -> u32 {
        let game = self.game.as_ref().map(|a| &a.game);
        // The build directions of the relics the player holds.
        let directions: Vec<crate::content::BuildTag> = if count.direction {
            self.held_relic_defs(content)
                .flat_map(|r| r.tags.iter().copied())
                .collect()
        } else {
            Vec::new()
        };
        let trump = game
            .and_then(Game::declaration)
            .filter(|d| !d.is_ramsch())
            .and_then(|d| match d.game() {
                GameType::Suit(suit) => Some(suit),
                _ => None,
            });
        let held = game.map(player_cards).unwrap_or_default();
        let n = self
            .edition
            .cards()
            .iter()
            .filter(|c| {
                // With `in_hand` the card as the player holds it.
                let face = if count.in_hand {
                    let Some(copy) = held.iter().find(|h| h.id == c.id) else {
                        return false;
                    };
                    copy.face()
                } else {
                    c.kind.face()
                };
                if count.trump_suit {
                    let Some(suit) = trump else {
                        return false;
                    };
                    if face.is_none_or(|f| f.suit != suit) {
                        return false;
                    }
                }
                if count.off_trump {
                    let Some(suit) = trump else {
                        return false;
                    };
                    if face.is_none_or(|f| f.suit == suit || f.is_jack()) {
                        return false;
                    }
                }
                if count.no_jacks && face.is_some_and(|f| f.is_jack()) {
                    return false;
                }
                if count.all_cards {
                    return true;
                }
                c.mods
                    .iter()
                    .filter_map(|m| content.card_mod(m))
                    .any(|def| {
                        let named = if count.mods.is_empty() {
                            !def.is_curse()
                        } else {
                            count.mods.contains(&def.id)
                        };
                        named
                            && count.kind.is_none_or(|k| def.kind == k)
                            && (!count.direction || def.tags.iter().any(|t| directions.contains(t)))
                    })
            })
            .count();
        u32::try_from(n).unwrap_or(u32::MAX)
    }

    /// Ids of the cards of the player's edition that carry a Zinke: any
    /// modification that is no curse
    /// ([`crate::content::Condition::TrickWonWithMarkedCard`]).
    #[must_use]
    pub fn marked_cards(&self, content: &Content) -> BTreeSet<u32> {
        self.edition
            .cards()
            .iter()
            .filter(|c| {
                c.mods
                    .iter()
                    .filter_map(|m| content.card_mod(m))
                    .any(|def| !def.is_curse())
            })
            .map(|c| c.id)
            .collect()
    }

    /// Whether a card of the player's edition carries a curse (a special
    /// curse such as "Abgestuft" included).
    #[must_use]
    pub fn card_is_cursed(&self, content: &Content, card: u32) -> bool {
        self.edition.get(card).is_some_and(|c| {
            c.mods
                .iter()
                .any(|m| content.card_mod(m).is_some_and(CardModDef::is_curse))
        })
    }

    /// How the character's Null games are scored (Fixed if the data says
    /// nothing), with the Kurzschluss lowered by
    /// [`Effect::ShortCircuitThreshold`] ("Wackelkontakt": the lowest
    /// value of the character's rule, the relics and the evening rules).
    pub(super) fn null_scoring(&self, content: &Content) -> NullScoring {
        let mut scoring = content
            .character(self.character)
            .and_then(|c| c.null_scoring.clone())
            .unwrap_or(NullScoring::Fixed);
        if let NullScoring::Formula(formula) = &mut scoring {
            let rule = content
                .character(self.character)
                .map(|c| c.rule.passive.as_slice())
                .unwrap_or_default();
            let relics = self.held_passives(content);
            for effect in rule.iter().chain(relics) {
                if let Effect::ShortCircuitThreshold(points) = effect {
                    formula.short_circuit_points = formula.short_circuit_points.min(*points);
                }
            }
        }
        scoring
    }
}
