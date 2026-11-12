"""Soundtrack of the App Store preview: about 28.6 s at 130 BPM, A minor.

    python3 store/make-music.py <out.wav>

Composed for this project and synthesized here from sine partials and
seeded noise (numpy only), like web/scripts/make-sounds.mjs: no recording,
no third-party music, no licence question. Deterministic.

Form (beats; one beat = 60/130 s; the video cuts on the same grid, see
assemble.py):
  0-8    intro: accordion stabs, kick, bass enters, snare build
  8-16   groove: four on the floor, the theme on accordion and chip lead
  16-20  build into the stamp (F, G)
  20-40  the theme again, an octave up from 28, the special cards
  40-46  boss: D minor and E, toms, half time
  46-52  build: snare roll, riser, a breath before the end
  52-62  end card: the big A minor, a last phrase, the final hit at 60
Chord progression i-VI-III-VII (Am F C G) with E as the dominant.
"""

import sys
import wave

import numpy as np

SR = 44100
BPM = 130
BEAT = 60 / BPM
TOTAL_BEATS = 62
LENGTH = TOTAL_BEATS * BEAT + 0.05
N = int(LENGTH * SR)

rng = np.random.default_rng(1813)  # Skat since 1813


def t_of(beat):
    return beat * BEAT


def midi_hz(m):
    return 440.0 * 2 ** ((m - 69) / 12)


NOTE = {'C': 0, 'C#': 1, 'Db': 1, 'D': 2, 'D#': 3, 'Eb': 3, 'E': 4, 'F': 5, 'F#': 6, 'Gb': 6, 'G': 7, 'G#': 8, 'Ab': 8, 'A': 9, 'A#': 10, 'Bb': 10, 'B': 11}


def m(name):
    """'A4' -> 69."""
    return NOTE[name[:-1]] + 12 * (int(name[-1]) + 1)


class Track:
    def __init__(self, pan=0.0):
        self.buf = np.zeros(N)
        self.pan = pan

    def add(self, start, sig):
        i = int(round(start * SR))
        if i >= N:
            return
        j = min(N, i + len(sig))
        self.buf[i:j] += sig[: j - i]

    def stereo(self):
        left = np.cos((self.pan + 1) * np.pi / 4)
        right = np.sin((self.pan + 1) * np.pi / 4)
        return np.stack([self.buf * left * 1.414, self.buf * right * 1.414])


def env(n, a=0.005, d=0.1, s=0.7, r=0.08, hold=None):
    """ADSR over n samples; `hold` seconds before the release (default: all but r)."""
    t = np.arange(n) / SR
    dur = n / SR
    hold = dur - r if hold is None else hold
    e = np.where(t < a, t / max(a, 1e-6), s + (1 - s) * np.exp(-(t - a) / max(d, 1e-6)))
    rel = np.clip((t - hold) / max(r, 1e-6), 0, 1)
    return e * (1 - rel) ** 2


def additive(freq, dur, amps, detune=0.0, vib=None, bright_decay=0.0, phase0=0.0):
    """Sum of harmonics with amplitude list `amps` (index 0 = fundamental)."""
    n = int(dur * SR)
    t = np.arange(n) / SR
    f = freq * 2 ** (detune / 1200)
    if vib:
        rate, cents, delay = vib
        depth = np.clip((t - delay) / 0.15, 0, 1) * cents
        inst = f * 2 ** (depth * np.sin(2 * np.pi * rate * t) / 1200)
        phase = 2 * np.pi * np.cumsum(inst) / SR + phase0
    else:
        phase = 2 * np.pi * f * t + phase0
    out = np.zeros(n)
    for k, a in enumerate(amps, start=1):
        if a == 0 or f * k > 18000:
            continue
        g = a * (np.exp(-t * bright_decay * (k - 1)) if bright_decay else 1)
        out += g * np.sin(k * phase)
    return out


def saw_amps(h=40):
    return [1 / k for k in range(1, h + 1)]


def square_amps(h=40):
    return [1 / k if k % 2 else 0 for k in range(1, h + 1)]


def pulse_amps(duty=0.25, h=40):
    return [abs(np.sin(np.pi * k * duty)) / k for k in range(1, h + 1)]


def reed_amps(h=24):
    """Accordion-ish reed: strong low partials, a nasal bump around 3-5."""
    base = [1.0, 0.7, 0.75, 0.55, 0.5, 0.3, 0.25, 0.2]
    return base + [0.18 / (k - 7) for k in range(9, h + 1)]


# ------------------------------------------------------------- instruments

def accordion(freq, dur, gain=0.2):
    """Two reeds detuned (the musette beat) plus one an octave down, soft attack."""
    sig = additive(freq, dur, reed_amps(), detune=-7, vib=(5.5, 6, 0.25))
    sig += additive(freq, dur, reed_amps(), detune=+8, vib=(5.2, 6, 0.25), phase0=1.3)
    sig += 0.45 * additive(freq / 2, dur, reed_amps(16), detune=2)
    return gain * sig * env(len(sig), a=0.012, d=0.12, s=0.8, r=0.05)


def chip(freq, dur, gain=0.08):
    sig = additive(freq, dur, pulse_amps(0.25, 30), vib=(6, 12, 0.18))
    return gain * sig * env(len(sig), a=0.003, d=0.08, s=0.6, r=0.04)


def bass(freq, dur, gain=0.32):
    sig = additive(freq, dur, saw_amps(30), bright_decay=7)
    sig += 0.6 * additive(freq / 2, dur, [1, 0, 0.1])
    return gain * sig * env(len(sig), a=0.004, d=0.12, s=0.55, r=0.03)


def pad(freq, dur, gain=0.05):
    sig = np.zeros(int(dur * SR))
    for i, c in enumerate((-14, -6, 0, 7, 15)):
        sig += additive(freq, dur, saw_amps(14), detune=c, phase0=i * 0.9)
    return gain * sig * env(len(sig), a=0.25, d=0.4, s=0.85, r=0.35)


def stab(freqs, dur=0.16, gain=0.1):
    sig = sum(additive(f, dur, reed_amps(18), detune=d) for f in freqs for d in (-6, 6))
    return gain * sig * env(len(sig), a=0.002, d=0.06, s=0.4, r=0.05)


def kick(gain=0.9):
    n = int(0.42 * SR)
    t = np.arange(n) / SR
    f = 45 + 110 * np.exp(-t * 32)
    ph = 2 * np.pi * np.cumsum(f) / SR
    body = np.sin(ph) * np.exp(-t * 7.5)
    click = rng.standard_normal(n) * np.exp(-t * 380) * 0.4
    return gain * np.tanh(1.6 * (body + click))


def snare(gain=0.42, tone=190):
    n = int(0.3 * SR)
    t = np.arange(n) / SR
    noise = rng.standard_normal(n)
    noise = np.diff(np.concatenate([[0], noise]))  # tilt up: brighter
    body = np.sin(2 * np.pi * tone * t) * np.exp(-t * 25)
    return gain * (0.55 * noise * np.exp(-t * 17) + 0.6 * body)


def hat(open_=False, gain=0.1):
    n = int((0.28 if open_ else 0.05) * SR)
    t = np.arange(n) / SR
    noise = rng.standard_normal(n)
    noise = noise - np.convolve(noise, np.ones(4) / 4, mode='same')
    return gain * noise * np.exp(-t * (9 if open_ else 70))


def crash(gain=0.22, length=2.2):
    n = int(length * SR)
    t = np.arange(n) / SR
    noise = rng.standard_normal(n)
    noise = noise - np.convolve(noise, np.ones(3) / 3, mode='same')
    return gain * noise * np.exp(-t * 2.2) * (0.6 + 0.4 * np.exp(-t * 20))


def tom(freq, gain=0.5):
    n = int(0.45 * SR)
    t = np.arange(n) / SR
    f = freq * (1 + 0.6 * np.exp(-t * 25))
    ph = 2 * np.pi * np.cumsum(f) / SR
    return gain * np.sin(ph) * np.exp(-t * 7) + 0.08 * rng.standard_normal(n) * np.exp(-t * 40)


def boom(gain=0.8):
    """Sub drop for the big hits."""
    n = int(1.6 * SR)
    t = np.arange(n) / SR
    f = 32 + 60 * np.exp(-t * 6)
    return gain * np.sin(2 * np.pi * np.cumsum(f) / SR) * np.exp(-t * 2.5)


def riser(dur, gain=0.18):
    n = int(dur * SR)
    t = np.arange(n) / SR
    x = t / dur
    noise = rng.standard_normal(n)
    # Brighter as it rises: mix of the noise and its first difference.
    bright = np.diff(np.concatenate([[0], noise]))
    sweep = np.sin(2 * np.pi * np.cumsum(300 + 2200 * x ** 2) / SR)
    return gain * (x ** 2) * ((1 - x) * noise * 0.5 + x * bright * 0.5 + 0.35 * sweep)


# ---------------------------------------------------------------- harmony

CHORDS = [  # (start beat, root, quality)
    (0, 'A', 'm'), (4, 'F', ''), (8, 'C', ''), (12, 'G', ''),
    (16, 'F', ''), (18, 'G', ''), (20, 'A', 'm'), (24, 'F', ''), (28, 'C', ''), (32, 'G', ''),
    (36, 'A', 'm'), (40, 'D', 'm'), (44, 'E', ''), (46, 'F', ''), (48, 'G', ''), (50, 'E', ''),
    (52, 'A', 'm'), (56, 'F', ''), (58, 'G', ''), (60, 'A', 'm'), (TOTAL_BEATS, 'A', 'm'),
]


def chord_tones(root, quality, octave):
    r = NOTE[root] + 12 * (octave + 1)
    return [r, r + (3 if quality == 'm' else 4), r + 7]


def chord_at(beat):
    cur = CHORDS[0]
    for c in CHORDS:
        if c[0] <= beat:
            cur = c
    return cur


# ----------------------------------------------------------------- melody

# The theme: (beat within its 4-beat bar, length in beats, note).
THEME = {
    'Am': [(0, .5, 'A4'), (.5, .5, 'C5'), (1, .75, 'E5'), (1.75, .25, 'D5'), (2, .5, 'C5'), (2.5, .5, 'B4'), (3, .5, 'A4'), (3.5, .5, 'E5')],
    'F': [(0, .75, 'F5'), (.75, .25, 'E5'), (1, .5, 'C5'), (1.5, .5, 'A4'), (2, 1, 'C5'), (3, .5, 'D5'), (3.5, .5, 'E5')],
    'C': [(0, .75, 'G5'), (.75, .25, 'E5'), (1, .5, 'C5'), (1.5, .5, 'E5'), (2, .5, 'G5'), (2.5, .5, 'A5'), (3, .5, 'G5'), (3.5, .5, 'E5')],
    'G': [(0, .5, 'D5'), (.5, .5, 'B4'), (1, .5, 'G4'), (1.5, .5, 'B4'), (2, 1.5, 'D5'), (3.5, .5, 'B4')],
    'Dm': [(0, .5, 'D5'), (.5, .5, 'F5'), (1, .5, 'A5'), (1.5, .5, 'F5'), (2, .5, 'D5'), (2.5, .5, 'F5'), (3, 1, 'A5')],
    'E': [(0, .5, 'G#4'), (.5, .5, 'B4'), (1, .5, 'E5'), (1.5, .5, 'G#5')],
    'FG': [(0, .5, 'C5'), (.5, .5, 'F5'), (1, .5, 'A5'), (1.5, .5, 'C6'), (2, .5, 'B5'), (2.5, .5, 'G5'), (3, .5, 'D5'), (3.5, .5, 'B4')],
    'Ebuild': [(0, .25, 'E5'), (.25, .25, 'G#5'), (.5, .25, 'B5'), (.75, .25, 'E6'), (1, .25, 'D6'), (1.25, .25, 'B5'), (1.5, .25, 'G#5'), (1.75, .25, 'F5')],
    'End': [(0, 2, 'A5'), (2, .5, 'G5'), (2.5, .5, 'E5'), (3, 1, 'C5'), (4, 1, 'F5'), (5, 1, 'E5'), (6, .5, 'D5'), (6.5, .5, 'B4'), (7, 1, 'G#4'), (8, 2, 'A4')],
}

MELODY_PLAN = [  # (start beat, theme key, octave shift)
    (8, 'C', 0), (12, 'G', 0), (16, 'FG', 0), (20, 'Am', 0), (24, 'F', 0),
    (28, 'C', 12), (32, 'G', 12), (36, 'Am', 12), (40, 'Dm', 0), (44, 'E', 0), (46, 'FG', 0), (50, 'Ebuild', 0), (52, 'End', 12),
]


# ------------------------------------------------------------------- build

lead = Track(0.1)
leadchip = Track(-0.25)
acc = Track(-0.15)
bas = Track(0.0)
pads = Track(0.0)
drums = Track(0.0)
hats = Track(0.3)
fx = Track(0.0)
cym = Track(-0.2)

for start, key, shift in MELODY_PLAN:
    for b, length, name in THEME[key]:
        beat = start + b
        if beat >= TOTAL_BEATS:
            continue
        dur = length * BEAT * 0.95
        f = midi_hz(m(name) + shift)
        lead.add(t_of(beat), accordion(f, dur + 0.04, gain=0.16 if shift else 0.18))
        leadchip.add(t_of(beat), chip(f * (1 if shift else 2), dur, gain=0.05))

# Pads from the groove on, with the chord of each span; silent in the breath before the end.
for i, (start, root, q) in enumerate(CHORDS[:-1]):
    end = CHORDS[i + 1][0]
    if start < 4:
        continue
    span = (end - start) * BEAT + 0.3
    for tone in chord_tones(root, q, 3):
        pads.add(t_of(start), pad(midi_hz(tone), span, gain=0.032 if start < 20 else 0.04))

# Bass: eighths with the octave from beat 4; half time in the boss part; long notes at the end.
for b8 in range(8, TOTAL_BEATS * 2):
    beat = b8 / 2
    _, root, q = chord_at(beat)
    r = NOTE[root] + 12 * 2  # octave 1
    if r < m('E1'):
        r += 12
    if 51.5 <= beat < 52:
        continue
    if beat >= 52:
        if beat in (52, 56, 58, 60):
            length = {52: 4, 56: 2, 58: 2, 60: 2}[beat]
            bas.add(t_of(beat), bass(midi_hz(r), length * BEAT, gain=0.36))
        continue
    if 40 <= beat < 46 and b8 % 2:
        continue
    note = r + (12 if b8 % 2 else 0)
    bas.add(t_of(beat), bass(midi_hz(note), BEAT * 0.48))

# Accordion stabs: intro on every beat, the groove on the off-beats (polka push).
for b8 in range(0, 104):
    beat = b8 / 2
    _, root, q = chord_at(beat)
    tones = [midi_hz(x) for x in chord_tones(root, q, 4)]
    if beat < 8:
        if b8 % 2 == 0:
            acc.add(t_of(beat), stab(tones, 0.2, 0.07 + 0.01 * beat))
    elif beat < 40 or 46 <= beat < 51.5:
        if b8 % 2:
            acc.add(t_of(beat), stab(tones, 0.14, 0.055))

# Drums.
for b16 in range(0, TOTAL_BEATS * 4):
    beat = b16 / 4
    q = b16 % 4
    if beat < 4:
        if q == 0 and int(beat) % 2 == 0:
            drums.add(t_of(beat), kick(0.8))
    elif beat < 8:
        if q == 0:
            drums.add(t_of(beat), kick(0.85))
        if beat >= 6 and (q % 2 == 0 or beat >= 7):
            drums.add(t_of(beat), snare(0.12 + 0.2 * (beat - 6) / 2))
    elif beat < 40 or 46 <= beat < 50:
        if q == 0:
            drums.add(t_of(beat), kick())
            if int(beat) % 2 == 1:
                drums.add(t_of(beat), snare())
        if q == 2:
            hats.add(t_of(beat), hat(open_=True, gain=0.06))
        elif q == 0:
            hats.add(t_of(beat), hat(gain=0.08))
        if q == 3 and beat >= 20:
            hats.add(t_of(beat), hat(gain=0.04))
        # Fills before the big moments.
        if 19 <= beat < 20 or 35 <= beat < 36:
            drums.add(t_of(beat), tom(150 - 30 * q, 0.35))
    elif beat < 46:
        # Boss: half time, toms.
        if q == 0 and int(beat) % 2 == 0:
            drums.add(t_of(beat), kick(1.0))
        if q == 0 and int(beat) % 4 == 2:
            drums.add(t_of(beat), snare(0.5, tone=160))
        if q in (2, 3) and int(beat) % 2 == 1:
            drums.add(t_of(beat), tom(95 if q == 2 else 80, 0.42))
        if q == 0:
            hats.add(t_of(beat), hat(gain=0.06))
    elif beat < 51.5:
        # Build: kick in eighths, the snare roll speeds up and rises.
        if q % 2 == 0:
            drums.add(t_of(beat), kick(0.8))
        x = (beat - 50) / 1.5
        drums.add(t_of(beat), snare(0.15 + 0.3 * x, tone=180 + 120 * x))
        drums.add(t_of(beat + 0.125), snare(0.12 + 0.3 * x, tone=200 + 120 * x))
    elif beat >= 52 and beat < 60:
        # End card: half time, big.
        if q == 0 and int(beat) % 2 == 0:
            drums.add(t_of(beat), kick(1.0))
        if q == 0 and int(beat) % 4 == 2:
            drums.add(t_of(beat), snare(0.55))
        if q == 2:
            hats.add(t_of(beat), hat(gain=0.05))
        if 59 <= beat < 60:
            drums.add(t_of(beat), tom(160 - 25 * q, 0.42))

# Snare build at the end of the groove halves.
for b in (15.5, 15.75, 31.5, 31.75):
    drums.add(t_of(b), snare(0.3))

# Big hits: the stamp (20), the boss (40), the end card (52) and the last chord (60).
for b, g in ((0, 0.16), (8, 0.14), (20, 0.26), (36, 0.16), (40, 0.24), (52, 0.3), (60, 0.32)):
    cym.add(t_of(b), crash(g))
for b in (0, 20, 40, 52, 60):
    drums.add(t_of(b), boom(0.55 if b != 60 else 0.7))
    drums.add(t_of(b), kick(1.0))
fx.add(t_of(46), riser(5.5 * BEAT, gain=0.16))

# The final chord: all in on beat 60, ringing out.
for tone in chord_tones('A', 'm', 4) + [m('A3'), m('E5')]:
    lead.add(t_of(60), accordion(midi_hz(tone), 2.2 * BEAT, gain=0.08))

# ------------------------------------------------------------------- mix


def lowpass_fft(x, cutoff, order=2):
    spec = np.fft.rfft(x)
    f = np.fft.rfftfreq(len(x), 1 / SR)
    spec *= 1 / np.sqrt(1 + (f / cutoff) ** (2 * order))
    return np.fft.irfft(spec, len(x))


def highpass_fft(x, cutoff, order=2):
    spec = np.fft.rfft(x)
    f = np.fft.rfftfreq(len(x), 1 / SR)
    spec *= 1 - 1 / np.sqrt(1 + (f / cutoff) ** (2 * order))
    return np.fft.irfft(spec, len(x))


def reverb(x, seconds=1.4, mix=0.18):
    n = int(seconds * SR)
    t = np.arange(n) / SR
    ir = rng.standard_normal(n) * np.exp(-t * 6.9 / seconds)
    ir = lowpass_fft(ir, 5000)
    ir[: int(0.012 * SR)] = 0
    ir /= np.sqrt(np.sum(ir ** 2))
    size = 1 << int(np.ceil(np.log2(len(x) + n)))
    wet = np.fft.irfft(np.fft.rfft(x, size) * np.fft.rfft(ir, size), size)[: len(x)]
    return x + mix * wet


pads.buf = lowpass_fft(pads.buf, 2600)
bas.buf = lowpass_fft(bas.buf, 1400)
acc.buf = lowpass_fft(acc.buf, 5200)
lead.buf = reverb(lowpass_fft(lead.buf, 6500), 1.3, 0.22)
leadchip.buf = reverb(lowpass_fft(leadchip.buf, 7000), 1.0, 0.25)
hats.buf = highpass_fft(hats.buf, 6000)
cym.buf = highpass_fft(cym.buf, 3500)
drums.buf = reverb(drums.buf, 0.9, 0.08)
pads.buf = reverb(pads.buf, 2.0, 0.3)

# Sidechain feel: duck pads and bass a little on each kick of the groove.
duck = np.ones(N)
for b in range(8, 52):
    if 40 <= b < 46 and b % 2:
        continue
    i = int(t_of(b) * SR)
    k = np.arange(min(int(0.22 * SR), N - i))
    duck[i:i + len(k)] = np.minimum(duck[i:i + len(k)], 1 - 0.45 * np.exp(-k / SR * 14))
pads.buf *= duck
bas.buf *= 0.75 + 0.25 * duck

tracks = {'lead': lead, 'chip': leadchip, 'acc': acc, 'bass': bas, 'pads': pads, 'drums': drums, 'hats': hats, 'fx': fx, 'cym': cym}
if '--levels' in sys.argv:
    for name, tr in tracks.items():
        print(f'  {name:6s} rms {20 * np.log10(np.sqrt(np.mean(tr.buf ** 2)) + 1e-9):6.1f} dB  peak {20 * np.log10(np.max(np.abs(tr.buf)) + 1e-9):6.1f} dB')
LEVELS = {'drums': 0.6, 'hats': 3.0, 'chip': 2.2, 'acc': 1.3, 'cym': 1.2}
for name, g in LEVELS.items():
    tracks[name].buf *= g
mix = sum(tr.stereo() for tr in tracks.values())
# Glue: a gentle soft clip on the peaks only, fade the last 0.25 s.
mix /= np.max(np.abs(mix))
mix = np.tanh(mix * 1.4) / np.tanh(1.4)
fade = int(0.25 * SR)
mix[:, -fade:] *= np.linspace(1, 0, fade)
mix /= np.max(np.abs(mix)) / 0.89

out = next((a for a in sys.argv[1:] if not a.startswith('--')), 'music.wav')
pcm = (np.clip(mix.T, -1, 1) * 32767).astype('<i2')
with wave.open(out, 'wb') as w:
    w.setnchannels(2)
    w.setsampwidth(2)
    w.setframerate(SR)
    w.writeframes(pcm.tobytes())
print(f'[music] {out}: {N / SR:.2f} s, {TOTAL_BEATS} beats at {BPM} BPM')
