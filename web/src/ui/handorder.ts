// "Hand sortieren: manuell": the player's own order of the hand cards.
// Display only: the core keeps its hand as it likes and sends it in its
// display order (trumps first, then the suits, for the game declared);
// the player's order is laid over it by card id. Cards new to the hand
// (the skat) come in behind the card they follow in the core's order.

/** The cards in the order of `order` (card ids); null keeps the core's order. */
export function arrangeHand<T extends { id: number }>(cards: readonly T[], order: readonly number[] | null): T[] {
  if (!order) return cards.slice();
  const rank = new Map(order.map((id, i) => [id, i]));
  const out = cards.filter((c) => rank.has(c.id)).sort((a, b) => (rank.get(a.id) ?? 0) - (rank.get(b.id) ?? 0));
  cards.forEach((c, i) => {
    if (rank.has(c.id)) return;
    // Behind its neighbour in the core's order, else at the front.
    const before = i > 0 ? cards[i - 1] : undefined;
    const at = before ? out.findIndex((x) => x.id === before.id) + 1 : 0;
    out.splice(at, 0, c);
  });
  return out;
}

/** `order` with card `id` moved next to card `target`: before it, or after it. */
export function moveInOrder(order: readonly number[], id: number, target: number, after: boolean): number[] {
  if (id === target) return order.slice();
  const out = order.filter((x) => x !== id);
  const at = out.indexOf(target);
  if (at < 0) return order.slice();
  out.splice(after ? at + 1 : at, 0, id);
  return out;
}
