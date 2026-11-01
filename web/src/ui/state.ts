// UI state of the app. Holds the latest view from the core plus purely
// visual state (selection, overlays, the animation snapshot).

import type { Settings } from '../settings';
import type { Record } from '../storage';
import type { BossInfo, CardView, OwnedRelic, Payout, PlayedCard, RiskLevel, RunView, ScoreView, Seat } from '../types';

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
  resolve: () => void;
}

export type Overlay =
  | ScoreOverlay
  | { kind: 'boss'; boss: BossInfo; resolve: () => void }
  | { kind: 'info'; title: string; subtitle: string; text: string; use?: number; coaster?: string; good?: string; odds?: boolean; buy?: { action: 'buy-relic' | 'buy-item'; slot: number; price: number } }
  | { kind: 'target'; slot: number }
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
  overlay: Overlay | null;
  message: string | null;
  saveInfo: string | null;
  record: Record;
  settings: Settings;
  /** The run ended: won or lost. */
  ended: { won: boolean; view: RunView } | null;
}
