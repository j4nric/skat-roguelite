// The city map of the Kneipentour (approved design "Karte Runde 3"): a
// seeded street network per district, drawn in big pixels, and the three
// ways of a leg routed along its streets. Pure presentation: the network,
// the places of the pubs and the routes are decoration derived from the
// core's map seed and node ids (same seed, same map). No game state is
// decided here: which way leads where, its stop and its minutes all come
// from the core's route view.
//
// One art pixel is drawn as one canvas pixel and scaled up crisply. The
// "core" frame is MW × MH art pixels (2 CSS px each at 390 px width); the
// canvas reaches beyond it so wider or taller screens see more city.

export const MW = 195;
export const MH = 309;
/** The drawn canvas in art pixels, around the core frame. */
const X0 = -100;
const Y0 = -40;
const CW = 395;
const CH = 390;

export type Look = 'altstadt' | 'hafen' | 'bahnhof';
export type Way = 'fuss' | 'bahn' | 'rad';
type Pt = [number, number];
type Cells = Pt[];

/**
 * The look of a district, by its name from the core content
 * (core/content/tour.ron): water and quays for the port, tracks for the
 * station and the newer quarters, river and old lanes for the rest.
 */
export function lookOf(name: string): Look {
  const n = name.toLowerCase();
  if (n.includes('hafen')) return 'hafen';
  if (['bahnhof', 'zeche', 'neustadt', 'gründerzeit'].some((w) => n.includes(w))) return 'bahnhof';
  return 'altstadt';
}

interface Palette {
  bg: string;
  house: string;
  roof: string;
  street: string;
  side: string;
  edge: string;
  water: string;
  waterDark: string;
  park: string;
  parkDark: string;
  plaza: string;
  bridge: string;
}

const PALETTE: Record<Look, Palette> = {
  altstadt: { bg: '#cdbf9f', house: '#c1b290', roof: '#b3a37e', street: '#fbf7ec', side: '#f4eedd', edge: '#a8987a', water: '#5f8f91', waterDark: '#2f6466', park: '#a9b58c', parkDark: '#8d9a70', plaza: '#efe7d2', bridge: '#d9cfb8' },
  hafen: { bg: '#c9c8bd', house: '#bbb9ac', roof: '#acaa9c', street: '#f6f6f0', side: '#ecebe2', edge: '#9c9a8e', water: '#4f8288', waterDark: '#21585c', park: '#a3ae8e', parkDark: '#87916f', plaza: '#ebe9df', bridge: '#d4d2c6' },
  // Grey-petrol: concrete, slate roofs, the petrol of the station hall.
  bahnhof: { bg: '#b7bfbd', house: '#a9b3b2', roof: '#97a3a3', street: '#f3f5f2', side: '#e5eae7', edge: '#85928f', water: '#4f8288', waterDark: '#24585c', park: '#9cab92', parkDark: '#7f9076', plaza: '#e9ece8', bridge: '#d0d6d3' },
};

export const INK = '#2a2320';
const PAPER = '#fbf6e8';

// ---------------------------------------------------------------- randomness

/** FNV-1a over a string: the seed of every random stream here. */
export function hashText(text: string): number {
  let h = 0x811c9dc5;
  for (let i = 0; i < text.length; i += 1) {
    h ^= text.charCodeAt(i);
    h = Math.imul(h, 0x01000193) >>> 0;
  }
  return h >>> 0;
}

/** A small seeded generator (mulberry32) with the helpers the map needs. */
class Rng {
  private s: number;
  constructor(seed: number) {
    this.s = seed >>> 0 || 0x9e3779b9;
  }
  next(): number {
    this.s = (this.s + 0x6d2b79f5) >>> 0;
    let t = this.s;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  }
  int(a: number, b: number): number {
    return a + Math.floor(this.next() * (b - a + 1));
  }
  uniform(a: number, b: number): number {
    return a + this.next() * (b - a);
  }
  pick<T>(list: T[]): T {
    return list[Math.floor(this.next() * list.length)] as T;
  }
  pickTwo<T>(list: T[]): [T, T] {
    const i = Math.floor(this.next() * list.length);
    let j = Math.floor(this.next() * (list.length - 1));
    if (j >= i) j += 1;
    return [list[i] as T, list[j] as T];
  }
}

// ---------------------------------------------------------------- geometry

function bez(p0: Pt, p1: Pt, p2: Pt, p3: Pt): Cells {
  const n = Math.floor(Math.max(Math.abs(p3[0] - p0[0]), Math.abs(p3[1] - p0[1])) * 1.4) + 6;
  const out: Cells = [];
  for (let i = 0; i <= n; i += 1) {
    const t = i / n;
    const a = (1 - t) ** 3;
    const b = 3 * (1 - t) ** 2 * t;
    const c = 3 * (1 - t) * t * t;
    const d = t ** 3;
    const cell: Pt = [Math.round(a * p0[0] + b * p1[0] + c * p2[0] + d * p3[0]), Math.round(a * p0[1] + b * p1[1] + c * p2[1] + d * p3[1])];
    const last = out[out.length - 1];
    if (!last || last[0] !== cell[0] || last[1] !== cell[1]) out.push(cell);
  }
  return out;
}

function disc(cx: number, cy: number, r: number): Cells {
  const out: Cells = [];
  for (let x = Math.floor(cx - r); x <= Math.floor(cx + r); x += 1) {
    for (let y = Math.floor(cy - r); y <= Math.floor(cy + r); y += 1) {
      if ((x - cx) ** 2 + (y - cy) ** 2 <= r * r) out.push([x, y]);
    }
  }
  return out;
}

const d2 = (a: Pt, b: Pt): number => (a[0] - b[0]) ** 2 + (a[1] - b[1]) ** 2;

// ---------------------------------------------------------------- the network

interface Node {
  key: string;
  i: number;
  j: number;
  p: Pt;
}

interface Edge {
  key: string;
  a: string;
  b: string;
  geo: Cells;
  main: boolean;
  tram: boolean;
  bridge: boolean;
}

export interface City {
  key: string;
  look: Look;
  pal: Palette;
  seed: number;
  nodes: Map<string, Node>;
  edges: Map<string, Edge>;
  adj: Map<string, string[]>;
  alleys: Cells[];
  plazas: Pt[];
  parks: [number, number, number, number][];
  /** River (Altstadt) or the middle line of the tracks (Bahnhofsviertel). */
  river: Cells | null;
  tracks: Cells | null;
  station: { x: number; y: number } | null;
  /** The quay line (Hafen). */
  quay: number;
}

const nk = (i: number, j: number): string => `${i},${j}`;
const ek = (a: string, b: string): string => `${a}|${b}`;

/** A seeded street network for one district: every street joins others. */
export function genCity(seedText: string, look: Look): City {
  const seed = hashText(`${look}:${seedText}`);
  const rnd = new Rng(seed);
  const pal = PALETTE[look];
  const nodes = new Map<string, Node>();
  const cx = rnd.int(70, 125);
  const cy = rnd.int(120, 180);
  const qy = rnd.int(246, 262);
  const turn = seed % 2 ? 1 : -1;
  const jitter = look === 'bahnhof' ? 4 : 7;
  for (let j = -1; j <= 13; j += 1) {
    for (let i = -3; i <= 11; i += 1) {
      let x = -18 + i * 30 + rnd.uniform(-jitter, jitter);
      let y = -18 + j * 30 + rnd.uniform(-jitter, jitter);
      if (look === 'altstadt') {
        // Swirl round the old town centre: curved streets.
        const dx = x - cx;
        const dy = y - cy;
        const a = 0.75 * Math.exp(-Math.hypot(dx, dy) / 70) * turn;
        x = cx + dx * Math.cos(a) - dy * Math.sin(a);
        y = cy + dx * Math.sin(a) + dy * Math.cos(a);
      } else if (look === 'hafen') {
        y += 9 * Math.sin(x / 38 + (seed % 7));
        if (y > qy - 4 + 6 * Math.sin(x / 50)) continue;
      } else {
        y += 3 * Math.sin(x / 45 + (seed % 5));
      }
      nodes.set(nk(i, j), { key: nk(i, j), i, j, p: [x, y] });
    }
  }
  let river: Cells | null = null;
  let tracks: Cells | null = null;
  let barrier: Cells | null = null;
  let push = 13;
  if (look === 'altstadt') {
    const y0 = rnd.int(110, 230);
    river = bez([X0 - 10, y0], [40, y0 + rnd.int(-50, 50)], [150, y0 + rnd.int(-50, 50)], [X0 + CW + 10, y0 + rnd.int(-30, 30)]);
    barrier = river;
  } else if (look === 'bahnhof') {
    // The tracks: an almost straight band across the district.
    const y0 = rnd.int(120, 165);
    const y1 = y0 + rnd.int(-22, 22);
    tracks = bez([X0 - 10, y0 - (y1 - y0) * 0.5], [60, y0], [140, y1], [X0 + CW + 10, y1 + (y1 - y0) * 0.5]);
    barrier = tracks;
    push = 17;
  }
  if (barrier) {
    // Push nodes out of the river or the tracks onto their banks.
    for (const n of nodes.values()) {
      let best = barrier[0] as Pt;
      let bd = Infinity;
      for (const c of barrier) {
        const d = d2(c, n.p);
        if (d < bd) {
          bd = d;
          best = c;
        }
      }
      const dx = n.p[0] - best[0];
      const dy = n.p[1] - best[1];
      const dist = Math.hypot(dx, dy) || 0.01;
      if (dist < push) n.p = [best[0] + (dx / dist) * push, best[1] + (dy / dist) * push];
    }
  }
  // Edges: right and down neighbours, a few diagonals.
  const pairs: [string, string][] = [];
  const has = new Set<string>();
  const addPair = (a: string, b: string): void => {
    if (has.has(ek(a, b))) return;
    has.add(ek(a, b));
    pairs.push([a, b]);
  };
  for (const n of nodes.values()) {
    for (const [di, dj] of [
      [1, 0],
      [0, 1],
    ] as const) {
      const o = nk(n.i + di, n.j + dj);
      if (nodes.has(o)) addPair(n.key, o);
    }
    const diag = nk(n.i + 1, n.j + 1);
    if (look !== 'bahnhof' && rnd.next() < 0.12 && nodes.has(diag)) addPair(n.key, diag);
  }
  const deg = new Map<string, number>();
  for (const n of nodes.keys()) deg.set(n, 0);
  const bump = (k: string, by: number): void => {
    deg.set(k, (deg.get(k) ?? 0) + by);
  };
  for (const [a, b] of pairs) {
    bump(a, 1);
    bump(b, 1);
  }
  const live = new Set(pairs.map(([a, b]) => ek(a, b)));
  const P = (k: string): Pt => (nodes.get(k) as Node).p;
  const crosses = (a: string, b: string): boolean => {
    if (!barrier) return false;
    const pa = P(a);
    const pb = P(b);
    const limit = look === 'bahnhof' ? 100 : 49;
    for (const t of [0.25, 0.5, 0.75]) {
      const q: Pt = [pa[0] + (pb[0] - pa[0]) * t, pa[1] + (pb[1] - pa[1]) * t];
      for (let k = 0; k < barrier.length; k += 3) if (d2(barrier[k] as Pt, q) < limit) return true;
    }
    return false;
  };
  // Few bridges (or underpasses): drop most crossings, keeping no dead ends.
  const bridges = new Set<string>();
  const keepShare = look === 'bahnhof' ? 0.12 : 0.3;
  // Under the tracks: three passages spread over the frame for sure.
  const passages = look === 'bahnhof' ? [rnd.int(1, 2), rnd.int(3, 4), rnd.int(5, 6)] : [];
  for (const [a, b] of pairs) {
    if (!crosses(a, b)) continue;
    const e = ek(a, b);
    const na = nodes.get(a) as Node;
    const nb = nodes.get(b) as Node;
    if (na.i === nb.i && passages.includes(na.i)) bridges.add(e);
    else if (rnd.next() < keepShare) bridges.add(e);
    else if ((deg.get(a) ?? 0) > 2 && (deg.get(b) ?? 0) > 2) {
      live.delete(e);
      bump(a, -1);
      bump(b, -1);
    } else bridges.add(e);
  }
  // Merge some blocks: drop streets only where no dead end appears.
  for (const [a, b] of pairs) {
    const e = ek(a, b);
    if (!live.has(e) || bridges.has(e)) continue;
    if (rnd.next() < 0.14 && (deg.get(a) ?? 0) > 2 && (deg.get(b) ?? 0) > 2) {
      live.delete(e);
      bump(a, -1);
      bump(b, -1);
    }
  }
  // Hierarchy: one main row and one main column (tram on the row).
  let maxJ = -99;
  for (const n of nodes.values()) maxJ = Math.max(maxJ, n.j);
  const mrow = look === 'hafen' ? maxJ : rnd.int(4, 7);
  const mcol = rnd.int(2, 5);
  const kept = pairs.filter(([a, b]) => live.has(ek(a, b)));
  const N = (k: string): Node => nodes.get(k) as Node;
  const main = new Set<string>();
  const tram = new Set<string>();
  for (const [a, b] of kept) {
    const na = N(a);
    const nb = N(b);
    if ((na.j === mrow && nb.j === mrow) || (na.i === mcol && nb.i === mcol)) main.add(ek(a, b));
    if (look !== 'hafen' && na.j === mrow && nb.j === mrow) tram.add(ek(a, b));
  }
  if (look === 'hafen') {
    // The quay: the lowest node of each column, joined along the water.
    const low = new Map<number, number>();
    for (const n of nodes.values()) if (!low.has(n.i) || n.j > (low.get(n.i) as number)) low.set(n.i, n.j);
    for (let i = -3; i < 11; i += 1) {
      if (!low.has(i) || !low.has(i + 1)) continue;
      const a = nk(i, low.get(i) as number);
      const b = nk(i + 1, low.get(i + 1) as number);
      const e = ek(a, b);
      if (!live.has(e)) {
        live.add(e);
        kept.push([a, b]);
      }
      main.add(e);
      tram.add(e);
    }
  }
  // Curved geometry per street.
  const edges = new Map<string, Edge>();
  const bendMax = look === 'bahnhof' ? 1.5 : 4;
  for (const [a, b] of kept) {
    const pa = P(a);
    const pb = P(b);
    const dx = pb[0] - pa[0];
    const dy = pb[1] - pa[1];
    const L = Math.hypot(dx, dy) || 1;
    const nx = -dy / L;
    const ny = dx / L;
    const bend = rnd.uniform(-bendMax, bendMax);
    const geo = bez(pa, [pa[0] + dx / 3 + nx * bend, pa[1] + dy / 3 + ny * bend], [pa[0] + (2 * dx) / 3 + nx * bend, pa[1] + (2 * dy) / 3 + ny * bend], pb);
    const e = ek(a, b);
    edges.set(e, { key: e, a, b, geo, main: main.has(e), tram: tram.has(e), bridge: bridges.has(e) });
  }
  // The station hall stands on the tracks' bank: no street runs through it.
  let station: { x: number; y: number } | null = null;
  const underHall = (c: Pt): boolean => station !== null && Math.abs(c[0] - station.x) <= 33 && c[1] >= station.y - 34 && c[1] <= station.y + 3;
  if (tracks) {
    // The hall between the passages, as far from them as it gets.
    const cols = passages.map((i) => -18 + i * 30);
    let sx = 97;
    let room = -1;
    for (let x = 50; x <= 145; x += 1) {
      const r = Math.min(...cols.map((c) => Math.abs(c - x))) + rnd.next() * 3;
      if (r > room) {
        room = r;
        sx = x;
      }
    }
    const near = tracks.reduce((best, c) => (Math.abs(c[0] - sx) < Math.abs(best[0] - sx) ? c : best), tracks[0] as Pt);
    station = { x: sx, y: near[1] };
    for (const e of [...edges.values()]) if (!e.bridge && e.geo.some(underHall)) edges.delete(e.key);
  }
  // Alleys inside some blocks: from the middle of one street to the middle of another.
  const alleys: Cells[] = [];
  const elist = [...edges.values()];
  for (let k = 0; k < 40 && elist.length > 1; k += 1) {
    const [e1, e2] = rnd.pickTwo(elist);
    const m1 = e1.geo[Math.floor(e1.geo.length / 2)] as Pt;
    const m2 = e2.geo[Math.floor(e2.geo.length / 2)] as Pt;
    const dist = Math.hypot(m1[0] - m2[0], m1[1] - m2[1]);
    if (dist > 18 && dist < 34) {
      const alley = bez(m1, [m1[0], (m1[1] + m2[1]) / 2], [m2[0], (m1[1] + m2[1]) / 2], m2);
      if (!alley.some(underHall)) alleys.push(alley);
    }
  }
  const adj = new Map<string, string[]>();
  for (const n of nodes.keys()) adj.set(n, []);
  for (const e of edges.values()) {
    adj.get(e.a)?.push(e.b);
    adj.get(e.b)?.push(e.a);
  }
  // Plazas at well-connected junctions inside the frame.
  const inside = [...nodes.values()].filter((n) => n.p[0] > 25 && n.p[0] < 170 && n.p[1] > 30 && n.p[1] < 260 && (adj.get(n.key)?.length ?? 0) >= 4);
  const plazas: Pt[] = [];
  if (inside.length >= 2) {
    const [a, b] = rnd.pickTwo(inside);
    plazas.push(a.p, b.p);
  } else if (inside.length === 1) plazas.push((inside[0] as Node).p);
  const parks: [number, number, number, number][] = [];
  const parkable = [...nodes.values()].filter((n) => nodes.has(nk(n.i + 1, n.j + 1)));
  for (let k = 0; k < 5 && parkable.length; k += 1) {
    const n = rnd.pick(parkable);
    const a = n.p;
    const b = P(nk(n.i + 1, n.j + 1));
    parks.push([a[0] + 5, a[1] + 5, b[0] - a[0] - 10, b[1] - a[1] - 10]);
  }
  if (station) {
    // The Bahnhofsplatz in front of the hall.
    plazas.unshift([station.x, station.y - 40]);
  }
  return { key: `${look}:${seedText}`, look, pal, seed, nodes, edges, adj, alleys, plazas, parks, river, tracks, station, quay: qy };
}

// ---------------------------------------------------------------- one leg

export interface LegPlan {
  pos: Pt;
  target: Pt;
  spelunke: Pt | null;
  future: Pt[];
  routes: Record<Way, Cells>;
}

function edgeOf(city: City, u: string, v: string): Edge | undefined {
  return city.edges.get(ek(u, v)) ?? city.edges.get(ek(v, u));
}

/** Shortest way along the streets, with penalties for streets already taken. */
function shortest(city: City, from: string, to: string, penal: Map<string, number>, inFrame: (k: string) => boolean): string[] | null {
  const dist = new Map<string, number>([[from, 0]]);
  const prev = new Map<string, string>();
  const done = new Set<string>();
  // The networks are small: a plain scan for the nearest open node will do.
  const open = new Set<string>([from]);
  while (open.size) {
    let u = '';
    let best = Infinity;
    for (const k of open) {
      const d = dist.get(k) ?? Infinity;
      if (d < best) {
        best = d;
        u = k;
      }
    }
    open.delete(u);
    if (u === to) break;
    done.add(u);
    for (const v of city.adj.get(u) ?? []) {
      if (done.has(v) || !inFrame(v)) continue;
      const e = edgeOf(city, u, v) as Edge;
      const w = e.geo.length * (penal.get(e.key) ?? 1);
      if (best + w < (dist.get(v) ?? Infinity)) {
        dist.set(v, best + w);
        prev.set(v, u);
        open.add(v);
      }
    }
  }
  if (!dist.has(to)) return null;
  const path = [to];
  while (path[path.length - 1] !== from) path.push(prev.get(path[path.length - 1] as string) as string);
  return path.reverse();
}

function pathCells(city: City, path: string[]): Cells {
  const out: Cells = [];
  for (let k = 0; k + 1 < path.length; k += 1) {
    const u = path[k] as string;
    const v = path[k + 1] as string;
    const e = edgeOf(city, u, v) as Edge;
    const seg = e.a === u ? e.geo : [...e.geo].reverse();
    out.push(...(out.length ? seg.slice(1) : seg));
  }
  return out;
}

/**
 * Where the position, the target and a Spelunke sit for one leg, and the
 * three ways along the streets (each later way avoids the streets the
 * earlier ones took, so they differ). `legKey` holds the node ids.
 */
export function planLeg(city: City, legKey: string, spelunkeWay: Way | null, futureCount: number): LegPlan {
  const rnd = new Rng(hashText(`${city.key}|${legKey}`));
  const all = [...city.nodes.values()].filter((n) => (city.adj.get(n.key)?.length ?? 0) >= 2);
  const vis = all.filter((n) => n.p[0] > 4 && n.p[0] < MW - 4 && n.p[1] > 4 && n.p[1] < MH - 4);
  const band = (y0: number, y1: number, x0: number, x1: number): Node[] => vis.filter((n) => n.p[1] >= y0 && n.p[1] <= y1 && n.p[0] > x0 && n.p[0] < x1);
  const posC = band(160, 195, 35, 160);
  const pos = rnd.pick(posC.length ? posC : vis);
  const tgtC = band(48, 82, 35, 150);
  const tgt = rnd.pick(tgtC.length ? tgtC : vis);
  const spC = band(52, 120, 30, 160).filter((n) => Math.abs(n.p[0] - tgt.p[0]) > 45 && n.key !== pos.key);
  const spel = rnd.pick(spC.length ? spC : tgtC.filter((n) => n.key !== tgt.key).length ? tgtC.filter((n) => n.key !== tgt.key) : vis);
  const futC = vis.filter((n) => n.p[1] < 34 && n.p[1] > 12);
  const future: Pt[] = [];
  for (let k = 0; k < futureCount && futC.length; k += 1) future.push(rnd.pick(futC).p);
  const penal = new Map<string, number>();
  const routes = {} as Record<Way, Cells>;
  // The ways stay inside the frame that every screen shows.
  const frame = (k: string): boolean => {
    const p = (city.nodes.get(k) as Node).p;
    return p[0] > 3 && p[0] < MW - 3 && p[1] > 3 && p[1] < MH - 3;
  };
  for (const way of ['fuss', 'rad', 'bahn'] as Way[]) {
    const to = way === spelunkeWay ? spel : tgt;
    const path = shortest(city, pos.key, to.key, penal, frame) ?? shortest(city, pos.key, to.key, penal, () => true);
    if (!path || path.length < 2) {
      routes[way] = bez(pos.p, [pos.p[0], (pos.p[1] + to.p[1]) / 2], [to.p[0], (pos.p[1] + to.p[1]) / 2], to.p);
      continue;
    }
    for (let k = 0; k + 1 < path.length; k += 1) {
      const e = edgeOf(city, path[k] as string, path[k + 1] as string) as Edge;
      penal.set(e.key, (penal.get(e.key) ?? 1) * 3.5);
    }
    routes[way] = pathCells(city, path);
  }
  return { pos: pos.p, target: tgt.p, spelunke: spelunkeWay ? spel.p : null, future, routes };
}

// ---------------------------------------------------------------- the base image

interface Painter {
  sq(cells: Cells, colour: string, w: number): void;
  rect(x: number, y: number, w: number, h: number, colour: string): void;
}

function paintCity(city: City, p: Painter): void {
  const d = city.pal;
  const rnd = new Rng(city.seed ^ 0x5bd1e995);
  p.rect(X0, Y0, CW, CH, d.bg);
  // House roofs: small blocks with a darker eave.
  for (let k = 0; k < 980; k += 1) {
    const x = rnd.int(X0 - 4, X0 + CW);
    const y = rnd.int(Y0 - 4, Y0 + CH);
    const w = rnd.int(4, 8);
    const h = rnd.int(3, 6);
    p.rect(x, y, w, h, d.house);
    p.rect(x, y + h - 1, w, 1, d.roof);
  }
  for (const [x, y, w, h] of city.parks) {
    if (w > 6 && h > 6) {
      p.rect(Math.round(x), Math.round(y), Math.round(w), Math.round(h), d.park);
      p.sq(
        [
          [Math.floor(x + w / 3), Math.floor(y + h / 3)],
          [Math.floor(x + (2 * w) / 3), Math.floor(y + (2 * h) / 2.5)],
        ],
        d.parkDark,
        3,
      );
    }
  }
  if (city.river) {
    p.sq(city.river, d.waterDark, 16);
    p.sq(city.river, d.water, 13);
  }
  if (city.tracks) {
    // Ballast, then four rails with sleepers.
    p.sq(city.tracks, '#7f8481', 24);
    p.sq(city.tracks, '#8e928e', 21);
    for (const off of [-8, -3, 3, 8]) {
      const rail = city.tracks.map(([x, y]) => [x, y + off] as Pt);
      p.sq(rail.filter((_, k) => k % 3 === 0), '#5e5a52', 3);
      p.sq(rail, '#3d4a4c', 1);
    }
  }
  if (city.look === 'hafen') {
    const qy = city.quay;
    const wpts: Cells = [];
    for (let x = X0; x <= X0 + CW; x += 1) wpts.push([x, Math.round(qy + 2 + 6 * Math.sin(x / 50))]);
    for (const [x, y] of wpts) p.rect(x, y, 1, Y0 + CH - y, d.water);
    p.sq(wpts, d.waterDark, 2);
    const prnd = new Rng(city.seed ^ 0x27d4eb2f);
    for (let k = -2; k < 8; k += 1) {
      const x = 14 + k * 40 + prnd.int(-6, 6);
      const y = qy + 4 + Math.round(6 * Math.sin(x / 50));
      const pts = bez([x, y], [x, y + 10], [x + prnd.int(-8, 8), y + 20], [x + prnd.int(-10, 10), y + 34 + prnd.int(0, 30)]);
      p.sq(pts, d.edge, 7);
      p.sq(pts, d.street, 5);
    }
  }
  // Streets: kerbs first, inner fills after.
  const edges = [...city.edges.values()];
  for (const e of edges) p.sq(e.geo, d.edge, (e.main ? 6 : 4) + 2);
  for (const a of city.alleys) p.sq(a, d.edge, 4);
  for (const a of city.alleys) p.sq(a, d.side, 2);
  for (const e of edges) p.sq(e.geo, e.main ? d.street : d.side, e.main ? 6 : 4);
  const barrier = city.river ?? city.tracks;
  if (barrier) {
    const near = city.tracks ? 144 : 64;
    for (const e of edges) {
      if (!e.bridge) continue;
      const cells = e.geo.filter((c) => {
        for (let k = 0; k < barrier.length; k += 2) if (d2(barrier[k] as Pt, c) < near) return true;
        return false;
      });
      p.sq(cells, INK, 8);
      p.sq(cells, d.bridge, 6);
    }
  }
  for (const e of edges) {
    if (!e.tram) continue;
    p.sq(e.geo, '#8f8a80', 1);
    p.sq(
      e.geo.filter((_, k) => k % 5 === 0),
      '#6e6a62',
      2,
    );
  }
  for (const [x, y] of city.plazas) {
    p.sq(disc(x, y, 10), d.edge, 1);
    p.sq(disc(x, y, 9), d.plaza, 1);
    p.sq(disc(x, y, 2.5), d.water, 1);
  }
  if (city.station) {
    // The station hall above the tracks: petrol roof, clock, platforms.
    const { x, y } = city.station;
    const top = Math.round(y - 30);
    p.rect(x - 40, Math.round(y - 2), 80, 4, '#d9ddd7');
    p.rect(x - 30, top - 1, 60, 20, INK);
    p.rect(x - 29, top, 58, 18, '#c9c4b5');
    p.rect(x - 29, top, 58, 6, '#2f5d63');
    p.rect(x - 29, top + 6, 58, 1, '#1e4044');
    for (let k = 0; k < 5; k += 1) p.rect(x - 24 + k * 11, top + 9, 5, 7, '#6f7a78');
    p.rect(x - 3, top - 5, 7, 7, INK);
    p.rect(x - 2, top - 4, 5, 5, PAPER);
    p.rect(x, top - 3, 1, 2, INK);
  }
}

const IMAGES = new Map<string, string>();

/** The city drawn at one canvas pixel per art pixel, as a PNG data URL (cached). */
export function cityImage(city: City): string {
  const hit = IMAGES.get(city.key);
  if (hit) return hit;
  let url = '';
  try {
    const canvas = document.createElement('canvas');
    canvas.width = CW;
    canvas.height = CH;
    const ctx = canvas.getContext('2d');
    if (ctx) {
      ctx.imageSmoothingEnabled = false;
      const rect = (x: number, y: number, w: number, h: number, colour: string): void => {
        ctx.fillStyle = colour;
        ctx.fillRect(Math.round(x - X0), Math.round(y - Y0), Math.round(w), Math.round(h));
      };
      paintCity(city, {
        rect,
        sq(cells, colour, w) {
          ctx.fillStyle = colour;
          const h = Math.floor(w / 2);
          for (const [x, y] of cells) ctx.fillRect(Math.round(x) - h - X0, Math.round(y) - h - Y0, w, w);
        },
      });
      url = canvas.toDataURL('image/png');
    }
  } catch {
    url = '';
  }
  IMAGES.set(city.key, url);
  return url;
}

// ---------------------------------------------------------------- vector overlay helpers

export type Pattern = 'solid' | 'dashed' | 'dotted';

/** Square stamps along cells, as one SVG path (dashed and dotted by count). */
export function stamps(cells: Cells, colour: string, w: number, pattern: Pattern = 'solid', extra = ''): string {
  const h = Math.floor(w / 2);
  const d: string[] = [];
  cells.forEach(([x, y], k) => {
    if (pattern === 'dashed' && Math.floor(k / 6) % 2) return;
    if (pattern === 'dotted' && k % 8 > 1) return;
    d.push(`M${x - h} ${y - h}h${w}v${w}h-${w}z`);
  });
  return d.length ? `<path d="${d.join('')}" fill="${colour}"${extra}/>` : '';
}

/** The base layer element for the SVG overlay. */
export function cityImageTag(city: City): string {
  const url = cityImage(city);
  if (!url) return `<rect x="${X0}" y="${Y0}" width="${CW}" height="${CH}" fill="${city.pal.bg}"/>`;
  return `<image href="${url}" x="${X0}" y="${Y0}" width="${CW}" height="${CH}" preserveAspectRatio="none" style="image-rendering:pixelated"/>`;
}
