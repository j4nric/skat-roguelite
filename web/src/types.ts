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
  /** "Unsichtbar": never takes a trick. */
  never_wins?: boolean;
  /** "Unter ehrenhalber": a trump below the Schellen-Unter in suit games. */
  honorary_jack?: boolean;
  /** "Ramponiert": must not be discarded. */
  no_discard?: boolean;
  /** "Wandelkarte" of the player whose suit is not settled yet (until the game starts). */
  wild?: boolean;
  /** "Abgestuft" (Nullspieler): a Unter is the lowest card of its suit, any other card one step lower. */
  demoted?: boolean;
  /** Whose edition this copy is (ADR 0015); absent in old views: the player's. */
  owner?: Owner;
  /** "Winkel" (03.11.2026): how many steps the card rose; `kind` is the card it became (left out when 0). */
  promoted?: number;
  /** "Blei": the lowest card of its suit in Null (left out when false). */
  leaden?: boolean;
}

/** Whose edition a card is (ADR 0015): every player owns his own edition of each card. */
export type Owner = 'Player' | 'Guest' | 'Boss' | 'BossPartner';

export interface CardView extends DealtCard {
  /** Modifications in the edition of the card's owner (its holder). */
  mods: string[];
  /** Card back of an opponent's card (ui/backs.ts); absent for the player's own. */
  back?: string;
  /** Effekte v2: the value of each modification that has one (absent: 1). */
  mod_values?: Record<string, number>;
  legal: boolean;
  from_skat: boolean;
  /** "Drei drücken": the defender this card was drawn from with the skat, while discarding. */
  drawn_from?: Seat | null;
  /**
   * Whether the card may go into the skat now (hand cards while discarding;
   * a "Ramponiert" card only when fewer than two others are left).
   */
  discardable: boolean;
  /** "Blue Curaçao": the printed suit of a card repainted for this game; `kind` is the suit it has now. */
  chalked?: Suit | null;
  /**
   * Once the game is declared: whether each conditional modification on
   * this card works in this game (core `CardModStatus`). Empty before the
   * declaration; modifications without a condition are left out. Optional
   * for views from an older core.
   */
  mod_status?: Partial<Record<string, CardModStatus>>;
  /** "Wandelkarte" whose suit is still open: before the game starts (left out when false). */
  unsettled?: boolean;
  /** A Wandelkarte settled as this game started: its printed suit; `kind` has the settled suit. */
  named?: Suit | null;
  /** "Untot": would rise if played now (left out when false). */
  can_rise?: boolean;
}

/** Whether a card modification works in the game in progress: yes, no, or it depends on how the game goes. */
export type CardModStatus = 'Active' | 'Inactive' | 'Open';

export interface DeckCard {
  id: number;
  kind: CardKind;
  mods: string[];
  /** Effekte v2: the value of each modification that has one (absent: 1). */
  mod_values?: Record<string, number>;
}

export type GameType = { Suit: Suit } | 'Grand' | 'Null';
export type RiskLevel = 'Normal' | 'Hand' | 'SchneiderDeclared' | 'SchwarzDeclared' | 'Ouvert';
/** Risikostufe of a Null game (Nullspieler). */
export type NullRisk = 'Normal' | 'Hand' | 'Ouvert' | 'OuvertHand';
/** Spielart in a declaration: a GameType, or Ramsch (forced by "Der Ramscher"). */
export type DeclaredGame = GameType | 'Ramsch';
/** Who plays the run. */
export type Character = 'Declarer' | 'NullPlayer' | 'Shirker' | 'Tailoress' | 'HeartBrother';
/** Every character, in the order of the core (`Character::ALL`). */
export const CHARACTERS: readonly Character[] = ['Declarer', 'NullPlayer', 'Shirker', 'Tailoress', 'HeartBrother'];
/** Ganzer Abend (5 sections) or kurzer Abend (3 sections), DESIGN 5.3. */
export type RunLength = 'Full' | 'Short';
export type Position = 'Forehand' | 'Middlehand' | 'Rearhand';
export type RunPhase = 'Game' | 'Shop' | 'Encounter' | 'Route' | 'BossReward' | 'Won' | 'Lost';
/**
 * Phase of the game, in the order of the Skatordnung: ten cards (hand game or
 * pick up the skat), twelve cards (discard two), the Ansage after the skat,
 * tricks, scored.
 */
export type TablePhase = 'Declaring' | 'Discarding' | 'DeclaringAfterSkat' | 'Playing' | 'Finished';
export type RelicKind = 'HouseRule' | 'Regular';
export type Rarity = 'Common' | 'Uncommon' | 'Rare' | 'Legendary';
export type Kontra = 'None' | { Kontra: Seat } | { Re: Seat };

/**
 * A declaration as the core writes it: `{game, risk}`. Null carries a
 * NullRisk, Ramsch always `"Normal"`. Schneider, Schwarz or Ouvert announced
 * after the skat (ADR 0013) carry `after_skat: true`; sent, the core takes
 * it from the phase.
 */
export interface Declaration {
  game: DeclaredGame;
  risk: RiskLevel | NullRisk;
  after_skat?: boolean;
}

export interface BossInfo {
  id: string;
  name: string;
  text: string;
  /** Factor on the Zielwert of the boss pub, as a decimal (1.4). */
  target_factor: number;
  /** The two boss players (ADR 0017): left of the player, then right. */
  figures: BossFigure[];
  /** Card back of the boss's cards (ui/backs.ts). */
  back?: string;
}

/** One of the two boss players at the boss table (ADR 0017). */
export interface BossFigure {
  name: string;
  /** The figure's edition (ADR 0015): the modifications its cards carry. Empty for Der Alte and Die Alte. */
  edition: EditionEntry[];
  /** Card back of the figure's cards. */
  back: string;
}

/** The relic pick after a won boss pub (ADR 0017). */
export interface BossRewardView {
  offers: { index: number; relic: RelicInfo }[];
  /** Every relic slot taken: a relic only for an owned one (sold for half its price). */
  slots_full: boolean;
}

/** One line of a boss edition: every card matching the filter carries the modification. */
export interface EditionEntry {
  filter: { suit?: Suit | null; rank?: string | null };
  card_mod: CardModInfo;
}

/** Preview of the series after the current one. */
export interface NextSeriesView {
  section: number;
  index: number;
  is_boss: boolean;
  target: number;
  game_limit: number;
  /** Einmischen per game of the character, without relics. */
  redeals: number;
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
  /** Einmischen left now: this game's own plus the series' extras ("Ein Bier"). */
  redeals_left: number;
  redeals_used: number;
  boss: BossInfo | null;
  /** The Stammgäste at the table: seat 1 (left) and seat 2 (right). In a boss series the boss takes the left seat. */
  guests: GuestInfo[];
}

/** A Stammgast (normal opponent): name, face and speech bubble lines, all from the core content. */
export interface GuestInfo {
  seat: Seat;
  id: string;
  name: string;
  /** Id of the face in ui/people.ts. */
  look: string;
  greet: string;
  win: string;
  lose: string;
  /** Card back of his cards (ui/backs.ts). */
  back?: string;
}

/** Display group of a relic (shape of its beer mat), from the content. */
export type RelicGroup = 'Value' | 'Multiplier' | 'Control' | 'Tour';

export interface RelicInfo {
  id: string;
  kind: RelicKind;
  name: string;
  text: string;
  rarity: Rarity;
  price: number;
  /** Absent in an older core: Value. */
  group?: RelicGroup;
}

export interface CardModInfo {
  id: string;
  name: string;
  text: string;
  rarity: Rarity;
  /** A curse (Fluch). */
  curse: boolean;
  kind?: ModKind;
  /** The value it comes with (the text has it filled in). */
  value?: number | null;
}

/** Group of a consumable, for labels: the Schnäpse are `Schnapps`. */
export type ConsumableCategory = 'Item' | 'Schnapps';

export interface ConsumableInfo {
  id: string;
  name: string;
  text: string;
  rarity: Rarity;
  price: number;
  /** Needs a deck card (at the Tresen). */
  needs_target: boolean;
  /** "Blue Curaçao": needs a card of the hand (one of `targets`) and another suit. */
  needs_suit?: boolean;
  category?: ConsumableCategory;
  /** Where the Schnaps is poured (drag and drop, DESIGN 8.9). Presentation only; see `usable_now`. */
  target?: ConsumableTarget;
}

/** What a Schnaps is poured on: the skat, the table, a hand card, the player himself, the city map. */
export type ConsumableTarget = 'Skat' | 'Table' | 'Card' | 'Self' | 'Map';

/** The Schnaps of a won pub with every slot full (DESIGN 8.9): take it for an owned one or sell it. */
export interface SchnappsRewardView {
  consumable: ConsumableInfo;
  /** Groschen when sold (half its price). */
  sell_price: number;
}

/** What a Schnaps does to the next declared game. */
export type NextGameEffect = { Multiply: number } | { MultiplyWin: number } | 'SingleLoss';

/** A Schnaps waiting for the next declared game. */
export interface PendingNextGame {
  consumable: string;
  effect: NextGameEffect;
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
  /** Whether the core would accept drinking it now (until 03.11.2026 `can_use`). */
  usable_now: boolean;
  targets: number[];
}

export interface RelicOffer {
  slot: number;
  relic: RelicInfo;
  /** What it costs now (discounts applied by the core, half for Hehlerware). */
  price: number;
  can_buy: boolean;
  /** "Hehlerware" (curses as trade): half price, buying it curses a random card. */
  fence?: boolean;
  /** For Hehlerware: the price without the trade. */
  was?: number | null;
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
  /** Zinken (DESIGN 8.6): the Auslage, null once bought. Absent in old cores. */
  card_offers?: (CardOfferView | null)[];
  /** Price of the next "Neu auslosen". */
  card_reroll_cost?: number;
  can_reroll_cards?: boolean;
  card_rerolls?: number;
  /** The Kartenmaler. */
  painter?: ServiceView;
  /** "Karte reinigen". */
  cleanse?: ServiceView;
  /** A Kiosk (03.11.2026), not the Tresen: two Schnäpse, one may be bought. */
  kiosk?: boolean;
  /** At a Kiosk: Groschen off each Schnaps against the Tresen. */
  kiosk_discount?: number;
}

/** An offer of the Auslage: a card of the edition with a Zinke. */
export interface CardOfferView {
  slot: number;
  /** The card now. */
  card: DeckCard;
  /** The card after buying. */
  result: DeckCard;
  card_mod: CardModInfo;
  price: number;
  can_buy: boolean;
  /** The modification it would replace (ask first). */
  replaces: string | null;
  /** It would raise the same modification. */
  raises: boolean;
  /** "Schnäppchen" (curses as trade): half price, buying it curses another card. */
  bargain?: boolean;
  /** For a Schnäppchen: the price without the trade. */
  was?: number | null;
}

/** A card service of the Tresen (Kartenmaler, Karte reinigen), once per visit. */
export interface ServiceView {
  price: number;
  used: boolean;
  can_use: boolean;
  /** Cards it can work on now. */
  targets: number[];
  /** Its price on each target, same order (Bruderherz: half on Rot). */
  target_prices?: number[];
}

/** The Kartenmaler's choice, waiting for the player. */
export interface PainterView {
  card: DeckCard;
  options: PainterOptionView[];
  /** Hired at the Werkstatt. */
  encounter: boolean;
}

export interface PainterOptionView {
  index: number;
  card_mod: CardModInfo;
  result: DeckCard;
  replaces: string | null;
  raises: boolean;
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

/**
 * What the player may declare now. Before the skat the lists hold only the
 * hand levels; after picking it up and discarding only the levels without
 * Hand. The core decides which are allowed.
 */
export interface DeclareOptions {
  games: GameOption[];
  risks: RiskOption[];
  /** Null levels of this phase; every character may declare Null (ADR 0012). */
  null_risks: NullRiskOption[];
  /** Whether a won Null earns a style bonus (the Nullspieler's formula). */
  null_style: boolean;
  /** Whether the skat may be picked up (RunAction `PickUpSkat`). */
  can_pick_up_skat: boolean;
  /** "Der Angeber" demands Ouvert. */
  ouvert_required: boolean;
  /** Einmischen possible now (RunAction `Redeal`). */
  can_redeal: boolean;
  redeals_left: number;
  hand_required: boolean;
}

/**
 * Why the player knows where a card is: a modification (by id), "Der
 * Kiebitz", or "Der Zaungast" (the whole left hand).
 */
export type RevealSource = { CardMod: string } | 'Kibitz' | 'Onlooker' | 'Returned';

/** What a defender must lead when the player names it ("Lockvogel"): trump or a suit. */
export type LeadSuit = 'Trump' | { Suit: Suit };

/** A card the player sees in another seat, and why. */
export interface RevealedCard {
  card: CardView;
  holder: Seat | null;
  source: RevealSource;
}

/** The count of "Der Zähler". */
export interface CountView {
  declarer: number;
  defenders: number;
  unseen: number;
  /** Augen per seat (index = seat): who has the most in a Ramsch. */
  seats: [number, number, number];
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
  /** "Wandelkarte": the cards settled as this game started, played ones too (absent in old views). */
  wild_settled?: { card: number; kind: CardKind; printed: Suit }[];
  /** The boss rule of this game (for "Der Alte" the rule laid on this game). */
  boss_rule: BossInfo | null;
  /** Card back per seat (ADR 0015): null for the player (his own choice), the opponent's back otherwise. */
  backs?: (string | null)[];
  count: CountView | null;
  /** "Die Oma" (or "Der Opa") lets the player take a card from the skat now. */
  can_take_from_skat: boolean;
  /** Skat cards a skat swap may take now; the first is taken by default. */
  skat_swap_cards: CardView[];
  /** The untouched skat the player looked at ("Kurzer"); null if not, and once picked up. */
  skat_peek?: CardView[] | null;
  /** Tricks per seat so far (index = seat). */
  seat_tricks: [number, number, number];
  /**
   * What the game is worth if the player wins it, as far as it is certain
   * now (core `ValuePreview`). Only while playing a suit game, grand or
   * Null; `null` otherwise. Optional for views from an older core.
   */
  value_preview?: ValuePreview | null;
  /**
   * The rest of the game can no longer change its score, however anyone
   * plays: the player may end it with `SettleGame` ("Rest abkürzen").
   * Optional for views from an older core.
   */
  can_settle?: boolean;
  /** Cards to press now: 2, or 3 with "Drei drücken"; 0 outside Discarding. Optional for views from an older core. */
  discard_count?: number;
  /** Skat cards "Spickzettel" showed after a hand declaration, while they lie in the skat. */
  skat_shown?: CardView[];
  /** "Drei drücken" (first game of each pub) draws a card at the next pickup of the skat. */
  discard_three_ready?: boolean;
  /** "Untot": the card in the last trick waits for three cards of the hand (ChooseRiseCards). */
  rise_choice?: RiseChoiceView;
}

/** "Untot" (core `RiseChoiceView`): pick `pick` cards of the hand, one of them goes into the trick. */
export interface RiseChoiceView {
  /** The card as it lies in the trick. */
  card: CardView;
  /** The card it comes back as: a copy of the trick's winning card. */
  becomes: CardView;
  pick: number;
}

/** The table's score line while playing, computed by the core. */
export interface ValuePreview {
  /** Grundwert after the certain modifications. */
  base_value: number;
  /** Gewinnstufen known so far. */
  levels: number;
  /** Product of the certain Spielwert factors (Kontra, Re, …), 1 without. */
  multiplier: number;
  /** base_value × levels × multiplier. */
  game_value: number;
}

export type Source =
  | { Relic: string }
  | { CardMod: { card_mod: string; card: number } }
  | { Boss: string }
  | 'Kontra'
  | 'Re'
  | { Encounter: string }
  | { Consumable: string }
  /** The own rule of the character ("Der Grandspieler"). */
  | { Character: Character }
  /** The malus of a Spelunke (ADR 0016). */
  | { Malus: string };

export type Lever = 'BaseValue' | 'Levels' | 'GameValue' | 'CardPoints' | 'WinThreshold' | 'Schneider' | 'LossFactor';

/** `Scale`: a factor that need not be whole, as a decimal (1.5, "Stammfarbe"). */
export type Change = { Add: number } | { Multiply: number } | { Scale: number } | { Set: number };

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
  /** Augen and tricks per seat, set by the run (who has the most in a Ramsch). */
  seats?: SeatTotals | null;
}

/** Augen and tricks per seat (index = seat). */
export interface SeatTotals {
  card_points: [number, number, number];
  tricks: [number, number, number];
  /** Who has the most Augen (several on a tie): in a Ramsch the losers. */
  most: [boolean, boolean, boolean];
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
  /** Product of the Spielwert factors so far, a decimal (2, 1.5). */
  multiplier: number;
}

export interface ScoreView {
  breakdown: ScoreBreakdown;
  steps: ScoreStep[];
}

export interface RunStats {
  games_won: number;
  games_lost: number;
  /** Einmischen in the run. */
  redeals: number;
  series_won: number;
  /** Pubs lost (each cost a life, ADR 0016). */
  pubs_lost?: number;
  best_game: number;
}

export interface CharacterInfo {
  character: Character;
  name: string;
  text: string;
  /** Card back of the player's edition (DESIGN 9.3); absent in an older core (Distel). */
  back?: string;
}

/** Which card a Mark effect goes on. */
export type CardPick = 'Random' | 'Chosen';

/**
 * One effect of an encounter option (DESIGN 13, "Wege und Begegnungen v2",
 * 03.11.2026). Draws are filled in when the encounter opens (`null`: nothing
 * was left to draw; the option is blocked then).
 */
export type EncounterEffect =
  | { Coins: number }
  | { EveningRule: string | null }
  | {
      Mark: {
        card: CardPick;
        card_mod: string | null;
        value: number | null;
        offers?: [string, number][];
        curse?: boolean;
        curse_percent?: number;
      };
    }
  | { SwapRelic: string | null }
  | { SellRelic: number }
  | { Relic: string | null }
  | { RelicCoins: number }
  | { NextGameFactor: number }
  | { NextPubRedeals: number }
  | 'SeeSkat'
  | { Schnapps: string | null }
  | { Lives: number }
  | { Scout: number }
  | { Bet: { on: 'Schneider' | 'Win'; percent: number } }
  | { Dice: { chance: number; percent: number } }
  | 'Painter'
  | 'Cleanse';

/** Why an option cannot be taken now (stable names). */
export type OptionBlock =
  | 'NotEnoughCoins'
  | 'NothingLeft'
  | 'SpecialCardCap'
  | 'NoFittingCard'
  | 'NoRelic'
  | 'RelicSlotsFull'
  | 'LivesFull'
  | 'BetOpen'
  | 'NotNeeded';

/** One effect as the player sees it, with what was drawn for it. */
export interface EncounterEffectView {
  effect: EncounterEffect;
  relic: RelicInfo | null;
  card_mod: CardModInfo | null;
  consumable: ConsumableInfo | null;
  /** A Schnaps with every slot full: these Groschen instead. */
  coins_instead: number | null;
}

export interface EncounterOptionView {
  index: number;
  /** German title from the content. */
  label: string;
  /** Groschen it costs (the stake of a bet or a throw of the dice). */
  cost: number;
  effects: EncounterEffectView[];
  /** Offered only when no other option can be taken ("Zuschauen"). */
  fallback: boolean;
  can_choose: boolean;
  blocked: OptionBlock | null;
  /** What it needs as target: a card of the edition or an owned relic (its index). */
  target: 'None' | 'Card' | 'Relic';
  targets: number[];
  /** Cost on each of `targets` (a Zinke on Rot is cheaper for Bruderherz). */
  target_costs?: number[];
}

/** The open encounter (Ereignis); one option must be taken. */
export interface EncounterView {
  id: string;
  name: string;
  text: string;
  category: StopCategory;
  options: EncounterOptionView[];
}

export interface RunView {
  seed: string;
  character: CharacterInfo;
  phase: RunPhase;
  /**
   * The defender who thinks next. While set, the run waits for the AI:
   * call `CoreRun.step()` (one card) until it is null; actions fail with
   * `AiToMove`.
   */
  ai_to_move: Seat | null;
  series: SeriesView;
  upcoming_boss: BossInfo | null;
  next_series: NextSeriesView | null;
  deal_odds: DealOdds;
  /** Districts (Stadtteile) of this run. */
  sections: number;
  /** Always the ganzer Abend since ADR 0016. */
  length: RunLength;
  /** The Kneipentour: lives, the city, the route choice (ADR 0016). */
  tour: TourView;
  /** Stake of the Einsatz-Leiter (DESIGN 9.3), 0 for the normal game. */
  stake: number;
  /** Augen a suit game or Grand needs in this run before relics: 61 + stake. */
  win_threshold: number;
  coins: number;
  relics: OwnedRelic[];
  /** Relic slots; null: any number (unlimited since 03.11.2026). */
  relic_slots: number | null;
  consumables: OwnedConsumable[];
  consumable_slots: number;
  flags: string[];
  counters: Record<string, number>;
  game: GameView | null;
  shop: ShopView | null;
  encounter: EncounterView | null;
  /** The relic pick after a won boss pub (ADR 0017). */
  boss_reward?: BossRewardView | null;
  /** The Kartenmaler's choice, waiting (Tresen or Werkstatt). */
  painter?: PainterView | null;
  /** The Schnaps of a won pub while every slot is full: nothing but `ChooseSchnappsReward` goes now. */
  schnapps_reward?: SchnappsRewardView | null;
  /** House rules of the evening (Stammtisch-Streit), until the section ends. */
  evening_relics: RelicInfo[];
  /** Factor waiting for the next declared game ("Letzte Runde"). */
  pending_factor: number | null;
  /** Schnäpse waiting for the next declared game ("Doppelkorn"). */
  pending_next_game?: PendingNextGame[];
  /** Changes waiting for the next pub (Wegbier, Skatlehrer, Kiebitz am Fenster). */
  next_pub?: NextPubPlan;
  /** Groschen owed at the Tresen ("Deckel anschreiben"). */
  debt?: number;
  /** How far the Tresen sells on credit now. */
  credit_left?: number;
  /** The bet on the next declared game ("Die Wette"). */
  wager?: { encounter: string; stake: number; factor: number; on?: 'Schneider' | 'Win'; percent?: number } | null;
  /** Route choices still to come whose stops are shown (Kiebitz am Fenster). */
  scout_legs?: number;
  last_score: ScoreView | null;
  /** "Kneipe verlassen" goes now: between two games, from the second on (DESIGN 8.7). */
  can_give_up?: boolean;
  /** "Noch nötig" before the last game of a pub (DESIGN 8.7). */
  needed?: NeededView | null;
  /** The player's edition of the 32 cards (ADR 0015). */
  edition: DeckCard[];
  stats: RunStats;
}

// ---------------------------------------------------------------------------
// Kneipentour (ADR 0016)
// ---------------------------------------------------------------------------

/** How the player goes to the next pub; flavour only, every way is alike. */
export type Transport = 'Walk' | 'Train' | 'Bike';
/** The kind of a stop on the way. */
export type StopCategory = 'Tresen' | 'Event' | 'Upgrade' | 'Rest' | 'FoundCoins' | 'Kiosk' | 'DiceStall';
export type PubState = 'Won' | 'Lost' | 'Current' | 'Ahead' | 'Skipped';

export interface MalusInfo {
  id: string;
  name: string;
  text: string;
}

/** A pub before or while it is played. */
export interface PubPreview {
  /** Stable node id: `d0-p3`, a Spelunke `d0-p3-s`. */
  id: string;
  section: number;
  index: number;
  name: string;
  district: string;
  target: number;
  game_limit: number;
  is_boss: boolean;
  boss: BossInfo | null;
  guests: GuestInfo[];
  /** The malus of a Spelunke. */
  malus: MalusInfo | null;
}

/** Changes waiting for the next pub. */
export interface NextPubPlan {
  redeals: number;
  target: number;
}

export interface RouteOptionView {
  /** Stable node id of the way (`d0-p3-r1`). */
  id: string;
  index: number;
  transport: Transport;
  /** Decorative travel time. */
  minutes: number;
  can_choose: boolean;
  /** `encounter` only while "Kiebitz am Fenster" scouts. */
  stops: { category: StopCategory; encounter?: { id: string; name: string } | null }[];
  /** The Spelunke it leads into instead of the next pub. */
  spelunke: PubPreview | null;
  /** "Spelunkenkenner": maluses to choose from (RunAction `ChooseMalus`); empty without. */
  malus_options?: MalusInfo[];
}

export interface RouteView {
  next: PubPreview;
  options: RouteOptionView[];
}

export interface PubNode {
  id: string;
  index: number;
  name: string;
  is_boss: boolean;
  state: PubState;
}

export interface SpelunkeNode {
  id: string;
  index: number;
  name: string;
  state: PubState;
}

/** How a district looks on the map (core/content/tour.ron). */
export type DistrictLook = 'OldTown' | 'Harbour' | 'Station';

export interface DistrictView {
  index: number;
  name: string;
  /** Look on the map, data of core/content/tour.ron. */
  look: DistrictLook;
  boss: BossInfo | null;
  map_seed: number;
  pubs: PubNode[];
  spelunken: SpelunkeNode[];
}

export interface TourView {
  lives: number;
  /** Lives at the start of a run: the head never shows an empty heart above them. */
  start_lives: number;
  max_lives: number;
  map_seed: number;
  districts: DistrictView[];
  current: PubPreview | null;
  route: RouteView | null;
  trip: { transport: Transport; stops_left: number; spelunke: boolean } | null;
}

export interface Payout {
  base: number;
  games_left: number;
  interest: number;
  /** Extra from the character ("Die Wirtin": the parts once more); absent in old saves. */
  bonus?: number;
  /** A won boss pub pays double (ADR 0017): everything above once more. */
  boss?: number;
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
  | { CardChalked: { old: DealtCard; new: DealtCard } }
  /** "Wandelkarte": its suit was settled as the game started; `new` is the card from now on. */
  | { WildCardSettled: { old: DealtCard; new: DealtCard } }
  /** "Drei drücken": drawn from a defender with the skat. */
  | { CardDrawn: { seat: Seat; card: DealtCard } }
  /** "Drei drücken": the lowest discard went back to that defender. */
  | { CardReturned: { seat: Seat; card: DealtCard } }
  /** "Lockvogel" (removed): only in old saves. */
  | { LeadDemanded: { suit: LeadSuit } }
  /** "Untot": the card rose back into the hand, the substitute lies in the trick. */
  | { CardRose: { seat: Seat; card: DealtCard; substitute: DealtCard; risen?: DealtCard } }
  /** "Doppelgänger": `stolen` from `from` lies in the trick for `card`; `from` holds `given` now. CardPlayed (stolen) follows. */
  | { DoppelgangerSwapped: { seat: Seat; card: DealtCard; from: Seat; stolen: DealtCard; given: DealtCard } }
  /** ADR 0015: a card changed its holder and shows his edition (the card only if the player knows it). */
  | { CardChangedHands: { seat: Seat; card?: DealtCard } }
  | { GameFinished: unknown };

export type DeckChange =
  | { Modified: { card: number; card_mod: string; value?: number; replaced?: string | null } }
  | { Unmodified: { card: number; card_mod: string } }
  /** Reinigen ("Karte reinigen", Werkstatt): every modification fell off the card. */
  | { Cleansed: { card: number; removed: string[] } };

/** "Noch nötig": what the last game of a pub has to bring (core `NeededView`). */
export interface NeededView {
  /** Zielwert minus account; 0 or less when the target is reached. */
  points: number;
  /** Smallest plain Spielwert (no relics) of at least `points`. */
  smallest_game: number | null;
  /** Upper bound of a won game's value now. */
  max_value: number;
  /** Whether a won game can still reach the target. */
  reachable: boolean;
}

export type RunEvent =
  | 'Re'
  | 'ShopLeft'
  | 'Redealt'
  | 'RedealUndone'
  | 'RamschDealt'
  | 'EveningRulesEnded'
  | { RunWon: { character: Character; length?: RunLength; stake?: number } }
  | { BossAnnounced: { section: number; boss: string } }
  | { SeriesStarted: { section: number; index: number; target: number; game_limit: number; redeals: number; boss: string | null; malus?: string } }
  /** Kneipentour (ADR 0016). */
  | { RouteOpened: { section: number; index: number } }
  | { RouteChosen: { option: number; transport: Transport; spelunke: boolean } }
  | { PubLost: { section: number; index: number; boss: string | null } }
  | { LivesChanged: { change: number; lives: number } }
  | { RelicGained: { relic: string } }
  | { GameDealt: { number: number; forehand: Seat } }
  | { Game: GameEvent }
  /** "Untot": the card waits in the trick for `pick` cards of the hand; `becomes` is what it comes back as. */
  | { RiseChoice: { card: DealtCard; becomes: DealtCard; pick: number } }
  /** "Doppelgänger": nobody held a counterpart; it is played as itself. */
  | { DoppelgangerUnmatched: { card: DealtCard } }
  | { Kontra: { seat: Seat } }
  | { FlagSet: string }
  | { FlagCleared: string }
  | { CounterRaised: { counter: string; value: number } }
  | { CounterReset: { counter: string } }
  /** "Skatpfleger": the discarded skat counts `card_points` Augen more for the player. */
  | { SkatPointsRaised: { source: Source; card_points: number } }
  | { CoinsGained: { source: Source; amount: number; coins: number } }
  | { GameScored: { breakdown: ScoreBreakdown; series_score: number } }
  | { SeriesWon: { section: number; index: number; payout: Payout; coins: number; boss: string | null; redeals_used: number; games_left: number } }
  | { ShopOpened: unknown }
  | { RelicBought: { relic: string; price: number } }
  | { ConsumableBought: { consumable: string; price: number } }
  | { RelicSold: { relic: string; price: number } }
  | { ConsumableSold: { consumable: string; price: number } }
  | { ShopRerolled: { cost: number } }
  | { CardOffersDrawn: { offers: ({ card: number; card_mod: string; value: number; price: number } | null)[] } }
  | { CardOfferBought: { slot: number; card: number; card_mod: string; value: number; price: number } }
  | { CardOffersRerolled: { cost: number } }
  | { PainterHired: { card: number; options: [string, number][]; price: number } }
  | { PainterChosen: { card: number; card_mod: string; value: number } }
  | { CardCleansed: { card: number; price: number } }
  /** Curses as trade: a Schnäppchen, Hehlerware or the Schnaps Fusel cursed a card for good. */
  | { TradeCurse: { card: number; curse: string; cause: 'Bargain' | 'Fence' | 'Schnapps' } }
  | { ConsumableUsed: { consumable: string } }
  | { DeckChanged: DeckChange }
  | { RedealsAdded: { redeals: number; redeals_left: number } }
  | { CardPeeked: { card: DealtCard; seat: Seat } }
  /** Effekte v2: a voucher was redeemed when its card was discarded; `via_hand`: by declaring a hand game (04.11.2026), the card stays in the hand. */
  | { VoucherRedeemed: { card: number; card_mod: string; value: number; via_hand?: boolean } }
  | { VoucherReturned: { card: number; card_mod: string; value: number } }
  /** A Totenmarke or Narrenmarke whose special card another card is already (one each per edition): Groschen instead. */
  | { VoucherCapped: { card: number; card_mod: string; special: string; holder: number; coins: number } }
  /** Effekte v2 ("Erster Stich"): extra Augen for a seat. */
  | { BonusPoints: { source: Source; seat: Seat; card_points: number } }
  | { EncounterOpened: { encounter: string; options: unknown[] } }
  | { EncounterResolved: { encounter: string; option: number } }
  /** The dice fell at the Würfelbude. */
  | { DiceThrown: { stake: number; won: boolean; payout: number } }
  /** Both skat cards of the next game will be shown. */
  | 'SkatPeekPlanned'
  /** A Kiosk opened on the way. */
  | { KioskOpened: unknown }
  | { EveningRuleAdded: string }
  | { RelicSwapped: { old: string; new: string } }
  | { NextGameDoubled: { factor: number } }
  | { SkatPeeked: { skat: DealtCard[] } }
  /** "Spickzettel": a card of the untouched skat after a hand declaration. */
  | { SkatCardShown: { source: Source; card: DealtCard } }
  | { RunLost: { section: number; index: number; character: Character; length?: RunLength; stake?: number; boss?: string | null } }
  /** "Die Oma" (ADR 0017): cards of the player's edition cursed for the pub. */
  | { BossCursesLaid: { boss: string; cards: [number, string][] } }
  | { BossCursesLifted: { cards: number[] } }
  | { BossRewardOffered: { relics: string[] } }
  | 'BossRewardDeclined'
  | { SchnappsRewarded: { consumable: string; slots_full: boolean } }
  | { SchnappsRewardTaken: { consumable: string; given_up: string } }
  /** Item review (02.11.2026). */
  | 'RoutesRerolled'
  | { DebtChanged: { debt: number } }
  | { LifeSpared: { source: Source } }
  | { CoinsStashed: { source: Source; amount: number; pot: number; coins: number } }
  /** A relic that stashes Groschen left; its pot was paid out as it was. */
  | { StashPaidOut: { relic: string; amount: number; coins: number } }
  | { NextPubPlanned: { redeals: number; target: number } }
  | { NextPubApplied: { redeals: number; target: number } }
  | { WagerPlaced: { stake: number } }
  | { WagerSettled: { stake: number; won: boolean; payout: number } }
  | { StopsRevealed: { legs: number } }
  | { MalusChosen: { option: number; malus: string } }
  | { GambleResolved: { curse: boolean } }
  | { ConsumableGained: { consumable: string } }
  /** Pub rescue (DESIGN 8.7). */
  | { PubGivenUp: { section: number; index: number; score: number } }
  | { AccountCleared: { before: number; score: number } }
  | { GamesAdded: { games: number; game_limit: number; games_left: number } };

export type RunAction =
  | 'Redeal'
  | 'Reroll'
  | 'LeaveShop'
  | 'GiveUpPub'
  | 'PickUpSkat'
  | { Declare: Declaration }
  | { AnswerKontra: { re: boolean } }
  | { Discard: [number, number] }
  | { DiscardThree: [number, number, number] }
  | { Play: number }
  | { UseConsumable: { slot: number; target: number | null; suit?: Suit | null } }
  | { BuyRelic: number }
  | { BuyConsumable: number }
  | { SellRelic: number }
  | { SellConsumable: number }
  | { TakeFromSkat: { give: number; take: number | null } }
  | 'SettleGame'
  | { ChooseEncounter: { option: number; target: number | null } }
  | { ChooseRoute: number }
  | { ChooseMalus: { option: number; malus: number } }
  | { ChooseSchnappsReward: { give_up: number | null } }
  | { ChooseBossRelic: { offer: number | null; replace: number | null } }
  /** Zinken (DESIGN 8.6). */
  | { BuyCardOffer: number }
  | 'RerollCardOffers'
  | { HirePainter: number }
  | { ChoosePaint: number }
  | { CleanseCard: number }
  /** "Untot": the three cards of the hand of which one goes into the trick. */
  | { ChooseRiseCards: [number, number, number] };

export interface Step {
  events: RunEvent[];
  view: RunView;
}

export interface CatalogEntry {
  name: string;
  text: string;
}

/** The number a card modification prints on its card (from the content, copied by skat_wasm). */
export interface ModFigure {
  lever: Lever;
  change: Change;
}

/** A card modification in the catalog, with rarity, curse flag and its printed number. */
/** Effekte v2: Rand, Moment, Gutschein, Fluch, Spezialkarte. */
export type ModKind = 'Edge' | 'Moment' | 'Voucher' | 'Curse' | 'Special' | 'SpecialCurse';

export interface CardModEntry extends CatalogEntry {
  rarity: Rarity;
  curse: boolean;
  kind?: ModKind;
  /** Values on offer; empty: no value. */
  values?: number[];
  max?: number | null;
  /** The printed number per unit of the card's value. */
  figure?: ModFigure | null;
}

/** A relic in the catalog, with the run counters it reads or raises. */
export interface RelicEntry extends CatalogEntry {
  counters?: string[];
  /** Absent in an older core. */
  kind?: RelicKind;
  rarity?: Rarity;
  group?: RelicGroup;
}

/** A consumable in the catalog, with rarity and group (absent in an older core). */
export interface ConsumableEntry extends CatalogEntry {
  rarity?: Rarity;
  category?: ConsumableCategory;
  /** The card modification it puts on a card, if the content names one. */
  card_mod?: string | null;
  /** It puts a modification drawn at random on a card (no content since the Zinken, DESIGN 8.6). */
  random_card_mod?: boolean;
}

export interface Catalog {
  relics: Record<string, RelicEntry>;
  card_mods: Record<string, CardModEntry>;
  bosses: Record<string, CatalogEntry>;
  consumables: Record<string, ConsumableEntry>;
  characters: Record<string, CatalogEntry>;
  encounters: Record<string, CatalogEntry>;
  unlocks: Record<string, CatalogEntry>;
  maluses: Record<string, CatalogEntry>;
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
  /** Highest stake unlocked per run length (Einsatz-Leiter); absent in an older core. */
  max_stakes?: Partial<Record<RunLength, number>>;
  /** Card back of the character's edition; absent in an older core (Distel). */
  back?: string;
}

export type Reward = { Relic: string } | { Character: Character } | { CardMod: string };

export type RewardKind = 'HouseRule' | 'Regular' | 'CardMod' | 'Character';

/** A reward with kind, name, text and rarity, from the core. */
export interface RewardInfo {
  reward: Reward;
  kind: RewardKind;
  name: string;
  text: string;
  /** None for a character. */
  rarity: Rarity | null;
}

export interface UnlockEntry {
  id: string;
  name: string;
  text: string;
  unlocked: boolean;
  rewards: Reward[];
  /** The rewards with kind, name and rarity, in the same order. */
  reward_info: RewardInfo[];
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
  /** Per run length; runs finished before the short evening are not in here. */
  lengths?: Partial<Record<RunLength, CharacterStats>>;
}

export interface MetaView {
  characters: CharacterEntry[];
  unlocks: UnlockEntry[];
  locked_relics: string[];
  /** Card modifications still locked (not drawn at random until unlocked). */
  locked_card_mods: string[];
  stats: MetaStats;
}

/** Result of feeding events to the meta state. */
export interface MetaStep {
  earned: string[];
  /** Special cards (Untot, Wandelkarte) received for the first time in this save: explain each once. */
  first_specials?: string[];
  view: MetaView;
}
