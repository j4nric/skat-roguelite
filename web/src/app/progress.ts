// What is stored as the player goes: the run and the meta state (unlocks
// and statistics across runs, learned from the events of every step), the
// unlocks just earned and the list of finished runs.

import { sound } from '../audio';
import { CoreMeta, type CoreRun } from '../core';
import { de } from '../i18n';
import { addToHistory, backupMeta, loadMeta, loadRunInfo, saveMeta, saveRun } from '../storage';
import type { RunEvent, RunView } from '../types';
import type { AccountFlow } from './account';
import type { AppCtx } from './context';

export class Progress {
  /**
   * Changed in memory, not stored yet (../playback/steps.ts, persist): the
   * run's save text after its last step, and whether the meta state (its
   * save text kept by CoreMeta) changed.
   */
  unsaved: { run: { core: CoreRun; save: string } | null; meta: boolean } = { run: null, meta: false };
  /** Special cards received for the first time in this save, waiting for their explanation sheet. */
  readonly specialQueue: string[] = [];

  constructor(
    private readonly ctx: AppCtx,
    private readonly account: AccountFlow,
  ) {}

  /** Load the meta state, or start a new one (keeping an unreadable save aside). */
  async loadMetaState(): Promise<void> {
    const ctx = this.ctx;
    const saved = loadMeta();
    if (saved) {
      try {
        ctx.meta = await CoreMeta.load(saved);
      } catch (err) {
        console.warn('meta state discarded', err);
        backupMeta(saved);
        ctx.ui.message = de.start.metaFailed;
      }
    }
    try {
      ctx.meta ??= await CoreMeta.fresh();
      if (!saved) saveMeta(ctx.meta.save());
      ctx.ui.meta = ctx.meta.view();
    } catch (err) {
      console.warn('meta state unavailable', err);
      ctx.meta = null;
      ctx.ui.meta = null;
    }
    const picked = ctx.ui.meta?.characters.find((c) => c.character === ctx.ui.pickCharacter);
    if (!picked?.unlocked) ctx.ui.pickCharacter = 'Declarer';
  }

  /**
   * Feed the events of a step to the meta state; returns the unlocks just
   * earned. `defer` keeps the new meta state in memory until `persist()`
   * (an opponent's card: stored together with the run, see playSteps).
   */
  async observe(events: RunEvent[] | string, defer = false): Promise<string[]> {
    const meta = this.ctx.meta;
    if (!meta) return [];
    try {
      const step = await meta.observe(events);
      this.ctx.ui.meta = step.view;
      if (defer) this.unsaved.meta = true;
      else saveMeta(meta.save());
      // The first Untot or Wandelkarte of this save: explained once (DESIGN 7.2), shown by render().
      for (const id of step.first_specials ?? []) if (!this.specialQueue.includes(id)) this.specialQueue.push(id);
      return step.earned;
    } catch (err) {
      console.warn('meta state did not take the events', err);
      return [];
    }
  }

  /**
   * Store what playSteps kept in memory: the run (if it is still the one
   * that changed) and the meta state. Also on leaving the page: it asks
   * the core nothing, both save texts are at hand.
   */
  persist(): void {
    const { run, meta } = this.unsaved;
    this.unsaved = { run: null, meta: false };
    try {
      if (run && run.core === this.ctx.core) saveRun(run.save);
      if (meta && this.ctx.meta) saveMeta(this.ctx.meta.save());
    } catch (err) {
      console.warn('state not stored', err);
    }
  }

  /** Announce unlocks just earned. */
  announceUnlocks(ids: string[]): void {
    if (!ids.length || !this.ctx.ui.meta) return;
    this.ctx.ui.overlay = { kind: 'unlock', ids };
    sound.play('win', { delay: 150 });
  }

  /** Remember a finished run in the list of this browser. */
  recordRun(view: RunView): void {
    // The Einführung is no run (DESIGN 16.1): no history, no board.
    if (view.tutorial) return;
    const info = loadRunInfo();
    this.ctx.ui.history = addToHistory({
      character: view.character.character,
      seed: view.seed,
      ...(view.stake > 0 ? { stake: view.stake } : {}),
      section: view.series.section + 1,
      series: view.series.index + 1,
      won: view.phase === 'Won',
      bestGame: view.stats.best_game,
      date: new Date().toISOString(),
      daily: info?.seed === view.seed && info.daily,
    });
    if (info?.seed === view.seed && info.daily) this.account.submitDailyResult(view);
  }
}
