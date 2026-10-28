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
verts, uvs, norms, faces = [], [], [], []   # faces: (material, [(v, vt)...])


def flat(name, rgb, size=8):
    Image.new('RGB', (size, size), rgb).save(os.path.join(out, name + '.png'))


def add(material, pts, tris, uv, centre, normals=None):
    """A piece of the figure: its triangles are turned so that they face away from `centre` (the game lights a triangle by the side the winding says is outside)."""
    base = len(verts)
    for p, t in zip(pts, uv):
        verts.append(p)
        uvs.append(t)
    if normals:
        base_n = len(norms)
        norms.extend(normals)
    for a, b, c in tris:
        pa, pb, pc = (np.array(pts[i]) for i in (a, b, c))
        n = np.cross(pb - pa, pc - pa)
        if np.linalg.norm(n) < 1e-9:
            continue
        if np.dot(n, (pa + pb + pc) / 3 - np.array(centre)) < 0:
            b, c = c, b
        ids = (a, b, c)
        faces.append((material, [(base + i + 1, base + i + 1, (base_n + i + 1) if normals else None) for i in ids]))


def ellipsoid(material, c, r, n=20, m=14, planar=True):
    pts, uv, tris, nrm = [], [], [], []
    for j in range(m + 1):
        th = math.pi * j / m
        for i in range(n + 1):
            ph = 2 * math.pi * i / n
            p = (c[0] + r[0] * math.sin(th) * math.sin(ph), c[1] + r[1] * math.cos(th), c[2] + r[2] * math.sin(th) * math.cos(ph))
            pts.append(p)
            nrm.append(((p[0] - c[0]) / r[0] ** 2, (p[1] - c[1]) / r[1] ** 2, (p[2] - c[2]) / r[2] ** 2))
            # the front (+z) is the middle of the picture; seen from the front it is mapped flat
            uv.append((0.5 + 0.5 * math.sin(th) * math.sin(ph), 0.5 - 0.5 * math.cos(th)) if planar else (i / n, 1 - j / m))
    for j in range(m):
        for i in range(n):
            a = j * (n + 1) + i
            b = a + n + 1
            tris += [(a, a + 1, b), (a + 1, b + 1, b)]
    nrm = [tuple(np.array(v) / np.linalg.norm(v)) for v in nrm]
    add(material, pts, tris, uv, c, nrm)


def box(material, lo, hi, cell=0.1):
    """A box whose faces are cut into small squares: a triangle then follows one bone of the skeleton and the joints bend the figure instead of tearing it."""
    lo, hi = np.array(lo), np.array(hi)
    centre = (lo + hi) / 2
    for axis in range(3):
        u, v = [a for a in range(3) if a != axis]
        nu = max(1, int(math.ceil((hi[u] - lo[u]) / cell)))
        nv = max(1, int(math.ceil((hi[v] - lo[v]) / cell)))
        for side in (lo, hi):
            pts, uv, tris = [], [], []
            for j in range(nv + 1):
                for i in range(nu + 1):
                    p = np.array(side, dtype=float)
                    p[u] = lo[u] + (hi[u] - lo[u]) * i / nu
                    p[v] = lo[v] + (hi[v] - lo[v]) * j / nv
                    pts.append(tuple(p))
                    uv.append((i / nu, j / nv))
            for j in range(nv):
                for i in range(nu):
                    a = j * (nu + 1) + i
                    b = a + nu + 1
                    tris += [(a, a + 1, b), (a + 1, b + 1, b)]
            add(material, pts, tris, uv, centre)


def cylinder(material, c, r, h, n=12):
    pts, uv, tris = [], [], []
    for k, y in enumerate((c[1], c[1] + h)):
        for i in range(n + 1):
            a = 2 * math.pi * i / n
            pts.append((c[0] + r * math.sin(a), y, c[2] + r * math.cos(a)))
            uv.append((i / n, k))
    for i in range(n):
        tris += [(i, i + 1, n + 1 + i), (i + 1, n + 2 + i, n + 1 + i)]
    add(material, pts, tris, uv, (c[0], c[1] + h / 2, c[2]))


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

# the joints of the game's skeleton, which the parts here are set on (shoulders at x 0.23, y 1.47; elbows 1.19; wrists 0.91; hips at x 0.1, y 0.99; neck 1.57)
# legs and boots (x = side, y = up, z = front)
for s in (-1, 1):
    box('boots', (s * 0.1 - 0.075, 0.0, -0.07), (s * 0.1 + 0.075, 0.14, 0.19))
    box('trousers', (s * 0.1 - 0.085, 0.14, -0.06), (s * 0.1 + 0.085, 1.12, 0.08))
# the coat and the straw sticking out of it
box('coat', (-0.18, 1.0, -0.09), (0.18, 1.5, 0.11))
box('patch', (0.02, 1.2, 0.11), (0.12, 1.3, 0.12))
box('straw', (-0.1, 1.49, -0.05), (0.1, 1.55, 0.07))
# the arms hang at the sides, in coat sleeves with straw hands
for s in (-1, 1):
    pass
    box('coat', (s * 0.23 - 0.05, 0.93, -0.05), (s * 0.23 + 0.05, 1.45, 0.05), 0.08)
    box('straw', (s * 0.23 - 0.045, 0.8, -0.045), (s * 0.23 + 0.045, 0.93, 0.045), 0.08)
    for f in (-1, 0, 1):
        box('straw', (s * 0.23 + f * 0.03 - 0.012, 0.7, -0.012), (s * 0.23 + f * 0.03 + 0.012, 0.8, 0.012))
# the head
ellipsoid('head', (0, 1.68, 0.0), (0.27, 0.23, 0.25), 16, 10)
cylinder('stem', (0.0, 1.88, 0.0), 0.04, 0.07)

# the object file, with a material file
with open(os.path.join(out, 'pumpkin-jack.obj'), 'w') as f:
    f.write('mtllib pumpkin-jack.mtl\n')
    for v in verts:
        f.write('v %.5f %.5f %.5f\n' % v)
    for t in uvs:
        f.write('vt %.5f %.5f\n' % t)
    for n in norms:
        f.write('vn %.5f %.5f %.5f\n' % tuple(n))
    cur = None
    for m, tri in faces:
        if m != cur:
            f.write('usemtl %s\n' % m)
            cur = m
        f.write('f ' + ' '.join(('%d/%d/%d' % c) if c[2] else ('%d/%d' % c[:2]) for c in tri) + '\n')
with open(os.path.join(out, 'pumpkin-jack.mtl'), 'w') as f:
    for m in ('straw', 'coat', 'trousers', 'boots', 'stem', 'patch', 'head'):
        f.write('newmtl %s\nKd 1 1 1\nmap_Kd %s.png\n\n' % (m, m))
print(len(verts), 'vertices', len(faces), 'triangles')
