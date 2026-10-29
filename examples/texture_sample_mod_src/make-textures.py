"""Makes the pictures of the texture sample mod, all drawn from nothing and much bigger than the game's own:

    python examples/texture_sample_mod_src/make-nebula.py <mod folder>/MISC.PAK/textures [size]

misc/back1, misc/back4 and misc/clouds (behind the menus): a 2048 x 2048 nebula of teal and magenta clouds that tiles (the game's own are 256 x 256);
1181 and 1183 (the floor tiles of the Prison level, 64 x 64 in the game): 2048 x 2048 metal plates, the second with the red mark the game's has;
1180 and 1182 (the walls of the Prison, 128 x 64): 2048 x 1024 brushed metal with panel seams, the second with the red line.
"""
import os
import sys

import numpy as np
from PIL import Image

root = sys.argv[1]
out = os.path.join(root, 'misc')
size = int(sys.argv[2]) if len(sys.argv) > 2 else 2048
rng = np.random.default_rng(11)


def noise(power):
    """Periodic noise whose strength falls with the frequency to `power` (so the picture repeats at its edges)."""
    f = np.fft.fftfreq(size)
    k = np.sqrt(f[:, None] ** 2 + f[None, :] ** 2)
    k[0, 0] = 1
    spec = (rng.normal(size=(size, size)) + 1j * rng.normal(size=(size, size))) / k ** power
    spec[0, 0] = 0
    n = np.real(np.fft.ifft2(spec))
    return (n - n.min()) / (n.max() - n.min())


a, b, c = noise(1.8), noise(1.4), noise(0.6)
t = np.clip(a * 1.3 - 0.15, 0, 1)
# two colours that blend through the clouds, bright where the second noise is high
teal = np.array([0.0, 0.85, 0.9])
magenta = np.array([0.9, 0.15, 0.7])
mix = b[..., None]
img = (teal * (1 - mix) + magenta * mix) * (0.3 + 0.9 * t[..., None] ** 1.2)
img += 0.22 * (c[..., None] - 0.5) * t[..., None]
# stars: a few small bright points
stars = rng.random((size, size)) > 0.9993
img += stars[..., None] * rng.random((size, size, 1))
img = np.clip(img, 0, 1)
os.makedirs(out, exist_ok=True)
for name in ('back1', 'back4', 'clouds'):
    Image.fromarray((img * 255).astype(np.uint8)).save(os.path.join(out, name + '.png'))
print('back1.png, back4.png and clouds.png', size, 'x', size)

# ---- the Prison's floor and walls ------------------------------------------------------------------------------------------------------------
from PIL import ImageDraw, ImageFilter


def floor(dot):
    im = Image.new('RGB', (size, size), (24, 30, 44))
    grain = (rng.normal(size=(size, size)) * 6).astype(np.int16)
    arr = np.array(im).astype(np.int16) + grain[..., None]
    im = Image.fromarray(np.clip(arr, 0, 255).astype(np.uint8))
    d = ImageDraw.Draw(im)
    u = size // 8
    for i in range(1, 8):
        d.line([(i * u, 0), (i * u, size)], fill=(40, 52, 74), width=max(2, size // 512))
        d.line([(0, i * u), (size, i * u)], fill=(40, 52, 74), width=max(2, size // 512))
    b = max(6, size // 64)
    d.rectangle([0, 0, size - 1, size - 1], outline=(0, 200, 190), width=b)
    d.rectangle([b + 4, b + 4, size - b - 5, size - b - 5], outline=(0, 90, 100), width=max(2, b // 3))
    r = max(4, size // 128)
    for x in (b * 3, size - b * 3):
        for y in (b * 3, size - b * 3):
            d.ellipse([x - r * 2, y - r * 2, x + r * 2, y + r * 2], fill=(90, 100, 120))
    if dot:
        c = size // 2
        d.ellipse([c - size // 40, c - size // 40, c + size // 40, c + size // 40], fill=(255, 40, 30))
        im = im.filter(ImageFilter.GaussianBlur(1))
    return im


def wall(line):
    w, h = size, size // 2
    streak = rng.normal(size=(h, w))
    streak = np.cumsum(streak, axis=1)
    streak = (streak - streak.mean(axis=1, keepdims=True)) / (streak.std(axis=1, keepdims=True) + 1e-6)
    base = 105 + streak * 9 + rng.normal(size=(h, w)) * 3
    arr = np.stack([base * 0.82, base * 0.95, base * 1.05], axis=-1)
    im = Image.fromarray(np.clip(arr, 0, 255).astype(np.uint8))
    d = ImageDraw.Draw(im)
    for x in range(0, w + 1, w // 4):
        d.line([(x, 0), (x, h)], fill=(30, 40, 50), width=max(3, w // 400))
    d.line([(0, h - 1), (w, h - 1)], fill=(30, 40, 50), width=max(3, w // 400))
    for x in range(w // 8, w, w // 4):
        for y in (h // 6, h - h // 6):
            d.ellipse([x - 10, y - 10, x + 10, y + 10], fill=(60, 70, 80))
    if line:
        y = int(h * 0.38)
        d.line([(0, y), (w, y)], fill=(255, 60, 50), width=max(3, w // 500))
    return im


for n, dot in ((1181, False), (1183, True)):
    floor(dot).save(os.path.join(root, f'{n}.png'))
for n, line in ((1180, False), (1182, True)):
    wall(line).save(os.path.join(root, f'{n}.png'))
print('1180 to 1183 written')
