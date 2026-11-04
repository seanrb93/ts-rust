"""Make the HD font mod from the game's own font textures (they are not in the repository: run tools/extract_textures.py first).

    python tools/make-hd-font.py <extracted textures folder> <mods folder> [scale]

Writes <mods folder>/hd_font/MISC.PAK/textures/misc/<font>.png for the six fonts of the game (bankfont, font07, font09, itckabelfont, newsfont,
squarefont), each as many times bigger as the scale says (8 by default), and a mod.ini.  The game's art stays out of the repository: the mod is written
next to the game, and this script is what is kept.

A glyph of the game's fonts is a few pixels of an alpha mask (white, with the alpha as the shape, 0 to 128 in the console's steps).  Each atlas is made
bigger by Lanczos resampling of the alpha (and of the colour, weighted by the alpha, so that the edges have no halo), and the edge is then smoothed and sharpened (an unsharp mask) so
that the edge is a clean curve at the new size, not a blur and not the stairs of the original, and a thin faint stroke keeps its weight.  The picture is drawn over the same surface by the same coordinates, so nothing in the game changes: only the glyphs are sharper.
"""
import os
import sys

import numpy as np
from PIL import Image

FONTS = ['bankfont', 'font07', 'font09', 'itckabelfont', 'newsfont', 'squarefont']
# the sharpening of the resampled alpha: a smoothing (in pixels of the original) that takes out the ringing of the resampling and the steps of the original's alpha, then an unsharp mask
# of a width (in pixels of the original) and a strength; a thin, faint stroke keeps its weight where a threshold would take it away
SMOOTH = 0.42
SHARP_WIDTH = 0.75
SHARP_AMOUNT = 2.6


def resize(channel, scale):
    """One channel (a float array) at `scale` times the size by Lanczos."""
    h, w = channel.shape
    return np.asarray(Image.fromarray(channel.astype(np.float32), mode='F').resize((w * scale, h * scale), Image.LANCZOS), dtype=np.float32)


def blur(channel, sigma):
    """A Gaussian blur of a float array (separable, by shifts), `sigma` in pixels."""
    radius = max(1, int(sigma * 3.0))
    taps = np.exp(-0.5 * (np.arange(-radius, radius + 1) / sigma) ** 2)
    taps /= taps.sum()
    out = channel
    for axis in (0, 1):
        padded = np.pad(out, [(radius, radius) if a == axis else (0, 0) for a in (0, 1)], mode='edge')
        acc = np.zeros_like(out)
        for k, w in enumerate(taps):
            acc += w * (padded[k:k + out.shape[0]] if axis == 0 else padded[:, k:k + out.shape[1]])
        out = acc
    return out


def power_of_two(n):
    """The next power of two (the game keeps a texture in a surface of such a size: a 128 by 108 font is on 128 by 128, and the picture that replaces it is stretched over the whole surface)."""
    size = 1
    while size < n:
        size *= 2
    return size


def make(src_path, scale):
    image = np.asarray(Image.open(src_path).convert('RGBA'), dtype=np.float32) / 255.0
    alpha = image[..., 3]
    # (the colours weighted by the alpha, so that a pixel with no alpha does not pull its colour into the edge)
    weighted = [resize(image[..., c] * alpha, scale) for c in range(3)]
    alpha_up = np.clip(resize(alpha, scale), 0.0, 1.0)
    smooth = blur(alpha_up, SMOOTH * scale)
    shape = np.clip(smooth + SHARP_AMOUNT * (smooth - blur(smooth, SHARP_WIDTH * scale)), 0.0, 1.0)
    out = np.zeros(alpha_up.shape + (4,), dtype=np.float32)
    covered = alpha_up > 1.0e-3
    for c in range(3):
        out[..., c] = np.where(covered, np.clip(weighted[c] / np.maximum(alpha_up, 1.0e-3), 0.0, 1.0), 1.0)
    out[..., 3] = shape
    picture = Image.fromarray((out * 255.0 + 0.5).astype(np.uint8), 'RGBA')
    # (on the surface of the texture: the top left of it, the rest empty)
    surface = Image.new('RGBA', (power_of_two(image.shape[1]) * scale, power_of_two(image.shape[0]) * scale), (255, 255, 255, 0))
    surface.paste(picture, (0, 0))
    return surface


def main():
    if len(sys.argv) not in (3, 4):
        print(__doc__)
        sys.exit(1)
    src, mods = sys.argv[1:3]
    scale = int(sys.argv[3]) if len(sys.argv) == 4 else 8
    out = os.path.join(mods, 'hd_font', 'MISC.PAK', 'textures', 'misc')
    os.makedirs(out, exist_ok=True)
    for font in FONTS:
        path = os.path.join(src, 'misc', 'fonts', font + '.png')
        picture = make(path, scale)
        picture.save(os.path.join(out, font + '.png'), optimize=True)
        print(f'{font}: {picture.width}x{picture.height}')
    with open(os.path.join(mods, 'hd_font', 'mod.ini'), 'w', encoding='utf-8') as f:
        f.write('name = HD Font\nenabled = true\n')


if __name__ == '__main__':
    main()
