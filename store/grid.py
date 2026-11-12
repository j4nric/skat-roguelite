"""The edit grid of the preview video, shared by make-music.py and assemble.py.

Each scene has a target length in seconds; for a tempo the cuts snap to
whole beats (an even number per scene, at least two), so every cut falls
on the music's beat. The score scene lands its stamp (Spielwert 342) on a
beat of its own: the stamp is where the music hits.

    python3 store/grid.py jazz-a      # print the grid of a variant
"""

import sys

# Tempo of each soundtrack variant (quarter notes per minute).
VARIANTS = {
    'jazz-a': 160,  # uptempo swing
    'jazz-b': 120,  # slower blues shuffle
}

# (scene, target seconds). The order is the order of the video.
SCENES = [
    ('sign', 1.85),
    ('trick', 1.85),
    ('quip', 2.3),
    ('ansage', 1.4),
    ('score', 3.7),
    ('undead', 1.85),
    ('doppel', 1.85),
    ('curse', 1.85),
    ('tresen', 1.85),
    ('boss', 2.77),
    ('map', 2.77),
    ('end', 4.3),
]
# Seconds of the score scene before the stamp lands.
STAMP_AFTER = 2.1


def beats_for(seconds, bpm, step=2):
    beat = 60 / bpm
    return max(step, int(round(seconds / beat / step)) * step)


def grid(variant):
    """{'bpm', 'beat', 'scenes': {name: (from beat, to beat)}, 'order', 'stamp', 'total'}"""
    bpm = VARIANTS[variant]
    at = 0
    scenes = {}
    for name, seconds in SCENES:
        n = beats_for(seconds, bpm)
        scenes[name] = (at, at + n)
        at += n
    s0, s1 = scenes['score']
    stamp = s0 + min(beats_for(STAMP_AFTER, bpm), s1 - s0 - 2)
    return {'bpm': bpm, 'beat': 60 / bpm, 'scenes': scenes, 'order': [n for n, _ in SCENES], 'stamp': stamp, 'total': at}


if __name__ == '__main__':
    g = grid(sys.argv[1] if len(sys.argv) > 1 else 'jazz-a')
    print(f"{g['bpm']} BPM, {g['total']} beats = {g['total'] * g['beat']:.2f} s, stamp on beat {g['stamp']}")
    for name in g['order']:
        a, b = g['scenes'][name]
        print(f'  {name:7s} {a:3d}-{b:3d}  {a * g["beat"]:5.2f}-{b * g["beat"]:5.2f} s')
