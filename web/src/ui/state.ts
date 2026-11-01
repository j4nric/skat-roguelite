// UI state of the app. Holds the latest view from the core plus purely
// visual state (selection, overlays, the animation snapshot).

import type { SharedSeed } from '../seed';
import type { Quip } from './guests';
import type { Settings } from '../settings';
import type { RunRecord } from '../storage';
import type { BossInfo, CardView, Character, MetaView, NullRisk, OwnedRelic, Payout, PlayedCard, RiskLevel, RunLength, RunView, ScoreView, Seat } from '../types';

/** What the table shows while events are being animated. */
export interface AnimState {
  trick: PlayedCard[];
  hand: CardView[];
  handSizes: number[];
  status: string;
  /** Seat whose card is expected next, for highlighting. */
  turn: Seat | null;
}

export interface ScoreOverlay {
  kind: 'score';
  score: ScoreView;
  /** Series account before and after this game, both from the core. */
  previousScore: number;
  seriesScore: number;
  target: number;
  payout: { payout: Payout; coins: number } | null;
  relics: OwnedRelic[];
  /** Display names of the seats (a boss sits in seat 1). */
  seatNames: string[];
  resolve: () => void;
}

/** A new run waiting for the player to give up the running one. */
export interface PendingStart {
  seed: string;
  character: Character;
  /** Ganzer or kurzer Abend. */
  length: RunLength;
  /** Stake of the Einsatz-Leiter. */
  stake: number;
  daily: boolean;
}

export type Overlay =
  | ScoreOverlay
  | { kind: 'boss'; boss: BossInfo; resolve: () => void }
  | { kind: 'target'; slot: number }
  /** "Kreide": pick a card of the hand (`card`), then its new suit. */
  | { kind: 'chalk'; slot: number; card: number | null }
  /** Pick the card for an encounter option that needs one. */
  | { kind: 'encounter-target'; option: number }
  /** The Vereinsbuch: characters and unlocks, or the statistics page. */
  | { kind: 'book'; tab: 'book' | 'stats' }
  /** Give up the running run for a new one? */
  | { kind: 'confirm-new'; start: PendingStart }
  /** Unlocks just earned, ids from the meta state. */
  | { kind: 'unlock'; ids: string[] }
  | { kind: 'menu' }
  /** "Spielstand": why to keep a copy, then export or import. */
  | { kind: 'save-menu' }
  /** The save code of everything stored, to copy or share; `copied` after a copy attempt. */
  | { kind: 'save-export'; code: string; copied: boolean | null }
  /** Paste a save code; `error` names why the last one was rejected. */
  | { kind: 'save-import'; text: string; error: string | null; checking: boolean }
  /** A checked save code, waiting for the player to replace the current progress. */
  | { kind: 'save-confirm'; keys: Record<string, string>; lines: string[] }
  | { kind: 'overview' }
  /** The Regelkarte: what is different here from the Skatordnung. */
  | { kind: 'rules' }
  | { kind: 'credits' };

export interface UiState {
  screen: 'loading' | 'start' | 'run' | 'end' | 'error';
  view: RunView | null;
  anim: AnimState | null;
  busy: boolean;
  /** Card ids picked for discarding. */
  selected: number[];
  /** The player chose "Hand spielen" before the skat: show the hand Ansage. */
  handGame: boolean;
  /** One tap on the deck while choosing: "Einmischen" waits for a confirmation. */
  mixArmed: boolean;
  /** Index into the declare options' games. */
  pickGame: number | null;
  pickRisk: RiskLevel | null;
  /** Picked Null-Risikostufe (Nullspieler). */
  pickNullRisk: NullRisk | null;
  /** Skat swap ("Die Oma", "Der Opa"): the player picks the card to put into the skat. */
  omaPick: boolean;
  /** Skat card picked to take in a skat swap; null takes the core's first. */
  swapTake: number | null;
  /** The player looks at the last finished trick (the core's `last_trick`) instead of the open one. */
  peekTrick: boolean;
  /** The opponent whose move the core is computing (thought bubble). */
  thinking: Seat | null;
  /** A Stammgast's speech bubble on the table, at most one at a time. */
  quip: Quip | null;
  /** A seed from a shared link, offered on the start screen. */
  shared: (SharedSeed & { locked: boolean }) | null;
  /** Result of the last "copy link" tap: the link and whether it worked. */
  copy: { url: string; ok: boolean } | null;
  /** Finished runs of this browser, newest first. */
  history: RunRecord[];
  /** Character picked on the start screen. */
  pickCharacter: Character;
  /** Run length picked on the start screen (ganzer or kurzer Abend). */
  pickLength: RunLength;
  /** Stake picked on the start screen; null plays the highest unlocked. */
  pickStake: number | null;
  overlay: Overlay | null;
  message: string | null;
  saveInfo: string | null;
  /** Unlocks and statistics from the core's meta state. */
  meta: MetaView | null;
  settings: Settings;
  /** The run ended: won or lost. */
  ended: { won: boolean; view: RunView } | null;
}
