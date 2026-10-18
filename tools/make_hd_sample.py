"""Make the HD texture sample mod from the game's own Tomb textures (they are not in the repository: run tools/extract_textures.py first).

    python tools/make_hd_sample.py <extracted textures folder> <mods folder>

Writes <mods folder>/hd_sample/MISC.PAK/textures/<number>.png: a few wall textures of the Tomb, 16 times as big in each direction, smoothed and
given a fine grain, a carved line along the edges and a teal tint, so that the difference to the originals is easy to see.  The mod is only for trying
the mod loader's pictures of any size; the game's art stays out of the repository (the mod is written next to the game).
"""
import os
import random
import sys

from PIL import Image, ImageDraw, ImageFilter

NUMBERS = [1977, 1978, 1979, 1980, 1985, 1986, 1987, 1988, 2001, 2002]
SCALE = 16


def main():
    if len(sys.argv) != 3:
        print(__doc__)
        sys.exit(1)
    src, mods = sys.argv[1:]
    out = os.path.join(mods, 'hd_sample', 'MISC.PAK', 'textures')
    os.makedirs(out, exist_ok=True)
    folder = next(os.path.join(src, 'levels', d) for d in os.listdir(os.path.join(src, 'levels')) if d.endswith('Tomb'))
    rnd = random.Random(7)
    for n in NUMBERS:
        path = os.path.join(folder, f'{n}.png')
        if not os.path.isfile(path):
            continue
        im = Image.open(path).convert('RGBA')
        w, h = im.size
        im = im.resize((w * SCALE, h * SCALE), Image.LANCZOS).filter(ImageFilter.UnsharpMask(8, 120, 2))
        px = im.load()
        for y in range(im.height):
            for x in range(im.width):
                r, g, b, a = px[x, y]
                k = rnd.randint(-14, 14)
                px[x, y] = (max(0, min(255, int(r * 0.35) + k)), max(0, min(255, int(g * 1.3) + 50 + k)), max(0, min(255, int(b * 1.5) + 80 + k)), a)
        d = ImageDraw.Draw(im)
        line = max(4, SCALE // 2)
        d.rectangle([line, line, im.width - 1 - line, im.height - 1 - line], outline=(255, 215, 90, 255), width=line)
        im.save(os.path.join(out, f'{n}.png'))
        print(n, im.size)
    open(os.path.join(mods, 'hd_sample', 'mod.ini'), 'w').write('enabled = true\n')


if __name__ == '__main__':
    main()
