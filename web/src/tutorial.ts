// Die Einführung (DESIGN 16.1): one guided pub with the fixed deals of the
// core's `tutorial.ron`. The Wirt explains, step by step, what matters at
// the moment it matters: the goal, Skat or Hand, Drücken, the Ansage,
// Trumpf, Bedienen, Augen, Spielwert, the Abrechnung, and after the pub the
// Zinken, the way and the Tresen.
//
// This file is presentation only: the texts (./i18n/de.ts, ./i18n/en.ts) and when each
// one shows. Every trigger reads what the screen already shows (the core's
// view, the open dialog); nothing here decides a rule. The lessons stay
// resilient: a step whose situation never comes is skipped, a step whose
// situation has passed counts as done, and the generic steps cover a
// player who does something else than the Wirt suggests (Hand instead of
// the skat, another Spielart).
//
// Whether the player has seen or declined the Einführung is kept in
// localStorage under a `stammtisch.` key (so the save code carries it).

import { de, t } from './i18n';
import type { GameOption, ScoreBreakdown } from './types';
import { readKey, storageUsable, writeKey } from './storage';
import type { UiState } from './ui/state';

/** What a step asks of the player: read and tap on, act on the table, or end the Einführung. */
export type CoachKind = 'info' | 'do' | 'end';

export interface CoachStep {
  id: string;
  kind: CoachKind;
  /** The situation the step is about is on screen. */
  when: (ui: UiState) => boolean;
  text: (ui: UiState) => string;
  /** CSS selector of what the Wirt points at; null for no element. */
  target?: (ui: UiState) => string | null;
  /** The dialog the step talks about; without, it shows only when no dialog is open. */
  overlay?: 'score';
  /** Where the bubble goes: above or below its target; without, the freer half of the screen. */
  place?: 'above' | 'below';
  /** Shown again in each game of the pub (once otherwise). */
  perGame?: boolean;
}

type Game = NonNullable<NonNullable<UiState['view']>['game']>;

/** The table of a game of the tutorial pub (not the score board, not a stop). */
function table(ui: UiState): Game | null {
  const view = ui.view;
  if (ui.screen !== 'run' || !view?.tutorial || view.phase !== 'Game') return null;
  if (view.series.section !== 0 || view.series.index !== 0) return null;
  return view.game ?? null;
}

/**
 * The game of the pub on screen, from 0. The score board is shown while
 * the view of its game is still on screen (the playback sets the new view
 * after it), so it counts as that game.
 */
function gameNo(ui: UiState): number {
  return ui.view?.series.games_played ?? -1;
}

/** Skat oder Hand: the choice with ten cards is on the felt. */
function choosing(ui: UiState): boolean {
  const game = table(ui);
  return game?.phase === 'Declaring' && Boolean(game.declare?.can_pick_up_skat) && !ui.handGame && !ui.anim;
}

/** The Ansage with its Spielarten is on screen (Hand, or after the skat). */
function declaring(ui: UiState): boolean {
  const game = table(ui);
  if (!game?.declare || ui.anim) return false;
  if (game.phase === 'DeclaringAfterSkat') return true;
  return game.phase === 'Declaring' && (ui.handGame || !game.declare.can_pick_up_skat);
}

function discarding(ui: UiState): boolean {
  return table(ui)?.phase === 'Discarding' && !ui.anim;
}

/** Playing, the player to move, no animation running. */
function playing(ui: UiState): boolean {
  const game = table(ui);
  return game?.phase === 'Playing' && game.your_turn && !ui.anim && !ui.busy;
}

function isGame(ui: UiState, n: number): boolean {
  return gameNo(ui) === n;
}

const isClubs = (g: GameOption): boolean => typeof g.game === 'object' && g.game.Suit === 'Clubs';
const isNullGame = (g: GameOption): boolean => g.game === 'Null';
const isGrand = (g: GameOption): boolean => g.game === 'Grand';

/** Whether the Spielart the player picked in the Ansage is this one. */
function picked(ui: UiState, pick: (g: GameOption) => boolean): boolean {
  const games = table(ui)?.declare?.games ?? [];
  const g = ui.pickGame === null ? undefined : games[ui.pickGame];
  return g !== undefined && pick(g);
}

/** The selector of a Spielart button of the Ansage. */
function gameButton(ui: UiState, pick: (g: GameOption) => boolean): string | null {
  const games = table(ui)?.declare?.games ?? [];
  const i = games.findIndex(pick);
  return i < 0 ? null : `.game-btn[data-index="${i}"]`;
}

function scoreOf(ui: UiState): ScoreBreakdown | null {
  return ui.overlay?.kind === 'score' ? ui.overlay.score.breakdown : null;
}

/** The pub is over (won or lost) and the run is still the tutorial's. */
function afterPub(ui: UiState): boolean {
  const view = ui.view;
  if (!view?.tutorial) return false;
  if (ui.screen === 'end') return true;
  if (ui.screen !== 'run' || ui.anim) return false;
  return view.phase !== 'Game' || view.series.index !== 0 || view.series.section !== 0;
}

function pubLost(ui: UiState): boolean {
  return ui.screen === 'end' && ui.ended?.won === false;
}

const HAND = '[data-testid="hand"]';
/** The Einführung's texts in the active language (read at each use: the language can change). */
const TD = new Proxy({} as typeof de.tutorial, { get: (_, key) => de.tutorial[key as keyof typeof de.tutorial] });

/**
 * The steps in the order they win when several apply at once: per game the
 * lesson, then the generic ones that catch a player going his own way.
 */
export const STEPS: CoachStep[] = [
  // ---- Game 1: the goal, Skat or Hand, Drücken, Ansage, Trumpf, Bedienen, Augen, Spielwert
  { id: 'goal', kind: 'info', when: (ui) => choosing(ui) && isGame(ui, 0), text: (ui) => t(TD.goal, { target: ui.view?.series.target ?? '' }), target: () => '.hud .slate' },
  { id: 'hand', kind: 'info', when: (ui) => choosing(ui) && isGame(ui, 0), text: () => TD.hand, target: () => HAND },
  { id: 'skat', kind: 'do', when: (ui) => choosing(ui) && isGame(ui, 0), text: () => TD.skat, target: () => '[data-testid="skat-pile"]' },
  // Game 2: the Null hand, again with the skat.
  { id: 'nullIntro', kind: 'info', when: (ui) => choosing(ui) && isGame(ui, 1), text: () => TD.nullIntro, target: () => HAND },
  { id: 'nullSkat', kind: 'do', when: (ui) => choosing(ui) && isGame(ui, 1), text: () => TD.nullSkat, target: () => '[data-testid="skat-pile"]' },
  // Game 3: Grand Hand.
  {
    id: 'grandIntro',
    kind: 'info',
    when: (ui) => choosing(ui) && isGame(ui, 2),
    text: (ui) => {
      const needed = (ui.view?.series.target ?? 0) - (ui.view?.series.score ?? 0);
      return needed > 0 ? t(TD.grandIntro, { needed }) : TD.grandIntroReached;
    },
    target: () => HAND,
  },
  { id: 'grandHand', kind: 'do', when: (ui) => choosing(ui) && isGame(ui, 2), text: () => TD.grandHand, target: () => '[data-action="play-hand"]' },

  // ---- Drücken
  { id: 'pressWhy', kind: 'info', when: (ui) => discarding(ui) && isGame(ui, 0), text: () => TD.pressWhy, target: () => '[data-testid="skat-zone"]', place: 'above' },
  { id: 'press', kind: 'do', when: (ui) => discarding(ui) && isGame(ui, 0), text: () => TD.press, target: () => '[data-testid="skat-zone"]', place: 'above' },
  { id: 'nullPress', kind: 'do', when: (ui) => discarding(ui) && isGame(ui, 1), text: () => TD.nullPress, target: () => '[data-testid="skat-zone"]', place: 'above' },
  { id: 'pressGeneric', kind: 'do', perGame: true, when: (ui) => discarding(ui), text: () => TD.pressGeneric, target: () => '[data-testid="skat-zone"]', place: 'above' },

  // ---- Ansage
  { id: 'handChosen', kind: 'info', when: (ui) => declaring(ui) && table(ui)?.phase === 'Declaring' && !isGame(ui, 2), text: () => TD.handChosen, target: () => '.games' },
  { id: 'ansage', kind: 'info', when: (ui) => declaring(ui), text: () => TD.ansage, target: () => '.games' },
  {
    id: 'pickClubs',
    kind: 'do',
    when: (ui) => declaring(ui) && isGame(ui, 0) && !picked(ui, isClubs),
    text: () => TD.pickClubs,
    target: (ui) => gameButton(ui, isClubs),
  },
  { id: 'nullPick', kind: 'do', when: (ui) => declaring(ui) && isGame(ui, 1) && !picked(ui, isNullGame), text: () => TD.nullPick, target: (ui) => gameButton(ui, isNullGame) },
  { id: 'grandPick', kind: 'do', when: (ui) => declaring(ui) && isGame(ui, 2) && !picked(ui, isGrand), text: () => TD.grandPick, target: (ui) => gameButton(ui, isGrand) },
  { id: 'declareGo', kind: 'do', perGame: true, when: (ui) => declaring(ui) && ui.pickGame !== null, text: () => TD.declareGo, target: () => '[data-action="declare"], [data-action="declare-null"]' },

  // ---- Playing
  {
    id: 'trump',
    kind: 'info',
    when: (ui) => {
      const game = table(ui)?.declaration?.game;
      return playing(ui) && typeof game === 'object';
    },
    text: () => TD.trump,
    target: () => HAND,
  },
  { id: 'lead', kind: 'info', when: (ui) => playing(ui) && isGame(ui, 0) && (table(ui)?.trick.length ?? 1) === 0, text: () => TD.lead, target: () => HAND },
  { id: 'follow', kind: 'info', when: (ui) => playing(ui) && (table(ui)?.trick.length ?? 0) > 0, text: () => TD.follow, target: () => '[data-testid="trick"]' },
  { id: 'augen', kind: 'info', when: (ui) => playing(ui) && (table(ui)?.tricks_played ?? 0) >= 1 && !isNull(ui), text: () => TD.augen, target: () => '[data-testid="win-from"], [data-tricks]' },
  {
    id: 'value',
    kind: 'info',
    when: (ui) => playing(ui) && (table(ui)?.tricks_played ?? 0) >= 2 && Boolean(table(ui)?.value_preview) && !isNull(ui),
    text: (ui) => {
      const p = table(ui)?.value_preview;
      const levels = p?.levels ?? 0;
      return t(TD.value, { base: p?.base_value ?? '', levels, matadors: Math.max(1, levels - 1) });
    },
    target: () => '[data-testid="value-preview"]',
  },
  { id: 'nullPlay', kind: 'info', when: (ui) => playing(ui) && isNull(ui), text: () => TD.nullPlay, target: () => HAND },
  { id: 'grandPlay', kind: 'info', when: (ui) => playing(ui) && isGame(ui, 2) && table(ui)?.declaration?.game === 'Grand', text: () => TD.grandPlay, target: () => HAND },

  // ---- Abrechnung
  {
    id: 'score',
    kind: 'info',
    overlay: 'score',
    when: (ui) => Boolean(ui.view?.tutorial) && Boolean(scoreOf(ui)) && scoreOf(ui)?.declaration.game !== 'Ramsch' && !isNullScore(ui),
    text: (ui) => {
      const b = scoreOf(ui);
      if (!b) return '';
      return t(b.won ? TD.score : TD.scoreLost, { base: b.base_value, levels: b.levels, value: b.value });
    },
    target: () => '.score-sheet .formula',
  },
  { id: 'nullScore', kind: 'info', overlay: 'score', when: (ui) => Boolean(ui.view?.tutorial) && isNullScore(ui), text: () => TD.nullScore, target: () => '.score-sheet .formula' },

  // ---- After the pub: Zinken, the way, the Tresen, a real evening
  { id: 'pubLost', kind: 'info', when: (ui) => afterPub(ui) && pubLost(ui), text: () => TD.pubLost, target: () => null },
  { id: 'pubWon', kind: 'info', when: (ui) => afterPub(ui) && ui.view?.phase === 'CardReward', text: () => TD.pubWon, target: () => '[data-testid="card-reward-offers"]' },
  { id: 'route', kind: 'info', when: (ui) => afterPub(ui) && (ui.view?.phase === 'Route' || pubLost(ui)), text: () => TD.route, target: () => '.route-options' },
  {
    id: 'tresen',
    kind: 'info',
    when: (ui) => afterPub(ui) && ui.view?.phase !== 'CardReward',
    text: (ui) => (ui.screen === 'end' || !ui.view?.relics.length ? TD.tresenEnd : TD.tresen),
    target: (ui) => (ui.screen === 'end' ? null : '[data-testid="leiste-relics"]'),
  },
  { id: 'evening', kind: 'end', when: (ui) => afterPub(ui) && ui.view?.phase !== 'CardReward', text: () => TD.evening, target: () => null },
];

function isNull(ui: UiState): boolean {
  return table(ui)?.declaration?.game === 'Null';
}

function isNullScore(ui: UiState): boolean {
  return scoreOf(ui)?.declaration.game === 'Null';
}

function stepKey(step: CoachStep, ui: UiState): string {
  return step.perGame ? `${step.id}#${gameNo(ui)}` : step.id;
}

/** Whether nothing else is in the way of a step (another dialog). */
function free(step: CoachStep, ui: UiState): boolean {
  if (step.overlay) return ui.overlay?.kind === step.overlay;
  return !ui.overlay;
}

/**
 * The coach of one Einführung: which steps are done, which one is on
 * screen. A step shown and then passed by (its situation is gone) counts
 * as done; a tap on it does too.
 */
export class Coach {
  private readonly done = new Set<string>();
  private current: CoachStep | null = null;
  /** The key the current step is done under (per game for `perGame` steps). */
  private currentKey = '';

  /** The step to show now, or null; call after every render. */
  update(ui: UiState): CoachStep | null {
    const cur = this.current;
    if (cur && !(cur.when(ui) && free(cur, ui))) {
      // Its situation has passed (or a dialog came in front): done once passed.
      if (!cur.when(ui)) this.done.add(this.currentKey);
      this.current = null;
    }
    if (this.current) return this.current;
    for (const step of STEPS) {
      const key = stepKey(step, ui);
      if (this.done.has(key)) continue;
      if (step.when(ui) && free(step, ui)) {
        this.current = step;
        this.currentKey = key;
        return step;
      }
    }
    return null;
  }

  /** The player tapped the bubble: on to the next step. */
  next(): void {
    if (this.current && this.current.kind !== 'end') this.done.add(this.currentKey);
    this.current = null;
  }

  get step(): CoachStep | null {
    return this.current;
  }

  /** "Spiel 2 von 3", or "nach der Kneipe". */
  label(ui: UiState): string {
    const n = gameNo(ui);
    return afterPub(ui) || n < 0 || n > 2 ? TD.after : t(TD.progress, { game: n + 1 });
  }
}

// ---------------------------------------------------------------------------
// Storage: whether the Einführung was played or declined
// ---------------------------------------------------------------------------

const KEY = 'stammtisch.tutorial';

/** Whether the player played, skipped or declined the Einführung. */
// Storage blocked: counts as seen, the offer would come back on every visit.
export function tutorialSeen(): boolean {
  return !storageUsable() || readKey(KEY) !== null;
}

/** Remember the Einführung as played ('done') or declined ('declined'). */
export function markTutorial(state: 'done' | 'declined'): void {
  writeKey(KEY, state);
}
