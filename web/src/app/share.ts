// "Teilen": the ended run as a picture (../ui/runcard.ts), and the link
// of a seed, into the system's share sheet or the clipboard.

import { sound } from '../audio';
import { de } from '../i18n';
import type { RunView } from '../types';
import { runCardBlob, runCardName, runCardText } from '../ui/runcard';
import type { Actions, AppCtx } from './context';
import type { Layer } from './layer';

export class Share {
  /** The picture of the ended run for "Teilen", drawn in advance (../ui/runcard.ts). */
  private runCard: { view: RunView; blob: Promise<Blob> } | null = null;

  constructor(
    private readonly ctx: AppCtx,
    private readonly layer: Layer,
  ) {}

  /**
   * The picture of the ended run, drawn once as the end screen comes, so a
   * tap on "Teilen" reaches the share sheet while it still counts as the
   * player's gesture (Safari is strict about that).
   */
  prepareRunCard(): Promise<Blob> {
    const ended = this.ctx.ui.ended;
    if (!ended) return Promise.reject(new Error('no ended run'));
    const view = ended.view;
    if (this.runCard?.view === view) return this.runCard.blob;
    const record = this.ctx.ui.history[0];
    const daily = Boolean(record && record.seed === view.seed && record.daily);
    const blob = runCardBlob(view, { won: ended.won, daily, date: record && record.seed === view.seed ? new Date(record.date) : new Date() });
    // A failed drawing is tried again on the tap.
    blob.catch(() => {
      if (this.runCard?.blob === blob) this.runCard = null;
    });
    this.runCard = { view, blob };
    return blob;
  }


  /**
   * "Teilen" on the end screen: the run as a picture (./ui/runcard.ts),
   * into the share sheet with the file where the browser can, else saved
   * as a download.
   */
  async shareRun(button: HTMLElement): Promise<void> {
    const ended = this.ctx.ui.ended;
    if (!ended || button.hasAttribute('aria-busy')) return;
    const view = ended.view;
    button.setAttribute('aria-busy', 'true');
    let blob: Blob;
    try {
      blob = await this.prepareRunCard();
    } catch (err) {
      console.warn('run picture failed', err);
      this.ctx.showToast(de.runCard.failed);
      button.removeAttribute('aria-busy');
      return;
    }
    button.removeAttribute('aria-busy');
    const name = runCardName(view);
    const text = runCardText(view, ended.won);
    try {
      const file = new File([blob], name, { type: 'image/png' });
      if (typeof navigator.share === 'function' && navigator.canShare?.({ files: [file] })) {
        await navigator.share({ files: [file], title: de.runCard.shareTitle, text });
        return;
      }
    } catch (err) {
      // Closed by the player: nothing more to do.
      if (err instanceof DOMException && err.name === 'AbortError') return;
      console.warn('share sheet failed, saving the picture instead', err);
    }
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = name;
    link.rel = 'noopener';
    document.body.appendChild(link);
    link.click();
    link.remove();
    window.setTimeout(() => URL.revokeObjectURL(url), 10000);
    this.ctx.showToast(de.runCard.saved);
  }


  /** Copy the link of a seed; on failure show it to be copied by hand. */
  copyLink(url: string): void {
    const done = (ok: boolean): void => {
      this.ctx.ui.copy = { url, ok };
      this.layer.markDirty();
      this.ctx.render();
      if (ok) sound.play('clink');
      else this.ctx.root.querySelector<HTMLInputElement>('[data-testid="share-fallback"]')?.select();
    };
    try {
      const clip = navigator.clipboard as Clipboard | undefined;
      if (!clip) {
        done(false);
        return;
      }
      clip.writeText(url).then(
        () => done(true),
        (err: unknown) => {
          console.warn('clipboard refused', err);
          done(false);
        },
      );
    } catch (err) {
      console.warn('clipboard unavailable', err);
      done(false);
    }
  }


  /** The share sheet of the system, where there is one. */
  shareNative(url: string, text: string): void {
    try {
      if (typeof navigator.share !== 'function') {
        this.copyLink(url);
        return;
      }
      navigator.share({ title: de.app.title, text, url }).catch((err: unknown) => {
        // Closing the sheet is no error.
        if (!(err instanceof DOMException && err.name === 'AbortError')) this.copyLink(url);
      });
    } catch {
      this.copyLink(url);
    }
  }

  actions(): Actions {
    return {
      'copy-link': ({ el }) => this.copyLink(el.dataset.url ?? ''),
      'share-run': ({ el }) => void this.shareRun(el),
      'share-native': ({ el }) => this.shareNative(el.dataset.url ?? '', el.dataset.text ?? ''),
    };
  }
}
