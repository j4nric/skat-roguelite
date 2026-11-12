// Rendering of the screen. The DOM has three layers: the screen (drawn on
// every change, part by part: only the parts whose HTML changed are
// rebuilt, ../ui/patch.ts), the overlay (./layer.ts, rendered only when
// the dialog changes, so the score board can animate undisturbed) and the
// toast line (./playback/fx.ts). After every render the Wirt's bubbles
// (./guidance.ts) and the effects waiting for the new DOM follow.

import { type MusicId, music } from '../music';
import { Announcer, DialogFocus } from '../ui/a11y';
import { setPlayerBack } from '../ui/backs';
import { renderBossReward } from '../ui/bossreward';
import { renderCardReward } from '../ui/cardreward';
import { renderEncounter } from '../ui/encounter';
import { type FlightSnapshot, noteFlights, resumeFlights } from '../ui/fx';
import { renderHud } from '../ui/hud';
import { hasSides, isWide } from '../ui/layout';
import { type Part, Patcher, shell } from '../ui/patch';
import { defaultPick, legKey, renderRoute } from '../ui/route';
import { bossEntrance } from '../ui/scenes';
import { renderEnd, renderError, renderKontra, renderLoading, renderRiseChoice, renderStart, settleCharacterPicker } from '../ui/screens';
import { renderShop, watchShop } from '../ui/shop';
import { renderTable } from '../ui/table';
import type { AppCtx } from './context';
import type { Guidance } from './guidance';
import type { Layer } from './layer';
import type { Flights } from './playback/flights';
import type { Progress } from './progress';
import type { Share } from './share';
import type { TableInput } from './table-input';

/**
 * Cards whose flights survive a re-render: the open trick, the hand, the
 * cards to press on the table and the pressed stack.
 */
const FLYING = '.trick:not(.is-last) .trick-card, .hand-card, .skat-card, .pressed-card[data-fly]';

const BUILD_INFO = typeof __BUILD_INFO__ === 'string' ? __BUILD_INFO__ : '';

export class Renderer {
  /** Draws the screen part by part (../ui/patch.ts). */
  private readonly screen: Patcher;
  /** Cards in flight in the parts the patcher just replaced. */
  private snapshots: FlightSnapshot[] = [];
  /** Focus for the sheets drawn on the screen itself (Kontra, Untot pick). */
  private readonly screenFocus: DialogFocus;
  /** What a screen reader hears of the playback (../ui/a11y.ts). */
  private readonly announcer = new Announcer();

  constructor(
    private readonly ctx: AppCtx,
    private readonly layer: Layer,
    private readonly guidance: Guidance,
    private readonly share: Share,
    private readonly table: TableInput,
    private readonly flights: Flights,
    private readonly progress: Progress,
  ) {
    this.screenFocus = new DialogFocus(() => ctx.screenEl);
    this.screen = new Patcher(ctx.screenEl, (nodes) => {
      for (const n of nodes) if (n instanceof Element) this.snapshots.push(...noteFlights(n, FLYING));
    });
  }

  render(): void {
    const ui = this.ctx.ui;
    this.table.keepHandOrder();
    // A waiting explanation of a new special card, once the table is quiet and no other sheet is open.
    if (this.progress.specialQueue.length && !ui.overlay && !ui.busy && !ui.anim && ui.screen === 'run') {
      ui.overlay = { kind: 'special', ids: this.progress.specialQueue.splice(0) };
    }
    // The player's edition shows his character's back (DESIGN 9.3).
    // A back earned in the long night replaces it if chosen (ADR 0020).
    const chosenBack = ui.settings.back && ui.meta?.card_backs?.includes(ui.settings.back) ? ui.settings.back : null;
    setPlayerBack(chosenBack ?? (ui.screen === 'end' ? ui.ended?.view : ui.view)?.character.back);
    let parts: Part[] = [];
    switch (ui.screen) {
      case 'loading':
        parts = [renderLoading()];
        break;
      case 'error':
        parts = [renderError()];
        break;
      case 'start':
        parts = [renderStart(ui, BUILD_INFO)];
        break;
      case 'end':
        parts = [ui.ended ? renderEnd(ui.ended.won, ui.ended.view, ui) : ''];
        if (ui.ended) this.ctx.afterRender.push(() => void this.share.prepareRunCard().catch(() => undefined));
        break;
      case 'run': {
        const view = ui.view;
        if (!view) break;
        const inShop = view.phase === 'Shop' && !ui.anim;
        const inEncounter = view.phase === 'Encounter' && !ui.anim;
        const inRoute = view.phase === 'Route' && !ui.anim;
        const inReward = view.phase === 'BossReward' && !ui.anim;
        const inCardReward = view.phase === 'CardReward' && !ui.anim;
        if (!inReward) ui.rewardPick = null;
        if (inRoute && ui.routeLeg !== legKey(view)) {
          // A new leg: the default way, the map in front.
          ui.routeLeg = legKey(view);
          ui.routePick = defaultPick(view);
          ui.routeOverview = false;
        }
        const atTable = !inShop && !inEncounter && !inReward && !inCardReward && !inRoute;
        // Desktop with room beside the column (./ui/layout.ts): the relics leave the head for the left column.
        const sides = atTable && hasSides() && Boolean(view.game);
        const main: Part[] = inShop
          ? [renderShop(view, ui)]
          : inEncounter
            ? [renderEncounter(view)]
            : inReward
              ? [renderBossReward(view, ui.rewardPick)]
              : inCardReward
                ? [renderCardReward(view)]
                : inRoute
                ? [renderRoute(view, ui.routePick ?? defaultPick(view), ui.routeOverview)]
                : renderTable(view, ui, { wide: isWide(), sides });
        const cls = inShop ? 'is-shop' : inEncounter || inReward || inCardReward ? 'is-encounter' : inRoute ? 'is-route' : 'is-table';
        // HUD and main area are parts of their own (./ui/patch.ts): what did not change stays in the DOM.
        const run = shell(`<div class="run ${cls} ${sides ? 'is-sides' : ''} ${ui.busy ? 'is-busy' : ''} ${view.night ? 'is-night' : ''}">`, [renderHud(view, sides), ...main], { sticky: true });
        // The Kartenmaler waits (Tresen or Werkstatt): his sheet stays until a Zinke is taken.
        if (view.painter && !ui.busy && !ui.anim && (!ui.overlay || ui.overlay.kind === 'zinken-card')) ui.overlay = { kind: 'painter' };
        if (!view.painter && ui.overlay?.kind === 'painter') ui.overlay = null;
        // Die lange Nacht (ADR 0020): a new night district shows its Verschärfung first.
        const night = view.night;
        if (inRoute && night?.announced && ui.nightSeen !== night.district && !ui.busy && !ui.anim && !ui.overlay) {
          ui.nightSeen = night.district;
          ui.overlay = { kind: 'night' };
        }
        // The Schnaps of a won pub with every slot full: its sheet stays until it is taken or sold.
        if (view.schnapps_reward && !ui.busy && !ui.anim && !ui.overlay) ui.overlay = { kind: 'schnapps-reward' };
        if (!view.schnapps_reward && ui.overlay?.kind === 'schnapps-reward') ui.overlay = null;
        const kontra = view.game?.awaiting_kontra_answer && !ui.busy && !ui.overlay;
        // "Untot": the card waits in the trick for three cards of the hand.
        const rise = view.game?.rise_choice;
        parts = [run, kontra ? renderKontra(view) : '', rise && !ui.busy && !ui.overlay && !kontra ? renderRiseChoice(view, ui.risePick) : ''];
        if (!rise && ui.risePick.length) ui.risePick = [];
        break;
      }
    }
    // Keep the typed seed and the picker row across re-renders of the start screen.
    const seed = this.ctx.root.querySelector<HTMLInputElement>('#seed')?.value;
    const charScroll = this.ctx.root.querySelector<HTMLElement>('[data-testid="characters"]')?.scrollLeft;
    // The Tresen stays where it was scrolled to after buying or selling.
    const shopScroll = this.ctx.screenEl.querySelector<HTMLElement>('.run > .shop')?.scrollTop ?? 0;
    // A sheet on the screen that is drawn again (the Untot pick) does not enter again.
    const riseShown = this.ctx.screenEl.querySelector('.sheet-rise') !== null;
    // Cards flying onto the felt keep flying through the re-render: the
    // patcher names the nodes it replaces (noteFlights in the constructor).
    this.snapshots = [];
    const keepFocus = this.screenFocus.before();
    this.screen.patch(parts);
    this.screenFocus.after(keepFocus, true);
    const flights = this.snapshots;
    this.snapshots = [];
    if (riseShown) {
      const still = this.ctx.screenEl.querySelector('.sheet-rise')?.closest('.overlay');
      if (still && !still.classList.contains('is-still')) still.classList.add('is-still');
    }
    if (flights.length) resumeFlights(this.ctx.screenEl, FLYING, flights);
    if (seed) {
      const input = this.ctx.root.querySelector<HTMLInputElement>('#seed');
      if (input) input.value = seed;
    }
    const shop = this.ctx.screenEl.querySelector<HTMLElement>('.run > .shop');
    if (shop && shopScroll) shop.scrollTop = shopScroll;
    if (ui.screen === 'start') settleCharacterPicker(this.ctx.screenEl, charScroll);
    this.layer.render();
    // The score board and the boss take the screen: no info card, no trigger lines over them.
    const big = ui.overlay?.kind === 'score' || ui.overlay?.kind === 'boss';
    if (big) {
      this.ctx.info.close();
      this.ctx.triggers.clear();
    } else {
      this.ctx.info.refresh();
    }
    this.guidance.updateHint();
    this.guidance.updateCoach();
    music.setScene(this.musicScene());
    watchShop(this.ctx.screenEl);
    const effects = this.ctx.afterRender.splice(0);
    // An effect is decoration: one that fails must not take the render (and the playback) down.
    for (const fx of effects) {
      try {
        fx();
      } catch (err) {
        console.error('effect after render failed', err);
      }
    }
    this.announcer.observe(ui, this.ctx.fastForward);
    if (ui.screen === 'run' && ui.view) this.ctx.hudFx.sync(ui.view);
    else if (ui.screen === 'start') this.ctx.hudFx.reset();
    if (ui.screen === 'run' && !ui.busy && !ui.overlay) this.ctx.drink.refresh();
    else this.ctx.drink.cancel();
    this.flights.flyPressed();
    if (ui.screen !== 'run') this.ctx.entrancePending = false;
    if (this.ctx.entrancePending && !ui.overlay && !ui.anim) {
      this.ctx.entrancePending = false;
      if (ui.view?.series.is_boss && ui.view.phase === 'Game') bossEntrance(this.ctx.screenEl);
    }
  }


  /**
   * The music loop for what is on screen: the Tresen loop at the Tresen and
   * at an encounter, the boss loop in a boss series, the table loop at the
   * table and on the start and end screens.
   */
  private musicScene(): MusicId | null {
    const ui = this.ctx.ui;
    if (ui.screen === 'start' || ui.screen === 'end') return 'table';
    if (ui.screen !== 'run' || !ui.view) return null;
    const view = ui.view;
    if ((view.phase === 'Shop' || view.phase === 'Encounter' || view.phase === 'Route' || view.phase === 'BossReward' || view.phase === 'CardReward') && !ui.anim) return 'shop';
    return view.series.is_boss ? 'boss' : 'table';
  }
}
