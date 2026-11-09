//! What the bot thinks things are worth: relics, consumables and Zinken
//! (buy values, not calibrated), its Tresen reserves, and the scores that
//! add the bonus of the player's build direction.

use crate::content::{BuildTag, CardModId, ConsumableId, Content, RelicId};

use super::Run;

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
/// | `teetotaller` | 2 | the bot skips weak card rewards while holding it ([`RunBot::card_reward_action`](super::RunBot::card_reward_action)) |
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
