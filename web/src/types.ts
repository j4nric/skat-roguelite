// Types of the JSON that skat_wasm hands out. They mirror the serde output
// of skat_core (run.rs, run/view.rs, content/score.rs, game.rs). The web app
// only reads these values; it never derives game facts from them.

export type Suit = 'Clubs' | 'Spades' | 'Hearts' | 'Diamonds';
export type Rank = '7' | '8' | '9' | 'T' | 'J' | 'Q' | 'K' | 'A';
export type Seat = 0 | 1 | 2;

/** Card kind: `{ Normal: "CJ" }` or `"Blank"`. */
export type CardKind = { Normal: string } | 'Blank';

export interface DealtCard {
  kind: CardKind;
  id: number;
  card_points: number;
  rank_bonus: number;
  revealed: boolean;
}

export interface CardView extends DealtCard {
  mods: string[];
  legal: boolean;
  from_skat: boolean;
}

export interface DeckCard {
  id: number;
  kind: CardKind;
  mods: string[];
}

export type GameType = { Suit: Suit } | 'Grand' | 'Null';
export type RiskLevel = 'Normal' | 'Hand' | 'SchneiderDeclared' | 'SchwarzDeclared' | 'Ouvert';
export type Position = 'Forehand' | 'Middlehand' | 'Rearhand';
export type RunPhase = 'Game' | 'Shop' | 'Won' | 'Lost';
export type TablePhase = 'Declaring' | 'Discarding' | 'Playing' | 'Finished';
export type RelicKind = 'HouseRule' | 'Regular';
export type Rarity = 'Common' | 'Uncommon' | 'Rare';
export type Kontra = 'None' | { Kontra: Seat } | { Re: Seat };

export interface Declaration {
  game: GameType;
  risk: RiskLevel;
}

export interface BossInfo {
  id: string;
  name: string;
  text: string;
  /** Factor on the Zielwert of the boss series, as a decimal (1.4). */
  target_factor: number;
}

/** Preview of the series after the current one. */
export interface NextSeriesView {
  section: number;
  index: number;
  is_boss: boolean;
  target: number;
  game_limit: number;
  passes: number;
  boss: BossInfo | null;
}

/** A probability from the core, with its value in thousandths. */
export interface Odds {
  numerator: number;
  denominator: number;
  permille: number;
}

/** Where any one card of the deck lands when a game is dealt (DESIGN 7.4). */
export interface DealOdds {
  deck_size: number;
  hand: Odds;
  skat: Odds;
  hand_or_skat: Odds;
  defenders: Odds;
  talon: Odds;
}

export interface SeriesView {
  section: number;
  index: number;
  is_boss: boolean;
  target: number;
  score: number;
  games_played: number;
  game_limit: number;
  games_left: number;
  passes_left: number;
  boss: BossInfo | null;
}

export interface RelicInfo {
  id: string;
  kind: RelicKind;
  name: string;
  text: string;
  rarity: Rarity;
  price: number;
}

export interface ConsumableInfo {
  id: string;
  name: string;
  text: string;
  rarity: Rarity;
  price: number;
  needs_target: boolean;
}

export interface OwnedRelic {
  index: number;
  relic: RelicInfo;
  sell_price: number;
  can_sell: boolean;
}

export interface OwnedConsumable {
  slot: number;
  consumable: ConsumableInfo;
  sell_price: number;
  can_sell: boolean;
  can_use: boolean;
  targets: number[];
}

export interface RelicOffer {
  slot: number;
  relic: RelicInfo;
  can_buy: boolean;
}

export interface ConsumableOffer {
  slot: number;
  consumable: ConsumableInfo;
  can_buy: boolean;
}

export interface ShopView {
  relics: (RelicOffer | null)[];
  consumables: (ConsumableOffer | null)[];
  reroll_cost: number;
  can_reroll: boolean;
  rerolls: number;
}

export interface PlayedCard {
  seat: Seat;
  card: CardView;
}

export interface TrickView {
  cards: PlayedCard[];
  winner: Seat;
}

export interface GameOption {
  game: GameType;
  base_value: number;
  allowed: boolean;
}

export interface RiskOption {
  risk: RiskLevel;
  allowed: boolean;
}

export interface DeclareOptions {
  games: GameOption[];
  risks: RiskOption[];
  can_pass: boolean;
  can_redeal: boolean;
  redeals_left: number;
  hand_required: boolean;
}

export interface RevealedCard {
  card: CardView;
  holder: Seat | null;
}

export interface GameView {
  number: number;
  phase: TablePhase;
  forehand: Seat;
  positions: Position[];
  declaration: Declaration | null;
  hand: CardView[];
  hand_sizes: number[];
  trick_lead: Seat;
  trick: PlayedCard[];
  last_trick: TrickView | null;
  tricks_played: number;
  declarer_tricks: number;
  defender_tricks: number;
  skat: CardView[] | null;
  your_turn: boolean;
  kontra: Kontra;
  awaiting_kontra_answer: boolean;
  kontra_possible: boolean;
  win_threshold: number;
  declare: DeclareOptions | null;
  revealed: RevealedCard[];
}

export type Source =
  | { Relic: string }
  | { CardMod: { card_mod: string; card: number } }
  | { Boss: string }
  | 'Kontra'
  | 'Re';

export type Lever = 'BaseValue' | 'Levels' | 'GameValue' | 'CardPoints' | 'WinThreshold' | 'Schneider';

export type Change = { Add: number } | { Multiply: number } | { Set: number };

export interface ScoreLine {
  source: Source;
  lever: Lever;
  change: Change;
}

export interface Outcome {
  declarer_card_points: number;
  defender_card_points: number;
  declarer_tricks: number;
  defender_tricks: number;
}

export interface Matadors {
  with: boolean;
  count: number;
}

export interface ScoreBreakdown {
  declaration: Declaration;
  outcome: Outcome;
  matadors: Matadors;
  won: boolean;
  schneider: boolean;
  schwarz: boolean;
  rule_base_value: number;
  lines: ScoreLine[];
  base_value: number;
  levels: number;
  multiplier: number;
  value: number;
  score: number;
}

export type ScoreTerm =
  | { BaseValue: { game: GameType; value: number } }
  | { NullValue: number }
  | { Matadors: { with: boolean; count: number } }
  | 'Game'
  | 'Hand'
  | 'Schneider'
  | 'SchneiderDeclared'
  | 'Schwarz'
  | 'SchwarzDeclared'
  | 'Ouvert'
  | { Line: ScoreLine };

export interface ScoreStep {
  term: ScoreTerm;
  base_value: number;
  levels: number;
  multiplier: number;
}

export interface ScoreView {
  breakdown: ScoreBreakdown;
  steps: ScoreStep[];
}

export interface RunStats {
  games_won: number;
  games_lost: number;
  games_passed: number;
  series_won: number;
  best_game: number;
}

export interface RunView {
  seed: string;
  phase: RunPhase;
  series: SeriesView;
  upcoming_boss: BossInfo | null;
  next_series: NextSeriesView | null;
  deal_odds: DealOdds;
  sections: number;
  coins: number;
  relics: OwnedRelic[];
  relic_slots: number;
  consumables: OwnedConsumable[];
  consumable_slots: number;
  flags: string[];
  counters: Record<string, number>;
  game: GameView | null;
  shop: ShopView | null;
  last_score: ScoreView | null;
  deck: DeckCard[];
  stats: RunStats;
}

export interface Payout {
  base: number;
  games_left: number;
  interest: number;
  total: number;
}

export interface CompletedTrick {
  lead: Seat;
  cards: DealtCard[];
  winner: Seat;
}

export type GameEvent =
  | { Dealt: unknown }
  | { SkatRedrawn: unknown }
  | { Declared: { declaration: Declaration } }
  | { SkatPickedUp: { skat: DealtCard[] } }
  | { Discarded: { cards: DealtCard[] } }
  | { CardPlayed: { seat: Seat; card: DealtCard } }
  | { TrickWon: { trick: CompletedTrick; card_points: number } }
  | { Conceded: { tricks: number; card_points: number } }
  | { GameFinished: unknown };

export type RunEvent =
  | 'Re'
  | 'ShopLeft'
  | 'Redealt'
  | 'RunWon'
  | { BossAnnounced: { section: number; boss: string } }
  | { SeriesStarted: { section: number; index: number; target: number; game_limit: number; passes: number; boss: string | null } }
  | { GameDealt: { number: number; forehand: Seat } }
  | { Game: GameEvent }
  | { Passed: { passes_left: number } }
  | { Kontra: { seat: Seat } }
  | { FlagSet: string }
  | { FlagCleared: string }
  | { CounterRaised: { counter: string; value: number } }
  | { GameScored: { breakdown: ScoreBreakdown; series_score: number } }
  | { SeriesWon: { section: number; index: number; payout: Payout; coins: number } }
  | { ShopOpened: unknown }
  | { RelicBought: { relic: string; price: number } }
  | { ConsumableBought: { consumable: string; price: number } }
  | { RelicSold: { relic: string; price: number } }
  | { ConsumableSold: { consumable: string; price: number } }
  | { ShopRerolled: { cost: number } }
  | { ConsumableUsed: { consumable: string } }
  | { DeckChanged: unknown }
  | { PassesAdded: { passes: number; passes_left: number } }
  | { RunLost: { section: number; index: number } };

export type RunAction =
  | 'Pass'
  | 'Redeal'
  | 'Reroll'
  | 'LeaveShop'
  | { Declare: Declaration }
  | { AnswerKontra: { re: boolean } }
  | { Discard: [number, number] }
  | { Play: number }
  | { UseConsumable: { slot: number; target: number | null } }
  | { BuyRelic: number }
  | { BuyConsumable: number }
  | { SellRelic: number }
  | { SellConsumable: number };

export interface Step {
  events: RunEvent[];
  view: RunView;
}

export interface CatalogEntry {
  name: string;
  text: string;
}

export interface Catalog {
  relics: Record<string, CatalogEntry>;
  card_mods: Record<string, CatalogEntry>;
  bosses: Record<string, CatalogEntry>;
  consumables: Record<string, CatalogEntry>;
}
