// Start and end screens and the dialogs: Kontra, boss, info, menu with
// settings, Kreidetafel, Vereinsbuch with its statistics page, new unlocks,
// Abspann, target picking, sharing a seed, giving up a running run and the
// save code (./savegame.ts).

import { de, cardName, contentName, contentText, face, relicLook, t } from '../i18n';
import { dailyLabel, dailySeed, shareUrl } from '../seed';
import { ladderStarted, maxStake, pickedStake, winThreshold } from '../stake';
import type { RunRecord } from '../storage';
import type { Character, CharacterEntry, MetaView, Reward, RewardInfo, RunLength, RunView, Suit, UnlockEntry } from '../types';
import { cardSvg, modIcon, suitIcon } from './cards';
import { attrs, esc, reducedMotion } from './dom';
import { bossCoaster, relicCoaster } from './emblems';
import { seatPerson } from './guests';
import { oddsLine } from './odds';
import { calendarIcon, linkIcon, lockIcon } from './icons';
import { renderOverview } from './overview';
import { renderRules } from './rules';
import { bossPortraitId, characterPortraitId, portrait } from './people';
import { renderSaveConfirm, renderSaveExport, renderSaveImport, renderSaveMenu } from './savegame';
import type { Overlay, UiState } from './state';

export { lockIcon };

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

/** Short name of a character ("Alleinspieler"). */
function shortName(character: Character): string {
  return de.characters[character]?.short ?? contentName('characters', character);
}

/** "Seed 123 · Alleinspieler", with "· kurzer Abend" for a short evening and "· Einsatz 2" for a stake. */
function seedLine(seed: string, character: Character, length: RunLength, stake = 0): string {
  const line = t(length === 'Short' ? de.share.lineShort : de.share.line, { seed, character: shortName(character) });
  return stake > 0 ? line + t(de.share.lineStake, { stake }) : line;
}

/** Text for the system share sheet. */
function nativeText(seed: string, character: Character, length: RunLength, stake = 0): string {
  const text = t(length === 'Short' ? de.share.nativeTextShort : de.share.nativeText, { seed, character: shortName(character) });
  return stake > 0 ? text + t(de.share.nativeStake, { stake, augen: winThreshold(stake) }) : text;
}

/**
 * Einsatz-Leiter (DESIGN 9.3): one row with arrows, "Einsatz 2 · Gewinn ab
 * 63 Augen". Shown once any character has won a run; before that every
 * stake is 0 and the row would only take room.
 */
function stakePick(ui: UiState): string {
  if (!ladderStarted(ui.meta)) return '';
  const max = maxStake(ui.meta, ui.pickCharacter, ui.pickLength);
  const stake = pickedStake(ui.meta, ui.pickCharacter, ui.pickLength, ui.pickStake);
  const augen = winThreshold(stake);
  const arrow = (to: number, label: string, path: string): string =>
    `<button class="seg stake-arrow" ${attrs({
      'data-action': 'pick-stake',
      'data-stake': to,
      'aria-label': label,
      disabled: to < 0 || to > max,
    })}><svg viewBox="0 0 10 10" width="12" height="12" aria-hidden="true"><path d="${path}" fill="currentColor"/></svg></button>`;
  return `<div class="stake-pick" role="group" data-testid="stake" ${attrs({
    'data-stake': stake,
    'data-max': max,
    'aria-label': t(de.start.stakeAria, { stake, max, augen }),
  })}>
    ${arrow(stake - 1, de.start.stakeDown, 'M7 0 L7 10 L1 5 Z')}
    <span class="stake-text"><b>${esc(t(de.start.stake, { stake }))}</b><span>${esc(t(de.start.stakeWin, { augen }))}</span></span>
    ${arrow(stake + 1, de.start.stakeUp, 'M3 0 L3 10 L9 5 Z')}
  </div>`;
}

/** Ganzer or kurzer Abend (DESIGN 5.3): two buttons side by side. */
function lengthPick(ui: UiState): string {
  const buttons = (['Full', 'Short'] as const)
    .map((length) => {
      const text = de.start.lengths[length];
      const picked = ui.pickLength === length;
      return `<button class="seg length-opt ${picked ? 'is-picked' : ''}" ${attrs({
        'data-action': 'pick-length',
        'data-length': length,
        'aria-pressed': picked ? 'true' : 'false',
        'aria-label': text.aria,
      })}><b>${esc(text.name)}</b><span>${esc(text.sections)}</span></button>`;
    })
    .join('');
  return `<div class="segmented length-pick" role="group" data-testid="run-length" ${attrs({ 'aria-label': de.start.howLong, 'data-length': ui.pickLength })}>${buttons}</div>`;
}

/** A seed from a shared link: play it or not. A running run stays until the player decides. */
function sharedOffer(ui: UiState): string {
  const shared = ui.shared;
  if (!shared) return '';
  const name = shortName(shared.character);
  const locked = shared.locked
    ? `<p class="small is-warning">${esc(t(de.start.sharedLocked, { character: name, fallback: shortName('Declarer') }))}</p>`
    : '';
  const playable = sharedStake(ui);
  const stake =
    shared.stake > 0
      ? `<p class="small">${esc(t(de.start.sharedStake, { stake: shared.stake, augen: winThreshold(shared.stake) }))}</p>${
          playable < shared.stake ? `<p class="small is-warning">${esc(t(de.start.sharedStakeLocked, { stake: shared.stake, fallback: playable }))}</p>` : ''
        }`
      : '';
  return `<section class="shared-offer" data-testid="shared-seed" ${attrs({ 'data-seed': shared.seed, 'data-character': shared.character, 'data-length': shared.length, 'data-stake': playable })}>
    <div class="shared-head">${linkIcon(20)}<h2>${esc(de.start.sharedTitle)}</h2></div>
    <p>${esc(t(shared.length === 'Short' ? de.start.sharedTextShort : de.start.sharedText, { seed: shared.seed, character: name }))}</p>
    ${stake}
    ${locked}
    ${ui.saveInfo ? `<p class="small">${esc(de.start.sharedKeep)}</p>` : ''}
    <div class="sheet-actions">
      <button class="btn btn-primary" ${attrs({ 'data-action': 'play-shared' })}>${esc(de.start.sharedPlay)}</button>
      <button class="btn" ${attrs({ 'data-action': 'dismiss-shared' })}>${esc(de.start.sharedDismiss)}</button>
    </div>
  </section>`;
}

/**
 * The stake a shared link is played with: its own, but at most the highest
 * this player has unlocked for the character that actually plays.
 */
export function sharedStake(ui: UiState): number {
  const shared = ui.shared;
  if (!shared) return 0;
  const character = shared.locked ? 'Declarer' : shared.character;
  return Math.min(shared.stake, maxStake(ui.meta, character, shared.length));
}

/** Result of a recorded run in a few words. */
function runResult(r: RunRecord): string {
  return r.won ? de.book.recentWon : t(de.book.recentLost, { section: r.section, series: r.series });
}

/** The daily seed: the same run for everyone today. */
function dailyBlock(ui: UiState): string {
  if (!ui.meta) return '';
  const today = new Date();
  const seed = dailySeed(today);
  const played = ui.history.find((r) => r.daily && r.seed === seed);
  return `<section class="daily" data-testid="daily" ${attrs({ 'data-seed': seed })}>
    ${calendarIcon(today.getDate(), 40)}
    <div class="daily-text">
      <b>${esc(de.start.dailyTitle)}</b> <span>${esc(t(de.start.dailyDate, { date: dailyLabel(today) }))}</span>
      <span class="small">${esc(de.start.dailyText)} ${esc(seedLine(seed, ui.pickCharacter, ui.pickLength, pickedStake(ui.meta, ui.pickCharacter, ui.pickLength, ui.pickStake)))}</span>
      ${played ? `<span class="small daily-done">${esc(t(de.start.dailyDone, { result: runResult(played) }))}</span>` : ''}
    </div>
    <button class="btn btn-wide" ${attrs({ 'data-action': 'play-daily' })}>${esc(t(de.start.dailyPlay, { name: shortName(ui.pickCharacter) }))}</button>
  </section>`;
}

/**
 * The seed of a run to pass on: visible and selectable, a button that
 * copies the link, the system share sheet where there is one. If copying
 * fails, the link shows in a field to copy by hand.
 */
export function shareBlock(seed: string, character: Character, length: RunLength, ui: UiState, stake = 0): string {
  const url = shareUrl(seed, character, length, stake);
  const name = shortName(character);
  const copy = ui.copy && ui.copy.url === url ? ui.copy : null;
  const status = copy
    ? copy.ok
      ? `<p class="share-ok" role="status" data-testid="share-copied">${esc(de.share.copied)}</p>`
      : `<p class="small" role="status">${esc(de.share.copyFailed)}</p><input class="share-fallback" data-testid="share-fallback" readonly ${attrs({ value: url, 'aria-label': de.share.link })}>`
    : '';
  let native = '';
  try {
    if (typeof navigator.share === 'function') {
      native = `<button class="btn" ${attrs({ 'data-action': 'share-native', 'data-url': url, 'data-text': nativeText(seed, character, length, stake) })}>${esc(de.share.native)}</button>`;
    }
  } catch {
    native = '';
  }
  return `<section class="share" data-testid="share">
    <div class="share-head">${linkIcon(18)}<b>${esc(de.share.title)}</b></div>
    <p class="share-seed">${esc(de.end.seed)} <span class="seed-num" data-testid="run-seed">${esc(seed)}</span> · ${esc(name)}${length === 'Short' ? ` · ${esc(de.start.lengths.Short.name)}` : ''}${stake > 0 ? esc(t(de.share.lineStake, { stake })) : ''}</p>
    <p class="small">${esc(de.share.hint)}</p>
    <div class="share-actions">
      <button class="btn btn-primary" ${attrs({ 'data-action': 'copy-link', 'data-url': url, 'data-testid': 'copy-link' })}>${linkIcon(16)}<span>${esc(de.share.copy)}</span></button>
      ${native}
    </div>
    ${status}
  </section>`;
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
    ${sharedOffer(ui)}
    <div class="start-actions">
      ${resume}
    </div>
    ${chars ? `<section class="char-pick" aria-label="${esc(de.start.whoPlays)}"><h2 class="char-title">${esc(de.start.whoPlays)}</h2><div class="char-cards" data-testid="characters">${chars}</div></section>` : ''}
    ${meta ? lengthPick(ui) : ''}
    ${meta ? stakePick(ui) : ''}
    <div class="start-actions">
      <button class="btn ${ui.saveInfo ? '' : 'btn-primary'} btn-wide" ${attrs({ 'data-action': 'new-run' })}>${esc(newLabel)}</button>
      <label class="seed"><span>${esc(de.start.seedLabel)}</span>
        <input id="seed" name="seed" inputmode="numeric" autocomplete="off" placeholder="${esc(de.start.seedPlaceholder)}"></label>
    </div>
    ${dailyBlock(ui)}
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
      ${shareBlock(view.seed, view.character.character, view.length, ui, view.stake ?? 0)}
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
  const kontraSeat = typeof kontra === 'object' && 'Kontra' in kontra ? kontra.Kontra : 1;
  const { face, name } = seatPerson(view, kontraSeat === 2 ? 2 : 1);
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

/** A segmented on/off switch for a setting. */
function onOff(action: string, on: boolean): string {
  return `<div class="segmented">
      <button class="seg ${on ? 'is-picked' : ''}" ${attrs({ 'data-action': action, 'data-on': 'true', 'aria-pressed': on ? 'true' : 'false' })}>${esc(de.menu.soundOn)}</button>
      <button class="seg ${!on ? 'is-picked' : ''}" ${attrs({ 'data-action': action, 'data-on': 'false', 'aria-pressed': !on ? 'true' : 'false' })}>${esc(de.menu.soundOff)}</button>
    </div>`;
}

/** A toggle that names its setting and state ("Musik: An"); a tap flips it. */
function toggle(action: string, name: string, on: boolean): string {
  return `<button class="seg seg-toggle ${on ? 'is-picked' : ''}" ${attrs({
    'data-action': action,
    'data-on': on ? 'false' : 'true',
    'data-testid': `toggle-${action}`,
    'aria-pressed': on ? 'true' : 'false',
    'aria-label': name,
  })}>${esc(t(on ? de.menu.toggleOn : de.menu.toggleOff, { name }))}</button>`;
}

/** The settings as tight rows: label left, switch right. */
function settingsBlock(ui: UiState): string {
  const tempos = (['slow', 'normal', 'fast'] as const)
    .map(
      (k) =>
        `<button class="seg ${ui.settings.tempo === k ? 'is-picked' : ''}" ${attrs({ 'data-action': 'tempo', 'data-tempo': k, 'aria-pressed': ui.settings.tempo === k ? 'true' : 'false' })}>${esc(de.menu.tempos[k])}</button>`,
    )
    .join('');
  return `<div class="menu-settings">
    <div class="menu-row"><span class="setting-label">${esc(de.menu.tempo)}</span><div class="segmented">${tempos}</div></div>
    ${reducedMotion() ? `<p class="small menu-note">${esc(de.menu.reducedMotion)}</p>` : ''}
    <div class="menu-row"><span class="setting-label">${esc(de.menu.sound)}</span><div class="segmented">${toggle('sound', de.menu.effects, ui.settings.sound)}${toggle('music', de.menu.music, ui.settings.music)}</div></div>
    <div class="menu-row" ${attrs({ title: de.menu.thinkingHint })}><span class="setting-label">${esc(de.menu.thinking)}</span>${onOff('thinking', ui.settings.thinking)}</div>
    <div class="menu-row menu-hints"><span class="setting-label">${esc(de.menu.hints)}</span>${onOff('hints', ui.settings.hints)}<button class="btn" ${attrs({
      'data-action': 'hints-reset',
      'data-testid': 'hints-reset',
      'aria-label': de.menu.hintsResetAria,
      title: de.menu.hintsResetAria,
    })}>${esc(de.menu.hintsReset)}</button></div>
  </div>`;
}

/** The seed of the running run as one row with "Link kopieren" (menu). */
function seedRow(seed: string, character: Character, length: RunLength, ui: UiState, stake = 0): string {
  const url = shareUrl(seed, character, length, stake);
  const copy = ui.copy && ui.copy.url === url ? ui.copy : null;
  const status = copy
    ? copy.ok
      ? `<p class="share-ok" role="status" data-testid="share-copied">${esc(de.share.copied)}</p>`
      : `<p class="small" role="status">${esc(de.share.copyFailed)}</p><input class="share-fallback" data-testid="share-fallback" readonly ${attrs({ value: url, 'aria-label': de.share.link })}>`
    : '';
  let native = '';
  try {
    if (typeof navigator.share === 'function') {
      native = `<button class="btn btn-icon" ${attrs({
        'data-action': 'share-native',
        'data-url': url,
        'data-text': nativeText(seed, character, length, stake),
        'aria-label': de.share.native,
        title: de.share.native,
      })}>${linkIcon(18)}</button>`;
    }
  } catch {
    native = '';
  }
  return `<div class="menu-seed" data-testid="share">
    <div class="menu-row">
      <span class="setting-label">${esc(de.end.seed)}</span>
      <span class="menu-seed-num"><span class="seed-num" data-testid="run-seed">${esc(seed)}</span></span>
      <span class="menu-seed-actions">
        <button class="btn" ${attrs({ 'data-action': 'copy-link', 'data-url': url, 'data-testid': 'copy-link', 'aria-label': de.share.copy, title: de.share.copy })}>${linkIcon(16)}<span>${esc(de.share.copyShort)}</span></button>
        ${native}
      </span>
    </div>
    ${status}
  </div>`;
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
         <p class="small boss-kicker">${bossCoaster(overlay.boss.id, { size: 32 })}<span>${esc(de.boss.title)} · ${esc(de.boss.intro)}</span></p>
         <h2>${esc(contentName('bosses', overlay.boss.id))}</h2>
         <p>${esc(contentText('bosses', overlay.boss.id) || overlay.boss.text)}</p></div>
         <button class="btn btn-primary btn-wide" ${attrs({ 'data-action': 'boss-go' })}>${esc(de.boss.go)}</button>`,
        false,
        'sheet-boss',
      );
    case 'menu':
      return sheet(
        `<h2>${esc(de.header.menu)}</h2>
         ${settingsBlock(ui)}
         ${view && ui.screen === 'run' ? seedRow(view.seed, view.character.character, view.length, ui, view.stake ?? 0) : ''}
         <div class="menu-list menu-grid">
           <button class="btn" ${attrs({ 'data-action': 'rules', 'data-testid': 'rules-open' })}>${esc(de.menu.rules)}</button>
           ${view ? `<button class="btn" ${attrs({ 'data-action': 'overview' })}>${esc(de.menu.overview)}</button>` : ''}
           ${ui.meta ? `<button class="btn" ${attrs({ 'data-action': 'book' })}>${esc(de.book.title)}</button>` : ''}
           <button class="btn" ${attrs({ 'data-action': 'save-menu', 'data-testid': 'save-menu' })}>${esc(de.save.title)}</button>
           <button class="btn" ${attrs({ 'data-action': 'credits' })}>${esc(de.menu.credits)}</button>
           ${view && ui.screen === 'run' ? `<button class="btn" ${attrs({ 'data-action': 'to-start' })}>${esc(de.end.toStart)}</button>` : ''}
           <button class="btn menu-close" ${attrs({ 'data-action': 'close-overlay', 'data-testid': 'menu-close' })}>${esc(de.app.close)}</button>
         </div>`,
        true,
        'sheet-menu',
      );
    case 'save-menu':
      return renderSaveMenu(sheet);
    case 'save-export':
      return renderSaveExport(sheet, overlay.code, overlay.copied);
    case 'save-import':
      return renderSaveImport(sheet, overlay.text, overlay.error, overlay.checking);
    case 'save-confirm':
      return renderSaveConfirm(sheet, overlay.lines);
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
    case 'rules':
      return sheet(renderRules(), true, 'sheet-rules');
    case 'target': {
      const item = view?.consumables.find((c) => c.slot === overlay.slot);
      if (!view || !item) return '';
      const cards = view.deck
        .map((c) => {
          const ok = item.targets.includes(c.id);
          return `<button class="card target-card ${ok ? '' : 'is-dim'}" ${attrs({
            'data-action': 'target',
            'data-id': c.id,
            'data-info': `card:${c.id}`,
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
    case 'chalk': {
      const item = view?.consumables.find((c) => c.slot === overlay.slot);
      const hand = view?.game?.hand;
      if (!view || !item || !hand) return '';
      const picked = hand.find((c) => c.id === overlay.card) ?? null;
      const cards = hand
        .map((c) => {
          const ok = item.targets.includes(c.id);
          const on = c.id === overlay.card;
          return `<button class="card target-card ${ok ? '' : 'is-dim'} ${on ? 'is-picked' : ''}" ${attrs({
            'data-action': 'chalk-card',
            'data-id': c.id,
            disabled: !ok,
            'aria-pressed': on ? 'true' : 'false',
            'aria-label': cardName(c.kind),
          })}>${cardSvg(c.kind, c.mods)}</button>`;
        })
        .join('');
      const own = picked ? face(picked.kind)?.suit : undefined;
      const suits = (['Clubs', 'Spades', 'Hearts', 'Diamonds'] as Suit[])
        .map(
          (suit) =>
            `<button class="btn chalk-suit" ${attrs({ 'data-action': 'chalk-suit', 'data-suit': suit, disabled: !picked || suit === own })}>${suitIcon(suit, 20, null)}<span>${esc(de.suits[suit])}</span></button>`,
        )
        .join('');
      return sheet(
        `<h2>${esc(contentName('consumables', item.consumable.id))}</h2>
         <p class="small">${esc(picked ? t(de.shop.chalkSuit, { card: cardName(picked.kind) }) : de.shop.chalkCard)}</p>
         <div class="target-grid chalk-grid" data-testid="chalk-cards">${cards}</div>
         <div class="chalk-suits" data-testid="chalk-suits">${suits}</div>
         <button class="btn btn-wide" ${attrs({ 'data-action': 'close-overlay' })}>${esc(de.shop.cancel)}</button>`,
        true,
        'sheet-chalk',
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
            'data-info': `card:${c.id}`,
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
      return ui.meta ? sheet(renderBook(ui.meta, overlay.tab, ui.history), true, 'ledger', 'overlay-book') : '';
    case 'confirm-new':
      return sheet(
        `<h2>${esc(de.confirm.title)}</h2>
         <p>${esc(de.confirm.text)}</p>
         ${ui.saveInfo ? `<p class="confirm-run"><b>${esc(ui.saveInfo)}</b></p>` : ''}
         <p class="small">${esc(seedLine(overlay.start.seed, overlay.start.character, overlay.start.length, overlay.start.stake))}</p>
         <div class="sheet-actions">
           <button class="btn btn-primary" ${attrs({ 'data-action': 'confirm-new' })}>${esc(de.confirm.go)}</button>
           <button class="btn" ${attrs({ 'data-action': 'close-overlay' })}>${esc(de.confirm.cancel)}</button>
         </div>`,
        true,
        'sheet-confirm',
      );
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

/** Display name of a reward: the relic's, the modification's or the character's. */
function rewardName(reward: Reward, meta: MetaView): string {
  if ('Relic' in reward) return contentName('relics', reward.Relic);
  if ('CardMod' in reward) return contentName('card_mods', reward.CardMod);
  const c = meta.characters.find((x) => x.character === reward.Character);
  return t(de.book.rewardCharacter, { name: c ? characterName(c) : contentName('characters', reward.Character) });
}

/** A stamp in the ledger: done or open. */
function stamp(done: boolean): string {
  return done
    ? `<span class="ledger-stamp is-done" aria-label="${esc(de.book.done)}"><svg viewBox="0 0 40 30" aria-hidden="true"><path d="M6 16 L15 24 L34 5"/></svg><span>${esc(de.book.done)}</span></span>`
    : `<span class="ledger-stamp is-open" aria-label="${esc(de.book.open)}"><span class="ledger-box"></span><span>${esc(de.book.open)}</span></span>`;
}

/** Name of a reward from the core's reward info, German-deck override first. */
function infoName(info: RewardInfo, meta: MetaView): string {
  return rewardName(info.reward, meta) || info.name;
}

/** Text of a reward from the core's reward info, German-deck override first. */
function infoText(info: RewardInfo): string {
  const r = info.reward;
  const own = 'Relic' in r ? contentText('relics', r.Relic) : 'CardMod' in r ? contentText('card_mods', r.CardMod) : contentText('characters', r.Character);
  return own || info.text;
}

/** "Hausregel, selten", "Charakter". */
function infoKind(info: RewardInfo): string {
  const kind = de.rewardKinds[info.kind];
  return info.rarity ? t(de.book.rewardKind, { kind, rarity: de.rarity[info.rarity] }) : kind;
}

/** Names of the rewards of an unlock, with kind and rarity. */
function rewardsLine(u: UnlockEntry, meta: MetaView): string {
  if (!u.reward_info?.length) return u.rewards.map((r) => rewardName(r, meta)).join(', ');
  return u.reward_info.map((i) => `${infoName(i, meta)} (${infoKind(i)})`).join(', ');
}

function unlockRow(u: UnlockEntry, meta: MetaView): string {
  const rewards = rewardsLine(u, meta);
  return `<li class="ledger-row ${u.unlocked ? 'is-done' : 'is-open'}" ${attrs({ 'data-unlock': u.id })}>
    ${stamp(u.unlocked)}
    <div class="ledger-entry">
      <b>${esc(contentName('unlocks', u.id) || u.name)}</b>
      <span class="ledger-cond">${esc(contentText('unlocks', u.id) || u.text)}</span>
      <span class="ledger-reward"><span>${esc(de.book.reward)}:</span> ${esc(rewards)}</span>
    </div>
  </li>`;
}

/** The two pages of the Vereinsbuch. */
function bookTabs(tab: 'book' | 'stats'): string {
  const btn = (key: 'book' | 'stats'): string =>
    `<button class="seg ${tab === key ? 'is-picked' : ''}" ${attrs({ 'data-action': 'book-tab', 'data-tab': key, 'aria-pressed': tab === key ? 'true' : 'false' })}>${esc(de.book.tabs[key])}</button>`;
  return `<div class="segmented ledger-tabs" role="group">${btn('book')}${btn('stats')}</div>`;
}

/** Who sits at the table and every unlock with its reward. */
function bookPage(meta: MetaView): string {
  const done = meta.unlocks.filter((u) => u.unlocked).length;
  const chars = meta.characters
    .map((c) => {
      const line = c.unlocked ? (de.characters[c.character]?.role ?? '') : t(de.book.lockedHint, { text: unlockText(meta, c.unlocked_by) });
      return `<li class="ledger-char ${c.unlocked ? '' : 'is-locked'}">${portrait(characterPortraitId(c.character), 'portrait-sm')}
        <div><b>${esc(characterName(c))}</b>${c.unlocked ? '' : lockIcon(16)}<span class="ledger-cond">${esc(line)}</span></div></li>`;
    })
    .join('');
  const rows = meta.unlocks.map((u) => unlockRow(u, meta)).join('');
  const lockedMods = meta.locked_card_mods ?? [];
  return `<h3 class="ledger-h">${esc(de.book.characters)}</h3>
    <ul class="ledger-chars">${chars}</ul>
    <h3 class="ledger-h">${esc(de.book.unlocks)} <span class="ledger-progress">${esc(t(de.book.progress, { done, total: meta.unlocks.length }))}</span></h3>
    <ol class="ledger-list" data-testid="unlocks">${rows}</ol>
    ${meta.locked_relics.length ? `<p class="ledger-note">${esc(t(de.book.lockedRelics, { count: meta.locked_relics.length }))}</p>` : ''}
    ${lockedMods.length ? `<p class="ledger-note">${esc(t(de.book.lockedMods, { count: lockedMods.length }))}</p>` : ''}`;
}

/** One finished run of this browser. */
function recentRow(r: RunRecord, meta: MetaView): string {
  const entry = meta.characters.find((c) => c.character === r.character);
  const date = new Date(r.date);
  const when = Number.isNaN(date.getTime()) ? '' : date.toLocaleDateString('de-DE', { day: '2-digit', month: '2-digit', year: '2-digit' });
  const replay = entry?.unlocked
    ? `<button class="btn btn-small" ${attrs({ 'data-action': 'replay', 'data-seed': r.seed, 'data-character': r.character, 'data-length': r.short ? 'Short' : 'Full', 'data-stake': r.stake ?? 0, 'aria-label': t(de.book.replayAria, { seed: r.seed, character: shortName(r.character) }) })}>${esc(de.book.replay)}</button>`
    : '';
  return `<li class="recent-row ${r.won ? 'is-won' : ''}" ${attrs({ 'data-seed': r.seed })}>
    ${portrait(characterPortraitId(r.character), 'portrait-xs')}
    <div class="recent-main">
      <b>${esc(shortName(r.character))}</b>${r.short ? `<span class="recent-daily recent-short">${esc(de.book.short)}</span>` : ''}${r.stake ? `<span class="recent-daily recent-short">${esc(t(de.book.stake, { stake: r.stake }))}</span>` : ''}${r.daily ? `<span class="recent-daily">${esc(de.book.daily)}</span>` : ''}
      <span class="recent-result">${esc(runResult(r))} · ${esc(t(de.book.recentBest, { n: r.bestGame }))}</span>
      <span class="recent-seed">${esc(when)} · ${esc(t(de.book.recentSeed, { seed: r.seed }))}</span>
    </div>
    ${replay}
  </li>`;
}

/** The statistics page: totals of the core's meta state, per character, the recent runs. */
function statsPage(meta: MetaView, history: RunRecord[]): string {
  const s = meta.stats;
  const stats: [string, number][] = [
    [de.book.runs, s.runs_finished],
    [de.book.runsWon, s.runs_won],
    [de.book.bestSection, s.best_section],
    [de.book.gamesWon, s.games_won],
    [de.book.gamesLost, s.games_lost],
    [de.book.bestGame, s.best_game],
  ];
  const chars = meta.characters
    .map((c) => {
      const st = s.characters[c.character];
      const line = c.unlocked
        ? t(de.book.perCharacter, { runs: t((st?.runs ?? 0) === 1 ? de.book.oneRun : de.book.manyRuns, { n: st?.runs ?? 0 }), won: st?.won ?? 0, best: st?.best_section ?? 0 })
        : t(de.book.lockedHint, { text: unlockText(meta, c.unlocked_by) });
      return `<li class="ledger-char ${c.unlocked ? '' : 'is-locked'}">${portrait(characterPortraitId(c.character), 'portrait-sm')}
        <div><b>${esc(characterName(c))}</b>${c.unlocked ? '' : lockIcon(16)}<span class="ledger-cond">${esc(line)}</span></div></li>`;
    })
    .join('');
  const recent = history.length
    ? `<p class="ledger-note">${esc(history.length === 1 ? de.book.recentHintOne : t(de.book.recentHint, { n: history.length }))}</p><ol class="recent-list" data-testid="recent-runs">${history.map((r) => recentRow(r, meta)).join('')}</ol>`
    : `<p class="ledger-note" data-testid="recent-none">${esc(de.book.recentNone)}</p>`;
  return `<h3 class="ledger-h">${esc(de.book.totals)}</h3>
    <dl class="ledger-stats" data-testid="meta-stats">${stats.map(([k, v]) => `<dt>${esc(k)}</dt><dd>${esc(v)}</dd>`).join('')}</dl>
    <h3 class="ledger-h">${esc(de.book.perCharacterTitle)}</h3>
    <ul class="ledger-chars">${chars}</ul>
    <h3 class="ledger-h">${esc(de.book.recent)}</h3>
    ${recent}`;
}

/** The Vereinsbuch: the unlocks page or the statistics page. */
export function renderBook(meta: MetaView, tab: 'book' | 'stats', history: RunRecord[]): string {
  return `<header class="ledger-head"><h2>${esc(de.book.title)}</h2><p class="ledger-sub">${esc(de.book.subtitle)}</p></header>
    ${bookTabs(tab)}
    <div class="ledger-page" data-page="${tab}">${tab === 'stats' ? statsPage(meta, history) : bookPage(meta)}</div>
    <button class="btn btn-wide" ${attrs({ 'data-action': 'close-overlay' })}>${esc(de.app.close)}</button>`;
}

/** Rays of light behind a new reward. */
function burst(): string {
  return `<span class="unlock-burst" aria-hidden="true">${Array.from({ length: 12 }, (_, i) => `<i style="--i:${i}"></i>`).join('')}</span>`;
}

/** The picture of a reward: portrait, modification badge or the relic's beer mat. */
function rewardPic(reward: Reward, info?: RewardInfo): string {
  if ('Character' in reward) return portrait(characterPortraitId(reward.Character), 'portrait-lg');
  if ('CardMod' in reward) return `<span class="unlock-coaster unlock-mod">${modIcon(reward.CardMod, 46)}</span>`;
  const look = relicLook(reward.Relic);
  const kind = info?.kind === 'Regular' || info?.kind === 'HouseRule' ? info.kind : look.kind;
  return `<span class="unlock-deckel">${relicCoaster({ id: reward.Relic, kind, rarity: info?.rarity ?? look.rarity }, { size: 72 })}</span>`;
}

/** Festive notice of unlocks just earned. */
function renderUnlocks(meta: MetaView, ids: string[]): string {
  const items = ids
    .map((id) => meta.unlocks.find((u) => u.id === id))
    .filter((u): u is UnlockEntry => u !== undefined)
    .map((u) => {
      const infos: { reward: Reward; name: string; text: string; kind: string; info?: RewardInfo }[] = u.reward_info?.length
        ? u.reward_info.map((i) => ({ reward: i.reward, name: infoName(i, meta), text: infoText(i), kind: infoKind(i), info: i }))
        : u.rewards.map((r) => ({
            reward: r,
            name: rewardName(r, meta),
            text: 'Character' in r ? contentText('characters', r.Character) : 'CardMod' in r ? contentText('card_mods', r.CardMod) : contentText('relics', r.Relic),
            kind: '',
          }));
      const rewards = infos
        .map(
          (i) =>
            `<div class="unlock-reward"><div class="unlock-pic">${burst()}${rewardPic(i.reward, i.info)}</div>${i.kind ? `<span class="unlock-kind" data-testid="unlock-kind">${esc(i.kind)}</span>` : ''}<b>${esc(i.name)}</b><span class="small">${esc(i.text)}</span></div>`,
        )
        .join('');
      const line = t(de.unlock.line, { name: infos.map((i) => i.name).join(', ') });
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
