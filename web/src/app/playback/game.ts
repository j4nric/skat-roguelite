// One event of a game played back: a card onto the felt, a trick swept
// to its winner, the skat picked up, a card rising from the trick or
// settling its suit, Groschen paid, a voucher redeemed, the score board.
// Everything shown comes from the event and the core's views.

import { sound } from '../../audio';
import { buzz } from '../../haptics';
import { cardName, de, face, t } from '../../i18n';
import { TEMPO_FACTOR } from '../../settings';
import type { CardKind, CardView, DealtCard, DeckCard, EditionEntry, RunEvent, RunView, Source, Suit } from '../../types';
import { SUITS } from '../../ui/art';
import { cardBack, cardSvg, lookSvg, showWildSuit } from '../../ui/cards';
import { esc, reducedMotion } from '../../ui/dom';
import { crumbleInto, finishOn, flip, flyIn, glideAll, motionOn, spotsOf, sweepAway } from '../../ui/fx';
import { seatLabel } from '../../ui/guests';
import { coinIcon } from '../../ui/hud';
import { mirrorShimmer, sceneWord } from '../../ui/scenes';
import { sourceName } from '../../ui/score';
import type { ScoreOverlay } from '../../ui/state';
import { DELAY, rectOf } from '../consts';
import type { AppCtx } from '../context';
import type { Layer } from '../layer';
import type { Flights } from './flights';
import type { Fx } from './fx';

/** The boss edition's modifications on a boss card (ADR 0015), from the filters the core names. */
function bossEditionOf(card: DealtCard, edition: EditionEntry[]): { mods: string[]; mod_values: Record<string, number> } | null {
  if (typeof card.kind !== 'object') return null;
  const code = card.kind.Normal;
  const suits: Record<string, Suit> = { C: 'Clubs', S: 'Spades', H: 'Hearts', D: 'Diamonds' };
  const ranks: Record<string, string> = { '7': 'Seven', '8': 'Eight', '9': 'Nine', T: 'Ten', J: 'Jack', Q: 'Queen', K: 'King', A: 'Ace' };
  const suit = suits[code[0] ?? ''];
  const rank = ranks[code[1] ?? ''];
  const mods: string[] = [];
  const mod_values: Record<string, number> = {};
  for (const e of edition) {
    if ((e.filter.suit && e.filter.suit !== suit) || (e.filter.rank && e.filter.rank !== rank)) continue;
    mods.push(e.card_mod.id);
    if (e.card_mod.value != null) mod_values[e.card_mod.id] = e.card_mod.value;
  }
  return { mods, mod_values };
}

/** "Doppelgänger": the event of its mirror. */
type DoppelMirror = Extract<Extract<RunEvent, { Game: unknown }>['Game'], { DoppelgangerMirrored: unknown }>['DoppelgangerMirrored'];
/** "Untot": the event of a card that rose from the trick. */
export type RoseEvent = Extract<Extract<RunEvent, { Game: unknown }>['Game'], { CardRose: unknown }>['CardRose'];

export class GamePlayback {
  /** "Untot": cards whose rising was already shown with their trick in this game. */
  readonly riseShown = new Set<number>();

  constructor(
    private readonly ctx: AppCtx,
    private readonly fx: Fx,
    private readonly flights: Flights,
    private readonly layer: Layer,
  ) {}

  private rect(selector: string): DOMRect | null {
    return rectOf(this.ctx.screenEl, selector);
  }

  /** A card from an event as the table shows it: the modifications of its owner's edition (ADR 0015). */
  cardOf(card: DealtCard, view: RunView): CardView {
    const owner = card.owner ?? 'Player';
    const own = owner === 'Player' ? view.edition.find((d) => d.id === card.id) : undefined;
    // ADR 0017: each boss player plays the edition of his figure of the rule in force.
    const figure = owner === 'Boss' ? 0 : owner === 'BossPartner' ? 1 : -1;
    const boss = figure >= 0 ? bossEditionOf(card, view.game?.boss_rule?.figures?.[figure]?.edition ?? []) : null;
    const mods = own?.mods ?? boss?.mods ?? [];
    const mod_values = own?.mod_values ?? boss?.mod_values ?? {};
    const back = owner === 'Player' ? undefined : (view.game?.backs?.[owner === 'Boss' ? 1 : 2] ?? 'regulars');
    return { ...card, mods, mod_values, ...(back ? { back } : {}), legal: false, from_skat: false, discardable: false, named: this.ctx.ui.wildSettled[card.id]?.printed ?? null };
  }


  async playGameEvent(ev: Extract<RunEvent, { Game: unknown }>['Game'], after: RunView, breaking: number[] = [], rising: RoseEvent | null = null): Promise<void> {
    const anim = this.ctx.ui.anim;
    if ('DoppelgangerMirrored' in ev) {
      await this.doppelMirror(ev.DoppelgangerMirrored, after);
      return;
    }
    if ('CardPlayed' in ev) {
      if (!anim) return;
      const { seat, card } = ev.CardPlayed;
      if (anim.trick.length >= 3) anim.trick = [];
      // Where the card comes from, measured before the table changes.
      const from = seat === 0 ? (this.flights.playedFrom ?? this.rect('.hand')) : this.rect(`.opp[data-seat="${seat}"] .portrait`);
      anim.trick.push({ seat, card: this.cardOf(card, after) });
      anim.handSizes[seat] = Math.max(0, (anim.handSizes[seat] ?? 0) - 1);
      if (seat === 0) anim.hand = anim.hand.filter((c) => c.id !== card.id);
      anim.turn = seat === 0 ? null : seat;
      anim.status = '';
      this.ctx.render();
      const el = this.ctx.screenEl.querySelector(`[data-trick-seat="${seat}"]`);
      const played = anim.trick[anim.trick.length - 1]?.card;
      if (el instanceof HTMLElement && from && !this.ctx.fastForward) this.flights.throwOnto(el, seat, from, played?.back ?? null);
      // An Unter hits the table harder (by its printed face only, no rule asked).
      if (face(card.kind)?.rank === 'J') sound.play('slam', { rate: 0.96 + seat * 0.04 });
      else sound.play('card', { rate: 0.94 + seat * 0.05 });
      await this.fx.pause(seat === 0 ? DELAY.ownCard : DELAY.aiCard);
      return;
    }
    if ('TrickWon' in ev) {
      if (!anim) return;
      const { trick, card_points } = ev.TrickWon;
      anim.turn = null;
      anim.status =
        trick.winner === 0
          ? t(de.table.trickToYou, { points: card_points })
          : t(de.table.trickTo, { name: seatLabel(after, trick.winner), points: card_points });
      this.fx.trickQuip(after, trick.winner, card_points);
      if (trick.winner === 0 && !this.ctx.fastForward) buzz('trick');
      this.ctx.render();
      if (!this.ctx.fastForward) this.flights.reactToTrick(after, trick.winner, card_points);
      await this.fx.pause(DELAY.trickShow);
      if (breaking.length) {
        // "Glasdaus": the card cracks and shatters before the trick leaves.
        for (const id of breaking) this.ctx.screenEl.querySelector(`.trick-card[data-card-id="${id}"]`)?.classList.add('is-shattering');
        sound.play('clink', { rate: 1.6 });
        await this.fx.pause(DELAY.notice);
      }
      // "Untot": the card rises back into the hand, the substitute takes its place, then the trick leaves.
      if (rising) await this.riseCard(rising, after);
      const target = this.flights.sweepTarget(trick.winner);
      const cards = Array.from(this.ctx.screenEl.querySelectorAll('.trick-card'));
      // A card still sliding onto its place lies there now.
      finishOn(cards);
      sound.play('trick');
      // The stack forms on the card that took the trick.
      const onto = cards.findIndex((c) => c.getAttribute('data-trick-seat') === String(trick.winner));
      if (target && !this.ctx.fastForward) await Promise.race([sweepAway(cards, target, DELAY.trickFly, onto), this.fx.pause(DELAY.trickFly + 200)]);
      anim.trick = [];
      anim.status = '';
      this.ctx.render();
      return;
    }
    if ('Conceded' in ev) {
      this.ctx.showToast(de.toast.conceded);
      await this.fx.pause(DELAY.notice);
      return;
    }
    if ('DefenderHandsSwapped' in ev) {
      if (anim) anim.handSizes = ev.DefenderHandsSwapped.hand_sizes.slice();
      this.ctx.showToast(de.toast.handsSwapped);
      sound.play('card');
      this.ctx.render();
      await this.fx.pause(DELAY.notice);
      return;
    }
    if ('CardRose' in ev) {
      // Shown with its trick (riseCard) unless the trick was not on screen.
      if (!this.riseShown.has(ev.CardRose.card.id)) await this.riseCard(ev.CardRose, after);
      return;
    }
    if ('CardChalked' in ev) {
      const { old, new: painted } = ev.CardChalked;
      this.ctx.showToast(t(de.toast.chalked, { old: cardName(old.kind), new: cardName(painted.kind) }));
      return;
    }
    if ('WildCardSettled' in ev) {
      // "Wandelkarte": its suit is settled as the game starts.
      const { old, new: settled } = ev.WildCardSettled;
      const printed = face(old.kind)?.suit;
      if (printed) this.ctx.ui.wildSettled = { ...this.ctx.ui.wildSettled, [old.id]: { kind: settled.kind, printed } };
      this.ctx.showToast(t(de.toast.wildSettled, { old: cardName(old.kind), new: cardName(settled.kind) }));
      if (anim && printed) {
        // The hand shows the settled card in the core's order from now on.
        anim.hand = anim.hand.map((c) => (c.id === old.id ? { ...c, kind: settled.kind, unsettled: false, named: printed } : c));
        const order = after.game?.hand.map((c) => c.id) ?? [];
        if (order.length) anim.hand.sort((a, b) => order.indexOf(a.id) - order.indexOf(b.id));
        this.ctx.render();
        await this.cycleWild(old.id, settled.kind);
      }
      return;
    }
    if ('SkatSwapped' in ev) {
      const { took, gave } = ev.SkatSwapped;
      const g = after.game;
      if (anim && g) {
        anim.hand = g.hand.map((c) => ({ ...c, legal: false }));
        anim.handSizes[0] = anim.hand.length;
      }
      this.ctx.showToast(t(de.toast.skatSwapped, { took: cardName(took.kind), gave: cardName(gave.kind) }));
      this.ctx.render();
      const from = this.rect('.deck-spot');
      const el = this.ctx.screenEl.querySelector(`.hand-card[data-id="${took.id}"]`);
      if (from && el && !this.ctx.fastForward) flyIn(el, from, { ms: 320 });
      sound.play('card');
      await this.fx.pause(DELAY.notice);
      return;
    }
    if ('CardDrawn' in ev) {
      const { seat, card } = ev.CardDrawn;
      if (anim) {
        anim.handSizes[seat] = Math.max(0, (anim.handSizes[seat] ?? 0) - 1);
        this.ctx.render();
      }
      this.ctx.showToast(t(de.toast.cardDrawn, { card: cardName(card.kind), name: seatLabel(after, seat) }), 'top');
      const from = this.rect(`.opp[data-seat="${seat}"]`);
      const el = this.ctx.screenEl.querySelector(`.skat-zone [data-id="${card.id}"], .hand-card[data-id="${card.id}"]`);
      if (from && el && !this.ctx.fastForward) flyIn(el, from, { ms: 360 });
      sound.play('card');
      await this.fx.pause(DELAY.notice);
      return;
    }
    if ('CardReturned' in ev) {
      const { seat, card } = ev.CardReturned;
      if (anim) {
        anim.hand = anim.hand.filter((c) => c.id !== card.id);
        anim.handSizes[0] = anim.hand.length;
        anim.handSizes[seat] = (anim.handSizes[seat] ?? 0) + 1;
        this.ctx.render();
      }
      this.ctx.showToast(t(de.toast.cardReturned, { card: cardName(card.kind), name: seatLabel(after, seat) }), 'top');
      return;
    }
    if ('Discarded' in ev && anim) {
      sound.play('discard');
      const gone = ev.Discarded.cards.map((c) => c.id);
      anim.hand = anim.hand.filter((c) => !gone.includes(c.id));
      anim.handSizes[0] = anim.hand.length;
      this.ctx.render();
      return;
    }
    if (('Declared' in ev || 'SkatPickedUp' in ev) && anim) {
      // Show the new hand order and the declaration right away.
      const g = after.game;
      // "Kurzer": the skat lay face up already, nothing to turn over.
      const peeked = Boolean(this.ctx.ui.view?.game?.skat_peek?.length);
      if (g && 'SkatPickedUp' in ev) {
        anim.hand = g.hand.map((c) => ({ ...c, legal: false }));
        anim.handSizes[0] = anim.hand.length;
        // The skat stays on the table, face up: the cards to press lie there.
        const loose = new Set(g.hand.filter((c) => c.discardable).map((c) => c.id));
        // "Drei drücken": the card drawn from a defender joins them on the table.
        const drawn = g.hand.filter((c) => c.drawn_from != null).map((c) => c.id);
        if (g.phase === 'Discarding') this.ctx.ui.selected = [...ev.SkatPickedUp.skat.map((c) => c.id), ...drawn].filter((id) => loose.has(id));
      }
      if (g && this.ctx.ui.view?.game) {
        this.ctx.ui.view = { ...this.ctx.ui.view, game: { ...this.ctx.ui.view.game, declaration: g.declaration, phase: g.phase, kontra: g.kontra } };
      }
      // "Hand sortieren: automatisch": the hand takes the core's order for the game declared at once.
      const resort = Boolean(g && 'Declared' in ev && this.ctx.ui.settings.handSort === 'auto');
      const spots = resort ? spotsOf(this.ctx.screenEl, '.hand .hand-card') : null;
      if (g && resort) {
        const order = g.hand.map((c) => c.id);
        anim.hand.sort((a, b) => order.indexOf(a.id) - order.indexOf(b.id));
      }
      this.ctx.render();
      if (spots) glideAll(Array.from(this.ctx.screenEl.querySelectorAll<HTMLElement>('.hand .hand-card')), spots, { ms: 320 });
      if ('Declared' in ev && !this.ctx.fastForward) this.flights.declareEffect(ev.Declared.declaration);
      if ('SkatPickedUp' in ev) sound.play('pickup');
      if ('SkatPickedUp' in ev && !this.ctx.fastForward) {
        this.flights.pickUpFlights(ev.SkatPickedUp.skat, peeked);
        await this.fx.pause(DELAY.flip);
      }
    }
  }


  /**
   * Groschen paid by content (a curse when it is attached): a coin pops up
   * at the purse with the amount and where it came from, the purse shows
   * the new total from the core.
   */
  async coinsGained(source: Source, amount: number, coins: number, after: RunView): Promise<void> {
    const from = sourceName(source, after.edition);
    this.fx.wiggleSource(source);
    const sign = amount > 0 ? `+${amount}` : `−${Math.abs(amount)}`;
    this.ctx.showToast(t(de.toast.coins, { amount: sign, source: from }));
    // The coins fly from what paid them (its chip or card on screen), else from the last tap.
    const payer = this.fx.triggerTargets(source)[0]?.getBoundingClientRect() ?? null;
    if (amount > 0) this.ctx.hudFx.gainTo(coins, payer?.width ? payer : null);
    else this.ctx.hudFx.loseTo(coins);
    const box = this.ctx.screenEl.querySelector('.hud-coins')?.getBoundingClientRect();
    const pop = document.createElement('div');
    pop.className = `coin-pop ${amount < 0 ? 'is-minus' : ''}`;
    pop.dataset.testid = 'coin-pop';
    pop.setAttribute('aria-hidden', 'true');
    pop.innerHTML = `${coinIcon(22)}<b>${esc(sign)}</b><span>${esc(from)}</span>`;
    if (box) {
      pop.style.top = `${Math.round(box.bottom + 6)}px`;
      pop.style.right = `${Math.max(8, Math.round(window.innerWidth - box.right))}px`;
    }
    this.ctx.root.appendChild(pop);
    window.setTimeout(() => pop.remove(), Math.max(1400, 2200 * TEMPO_FACTOR[this.ctx.ui.settings.tempo]));
    await this.fx.pause(DELAY.notice);
  }


  /**
   * A voucher redeemed (VoucherRedeemed, board "Einlösen in drei
   * Schritten"): the card shows big over the table, its stamp margin tears
   * off, then a coin flies to the Groschen counter, or the card turns over
   * into its new form (Untot, Wandelkarte). Both faces come from the core's
   * views before and after the discard; the Groschen follow as their own
   * event. Skipped with reduced motion and when the table is hurried.
   */
  async redeemVoucher(card: DeckCard, now: DeckCard | null): Promise<void> {
    if (!motionOn() || this.ctx.fastForward) return;
    const from = this.ctx.screenEl.querySelector(`[data-id="${card.id}"]`)?.getBoundingClientRect() ?? null;
    const box = document.createElement('div');
    box.className = 'voucher-redeem';
    box.dataset.testid = 'voucher-redeem';
    box.dataset.stage = 'voucher';
    box.setAttribute('aria-hidden', 'true');
    box.innerHTML = `<span class="card">${cardSvg(card.kind, card.mods, 'full', card.mod_values)}</span>`;
    this.ctx.root.appendChild(box);
    const el = box.querySelector<HTMLElement>('.card');
    if (!el) return;
    if (from?.width) flyIn(el, from, { ms: 260 });
    await this.fx.pause(DELAY.notice * 0.5);
    // Step 2: the stamp margin tears off.
    el.innerHTML = cardSvg(card.kind, card.mods, 'full', card.mod_values, { redeeming: true });
    box.dataset.stage = 'torn';
    el.animate([{ transform: 'rotate(0)' }, { transform: 'rotate(-4deg)' }, { transform: 'rotate(3deg)' }, { transform: 'rotate(0)' }], { duration: 260, easing: 'ease-out' });
    sound.play('card', { rate: 1.3 });
    await this.fx.pause(DELAY.notice * 0.6);
    // Step 3: what the voucher gives.
    if (now) {
      const turned = now.mods.some((m) => !card.mods.includes(m));
      if (turned) {
        el.innerHTML = cardSvg(now.kind, now.mods, 'full', now.mod_values);
        flip(el, cardBack(), { ms: 360 });
        box.dataset.stage = 'turned';
      } else {
        el.innerHTML = cardSvg(now.kind, now.mods, 'full', now.mod_values);
        box.dataset.stage = 'coin';
        const purse = this.ctx.screenEl.querySelector('.hud-coins')?.getBoundingClientRect();
        if (purse) {
          const coin = document.createElement('span');
          coin.className = 'voucher-coin';
          coin.innerHTML = coinIcon(28);
          coin.style.left = `${Math.round(purse.left + purse.width / 2 - 14)}px`;
          coin.style.top = `${Math.round(purse.top + purse.height / 2 - 14)}px`;
          this.ctx.root.appendChild(coin);
          const anim = flyIn(coin, el.getBoundingClientRect(), { ms: 420, arc: 40 });
          const gone = (): void => coin.remove();
          if (anim) anim.finished.then(gone, gone);
          else gone();
        }
      }
    }
    await this.fx.pause(DELAY.notice * 0.8);
    box.remove();
  }


  /**
   * "Untot" (CardRose): the card lifts out of the trick back into the hand,
   * the substitute from the hand turns over into its place; a toast above
   * the Stammgäste says so. Everything from the event and the core's view.
   */
  private async riseCard(rose: RoseEvent, after: RunView): Promise<void> {
    const { card, substitute } = rose;
    // Since 03.11.2026 it comes back as a copy of the trick's winning card.
    const risen = rose.risen ?? card;
    this.riseShown.add(card.id);
    this.ctx.ui.risen = { substitute: substitute.id, card: cardName(card.kind) };
    this.ctx.showToast(t(de.toast.cardRose, { card: cardName(card.kind), risen: cardName(risen.kind), substitute: cardName(substitute.kind) }), 'top');
    const anim = this.ctx.ui.anim;
    if (!anim) return;
    const subFrom = this.ctx.screenEl.querySelector(`.hand-card[data-id="${substitute.id}"]`)?.getBoundingClientRect() ?? null;
    // The card as it was played crumbles big over the table into the
    // zombie card it comes back as (playtest 05.11.2026), then that one
    // flies into the hand; without motion or hurried it is just there.
    // The pick sheet goes first (the run is busy now), so the scene is seen.
    if (this.ctx.screenEl.querySelector('.sheet-rise')) this.ctx.render();
    const played = this.ctx.screenEl.querySelector(`.trick-card[data-card-id="${card.id}"]`)?.getBoundingClientRect() ?? null;
    const morphed = this.ctx.fastForward ? null : await crumbleInto(this.ctx.root, lookSvg(this.cardOf(card, after)), lookSvg({ ...this.cardOf(risen, after), risen: true }));
    const from = morphed ?? played;
    anim.trick = anim.trick.map((p) => (p.card.id === card.id ? { seat: p.seat, card: this.cardOf(substitute, after) } : p));
    // The trick's Augen changed with the card; the core's count follows with the score.
    anim.status = '';
    const hand = after.game?.hand;
    anim.hand = hand
      ? hand.map((c) => ({ ...c, legal: false }))
      : [...anim.hand.filter((c) => c.id !== substitute.id), { ...this.cardOf(risen, after), can_rise: false, risen: true }];
    // A short green glow while it climbs back into the hand (style.css; with reduced motion a still outline).
    this.ctx.ui.rising = card.id;
    this.ctx.render();
    if (!this.ctx.fastForward) {
      const back = this.ctx.screenEl.querySelector(`.hand-card[data-id="${card.id}"]`);
      if (back && from) flyIn(back, from, { ms: 520, arc: -46 });
      const sub = this.ctx.screenEl.querySelector<HTMLElement>(`.trick-card[data-card-id="${substitute.id}"]`);
      if (sub) {
        if (subFrom) flyIn(sub, subFrom, { ms: 300, delay: 120 });
        flip(sub, cardBack(), { ms: 320, delay: 160 });
      }
    }
    sound.play('card', { rate: 0.8 });
    await this.fx.pause(DELAY.notice);
    this.ctx.ui.rising = null;
  }


  /**
   * "Doppelgänger", Spiegelbild (DoppelgangerMirrored): the played card
   * flashes in its violet mirror look with a shimmer, "Gespiegelt!"; then
   * its CardPlayed throws the copy (the copied face with the mirror look)
   * into the trick. Tempo-scaled, off with reduced motion, a tap ends it.
   */
  private async doppelMirror(m: DoppelMirror, after: RunView): Promise<void> {
    this.ctx.showToast(t(de.toast.doppelMirrored, { card: cardName(m.card.kind), copied: cardName(m.copied.kind) }), 'top');
    const rect = this.flights.playedFrom ?? this.rect(`.hand-card[data-id="${m.card.id}"]`);
    if (this.ctx.fastForward || !rect) return;
    sceneWord(de.doppel.mirrored, rect, 'doppel-word');
    await mirrorShimmer(lookSvg(this.cardOf(m.card, after)), rect);
  }


  async showScore(
    previousScore: number,
    seriesScore: number,
    target: number,
    after: RunView,
    payout: ScoreOverlay['payout'],
    quietLoss = false,
  ): Promise<void> {
    const score = after.last_score;
    if (!score) return;
    const seatNames = ([0, 1, 2] as const).map((seat) => seatLabel(after, seat));
    await this.layer.waitOn((resolve) => ({ kind: 'score', score, previousScore, seriesScore, target, payout, relics: after.relics, seatNames, quietLoss, resolve }));
    this.layer.scoreBoard = null;
    this.ctx.ui.overlay = null;
    // The board goes; the account in the head counts to the new value.
    this.layer.render();
    if (previousScore !== seriesScore && this.ctx.screenEl.querySelector('.hud [data-testid="account"]')) {
      void this.ctx.hudFx.account(seriesScore, target);
      if (motionOn()) await this.fx.pause(520);
    }
  }


  /**
   * "Wandelkarte": as the core settles its suit, the card in the hand runs
   * through the four suits' colours (about 0.8 s) and stops on the new one.
   * Already drawn settled, so reduced motion and hurrying just skip it.
   */
  private async cycleWild(id: number, kind: CardKind): Promise<void> {
    const f = face(kind);
    const card = this.ctx.screenEl.querySelector<HTMLElement>(`.hand [data-id="${id}"]`);
    const svg = card?.querySelector('svg.card-svg');
    if (!f || !card || !svg || reducedMotion() || this.ctx.fastForward) return;
    const steps = 8;
    const last = SUITS.indexOf(f.suit);
    card.classList.add('is-settling');
    sound.play('card', { rate: 1.4 });
    for (let i = 0; i < steps; i++) {
      const suit = SUITS[(last - (steps - 1 - i) + 4 * steps) % 4] ?? f.suit;
      showWildSuit(svg, f.rank, suit);
      if (this.ctx.fastForward) break;
      await this.fx.pause(i === steps - 1 ? 180 : 85);
    }
    showWildSuit(svg, f.rank, f.suit);
    card.classList.remove('is-settling');
  }
}
