"""Makes a new character from scratch: Pumpkin Jack, a scarecrow with a pumpkin head, as an OBJ (a pose of arms down, facing +z) with its pictures.

    python examples/pumpkin_mod_src/make-pumpkin.py <out folder> [head detail]

The figure is made of solid pieces, each in an object `bone<N>` (see tools/obchar.py) that says which bone of the game's skeleton moves it, with a ball over each joint, so that it
bends at the joints without tearing.
"""
import math
import os
import sys

import numpy as np
from PIL import Image, ImageDraw

out = sys.argv[1]
os.makedirs(out, exist_ok=True)
HEAD = float(sys.argv[2]) if len(sys.argv) > 2 else 1.0
verts, uvs, norms, groups = [], [], [], {}

# the joints of the game's skeleton (the right side; the left is the mirror)
SHOULDER, ELBOW, WRIST, HAND_END = (0.23, 1.47, 0.0), (0.23, 1.19, -0.05), (0.23, 0.91, 0.0), (0.23, 0.72, 0.03)
HIP, KNEE, ANKLE, TOE = (0.1, 0.99, 0.02), (0.1, 0.59, 0.17), (0.1, 0.14, 0.02), (0.1, -0.02, -0.04)
# part numbers of the bones (right, left)
UPPER_ARM, FOREARM, HAND = (8, 16), (10, 18), (12, 20)
THIGH, SHIN, FOOT = (24, 30), (26, 32), (28, 34)


def mirror(p, s):
    return (p[0] * s, p[1], p[2])


def flat(name, rgb):
    Image.new('RGB', (8, 8), rgb).save(os.path.join(out, name + '.png'))


def add(bone, material, pts, tris, centre, uv=None, nrm=None):
    base, base_n = len(verts), len(norms)
    for i, p in enumerate(pts):
        verts.append(tuple(float(x) for x in p))
        uvs.append(uv[i] if uv else (0.5, 0.5))
    if nrm:
        norms.extend(nrm)
    for a, b, c in tris:
        pa, pb, pc = (np.array(pts[i], dtype=float) for i in (a, b, c))
        n = np.cross(pb - pa, pc - pa)
        if np.linalg.norm(n) < 1e-12:
            continue
        if np.dot(n, (pa + pb + pc) / 3 - np.array(centre)) < 0:
            b, c = c, b
        groups.setdefault('bone%d' % bone, []).append((material, [(base + i + 1, (base_n + i + 1) if nrm else None) for i in (a, b, c)]))


def ball(bone, material, c, r, n=16, m=10, face=False):
    """An ellipsoid (radius r, a number or three), smooth; with `face` the front is the middle of the picture (the head)."""
    r = (r, r, r) if isinstance(r, (int, float)) else r
    pts, uv, tris, nrm = [], [], [], []
    for j in range(m + 1):
        th = math.pi * j / m
        for i in range(n + 1):
            ph = 2 * math.pi * i / n
            p = (c[0] + r[0] * math.sin(th) * math.sin(ph), c[1] + r[1] * math.cos(th), c[2] + r[2] * math.sin(th) * math.cos(ph))
            pts.append(p)
            v = ((p[0] - c[0]) / r[0] ** 2, (p[1] - c[1]) / r[1] ** 2, (p[2] - c[2]) / r[2] ** 2)
            nrm.append(tuple(np.array(v) / np.linalg.norm(v)))
            uv.append((0.5 + 0.5 * math.sin(th) * math.sin(ph), 0.5 - 0.5 * math.cos(th)) if face else (i / n, 1 - j / m))
    for j in range(m):
        for i in range(n):
            a = j * (n + 1) + i
            b = a + n + 1
            tris += [(a, a + 1, b), (a + 1, b + 1, b)]
    add(bone, material, pts, tris, c, uv, nrm)


def prism(bone, material, a, b, ra, rb=None, depth=1.0):
    """A square-sectioned piece from the point a to the point b (half-width ra at a and rb at b, `depth` times as thick front to back)."""
    rb = ra if rb is None else rb
    a, b = np.array(a, dtype=float), np.array(b, dtype=float)
    d = (b - a) / np.linalg.norm(b - a)
    side = np.cross(d, (0.0, 0.0, 1.0))
    if np.linalg.norm(side) < 1e-6:
        side = np.array((1.0, 0.0, 0.0))
    side /= np.linalg.norm(side)
    front = np.cross(side, d)
    corners = []
    for p, r in ((a, ra), (b, rb)):
        for sx, sz in ((-1, -1), (1, -1), (1, 1), (-1, 1)):
            corners.append(p + side * sx * r + front * sz * r * depth)
    for q in [(0, 1, 2, 3), (4, 5, 6, 7), (0, 1, 5, 4), (1, 2, 6, 5), (2, 3, 7, 6), (3, 0, 4, 7)]:
        add(bone, material, [corners[i] for i in q], [(0, 1, 2), (0, 2, 3)], (a + b) / 2, [(0, 0), (1, 0), (1, 1), (0, 1)])


def box(bone, material, lo, hi):
    lo, hi = np.array(lo, dtype=float), np.array(hi, dtype=float)
    prism(bone, material, ((lo[0] + hi[0]) / 2, lo[1], (lo[2] + hi[2]) / 2), ((lo[0] + hi[0]) / 2, hi[1], (lo[2] + hi[2]) / 2), (hi[0] - lo[0]) / 2, depth=(hi[2] - lo[2]) / (hi[0] - lo[0]))


def tube(bone, material, c, r, h, n=10):
    pts, tris, nrm = [], [], []
    for y in (c[1], c[1] + h):
        for i in range(n):
            a = 2 * math.pi * i / n
            pts.append((c[0] + r * math.sin(a), y, c[2] + r * math.cos(a)))
            nrm.append((math.sin(a), 0.0, math.cos(a)))
    for i in range(n):
        j = (i + 1) % n
        tris += [(i, j, n + i), (j, n + j, n + i)]
    add(bone, material, pts, tris, (c[0], c[1] + h / 2, c[2]), None, nrm)


flat('straw', (200, 170, 70))
flat('coat', (60, 40, 90))
flat('trousers', (45, 45, 55))
flat('boots', (30, 22, 18))
flat('stem', (50, 120, 40))
flat('patch', (180, 60, 50))

# the pumpkin head: orange with a carved, glowing face
S = 128
img = Image.new('RGB', (S, S), (235, 115, 20))
d = ImageDraw.Draw(img)
for x in range(0, S, 16):
    d.line([(x, 0), (x, S)], fill=(205, 95, 15), width=2)
d.polygon([(22, 40), (50, 40), (36, 62)], fill=(255, 225, 90))
d.polygon([(78, 40), (106, 40), (92, 62)], fill=(255, 225, 90))
d.polygon([(58, 66), (70, 66), (64, 78)], fill=(255, 225, 90))
d.polygon([(24, 86), (36, 96), (48, 88), (64, 100), (80, 88), (92, 96), (104, 86), (98, 108), (80, 104), (64, 114), (48, 104), (30, 108)], fill=(255, 225, 90))
img.save(os.path.join(out, 'head.png'))

# the head and the stem on it (the head bone is 22)
ball(22, 'head', (0, 1.68, 0.0), (0.27, 0.23, 0.25), int(24 * HEAD), int(16 * HEAD), face=True)
tube(22, 'stem', (0.0, 1.88, 0.0), 0.04, 0.07)
# the body: the chest follows the upper spine (4), the waist the lower (2), the straw collar the chest
box(4, 'coat', (-0.17, 1.2, -0.09), (0.17, 1.5, 0.11))
box(4, 'patch', (0.02, 1.25, 0.11), (0.12, 1.35, 0.12))
box(4, 'straw', (-0.1, 1.49, -0.05), (0.1, 1.55, 0.07))
box(2, 'coat', (-0.165, 1.0, -0.085), (0.165, 1.22, 0.1))
box(0, 'trousers', (-0.17, 0.92, -0.08), (0.17, 1.04, 0.1))
for k, s in enumerate((1, -1)):
    # arms: sleeves from the shoulder to the elbow to the wrist, straw hands; a ball of the sleeve over each joint
    sh, el, wr, he = (mirror(p, s) for p in (SHOULDER, ELBOW, WRIST, HAND_END))
    ball(UPPER_ARM[k], 'coat', sh, 0.08)
    prism(UPPER_ARM[k], 'coat', sh, el, 0.055, 0.05, 1.0)
    ball(FOREARM[k], 'coat', el, 0.062)
    prism(FOREARM[k], 'coat', el, wr, 0.05, 0.045, 1.0)
    ball(HAND[k], 'straw', wr, 0.05)
    prism(HAND[k], 'straw', wr, he, 0.045, 0.035, 0.7)
    # legs: trousers from the hip to the knee to the ankle, a boot on the foot
    hp, kn, an, to = (mirror(p, s) for p in (HIP, KNEE, ANKLE, TOE))
    ball(THIGH[k], 'trousers', hp, 0.085)
    prism(THIGH[k], 'trousers', hp, kn, 0.082, 0.07, 1.0)
    ball(SHIN[k], 'trousers', kn, 0.075)
    prism(SHIN[k], 'trousers', kn, an, 0.07, 0.06, 1.0)
    ball(FOOT[k], 'boots', an, 0.07)
    box(FOOT[k], 'boots', (an[0] - 0.075, 0.0, -0.07), (an[0] + 0.075, 0.14, 0.2))

with open(os.path.join(out, 'pumpkin-jack.obj'), 'w') as f:
    f.write('mtllib pumpkin-jack.mtl\n')
    for v in verts:
        f.write('v %.5f %.5f %.5f\n' % v)
    for t in uvs:
        f.write('vt %.5f %.5f\n' % t)
    for n in norms:
        f.write('vn %.5f %.5f %.5f\n' % tuple(n))
    for bone, fs in groups.items():
        f.write('o %s\n' % bone)
        cur = None
        for m, tri in fs:
            if m != cur:
                f.write('usemtl %s\n' % m)
                cur = m
            f.write('f ' + ' '.join(('%d/%d/%d' % (i, i, n)) if n else ('%d/%d' % (i, i)) for i, n in tri) + '\n')
with open(os.path.join(out, 'pumpkin-jack.mtl'), 'w') as f:
    for m in ('straw', 'coat', 'trousers', 'boots', 'stem', 'patch', 'head'):
        f.write('newmtl %s\nKd 1 1 1\nmap_Kd %s.png\n\n' % (m, m))
print(len(verts), 'vertices,', sum(len(v) for v in groups.values()), 'triangles')
