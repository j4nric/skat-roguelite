// Start and end screens and the dialogs: Kontra, boss, info, menu with
// settings, Kreidetafel, Vereinsbuch with its statistics page, new unlocks,
// Abspann, target picking, sharing a seed, giving up a running run and the
// save code (./savegame.ts).

import { de, LANGS, lang, bossEditionText, dateText, bossFiguresText, bossLine, cardName, contentName, contentText, face, isCurse, modText, relicLook, t } from '../i18n';
import { dailyLabel, dailySeed, shareUrl } from '../seed';
import { ladderStarted, maxStake, pickedStake, winThreshold } from '../stake';
import type { RunRecord } from '../storage';
import { CHARACTERS } from '../types';
import type { Character, CharacterEntry, DeckCard, MetaView, Reward, RewardInfo, RunView, Suit, UnlockEntry } from '../types';
import { backDesign } from './backs';
import { cardBack, cardSvg, editionLook, lookSvg, modIcon, suitIcon, type CardLook } from './cards';
import { attrs, esc, reducedMotion } from './dom';
import { canVibrate } from '../haptics';
import { moonIcon, renderNightSheet } from './night';
import { bossCoaster, consumableCoaster, relicCoaster } from './emblems';
import { coinIcon } from './hud';
import { seatPerson } from './guests';
import { renderMarkePick, renderRelicPick } from './encounter';
import { oddsLine } from './odds';
import { calendarIcon, linkIcon, lockIcon } from './icons';
import { renderOverview } from './overview';
import { renderRules } from './rules';
import { renderGlossary } from './glossary';
import { bossPortraitId, characterPortraitId, portrait } from './people';
import { renderSaveConfirm, renderSaveExport, renderSaveImport, renderSaveMenu } from './savegame';
import { accountName, renderAccount, renderConflict, renderDeleteAccount, renderPasskeys, renderRecover, renderRecoveryCode, syncPill } from './account';
import type { DailyBoard, DailyEntry } from '../account';
import { renderRelicReplace, zinkeName } from './shop';
import type { ModQuestion, Overlay, UiState } from './state';

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

/**
 * A character in the picker row: portrait, short name and role; a locked one
 * shows its unlock condition instead of the role. The picked character's
 * whole text stands below the row ([`characterDetail`]).
 */
function characterCard(c: CharacterEntry, ui: UiState, meta: MetaView): string {
  const name = characterName(c);
  const short = de.characters[c.character];
  const label = short?.short ?? name;
  if (!c.unlocked) {
    const cond = t(de.start.lockedBy, { text: unlockText(meta, c.unlocked_by) });
    return `<div class="char-card is-locked" ${attrs({ 'data-character': c.character, 'aria-disabled': 'true', role: 'group', 'aria-label': `${name}, ${de.start.locked}. ${cond}` })}>
      <span class="char-lock">${lockIcon(20)}<span>${esc(de.start.locked)}</span></span>
      ${portrait(characterPortraitId(c.character), 'portrait-char')}
      <b class="char-name">${esc(label)}</b>
      <span class="char-cond">${esc(cond)}</span>
    </div>`;
  }
  const picked = ui.pickCharacter === c.character;
  return `<button class="char-card ${picked ? 'is-picked' : ''}" ${attrs({
    'data-action': 'pick-character',
    'data-character': c.character,
    'aria-pressed': picked ? 'true' : 'false',
    'aria-label': `${name}: ${short?.role ?? ''}`,
  })}>
    ${picked ? `<span class="char-check">${esc(de.start.picked)}</span>` : ''}
    ${portrait(characterPortraitId(c.character), 'portrait-char')}
    <b class="char-name">${esc(label)}</b>
    <span class="char-role">${esc(short?.role ?? '')}</span>
  </button>`;
}

/** The whole text of the picked character, below the picker row. */
function characterDetail(meta: MetaView, ui: UiState): string {
  const c = meta.characters.find((x) => x.character === ui.pickCharacter && x.unlocked);
  if (!c) return '';
  return `<p class="char-detail" data-testid="char-detail"><b>${esc(characterName(c))}:</b> ${esc(contentText('characters', c.character) || c.text)}</p>`;
}

/** The picker row: every character in one row, swiped or moved with the arrows. */
function characterPicker(meta: MetaView, ui: UiState): string {
  const cards = meta.characters.map((c) => characterCard(c, ui, meta)).join('');
  const arrow = (dir: -1 | 1, label: string, path: string): string =>
    `<button class="char-nav" ${attrs({ 'data-action': 'char-scroll', 'data-dir': dir, 'aria-label': label })}><svg viewBox="0 0 10 10" width="12" height="12" aria-hidden="true"><path d="${path}" fill="currentColor"/></svg></button>`;
  return `<section class="char-pick" aria-label="${esc(de.start.whoPlays)}">
    <h2 class="char-title">${esc(de.start.whoPlays)}</h2>
    <div class="char-row">
      ${arrow(-1, de.start.charsBack, 'M7 0 L7 10 L1 5 Z')}
      <div class="char-cards" data-testid="characters">${cards}</div>
      ${arrow(1, de.start.charsNext, 'M3 0 L3 10 L9 5 Z')}
    </div>
    ${characterDetail(meta, ui)}
  </section>`;
}

/**
 * After the start screen was drawn: restore the picker row's scroll
 * position, keep the picked character in sight and show the arrows only
 * when the row does not fit.
 */
export function settleCharacterPicker(root: HTMLElement, scrollLeft: number | undefined): void {
  const cards = root.querySelector<HTMLElement>('[data-testid="characters"]');
  if (!cards) return;
  cards.scrollLeft = scrollLeft ?? 0;
  const picked = cards.querySelector<HTMLElement>('.char-card.is-picked');
  if (picked) {
    const left = picked.offsetLeft - cards.offsetLeft;
    const right = left + picked.offsetWidth;
    if (left < cards.scrollLeft) cards.scrollLeft = left;
    else if (right > cards.scrollLeft + cards.clientWidth) cards.scrollLeft = right - cards.clientWidth;
  }
  cards.closest('.char-row')?.classList.toggle('has-overflow', cards.scrollWidth > cards.clientWidth + 1);
}

/** Move the picker row by about two cards. */
export function scrollCharacterPicker(root: HTMLElement, dir: number, smooth: boolean): void {
  const cards = root.querySelector<HTMLElement>('[data-testid="characters"]');
  if (!cards) return;
  const step = Math.max(cards.clientWidth * 0.66, 80);
  cards.scrollBy({ left: dir * step, behavior: smooth ? 'smooth' : 'auto' });
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

/** "Seed 123 · Alleinspieler", with "· Einsatz 2" for a stake. */
function seedLine(seed: string, character: Character, stake = 0): string {
  const line = t(de.share.line, { seed, character: shortName(character) });
  return stake > 0 ? line + t(de.share.lineStake, { stake }) : line;
}

/** Text for the system share sheet. */
function nativeText(seed: string, character: Character, stake = 0): string {
  const text = t(de.share.nativeText, { seed, character: shortName(character) });
  return stake > 0 ? text + t(de.share.nativeStake, { stake, augen: winThreshold(stake) }) : text;
}

/**
 * Einsatz-Leiter (DESIGN 9.3): one row with arrows, "Einsatz 2 · Gewinn ab
 * 63 Augen". Shown once any character has won a run; before that every
 * stake is 0 and the row would only take room.
 */
function stakePick(ui: UiState): string {
  if (!ladderStarted(ui.meta)) return '';
  const max = maxStake(ui.meta, ui.pickCharacter);
  const stake = pickedStake(ui.meta, ui.pickCharacter, ui.pickStake);
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
  return `<section class="shared-offer" data-testid="shared-seed" ${attrs({ 'data-seed': shared.seed, 'data-character': shared.character, 'data-stake': playable })}>
    <div class="shared-head">${linkIcon(20)}<h2>${esc(de.start.sharedTitle)}</h2></div>
    <p>${esc(t(de.start.sharedText, { seed: shared.seed, character: name }))}</p>
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
  return Math.min(shared.stake, maxStake(ui.meta, character));
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
      <span class="small">${esc(de.start.dailyText)} ${esc(seedLine(seed, ui.pickCharacter, pickedStake(ui.meta, ui.pickCharacter, ui.pickStake)))}</span>
      ${played ? `<span class="small daily-done">${esc(t(de.start.dailyDone, { result: runResult(played) }))}</span>` : ''}
    </div>
    ${ui.board && ui.board.day === seed ? dailyBoard(ui.board, ui.account.user !== null, played !== undefined) : !ui.account.online ? `<p class="small board-offline" data-testid="board-offline">${esc(de.offline.board)}</p>` : ''}
    <button class="btn btn-wide" ${attrs({ 'data-action': 'play-daily' })}>${esc(t(de.start.dailyPlay, { name: shortName(ui.pickCharacter) }))}</button>
  </section>`;
}

/** A result on the daily board in a few words. */
function boardResult(e: DailyEntry): string {
  const result = e.won ? de.board.won : t(de.board.reached, { section: e.section, series: e.series });
  return `${result}${e.stake > 0 ? ` · ${t(de.board.stake, { stake: e.stake })}` : ''}`;
}

function boardRow(e: DailyEntry): string {
  const character = (CHARACTERS as readonly string[]).includes(e.character) ? shortName(e.character as Character) : e.character;
  return `<li class="board-row${e.me ? ' is-me' : ''}"><span class="board-rank">${esc(e.rank)}.</span><span class="board-who"><span class="board-name">${esc(e.name ?? de.account.noName)}</span><span class="board-char">${esc(character)}</span></span><span class="board-result">${esc(boardResult(e))}</span></li>`;
}

/**
 * Today's daily board (ADR 0021): the best ten with names, the own place
 * below if it is not among them. Read-only without an account.
 */
function dailyBoard(board: DailyBoard, signedIn: boolean, played: boolean): string {
  const rows = board.entries.map(boardRow).join('');
  const meShown = board.entries.some((e) => e.me);
  let foot = '';
  if (!signedIn) {
    foot = `<p class="small board-hint">${esc(de.board.signInHint)} <button class="link-btn" ${attrs({ 'data-action': 'account', 'data-testid': 'board-signin' })}>${esc(de.board.signIn)}</button></p>`;
  } else if (board.me && !meShown) {
    foot = `<ol class="board-list board-me">${boardRow(board.me)}</ol><p class="small">${esc(t(de.board.you, { rank: board.me.rank, total: board.total }))}</p>`;
  } else if (!board.me && !played) {
    foot = `<p class="small board-hint">${esc(de.board.notYet)}</p>`;
  }
  return `<div class="board" data-testid="daily-board">
    <div class="board-head"><b>${esc(de.board.title)}</b>${board.total ? `<span class="small">${esc(t(de.board.total, { total: board.total }))}</span>` : ''}</div>
    ${rows ? `<ol class="board-list">${rows}</ol>` : `<p class="small">${esc(de.board.empty)}</p>`}
    ${foot}
  </div>`;
}

/**
 * The seed of a run to pass on: visible and selectable, a button that
 * copies the link, the system share sheet where there is one. If copying
 * fails, the link shows in a field to copy by hand.
 */
export function shareBlock(seed: string, character: Character, ui: UiState, stake = 0): string {
  const url = shareUrl(seed, character, stake);
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
      native = `<button class="btn" ${attrs({ 'data-action': 'share-native', 'data-url': url, 'data-text': nativeText(seed, character, stake) })}>${esc(de.share.native)}</button>`;
    }
  } catch {
    native = '';
  }
  return `<section class="share" data-testid="share">
    <div class="share-head">${linkIcon(18)}<b>${esc(de.share.title)}</b></div>
    <p class="share-seed">${esc(de.end.seed)} <span class="seed-num" data-testid="run-seed">${esc(seed)}</span> · ${esc(name)}${stake > 0 ? esc(t(de.share.lineStake, { stake })) : ''}</p>
    <p class="small">${esc(de.share.hint)}</p>
    <div class="share-actions">
      <button class="btn btn-primary" ${attrs({ 'data-action': 'copy-link', 'data-url': url, 'data-testid': 'copy-link' })}>${linkIcon(16)}<span>${esc(de.share.copy)}</span></button>
      ${native}
    </div>
    ${status}
  </section>`;
}

/** Die Einführung offered to a new player, once (DESIGN 16.1). */
function tutorialOffer(ui: UiState): string {
  if (!ui.tutorialOffer) return '';
  const TD = de.tutorial;
  return `<section class="tutorial-offer chalkboard" data-testid="tutorial-offer" ${attrs({ 'aria-label': TD.offerTitle })}>
    ${portrait('p-wirt', 'portrait-hint')}
    <div class="tutorial-offer-text">
      <h2>${esc(TD.offerTitle)}</h2>
      <p class="chalk-small">${esc(TD.offerText)}</p>
      <div class="tutorial-offer-actions">
        <button class="btn btn-small btn-primary" ${attrs({ 'data-action': 'tutorial-start', 'data-testid': 'tutorial-offer-play' })}>${esc(TD.offerPlay)}</button>
        <button class="btn btn-small" ${attrs({ 'data-action': 'tutorial-offer-no', 'data-testid': 'tutorial-offer-no' })}>${esc(TD.offerNo)}</button>
      </div>
    </div>
  </section>`;
}

export function renderStart(ui: UiState, buildInfo: string): string {
  const resume = ui.saveInfo
    ? `<button class="btn btn-primary btn-wide" ${attrs({ 'data-action': 'resume' })}>${esc(de.start.resume)}</button>
       <p class="small" data-testid="resume-info">${esc(ui.saveInfo)}</p>`
    : '';
  const meta = ui.meta;
  const chars = meta ? characterPicker(meta, ui) : '';
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
    ${tutorialOffer(ui)}
    <div class="start-actions">
      ${resume}
    </div>
    ${chars}
    ${''/* ADR 0016: the short evening is dropped, no length choice */}
    ${meta ? stakePick(ui) : ''}
    <div class="start-actions">
      <button class="btn ${ui.saveInfo ? '' : 'btn-primary'} btn-wide" ${attrs({ 'data-action': 'new-run' })}>${esc(newLabel)}</button>
      <label class="seed"><span>${esc(de.start.seedLabel)}</span>
        <input id="seed" name="seed" inputmode="numeric" autocomplete="off" placeholder="${esc(de.start.seedPlaceholder)}"></label>
    </div>
    ${dailyBlock(ui)}
    ${ui.tutorialOffer ? '' : `<div class="start-actions start-tutorial"><button class="btn btn-wide" ${attrs({ 'data-action': 'tutorial-start', 'data-testid': 'tutorial-start' })}>${esc(de.tutorial.start)}</button><p class="small">${esc(de.tutorial.startHint)}</p></div>`}
    ${meta ? startStats(meta) : ''}
    ${meta ? startBoard(meta) : ''}
    ${meta ? `<button class="btn btn-wide btn-book" ${attrs({ 'data-action': 'book' })}>${bookIcon()}<span>${esc(de.start.book)}</span><span class="book-progress">${esc(t(de.start.bookHint, { done, total }))}</span></button>` : ''}
    <p class="howto">${esc(de.start.howTo)}</p>
    <footer class="start-foot">
      <span class="account-link"><button class="link-btn" ${attrs({ 'data-action': 'account', 'data-testid': 'account-open' })}>${esc(ui.account.user ? accountName(ui.account.user) : de.account.menuSignIn)}</button>${ui.account.user ? syncPill(ui.account.status) : ''}</span>
      <button class="link-btn" ${attrs({ 'data-action': 'settings' })}>${esc(de.start.settings)}</button>
      <button class="link-btn" ${attrs({ 'data-action': 'credits' })}>${esc(de.start.credits)}</button>
      <span class="build" data-testid="build">${esc(t(de.app.build, { info: buildInfo }))}</span>
    </footer>
  </main>`;
}

/** Why the run ended: a lost pub (ADR 0019), a lost boss pub naming its boss players (ADR 0017). */
function lostReason(view: RunView): string {
  const s = view.series;
  if (s.is_boss && s.boss) {
    return t(de.end.lostBoss, { names: bossLine(s.boss), section: s.section + 1 });
  }
  return t(de.end.lostText, { section: s.section + 1, series: s.index + 1 });
}

/** A small framed picture, for "Teilen" (the run as an image). */
function pictureIcon(size = 18): string {
  return `<svg class="picture-icon" viewBox="0 0 12 10" width="${size}" height="${Math.round((size * 10) / 12)}" aria-hidden="true" shape-rendering="crispEdges"><path d="M0 0h12v10h-12z" fill="currentColor"/><path d="M1 1h10v8h-10z" fill="#23282c"/><path d="M2 8l3-4 2 2 1-1 2 3z" fill="#8fc7a8"/><path d="M8 2h2v2h-2z" fill="#d1773f"/></svg>`;
}

export function renderEnd(won: boolean, view: RunView, ui: UiState): string {
  // Die lange Nacht (ADR 0020): after der Alte the player may go on; a
  // night that ends is no lost run, the win stays.
  const night = view.night ?? null;
  const title = won ? de.end.won : night ? de.night.over : de.end.lost;
  const reason = won ? de.end.wonText : night ? t(de.night.overText, { n: night.district, pubs: night.pubs_won }) : lostReason(view);
  const enter = won && view.can_enter_night;
  const actions = enter
    ? `<p class="small end-night-hint">${esc(de.night.enterHint)}</p>
      <button class="btn btn-primary btn-wide btn-night" ${attrs({ 'data-action': 'enter-night', 'data-testid': 'enter-night' })}>${moonIcon(18)}<span>${esc(de.night.enter)}</span></button>
      <button class="btn btn-wide" ${attrs({ 'data-action': 'to-start', 'data-testid': 'end-run' })}>${esc(de.night.endRun)}</button>
      <button class="btn btn-wide" ${attrs({ 'data-action': 'new-run' })}>${esc(de.end.newRun)}</button>`
    : `<button class="btn btn-primary btn-wide" ${attrs({ 'data-action': 'new-run' })}>${esc(de.end.newRun)}</button>
      <button class="btn btn-wide" ${attrs({ 'data-action': 'to-start' })}>${esc(de.end.toStart)}</button>`;
  // The run as a picture to pass on (./runcard.ts).
  const shareRun = `<button class="btn btn-wide btn-share-run" ${attrs({ 'data-action': 'share-run', 'data-testid': 'share-run', 'aria-label': de.runCard.buttonAria, title: de.runCard.buttonAria })}>${pictureIcon()}<span>${esc(de.runCard.button)}</span></button>`;
  return `<main class="end ${night ? 'is-night' : ''}" data-testid="end">
    <div class="sheet chalkboard end-board">
      <h1 class="${won ? 'is-won' : night ? 'is-night' : 'is-lost'}">${esc(title)}</h1>
      <p class="chalk-small" data-testid="end-reason">${esc(reason)}</p>
      ${shareBlock(view.seed, view.character.character, ui, view.stake ?? 0)}
      ${renderOverview(view, ui.meta, !won && !night)}
    </div>
    <div class="end-actions">
      ${actions}
      ${shareRun}
    </div>
  </main>`;
}

/**
 * Kartenrückseite (ADR 0020): the backs earned at the night's milestones,
 * next to the character's own. Shown only once one is earned.
 */
function backPick(ui: UiState): string {
  const earned = ui.meta?.card_backs ?? [];
  if (!earned.length) return '';
  const chosen = ui.settings.back && earned.includes(ui.settings.back) ? ui.settings.back : null;
  const own = ui.view?.character.back ?? ui.meta?.characters.find((c) => c.character === ui.pickCharacter)?.back ?? null;
  const option = (id: string | null, label: string): string =>
    `<button class="btn btn-small back-option ${chosen === id ? 'is-picked' : ''}" ${attrs({ 'data-action': 'card-back', 'data-back': id ?? '', 'aria-pressed': chosen === id ? 'true' : 'false', 'aria-label': label, title: label })}><span class="back-swatch">${cardBack('', backDesign(id ?? own), 'small')}</span></button>`;
  const options = [option(null, de.menu.backOwn), ...earned.map((id) => option(id, de.backs[id]?.name ?? id))].join('');
  return `<div class="menu-row menu-backs" data-testid="setting-back"><span class="setting-label">${esc(de.menu.back)}</span><div class="back-options">${options}</div></div>`;
}

/**
 * Bestenliste (ADR 0020): per character the runs won, the deepest night
 * district, the most pubs of one night and the best game.
 */
function startBoard(meta: MetaView): string {
  const rows = meta.characters
    .filter((c) => c.unlocked)
    .map((c) => ({ c, s: meta.stats.characters[c.character] }))
    .filter((r) => r.s && (r.s.runs > 0 || (r.s.best_game ?? 0) > 0));
  if (!rows.length) return '';
  const none = de.start.boardNone;
  const body = rows
    .map(({ c, s }) => {
      const won = s?.won ?? 0;
      const night = s?.night_best_section ?? 0;
      const pubs = s?.night_pubs_won ?? 0;
      const game = s?.best_game ?? 0;
      const name = shortName(c.character);
      const label = t(de.start.boardAria, { name, won, night: night || none, pubs, game });
      return `<tr ${attrs({ 'data-character': c.character, 'aria-label': label })}><th scope="row">${esc(name)}</th><td>${esc(won)}</td><td>${night ? esc(t(de.start.boardNight, { n: night })) : none}</td><td>${night ? esc(pubs) : none}</td><td>${game ? esc(game) : none}</td></tr>`;
    })
    .join('');
  const h = de.start.boardHead;
  return `<section class="start-board chalkboard" data-testid="start-board">
    <h2>${moonIcon(14)} ${esc(de.start.board)}</h2>
    <table><thead><tr><th scope="col">${esc(h.who)}</th><th scope="col">${esc(h.won)}</th><th scope="col">${esc(h.night)}</th><th scope="col">${esc(h.pubs)}</th><th scope="col">${esc(h.game)}</th></tr></thead><tbody>${body}</tbody></table>
  </section>`;
}

function sheet(inner: string, closable = true, cls = '', overlayCls = ''): string {
  return `<div class="overlay ${overlayCls}" ${closable ? attrs({ 'data-action': 'close-overlay' }) : ''}><div class="sheet ${cls}" role="dialog" aria-modal="true">${inner}</div></div>`;
}

/**
 * "Untot" (core `rise_choice`): the card in the trick comes back as
 * `becomes`; the player picks three cards of his hand, one of them is
 * drawn and goes into the trick. Not closable: nothing goes before it.
 */
export function renderRiseChoice(view: RunView, picked: number[]): string {
  const choice = view.game?.rise_choice;
  if (!choice) return '';
  const hand = view.game?.hand ?? [];
  const cards = hand
    .map((c) => {
      const on = picked.includes(c.id);
      return `<button class="rise-card${on ? ' is-picked' : ''}" ${attrs({ 'data-action': 'rise-pick', 'data-id': c.id, 'data-testid': `rise-pick-${c.id}`, 'aria-pressed': on ? 'true' : 'false', 'aria-label': cardName(c.kind) })}><span class="card">${lookSvg(c)}</span></button>`;
    })
    .join('');
  const ready = picked.length === choice.pick;
  return sheet(
    `<div class="rise-head"><span class="card rise-becomes" data-testid="rise-becomes">${lookSvg({ ...choice.becomes, risen: true })}</span>
       <div><h2>${esc(t(de.rise.title, { n: choice.pick }))}</h2>
       <p class="small">${esc(t(de.rise.text, { card: cardName(choice.card.kind), becomes: cardName(choice.becomes.kind) }))}</p></div></div>
     <div class="rise-grid" data-testid="rise-choice">${cards}</div>
     <button class="btn btn-primary btn-wide" ${attrs({ 'data-action': 'rise-go', 'data-testid': 'rise-go', disabled: ready ? null : 'disabled' })}>${esc(ready ? de.rise.go : t(de.rise.more, { n: choice.pick - picked.length }))}</button>`,
    false,
    'sheet-rise',
  );
}

/**
 * "Untot": a tap picked or dropped a card. Only that card's state and the
 * button change in place (no re-render, so the sheet does not enter again).
 */
export function updateRiseChoice(sheetEl: HTMLElement, picked: number[], need: number): void {
  sheetEl.querySelectorAll<HTMLElement>('.rise-card').forEach((el) => {
    const on = picked.includes(Number(el.dataset.id));
    el.classList.toggle('is-picked', on);
    el.setAttribute('aria-pressed', on ? 'true' : 'false');
  });
  const go = sheetEl.querySelector<HTMLButtonElement>('[data-action="rise-go"]');
  if (!go) return;
  const ready = picked.length === need;
  go.disabled = !ready;
  go.textContent = ready ? de.rise.go : t(de.rise.more, { n: need - picked.length });
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

/**
 * "Hand sortieren" as a toggle: automatisch (the core's order for the
 * game) or manuell (the player drags); a tap switches.
 */
function handSortToggle(ui: UiState): string {
  const manual = ui.settings.handSort === 'manual';
  return `<button class="seg seg-toggle ${manual ? 'is-picked' : ''}" ${attrs({
    'data-action': 'hand-sort',
    'data-sort': manual ? 'auto' : 'manual',
    'data-testid': 'toggle-hand-sort',
    'aria-pressed': manual ? 'true' : 'false',
    'aria-label': `${de.handSort.label}: ${manual ? de.handSort.manual : de.handSort.auto}`,
    title: de.handSort.hint,
  })}>${esc(t(de.handSort.toggle, { mode: manual ? de.handSort.manual : de.handSort.auto }))}</button>`;
}

/**
 * "Sprache / Language" (ADR 0023) in the menu's head: each language named
 * in itself, so it is found whichever is on; the group says it in both.
 */
function languageSwitch(): string {
  const langs = LANGS.map(
    (l) =>
      `<button class="seg ${lang() === l ? 'is-picked' : ''}" ${attrs({ 'data-action': 'language', 'data-lang': l, 'data-testid': `lang-${l}`, lang: l, 'aria-pressed': lang() === l ? 'true' : 'false' })}>${esc(de.lang.names[l])}</button>`,
  ).join('');
  return `<div class="segmented lang-switch" ${attrs({ role: 'group', 'aria-label': de.lang.label, title: de.lang.label, 'data-testid': 'setting-language' })}>${langs}</div>`;
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
    <div class="menu-row"><span class="setting-label">${esc(de.menu.sound)}</span><div class="segmented">${toggle('sound', de.menu.effects, ui.settings.sound)}${toggle('music', de.menu.music, ui.settings.music)}${canVibrate() ? toggle('vibrate', de.vibration.label, ui.settings.vibrate) : ''}</div></div>
    <div class="menu-row" ${attrs({ 'data-testid': 'setting-table' })}><span class="setting-label">${esc(de.handSort.row)}</span><div class="segmented" ${attrs({ title: de.menu.thinkingHint })}>${toggle('thinking', de.menu.thinking, ui.settings.thinking)}${handSortToggle(ui)}</div></div>
    <div class="menu-row" ${attrs({ title: de.menu.suitTintHint, 'data-testid': 'setting-suit-tint' })}><span class="setting-label">${esc(de.menu.suitTint)}</span>${onOff('suit-tint', ui.settings.suitTint)}</div>
    ${backPick(ui)}
    <div class="menu-row menu-hints"><span class="setting-label">${esc(de.menu.hints)}</span>${onOff('hints', ui.settings.hints)}<button class="btn" ${attrs({
      'data-action': 'hints-reset',
      'data-testid': 'hints-reset',
      'aria-label': de.menu.hintsResetAria,
      title: de.menu.hintsResetAria,
    })}>${esc(de.menu.hintsReset)}</button></div>
  </div>`;
}

/** The seed of the running run as one row with "Link kopieren" (menu). */
function seedRow(seed: string, character: Character, ui: UiState, stake = 0): string {
  const url = shareUrl(seed, character, stake);
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
        'data-text': nativeText(seed, character, stake),
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
    <div class="sheet-body">
      <p>${esc(de.credits.intro)}</p>
      <ul class="credits">${rows}</ul>
      <p class="credits-thanks">${esc(de.credits.thanks)}</p>
    </div>
    <button class="btn btn-wide" ${attrs({ 'data-action': 'close-overlay' })}>${esc(de.app.close)}</button>`;
}

export function renderOverlay(overlay: Overlay, view: RunView | null, ui: UiState): string {
  switch (overlay.kind) {
    case 'boss':
      return sheet(
        `<div class="boss-intro"><div class="boss-faces">${portrait(bossPortraitId(overlay.boss.id, 0), 'portrait-boss')}<span class="boss-back" data-testid="boss-back" data-back="${esc(backDesign(overlay.boss.back))}">${cardBack('', backDesign(overlay.boss.back))}</span>${portrait(bossPortraitId(overlay.boss.id, 1), 'portrait-boss')}</div>
         <p class="small boss-kicker">${bossCoaster(overlay.boss.id, { size: 32 })}<span>${esc(de.boss.title)} · ${esc(de.boss.intro)}</span></p>
         <h2>${esc(contentName('bosses', overlay.boss.id))}</h2>
         <p class="boss-figures" data-testid="boss-figures">${esc(bossFiguresText(overlay.boss))}</p>
         <p>${esc(contentText('bosses', overlay.boss.id) || overlay.boss.text)}</p>${bossEditionText(overlay.boss) ? `<p class="boss-edition">${esc(bossEditionText(overlay.boss))}</p>` : ''}
         <p class="boss-stakes" data-testid="boss-stakes">${esc(de.boss.stakes)}</p></div>
         <button class="btn btn-primary btn-wide" ${attrs({ 'data-action': 'boss-go' })}>${esc(de.boss.go)}</button>`,
        false,
        'sheet-boss',
      );
    case 'menu':
      return sheet(
        `<div class="menu-head"><h2>${esc(de.header.menu)}</h2>
           ${languageSwitch()}
           <span class="account-link"><button class="link-btn" ${attrs({ 'data-action': 'account', 'data-testid': 'menu-account' })}>${esc(ui.account.user ? t(de.account.menuAccount, { name: accountName(ui.account.user) }) : de.account.menuSignIn)}</button>${ui.account.user ? syncPill(ui.account.status) : ''}</span></div>
         ${settingsBlock(ui)}
         ${view && ui.screen === 'run' ? seedRow(view.seed, view.character.character, ui, view.stake ?? 0) : ''}
         <div class="menu-list menu-grid">
           <button class="btn" ${attrs({ 'data-action': 'rules', 'data-testid': 'rules-open' })}>${esc(de.menu.rules)}</button>
           <button class="btn" ${attrs({ 'data-action': 'glossary', 'data-testid': 'glossary-open' })}>${esc(de.menu.glossary)}</button>
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
    case 'account':
      return renderAccount(sheet, ui.account, overlay.busy, overlay.error, overlay.note);
    case 'account-code':
      return renderRecoveryCode(sheet, overlay.code, overlay.why, overlay.copied, ui.account.supported);
    case 'account-recover':
      return renderRecover(sheet, overlay.busy, overlay.error);
    case 'account-delete':
      return renderDeleteAccount(sheet, overlay.busy, overlay.error);
    case 'account-passkeys':
      return renderPasskeys(sheet, overlay.list, overlay.busy, overlay.error, overlay.note);
    case 'sync-conflict':
      return renderConflict(sheet, overlay.here, overlay.server);
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
         <div class="sheet-body">${renderOverview(view, ui.meta, false, view.game ? ui.wildSettled : {})}</div>
         <button class="btn btn-wide" ${attrs({ 'data-action': 'close-overlay' })}>${esc(de.app.close)}</button>`,
        true,
        'chalkboard sheet-overview has-body',
      );
    case 'credits':
      return sheet(credits(), true, 'sheet-credits has-body');
    case 'rules':
      return sheet(renderRules(), true, 'sheet-rules');
    case 'glossary':
      return sheet(renderGlossary(), true, 'sheet-rules sheet-glossary');
    case 'target': {
      const item = view?.consumables.find((c) => c.slot === overlay.slot);
      if (!view || !item) return '';
      const cards = view.edition.map((c) => targetCard(editionLook(c, view, view.game ? ui.wildSettled : {}), item.targets.includes(c.id), 'target', overlay.reasons?.[c.id])).join('');
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
          })}>${lookSvg(c, 'full')}</button>`;
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
      // After a Marke was picked: only the cards that Marke may go on.
      const targets = overlay.marke ? (option.marken?.find((m) => m.card_mod.id === overlay.marke)?.targets ?? []) : option.targets;
      const cards = view.edition.map((c) => targetCard(editionLook(c, view, view.game ? ui.wildSettled : {}), targets.includes(c.id), 'encounter-target', overlay.reasons?.[c.id])).join('');
      return sheet(
        `<h2>${esc(de.encounter.pickTarget)}</h2>
         <p class="small">${esc(contentText('encounters', view.encounter?.id ?? ''))}</p>
         ${oddsLine(view.deal_odds)}
         <div class="target-grid">${cards}</div>
         <button class="btn btn-wide" ${attrs({ 'data-action': 'close-overlay' })}>${esc(de.shop.cancel)}</button>`,
      );
    }
    case 'encounter-relic': {
      const option = view?.encounter?.options.find((o) => o.index === overlay.option);
      if (!view || !option) return '';
      return sheet(renderRelicPick(view, option));
    }
    case 'relic-replace':
      return view ? sheet(renderRelicReplace(view, overlay.slot)) : '';
    case 'encounter-marke': {
      const option = view?.encounter?.options.find((o) => o.index === overlay.option);
      if (!view || !option) return '';
      return sheet(renderMarkePick(option), false, 'sheet-marke');
    }
    case 'mod-confirm':
      return renderModConfirm(overlay.ask);
    case 'zinken-card': {
      // Zinken (DESIGN 8.6): the card for the Kartenmaler or "Karte reinigen".
      const s = overlay.service === 'painter' ? view?.shop?.painter : view?.shop?.cleanse;
      if (!view || !s) return '';
      const cards = view.edition.map((c) => targetCard(c, s.targets.includes(c.id), 'zinken-card', overlay.reasons?.[c.id])).join('');
      const painter = overlay.service === 'painter';
      const cheaper = Math.min(s.price, ...(s.target_prices ?? []));
      return sheet(
        `<h2>${esc(painter ? de.shop.painterPick : de.shop.cleansePick)}</h2>
         <p class="small">${esc(t(painter ? de.shop.painterGo : de.shop.cleanseGo, { price: s.price }))} · ${esc(painter ? de.shop.painterHint : de.shop.cleanseHint)}</p>
         ${cheaper < s.price ? `<p class="small" data-testid="zinken-cheaper">${esc(t(de.shop.cheaperOn, { price: cheaper }))}</p>` : ''}
         ${oddsLine(view.deal_odds)}
         <div class="target-grid" data-testid="zinken-cards">${cards}</div>
         <button class="btn btn-wide" ${attrs({ 'data-action': 'close-overlay' })}>${esc(de.shop.cancel)}</button>`,
      );
    }
    case 'painter': {
      const painter = view?.painter;
      if (!painter) return '';
      const options = painter.options
        .map((o) => {
          const note = o.replaces
            ? t(de.shop.zinkeReplaces, { old: contentName('card_mods', o.replaces) })
            : o.raises
              ? t(de.shop.zinkeRaises, { mod: contentName('card_mods', o.card_mod.id) })
              : '';
          return `<button class="painter-option" ${attrs({ 'data-action': 'paint', 'data-index': o.index, 'data-testid': `paint-${o.index}`, 'data-card-mod': o.card_mod.id })}>
            <span class="card">${cardSvg(o.result.kind, o.result.mods, 'full', o.result.mod_values)}</span>
            <b class="zinke-name">${esc(zinkeName(o.card_mod))}</b>
            <span class="zinke-text">${esc(modText(o.card_mod.id, o.card_mod.value ?? 1) || o.card_mod.text)}</span>
            ${note ? `<span class="zinke-note">${esc(note)}</span>` : ''}
            <span class="btn btn-small btn-buy">${esc(de.shop.painterTake)}</span>
          </button>`;
        })
        .join('');
      return sheet(
        `<h2>${esc(de.shop.painterTitle)}</h2>
         <p class="small">${esc(t(de.shop.painterChoose, { card: cardName(painter.card.kind) }))}${painter.encounter ? ` ${esc(de.shop.painterFree)}` : ''}</p>
         <div class="painter-grid" data-testid="painter-options">${options}</div>`,
        false,
        'sheet-painter',
      );
    }
    case 'schnapps-reward': {
      // The Schnaps of a won pub with every slot full (DESIGN 8.9): the new
      // one big, the held ones to give up for it, or sell the new one.
      const reward = view?.schnapps_reward;
      if (!view || !reward) return '';
      const fresh = reward.consumable;
      const swaps = view.consumables
        .map((c) => {
          const name = contentName('consumables', c.consumable.id);
          return `<button class="swap-schnaps" ${attrs({ 'data-action': 'schnapps-swap', 'data-slot': c.slot, 'data-testid': `schnapps-swap-${c.slot}`, 'aria-label': t(de.shop.schnappsRewardSwap, { name }) })}>
            ${consumableCoaster(c.consumable, { size: 40 })}<b>${esc(name)}</b><small>${esc(de.drink.effect[c.consumable.id] ?? '')}</small><span class="btn btn-small">${esc(de.shop.schnappsRewardGive)}</span></button>`;
        })
        .join('');
      return sheet(
        `<div class="schnapps-reward" data-testid="schnapps-reward">
         <h2>${esc(de.shop.schnappsRewardTitle)}</h2>
         <div class="reward-schnaps" data-testid="schnapps-reward-new">
           <span class="reward-schnaps-pic">${consumableCoaster(fresh, { size: 80 })}</span>
           <span class="reward-schnaps-text"><b>${esc(contentName('consumables', fresh.id))}</b><small>${esc(de.rarity[fresh.rarity] ?? '')}</small><span>${esc(contentText('consumables', fresh.id))}</span></span>
         </div>
         <p class="small">${esc(t(de.shop.schnappsRewardChoose, { name: contentName('consumables', fresh.id) }))}</p>
         <h3 class="swap-title">${esc(de.shop.schnappsRewardSwapTitle)}</h3>
         <div class="swap-row">${swaps}</div>
         <button class="btn btn-wide btn-primary" ${attrs({ 'data-action': 'schnapps-sell', 'data-testid': 'schnapps-sell' })}>${coinIcon(16)}${esc(t(de.shop.schnappsRewardSell, { price: reward.sell_price }))}</button>
         </div>`,
        false,
        'sheet-schnapps-reward',
      );
    }
    case 'book':
      return ui.meta ? sheet(renderBook(ui.meta, overlay.tab, ui.history), true, 'ledger has-body', 'overlay-book') : '';
    case 'confirm-new':
      return sheet(
        `<h2>${esc(de.confirm.title)}</h2>
         <p>${esc(de.confirm.text)}</p>
         ${ui.saveInfo ? `<p class="confirm-run"><b>${esc(ui.saveInfo)}</b></p>` : ''}
         <p class="small">${esc(seedLine(overlay.start.seed, overlay.start.character, overlay.start.stake))}</p>
         <div class="sheet-actions">
           <button class="btn btn-primary" ${attrs({ 'data-action': 'confirm-new' })}>${esc(de.confirm.go)}</button>
           <button class="btn" ${attrs({ 'data-action': 'close-overlay' })}>${esc(de.confirm.cancel)}</button>
         </div>`,
        true,
        'sheet-confirm',
      );
    case 'confirm-give-up': {
      const boss = Boolean(view?.series.is_boss);
      // ADR 0019: giving up ends the run, unless the Schutzengel has the pub played again.
      const angel = Boolean(view?.relics.some((r) => r.relic.id === 'guardian_angel'));
      const lines = angel
        ? `<p>${esc(boss ? de.giveUp.bossText : de.giveUp.text)}</p><p class="small" data-testid="give-up-angel">${esc(de.giveUp.angel)}</p>`
        : `<p class="give-up-warning" ${attrs({ 'data-testid': boss ? 'give-up-boss' : 'give-up-end' })}><b>${esc(de.giveUp.boss)}</b></p><p>${esc(boss ? de.giveUp.bossText : de.giveUp.text)}</p>`;
      return sheet(
        `<h2>${esc(de.giveUp.title)}</h2>
         ${lines}
         <div class="sheet-actions">
           <button class="btn btn-danger" ${attrs({ 'data-action': 'confirm-give-up', 'data-testid': 'give-up-confirm' })}>${esc(de.giveUp.go)}</button>
           <button class="btn btn-primary" ${attrs({ 'data-action': 'close-overlay' })}>${esc(de.giveUp.cancel)}</button>
         </div>`,
        true,
        'sheet-confirm',
      );
    }
    case 'unlock':
      return ui.meta ? sheet(renderUnlocks(ui.meta, overlay.ids), false, 'sheet-unlock', 'overlay-unlock') : '';
    case 'special':
      return sheet(renderSpecials(overlay.ids, view?.edition ?? []), false, 'sheet-special', 'overlay-special');
    case 'night':
      return view ? sheet(renderNightSheet(view), false, 'sheet-night', 'overlay-night') : '';
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
  if ('CardBack' in reward) return de.backs[reward.CardBack]?.name ?? reward.CardBack;
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
  const own = 'Relic' in r ? contentText('relics', r.Relic) : 'CardMod' in r ? contentText('card_mods', r.CardMod) : 'CardBack' in r ? (de.backs[r.CardBack]?.text ?? '') : contentText('characters', r.Character);
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
  const when = Number.isNaN(date.getTime()) ? '' : dateText(date, { day: '2-digit', month: '2-digit', year: '2-digit' });
  const replay = entry?.unlocked
    ? `<button class="btn btn-small" ${attrs({ 'data-action': 'replay', 'data-seed': r.seed, 'data-character': r.character, 'data-stake': r.stake ?? 0, 'aria-label': t(de.book.replayAria, { seed: r.seed, character: shortName(r.character) }) })}>${esc(de.book.replay)}</button>`
    : '';
  return `<li class="recent-row ${r.won ? 'is-won' : ''}" ${attrs({ 'data-seed': r.seed })}>
    ${portrait(characterPortraitId(r.character), 'portrait-xs')}
    <div class="recent-main">
      <b>${esc(shortName(r.character))}</b>${r.stake ? `<span class="recent-daily recent-short">${esc(t(de.book.stake, { stake: r.stake }))}</span>` : ''}${r.daily ? `<span class="recent-daily">${esc(de.book.daily)}</span>` : ''}
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
    <div class="ledger-page sheet-body" data-page="${tab}">${tab === 'stats' ? statsPage(meta, history) : bookPage(meta)}</div>
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
  if ('CardBack' in reward) return `<span class="unlock-back">${cardBack('', backDesign(reward.CardBack))}</span>`;
  const look = relicLook(reward.Relic);
  const kind = info?.kind === 'Regular' || info?.kind === 'HouseRule' ? info.kind : look.kind;
  return `<span class="unlock-deckel">${relicCoaster({ id: reward.Relic, kind, rarity: info?.rarity ?? look.rarity, group: look.group }, { size: 72 })}</span>`;
}

/**
 * The first Untot or Wandelkarte of this save (DESIGN 7.2): the card as it
 * looks now and a short rule summary, once per special card.
 */
function renderSpecials(ids: string[], edition: DeckCard[]): string {
  const items = ids
    .map((id) => {
      const card = edition.find((c) => c.mods.includes(id));
      const special = contentName('card_mods', id);
      const rules = (id === 'undead' ? de.special.undead : id === 'wild_card' ? de.special.wild_card : id === 'doppelganger' ? de.special.doppelganger : []).map((r) => `<li>${esc(r)}</li>`).join('');
      const lead = card ? t(de.special.lead, { card: cardName(card.kind), special }) : special;
      return `<li class="special-item" ${attrs({ 'data-special': id })}>
        ${card ? `<span class="card special-card">${cardSvg(card.kind, card.mods, 'full', card.mod_values)}</span>` : ''}
        <div class="special-body"><b class="special-name">${esc(special)}</b><p>${esc(lead)}</p><ul class="special-rules">${rules}</ul></div>
      </li>`;
    })
    .join('');
  return `<h2>${esc(de.special.title)}</h2>
    <ul class="special-list" data-testid="special-hint">${items}</ul>
    <p class="small">${esc(de.special.cap)}</p>
    <button class="btn btn-primary btn-wide" ${attrs({ 'data-action': 'close-overlay', 'data-testid': 'special-ok' })}>${esc(de.special.go)}</button>`;
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
            text: 'Character' in r ? contentText('characters', r.Character) : 'CardMod' in r ? contentText('card_mods', r.CardMod) : 'CardBack' in r ? (de.backs[r.CardBack]?.text ?? '') : contentText('relics', r.Relic),
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

/**
 * A card in a picker (consumable or encounter). A card the core refuses is
 * dimmed with the core's reason (its error code from the preview): a
 * short tag on the card, the full reason as its label.
 */
function targetCard(c: DeckCard & CardLook, ok: boolean, action: string, reason: string | undefined): string {
  const curse = c.mods.find(isCurse);
  const why = !ok && reason ? de.modConfirm.reasons[reason] : undefined;
  const tag = !ok && reason ? de.modConfirm.tags[reason] : undefined;
  const label = why ? `${cardName(c.kind)}: ${t(why, { curse: curse ? contentName('card_mods', curse) : '' })}` : cardName(c.kind);
  return `<button class="card target-card ${ok ? '' : 'is-dim'}" ${attrs({
    'data-action': action,
    'data-id': c.id,
    'data-info': `card:${c.id}`,
    'data-reason': !ok && reason ? reason : null,
    disabled: !ok,
    title: why ? label : null,
    'aria-label': label,
  })}>${lookSvg(c, 'full')}${tag ? `<span class="target-reason">${esc(tag)}</span>` : ''}</button>`;
}

/**
 * Asking before a card modification replaces, raises or cancels the one on
 * a card: what the core's preview reported, with the card now and after.
 */
function renderModConfirm(ask: ModQuestion): string {
  const card = cardName(ask.card.kind);
  const name = (m: string | null): string => (m ? contentName('card_mods', m) : '');
  const c = de.modConfirm;
  const [title, text, go] =
    ask.type === 'replace'
      ? [c.replaceTitle, t(c.replace, { card, old: name(ask.old), new: name(ask.mod) }), c.replaceGo]
      : ask.type === 'raise'
        ? [c.raiseTitle, t(c.raise, { card, mod: name(ask.mod), from: ask.from, to: ask.to }), c.raiseGo]
        : ask.type === 'change'
          ? [c.changeTitle, t(c.change, { card, old: name(ask.old), new: name(ask.mod), from: ask.from, to: ask.to }), c.changeGo]
        : ask.type === 'drop'
          ? [c.dropTitle, t(c.drop, { card, old: name(ask.old), new: name(ask.mod) }), c.dropGo]
          : ask.type === 'apply'
            ? [c.applyTitle, ask.mod ? t(c.apply, { card, new: name(ask.mod) }) : t(c.applyPlain, { card }), ask.price != null ? t(c.buyGo, { price: ask.price }) : c.applyGo]
            : ask.type === 'cleanse'
              ? [c.cleanseTitle, t(c.cleanse, { card }), c.cleanseGo]
              : ask.type === 'randomMarke'
                ? [c.randomMarkeTitle, t(c.randomMarke, { card, marken: ask.marken ?? '' }), c.randomMarkeGo]
                : [c.randomTitle, t(c.random, { card, old: name(ask.old) }), c.randomGo];
  // A fixed offer (card reward, Auslage): back to the offers. From a card
  // picker the way back is another card; otherwise keep the card as it is.
  const back = ask.pair ? de.cardReward.backOffers : ask.type === 'apply' || ask.type === 'cleanse' ? c.back : c.keep;
  const face = (d: DeckCard, label: string): string =>
    `<figure class="mod-confirm-card"><span class="card">${cardSvg(d.kind, d.mods, 'full', d.mod_values)}</span><figcaption>${esc(label)}</figcaption></figure>`;
  const cards = ask.after
    ? `${face(ask.card, c.before)}<span class="mod-confirm-arrow" aria-hidden="true">→</span>${face(ask.after, c.after)}`
    : face(ask.card, c.before);
  return sheet(
    `<h2>${esc(title)}</h2>
     <p data-testid="mod-confirm-text">${esc(text)}</p>
     <div class="mod-confirm-cards" data-testid="mod-confirm-cards" data-type="${ask.type}">${cards}</div>
     <div class="sheet-actions">
       <button class="btn btn-primary" ${attrs({ 'data-action': 'mod-confirm-go', 'data-testid': 'mod-confirm-go' })}>${esc(go)}</button>
       <button class="btn" ${attrs({ 'data-action': 'mod-confirm-back', 'data-testid': 'mod-confirm-back' })}>${esc(back)}</button>
     </div>`,
    false,
    'sheet-mod-confirm',
  );
}
