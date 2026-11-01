// The Abrechnung at the end of a game, big and short: the score breakdown
// from the core, played step by step like in Balatro. Grundwert and Stufen
// count up, each relic that contributes lights up on its little card, then
// the result lands with weight and the series account moves. A tap skips
// to the end; with reduced motion it shows at once.
//
// All numbers come from the core's ScoreStep list and breakdown; this file
// only labels, formats and animates them.

import { sound } from '../audio';
import { de, cardName, consumableLook, contentName, declarationName, gameName, relicLook, t } from '../i18n';
import type { Change, DeckCard, LevelParts, Lever, ScoreBreakdown, ScoreLine, ScoreStep, ScoreTerm, Source } from '../types';
import { cardSvg, modIcon } from './cards';
import { attrs, esc } from './dom';
import { countUp, dur, finishAll, grow, pulse, slam } from './fx';
import { bossCoaster, consumableCoaster, relicCoaster } from './emblems';
import { coinIcon, meterPercent } from './hud';
import { encounterFaces, portrait } from './people';
import type { ScoreOverlay } from './state';

/** "+3" / "−3" with a real minus sign. */
export function signed(n: number): string {
  if (n > 0) return `+${n}`;
  if (n < 0) return `−${Math.abs(n)}`;
  return '0';
}

function plain(n: number): string {
  return n < 0 ? `−${Math.abs(n)}` : String(n);
}

/** Display name of where a score line (or Groschen) came from. */
export function sourceName(source: Source, deck: DeckCard[]): string {
  if (source === 'Kontra') return de.score.kontra;
  if (source === 'Re') return de.score.re;
  if ('Relic' in source) return contentName('relics', source.Relic);
  if ('Boss' in source) return contentName('bosses', source.Boss);
  if ('Encounter' in source) return contentName('encounters', source.Encounter);
  if ('Consumable' in source) return contentName('consumables', source.Consumable);
  const card = deck.find((c) => c.id === source.CardMod.card);
  return t(de.score.cardMod, {
    mod: contentName('card_mods', source.CardMod.card_mod),
    card: card ? cardName(card.kind) : '',
  });
}

function sourceIcon(source: Source, deck: DeckCard[]): string {
  if (source === 'Kontra' || source === 'Re') return `<span class="step-icon icon-kontra">K</span>`;
  if ('Relic' in source) return `<span class="step-icon">${relicCoaster(relicLook(source.Relic), { size: 22 })}</span>`;
  if ('Boss' in source) return `<span class="step-icon">${bossCoaster(source.Boss, { size: 22 })}</span>`;
  if ('Encounter' in source) return `<span class="step-icon">${portrait(encounterFaces(source.Encounter)[0] ?? 'p-wirt', 'portrait-xs')}</span>`;
  if ('Consumable' in source) return `<span class="step-icon">${consumableCoaster(consumableLook(source.Consumable), { size: 22 })}</span>`;
  const card = deck.find((c) => c.id === source.CardMod.card);
  return card
    ? `<span class="step-icon icon-card">${cardSvg(card.kind, card.mods)}</span>`
    : `<span class="step-icon">${modIcon(source.CardMod.card_mod, 20)}</span>`;
}

function levelText(n: number): string {
  if (n === 1) return t(de.score.plusLevels, { n });
  if (n > 1) return t(de.score.plusLevelsMany, { n });
  return t(de.score.minusLevels, { n: signed(n) });
}

/** What a score line does, e.g. "+1 Stufe", "Grundwert ×2". */
export function changeText(lever: Lever, change: Change): string {
  if ('Multiply' in change) {
    return lever === 'BaseValue'
      ? t(de.score.baseMul, { n: change.Multiply })
      : t(de.score.valueMul, { n: change.Multiply });
  }
  if ('Set' in change) {
    if (lever === 'WinThreshold') return t(de.score.threshold, { n: change.Set });
    if (lever === 'Schneider') return de.score.schneiderBy;
    if (lever === 'LossFactor') return change.Set === 1 ? de.score.lossSingle : t(de.score.lossFactor, { n: change.Set });
    return t(de.score.setTo, { n: change.Set });
  }
  const n = change.Add;
  switch (lever) {
    case 'BaseValue':
      return n >= 0 ? t(de.score.baseAdd, { n }) : t(de.score.baseSub, { n: signed(n) });
    case 'Levels':
      return levelText(n);
    case 'CardPoints':
      return n >= 0 ? t(de.score.pointsAdd, { n }) : t(de.score.pointsSub, { n: signed(n) });
    default:
      return signed(n);
  }
}

interface Row {
  label: string;
  effect: string;
  cls: string;
  icon: string;
  relic: string | null;
}

function lineRow(line: ScoreLine, deck: DeckCard[]): Row {
  const relic = typeof line.source === 'object' && 'Relic' in line.source ? line.source.Relic : null;
  return {
    label: sourceName(line.source, deck),
    effect: changeText(line.lever, line.change),
    cls: relic ? 'is-relic' : 'is-line',
    icon: sourceIcon(line.source, deck),
    relic,
  };
}

/** Label, effect and style of one step. Level amounts come from the core's `rule_levels`. */
function termRow(term: ScoreTerm, deck: DeckCard[], parts: LevelParts): Row {
  const row = (label: string, effect: string, cls: string): Row => ({ label, effect, cls, icon: '', relic: null });
  if (typeof term === 'string') {
    const labels: Record<typeof term, [string, number]> = {
      Game: [de.score.game, parts.game],
      Hand: [de.score.hand, parts.hand],
      Schneider: [de.score.schneider, parts.schneider],
      SchneiderDeclared: [de.score.schneiderDeclared, parts.schneider_declared],
      Schwarz: [de.score.schwarz, parts.schwarz],
      SchwarzDeclared: [de.score.schwarzDeclared, parts.schwarz_declared],
      Ouvert: [de.score.ouvert, parts.ouvert],
      NeverFollowed: [de.score.neverFollowed, parts.never_followed],
      ShortCircuit: [de.score.shortCircuit, parts.short_circuit],
      Virgin: [de.score.virgin, parts.virgin],
    };
    const [label, amount] = labels[term];
    const style = term === 'NeverFollowed' || term === 'ShortCircuit' ? 'is-level is-style' : 'is-level';
    return row(label, levelText(amount), style);
  }
  if ('BaseValue' in term) {
    return row(t(de.score.baseOf, { game: gameName(term.BaseValue.game) }), String(term.BaseValue.value), 'is-base');
  }
  if ('NullValue' in term) return row(de.score.nullValue, String(term.NullValue), 'is-base');
  if ('RamschLoser' in term) return row(de.score.ramschLoser, String(term.RamschLoser), 'is-base');
  if ('SevensDucked' in term) return row(de.score.sevensDucked, levelText(term.SevensDucked), 'is-level is-style');
  if ('Matadors' in term) {
    const m = term.Matadors;
    return row(t(m.with ? de.score.with : de.score.without, { count: m.count }), levelText(m.count), 'is-level');
  }
  return lineRow(term.Line, deck);
}

function coasterStrip(o: ScoreOverlay): string {
  if (!o.relics.length) return '';
  return `<div class="relic-strip">${o.relics
    .map((r) => `<span class="mini-coaster rarity-${esc(r.relic.rarity)}" data-relic="${esc(r.relic.id)}">${relicCoaster(r.relic, { size: 22 })}${esc(contentName('relics', r.relic.id))}</span>`)
    .join('')}</div>`;
}

export function renderScore(o: ScoreOverlay, deck: DeckCard[]): string {
  const b = o.score.breakdown;
  const steps = o.score.steps;
  const hasMul = steps.some((s) => s.multiplier !== 1);
  const rows = steps
    .map((step, i) => {
      const r = termRow(step.term, deck, b.rule_levels);
      return `<li class="step ${r.cls}" data-step="${i}" ${r.relic ? attrs({ 'data-relic': r.relic }) : ''}>${r.icon}<span class="step-label">${esc(r.label)}</span><span class="step-effect">${esc(r.effect)}</span></li>`;
    })
    .join('');
  const decl = declarationName(b.declaration);
  const note = gameNote(b);
  const seatsTable = ramschSeats(b, o.seatNames);
  const fill = meterPercent(o.previousScore, o.target);
  const payout = o.payout
    ? `<div class="payout" data-part="payout">
        <h3>${esc(de.score.seriesWon)}</h3>
        <dl>
          <dt>${esc(de.score.payoutBase)}</dt><dd>${esc(signed(o.payout.payout.base))}</dd>
          <dt>${esc(de.score.payoutGames)}</dt><dd>${esc(signed(o.payout.payout.games_left))}</dd>
          <dt>${esc(de.score.payoutInterest)}</dt><dd>${esc(signed(o.payout.payout.interest))}</dd>
          <dt>${esc(de.header.coins)}</dt><dd><b>${coinIcon(16)} ${esc(o.payout.coins)}</b></dd>
        </dl>
      </div>`
    : '';
  return `<div class="overlay overlay-score" data-action="score-skip"><div class="sheet score-sheet chalkboard is-running" role="dialog" aria-modal="true" aria-labelledby="score-title">
    <header class="score-head">
      <h2 id="score-title" class="${b.won ? 'is-won' : 'is-lost'}">${esc(b.won ? de.score.won : de.score.lost)}</h2>
      <div class="score-decl">${esc(decl)}</div>
      ${
        seatsTable
          ? seatsTable
          : `<div class="score-meta">${esc(t(de.score.points, { own: b.outcome.declarer_card_points, theirs: b.outcome.defender_card_points }))} · ${esc(t(de.score.tricks, { own: b.outcome.declarer_tricks, theirs: b.outcome.defender_tricks }))}</div>`
      }
      ${note ? `<p class="score-note" data-testid="score-note">${esc(note)}</p>` : ''}
    </header>
    ${coasterStrip(o)}
    <div class="formula" aria-hidden="true">
      <div class="fbox fbox-base"><span>${esc(de.score.boxBase)}</span><b data-num="base">0</b></div>
      <span class="fop">×</span>
      <div class="fbox fbox-levels"><span>${esc(de.score.boxLevels)}</span><b data-num="levels">0</b></div>
      ${hasMul ? `<span class="fop">×</span><div class="fbox fbox-mul"><span>${esc(de.score.boxMul)}</span><b data-num="mul">1</b></div>` : ''}
    </div>
    <ol class="steps">${rows}</ol>
    <div class="score-result ${b.won ? 'is-won' : 'is-lost'}" data-part="result">
      <div class="value-line"><span>${esc(de.score.value)}</span><b data-num="value">${esc(b.value)}</b></div>
      <div class="result-big" data-testid="score-result">${esc(signed(b.score))}</div>
      ${b.won ? '' : `<p class="small">${esc(lossNote(b))}</p>`}
    </div>
    <div class="score-account" data-part="account">
      <div class="account-row"><span>${esc(de.score.accountLabel)}</span><b data-num="account">${esc(plain(o.previousScore))}</b><span>${esc(t(de.score.targetLabel, { target: o.target }))}</span></div>
      <div class="chalk-meter"><div class="chalk-fill" style="width:${fill.toFixed(1)}%"></div></div>
    </div>
    ${payout}
    <div class="score-actions">
      <button class="btn btn-wide btn-ghost" ${attrs({ 'data-action': 'score-skip' })}>${esc(de.score.skip)}</button>
    </div>
  </div></div>`;
}

/**
 * Augen and tricks of every seat in a Ramsch, from the core's `seats`. The
 * seats marked as having the most (the losers; a tie marks all) are the
 * core's `most`.
 */
function ramschSeats(b: ScoreBreakdown, names: string[]): string {
  const seats = b.seats;
  if (b.declaration.game !== 'Ramsch' || !seats) return '';
  const rows = ([0, 1, 2] as const)
    .map((seat) => {
      const points = seats.card_points[seat];
      const most = seats.most[seat] === true;
      const tricks = seats.tricks[seat];
      return `<li class="seat-row ${seat === 0 ? 'is-you' : ''} ${most ? 'is-most' : ''}" ${attrs({ 'data-seat': seat })}>
        <span class="seat-name"><span>${esc(names[seat] ?? '')}</span>${most ? `<span class="seat-most">${esc(de.score.most)}</span>` : ''}</span>
        <b class="seat-points">${esc(t(de.score.seatPoints, { n: points }))}</b>
        <span class="seat-tricks">${esc(t(tricks === 1 ? de.score.seatTrick : de.score.seatTricks, { n: tricks }))}</span>
      </li>`;
    })
    .join('');
  return `<ol class="ramsch-seats" data-testid="ramsch-seats" aria-label="${esc(de.score.ramschSeats)}">${rows}</ol>`;
}

/** How a lost game counts: double, or single with "Magenbitter" (a LossFactor line). */
function lossNote(b: ScoreBreakdown): string {
  const line = b.lines.find((l) => l.lever === 'LossFactor' && 'Set' in l.change);
  if (!line || !('Set' in line.change)) return de.score.lostDouble;
  return line.change.Set === 1 ? t(de.score.lostSingle, { name: sourceName(line.source, []) }) : de.score.lostDouble;
}

/** Why a Null or Ramsch was won or lost, from the core's `won` flag. */
function gameNote(b: ScoreBreakdown): string {
  if (b.declaration.game === 'Ramsch') return b.won ? de.score.ramschWon : de.score.ramschLost;
  if (b.declaration.game === 'Null') return b.won ? de.score.nullWon : de.score.nullLost;
  return '';
}

/** Flex gap between the parts of the score sheet (CSS `.score-sheet`). */
const SHEET_GAP = 8;

function continueButton(): string {
  return `<button class="btn btn-primary btn-wide" ${attrs({ 'data-action': 'score-continue' })}>${esc(de.score.continue)}</button>`;
}

/**
 * Plays the score board inside `root` (the overlay layer). `skip()` jumps
 * to the final state at once.
 */
export class ScoreBoard {
  private skipped = false;
  private finished = false;
  private timers: number[] = [];
  private wake: (() => void) | null = null;

  constructor(
    private readonly root: HTMLElement,
    private readonly o: ScoreOverlay,
  ) {}

  get done(): boolean {
    return this.finished;
  }

  private q<T extends Element = HTMLElement>(sel: string): T | null {
    return this.root.querySelector<T>(sel);
  }

  private wait(ms: number): Promise<void> {
    const d = dur(ms);
    if (this.skipped || !d) return Promise.resolve();
    return new Promise((resolve) => {
      const id = window.setTimeout(() => {
        this.wake = null;
        resolve();
      }, d);
      this.timers.push(id);
      this.wake = () => {
        window.clearTimeout(id);
        resolve();
      };
    });
  }

  async play(): Promise<void> {
    const steps = this.o.score.steps;
    if (!dur(1)) {
      this.skip();
      return;
    }
    this.root.querySelectorAll('[data-step], [data-part], .result-big').forEach((el) => el.classList.add('is-pending'));
    let prev: ScoreStep = { term: 'Game', base_value: 0, levels: 0, multiplier: 1 };
    await this.wait(250);
    for (let i = 0; i < steps.length && !this.skipped; i++) {
      const step = steps[i];
      if (!step) continue;
      this.showStep(i, prev, step);
      prev = step;
      await this.wait(i === 0 ? 520 : 440);
    }
    if (this.skipped) return;
    await this.showResult();
  }

  /**
   * Show a hidden part of the board. Hidden parts take no space (no gap
   * while the board plays); a shown part opens up from zero height, so the
   * lines below slide down instead of jumping, and the board scrolls along
   * when it is taller than the screen.
   */
  private reveal(el: HTMLElement | null, opts: { gap?: number; fade?: boolean } = {}): Promise<void> {
    if (!el) return Promise.resolve();
    el.classList.remove('is-pending');
    if (this.skipped) return Promise.resolve();
    return grow(el, opts).then(() => this.follow(el));
  }

  /** Keep the newest line in sight on a board taller than the screen. */
  private follow(el: HTMLElement): void {
    const sheet = this.q('.score-sheet');
    if (!sheet || this.skipped || sheet.scrollHeight <= sheet.clientHeight) return;
    const box = el.getBoundingClientRect();
    const view = sheet.getBoundingClientRect();
    const below = box.bottom - view.bottom + 64;
    if (below > 0) sheet.scrollBy({ top: below, behavior: dur(1) ? 'smooth' : 'auto' });
  }

  private showStep(i: number, prev: ScoreStep, step: ScoreStep): void {
    const row = this.q(`[data-step="${i}"]`);
    void this.reveal(row);
    row?.classList.add('is-in');
    const boxes: [string, number, number][] = [
      ['base', prev.base_value, step.base_value],
      ['levels', prev.levels, step.levels],
      ['mul', prev.multiplier, step.multiplier],
    ];
    for (const [name, from, to] of boxes) {
      if (from === to) continue;
      const el = this.q(`[data-num="${name}"]`);
      if (!el) continue;
      void countUp(el, from, to, 260, plain);
      const box = el.closest('.fbox');
      if (box) pulse(box, 1.14, 300);
    }
    const relic = row?.dataset.relic;
    if (relic) {
      const coaster = this.q(`.mini-coaster[data-relic="${CSS.escape(relic)}"]`);
      coaster?.classList.add('is-lit');
      if (coaster) pulse(coaster, 1.25, 380);
      sound.play('clink');
    } else {
      sound.play('tick', { rate: 1 + Math.min(i, 10) * 0.07 });
    }
  }

  private async showResult(): Promise<void> {
    const result = this.q('[data-part="result"]');
    const big = this.q('.result-big');
    const value = this.q('[data-num="value"]');
    if (value) value.textContent = plain(0);
    await this.reveal(result, { gap: SHEET_GAP });
    if (value) await countUp(value, 0, this.o.score.breakdown.value, 380, plain);
    await this.wait(120);
    if (this.skipped) return;
    if (big) {
      // The space opens while the number is still invisible, then it lands.
      await this.reveal(big, { fade: true });
      if (this.skipped) return;
      big.classList.add('is-landed');
      slam(big, this.q('.score-sheet'));
    }
    sound.play('knock', { delay: 120 });
    sound.play(this.o.score.breakdown.won ? 'win' : 'lose', { delay: 380 });
    await this.wait(620);
    if (this.skipped) return;
    const account = this.q('[data-part="account"]');
    void this.reveal(account, { gap: SHEET_GAP });
    const fill = this.q('.score-account .chalk-fill');
    if (fill) fill.style.width = `${meterPercent(this.o.seriesScore, this.o.target).toFixed(1)}%`;
    const num = this.q('[data-num="account"]');
    if (num) await countUp(num, this.o.previousScore, this.o.seriesScore, 520, plain);
    if (this.skipped) return;
    const payout = this.q('[data-part="payout"]');
    if (payout) {
      await this.wait(200);
      void this.reveal(payout, { gap: SHEET_GAP });
      payout.classList.add('is-in');
      sound.play('coin');
      await this.wait(300);
    }
    this.complete();
  }

  /** Jump to the final state. */
  skip(): void {
    if (this.finished) return;
    this.skipped = true;
    for (const id of this.timers) window.clearTimeout(id);
    this.wake?.();
    finishAll();
    const steps = this.o.score.steps;
    const last = steps[steps.length - 1];
    const set = (name: string, n: number): void => {
      const el = this.q(`[data-num="${name}"]`);
      if (el) el.textContent = plain(n);
    };
    if (last) {
      set('base', last.base_value);
      set('levels', last.levels);
      set('mul', last.multiplier);
    }
    set('value', this.o.score.breakdown.value);
    set('account', this.o.seriesScore);
    const fill = this.q('.score-account .chalk-fill');
    if (fill) fill.style.width = `${meterPercent(this.o.seriesScore, this.o.target).toFixed(1)}%`;
    this.root.querySelectorAll('.is-pending').forEach((el) => el.classList.remove('is-pending'));
    this.root.querySelectorAll('.step[data-relic]').forEach((row) => {
      const relic = (row as HTMLElement).dataset.relic;
      if (relic) this.q(`.mini-coaster[data-relic="${CSS.escape(relic)}"]`)?.classList.add('is-lit');
    });
    this.q('.result-big')?.classList.add('is-landed');
    this.complete();
  }

  private complete(): void {
    if (this.finished) return;
    this.finished = true;
    this.q('.score-sheet')?.classList.remove('is-running');
    const actions = this.q('.score-actions');
    if (actions) actions.innerHTML = continueButton();
  }
}
