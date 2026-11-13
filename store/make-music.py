"""Soundtrack of the App Store preview: jazz, composed for this project and
synthesized here from sine partials and seeded noise (numpy only). No
recording, no third-party music, no licence question. Deterministic.

    python3 store/make-music.py <variant> <out.wav>      # variant: jazz-a | jazz-b
    python3 store/make-music.py jazz-a out.wav --levels  # print the level of each part

The cuts of the video come from store/grid.py: every scene starts on a
beat, and the music marks the cuts with hits. Both variants follow the same
plan: a pickup into the head, a drum break that lands the stamp (342) on
its beat with the whole band, a shout chorus for the boss, a build under
the city map and a clean final chord on the end card.

- jazz-a: uptempo swing in F at 160 BPM. Clarinet lead, piano comping with
  rootless voicings, walking upright bass, ride cymbal with the hi-hat on 2
  and 4, a brass section for hits and the shout chorus.
- jazz-b: a slower blues shuffle in F at 120 BPM. Tenor-ish saxophone lead
  with blue notes, Hammond-style organ with a rotary wobble, piano, walking
  bass, shuffle drums with a backbeat, the horns for the shout.
"""

import sys
import wave

import numpy as np

sys.path.insert(0, __import__('os').path.dirname(__file__))
from grid import grid  # noqa: E402

SR = 44100
ARGS = [a for a in sys.argv[1:] if not a.startswith('--')]
VARIANT = ARGS[0] if ARGS else 'jazz-a'
OUT = ARGS[1] if len(ARGS) > 1 else f'music-{VARIANT}.wav'
G = grid(VARIANT)
BEAT = G['beat']
TOTAL = G['total']
SC = G['scenes']
STAMP = G['stamp']
N = int((TOTAL * BEAT + 0.05) * SR)
rng = np.random.default_rng(1813 if VARIANT == 'jazz-a' else 1927)

SWING = 0.63 if VARIANT == 'jazz-a' else 0.68

NOTE = {'C': 0, 'C#': 1, 'Db': 1, 'D': 2, 'D#': 3, 'Eb': 3, 'E': 4, 'F': 5, 'F#': 6, 'Gb': 6, 'G': 7, 'G#': 8, 'Ab': 8, 'A': 9, 'A#': 10, 'Bb': 10, 'B': 11}


def m(name):
    return NOTE[name[:-1]] + 12 * (int(name[-1]) + 1)


def hz(midi):
    return 440.0 * 2 ** ((midi - 69) / 12)


def at(beat):
    """Seconds of a beat position written in straight eighths; offbeat eighths swing."""
    whole = np.floor(beat + 1e-9)
    f = beat - whole
    if f < 0.5:
        s = f / 0.5 * SWING
    else:
        s = SWING + (f - 0.5) / 0.5 * (1 - SWING)
    return (whole + s) * BEAT


def human(ms=7):
    return rng.normal(0, ms / 1000)


def vel(v, spread=0.08):
    return max(0.05, v * (1 + rng.normal(0, spread)))


class Track:
    def __init__(self, pan=0.0, room=0.15):
        self.buf = np.zeros(N)
        self.pan = pan
        self.room = room

    def add(self, t, sig):
        i = int(round(max(0.0, t) * SR))
        if i >= N:
            return
        j = min(N, i + len(sig))
        self.buf[i:j] += sig[: j - i]


# ------------------------------------------------------------------ voices

def partials(freqs, amps, decays, n, phase=None):
    t = np.arange(n) / SR
    out = np.zeros(n)
    for k, (f, a, d) in enumerate(zip(freqs, amps, decays)):
        if f > 17000 or a == 0:
            continue
        p = 0.0 if phase is None else phase[k]
        out += a * np.exp(-t * d) * np.sin(2 * np.pi * f * t + p)
    return out


def release(sig, hold, r):
    n = len(sig)
    t = np.arange(n) / SR
    return sig * np.clip(1 - (t - hold) / r, 0, 1) ** 2


def upright(midi, dur, v=0.5):
    """Plucked upright bass: bright pluck, warm body, a little thump."""
    f = hz(midi)
    n = int((dur + 0.25) * SR)
    t = np.arange(n) / SR
    bend = 1 + 0.012 * np.exp(-t * 40)
    phase = 2 * np.pi * np.cumsum(f * bend) / SR
    out = np.zeros(n)
    for k in range(1, 18):
        if f * k > 6000:
            break
        out += (1 / k ** 1.25) * np.exp(-t * (1.8 + 1.1 * k)) * np.sin(k * phase)
    thump = rng.standard_normal(n) * np.exp(-t * 90) * 0.15
    thump = np.convolve(thump, np.ones(12) / 12, mode='same')
    sig = (out + thump) * (1 - np.exp(-t * 900))
    return v * release(sig, dur, 0.08)


def piano(midi, dur, v=0.4):
    f = hz(midi)
    n = int((dur + 0.6) * SR)
    B = 0.00035
    ks = np.arange(1, 15)
    freqs = ks * f * np.sqrt(1 + B * ks ** 2)
    bright = 0.6 + 0.8 * v
    amps = (1 / ks ** (1.6 - 0.5 * bright)) * (1 + 0.3 * (ks == 2))
    decays = 0.9 + 0.45 * ks + f / 900
    sig = partials(freqs, amps, decays, n) + 0.6 * partials(freqs * 1.0007, amps, decays * 1.1, n, phase=ks * 0.7)
    t = np.arange(n) / SR
    sig += rng.standard_normal(n) * np.exp(-t * 300) * 0.05 * v
    return 0.5 * v * release(sig, dur, 0.18)


def organ(midi, dur, v=0.35, t0=0.0):
    """Drawbars 16', 8', 5 1/3', 4', 2' with key click, percussion and a rotary wobble."""
    f = hz(midi)
    n = int((dur + 0.12) * SR)
    t = np.arange(n) / SR
    tt = t + t0
    wob = 1 + 0.0025 * np.sin(2 * np.pi * 6.4 * tt)
    ph = 2 * np.pi * np.cumsum(f * wob) / SR
    sig = 0.8 * np.sin(0.5 * ph) + 1.0 * np.sin(ph) + 0.55 * np.sin(1.5 * ph) + 0.45 * np.sin(2 * ph) + 0.25 * np.sin(4 * ph)
    sig += 0.5 * np.sin(3 * ph) * np.exp(-t * 9)  # percussion
    sig *= 1 + 0.18 * np.sin(2 * np.pi * 6.4 * tt + 1)  # rotary amplitude
    click = rng.standard_normal(n) * np.exp(-t * 600) * 0.15
    sig = (sig + click) * np.clip(t / 0.008, 0, 1)
    return 0.22 * v * release(sig, dur, 0.06)


def reed(midi, dur, v=0.5, kind='clarinet', scoop=0.0, fall=0.0):
    """Clarinet (odd partials, woody) or saxophone (all partials, a nasal formant, breath)."""
    f = hz(midi)
    n = int((dur + 0.12) * SR)
    t = np.arange(n) / SR
    cents = -scoop * np.exp(-t * 18) + 18 * np.clip((t - 0.22) / 0.2, 0, 1) * np.sin(2 * np.pi * 5.2 * t)
    if fall:
        cents -= fall * np.clip((t - dur * 0.6) / (dur * 0.4 + 1e-3), 0, 1) ** 2
    ph = 2 * np.pi * np.cumsum(f * 2 ** (cents / 1200)) / SR
    attack = np.clip(t / 0.035, 0, 1)
    # Brighter while the breath is strong: high partials follow the envelope.
    breath = attack * (0.8 + 0.2 * v)
    sig = np.zeros(n)
    for k in range(1, 20):
        if f * k > 12000:
            break
        if kind == 'clarinet':
            a = (1 / k) if k % 2 else 0.12 / k
        else:
            formant = np.exp(-((f * k - 1400) / 1100) ** 2)
            a = (0.9 / k ** 0.7) * (0.35 + formant)
        sig += a * (breath ** (0.5 + 0.12 * k)) * np.sin(k * ph)
    air = rng.standard_normal(n)
    air = air - np.convolve(air, np.ones(6) / 6, mode='same')
    sig += air * (0.02 if kind == 'clarinet' else 0.035) * attack
    return 0.2 * v * release(sig, dur, 0.07)


def brass(midi, dur, v=0.5):
    """Trumpet/trombone section voice: brightness swells with the attack."""
    f = hz(midi)
    n = int((dur + 0.1) * SR)
    t = np.arange(n) / SR
    env = np.clip(t / 0.03, 0, 1) * (0.75 + 0.25 * np.exp(-t * 6))
    ph = 2 * np.pi * np.cumsum(f * 2 ** ((-25 * np.exp(-t * 30)) / 1200)) / SR
    sig = np.zeros(n)
    for k in range(1, 22):
        if f * k > 12000:
            break
        sig += (1 / k ** 0.9) * env ** (0.4 + 0.18 * k) * np.sin(k * ph + k)
    return 0.12 * v * release(sig, dur, 0.06)


def noise(n):
    return rng.standard_normal(n)


def hp(x, k=3):
    return x - np.convolve(x, np.ones(k) / k, mode='same')


def lp(x, k=6):
    return np.convolve(x, np.ones(k) / k, mode='same')


RIDE_FREQS = rng.uniform(2800, 9500, 14)


def ride(v=0.25, bell=False):
    n = int(1.3 * SR)
    t = np.arange(n) / SR
    metal = sum(np.sin(2 * np.pi * f * t + i) for i, f in enumerate(RIDE_FREQS)) / 6
    sig = metal * np.exp(-t * 3.2) + 0.5 * hp(noise(n)) * np.exp(-t * 5)
    ping = np.sin(2 * np.pi * (5200 if not bell else 2400) * t) * np.exp(-t * (40 if not bell else 8))
    return v * (sig * 0.35 + 0.5 * ping)


def chick(v=0.2):
    n = int(0.09 * SR)
    t = np.arange(n) / SR
    return v * hp(noise(n), 2) * np.exp(-t * 55)


def hat(v=0.12, open_=False):
    n = int((0.3 if open_ else 0.06) * SR)
    t = np.arange(n) / SR
    return v * hp(noise(n), 2) * np.exp(-t * (10 if open_ else 65))


def kick(v=0.4):
    n = int(0.35 * SR)
    t = np.arange(n) / SR
    f = 48 + 70 * np.exp(-t * 30)
    return v * np.sin(2 * np.pi * np.cumsum(f) / SR) * np.exp(-t * 9)


def snare(v=0.3, tone=200):
    n = int(0.28 * SR)
    t = np.arange(n) / SR
    return v * (0.6 * hp(noise(n)) * np.exp(-t * 16) + 0.5 * np.sin(2 * np.pi * tone * t) * np.exp(-t * 28))


def brush(v=0.15):
    n = int(0.2 * SR)
    t = np.arange(n) / SR
    return v * lp(hp(noise(n)), 3) * np.clip(t / 0.02, 0, 1) * np.exp(-t * 14)


def tom(freq, v=0.35):
    n = int(0.4 * SR)
    t = np.arange(n) / SR
    f = freq * (1 + 0.4 * np.exp(-t * 25))
    return v * (np.sin(2 * np.pi * np.cumsum(f) / SR) * np.exp(-t * 8) + 0.1 * noise(n) * np.exp(-t * 50))


def crash(v=0.25, length=2.4):
    n = int(length * SR)
    t = np.arange(n) / SR
    metal = sum(np.sin(2 * np.pi * f * 0.9 * t + i) for i, f in enumerate(RIDE_FREQS)) / 8
    return v * (hp(noise(n)) * 0.7 + metal * 0.3) * np.exp(-t * 2.0) * (0.7 + 0.3 * np.exp(-t * 18))


# --------------------------------------------------------------- harmony

QUAL = {
    'maj7': [0, 4, 7, 11], '6': [0, 4, 7, 9], '69': [0, 4, 7, 9, 14], 'maj9': [0, 4, 7, 11, 14],
    '7': [0, 4, 7, 10], '9': [0, 4, 7, 10, 14], '7b9': [0, 4, 7, 10, 13], '7#9': [0, 4, 7, 10, 15], '13': [0, 4, 10, 14, 21],
    'm7': [0, 3, 7, 10], 'm9': [0, 3, 7, 10, 14], 'm6': [0, 3, 7, 9], 'm7b5': [0, 3, 6, 10], 'dim7': [0, 3, 6, 9],
}


def parse(sym):
    root = sym[:2] if len(sym) > 1 and sym[1] in 'b#' else sym[:1]
    return NOTE[root], sym[len(root):]


def rootless(sym, prev=None, low=m('E3'), high=m('D5')):
    """A rootless voicing (3 7 9 5 or 7 3 5 9) near the last one: voice leading."""
    root, q = parse(sym)
    iv = QUAL[q]
    third = iv[1]
    seventh = iv[3] if len(iv) > 3 else iv[2]
    ninth = 14 if q not in ('7b9', '7#9', 'm7b5', 'dim7') else (13 if q == '7b9' else (15 if q == '7#9' else iv[2]))
    fifth = iv[2] if q not in ('13',) else 21
    shapes = [[third, fifth, seventh, ninth], [seventh, ninth, third + 12, fifth + 12]]
    best = None
    for shape in shapes:
        for octave in range(2, 6):
            notes = [root + 12 * octave + x for x in shape]
            if notes[0] < low or notes[-1] > high:
                continue
            cost = 0 if prev is None else sum(abs(a - b) for a, b in zip(sorted(notes), sorted(prev)))
            if prev is None:
                cost = abs(np.mean(notes) - m('G4'))
            if best is None or cost < best[0]:
                best = (cost, notes)
    return best[1]


def chord_tones(sym):
    root, q = parse(sym)
    return [(root + x) % 12 for x in QUAL[q]]


def chord_at(chords, beat):
    cur = chords[0][1]
    for b, sym in chords:
        if b <= beat + 1e-9:
            cur = sym
    return cur


def next_change(chords, beat):
    for b, sym in chords:
        if b > beat + 1e-9:
            return b, sym
    return TOTAL, chords[-1][1]


# ---------------------------------------------------------------- tracks

bass_t = Track(0.0, 0.08)
piano_t = Track(-0.3, 0.16)
organ_t = Track(-0.2, 0.14)
lead_t = Track(0.18, 0.2)
horns_t = Track(0.05, 0.2)
ride_t = Track(0.35, 0.12)
kit_t = Track(-0.05, 0.1)
cym_t = Track(-0.3, 0.15)


def play_line(line, base, kind, gain=0.55, octave=0):
    for b, length, name, *extra in line:
        beat = base + b
        if beat >= TOTAL:
            continue
        opts = extra[0] if extra else {}
        start = at(beat) + human(9)
        end = at(beat + length)
        dur = max(0.06, (end - start) * 0.92)
        lead_t.add(start, reed(m(name) + octave, dur, vel(gain, 0.1) * opts.get('v', 1), kind=kind, scoop=opts.get('scoop', 30 if length >= 1 else 12), fall=opts.get('fall', 0)))


def walk(chords, start, end, rests=()):
    """Walking bass in quarters: root on a change, chord tones, a chromatic step into the next root."""
    prev = m('F2')
    for beat in range(start, end):
        if any(a <= beat < b for a, b in rests):
            continue
        sym = chord_at(chords, beat)
        root, _ = parse(sym)
        nb, nsym = next_change(chords, beat)
        if any(abs(beat - b) < 1e-9 for b, _ in chords) or beat == start:
            pc = root
        elif nb - beat <= 1:
            target = parse(nsym)[0]
            pc = (target + rng.choice([-1, 1])) % 12
        else:
            tones = chord_tones(sym)
            pc = tones[int(rng.integers(1, len(tones)))]
        cands = [pc + 12 * o for o in range(1, 5) if m('E1') <= pc + 12 * o <= m('A2')]
        note = min(cands, key=lambda x: abs(x - prev)) if cands else pc + 24
        prev = note
        bass_t.add(at(beat) + human(5), upright(note, BEAT * 0.95, vel(0.62, 0.07)))
        # Now and then a ghosted triplet pickup into the next beat.
        if rng.random() < 0.12 and beat + 1 < end:
            bass_t.add(at(beat + 0.67) + human(4), upright(note + rng.choice([-2, 2]), BEAT * 0.3, 0.3))


def comp(chords, start, end, instrument='piano', rests=(), pattern_set=None):
    prev = None
    patterns = pattern_set or [[(0, 0.4), (1.5, 1.0)], [(1.5, 0.5), (3.5, 0.6)], [(0.5, 0.4), (2.5, 0.8)], [(0, 0.4), (2.5, 0.6)], [(1.5, 1.5)], [(-0.5, 0.9), (2, 0.4)]]
    for bar in range(start // 4, (end + 3) // 4):
        pat = patterns[int(rng.integers(0, len(patterns)))]
        for off, length in pat:
            beat = bar * 4 + off
            if beat < start or beat >= end or any(a <= beat < b for a, b in rests):
                continue
            sym = chord_at(chords, beat + 0.5)  # anticipations take the coming chord
            notes = rootless(sym, prev)
            prev = notes
            v = vel(0.38 if instrument == 'piano' else 0.5, 0.12)
            t = at(beat) + human(10)
            dur = length * BEAT
            for k, note in enumerate(notes):
                roll = k * 0.006 * rng.random()
                if instrument == 'piano':
                    piano_t.add(t + roll, piano(note, dur, v * (0.85 + 0.1 * k)))
                else:
                    organ_t.add(t, organ(note, dur, v, t0=t))


def hit(beat, sym, top, v=0.7, length=0.5, horns=True, keys=True, crash_v=0.0, low=True):
    """A band hit: horns in close position under `top`, piano, bass and kick together."""
    t = at(beat) + human(3)
    dur = length * BEAT
    root, _ = parse(sym)
    tones = sorted(set(chord_tones(sym)))
    voices = [m(top)]
    while len(voices) < 4:
        below = [voices[-1] - d for d in range(1, 12) if (voices[-1] - d) % 12 in tones]
        voices.append(below[0] if below else voices[-1] - 3)
    if horns:
        for i, note in enumerate(voices):
            horns_t.add(t + human(4), brass(note, dur, vel(v * (1.0 if i == 0 else 0.8))))
    if keys:
        for note in rootless(sym):
            piano_t.add(t, piano(note, dur, v * 0.8))
    if low:
        r = root + 12 * 2
        while r < m('E1'):
            r += 12
        bass_t.add(t, upright(r, max(dur, BEAT * 0.9), 0.75))
        kit_t.add(t, kick(0.5 * v + 0.2))
    if crash_v:
        cym_t.add(t, crash(crash_v))


def harmonize(line, chords, base, v=0.55):
    """The shout chorus: each melody note in four-part close harmony on the horns."""
    for b, length, name, *_ in line:
        beat = base + b
        sym = chord_at(chords, beat)
        hit(beat, sym, name, v=v, length=length * 0.95, keys=False, low=False)


def drums_swing(start, end, rests=(), ride_v=0.2, ghosts=0.25, kick_v=0.12):
    for beat in range(start, end):
        if any(a <= beat < b for a, b in rests):
            continue
        q = beat % 4
        accent = 1.15 if q in (1, 3) else 1.0
        ride_t.add(at(beat) + human(5), ride(vel(ride_v * accent, 0.1)))
        if q in (1, 3):
            ride_t.add(at(beat + 0.5) + human(5), ride(vel(ride_v * 0.7, 0.12)))
            kit_t.add(at(beat) + human(4), chick(vel(0.2)))
        kit_t.add(at(beat) + human(6), kick(vel(kick_v, 0.2)))  # feathered
        if rng.random() < ghosts:
            kit_t.add(at(beat + 0.5) + human(8), snare(vel(0.06, 0.3)))
        if rng.random() < ghosts * 0.4:
            kit_t.add(at(beat + 0.5) + human(6), snare(vel(0.16, 0.2)))  # a comping drop


def drums_shuffle(start, end, rests=(), v=1.0):
    for beat in range(start, end):
        if any(a <= beat < b for a, b in rests):
            continue
        q = beat % 4
        ride_t.add(at(beat) + human(5), hat(vel(0.12 * v), open_=False))
        ride_t.add(at(beat + 0.5) + human(5), hat(vel(0.08 * v)))
        if q in (1, 3):
            kit_t.add(at(beat) + human(4), snare(vel(0.32 * v, 0.06)))
        else:
            kit_t.add(at(beat) + human(4), kick(vel(0.42 * v, 0.06)))
        if q == 2 and rng.random() < 0.5:
            kit_t.add(at(beat + 0.5) + human(4), kick(vel(0.25 * v)))
        if rng.random() < 0.2:
            kit_t.add(at(beat + 0.5) + human(8), snare(vel(0.07, 0.3)))


def fill(start, end, v=0.35, toms=True):
    """Triplet fill from beat `start` to `end`, growing louder; snare to toms."""
    steps = int(round((end - start) * 3))
    for i in range(steps):
        beat = start + i / 3
        x = i / max(1, steps - 1)
        t = beat * BEAT + human(4)
        if toms and x > 0.5:
            kit_t.add(t, tom(190 - 70 * x, vel(v * (0.7 + 0.5 * x))))
        else:
            kit_t.add(t, snare(vel(v * (0.6 + 0.6 * x)), tone=210))


def cut_hits(skip=()):
    """A light hit on each cut (a stab of the band and a cymbal)."""
    for name in G['order'][1:]:
        b = SC[name][0]
        if b in skip:
            continue
        cym_t.add(at(b) + human(3), crash(0.09, 1.2))
        kit_t.add(at(b), kick(0.3))


# ----------------------------------------------------------------- scores

def score_a():
    """Uptempo swing in F, 160 BPM."""
    s_end, s_boss, s_map, s_stamp = SC['end'][0], SC['boss'][0], SC['map'][0], STAMP
    chords = [
        (0, 'C7'), (4, 'Fmaj7'), (6, 'D7b9'), (8, 'Gm7'), (10, 'C7'), (12, 'Am7'), (14, 'D7b9'), (16, 'Gm7'), (18, 'C7'),
        (20, 'Fmaj7'), (s_stamp - 2, 'C7#9'), (s_stamp, 'Fmaj9'), (s_stamp + 2, 'D7b9'), (s_stamp + 4, 'Gm7'), (s_stamp + 6, 'C7'),
        (s_stamp + 8, 'Fmaj7'), (s_stamp + 10, 'F7'), (s_stamp + 12, 'Bbmaj7'), (s_stamp + 14, 'Bbm6'), (s_stamp + 16, 'Am7'), (s_stamp + 18, 'D7b9'),
        (s_boss, 'Bbmaj7'), (s_boss + 2, 'Eb9'), (s_boss + 4, 'Am7'), (s_boss + 6, 'D7#9'),
        (s_map, 'Gm7'), (s_map + 2, 'C7'), (s_map + 4, 'Gm7'), (s_map + 6, 'C7b9'),
        (s_end, 'F69'), (s_end + 4, 'Gm7'), (s_end + 6, 'C7'), (s_end + 8, 'F69'),
    ]
    chords = sorted(chords)
    brk = (s_stamp - 2, s_stamp)
    # Intro: drum pickup, the band on beat 2 and 4 of the first bar, walking from beat 4.
    fill(2, 4, 0.3, toms=False)
    hit(0, 'C7#9', 'Bb5', v=0.7, length=0.4, crash_v=0.2)
    hit(1.5, 'C7#9', 'Bb5', v=0.55, length=0.3, low=False)
    walk(chords, 4, s_end, rests=[brk, (s_boss, s_boss + 0)])
    comp(chords, 4, s_boss, rests=[brk])
    drums_swing(4, s_boss, rests=[brk])
    # Drum break into the stamp.
    hit(brk[0], 'C7#9', 'Bb5', v=0.75, length=0.35)
    fill(brk[0] + 1 / 3, brk[1], 0.42)
    hit(s_stamp, 'Fmaj9', 'A5', v=1.0, length=1.4, crash_v=0.3)
    # The head on clarinet.
    head1 = [
        (0, .5, 'A4'), (.5, .5, 'C5'), (1, .5, 'E5'), (1.5, .5, 'G5'), (2, 1, 'F#5'), (3, .5, 'A5'), (3.5, .5, 'F#5'),
        (4, 1.5, 'F5'), (5.5, .5, 'D5'), (6, .5, 'E5'), (6.5, .5, 'Bb4'), (7, .5, 'C5'), (7.5, .5, 'E5'),
        (8, .5, 'G5'), (8.5, .5, 'E5'), (9, .5, 'C5'), (9.5, .5, 'A4'), (10, 1.5, 'C5'), (11.5, .5, 'Eb5'),
        (12, .5, 'D5'), (12.5, .5, 'Bb4'), (13, .5, 'G4'), (13.5, .5, 'F4'), (14, .5, 'E4'), (14.5, .5, 'G4'), (15, .5, 'Bb4'), (15.5, .5, 'Db5'),
    ]
    play_line([x for x in head1 if 4 + x[0] < brk[0] - 0.01], 4, 'clarinet', 0.6)
    head2 = [
        (1, .5, 'F#5'), (1.5, .5, 'A5'), (2, .5, 'C6'), (2.5, .5, 'A5'), (3, 1, 'F#5'),
        (4, .5, 'Bb5'), (4.5, .5, 'A5'), (5, .5, 'G5'), (5.5, .5, 'F5'), (6, .5, 'E5'), (6.5, .5, 'G5'), (7, 1, 'Bb5'),
        (8, 1, 'A5'), (9, .5, 'G5'), (9.5, .5, 'F5'), (10, .5, 'Eb5'), (10.5, .5, 'C5'), (11, .5, 'A4'), (11.5, .5, 'Eb5'),
        (12, 1.5, 'D5'), (13.5, .5, 'F5'), (14, 1, 'Db5'), (15, .5, 'G5'), (15.5, .5, 'F5'),
        (16, .5, 'E5'), (16.5, .5, 'C5'), (17, .5, 'A4'), (17.5, .5, 'G4'), (18, .5, 'F#4'), (18.5, .5, 'A4'), (19, .5, 'C5'), (19.5, .5, 'Eb5', {'v': 1.1}),
    ]
    play_line([x for x in head2 if s_stamp + x[0] < s_boss], s_stamp, 'clarinet', 0.62)
    # Boss: the shout chorus, full band, crash ride.
    shout = [
        (0, .5, 'D6'), (.5, .5, 'C6'), (1, .5, 'A5'), (1.5, 1, 'F5'), (2.5, .5, 'Db6'), (3, 1, 'C6'),
        (4, .5, 'C6'), (4.5, .5, 'B5'), (5, .5, 'A5'), (5.5, .5, 'G5'), (6, .5, 'F#5'), (6.5, .5, 'Eb6'), (7, 1, 'C6'),
    ]
    hit(s_boss, 'Bbmaj7', 'D6', v=1.0, length=0.45, crash_v=0.28)
    harmonize([x for x in shout[1:] if s_boss + x[0] < s_map], chords, s_boss, v=0.85)
    play_line([x for x in shout if s_boss + x[0] < s_map], s_boss, 'clarinet', 0.5, octave=0)
    comp(chords, s_boss, s_map, pattern_set=[[(0, 0.3), (1.5, 0.4), (2.5, 0.4), (3.5, 0.4)]])
    for beat in range(s_boss, s_map):
        cym_t.add(at(beat) + human(4), ride(vel(0.3), bell=beat % 2 == 0))
        kit_t.add(at(beat), kick(0.35 if beat % 2 == 0 else 0.2))
        if beat % 4 in (1, 3):
            kit_t.add(at(beat), chick(0.25))
            kit_t.add(at(beat + 0.5) + human(5), snare(0.22))
    fill(s_map - 1, s_map, 0.4)
    # Build under the city map: climbing line, the drums rise, the horns swell on C7b9.
    climb = [(0, .5, 'G4'), (.5, .5, 'Bb4'), (1, .5, 'D5'), (1.5, .5, 'F5'), (2, .5, 'E5'), (2.5, .5, 'G5'), (3, .5, 'Bb5'), (3.5, .5, 'Db6'),
             (4, 1, 'D6'), (5, .5, 'C6'), (5.5, .5, 'Bb5'), (6, 2, 'Db6', {'v': 1.15})]
    play_line([x for x in climb if s_map + x[0] < s_end], s_map, 'clarinet', 0.62)
    comp(chords, s_map, s_end - 2)
    drums_swing(s_map, s_end - 2, ride_v=0.24, ghosts=0.45)
    for i, note in enumerate(['E5', 'Db5', 'Bb4', 'G4']):
        horns_t.add(at(s_end - 2), brass(m(note), 2 * BEAT, 0.5 + 0.1 * i))
    fill(s_end - 2, s_end, 0.45)
    # The end card: the big hit, a last phrase, the final chord.
    hit(s_end, 'F69', 'A5', v=1.0, length=1.6, crash_v=0.32)
    lick = [(2, .5, 'C6'), (2.5, .5, 'A5'), (3, .5, 'G5'), (3.5, .5, 'F5'), (4, .5, 'Eb5'), (4.5, .5, 'D5'), (5, .5, 'C5'), (5.5, .5, 'A4'), (6, .5, 'Bb4'), (6.5, .5, 'Db5'), (7, .5, 'E5'), (7.5, .5, 'G5')]
    play_line(lick, s_end, 'clarinet', 0.6)
    walk(chords, s_end + 4, s_end + 8)
    comp(chords, s_end + 4, s_end + 8)
    drums_swing(s_end + 2, s_end + 8, ride_v=0.18)
    hit(s_end + 7.5, 'C7#9', 'A5', v=0.8, length=0.3)
    final = s_end + 8
    hit(final, 'F69', 'A5', v=1.0, length=TOTAL - final - 0.2, crash_v=0.3)
    play_line([(0, TOTAL - final - 0.3, 'F6', {'scoop': 40})], final, 'clarinet', 0.55)
    piano_t.add(at(final) + 0.02, piano(m('F2'), (TOTAL - final) * BEAT, 0.5))
    cut_hits(skip=(s_stamp, s_boss, s_end, SC['boss'][0], SC['end'][0]))


def score_b():
    """A slower blues shuffle in F, 120 BPM."""
    s_end, s_boss, s_map, s_stamp = SC['end'][0], SC['boss'][0], SC['map'][0], STAMP
    chords = sorted([
        (0, 'F7'), (4, 'Bb7'), (8, 'F7'), (12, 'F7'), (s_stamp - 2, 'C7#9'), (s_stamp, 'Bb9'), (s_stamp + 2, 'Bdim7'),
        (s_stamp + 4, 'F7'), (s_stamp + 8, 'D7#9'), (s_stamp + 12, 'Gm7'), (s_stamp + 16, 'C7'),
        (s_boss, 'F7'), (s_boss + 2, 'Bb7'), (s_boss + 4, 'Gm7'), (s_boss + 5, 'C7'),
        (s_map, 'Gm7'), (s_map + 2, 'C7'), (s_map + 4, 'C7#9'),
        (s_end, 'F9'), (s_end + 2, 'Bb9'), (s_end + 4, 'F7'), (s_end + 5, 'C7'), (s_end + 6, 'F13'),
    ])
    brk = (s_stamp - 2, s_stamp)
    shuffle_patterns = [[(1, 0.4), (3, 0.4)], [(1, 0.4), (2.5, 1.2)], [(0.5, 0.4), (1.5, 0.4), (3, 0.6)], [(-0.5, 1.5), (3, 0.4)]]
    # Intro: organ swell and a pickup.
    organ_t.add(0, sum(organ(n, 4 * BEAT, 0.5) for n in rootless('F7')) * np.minimum(1, np.arange(int((4 * BEAT + 0.12) * SR)) / SR / (3 * BEAT)))
    fill(2, 4, 0.3, toms=False)
    kit_t.add(0, kick(0.5))
    cym_t.add(0, crash(0.15))
    walk(chords, 4, s_end, rests=[brk])
    comp(chords, 4, s_boss, instrument='organ', rests=[brk], pattern_set=[[(0, 3.8)], [(0, 1.8), (2, 1.8)], [(-0.5, 2.3), (2, 1.8)]])
    comp(chords, 4, s_boss, rests=[brk], pattern_set=shuffle_patterns)
    drums_shuffle(4, s_boss, rests=[brk])
    hit(brk[0], 'C7#9', 'Eb5', v=0.75, length=0.35)
    fill(brk[0] + 1 / 3, brk[1], 0.42)
    hit(s_stamp, 'Bb9', 'D6', v=1.0, length=1.5, crash_v=0.3)
    sax1 = [
        (0, .5, 'F4'), (.5, .5, 'Ab4', {'scoop': 60}), (1, 1, 'Bb4'), (2, .5, 'D5'), (2.5, .5, 'F5'), (3, 1, 'Ab5', {'scoop': 80}),
        (4, 1.5, 'A5', {'scoop': 70}), (5.5, .5, 'Ab5'), (6, .5, 'F5'), (6.5, .5, 'Eb5'), (7, .5, 'C5'), (7.5, .5, 'A4'),
        (8, .5, 'C5'), (8.5, .5, 'Eb5'), (9, 1, 'F5', {'fall': 120}),
    ]
    play_line([x for x in sax1 if 4 + x[0] < brk[0] - 0.01], 4, 'sax', 0.62)
    sax2 = [
        (1, .5, 'C6'), (1.5, .5, 'Bb5'), (2, .5, 'Ab5', {'scoop': 60}), (2.5, .5, 'F5'), (3, 1, 'D5'),
        (4, .5, 'C5'), (4.5, .5, 'F5'), (5, .5, 'Ab5'), (5.5, .5, 'A5'), (6, 1.5, 'C6', {'scoop': 50}), (7.5, .5, 'A5'),
        (8, 1, 'F#5'), (9, .5, 'A5'), (9.5, .5, 'C6'), (10, 1, 'Eb6', {'scoop': 70}), (11, 1, 'D6', {'fall': 100}),
        (12, .5, 'Bb5'), (12.5, .5, 'A5'), (13, .5, 'G5'), (13.5, .5, 'F5'), (14, 1, 'D5'), (15, .5, 'F5'), (15.5, .5, 'G5'),
        (16, 1.5, 'Bb5'), (17.5, .5, 'G5'), (18, .5, 'E5'), (18.5, .5, 'Db5'), (19, 1, 'C5'),
    ]
    play_line([x for x in sax2 if s_stamp + x[0] < s_boss], s_stamp, 'sax', 0.64)
    shout = [(0, .5, 'A5'), (.5, 1, 'C6'), (1.5, .5, 'A5'), (2, .5, 'Ab5'), (2.5, 1.5, 'F5'),
             (4, .5, 'Bb5'), (4.5, .5, 'A5'), (5, .5, 'G5'), (5.5, .5, 'Bb5')]
    hit(s_boss, 'F7', 'A5', v=1.0, length=0.45, crash_v=0.28)
    harmonize([x for x in shout[1:] if s_boss + x[0] < s_map], chords, s_boss, v=0.9)
    play_line([x for x in shout if s_boss + x[0] < s_map], s_boss, 'sax', 0.55)
    comp(chords, s_boss, s_map, instrument='organ', pattern_set=[[(0, 1.8), (2, 1.8)]])
    drums_shuffle(s_boss, s_map, v=1.25)
    for beat in range(s_boss, s_map):
        cym_t.add(at(beat) + human(4), ride(0.22, bell=True))
    fill(s_map - 1, s_map, 0.4)
    climb = [(0, .5, 'G4'), (.5, .5, 'Bb4'), (1, .5, 'D5'), (1.5, .5, 'F5'), (2, .5, 'E5'), (2.5, .5, 'G5'), (3, .5, 'Bb5'), (3.5, .5, 'Db6'),
             (4, .5, 'C6'), (4.5, .5, 'Db6'), (5, .5, 'C6'), (5.5, .5, 'Db6', {'v': 1.2})]
    play_line([x for x in climb if s_map + x[0] < s_end], s_map, 'sax', 0.62)
    comp(chords, s_map, s_end - 2, instrument='organ', pattern_set=[[(0, 1.9), (2, 1.9)]])
    drums_shuffle(s_map, s_end - 2, v=1.1)
    for i, note in enumerate(['E5', 'Db5', 'Bb4', 'G4']):
        horns_t.add(at(s_end - 2), brass(m(note), 2 * BEAT, 0.5 + 0.1 * i))
    fill(s_end - 2, s_end, 0.45)
    hit(s_end, 'F9', 'A5', v=1.0, length=1.6, crash_v=0.32)
    lick = [(2, .5, 'Ab5', {'scoop': 60}), (2.5, .5, 'F5'), (3, .5, 'Eb5'), (3.5, .5, 'C5'), (4, 1, 'D5'), (5, .5, 'Eb5'), (5.5, .5, 'E5')]
    play_line(lick, s_end, 'sax', 0.6)
    walk(chords, s_end + 2, s_end + 6)
    comp(chords, s_end + 2, s_end + 6, instrument='organ', pattern_set=[[(0, 1.9), (2, 1.9)]])
    drums_shuffle(s_end + 2, s_end + 6, v=0.9)
    final = s_end + 6
    hit(final, 'F13', 'A5', v=1.0, length=TOTAL - final - 0.2, crash_v=0.3)
    organ_t.add(at(final), sum(organ(n, (TOTAL - final) * BEAT, 0.55) for n in rootless('F13')))
    play_line([(0, TOTAL - final - 0.3, 'F5', {'scoop': 60})], final, 'sax', 0.55)
    cut_hits(skip=(s_stamp, s_boss, s_end))


score_a() if VARIANT == 'jazz-a' else score_b()

# ------------------------------------------------------------------- mix


def room_ir(seconds=0.75, seed=0):
    r = np.random.default_rng(seed)
    n = int(seconds * SR)
    t = np.arange(n) / SR
    ir = r.standard_normal(n) * np.exp(-t * 6.9 / seconds)
    ir = np.convolve(ir, np.ones(5) / 5, mode='same')  # darker tail
    ir[: int(0.009 * SR)] = 0
    for d, g in ((0.011, 0.5), (0.017, 0.35), (0.023, 0.25)):  # early reflections
        ir[int(d * SR)] += g * r.choice([-1, 1]) * 4
    return ir / np.sqrt(np.sum(ir ** 2))


def convolve(x, ir):
    size = 1 << int(np.ceil(np.log2(len(x) + len(ir))))
    return np.fft.irfft(np.fft.rfft(x, size) * np.fft.rfft(ir, size), size)[: len(x)]


def tone(x, low_cut=None, high_cut=None):
    spec = np.fft.rfft(x)
    f = np.fft.rfftfreq(len(x), 1 / SR)
    if high_cut:
        spec *= 1 / np.sqrt(1 + (f / high_cut) ** 4)
    if low_cut:
        spec *= 1 - 1 / np.sqrt(1 + (f / low_cut) ** 4)
    return np.fft.irfft(spec, len(x))


tracks = {'bass': bass_t, 'piano': piano_t, 'organ': organ_t, 'lead': lead_t, 'horns': horns_t, 'ride': ride_t, 'kit': kit_t, 'cym': cym_t}
# Balance (measured with --levels): the lead on top, the bass and the ride clear, the comping behind.
LEVELS = {
    'jazz-a': {'bass': 0.75, 'piano': 0.32, 'organ': 0.5, 'lead': 2.6, 'horns': 1.4, 'ride': 2.2, 'kit': 0.85, 'cym': 0.55},
    'jazz-b': {'bass': 0.85, 'piano': 0.25, 'organ': 0.5, 'lead': 2.4, 'horns': 1.4, 'ride': 3.5, 'kit': 0.8, 'cym': 0.55},
}[VARIANT]
bass_t.buf = tone(bass_t.buf, low_cut=35, high_cut=2500)
ride_t.buf = tone(ride_t.buf, low_cut=2500)
cym_t.buf = tone(cym_t.buf, low_cut=1800)
lead_t.buf = tone(lead_t.buf, low_cut=150, high_cut=7000)
horns_t.buf = tone(horns_t.buf, low_cut=120, high_cut=6500)
for name, tr in tracks.items():
    tr.buf *= LEVELS[name]
if '--levels' in sys.argv:
    for name, tr in tracks.items():
        print(f'  {name:6s} rms {20 * np.log10(np.sqrt(np.mean(tr.buf ** 2)) + 1e-9):6.1f} dB  peak {20 * np.log10(np.max(np.abs(tr.buf)) + 1e-9):6.1f} dB')

irl, irr = room_ir(0.75, 1), room_ir(0.75, 2)
left = np.zeros(N)
right = np.zeros(N)
send = np.zeros(N)
for tr in tracks.values():
    gl = np.cos((tr.pan + 1) * np.pi / 4) * 1.414
    gr = np.sin((tr.pan + 1) * np.pi / 4) * 1.414
    left += tr.buf * gl
    right += tr.buf * gr
    send += tr.buf * tr.room
left += convolve(send, irl) * 0.9
right += convolve(send, irr) * 0.9
mix = np.stack([left, right])
# Glue: a gentle bus compression (RMS follower), a soft clip on the peaks only.
env = np.sqrt(np.convolve(np.mean(mix ** 2, axis=0), np.ones(2048) / 2048, mode='same')) + 1e-6
ref = np.percentile(env, 90)
gain = np.minimum(1, (env / ref) ** -0.35)
mix *= gain
mix /= np.max(np.abs(mix))
mix = np.tanh(mix * 1.3) / np.tanh(1.3)
fade = int(0.3 * SR)
mix[:, -fade:] *= np.linspace(1, 0, fade) ** 1.5
mix *= 0.89 / np.max(np.abs(mix))

pcm = (np.clip(mix.T, -1, 1) * 32767).astype('<i2')
with wave.open(OUT, 'wb') as w:
    w.setnchannels(2)
    w.setsampwidth(2)
    w.setframerate(SR)
    w.writeframes(pcm.tobytes())
print(f'[music] {OUT}: {VARIANT}, {G["bpm"]} BPM, {TOTAL} beats, {N / SR:.2f} s')
