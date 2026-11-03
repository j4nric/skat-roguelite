// The relic pick after a won boss pub (ADR 0017): the two boss players
// shake hands, three relics lie on the table, the player takes one, makes
// room for it by selling an owned relic (when every slot is full) or walks
// on without. What is on offer and what a choice does come from the core
// (view.boss_reward, RunAction::ChooseBossRelic); this only draws it.

import { de, bossFiguresText, contentName, t } from '../i18n';
import type { RunView } from '../types';
import { attrs, esc } from './dom';
import { bossPortraitId, portrait } from './people';
import { coaster } from './shop';

/** The pick; `picked` is the offer chosen while every slot is full (then the owned relics follow). */
export function renderBossReward(view: RunView, picked: number | null): string {
  const reward = view.boss_reward;
  if (!reward) return '';
  // The boss of the pub just won (its figures shake hands).
  const boss = view.series.boss;
  const faces = boss ? [0, 1].map((f) => portrait(bossPortraitId(boss.id, f), 'portrait-enc')).join('') : '';
  const full = reward.slots_full;
  const chosen = picked === null ? null : (reward.offers.find((o) => o.index === picked) ?? null);
  const offers = reward.offers
    .map((o, i) => {
      const isPicked = chosen?.index === o.index;
      const button = `<button class="btn btn-small ${isPicked ? 'is-picked' : 'btn-primary'}" ${attrs({
        'data-action': full ? 'boss-relic-pick' : 'boss-relic',
        'data-offer': o.index,
        'data-testid': 'boss-relic-take',
        'aria-pressed': full ? (isPicked ? 'true' : 'false') : null,
      })}>${esc(de.boss.rewardTake)}</button>`;
      return `<div class="coaster-wrap" style="--i:${i}">${coaster(o.relic, button, { 'data-action': 'info-relic', 'data-info': `reward-relic:${o.relic.id}` }, { extra: isPicked ? 'is-picked' : '' })}</div>`;
    })
    .join('');
  const replace = chosen
    ? `<section class="boss-reward-replace" data-testid="boss-relic-replace">
        <p>${esc(t(de.boss.rewardReplaceFor, { name: contentName('relics', chosen.relic.id) }))}</p>
        <div class="boss-reward-owned">${view.relics
          .map(
            (r) =>
              `<button class="btn btn-wide" ${attrs({ 'data-action': 'boss-relic', 'data-offer': chosen.index, 'data-replace': r.index })}>${esc(
                t(de.boss.rewardReplace, { name: contentName('relics', r.relic.id), coins: r.sell_price }),
              )}</button>`,
          )
          .join('')}</div>
        <button class="btn btn-small" ${attrs({ 'data-action': 'boss-relic-back' })}>${esc(de.boss.rewardBack)}</button>
      </section>`
    : '';
  return `<main class="encounter boss-reward" data-testid="boss-reward">
    <div class="enc-wall">
      <div class="enc-sign"><span class="enc-kicker">${esc(de.boss.rewardKicker)}</span><span class="enc-sub">${esc(boss ? bossFiguresText(boss) : '')}</span></div>
      <div class="enc-faces">${faces}</div>
    </div>
    <section class="enc-story">
      <h2>${esc(de.boss.rewardTitle)}</h2>
      <p>${esc(de.boss.rewardText)}</p>
      ${full ? `<p class="boss-reward-full" data-testid="boss-reward-full">${esc(de.boss.rewardFull)}</p>` : ''}
    </section>
    <div class="coasters boss-reward-offers">${offers}</div>
    ${replace}
    <button class="btn btn-wide" ${attrs({ 'data-action': 'boss-relic-skip', 'data-testid': 'boss-relic-skip' })}>${esc(de.boss.rewardSkip)}</button>
  </main>`;
}
