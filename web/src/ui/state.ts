// UI state of the app. Holds the latest view from the core plus purely
// visual state (selection, overlays, the animation snapshot).

import type { SharedSeed } from '../seed';
import type { Quip } from './guests';
import type { Settings } from '../settings';
import type { DailyBoard } from '../account';
import type { AccountView, ConflictSide } from './account';
import type { RunRecord } from '../storage';
import type { SettledWild } from './cards';
import type { BossInfo, CardView, Character, DeckCard, MetaView, NullRisk, OwnedRelic, Payout, PlayedCard, RiskLevel, RunAction, RunLength, RunView, ScoreView, Seat } from '../types';

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
  /** A lost game that ends the run: the end screen sounds the loss, the board stays quiet. */
  quietLoss?: boolean;
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

/**
 * What applying a card modification would do to a card, as the core's
 * preview reported it (`DeckChanged`): replace another, raise the same,
 * net with it or cancel out (Augen and Ramponiert), or, drawn at random,
 * maybe replace.
 */
export interface ModQuestion {
  /** `apply`: a plain new Zinke (or effect) from a card picker, shown before and after (playtest 04.11.2026); `cleanse`: "Karte reinigen"; `randomMarke`: a Marke drawn when taken goes on the card (04.11.2026). */
  type: 'replace' | 'raise' | 'change' | 'drop' | 'random' | 'apply' | 'cleanse' | 'randomMarke';
  /** The card now and, from the preview's view, afterwards (null when drawn at random). */
  card: DeckCard;
  after: DeckCard | null;
  /** The modification on the card that goes (replace, drop, random). */
  old: string | null;
  /** The modification that comes (replace, raise, drop). */
  mod: string | null;
  /** The band texts before and after a raise ("+4", "+8"). */
  from: string;
  to: string;
  /** `randomMarke`: the Marken that may come, as text ("Pfandmarke, Totenmarke oder Narrenmarke"). */
  marken?: string;
  /** A fixed offer of card and Zinke (card reward, Auslage): the way back is "Anderes Angebot". */
  pair?: boolean;
  /** An offer of the Auslage: its price on the button. */
  price?: number;
}

export type Overlay =
  | ScoreOverlay
  | { kind: 'boss'; boss: BossInfo; resolve: () => void }
  /** Pick the card for a consumable; `reasons`: the core's error code per card it cannot take. */
  | { kind: 'target'; slot: number; reasons?: Record<number, string> }
  /** "Blue Curaçao": pick a card of the hand (`card`), then its new suit. */
  | { kind: 'chalk'; slot: number; card: number | null }
  /** Pick the card for an encounter option that needs one. */
  /** The card for an encounter option; `marke`: the Marke picked before (a chosen Marke on a chosen card). */
  | { kind: 'encounter-target'; option: number; reasons?: Record<number, string>; marke?: string }
  /** The card for the Zinke at `offer` of the card reward (DESIGN 5.0); back: the three Zinken. */
  /** Pick the Marke of a Marke option (Pfandleiher, Markensammlerin, 04.11.2026). */
  | { kind: 'encounter-marke'; option: number }
  | { kind: 'encounter-relic'; option: number }
  /** Tresen with every relic slot full (DESIGN 8.0): the owned relic that makes room for the offer in `slot`. */
  | { kind: 'relic-replace'; slot: number }
  /** Before a card modification replaces or raises the one on a card: ask, then `action`, or back (null: the screen). */
  | { kind: 'mod-confirm'; action: RunAction; back: Overlay | null; ask: ModQuestion }
  /** Zinken (DESIGN 8.6): pick the card for the Kartenmaler or "Karte reinigen". */
  | { kind: 'zinken-card'; service: 'painter' | 'cleanse'; reasons?: Record<number, string> }
  /** The Kartenmaler waits: take one of his Zinken (not closable). */
  | { kind: 'painter' }
  /** The Schnaps of a won pub with every slot full: take it for an owned one or sell it (not closable). */
  | { kind: 'schnapps-reward' }
  /** The Vereinsbuch: characters and unlocks, or the statistics page. */
  | { kind: 'book'; tab: 'book' | 'stats' }
  /** "Kneipe verlassen?" (DESIGN 8.7): give up the pub, then `GiveUpPub`. */
  | { kind: 'confirm-give-up' }
  /** Give up the running run for a new one? */
  | { kind: 'confirm-new'; start: PendingStart }
  /** Unlocks just earned, ids from the meta state. */
  | { kind: 'unlock'; ids: string[] }
  /** The first Untot or Wandelkarte of this save: what it does (DESIGN 7.2), once per special card. */
  | { kind: 'special'; ids: string[] }
  | { kind: 'menu' }
  /** "Spielstand": why to keep a copy, then export or import. */
  | { kind: 'save-menu' }
  /** The save code of everything stored, to copy or share; `copied` after a copy attempt. */
  | { kind: 'save-export'; code: string; copied: boolean | null }
  /** Paste a save code; `error` names why the last one was rejected. */
  | { kind: 'save-import'; text: string; error: string | null; checking: boolean }
  /** A checked save code, waiting for the player to replace the current progress. */
  | { kind: 'save-confirm'; keys: Record<string, string>; lines: string[] }
  /** The account (ADR 0021): sign up and in, or name, sign out, delete. */
  | { kind: 'account'; busy: boolean; error: string | null; note: string | null }
  /** The recovery code, shown once (sign-up, recovery login, renewed). */
  | { kind: 'account-code'; code: string; why: 'new' | 'recovered' | 'renewed'; copied: boolean | null }
  /** Sign in with the recovery code. */
  | { kind: 'account-recover'; busy: boolean; error: string | null }
  /** Delete the account? */
  | { kind: 'account-delete'; busy: boolean; error: string | null }
  /** Played on two devices: which state stays. */
  | { kind: 'sync-conflict'; here: ConflictSide; server: ConflictSide }
  | { kind: 'overview' }
  /** The Regelkarte: what is different here from the Skatordnung. */
  | { kind: 'rules' }
  /** The Glossar: every Zinke grouped by the frame language, with the value icons. */
  | { kind: 'glossary' }
  | { kind: 'credits' }
  /** Die lange Nacht (ADR 0020): the start sheet of a night district. */
  | { kind: 'night' };

export interface UiState {
  screen: 'loading' | 'start' | 'run' | 'end' | 'error';
  view: RunView | null;
  anim: AnimState | null;
  busy: boolean;
  /** Card ids picked for discarding. */
  selected: number[];
  /** The player chose "Hand spielen" before the skat: show the hand Ansage. */
  handGame: boolean;
  /** Index into the declare options' games. */
  pickGame: number | null;
  pickRisk: RiskLevel | null;
  /** Picked Null-Risikostufe (Nullspieler). */
  pickNullRisk: NullRisk | null;
  /** Skat swap ("Die Oma", "Der Opa"): the player picks the card to put into the skat. */
  omaPick: boolean;
  /** Skat card picked to take in a skat swap; null takes the core's first. */
  swapTake: number | null;
  /** Route choice: the way highlighted on the map (by option index) for the leg `routeLeg`. */
  routePick: number | null;
  routeLeg: string;
  /** Route choice: the whole tour as a line plan instead of the map ("−"). */
  routeOverview: boolean;
  /** Boss relic pick with every slot full: the offer chosen, before the owned relic it replaces. */
  rewardPick: number | null;
  /** The player looks at the last finished trick (the core's `last_trick`) instead of the open one. */
  peekTrick: boolean;
  /**
   * "Hand sortieren: manuell": the player's order of the hand by card id
   * (display only, ./handorder.ts); null keeps the core's order.
   */
  handOrder: number[] | null;
  /**
   * "Wandelkarte": cards the core settled in this game (event
   * `WildCardSettled`), by id: the card from now on and its printed suit.
   * For the places that only know the edition (overview, score); cleared
   * with the next deal.
   */
  wildSettled: Record<number, SettledWild>;
  /** "Untot": the card that last rose from a trick in this game and its substitute there (from the CardRose event), for a note in the last trick. */
  risen: { substitute: number; card: string } | null;
  /** "Untot": the card climbing back into the hand right now (CardRose), drawn with a green glow. */
  rising: number | null;
  /** "Untot": the hand cards picked so far for the rising card (core `rise_choice`). */
  risePick: number[];
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
  /** The night district whose start sheet was shown (ADR 0020). */
  nightSeen?: number | null;
  message: string | null;
  saveInfo: string | null;
  /** Unlocks and statistics from the core's meta state. */
  meta: MetaView | null;
  settings: Settings;
  /** The run ended: won or lost. */
  ended: { won: boolean; view: RunView } | null;
  /** Die Einführung is offered on the start screen (a new player, once; DESIGN 16.1). */
  tutorialOffer?: boolean;
  /** The account and its sync status (ADR 0021). */
  account: AccountView;
  /** Today's daily board, once fetched (null: not there or not reachable). */
  board: DailyBoard | null;
}
