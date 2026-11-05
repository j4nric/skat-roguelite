//! Runs mode: complete runs with the simple run policy.
//!
//! Statistics: section reached, win rate per series, Groschen, and for
//! every relic how often it was bought and how runs with it went. Per pub
//! (Spelunken in their own rows): games and Einmischen per pub, relics on
//! entering, Groschen per won pub. Per district: relics at its start and
//! before the boss, lost pubs per run, where the lost runs ended (normal
//! pub, Spelunke, boss pub; since ADR 0019 every lost pub ends the run)
//! and pubs played again by the "Schutzengel". The Faktor (ADR 0018): the
//! factor of won games, the highest per run, runs with a Faktor relic.
//! Groschen: income by source and spending by kind ([`SPENDING`]).
//!
//! Options (all reproducible with the same seed, see [`Options`]):
//! `--no-shop`, `--relic <id>` (repeatable), `--without <id>`
//! (repeatable), `--boss <id>`, `--character declarer|null`,
//! `--null-scoring fixed|formula`, `--curve` (success curve of a single
//! series instead of whole runs, see [`curve`]; `--district N` places it
//! in district N), `--unlocks`
//! (reachability of every unlock, see [`unlocks`]) and `--consumable <id>` (a steady supply of one
//! consumable, see [`Options::consumable`]).
//!
//! Runs start with everything unlocked (no relic is locked), so that the
//! measurement does not depend on a player's progress. `--unlocks` plays
//! with a growing meta state instead.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::process::ExitCode;
use std::time::Instant;

use skat_core::ai::pimc::Opponents;
use skat_core::cards::{Card, CardKind, Suit};
use skat_core::content::{
    BossId, CardModId, Character, ConsumableId, Content, Effect, EncounterId, Hook, RelicGroup,
    RelicId, RunLength, UnlockId,
};
use skat_core::deck::DeckChange;
use skat_core::rng::Rng;
use skat_core::rules::{Declaration, NullFormula, NullScoring};
use skat_core::run::bot::RunBot;
use skat_core::run::meta::MetaState;
use skat_core::run::{
    PUBS_PER_DISTRICT, Run, RunAction, RunEvent, RunPhase, RunStart, series_target,
};

/// Measurement options of the runs mode.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Options {
    /// The bot never buys anything: it leaves every Tresen at once.
    pub no_shop: bool,
    /// Relics owned from the start of every run, with their passive
    /// effects already in the first game ([`RunStart::relics`]).
    pub relics: Vec<RelicId>,
    /// Relics taken out of the content: never offered, so never bought.
    pub without: Vec<RelicId>,
    /// The only boss a boss series may draw.
    pub boss: Option<BossId>,
    /// Measure the success curve of one series instead of whole runs.
    pub curve: bool,
    /// Who plays.
    pub character: Character,
    /// Override of the Null scoring of the character that plays.
    pub null_scoring: Option<NullScoring>,
    /// Measure how many runs it takes to earn every unlock.
    pub unlocks: bool,
    /// The player's card play (`--bot`): heuristic or PIMC.
    pub bot: Opponents,
    /// The defenders' card play in every section (`--defenders`), instead
    /// of the per-section list of `run.ron`.
    pub defenders: Option<Opponents>,
    /// Ganzer or kurzer Abend (`--short`, DESIGN 5.3).
    pub length: RunLength,
    /// Stake of the Einsatz-Leiter (`--stake N`, DESIGN 9.3): suit games
    /// and Grand are won from `61 + N` Augen.
    pub stake: u32,
    /// A steady supply of one consumable (`--consumable <id>`), to measure
    /// its effect alone: every run starts with one, and every Tresen offers
    /// only this one, for free, and no relics (so with the bot it is like
    /// `--no-shop` plus the consumable). Compare against `--no-shop`.
    pub consumable: Option<ConsumableId>,
    /// Card modifications on the start deck (`--card-mod
    /// <card>:<id>[:<value>]`), each on the first instance of its printed
    /// card with its value (default: its first value on offer), like
    /// [`skat_core::content::DeckSpec::mods`].
    pub card_mods: Vec<(Card, CardModId, Option<i32>)>,
    /// District of the measured pub under `--curve` (`--district N`, from
    /// 1): the pub starts there, so the defenders are that district's
    /// (`run.ron`'s `opponents`) unless `--defenders` overrides them.
    /// `None`: district 1.
    pub district: Option<u32>,
}

/// Price of the Neuauslage and the Zinken under `--consumable`: never
/// affordable, so the bot keeps its Groschen as under `--no-shop`.
const NO_REROLL: u32 = 1_000;

/// The Null formula of DESIGN 9.2 with the values of `characters.ron`, for
/// `--null-scoring formula`.
#[must_use]
pub fn formula_scoring() -> NullScoring {
    NullScoring::Formula(NullFormula {
        base_value: 23,
        per_seven_ducked: 1,
        never_followed: 2,
        short_circuit: 3,
        short_circuit_tricks: 5,
        short_circuit_points: 61,
    })
}

impl Options {
    /// The content with `--without` and `--boss` applied. Fails on unknown
    /// ids.
    pub fn content(&self, base: &Content) -> Result<Content, String> {
        let mut content = base.clone();
        for id in &self.relics {
            if content.relic(id).is_none() {
                return Err(format!("--relic: unknown relic {id}"));
            }
        }
        for id in &self.without {
            if content.relics.remove(id).is_none() {
                return Err(format!("--without: unknown relic {id}"));
            }
            if self.relics.contains(id) {
                return Err(format!("--without {id} and --relic {id} contradict"));
            }
        }
        if let Some(boss) = &self.boss {
            if content.boss(boss).is_none() {
                return Err(format!("--boss: unknown boss {boss}"));
            }
            content.run.bosses = vec![boss.clone()];
            content.run.final_boss = None;
        }
        if let Some(scoring) = &self.null_scoring {
            let def = content
                .characters
                .get_mut(&self.character)
                .ok_or("--null-scoring: unknown character")?;
            def.null_scoring = Some(scoring.clone());
        }
        if content.character(self.character).is_none() {
            return Err(format!("--character: unknown {}", self.character));
        }
        for (card, id, value) in &self.card_mods {
            let def = content
                .card_mod(id)
                .ok_or_else(|| format!("--card-mod: unknown card mod {id}"))?;
            if !def.fits(CardKind::Normal(*card)) {
                return Err(format!("--card-mod: {id} does not fit {card}"));
            }
            let value = value.unwrap_or_else(|| def.default_value());
            if let Some(def) = content.characters.get_mut(&self.character) {
                def.deck.mods.push((*card, id.clone(), value));
            }
        }
        if let Some(district) = self.district {
            if !self.curve {
                return Err("--district needs --curve".into());
            }
            if district == 0 || district > content.run.sections() {
                return Err(format!(
                    "--district: {district} is not a district (1 to {})",
                    content.run.sections()
                ));
            }
        }
        if let Some(defenders) = self.defenders {
            content.run.opponents = vec![defenders];
        }
        if let Some(id) = &self.consumable {
            if content.consumable(id).is_none() {
                return Err(format!("--consumable: unknown consumable {id}"));
            }
            // Every other consumable only for the other characters: off
            // this character's Tresen.
            let others: Vec<Character> = content
                .characters
                .keys()
                .copied()
                .filter(|c| *c != self.character)
                .collect();
            for def in content.consumables.values_mut() {
                if def.id == *id {
                    def.price = 0;
                } else {
                    def.characters.clone_from(&others);
                }
            }
            content.run.shop.relic_offers = 0;
            content.run.shop.consumable_offers = 1;
            content.run.economy.reroll_cost = NO_REROLL;
            // No Zinken either (DESIGN 8.6): only that consumable is bought.
            let zinken = &mut content.run.shop.zinken;
            zinken.offers = 0;
            zinken.reroll_cost = NO_REROLL;
            zinken.painter_price = NO_REROLL;
            zinken.cleanse_price = NO_REROLL;
            let slots = usize::try_from(content.run.consumable_slots).unwrap_or(0);
            if let Some(def) = content.characters.get_mut(&self.character) {
                if def.start_consumables.len() < slots {
                    def.start_consumables.push(id.clone());
                }
            }
        }
        Ok(content)
    }

    /// How every measured run starts.
    fn start(&self) -> RunStart {
        RunStart {
            relics: self.relics.clone(),
            character: self.character,
            length: self.length,
            stake: self.stake,
            ..RunStart::default()
        }
    }

    /// The options as one line for the report header.
    fn describe(&self) -> String {
        let mut parts = Vec::new();
        if self.no_shop {
            parts.push("no-shop".to_string());
        }
        for id in &self.relics {
            parts.push(format!("relic={id}"));
        }
        for id in &self.without {
            parts.push(format!("without={id}"));
        }
        if let Some(boss) = &self.boss {
            parts.push(format!("boss={boss}"));
        }
        if self.character != Character::Declarer {
            parts.push(format!("character={}", self.character));
        }
        match &self.null_scoring {
            Some(NullScoring::Fixed) => parts.push("null-scoring=fixed".into()),
            Some(NullScoring::Formula(_)) => parts.push("null-scoring=formula".into()),
            None => {}
        }
        if let Opponents::Pimc { samples, depth } = self.bot {
            parts.push(format!("bot=pimc:{samples}:{depth}"));
        }
        match self.defenders {
            Some(Opponents::Heuristic) => parts.push("defenders=heuristic".into()),
            Some(Opponents::Pimc { samples, depth }) => {
                parts.push(format!("defenders=pimc:{samples}:{depth}"));
            }
            None => {}
        }
        if self.length == RunLength::Short {
            parts.push("short".into());
        }
        if self.stake > 0 {
            parts.push(format!("stake={}", self.stake));
        }
        if let Some(id) = &self.consumable {
            parts.push(format!("consumable={id}"));
        }
        for (card, id, value) in &self.card_mods {
            match value {
                Some(value) => parts.push(format!("card-mod={card}:{id}:{value}")),
                None => parts.push(format!("card-mod={card}:{id}")),
            }
        }
        if let Some(district) = self.district {
            parts.push(format!("district={district}"));
        }
        if parts.is_empty() {
            "standard".to_string()
        } else {
            parts.join(" ")
        }
    }

    /// The bot's next action under these options.
    fn next_action(&self, run: &Run, content: &Content) -> Option<RunAction> {
        if self.no_shop && run.phase() == RunPhase::Shop {
            return Some(RunAction::LeaveShop);
        }
        let action = RunBot.next_action(run, content)?;
        let plays = matches!(action, RunAction::Play(_));
        // An "Untot" card that takes the trick and rises: the bot's rule
        // comes before the search, which knows nothing of rising.
        if plays && RunBot::undead_card(run, content).is_some() {
            return Some(action);
        }
        if let (true, Some(config)) = (plays, self.bot.pimc()) {
            if let Some(card) = skat_core::run::opponents::player_card(run, config) {
                return Some(RunAction::Play(card.id));
            }
        }
        Some(action)
    }
}

/// Upper bound of actions per run, a guard against a policy that loops.
const MAX_ACTIONS: usize = 1_000_000;

/// How runs with one relic went.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RelicStat {
    /// Times bought (a relic sold and bought again counts twice).
    pub bought: u64,
    /// Runs in which it was bought at least once.
    pub runs: u64,
    /// Of those, runs won.
    pub runs_won: u64,
    /// Sum of the sections reached (1-based) by those runs.
    pub sections: u64,
}

/// How one pub slot went over all runs: a normal pub, a boss pub, or a
/// Spelunke in its place.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PubStat {
    /// Pubs entered.
    pub attempts: u64,
    /// Pubs won.
    pub wins: u64,
    /// Games played in them, summed.
    pub games: u64,
    /// Einmischen in them, summed.
    pub redeals: u64,
    /// Of the attempts, pubs played again ("Schutzengel", ADR 0019).
    pub replays: u64,
    /// Relics owned on entering, summed.
    pub relics: u64,
    /// Groschen paid by the won ones, summed.
    pub payout: u64,
}

/// Where a lost run ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RunEnd {
    /// A normal pub was lost (ADR 0019: no lives).
    Pub,
    /// A Spelunke was lost.
    Spelunke,
    /// A boss pub was lost (ADR 0017).
    Boss,
}

/// What the bot spends Groschen on, in report order.
pub const SPENDING: [&str; 7] = [
    "relics",
    "schnaps",
    "zinken",
    "painter",
    "cleanse",
    "zinken_reroll",
    "reroll",
];

/// Per district, over all runs that reached it.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DistrictStat {
    /// Runs that started the district.
    pub runs: u64,
    /// Boss pubs entered (a boss pub played again counts once).
    pub boss_runs: u64,
    /// Relics owned when the district started, summed.
    pub relics_start: u64,
    /// Relics owned before the boss pub, summed.
    pub relics_boss: u64,
    /// Pubs lost, the boss pub included (since ADR 0019 each ends the run
    /// unless the "Schutzengel" has it played again).
    pub lost_pubs: u64,
    /// Pubs played again by the "Schutzengel" (ADR 0019).
    pub second_chances: u64,
    /// Lost runs that ended in a normal pub, a Spelunke or the boss pub.
    pub ended: [u64; 3],
}

/// Aggregate over all runs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RunStats {
    /// Base seed.
    pub seed: u64,
    /// Number of runs.
    pub runs: u32,
    /// Lost runs by section reached (index 0 = section 1).
    pub lost_in_section: Vec<u64>,
    /// Runs won.
    pub won: u64,
    /// Normal and boss pubs per (section, pub); Spelunken are in
    /// [`RunStats::spelunken`].
    pub series: Vec<[PubStat; 5]>,
    /// Spelunken per (section, pub they replace).
    pub spelunken: Vec<[PubStat; 5]>,
    /// Per district: relics, lost pubs, second chances, where runs ended.
    pub districts: Vec<DistrictStat>,
    /// Attempts and wins per boss in section 1.
    pub bosses: BTreeMap<BossId, (u64, u64)>,
    /// Attempts and wins per boss from section 2 on.
    pub bosses_later: BTreeMap<BossId, (u64, u64)>,
    /// Games played.
    pub games: u64,
    /// Games won.
    pub games_won: u64,
    /// Games declared as Null (ADR 0012: every character may).
    pub null_games: u64,
    /// Null games won.
    pub null_won: u64,
    /// Sum of the series account changes of all games.
    pub points: i64,
    /// Einmischen (games dealt again, also by "Neues Blatt").
    pub redeals: u64,
    /// Groschen held at the end of each run, summed.
    pub coins_end: u64,
    /// Groschen held at the start of each run, summed.
    pub coins_start: u64,
    /// Groschen earned from series, summed.
    pub coins_earned: u64,
    /// Groschen from selling relics and consumables, summed.
    pub coins_sold: u64,
    /// Groschen paid or taken by content (`CoinsGained`: curses, vouchers,
    /// relics, encounters, the pot), summed with sign.
    pub coins_other: i64,
    /// Groschen spent at the Tresen, summed.
    pub coins_spent: u64,
    /// Groschen spent per kind, in the order of [`SPENDING`].
    pub spending: [u64; 7],
    /// Times the Tresen opened.
    pub shops: u64,
    /// Per relic.
    pub relics: BTreeMap<RelicId, RelicStat>,
    /// Encounters opened, by id: (opened, times each option was taken,
    /// by index).
    pub encounters: BTreeMap<EncounterId, (u64, [u64; 4])>,
    /// Times a Kiosk opened.
    pub kiosks: u64,
    /// Highest Spielwert of a won game, summed over runs.
    pub best_game: u64,
    /// Per consumable: (bought, used).
    pub consumables: BTreeMap<ConsumableId, (u64, u64)>,
    /// Games per Spielart, in the order of [`GAME_KINDS`].
    pub kinds: [KindStat; 7],
    /// Vouchers redeemed, by modification id: (times, summed value).
    pub vouchers_redeemed: BTreeMap<CardModId, (u64, i64)>,
    /// Vouchers put back on their card (Drückeberger, DESIGN 9.4).
    pub vouchers_returned: u64,
    /// The Faktor (ADR 0018) of the games won.
    pub factor: FactorStat,
    /// The card reward after a won pub (DESIGN 5.0).
    pub card_rewards: CardRewardStat,
    /// The card reward per Zinke: (offered, taken).
    pub card_reward_zinken: BTreeMap<CardModId, (u64, u64)>,
}

/// The card reward after a won normal pub or Spelunke (DESIGN 5.0), over
/// all runs, for the balance round.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CardRewardStat {
    /// Rewards offered (one per won normal pub or Spelunke).
    pub offered: u64,
    /// Rewards taken (a Zinke on a card).
    pub taken: u64,
    /// Of those, a Zinke raised on a card that had it already.
    pub raised: u64,
    /// Rewards skipped for Groschen.
    pub skipped: u64,
    /// Groschen from skipping, summed.
    pub skip_coins: u64,
}

/// The Faktor (ADR 0018, `ScoreBreakdown::multiplier`) over all runs, for
/// the balance round: how often it is above ×1 and how far it grows.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FactorStat {
    /// Games won.
    pub won_games: u64,
    /// Their factors in thousandths, summed.
    pub won_permille: u64,
    /// Won games with a factor above ×1.
    pub won_above_one: u64,
    /// The highest factor of a won game per run, in thousandths, summed
    /// over the runs (×1 for a run without a won game).
    pub run_max_permille: u64,
    /// The highest factor of a won game in any run, in thousandths.
    pub max_permille: u64,
    /// Runs that ended holding at least one relic of the group Faktor.
    pub runs_with_relic: u64,
    /// Relics of the group Faktor held at the end, summed over runs.
    pub relics_end: u64,
    /// The factor a won game would get at the end of the run from the
    /// growing relics alone ("Zapfhahn", "Heiße Hand"), in thousandths,
    /// summed over the runs that hold one.
    pub growing_permille: u64,
    /// Runs that ended holding a growing Faktor relic.
    pub growing_runs: u64,
}

/// Labels of [`RunStats::kinds`].
pub const GAME_KINDS: [&str; 7] = [
    "clubs", "spades", "hearts", "diamonds", "grand", "null", "ramsch",
];

/// Games of one Spielart.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct KindStat {
    /// Games played.
    pub games: u64,
    /// Games won.
    pub won: u64,
    /// Won games with the defenders Schneider.
    pub won_schneider: u64,
    /// Hand games.
    pub hand: u64,
    /// Sum of the series account changes.
    pub points: i64,
    /// Sum of the Spielwert of the won games.
    pub won_value: u64,
}

/// Index into [`GAME_KINDS`].
fn kind_index(declaration: Declaration) -> usize {
    match declaration {
        Declaration::Suit { suit, .. } => match suit {
            Suit::Clubs => 0,
            Suit::Spades => 1,
            Suit::Hearts => 2,
            Suit::Diamonds => 3,
        },
        Declaration::Grand { .. } => 4,
        Declaration::Null { .. } => 5,
        _ => 6,
    }
}

/// The pub in progress while a run is observed.
#[derive(Debug, Clone)]
struct OpenPub {
    section: usize,
    index: usize,
    spelunke: bool,
    boss: Option<BossId>,
}

/// Follows one run's events into the statistics.
struct Observer<'a> {
    stats: &'a mut RunStats,
    open: Option<OpenPub>,
    /// The last pub entered, kept after it ends (where a run ended).
    last: Option<OpenPub>,
    bought: BTreeSet<RelicId>,
    /// The next pub started is a pub played again ("Schutzengel").
    replay: bool,
    /// The highest factor of a won game in this run, in thousandths.
    max_factor: u32,
    /// The value of the card reward just taken, until its deck change.
    reward_value: Option<i32>,
}

impl Observer<'_> {
    /// The stats of the open pub.
    fn pub_stat(&mut self) -> Option<&mut PubStat> {
        let open = self.open.as_ref()?;
        let table = if open.spelunke {
            &mut self.stats.spelunken
        } else {
            &mut self.stats.series
        };
        table.get_mut(open.section)?.get_mut(open.index)
    }

    fn spend(&mut self, kind: usize, amount: u32) {
        self.stats.coins_spent += u64::from(amount);
        self.stats.spending[kind] += u64::from(amount);
    }

    /// One batch of events, with the run as it is after them (relics when
    /// a pub starts: nothing changes them later in the batch that starts
    /// it).
    fn observe(&mut self, run: &Run, events: &[RunEvent]) {
        let boss_slot = usize::try_from(PUBS_PER_DISTRICT - 1).unwrap_or(4);
        for event in events {
            match event {
                RunEvent::SeriesStarted {
                    section,
                    index,
                    boss,
                    malus,
                    ..
                } => {
                    let open = OpenPub {
                        section: usize::try_from(*section).unwrap_or(usize::MAX),
                        index: usize::try_from(*index).unwrap_or(usize::MAX),
                        spelunke: malus.is_some(),
                        boss: boss.clone(),
                    };
                    let relics = u64::try_from(run.relics().len()).unwrap_or(0);
                    let replay = std::mem::take(&mut self.replay);
                    if let Some(d) = self
                        .stats
                        .districts
                        .get_mut(open.section)
                        .filter(|_| !replay)
                    {
                        if open.index == 0 {
                            d.runs += 1;
                            d.relics_start += relics;
                        } else if open.index == boss_slot {
                            d.boss_runs += 1;
                            d.relics_boss += relics;
                        }
                    }
                    if let Some(boss) = boss.as_ref().filter(|_| !replay) {
                        let table = if *section == 0 {
                            &mut self.stats.bosses
                        } else {
                            &mut self.stats.bosses_later
                        };
                        table.entry(boss.clone()).or_default().0 += 1;
                    }
                    self.open = Some(open.clone());
                    self.last = Some(open);
                    if let Some(p) = self.pub_stat() {
                        p.attempts += 1;
                        p.replays += u64::from(replay);
                        p.relics += relics;
                    }
                }
                RunEvent::SeriesWon { payout, .. } => {
                    if let Some(p) = self.pub_stat() {
                        p.wins += 1;
                        p.payout += u64::from(payout.total());
                    }
                    if let Some(OpenPub {
                        section,
                        boss: Some(boss),
                        ..
                    }) = &self.open
                    {
                        let table = if *section == 0 {
                            &mut self.stats.bosses
                        } else {
                            &mut self.stats.bosses_later
                        };
                        table.entry(boss.clone()).or_default().1 += 1;
                    }
                    self.stats.coins_earned += u64::from(payout.total());
                    self.open = None;
                }
                RunEvent::PubLost { .. } => {
                    if let Some(d) = self
                        .open
                        .as_ref()
                        .and_then(|o| self.stats.districts.get_mut(o.section))
                    {
                        d.lost_pubs += 1;
                    }
                    self.open = None;
                }
                RunEvent::SecondChance { section, .. } => {
                    let section = usize::try_from(*section).unwrap_or(usize::MAX);
                    if let Some(d) = self.stats.districts.get_mut(section) {
                        d.second_chances += 1;
                    }
                    self.replay = true;
                }
                RunEvent::RunLost { .. } => {
                    if let Some(last) = &self.last {
                        let end = match (&last.boss, last.spelunke) {
                            (Some(_), _) => RunEnd::Boss,
                            (None, true) => RunEnd::Spelunke,
                            (None, false) => RunEnd::Pub,
                        };
                        if let Some(d) = self.stats.districts.get_mut(last.section) {
                            d.ended[end as usize] += 1;
                        }
                    }
                }
                RunEvent::GameScored { breakdown, .. } => {
                    self.stats.games += 1;
                    self.stats.points += breakdown.score;
                    if breakdown.won {
                        self.stats.games_won += 1;
                    }
                    let kind = &mut self.stats.kinds[kind_index(breakdown.declaration)];
                    kind.games += 1;
                    kind.points += breakdown.score;
                    kind.hand += u64::from(breakdown.declaration.hand());
                    if breakdown.won {
                        let factor = breakdown.multiplier.permille();
                        let f = &mut self.stats.factor;
                        f.won_games += 1;
                        f.won_permille += u64::from(factor);
                        f.won_above_one += u64::from(factor > 1000);
                        f.max_permille = f.max_permille.max(u64::from(factor));
                        self.max_factor = self.max_factor.max(factor);
                        kind.won += 1;
                        kind.won_value += breakdown.value;
                        kind.won_schneider += u64::from(breakdown.schneider);
                    }
                    if breakdown.declaration.is_null() {
                        self.stats.null_games += 1;
                        self.stats.null_won += u64::from(breakdown.won);
                    }
                    if let Some(p) = self.pub_stat() {
                        p.games += 1;
                    }
                }
                RunEvent::Redealt => {
                    self.stats.redeals += 1;
                    if let Some(p) = self.pub_stat() {
                        p.redeals += 1;
                    }
                }
                RunEvent::RelicBought { relic, price } => {
                    self.stats.relics.entry(relic.clone()).or_default().bought += 1;
                    self.spend(0, *price);
                    self.bought.insert(relic.clone());
                }
                RunEvent::ConsumableBought { consumable, price } => {
                    self.spend(1, *price);
                    self.stats
                        .consumables
                        .entry(consumable.clone())
                        .or_default()
                        .0 += 1;
                }
                RunEvent::ConsumableUsed { consumable } => {
                    self.stats
                        .consumables
                        .entry(consumable.clone())
                        .or_default()
                        .1 += 1;
                }
                RunEvent::RelicSold { price, .. } | RunEvent::ConsumableSold { price, .. } => {
                    self.stats.coins_sold += u64::from(*price);
                }
                RunEvent::CoinsGained { amount, .. } => {
                    self.stats.coins_other += i64::from(*amount);
                }
                RunEvent::VoucherRedeemed {
                    card_mod, value, ..
                } => {
                    let entry = self
                        .stats
                        .vouchers_redeemed
                        .entry(card_mod.clone())
                        .or_default();
                    entry.0 += 1;
                    entry.1 += i64::from(*value);
                }
                RunEvent::VoucherReturned { .. } => self.stats.vouchers_returned += 1,
                RunEvent::ShopOpened { .. } => self.stats.shops += 1,
                RunEvent::EncounterOpened { encounter, .. } => {
                    self.stats
                        .encounters
                        .entry(encounter.clone())
                        .or_default()
                        .0 += 1;
                }
                RunEvent::EncounterResolved { encounter, option } => {
                    let taken = &mut self
                        .stats
                        .encounters
                        .entry(encounter.clone())
                        .or_default()
                        .1;
                    if let Some(n) = taken.get_mut(*option) {
                        *n += 1;
                    }
                }
                RunEvent::KioskOpened { .. } => self.stats.kiosks += 1,
                // The card reward (DESIGN 5.0).
                RunEvent::CardRewardOffered { offers } => {
                    self.stats.card_rewards.offered += 1;
                    for (card_mod, _) in offers {
                        self.stats
                            .card_reward_zinken
                            .entry(card_mod.clone())
                            .or_default()
                            .0 += 1;
                    }
                }
                RunEvent::CardRewardTaken {
                    card_mod, value, ..
                } => {
                    self.reward_value = Some(*value);
                    self.stats.card_rewards.taken += 1;
                    self.stats
                        .card_reward_zinken
                        .entry(card_mod.clone())
                        .or_default()
                        .1 += 1;
                }
                // The change of a taken card reward: above the offered value
                // it was raised.
                RunEvent::DeckChanged(DeckChange::Modified { value, .. })
                    if self.reward_value.is_some() =>
                {
                    let offered = self.reward_value.take().unwrap_or(0);
                    self.stats.card_rewards.raised += u64::from(*value > offered);
                }
                RunEvent::CardRewardSkipped { coins } => {
                    self.stats.card_rewards.skipped += 1;
                    self.stats.card_rewards.skip_coins += u64::from(*coins);
                }
                // Zinken at the Tresen (DESIGN 8.6).
                RunEvent::CardOfferBought { price, .. } => self.spend(2, *price),
                RunEvent::PainterHired { price, .. } => self.spend(3, *price),
                RunEvent::CardCleansed { price, .. } => self.spend(4, *price),
                RunEvent::CardOffersRerolled { cost } => self.spend(5, *cost),
                RunEvent::ShopRerolled { cost } => self.spend(6, *cost),
                _ => {}
            }
        }
    }
}

/// Play one run to the end and add it to the statistics.
fn play_run(
    content: &Content,
    options: &Options,
    seed: u64,
    stats: &mut RunStats,
) -> Result<(), String> {
    let start = options.start();
    let (mut run, events) = Run::new_with(content, seed, &start).map_err(|e| e.to_string())?;
    stats.coins_start += u64::from(run.coins());
    let mut observer = Observer {
        stats,
        open: None,
        last: None,
        bought: BTreeSet::new(),
        replay: false,
        max_factor: 1000,
        reward_value: None,
    };
    observer.observe(&run, &events);
    let mut actions = 0;
    while let Some(action) = options.next_action(&run, content) {
        let events = run.apply(content, action).map_err(|e| e.to_string())?;
        observer.observe(&run, &events);
        actions += 1;
        if actions > MAX_ACTIONS {
            return Err("run does not end".into());
        }
    }
    let bought = std::mem::take(&mut observer.bought);
    let max_factor = observer.max_factor;
    observe_factor_relics(&run, content, max_factor, &mut stats.factor);
    let sections = usize::try_from(content.run.sections()).unwrap_or(1);
    let won = run.phase() == RunPhase::Won;
    let section = usize::try_from(run.series().section).unwrap_or(0);
    if won {
        stats.won += 1;
    } else if let Some(n) = stats.lost_in_section.get_mut(section.min(sections - 1)) {
        *n += 1;
    }
    stats.coins_end += u64::from(run.coins());
    stats.best_game += run.stats().best_game;
    for relic in bought {
        let entry = stats.relics.entry(relic).or_default();
        entry.runs += 1;
        entry.runs_won += u64::from(won);
        entry.sections += u64::try_from(section + 1).unwrap_or(0);
    }
    Ok(())
}

/// The Faktor relics a run ends with (ADR 0018): how many, and the factor
/// the growing ones ("Zapfhahn", "Heiße Hand": `ScaleGameValuePerCounter`
/// on a won game) would give the next won game.
fn observe_factor_relics(run: &Run, content: &Content, max_factor: u32, stat: &mut FactorStat) {
    stat.run_max_permille += u64::from(max_factor);
    let defs: Vec<_> = run
        .relics()
        .iter()
        .filter_map(|id| content.relic(id))
        .filter(|d| d.group == RelicGroup::Factor)
        .collect();
    stat.relics_end += u64::try_from(defs.len()).unwrap_or(0);
    stat.runs_with_relic += u64::from(!defs.is_empty());
    let mut growing = false;
    let mut permille: u64 = 1000;
    for effect in defs
        .iter()
        .flat_map(|d| d.hooks.iter().flat_map(Hook::then))
    {
        if let Effect::ScaleGameValuePerCounter { counter, per } = effect {
            growing = true;
            let value = u64::from(run.counters().get(counter).copied().unwrap_or(0));
            let factor = 1000 + u64::from(per.permille()) * value;
            permille = (permille * factor + 500) / 1000;
        }
    }
    if growing {
        stat.growing_runs += 1;
        stat.growing_permille += permille;
    }
}

/// Play `runs` runs. Run seeds come from one master generator, so the same
/// seed gives the same runs with and without options (same deals).
pub fn simulate(
    content: &Content,
    options: &Options,
    seed: u64,
    runs: u32,
) -> Result<RunStats, String> {
    let sections = usize::try_from(content.run.sections()).unwrap_or(1);
    let mut stats = RunStats {
        seed,
        runs,
        lost_in_section: vec![0; sections],
        series: vec![[PubStat::default(); 5]; sections],
        spelunken: vec![[PubStat::default(); 5]; sections],
        districts: vec![DistrictStat::default(); sections],
        ..RunStats::default()
    };
    // Every relic appears in the table, also if it was never bought.
    for id in content.relics.keys() {
        stats.relics.insert(id.clone(), RelicStat::default());
    }
    for id in content.consumables.keys() {
        stats.consumables.insert(id.clone(), (0, 0));
    }
    let mut master = Rng::from_seed(seed);
    for i in 0..runs {
        let run_seed = master.next_u64();
        play_run(content, options, run_seed, &mut stats).map_err(|e| format!("run {i}: {e}"))?;
    }
    Ok(stats)
}

fn pct(part: u64, whole: u64) -> f64 {
    if whole == 0 {
        0.0
    } else {
        100.0 * part as f64 / whole as f64
    }
}

fn mean(sum: u64, n: u64) -> f64 {
    if n == 0 { 0.0 } else { sum as f64 / n as f64 }
}

fn mean_signed(sum: i64, n: u64) -> f64 {
    if n == 0 { 0.0 } else { sum as f64 / n as f64 }
}

/// Render the statistics as a text table.
pub fn render(stats: &RunStats, content: &Content, options: &Options, runtime_us: u64) -> String {
    let runs = u64::from(stats.runs);
    let mut out = String::new();
    let _ = writeln!(
        out,
        "skat_sim seed={} runs={} options: {}",
        stats.seed,
        stats.runs,
        options.describe()
    );
    let _ = write!(out, "section reached:");
    for (i, n) in stats.lost_in_section.iter().enumerate() {
        let _ = write!(out, " {}={}", i + 1, n);
    }
    let _ = writeln!(out, " won={}", stats.won);
    // Share of runs that reached a section (lost there or later, or won).
    let _ = write!(out, "reached at least:");
    for section in 1..stats.lost_in_section.len() {
        let later: u64 = stats.lost_in_section[section..].iter().sum::<u64>() + stats.won;
        let _ = write!(out, " {}={:.1}%", section + 1, pct(later, runs));
    }
    out.push('\n');
    render_pubs(&mut out, stats, content, options);
    render_districts(&mut out, stats);
    for (label, table) in [
        ("bosses section 1:", &stats.bosses),
        ("bosses section 2+:", &stats.bosses_later),
    ] {
        let _ = write!(out, "{label}");
        for (boss, (attempts, wins)) in table {
            let _ = write!(
                out,
                " {boss}={wins}/{attempts} ({:.1}%)",
                pct(*wins, *attempts)
            );
        }
        out.push('\n');
    }
    let _ = writeln!(
        out,
        "games: {} won%={:.1} redeals={} points per game={:.1}",
        stats.games,
        pct(stats.games_won, stats.games),
        stats.redeals,
        mean_signed(stats.points, stats.games)
    );
    let _ = writeln!(
        out,
        "null games: {} ({:.1}% of games) won%={:.1}",
        stats.null_games,
        pct(stats.null_games, stats.games),
        pct(stats.null_won, stats.null_games)
    );
    render_kinds(&mut out, stats, runs);
    render_factor(&mut out, stats, runs);
    let _ = writeln!(out, "best game per run: {:.1}", mean(stats.best_game, runs));
    let _ = write!(out, "encounters (opened/taken per option):");
    for (id, (opened, taken)) in &stats.encounters {
        let taken: Vec<String> = taken.iter().map(u64::to_string).collect();
        let _ = write!(out, " {id}={opened}/{}", taken.join("+"));
    }
    out.push('\n');
    let _ = writeln!(
        out,
        "groschen per run: earned {:.2}, spent {:.2}, held at end {:.2}",
        mean(stats.coins_earned, runs),
        mean(stats.coins_spent, runs),
        mean(stats.coins_end, runs)
    );
    render_groschen(&mut out, stats);
    render_card_rewards(&mut out, stats, runs);
    let _ = write!(out, "consumables (bought/used):");
    for (id, (bought, used)) in &stats.consumables {
        let _ = write!(out, " {id}={bought}/{used}");
    }
    out.push('\n');
    let _ = writeln!(
        out,
        "{:<18} {:>7} {:>6} {:>9} {:>13}",
        "relic", "bought", "runs", "run win%", "mean section"
    );
    for (id, r) in &stats.relics {
        let _ = writeln!(
            out,
            "{:<18} {:>7} {:>6} {:>9.1} {:>13.2}",
            id.as_str(),
            r.bought,
            r.runs,
            pct(r.runs_won, r.runs),
            mean(r.sections, r.runs)
        );
    }
    let per_run = if stats.runs == 0 {
        0.0
    } else {
        runtime_us as f64 / f64::from(stats.runs)
    };
    let _ = writeln!(
        out,
        "runtime: {:.1} ms total, {:.2} ms per run",
        runtime_us as f64 / 1000.0,
        per_run / 1000.0
    );
    out
}

/// The card reward after a won pub (DESIGN 5.0): offered, taken (raised),
/// skipped per run, and per Zinke how often it was offered and taken.
fn render_card_rewards(out: &mut String, stats: &RunStats, runs: u64) {
    let c = &stats.card_rewards;
    let _ = writeln!(
        out,
        "card rewards per run: offered {:.2}, taken {:.2} (raised {:.2}), skipped {:.2} ({:.1}% of offered, {:.2} Groschen)",
        mean(c.offered, runs),
        mean(c.taken, runs),
        mean(c.raised, runs),
        mean(c.skipped, runs),
        pct(c.skipped, c.offered),
        mean(c.skip_coins, runs)
    );
    let _ = write!(out, "card reward zinken (offered/taken):");
    for (id, (offered, taken)) in &stats.card_reward_zinken {
        let _ = write!(out, " {id}={offered}/{taken}");
    }
    out.push('\n');
}

/// The Faktor (ADR 0018): mean factor of the won games, the share above
/// ×1, the highest per run and overall, runs with a Faktor relic at the
/// end and the factor of the growing ones.
fn render_factor(out: &mut String, stats: &RunStats, runs: u64) {
    let f = &stats.factor;
    let _ = writeln!(
        out,
        "faktor: won games mean x{:.3}, above x1 {:.1}%, max per run mean x{:.3}, max x{:.3}; \
         runs with a faktor relic at the end {:.1}% ({:.2} relics per run), \
         growing relics at the end x{:.3} in {} runs",
        mean(f.won_permille, f.won_games) / 1000.0,
        pct(f.won_above_one, f.won_games),
        mean(f.run_max_permille, runs) / 1000.0,
        f.max_permille as f64 / 1000.0,
        pct(f.runs_with_relic, runs),
        mean(f.relics_end, runs),
        mean(f.growing_permille, f.growing_runs) / 1000.0,
        f.growing_runs
    );
}

/// Games per Spielart (share, win rate, Schneider among the won, hand
/// games, points) and the vouchers redeemed and returned per run.
fn render_kinds(out: &mut String, stats: &RunStats, runs: u64) {
    let _ = writeln!(
        out,
        "{:<9} {:>7} {:>7} {:>6} {:>12} {:>6} {:>8} {:>10}",
        "kind", "games", "share%", "win%", "schneider%", "hand%", "points", "value won"
    );
    for (label, k) in GAME_KINDS.iter().zip(&stats.kinds) {
        let _ = writeln!(
            out,
            "{:<9} {:>7} {:>7.1} {:>6.1} {:>12.1} {:>6.1} {:>8.1} {:>10.1}",
            label,
            k.games,
            pct(k.games, stats.games),
            pct(k.won, k.games),
            pct(k.won_schneider, k.won),
            pct(k.hand, k.games),
            mean_signed(k.points, k.games),
            mean(k.won_value, k.won)
        );
    }
    let _ = write!(
        out,
        "vouchers per run: returned {:.2}",
        mean(stats.vouchers_returned, runs)
    );
    for (id, (n, value)) in &stats.vouchers_redeemed {
        let _ = write!(
            out,
            " {id}={:.2} (value {:.2})",
            mean(*n, runs),
            mean_signed(*value, runs)
        );
    }
    out.push('\n');
}

/// The pub table: normal and boss pubs, then the Spelunken, each row with
/// attempts (pubs played again by the "Schutzengel" included and shown
/// apart), wins, games and Einmischen per pub, relics on entering and the
/// payout of a won one; then the games per pub by kind.
fn render_pubs(out: &mut String, stats: &RunStats, content: &Content, options: &Options) {
    let _ = writeln!(
        out,
        "{:<10} {:>7} {:>9} {:>7} {:>6} {:>8} {:>6} {:>7} {:>9}",
        "pub", "target", "attempts", "win%", "games", "redeals", "again", "relics", "groschen"
    );
    let boss_slot = usize::try_from(PUBS_PER_DISTRICT - 1).unwrap_or(4);
    let factor = content.run.spelunke.target_factor;
    let mut kinds = [
        (PubStat::default(), "normal"),
        (PubStat::default(), "boss"),
        (PubStat::default(), "spelunke"),
    ];
    for (spelunke, table) in [(false, &stats.series), (true, &stats.spelunken)] {
        for (section, row) in table.iter().enumerate() {
            for (index, p) in row.iter().enumerate() {
                if spelunke && p.attempts == 0 {
                    continue;
                }
                // The character's own Zielwerte if it has some, before boss
                // factors; a Spelunke's with its factor, without Wegbier.
                let target = u32::try_from(section)
                    .ok()
                    .zip(u32::try_from(index).ok())
                    .and_then(|(row, index)| {
                        series_target(content, options.character, row, index, None)
                    })
                    .unwrap_or(0);
                let target = if spelunke {
                    factor.apply(target)
                } else {
                    target
                };
                let label = match (spelunke, index == boss_slot) {
                    (true, _) => format!("{}-{} sp", section + 1, index + 1),
                    (false, true) => format!("{}-boss", section + 1),
                    (false, false) => format!("{}-{}", section + 1, index + 1),
                };
                let n = p.attempts;
                let _ = writeln!(
                    out,
                    "{:<10} {:>7} {:>9} {:>7.1} {:>6.2} {:>8.2} {:>6} {:>7.2} {:>9.2}",
                    label,
                    target,
                    n,
                    pct(p.wins, n),
                    mean(p.games, n),
                    mean(p.redeals, n),
                    p.replays,
                    mean(p.relics, n),
                    mean(p.payout, p.wins)
                );
                let kind = match (spelunke, index == boss_slot) {
                    (true, _) => 2,
                    (false, true) => 1,
                    (false, false) => 0,
                };
                let k = &mut kinds[kind].0;
                k.attempts += p.attempts;
                k.wins += p.wins;
                k.games += p.games;
                k.payout += p.payout;
            }
        }
    }
    let _ = write!(out, "games per pub:");
    for (k, name) in &kinds {
        let _ = write!(
            out,
            " {name}={:.2} (win%={:.1}, groschen per won={:.2})",
            mean(k.games, k.attempts),
            pct(k.wins, k.attempts),
            mean(k.payout, k.wins)
        );
    }
    out.push('\n');
}

/// Per district: relics at its start and before the boss, lost pubs per
/// run in it, pubs played again ("Schutzengel"), and where the lost runs
/// ended.
fn render_districts(out: &mut String, stats: &RunStats) {
    let runs = u64::from(stats.runs);
    let _ = writeln!(
        out,
        "{:<9} {:>6} {:>13} {:>13} {:>10} {:>8} {:>9} {:>9} {:>9}",
        "district",
        "runs",
        "relics start",
        "relics boss",
        "lost pubs",
        "again",
        "end pub",
        "end sp",
        "end boss"
    );
    let mut ended = [0u64; 3];
    for (i, d) in stats.districts.iter().enumerate() {
        let _ = writeln!(
            out,
            "{:<9} {:>6} {:>13.2} {:>13.2} {:>10.2} {:>8} {:>9} {:>9} {:>9}",
            i + 1,
            d.runs,
            mean(d.relics_start, d.runs),
            mean(d.relics_boss, d.boss_runs),
            mean(d.lost_pubs, d.runs),
            d.second_chances,
            d.ended[RunEnd::Pub as usize],
            d.ended[RunEnd::Spelunke as usize],
            d.ended[RunEnd::Boss as usize]
        );
        for (sum, n) in ended.iter_mut().zip(d.ended) {
            *sum += n;
        }
    }
    let _ = writeln!(
        out,
        "runs end: normal pub={} ({:.1}%) spelunke={} ({:.1}%) boss pub={} ({:.1}%) won={} ({:.1}%)",
        ended[0],
        pct(ended[0], runs),
        ended[1],
        pct(ended[1], runs),
        ended[2],
        pct(ended[2], runs),
        stats.won,
        pct(stats.won, runs)
    );
}

/// Groschen income and spending per run, spending by kind with its share.
fn render_groschen(out: &mut String, stats: &RunStats) {
    let runs = u64::from(stats.runs);
    let income = i64::try_from(stats.coins_start + stats.coins_earned + stats.coins_sold)
        .unwrap_or(i64::MAX)
        + stats.coins_other;
    // What is neither Tresen nor held at the end: encounter costs, wagers,
    // debt.
    let rest = income - i64::try_from(stats.coins_spent + stats.coins_end).unwrap_or(i64::MAX);
    let _ = writeln!(
        out,
        "groschen income per run: start {:.2}, pubs {:.2}, sold {:.2}, content {:.2}; paid outside the Tresen (encounters, debt) {:.2}",
        mean(stats.coins_start, runs),
        mean(stats.coins_earned, runs),
        mean(stats.coins_sold, runs),
        mean_signed(stats.coins_other, runs),
        mean_signed(rest, runs)
    );
    let _ = write!(out, "groschen spent per run (share):");
    for (name, amount) in SPENDING.iter().zip(stats.spending) {
        let _ = write!(
            out,
            " {name}={:.2} ({:.1}%)",
            mean(amount, runs),
            pct(amount, stats.coins_spent)
        );
    }
    let _ = writeln!(
        out,
        "; tresen visits={:.2}, kiosk visits={:.2}",
        mean(stats.shops, runs),
        mean(stats.kiosks, runs)
    );
}

// ---------------------------------------------------------------------------
// Success curve of one series
// ---------------------------------------------------------------------------

/// Success curve of one series (`--curve`): how often the bot wins a
/// series of the game limit and Einmischen in `run.ron`, for every Zielwert.
///
/// Every sample is one series with an unreachable Zielwert, played to the
/// game limit. The bot never looks at the Zielwert or the account, so it
/// plays exactly as it would against any target, and a series with
/// Zielwert T is won exactly when the highest account after a game reaches
/// T. One sample thus gives the result for every T. With `--boss` the
/// series is a boss pub with that boss: two boss players, all games
/// played, won when the account after the last game reaches T (ADR 0017).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Curve {
    /// Base seed.
    pub seed: u64,
    /// Number of series.
    pub series: u32,
    /// Highest account per series (`None`: never above 0 after a game);
    /// for a boss pub the account after its last game.
    pub best: Vec<Option<i64>>,
    /// Games played.
    pub games: u64,
    /// Games won.
    pub games_won: u64,
    /// Sum of the account changes of all games.
    pub points: i64,
    /// Einmischen (games dealt again).
    pub redeals: u64,
}

impl Curve {
    /// Share of series won at Zielwert `target`, in percent.
    #[must_use]
    pub fn win_pct(&self, target: u64) -> f64 {
        let won = self
            .best
            .iter()
            .flatten()
            .filter(|&&best| i128::from(best) >= i128::from(target))
            .count();
        pct(u64::try_from(won).unwrap_or(0), u64::from(self.series))
    }
}

/// Zielwerte the curve reports: 1 and every Zielwert of `run.ron`.
fn curve_targets(content: &Content) -> Vec<u64> {
    let mut targets: Vec<u64> = std::iter::once(1)
        .chain(content.run.targets.iter().flatten().copied())
        .collect();
    targets.sort_unstable();
    targets.dedup();
    targets
}

/// Measure the success curve over `series` series.
pub fn curve(
    content: &Content,
    options: &Options,
    seed: u64,
    series: u32,
) -> Result<Curve, String> {
    let mut content = content.clone();
    let unreachable = u64::MAX / 4;
    content
        .run
        .targets
        .iter_mut()
        .for_each(|row| row.fill(unreachable));
    // A character with its own Zielwerte (characters.ron) plays those.
    for def in content.characters.values_mut() {
        if let Some(targets) = def.targets.as_mut() {
            targets.iter_mut().for_each(|row| row.fill(unreachable));
        }
    }
    let start = RunStart {
        section: options.district.map_or(0, |d| d.saturating_sub(1)),
        index: if options.boss.is_some() {
            PUBS_PER_DISTRICT - 1
        } else {
            0
        },
        ..options.start()
    };
    let mut result = Curve {
        seed,
        series,
        ..Curve::default()
    };
    let boss_pub = options.boss.is_some();
    let mut master = Rng::from_seed(seed);
    for i in 0..series {
        let run_seed = master.next_u64();
        let (mut run, _) =
            Run::new_with(&content, run_seed, &start).map_err(|e| format!("series {i}: {e}"))?;
        let mut best: Option<i64> = None;
        let mut actions = 0;
        while run.phase() == RunPhase::Game {
            let Some(action) = options.next_action(&run, &content) else {
                break;
            };
            for event in run
                .apply(&content, action)
                .map_err(|e| format!("series {i}: {e}"))?
            {
                match event {
                    RunEvent::GameScored {
                        breakdown,
                        series_score,
                        ..
                    } => {
                        result.games += 1;
                        result.points += breakdown.score;
                        result.games_won += u64::from(breakdown.won);
                        best = Some(match (boss_pub, best) {
                            (false, Some(b)) => b.max(series_score),
                            _ => series_score,
                        });
                    }
                    RunEvent::Redealt => result.redeals += 1,
                    _ => {}
                }
            }
            actions += 1;
            if actions > MAX_ACTIONS {
                return Err(format!("series {i} does not end"));
            }
        }
        result.best.push(best);
    }
    Ok(result)
}

/// Render the success curve.
pub fn render_curve(
    curve: &Curve,
    content: &Content,
    options: &Options,
    runtime_us: u64,
) -> String {
    let mut out = String::new();
    let kind = if options.boss.is_some() {
        "boss pubs (two boss players, every game played)"
    } else {
        "series"
    };
    let _ = writeln!(
        out,
        "skat_sim seed={} curve: {} {kind}, game limit {}, redeals per game {}, options: {}",
        curve.seed,
        curve.series,
        content.run.game_limit,
        content.run.redeals,
        options.describe()
    );
    let _ = writeln!(out, "{:>7} {:>7}", "target", "win%");
    for target in curve_targets(content) {
        let _ = writeln!(out, "{:>7} {:>7.1}", target, curve.win_pct(target));
    }
    let n = u64::from(curve.series);
    let _ = writeln!(
        out,
        "games: {} won%={:.1} redeals={} points per game={:.1} games per series={:.2}",
        curve.games,
        pct(curve.games_won, curve.games),
        curve.redeals,
        mean_signed(curve.points, curve.games),
        mean(curve.games, n)
    );
    let _ = writeln!(out, "runtime: {:.1} ms total", runtime_us as f64 / 1000.0);
    out
}

/// Entry point of the runs mode.
pub fn main(seed: u64, runs: u32, options: &Options) -> ExitCode {
    let content = match Content::load()
        .map_err(|e| e.to_string())
        .and_then(|base| options.content(&base))
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };
    let start = Instant::now();
    let output = if options.unlocks {
        unlocks(&content, options, seed, runs).map(|u| render_unlocks(&u, &content))
    } else if options.curve {
        curve(&content, options, seed, runs).map(|c| {
            let runtime_us = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);
            render_curve(&c, &content, options, runtime_us)
        })
    } else {
        simulate(&content, options, seed, runs).map(|s| {
            let runtime_us = u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);
            render(&s, &content, options, runtime_us)
        })
    };
    match output {
        Ok(text) => {
            print!("{text}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

// ---------------------------------------------------------------------------
// Reachability of the unlocks
// ---------------------------------------------------------------------------

/// Sequences of runs played by `--unlocks`, each with a fresh meta state.
pub const UNLOCK_SEQUENCES: u32 = 20;

/// How many runs a fresh player (the bot) needs for each unlock.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Unlocks {
    /// Base seed.
    pub seed: u64,
    /// Sequences played.
    pub sequences: u32,
    /// Most runs per sequence.
    pub max_runs: u32,
    /// Per unlock: the run number (from 1) at which each sequence earned
    /// it; sequences that never earned it are missing.
    pub earned_at: BTreeMap<UnlockId, Vec<u32>>,
}

/// Play [`UNLOCK_SEQUENCES`] sequences of up to `max_runs` runs. Every
/// sequence starts with a fresh [`MetaState`]; each run starts from it
/// (locked relics stay off the Tresen) and its events feed it. The bot
/// plays the Alleinspieler and, once unlocked, every other run the
/// Nullspieler. A sequence stops when everything is unlocked.
pub fn unlocks(
    content: &Content,
    options: &Options,
    seed: u64,
    max_runs: u32,
) -> Result<Unlocks, String> {
    let mut result = Unlocks {
        seed,
        sequences: UNLOCK_SEQUENCES,
        max_runs,
        ..Unlocks::default()
    };
    for id in content.unlocks.keys() {
        result.earned_at.insert(id.clone(), Vec::new());
    }
    let mut master = Rng::from_seed(seed);
    for sequence in 0..UNLOCK_SEQUENCES {
        let mut meta = MetaState::new();
        for n in 1..=max_runs {
            if meta.unlocked.len() == content.unlocks.len() {
                break;
            }
            let character = if n % 2 == 0 && meta.character_unlocked(content, Character::NullPlayer)
            {
                Character::NullPlayer
            } else {
                Character::Declarer
            };
            let mut start = meta
                .run_start_with(content, character, options.length)
                .map_err(|e| e.to_string())?;
            start.relics.clone_from(&options.relics);
            start.stake = options.stake;
            let run_seed = master.next_u64();
            let (mut run, mut events) =
                Run::new_with(content, run_seed, &start).map_err(|e| e.to_string())?;
            let mut actions = 0;
            while let Some(action) = options.next_action(&run, content) {
                events.extend(run.apply(content, action).map_err(|e| e.to_string())?);
                actions += 1;
                if actions > MAX_ACTIONS {
                    return Err(format!("sequence {sequence}: run does not end"));
                }
            }
            for id in meta.observe(content, &events) {
                result.earned_at.entry(id).or_default().push(n);
            }
        }
    }
    Ok(result)
}

/// Render the reachability table.
pub fn render_unlocks(unlocks: &Unlocks, content: &Content) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "skat_sim seed={} unlocks: {} sequences of at most {} runs, fresh meta state each",
        unlocks.seed, unlocks.sequences, unlocks.max_runs
    );
    let _ = writeln!(
        out,
        "{:<20} {:>8} {:>10} {:>8} {:>8}  reward",
        "unlock", "reached", "mean runs", "min", "max"
    );
    for (id, at) in &unlocks.earned_at {
        let reached = at.len();
        let mean_runs = if reached == 0 {
            0.0
        } else {
            at.iter().map(|&n| f64::from(n)).sum::<f64>() / reached as f64
        };
        let rewards = content
            .unlock(id)
            .map(|u| format!("{:?}", u.rewards))
            .unwrap_or_default();
        let _ = writeln!(
            out,
            "{:<20} {:>5}/{:<2} {:>10.1} {:>8} {:>8}  {rewards}",
            id.as_str(),
            reached,
            unlocks.sequences,
            mean_runs,
            at.iter().min().copied().unwrap_or(0),
            at.iter().max().copied().unwrap_or(0),
        );
    }
    out
}

/// The content with heuristic defenders everywhere: the option tests
/// check plumbing, and PIMC in an unoptimized test build is slow.
#[cfg(test)]
fn heuristic_content() -> Content {
    let mut content = Content::load().expect("content");
    content.run.opponents.clear();
    content
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_gives_identical_run_statistics() {
        let content = heuristic_content();
        let options = Options::default();
        let a = simulate(&content, &options, 42, 30).expect("runs");
        let b = simulate(&content, &options, 42, 30).expect("runs");
        assert_eq!(a, b);
        assert_eq!(
            render(&a, &content, &options, 0),
            render(&b, &content, &options, 0)
        );
        assert_ne!(simulate(&content, &options, 43, 30).expect("runs"), a);
    }

    #[test]
    fn every_run_ends_once() {
        let content = heuristic_content();
        let stats = simulate(&content, &Options::default(), 7, 25).expect("runs");
        let ended: u64 = stats.lost_in_section.iter().sum::<u64>() + stats.won;
        assert_eq!(ended, 25);
        assert_eq!(
            stats.series[0][0].attempts, 25,
            "every run starts with series 1-1"
        );
    }
}

#[cfg(test)]
mod option_tests {
    use super::*;

    fn content() -> Content {
        heuristic_content()
    }

    #[test]
    fn pimc_defenders_and_bot_play_whole_runs_deterministically() {
        let base = Content::load().expect("content");
        let options = Options {
            bot: Opponents::Pimc {
                samples: 2,
                depth: 1,
            },
            defenders: Some(Opponents::Pimc {
                samples: 2,
                depth: 1,
            }),
            ..Options::default()
        };
        let content = options.content(&base).expect("content");
        assert_eq!(content.run.opponents.len(), 1);
        let a = simulate(&content, &options, 4, 3).expect("runs");
        assert_eq!(a, simulate(&content, &options, 4, 3).expect("runs"));
        assert_eq!(a.lost_in_section.iter().sum::<u64>() + a.won, 3);
        let text = render(&a, &content, &options, 0);
        assert!(text.contains("bot=pimc:2:1") && text.contains("defenders=pimc:2:1"));
    }

    #[test]
    fn no_shop_never_buys_and_without_never_offers() {
        let base = content();
        let options = Options {
            no_shop: true,
            ..Options::default()
        };
        let stats = simulate(&base, &options, 3, 40).expect("runs");
        assert_eq!(stats.coins_spent, 0);
        assert!(stats.relics.values().all(|r| r.bought == 0));
        let options = Options {
            without: vec![RelicId::new("old_lady")],
            ..Options::default()
        };
        let content = options.content(&base).expect("content");
        let stats = simulate(&content, &options, 3, 40).expect("runs");
        assert!(!stats.relics.contains_key(&RelicId::new("old_lady")));
        let bad = Options {
            without: vec![RelicId::new("nope")],
            ..Options::default()
        };
        assert!(bad.content(&base).is_err());
        let bad = Options {
            relics: vec![RelicId::new("nope")],
            ..Options::default()
        };
        assert!(bad.content(&base).is_err());
    }

    #[test]
    fn consumable_option_supplies_only_that_consumable() {
        let base = content();
        let options = Options {
            consumable: Some(ConsumableId::new("double_grain")),
            ..Options::default()
        };
        let content = options.content(&base).expect("content");
        assert_eq!(content.run.shop.relic_offers, 0);
        let stats = simulate(&content, &options, 6, 30).expect("runs");
        assert!(stats.relics.values().all(|r| r.bought == 0));
        assert_eq!(stats.coins_spent, 0, "free, and no Neuauslage");
        let (bought, used) = stats.consumables[&ConsumableId::new("double_grain")];
        assert!(bought > 0 && used > 0, "{bought}/{used}");
        assert!(
            stats
                .consumables
                .iter()
                .filter(|(id, _)| id.as_str() != "double_grain")
                .all(|(_, &(b, u))| b == 0 && u == 0)
        );
        assert!(render(&stats, &content, &options, 0).contains("consumable=double_grain"));
        let bad = Options {
            consumable: Some(ConsumableId::new("nope")),
            ..Options::default()
        };
        assert!(bad.content(&base).is_err());
    }

    #[test]
    fn boss_option_draws_only_that_boss() {
        let base = content();
        let options = Options {
            boss: Some(BossId::new("grandma")),
            ..Options::default()
        };
        let content = options.content(&base).expect("content");
        let stats = simulate(&content, &options, 5, 60).expect("runs");
        assert!(!stats.bosses.is_empty());
        assert!(
            stats
                .bosses
                .keys()
                .chain(stats.bosses_later.keys())
                .all(|b| b.as_str() == "grandma")
        );
    }

    #[test]
    fn start_relics_keep_the_deals() {
        let base = content();
        let plain = simulate(&base, &Options::default(), 8, 20).expect("runs");
        let options = Options {
            relics: vec![RelicId::new("crib_sheet")],
            no_shop: true,
            ..Options::default()
        };
        let with = simulate(&base, &options, 8, 20).expect("runs");
        assert_eq!(with.runs, plain.runs);
        assert!(render(&with, &base, &options, 0).contains("relic=crib_sheet"));
    }

    #[test]
    fn card_mods_go_on_the_start_deck_and_must_fit() {
        let base = content();
        let options = Options {
            card_mods: vec![(
                "CA".parse().expect("card"),
                CardModId::new("points"),
                Some(5),
            )],
            no_shop: true,
            ..Options::default()
        };
        let content = options.content(&base).expect("content");
        let deck = &content.character(Character::Declarer).expect("def").deck;
        assert!(
            deck.mods
                .iter()
                .any(|(_, id, value)| id.as_str() == "points" && *value == 5)
        );
        let stats = simulate(&base, &options, 8, 10).expect("runs");
        assert_eq!(stats.runs, 10);
        assert!(render(&stats, &base, &options, 0).contains("card-mod=CA:points:5"));
        let wrong = Options {
            card_mods: vec![(
                "CT".parse().expect("card"),
                CardModId::new("glass_ace"),
                None,
            )],
            ..Options::default()
        };
        assert!(
            wrong.content(&base).is_err(),
            "the Glasdaus is gone with effects v2"
        );
    }

    #[test]
    fn a_stake_makes_suit_games_harder_to_win() {
        let base = content();
        let plain = simulate(&base, &Options::default(), 11, 10).expect("plain");
        let options = Options {
            stake: 25,
            ..Options::default()
        };
        assert_eq!(options.start().stake, 25);
        let staked = simulate(&base, &options, 11, 10).expect("staked");
        // Same seeds, 86 Augen to win a suit game or Grand: fewer won.
        assert!(staked.games_won - staked.null_won < plain.games_won - plain.null_won);
        assert!(render(&staked, &base, &options, 0).contains("stake=25"));
    }

    #[test]
    fn curve_falls_with_the_target_and_is_deterministic() {
        let base = content();
        let options = Options::default();
        let a = curve(&base, &options, 42, 200).expect("curve");
        assert_eq!(a, curve(&base, &options, 42, 200).expect("curve"));
        assert_eq!(a.best.len(), 200);
        let targets = curve_targets(&base);
        assert_eq!(targets[0], 1);
        let pcts: Vec<f64> = targets.iter().map(|&t| a.win_pct(t)).collect();
        assert!(pcts.windows(2).all(|w| w[0] >= w[1]), "{pcts:?}");
        assert!(pcts[0] > 0.0 && pcts[0] < 100.0);
        // The same seed in the runs mode wins series 1-1 exactly when the
        // curve reaches its Zielwert.
        let runs = simulate(&base, &options, 42, 200).expect("runs");
        let won_1_1 = runs.series[0][0].wins;
        let first = base.run.targets[0][0];
        let from_curve = a
            .best
            .iter()
            .flatten()
            .filter(|&&b| b >= i64::try_from(first).unwrap_or(i64::MAX))
            .count();
        assert_eq!(u64::try_from(from_curve).unwrap_or(0), won_1_1);
        let boss = Options {
            boss: Some(BossId::new("the_pusher")),
            ..Options::default()
        };
        let content = boss.content(&base).expect("content");
        let b = curve(&content, &boss, 42, 50).expect("boss curve");
        assert_eq!(b.best.len(), 50);
        // A character with its own Zielwerte plays every series to the end
        // too, so the curve reaches past its first Zielwert.
        let null = Options {
            character: Character::NullPlayer,
            ..Options::default()
        };
        let n = curve(&base, &null, 42, 100).expect("null curve");
        let limit = u64::from(base.run.game_limit);
        assert_eq!(n.games, 100 * limit);
        assert_eq!(a.games, 200 * limit);
    }

    #[test]
    fn run_metrics_add_up() {
        let base = content();
        let options = Options::default();
        let stats = simulate(&base, &options, 9, 40).expect("runs");
        let lost: u64 = stats.lost_in_section.iter().sum();
        let ended: u64 = stats.districts.iter().flat_map(|d| d.ended).sum();
        assert_eq!(ended, lost, "every lost run ends in one pub");
        // Games per Spielart sum to all games.
        assert_eq!(
            stats.kinds.iter().map(|k| k.games).sum::<u64>(),
            stats.games
        );
        // Spending by kind sums to the whole.
        assert_eq!(stats.spending.iter().sum::<u64>(), stats.coins_spent);
        assert!(stats.spending[0] > 0, "relics are bought");
        // Every run starts district 1.
        let d1 = stats.districts[0];
        assert_eq!(d1.runs, 40);
        assert!(d1.relics_start >= 40, "the Zähler from the start");
        assert_eq!(d1.boss_runs, stats.series[0][4].attempts);
        // Spelunken are counted apart from the pubs they replace: a slot is
        // entered once per run that gets there, and again when the
        // "Schutzengel" has it played once more.
        for (section, (row, sp)) in stats.series.iter().zip(&stats.spelunken).enumerate() {
            for (index, (p, s)) in row.iter().zip(sp).enumerate() {
                if index == 0 {
                    assert_eq!(
                        p.attempts + s.attempts - p.replays - s.replays,
                        stats.districts[section].runs
                    );
                }
                assert!(p.games >= p.attempts, "at least one game per pub");
                assert!(s.wins <= s.attempts && p.wins <= p.attempts);
            }
            assert_eq!(sp[4].attempts, 0, "never a Spelunke before the boss");
        }
        assert!(
            stats.spelunken.iter().flatten().any(|s| s.attempts > 0),
            "40 runs meet a Spelunke"
        );
        let text = render(&stats, &base, &options, 0);
        // Without lives (ADR 0019) every lost pub ends the run, unless the
        // "Schutzengel" has it played again.
        let lost_pubs: u64 = stats.districts.iter().map(|d| d.lost_pubs).sum();
        let again: u64 = stats.districts.iter().map(|d| d.second_chances).sum();
        assert_eq!(lost_pubs, lost + again);
        // The Faktor of the won games is at least ×1 without a factor below.
        assert_eq!(stats.factor.won_games, stats.games_won);
        assert!(stats.factor.run_max_permille >= 40 * 1000);
        // A card reward after every won normal pub and Spelunke (DESIGN
        // 5.0); the bot takes it whenever a Zinke fits.
        let won_pubs: u64 = stats
            .series
            .iter()
            .chain(&stats.spelunken)
            .flat_map(|row| row[..4].iter())
            .map(|p| p.wins)
            .sum();
        let c = stats.card_rewards;
        assert_eq!(c.offered, won_pubs);
        assert_eq!(c.offered, c.taken + c.skipped);
        assert!(c.taken > 0 && c.raised <= c.taken);
        let taken: u64 = stats.card_reward_zinken.values().map(|(_, t)| t).sum();
        assert_eq!(taken, c.taken);
        for needle in [
            "card rewards per run",
            "card reward zinken",
            "relics start",
            "faktor: won games mean",
            "runs end:",
            "games per pub:",
            "groschen spent per run",
            "schneider%",
            "vouchers per run",
            "sp ",
        ] {
            assert!(text.contains(needle), "{needle} in {text}");
        }
    }

    #[test]
    fn the_curve_can_measure_a_later_district() {
        // The defenders of run.ron per district, made cheap: heuristic in
        // districts 1 and 2, a small PIMC in district 3.
        let mut base = Content::load().expect("content");
        base.run.opponents = vec![
            Opponents::Heuristic,
            Opponents::Heuristic,
            Opponents::Pimc {
                samples: 1,
                depth: 1,
            },
        ];
        let options = Options {
            curve: true,
            district: Some(3),
            ..Options::default()
        };
        let content = options.content(&base).expect("content");
        let a = curve(&content, &options, 42, 30).expect("curve");
        assert_eq!(a, curve(&content, &options, 42, 30).expect("curve"));
        assert!(render_curve(&a, &content, &options, 0).contains("district=3"));
        // Same seeds in district 2: other defenders, other games.
        let two = Options {
            district: Some(2),
            ..options.clone()
        };
        let c = curve(&two.content(&base).expect("content"), &two, 42, 30).expect("curve");
        assert_ne!((a.points, a.games_won), (c.points, c.games_won));
        let boss = Options {
            boss: Some(BossId::new("the_old_man")),
            ..options.clone()
        };
        let content = boss.content(&base).expect("content");
        let b = curve(&content, &boss, 42, 10).expect("boss curve");
        assert_eq!(b.games, 10 * u64::from(base.run.game_limit));
        for bad in [Some(0), Some(4)] {
            let wrong = Options {
                district: bad,
                ..options.clone()
            };
            assert!(wrong.content(&base).is_err());
        }
        let runs = Options {
            district: Some(2),
            ..Options::default()
        };
        assert!(runs.content(&base).is_err(), "--district needs --curve");
    }

    #[test]
    fn the_null_player_and_the_unlocks_can_be_measured() {
        let base = content();
        let options = Options {
            character: Character::NullPlayer,
            null_scoring: Some(NullScoring::Fixed),
            ..Options::default()
        };
        let content = options.content(&base).expect("content");
        let stats = simulate(&content, &options, 2, 20).expect("runs");
        assert_eq!(stats.lost_in_section.iter().sum::<u64>() + stats.won, 20);
        assert!(render(&stats, &content, &options, 0).contains("character=NullPlayer"));
        let options = Options {
            unlocks: true,
            ..Options::default()
        };
        let a = unlocks(&base, &options, 3, 15).expect("unlocks");
        assert_eq!(a, unlocks(&base, &options, 3, 15).expect("unlocks"));
        assert_eq!(a.earned_at.len(), base.unlocks.len());
        assert!(
            !a.earned_at[&UnlockId::new("second_evening")].is_empty(),
            "section 2 is reached within 15 runs"
        );
        assert!(render_unlocks(&a, &base).contains("tour_won"));
    }
}
