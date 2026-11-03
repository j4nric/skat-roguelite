//! The Kneipentour (ADR 0016, DESIGN 5): lives, the route choice between
//! two pubs, the Spelunke and the city's names.
//!
//! After every pub that does not end the run (won, or lost with a life
//! left) the player chooses one of three ways to the next pub
//! ([`RouteChoice`]), one per [`Transport`] (Zu Fuß, Bahn, Fahrrad). All
//! three behave the same (decision of the user, 02.11.2026): each leads
//! over exactly one stop whose category is rolled per way and shown;
//! nothing costs extra. Since 03.11.2026 the three show three different
//! categories, one of them the Tresen (Begegnung, Upgrade, Rast,
//! Fundgroschen, Kiosk and Würfelbude by weight for the others). They
//! differ only in the way drawn on the map and the rolled category.
//!
//! Sometimes one of the three leads into a Spelunke instead of the next
//! pub: another place with its own name and node (`d0-p2-s`), standing in
//! for that pub in the district's order, with a higher target, a malus
//! shown in advance and a relic as extra reward. A lost boss pub ends the
//! run (ADR 0017); a won one pays double, gives a life and a pick of one of
//! three relics ([`BossReward`]). Everything is drawn from the run's own route generator, so it
//! never changes the deals.
//!
//! The view ([`TourView`]) names every pub of the city and gives stable
//! node ids and map seeds, so that a frontend can draw a street map
//! deterministically; the core knows no geometry.

use serde::{Deserialize, Serialize};

use super::guests::{GuestInfo, seat_guests};
use super::view::BossInfo;
use super::{
    PUBS_PER_DISTRICT, Run, RunError, RunEvent, RunPhase, Shop, draw_weighted, sell_price,
    series_target,
};
use crate::content::{
    CityDistrict, Content, DistrictLook, EncounterId, MalusDef, MalusId, RelicId, StopCategory,
    in_pool,
};
use crate::deck::{CardKind, DeckChange};

/// Salt of the city's names ("CITY").
const CITY_STREAM: u64 = 0x4349_5459_0000_0000;

/// How the player goes to the next pub (Verkehrsmittel). Presentation
/// and flavour only: every way has one stop and costs nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Transport {
    /// Zu Fuß.
    Walk,
    /// Bahn.
    Train,
    /// Fahrrad.
    Bike,
}

impl Transport {
    /// The three in the order of the choice.
    pub const ALL: [Transport; 3] = [Transport::Walk, Transport::Train, Transport::Bike];
}

/// A stop on a route: the Tresen or an encounter of a category, drawn
/// when the route choice opens.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteStop {
    /// What kind of stop.
    pub category: StopCategory,
    /// The encounter; `None` for the Tresen.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encounter: Option<EncounterId>,
}

/// A Spelunke (elite pub): replaces a normal pub (ADR 0016).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spelunke {
    /// Its name.
    pub name: String,
    /// The malus of this pub.
    pub malus: MalusId,
    /// The maluses on offer ("Spelunkenkenner"); `malus` is one of them.
    /// Empty without a choice.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub malus_options: Vec<MalusId>,
}

/// One way to the next pub.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteOption {
    /// How the player goes.
    pub transport: Transport,
    /// The stops, in order (since 02.11.2026 always exactly one).
    pub stops: Vec<RouteStop>,
    /// The Spelunke it leads into instead of the next pub.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spelunke: Option<Spelunke>,
}

/// The open route choice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RouteChoice {
    /// District of the next pub.
    pub section: u32,
    /// The next pub within it.
    pub index: u32,
    /// The ways, one per [`Transport`].
    pub options: Vec<RouteOption>,
}

/// The route taken, while on the way.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trip {
    /// District of the pub it leads to.
    pub section: u32,
    /// The pub within it.
    pub index: u32,
    /// How the player goes.
    pub transport: Transport,
    /// Stops still ahead, the next first.
    pub stops: Vec<RouteStop>,
    /// The Spelunke it leads into.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spelunke: Option<Spelunke>,
}

/// How a pub went.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PubResult {
    /// District.
    pub section: u32,
    /// Pub within it.
    pub index: u32,
    /// Won or lost.
    pub won: bool,
    /// The name of the Spelunke, if one was played in its place.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spelunke: Option<String>,
}

/// The relic pick of a won boss pub (ADR 0017): one of these relics, drawn
/// when the pub was won, or none.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BossReward {
    /// The relics on offer (rarity Uncommon or higher, `boss_reward` in
    /// `run.ron`).
    pub relics: Vec<RelicId>,
}

impl Run {
    /// The open relic pick of a won boss pub.
    #[must_use]
    pub const fn boss_reward(&self) -> Option<&BossReward> {
        self.boss_reward.as_ref()
    }

    /// Open the relic pick after a won boss pub: relics the run may draw,
    /// of the configured rarity or higher, drawn with the encounter
    /// generator. Returns `false` if none is left; the way goes on then.
    fn open_boss_reward(&mut self, content: &Content, events: &mut Vec<RunEvent>) -> bool {
        let config = &content.run.boss_reward;
        let pool: Vec<(RelicId, u32)> = self
            .relic_pool(content, false)
            .into_iter()
            .filter(|(id, _)| {
                content
                    .relic(id)
                    .is_some_and(|r| r.rarity >= config.min_rarity)
            })
            .collect();
        let relics = draw_weighted(&mut self.encounter_rng, pool, config.relics);
        if relics.is_empty() {
            return false;
        }
        self.game = None;
        self.phase = RunPhase::BossReward;
        events.push(RunEvent::BossRewardOffered {
            relics: relics.clone(),
        });
        self.boss_reward = Some(BossReward { relics });
        true
    }

    /// [`RunAction::ChooseBossRelic`]: take one relic of the pick (selling
    /// an owned one for its slot if asked) or none, then the route choice.
    pub(super) fn choose_boss_relic(
        &mut self,
        content: &Content,
        offer: Option<usize>,
        replace: Option<usize>,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        if self.phase != RunPhase::BossReward {
            return Err(RunError::WrongPhase);
        }
        let reward = self.boss_reward.clone().ok_or(RunError::WrongPhase)?;
        match offer {
            None => {
                if replace.is_some() {
                    return Err(RunError::InvalidSlot);
                }
                events.push(RunEvent::BossRewardDeclined);
            }
            Some(i) => {
                let relic = reward.relics.get(i).cloned().ok_or(RunError::InvalidSlot)?;
                if let Some(index) = replace {
                    let old = self
                        .relics
                        .get(index)
                        .cloned()
                        .ok_or(RunError::InvalidSlot)?;
                    let price = content.relic(&old).map_or(0, |d| sell_price(d.price));
                    self.relics.remove(index);
                    self.pay_out_stash(content, &old, events);
                    self.coins = self.coins.saturating_add(price);
                    events.push(RunEvent::RelicSold { relic: old, price });
                    self.settle_debt(events);
                } else if !content.relic_slot_free(self.character, self.relics.len()) {
                    return Err(RunError::NoFreeSlot);
                }
                self.relics.push(relic.clone());
                events.push(RunEvent::RelicGained { relic });
            }
        }
        self.boss_reward = None;
        self.open_route(content, events)
    }

    /// Lives left (ADR 0016).
    #[must_use]
    pub const fn lives(&self) -> u32 {
        self.lives
    }

    /// The open route choice.
    #[must_use]
    pub const fn route(&self) -> Option<&RouteChoice> {
        self.route.as_ref()
    }

    /// The route taken, while on the way to the next pub.
    #[must_use]
    pub const fn trip(&self) -> Option<&Trip> {
        self.trip.as_ref()
    }

    /// Every pub played so far, in order.
    #[must_use]
    pub fn pub_results(&self) -> &[PubResult] {
        &self.pub_results
    }

    /// The malus of the current pub, if it is a Spelunke.
    #[must_use]
    pub fn malus<'c>(&self, content: &'c Content) -> Option<&'c MalusDef> {
        let spelunke = self.series.spelunke.as_ref()?;
        content.maluses.get(&spelunke.malus)
    }

    /// The names of the city: per district its name, its look on the map
    /// and the names of its pubs (the last the boss pub). Derived from
    /// the seed alone.
    #[must_use]
    pub fn city(&self, content: &Content) -> Vec<CityDistrict> {
        content.city.generate(
            self.seed ^ CITY_STREAM,
            content.run.sections(),
            PUBS_PER_DISTRICT,
        )
    }

    /// A seed for the map of the whole city or of one district
    /// (`Some(section)`), stable for the run: for a frontend that draws
    /// streets. Fits a JavaScript number.
    #[must_use]
    pub const fn map_seed(&self, section: Option<u32>) -> u32 {
        let salt = match section {
            Some(s) => s as u64 + 1,
            None => 0,
        };
        let mixed = (self.seed ^ CITY_STREAM ^ salt.wrapping_mul(0x9E37_79B9_7F4A_7C15))
            .wrapping_mul(0xBF58_476D_1CE4_E5B9);
        (mixed >> 32) as u32
    }

    /// Decorative travel time of a way to pub `index` of `section`.
    fn minutes(&self, section: u32, index: u32, transport: Transport) -> u32 {
        let (base, spread) = match transport {
            Transport::Walk => (9, 9),
            Transport::Train => (5, 5),
            Transport::Bike => (11, 8),
        };
        let salt = u64::from(section) * 16 + u64::from(index) * 4 + transport as u64;
        let mixed = (u64::from(self.map_seed(Some(section))) ^ salt.wrapping_mul(0x9E37_79B9))
            .wrapping_mul(0xBF58_476D_1CE4_E5B9);
        base + u32::try_from((mixed >> 40) % spread).unwrap_or(0)
    }

    /// Lives up by `n`, at most to the maximum.
    pub(super) fn gain_lives(&mut self, content: &Content, n: u32, events: &mut Vec<RunEvent>) {
        let max = content.run.lives.max;
        let new = self.lives.saturating_add(n).min(max).max(self.lives);
        let change = new - self.lives;
        if change > 0 {
            self.lives = new;
            events.push(RunEvent::LivesChanged {
                change: i32::try_from(change).unwrap_or(i32::MAX),
                lives: new,
            });
        }
    }

    /// A Spelunke malus with a curse: random cards of the player's edition
    /// without modification carry it for this pub (no Groschen, no
    /// `OnAttach`). Called when the pub starts.
    pub(super) fn curse_for_pub(&mut self, content: &Content, events: &mut Vec<RunEvent>) {
        let Some(curse) = self.malus(content).and_then(|m| m.curse.clone()) else {
            return;
        };
        let Some(def) = content.card_mod(&curse.card_mod) else {
            return;
        };
        let mut bare: Vec<u32> = self
            .edition
            .cards()
            .iter()
            .filter(|c| c.mods.is_empty() && c.kind != CardKind::Blank && def.fits(c.kind))
            .map(|c| c.id)
            .collect();
        for _ in 0..curse.cards {
            if bare.is_empty() {
                break;
            }
            let pick = self.tour_rng.gen_range(0..bare.len() as u64);
            let card = bare.remove(usize::try_from(pick).unwrap_or(0));
            if let Ok(change) = self
                .edition
                .add_mod(content, card, &curse.card_mod, curse.value)
            {
                if let DeckChange::Modified { card_mod, .. } = &change {
                    self.series.temp_curses.push((card, card_mod.clone()));
                }
                events.push(RunEvent::DeckChanged(change));
            }
        }
    }

    /// "Die Oma" (ADR 0017, [`crate::content::Effect::CurseForPub`]): when
    /// the first game under her rule is dealt, random cards of the
    /// player's edition get a random curse until the pub ends, drawn with
    /// the route generator. Each card is kept as it was, so the edition is
    /// exactly the same afterwards; no Groschen, no `OnAttach`.
    pub(super) fn lay_boss_curses(&mut self, content: &Content, events: &mut Vec<RunEvent>) {
        if self.series.boss_cursed {
            return;
        }
        let Some(boss) = self.boss_rule(content).and_then(|b| content.boss(&b)) else {
            return;
        };
        let Some((cards, curses)) = boss.passive.iter().find_map(|e| match e {
            crate::content::Effect::CurseForPub { cards, curses } => Some((*cards, curses)),
            _ => None,
        }) else {
            return;
        };
        self.series.boss_cursed = true;
        let mut pool: Vec<u32> = self
            .edition
            .cards()
            .iter()
            .filter(|c| c.kind != CardKind::Blank)
            .map(|c| c.id)
            .collect();
        let mut laid = Vec::new();
        for _ in 0..cards {
            if pool.is_empty() {
                break;
            }
            let pick = self.tour_rng.gen_range(0..pool.len() as u64);
            let card = pool.remove(usize::try_from(pick).unwrap_or(0));
            let kind = self.edition.get(card).map(|c| c.kind);
            let fitting: Vec<&crate::content::CardModId> = curses
                .iter()
                .filter(|m| {
                    content
                        .card_mod(m)
                        .is_some_and(|d| kind.is_some_and(|k| d.fits(k)))
                })
                .collect();
            if fitting.is_empty() {
                continue;
            }
            let which = self.tour_rng.gen_range(0..fitting.len() as u64);
            let curse = fitting[usize::try_from(which).unwrap_or(0)].clone();
            let value = content
                .card_mod(&curse)
                .map_or(1, crate::content::CardModDef::default_value);
            if let Ok(before) = self.edition.lay_temp_curse(content, card, &curse, value) {
                self.series.boss_curses.push(before);
                laid.push((card, curse));
            }
        }
        laid.sort();
        events.push(RunEvent::BossCursesLaid {
            boss: boss.id.clone(),
            cards: laid,
        });
    }

    /// The pub ended: the curses of its malus and its boss come off again,
    /// and for "Der Drückeberger" the vouchers redeemed in it come back.
    pub(super) fn end_pub(&mut self, content: &Content, events: &mut Vec<RunEvent>) {
        for (card, card_mod) in std::mem::take(&mut self.series.temp_curses) {
            if let Some(change) = self.edition.remove_mod(card, &card_mod) {
                events.push(RunEvent::DeckChanged(change));
            }
        }
        let restored = std::mem::take(&mut self.series.boss_curses);
        if !restored.is_empty() {
            let mut cards: Vec<u32> = restored.iter().map(|c| c.id).collect();
            cards.sort_unstable();
            for card in restored {
                self.edition.restore(card);
            }
            events.push(RunEvent::BossCursesLifted { cards });
        }
        self.return_vouchers(content, events);
    }

    /// "Der Drückeberger" (DESIGN 9.4): every voucher redeemed in this pub
    /// goes back onto its card with its value. A card that carries another
    /// normal modification or a curse by now keeps it, the voucher stays
    /// gone; a special card it turned into (Untot, Wandelkarte) stays.
    fn return_vouchers(&mut self, content: &Content, events: &mut Vec<RunEvent>) {
        for (card, card_mod, value) in std::mem::take(&mut self.series.returning_vouchers) {
            let Some(deck_card) = self.edition.get(card) else {
                continue;
            };
            let taken = deck_card
                .mods
                .iter()
                .any(|m| content.card_mod(m).is_none_or(|d| !d.is_special()));
            if taken {
                continue;
            }
            if let Ok(change) = self.edition.add_mod(content, card, &card_mod, value) {
                events.push(RunEvent::VoucherReturned {
                    card,
                    card_mod,
                    value,
                });
                events.push(RunEvent::DeckChanged(change));
            }
        }
    }

    /// The pub was won and paid: a boss pub gives a life, the last one wins
    /// the run, the others open the relic pick (ADR 0017); a Spelunke
    /// opens its loot; then the route choice.
    pub(super) fn pub_won(
        &mut self,
        content: &Content,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        self.end_pub(content, events);
        self.pub_results.push(PubResult {
            section: self.series.section,
            index: self.series.index,
            won: true,
            spelunke: self.series.spelunke.as_ref().map(|s| s.name.clone()),
        });
        if self.series.is_boss_pub() {
            self.gain_lives(content, content.run.lives.boss_bonus, events);
            if self.series.section + 1 >= content.run.sections() {
                self.phase = RunPhase::Won;
                events.push(RunEvent::RunWon {
                    character: self.character,
                    length: self.length,
                    stake: self.stake,
                });
                return Ok(());
            }
            if self.open_boss_reward(content, events) {
                return Ok(());
            }
        }
        if self.series.spelunke.is_some() {
            let loot = content.run.spelunke.loot.clone();
            if self.open_encounter(content, &loot, events) {
                return Ok(());
            }
        }
        self.open_route(content, events)
    }

    /// The pub ended below its target. A boss pub ends the run whatever
    /// lives are left (ADR 0017; "Schutzengel" does not help there).
    /// Otherwise a life goes: with none left the run is lost, else on to
    /// the route choice without reward.
    pub(super) fn pub_lost(
        &mut self,
        content: &Content,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        self.end_pub(content, events);
        self.stats.pubs_lost += 1;
        let (section, index) = (self.series.section, self.series.index);
        self.pub_results.push(PubResult {
            section,
            index,
            won: false,
            spelunke: self.series.spelunke.as_ref().map(|s| s.name.clone()),
        });
        events.push(RunEvent::PubLost {
            section,
            index,
            boss: self.series.boss.clone(),
        });
        if self.series.is_boss_pub() {
            self.phase = RunPhase::Lost;
            events.push(RunEvent::RunLost {
                section,
                index,
                character: self.character,
                length: self.length,
                stake: self.stake,
                boss: self.series.boss.clone(),
            });
            return Ok(());
        }
        let angel = self
            .relic_with(content, |e| *e == crate::content::Effect::SpareLife)
            .filter(|_| self.spared_section != Some(section));
        if let Some(source) = angel {
            self.spared_section = Some(section);
            events.push(RunEvent::LifeSpared { source });
        } else {
            self.lives = self.lives.saturating_sub(1);
            events.push(RunEvent::LivesChanged {
                change: -1,
                lives: self.lives,
            });
        }
        if self.lives == 0 {
            self.phase = RunPhase::Lost;
            events.push(RunEvent::RunLost {
                section,
                index,
                character: self.character,
                length: self.length,
                stake: self.stake,
                boss: None,
            });
            return Ok(());
        }
        self.open_route(content, events)
    }

    /// Whether a stop of this category can be put on a way now: the Rast
    /// only with a life missing, the Kiosk with a Schnaps to sell, an
    /// encounter category with an encounter that has an option to take.
    fn category_open(&self, content: &Content, category: StopCategory) -> bool {
        match category {
            StopCategory::Tresen => true,
            StopCategory::Rest if self.lives >= content.run.lives.max => false,
            StopCategory::Kiosk => !self.kiosk_pool(content).is_empty(),
            _ => !self.fitting_encounters(content, category).is_empty(),
        }
    }

    /// Encounters of a category the player may meet now, with their weight.
    fn fitting_encounters(
        &self,
        content: &Content,
        category: StopCategory,
    ) -> Vec<(EncounterId, u32)> {
        content
            .encounters
            .values()
            .filter(|e| e.category == category && e.weight > 0)
            .filter(|e| in_pool(&e.characters, self.character))
            .filter(|e| self.encounter_available(content, e))
            .map(|e| (e.id.clone(), e.weight))
            .collect()
    }

    /// The stop of a category: the encounter drawn by weight (one not on
    /// the choice yet if there is one), none for a shop.
    fn stop_of(
        &mut self,
        content: &Content,
        category: StopCategory,
        used: &mut Vec<EncounterId>,
    ) -> RouteStop {
        if category.is_shop() {
            return RouteStop {
                category,
                encounter: None,
            };
        }
        let fitting = self.fitting_encounters(content, category);
        let fresh: Vec<(EncounterId, u32)> = fitting
            .iter()
            .filter(|(id, _)| !used.contains(id))
            .cloned()
            .collect();
        let pool = if fresh.is_empty() { fitting } else { fresh };
        match draw_weighted(&mut self.tour_rng, pool, 1).pop() {
            Some(id) => {
                used.push(id.clone());
                RouteStop {
                    category,
                    encounter: Some(id),
                }
            }
            None => RouteStop {
                category: StopCategory::Tresen,
                encounter: None,
            },
        }
    }

    /// Open the route choice to the pub after the current one.
    pub(super) fn open_route(
        &mut self,
        content: &Content,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        self.game = None;
        self.shop = None;
        self.encounter = None;
        self.trip = None;
        let (section, index) = self.next_series_position();
        if section >= content.run.sections() {
            return Err(RunError::WrongPhase);
        }
        let mut options: Vec<RouteOption> = Transport::ALL
            .into_iter()
            .map(|transport| RouteOption {
                transport,
                stops: Vec::new(),
                spelunke: None,
            })
            .collect();
        self.draw_stops(content, &mut options);
        // A Spelunke replaces a normal pub, never a boss pub.
        let chance = u64::from(content.run.spelunke.chance.permille());
        if index + 1 < PUBS_PER_DISTRICT && chance > 0 && self.tour_rng.gen_range(0..1000) < chance
        {
            let maluses: Vec<(MalusId, u32)> = content
                .maluses
                .values()
                .filter(|m| m.weight > 0 && in_pool(&m.characters, self.character))
                .map(|m| (m.id.clone(), m.weight))
                .collect();
            let offers = self
                .relic_passives(content)
                .into_iter()
                .map(|e| match e {
                    crate::content::Effect::ChooseMalus(n) => *n,
                    _ => 1,
                })
                .max()
                .unwrap_or(1)
                .max(1);
            let drawn = draw_weighted(&mut self.tour_rng, maluses, offers);
            if let Some(malus) = drawn.first().cloned() {
                let malus_options = if drawn.len() > 1 { drawn } else { Vec::new() };
                let pick = self.tour_rng.gen_range(0..options.len() as u64);
                let mut names: Vec<String> = self
                    .city(content)
                    .into_iter()
                    .flat_map(|d| d.pubs)
                    .collect();
                let city = &content.city;
                let name = city.fresh(&mut self.tour_rng, &city.spelunken, &mut names);
                if let Some(option) = options.get_mut(usize::try_from(pick).unwrap_or(0)) {
                    option.spelunke = Some(Spelunke {
                        name,
                        malus,
                        malus_options,
                    });
                }
            }
        }
        self.phase = RunPhase::Route;
        self.route = Some(RouteChoice {
            section,
            index,
            options,
        });
        events.push(RunEvent::RouteOpened { section, index });
        Ok(())
    }

    /// One stop per way, drawn anew (decision of the user, 03.11.2026):
    /// the three ways show three different categories, one of them the
    /// Tresen on a random way; the others are drawn by their weight
    /// without putting back, among the categories open now.
    fn draw_stops(&mut self, content: &Content, options: &mut [RouteOption]) {
        let ways = options.len();
        if ways == 0 {
            return;
        }
        let tresen = usize::try_from(self.tour_rng.gen_range(0..ways as u64)).unwrap_or(0);
        let weights = &content.run.route.weights;
        let mut pool: Vec<(StopCategory, u32)> = StopCategory::ALL
            .into_iter()
            .filter(|c| *c != StopCategory::Tresen)
            .map(|c| (c, weights.of(c)))
            .filter(|(c, w)| *w > 0 && self.category_open(content, *c))
            .collect();
        let mut used = Vec::new();
        for (i, option) in options.iter_mut().enumerate() {
            let category = if i == tresen {
                StopCategory::Tresen
            } else {
                match draw_weighted(&mut self.tour_rng, pool.clone(), 1).pop() {
                    Some(c) => {
                        pool.retain(|(p, _)| *p != c);
                        c
                    }
                    None => StopCategory::Tresen,
                }
            };
            option.stops = vec![self.stop_of(content, category, &mut used)];
        }
    }

    /// "Schwenker": the stops of the open route choice drawn again; Spelunken
    /// stay.
    pub(super) fn reroll_routes(&mut self, content: &Content) {
        let Some(mut route) = self.route.take() else {
            return;
        };
        self.draw_stops(content, &mut route.options);
        self.route = Some(route);
    }

    /// "Spelunkenkenner": pick the malus of the Spelunke on route `option`
    /// ([`super::RunAction::ChooseMalus`]).
    pub(super) fn choose_malus(
        &mut self,
        _content: &Content,
        option: usize,
        malus: usize,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        if self.phase != RunPhase::Route {
            return Err(RunError::WrongPhase);
        }
        let spelunke = self
            .route
            .as_mut()
            .and_then(|r| r.options.get_mut(option))
            .and_then(|o| o.spelunke.as_mut())
            .ok_or(RunError::InvalidSlot)?;
        let chosen = spelunke
            .malus_options
            .get(malus)
            .cloned()
            .ok_or(RunError::InvalidSlot)?;
        spelunke.malus = chosen.clone();
        events.push(RunEvent::MalusChosen {
            option,
            malus: chosen,
        });
        Ok(())
    }

    /// Take a route ([`super::RunAction::ChooseRoute`]).
    pub(super) fn choose_route(
        &mut self,
        content: &Content,
        option: usize,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        if self.phase != RunPhase::Route {
            return Err(RunError::WrongPhase);
        }
        let route = self.route.as_ref().ok_or(RunError::WrongPhase)?;
        let chosen = route
            .options
            .get(option)
            .cloned()
            .ok_or(RunError::InvalidSlot)?;
        events.push(RunEvent::RouteChosen {
            option,
            transport: chosen.transport,
            spelunke: chosen.spelunke.is_some(),
        });
        self.trip = Some(Trip {
            section: route.section,
            index: route.index,
            transport: chosen.transport,
            stops: chosen.stops,
            spelunke: chosen.spelunke,
        });
        self.route = None;
        if self.scout_legs > 0 {
            self.scout_legs -= 1;
        }
        self.fire_with(
            content,
            crate::content::Stage::OnRouteChosen,
            None,
            None,
            Some(chosen.transport),
            events,
        );
        self.continue_trip(content, events)
    }

    /// On along the way: the next stop that has something to offer, or the
    /// pub at the end. Without a route taken (after the loot of a
    /// Spelunke, or a save from before the Kneipentour at the Tresen) the
    /// route choice opens.
    pub(super) fn continue_trip(
        &mut self,
        content: &Content,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        loop {
            let Some(trip) = self.trip.as_mut() else {
                return self.open_route(content, events);
            };
            if trip.stops.is_empty() {
                let Some(trip) = self.trip.take() else {
                    return Err(RunError::WrongPhase);
                };
                return self.start_pub(content, trip.section, trip.index, trip.spelunke, events);
            }
            let stop = trip.stops.remove(0);
            if self.open_stop(content, &stop, events) {
                return Ok(());
            }
        }
    }

    /// Schnäpse the Kiosk may sell, with their rarity weight.
    fn kiosk_pool(&self, content: &Content) -> Vec<(crate::content::ConsumableId, u32)> {
        self.schnapps_pool(content)
    }

    /// Open a Kiosk (03.11.2026): a shop with a few Schnäpse, drawn with
    /// the Tresen's generator.
    fn open_kiosk(&mut self, content: &Content, events: &mut Vec<RunEvent>) {
        self.phase = RunPhase::Shop;
        let pool = self.kiosk_pool(content);
        let drawn = draw_weighted(&mut self.shop_rng, pool, content.run.route.kiosk.offers);
        let shop = Shop {
            consumables: drawn.into_iter().map(Some).collect(),
            kiosk: true,
            ..Shop::default()
        };
        events.push(RunEvent::KioskOpened {
            consumables: shop.consumables.clone(),
        });
        self.shop = Some(shop);
    }

    /// Open a stop; `false` if it has nothing to offer.
    fn open_stop(
        &mut self,
        content: &Content,
        stop: &RouteStop,
        events: &mut Vec<RunEvent>,
    ) -> bool {
        match (&stop.category, &stop.encounter) {
            (StopCategory::Kiosk, _) => {
                self.open_kiosk(content, events);
                true
            }
            (StopCategory::Tresen, _) | (_, None) => {
                self.phase = RunPhase::Shop;
                let shop: Shop = self.draw_offer(content, 0);
                events.push(RunEvent::ShopOpened {
                    relics: shop.relics.clone(),
                    consumables: shop.consumables.clone(),
                });
                self.shop = Some(shop);
                self.draw_card_offers(content, events);
                self.fire(
                    content,
                    crate::content::Stage::OnShopOpened,
                    None,
                    None,
                    events,
                );
                true
            }
            (_, Some(id)) => {
                if self.open_encounter(content, id, events) {
                    self.phase = RunPhase::Encounter;
                    true
                } else {
                    false
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // View
    // -----------------------------------------------------------------------

    /// Preview of a pub: name, target, who sits at the table.
    fn pub_preview(
        &self,
        content: &Content,
        city: &[CityDistrict],
        section: u32,
        index: u32,
        spelunke: Option<&Spelunke>,
    ) -> Option<PubPreview> {
        let is_boss = index + 1 == PUBS_PER_DISTRICT;
        // A new district's boss is drawn when it begins.
        let boss = if is_boss && section == self.series.section {
            self.section_boss.as_ref()
        } else {
            None
        };
        let mut target = series_target(content, self.character, section, index, boss)?;
        if spelunke.is_some() {
            target = content.run.spelunke.target_factor.apply(target);
        }
        if (section, index) != (self.series.section, self.series.index) {
            target = self.next_pub.apply_to(target);
        }
        let district = city.get(usize::try_from(section).ok()?)?;
        let name = spelunke.map_or_else(
            || district.pubs.get(usize::try_from(index).ok()?).cloned(),
            |s| Some(s.name.clone()),
        )?;
        let guests = seat_guests(content, self.seed, section, index, &self.guests)
            .iter()
            .zip(crate::game::Seat::DEFENDERS)
            .filter_map(|(id, seat)| self.guest_info(content, id, seat))
            .collect();
        Some(PubPreview {
            id: pub_id(section, index, spelunke.is_some()),
            section,
            index,
            name,
            district: district.name.clone(),
            target,
            game_limit: content.run.game_limit,
            is_boss,
            boss: boss.map(|id| self.boss_info(content, id)),
            guests,
            malus: spelunke
                .and_then(|s| content.maluses.get(&s.malus))
                .map(malus_info),
        })
    }

    /// The Kneipentour for a frontend.
    #[must_use]
    pub fn tour_view(&self, content: &Content) -> TourView {
        let city = self.city(content);
        let current = (self.series.section, self.series.index);
        let route = self.route.as_ref().and_then(|r| {
            let next = self.pub_preview(content, &city, r.section, r.index, None)?;
            let options = r
                .options
                .iter()
                .enumerate()
                .map(|(i, o)| RouteOptionView {
                    id: format!("{}-r{i}", pub_id(r.section, r.index, false)),
                    index: i,
                    transport: o.transport,
                    minutes: self.minutes(r.section, r.index, o.transport),
                    can_choose: self.accepts(content, super::RunAction::ChooseRoute(i)),
                    stops: o
                        .stops
                        .iter()
                        .map(|stop| StopView {
                            category: stop.category,
                            encounter: (self.scout_legs > 0)
                                .then_some(stop.encounter.as_ref())
                                .flatten()
                                .and_then(|id| content.encounter(id))
                                .map(|e| StopEncounter {
                                    id: e.id.clone(),
                                    name: e.name.clone(),
                                }),
                        })
                        .collect(),
                    malus_options: o
                        .spelunke
                        .as_ref()
                        .map(|s| {
                            s.malus_options
                                .iter()
                                .filter_map(|m| content.maluses.get(m))
                                .map(malus_info)
                                .collect()
                        })
                        .unwrap_or_default(),
                    spelunke: o.spelunke.as_ref().and_then(|s| {
                        self.pub_preview(content, &city, r.section, r.index, Some(s))
                    }),
                })
                .collect();
            Some(RouteView { next, options })
        });
        let done = |section: u32, index: u32| {
            self.pub_results
                .iter()
                .rev()
                .find(|p| p.section == section && p.index == index)
        };
        let districts = city
            .iter()
            .enumerate()
            .map(|(d, district)| {
                let section = u32::try_from(d).unwrap_or(0);
                let mut nodes = Vec::new();
                let mut spelunken = Vec::new();
                for (p, pub_name) in district.pubs.iter().enumerate() {
                    let index = u32::try_from(p).unwrap_or(0);
                    let here = (section, index) == current;
                    let result = done(section, index);
                    let playing = here && matches!(self.phase, RunPhase::Game | RunPhase::Lost);
                    let state = if playing {
                        PubState::Current
                    } else if let Some(r) = result {
                        if r.won { PubState::Won } else { PubState::Lost }
                    } else {
                        PubState::Ahead
                    };
                    let spelunke = if here && self.series.spelunke.is_some() {
                        self.series.spelunke.as_ref().map(|s| s.name.clone())
                    } else {
                        result.and_then(|r| r.spelunke.clone())
                    };
                    if let Some(spelunke) = spelunke {
                        // The Spelunke is a place of its own; the pub it
                        // stood in for was passed by.
                        spelunken.push(SpelunkeNode {
                            id: pub_id(section, index, true),
                            index,
                            name: spelunke,
                            state,
                        });
                        nodes.push(PubNode {
                            id: pub_id(section, index, false),
                            index,
                            name: pub_name.clone(),
                            is_boss: false,
                            state: PubState::Skipped,
                        });
                    } else {
                        nodes.push(PubNode {
                            id: pub_id(section, index, false),
                            index,
                            name: pub_name.clone(),
                            is_boss: index + 1 == PUBS_PER_DISTRICT,
                            state,
                        });
                    }
                }
                DistrictView {
                    index: section,
                    name: district.name.clone(),
                    look: district.look,
                    boss: (section == self.series.section)
                        .then_some(self.section_boss.as_ref())
                        .flatten()
                        .map(|id| self.boss_info(content, id)),
                    map_seed: self.map_seed(Some(section)),
                    pubs: nodes,
                    spelunken,
                }
            })
            .collect();
        let current_pub = if matches!(self.phase, RunPhase::Won) {
            None
        } else {
            self.pub_preview(
                content,
                &city,
                self.series.section,
                self.series.index,
                self.series.spelunke.as_ref(),
            )
            .map(|mut p| {
                p.target = self.series.target;
                p
            })
        };
        TourView {
            lives: self.lives,
            max_lives: content.run.lives.max,
            map_seed: self.map_seed(None),
            districts,
            current: current_pub,
            route,
            trip: self.trip.as_ref().map(|t| TripView {
                transport: t.transport,
                stops_left: t.stops.len(),
                spelunke: t.spelunke.is_some(),
            }),
        }
    }
}

/// Stable id of a pub node: `d0-p3`, a Spelunke in its place `d0-p3-s`.
#[must_use]
pub fn pub_id(section: u32, index: u32, spelunke: bool) -> String {
    if spelunke {
        format!("d{section}-p{index}-s")
    } else {
        format!("d{section}-p{index}")
    }
}

fn malus_info(def: &MalusDef) -> MalusInfo {
    MalusInfo {
        id: def.id.clone(),
        name: def.name.clone(),
        text: def.text.clone(),
    }
}

/// A Spelunke malus as the player sees it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MalusInfo {
    /// Id, also the key for the frontend's texts.
    pub id: MalusId,
    /// German name.
    pub name: String,
    /// German rule text.
    pub text: String,
}

/// A pub before or while it is played.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PubPreview {
    /// Stable node id ([`pub_id`]).
    pub id: String,
    /// District, from 0.
    pub section: u32,
    /// Pub within the district, from 0.
    pub index: u32,
    /// Its name.
    pub name: String,
    /// The district's name.
    pub district: String,
    /// Zielwert (with the boss's and the Spelunke's factor).
    pub target: u64,
    /// Spiellimit.
    pub game_limit: u32,
    /// Whether it is the boss pub.
    pub is_boss: bool,
    /// Its boss, once known.
    pub boss: Option<BossInfo>,
    /// The Stammgäste at the table (in a boss pub the boss takes the left
    /// seat).
    pub guests: Vec<GuestInfo>,
    /// The malus of a Spelunke.
    pub malus: Option<MalusInfo>,
}

/// A stop as the route choice shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StopView {
    /// Its category (the exact encounter stays hidden).
    pub category: StopCategory,
    /// The exact encounter, shown only while "Kiebitz am Fenster" scouts
    /// ([`Run::scout_legs`]).
    pub encounter: Option<StopEncounter>,
}

/// An encounter a scouted stop shows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StopEncounter {
    /// Id, also the key for the frontend's texts.
    pub id: EncounterId,
    /// German name.
    pub name: String,
}

/// One way of the route choice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RouteOptionView {
    /// Stable node id of the way (`d0-p3-r1`).
    pub id: String,
    /// Index for [`super::RunAction::ChooseRoute`].
    pub index: usize,
    /// How the player goes.
    pub transport: Transport,
    /// How long the way takes, in minutes: decoration only, stable for
    /// the run (from the seed and the place).
    pub minutes: u32,
    /// Whether it can be taken now.
    pub can_choose: bool,
    /// The stops, in order.
    pub stops: Vec<StopView>,
    /// The Spelunke it leads into instead of the next pub.
    pub spelunke: Option<PubPreview>,
    /// "Spelunkenkenner": the maluses the Spelunke offers
    /// ([`super::RunAction::ChooseMalus`]); its `malus` is the one chosen.
    /// Empty without a choice.
    pub malus_options: Vec<MalusInfo>,
}

/// The open route choice.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RouteView {
    /// The next pub, where every way leads but a Spelunke's.
    pub next: PubPreview,
    /// The three ways.
    pub options: Vec<RouteOptionView>,
}

/// The way taken, while on it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TripView {
    /// How the player goes.
    pub transport: Transport,
    /// Stops still ahead after the current one.
    pub stops_left: usize,
    /// Whether it leads into a Spelunke.
    pub spelunke: bool,
}

/// Where a pub stands on the map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum PubState {
    /// Played and won.
    Won,
    /// Played and lost.
    Lost,
    /// Being played.
    Current,
    /// Still ahead.
    Ahead,
    /// Passed by: a Spelunke was played in its place.
    Skipped,
}

/// A pub on the map.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PubNode {
    /// Stable node id ([`pub_id`], without the Spelunke suffix).
    pub id: String,
    /// Pub within the district.
    pub index: u32,
    /// Its name.
    pub name: String,
    /// Whether it is the boss pub.
    pub is_boss: bool,
    /// Where it stands.
    pub state: PubState,
}

/// A Spelunke on the map: a place of its own, played instead of the pub
/// with the same index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SpelunkeNode {
    /// Stable node id (`d0-p2-s`).
    pub id: String,
    /// The index of the pub it stood in for.
    pub index: u32,
    /// Its name.
    pub name: String,
    /// Where it stands (current, won or lost).
    pub state: PubState,
}

/// A district on the map.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DistrictView {
    /// From 0.
    pub index: u32,
    /// Its name.
    pub name: String,
    /// How it looks on the map (from the content, `tour.ron`).
    pub look: DistrictLook,
    /// Its boss once known (from the start of the district; earlier and
    /// later districts show none).
    pub boss: Option<BossInfo>,
    /// Seed for drawing its streets.
    pub map_seed: u32,
    /// Its pubs in order, the boss pub last.
    pub pubs: Vec<PubNode>,
    /// The Spelunken played or being played in it.
    pub spelunken: Vec<SpelunkeNode>,
}

/// Everything of the Kneipentour a frontend shows (ADR 0016).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TourView {
    /// Lives left.
    pub lives: u32,
    /// Most lives a player can hold.
    pub max_lives: u32,
    /// Seed for drawing the whole city.
    pub map_seed: u32,
    /// The districts with their pubs.
    pub districts: Vec<DistrictView>,
    /// The current pub (the one just played while on the way); `None`
    /// once the run is won.
    pub current: Option<PubPreview>,
    /// The open route choice.
    pub route: Option<RouteView>,
    /// The way taken, while on it.
    pub trip: Option<TripView>,
}
