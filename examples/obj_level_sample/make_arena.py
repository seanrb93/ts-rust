"""Make a small arena as an OBJ (with its pictures) to try tools/obj2level.py on.

    python examples/obj_level_sample/make_arena.py [out folder]
    python tools/obj2level.py <out folder>/arena.obj --out mods/Arena --number 33 --name "Arena" --like 25

The arena is 60 x 60 units with walls 8 high, a platform in the middle with a ramp up to it, four pillars and a pane of glass.  The pictures are
drawn here (nothing of the game's is used).  The `spawn_a` object marks where a player starts, `weapon1`, `weapon3` and `weapon5` objects
mark where weapons lie.
"""
import os
import random
import sys

from PIL import Image, ImageDraw

out = sys.argv[1] if len(sys.argv) > 1 else os.path.join(os.path.dirname(os.path.abspath(__file__)), 'arena')
os.makedirs(out, exist_ok=True)
random.seed(7)


def noisy(img, amount):
    px = img.load()
    for y in range(img.size[1]):
        for x in range(img.size[0]):
            r, g, b, a = px[x, y]
            n = random.randint(-amount, amount)
            px[x, y] = (max(0, min(255, r + n)), max(0, min(255, g + n)), max(0, min(255, b + n)), a)
    return img


# floor: grey tiles with joints
floor = Image.new('RGBA', (128, 128), (120, 120, 126, 255))
d = ImageDraw.Draw(floor)
for i in range(0, 128, 32):
    d.line([(i, 0), (i, 127)], fill=(70, 70, 76, 255), width=2)
    d.line([(0, i), (127, i)], fill=(70, 70, 76, 255), width=2)
noisy(floor, 10).save(os.path.join(out, 'floor.png'))
# wall: red brick
wall = Image.new('RGBA', (128, 128), (110, 50, 40, 255))
d = ImageDraw.Draw(wall)
for row in range(0, 128, 16):
    d.line([(0, row), (127, row)], fill=(190, 180, 170, 255), width=2)
    off = 0 if (row // 16) % 2 == 0 else 16
    for x in range(off, 128, 32):
        d.line([(x, row), (x, row + 15)], fill=(190, 180, 170, 255), width=2)
noisy(wall, 14).save(os.path.join(out, 'wall.png'))
# platform: orange and black stripes
plat = Image.new('RGBA', (64, 64), (230, 140, 20, 255))
d = ImageDraw.Draw(plat)
for i in range(-64, 64, 16):
    d.polygon([(i, 0), (i + 8, 0), (i + 72, 64), (i + 64, 64)], fill=(30, 30, 30, 255))
plat.save(os.path.join(out, 'platform.png'))
# glass: pale blue, mostly clear, with a frame
glass = Image.new('RGBA', (64, 64), (150, 200, 255, 70))
d = ImageDraw.Draw(glass)
d.rectangle([0, 0, 63, 63], outline=(230, 240, 255, 230), width=3)
glass.save(os.path.join(out, 'glass.png'))

V, T, F = [], [], []


def vtx(p):
    V.append(p)
    return len(V)


def tex(uv):
    T.append(uv)
    return len(T)


def quad(a, b, c, d, mat, rep=1.0, reph=None):
    """A quad a, b, c, d going round counter-clockwise as seen from the front; the picture repeats `rep` times across (and `reph` up)."""
    reph = rep if reph is None else reph
    uvs = ((0, 0), (rep, 0), (rep, reph), (0, reph))
    F.append((mat, [(vtx(p), tex(uv)) for p, uv in zip((a, b, c, d), uvs)]))


S, H = 30, 8
for i in range(-3, 3):
    for j in range(-3, 3):
        x0, x1, z0, z1 = i * 10, i * 10 + 10, j * 10, j * 10 + 10
        quad((x0, 0, z1), (x1, 0, z1), (x1, 0, z0), (x0, 0, z0), 'floor')
quad((-S, 0, -S), (S, 0, -S), (S, H, -S), (-S, H, -S), 'wall', 6, 2)
quad((S, 0, S), (-S, 0, S), (-S, H, S), (S, H, S), 'wall', 6, 2)
quad((-S, 0, S), (-S, 0, -S), (-S, H, -S), (-S, H, S), 'wall', 6, 2)
quad((S, 0, -S), (S, 0, S), (S, H, S), (S, H, -S), 'wall', 6, 2)
quad((S, H, S), (-S, H, S), (-S, H, -S), (S, H, -S), 'wall', 6)
# the platform (3 high) and its sides
quad((-5, 3, 5), (5, 3, 5), (5, 3, -5), (-5, 3, -5), 'platform', 2)
quad((-5, 0, 5), (5, 0, 5), (5, 3, 5), (-5, 3, 5), 'platform', 2, 1)
quad((5, 0, -5), (-5, 0, -5), (-5, 3, -5), (5, 3, -5), 'platform', 2, 1)
quad((-5, 0, -5), (-5, 0, 5), (-5, 3, 5), (-5, 3, -5), 'platform', 2, 1)
# the ramp up to the platform from the +x side
quad((5, 3, 3), (15, 0, 3), (15, 0, -3), (5, 3, -3), 'platform', 3, 1)
# (the ramp's sides are triangles: a quad that repeats a corner is read as the triangle)
quad((15, 0, -3), (5, 0, -3), (5, 3, -3), (5, 3, -3), 'platform', 1)
quad((5, 0, 3), (15, 0, 3), (5, 3, 3), (5, 3, 3), 'platform', 1)
# four pillars 2 x 2 and 6 high
for px, pz in ((-18, -18), (18, -18), (-18, 18), (18, 18)):
    x0, x1, z0, z1 = px - 1, px + 1, pz - 1, pz + 1
    quad((x0, 0, z1), (x1, 0, z1), (x1, 6, z1), (x0, 6, z1), 'wall', 1, 3)
    quad((x1, 0, z1), (x1, 0, z0), (x1, 6, z0), (x1, 6, z1), 'wall', 1, 3)
    quad((x1, 0, z0), (x0, 0, z0), (x0, 6, z0), (x1, 6, z0), 'wall', 1, 3)
    quad((x0, 0, z0), (x0, 0, z1), (x0, 6, z1), (x0, 6, z0), 'wall', 1, 3)
    quad((x0, 6, z1), (x1, 6, z1), (x1, 6, z0), (x0, 6, z0), 'wall', 1, 1)
# a pane of glass, drawn from both sides (translucent: drawn after the solid surfaces)
quad((-12, 0, -10), (-4, 0, -10), (-4, 5, -10), (-12, 5, -10), 'glass')
quad((-4, 0, -10), (-12, 0, -10), (-12, 5, -10), (-4, 5, -10), 'glass')


def marker(name, x, z):
    a = [vtx((x, 0, z)), vtx((x + 0.5, 0, z)), vtx((x, 0, z + 0.5))]
    return name, a


markers = [marker('spawn_a', 12, 14), marker('spawn_b', -20, 20), marker('spawn_c', 22, -22), marker('spawn_d', -22, -5),
           marker('weapon1', 0, 12), marker('weapon3', 0, -12), marker('weapon5', 0, -17)]
with open(os.path.join(out, 'arena.obj'), 'w') as f:
    f.write('mtllib arena.mtl\n')
    for p in V:
        f.write('v %g %g %g\n' % p)
    for t in T:
        f.write('vt %g %g\n' % t)
    cur = None
    for m, ids in F:
        if m != cur:
            f.write('usemtl %s\n' % m)
            cur = m
        f.write('f ' + ' '.join('%d/%d' % i for i in ids) + '\n')
    for name, ids in markers:
        f.write('o %s\nf %s\n' % (name, ' '.join(str(i) for i in ids)))
with open(os.path.join(out, 'arena.mtl'), 'w') as f:
    for m in ('floor', 'wall', 'platform', 'glass'):
        f.write('newmtl %s\nmap_Kd %s.png\n' % (m, m))
print('wrote', os.path.join(out, 'arena.obj'))
