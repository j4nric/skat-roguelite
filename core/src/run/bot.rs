//! A simple run policy for the simulator and for tests.
//!
//! Games: the declarer heuristic of M1 for the card play and the suit game
//! or grand, plus run decisions on top. Every character may declare every
//! game (ADR 0012), so the bot chooses between suit game, grand and Null
//! for both:
//!
//! - **Hand or skat** (Skatordnung, ADR 0011): with ten cards the bot
//!   declares a hand game only for a very strong hand: Null Ouvert Hand
//!   without an exposed card (classic Null rule, see
//!   [`crate::ai::null_exposure`]), a suit game or grand with eight trumps
//!   or an announcement below. Otherwise it picks up the skat and chooses
//!   with twelve cards (`RunBot::after_skat`): the suit game or grand of
//!   [`DeclarerHeuristic::choose_game_after_skat`] or Null (the two most
//!   exposed cards discarded, Null Ouvert if none is left), whichever has
//!   the better expected score ([`suit_expected_score`],
//!   [`null_expected_score`], win rates measured with the PIMC player). Null
//!   is left out where a won Null would count nothing (no boss does that
//!   since "Der Schwarze Peter" pays half, but a rule could).
//!   A hand game redeems a random voucher of the hand (04.11.2026, DESIGN
//!   7.2); the bot does not count it in its hand rule: a few Groschen
//!   weigh little against the risk of the hand game, to be measured once
//!   the simulations run again.
//! - **Einmischen** ([`RunBot::is_weak`]): a suit game needs six trumps,
//!   or five trumps and two aces; a grand needs two jacks and two aces;
//!   Null at most [`NULL_MAX_EXPOSED`] exposed cards. A hand weak for both
//!   is thrown in, cheapest first: an Einmischen left in this game (or from
//!   the series' extras), the consumable "Neues Blatt", and with none left
//!   "Ein Bier" for one more. With nothing left it is played. The balancing
//!   measurement of 01.11.2026 showed that the weaker hands lose points on
//!   average (`docs/balance/2026-11-01-m2-zielwerte.md`). "Reue" is never
//!   used. Since 05.11.2026 a hand that is not weak is thrown in too while
//!   the game has an Einmischen of its own left and its
//!   [`ten_card_score`] is below [`RunBot::redeal_threshold`].
//! - **Rescue Schnäpse** (`RunBot::rescue_schnapps`, before Skat oder
//!   Hand): "Klarer" with the account below 0, "Absacker" before the last
//!   game short of the target, "Eierlikör" whenever it is accepted. The
//!   bot never gives a pub up ("Kneipe verlassen"): a lost pub ends the
//!   run either way (ADR 0019), and played a miracle may still help.
//! - **Forced hand** ("Der Schieber", `RunBot::forced_hand`): Null Hand
//!   only with at most [`NULL_HAND_MAX_EXPOSED`] exposed cards, else the
//!   suit game or grand with Hand; the better expected score if both fit.
//! - **Always open** ("Der Angeber"): an open suit game or grand only with
//!   an Ouvert hand, otherwise Null Ouvert (Hand without exposed card).
//! - **Announcements** ([`RunBot::announcement`]): Schneider angesagt or
//!   Ouvert only for hands that make it almost surely (conservative rules,
//!   measured against the defender heuristic, see the function), from the
//!   hand and, with the same rules for the ten cards kept, after the skat
//!   (ADR 0013). Re on Kontra only with a very strong hand
//!   ([`RunBot::says_re`]).
//! - **Relics that pay for announcements** (relics v4, 07.11.2026): with
//!   "Große Klappe" or "Offenes Visier" the bot announces Schneider or
//!   plays Ouvert after the skat when double-dummy deals of the unseen
//!   cards say Schneider is near certain ([`schneider_percent`],
//!   `RunBot::bold_announcement`). With "Kahlschlag" it discards for void
//!   side suits (`RunBot::void_discard`). Without these relics nothing
//!   changes.
//!
//! Tresen ([`RunBot::shop_action`]): one action per call, in this order:
//! take the Kartenmaler's best modification while he waits, "Karte
//! reinigen" on a cursed card, sell the weakest relic for a clearly better
//! one when all slots are full, buy the most valuable relic, buy the most
//! valuable consumable, buy the best offer of the Auslage on a card that
//! carries nothing else (never replacing), hire the Kartenmaler for an ace
//! without modification when Groschen are plenty, draw a new offer when
//! nothing sensible can be bought and Groschen are plenty, leave. Values
//! come from [`relic_value`], [`consumable_value`] and [`zinken_value`].
//! The bot never pays on credit for Zinken. A trade offer ("Hehlerware",
//! "Schnäppchen") counts [`TRADE_PENALTY`] less at its half price; with
//! "Pechvogel" it counts in full and the bot keeps its curses.
//!
//! Encounters ([`RunBot::encounter_action`], since 03.11.2026 one option
//! must be taken): the option with the best value ([`RunBot::option_value`],
//! simple rules in rough Groschen, not tuned), among those that can be
//! taken. A swap only for a relic worth at least [`SWAP_MARGIN`] more than
//! the weakest owned one; double stakes are never sought. At a Kiosk the
//! most valuable Schnaps, if any.
//!
//! Kneipentour (ADR 0016, simple rules, not tuned, not measured): at a
//! Rast the cursed card cleansed, else one Einmischen more in the next pub
//! (ADR 0019), at the Werkstatt the cursed card cleansed (else the free
//! Kartenmaler on an ace, his best modification taken), from a Spelunke's loot the relic into a free
//! slot (else the Groschen). The route ([`RunBot::route_action`]) scores
//! each way by the category it shows: the Tresen when rich, an Upgrade
//! with a cursed card, a Rast a little; a Spelunke only when strong
//! (since ADR 0019 a lost Spelunke ends the run: since 05.11.2026 five
//! relics, or four and the "Schutzengel"; before four, or three). After a won boss pub (ADR 0017)
//! the most valuable relic of the pick into a free slot (slots are
//! unlimited since 03.11.2026), with a cap and every slot full in place of
//! the weakest owned one if it is worth at least [`SWAP_MARGIN`] more, else
//! none ([`RunBot::boss_reward_action`]).
//!
//! The Schnaps of a won pub with every slot full (DESIGN 8.9): taken for
//! the weakest owned Schnaps if it is worth more ([`consumable_value`]),
//! else sold ([`RunBot::schnapps_reward_action`]).
//!
//! Schnäpse (simple rules, not tuned): "Kurzer" right before picking up the
//! skat (the bot does not use what it sees); after the skat, "Blue Curaçao"
//! paints the cheapest side card (never a jack) into the trump suit of a
//! suit game, then "Doppelkorn" before a game that is not weak
//! ([`RunBot::is_weak`]) and "Magenbitter" before a weak one, never before
//! a Null. "Kräuter" counts as "Ein Bier". "Fusel" before Skat oder Hand
//! with a hand that is not weak, when no other Schnaps waits for the game.
//!
//! Figures "Die Oma" and "Der Opa": in a hand game the bot takes the card
//! the run offers from the skat whenever it may (never in a Null), giving
//! away its cheapest side card. It never discards a card that must not be discarded
//! ("Ramponiert") while it has others.
//!
//! "Drei drücken": with thirteen cards the bot chooses its game as with
//! twelve and discards three: the two it would discard, then the one it
//! would discard next (the run gives the lowest of them back).
//!
//! The bot never looks at the Zielwert or the series account. The
//! simulator's success curve (`--curve`) relies on that.
//!
//! It lives in the core (not in `sim/`) so that core tests can drive whole
//! runs with it; a frontend could use it for an "auto play" button.

use super::{EncounterOption, Run, RunAction, RunPhase};
use crate::ai::{DeclarerHeuristic, Policy, null_unsafe_cards, trump_count};
use crate::cards::{Card, CardKind, PlayingCard, Rank, Suit};
use crate::content::{
    BetOn, BuildTag, CardModDef, CardModId, CardPick, ConsumableAction, ConsumableId, Content,
    Effect, EncounterEffect, NextGameEffect, RelicId, StopCategory,
};
use crate::deck::DealtCard;
use crate::game::{Phase, Seat};
use crate::rng::mix64;
use crate::rules::{Declaration, GameType, NullRisk, NullScoring, RiskLevel};

/// How much the bot wants each relic, by the measured effect of the relic
/// alone (`docs/balance/2026-11-01-m5-v1.md`: PIMC card play, relic from the
/// start, no Tresen, 3 × 500 runs; change of "section 3 reached" in
/// percentage points for the Alleinspieler unless noted). The bot buys the
/// highest value it can afford and swaps a relic for one that is worth at
/// least [`SWAP_MARGIN`] more; value 0 is never bought. One table serves
/// both characters. Relics not listed are worth [`DEFAULT_RELIC_VALUE`].
/// `every_bit_counts`, `old_lady` and `fat_sow` were measured while they
/// changed every card; since 04.11.2026 they change only the player's own
/// cards, so their values are too high until measured again (`balance`).
///
/// | Relic | Value | Measured |
/// |---|---|---|
/// | `every_bit_counts` | 10 | +8.3 section 3 (Nullspieler +0.8) |
/// | `bock_round` | 10 | +7.1 (Nullspieler +7.7) |
/// | `old_lady` | 10 | +7.1 |
/// | `regular_guest` | 8 | +5.6, section 4 +6.7: the late multiplier |
/// | `concede` | 8 | +5.1 |
/// | `blind_eye` | 8 | +4.9 |
/// | `seven_sleeper` | 8 | Null: +5.5 |
/// | `skat_redraw` | 7 | +3.5 (Nullspieler +4.7) |
/// | `march_through` | 7 | +4.5 |
/// | `safety_net` | 7 | Null: +4.3 |
/// | `lucky_streak` | 6 | +4.0, section 4 +4.0 (Nullspieler +0.1) |
/// | `tuition` | 6 | +2.3 (Nullspieler +4.2) |
/// | `virgin` | 6 | +3.2 |
/// | `tour` | 6 | +3.1, grows over the run |
/// | `tailor` | 6 | +3.1 |
/// | `landlord` | 5 | cheaper Tresen, measured in runs with the Tresen |
/// | `null_eight_fifteen` | 5 | Null: +2.9 |
/// | `tally_stick` | 5 | Null: +2.7 |
/// | `fat_sow` | 4 | Null: +1.7 |
/// | `kontra` | 3 | +1.6, variance: helps late, hurts the Nullspieler (-0.9) |
/// | `kibitzer` | 3 | +1.3, PIMC samples use the peeked card |
/// | `high_and_dry` | 3 | +1.2 |
/// | `schneider_bock` | 3 | +1.1 |
/// | `granny` | 2 | +0.5, the bot rarely plays hand |
/// | `grandpa` | 2 | +0.3, the bot rarely plays hand |
/// | `last_trick_seven` | 2 | +0.4 |
/// | `with_four` | 2 | +0.1 |
/// | `blind_flight` | 2 | Null: +0.3, the bot rarely plays Null Hand |
/// | `referee` | 1 | +0.2, only against two bosses |
/// | `beer_mat` | 1 | +0.4, the bot rarely plays hand |
/// | `ouvert_bonus` | 1 | 0, the bot almost never plays Ouvert |
/// | `glass_house` | 1 | Null: +0.3, the bot rarely plays Null Ouvert |
/// | `clean_slate` | 1 | Null: +0.1, the bot almost never avoids following suit |
/// | `card_counter` | 1 | information the PIMC bot already has |
/// | `show_off` | 0 | Null: -15.9, forced Ouvert ruins the bot: never bought |
///
/// Wave 2a (`docs/balance/2026-11-02-relikte-welle2.md`, same method with
/// 3-game series, 3 × 500 runs). The baseline reaches section 3 less often
/// now (11.7 % instead of 14.5 %; "Alte Dame" +5.5 instead of +7.1), so the
/// effects are scaled by about 1.3 to compare with the table above:
///
/// | Relic | Value | Measured (section 3, section 2) |
/// |---|---|---|
/// | `all_rounder` | 7 | +3.7, +6.6 |
/// | `old_deck` | 7 | +3.4, +5.1 |
/// | `watertight` | 6 | +3.0, +3.3 |
/// | `skat_keeper` | 6 | +2.5, +2.6; the bot discards few Augen |
/// | `jack_parade` | 4 | +1.8, +2.9 |
/// | `blank_king` | 4 | +1.3, +1.9 |
/// | `changeling` | 2 | +0.5, +2.4; -1.4 points per game (levels on lost Null games) |
/// | `home_suit` | 1 | +0.1: the bot almost never plays three suit games in a row with one trump suit |
///
/// Wave 2b (same report, same method):
///
/// | Relic | Value | Measured (section 3, section 2) |
/// |---|---|---|
/// | `three_discard` | 10 | once per series +7.9, +12.1 (new targets of PR #47); in every game it was +18.0 |
/// | `decoy` | 3 | +1.1, +2.2 |
/// | `crib_sheet` | 1 | +0.3, +0.2: the bot rarely plays hand; both skat cards since 02.11.2026, not measured |
///
/// Item review of 02.11.2026: the relics removed then (`bock_round`,
/// `kontra`, `regular_guest`, `seven_sleeper` and the rest of the tables
/// above that have no entry below, and `decoy`) stay in the
/// tables as history only. The Kneipentour relics added then are set by
/// judgement, not measured (simulations paused):
///
/// | Relic | Value | Why |
/// |---|---|---|
/// | `closing_time` | 3 | +4 Groschen per pub won in the first game, about one pub in four |
/// | `season_ticket` | 2 | +1 Groschen per Bahn; the route policy does not seek the Bahn |
/// | `kitty` | 3 | +1 Groschen per pub won, paid at the next Tresen |
/// | `slate` | 2 | the bot never buys on credit, it only pays the debt back |
/// | `guardian_angel` | 7 | since ADR 0019 one lost pub played again, a boss pub included; not re-measured |
/// | `dive_expert` | 1 | the bot rarely enters a Spelunke |
///
/// Curses as trade (03.11.2026), by judgement, not measured:
///
/// | Relic | Value | Why |
/// |---|---|---|
/// | `jinx` | 4 | +1 Grundwert per cursed card: +9 for "Bruderherz" from the start, +4 for the Nullspieler, little for the Alleinspieler |
///
/// New on 04.11.2026, by judgement, not measured:
///
/// | Relic | Value | Why |
/// |---|---|---|
/// | `glass_eye` | 3 | like `kibitzer`: two defender cards known for the whole game, the PIMC samples pin them |
///
/// Faktor (ADR 0018, 04.11.2026), provisional by judgement until the
/// balance round measures them:
///
/// | Relic | Value | Why |
/// |---|---|---|
/// | `jay`, `tree_frog`, `robin`, `bell_tree` | 3 | ×1.5 on won suit games of one trump suit, about a quarter of them |
/// | `black_zero` | 2 | ×1.5 on won Null games; the Alleinspieler seldom plays Null |
/// | `tap` | 6 | +×0.1 per pub won, on every won game: ×1.5 to ×2 late in the run |
/// | `hot_hand` | 5 | +×0.15 per won game in a row, back to ×1 after a loss |
///
/// Build directions (04.11.2026, Cockpit q26) and "Großvaters Grand"
/// (Cockpit q27), provisional by judgement, not measured:
///
/// | Relic | Value | Why |
/// |---|---|---|
/// | `club_colours` | 3 | +1 Grundwert per Zinke on the trump suit; the bot buys few Zinken on one suit |
/// | `true_soul` | 1 | like `home_suit`: the bot seldom plays one trump suit in a row |
/// | `rebate_book` | 2 | +1 level per redeemed Marke for the pub; the bot redeems few (more as Drückeberger) |
/// | `empties` | 2 | +2 Groschen per redeemed Marke |
/// | `stamp_album` | 2 | +1 Grundwert per Marke kept on a card |
/// | `fingertips` | 4 | +3 Augen per trick taken with a Zinke, also for Schneider |
/// | `tally_sheet` | 3 | +1 or +2 levels with three or five such tricks |
/// | `tip` | 3 | up to 3 Groschen per game |
/// | `first_names` | 2 | "Neu auslosen" for free (05.11.2026, not measured) |
/// | `golden_shears` | 5 | ×1.5 on a Schneider win, about four won games in ten |
/// | `tape_measure` | 5 | +1 Grundwert per Schneider win for the rest of the run |
/// | `sewing_pattern` | 2 | +1 Grundwert per Aufdruck or Schneiderkreide on a Schneider win |
/// | `lead_pouring` | 2 | +1 level per Blei card in the hand of a won Null |
/// | `plumb_line` | 2 | +2 Grundwert per Blei card in every Null |
/// | `grandfathers_grand` | 3 | +6 Grundwert on every Grand |
///
/// Relics v4 (07.11.2026, decided by the user), provisional by judgement,
/// not measured; the four reworked relics are listed again with their new
/// value:
///
/// | Relic | Value | Why |
/// |---|---|---|
/// | `true_soul` | 4 | rework: +1 Grundwert per trump of the suit in the hand, every suit game |
/// | `crib_sheet` | 1 | rework: +1 level in a hand game; the bot rarely plays hand |
/// | `beer_mat` | 4 | rework: +2 / +4 Grundwert on the second and third game of a pub, 2 Groschen per pub |
/// | `dive_expert` | 3 | rework: +2 levels on won games under a malus (Spelunke, the night) |
/// | `fan_block` | 2 | +2 levels with only trump-suit Siegel; the bot does not gather Siegel on one suit |
/// | `clear_cut` | 3 | +1 level per void side suit after the skat, at most +2 |
/// | `pump_attendant` | 2 | a Pfandmarke per pub won |
/// | `rebate_king` | 3 | ×1.5 from the third Marke of a pub; mostly the Drückeberger ([`relic_score`]: build) |
/// | `double_stamp` | 1 | nothing alone; worth 5 with a Marken build ([`relic_score`]) |
/// | `seven_at_one_blow` | 5 | ×1.5 on won games with seven tricks, about a third of the wins |
/// | `upstart` | 3 | Winkel cards count the Augen of their new rank; Winkel is the bot's favourite Zinke |
/// | `trick_after_trick` | 3 | up to +2 levels for runs of three tricks |
/// | `big_mouth` | 1 | the bot rarely announces Schneider |
/// | `yardstick` | 3 | +1 Grundwert per 5 Augen above 90 on a Schneider win |
/// | `plumber` | 2 | Blei on a card per won Null |
/// | `loose_contact` | 2 | Kurzschluss from 50 Augen; Null only |
/// | `cards_on_table` | 1 | the bot rarely plays Null Ouvert |
/// | `big_shots` | 3 | Daus as Spitzen in a Grand |
/// | `safe_haven` | 5 | keeps the Unter at an Einmischen; the bot redeals often |
/// | `beer_calm` | 2 | a lost Grand counts half |
/// | `open_visor` | 1 | the bot never announces Ouvert in suit games or Grand |
/// | `empty_chair` | 2 | 5 while it leaves two slots free ([`relic_score`]) |
/// | `junk_dealer` | 2 | the bot sells or swaps about once per run |
/// | `teetotaller` | 2 | the bot skips weak card rewards while holding it ([`RunBot::card_reward_action`]) |
pub const RELIC_VALUES: [(&str, u32); 74] = [
    ("every_bit_counts", 10),
    ("old_lady", 10),
    ("concede", 8),
    ("blind_eye", 8),
    ("skat_redraw", 7),
    ("lucky_streak", 6),
    ("virgin", 6),
    ("tour", 6),
    ("tailor", 6),
    ("landlord", 5),
    ("null_eight_fifteen", 5),
    ("fat_sow", 4),
    ("kibitzer", 3),
    ("last_trick_seven", 2),
    ("with_four", 2),
    ("beer_mat", 4),
    ("card_counter", 1),
    ("all_rounder", 7),
    ("watertight", 6),
    ("skat_keeper", 6),
    ("blank_king", 4),
    ("home_suit", 1),
    ("three_discard", 10),
    ("crib_sheet", 1),
    ("closing_time", 3),
    ("season_ticket", 2),
    ("kitty", 3),
    ("slate", 2),
    ("guardian_angel", 7),
    ("dive_expert", 3),
    ("jinx", 4),
    ("glass_eye", 3),
    ("jay", 3),
    ("tree_frog", 3),
    ("robin", 3),
    ("bell_tree", 3),
    ("black_zero", 2),
    ("tap", 6),
    ("hot_hand", 5),
    ("club_colours", 3),
    ("true_soul", 4),
    ("rebate_book", 2),
    ("empties", 2),
    ("stamp_album", 2),
    ("fingertips", 4),
    ("tally_sheet", 3),
    ("tip", 3),
    ("golden_shears", 5),
    ("tape_measure", 5),
    ("sewing_pattern", 2),
    ("lead_pouring", 2),
    ("plumb_line", 2),
    ("grandfathers_grand", 3),
    ("first_names", 2),
    ("fan_block", 2),
    ("clear_cut", 3),
    ("pump_attendant", 2),
    ("rebate_king", 3),
    ("double_stamp", 1),
    ("seven_at_one_blow", 5),
    ("upstart", 3),
    ("trick_after_trick", 3),
    ("big_mouth", 1),
    ("yardstick", 3),
    ("plumber", 2),
    ("loose_contact", 2),
    ("cards_on_table", 1),
    ("big_shots", 3),
    ("safe_haven", 5),
    ("beer_calm", 2),
    ("open_visor", 1),
    ("empty_chair", 2),
    ("junk_dealer", 2),
    ("teetotaller", 2),
];

/// Bot values of the legendary relics (M8 wave 3,
/// `docs/balance/2026-11-02-welle3.md`, same method as wave 2a), kept apart
/// from [`RELIC_VALUES`] so that the tiers stay readable.
///
/// | Relic | Value | Measured (section 3, section 2) |
/// |---|---|---|
/// | `skat_god` | 9 | +4.7, +3.6; +3.7 on section 4, levels on won games only |
/// | `honorary_chair` | 1 | -0.4, -0.6: leading every first trick does not help the bot |
/// | `grand_seigneur` | 4 | not measured (ADR 0018, provisional): ×2 on won Grands |
pub const LEGENDARY_VALUES: [(&str, u32); 3] = [
    ("skat_god", 9),
    ("honorary_chair", 1),
    ("grand_seigneur", 4),
];

/// Bot values of the relics of the long night ("Nachtschwärmer", ADR 0020),
/// provisional, not measured: the growing and counting Faktor relics high,
/// the conditional ×2 and ×1.5 like the other rare Faktor relics.
///
/// | Relic | Value | Why |
/// |---|---|---|
/// | `night_owl` | 6 | +×0.15 per Verschärfung, grows with the night |
/// | `sweep_out` | 3 | ×2 on a won third game; the bot often finishes earlier |
/// | `rag_collector` | 6 | +×0.05 per card with a Zinke: ×1.5 with ten |
/// | `moonlight` | 6 | like the Zapfhahn with +×0.2 |
/// | `night_shift` | 5 | +×0.1 per Zinke of the bot's direction |
/// | `dawn` | 4 | ×1.5 on a won first game, the bot's usual win |
pub const NIGHT_VALUES: [(&str, u32); 6] = [
    ("night_owl", 6),
    ("sweep_out", 3),
    ("rag_collector", 6),
    ("moonlight", 6),
    ("night_shift", 5),
    ("dawn", 4),
];

/// Value of a relic that is not in [`RELIC_VALUES`].
pub const DEFAULT_RELIC_VALUE: u32 = 3;

/// With "Der Abstinenzler" the bot takes a card reward only if it is worth
/// more than this ([`zinke_score`] hundredths): a Zinke of its build
/// direction, never a plain one (provisional, relics v4).
pub const TEETOTALLER_SKIP_WORTH: i64 = 300;

/// A relic is swapped only for one worth at least this much more.
pub const SWAP_MARGIN: u32 = 3;

/// How much the bot wants each consumable. Not listed: 0 (never bought).
///
/// | Consumable | Value | Use |
/// |---|---|---|
/// | `herbal` | 1 | "Kräuter" (since 02.11.2026 what "Neues Blatt" did): throws a weak hand in once the game's Einmischen are used up |
/// | `double_grain` | 1 | "Doppelkorn" (since 02.11.2026 it deals again): throws a weak hand in after "Kräuter", the new game counts double |
/// | `blue_curacao` | 2 | "Blue Curaçao": the cheapest side card into trump |
/// | `shot` | 1 | "Kurzer": before picking up the skat; the bot does not use what it sees |
/// | `road_beer` | 2 | "Wegbier": at the route choice, at once |
/// | `clear_spirit` | 1 | "Klarer": between two games when the account is below 0 |
/// | `nightcap` | 2 | "Absacker": before the last game of a normal pub when the target is not reached |
/// | `eggnog` | 1 | "Eierlikör": before Skat oder Hand whenever the account is below half the target |
/// | `rotgut` | 1 | "Fusel": before Skat oder Hand with a hand that is not weak; the curse costs a card |
/// | `snifter` | 0 | "Schwenker": never bought; from "Der Wirt spendiert" used at the route choice when no way has what the route policy wants most (an Upgrade or a Rast with a cursed card) |
///
/// The Schnäpse are placeholders, not measured. "Konterbier"
/// (`counter_beer`, a life) went with the lives (ADR 0019). Removed on 02.11.2026:
/// `new_deal`, `beer`, `bitters`, `regret`, `points_5`, `trump_4`, and with
/// the Zinken at the Tresen (DESIGN 8.6) `fresh_deck` (now "Karte
/// reinigen"), `ink_pad`, `points_3`, `trump_2`, `rank_1` (now the Auslage
/// and the Kartenmaler, [`ZINKEN_VALUES`]). "Neues
/// Blatt", "Ein Bier" and "Kräuter" only help once the two Einmischen of a game are used up:
/// the bot throws in about every second deal (0.75 Einmischen per game in
/// the run measurement of 01.11.2026, DESIGN 5.2), so three weak deals in
/// a row come in about one game of eight, once every three series. Before the
/// Einmischen rule they were worth 4, 3 and 3 and were bought far more
/// often than used. Kräuter and Doppelkorn 2 → 1 on 03.11.2026: about
/// half of them were never drunk
/// (`docs/balance/2026-11-03-kleine-runde.md`, 1387 / 772 and 695 / 349).
pub const CONSUMABLE_VALUES: [(&str, u32); 10] = [
    ("herbal", 1),
    ("double_grain", 1),
    ("blue_curacao", 2),
    ("shot", 1),
    ("road_beer", 2),
    ("snifter", 0),
    ("clear_spirit", 1),
    ("nightcap", 2),
    ("eggnog", 1),
    ("rotgut", 1),
];

/// "Neu auslosen" needs at least this many Groschen left over after
/// paying for it, enough for a cheap relic from the new offer. Once per
/// Tresen visit (the rule since 05.11.2026).
pub const REROLL_RESERVE: u32 = 6;

/// How much the bot wants each Zinke (Zinken at the Tresen, DESIGN 8.6;
/// simple rules, not measured). Not listed: 0 (never bought). The
/// Kartenmaler's choice takes the most valuable, the first on a tie.
///
/// | Zinke | Value | Why |
/// |---|---|---|
/// | `trump_bonus` (Siegel) | 3 | Grundwert, on an ace often in the trump suit |
/// | `rank` (Winkel) | 3 | the card becomes the next higher one, wins tricks |
/// | `points` (Aufdruck) | 2 | Augen |
/// | `first_trick` (Eselsohr) | 2 | an ace often wins the first trick |
/// | `last_trick` (Kerbe) | 1 | the last trick is rarely the ace's |
/// | `voucher_coins` (Pfandmarke) | 1 | Groschen, if discarded |
/// | `undead` (Untot, rare in the Auslage) | 2 | a strong card comes back once per game |
/// | `wild_card` (Wandelkarte, rare in the Auslage) | 1 | helps to follow suit, hard to judge |
/// | `gilt_edge` (Goldkante) | 2 | a Groschen per trick the card takes |
/// | `crosshair` (Fadenkreuz) | 1 | only when it beats a defender's Daus |
/// | `tailors_chalk` (Schneiderkreide) | 1 | only in a won Schneider game |
/// | `lead` (Blei) | 1 | only in Null, which the bot rarely plays |
/// | `doppelganger` (Doppelgänger, rare in the Auslage) | 1 | one mirror per game, only as the last card |
pub const ZINKEN_VALUES: [(&str, u32); 13] = [
    ("trump_bonus", 3),
    ("rank", 3),
    ("points", 2),
    ("first_trick", 2),
    ("last_trick", 1),
    ("voucher_coins", 1),
    ("undead", 2),
    ("wild_card", 1),
    ("gilt_edge", 2),
    ("crosshair", 1),
    ("tailors_chalk", 1),
    ("lead", 1),
    ("doppelganger", 1),
];

/// A trade offer ("Hehlerware", "Schnäppchen", curses as trade) is worth
/// this much less to the bot than the same offer without the curse, unless
/// it owns a relic that likes curses ("Pechvogel"). Not measured.
pub const TRADE_PENALTY: u32 = 1;

/// The Kartenmaler needs at least this many Groschen left over after
/// paying for him (not measured).
pub const PAINTER_RESERVE: u32 = 4;

/// Groschen from which the route policy heads for the Tresen (Kneipentour,
/// not measured).
pub const RICH: u32 = 8;

/// The bot's value of a relic.
#[must_use]
pub fn relic_value(id: &RelicId) -> u32 {
    RELIC_VALUES
        .iter()
        .chain(LEGENDARY_VALUES.iter())
        .chain(NIGHT_VALUES.iter())
        .find(|(name, _)| *name == id.as_str())
        .map_or(DEFAULT_RELIC_VALUE, |&(_, v)| v)
}

/// The bot's value of a Zinke ([`ZINKEN_VALUES`]).
#[must_use]
pub fn zinken_value(id: &CardModId) -> u32 {
    ZINKEN_VALUES
        .iter()
        .find(|(name, _)| *name == id.as_str())
        .map_or(0, |&(_, v)| v)
}

/// The bot's value of a consumable.
#[must_use]
pub fn consumable_value(id: &ConsumableId) -> u32 {
    CONSUMABLE_VALUES
        .iter()
        .find(|(name, _)| *name == id.as_str())
        .map_or(0, |&(_, v)| v)
}

/// A Zinke that shares a build direction ([`BuildTag`]) with a relic the
/// bot holds is worth this many percent more to it ([`zinke_score`]): in
/// the card reward, the Auslage and the Kartenmaler. Measurement tool
/// (05.11.2026, provisional): without it the bot took Rang +1 in 47 % of
/// its card rewards and the build directions hardly formed in the
/// simulator (`docs/balance/2026-11-05-runde3.md`). + 200 % (three times
/// the value) lets a direction Zinke worth 1 (Blei, Schneiderkreide,
/// Pfandmarke) tie with the best plain ones (3) and win the tie; + 50 %
/// would never change a pick against Rang +1.
pub const ZINKE_DIRECTION_BONUS_PERCENT: u32 = 200;

/// A relic offer that shares a build direction with a relic the bot holds
/// or a Zinke in its deck is worth this much more ([`relic_score`], in
/// hundredths of a [`relic_value`] point): one point, enough to break ties
/// and close gaps of one (provisional, 05.11.2026).
pub const RELIC_DIRECTION_BONUS: u32 = 100;

/// The build directions of the relics the run holds.
fn relic_tags(run: &Run, content: &Content) -> Vec<BuildTag> {
    let mut tags: Vec<BuildTag> = run
        .relics()
        .iter()
        .filter_map(|id| content.relic(id))
        .flat_map(|def| def.tags.iter().copied())
        .collect();
    tags.sort_unstable();
    tags.dedup();
    tags
}

/// The build directions of the relics held and of the Zinken (no curses)
/// in the deck.
fn build_tags(run: &Run, content: &Content) -> Vec<BuildTag> {
    let mut tags = relic_tags(run, content);
    tags.extend(
        run.edition()
            .cards()
            .iter()
            .flat_map(|c| c.mods.iter())
            .filter_map(|m| content.card_mod(m))
            .filter(|def| !def.is_curse())
            .flat_map(|def| def.tags.iter().copied()),
    );
    tags.sort_unstable();
    tags.dedup();
    tags
}

/// The bot's score of a Zinke for this run, in hundredths of a
/// [`zinken_value`] point: its value, + [`ZINKE_DIRECTION_BONUS_PERCENT`]
/// if it shares a build direction with a relic the bot holds.
#[must_use]
pub fn zinke_score(run: &Run, content: &Content, id: &CardModId) -> u32 {
    let base = zinken_value(id) * 100;
    let tags = relic_tags(run, content);
    let matches = content
        .card_mod(id)
        .is_some_and(|def| def.tags.iter().any(|t| tags.contains(t)));
    if matches && base > 0 {
        // + 1 wins a tie for the direction.
        base * (100 + ZINKE_DIRECTION_BONUS_PERCENT) / 100 + 1
    } else {
        base
    }
}

/// The bot's score of a relic offer for this run, in hundredths of a
/// [`relic_value`] point: its value, + [`RELIC_DIRECTION_BONUS`] if it
/// shares a build direction with a relic held or a Zinke in the deck. A
/// relic the bot does not want (value 0) stays 0.
#[must_use]
pub fn relic_score(run: &Run, content: &Content, id: &RelicId) -> u32 {
    let base = relic_value(id) * 100;
    if base == 0 {
        return 0;
    }
    let tags = build_tags(run, content);
    // Relics whose worth hangs on the run (relics v4, by judgement).
    let base = match id.as_str() {
        // Doubles what the Marken do: worth it in a Marken build only.
        "double_stamp" if tags.contains(&BuildTag::Vouchers) => base + 400,
        "rebate_king" if tags.contains(&BuildTag::Vouchers) => base + 200,
        // +1 level while two slots are left free once it is bought.
        "empty_chair" => {
            let owned = run.relics().len().saturating_add(1);
            let free = content.relic_slots(run.character()).map_or(0, |slots| {
                usize::try_from(slots)
                    .unwrap_or(usize::MAX)
                    .saturating_sub(owned)
            });
            if free >= 2 { base + 300 } else { base }
        }
        _ => base,
    };
    let matches = content
        .relic(id)
        .is_some_and(|def| def.tags.iter().any(|t| tags.contains(t)));
    if matches {
        base + RELIC_DIRECTION_BONUS
    } else {
        base
    }
}

/// The Nullspieler plays a hand with at most this many exposed cards
/// (before the skat). Measured with the Null formula (DESIGN 9.2, 3 × 1000
/// runs): with 2 the bot reaches section 2 in 15 %, with 4 in 28 %, with 5
/// in 35 %. Hands with 3 or 4 exposed cards win 60 % and 45 % of their
/// games. M5 (`docs/balance/2026-11-01-m5-v1.md`, PIMC card play, 8 games
/// and 6 passes per series): section 3 in 20.1 % with 4, 25.1 % with 5 and
/// 24.8 % with 6 (6 plays a third more games per run). 5 is the stronger
/// policy, and the bot stands in for a good player.
pub const NULL_MAX_EXPOSED: usize = 5;

/// Under forced hand play ("Der Schieber") the bot declares Null Hand only
/// with at most this many exposed cards: it cannot discard them. Null after
/// the skat wins 74 % with one and 51 % with two exposed cards left, 28 %
/// with three ([`NULL_WIN_PERCENT`]).
pub const NULL_HAND_MAX_EXPOSED: usize = 2;

/// Percent of Null games the bot wins by exposed cards in the ten cards it
/// plays (index; 0 is played Ouvert). Measured with the PIMC player
/// (`--bot pimc:12:3`, 01.11.2026), Null after the skat: the Nullspieler
/// with eight sevens won 94, 74, 51, 28, 19 and 7 % for 0 to 5 (600 runs,
/// seed 99, 5300 games), the Alleinspieler with the normal deck 99, 79, 58,
/// 41, 33 and 21 % (300 runs, seed 42, 1100 games). The table is the mean.
/// More than 7: 0.
pub const NULL_WIN_PERCENT: [i64; 8] = [96, 76, 54, 34, 26, 14, 7, 3];

/// Percent of suit games the bot wins, by trumps (index, up to 8) and aces
/// (up to 3) of the ten cards it plays, in a deck with all eleven trumps
/// (the Alleinspieler, 600 runs, seed 99, 2516 suit games); hands with
/// fewer than five trumps are rare (the bot passes them) and rounded. A
/// deck with fewer trumps counts each missing one as held (the
/// Nullspieler's suits have eight: with four or five he won 92 %).
pub const SUIT_WIN_PERCENT: [[i64; 4]; 9] = [
    [0, 0, 5, 10],
    [0, 0, 5, 10],
    [0, 5, 10, 15],
    [0, 5, 15, 20],
    [5, 20, 44, 50],
    [5, 23, 52, 85],
    [55, 74, 81, 93],
    [94, 94, 97, 98],
    [98, 98, 98, 98],
];

/// Trumps of a suit game in the standard deck: four jacks and seven cards.
const SUIT_TRUMPS: usize = 11;

/// Percent of grands the bot wins by jacks (index) and aces (up to 3), same
/// measurement (1650 grands, almost all with two jacks and two aces or
/// more).
pub const GRAND_WIN_PERCENT: [[i64; 4]; 5] = [
    [0, 0, 2, 5],
    [0, 2, 5, 10],
    [5, 20, 67, 89],
    [20, 60, 93, 98],
    [90, 98, 98, 98],
];

/// Expected style bonus of a won Null in hundredths of a Gewinnstufe:
/// a ducked seven per seven held, plus this for "nie bedient" and
/// "Kurzschluss". Measured: a won Null with the skat counts on average
/// 23 × (1.3 + sevens) (Nullspieler, same measurement).
const NULL_STYLE_EXTRA: i64 = 30;

fn percent(table: &[i64], index: usize) -> i64 {
    table.get(index).copied().unwrap_or(0)
}

/// Expected score of a Null with these ten cards, in hundredths of a point
/// (won: the value; lost: minus the value times
/// [`crate::content::RUN_LOSS_FACTOR`], as in the series account), without relics. `exposed` counts the exposed cards
/// ([`null_unsafe_cards`]). The Null formula (DESIGN 9.2) counts the
/// expected style bonus, each seven held as one ducked trick.
#[must_use]
pub fn null_expected_score(
    hand: &[DealtCard],
    exposed: usize,
    declaration: Declaration,
    scoring: &NullScoring,
) -> i64 {
    let Some(risk) = declaration.null_risk() else {
        return 0;
    };
    let win = percent(&NULL_WIN_PERCENT, exposed);
    let (won, lost) = match scoring {
        NullScoring::Fixed => {
            let value = i64::from(crate::rules::null_value(risk.hand(), risk.ouvert())) * 100;
            (value, crate::content::RUN_LOSS_FACTOR * value)
        }
        NullScoring::Formula(f) => {
            let base = i64::from(f.base_value);
            let step = i64::from(risk.step()) * 100;
            let sevens = i64::try_from(count_rank(hand, Rank::Seven)).unwrap_or(0);
            let style = sevens * i64::from(f.per_seven_ducked) * 100 + NULL_STYLE_EXTRA;
            (
                base * (step + style),
                crate::content::RUN_LOSS_FACTOR * base * step,
            )
        }
    };
    (win * won - (100 - win) * lost) / 100
}

/// Expected score of a suit game or grand with these ten cards, in
/// hundredths of a point, without relics: the Spielwert with the Spitzen
/// of `all_cards` (the ten plus the skat, which counts for the Spitzen),
/// won with [`SUIT_WIN_PERCENT`] or [`GRAND_WIN_PERCENT`], lost
/// [`crate::content::RUN_LOSS_FACTOR`] times (once since 04.11.2026).
/// `deck` is every card of the game (for trumps missing from the deck).
/// `base` is the run's Grundwert of the game
/// ([`crate::content::RunConfig::base_value`], Grand 18); 0 gives 0.
///
/// `win_threshold` is the Augen the game needs (`61 + stake`, lowered by
/// relics; [`crate::rules::Thresholds::win`]). The tables are measured at
/// 61; above it the win chance shrinks with the Augen left over, `(120 −
/// threshold) / 59` of it (a rough model, not measured), so that at high
/// stakes Null, which counts no Augen, wins the comparison more often.
/// Below 61 the tables are kept.
#[must_use]
pub fn suit_expected_score(
    hand: &[DealtCard],
    all_cards: &[DealtCard],
    deck: &[DealtCard],
    declaration: Declaration,
    base: u32,
    win_threshold: u32,
) -> i64 {
    let game = declaration.game();
    if game == GameType::Null || base == 0 {
        return 0;
    }
    let aces = count_rank(hand, Rank::Ace).min(3);
    let win = match game {
        GameType::Grand => GRAND_WIN_PERCENT
            .get(count_rank(hand, Rank::Jack).min(4))
            .map_or(0, |row| percent(row, aces)),
        _ => {
            let missing = SUIT_TRUMPS.saturating_sub(trump_count(game, deck));
            SUIT_WIN_PERCENT
                .get((trump_count(game, hand) + missing).min(8))
                .map_or(0, |row| percent(row, aces))
        }
    };
    let total = i64::from(crate::rules::TOTAL_CARD_POINTS);
    let normal = i64::from(crate::rules::WIN_THRESHOLD);
    let needed = i64::from(win_threshold).max(normal);
    let win = win * (total - needed).max(0) / (total - normal);
    let faces: Vec<Card> = all_cards.iter().filter_map(|c| c.face()).collect();
    let matadors = crate::rules::matadors(game, &faces).count;
    let levels = matadors + 1 + u32::from(declaration.hand());
    let value = i64::from(base) * i64::from(levels) * 100;
    (win * value - (100 - win) * crate::content::RUN_LOSS_FACTOR * value) / 100
}

/// Exposed cards the skat is assumed to take away from a Null before the
/// skat is seen ([`ten_card_score`]): the two the bot discards.
const NULL_SKAT_DISCARDS: usize = 2;

/// Expected score of ten cards before the skat, in hundredths of a point,
/// without relics: the best of the four suit games, the grand
/// ([`suit_expected_score`] with the run's Grundwert `grand_base`) and,
/// with `null`, a Null after the skat with [`NULL_SKAT_DISCARDS`] exposed
/// cards fewer ([`null_expected_score`]). A rough measure of how good a
/// deal is, for the redeal decision ([`RunBot::redeal_threshold`]); the
/// skat is not counted, so every hand is a little better than this.
#[must_use]
pub fn ten_card_score(
    hand: &[DealtCard],
    deck: &[DealtCard],
    grand_base: u32,
    null: Option<&NullScoring>,
) -> i64 {
    let suits = Suit::ALL.iter().map(|&s| {
        let declaration = Declaration::suit(s, RiskLevel::Normal);
        let base = GameType::Suit(s).base_value().unwrap_or(0);
        suit_expected_score(hand, hand, deck, declaration, base, 61)
    });
    let grand = suit_expected_score(
        hand,
        hand,
        deck,
        Declaration::grand(RiskLevel::Normal),
        grand_base,
        61,
    );
    let null = null.map_or(i64::MIN, |scoring| {
        let exposed = null_unsafe_cards(hand, deck).saturating_sub(NULL_SKAT_DISCARDS);
        null_expected_score(hand, exposed, Declaration::null(NullRisk::Normal), scoring)
    });
    suits.chain([grand, null]).max().unwrap_or(0)
}

/// Optimal-stopping values of [`ten_card_score`] over random deals of the
/// standard deck (hundredths; 200 000 deals, Grand 18, Null formula of the
/// Alleinspieler): index `k − 1` is the expected best of `k` deals taken
/// with this rule, so a hand with `k` Einmischen left is thrown in below
/// index `k − 1` (`redeal_probe` test prints them).
pub const REDEAL_STOP: [i64; 5] = [877, 1759, 2355, 2824, 3203];

/// "Kahlschlag": +1 level per void side suit after the skat, at most this
/// many ([`RunBot::void_discard`]).
const CLEAR_CUT_MAX_VOIDS: usize = 2;
/// Relic ids the game decisions look at (relics v4).
const CLEAR_CUT: &str = "clear_cut";
/// "Große Klappe": a won game with Schneider announced counts the Grundwert
/// twice.
const BIG_MOUTH: &str = "big_mouth";
/// "Offenes Visier": Ouvert asks for Schneider only.
const OPEN_VISOR: &str = "open_visor";

/// Deals of the unseen cards [`schneider_percent`] solves.
pub const BOLD_SAMPLES: u64 = 12;

/// The least chance of Schneider (percent of [`BOLD_SAMPLES`] deals) for
/// which the bot announces it with "Große Klappe" or plays Ouvert with
/// "Offenes Visier" ([`RunBot::bold_announcement`]). High on purpose: the
/// double-dummy deals see every card, the bot in play does not.
pub const BOLD_MIN_PERCENT: i64 = 80;

/// Nodes per deal after which [`schneider_percent`] estimates the rest
/// (keeps the search cheap; the estimate is the solver's static one).
const BOLD_NODES: u64 = 20_000;

/// Whether the player holds the relic `id`.
fn holds_relic(run: &Run, id: &str) -> bool {
    run.relics().iter().any(|r| r.as_str() == id)
}

/// Percent of [`BOLD_SAMPLES`] deals of the unseen cards on which the
/// declarer makes Schneider in `game` with the ten cards `kept`, solved
/// double-dummy ([`crate::ai::solver`]): the defenders' Augen at most
/// `schneider_limit`, the skat counted for the declarer. `cards` are the
/// twelve cards (kept and skat), `deck` every card of the game, `forehand`
/// leads the first trick. The deals come from a generator seeded with
/// `seed` and the cards of the declarer (in any order), so the same
/// decision gives the same answer in every phase. Stops early once the
/// deals left cannot lift it to [`BOLD_MIN_PERCENT`] (the deals not solved
/// count as missed). 0 if the unseen cards are not two
/// hands of ten ("Drei drücken").
#[must_use]
pub fn schneider_percent(
    game: GameType,
    kept: &[DealtCard],
    cards: &[DealtCard],
    deck: &[DealtCard],
    forehand: Seat,
    schneider_limit: u32,
    seed: u64,
) -> i64 {
    use crate::ai::solver::{Goal, Layout, Limits, probe_with, with_table};
    let unseen: Vec<DealtCard> = deck
        .iter()
        .copied()
        .filter(|c| !cards.contains(c))
        .collect();
    if kept.len() != 10 || unseen.len() != 20 {
        return 0;
    }
    let pool: u32 = kept.iter().chain(&unseen).map(|c| c.card_points()).sum();
    let need = i32::try_from(pool.saturating_sub(schneider_limit)).unwrap_or(i32::MAX);
    let key = cards
        .iter()
        .fold(seed, |acc, c| acc.wrapping_add(mix64(u64::from(c.id))));
    let mut rng = crate::rng::Rng::from_seed(mix64(key));
    let samples = i64::try_from(BOLD_SAMPLES).unwrap_or(1);
    // Misses after which the percent is below BOLD_MIN_PERCENT for sure.
    let allowed = samples - (BOLD_MIN_PERCENT * samples + 99) / 100;
    let limits = Limits {
        depth: 0,
        nodes: BOLD_NODES,
    };
    let made = with_table(16, |table| {
        let (mut made, mut missed) = (0, 0);
        for _ in 0..BOLD_SAMPLES {
            if missed > allowed {
                break;
            }
            let mut shuffled = unseen.clone();
            rng.shuffle(&mut shuffled);
            let (first, second) = shuffled.split_at(10);
            let layout = Layout {
                hands: [kept.to_vec(), first.to_vec(), second.to_vec()],
                trick_lead: forehand,
                trick: Vec::new(),
                skat_points: 0,
            };
            let (moves, _) = probe_with(game, Goal::DeclarerPoints, &layout, need, limits, table);
            let reached = if forehand.is_declarer() {
                moves.iter().any(|&(_, r)| r)
            } else {
                !moves.is_empty() && moves.iter().all(|&(_, r)| r)
            };
            made += i64::from(reached);
            missed += i64::from(!reached);
        }
        made
    });
    made * 100 / samples
}

/// The simple run policy.
#[derive(Debug, Clone, Copy, Default)]
pub struct RunBot;

fn count_rank<C: PlayingCard>(hand: &[C], rank: Rank) -> usize {
    // A demoted Unter ("Abgestuft") is no Unter.
    hand.iter()
        .filter(|c| c.face().is_some_and(|f| f.rank == rank))
        .filter(|c| !crate::rules::is_demoted_jack(**c))
        .count()
}

/// Whether the hand holds this printed card.
fn holds<C: PlayingCard>(hand: &[C], card: Card) -> bool {
    hand.iter().any(|c| c.face() == Some(card))
}

/// Number of trumps the hand holds in an unbroken run from the top of the
/// trump order (Kreuz-Bube downwards), i.e. Spitzen "mit".
fn top_trumps<C: PlayingCard>(hand: &[C], game: GameType) -> usize {
    game.trump_order()
        .into_iter()
        .take_while(|&card| holds(hand, card))
        .count()
}

/// Non-trump cards of the hand that may lose a trick: everything but
/// aces and tens backed by their ace.
fn side_losers<C: PlayingCard>(hand: &[C], game: GameType) -> usize {
    hand.iter()
        .filter(|&&c| !crate::rules::is_trump(game, c))
        .filter(|c| match c.face() {
            Some(f) if f.rank == Rank::Ace => false,
            Some(f) if f.rank == Rank::Ten => !holds(hand, Card::new(f.suit, Rank::Ace)),
            _ => true,
        })
        .count()
}

impl RunBot {
    /// Whether a hand is too weak to play (pass rule of DESIGN 10 and the
    /// balancing report): a suit game with fewer than six trumps unless it
    /// has five trumps and two aces; a grand without two jacks and two
    /// aces.
    #[must_use]
    pub fn is_weak(hand: &[DealtCard], declaration: Declaration) -> bool {
        let aces = count_rank(hand, Rank::Ace);
        match declaration.game() {
            GameType::Grand => count_rank(hand, Rank::Jack) < 2 || aces < 2,
            game => {
                let trumps = trump_count(game, hand);
                !(trumps >= 6 || (trumps == 5 && aces >= 2))
            }
        }
    }

    /// The highest Risikostufe the bot dares with this ten-card hand.
    ///
    /// Measured on 2 000 000 deals against the defender heuristic (DESIGN
    /// 14.3): an announcement only pays off when the hand has at most one
    /// side card that may lose a trick ("loser": anything but an ace or a
    /// ten backed by its ace). The rules take exactly the hand shapes whose
    /// mean score beat the plain declaration:
    ///
    /// - **Ouvert** (Hand, Schneider and Schwarz angesagt, open): no loser
    ///   and a suit game with eight trumps and the Kreuz-Bube, or a grand
    ///   with all four jacks. Won 100 % in the sample; about 5 hands in a
    ///   million.
    /// - **Schneider angesagt** (a hand game): a suit game without loser,
    ///   seven trumps and the Kreuz-Bube; or with one loser and eight
    ///   trumps from the top three jacks, or nine trumps from the top two.
    ///   A grand with three jacks and at most one loser. Won 92 to 100 %;
    ///   about 1 hand in 8 000.
    /// - Otherwise the risk of the declarer heuristic (Hand from eight
    ///   trumps).
    #[must_use]
    pub fn announcement(hand: &[DealtCard], declaration: Declaration) -> RiskLevel {
        let game = declaration.game();
        let trumps = trump_count(game, hand);
        let top = top_trumps(hand, game);
        let losers = side_losers(hand, game);
        let (ouvert, schneider) = match game {
            GameType::Suit(_) => (
                losers == 0 && trumps >= 8 && top >= 1,
                (losers == 0 && trumps >= 7 && top >= 1)
                    || (losers == 1 && ((trumps >= 8 && top >= 3) || (trumps >= 9 && top >= 2))),
            ),
            GameType::Grand => (
                losers == 0 && top >= 4,
                losers <= 1 && count_rank(hand, Rank::Jack) >= 3,
            ),
            GameType::Null => (false, false),
        };
        if ouvert {
            RiskLevel::Ouvert
        } else if schneider {
            RiskLevel::SchneiderDeclared
        } else {
            declaration.risk().unwrap_or(RiskLevel::Normal)
        }
    }

    /// Whether to answer Kontra with Re: only with a very strong hand. A
    /// suit game with seven trumps, or six trumps and two aces; a grand
    /// with three jacks and three aces, or all four jacks and two aces.
    #[must_use]
    pub fn says_re(hand: &[DealtCard], game: GameType) -> bool {
        let aces = count_rank(hand, Rank::Ace);
        match game {
            GameType::Grand => {
                let jacks = count_rank(hand, Rank::Jack);
                (jacks >= 3 && aces >= 3) || (jacks == 4 && aces >= 2)
            }
            game => {
                let trumps = trump_count(game, hand);
                trumps >= 7 || (trumps >= 6 && aces >= 2)
            }
        }
    }

    /// The long night (ADR 0020), for measurement: after der Alte the bot
    /// always continues ([`RunAction::EnterNight`]). [`RunBot::next_action`]
    /// stops at the won run as before; only a caller that wants the night
    /// (the simulator's `--night`) asks here.
    #[must_use]
    pub fn night_action(&self, run: &Run) -> Option<RunAction> {
        (run.phase() == RunPhase::Won && !run.in_night()).then_some(RunAction::EnterNight)
    }

    /// The next action, or `None` once the run is over.
    #[must_use]
    pub fn next_action(&self, run: &Run, content: &Content) -> Option<RunAction> {
        if let Some(action) = Self::schnapps_reward_action(run) {
            return Some(action);
        }
        match run.phase() {
            RunPhase::Won | RunPhase::Lost => None,
            RunPhase::Shop => Some(self.shop_action(run, content)),
            RunPhase::Encounter => Some(self.encounter_action(run, content)),
            RunPhase::Route => Some(self.route_action(run, content)),
            RunPhase::BossReward => Some(Self::boss_reward_action(run, content)),
            RunPhase::CardReward => Some(Self::card_reward_action(run, content)),
            RunPhase::Game => self.game_action(run, content),
        }
    }

    /// The relic pick of a won boss pub (ADR 0017): the most valuable
    /// relic on offer ([`relic_value`], the first on a tie) into a free
    /// slot (always, unless the slots are capped); with every slot full in
    /// place of the weakest owned relic if it is worth at least
    /// [`SWAP_MARGIN`] more, else none.
    #[must_use]
    pub fn boss_reward_action(run: &Run, content: &Content) -> RunAction {
        let none = RunAction::ChooseBossRelic {
            offer: None,
            replace: None,
        };
        let Some(reward) = run.boss_reward() else {
            return none;
        };
        let mut best: Option<(u32, usize)> = None;
        for (i, id) in reward.relics.iter().enumerate() {
            let value = relic_score(run, content, id);
            if best.is_none_or(|(b, _)| value > b) {
                best = Some((value, i));
            }
        }
        let Some((value, offer)) = best else {
            return none;
        };
        if content.relic_slot_free(run.character(), run.relics().len()) {
            return RunAction::ChooseBossRelic {
                offer: Some(offer),
                replace: None,
            };
        }
        let weakest = run
            .relics()
            .iter()
            .enumerate()
            .min_by_key(|(i, id)| (relic_value(id), *i));
        match weakest {
            Some((index, id)) if value >= (relic_value(id) + SWAP_MARGIN) * 100 => {
                RunAction::ChooseBossRelic {
                    offer: Some(offer),
                    replace: Some(index),
                }
            }
            _ => none,
        }
    }

    /// The card reward of a won pub (DESIGN 5.0, fixed pairs since
    /// 05.11.2026): the offer worth most to the bot. An offer is worth its
    /// Zinke ([`zinke_score`]), less the Zinke it would replace; on a tie
    /// a raise before a bare card before a replace, then the card with the
    /// most Augen, Unter last (trumps anyway), then the first offer.
    /// Skipped for the Groschen if no offer is worth anything.
    #[must_use]
    pub fn card_reward_action(run: &Run, content: &Content) -> RunAction {
        let Some(reward) = run.card_reward() else {
            return RunAction::SkipCardReward;
        };
        let best = reward
            .offers
            .iter()
            .enumerate()
            .filter(|&(offer, _)| run.card_reward_open(content, offer))
            .filter_map(|(offer, o)| {
                let (_, replaces, raises) = run.mod_result(content, o.card, &o.card_mod, o.value);
                let gain = i64::from(zinke_score(run, content, &o.card_mod));
                let lost = replaces
                    .as_ref()
                    .map_or(0, |old| i64::from(zinke_score(run, content, old)));
                let fit = if raises {
                    2
                } else if replaces.is_none() {
                    1
                } else {
                    0
                };
                let (points, jack) = match run.edition().get(o.card)?.kind {
                    CardKind::Normal(card) => (card.card_points(), card.is_jack()),
                    _ => (0, false),
                };
                Some(((gain - lost, fit, !jack, points), offer))
            })
            .max_by_key(|&(key, offer)| (key, std::cmp::Reverse(offer)));
        // "Der Abstinenzler": skipping is worth +2 Grundwert for the rest
        // of the run, more than any Zinke the bot does not build on.
        let skip_worth = if run.relics().iter().any(|r| r.as_str() == "teetotaller") {
            TEETOTALLER_SKIP_WORTH
        } else {
            0
        };
        match best {
            Some(((value, ..), offer)) if value > skip_worth => RunAction::TakeCardReward { offer },
            _ => RunAction::SkipCardReward,
        }
    }

    /// The Schnaps of a won pub with every slot full, if it waits: in
    /// place of the weakest owned Schnaps (the first on a tie) if it is
    /// worth more, else sold.
    #[must_use]
    pub fn schnapps_reward_action(run: &Run) -> Option<RunAction> {
        let new = consumable_value(run.schnapps_reward()?);
        let give_up = run
            .consumables()
            .iter()
            .enumerate()
            .min_by_key(|(i, id)| (consumable_value(id), *i))
            .filter(|(_, id)| new > consumable_value(id))
            .map(|(slot, _)| slot);
        Some(RunAction::ChooseSchnappsReward { give_up })
    }

    /// The route policy, see the module docs: the way with the best
    /// score, the first on a tie.
    #[must_use]
    pub fn route_action(&self, run: &Run, content: &Content) -> RunAction {
        let Some(route) = run.route() else {
            return RunAction::LeaveShop;
        };
        if let Some(action) = Self::route_schnapps(run, content) {
            return action;
        }
        if let Some(action) = Self::malus_choice(run) {
            return action;
        }
        let cursed = Self::cursed_card(run, content).is_some();
        let angel = run.relics().iter().any(|r| {
            content
                .relic(r)
                .is_some_and(|d| d.passive.contains(&Effect::SecondChance))
        });
        let relics = run.relics().len();
        let strong = relics >= 5 || (relics >= 4 && angel);
        let mut best: Option<(i64, usize)> = None;
        for (i, option) in route.options.iter().enumerate() {
            let mut score: i64 = 0;
            for stop in &option.stops {
                score += match stop.category {
                    StopCategory::Tresen if run.coins() >= RICH => 4,
                    StopCategory::Tresen => 1,
                    StopCategory::Upgrade if cursed => 5,
                    StopCategory::Upgrade => 2,
                    StopCategory::Rest if cursed => 3,
                    StopCategory::Rest => 2,
                    StopCategory::Event => 1,
                    StopCategory::FoundCoins => 2,
                    StopCategory::Kiosk => 1,
                    StopCategory::DiceStall => 0,
                };
            }
            if option.spelunke.is_some() {
                score += if strong { 3 } else { -20 };
            }
            if best.is_none_or(|(b, _)| score > b) {
                best = Some((score, i));
            }
        }
        RunAction::ChooseRoute(best.map_or(0, |(_, i)| i))
    }

    /// The encounter policy, see the module docs: the option with the best
    /// value ([`RunBot::option_value`]) among those that can be taken, the
    /// first on a tie. One is always taken.
    #[must_use]
    pub fn encounter_action(&self, run: &Run, content: &Content) -> RunAction {
        if let Some(action) = Self::paint_choice(run, content) {
            return action;
        }
        let Some(encounter) = run.encounter() else {
            return RunAction::LeaveShop;
        };
        let mut best: Option<(i64, RunAction)> = None;
        for (i, option) in encounter.options.iter().enumerate() {
            if run.encounter_option_block(content, i).is_some() {
                continue;
            }
            let Some((target, marke)) = Self::option_choice(run, content, i, option) else {
                continue;
            };
            let value = Self::option_value(run, content, option);
            if best.as_ref().is_none_or(|(v, _)| value > *v) {
                best = Some((
                    value,
                    RunAction::ChooseEncounter {
                        option: i,
                        target,
                        marke,
                    },
                ));
            }
        }
        best.map_or(
            RunAction::ChooseEncounter {
                option: 0,
                target: None,
                marke: None,
            },
            |(_, action)| action,
        )
    }

    /// Target and Marke the bot uses for an option, `None` if none is
    /// accepted. With a Marke to choose (04.11.2026), the most valuable
    /// one by [`zinken_value`] (the Pfandmarke, then the first by id)
    /// that some target accepts, on the target [`RunBot::option_target`]
    /// would pick for it.
    #[allow(clippy::type_complexity)]
    fn option_choice(
        run: &Run,
        content: &Content,
        index: usize,
        option: &EncounterOption,
    ) -> Option<(Option<u32>, Option<CardModId>)> {
        if !option.needs_marke() {
            return Self::option_target(run, content, index, option, None).map(|t| (t, None));
        }
        let mut marken: Vec<CardModId> = run
            .marken(content)
            .into_iter()
            .map(|m| m.id.clone())
            .collect();
        marken.sort_by_key(|m| std::cmp::Reverse(zinken_value(m)));
        marken.into_iter().find_map(|marke| {
            Self::option_target(run, content, index, option, Some(&marke)).map(|t| (t, Some(marke)))
        })
    }

    /// The target the bot uses for an option: `Some(None)` without one,
    /// `None` if no target is accepted. Cleansing aims at a cursed card, a
    /// curse at a low card without modification, everything else at an
    /// ace; a relic option at the weakest relic.
    #[allow(clippy::option_option)]
    fn option_target(
        run: &Run,
        content: &Content,
        index: usize,
        option: &EncounterOption,
        marke: Option<&CardModId>,
    ) -> Option<Option<u32>> {
        let accepted = |target: u32| {
            run.accepts(
                content,
                RunAction::ChooseEncounter {
                    option: index,
                    target: Some(target),
                    marke: marke.cloned(),
                },
            )
        };
        if run.option_needs_relic(content, option) {
            let weakest = run
                .relics()
                .iter()
                .enumerate()
                .min_by_key(|(i, id)| (relic_value(id), *i))
                .and_then(|(i, _)| u32::try_from(i).ok())?;
            return accepted(weakest).then_some(Some(weakest));
        }
        if !option.needs_card() {
            let ok = run.accepts(
                content,
                RunAction::ChooseEncounter {
                    option: index,
                    target: None,
                    marke: marke.cloned(),
                },
            );
            return (ok || marke.is_none()).then_some(None);
        }
        let cleanse = option.effects.contains(&EncounterEffect::Cleanse);
        let curse = option.effects.iter().any(|e| match e {
            EncounterEffect::Mark {
                card: CardPick::Chosen,
                card_mod: Some(m),
                ..
            } => content.card_mod(m).is_some_and(CardModDef::is_curse),
            _ => false,
        });
        let mut candidates: Vec<u32> = if cleanse {
            Self::cursed_card(run, content).into_iter().collect()
        } else if curse {
            Self::low_targets(run)
        } else {
            Self::ace_targets(run)
        };
        if !cleanse {
            candidates.extend(run.edition().cards().iter().map(|c| c.id));
        }
        candidates.into_iter().find(|&c| accepted(c)).map(Some)
    }

    /// Deck ids of sevens, eights and nines without modification.
    fn low_targets(run: &Run) -> Vec<u32> {
        run.edition()
            .cards()
            .iter()
            .filter(|c| c.mods.is_empty())
            .filter_map(|c| match c.kind {
                CardKind::Normal(card)
                    if matches!(card.rank, Rank::Seven | Rank::Eight | Rank::Nine) =>
                {
                    Some(c.id)
                }
                _ => None,
            })
            .collect()
    }

    /// The bot's value of an encounter option, in rough Groschen: what its
    /// effects bring minus what it costs. Simple rules, not tuned: Groschen
    /// count as they are, a house rule or relic as [`relic_value`] + 1, a
    /// Schnaps as [`consumable_value`] + 1 (its price with every slot full),
    /// a life of an old save its Groschen, a Zinke 1 on a random and 2 on a
    /// chosen card, a curse -2, a swap the gain in value when it is at
    /// least [`SWAP_MARGIN`], higher stakes -1 (the bot never seeks them),
    /// a bet its expected payout (Schneider a quarter of the time, a win
    /// three in five), a throw of the dice its expected payout.
    #[must_use]
    pub fn option_value(run: &Run, content: &Content, option: &EncounterOption) -> i64 {
        let free_slot = content.relic_slot_free(run.character(), run.relics().len());
        let item_slot =
            run.consumables().len() < usize::try_from(content.run.consumable_slots).unwrap_or(0);
        let weakest = run.relics().iter().map(relic_value).min().unwrap_or(0);
        let stake = i64::from(option.cost);
        let mut value: i64 = -stake;
        for effect in &option.effects {
            value += match effect {
                EncounterEffect::Coins(n) => i64::from(*n),
                EncounterEffect::EveningRule(Some(id)) => i64::from(relic_value(id)) + 1,
                EncounterEffect::Relic(Some(id)) if free_slot => i64::from(relic_value(id)) + 1,
                // Every slot full: in place of the weakest, only if
                // clearly better (like a swap).
                EncounterEffect::Relic(Some(new)) | EncounterEffect::SwapRelic(Some(new)) => {
                    let gain = i64::from(relic_value(new)) - i64::from(weakest);
                    if gain >= i64::from(SWAP_MARGIN) {
                        gain
                    } else {
                        -1
                    }
                }
                EncounterEffect::Mark {
                    card,
                    card_mod: Some(m),
                    curse_percent,
                    ..
                } => {
                    let good = match card {
                        CardPick::Random => 1,
                        CardPick::Chosen => 2,
                    };
                    let base = if content.card_mod(m).is_some_and(CardModDef::is_curse) {
                        -2
                    } else {
                        good
                    };
                    let p = i64::from(*curse_percent);
                    (base * (100 - p) - 2 * p) / 100
                }
                // A Marke like a Zinke: 1 on a random, 2 on a chosen card.
                EncounterEffect::Marke { card, .. } => match card {
                    CardPick::Random => 1,
                    CardPick::Chosen => 2,
                },
                EncounterEffect::Schnapps(Some(id)) => {
                    if item_slot {
                        i64::from(consumable_value(id)) + 1
                    } else {
                        content.consumable(id).map_or(0, |c| i64::from(c.price))
                    }
                }
                EncounterEffect::Lives(n) => {
                    i64::from(*n) * i64::from(crate::content::LEGACY_LIFE_COINS)
                }
                EncounterEffect::NextPubRedeals(n) => 2 * i64::from(*n),
                EncounterEffect::SeeSkat | EncounterEffect::Scout(_) => 1,
                EncounterEffect::NextGameFactor(_) => -1,
                EncounterEffect::Bet { on, percent } => {
                    let chance = match on {
                        BetOn::Schneider => 25,
                        BetOn::Win => 60,
                    };
                    stake * i64::from(*percent) * chance / 10_000
                }
                EncounterEffect::Dice { chance, percent } => {
                    stake * i64::from(*percent) * i64::from(*chance) / 10_000
                }
                EncounterEffect::Painter => 3,
                EncounterEffect::Cleanse => 5,
                _ => 0,
            };
        }
        value
    }

    /// At a Kiosk: the most valuable Schnaps ([`consumable_value`]) if it
    /// can be bought, else on.
    fn kiosk_action(run: &Run, content: &Content) -> RunAction {
        let Some(shop) = run.shop() else {
            return RunAction::LeaveShop;
        };
        shop.consumables
            .iter()
            .enumerate()
            .filter_map(|(slot, id)| Some((slot, consumable_value(id.as_ref()?))))
            .filter(|&(_, value)| value > 0)
            .filter(|&(slot, _)| run.accepts(content, RunAction::BuyConsumable(slot)))
            .max_by_key(|&(slot, value)| (value, std::cmp::Reverse(slot)))
            .map_or(RunAction::LeaveShop, |(slot, _)| {
                RunAction::BuyConsumable(slot)
            })
    }

    /// Slot of an owned consumable with this action, if usable now.
    fn usable(
        run: &Run,
        content: &Content,
        wanted: impl Fn(&ConsumableAction) -> bool,
        target: Option<u32>,
    ) -> Option<RunAction> {
        run.consumables()
            .iter()
            .enumerate()
            .filter(|(_, id)| content.consumable(id).is_some_and(|d| wanted(&d.action)))
            .map(|(slot, _)| RunAction::UseConsumable {
                slot,
                target,
                suit: None,
            })
            .find(|action| run.accepts(content, action.clone()))
    }

    /// The Tresen policy: one action per call, see the module docs.
    #[must_use]
    pub fn shop_action(&self, run: &Run, content: &Content) -> RunAction {
        let Some(shop) = run.shop() else {
            return RunAction::LeaveShop;
        };
        if let Some(action) = Self::paint_choice(run, content) {
            return action;
        }
        if shop.kiosk {
            return Self::kiosk_action(run, content);
        }
        let coins = run.coins();
        let price = |p: u32| run.shop_price(content, p);
        let cursed = Self::cursed_card(run, content);
        let likes_curses = Self::likes_curses(run, content);
        let zinken = &content.run.shop.zinken;
        // 1. "Karte reinigen" on a cursed card (ADR 0015).
        if let Some(target) = cursed {
            let action = RunAction::CleanseCard(target);
            if price(zinken.cleanse_price) <= coins && run.accepts(content, action.clone()) {
                return action;
            }
        }
        let free_slot = content.relic_slot_free(run.character(), run.relics().len());
        // Relic offers the bot wants: (value, price, slot).
        let offers: Vec<(u32, u32, usize)> = shop
            .relics
            .iter()
            .enumerate()
            .filter_map(|(slot, id)| {
                let id = id.as_ref()?;
                let def = content.relic(id)?;
                let score = relic_score(run, content, id);
                let value = if run.is_fence(slot) && !likes_curses {
                    score.saturating_sub(TRADE_PENALTY * 100)
                } else {
                    score
                };
                Some((value, run.relic_offer_cost(content, slot, def.price), slot))
            })
            .filter(|&(value, _, _)| value > 0)
            .collect();
        // 2. All slots full (8 since 05.11.2026): the most valuable offer
        //    in place of the weakest relic, if it is clearly better
        //    ([`SWAP_MARGIN`]) and affordable with the weakest sold.
        if !free_slot {
            let weakest = run
                .relics()
                .iter()
                .enumerate()
                .map(|(index, id)| {
                    let refund = content.relic(id).map_or(0, |d| super::sell_price(d.price));
                    (relic_value(id) * 100, index, refund)
                })
                .min_by_key(|&(value, index, _)| (value, std::cmp::Reverse(index)));
            if let Some((weak_value, replace, refund)) = weakest {
                let best = offers
                    .iter()
                    .filter(|&&(value, price, _)| {
                        value >= weak_value + SWAP_MARGIN * 100 && price <= coins + refund
                    })
                    .max_by_key(|&&(value, price, slot)| (value, price, std::cmp::Reverse(slot)));
                if let Some(&(_, _, slot)) = best {
                    let action = RunAction::ReplaceRelic { slot, replace };
                    if run.accepts(content, action.clone()) {
                        return action;
                    }
                }
            }
        }
        // 3. Buy the most valuable relic.
        if free_slot {
            let best = offers
                .iter()
                .filter(|&&(_, price, _)| price <= coins)
                .max_by_key(|&&(value, price, slot)| (value, price, std::cmp::Reverse(slot)));
            if let Some(&(_, _, slot)) = best {
                return RunAction::BuyRelic(slot);
            }
        }
        // 4. Buy the most valuable consumable.
        let consumable_slot =
            run.consumables().len() < usize::try_from(content.run.consumable_slots).unwrap_or(0);
        if consumable_slot {
            let best = shop
                .consumables
                .iter()
                .enumerate()
                .filter_map(|(slot, id)| {
                    let id = id.as_ref()?;
                    let def = content.consumable(id)?;
                    let value = consumable_value(id);
                    let cost = price(def.price);
                    (value > 0 && cost <= coins).then_some((value, cost, slot))
                })
                .max_by_key(|&(value, price, slot)| (value, price, std::cmp::Reverse(slot)));
            if let Some((_, _, slot)) = best {
                return RunAction::BuyConsumable(slot);
            }
        }
        // 5. The Auslage: the best Zinke on a card that carries nothing
        // else (the same one raised is fine), never replacing.
        let best = shop
            .card_offers
            .iter()
            .enumerate()
            .filter_map(|(slot, offer)| {
                let offer = offer.as_ref()?;
                let card = run.edition().get(offer.card)?;
                let bare = card.mods.iter().all(|m| *m == offer.card_mod);
                let score = zinke_score(run, content, &offer.card_mod);
                let value = if offer.bargain.is_some() && !likes_curses {
                    score.saturating_sub(TRADE_PENALTY * 100)
                } else {
                    score
                };
                let cost = run.card_offer_cost(content, offer);
                (bare && value > 0 && cost <= coins).then_some((value, cost, slot))
            })
            .max_by_key(|&(value, cost, slot)| {
                (value, std::cmp::Reverse(cost), std::cmp::Reverse(slot))
            });
        if let Some((_, _, slot)) = best {
            let action = RunAction::BuyCardOffer(slot);
            if run.accepts(content, action.clone()) {
                return action;
            }
        }
        // 6. The Kartenmaler for an ace without modification, if Groschen
        // are plenty.
        let bare_ace = Self::ace_targets(run).into_iter().find(|&card| {
            run.edition().get(card).is_some_and(|c| c.mods.is_empty())
                && coins >= run.painter_cost(content, card) + PAINTER_RESERVE
                && run.accepts(content, RunAction::HirePainter(card))
        });
        if let Some(card) = bare_ace {
            return RunAction::HirePainter(card);
        }
        // 7. Nothing sensible to buy: "Neu auslosen" (once per visit), if
        // Groschen are plenty.
        let cost = run.reroll_cost(content);
        if shop.rerolls == 0
            && coins >= cost + REROLL_RESERVE
            && run.accepts(content, RunAction::Reroll)
        {
            return RunAction::Reroll;
        }
        RunAction::LeaveShop
    }

    /// The Kartenmaler waits: take his most valuable modification, the
    /// first on a tie.
    fn paint_choice(run: &Run, content: &Content) -> Option<RunAction> {
        let painter = run.painter()?;
        let best = painter
            .options
            .iter()
            .enumerate()
            .max_by_key(|(i, (m, _))| (zinke_score(run, content, m), std::cmp::Reverse(*i)))
            .map_or(0, |(i, _)| i);
        Some(RunAction::ChoosePaint(best))
    }

    /// The first card of the player's edition that carries a curse ("Abgestuft"
    /// stays: it is the Nullspieler's start edition).
    fn cursed_card(run: &Run, content: &Content) -> Option<u32> {
        // "Bruderherz" cannot cleanse his curses: nothing to aim for.
        // With "Pechvogel" a curse is worth Grundwert.
        if content
            .character(run.character())
            .is_some_and(|c| c.rule.keep_curses)
            || Self::likes_curses(run, content)
        {
            return None;
        }
        run.edition()
            .cards()
            .iter()
            .find(|c| {
                c.mods.iter().any(|m| {
                    content
                        .card_mod(m)
                        .is_some_and(|d| d.kind == crate::content::ModKind::Curse)
                })
            })
            .map(|c| c.id)
    }

    /// Whether the bot owns a relic that pays for cursed cards
    /// ("Pechvogel"): then it keeps its curses and takes trade offers at
    /// full value.
    fn likes_curses(run: &Run, content: &Content) -> bool {
        run.relics()
            .iter()
            .filter_map(|r| content.relic(r))
            .flat_map(|d| {
                d.passive
                    .iter()
                    .chain(d.hooks.iter().flat_map(|h| h.then()))
            })
            .any(|e| matches!(e, Effect::AddBaseValuePerCursedCard(n) if *n > 0))
    }

    /// Deck ids of the aces, Kreuz first, the least modified first.
    fn ace_targets(run: &Run) -> Vec<u32> {
        let mut aces: Vec<(usize, Suit, u32)> = run
            .edition()
            .cards()
            .iter()
            .filter_map(|c| match c.kind {
                CardKind::Normal(card) if card.rank == Rank::Ace => {
                    Some((c.mods.len(), card.suit, c.id))
                }
                _ => None,
            })
            .collect();
        aces.sort_by_key(|&(mods, suit, id)| (mods, std::cmp::Reverse(suit), id));
        aces.into_iter().map(|(_, _, id)| id).collect()
    }

    fn game_action(&self, run: &Run, content: &Content) -> Option<RunAction> {
        let active = run.game()?;
        let game = &active.game;
        let declarer = DeclarerHeuristic;
        if active.awaiting_re {
            let declaration = game.declaration()?;
            let re = RunBot::says_re(game.hand(Seat::DECLARER), declaration.game());
            return Some(RunAction::AnswerKontra { re });
        }
        if let Some(cards) = Self::rise_cards(run) {
            return Some(RunAction::ChooseRiseCards(cards));
        }
        match game.phase() {
            Phase::Declaring => {
                if let Some(rescue) = Self::rescue_schnapps(run, content) {
                    return Some(rescue);
                }
                let action = Self::before_skat(run, content)?;
                if action == RunAction::PickUpSkat {
                    if let Some(peek) = Self::usable(
                        run,
                        content,
                        |a| matches!(a, ConsumableAction::PeekSkat),
                        None,
                    ) {
                        return Some(peek);
                    }
                }
                Some(action)
            }
            Phase::Discarding => {
                // Saves from before ADR 0011 declared before the skat.
                if let Some(declaration) = game.declaration() {
                    let [a, b] = Self::discard_for(
                        game.hand(Seat::DECLARER),
                        declaration,
                        game.deck_cards(),
                    )?;
                    return Some(RunAction::Discard([a.id, b.id]));
                }
                let (_, discard) = Self::after_skat(run, content)?;
                Some(Self::discard_action(&discard)?)
            }
            Phase::DeclaringAfterSkat => {
                let (declaration, _) = Self::after_skat(run, content)?;
                if let Some(action) = Self::schnapps_before_declaring(run, content, declaration) {
                    return Some(action);
                }
                Some(RunAction::Declare(declaration))
            }
            Phase::Playing => {
                if !active.rules.skat_swaps.is_empty() {
                    if let Some(action) = Self::take_from_skat(run, content) {
                        return Some(action);
                    }
                }
                if let Some(card) = Self::undead_card(run, content) {
                    return Some(RunAction::Play(card.id));
                }
                if let Some(card) = Self::mirror_card(run, content) {
                    return Some(RunAction::Play(card.id));
                }
                // A Wandelkarte has its settled suit by now: a card like
                // any other. A Doppelgänger waits for its mirror.
                let mut view = game.view(Seat::DECLARER)?;
                Self::keep_for_mirror(run, content, &mut view.legal_moves);
                let chosen = declarer.choose(&view)?;
                Some(RunAction::Play(chosen.id))
            }
            Phase::Finished(_) => None,
        }
    }

    /// The three cards the bot offers for a rising "Untot" card
    /// ([`Run::rise_pending`]; simple rule, not tuned): the weakest of his
    /// hand, fewest Augen first, then the lowest strength in the game, then
    /// the id. `None` if nothing waits.
    #[must_use]
    pub fn rise_cards(run: &Run) -> Option<[u32; 3]> {
        run.rise_pending()?;
        let game = &run.game()?.game;
        let declaration = game.declaration()?;
        let mut hand: Vec<DealtCard> = game.hand(Seat::DECLARER).to_vec();
        hand.sort_by_key(|&c| {
            (
                c.signed_points(),
                crate::ai::solver::card_strength(declaration.game(), c),
                c.id,
            )
        });
        match hand.as_slice() {
            [a, b, c, ..] => Some([a.id, b.id, c.id]),
            _ => None,
        }
    }

    /// An "Untot" card the bot plays now (simple rule, not tuned): playing
    /// last to the trick, a card of the hand that may rise
    /// ([`Run::can_rise`]), is legal and takes the trick. It comes back,
    /// and the substitute's Augen go to the player. `None` otherwise.
    #[must_use]
    pub fn undead_card(run: &Run, content: &Content) -> Option<DealtCard> {
        let active = run.game()?;
        let game = &active.game;
        let declaration = game.declaration()?;
        let trick = game.current_trick();
        if trick.len() != 2 || active.awaiting_re {
            return None;
        }
        game.legal_moves(Seat::DECLARER)
            .into_iter()
            .filter(|&c| run.can_rise(content, c))
            .find(|&c| {
                let mut cards = trick.to_vec();
                cards.push(c);
                crate::rules::winning_index(declaration.game(), &cards) == Some(2)
            })
    }

    /// The least Augen a trick must be worth with the copy before the bot
    /// mirrors a "Doppelgänger" into it ([`RunBot::mirror_card`]; simple
    /// rule, not tuned): a Daus or a Zehn in the trick, copied.
    pub const MIRROR_MIN_AUGEN: i64 = 15;

    /// A "Doppelgänger" the bot plays now (simple rule, not tuned): playing
    /// last to the trick, a card of the hand that mirrors
    /// ([`Run::mirrors_now`]) when the trick with the copy is worth at
    /// least [`RunBot::MIRROR_MIN_AUGEN`] Augen, or when it is the only
    /// legal card. `None` otherwise.
    #[must_use]
    pub fn mirror_card(run: &Run, content: &Content) -> Option<DealtCard> {
        let active = run.game()?;
        let game = &active.game;
        let declaration = game.declaration()?;
        let trick = game.current_trick();
        if trick.len() != 2 || active.awaiting_re {
            return None;
        }
        let legal = game.legal_moves(Seat::DECLARER);
        let card = legal
            .iter()
            .copied()
            .find(|&c| run.mirrors_now(content, c))?;
        if legal.len() == 1 {
            return Some(card);
        }
        let copied = trick[crate::rules::winning_index(declaration.game(), trick)?];
        let worth: i64 =
            trick.iter().map(|c| c.signed_points()).sum::<i64>() + copied.signed_points();
        (worth >= Self::MIRROR_MIN_AUGEN).then_some(card)
    }

    /// Keep a "Doppelgänger" whose mirror is still to come
    /// ([`Run::can_mirror`]) out of `legal` while another card may be
    /// played: the bot plays it only through [`RunBot::mirror_card`] or
    /// when nothing else is legal. The search knows nothing of the mirror.
    pub fn keep_for_mirror(run: &Run, content: &Content, legal: &mut Vec<DealtCard>) {
        if legal.iter().any(|&c| !run.can_mirror(content, c)) {
            legal.retain(|&c| !run.can_mirror(content, c));
        }
    }

    /// The decision with ten cards (Skatordnung, ADR 0011), for every
    /// character (ADR 0012):
    ///
    /// - Null Ouvert Hand without an exposed card.
    /// - A hand game of the declarer heuristic (eight trumps, or an
    ///   announcement from [`RunBot::announcement`]).
    /// - Throw the hand in ([`RunBot::throw_in`]) if it is weak for a suit
    ///   game or grand ([`RunBot::is_weak`]) and has more than
    ///   [`NULL_MAX_EXPOSED`] exposed cards for Null.
    /// - Use one of the game's own Einmischen on a mediocre hand
    ///   ([`RunBot::redeal_threshold`], since 05.11.2026).
    /// - Otherwise pick up the skat; the game is chosen with twelve cards
    ///   ([`RunBot::after_skat`]).
    ///
    /// "Der Schieber" (hand games only) and "Der Angeber" (always open)
    /// narrow the choice, see [`RunBot::forced_hand`].
    fn before_skat(run: &Run, content: &Content) -> Option<RunAction> {
        let active = run.game()?;
        let game = &active.game;
        let hand = game.hand(Seat::DECLARER);
        let deck = game.deck_cards();
        let exposed = null_unsafe_cards(hand, deck);
        let suit = DeclarerHeuristic.choose_declaration(hand);
        let suit = suit.with_risk(RunBot::announcement(hand, suit));
        let rules = &active.rules;
        if rules.require_hand {
            return Self::forced_hand(run, content);
        }
        // "Fusel" on a hand that is not weak, one next-game Schnaps per game.
        if !rules.require_ouvert
            && run.pending_next_game().is_empty()
            && !RunBot::is_weak(hand, suit)
        {
            if let Some(action) = Self::usable(
                run,
                content,
                |a| matches!(a, ConsumableAction::NextGameCursed(_)),
                None,
            ) {
                return Some(action);
            }
        }
        let null_counts = Self::null_counts(run, content, None);
        let exposed = if null_counts { exposed } else { usize::MAX };
        if exposed == 0 {
            return Some(RunAction::Declare(Declaration::null(NullRisk::OuvertHand)));
        }
        if rules.require_ouvert {
            if suit.ouvert() {
                return Some(RunAction::Declare(suit));
            }
        } else if suit.hand() {
            return Some(RunAction::Declare(suit));
        }
        let weak = if rules.require_ouvert {
            true
        } else {
            RunBot::is_weak(hand, suit)
        };
        if weak && exposed > NULL_MAX_EXPOSED {
            if let Some(action) = Self::throw_in(run, content) {
                return Some(action);
            }
        }
        if Self::redeal_mediocre(run, content, hand, deck, null_counts) {
            return Some(RunAction::Redeal);
        }
        Some(RunAction::PickUpSkat)
    }

    /// Whether to spend one of the game's own Einmischen on a hand that is
    /// not weak but mediocre: its [`ten_card_score`] is below
    /// [`RunBot::redeal_threshold`] for the Einmischen left. Only real
    /// Einmischen, no Schnaps.
    fn redeal_mediocre(
        run: &Run,
        content: &Content,
        hand: &[DealtCard],
        deck: &[DealtCard],
        null_counts: bool,
    ) -> bool {
        let left = run.redeals_left(content);
        if left == 0 || !run.accepts(content, RunAction::Redeal) {
            return false;
        }
        let scoring = run.null_scoring(content);
        let grand = content.run.base_value(GameType::Grand).unwrap_or(0);
        let score = ten_card_score(hand, deck, grand, null_counts.then_some(&scoring));
        score < Self::redeal_threshold(left)
    }

    /// The [`ten_card_score`] (hundredths) below which the bot throws a
    /// deal in with `left` Einmischen: the optimal stopping rule on the
    /// score of a random deal ([`REDEAL_STOP`]: with `k` left, redeal when
    /// the hand is worth less than the expected best of `k − 1` more
    /// deals). Measurement tool: it stands in for a sensible human, who
    /// throws in mediocre hands while Einmischen are left (round 2 found
    /// the bot used the third one rarely).
    ///
    /// Measured on 05.11.2026 (curve of the first pub, 1000 pubs, seed 42,
    /// two Einmischen): the first pub at Zielwert 35 / 55 / 90 / 145 went
    /// from 83,2 / 73,4 / 56,7 / 30,9 % (only weak hands thrown in) to
    /// 85,3 / 77,4 / 62,9 / 36,9 %. The thresholds × 0,75, × 1,25, × 1,5
    /// and × 2 were equal or worse, also at high Zielwerte, and keeping
    /// every hand with a table win chance of 70, 80 or 90 % or more was
    /// worse too; so the rule does not look at the pub's Zielwert (which
    /// also keeps `--curve`, played without one, equal to the runs).
    #[must_use]
    pub fn redeal_threshold(left: u32) -> i64 {
        let index = usize::try_from(left.saturating_sub(1)).unwrap_or(usize::MAX);
        REDEAL_STOP
            .get(index)
            .or(REDEAL_STOP.last())
            .copied()
            .unwrap_or(0)
    }

    /// A hand game is forced ("Der Schieber"): the declaration with the
    /// better expected score of the heuristic's suit game or grand with Hand
    /// and Null Hand (Null Ouvert Hand without an exposed card). Without the
    /// skat the bot cannot discard its exposed cards, so it takes Null Hand
    /// only with at most [`NULL_HAND_MAX_EXPOSED`] of them. A weak hand is
    /// thrown in if it can be; "Der Angeber" leaves Null Ouvert Hand or an
    /// open suit game.
    fn forced_hand(run: &Run, content: &Content) -> Option<RunAction> {
        let active = run.game()?;
        let game = &active.game;
        let hand = game.hand(Seat::DECLARER);
        let exposed = null_unsafe_cards(hand, game.deck_cards());
        let scoring = run.null_scoring(content);
        let mut suit = DeclarerHeuristic.choose_declaration(hand);
        suit = suit.with_risk(RunBot::announcement(hand, suit).max(RiskLevel::Hand));
        let null = if exposed == 0 || active.rules.require_ouvert {
            Declaration::null(NullRisk::OuvertHand)
        } else {
            Declaration::null(NullRisk::Hand)
        };
        let null_ev = null_expected_score(hand, exposed, null, &scoring);
        let suit_ev = suit_expected_score(
            hand,
            hand,
            game.deck_cards(),
            suit,
            content.run.base_value(suit.game()).unwrap_or(0),
            active.rules.thresholds.win,
        );
        let null_fits = exposed <= NULL_HAND_MAX_EXPOSED && Self::null_counts(run, content, None);
        let suit_fits =
            !RunBot::is_weak(hand, suit) && (!active.rules.require_ouvert || suit.ouvert());
        if !null_fits && !suit_fits {
            if let Some(action) = Self::throw_in(run, content) {
                return Some(action);
            }
        }
        let take_null = if active.rules.require_ouvert {
            !suit.ouvert() || null_ev > suit_ev
        } else {
            match (null_fits, suit_fits) {
                (true, false) => true,
                (false, true) => false,
                _ => null_ev > suit_ev,
            }
        };
        Some(RunAction::Declare(if take_null { null } else { suit }))
    }

    /// The game the bot declares after picking up the skat, with the two
    /// cards it discards for it, from the twelve cards (also once discarded,
    /// so that discard and declaration agree). For every character (ADR
    /// 0012) the better expected score of two candidates
    /// ([`suit_expected_score`], [`null_expected_score`]):
    ///
    /// - the suit game or grand the declarer heuristic picks for the twelve
    ///   cards ([`DeclarerHeuristic::choose_game_after_skat`]), discarding
    ///   for it;
    /// - Null, discarding the two most exposed cards, as Null Ouvert if no
    ///   exposed card is left.
    ///
    /// The suit game or grand carries the announcement the ten cards kept
    /// allow ([`RunBot::announcement`], ADR 0013: also after the skat,
    /// without Hand). With "Der Angeber" it is played only if that is
    /// Ouvert, otherwise Null Ouvert.
    fn after_skat(run: &Run, content: &Content) -> Option<(Declaration, Vec<DealtCard>)> {
        let active = run.game()?;
        let game = &active.game;
        let cards = game.declarer_cards();
        let deck = game.deck_cards();
        let count = game.discard_count().max(2);
        let null_discard =
            Self::discard_n(cards, Declaration::null(NullRisk::Normal), deck, count)?;
        let null_hand: Vec<DealtCard> = cards
            .iter()
            .copied()
            .filter(|c| !null_discard.contains(c))
            .collect();
        let rest: Vec<DealtCard> = deck
            .iter()
            .copied()
            .filter(|c| !null_discard.contains(c))
            .collect();
        let exposed = null_unsafe_cards(&null_hand, &rest);
        let null = if exposed == 0 || active.rules.require_ouvert {
            Declaration::null(NullRisk::Ouvert)
        } else {
            Declaration::null(NullRisk::Normal)
        };
        let suit = DeclarerHeuristic.choose_game_after_skat(cards);
        let voids = holds_relic(run, CLEAR_CUT) && matches!(suit.game(), GameType::Suit(_));
        let suit_discard = if voids {
            Self::void_discard_n(cards, suit, deck, count)?
        } else {
            Self::discard_n(cards, suit, deck, count)?
        };
        let suit_hand: Vec<DealtCard> = cards
            .iter()
            .copied()
            .filter(|c| !suit_discard.contains(c))
            .collect();
        let suit = suit
            .with_risk(RunBot::announcement(&suit_hand, suit))
            .with_hand(false);
        let suit = Self::bold_announcement(run, suit, &suit_hand, cards, deck);
        if active.rules.require_ouvert {
            if suit.ouvert() {
                return Some((suit, suit_discard));
            }
            return Some((null, null_discard));
        }
        if !Self::null_counts(run, content, Some(&null_discard)) {
            return Some((suit, suit_discard));
        }
        let scoring = run.null_scoring(content);
        let null_ev = null_expected_score(&null_hand, exposed, null, &scoring);
        let base = content.run.base_value(suit.game()).unwrap_or(0);
        let suit_ev = suit_expected_score(
            &suit_hand,
            cards,
            deck,
            suit,
            base,
            active.rules.thresholds.win,
        );
        if null_ev > suit_ev {
            Some((null, null_discard))
        } else {
            Some((suit, suit_discard))
        }
    }

    /// Schneider angesagt or Ouvert after the skat for a relic that pays
    /// for it (relics v4, 07.11.2026): with "Große Klappe" (`big_mouth`, a
    /// won game with Schneider announced counts the Grundwert twice) and
    /// "Offenes Visier" (`open_visor`, Ouvert asks for Schneider only).
    /// Without either relic, for a weak hand ([`RunBot::is_weak`]) or a
    /// Null, `suit` is returned unchanged.
    ///
    /// The chance of Schneider is [`schneider_percent`]: the unseen cards
    /// dealt [`BOLD_SAMPLES`] times, each deal solved double-dummy. Below
    /// [`BOLD_MIN_PERCENT`] nothing is announced. Above it the bot takes the
    /// best expected Gewinnstufen (hundredths) of a simple model that
    /// counts the game as won whenever Schneider is not announced: plain
    /// `Spitzen + 1` plus the chance of Schneider; announced `k` levels
    /// (Spitzen, Spiel, Schneider, angesagt; Ouvert one more) won with
    /// chance `p`, doubled with "Große Klappe", lost `k` otherwise. With
    /// "Der Angeber" (always open) only Ouvert counts.
    fn bold_announcement(
        run: &Run,
        suit: Declaration,
        kept: &[DealtCard],
        cards: &[DealtCard],
        deck: &[DealtCard],
    ) -> Declaration {
        let Some(active) = run.game() else {
            return suit;
        };
        let rules = &active.rules;
        let big_mouth = holds_relic(run, BIG_MOUTH);
        let visor = holds_relic(run, OPEN_VISOR) && rules.thresholds.ouvert_schneider;
        let risk = suit.risk().unwrap_or(RiskLevel::Normal);
        if suit.is_null()
            || !(big_mouth || visor)
            || risk >= RiskLevel::Ouvert
            || !Self::may_make_schneider(kept, suit.game())
        {
            return suit;
        }
        let p = schneider_percent(
            suit.game(),
            kept,
            cards,
            deck,
            active.game.forehand(),
            rules.thresholds.schneider_limit,
            run.seed(),
        );
        if p < BOLD_MIN_PERCENT {
            return suit;
        }
        if rules.require_ouvert {
            return if visor {
                suit.with_risk(RiskLevel::Ouvert).with_hand(false)
            } else {
                suit
            };
        }
        let faces: Vec<Card> = cards.iter().filter_map(|c| c.face()).collect();
        let spitzen = i64::from(crate::rules::matadors(suit.game(), &faces).count);
        let double = if big_mouth { 2 } else { 1 };
        let announced = |levels: i64| p * levels * double - (100 - p) * levels;
        let mut best = (
            match risk {
                RiskLevel::SchneiderDeclared => announced(spitzen + 3),
                _ => 100 * (spitzen + 1) + p,
            },
            risk,
        );
        if big_mouth && risk < RiskLevel::SchneiderDeclared {
            let ev = announced(spitzen + 3);
            if ev > best.0 {
                best = (ev, RiskLevel::SchneiderDeclared);
            }
        }
        if visor {
            let ev = announced(spitzen + 4);
            if ev > best.0 {
                best = (ev, RiskLevel::Ouvert);
            }
        }
        if best.1 == risk {
            suit
        } else {
            suit.with_risk(best.1).with_hand(false)
        }
    }

    /// Cheap filter before [`schneider_percent`]: a suit game with six
    /// trumps or more, a grand with two jacks or more, and at most two side
    /// cards that may lose a trick.
    fn may_make_schneider(kept: &[DealtCard], game: GameType) -> bool {
        let strong = match game {
            GameType::Grand => count_rank(kept, Rank::Jack) >= 2,
            GameType::Suit(_) => trump_count(game, kept) >= 6,
            GameType::Null => false,
        };
        strong && side_losers(kept, game) <= 2
    }

    /// `count` discards for a suit game with "Kahlschlag" (`clear_cut`, +1
    /// level per side suit without a card after the skat, at most +2): the
    /// pair of [`RunBot::void_discard`], then for a third discard ("Drei
    /// drücken") the plain rule of [`RunBot::discard_n`].
    fn void_discard_n(
        cards: &[DealtCard],
        declaration: Declaration,
        deck: &[DealtCard],
        count: usize,
    ) -> Option<Vec<DealtCard>> {
        let pair = Self::discard(cards, |free| Self::void_discard(free, declaration, deck))?;
        let mut chosen = pair.to_vec();
        if count > 2 {
            let left: Vec<DealtCard> = cards
                .iter()
                .copied()
                .filter(|c| !chosen.contains(c))
                .collect();
            chosen.extend(Self::discard_n(&left, declaration, deck, count - 2)?);
        }
        Some(chosen)
    }

    /// The two discards that leave the most side suits void (counted up to
    /// the two "Kahlschlag" pays for) in a suit game: the plain choice of
    /// [`RunBot::discard_for`] if it leaves as many, else among the pairs
    /// without a trump that do, the one with the most Augen (they count for
    /// the declarer in the skat), the first such pair in hand order on a
    /// tie. Other games: the plain choice.
    fn void_discard(
        cards: &[DealtCard],
        declaration: Declaration,
        deck: &[DealtCard],
    ) -> Option<[DealtCard; 2]> {
        let plain = Self::discard_for(cards, declaration, deck)?;
        let GameType::Suit(trump) = declaration.game() else {
            return Some(plain);
        };
        let game = declaration.game();
        let voids_without = |pair: [DealtCard; 2]| -> usize {
            Suit::ALL
                .iter()
                .filter(|&&suit| suit != trump)
                .filter(|&&suit| {
                    !cards.iter().any(|c| {
                        !pair.contains(c)
                            && !crate::rules::is_trump(game, *c)
                            && c.face().is_some_and(|f| f.suit == suit)
                    })
                })
                .count()
                .min(CLEAR_CUT_MAX_VOIDS)
        };
        let plain_voids = voids_without(plain);
        let side: Vec<DealtCard> = cards
            .iter()
            .copied()
            .filter(|&c| !crate::rules::is_trump(game, c))
            .collect();
        let mut best: Option<(usize, u32, [DealtCard; 2])> = None;
        for (i, &a) in side.iter().enumerate() {
            for &b in &side[i + 1..] {
                let pair = [a, b];
                let key = (voids_without(pair), a.card_points() + b.card_points());
                if best.is_none_or(|(v, p, _)| key > (v, p)) {
                    best = Some((key.0, key.1, pair));
                }
            }
        }
        match best {
            Some((voids, _, pair)) if voids > plain_voids => Some(pair),
            _ => Some(plain),
        }
    }

    /// Whether a won Null would count anything here: declare Null on a copy
    /// of the run (after `discard` while discarding) and read the value
    /// preview, which holds every hook certain for a win. A factor 0 on
    /// the Spielwert makes it 0 ("Der Schwarze Peter" only halves it since
    /// 02.11.2026). If the probe cannot declare Null, it counts.
    fn null_counts(run: &Run, content: &Content, discard: Option<&[DealtCard]>) -> bool {
        let Some(active) = run.game() else {
            return true;
        };
        let ouvert = active.rules.require_ouvert;
        let mut probe = run.clone();
        let risk = match active.game.phase() {
            Phase::Declaring if ouvert => NullRisk::OuvertHand,
            Phase::Declaring => NullRisk::Hand,
            Phase::Discarding => {
                let Some(action) = discard.and_then(Self::discard_action) else {
                    return true;
                };
                if probe.apply(content, action).is_err() {
                    return true;
                }
                if ouvert {
                    NullRisk::Ouvert
                } else {
                    NullRisk::Normal
                }
            }
            Phase::DeclaringAfterSkat if ouvert => NullRisk::Ouvert,
            Phase::DeclaringAfterSkat => NullRisk::Normal,
            Phase::Playing | Phase::Finished(_) => return true,
        };
        if probe
            .apply_step(content, RunAction::Declare(Declaration::null(risk)))
            .is_err()
        {
            return true;
        }
        probe
            .value_preview(content)
            .is_none_or(|p| p.game_value > 0)
    }

    /// The action that discards `cards`: two, or three after drawing a card
    /// with the skat ("Drei drücken").
    fn discard_action(cards: &[DealtCard]) -> Option<RunAction> {
        match *cards {
            [a, b] => Some(RunAction::Discard([a.id, b.id])),
            [a, b, c] => Some(RunAction::DiscardThree([a.id, b.id, c.id])),
            _ => None,
        }
    }

    /// `count` discards for `declaration` (two, or three with "Drei
    /// drücken"): the two of [`RunBot::discard_for`], then for a third the
    /// first of the two it would discard from the cards left.
    fn discard_n(
        cards: &[DealtCard],
        declaration: Declaration,
        deck: &[DealtCard],
        count: usize,
    ) -> Option<Vec<DealtCard>> {
        let mut chosen: Vec<DealtCard> = Vec::with_capacity(count);
        let mut left = cards.to_vec();
        while chosen.len() < count {
            let pair = Self::discard_for(&left, declaration, deck)?;
            let take = (count - chosen.len()).min(2);
            for card in pair.into_iter().take(take) {
                left.retain(|&c| c != card);
                chosen.push(card);
            }
        }
        Some(chosen)
    }

    /// The two discards for `declaration` from the twelve cards: the declarer
    /// heuristic's choice (the most exposed cards in a Null), never a card
    /// that must not be discarded while others are left.
    #[must_use]
    pub fn discard_for(
        cards: &[DealtCard],
        declaration: Declaration,
        deck: &[DealtCard],
    ) -> Option<[DealtCard; 2]> {
        let declarer = DeclarerHeuristic;
        Self::discard(cards, |cards| {
            if declaration.is_null() {
                declarer.choose_null_discard(cards, deck)
            } else {
                declarer.choose_discard(cards, declaration.game())
            }
        })
    }

    /// The two discards: chosen by `choose` among the cards that may be
    /// discarded ("Ramponiert" stays in the hand); only if fewer than two
    /// are left, the rest is filled from the others.
    fn discard(
        hand: &[DealtCard],
        choose: impl Fn(&[DealtCard]) -> Option<[DealtCard; 2]>,
    ) -> Option<[DealtCard; 2]> {
        let free: Vec<DealtCard> = hand.iter().copied().filter(|c| !c.no_discard()).collect();
        if free.len() >= 2 {
            return choose(&free);
        }
        let mut chosen = free;
        let bound: Vec<DealtCard> = hand.iter().copied().filter(|c| c.no_discard()).collect();
        let fill = choose(&bound).map_or_else(|| bound.clone(), |pair| pair.to_vec());
        chosen.extend(fill.into_iter().take(2 - chosen.len()));
        match chosen[..] {
            [a, b] => Some([a, b]),
            _ => None,
        }
    }

    /// Skat swaps ("Die Oma", "Der Opa"): give the cheapest side card for
    /// the card the run offers first.
    fn take_from_skat(run: &Run, content: &Content) -> Option<RunAction> {
        let active = run.game()?;
        let game = &active.game;
        let trump = game.declaration()?.game();
        // A jack or an ace from the skat only hurts a Null.
        if trump == GameType::Null {
            return None;
        }
        let give = game
            .hand(Seat::DECLARER)
            .iter()
            .copied()
            .filter(|&c| !crate::rules::is_trump(trump, c) && !c.no_discard())
            .min_by_key(|c| (c.card_points(), c.face().map(|f| f.rank)))?;
        let action = RunAction::TakeFromSkat {
            give: give.id,
            take: None,
        };
        run.accepts(content, action.clone()).then_some(action)
    }

    /// Schnäpse right before the declaration after the skat, see the
    /// module docs: "Blue Curaçao" on the cheapest side card of a suit game,
    /// then "Doppelkorn" (not weak) or "Magenbitter" (weak).
    fn schnapps_before_declaring(
        run: &Run,
        content: &Content,
        declaration: Declaration,
    ) -> Option<RunAction> {
        if declaration.is_null() {
            return None;
        }
        let hand = run.game()?.game.hand(Seat::DECLARER);
        if let GameType::Suit(trump) = declaration.game() {
            let side = hand
                .iter()
                .copied()
                .filter(|&c| !crate::rules::is_trump(declaration.game(), c))
                .filter(|c| c.face().is_some_and(|f| f.rank != Rank::Jack))
                .min_by_key(|c| (c.card_points(), c.face().map(|f| f.rank), c.id));
            if let Some(card) = side {
                let chalk = run
                    .consumables()
                    .iter()
                    .enumerate()
                    .filter(|(_, id)| {
                        content
                            .consumable(id)
                            .is_some_and(|d| d.action == ConsumableAction::ChalkCard)
                    })
                    .map(|(slot, _)| RunAction::UseConsumable {
                        slot,
                        target: Some(card.id),
                        suit: Some(trump),
                    })
                    .find(|action| run.accepts(content, action.clone()));
                if chalk.is_some() {
                    return chalk;
                }
            }
        }
        // One "Doppelkorn" or "Magenbitter" per game.
        if !run.pending_next_game().is_empty() {
            return None;
        }
        let wanted = if RunBot::is_weak(hand, declaration) {
            NextGameEffect::SingleLoss
        } else {
            NextGameEffect::Multiply(2)
        };
        Self::usable(
            run,
            content,
            |a| match (a, wanted) {
                (
                    ConsumableAction::NextGame(NextGameEffect::Multiply(_)),
                    NextGameEffect::Multiply(_),
                ) => true,
                (ConsumableAction::NextGame(e), w) => *e == w,
                _ => false,
            },
            None,
        )
    }

    /// The rescue Schnäpse between two games: "Klarer" with the account
    /// below 0, "Absacker" before the last game of a pub short of its
    /// target, "Eierlikör" whenever the run accepts it (account below half
    /// the target). The run refuses each when it would do nothing, so none
    /// of them is drunk twice for the same reason.
    fn rescue_schnapps(run: &Run, content: &Content) -> Option<RunAction> {
        let series = run.series();
        if series.score < 0 {
            if let Some(action) = Self::usable(
                run,
                content,
                |a| matches!(a, ConsumableAction::ClearAccount),
                None,
            ) {
                return Some(action);
            }
        }
        if series.games_left() == 1 && !series.target_reached() {
            if let Some(action) = Self::usable(
                run,
                content,
                |a| matches!(a, ConsumableAction::ExtraGames(_)),
                None,
            ) {
                return Some(action);
            }
        }
        Self::usable(
            run,
            content,
            |a| matches!(a, ConsumableAction::NextGameIfBehind(_)),
            None,
        )
    }

    /// The cheapest way to throw a weak hand in, if any: an Einmischen
    /// left, "Kräuter", "Doppelkorn" (the new game counts double),
    /// "Ein Bier" of older content for one more Einmischen. Einmischen
    /// costs no game, so the last game of a series is no exception.
    fn throw_in(run: &Run, content: &Content) -> Option<RunAction> {
        if run.redeals_left(content) > 0 && run.accepts(content, RunAction::Redeal) {
            return Some(RunAction::Redeal);
        }
        if let Some(action) = Self::usable(
            run,
            content,
            |a| matches!(a, ConsumableAction::Redeal),
            None,
        ) {
            return Some(action);
        }
        if let Some(action) = Self::usable(
            run,
            content,
            |a| matches!(a, ConsumableAction::RedealNextGame(_)),
            None,
        ) {
            return Some(action);
        }
        Self::usable(
            run,
            content,
            |a| matches!(a, ConsumableAction::ExtraRedeals(_)),
            None,
        )
    }

    /// Schnäpse at the route choice: "Wegbier" at once, "Schwenker" when
    /// no way has what the bot wants most.
    fn route_schnapps(run: &Run, content: &Content) -> Option<RunAction> {
        if let Some(action) = Self::usable(
            run,
            content,
            |a| matches!(a, ConsumableAction::NextPub { .. }),
            None,
        ) {
            return Some(action);
        }
        let route = run.route()?;
        let has = |category: StopCategory| {
            route
                .options
                .iter()
                .any(|o| o.stops.iter().any(|s| s.category == category))
        };
        let wants_upgrade = Self::cursed_card(run, content).is_some()
            && !has(StopCategory::Upgrade)
            && !has(StopCategory::Rest);
        if wants_upgrade {
            return Self::usable(
                run,
                content,
                |a| matches!(a, ConsumableAction::RerollRoutes),
                None,
            );
        }
        None
    }

    /// "Spelunkenkenner": the malus the bot likes best on a Spelunke way,
    /// if it is not the chosen one yet (Schnapsverbot, then Klebrige
    /// Karten, Kein Einmischen, Kein Skat).
    fn malus_choice(run: &Run) -> Option<RunAction> {
        const LIKED: [&str; 4] = ["schnapps_locked", "cursed_cards", "no_redeal", "hand_only"];
        let route = run.route()?;
        route.options.iter().enumerate().find_map(|(option, o)| {
            let spelunke = o.spelunke.as_ref()?;
            let best = spelunke
                .malus_options
                .iter()
                .enumerate()
                .min_by_key(|(_, m)| LIKED.iter().position(|l| *l == m.as_str()).unwrap_or(9))?;
            (*best.1 != spelunke.malus).then_some(RunAction::ChooseMalus {
                option,
                malus: best.0,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cards::CardKind;

    fn hand(cards: &str) -> Vec<DealtCard> {
        cards
            .split_whitespace()
            .zip(0u32..)
            .map(|(s, id)| DealtCard::plain(id, s.parse::<CardKind>().expect("card")))
            .collect()
    }

    fn suit(s: Suit) -> Declaration {
        Declaration::suit(s, RiskLevel::Normal)
    }

    #[test]
    fn pass_rule_follows_trumps_and_aces() {
        let hearts = suit(Suit::Hearts);
        let five_two_aces = hand("CJ HA HT HK H9 SA ST S9 D7 D8");
        assert!(!RunBot::is_weak(&five_two_aces, hearts));
        let five_one_ace = hand("CJ HA HT HK H9 SK ST S9 D7 D8");
        assert!(RunBot::is_weak(&five_one_ace, hearts));
        let six = hand("CJ SJ HT HK H9 H8 ST S9 D7 D8");
        assert!(!RunBot::is_weak(&six, hearts));
        let grand = Declaration::grand(RiskLevel::Normal);
        assert!(!RunBot::is_weak(
            &hand("CJ DJ CA SA S9 S8 H7 H8 D7 D8"),
            grand
        ));
        assert!(RunBot::is_weak(
            &hand("CJ SJ HJ CA S9 S8 H7 H8 D7 D8"),
            grand
        ));
    }

    #[test]
    fn announcements_are_conservative() {
        let clubs = suit(Suit::Clubs);
        // Eight top trumps and two aces: Ouvert.
        let ouvert = hand("CJ SJ HJ DJ CA CT CK CQ HA SA");
        assert_eq!(RunBot::announcement(&ouvert, clubs), RiskLevel::Ouvert);
        // Seven trumps with the top three jacks and standing aces: Schneider.
        let schneider = hand("CJ SJ HJ CA CT CK C9 HA SA ST");
        assert_eq!(
            RunBot::announcement(&schneider, clubs),
            RiskLevel::SchneiderDeclared
        );
        // A bare king outside: no announcement.
        let bare = hand("CJ SJ HJ CA CT CK C9 HA SA SK");
        assert_eq!(RunBot::announcement(&bare, clubs), RiskLevel::Normal);
        // Eight trumps from the top three jacks and one loser: Schneider.
        let one_loser = hand("CJ SJ HJ CA CT CK C9 C8 SA SK");
        assert_eq!(
            RunBot::announcement(&one_loser, clubs),
            RiskLevel::SchneiderDeclared
        );
        let grand = Declaration::grand(RiskLevel::Normal);
        let jacks = hand("CJ SJ HJ DJ CA CT SA ST HA DA");
        assert_eq!(RunBot::announcement(&jacks, grand), RiskLevel::Ouvert);
        let three = hand("CJ SJ HJ CA CT SA ST HA DA D7");
        assert_eq!(
            RunBot::announcement(&three, grand),
            RiskLevel::SchneiderDeclared
        );
        let two_losers = hand("CJ SJ HJ CA CT SA ST HA D8 D7");
        assert_eq!(RunBot::announcement(&two_losers, grand), RiskLevel::Normal);
        assert!(RunBot::says_re(&jacks, GameType::Grand));
        assert!(!RunBot::says_re(&bare[..8], GameType::Grand));
    }

    fn open_shop(content: &Content, coins: u32) -> Run {
        let (mut run, _) = Run::new(content, 9).expect("start");
        run.coins = coins;
        run.game = None;
        run.phase = RunPhase::Shop;
        run.shop = Some(run.draw_offer(content, 0));
        run
    }

    fn offer(run: &mut Run, relics: &[&str], consumables: &[&str]) {
        let shop = run.shop.as_mut().expect("shop");
        shop.relics = relics.iter().map(|r| Some(RelicId::new(r))).collect();
        shop.consumables = consumables
            .iter()
            .map(|c| Some(ConsumableId::new(c)))
            .collect();
    }

    #[test]
    fn shop_swaps_the_weakest_relic_for_a_clearly_better_one() {
        // With every slot full (5 here, 8 in run.ron since 05.11.2026).
        let mut content = Content::load().expect("content");
        content.run.relic_slots = Some(5);
        let mut run = open_shop(&content, 6);
        run.relics = ["tour", "card_counter", "skat_redraw", "virgin", "concede"]
            .iter()
            .map(|r| RelicId::new(r))
            .collect();
        offer(&mut run, &["old_lady"], &[]);
        // Alte Dame costs 9; selling Der Zähler brings 2: 8 is not enough.
        assert_eq!(RunBot.shop_action(&run, &content), RunAction::LeaveShop);
        run.coins = 7;
        let replace = RunAction::ReplaceRelic {
            slot: 0,
            replace: 1,
        };
        assert_eq!(RunBot.shop_action(&run, &content), replace);
        run.apply(&content, replace).expect("replace");
        assert_eq!(run.relics().len(), 5);
        assert!(run.relics().contains(&RelicId::new("old_lady")));
        assert!(!run.relics().contains(&RelicId::new("card_counter")));
        // Not clearly better: the Kiebitz (3) for the Spickzettel (1).
        let mut run = open_shop(&content, 50);
        run.relics = [
            "crib_sheet",
            "every_bit_counts",
            "old_lady",
            "concede",
            "tour",
        ]
        .iter()
        .map(|r| RelicId::new(r))
        .collect();
        offer(&mut run, &["kibitzer"], &[]);
        assert!(!matches!(
            RunBot.shop_action(&run, &content),
            RunAction::SellRelic(_) | RunAction::ReplaceRelic { .. }
        ));
    }

    #[test]
    fn shop_buys_a_schnaps_and_cleanses_a_cursed_card() {
        let content = Content::load().expect("content");
        let mut run = open_shop(&content, 10);
        run.shop.as_mut().expect("shop").card_offers.clear();
        offer(&mut run, &[], &["herbal", "road_beer"]);
        // Wegbier (2) before Kräuter (1).
        assert_eq!(
            RunBot.shop_action(&run, &content),
            RunAction::BuyConsumable(1)
        );
        // "Karte reinigen" with a cursed card, before anything else.
        run.edition
            .add_mod(&content, 9, &CardModId::new("battered"), 5)
            .expect("curse");
        assert_eq!(
            RunBot.shop_action(&run, &content),
            RunAction::CleanseCard(9)
        );
        run.apply(&content, RunAction::CleanseCard(9))
            .expect("cleanse");
        assert!(run.edition().get(9).is_some_and(|c| c.mods.is_empty()));
    }

    #[test]
    fn shop_buys_zinken_on_bare_cards_and_hires_the_painter() {
        let content = Content::load().expect("content");
        let mut run = open_shop(&content, 7);
        offer(&mut run, &[], &[]);
        let kreuz_ass = 31;
        let offer_on = |card: u32, card_mod: &str, price: u32| {
            Some(super::super::zinken::CardOffer {
                card,
                card_mod: CardModId::new(card_mod),
                value: 1,
                price,
                bargain: None,
            })
        };
        run.edition
            .add_mod(&content, 0, &CardModId::new("rank"), 1)
            .expect("rank");
        {
            let shop = run.shop.as_mut().expect("shop");
            // Siegel would replace Winkel on card 0: never; Kerbe (1) on a
            // bare card loses to Aufdruck (2).
            shop.card_offers = vec![
                offer_on(0, "trump_bonus", 3),
                offer_on(5, "last_trick", 3),
                offer_on(kreuz_ass, "points", 3),
            ];
        }
        assert_eq!(
            RunBot.shop_action(&run, &content),
            RunAction::BuyCardOffer(2)
        );
        run.apply(&content, RunAction::BuyCardOffer(2))
            .expect("buy");
        assert_eq!(run.coins(), 4);
        // Kerbe next; then 1 Groschen is left, no Kartenmaler.
        assert_eq!(
            RunBot.shop_action(&run, &content),
            RunAction::BuyCardOffer(1)
        );
        run.shop.as_mut().expect("shop").card_offers.clear();
        run.coins = content.run.shop.zinken.painter_price + PAINTER_RESERVE;
        let hire = RunBot.shop_action(&run, &content);
        let RunAction::HirePainter(card) = hire else {
            panic!("painter: {hire:?}");
        };
        assert!(run.edition().get(card).is_some_and(|c| c.mods.is_empty()));
        run.apply(&content, hire).expect("hire");
        let choice = RunBot.shop_action(&run, &content);
        assert!(matches!(choice, RunAction::ChoosePaint(_)));
        run.apply(&content, choice).expect("paint");
        assert!(run.painter().is_none());
        assert!(run.edition().get(card).is_some_and(|c| c.mods.len() == 1));
        // Once per Tresen.
        assert!(!matches!(
            RunBot.shop_action(&run, &content),
            RunAction::HirePainter(_)
        ));
    }

    #[test]
    fn shop_rerolls_only_with_plenty_of_groschen() {
        let content = Content::load().expect("content");
        let cost = content.run.economy.reroll_cost;
        let mut run = open_shop(&content, cost + REROLL_RESERVE);
        offer(&mut run, &[], &[]);
        assert_eq!(RunBot.shop_action(&run, &content), RunAction::Reroll);
        run.coins -= 1;
        assert_eq!(RunBot.shop_action(&run, &content), RunAction::LeaveShop);
        run.coins += 1;
        // Once per visit.
        run.shop.as_mut().expect("shop").rerolls = 1;
        assert_eq!(RunBot.shop_action(&run, &content), RunAction::LeaveShop);
    }

    #[test]
    fn weak_hands_take_the_cheapest_way_out() {
        let content = Content::load().expect("content");
        let mut seed = 0;
        let mut run = loop {
            let (run, _) = Run::new(&content, seed).expect("start");
            let game = &run.game().expect("game").game;
            let hand = game.hand(Seat::DECLARER);
            if RunBot::is_weak(hand, DeclarerHeuristic.choose_declaration(hand))
                && null_unsafe_cards(hand, game.deck_cards()) > NULL_MAX_EXPOSED
            {
                break run;
            }
            seed += 1;
        };
        run.consumables = vec![
            ConsumableId::new("double_grain"),
            ConsumableId::new("herbal"),
        ];
        let next = |run: &Run| RunBot.next_action(run, &content).expect("action");
        // The game's own Einmischen first.
        assert_eq!(next(&run), RunAction::Redeal);
        run.series.redeals_in_game = 99;
        assert_eq!(
            next(&run),
            RunAction::UseConsumable {
                slot: 1,
                target: None,
                suit: None
            }
        );
        run.consumables.pop();
        assert_eq!(
            next(&run),
            RunAction::UseConsumable {
                slot: 0,
                target: None,
                suit: None
            }
        );
        // Nothing left to throw it in: the hand is played, with the skat.
        run.consumables.clear();
        assert_eq!(next(&run), RunAction::PickUpSkat);
    }

    /// Hand only for a very strong hand; otherwise pick up the skat, then
    /// discard and declare the game chosen for all twelve cards: the same
    /// choice before and after the discard. Both characters play suit
    /// games, grand and Null (ADR 0012); Null after the skat is Ouvert
    /// exactly when no exposed card is left.
    #[test]
    fn the_bot_sees_the_skat_before_it_chooses_the_game() {
        let content = Content::load().expect("content");
        for character in crate::content::Character::ALL {
            let start = super::super::RunStart {
                character,
                ..super::super::RunStart::default()
            };
            let (mut hand_games, mut skat_games, mut nulls) = (0, 0, 0);
            for seed in 0..80 {
                let (mut run, _) = Run::new_with(&content, seed, &start).expect("start");
                let next = |run: &Run| RunBot.next_action(run, &content).expect("action");
                match next(&run) {
                    RunAction::Declare(d) => {
                        assert!(d.hand(), "seed {seed}: {d:?} before the skat");
                        hand_games += 1;
                    }
                    RunAction::PickUpSkat => {
                        run.apply(&content, RunAction::PickUpSkat).expect("pick up");
                        let (planned, discard) =
                            RunBot::after_skat(&run, &content).expect("choice");
                        let ids = [discard[0].id, discard[1].id];
                        assert_eq!(next(&run), RunAction::Discard(ids), "seed {seed}");
                        run.apply(&content, RunAction::Discard(ids))
                            .expect("discard");
                        assert_eq!(next(&run), RunAction::Declare(planned), "seed {seed}");
                        assert!(!planned.hand());
                        if planned.is_null() {
                            let game = &run.game().expect("game").game;
                            let skat = game.skat();
                            let rest: Vec<DealtCard> = game
                                .deck_cards()
                                .iter()
                                .copied()
                                .filter(|c| !skat.contains(c))
                                .collect();
                            let exposed = null_unsafe_cards(game.hand(Seat::DECLARER), &rest);
                            assert_eq!(planned.ouvert(), exposed == 0, "seed {seed}");
                            nulls += 1;
                        }
                        run.apply(&content, RunAction::Declare(planned))
                            .expect("declare");
                        skat_games += 1;
                    }
                    _ => {} // thrown in
                }
            }
            assert!(
                skat_games > 10 && hand_games < skat_games && nulls > 0 && nulls < skat_games,
                "{character}: {hand_games} {skat_games} {nulls}"
            );
        }
    }

    /// The expected scores behind the choice: Null with one exposed card
    /// and three sevens beats a suit game with five trumps; a suit game
    /// with seven trumps beats Null with three exposed cards.
    #[test]
    fn expected_scores_compare_null_with_suit_games() {
        let formula = NullScoring::Formula(crate::rules::NullFormula {
            base_value: 23,
            per_seven_ducked: 1,
            never_followed: 2,
            short_circuit: 3,
            short_circuit_tricks: 5,
            short_circuit_points: 61,
        });
        let null = Declaration::null(NullRisk::Normal);
        let sevens = hand("C7 S7 H7 C8 S8 H9 D8 D9 DT DQ");
        // 76 % of 23 × 4.3, minus 24 % of 23 (a lost game counts once
        // since 04.11.2026).
        assert_eq!(null_expected_score(&sevens, 1, null, &formula), 6964);
        assert!(null_expected_score(&sevens, 7, null, &formula) < 0);
        // Fixed: 76 % of 23, minus 24 % of 23.
        assert_eq!(
            null_expected_score(&sevens, 1, null, &NullScoring::Fixed),
            1196
        );
        let hearts = suit(Suit::Hearts);
        let deck: Vec<DealtCard> = crate::deck::Edition::standard()
            .cards()
            .iter()
            .filter_map(|c| match c.kind {
                CardKind::Normal(card) => Some(DealtCard::plain(c.id, CardKind::Normal(card))),
                CardKind::Blank => None,
            })
            .collect();
        let five = hand("HJ HA HT H9 H8 SA S9 C8 D7 D8");
        let seven = hand("CJ HJ HA HT HK H9 H8 SA S9 C8");
        // Five trumps and two aces, ohne 2: 52 % of 10 × 3, minus 48 % of 30.
        assert_eq!(
            suit_expected_score(&five, &five, &deck, hearts, 10, 61),
            120
        );
        assert!(suit_expected_score(&seven, &seven, &deck, hearts, 10, 61) > 0);
        assert!(
            null_expected_score(&sevens, 1, null, &formula)
                > suit_expected_score(&five, &five, &deck, hearts, 10, 61)
        );
        assert!(
            suit_expected_score(&seven, &seven, &deck, hearts, 10, 61)
                > null_expected_score(&sevens, 3, null, &formula)
        );
        // A high stake makes the suit game worse and Null the better
        // choice; a lowered threshold keeps the measured table.
        let staked = suit_expected_score(&seven, &seven, &deck, hearts, 10, 61 + 30);
        assert!(staked < suit_expected_score(&seven, &seven, &deck, hearts, 10, 61));
        assert!(staked < null_expected_score(&sevens, 3, null, &formula));
        assert_eq!(
            suit_expected_score(&five, &five, &deck, hearts, 10, 58),
            suit_expected_score(&five, &five, &deck, hearts, 10, 61)
        );
    }

    /// "Der Schieber": Null Hand only with at most two exposed cards.
    #[test]
    fn forced_hand_takes_null_only_with_few_exposed_cards() {
        let content = Content::load().expect("content");
        for character in crate::content::Character::ALL {
            let start = super::super::RunStart {
                character,
                ..super::super::RunStart::default()
            };
            for seed in 0..80 {
                let (mut run, _) = Run::new_with(&content, seed, &start).expect("start");
                run.game.as_mut().expect("game").rules.require_hand = true;
                let game = &run.game().expect("game").game;
                let exposed = null_unsafe_cards(game.hand(Seat::DECLARER), game.deck_cards());
                if let Some(RunAction::Declare(d)) = RunBot.next_action(&run, &content) {
                    assert!(d.hand(), "seed {seed}");
                    if d.is_null() {
                        assert!(exposed <= NULL_HAND_MAX_EXPOSED, "seed {seed}: {exposed}");
                    }
                }
            }
        }
    }

    #[test]
    fn value_tables_know_every_content_item() {
        let content = Content::load().expect("content");
        for id in content.relics.keys() {
            assert!(
                RELIC_VALUES
                    .iter()
                    .chain(LEGENDARY_VALUES.iter())
                    .chain(NIGHT_VALUES.iter())
                    .any(|(name, _)| *name == id.as_str()),
                "relic {id} has no bot value"
            );
        }
        for id in content.consumables.keys() {
            assert!(
                CONSUMABLE_VALUES
                    .iter()
                    .any(|(name, _)| *name == id.as_str()),
                "consumable {id} has no bot value"
            );
        }
    }
    /// A run with `relics` on a constructed deal (the player forehand), the
    /// skat picked up: the bot's planned declaration and discard.
    fn plan_after_skat(
        relics: &[&str],
        hands: [&str; 3],
        skat: &str,
    ) -> (Declaration, Vec<DealtCard>) {
        use super::super::content_tests::{content, fixed_deal, relic_run};
        let content = content();
        let mut run = relic_run(&content, relics);
        fixed_deal(&mut run, &content, hands, skat);
        run.apply(&content, RunAction::PickUpSkat).expect("pick up");
        RunBot::after_skat(&run, &content).expect("plan")
    }

    /// A grand mit 4 with the four clubs from the ace and two aces: two
    /// side cards may lose a trick by the plain rules, so the bot plays it
    /// without announcement; double-dummy the declarer takes every trick.
    const SCHNEIDER_HAND: [&str; 3] = [
        "CJ SJ HJ DJ CA CT CK CQ SA HA",
        "C9 C8 HT HK HQ H9 H8 SK SQ S9",
        "C7 H7 ST S8 S7 DA DT DK DQ D9",
    ];
    const SCHNEIDER_SKAT: &str = "D8 D7";

    #[test]
    fn big_mouth_and_open_visor_make_the_bot_announce() {
        let grand = |risk| Declaration::grand(risk).with_hand(false);
        let (plain, plain_discard) = plan_after_skat(&[], SCHNEIDER_HAND, SCHNEIDER_SKAT);
        assert_eq!(plain, grand(RiskLevel::Normal));
        let (big, big_discard) = plan_after_skat(&["big_mouth"], SCHNEIDER_HAND, SCHNEIDER_SKAT);
        assert_eq!(big, grand(RiskLevel::SchneiderDeclared));
        assert_eq!(big_discard, plain_discard);
        let (visor, _) = plan_after_skat(&["open_visor"], SCHNEIDER_HAND, SCHNEIDER_SKAT);
        assert_eq!(visor, grand(RiskLevel::Ouvert));
        let (both, _) =
            plan_after_skat(&["big_mouth", "open_visor"], SCHNEIDER_HAND, SCHNEIDER_SKAT);
        assert_eq!(both, grand(RiskLevel::Ouvert));
        // A hand that does not make Schneider stays plain: STRONG of the
        // content tests without two of its trumps.
        let middling = [
            "CJ HJ DA DK DQ D9 D7 HA S7 C8",
            "SJ DJ HT HK HQ H9 H8 H7 SA ST",
            "DT SK SQ S9 S8 CA CT CK CQ C9",
        ];
        let (decl, _) = plan_after_skat(&["big_mouth", "open_visor"], middling, "D8 C7");
        assert!(!decl.schneider_declared(), "{decl:?}");
    }

    #[test]
    fn schneider_percent_is_deterministic_and_reads_the_cards() {
        let deck: Vec<DealtCard> = hand(
            "CJ SJ HJ DJ CA CT CK SA ST H7 HA HT HK HQ H9 H8 SK SQ S9 S8 \
             CQ C9 C8 C7 DA DT DK DQ D9 S7 D8 D7",
        );
        let cards: Vec<DealtCard> = deck[..10].iter().chain(&deck[30..]).copied().collect();
        let kept = &deck[..10];
        let clubs = GameType::Suit(Suit::Clubs);
        let p = schneider_percent(clubs, kept, &cards, &deck, Seat::DECLARER, 30, 7);
        assert!(p >= BOLD_MIN_PERCENT, "{p}");
        assert_eq!(
            p,
            schneider_percent(clubs, kept, &cards, &deck, Seat::DECLARER, 30, 7)
        );
        // A defender's hand in the declarer's seat: no trump, no Schneider.
        let weak: Vec<DealtCard> = deck[10..20].iter().chain(&deck[30..]).copied().collect();
        assert_eq!(
            schneider_percent(clubs, &deck[10..20], &weak, &deck, Seat::DECLARER, 30, 7),
            0
        );
    }

    #[test]
    fn clear_cut_makes_the_bot_discard_for_voids() {
        // Clubs with nine trumps, HA, S7 and from the skat D7 D8: the plain
        // rule never discards an ace and leaves one void suit (spades);
        // with "Kahlschlag" HA and S7 go, hearts and spades are void.
        let hands = [
            "CJ SJ HJ CT CK CQ C9 C8 HA S7",
            "DJ CA C7 HT HK HQ H9 H8 H7 SA",
            "ST SK SQ S9 S8 DA DT DK DQ D9",
        ];
        let faces = |cards: &[DealtCard]| -> Vec<String> {
            let mut v: Vec<String> = cards
                .iter()
                .filter_map(|c| c.face())
                .map(|f| f.to_string())
                .collect();
            v.sort();
            v
        };
        let (plain, plain_discard) = plan_after_skat(&[], hands, "D7 D8");
        assert_eq!(plain.game(), GameType::Suit(Suit::Clubs));
        assert!(!faces(&plain_discard).contains(&"HA".to_string()));
        let (decl, discard) = plan_after_skat(&["clear_cut"], hands, "D7 D8");
        assert_eq!(decl.game(), GameType::Suit(Suit::Clubs));
        assert_eq!(faces(&discard), vec!["HA".to_string(), "S7".to_string()]);
        // Not a suit game: the plain discard.
        let deck = hand("CJ SJ HJ DJ CA C7 HA H7 S7 S8 D7 D8");
        let grand = Declaration::grand(RiskLevel::Normal);
        assert_eq!(
            RunBot::void_discard(&deck, grand, &deck),
            RunBot::discard_for(&deck, grand, &deck)
        );
    }
}

#[cfg(test)]
mod redeal_tests {
    use super::*;
    use crate::content::Content;

    #[test]
    fn the_redeal_threshold_falls_with_fewer_einmischen() {
        assert_eq!(RunBot::redeal_threshold(1), REDEAL_STOP[0]);
        assert_eq!(RunBot::redeal_threshold(3), REDEAL_STOP[2]);
        assert!(RunBot::redeal_threshold(2) < RunBot::redeal_threshold(3));
        // Beyond the table the last value holds.
        assert_eq!(RunBot::redeal_threshold(9), REDEAL_STOP[4]);
    }

    /// A deal that is not weak but scores below the threshold is thrown
    /// in while an Einmischen is left, and played without one.
    #[test]
    fn the_bot_throws_in_a_mediocre_hand_only_with_einmischen_left() {
        let content = Content::load().expect("content");
        let grand = content.run.base_value(GameType::Grand).unwrap_or(0);
        let found = (0..500).find_map(|seed| {
            let (run, _) = Run::new(&content, seed).ok()?;
            let active = run.game()?;
            let hand = active.game.hand(Seat::DECLARER);
            let deck = active.game.deck_cards();
            let suit = DeclarerHeuristic.choose_declaration(hand);
            let scoring = run.null_scoring(&content);
            let score = ten_card_score(hand, deck, grand, Some(&scoring));
            let left = run.redeals_left(&content);
            let mediocre = !RunBot::is_weak(hand, suit)
                && !suit.hand()
                && null_unsafe_cards(hand, deck) > 0
                && score < RunBot::redeal_threshold(left);
            mediocre.then_some(run)
        });
        let mut run = found.expect("a mediocre deal in 500 seeds");
        assert_eq!(RunBot.next_action(&run, &content), Some(RunAction::Redeal));
        run.series.redeals_in_game = 99;
        run.series.extra_redeals = 0;
        assert_eq!(run.redeals_left(&content), 0);
        assert_ne!(RunBot.next_action(&run, &content), Some(RunAction::Redeal));
    }
}

#[cfg(test)]
mod redeal_probe {
    use super::*;
    use crate::cards::CardKind;
    use crate::content::Character;
    use crate::content::Content;

    /// Prints the distribution of [`ten_card_score`] over random deals and
    /// the optimal-stopping values (balance tool, not a check).
    #[test]
    #[ignore = "balance probe, prints numbers"]
    fn ten_card_score_distribution() {
        let content = Content::load().expect("content");
        for character in [Character::Declarer, Character::NullPlayer] {
            let def = content.character(character).expect("character");
            let scoring = def.null_scoring.clone().unwrap_or(NullScoring::Fixed);
            let deck: Vec<DealtCard> = crate::deck::Edition::standard()
                .cards()
                .iter()
                .filter_map(|c| match c.kind {
                    CardKind::Normal(card) => Some(DealtCard::plain(c.id, CardKind::Normal(card))),
                    CardKind::Blank => None,
                })
                .collect();
            let mut rng = crate::rng::Rng::from_seed(7);
            let mut scores = Vec::new();
            for _ in 0..200_000 {
                let mut d = deck.clone();
                rng.shuffle(&mut d);
                scores.push(ten_card_score(&d[..10], &deck, 18, Some(&scoring)));
            }
            scores.sort_unstable();
            let n = scores.len();
            let q: Vec<i64> = [5, 10, 25, 50, 75, 90, 95]
                .iter()
                .map(|p| scores[n * p / 100])
                .collect();
            let mut v = scores.iter().sum::<i64>() / i64::try_from(n).expect("n");
            let mut stops = vec![v];
            for _ in 0..4 {
                let t = v;
                v = scores.iter().map(|&s| s.max(t)).sum::<i64>() / i64::try_from(n).expect("n");
                stops.push(v);
            }
            println!("{character:?} quantiles {q:?} stopping V0.. {stops:?}");
        }
    }
}
