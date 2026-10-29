"""Make a mod that shows the textures of a level larger and smoother (the pictures are the game's own, so the mod is written next to the game, not into the repository).

    python tools/upscale_textures.py <extracted textures folder> <mods folder> [--level Tomb] [--scale 8]

The textures are first extracted with `python tools/extract_textures.py <PAK folder> <folder>`.  The mod gets `<mods folder>/<level>_upscale/MISC.PAK/textures/<number>.png`, one picture
for each numbered texture of the level, `scale` times as large in each direction (8 makes the usual 128 x 64 texture 1024 x 512).  The game still loads its own small texture; the
renderer shows the PNG over the same surface (docs/reference.md, "Textures").

How a picture is made: the game's textures have 256 colours with a dither, so a speck of one colour among its neighbours is a pattern, not detail.  The picture is smoothed a little
(a median of 3 x 3 on a double-size copy) to take the dither out, enlarged with the Lanczos filter, and sharpened so that edges stay clear.  This is an enlargement, not new detail: the
brick joints and cracks are as sharp as the original had them, only without the blocks of pixels.  The colours are unchanged and the transparency is kept.
"""
import os
import sys

from PIL import Image, ImageFilter


def upscale(path, scale):
    im = Image.open(path).convert('RGBA')
    w, h = im.size
    rgb, alpha = im.convert('RGB'), im.getchannel('A')
    # (a double size copy first, so the median cleans the dither without taking the detail away)
    mid = rgb.resize((w * 2, h * 2), Image.LANCZOS).filter(ImageFilter.MedianFilter(3))
    big = mid.resize((w * scale, h * scale), Image.LANCZOS).filter(ImageFilter.UnsharpMask(radius=scale * 0.8, percent=90, threshold=2))
    out = big.convert('RGBA')
    out.putalpha(alpha.resize((w * scale, h * scale), Image.BILINEAR))
    return out


def main():
    args = sys.argv[1:]
    if len(args) < 2:
        print(__doc__)
        sys.exit(1)
    src, mods = args[0], args[1]
    level = args[args.index('--level') + 1] if '--level' in args else 'Tomb'
    scale = int(args[args.index('--scale') + 1]) if '--scale' in args else 8
    levels = os.path.join(src, 'levels')
    folders = [d for d in os.listdir(levels) if d.lower().endswith(level.lower())]
    if not folders:
        sys.exit(f'no level folder ending in {level} in {levels}')
    folder = os.path.join(levels, folders[0])
    out = os.path.join(mods, f'{level.lower()}_upscale', 'MISC.PAK', 'textures')
    os.makedirs(out, exist_ok=True)
    n = 0
    for name in sorted(os.listdir(folder)):
        stem, ext = os.path.splitext(name)
        if ext.lower() != '.png' or not stem.isdigit():
            continue
        upscale(os.path.join(folder, name), scale).save(os.path.join(out, name))
        n += 1
    with open(os.path.join(mods, f'{level.lower()}_upscale', 'mod.ini'), 'w') as f:
        f.write('enabled = true\n')
    print(f'{n} textures of {folders[0]} at {scale}x written to {out}')


if __name__ == '__main__':
    main()
