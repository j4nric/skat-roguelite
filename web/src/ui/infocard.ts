// The info card: one shared component that explains what is under the
// finger. On a phone it is a bottom sheet (long-press, ./press.ts); with a
// mouse it is a small popover next to the thing hovered; the keyboard opens
// the sheet on the focused thing. It shows name, kind, the effect text with
// the glossary terms marked (a tap explains one inline, below its
// paragraph) and what the view says about it (./info.ts). The popover
// takes no taps (it must never sit in the way of a click); a click, a
// long-press or "i" opens the full sheet.
//
// Closes by a tap outside, a swipe down, Escape or the close button. A
// button with `data-action` in it (buy, use) goes on to the app and closes
// the card. Purely presentation: it never changes the run.

import { de, t } from '../i18n';
import { attrs, esc } from './dom';
import type { InfoModel, InfoSection } from './info';
import { swallowClick } from './press';

export type InfoMode = 'sheet' | 'popover';

/**
 * Matches the glossary terms of `de.keywordForms` as whole words. Two terms
 * have an everyday sense that is no keyword: "Stufe höher" of a card's rank
 * (Gezinkt) and "deine Hand" (the cards held).
 */
function keywordPattern(): RegExp {
  const forms = Object.keys(de.keywordForms)
    .sort((a, b) => b.length - a.length)
    .map((w) => {
      if (/^Stufen?$/.test(w)) return `${w}(?! höher)`;
      if (w === 'Hand') return `(?<!deine |seine |meine |ihre |die |der )Hand`;
      return w;
    });
  return new RegExp(`(?<![\\p{L}])(${forms.join('|')})(?![\\p{L}])([.,:;!?)]*)`, 'gu');
}

const KEYWORD_RE = keywordPattern();

/**
 * An effect text with its glossary terms as buttons; `open` is the keyword
 * shown. Without `buttons` (the popover, which takes no taps) they are only
 * marked.
 */
export function markKeywords(text: string, section: number, open: string | null, buttons = true): string {
  let out = '';
  let last = 0;
  for (const m of text.matchAll(KEYWORD_RE)) {
    const word = m[1] ?? '';
    const tail = m[2] ?? '';
    const key = de.keywordForms[word];
    const at = m.index ?? 0;
    if (!key || !de.keywords[key]) continue;
    out += esc(text.slice(last, at));
    const on = open === key;
    // Punctuation right after the word stays on its line.
    if (!buttons) {
      out += `<span class="kw-wrap"><span class="kw">${esc(word)}</span>${esc(tail)}</span>`;
      last = at + word.length + tail.length;
      continue;
    }
    out += `<span class="kw-wrap"><button type="button" class="kw ${on ? 'is-open' : ''}" ${attrs({ 'data-kw': key, 'data-sec': section, 'aria-expanded': on ? 'true' : 'false' })}>${esc(word)}</button>${esc(tail)}</span>`;
    last = at + word.length + tail.length;
  }
  return out + esc(text.slice(last));
}

function section(s: InfoSection, i: number, open: { sec: number; kw: string } | null, buttons: boolean): string {
  const kw = open && open.sec === i ? open.kw : null;
  const head = s.name
    ? `<div class="info-sec-head"><b>${esc(s.name)}</b>${s.tag ? `<span class="info-tag ${s.curse ? 'is-curse' : ''}">${esc(s.tag)}</span>` : ''}</div>`
    : '';
  const def = kw ? `<p class="kw-def" data-testid="kw-def"><b>${esc(kw)}</b> ${esc(de.keywords[kw] ?? '')}</p>` : '';
  const status = s.status
    .map((x) => `<li class="info-status ${x.on === true ? 'is-on' : x.on === false ? 'is-off' : ''}">${esc(x.text)}</li>`)
    .join('');
  return `<section class="info-sec ${s.curse ? 'is-curse' : ''}">${head}<p class="info-text">${markKeywords(s.text, i, kw, buttons)}</p>${def}${status ? `<ul class="info-states">${status}</ul>` : ''}</section>`;
}

/** The card itself. */
export function renderInfoCard(m: InfoModel, mode: InfoMode, open: { sec: number; kw: string } | null): string {
  const facts = m.facts.length ? `<ul class="info-facts">${m.facts.map((f) => `<li>${esc(f)}</li>`).join('')}</ul>` : '';
  const sheet = mode === 'sheet';
  return `<div class="info-card is-${mode}" role="dialog" aria-modal="${sheet ? 'true' : 'false'}" ${attrs({ 'aria-label': t(de.info.label, { name: m.title }), 'data-testid': 'info-card', 'data-info-key': m.key })}>
    ${sheet ? `<div class="info-grab" aria-hidden="true" title="${esc(de.info.swipe)}"></div>` : ''}
    <header class="info-head">
      <div class="info-art">${m.art}</div>
      <div class="info-title"><span class="info-kind">${esc(m.kind)}</span><h2>${esc(m.title)}</h2>${facts}</div>
    </header>
    ${m.sections.map((s, i) => section(s, i, open, sheet)).join('')}
    ${m.extra ?? ''}
    ${sheet ? `<div class="info-actions">${m.actions ?? ''}<button type="button" class="btn" data-info-close="true">${esc(de.info.close)}</button></div>` : ''}
  </div>`;
}

/**
 * The layer with the info card. `resolve` builds the content for a key from
 * the current view; it returns null when the thing is gone.
 */
export class InfoCard {
  private key: string | null = null;
  private mode: InfoMode = 'sheet';
  private open: { sec: number; kw: string } | null = null;
  private anchor: DOMRect | null = null;
  /** Where focus goes back to after the sheet closes. */
  private opener: HTMLElement | null = null;
  private hoverLeave: number | undefined;
  private drag: { id: number; y: number; dy: number } | null = null;
  /** What the layer shows now. */
  private html = '';

  constructor(
    private readonly layer: HTMLElement,
    private readonly resolve: (key: string) => InfoModel | null,
    private readonly onClose: () => void = () => {},
  ) {
    layer.addEventListener('click', (e) => this.onClick(e));
    layer.addEventListener('pointerdown', (e) => this.onPointerDown(e));
    layer.addEventListener('pointermove', (e) => this.onPointerMove(e));
    layer.addEventListener('pointerup', (e) => this.onPointerUp(e));
    layer.addEventListener('pointercancel', (e) => this.onPointerUp(e));
    // Escape closes the card before anything else hears it.
    window.addEventListener(
      'keydown',
      (e) => {
        if (e.key !== 'Escape' || !this.key) return;
        e.preventDefault();
        e.stopImmediatePropagation();
        this.close();
      },
      true,
    );
  }

  /** Whether a card is open, and in which mode. */
  get state(): { key: string; mode: InfoMode } | null {
    return this.key ? { key: this.key, mode: this.mode } : null;
  }

  /** Open the card for a key: as a sheet, or as a popover next to `anchor`. */
  show(key: string, mode: InfoMode, anchor: Element | null = null): boolean {
    if (mode === 'popover' && this.key && this.mode === 'sheet') return false;
    if (!this.resolve(key)) return false;
    const fresh = this.key !== key || this.mode !== mode;
    this.key = key;
    this.mode = mode;
    if (fresh) this.open = null;
    this.anchor = anchor ? anchor.getBoundingClientRect() : null;
    if (mode === 'sheet') this.opener = anchor instanceof HTMLElement ? anchor : null;
    window.clearTimeout(this.hoverLeave);
    this.render(fresh);
    if (mode === 'sheet' && fresh) this.layer.querySelector<HTMLElement>('[data-info-close]')?.focus({ preventScroll: true });
    return true;
  }

  /** Re-read the content (the view changed); closes if the thing is gone. */
  refresh(): void {
    if (!this.key) return;
    if (!this.resolve(this.key)) {
      this.close();
      return;
    }
    // A popover keeps its place; its thing may have been re-rendered.
    this.render(false);
  }

  close(): void {
    if (!this.key) return;
    const wasSheet = this.mode === 'sheet';
    this.key = null;
    this.open = null;
    this.drag = null;
    this.layer.innerHTML = '';
    this.html = '';
    this.layer.className = 'info-layer';
    if (wasSheet && this.opener?.isConnected) this.opener.focus({ preventScroll: true });
    this.opener = null;
    this.onClose();
  }

  /**
   * The pointer left the thing hovered: close the popover after a moment
   * (moving straight onto the next thing shows that one instead).
   */
  closeHoverSoon(): void {
    if (this.mode !== 'popover') return;
    window.clearTimeout(this.hoverLeave);
    this.hoverLeave = window.setTimeout(() => {
      if (this.mode === 'popover') this.close();
    }, 120);
  }

  private render(fresh: boolean): void {
    const m = this.key ? this.resolve(this.key) : null;
    if (!m) return;
    const sheet = this.mode === 'sheet';
    const html = `${sheet ? `<div class="info-backdrop" data-info-close="true"></div>` : ''}${renderInfoCard(m, this.mode, this.open)}`;
    // The app re-renders often; leave the card alone while it says the same.
    if (!fresh && html === this.html) return;
    this.html = html;
    this.layer.className = `info-layer is-${this.mode} ${fresh ? '' : 'is-static'}`;
    this.layer.innerHTML = html;
    if (!sheet) this.place();
  }

  /** Put the popover above the thing hovered, or below it when there is no room. */
  private place(): void {
    const card = this.layer.querySelector<HTMLElement>('.info-card');
    const a = this.anchor;
    if (!card || !a) return;
    const w = card.offsetWidth;
    const h = card.offsetHeight;
    const vw = window.innerWidth;
    const vh = window.innerHeight;
    const left = Math.max(8, Math.min(vw - w - 8, a.left + a.width / 2 - w / 2));
    const above = a.top - h - 8;
    const top = above >= 8 ? above : Math.min(vh - h - 8, a.bottom + 8);
    card.style.left = `${Math.round(left)}px`;
    card.style.top = `${Math.round(Math.max(8, top))}px`;
  }

  private onClick(e: MouseEvent): void {
    const target = e.target as HTMLElement | null;
    const kw = target?.closest<HTMLElement>('[data-kw]');
    if (kw) {
      e.stopPropagation();
      const sec = Number(kw.dataset.sec ?? '0');
      const key = kw.dataset.kw ?? '';
      this.open = this.open && this.open.sec === sec && this.open.kw === key ? null : { sec, kw: key };
      this.render(false);
      this.layer.querySelector<HTMLElement>(`[data-kw="${key}"][data-sec="${sec}"]`)?.focus({ preventScroll: true });
      return;
    }
    if (target?.closest('[data-info-close]')) {
      e.stopPropagation();
      this.close();
      return;
    }
    // Buy, use: the app handles it; the card has done its job.
    if (target?.closest('[data-action]')) {
      window.setTimeout(() => this.close(), 0);
      return;
    }
    // Anything else in the card is no tap on the table behind it.
    e.stopPropagation();
  }

  // Swipe the sheet down to close it.
  private onPointerDown(e: PointerEvent): void {
    // From the grab bar or the head; the text below may scroll.
    const handle = (e.target as HTMLElement | null)?.closest<HTMLElement>('.info-card.is-sheet .info-grab, .info-card.is-sheet .info-head');
    if (!handle) return;
    this.drag = { id: e.pointerId, y: e.clientY, dy: 0 };
  }

  private onPointerMove(e: PointerEvent): void {
    const d = this.drag;
    if (!d || d.id !== e.pointerId) return;
    d.dy = Math.max(0, e.clientY - d.y);
    const card = this.layer.querySelector<HTMLElement>('.info-card');
    if (card && d.dy > 6) card.style.transform = `translateY(${Math.round(d.dy)}px)`;
  }

  private onPointerUp(e: PointerEvent): void {
    const d = this.drag;
    if (!d || d.id !== e.pointerId) return;
    this.drag = null;
    if (d.dy > 70) {
      // The click after the swipe must not reach the table behind.
      swallowClick();
      this.close();
      return;
    }
    const card = this.layer.querySelector<HTMLElement>('.info-card');
    if (card) card.style.transform = '';
  }
}
