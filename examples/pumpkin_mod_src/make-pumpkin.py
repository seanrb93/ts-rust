"""Makes a new character from scratch: Pumpkin Jack, a scarecrow with a pumpkin head, as an OBJ (a pose of arms down, facing +z) with its pictures.

    python examples/pumpkin_mod_src/make-pumpkin.py <out folder>
"""
import math
import os
import sys

import numpy as np
from PIL import Image, ImageDraw

out = sys.argv[1]
os.makedirs(out, exist_ok=True)
verts, uvs, faces = [], [], []   # faces: (material, [(v, vt)...])


def flat(name, rgb, size=8):
    Image.new('RGB', (size, size), rgb).save(os.path.join(out, name + '.png'))


def add(material, pts, tris, uv):
    base = len(verts)
    for p, t in zip(pts, uv):
        verts.append(p)
        uvs.append(t)
    for a, b, c in tris:
        faces.append((material, [(base + a + 1, base + a + 1), (base + b + 1, base + b + 1), (base + c + 1, base + c + 1)]))


def ellipsoid(material, c, r, n=16, m=10, planar=True):
    pts, uv, tris = [], [], []
    for j in range(m + 1):
        th = math.pi * j / m
        for i in range(n + 1):
            ph = 2 * math.pi * i / n
            p = (c[0] + r[0] * math.sin(th) * math.sin(ph), c[1] + r[1] * math.cos(th), c[2] + r[2] * math.sin(th) * math.cos(ph))
            pts.append(p)
            # the front (+z) is the middle of the picture; seen from the front it is mapped flat
            uv.append((0.5 + 0.5 * math.sin(th) * math.sin(ph), 0.5 - 0.5 * math.cos(th)) if planar else (i / n, 1 - j / m))
    for j in range(m):
        for i in range(n):
            a = j * (n + 1) + i
            b = a + n + 1
            tris += [(a, a + 1, b), (a + 1, b + 1, b)]
    add(material, pts, tris, uv)


def box(material, lo, hi):
    x0, y0, z0 = lo
    x1, y1, z1 = hi
    corners = [(x0, y0, z0), (x1, y0, z0), (x1, y1, z0), (x0, y1, z0), (x0, y0, z1), (x1, y0, z1), (x1, y1, z1), (x0, y1, z1)]
    quads = [(4, 5, 6, 7), (1, 0, 3, 2), (0, 4, 7, 3), (5, 1, 2, 6), (3, 7, 6, 2), (0, 1, 5, 4)]
    for q in quads:
        add(material, [corners[i] for i in q], [(0, 1, 2), (0, 2, 3)], [(0, 0), (1, 0), (1, 1), (0, 1)])


def cylinder(material, c, r, h, n=12):
    pts, uv, tris = [], [], []
    for k, y in enumerate((c[1], c[1] + h)):
        for i in range(n + 1):
            a = 2 * math.pi * i / n
            pts.append((c[0] + r * math.sin(a), y, c[2] + r * math.cos(a)))
            uv.append((i / n, k))
    for i in range(n):
        tris += [(i, i + 1, n + 1 + i), (i + 1, n + 2 + i, n + 1 + i)]
    add(material, pts, tris, uv)


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

# legs and boots (x = side, y = up, z = front)
for s in (-1, 1):
    box('boots', (s * 0.1 - 0.085, 0.0, -0.08), (s * 0.1 + 0.085, 0.13, 0.2))
    box('trousers', (s * 0.1 - 0.075, 0.13, -0.06), (s * 0.1 + 0.075, 0.88, 0.07))
box('trousers', (-0.19, 0.78, -0.09), (0.19, 0.96, 0.11))
# the coat and the straw sticking out of it
box('coat', (-0.24, 0.94, -0.11), (0.24, 1.46, 0.13))
box('patch', (0.02, 1.1, 0.13), (0.14, 1.22, 0.14))
box('straw', (-0.13, 1.44, -0.07), (0.13, 1.5, 0.08))
# the arms hang at the sides, in coat sleeves with straw hands
for s in (-1, 1):
    ellipsoid('coat', (s * 0.31, 1.38, 0.0), (0.09, 0.09, 0.09), 10, 6, False)
    box('coat', (s * 0.31 - 0.055, 0.98, -0.055), (s * 0.31 + 0.055, 1.38, 0.055))
    box('straw', (s * 0.31 - 0.05, 0.84, -0.05), (s * 0.31 + 0.05, 0.98, 0.05))
    for f in (-1, 0, 1):
        box('straw', (s * 0.31 + f * 0.035 - 0.012, 0.72, -0.012), (s * 0.31 + f * 0.035 + 0.012, 0.84, 0.012))
# the head
ellipsoid('head', (0, 1.68, 0.0), (0.27, 0.23, 0.25))
cylinder('stem', (0.0, 1.88, 0.0), 0.04, 0.07)

# the object file, with a material file
with open(os.path.join(out, 'pumpkin-jack.obj'), 'w') as f:
    f.write('mtllib pumpkin-jack.mtl\n')
    for v in verts:
        f.write('v %.5f %.5f %.5f\n' % v)
    for t in uvs:
        f.write('vt %.5f %.5f\n' % t)
    cur = None
    for m, tri in faces:
        if m != cur:
            f.write('usemtl %s\n' % m)
            cur = m
        f.write('f ' + ' '.join('%d/%d' % c for c in tri) + '\n')
with open(os.path.join(out, 'pumpkin-jack.mtl'), 'w') as f:
    for m in ('straw', 'coat', 'trousers', 'boots', 'stem', 'patch', 'head'):
        f.write('newmtl %s\nKd 1 1 1\nmap_Kd %s.png\n\n' % (m, m))
print(len(verts), 'vertices', len(faces), 'triangles')
