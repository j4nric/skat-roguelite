//! Opening a pub and dealing its games: Stammgäste and boss editions,
//! the deal itself (fixed deals of the Einführung included), the skat
//! peek of an encounter, the defender cards "Glasauge" reveals, and the
//! rules of each game ([`GameRules`]).

use crate::cards::{Card, PlayingCard};
use crate::content::{
    ActiveSource, BossId, Content, Effect, Source, Stage, card_mod_sources, in_pool,
    resolve_edition,
};
use crate::deck::{CardKind, DealtCard, Edition, Owner};
use crate::game::{Event, Game, Phase, Seat};
use crate::rng::Rng;
use crate::rules::{Declaration, Thresholds};

use super::{
    ActiveGame, GameRules, Kontra, PUBS_PER_DISTRICT, Run, RunError, RunEvent, RunPhase, Series,
    opponents, tour,
};

/// Salt of the draw of the defender cards "Glasauge" makes "Durchsichtig"
/// for a game ([`Effect::RevealDefenderCards`]).
pub(super) const TRANSPARENT_SEED: u64 = 0x61A5_5E7E_0000_0005;

/// The editions of the two boss players of a game, left and right; both
/// `None` outside a boss pub.
pub(super) type BossEditions = [Option<Edition>; 2];

/// A game dealt by hand: every card in the edition of the seat that gets
/// it (the skat in the player's), found by its printed face ("Winkel"
/// changes the kind of a dealt card). Fails if a card is not in a deck.
pub(super) fn fixed_game(
    resolved: &[crate::content::ResolvedDeck; 3],
    hands: [&[CardKind]; 3],
    skat: [CardKind; 2],
    forehand: Seat,
) -> Result<Game<DealtCard>, RunError> {
    let editions = resolved.clone().map(|r| r.cards);
    let printed = Edition::standard();
    let pick = |seat: usize, kind: CardKind| -> Result<DealtCard, RunError> {
        let missing = || RunError::UnknownContent(format!("card {kind} in the deal"));
        let id = printed
            .cards()
            .iter()
            .find(|c| c.kind == kind)
            .ok_or_else(missing)?
            .id;
        editions[seat]
            .iter()
            .find(|c| c.id == id)
            .copied()
            .ok_or_else(missing)
    };
    let mut dealt: [Vec<DealtCard>; 3] = Default::default();
    for (seat, cards) in hands.iter().enumerate() {
        for &kind in *cards {
            dealt[seat].push(pick(seat, kind)?);
        }
    }
    let skat = [pick(0, skat[0])?, pick(0, skat[1])?];
    Ok(Game::with_editions(dealt, skat, editions, forehand)?)
}

impl Run {
    /// Start pub `index` of district `section`, a Spelunke if `spelunke`
    /// is set (ADR 0016), and deal its first game.
    pub(super) fn start_pub(
        &mut self,
        content: &Content,
        section: u32,
        index: u32,
        spelunke: Option<tour::Spelunke>,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        self.open_pub(content, section, index, spelunke, false, events)
    }

    /// Play a lost pub again from the start ("Schutzengel", ADR 0019): the
    /// same pub (or Spelunke with its malus) and boss, a fresh account and
    /// a fresh deal; the house rules of the evening stay, the boss is not
    /// drawn again. Its Zielwert is the pub's own, without changes planned
    /// for it on the way ("Wegbier": they were used up by the first try).
    pub(super) fn replay_pub(
        &mut self,
        content: &Content,
        section: u32,
        index: u32,
        spelunke: Option<tour::Spelunke>,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        self.open_pub(content, section, index, spelunke, true, events)
    }

    /// [`Run::start_pub`], or with `replay` [`Run::replay_pub`].
    fn open_pub(
        &mut self,
        content: &Content,
        section: u32,
        index: u32,
        spelunke: Option<tour::Spelunke>,
        replay: bool,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let config = &content.run;
        let sections = config.sections();
        if !self.district_exists(content, section) || index >= PUBS_PER_DISTRICT {
            return Err(RunError::WrongPhase);
        }
        if index == 0 && !replay && !self.evening_relics.is_empty() {
            for relic in std::mem::take(&mut self.evening_relics) {
                self.pay_out_stash(content, &relic, events);
            }
            events.push(RunEvent::EveningRulesEnded);
        }
        if (index == 0 && !replay) || self.section_boss.is_none() {
            let last = section + 1 == sections;
            self.section_boss = match (&config.final_boss, last) {
                _ if section >= sections => self.draw_night_boss(content),
                (Some(boss), true) => Some(boss.clone()),
                _ => {
                    let pool: Vec<&BossId> = config
                        .bosses
                        .iter()
                        .filter(|id| {
                            content
                                .boss(id)
                                .is_some_and(|b| in_pool(&b.characters, self.character))
                        })
                        .collect();
                    let pick = self
                        .misc_rng
                        .gen_range(0..u64::try_from(pool.len()).unwrap_or(1));
                    pool.get(usize::try_from(pick).unwrap_or(0))
                        .map(|id| (*id).clone())
                }
            };
            if let Some(boss) = &self.section_boss {
                events.push(RunEvent::BossAnnounced {
                    section,
                    boss: boss.clone(),
                });
            }
            let extra = self
                .night
                .as_ref()
                .map(|n| n.extra_bosses.clone())
                .unwrap_or_default();
            if section >= sections && !extra.is_empty() {
                events.push(RunEvent::ExtraBossRules {
                    section,
                    bosses: extra,
                });
            }
        }
        let boss = if index + 1 == PUBS_PER_DISTRICT {
            self.section_boss.clone()
        } else {
            None
        };
        let mut target = self
            .pub_target(content, section, index, boss.as_ref())
            .ok_or(RunError::WrongPhase)?;
        if spelunke.is_some() {
            target = config.spelunke.target_factor.apply(target);
        }
        let plan = std::mem::take(&mut self.next_pub);
        target = plan.apply_to(target);
        if self.tutorial && section == 0 && index == 0 {
            target = content.tutorial.target;
        }
        let redeals_withheld = plan.redeals.min(0).unsigned_abs();
        let malus = spelunke.as_ref().map(|s| s.malus.clone());
        self.series = Series {
            section,
            index,
            target,
            game_limit: config.game_limit,
            boss: boss.clone(),
            spelunke,
            redeals_withheld,
            ..Series::default()
        };
        self.series.night_malus = self.draw_night_malus(content, boss.is_some());
        let malus = malus.or_else(|| self.series.night_malus.clone());
        self.phase = RunPhase::Game;
        self.shop = None;
        self.encounter = None;
        self.route = None;
        self.trip = None;
        self.series.extra_redeals = u32::try_from(plan.redeals.max(0)).unwrap_or(0);
        self.seat_guests(content, section, index);
        if !plan.is_empty() {
            events.push(RunEvent::NextPubApplied {
                redeals: plan.redeals,
                target: plan.target,
            });
        }
        events.push(RunEvent::SeriesStarted {
            section,
            index,
            target,
            game_limit: config.game_limit,
            redeals: content.redeals_in(self.character, section),
            boss,
            malus,
        });
        self.curse_for_pub(content, events);
        self.lay_night_curses(content, events);
        self.deal(content, true, events)
    }

    /// Set up and deal a game. `rotate` advances the forehand (not for a
    /// redeal of the same game).
    pub(super) fn deal(
        &mut self,
        content: &Content,
        rotate: bool,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        self.deal_keeping(content, rotate, &[], events)
    }

    /// [`Run::deal`], the player keeping the cards with the ids in `keep`
    /// in his hand ("Unterschlupf" at an Einmischen).
    pub(super) fn deal_keeping(
        &mut self,
        content: &Content,
        rotate: bool,
        keep: &[u32],
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        let rotation = if rotate {
            self.games_dealt
        } else {
            self.games_dealt.saturating_sub(1)
        };
        self.lay_boss_curses(content, events);
        let global = self.sources(content, false);
        // "Ehrenvorsitz": the player always leads; the rotation runs on.
        let always_forehand = global
            .iter()
            .any(|s| s.passive.contains(&Effect::AlwaysForehand));
        let forehand = if always_forehand {
            Seat::DECLARER
        } else {
            Seat::from_index(usize::try_from(rotation % 3).unwrap_or(0)).unwrap_or(Seat::DECLARER)
        };
        if rotate {
            self.games_dealt += 1;
            self.series.redeals_in_game = 0;
            self.series.redeals_this_game = 0;
        }
        let (resolved, boss_editions) = self.resolve_editions(content, &global);
        let mut rng = self.deal_rng.fork();
        // The Einführung deals its fixed games (DESIGN 16.1); the generator
        // is forked all the same, so everything drawn after stays put.
        let fixed = self
            .in_tutorial_pub()
            .then(|| {
                content
                    .tutorial
                    .deals
                    .get(self.series.games_played as usize)
            })
            .flatten();
        let game = if let Some(deal) = fixed {
            let kinds = |cards: &[Card]| -> Vec<CardKind> {
                cards.iter().map(|&c| CardKind::Normal(c)).collect()
            };
            let skat = [deal.skat[0], deal.skat[1]].map(CardKind::Normal);
            fixed_game(
                &resolved,
                [
                    &kinds(&deal.player),
                    &kinds(&deal.left),
                    &kinds(&deal.right),
                ],
                skat,
                Seat::from_index(deal.forehand).unwrap_or(Seat::DECLARER),
            )?
        } else {
            Game::deal_editions_keeping(
                &mut rng,
                resolved.clone().map(|r| r.cards),
                forehand,
                keep,
            )?
        };
        self.install_game(content, game, &resolved, boss_editions, events);
        self.peek(&mut rng, events);
        // "Untot": the substitutes come from the game's generator, after
        // everything the deal drew from it.
        let undead_in_play = self.game.as_ref().is_some_and(|active| {
            active
                .game
                .cards_in_play()
                .iter()
                .any(|c| !c.is_blank() && self.has_card_effect(content, *c, &Effect::Undead))
        });
        if let (true, Some(active)) = (undead_in_play, self.game.as_mut()) {
            active.undead_rng = Some(rng.fork());
        }
        if self.game.as_ref().is_some_and(|a| a.rules.ramsch) {
            return self.start_ramsch(content, events);
        }
        Ok(())
    }

    /// "Der Kiebitz": show the player random cards of the defenders, drawn
    /// with the game's own generator (so the deal stays the same).
    pub(super) fn peek(&mut self, rng: &mut Rng, events: &mut Vec<RunEvent>) {
        let Some(active) = self.game.as_mut() else {
            return;
        };
        // Cards the player already sees (`RevealDefenderHand`) are not drawn.
        let shown = active.peeked.clone();
        let mut pool: Vec<(DealtCard, Seat)> = Seat::DEFENDERS
            .iter()
            .flat_map(|&seat| active.game.hand(seat).iter().map(move |&c| (c, seat)))
            .filter(|(c, _)| !shown.iter().any(|(s, _)| s.id == c.id))
            .collect();
        for _ in 0..active.rules.peek {
            if pool.is_empty() {
                break;
            }
            let pick = rng.gen_range(0..u64::try_from(pool.len()).unwrap_or(1));
            let (card, seat) = pool.remove(usize::try_from(pick).unwrap_or(0));
            active.peeked.push((card, seat));
            events.push(RunEvent::CardPeeked { card, seat });
        }
    }

    /// "Der Ramscher": the dealt game is the series' Ramsch. Nobody
    /// declares; it is played at once.
    fn start_ramsch(
        &mut self,
        content: &Content,
        events: &mut Vec<RunEvent>,
    ) -> Result<(), RunError> {
        self.series.ramsch_played = true;
        let active = self.active_mut()?;
        let game_events = active.game.declare(Declaration::Ramsch)?;
        active.note_settled(&game_events);
        events.push(RunEvent::RamschDealt);
        events.extend(game_events.into_iter().map(RunEvent::Game));
        self.after_declaration(content, events);
        self.settle(content, events)
    }

    /// ADR 0015: every seat owns an edition of the 32 cards. Resolve the
    /// edition of each seat for the next game (index = seat; the skat goes
    /// with the player's), plus the boss edition at the table.
    fn resolve_editions(
        &self,
        content: &Content,
        global: &[ActiveSource<'_>],
    ) -> ([crate::content::ResolvedDeck; 3], BossEditions) {
        let boss_editions = self.boss_editions(content);
        let plain = Edition::standard();
        let resolved = Seat::ALL.map(|seat| {
            let owner = self.owner_of_seat(seat);
            let edition = match owner {
                Owner::Player => &self.edition,
                Owner::Boss => boss_editions[0].as_ref().unwrap_or(&plain),
                Owner::BossPartner => boss_editions[1].as_ref().unwrap_or(&plain),
                Owner::Guest => &plain,
            };
            resolve_edition(edition, owner, content, global)
        });
        (resolved, boss_editions)
    }

    /// Install a game dealt from [`Run::resolve_editions`]: the Augen
    /// changes of the copies actually dealt go to the breakdown, the boss
    /// edition and the player's "Durchsichtig" cards to the game.
    fn install_game(
        &mut self,
        content: &Content,
        game: Game<DealtCard>,
        resolved: &[crate::content::ResolvedDeck; 3],
        boss_editions: BossEditions,
        events: &mut Vec<RunEvent>,
    ) {
        let mut point_changes: Vec<(u32, Source, i32)> = Vec::new();
        for card in game.cards_in_play() {
            let seat = game.holder_of(*card).unwrap_or(Seat::DECLARER);
            point_changes.extend(
                resolved[seat.index()]
                    .point_changes
                    .iter()
                    .filter(|(id, _, _)| *id == card.id)
                    .cloned(),
            );
        }
        point_changes.sort_by_key(|(id, _, _)| *id);
        self.set_up_game(content, game, point_changes, events);
        if let Some(active) = self.game.as_mut() {
            let [boss, partner] = boss_editions;
            active.boss_edition = boss;
            active.partner_edition = partner;
        }
        self.mark_defender_cards(events);
    }

    /// "Glasauge" ([`Effect::RevealDefenderCards`]): make random cards of
    /// each defender's hand "Durchsichtig" for this game, drawn with an
    /// own generator from the game's seed and the Einmischen of the game
    /// ([`TRANSPARENT_SEED`]), so the deal and every other draw stay the
    /// same. Candidates in the order of the hand sorted by id; the left
    /// defender first.
    fn mark_defender_cards(&mut self, events: &mut Vec<RunEvent>) {
        let seed = opponents::game_seed(self)
            ^ TRANSPARENT_SEED
            ^ u64::from(self.series.redeals_in_game).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        let Some(active) = self.game.as_mut() else {
            return;
        };
        let n = active.rules.transparent_per_defender;
        let Some(source) = active.rules.transparent_source.clone() else {
            return;
        };
        if n == 0 {
            return;
        }
        let mut rng = Rng::from_seed(seed);
        for seat in Seat::DEFENDERS {
            let mut pool: Vec<DealtCard> = active
                .game
                .hand(seat)
                .iter()
                .filter(|c| !c.revealed)
                .copied()
                .collect();
            pool.sort_by_key(|c| c.id);
            for _ in 0..n {
                if pool.is_empty() {
                    break;
                }
                let len = u64::try_from(pool.len()).unwrap_or(1);
                let pick = usize::try_from(rng.gen_range(0..len)).unwrap_or(0);
                let card = pool.remove(pick.min(pool.len() - 1));
                active.transparent_for_game.push(card.id);
                events.push(RunEvent::CardMadeTransparent {
                    source: source.clone(),
                    card,
                    seat,
                });
            }
        }
    }

    /// Tests: replace the current game by a constructed deal, every card
    /// in the edition of the seat that gets it (the skat in the player's),
    /// with all content effects of a normal deal.
    #[cfg(test)]
    pub(crate) fn deal_fixed(
        &mut self,
        content: &Content,
        hands: [&[CardKind]; 3],
        skat: [CardKind; 2],
        forehand: Seat,
        events: &mut Vec<RunEvent>,
    ) {
        let global = self.sources(content, false);
        let (resolved, boss_editions) = self.resolve_editions(content, &global);
        let game = fixed_game(&resolved, hands, skat, forehand).expect("valid deal");
        self.install_game(content, game, &resolved, boss_editions, events);
    }

    /// Fixture tool: a run saved before ADR 0015 continues with editions
    /// (its game in progress resolved card by card by the holder).
    #[cfg(test)]
    pub(crate) fn adopt_editions(&mut self, content: &Content) {
        let global = self.sources(content, false);
        let (resolved, boss_editions) = self.resolve_editions(content, &global);
        let editions = resolved.map(|r| r.cards);
        let mut next = self.game.as_mut();
        while let Some(active) = next {
            active.game.adopt_editions(editions.clone());
            active.boss_edition.clone_from(&boss_editions[0]);
            active.partner_edition.clone_from(&boss_editions[1]);
            next = active.previous.as_deref_mut();
        }
    }

    /// The editions of the two boss players (ADR 0017): the figures of
    /// the boss whose rule is on (for "Der Alte" the boss of this game's
    /// rule), left and right, in a boss pub only.
    fn boss_editions(&self, content: &Content) -> BossEditions {
        let figures = self
            .series
            .boss
            .as_ref()
            .and_then(|_| self.boss_rule(content))
            .and_then(|b| content.boss(&b))
            .map(|def| def.figures.as_slice())
            .unwrap_or_default();
        // Validated when the content loads.
        [0, 1].map(|i| {
            figures
                .get(i)
                .and_then(|f| Edition::with_mods(&f.edition, content).ok())
        })
    }

    /// Install a freshly dealt game, apply passive effects and fire
    /// `OnDeal`.
    fn set_up_game(
        &mut self,
        content: &Content,
        game: Game<DealtCard>,
        point_changes: Vec<(u32, Source, i32)>,
        events: &mut Vec<RunEvent>,
    ) {
        let forehand = game.forehand();
        let dealt_events: Vec<Event<DealtCard>> = game.log().to_vec();
        // Boss, relics and the rest: the same before and after the game
        // is installed; the card modifications in play follow below.
        let mut sources = self.sources(content, false);
        let has = |wanted: &Effect| sources.iter().any(|s| s.passive.contains(wanted));
        let rules = GameRules {
            thresholds: Thresholds::with_stake(self.stake),
            no_cheating: has(&Effect::NoCheating),
            ..GameRules::default()
        };
        let boss_rule = self.boss_rule(content);
        self.game = Some(ActiveGame {
            game,
            rules,
            lines: Vec::new(),
            point_changes,
            kontra: Kontra::None,
            awaiting_re: false,
            peeked: Vec::new(),
            transparent_for_game: Vec::new(),
            boss_rule,
            skat_peeked: false,
            chalked: Vec::new(),
            single_loss: None,
            win_factors: Vec::new(),
            shown_hand: None,
            shown_hands: Vec::new(),
            previous: None,
            named: Vec::new(),
            skat_shown: Vec::new(),
            risen: Vec::new(),
            pending_rise: None,
            undead_rng: None,
            boss_edition: None,
            partner_edition: None,
            player_marked: Vec::new(),
        });
        events.push(RunEvent::GameDealt {
            number: self.series.games_played + 1,
            forehand,
        });
        events.extend(dealt_events.into_iter().map(RunEvent::Game));
        // Passive effects of boss, relics and card mods in play.
        if let Some(active) = &self.game {
            sources.extend(card_mod_sources(self.editions(), content, &active.game));
        }
        for source in &sources {
            for effect in source.passive {
                self.apply_effect(content, &source.source, effect, events);
            }
        }
        self.fire(content, Stage::OnDeal, None, None, events);
        // An encounter's "Kartentrick": the skat shown as with the Kurzer.
        if self.peek_next_skat.is_some() {
            if let Some(active) = self.game.as_mut() {
                if !active.rules.ramsch && active.game.phase() == Phase::Declaring {
                    active.skat_peeked = true;
                    events.push(RunEvent::SkatPeeked {
                        skat: active.game.skat(),
                    });
                }
            }
        }
    }
}
