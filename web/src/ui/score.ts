// The Abrechnung: the score breakdown from the core, revealed step by step.
// All numbers come from the core's ScoreStep list; this file only labels
// and formats them.

import { de, cardName, contentName, gameName, t } from '../i18n';
import type { Change, DeckCard, Lever, ScoreLine, ScoreTerm, Source } from '../types';
import { attrs, esc } from './dom';
import type { Overlay } from './state';

type ScoreOverlay = Extract<Overlay, { kind: 'score' }>;

/** "+3" / "−3" with a real minus sign. */
export function signed(n: number): string {
  if (n > 0) return `+${n}`;
  if (n < 0) return `−${Math.abs(n)}`;
  return '0';
}

function sourceName(source: Source, deck: DeckCard[]): string {
  if (source === 'Kontra') return de.score.kontra;
  if (source === 'Re') return de.score.re;
  if ('Relic' in source) return contentName('relics', source.Relic);
  if ('Boss' in source) return contentName('bosses', source.Boss);
  const card = deck.find((c) => c.id === source.CardMod.card);
  return t(de.score.cardMod, {
    mod: contentName('card_mods', source.CardMod.card_mod),
    card: card ? cardName(card.kind) : '',
  });
}

function levelText(n: number): string {
  if (n === 1) return t(de.score.plusLevels, { n });
  if (n > 1) return t(de.score.plusLevelsMany, { n });
  return t(de.score.minusLevels, { n: signed(n) });
}

function changeText(lever: Lever, change: Change): string {
  if ('Multiply' in change) {
    return lever === 'BaseValue'
      ? t(de.score.baseMul, { n: change.Multiply })
      : t(de.score.valueMul, { n: change.Multiply });
  }
  if ('Set' in change) {
    if (lever === 'WinThreshold') return t(de.score.threshold, { n: change.Set });
    if (lever === 'Schneider') return de.score.schneiderBy;
    return t(de.score.setTo, { n: change.Set });
  }
  const n = change.Add;
  switch (lever) {
    case 'BaseValue':
      return n >= 0 ? t(de.score.baseAdd, { n }) : t(de.score.baseSub, { n: signed(n) });
    case 'Levels':
      return levelText(n);
    case 'CardPoints':
      return n >= 0 ? t(de.score.pointsAdd, { n }) : t(de.score.pointsSub, { n: signed(n) });
    default:
      return signed(n);
  }
}

function lineRow(line: ScoreLine, deck: DeckCard[]): [string, string, string] {
  return [sourceName(line.source, deck), changeText(line.lever, line.change), 'is-relic'];
}

/** Label, effect and style of one step. */
function termRow(term: ScoreTerm, deck: DeckCard[]): [string, string, string] {
  if (typeof term === 'string') {
    const labels: Record<typeof term, string> = {
      Game: de.score.game,
      Hand: de.score.hand,
      Schneider: de.score.schneider,
      SchneiderDeclared: de.score.schneiderDeclared,
      Schwarz: de.score.schwarz,
      SchwarzDeclared: de.score.schwarzDeclared,
      Ouvert: de.score.ouvert,
    };
    return [labels[term], levelText(1), 'is-level'];
  }
  if ('BaseValue' in term) {
    return [t(de.score.baseOf, { game: gameName(term.BaseValue.game) }), String(term.BaseValue.value), 'is-base'];
  }
  if ('NullValue' in term) return [de.score.nullValue, String(term.NullValue), 'is-base'];
  if ('Matadors' in term) {
    const m = term.Matadors;
    return [t(m.with ? de.score.with : de.score.without, { count: m.count }), levelText(m.count), 'is-level'];
  }
  return lineRow(term.Line, deck);
}

export function renderScore(o: ScoreOverlay, deck: DeckCard[]): string {
  const b = o.score.breakdown;
  const steps = o.score.steps;
  const done = o.shown >= steps.length;
  const current = steps[Math.min(o.shown, steps.length) - 1];
  const rows = steps
    .slice(0, o.shown)
    .map((step) => {
      const [label, effect, cls] = termRow(step.term, deck);
      return `<li class="step ${cls}"><span class="step-label">${esc(label)}</span><span class="step-effect">${esc(effect)}</span></li>`;
    })
    .join('');
  const formula = current
    ? current.multiplier > 1
      ? t(de.score.formulaMul, { base: current.base_value, levels: current.levels, mul: current.multiplier })
      : t(de.score.formula, { base: current.base_value, levels: current.levels })
    : '';
  const decl = t(de.table.declaration, { game: gameName(b.declaration.game), risk: de.risks[b.declaration.risk] });
  const result = done
    ? `<div class="score-result ${b.won ? 'is-won' : 'is-lost'}">
        <div><span>${esc(de.score.value)}</span><b>${esc(b.value)}</b></div>
        <div><span>${esc(de.score.result)}</span><b data-testid="score-result">${esc(signed(b.score))}</b></div>
        ${b.won ? '' : `<p class="small">${esc(de.score.lostDouble)}</p>`}
        <div class="score-account">${esc(t(de.score.account, { score: o.seriesScore, target: o.target }))}</div>
      </div>`
    : '';
  const payout =
    done && o.payout
      ? `<div class="payout">
          <h3>${esc(de.score.seriesWon)}</h3>
          <dl>
            <dt>${esc(de.score.payoutBase)}</dt><dd>${esc(signed(o.payout.payout.base))}</dd>
            <dt>${esc(de.score.payoutGames)}</dt><dd>${esc(signed(o.payout.payout.games_left))}</dd>
            <dt>${esc(de.score.payoutInterest)}</dt><dd>${esc(signed(o.payout.payout.interest))}</dd>
            <dt>${esc(de.header.coins)}</dt><dd><b>${esc(o.payout.coins)}</b></dd>
          </dl>
        </div>`
      : '';
  const button = done
    ? `<button class="btn btn-primary btn-wide" ${attrs({ 'data-action': 'score-continue' })}>${esc(de.score.continue)}</button>`
    : `<button class="btn btn-wide" ${attrs({ 'data-action': 'score-skip' })}>${esc(de.score.skip)}</button>`;
  return `<div class="overlay" data-action="score-skip"><div class="sheet score-sheet" role="dialog" aria-modal="true" aria-labelledby="score-title">
    <h2 id="score-title" class="${b.won ? 'is-won' : 'is-lost'}">${esc(b.won ? de.score.won : de.score.lost)}</h2>
    <div class="score-decl">${esc(decl)}</div>
    <div class="score-meta">${esc(t(de.score.points, { own: b.outcome.declarer_card_points, theirs: b.outcome.defender_card_points }))} · ${esc(t(de.score.tricks, { own: b.outcome.declarer_tricks, theirs: b.outcome.defender_tricks }))}</div>
    <div class="score-formula" aria-live="polite">${esc(formula)}</div>
    <ol class="steps">${rows}</ol>
    ${result}
    ${payout}
    ${button}
  </div></div>`;
}
