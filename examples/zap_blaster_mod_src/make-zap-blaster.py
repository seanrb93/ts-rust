"""Makes a new weapon model from scratch: the Zap Blaster, a chunky ray gun, as an OBJ for the gun in the first person (the pistol's layout: the barrel along +z, the grip down).

    python examples/zap_blaster_mod_src/make-zap-blaster.py <out folder>

The objects `part3` (the frame) and `part2` (the capacitor on top, which the game moves back when the gun fires) replace those of the pistol's model.
"""
import math
import os
import sys

import numpy as np
from PIL import Image

out = sys.argv[1]
os.makedirs(out, exist_ok=True)
verts, uvs, norms, groups = [], [], [], {}


def flat(name, rgb):
    Image.new('RGB', (8, 8), rgb).save(os.path.join(out, name + '.png'))


def add(part, material, pts, tris, centre, nrm=None):
    base, base_n = len(verts), len(norms)
    for p in pts:
        verts.append(p)
        uvs.append((0.5, 0.5))
    if nrm:
        norms.extend(nrm)
    for a, b, c in tris:
        pa, pb, pc = (np.array(pts[i]) for i in (a, b, c))
        n = np.cross(pb - pa, pc - pa)
        if np.linalg.norm(n) < 1e-12:
            continue
        if np.dot(n, (pa + pb + pc) / 3 - np.array(centre)) < 0:
            b, c = c, b
        groups.setdefault(part, []).append((material, [(base + i + 1, (base_n + i + 1) if nrm else None) for i in (a, b, c)]))


def box(part, material, lo, hi):
    x0, y0, z0 = lo
    x1, y1, z1 = hi
    c = [(x0, y0, z0), (x1, y0, z0), (x1, y1, z0), (x0, y1, z0), (x0, y0, z1), (x1, y0, z1), (x1, y1, z1), (x0, y1, z1)]
    for q in [(4, 5, 6, 7), (1, 0, 3, 2), (0, 4, 7, 3), (5, 1, 2, 6), (3, 7, 6, 2), (0, 1, 5, 4)]:
        add(part, material, [c[i] for i in q], [(0, 1, 2), (0, 2, 3)], ((x0 + x1) / 2, (y0 + y1) / 2, (z0 + z1) / 2))


def tube(part, material, centre, radius, z0, z1, n=14, cap=True):
    """A cylinder along z (a barrel, a ring), smooth shaded."""
    cx, cy = centre
    pts, nrm, tris = [], [], []
    for z in (z0, z1):
        for i in range(n):
            a = 2 * math.pi * i / n
            pts.append((cx + radius * math.cos(a), cy + radius * math.sin(a), z))
            nrm.append((math.cos(a), math.sin(a), 0.0))
    for i in range(n):
        j = (i + 1) % n
        tris += [(i, j, n + i), (j, n + j, n + i)]
    add(part, material, pts, tris, (cx, cy, (z0 + z1) / 2), nrm)
    if cap:
        for z, k in ((z0, -1), (z1, 1)):
            pts = [(cx, cy, z)] + [(cx + radius * math.cos(2 * math.pi * i / n), cy + radius * math.sin(2 * math.pi * i / n), z) for i in range(n)]
            tris = [(0, 1 + i, 1 + (i + 1) % n) for i in range(n)]
            add(part, material, pts, tris, (cx, cy, z - k * 0.01), [(0.0, 0.0, float(k))] * len(pts))


flat('steel', (95, 105, 125))
flat('dark', (35, 35, 45))
flat('glow', (80, 255, 190))
flat('orange', (230, 120, 30))

# the frame: grip, receiver, barrel with glowing coils, a fin
box('part3', 'dark', (-0.016, -0.115, -0.075), (0.016, -0.005, -0.03))
box('part3', 'orange', (-0.017, -0.115, -0.075), (0.017, -0.1, -0.03))
box('part3', 'steel', (-0.024, -0.012, -0.1), (0.024, 0.038, 0.05))
box('part3', 'dark', (-0.026, 0.0, -0.1), (0.026, 0.012, -0.06))
box('part3', 'steel', (-0.006, 0.038, -0.1), (0.006, 0.062, -0.07))
tube('part3', 'dark', (0.0, 0.016), 0.012, 0.05, 0.16)
for k in range(4):
    z = 0.065 + 0.03 * k
    tube('part3', 'glow' if k % 2 == 0 else 'orange', (0.0, 0.016), 0.02, z, z + 0.014)
box('part3', 'dark', (-0.01, -0.03, -0.012), (0.01, -0.012, 0.03))
# the capacitor on top, which the slide of the pistol would be
tube('part2', 'glow', (0.0, 0.052), 0.011, -0.07, 0.04)
tube('part2', 'steel', (0.0, 0.052), 0.014, -0.075, -0.06)
tube('part2', 'steel', (0.0, 0.052), 0.014, 0.025, 0.04)

with open(os.path.join(out, 'zap-blaster.obj'), 'w') as f:
    f.write('mtllib zap-blaster.mtl\n')
    for v in verts:
        f.write('v %.5f %.5f %.5f\n' % v)
    for t in uvs:
        f.write('vt %.5f %.5f\n' % t)
    for n in norms:
        f.write('vn %.5f %.5f %.5f\n' % n)
    for part, fs in groups.items():
        f.write('o %s\n' % part)
        cur = None
        for m, tri in fs:
            if m != cur:
                f.write('usemtl %s\n' % m)
                cur = m
            f.write('f ' + ' '.join(('%d/%d/%d' % (i, i, n)) if n else ('%d/%d' % (i, i)) for i, n in tri) + '\n')
with open(os.path.join(out, 'zap-blaster.mtl'), 'w') as f:
    for m in ('steel', 'dark', 'glow', 'orange'):
        f.write('newmtl %s\nKd 1 1 1\nmap_Kd %s.png\n\n' % (m, m))
print(len(verts), 'vertices,', sum(len(v) for v in groups.values()), 'triangles')
