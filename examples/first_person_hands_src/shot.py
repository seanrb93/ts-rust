"""Take first person screenshots of weapons with the mod on, for looking at the hands.

    python shot.py <game folder> <out folder> <combo> [combo ...]      the combos: 1 pistol, 3 M16, 30 shotgun (`GUNCOMBOS`)
"""
import os
import subprocess
import sys

from PIL import Image

game, out = sys.argv[1], sys.argv[2]
os.makedirs(out, exist_ok=True)
for combo in sys.argv[3:]:
    png = os.path.join(out, f'fp_{combo}.png')
    env = dict(os.environ, TS_GIVE=f'100:{combo}', TS_EQUIP='1')
    subprocess.run([os.path.join(game, 'ts-rust.exe'), '--level', 'Prison', '--players', '1', '--screenshot', '260', png, '--frames', '270', '--headless'], cwd=game, env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=200)
    im = Image.open(png).convert('RGB')
    w, h = im.size
    # the lower right part, where the weapon is
    im.crop((w // 2, h // 3, w, h)).save(os.path.join(out, f'fp_{combo}_crop.png'))
print('ok')
