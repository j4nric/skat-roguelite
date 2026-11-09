// Card modifications from a card picker (Tresen, encounter, Schnaps, card
// reward, Kartenmaler): which cards the core refuses and why, and the
// sheet that shows a card before and after, from the core's preview.

import { modKind } from '../i18n';
import type { DeckCard, RunAction, RunView, Step } from '../types';
import { modValueText } from '../ui/cards';
import type { ModQuestion, Overlay } from '../ui/state';
import { errorCode } from './consts';
import type { AppCtx } from './context';

export class Mods {
  constructor(private readonly ctx: AppCtx) {}

  /**
   * Why the core refuses each card a modification would go on: its error
   * code for the action (CardCursed, ModAtMaximum, …), for the dimmed
   * cards of the picker. Asked with `check`, which builds no view (W5 of
   * the review: one preview per card made a picker of 32 cards slow).
   */
  refusals(view: RunView, targets: number[], action: (id: number) => RunAction): Record<number, string> {
    const out: Record<number, string> = {};
    const core = this.ctx.core;
    if (!core) return out;
    for (const c of view.edition) {
      if (targets.includes(c.id)) continue;
      let code: string | null;
      try {
        code = core.check(action(c.id));
      } catch (err) {
        code = errorCode(err);
      }
      if (code) out[c.id] = code;
    }
    return out;
  }

  /**
   * A card modification goes on a card: if the core's preview says it
   * replaces, raises or cancels the one on the card, ask first (an in-app
   * sheet); otherwise apply it at once. One drawn at random is not
   * previewed: the sheet only says that the card carries one already.
   */
  applyMod(
    view: RunView,
    action: RunAction,
    id: number,
    mod: string | null,
    random: boolean,
    back: Overlay | null,
    always: 'apply' | 'cleanse' | null = null,
    extra: Pick<ModQuestion, 'pair' | 'price'> = {},
  ): void {
    const card = view.edition.find((c) => c.id === id);
    const found = card ? this.modQuestion(card, action, mod, random, always) : null;
    if (!found) {
      void this.ctx.act(action);
      return;
    }
    const ask: ModQuestion = { ...found, ...extra };
    this.ctx.ui.overlay = { kind: 'mod-confirm', action, back, ask };
    this.ctx.render();
  }

  /**
   * A Marke drawn when the option is taken goes on the chosen card
   * (Kartenwerkstatt, 04.11.2026): the card now, the Marken that may come
   * and "Ziehen". The core is not asked for a preview, which would tell the
   * draw. With another Zinke on the card the sheet says it may go.
   */
  applyRandomMarke(view: RunView, action: RunAction, id: number, marken: string, back: Overlay): void {
    const card = view.edition.find((c) => c.id === id);
    if (!card) return;
    const old = card.mods.find((m) => modKind(m) !== 'Special' && modKind(m) !== 'SpecialCurse') ?? null;
    const blank = { card, after: null, mod: null, from: '', to: '' };
    const ask: ModQuestion = old ? { ...blank, type: 'random', old } : { ...blank, type: 'randomMarke', old: null, marken };
    this.ctx.ui.overlay = { kind: 'mod-confirm', action, back, ask };
    this.ctx.render();
  }

  /**
   * `always` (a card picker): with nothing to warn about, still show the
   * card before and after, from the core's preview, and wait for
   * "Anbringen" (or "Reinigen").
   */
  private modQuestion(card: DeckCard, action: RunAction, mod: string | null, random: boolean, always: 'apply' | 'cleanse' | null = null): ModQuestion | null {
    const blank = { card, after: null, old: null, mod: null, from: '', to: '' };
    if (random) {
      const old = card.mods.find((m) => modKind(m) !== 'Special' && modKind(m) !== 'SpecialCurse');
      return old ? { ...blank, type: 'random', old } : null;
    }
    let step: Step;
    try {
      step = this.ctx.core!.preview(action);
    } catch {
      // The core refuses it: applying shows its error.
      return null;
    }
    const after = step.view.edition.find((c) => c.id === card.id) ?? null;
    // Only a card the step changes is shown (the Kartenmaler's offers open his own sheet).
    const changed = after !== null && JSON.stringify(after) !== JSON.stringify(card);
    if (always === 'cleanse' && changed) return { ...blank, after, type: 'cleanse' };
    const plain = always && changed ? { ...blank, after, type: always, mod } : null;
    for (const ev of step.events) {
      if (typeof ev !== 'object' || !('DeckChanged' in ev)) continue;
      const change = ev.DeckChanged;
      if ('Modified' in change && change.Modified.card === card.id) {
        const m = change.Modified;
        if (m.replaced) return { ...blank, after, type: 'replace', old: m.replaced, mod: m.card_mod };
        if (card.mods.includes(m.card_mod)) {
          // The same again raises it; another (Ramponiert on Augen) is netted with it.
          const values = { from: modValueText(m.card_mod, card.mod_values), to: modValueText(m.card_mod, after?.mod_values ?? {}) };
          if (mod && mod !== m.card_mod) return { ...blank, ...values, after, type: 'change', old: m.card_mod, mod };
          return { ...blank, ...values, after, type: 'raise', mod: m.card_mod };
        }
        return plain ? { ...plain, mod: m.card_mod } : null;
      }
      if ('Unmodified' in change && change.Unmodified.card === card.id) {
        return { ...blank, after, type: 'drop', old: change.Unmodified.card_mod, mod };
      }
    }
    return plain;
  }
}
