// The menu and its settings, and the sheets opened from the head of the
// run or the start screen (rules, glossary, Vereinsbuch, overview, the
// info card of a chip or card).

import { sound } from '../audio';
import { buzz, setVibration } from '../haptics';
import { localize, setLanguage, type Lang } from '../i18n';
import { music } from '../music';
import { saveSettings, TEMPO_FACTOR, type Tempo } from '../settings';
import { setTempo } from '../ui/fx';
import type { UiState } from '../ui/state';
import type { Actions, AppCtx, Tap } from './context';
import type { Layer } from './layer';

export class Menu {
  constructor(
    private readonly ctx: AppCtx,
    private readonly layer: Layer,
  ) {}

  /** Put the stored settings into effect (tempo, sound, music, tint, vibration). */
  applySettings(): void {
    const settings = this.ctx.ui.settings;
    setTempo(TEMPO_FACTOR[settings.tempo]);
    sound.setEnabled(settings.sound);
    music.setEnabled(settings.music);
    // Pure presentation: style.css tints the cards' paper under this flag.
    document.documentElement.toggleAttribute('data-suit-tint', settings.suitTint);
    setVibration(settings.vibrate);
  }

  private setSettings(change: Partial<UiState['settings']>): void {
    const ui = this.ctx.ui;
    ui.settings = { ...ui.settings, ...change };
    saveSettings(ui.settings);
    this.applySettings();
    this.layer.markDirty();
    this.ctx.render();
  }

  /**
   * "Sprache / Language" (ADR 0022): switch and draw everything anew, no
   * reload. The views on screen get the content texts of the new language.
   */
  private setLanguage(next: Lang): void {
    void setLanguage(next, true).then(
      () => {
        const ui = this.ctx.ui;
        localize(ui.view);
        localize(ui.ended?.view);
        localize(ui.meta);
        this.layer.markDirty();
        this.ctx.render();
      },
      (err: unknown) => console.warn('language not loaded', err),
    );
  }

  actions(): Actions {
    const ui = this.ctx.ui;
    const open = (overlay: NonNullable<UiState['overlay']>): void => {
      ui.overlay = overlay;
      this.ctx.render();
    };
    const menu = (): void => {
      if (ui.busy) return;
      open({ kind: 'menu' });
    };
    const info = ({ el }: Tap): void => {
      // A tap on these opens their info card (the same as a long-press).
      const key = el.dataset.info;
      if (key) this.ctx.info.show(key, 'sheet', el);
    };
    return {
      'menu': menu,
      'settings': menu,
      'rules': () => open({ kind: 'rules' }),
      'glossary': () => open({ kind: 'glossary' }),
      'credits': () => open({ kind: 'credits' }),
      'overview': () => {
        if (ui.busy || !ui.view) return;
        open({ kind: 'overview' });
      },
      'book': () => {
        if (ui.busy || !ui.meta) return;
        open({ kind: 'book', tab: 'book' });
      },
      'book-tab': ({ el }) => {
        if (ui.overlay?.kind === 'book') open({ kind: 'book', tab: el.dataset.tab === 'stats' ? 'stats' : 'book' });
      },
      'info-rule': info,
      'info-evening': info,
      'info-boss': info,
      'info-relic': info,
      'info-offer-relic': info,
      'info-offer-item': info,
      'info-item': info,
      'info-malus': info,
      'info-night': info,
      'info-card': info,
      'thinking': ({ el }) => this.setSettings({ thinking: el.dataset.on === 'true' }),
      'hints': ({ el }) => this.setSettings({ hints: el.dataset.on === 'true' }),
      'suit-tint': ({ el }) => this.setSettings({ suitTint: el.dataset.on === 'true' }),
      'vibrate': ({ el }) => {
        this.setSettings({ vibrate: el.dataset.on === 'true' });
        if (el.dataset.on === 'true') buzz('trick');
      },
      'hand-sort': ({ el }) => this.setSettings({ handSort: el.dataset.sort === 'manual' ? 'manual' : 'auto' }),
      'card-back': ({ el }) => this.setSettings({ back: el.dataset.back ? el.dataset.back : null }),
      'tempo': ({ el }) => this.setSettings({ tempo: (el.dataset.tempo ?? 'normal') as Tempo }),
      'sound': ({ el }) => {
        this.setSettings({ sound: el.dataset.on === 'true' });
        if (el.dataset.on === 'true') sound.play('clink');
      },
      'music': ({ el }) => this.setSettings({ music: el.dataset.on === 'true' }),
      'language': ({ el }) => this.setLanguage(el.dataset.lang === 'en' ? 'en' : 'de'),
    };
  }
}
