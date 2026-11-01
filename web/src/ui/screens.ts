// Start and end screens and the dialogs: Kontra, boss, info, menu with
// settings, Kreidetafel, Vereinsbuch, new unlocks, Abspann, target picking.

import { de, cardName, contentName, contentText, t } from '../i18n';
import type { CharacterEntry, MetaView, Reward, RunView, UnlockEntry } from '../types';
import { cardSvg } from './cards';
import { attrs, esc, reducedMotion } from './dom';
import { goodSvg } from './goods';
import { oddsLine } from './odds';
import { renderOverview } from './overview';
import { bossPortraitId, characterPortraitId, portrait } from './people';
import type { Overlay, UiState } from './state';

/** A padlock, drawn. */
export function lockIcon(size = 22): string {
  return `<svg class="lock-icon" viewBox="0 0 24 24" width="${size}" height="${size}" aria-hidden="true"><path d="M7 11 V8 a5 5 0 0 1 10 0 V11" fill="none" stroke="currentColor" stroke-width="2.4"/><rect x="4.5" y="10.5" width="15" height="11" rx="2" fill="currentColor"/><circle cx="12" cy="15" r="1.7" fill="#2b1d10"/><path d="M12 16 L12 18.5" stroke="#2b1d10" stroke-width="1.6"/></svg>`;
}

/** Text of the unlock that frees a locked character. */
function unlockText(meta: MetaView, id: string | null): string {
  if (!id) return '';
  const entry = meta.unlocks.find((u) => u.id === id);
  return entry ? contentText('unlocks', entry.id) || entry.text : '';
}

function characterName(c: CharacterEntry): string {
  return contentName('characters', c.character) || c.name;
}

function characterCard(c: CharacterEntry, ui: UiState, meta: MetaView): string {
  const name = characterName(c);
  const short = de.characters[c.character];
  if (!c.unlocked) {
    return `<div class="char-card is-locked" ${attrs({ 'data-character': c.character, 'aria-disabled': 'true' })}>
      <span class="char-lock">${lockIcon(20)}<span>${esc(de.start.locked)}</span></span>
      ${portrait(characterPortraitId(c.character), 'portrait-char')}
      <b class="char-name">${esc(name)}</b>
      <span class="char-role">${esc(short?.role ?? '')}</span>
      <span class="char-cond">${esc(t(de.start.lockedBy, { text: unlockText(meta, c.unlocked_by) }))}</span>
    </div>`;
  }
  const picked = ui.pickCharacter === c.character;
  return `<button class="char-card ${picked ? 'is-picked' : ''}" ${attrs({
    'data-action': 'pick-character',
    'data-character': c.character,
    'aria-pressed': picked ? 'true' : 'false',
  })}>
    ${picked ? `<span class="char-check">${esc(de.start.picked)}</span>` : ''}
    ${portrait(characterPortraitId(c.character), 'portrait-char')}
    <b class="char-name">${esc(name)}</b>
    <span class="char-role">${esc(short?.role ?? '')}</span>
    <span class="char-text">${esc(contentText('characters', c.character) || c.text)}</span>
  </button>`;
}

function startStats(meta: MetaView): string {
  const s = meta.stats;
  if (!s.runs_finished) return `<p class="small start-stats-none">${esc(de.start.noRecord)}</p>`;
  const cell = (n: number, label: string): string => `<div><b>${esc(n)}</b><span>${esc(label)}</span></div>`;
  return `<div class="start-stats chalkboard" data-testid="start-stats">
    ${cell(s.runs_finished, de.start.statRuns)}${cell(s.runs_won, de.start.statWon)}${cell(s.best_section, de.start.statBest)}${cell(s.best_game, de.start.statGame)}
  </div>`;
}

export function renderLoading(): string {
  return `<main class="start"><div class="sign"><h1>${esc(de.app.title)}</h1></div><p>${esc(de.app.loading)}</p></main>`;
}

export function renderError(): string {
  return `<main class="start"><div class="sign"><h1>${esc(de.app.title)}</h1></div><p class="is-warning">${esc(de.app.loadFailed)}</p></main>`;
}

function startFan(): string {
  const cards = ['CA', 'SA', 'HA', 'DA']
    .map((c, i) => `<span class="fan-card" style="--i:${i}">${cardSvg({ Normal: c })}</span>`)
    .join('');
  return `<div class="start-fan" aria-hidden="true">${cards}</div>`;
}

export function renderStart(ui: UiState, buildInfo: string): string {
  const resume = ui.saveInfo
    ? `<button class="btn btn-primary btn-wide" ${attrs({ 'data-action': 'resume' })}>${esc(de.start.resume)}</button>
       <p class="small" data-testid="resume-info">${esc(ui.saveInfo)}</p>`
    : '';
  const meta = ui.meta;
  const chars = meta ? meta.characters.map((c) => characterCard(c, ui, meta)).join('') : '';
  const picked = meta?.characters.find((c) => c.character === ui.pickCharacter);
  const newLabel = picked ? t(de.start.newRunAs, { name: de.characters[picked.character]?.short ?? characterName(picked) }) : de.start.newRun;
  const done = meta ? meta.unlocks.filter((u) => u.unlocked).length : 0;
  const total = meta ? meta.unlocks.length : 0;
  return `<main class="start">
    <div class="lamp" aria-hidden="true"></div>
    <div class="sign">
      <span class="sign-small">${esc(de.start.sign)}</span>
      <h1>${esc(de.app.title)}</h1>
      <span class="sign-tag">${esc(de.app.tagline)}</span>
    </div>
    ${chars ? '' : startFan()}
    ${ui.message ? `<p class="is-warning" role="alert">${esc(ui.message)}</p>` : ''}
    <div class="start-actions">
      ${resume}
    </div>
    ${chars ? `<section class="char-pick" aria-label="${esc(de.start.whoPlays)}"><h2 class="char-title">${esc(de.start.whoPlays)}</h2><div class="char-cards" data-testid="characters">${chars}</div></section>` : ''}
    <div class="start-actions">
      <button class="btn ${ui.saveInfo ? '' : 'btn-primary'} btn-wide" ${attrs({ 'data-action': 'new-run' })}>${esc(newLabel)}</button>
      <label class="seed"><span>${esc(de.start.seedLabel)}</span>
        <input id="seed" name="seed" inputmode="numeric" autocomplete="off" placeholder="${esc(de.start.seedPlaceholder)}"></label>
    </div>
    ${meta ? startStats(meta) : ''}
    ${meta ? `<button class="btn btn-wide btn-book" ${attrs({ 'data-action': 'book' })}>${bookIcon()}<span>${esc(de.start.book)}</span><span class="book-progress">${esc(t(de.start.bookHint, { done, total }))}</span></button>` : ''}
    <p class="howto">${esc(de.start.howTo)}</p>
    <footer class="start-foot">
      <button class="link-btn" ${attrs({ 'data-action': 'settings' })}>${esc(de.start.settings)}</button>
      <button class="link-btn" ${attrs({ 'data-action': 'credits' })}>${esc(de.start.credits)}</button>
      <span class="build" data-testid="build">${esc(t(de.app.build, { info: buildInfo }))}</span>
    </footer>
  </main>`;
}

export function renderEnd(won: boolean, view: RunView, ui: UiState): string {
  return `<main class="end" data-testid="end">
    <div class="sheet chalkboard end-board">
      <h1 class="${won ? 'is-won' : 'is-lost'}">${esc(won ? de.end.won : de.end.lost)}</h1>
      <p class="chalk-small">${esc(won ? de.end.wonText : t(de.end.lostText, { section: view.series.section + 1, series: view.series.index + 1 }))}</p>
      <p class="chalk-small">${esc(de.end.seed)}: ${esc(view.seed)}</p>
      ${renderOverview(view, ui.meta, !won)}
    </div>
    <div class="end-actions">
      <button class="btn btn-primary btn-wide" ${attrs({ 'data-action': 'new-run' })}>${esc(de.end.newRun)}</button>
      <button class="btn btn-wide" ${attrs({ 'data-action': 'to-start' })}>${esc(de.end.toStart)}</button>
    </div>
  </main>`;
}

function sheet(inner: string, closable = true, cls = '', overlayCls = ''): string {
  return `<div class="overlay ${overlayCls}" ${closable ? attrs({ 'data-action': 'close-overlay' }) : ''}><div class="sheet ${cls}" role="dialog" aria-modal="true">${inner}</div></div>`;
}

export function renderKontra(view: RunView): string {
  const kontra = view.game?.kontra;
  const seat = typeof kontra === 'object' && 'Kontra' in kontra ? kontra.Kontra : 1;
  const boss = seat === 1 && view.series.is_boss && view.series.boss ? view.series.boss : null;
  const face = boss ? bossPortraitId(boss.id) : `p-opp-${seat}`;
  const name = boss ? contentName('bosses', boss.id) : (de.seats[seat] ?? '');
  return sheet(
    `<div class="speaker">${portrait(face, 'portrait-lg')}<div class="speech"><h2>${esc(de.kontra.title)}</h2>
     <p>${esc(t(de.kontra.text, { name }))}</p></div></div>
     <p>${esc(de.kontra.question)}</p>
     <div class="sheet-actions">
       <button class="btn btn-primary" ${attrs({ 'data-action': 're', 'data-re': 'true' })}>${esc(de.kontra.re)}</button>
       <button class="btn" ${attrs({ 'data-action': 're', 'data-re': 'false' })}>${esc(de.kontra.noRe)}</button>
     </div>`,
    false,
    'sheet-kontra',
    'overlay-kontra',
  );
}

function settingsBlock(ui: UiState): string {
  const tempos = (['slow', 'normal', 'fast'] as const)
    .map(
      (k) =>
        `<button class="seg ${ui.settings.tempo === k ? 'is-picked' : ''}" ${attrs({ 'data-action': 'tempo', 'data-tempo': k, 'aria-pressed': ui.settings.tempo === k ? 'true' : 'false' })}>${esc(de.menu.tempos[k])}</button>`,
    )
    .join('');
  const sound = ui.settings.sound;
  return `<div class="setting"><span class="setting-label">${esc(de.menu.tempo)}</span><div class="segmented">${tempos}</div></div>
    ${reducedMotion() ? `<p class="small">${esc(de.menu.reducedMotion)}</p>` : ''}
    <div class="setting"><span class="setting-label">${esc(de.menu.sound)}</span><div class="segmented">
      <button class="seg ${sound ? 'is-picked' : ''}" ${attrs({ 'data-action': 'sound', 'data-on': 'true', 'aria-pressed': sound ? 'true' : 'false' })}>${esc(de.menu.soundOn)}</button>
      <button class="seg ${!sound ? 'is-picked' : ''}" ${attrs({ 'data-action': 'sound', 'data-on': 'false', 'aria-pressed': !sound ? 'true' : 'false' })}>${esc(de.menu.soundOff)}</button>
    </div></div>`;
}

function credits(): string {
  const rows = de.credits.entries
    .map((e) => `<li><b>${esc(e.what)}</b><span>${esc(e.who)}</span><span class="credit-license">${esc(e.license)}</span></li>`)
    .join('');
  return `<h2>${esc(de.credits.title)}</h2>
    <p>${esc(de.credits.intro)}</p>
    <ul class="credits">${rows}</ul>
    <p class="credits-thanks">${esc(de.credits.thanks)}</p>
    <button class="btn btn-wide" ${attrs({ 'data-action': 'close-overlay' })}>${esc(de.app.close)}</button>`;
}

export function renderOverlay(overlay: Overlay, view: RunView | null, ui: UiState): string {
  switch (overlay.kind) {
    case 'boss':
      return sheet(
        `<div class="boss-intro">${portrait(bossPortraitId(overlay.boss.id), 'portrait-boss')}
         <p class="small">${esc(de.boss.title)} · ${esc(de.boss.intro)}</p>
         <h2>${esc(contentName('bosses', overlay.boss.id))}</h2>
         <p>${esc(contentText('bosses', overlay.boss.id) || overlay.boss.text)}</p></div>
         <button class="btn btn-primary btn-wide" ${attrs({ 'data-action': 'boss-go' })}>${esc(de.boss.go)}</button>`,
        false,
        'sheet-boss',
      );
    case 'info': {
      const pic = overlay.good ? `<div class="info-pic">${goodSvg(overlay.good, 72)}</div>` : '';
      const body = overlay.coaster
        ? `<article class="coaster coaster-big rarity-${esc(overlay.coaster)}"><div class="coaster-kind">${esc(overlay.subtitle)}</div><h4 class="coaster-name">${esc(overlay.title)}</h4><p class="coaster-text">${esc(overlay.text)}</p></article>`
        : `${pic}<p class="small">${esc(overlay.subtitle)}</p><h2>${esc(overlay.title)}</h2><p>${esc(overlay.text)}</p>${overlay.odds && view ? oddsLine(view.deal_odds) : ''}`;
      return sheet(
        `${body}
         <div class="sheet-actions">
           ${overlay.use !== undefined ? `<button class="btn btn-primary" ${attrs({ 'data-action': 'use-item', 'data-slot': overlay.use })}>${esc(de.shop.use)}</button>` : ''}
           ${overlay.buy ? `<button class="btn btn-primary" ${attrs({ 'data-action': overlay.buy.action, 'data-slot': overlay.buy.slot })}>${esc(t(de.shop.buy, { price: overlay.buy.price }))}</button>` : ''}
           <button class="btn" ${attrs({ 'data-action': 'close-overlay' })}>${esc(de.app.close)}</button>
         </div>`,
      );
    }
    case 'menu':
      return sheet(
        `<h2>${esc(de.header.menu)}</h2>
         ${settingsBlock(ui)}
         <div class="menu-list">
           ${view ? `<button class="btn btn-wide" ${attrs({ 'data-action': 'overview' })}>${esc(de.menu.overview)}</button>` : ''}
           ${ui.meta ? `<button class="btn btn-wide" ${attrs({ 'data-action': 'book' })}>${esc(de.book.title)}</button>` : ''}
           <button class="btn btn-wide" ${attrs({ 'data-action': 'credits' })}>${esc(de.menu.credits)}</button>
           ${view && ui.screen === 'run' ? `<button class="btn btn-wide" ${attrs({ 'data-action': 'to-start' })}>${esc(de.end.toStart)}</button>` : ''}
           <button class="btn btn-wide" ${attrs({ 'data-action': 'close-overlay' })}>${esc(de.app.close)}</button>
         </div>
         ${view ? `<p class="small">${esc(t(de.header.seed, { seed: view.seed }))}</p>` : ''}`,
      );
    case 'overview':
      if (!view) return '';
      return sheet(
        `<h2>${esc(de.overview.title)}</h2>
         ${renderOverview(view, ui.meta)}
         <button class="btn btn-wide" ${attrs({ 'data-action': 'close-overlay' })}>${esc(de.app.close)}</button>`,
        true,
        'chalkboard sheet-overview',
      );
    case 'credits':
      return sheet(credits(), true, 'sheet-credits');
    case 'target': {
      const item = view?.consumables.find((c) => c.slot === overlay.slot);
      if (!view || !item) return '';
      const cards = view.deck
        .map((c) => {
          const ok = item.targets.includes(c.id);
          return `<button class="card target-card ${ok ? '' : 'is-dim'}" ${attrs({
            'data-action': 'target',
            'data-id': c.id,
            disabled: !ok,
            'aria-label': cardName(c.kind),
          })}>${cardSvg(c.kind, c.mods)}</button>`;
        })
        .join('');
      return sheet(
        `<h2>${esc(de.shop.pickTarget)}</h2>
         <p class="small">${esc(t(de.shop.pickHint, { name: contentName('consumables', item.consumable.id), text: contentText('consumables', item.consumable.id) }))}</p>
         ${oddsLine(view.deal_odds)}
         <div class="target-grid">${cards}</div>
         <button class="btn btn-wide" ${attrs({ 'data-action': 'close-overlay' })}>${esc(de.shop.cancel)}</button>`,
      );
    }
    case 'encounter-target': {
      const option = view?.encounter?.options.find((o) => o.index === overlay.option);
      if (!view || !option) return '';
      const cards = view.deck
        .map((c) => {
          const ok = option.targets.includes(c.id);
          return `<button class="card target-card ${ok ? '' : 'is-dim'}" ${attrs({
            'data-action': 'encounter-target',
            'data-id': c.id,
            disabled: !ok,
            'aria-label': cardName(c.kind),
          })}>${cardSvg(c.kind, c.mods)}</button>`;
        })
        .join('');
      return sheet(
        `<h2>${esc(de.encounter.pickTarget)}</h2>
         <p class="small">${esc(contentText('encounters', view.encounter?.id ?? ''))}</p>
         ${oddsLine(view.deal_odds)}
         <div class="target-grid">${cards}</div>
         <button class="btn btn-wide" ${attrs({ 'data-action': 'close-overlay' })}>${esc(de.shop.cancel)}</button>`,
      );
    }
    case 'book':
      return ui.meta ? sheet(renderBook(ui.meta), true, 'ledger', 'overlay-book') : '';
    case 'unlock':
      return ui.meta ? sheet(renderUnlocks(ui.meta, overlay.ids), false, 'sheet-unlock', 'overlay-unlock') : '';
    case 'score':
      return '';
  }
}

/** An open book, drawn. */
function bookIcon(): string {
  return `<svg class="book-icon" viewBox="0 0 32 24" width="32" height="24" aria-hidden="true"><path d="M16 5 C12 2 6 2 2 3 L2 21 C6 20 12 20 16 22 C20 20 26 20 30 21 L30 3 C26 2 20 2 16 5 Z" fill="#7a2a1e" stroke="#2b1a0e" stroke-width="1.4"/><path d="M16 5 C12 3 7 3.5 4 4.2 L4 19 C8 18.4 12 18.6 16 20 Z M16 5 C20 3 25 3.5 28 4.2 L28 19 C24 18.4 20 18.6 16 20 Z" fill="#fbf3e2" stroke="#2b1a0e" stroke-width="1"/><path d="M7 8 L13 8.6 M7 11 L13 11.6 M7 14 L12 14.4 M19 8.6 L25 8 M19 11.6 L25 11 M19 14.4 L24 14" stroke="#8a7a66" stroke-width=".9"/></svg>`;
}

/** Display name of a reward: the relic's name or the character's. */
function rewardName(reward: Reward, meta: MetaView): string {
  if ('Relic' in reward) return contentName('relics', reward.Relic);
  const c = meta.characters.find((x) => x.character === reward.Character);
  return t(de.book.rewardCharacter, { name: c ? characterName(c) : contentName('characters', reward.Character) });
}

/** A stamp in the ledger: done or open. */
function stamp(done: boolean): string {
  return done
    ? `<span class="ledger-stamp is-done" aria-label="${esc(de.book.done)}"><svg viewBox="0 0 40 30" aria-hidden="true"><path d="M6 16 L15 24 L34 5"/></svg><span>${esc(de.book.done)}</span></span>`
    : `<span class="ledger-stamp is-open" aria-label="${esc(de.book.open)}"><span class="ledger-box"></span><span>${esc(de.book.open)}</span></span>`;
}

function unlockRow(u: UnlockEntry, meta: MetaView): string {
  const rewards = u.rewards.map((r) => rewardName(r, meta)).join(', ');
  return `<li class="ledger-row ${u.unlocked ? 'is-done' : 'is-open'}" ${attrs({ 'data-unlock': u.id })}>
    ${stamp(u.unlocked)}
    <div class="ledger-entry">
      <b>${esc(contentName('unlocks', u.id) || u.name)}</b>
      <span class="ledger-cond">${esc(contentText('unlocks', u.id) || u.text)}</span>
      <span class="ledger-reward"><span>${esc(de.book.reward)}:</span> ${esc(rewards)}</span>
    </div>
  </li>`;
}

/** The Vereinsbuch: who sits at the table, every unlock with its reward, the statistics. */
export function renderBook(meta: MetaView): string {
  const s = meta.stats;
  const done = meta.unlocks.filter((u) => u.unlocked).length;
  const chars = meta.characters
    .map((c) => {
      const st = s.characters[c.character];
      const line = c.unlocked
        ? t(de.book.perCharacter, { runs: st?.runs ?? 0, won: st?.won ?? 0, best: st?.best_section ?? 0 })
        : t(de.book.lockedHint, { text: unlockText(meta, c.unlocked_by) });
      return `<li class="ledger-char ${c.unlocked ? '' : 'is-locked'}">${portrait(characterPortraitId(c.character), 'portrait-sm')}
        <div><b>${esc(characterName(c))}</b>${c.unlocked ? '' : lockIcon(16)}<span class="ledger-cond">${esc(line)}</span></div></li>`;
    })
    .join('');
  const rows = meta.unlocks.map((u) => unlockRow(u, meta)).join('');
  const stats: [string, number][] = [
    [de.book.runs, s.runs_finished],
    [de.book.runsWon, s.runs_won],
    [de.book.bestSection, s.best_section],
    [de.book.gamesWon, s.games_won],
    [de.book.gamesLost, s.games_lost],
    [de.book.bestGame, s.best_game],
  ];
  return `<header class="ledger-head"><h2>${esc(de.book.title)}</h2><p class="ledger-sub">${esc(de.book.subtitle)}</p></header>
    <h3 class="ledger-h">${esc(de.book.characters)}</h3>
    <ul class="ledger-chars">${chars}</ul>
    <h3 class="ledger-h">${esc(de.book.unlocks)} <span class="ledger-progress">${esc(t(de.book.progress, { done, total: meta.unlocks.length }))}</span></h3>
    <ol class="ledger-list" data-testid="unlocks">${rows}</ol>
    ${meta.locked_relics.length ? `<p class="ledger-note">${esc(t(de.book.lockedRelics, { count: meta.locked_relics.length }))}</p>` : ''}
    <h3 class="ledger-h">${esc(de.book.stats)}</h3>
    <dl class="ledger-stats">${stats.map(([k, v]) => `<dt>${esc(k)}</dt><dd>${esc(v)}</dd>`).join('')}</dl>
    <button class="btn btn-wide" ${attrs({ 'data-action': 'close-overlay' })}>${esc(de.app.close)}</button>`;
}

/** Rays of light behind a new reward. */
function burst(): string {
  return `<span class="unlock-burst" aria-hidden="true">${Array.from({ length: 12 }, (_, i) => `<i style="--i:${i}"></i>`).join('')}</span>`;
}

/** A Bierdeckel with a foaming glass: a new relic for the Tresen. */
function coasterGlass(): string {
  return `<svg viewBox="0 0 40 40" width="46" height="46" aria-hidden="true"><path d="M12 12 L28 12 L26.5 33 L13.5 33 Z" fill="#e8a417" stroke="#2b1a0e" stroke-width="1.6"/><path d="M10.5 12.5 C10 7.5 15 7 17 9 C19 6 24 6 25.5 9 C28.5 7.5 31 10 29.5 12.5 Z" fill="#fffaf0" stroke="#2b1a0e" stroke-width="1.6"/><path d="M28 17 C33 17 33 27 27.2 27" fill="none" stroke="#2b1a0e" stroke-width="1.8"/><path d="M16 16 L15.6 30" stroke="#fff" stroke-width="1.3" opacity=".5"/></svg>`;
}

/** Festive notice of unlocks just earned. */
function renderUnlocks(meta: MetaView, ids: string[]): string {
  const items = ids
    .map((id) => meta.unlocks.find((u) => u.id === id))
    .filter((u): u is UnlockEntry => u !== undefined)
    .map((u) => {
      const rewards = u.rewards
        .map((r) => {
          const pic =
            'Character' in r
              ? portrait(characterPortraitId(r.Character), 'portrait-lg')
              : `<span class="unlock-coaster">${coasterGlass()}</span>`;
          const text = 'Character' in r ? contentText('characters', r.Character) : contentText('relics', r.Relic);
          return `<div class="unlock-reward"><div class="unlock-pic">${burst()}${pic}</div><b>${esc(rewardName(r, meta))}</b><span class="small">${esc(text)}</span></div>`;
        })
        .join('');
      const line = t(de.unlock.line, { name: u.rewards.map((r) => rewardName(r, meta)).join(', ') });
      return `<li class="unlock-item" ${attrs({ 'data-unlock': u.id, 'aria-label': line })}>
        ${rewards}
        <p class="small">${esc(t(de.unlock.for, { name: contentName('unlocks', u.id) || u.name, text: contentText('unlocks', u.id) || u.text }))}</p>
      </li>`;
    })
    .join('');
  return `<h2 class="unlock-title">${esc(de.unlock.title)}</h2>
    <ul class="unlock-list" data-testid="unlock-new">${items}</ul>
    <button class="btn btn-primary btn-wide" ${attrs({ 'data-action': 'close-overlay' })}>${esc(de.unlock.go)}</button>`;
}
