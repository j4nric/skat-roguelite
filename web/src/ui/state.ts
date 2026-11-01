// UI state of the app. Holds the latest view from the core plus purely
// visual state (selection, overlays, the animation snapshot).

import type { SharedSeed } from '../seed';
import type { Settings } from '../settings';
import type { RunRecord } from '../storage';
import type { BossInfo, CardView, Character, MetaView, NullRisk, OwnedRelic, Payout, PlayedCard, RiskLevel, RunView, ScoreView, Seat } from '../types';

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
  daily: boolean;
}

export type Overlay =
  | ScoreOverlay
  | { kind: 'boss'; boss: BossInfo; resolve: () => void }
  | { kind: 'info'; title: string; subtitle: string; text: string; use?: number; coaster?: string; good?: string; odds?: boolean; buy?: { action: 'buy-relic' | 'buy-item'; slot: number; price: number } }
  | { kind: 'target'; slot: number }
  /** Pick the card for an encounter option that needs one. */
  | { kind: 'encounter-target'; option: number }
  /** The Vereinsbuch: characters and unlocks, or the statistics page. */
  | { kind: 'book'; tab: 'book' | 'stats' }
  /** Give up the running run for a new one? */
  | { kind: 'confirm-new'; start: PendingStart }
  /** Unlocks just earned, ids from the meta state. */
  | { kind: 'unlock'; ids: string[] }
  | { kind: 'menu' }
  | { kind: 'overview' }
  | { kind: 'credits' };

export interface UiState {
  screen: 'loading' | 'start' | 'run' | 'end' | 'error';
  view: RunView | null;
  anim: AnimState | null;
  busy: boolean;
  /** Card ids picked for discarding. */
  selected: number[];
  /** Index into the declare options' games. */
  pickGame: number | null;
  pickRisk: RiskLevel | null;
  /** Picked Null-Risikostufe (Nullspieler). */
  pickNullRisk: NullRisk | null;
  /** Skat swap ("Die Oma", "Der Opa"): the player picks the card to put into the skat. */
  omaPick: boolean;
  /** Skat card picked to take in a skat swap; null takes the core's first. */
  swapTake: number | null;
  /** The core is computing the opponents' moves (thought bubble). */
  thinking: boolean;
  /** A seed from a shared link, offered on the start screen. */
  shared: (SharedSeed & { locked: boolean }) | null;
  /** Result of the last "copy link" tap: the link and whether it worked. */
  copy: { url: string; ok: boolean } | null;
  /** Finished runs of this browser, newest first. */
  history: RunRecord[];
  /** Character picked on the start screen. */
  pickCharacter: Character;
  overlay: Overlay | null;
  message: string | null;
  saveInfo: string | null;
  /** Unlocks and statistics from the core's meta state. */
  meta: MetaView | null;
  settings: Settings;
  /** The run ended: won or lost. */
  ended: { won: boolean; view: RunView } | null;
}
