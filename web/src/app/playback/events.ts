// The events of one step played back, one after another: toasts, sounds
// and effects for each, the game's own events through ./game.ts. Uses only
// the events and the views of the core.

import { sound } from '../../audio';
import { buzz } from '../../haptics';
import { cardName, contentName, de, t } from '../../i18n';
import type { RunEvent, RunView } from '../../types';
import { dur, motionOn } from '../../ui/fx';
import { seatLabel } from '../../ui/guests';
import { reshuffle } from '../../ui/scatter';
import { editionLook, lookSvg } from '../../ui/cards';
import { curseCard, relicToStrip, schnapsToSlot, STAMP_MS } from '../../ui/scenes';
import { DELAY } from '../consts';
import type { AppCtx } from '../context';
import type { Guidance } from '../guidance';
import type { Layer } from '../layer';
import { resetTableChoices } from '../table-input';
import type { DrinkFx } from './drinkfx';
import type { Flights } from './flights';
import type { Fx } from './fx';
import type { GamePlayback, RoseEvent } from './game';

export class EventPlayback {
  constructor(
    private readonly ctx: AppCtx,
    private readonly fx: Fx,
    private readonly flights: Flights,
    private readonly drinkFx: DrinkFx,
    private readonly game: GamePlayback,
    private readonly layer: Layer,
    private readonly guidance: Guidance,
  ) {}

  /** Animate what happened, using only the events and views of the core. */
  async playEvents(before: RunView | null, events: RunEvent[], after: RunView): Promise<void> {
    const game = before?.game;
    if (game) {
      this.ctx.ui.anim = {
        trick: game.trick.slice(),
        hand: game.hand.map((c) => ({ ...c, legal: false, from_skat: false, discardable: false })),
        handSizes: game.hand_sizes.slice(),
        status: '',
        turn: null,
      };
    }
    // Cards a trade cursed: their curse is told by the trade, not again as a Zinke.
    const tradeCursed = new Set<number>();
    // Zinken stamped on in this step, one after another.
    let stamps = 0;
    // Cards a deal cursed in this step: before and after, shown after the render.
    const curses: [string, string][] = [];
    for (let i = 0; i < events.length; i++) {
      const ev = events[i];
      if (ev === undefined) continue;
      if (typeof ev === 'string') {
        if (ev === 'Re') {
          this.ctx.showToast(de.toast.re);
          sound.play('kontra', { rate: 1.12 });
        }
        if (ev === 'Redealt') {
          // Kräuter, Doppelkorn: the old cards are washed off the table first.
          if (this.drinkFx.pouring?.wash.length) {
            this.drinkFx.washAway(this.drinkFx.pouring.wash);
            this.drinkFx.pouring.wash = [];
          }
          this.guidance.passedOnce = true;
          this.ctx.showToast(t(de.toast.redealt, { left: after.series.redeals_left }));
          sound.play('shuffle');
          reshuffle();
          this.ctx.afterRender.push(() => this.flights.dealEffect());
        }
        if (ev === 'RedealUndone') {
          this.ctx.showToast(de.toast.redealUndone);
          sound.play('shuffle');
          reshuffle();
          this.ctx.afterRender.push(() => this.flights.dealEffect());
        }
        if (ev === 'RamschDealt') {
          this.ctx.showToast(de.toast.ramsch);
          sound.play('knock');
        }
        if (ev === 'EveningRulesEnded') this.ctx.showToast(de.toast.eveningEnded);
        if (ev === 'RoutesRerolled') {
          this.ctx.showToast(de.toast.routesRerolled);
          sound.play('chalk', { rate: 1.2 });
          this.ctx.afterRender.push(() => this.drinkFx.reelRoutes());
        }
        if (ev === 'SkatPeekPlanned') this.ctx.showToast(de.toast.skatPeekPlanned);
        continue;
      }
      if ('MarkeGiven' in ev) {
        // A Marke from an encounter (04.11.2026): say which, and where, when drawn.
        const g = ev.MarkeGiven;
        if (g.drawn || g.random_card) {
          const card = after.edition.find((c) => c.id === g.card);
          const text = g.drawn ? de.toast.markeGivenDrawn : de.toast.markeGiven;
          this.ctx.showToast(t(text, { mod: contentName('card_mods', g.card_mod), card: card ? cardName(card.kind) : '' }));
        }
        continue;
      }
      if ('CardMadeTransparent' in ev) {
        // "Glasauge": one toast for all cards of this deal, at the first one.
        const prev = events[i - 1];
        if (prev !== undefined && typeof prev === 'object' && 'CardMadeTransparent' in prev) continue;
        const cards: string[] = [];
        for (let j = i; j < events.length; j++) {
          const e = events[j];
          if (e === undefined || typeof e !== 'object' || !('CardMadeTransparent' in e)) break;
          const { card, seat } = e.CardMadeTransparent;
          cards.push(t(de.toast.transparentCard, { name: seatLabel(after, seat), card: cardName(card.kind) }));
        }
        this.ctx.showToast(t(de.toast.glassEye, { cards: cards.join(', ') }));
        continue;
      }
      if ('CardPeeked' in ev) {
        const { card, seat } = ev.CardPeeked;
        // "Der Zaungast" sees the whole left hand at the table every game:
        // no whisper per card.
        if (after.game?.revealed.some((r) => r.card.id === card.id && r.source === 'Onlooker')) continue;
        this.ctx.showToast(t(de.toast.peeked, { name: seatLabel(after, seat), card: cardName(card.kind) }));
        continue;
      }
      if ('VoucherReturned' in ev) {
        // "Der Drückeberger": a voucher of the pub just ended is back on its card.
        const id = ev.VoucherReturned.card;
        const card = after.edition.find((c) => c.id === id);
        this.ctx.showToast(t(de.toast.voucherReturned, { card: card ? cardName(card.kind) : '', mod: contentName('card_mods', ev.VoucherReturned.card_mod) }), 'top');
        continue;
      }
      if ('VoucherRedeemed' in ev) {
        // Effekte v2: a voucher was redeemed when its card was discarded (or by a hand game).
        const id = ev.VoucherRedeemed.card;
        const card = before?.edition.find((c) => c.id === id) ?? after.edition.find((c) => c.id === id);
        // Declaring a hand game redeems one Marke of the hand (04.11.2026): the card stays in the hand.
        const text = ev.VoucherRedeemed.via_hand ? de.toast.voucherRedeemedHand : de.toast.voucherRedeemed;
        this.ctx.showToast(t(text, { card: card ? cardName(card.kind) : '', mod: contentName('card_mods', ev.VoucherRedeemed.card_mod) }), 'top');
        sound.play('clink');
        if (card) await this.game.redeemVoucher(card, after.edition.find((c) => c.id === id) ?? null);
        continue;
      }
      if ('EveningRuleAdded' in ev) {
        this.ctx.showToast(t(de.toast.eveningRule, { name: contentName('relics', ev.EveningRuleAdded) }));
        sound.play('prize');
        continue;
      }
      if ('RelicSwapped' in ev) {
        this.ctx.showToast(t(de.toast.relicSwapped, { old: contentName('relics', ev.RelicSwapped.old), new: contentName('relics', ev.RelicSwapped.new) }));
        sound.play('prize');
        continue;
      }
      if ('NextGameDoubled' in ev) {
        this.ctx.showToast(t(de.toast.nextDoubled, { factor: ev.NextGameDoubled.factor }));
        sound.play('clink');
        continue;
      }
      if ('TradeCurse' in ev) {
        // Curses as trade: Schnäppchen, Hehlerware, Fusel.
        const { card: id, curse, cause } = ev.TradeCurse;
        const card = after.edition.find((c) => c.id === id);
        tradeCursed.add(id);
        this.ctx.showToast(t(de.toast.tradeCurse[cause], { card: card ? cardName(card.kind) : '', curse: contentName('card_mods', curse) }));
        sound.play('clink');
        // The card is seen as it turns: it crumbles into its cursed self
        // (playtest 12.11.2026), once the screen shows the deal done.
        const old = before?.edition.find((c) => c.id === id);
        if (card && old && !this.ctx.fastForward) {
          curses.push([lookSvg(editionLook(old, before, this.ctx.ui.wildSettled)), lookSvg(editionLook(card, after, this.ctx.ui.wildSettled))]);
        }
        continue;
      }
      if ('DeckChanged' in ev) {
        const change = ev.DeckChanged;
        if ('Modified' in change && tradeCursed.delete(change.Modified.card)) continue;
        if ('Modified' in change) {
          const card = after.edition.find((c) => c.id === change.Modified.card);
          if (card) this.ctx.showToast(t(de.toast.cardMarked, { card: cardName(card.kind), mod: contentName('card_mods', change.Modified.card_mod) }));
          // A Zinke put on at the Tresen, by the Kartenmaler or at an encounter: stamped on.
          const old = before?.edition.find((c) => c.id === change.Modified.card);
          if (card && old && (before?.phase === 'Shop' || before?.phase === 'Encounter')) this.flights.stampLater(old, card, stamps++);
        }
        if ('Cleansed' in change) {
          const card = after.edition.find((c) => c.id === change.Cleansed.card);
          if (card) this.ctx.showToast(t(de.toast.cardCleansed, { card: cardName(card.kind) }));
        }
        continue;
      }
      if ('Game' in ev) {
        // "Glasdaus": cards that break with this trick (their events follow it).
        const breaking: number[] = [];
        // "Untot": a card that rises from this trick (its event follows the trick's hooks).
        let rising: RoseEvent | null = null;
        if ('TrickWon' in ev.Game) {
          for (const next of events.slice(i + 1)) {
            if (typeof next !== 'object') break;
            if ('Game' in next) {
              if ('CardRose' in next.Game) rising = next.Game.CardRose;
              break;
            }
          }
          if (!rising) {
            const later = events.slice(i + 1).find((e): e is Extract<RunEvent, { Game: unknown }> => typeof e === 'object' && 'Game' in e);
            if (later && 'CardRose' in later.Game) rising = later.Game.CardRose;
          }
        }
        await this.game.playGameEvent(ev.Game, after, breaking, rising);
        continue;
      }
      if ('RiseChoice' in ev) {
        // "Untot": the sheet of the core's `rise_choice` asks for the three cards.
        continue;
      }
      if ('CoinsGained' in ev) {
        await this.game.coinsGained(ev.CoinsGained.source, ev.CoinsGained.amount, ev.CoinsGained.coins, after);
        continue;
      }
      if ('GameDealt' in ev) {
        this.ctx.ui.wildSettled = {};
        this.ctx.ui.handOrder = null;
        this.ctx.ui.risen = null;
        this.game.riseShown.clear();
        // A new game: from here on the final view tells the story.
        this.ctx.ui.anim = null;
        this.ctx.ui.view = after;
        resetTableChoices(this.ctx.ui);
        this.fx.greetSeries(after);
        sound.play('shuffle');
        reshuffle();
        this.ctx.afterRender.push(() => this.flights.dealEffect());
        continue;
      }
      if ('NightStarted' in ev) {
        // The long night (ADR 0020): the church clock strikes.
        sound.play('night');
        continue;
      }
      if ('BossAnnounced' in ev) {
        this.ctx.showToast(t(de.toast.bossAnnounced, { name: contentName('bosses', ev.BossAnnounced.boss) }));
        continue;
      }
      if ('Kontra' in ev) {
        this.ctx.showToast(t(de.toast.kontra, { name: seatLabel(after, ev.Kontra.seat) }));
        sound.play('kontra');
        await this.fx.pause(DELAY.notice);
        continue;
      }
      if ('RedealsAdded' in ev) {
        this.ctx.showToast(t(de.toast.passesAdded, { left: ev.RedealsAdded.redeals_left }));
        continue;
      }
      if ('AccountCleared' in ev) {
        this.ctx.showToast(t(de.toast.accountCleared, { before: ev.AccountCleared.before }));
        sound.play('clink');
        continue;
      }
      if ('GamesAdded' in ev) {
        this.ctx.showToast(t(de.toast.gamesAdded, { left: ev.GamesAdded.games_left }));
        sound.play('clink');
        continue;
      }
      if ('PubGivenUp' in ev) {
        // The toasts of the lost pub follow (life, or the end of the run).
        continue;
      }
      if ('SkatPeeked' in ev) {
        this.ctx.afterRender.push(() => this.drinkFx.revealSkat());
      }
      if ('ConsumableUsed' in ev) {
        const id = ev.ConsumableUsed.consumable;
        // Poured over the table: a factor for the next game slams onto the felt.
        const poured = this.drinkFx.pouring;
        // Poured out, and down in one when the player drinks it himself.
        sound.play('pour');
        if (!poured || poured.kind === 'Self') sound.play('gulp', { delay: 420 });
        if (poured?.id === id && poured.kind === 'Table') {
          const effect = after.pending_next_game?.find((p) => p.consumable === id)?.effect;
          const factor = effect && typeof effect === 'object' ? ('Multiply' in effect ? effect.Multiply : effect.MultiplyWin) : null;
          if (factor) this.ctx.afterRender.push(() => this.drinkFx.slamFactor(factor));
        }
        // Klarer and Absacker have said what they did.
        if (events.some((e) => typeof e === 'object' && ('AccountCleared' in e || 'GamesAdded' in e))) continue;
        // A Schnaps that changes the next game says so; the others just get used.
        const waiting = after.pending_next_game?.find((p) => p.consumable === id);
        this.ctx.showToast(
          waiting
            ? t(de.toast.schnapsWaiting, { name: contentName('consumables', id) })
            : t(de.toast.consumableUsed, { name: contentName('consumables', id) }),
        );
        continue;
      }
      if ('RelicBought' in ev) {
        this.ctx.showToast(t(de.toast.bought, { name: contentName('relics', ev.RelicBought.relic) }));
        sound.play('till');
        const bought = this.flights.bought;
        this.flights.bought = null;
        const relic = ev.RelicBought.relic;
        if (bought) {
          this.ctx.afterRender.push(() => {
            const chips = this.ctx.screenEl.querySelectorAll<HTMLElement>(`.leiste-relics .coaster-chip[data-relic="${CSS.escape(relic)}"]`);
            relicToStrip(bought.html, bought.rect, chips[chips.length - 1] ?? null);
          });
        }
        continue;
      }
      if ('ConsumableBought' in ev) {
        this.ctx.showToast(t(de.toast.bought, { name: contentName('consumables', ev.ConsumableBought.consumable) }));
        sound.play('till');
        const bought = this.flights.bought;
        this.flights.bought = null;
        // Its slot: the one the core filled with it in this step.
        const id = ev.ConsumableBought.consumable;
        const slot = after.consumables.find((c) => c.consumable.id === id && !before?.consumables.some((b) => b.slot === c.slot && b.consumable.id === id))?.slot;
        if (bought) {
          this.ctx.afterRender.push(() => {
            const target = slot === undefined ? null : this.ctx.screenEl.querySelector<HTMLElement>(`.leiste-slots [data-testid="schnaps-slot-${slot}"]`);
            schnapsToSlot(bought.html, bought.rect, target);
          });
        }
        continue;
      }
      if ('RelicSold' in ev) {
        this.ctx.showToast(t(de.toast.sold, { name: contentName('relics', ev.RelicSold.relic), price: ev.RelicSold.price }));
        sound.play('coin');
        continue;
      }
      if ('ConsumableSold' in ev) {
        this.ctx.showToast(
          t(de.toast.sold, { name: contentName('consumables', ev.ConsumableSold.consumable), price: ev.ConsumableSold.price }),
        );
        sound.play('coin');
        continue;
      }
      if ('ShopRerolled' in ev) {
        this.ctx.showToast(de.toast.rerolled);
        sound.play('shuffle');
        continue;
      }
      if ('CardOfferBought' in ev || 'PainterHired' in ev || 'CardCleansed' in ev) {
        // The card itself is told by the DeckChanged that follows (or the painter's sheet).
        if (!('PainterHired' in ev) || ev.PainterHired.price > 0) sound.play('till');
        continue;
      }
      if ('CounterRaised' in ev) {
        this.fx.counterRaised(ev.CounterRaised.counter, ev.CounterRaised.value, after);
        continue;
      }
      if ('SkatCardShown' in ev) {
        // "Spickzettel": the card lies face up beside the trick from now on.
        this.fx.wiggleSource(ev.SkatCardShown.source);
        this.ctx.showToast(t(de.toast.skatShown, { card: cardName(ev.SkatCardShown.card.kind) }), 'top');
        continue;
      }
      if ('SkatPointsRaised' in ev) {
        // "Skatpfleger": the discards count more, say so at the relic.
        const { source, card_points } = ev.SkatPointsRaised;
        this.fx.fireTriggers([{ source, lever: 'CardPoints', change: { Add: card_points } }], after);
        continue;
      }
      if ('GameScored' in ev) {
        // What fired, on the chips and cards still on screen, before the board covers them.
        // The regulars take the result, a Schneider is cut across the felt; what fired lights up meanwhile.
        let wait = this.ctx.fastForward ? 0 : await this.flights.gameEndEffect(ev.GameScored.breakdown);
        if (this.fx.fireTriggers(ev.GameScored.breakdown.lines, after)) wait = Math.max(wait, DELAY.trigger);
        if (wait) await this.fx.pause(wait);
        const rest = events.slice(i + 1);
        const won = rest.find((e): e is Extract<RunEvent, { SeriesWon: unknown }> => typeof e === 'object' && 'SeriesWon' in e);
        // One loss sound per chain (playtest 04.11.2026): when this lost game ends the run
        // (ADR 0019), the end screen sounds it, not the board.
        const runEnds = rest.some((e) => typeof e === 'object' && 'RunLost' in e);
        // One pattern per chain: the pub's result, else the game's.
        const pubLost = runEnds || rest.some((e) => typeof e === 'object' && 'PubLost' in e);
        buzz(won ? 'pubWon' : pubLost ? 'pubLost' : ev.GameScored.breakdown.won ? 'gameWon' : 'gameLost');
        await this.game.showScore(
          before?.series.score ?? 0,
          ev.GameScored.series_score,
          before?.series.target ?? after.series.target,
          after,
          won ? { payout: won.SeriesWon.payout, coins: won.SeriesWon.coins } : null,
          runEnds,
        );
        if (!won && !pubLost) this.fx.scoreQuip(after, ev.GameScored.breakdown);
        continue;
      }
      if ('DebtChanged' in ev) {
        const debt = ev.DebtChanged.debt;
        const took = (before?.debt ?? 0) < debt;
        this.ctx.showToast(took ? t(de.toast.debt, { debt }) : debt === 0 ? de.toast.debtPaid : t(de.toast.debtLeft, { debt }));
        continue;
      }
      // ADR 0017: "Die Oma" curses half the edition for the pub, and lifts it after.
      if ('BossCursesLaid' in ev) {
        const { boss, cards } = ev.BossCursesLaid;
        this.ctx.showToast(t(de.toast.bossCursed, { name: contentName('bosses', boss), n: cards.length }), 'top');
        sound.play('kontra', { rate: 0.85 });
        continue;
      }
      if ('BossCursesLifted' in ev) {
        this.ctx.showToast(de.toast.bossCursesLifted);
        continue;
      }
      if ('RelicGained' in ev) {
        this.ctx.showToast(t(de.toast.relicGained, { name: contentName('relics', ev.RelicGained.relic) }));
        sound.play('prize');
        continue;
      }
      if ('SecondChance' in ev) {
        // ADR 0019: the Schutzengel has the lost pub played again and goes.
        const name = contentName('relics', ev.SecondChance.relic);
        this.ctx.showToast(t(de.toast.secondChance, { name }), 'top');
        sound.play('clink');
        if (motionOn()) await this.fx.pause(900);
        continue;
      }
      if ('CoinsStashed' in ev) {
        const { source, amount, pot } = ev.CoinsStashed;
        const name = typeof source === 'object' && 'Relic' in source ? contentName('relics', source.Relic) : '';
        this.fx.wiggleSource(source);
        this.ctx.showToast(t(de.toast.stashed, { name, amount, pot }));
        sound.play('coin');
        continue;
      }
      if ('StashPaidOut' in ev) {
        const { relic, amount } = ev.StashPaidOut;
        this.ctx.showToast(t(de.toast.stashPaidOut, { name: contentName('relics', relic), amount }));
        sound.play('coin');
        continue;
      }
      if ('NextPubPlanned' in ev) {
        const { redeals, target } = ev.NextPubPlanned;
        const parts: string[] = [];
        if (redeals) parts.push(t(de.toast.nextPubRedeals, { n: redeals > 0 ? `+${redeals}` : `−${-redeals}` }));
        if (target) parts.push(t(de.toast.nextPubTarget, { n: target > 0 ? `+${target}` : `−${-target}` }));
        if (parts.length) this.ctx.showToast(t(de.toast.nextPub, { parts: parts.join(', ') }));
        continue;
      }
      if ('WagerPlaced' in ev) {
        const bet = after.wager;
        const text = bet?.on === 'Win' ? de.toast.wagerPlacedWin : de.toast.wagerPlaced;
        this.ctx.showToast(t(text, { stake: ev.WagerPlaced.stake }));
        sound.play('coin');
        continue;
      }
      if ('DiceThrown' in ev) {
        const { stake, won, payout } = ev.DiceThrown;
        this.ctx.showToast(won ? t(de.toast.diceWon, { payout }) : t(de.toast.diceLost, { stake }), 'top');
        sound.play('dice');
        sound.play(won ? 'coin' : 'lose', { delay: 520 });
        continue;
      }
      if ('WagerSettled' in ev) {
        const { stake, won, payout } = ev.WagerSettled;
        this.ctx.showToast(won ? t(de.toast.wagerWon, { payout }) : t(de.toast.wagerLost, { stake }), 'top');
        continue;
      }
      if ('StopsRevealed' in ev) {
        this.ctx.showToast(t(de.toast.stopsRevealed, { n: ev.StopsRevealed.legs }));
        continue;
      }
      if ('MalusChosen' in ev) {
        this.ctx.showToast(t(de.toast.malusChosen, { name: contentName('maluses', ev.MalusChosen.malus) }));
        continue;
      }
      if ('GambleResolved' in ev) {
        this.ctx.showToast(ev.GambleResolved.curse ? de.toast.gambleCurse : de.toast.gambleGood, 'top');
        sound.play(ev.GambleResolved.curse ? 'lose' : 'clink');
        continue;
      }
      if ('SchnappsRewarded' in ev) {
        if (!ev.SchnappsRewarded.slots_full) {
          this.ctx.showToast(t(de.toast.schnappsRewarded, { name: contentName('consumables', ev.SchnappsRewarded.consumable) }));
          sound.play('prize');
        }
        continue;
      }
      if ('SchnappsRewardTaken' in ev) {
        this.ctx.showToast(
          t(de.toast.schnappsRewardTaken, {
            name: contentName('consumables', ev.SchnappsRewardTaken.consumable),
            old: contentName('consumables', ev.SchnappsRewardTaken.given_up),
          }),
        );
        continue;
      }
      if ('ConsumableGained' in ev) {
        this.ctx.showToast(t(de.toast.consumableGained, { name: contentName('consumables', ev.ConsumableGained.consumable) }));
        sound.play('prize');
        continue;
      }
      if ('SeriesStarted' in ev && ev.SeriesStarted.boss && after.series.boss) {
        this.ctx.ui.view = after;
        this.ctx.ui.anim = null;
        this.ctx.ui.screen = 'run';
        const boss = after.series.boss;
        // The door of the boss pub creaks open as his sheet comes up.
        sound.play('boss');
        await this.layer.waitOn((resolve) => ({ kind: 'boss', boss, resolve }));
        this.ctx.ui.overlay = null;
        // The table opens: lights down, the spotlight onto both boss players
        // (once no dialog is open any more, see render()).
        this.ctx.entrancePending = true;
      }
    }
    if (curses.length) {
      // After the next render (the deal done on screen), and after the
      // Zinken stamped on with it (./ui/scenes.ts zinkeStamp, 1150 ms each,
      // the next one at 55 %).
      const wait = stamps ? dur(STAMP_MS) * (0.55 * (stamps - 1) + 0.85) : 0;
      this.ctx.afterRender.push(() => void this.showCurses(curses, wait));
    }
  }

  /** The cursed cards of a step, one after another, after `wait` ms; a tap ends each (a scene). */
  private async showCurses(curses: [string, string][], wait: number): Promise<void> {
    if (wait) await new Promise((resolve) => window.setTimeout(resolve, wait));
    for (const [old, cursed] of curses) await curseCard(this.ctx.root, old, cursed);
  }
}
