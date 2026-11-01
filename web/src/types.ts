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
/** Risikostufe of a Null game (Nullspieler). */
export type NullRisk = 'Normal' | 'Hand' | 'Ouvert' | 'OuvertHand';
/** Spielart in a declaration: a GameType, or Ramsch (forced by "Der Ramscher"). */
export type DeclaredGame = GameType | 'Ramsch';
/** Who plays the run. */
export type Character = 'Declarer' | 'NullPlayer';
export type Position = 'Forehand' | 'Middlehand' | 'Rearhand';
export type RunPhase = 'Game' | 'Shop' | 'Encounter' | 'Won' | 'Lost';
export type TablePhase = 'Declaring' | 'Discarding' | 'Playing' | 'Finished';
export type RelicKind = 'HouseRule' | 'Regular';
export type Rarity = 'Common' | 'Uncommon' | 'Rare';
export type Kontra = 'None' | { Kontra: Seat } | { Re: Seat };

/**
 * A declaration as the core writes it: `{game, risk}`. Null carries a
 * NullRisk, Ramsch always `"Normal"`.
 */
export interface Declaration {
  game: DeclaredGame;
  risk: RiskLevel | NullRisk;
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
  passes_used: number;
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
  /** What it costs now (discounts applied by the core). */
  price: number;
  can_buy: boolean;
}

export interface ConsumableOffer {
  slot: number;
  consumable: ConsumableInfo;
  /** What it costs now (discounts applied by the core). */
  price: number;
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

/** A Null-Risikostufe in the Ansage of the Nullspieler, with its value from the core. */
export interface NullRiskOption {
  risk: NullRisk;
  allowed: boolean;
  value: number;
}

export interface DeclareOptions {
  games: GameOption[];
  risks: RiskOption[];
  /** Only for the Nullspieler. */
  null_risks: NullRiskOption[];
  /** "Der Angeber" demands Ouvert. */
  ouvert_required: boolean;
  can_pass: boolean;
  can_redeal: boolean;
  redeals_left: number;
  hand_required: boolean;
}

/**
 * A card the player sees in another seat: "Durchsichtig" (the card's
 * `revealed` flag is set) or shown by "Der Kiebitz" (flag not set).
 */
export interface RevealedCard {
  card: CardView;
  holder: Seat | null;
}

/** The count of "Der Zähler". */
export interface CountView {
  declarer: number;
  defenders: number;
  unseen: number;
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
  /** The boss rule of this game (for "Der Alte" the rule laid on this game). */
  boss_rule: BossInfo | null;
  count: CountView | null;
  /** "Die Oma" lets the player take a card from the skat now. */
  can_take_from_skat: boolean;
}

export type Source =
  | { Relic: string }
  | { CardMod: { card_mod: string; card: number } }
  | { Boss: string }
  | 'Kontra'
  | 'Re'
  | { Encounter: string };

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

/** Gewinnstufen by the rules, term by term. */
export interface LevelParts {
  matadors: number;
  game: number;
  hand: number;
  schneider: number;
  schneider_declared: number;
  schwarz: number;
  schwarz_declared: number;
  ouvert: number;
  sevens_ducked: number;
  never_followed: number;
  short_circuit: number;
  virgin: number;
}

export interface ScoreBreakdown {
  declaration: Declaration;
  outcome: Outcome;
  matadors: Matadors;
  won: boolean;
  schneider: boolean;
  schwarz: boolean;
  rule_base_value: number;
  rule_levels: LevelParts;
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
  | { SevensDucked: number }
  | 'NeverFollowed'
  | 'ShortCircuit'
  | { RamschLoser: number }
  | 'Virgin'
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

export interface CharacterInfo {
  character: Character;
  name: string;
  text: string;
}

/** What an encounter option does. */
export type EncounterChoice =
  | { TakeRule: string }
  | 'MarkRandomCard'
  | 'MarkChosenCard'
  | { SwapRelic: { index: number; relic: string } }
  | { DoubleNextGame: number }
  | 'Leave';

export interface EncounterOptionView {
  index: number;
  choice: EncounterChoice;
  cost: number;
  relic: RelicInfo | null;
  can_choose: boolean;
  needs_target: boolean;
  targets: number[];
}

/** The open encounter (Ereignis). */
export interface EncounterView {
  id: string;
  name: string;
  text: string;
  options: EncounterOptionView[];
}

export interface RunView {
  seed: string;
  character: CharacterInfo;
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
  encounter: EncounterView | null;
  /** House rules of the evening (Stammtisch-Streit), until the section ends. */
  evening_relics: RelicInfo[];
  /** Factor waiting for the next declared game ("Letzte Runde"). */
  pending_factor: number | null;
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
  | { DefenderHandsSwapped: { hand_sizes: number[] } }
  | { SkatSwapped: { took: DealtCard; gave: DealtCard } }
  | { GameFinished: unknown };

export type DeckChange =
  | { ReplacedByBlank: { removed: DeckCard; blank: number } }
  | { Removed: { removed: DeckCard } }
  | { FilledBlank: { card: number; blank: number } }
  | { Added: { card: number } }
  | { Modified: { card: number; card_mod: string } };

export type RunEvent =
  | 'Re'
  | 'ShopLeft'
  | 'Redealt'
  | 'RamschDealt'
  | 'EveningRulesEnded'
  | { RunWon: { character: Character } }
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
  | { SeriesWon: { section: number; index: number; payout: Payout; coins: number; boss: string | null; passes_used: number; games_left: number } }
  | { ShopOpened: unknown }
  | { RelicBought: { relic: string; price: number } }
  | { ConsumableBought: { consumable: string; price: number } }
  | { RelicSold: { relic: string; price: number } }
  | { ConsumableSold: { consumable: string; price: number } }
  | { ShopRerolled: { cost: number } }
  | { ConsumableUsed: { consumable: string } }
  | { DeckChanged: DeckChange }
  | { PassesAdded: { passes: number; passes_left: number } }
  | { CardPeeked: { card: DealtCard; seat: Seat } }
  | { EncounterOpened: { encounter: string; options: unknown[] } }
  | { EncounterResolved: { encounter: string; choice: EncounterChoice } }
  | { EveningRuleAdded: string }
  | { RelicSwapped: { old: string; new: string } }
  | { NextGameDoubled: { factor: number } }
  | { RunLost: { section: number; index: number; character: Character } };

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
  | { SellConsumable: number }
  | { TakeFromSkat: { give: number } }
  | { ChooseEncounter: { option: number; target: number | null } };

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
  characters: Record<string, CatalogEntry>;
  encounters: Record<string, CatalogEntry>;
  unlocks: Record<string, CatalogEntry>;
}

// ---------------------------------------------------------------------------
// Meta state across runs (unlocks and statistics, DESIGN 9.3)
// ---------------------------------------------------------------------------

export interface CharacterEntry {
  character: Character;
  name: string;
  text: string;
  unlocked: boolean;
  /** The unlock that frees it, if it is locked at the start. */
  unlocked_by: string | null;
}

export type Reward = { Relic: string } | { Character: Character };

export interface UnlockEntry {
  id: string;
  name: string;
  text: string;
  unlocked: boolean;
  rewards: Reward[];
}

export interface CharacterStats {
  runs: number;
  won: number;
  best_section: number;
}

export interface MetaStats {
  runs_finished: number;
  runs_won: number;
  best_section: number;
  games_won: number;
  games_lost: number;
  best_game: number;
  characters: Partial<Record<Character, CharacterStats>>;
}

export interface MetaView {
  characters: CharacterEntry[];
  unlocks: UnlockEntry[];
  locked_relics: string[];
  stats: MetaStats;
}

/** Result of feeding events to the meta state. */
export interface MetaStep {
  earned: string[];
  view: MetaView;
}
