// The menu and its settings, and the sheets opened from the head of the
// run or the start screen (rules, glossary, Vereinsbuch, overview, the
// info card of a chip or card).

import { sound } from '../audio';
import { buzz, setVibration } from '../haptics';
import { de, localize, setLanguage, t, type Lang } from '../i18n';
import { music } from '../music';
import { saveSettings, TEMPO_FACTOR, TEXT_SIZES, volumeOf, type Tempo, type TextSize } from '../settings';
import { setColorBlindEmblems } from '../ui/emblems';
import { setTempo } from '../ui/fx';
import type { UiState } from '../ui/state';
import type { Actions, AppCtx, Tap } from './context';
import type { Layer } from './layer';

export class Menu {
  constructor(
    private readonly ctx: AppCtx,
    private readonly layer: Layer,
  ) {}

  /** Put the stored settings into effect (tempo, sound, music, tint, display aids, vibration). */
  applySettings(): void {
    const settings = this.ctx.ui.settings;
    setTempo(TEMPO_FACTOR[settings.tempo]);
    sound.setEnabled(settings.sound);
    sound.setVolume(settings.soundVolume);
    music.setEnabled(settings.music);
    music.setVolume(settings.musicVolume);
    // Pure presentation: style.css tints the cards' paper under this flag.
    document.documentElement.toggleAttribute('data-suit-tint', settings.suitTint);
    // Display aids, also pure CSS: suit letters and shapes, the type scale.
    document.documentElement.toggleAttribute('data-color-blind', settings.colorBlind);
    setColorBlindEmblems(settings.colorBlind);
    if (settings.textSize === 'normal') document.documentElement.removeAttribute('data-text-size');
    else document.documentElement.setAttribute('data-text-size', settings.textSize);
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

  /** When the last preview of the effects' volume sounded (ms). */
  private lastPreview = 0;

  /**
   * A volume slider moved (`input`) or let go (`change`): the volume changes
   * at once, its value beside the name follows; let go, it is stored. No
   * new drawing of the menu, so the slider keeps the pointer and the focus.
   * Moving the effects' slider sounds a card now and then, to hear the level.
   */
  onVolume(e: Event, done: boolean): void {
    const el = e.target;
    if (!(el instanceof HTMLInputElement) || !el.dataset.volume) return;
    const isMusic = el.dataset.volume === 'music';
    const value = volumeOf(Number(el.value));
    const text = t(de.menu.volumeValue, { n: value });
    el.setAttribute('aria-valuetext', text);
    const shown = el.closest('.vol')?.querySelector('[data-vol-value]');
    if (shown) shown.textContent = text;
    if (isMusic) music.setVolume(value);
    else sound.setVolume(value);
    if (!isMusic) {
      const now = performance.now();
      if (done || now - this.lastPreview > 140) {
        this.lastPreview = now;
        sound.play(done ? 'clink' : 'card');
      }
    }
    if (!done) return;
    const ui = this.ctx.ui;
    ui.settings = { ...ui.settings, ...(isMusic ? { musicVolume: value } : { soundVolume: value }) };
    saveSettings(ui.settings);
  }

  /**
   * "Sprache / Language" (ADR 0023): switch and draw everything anew, no
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
      'weekly-board': () => {
        if (ui.busy) return;
        open({ kind: 'weekly-board' });
      },
      'book-tab': ({ el }) => {
        const tab = el.dataset.tab === 'stats' || el.dataset.tab === 'relics' ? el.dataset.tab : 'book';
        if (ui.overlay?.kind === 'book') open({ kind: 'book', tab });
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
      'color-blind': ({ el }) => this.setSettings({ colorBlind: el.dataset.on === 'true' }),
      'text-size': ({ el }) => {
        const size = el.dataset.size as TextSize | undefined;
        this.setSettings({ textSize: size && TEXT_SIZES.includes(size) ? size : 'normal' });
      },
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
