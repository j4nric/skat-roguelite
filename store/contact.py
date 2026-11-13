"""Contact sheet: one picture per `step` seconds of a video or a frame folder.

    python3 store/contact.py <video.mp4|clip-dir> <out.png> [step=1.0] [cols=10]
"""
import json
import subprocess
import sys
import tempfile
from pathlib import Path

from PIL import Image, ImageDraw

src, out = sys.argv[1], sys.argv[2]
step = float(sys.argv[3]) if len(sys.argv) > 3 else 1.0
cols = int(sys.argv[4]) if len(sys.argv) > 4 else 10
W, H = 221, 480

if Path(src).is_dir():
    clip = json.loads((Path(src) / 'clip.json').read_text())
    n = len(clip['frames'])
    fps = clip.get('fps', 30)
    pics = [(i / fps, Image.open(Path(src) / clip['frames'][i])) for i in range(0, n, max(1, round(step * fps)))]
else:
    tmp = Path(tempfile.mkdtemp())
    subprocess.run(['ffmpeg', '-v', 'error', '-i', src, '-vf', f'fps=1/{step}', str(tmp / 'c%04d.png')], check=True)
    files = sorted(tmp.glob('c*.png'))
    pics = [(i * step, Image.open(f)) for i, f in enumerate(files)]

rows = (len(pics) + cols - 1) // cols
sheet = Image.new('RGB', (cols * W, rows * (H + 18)), '#111')
d = ImageDraw.Draw(sheet)
for k, (t, im) in enumerate(pics):
    x, y = (k % cols) * W, (k // cols) * (H + 18)
    sheet.paste(im.convert('RGB').resize((W, H)), (x, y + 18))
    d.text((x + 4, y + 3), f'{t:.1f}s', fill='#fff')
sheet.save(out)
