// UI state of the app. Holds the latest view from the core plus purely
// visual state (selection, overlays, the animation snapshot).

import type { Record } from '../storage';
import type { BossInfo, CardView, Payout, PlayedCard, RiskLevel, RunView, ScoreView, Seat } from '../types';

/** What the table shows while events are being animated. */
export interface AnimState {
  trick: PlayedCard[];
  hand: CardView[];
  handSizes: number[];
  status: string;
  /** Seat whose card is expected next, for highlighting. */
  turn: Seat | null;
}

export type Overlay =
  | {
      kind: 'score';
      score: ScoreView;
      seriesScore: number;
      target: number;
      payout: { payout: Payout; coins: number } | null;
      shown: number;
      resolve: () => void;
    }
  | { kind: 'boss'; boss: BossInfo; resolve: () => void }
  | { kind: 'info'; title: string; subtitle: string; text: string; use?: number }
  | { kind: 'target'; slot: number }
  | { kind: 'menu' };

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
  toast: string | null;
  message: string | null;
  saveInfo: string | null;
  record: Record;
  /** The run ended: won or lost. */
  ended: { won: boolean; view: RunView } | null;
}
