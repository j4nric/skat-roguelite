"""Cut the recorded clips to the beat, lay the lines over them, mix the game
sounds onto the soundtrack and encode the App Store preview.

    python3 store/assemble.py <de|en> [jazz-a|jazz-b]

Reads store/build/<lang>/<clip>/clip.json (from make-preview.mjs), the
lines in store/build/<lang>/text/ and store/build/music-<variant>.wav.
Writes store/preview/preview-<lang>.mp4 and poster-<lang>.png (jazz-a, the
chosen soundtrack) or preview-<lang>-<variant>.mp4 for another variant.

The cuts come from store/grid.py: each scene starts on a beat of the
soundtrack. A scene shows one or more pieces of clips; a piece starts at a
frame (or a mark the recorder noted, plus frames) and covers a span of
source frames in its time (faster or slower within bounds). A scene may
zoom (a crop that eases from one scale to another around a centre,
normalized to the frame) and carry a line.
"""

import json
import subprocess
import sys
import wave
from pathlib import Path

import numpy as np
from PIL import Image

sys.path.insert(0, str(Path(__file__).resolve().parent))
from grid import grid  # noqa: E402

LANG = sys.argv[1]
VARIANT = sys.argv[2] if len(sys.argv) > 2 else 'jazz-a'
GRID = grid(VARIANT)
# The chosen soundtrack (jazz-a, 13.11.2026) writes preview-<lang>.mp4; another one carries its name.
SUFFIX = '' if VARIANT == 'jazz-a' else f'-{VARIANT}'
STORE = Path(__file__).resolve().parent
BUILD = STORE / 'build' / LANG
OUT = STORE / 'preview'
SOUNDS = STORE.parent / 'web' / 'src' / 'assets' / 'sounds'
W, H, FPS = 886, 1920, 30
SR = 44100


def frame_of(beat):
    return int(round(beat * GRID['beat'] * FPS))


# Per scene: pieces, zoom, line, line position, line until (fraction of the scene).
# A piece: (clip, start, span, beats or None for the rest). `start` is a frame
# number or 'mark+n' / 'mark-n'; `span` the source frames to cover (None:
# real time; 'stamp': up to the stamp; 'hold': that one frame), played at
# span / frames, bounded.
# Zoom: (scale from, scale to, centre x, centre y, ease) with ease 'inout' or 'out'.
SCENES = {
    'sign': dict(pieces=[('sign', 0, None, None)], zoom=(1.36, 1.26, 0.5, 0.2, 'inout'), line='real', pos=0.78),
    'trick': dict(pieces=[('trick', 'play-2', 44, None)], zoom=(1.18, 1.1, 0.5, 0.42, 'inout'), line='nights', pos=0.075),
    # The hello stands only about half a second at the table: the bubble's best frame is held, so it can be read.
    'quip': dict(pieces=[('factor', 'quip+3', 12, 2), ('factor', 'quip+17', 'hold', None)], zoom=[(1.05, 2.0, 0.33, 0.2, 'out'), (2.0, 2.3, 0.33, 0.2, 'inout')]),
    'ansage': dict(pieces=[('factor', 'hand-1', 52, None)], zoom=(1.15, 1.05, 0.5, 0.5, 'inout')),
    'score': dict(pieces=[('factor', 'score-8', 'stamp', 'stamp'), ('factor', 'stamp+0', 50, None)], zoom=[(1.0, 1.0, 0.5, 0.5, 'inout'), (1.0, 1.15, 0.5, 0.42, 'out')], line='rules', pos=0.87),
    'undead': dict(pieces=[('undead', 'go+3', 63, None)], zoom=(1.4, 1.3, 0.5, 0.36, 'inout'), line='undead', pos=0.065),
    'doppel': dict(pieces=[('doppel', 'play-3', 36, None)], zoom=(1.12, 1.05, 0.5, 0.5, 'inout'), line='doppel', pos=0.065),
    'curse': dict(pieces=[('curse', 14, 83, None)], zoom=(1.7, 1.55, 0.5, 0.33, 'inout'), line='cursed', pos=0.86),
    'tresen': dict(pieces=[('tresen', 'relic-3', 55, None)], zoom=(1.0, 1.06, 0.5, 0.3, 'inout'), line='tresen', pos=0.84),
    'boss': dict(pieces=[('boss', 'sheet+1', 83, None)], zoom=(1.0, 1.5, 0.5, 0.7, 'inout'), line='bosses', pos=0.12, until=0.67),
    'map': dict(pieces=[('map', 'pick-3', 75, None)], zoom=(1.12, 1.3, 0.5, 0.3, 'inout'), line='city', pos=0.86),
    'end': dict(pieces=[('end', 5, None, None)], zoom=None),
}
TIMELINE = [dict(SCENES[name], name=name, beats=GRID['scenes'][name]) for name in GRID['order']]
SPEED = (0.6, 2.2)
POSTER = ('factor', 'stamp+5')

# Level of each game sound under the music (they play quieter than the music).
SFX_GAIN = {
    'card': 0.55, 'trick': 0.5, 'tap': 0.25, 'quip': 0.8, 'bang': 0.8, 'slam': 0.6, 'declare': 0.7, 'clink': 0.5,
    'tick': 0.35, 'knock': 0.7, 'coin': 0.7, 'till': 0.6, 'boss': 0.9, 'chalk': 0.6, 'prize': 0.6, 'win': 0.5,
    'shuffle': 0.35, 'unlock': 0.4, 'kontra': 0.5, 'pickup': 0.4, 'discard': 0.4, 'gulp': 0.5, 'pour': 0.5,
    'dice': 0.5, 'snip': 0.5, 'schwarz': 0.5, 'lose': 0.0, 'night': 0.0,
}
SFX_MASTER = 0.5

clips = {}


def clip(name):
    if name not in clips:
        clips[name] = json.loads((BUILD / name / 'clip.json').read_text())
    return clips[name]


def start_frame(c, start):
    if isinstance(start, int):
        return start
    for op in ('+', '-'):
        if op in start:
            mark, n = start.split(op)
            return c['marks'][mark] + (int(n) if op == '+' else -int(n))
    return c['marks'][start]


def ease(x, kind):
    x = min(max(x, 0.0), 1.0)
    if kind == 'out':
        return 1 - (1 - x) ** 3
    return x * x * (3 - 2 * x)


def crop_box(scale, cx, cy, w, h):
    cw, ch = w / scale, h / scale
    x0 = min(max(cx * w - cw / 2, 0), w - cw)
    y0 = min(max(cy * h - ch / 2, 0), h - ch)
    return (x0, y0, x0 + cw, y0 + ch)


# Ordered dither (8 × 8 Bayer), in blocks of 6 px: the lines come and go in pixel steps.
BAYER = np.array([[0, 32, 8, 40, 2, 34, 10, 42], [48, 16, 56, 24, 50, 18, 58, 26], [12, 44, 4, 36, 14, 46, 6, 38], [60, 28, 52, 20, 62, 30, 54, 22],
                  [3, 35, 11, 43, 1, 33, 9, 41], [51, 19, 59, 27, 49, 17, 57, 25], [15, 47, 7, 39, 13, 45, 5, 37], [63, 31, 55, 23, 61, 29, 53, 21]]) / 64


def dither_mask(w, h, level, block=6):
    ys = (np.arange(h) // block) % 8
    xs = (np.arange(w) // block) % 8
    return (BAYER[ys][:, xs] < level).astype(np.uint8) * 255


lines = {}


def line_image(key):
    if key not in lines:
        lines[key] = Image.open(BUILD / 'text' / f'line-{key}.png').convert('RGBA')
    return lines[key]


def lay_line(img, key, pos, k, n):
    """The line `key` on `img` (RGBA), centred, top at pos × H; frame k of n of its time."""
    li = line_image(key)
    steps_in, steps_out = 4, 3
    level = 1.0
    if k < steps_in:
        level = (k + 1) / (steps_in + 1)
    elif k >= n - steps_out:
        level = (n - k) / (steps_out + 1)
    if level <= 0:
        return img
    alpha = np.array(li.split()[3])
    if level < 1:
        alpha = np.minimum(alpha, dither_mask(li.width, li.height, level))
    li = li.copy()
    li.putalpha(Image.fromarray(alpha))
    x = (W - li.width) // 2 + 11  # the PNG carries its shadow on the right and bottom
    y = int(pos * H)
    img.alpha_composite(li, (x, y))
    return img


def build():
    total = frame_of(TIMELINE[-1]['beats'][1])
    frames = []  # (clip, source frame index, zoom spec, t in segment, line spec)
    sfx = []  # (name, time s)
    for seg in TIMELINE:
        b0, b1 = seg['beats']
        f0, f1 = frame_of(b0), frame_of(b1)
        pieces = seg['pieces']
        zooms = seg.get('zoom')
        if not isinstance(zooms, list):
            zooms = [zooms] * len(pieces)
        at = f0
        pb = b0
        for i, (name, start, span, beats) in enumerate(pieces):
            pe = b1 if beats is None else (GRID['stamp'] if beats == 'stamp' else pb + beats)
            pf0, pf1 = frame_of(pb), frame_of(pe)
            c = clip(name)
            s0 = start_frame(c, start)
            n = pf1 - pf0
            if span == 'stamp':
                span = c['marks']['stamp'] - s0
            speed = 0.0 if span == 'hold' else 1.0 if span is None else min(max(span / n, SPEED[0]), SPEED[1])
            for k in range(n):
                src = min(int(s0 + k * speed), len(c['frames']) - 1)
                frames.append((name, src, zooms[i], k / max(n - 1, 1), seg, at - f0 + k, f1 - f0))
            # Sounds the app made inside this piece.
            for s in c['sounds'] if speed else []:
                rel = (s['t'] * FPS - s0) / speed
                if 0 <= rel < n:
                    sfx.append((s['name'], (pf0 + rel) / FPS))
            at += n
            pb = pe
    assert len(frames) == total, (len(frames), total)
    return frames, sfx, total


def render(frames, total):
    OUT.mkdir(parents=True, exist_ok=True)
    video = BUILD / 'video.mp4'
    enc = subprocess.Popen(
        ['ffmpeg', '-v', 'error', '-y', '-f', 'rawvideo', '-pix_fmt', 'rgb24', '-s', f'{W}x{H}', '-r', str(FPS), '-i', '-',
         '-c:v', 'libx264', '-profile:v', 'high', '-level', '4.0', '-preset', 'slow', '-crf', '15', '-maxrate', '14M', '-bufsize', '28M',
         '-pix_fmt', 'yuv420p', '-r', str(FPS), '-movflags', '+faststart', str(video)],
        stdin=subprocess.PIPE)
    cache = {}
    poster_clip, poster_at = POSTER
    poster_src = start_frame(clip(poster_clip), poster_at)
    poster = None
    for idx, (name, src, zoom, x, seg, k, n) in enumerate(frames):
        c = clip(name)
        key = (name, src)
        if key not in cache:
            cache.clear()
            cache[key] = Image.open(BUILD / name / c['frames'][src]).convert('RGB')
        img = cache[key]
        if img.size != (W, H):
            img = img.resize((W, H), Image.LANCZOS)
        if zoom:
            z0, z1, cx, cy, kind = zoom
            z = z0 + (z1 - z0) * ease(x, kind)
            if abs(z - 1) > 1e-3:
                img = img.resize((W, H), Image.LANCZOS, box=crop_box(z, cx, cy, W, H))
        if seg.get('line'):
            end = int(round(n * seg.get('until', 1.0)))
            if k < end:
                img = lay_line(img.convert('RGBA'), seg['line'], seg['pos'], k, end).convert('RGB')
        if name == poster_clip and src == poster_src and poster is None:
            poster = img.copy()
        enc.stdin.write(img.tobytes())
    enc.stdin.close()
    if enc.wait() != 0:
        raise SystemExit('ffmpeg (video) failed')
    if poster is not None:
        poster.save(OUT / f'poster-{LANG}{SUFFIX}.png')
    return video


def read_wav(path):
    with wave.open(str(path)) as w:
        data = np.frombuffer(w.readframes(w.getnframes()), dtype='<i2').astype(np.float64) / 32768
        return data.reshape(-1, w.getnchannels()).T


def decode(name):
    raw = subprocess.run(['ffmpeg', '-v', 'error', '-i', str(SOUNDS / f'{name}.mp3'), '-f', 's16le', '-ac', '2', '-ar', str(SR), '-'], capture_output=True, check=True).stdout
    return np.frombuffer(raw, dtype='<i2').astype(np.float64).reshape(-1, 2).T / 32768


def mix_audio(sfx, total):
    music = read_wav(STORE / 'build' / f'music-{VARIANT}.wav')
    length = int(total / FPS * SR)
    out = np.zeros((2, length))
    out[:, : min(length, music.shape[1])] = music[:, :length]
    effects = np.zeros((2, length))
    cache = {}
    for name, t in sfx:
        gain = SFX_GAIN.get(name, 0.4)
        if gain <= 0:
            continue
        if name not in cache:
            cache[name] = decode(name)
        s = cache[name]
        i = int(t * SR)
        j = min(length, i + s.shape[1])
        if i < length:
            effects[:, i:j] += gain * s[:, : j - i]
    out += SFX_MASTER * effects
    out /= max(1.0, np.max(np.abs(out)) / 0.95)
    wav = BUILD / 'audio.wav'
    with wave.open(str(wav), 'wb') as w:
        w.setnchannels(2)
        w.setsampwidth(2)
        w.setframerate(SR)
        w.writeframes((np.clip(out.T, -1, 1) * 32767).astype('<i2').tobytes())
    # Two-pass loudness normalization to -16 LUFS, true peak -1.5 dBTP.
    probe = subprocess.run(['ffmpeg', '-hide_banner', '-i', str(wav), '-af', 'loudnorm=I=-16:TP=-1.5:LRA=11:print_format=json', '-f', 'null', '-'], capture_output=True, text=True).stderr
    stats = json.loads(probe[probe.rindex('{'):probe.rindex('}') + 1])
    norm = BUILD / 'audio-norm.wav'
    subprocess.run(['ffmpeg', '-v', 'error', '-y', '-i', str(wav), '-af',
                    f"loudnorm=I=-16:TP=-1.5:LRA=11:measured_I={stats['input_i']}:measured_TP={stats['input_tp']}:measured_LRA={stats['input_lra']}:measured_thresh={stats['input_thresh']}:offset={stats['target_offset']}:linear=true",
                    '-ar', str(SR), str(norm)], check=True)
    return norm


def main():
    frames, sfx, total = build()
    print(f'[assemble] {LANG} {VARIANT}: {total} frames ({total / FPS:.2f} s), {len(sfx)} game sounds')
    video = render(frames, total)
    audio = mix_audio(sfx, total)
    out = OUT / f'preview-{LANG}{SUFFIX}.mp4'
    subprocess.run(['ffmpeg', '-v', 'error', '-y', '-i', str(video), '-i', str(audio), '-map', '0:v', '-map', '1:a', '-c:v', 'copy',
                    '-c:a', 'aac', '-b:a', '256k', '-ar', str(SR), '-ac', '2', '-shortest', '-movflags', '+faststart', str(out)], check=True)
    (BUILD / 'sfx.json').write_text(json.dumps(sfx, indent=1))
    print(f'[assemble] {out}')


main()
