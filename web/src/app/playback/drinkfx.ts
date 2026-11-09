// What a Schnaps does on screen while its events play: the table washed
// away for a new deal, a card dyed, the ways on the map spun like reels,
// the skat turned over, a factor slammed onto the felt. Only pictures; the
// effect itself comes from the core.

import { sound } from '../../audio';
import { de, t } from '../../i18n';
import { cardBack } from '../../ui/cards';
import { esc } from '../../ui/dom';
import type { Pour } from '../../ui/drink';
import { declareSign, flip, motionOn, play } from '../../ui/fx';
import { rectOf } from '../consts';
import type { AppCtx } from '../context';
import type { Fx } from './fx';

export class DrinkFx {
  /** The Schnaps just poured, for its effects while its events play: the table's cards washed away. */
  pouring: { id: string; kind: Pour['kind']; wash: HTMLElement[] } | null = null;

  constructor(
    private readonly ctx: AppCtx,
    private readonly fx: Fx,
  ) {}

  private rect(selector: string): DOMRect | null {
    return rectOf(this.ctx.screenEl, selector);
  }

  private layer(): HTMLElement | null {
    return this.ctx.root.querySelector<HTMLElement>('.drink-layer');
  }

  /** Pictures of the cards on the felt and in the hand, laid over them until the table is washed. */
  snapshotTable(): HTMLElement[] {
    if (!motionOn()) return [];
    const layer = this.layer();
    if (!layer) return [];
    return Array.from(this.ctx.screenEl.querySelectorAll<HTMLElement>('.declare-choice .felt .card, .declare-choice .felt .skat-pile, .table .hand .hand-card')).map((el) => {
      const r = el.getBoundingClientRect();
      const copy = el.cloneNode(true) as HTMLElement;
      copy.removeAttribute('data-testid');
      copy.removeAttribute('data-action');
      copy.removeAttribute('data-drag');
      copy.removeAttribute('data-info');
      copy.setAttribute('aria-hidden', 'true');
      copy.classList.add('wash-card');
      copy.style.left = `${r.left}px`;
      copy.style.top = `${r.top}px`;
      copy.style.width = `${r.width}px`;
      copy.style.height = `${r.height}px`;
      layer.appendChild(copy);
      return copy;
    });
  }

  /** Kräuter and Doppelkorn: a green wave washes the old cards off the table to the right. */
  washAway(cards: HTMLElement[]): void {
    const layer = this.layer();
    const ms = 760 * this.fx.tempo();
    if (!layer || !cards.length || !motionOn()) {
      for (const el of cards) el.remove();
      return;
    }
    const table = this.rect('.run .table') ?? new DOMRect(0, 0, window.innerWidth, window.innerHeight);
    // The wave stays on the table (a box that clips it); the cards fly off the screen.
    const box = document.createElement('div');
    box.className = 'wash-box';
    box.style.left = `${table.left}px`;
    box.style.top = `${table.top}px`;
    box.style.width = `${table.width}px`;
    box.style.height = `${table.height}px`;
    const wave = document.createElement('div');
    wave.className = 'wash-wave';
    wave.dataset.testid = 'wash-wave';
    box.appendChild(wave);
    layer.appendChild(box);
    const w = window.innerWidth;
    const sweep = play(wave, [{ transform: 'translateX(-100%)' }, { transform: `translateX(${(table.width * 1.05).toFixed(0)}px)` }], { duration: ms, easing: 'cubic-bezier(.4,0,.7,1)', fill: 'both' });
    const done = (): void => box.remove();
    sweep.finished.then(done, done);
    for (const el of cards) {
      const r = el.getBoundingClientRect();
      const reach = Math.max(0, Math.min(1, (r.left + r.width / 2 - table.left) / table.width));
      const spin = (Math.random() - 0.3) * 50;
      const anim = play(el,
        [
          { transform: 'translate(0, 0) rotate(0deg)', opacity: 1 },
          { transform: `translate(${(w - r.left + 40).toFixed(0)}px, ${(-20 - Math.random() * 60).toFixed(0)}px) rotate(${spin.toFixed(0)}deg)`, opacity: 0.9 },
        ],
        { duration: ms * 0.7, delay: ms * reach * 0.5, easing: 'cubic-bezier(.5,0,.8,.6)', fill: 'both' },
      );
      const gone = (): void => el.remove();
      anim.finished.then(gone, gone);
    }
    this.bigWord(de.drink.redeal, table, ms * 0.3, 'is-redeal');
  }

  /** A big pixel word over a box for a moment ("Neu geben!", "Wird neu ausgelost …"). */
  private bigWord(text: string, over: DOMRect, delay: number, cls: string, at = 0.38): void {
    const layer = this.layer();
    if (!layer || !motionOn()) return;
    const word = document.createElement('div');
    word.className = `drink-big ${cls}`;
    word.dataset.testid = `drink-big-${cls.replace('is-', '')}`;
    word.textContent = text;
    word.style.left = `${over.left + over.width / 2}px`;
    word.style.top = `${over.top + over.height * at}px`;
    layer.appendChild(word);
    const ms = 1100 * this.fx.tempo();
    const anim = play(word,
      [
        { transform: 'translate(-50%, -50%) rotate(-6deg) scale(.3)', opacity: 0 },
        { transform: 'translate(-50%, -50%) rotate(-6deg) scale(1.1)', opacity: 1, offset: 0.15 },
        { transform: 'translate(-50%, -50%) rotate(-6deg) scale(1)', opacity: 1, offset: 0.25 },
        { transform: 'translate(-50%, -50%) rotate(-6deg) scale(1)', opacity: 1, offset: 0.8 },
        { transform: 'translate(-50%, -60%) rotate(-6deg) scale(1)', opacity: 0 },
      ],
      { duration: ms, delay, fill: 'both' },
    );
    const gone = (): void => word.remove();
    anim.finished.then(gone, gone);
  }

  /** Blue Curaçao on a hand card: the card takes on the blue. */
  dyeCard(card: number, at: DOMRect): void {
    const layer = this.layer();
    if (!layer || !motionOn()) return;
    const dye = document.createElement('div');
    dye.className = 'drink-dye';
    dye.dataset.testid = 'drink-dye';
    dye.dataset.card = String(card);
    dye.style.left = `${at.left}px`;
    dye.style.top = `${at.top}px`;
    dye.style.width = `${at.width}px`;
    dye.style.height = `${at.height}px`;
    layer.appendChild(dye);
    const ms = 1100 * this.fx.tempo();
    const anim = play(dye,
      [
        { clipPath: 'inset(0 0 100% 0)', opacity: 0.85 },
        { clipPath: 'inset(0 0 0 0)', opacity: 0.85, offset: 0.4 },
        { clipPath: 'inset(0 0 0 0)', opacity: 0.6, offset: 0.75 },
        { clipPath: 'inset(0 0 0 0)', opacity: 0 },
      ],
      { duration: ms, fill: 'both', easing: 'steps(8, end)' },
    );
    const gone = (): void => dye.remove();
    anim.finished.then(gone, gone);
  }

  /** Schwenker: the stops of every way spin like the reels of a slot machine and settle one by one. */
  reelRoutes(): void {
    if (!motionOn()) return;
    const map = this.rect('.route-map');
    if (map) this.bigWord(de.drink.reroll, map, 0, 'is-reroll', 0.2);
    const reels = Array.from(this.ctx.screenEl.querySelectorAll<HTMLElement | SVGElement>('.route-options > *, .route-map .map-badge, .route-map .map-target'));
    reels.forEach((el, i) => {
      el.classList.add('is-reeling');
      el.style.setProperty('--reel-delay', `${Math.round(i * 90 * this.fx.tempo())}ms`);
      el.style.setProperty('--reel-ms', `${Math.round(620 * this.fx.tempo())}ms`);
      el.addEventListener('animationend', () => el.classList.remove('is-reeling'), { once: true });
    });
  }

  /** Kurzer: the skat turns face up. */
  revealSkat(): void {
    if (!motionOn()) return;
    this.ctx.screenEl.querySelectorAll<HTMLElement>('.skat-pile .skat-spot').forEach((el, i) => flip(el, cardBack(), { ms: 380, delay: 80 + i * 120 }));
  }

  /** Doppelkorn and Fusel: "×2" slams onto the felt. */
  slamFactor(factor: number): void {
    const felt = this.rect('.run .felt') ?? this.rect('.run .table');
    if (!felt || !motionOn()) return;
    const html = `<span class="decl-sign-text x2-sign" data-testid="x2-sign"><b>${esc(t(de.drink.times, { factor }))}</b></span>`;
    declareSign(this.ctx.root, html, felt, this.rect('.slate'), { ms: 1500, onHit: () => sound.play('knock', { rate: 0.9, volume: 0.6 }) });
  }
}
