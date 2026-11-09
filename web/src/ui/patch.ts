// Drawing a screen from HTML strings without rebuilding what did not change
// (W4 of the review). The app renders its screen as a list of parts: a part
// is an HTML string, or a shell (one element with its opening tag and
// parts of its own). A part whose string is the same as last time stays in
// the DOM as it is; only changed parts are parsed again. A shell whose
// opening tag changed is drawn anew with everything in it, as before: its
// classes may change the layout of what lies inside, and elements that
// stayed would start CSS transitions where new ones simply are in place.
// A `sticky` shell (the run's frame, whose classes only change the cursor
// and the backdrop) keeps its element and its parts and only takes over
// the attributes of its new opening tag.
//
// The screen is not only ours: effects add classes, move cards, append
// covers. Whatever touches a part outside of `patch` marks it (a
// MutationObserver watches the screen), and a marked part is drawn again
// next time even with the same string, exactly as a full re-render did.
// Presentation only, no rules.

/** One part of a screen: HTML, or an element whose children are parts. */
export type Part = string | Shell;

export interface Shell {
  /** The opening tag with its attributes, e.g. `<div class="run">`. */
  open: string;
  kids: Part[];
  /** Keep the parts when only the opening tag changed. */
  sticky: boolean;
}

/** A shell part: `open` is the element's opening tag. */
export function shell(open: string, kids: Part[], opts: { sticky?: boolean } = {}): Shell {
  return { open, kids, sticky: opts.sticky ?? false };
}

interface Leaf {
  html: string;
  nodes: Node[];
  dirty: boolean;
}

interface Box {
  /** The shell's tag name (lower case). */
  tag: string;
  /** Its opening tag as last drawn. */
  open: string;
  el: Element;
  kids: Slots;
}

type Slot = { leaf: Leaf } | { box: Box };

interface Slots {
  slots: Slot[];
  /** Children were added or removed by somebody else: rebuild them all. */
  dirty: boolean;
}

const TAG = /^<([a-zA-Z][\w-]*)/;

function tagOf(open: string): string {
  return (TAG.exec(open)?.[1] ?? 'div').toLowerCase();
}

function parse(html: string): DocumentFragment {
  const t = document.createElement('template');
  t.innerHTML = html;
  return t.content;
}

/** The element of a shell's opening tag, without children. */
function shellElement(open: string): Element {
  const tag = tagOf(open);
  const el = parse(`${open}</${tag}>`).firstElementChild;
  return el ?? document.createElement(tag);
}

/** Make `el`'s attributes those of `model`. */
function syncAttributes(el: Element, model: Element): void {
  for (const name of el.getAttributeNames()) {
    if (!model.hasAttribute(name)) el.removeAttribute(name);
  }
  for (const name of model.getAttributeNames()) {
    const value = model.getAttribute(name) ?? '';
    if (el.getAttribute(name) !== value) el.setAttribute(name, value);
  }
}

export class Patcher {
  private readonly observer: MutationObserver;
  private top: Slots | null = null;
  /** First-level nodes of a leaf, and the leaf. */
  private readonly leafOf = new WeakMap<Node, Leaf>();
  /** The root and every shell element, and their children. */
  private readonly slotsOf = new WeakMap<Node, Slots>();

  /**
   * `replacing` is told the nodes of a part just before they leave the
   * DOM (the app notes the cards in flight there).
   */
  constructor(
    private readonly root: Element,
    private readonly replacing: (nodes: Node[]) => void = () => undefined,
  ) {
    this.observer = new MutationObserver((records) => this.mark(records));
    this.observer.observe(root, { subtree: true, childList: true, attributes: true, characterData: true });
  }

  /** Draw the parts into the root. */
  patch(parts: Part[]): void {
    this.mark(this.observer.takeRecords());
    this.top = this.fill(this.root, parts, this.top);
    // What patch itself did is no change by somebody else.
    this.observer.takeRecords();
  }

  /** Forget what is drawn: the next patch builds everything anew. */
  reset(): void {
    this.top = null;
  }

  private mark(records: MutationRecord[]): void {
    for (const r of records) {
      let n: Node | null = r.target;
      while (n) {
        const leaf = this.leafOf.get(n);
        if (leaf) {
          leaf.dirty = true;
          break;
        }
        const slots = this.slotsOf.get(n);
        if (slots) {
          // On a shell itself: attributes are compared on every patch; added or removed children are not ours.
          if (r.type === 'childList') slots.dirty = true;
          break;
        }
        n = n.parentNode;
      }
    }
  }

  private fits(prev: Slots | null, parts: Part[]): prev is Slots {
    if (!prev || prev.dirty || prev.slots.length !== parts.length) return false;
    return prev.slots.every((slot, i) => {
      const part = parts[i];
      if (part === undefined) return false;
      return typeof part === 'string' ? 'leaf' in slot : 'box' in slot && slot.box.tag === tagOf(part.open);
    });
  }

  private fill(parent: Element, parts: Part[], prev: Slots | null): Slots {
    if (!this.fits(prev, parts)) return this.build(parent, parts);
    prev.slots.forEach((slot, i) => {
      const part = parts[i] as Part;
      if (typeof part === 'string' && 'leaf' in slot) {
        if (slot.leaf.dirty || slot.leaf.html !== part) slot.leaf = this.replaceLeaf(parent, prev, i, slot.leaf, part);
      } else if (typeof part !== 'string' && 'box' in slot) {
        const changed = part.open !== slot.box.open;
        syncAttributes(slot.box.el, shellElement(part.open));
        slot.box.open = part.open;
        slot.box.kids = changed && !part.sticky ? this.build(slot.box.el, part.kids) : this.fill(slot.box.el, part.kids, slot.box.kids);
      }
    });
    return prev;
  }

  /** Everything of `parent` anew. */
  private build(parent: Element, parts: Part[]): Slots {
    const old = Array.from(parent.childNodes);
    if (old.length) this.replacing(old);
    parent.replaceChildren();
    const slots: Slots = { slots: [], dirty: false };
    this.slotsOf.set(parent, slots);
    for (const part of parts) {
      if (typeof part === 'string') {
        const leaf = this.makeLeaf(part);
        parent.append(...leaf.nodes);
        slots.slots.push({ leaf });
      } else {
        const el = shellElement(part.open);
        parent.append(el);
        slots.slots.push({ box: { tag: tagOf(part.open), open: part.open, el, kids: this.build(el, part.kids) } });
      }
    }
    return slots;
  }

  private makeLeaf(html: string): Leaf {
    const nodes = Array.from(parse(html).childNodes);
    const leaf: Leaf = { html, nodes, dirty: false };
    for (const n of nodes) this.leafOf.set(n, leaf);
    return leaf;
  }

  private replaceLeaf(parent: Element, slots: Slots, index: number, old: Leaf, html: string): Leaf {
    const leaf = this.makeLeaf(html);
    const live = old.nodes.filter((n) => n.parentNode === parent);
    if (live.length) this.replacing(live);
    // Where it goes: before its old first node, else before the next part's first node.
    let ref: Node | null = live[0] ?? null;
    for (let i = index + 1; !ref && i < slots.slots.length; i++) {
      const next = slots.slots[i];
      if (!next) continue;
      ref = 'leaf' in next ? (next.leaf.nodes.find((n) => n.parentNode === parent) ?? null) : next.box.el;
    }
    const frag = document.createDocumentFragment();
    frag.append(...leaf.nodes);
    parent.insertBefore(frag, ref);
    for (const n of live) parent.removeChild(n);
    return leaf;
  }
}
