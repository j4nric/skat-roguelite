// Cards as small SVGs that reference the sprite in ./art.ts. A face is
// drawn in layers on the art grid of the deck (33 × 49.5 art pixels, one
// art pixel = 2 × 2 CSS px at the 66 × 99 hand size): paper (plain or,
// with the setting "Kartenfarbe tönen", lightly in the suit's colour), the
// picture, the art of the card's modification, then the corner index on
// top.
//
// Effekte v3 (tools/cardart/fx3.py, ./fxart.ts): frame = what, mark top
// right = when, band at the bottom middle = how much; curses anthracite.
// The special cards change the picture: "Untot" shows the zombie picture
// (rotten pips on number cards), "Wandelkarte" one of eight
// suit-independent faces with a rainbow sign. A card carries one normal
// modification, a special card one more on top.
//
// Tiny cards (about 16 to 26 px) use the small face: the index letter and
// the suit sign, big, with a curse or Untot as a thin ring. Pure drawing,
// no game logic: the band text is the card's value from the core,
// formatted with the catalog's printed figure.

import { de, cardName, contentName, face, factorText, isCurse, modFigure, t } from '../i18n';
import type { CardKind, ModFigure, Suit } from '../types';
import { faceId, indexId, modIconId, PAPER, smallFaceId, suitId, undeadFaceId, wildFaceIds } from './art';
import { backId, currentPlayerBack, type BackDesign, type BackSize } from './backs';
import { esc } from './dom';
import { fillOf, H, MOD_DRAW, modLayer, smallState, W } from './fxart';

/** A suit sign (the held sign of the deck) as a standalone inline SVG. */
export function suitIcon(suit: Suit, size = 24, label: string | null = de.suits[suit]): string {
  const a11y = label ? `role="img" aria-label="${esc(label)}"` : 'aria-hidden="true"';
  return `<svg class="suit-icon" viewBox="0 0 8 8" width="${size}" height="${size}" shape-rendering="crispEdges" ${a11y}><use href="#${suitId(suit)}"/></svg>`;
}

/** A card modification icon (its band glyph on its colour) as inline SVG. */
export function modIcon(mod: string, size = 20): string {
  return `<svg class="mod-icon" viewBox="0 0 16 16" width="${size}" height="${size}" shape-rendering="crispEdges" aria-hidden="true"><use href="#${modIconId(mod)}"/></svg>`;
}

/**
 * The plain paper of a face. A CSS variable, so the setting "Kartenfarbe
 * tönen" can tint it per suit (`data-suit` on the SVG, see style.css)
 * without drawing the cards again; the halos of marks follow it.
 */
const PLAIN = `var(--suit-paper, ${PAPER})`;

/** The modification "Untot" (core/content/card_mods.ron). */
export const UNDEAD = 'undead';
/** The modification "Wandelkarte". */
export const WILD = 'wild_card';

/**
 * "+4", "−3", "×2": the printed number of a modification. Formatting
 * only. Effekte v2: the catalog's figure is per unit of the card's value,
 * an `Add` is multiplied by it, a `Multiply` is the value itself; without
 * a figure (Gutschein Groschen) the value itself with its sign.
 */
export function figureText(figure: ModFigure | null, value = 1): string {
  if (!figure) return value > 0 ? `+${value}` : value < 0 ? `−${Math.abs(value)}` : '';
  const c = figure.change;
  if ('Multiply' in c) return `×${c.Multiply * value}`;
  if ('Scale' in c) return `×${factorText(c.Scale)}`;
  if ('Set' in c) return String(c.Set);
  const n = c.Add * value;
  return n > 0 ? `+${n}` : n < 0 ? `−${Math.abs(n)}` : '0';
}

/** The band text of a modification on a card: its value formatted. */
export function modValueText(mod: string, values: Record<string, number> = {}): string {
  return figureText(modFigure(mod), values[mod] ?? 1);
}

/** Size variant of a face: the full picture, or index and sign for tiny cards. */
export type FaceSize = 'full' | 'small';

/** Extra drawing options of a face. */
export interface FaceOptions {
  /** A voucher being redeemed: its stamp margin torn off (the redeem animation). */
  redeeming?: boolean;
}

/**
 * The face of a card as inline SVG (viewBox 33 × 49.5 art pixels). `size`
 * "small" is for tiny cards (about 16 to 26 px wide): the index letter and
 * the suit sign, big, with a curse or Untot as a thin ring, but without
 * frames and bands. Untot and Wandelkarte change the picture.
 */
export function cardSvg(kind: CardKind, mods: string[] = [], size: FaceSize = 'full', values: Record<string, number> = {}, opts: FaceOptions = {}): string {
  const f = face(kind);
  const modNames = mods.map((m) => (isCurse(m) ? `${contentName('card_mods', m)} (${de.card.curse})` : contentName('card_mods', m))).join(', ');
  const label = esc(cardName(kind) + (mods.length ? ', ' + t(de.card.mods, { mods: modNames }) : ''));
  const curse = mods.some(isCurse);
  const undead = mods.includes(UNDEAD);
  const wild = mods.includes(WILD);
  const paper = `<rect class="paper" width="${W}" height="${H}" ${fillOf(PLAIN)}/>`;
  const suit = f && !wild ? ` data-suit="${f.suit}"` : '';
  const state = undead || curse ? ` data-state="${undead && curse ? 'both' : undead ? 'undead' : 'curse'}"` : '';
  const special = wild ? ' data-special="wild"' : undead ? ' data-special="undead"' : '';
  let body: string;
  if (!f) body = `<use href="#f-blank"/>`;
  else if (size === 'small') {
    body = `${paper}<use href="#${wild ? wildFaceIds(f.rank).small : smallFaceId(f.suit, f.rank)}"/>${smallState(curse, undead)}`;
  } else {
    const picture = wild ? wildFaceIds(f.rank).picture : undead ? (undeadFaceId(f.suit, f.rank) ?? faceId(f.suit, f.rank)) : faceId(f.suit, f.rank);
    const index = wild ? wildFaceIds(f.rank).index : indexId(f.suit, f.rank);
    // One normal modification (the core allows no more); a special card changes the picture only.
    const over = mods
      .filter((m) => MOD_DRAW[m])
      .map((m) => modLayer(m, modValueText(m, values), values[m] ?? 1, PLAIN, opts.redeeming === true))
      .join('');
    body = `${paper}<use href="#${picture}"/>${over}<use href="#${index}"/>`;
  }
  return `<svg class="card-svg${size === 'small' ? ' is-small' : ''}" viewBox="0 0 ${W} ${H}" preserveAspectRatio="none" shape-rendering="crispEdges" role="img" aria-label="${label}"${suit}${state}${special}>${body}</svg>`;
}

/**
 * Back of a card in one of the back designs (a cosmetic setting, ./backs.ts).
 * `size` "small" picks the simplified picture for tiny backs.
 */
export function cardBack(className = '', design: BackDesign = currentPlayerBack(), size: BackSize = 'full'): string {
  return `<span class="card-back ${className}" role="img" aria-label="${esc(de.card.back)}"><svg viewBox="0 0 120 180" shape-rendering="crispEdges" aria-hidden="true"><use href="#${backId(design, size)}"/></svg></span>`;
}
