// Save code: export and import of everything stored (../savecode.ts),
// checked by the core before anything is replaced.

import { sound } from '../audio';
import { CoreMeta, CoreRun } from '../core';
import { de, t } from '../i18n';
import { decodeSave, encodeSave } from '../savecode';
import { readAllKeys, replaceAllKeys, runAndMetaOf } from '../storage';
import type { Overlay } from '../ui/state';
import { runLine } from './consts';
import type { AppCtx } from './context';

/** Whether a stored meta state is still the one of a new player (asked of the core). */
export function isFreshMeta(meta: string): boolean {
  try {
    const stored = CoreMeta.load(meta);
    const fresh = CoreMeta.fresh();
    try {
      return stored.save() === fresh.save();
    } finally {
      stored.meta.free();
      fresh.meta.free();
    }
  } catch {
    // Unreadable here: treat it as progress, never replace it unasked.
    return false;
  }
}

/**
 * What a set of stored keys holds, in lines for the player (running run,
 * Vereinsbuch), or null if the core cannot load its run or meta state.
 */
export function describeKeys(keys: Record<string, string>): string[] | null {
  const { run, meta } = runAndMetaOf(keys);
  const lines: string[] = [];
  try {
    if (run) {
      const core = CoreRun.load(run);
      try {
        const view = core.view();
        lines.push(t(de.save.containsRun, { line: `${runLine(view)} · ${de.end.seed} ${view.seed}` }));
      } finally {
        core.dispose();
      }
    } else {
      lines.push(de.save.containsNoRun);
    }
    if (meta) {
      const loaded = CoreMeta.load(meta);
      try {
        const view = loaded.view();
        lines.push(
          t(de.save.containsBook, {
            runs: view.stats.runs_finished,
            done: view.unlocks.filter((u) => u.unlocked).length,
            total: view.unlocks.length,
          }),
        );
      } finally {
        loaded.meta.free();
      }
    }
  } catch (err) {
    console.warn('stored state rejected by the core', err);
    return null;
  }
  return lines;
}

export class SaveCodeFlow {
  constructor(private readonly ctx: AppCtx) {}

  /** Menu "Spielstand sichern": the code of everything stored, to copy or share. */
  async openSaveExport(): Promise<void> {
    const ui = this.ctx.ui;
    const from = ui.overlay;
    let code: string;
    try {
      this.ctx.persist();
      code = await encodeSave(readAllKeys());
    } catch (err) {
      console.warn('save code not made', err);
      this.ctx.showToast(de.save.exportFailed);
      return;
    }
    // The player closed the menu meanwhile.
    if (ui.overlay !== from) return;
    ui.overlay = { kind: 'save-export', code, copied: null };
    this.ctx.render();
  }

  /** Mark the whole save code, for copying by hand. */
  selectSaveCode(): HTMLTextAreaElement | null {
    const area = this.ctx.root.querySelector<HTMLTextAreaElement>('[data-testid="save-code"]');
    if (!area) return null;
    area.focus();
    area.select();
    area.setSelectionRange(0, area.value.length);
    return area;
  }

  /** Copy the save code; without the Clipboard API mark it and try the old way. */
  copySave(): void {
    const ui = this.ctx.ui;
    const o = ui.overlay;
    if (o?.kind !== 'save-export') return;
    const done = (ok: boolean): void => {
      if (ui.overlay !== o) return;
      ui.overlay = { ...o, copied: ok };
      this.ctx.render();
      if (ok) sound.play('clink');
      else this.selectSaveCode();
    };
    const fallback = (): void => {
      let ok = false;
      try {
        ok = this.selectSaveCode() !== null && document.execCommand('copy');
      } catch {
        ok = false;
      }
      done(ok);
    };
    try {
      const clip = navigator.clipboard as Clipboard | undefined;
      if (!clip) {
        fallback();
        return;
      }
      clip.writeText(o.code).then(
        () => done(true),
        (err: unknown) => {
          console.warn('clipboard refused', err);
          fallback();
        },
      );
    } catch (err) {
      console.warn('clipboard unavailable', err);
      fallback();
    }
  }

  /** The share sheet of the system with the save code as text. */
  shareSave(): void {
    const o = this.ctx.ui.overlay;
    if (o?.kind !== 'save-export') return;
    try {
      if (typeof navigator.share !== 'function') {
        this.copySave();
        return;
      }
      navigator.share({ title: de.save.shareTitle, text: o.code }).catch((err: unknown) => {
        if (!(err instanceof DOMException && err.name === 'AbortError')) this.copySave();
      });
    } catch {
      this.copySave();
    }
  }

  /** "Prüfen": read the pasted code and let the core load what it carries. */
  async checkImport(): Promise<void> {
    const ui = this.ctx.ui;
    const o = ui.overlay;
    if (o?.kind !== 'save-import' || o.checking) return;
    const text = this.ctx.root.querySelector<HTMLTextAreaElement>('#save-input')?.value ?? '';
    const pending: Overlay = { kind: 'save-import', text, error: null, checking: true };
    ui.overlay = pending;
    this.ctx.render();
    const result = await this.readSaveCode(text);
    if (ui.overlay !== pending) return;
    ui.overlay =
      'error' in result
        ? { kind: 'save-import', text, error: result.error, checking: false }
        : { kind: 'save-confirm', keys: result.keys, lines: result.lines };
    this.ctx.render();
  }

  /**
   * Decode a save code and check that the core loads its run and meta
   * state. Returns the keys and what they hold, or the German reason.
   */
  private async readSaveCode(text: string): Promise<{ keys: Record<string, string>; lines: string[] } | { error: string }> {
    let decoded;
    try {
      decoded = await decodeSave(text);
    } catch (err) {
      console.warn('save code unreadable', err);
      return { error: de.save.errors.damaged ?? '' };
    }
    if (!decoded.ok) return { error: de.save.errors[decoded.error] ?? '' };
    const { run, meta } = runAndMetaOf(decoded.keys);
    if (!run && !meta) return { error: de.save.errors.nothing ?? '' };
    const lines = describeKeys(decoded.keys);
    if (!lines) return { error: de.save.errors.rejected ?? '' };
    return { keys: decoded.keys, lines };
  }

  /** Replace everything stored by the checked code and start over with it. */
  applyImport(): void {
    const ui = this.ctx.ui;
    const o = ui.overlay;
    if (o?.kind !== 'save-confirm') return;
    if (!replaceAllKeys(o.keys)) {
      ui.overlay = { kind: 'save-import', text: '', error: de.save.errors.write ?? '', checking: false };
      this.ctx.render();
      return;
    }
    window.location.reload();
  }
}
