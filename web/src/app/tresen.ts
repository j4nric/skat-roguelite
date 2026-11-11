// Between the pubs: the route on the map, the stops' encounters, the
// rewards after a won pub and the Tresen (relics, Schnaps, Zinken, the
// Kartenmaler). Each tap becomes the core's action; a card modification
// first shows the card before and after (./mods.ts).

import { contentName, de } from '../i18n';
import type { RunAction, RunView } from '../types';
import { markeEffect } from '../ui/encounter';
import { motionOn } from '../ui/fx';
import { legKey } from '../ui/route';
import type { Actions, AppCtx, RunActions, Tap } from './context';
import type { Mods } from './mods';
import type { Flights } from './playback/flights';

export class Tresen {
  constructor(
    private readonly ctx: AppCtx,
    private readonly mods: Mods,
    private readonly flights: Flights,
  ) {}

  actions(): Actions {
    return {
      'scroll-goods': () => this.ctx.root.querySelector('#shop-goods')?.scrollIntoView({ behavior: motionOn() ? 'smooth' : 'auto', block: 'start' }),
    };
  }

  runActions(): RunActions {
    const ui = this.ctx.ui;
    const render = (): void => this.ctx.render();
    const act = (action: RunAction): void => void this.ctx.act(action);
    const routeView = ({ action }: Tap, view: RunView): void => {
      ui.routeOverview = action === 'route-overview';
      ui.routeLeg = legKey(view);
      render();
    };
    const painterOrCleanse = ({ action }: Tap, view: RunView): void => {
      const service: 'painter' | 'cleanse' = action === 'painter' ? 'painter' : 'cleanse';
      const s = service === 'painter' ? view.shop?.painter : view.shop?.cleanse;
      if (!s) return;
      const ask = { targets: s.targets, action: (id: number): RunAction => (service === 'painter' ? { HirePainter: id } : { CleanseCard: id }) };
      void this.mods.openPicker(view, ask, (reasons) => ({ kind: 'zinken-card', service, reasons }));
    };
    const zinke = ({ el, num }: Tap, view: RunView): void => {
      // Zinken (DESIGN 8.6): the card is fixed; like the card reward (05.11.2026) the
      // tile opens the sheet with the card before and after, then "Kaufen".
      const offer = view.shop?.card_offers?.find((o) => o?.slot === num('slot'));
      if (!offer || !offer.can_buy) return;
      this.flights.noteStamp(el.closest('.zinke')?.querySelector('.zinke-card') ?? null);
      void this.mods.applyMod(view, { BuyCardOffer: offer.slot }, offer.card.id, offer.card_mod.id, false, null, 'apply', { pair: true, price: offer.price });
    };
    return {
      // The route (DESIGN 5.0).
      'pick-route': ({ num }, view) => {
        ui.routePick = num('option');
        ui.routeLeg = legKey(view);
        render();
      },
      'route-overview': routeView,
      'route-map': routeView,
      'choose-malus': ({ num }) => act({ ChooseMalus: { option: num('option'), malus: num('malus') } }),
      // The relic pick after a won boss pub (ADR 0017).
      'boss-relic': ({ el, num }) => {
        const replace = el.dataset.replace === undefined ? null : num('replace');
        ui.rewardPick = null;
        act({ ChooseBossRelic: { offer: num('offer'), replace } });
      },
      'boss-relic-pick': ({ num }) => {
        ui.rewardPick = num('offer');
        render();
      },
      'boss-relic-back': () => {
        ui.rewardPick = null;
        render();
      },
      'boss-relic-skip': () => {
        ui.rewardPick = null;
        act({ ChooseBossRelic: { offer: null, replace: null } });
      },
      // The card reward after a won pub (DESIGN 5.0, fixed pairs since 05.11.2026): an offer, then "Anbringen".
      'card-reward-pick': ({ el, num }, view) => {
        const o = view.card_reward?.offers.find((x) => x.index === num('offer'));
        if (!o || !o.can_take) return;
        this.flights.noteStamp(el.querySelector('.card'));
        void this.mods.applyMod(view, { TakeCardReward: { offer: o.index } }, o.card.id, o.card_mod.id, false, null, 'apply', { pair: true });
      },
      'card-reward-skip': () => {
        ui.overlay = null;
        act('SkipCardReward');
      },
      // Encounters at a stop.
      'choose-encounter': ({ num }, view) => {
        const option = view.encounter?.options.find((o) => o.index === num('option'));
        if (!option) return;
        if (markeEffect(option)?.marke === 'Chosen') {
          // A Marke of the player's choice first (04.11.2026), then the card or the draw.
          ui.overlay = { kind: 'encounter-marke', option: option.index };
          render();
        } else if (option.target === 'Card') {
          const marks = option.effects.some((e) => e.card_mod);
          const ask = marks ? { targets: option.targets, action: (id: number): RunAction => ({ ChooseEncounter: { option: option.index, target: id } }) } : null;
          void this.mods.openPicker(view, ask, (reasons) => ({ kind: 'encounter-target', option: option.index, reasons }));
        } else if (option.target === 'Relic') {
          ui.overlay = { kind: 'encounter-relic', option: option.index };
          render();
        } else {
          act({ ChooseEncounter: { option: option.index, target: null } });
        }
      },
      'encounter-target': ({ el, num }, view) => {
        const o = ui.overlay;
        if (o?.kind !== 'encounter-target') return;
        const option = view.encounter?.options.find((x) => x.index === o.option);
        const action: RunAction = { ChooseEncounter: { option: o.option, target: num('id'), marke: o.marke ?? null } };
        this.flights.noteStamp(el);
        // The Zinke on the chosen card (a Painter or cleansing names none); a chosen Marke.
        const chosen = option?.effects.find((e) => typeof e.effect === 'object' && 'Mark' in e.effect && e.effect.Mark.card === 'Chosen');
        // A Marke drawn when taken: no preview of the card afterwards (04.11.2026).
        const drawn = option ? markeEffect(option)?.marke === 'Random' : false;
        if (drawn && option) {
          const marken = (option.marken ?? []).filter((m) => m.can_choose).map((m) => contentName('card_mods', m.card_mod.id) || m.card_mod.name);
          const names = marken.length > 1 ? `${marken.slice(0, -1).join(', ')} ${de.encounter.or} ${marken[marken.length - 1]}` : (marken[0] ?? '');
          this.mods.applyRandomMarke(view, action, num('id'), names, o);
          return;
        }
        // The card before and after, then "Anbringen" (playtest 04.11.2026).
        void this.mods.applyMod(view, action, num('id'), o.marke ?? chosen?.card_mod?.id ?? null, false, o, 'apply');
      },
      'encounter-marke': ({ el }, view) => {
        const o = ui.overlay;
        if (o?.kind !== 'encounter-marke') return;
        const option = view.encounter?.options.find((x) => x.index === o.option);
        const marke = el.dataset.cardMod;
        if (!option || !marke) return;
        if (option.target === 'Card') {
          // Then the card picker, with the cards this Marke may go on.
          const ask = { targets: option.marken?.find((m) => m.card_mod.id === marke)?.targets ?? [], action: (id: number): RunAction => ({ ChooseEncounter: { option: option.index, target: id, marke } }) };
          void this.mods.openPicker(view, ask, (reasons) => ({ kind: 'encounter-target', option: option.index, reasons, marke }));
        } else {
          act({ ChooseEncounter: { option: option.index, target: null, marke } });
        }
      },
      'encounter-relic': ({ num }) => {
        const o = ui.overlay;
        if (o?.kind !== 'encounter-relic') return;
        act({ ChooseEncounter: { option: o.option, target: num('index') } });
      },
      // The sheet with a card before and after its modification (./mods.ts).
      'mod-confirm-go': () => {
        const o = ui.overlay;
        // The Zinke comes down on the card shown "Danach".
        if (o?.kind === 'mod-confirm') this.flights.noteStamp(this.ctx.layerEl.querySelector('.mod-confirm-card:last-of-type .card'));
        if (o?.kind === 'mod-confirm') act(o.action);
      },
      'mod-confirm-back': () => {
        const o = ui.overlay;
        if (o?.kind === 'mod-confirm') {
          ui.overlay = o.back;
          render();
        }
      },
      // The Tresen (DESIGN 8).
      'buy-relic': ({ el, num }) => {
        this.flights.noteBought(el.closest('.relic-tile')?.querySelector('.relic-tile-emblem svg') ?? null);
        act({ BuyRelic: num('slot') });
      },
      'replace-relic-pick': ({ num }) => {
        ui.overlay = { kind: 'relic-replace', slot: num('slot') };
        render();
      },
      'replace-relic': ({ num }) => {
        ui.overlay = null;
        act({ ReplaceRelic: { slot: num('slot'), replace: num('replace') } });
      },
      'buy-item': ({ el, num }) => {
        this.flights.noteBought(el.closest('.good')?.querySelector('.good-pic svg') ?? null);
        act({ BuyConsumable: num('slot') });
      },
      'sell-relic': ({ num }) => act({ SellRelic: num('index') }),
      'sell-item': ({ num }) => act({ SellConsumable: num('slot') }),
      'reroll': () => act('Reroll'),
      'zinke-tile': zinke,
      'buy-zinke': zinke,
      'painter': painterOrCleanse,
      'cleanse': painterOrCleanse,
      'zinken-card': ({ el, num }, view) => {
        const o = ui.overlay;
        if (o?.kind !== 'zinken-card') return;
        // The Kartenmaler's sheet shows each Zinke on the card; cleansing shows the card before and after first.
        if (o.service === 'painter') act({ HirePainter: num('id') });
        else {
          this.flights.noteStamp(el);
          void this.mods.applyMod(view, { CleanseCard: num('id') }, num('id'), null, false, o, 'cleanse');
        }
      },
      'paint': ({ el, num }, view) => {
        const painter = view.painter;
        const option = painter?.options.find((p) => p.index === num('index'));
        if (!painter || !option) return;
        this.flights.noteStamp(el.querySelector('.card'));
        void this.mods.applyMod(view, { ChoosePaint: option.index }, painter.card.id, option.card_mod.id, false, { kind: 'painter' });
      },
      'schnapps-swap': ({ num }, view) => {
        if (view.schnapps_reward) act({ ChooseSchnappsReward: { give_up: num('slot') } });
      },
      'schnapps-sell': (_tap, view) => {
        if (view.schnapps_reward) act({ ChooseSchnappsReward: { give_up: null } });
      },
      'leave-shop': () => act('LeaveShop'),
    };
  }
}
