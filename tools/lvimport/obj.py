"""Wavefront OBJ (and its MTL): what Blender, Maya, 3ds Max and most other tools export."""
import os

import numpy as np
from PIL import Image

from .common import Material, Scene

MARKER_PREFIXES = ('spawn', 'weapon', 'pickup')


def _read_mtl(path):
    mats, cur = {}, None
    if not os.path.exists(path):
        return mats
    for line in open(path, encoding='utf8', errors='replace'):
        t = line.split()
        if not t:
            continue
        if t[0] == 'newmtl':
            cur = mats.setdefault(' '.join(t[1:]), {})
        elif cur is not None and t[0] == 'map_Kd':
            cur['map'] = os.path.join(os.path.dirname(path), line.split(None, 1)[1].split()[-1].strip().replace('\\', '/'))
        elif cur is not None and t[0] == 'Kd' and len(t) >= 4:
            cur['kd'] = tuple(float(x) for x in t[1:4])
        elif cur is not None and t[0] in ('d', 'Tr') and len(t) >= 2:
            cur['d'] = float(t[1]) if t[0] == 'd' else 1.0 - float(t[1])
    return mats


def load(path, opts):
    verts, colours, uvs, faces, mtl = [], [], [], [], {}
    mat, obj = '', ''
    for line in open(path, encoding='utf8', errors='replace'):
        t = line.split()
        if not t or t[0].startswith('#'):
            continue
        k = t[0]
        if k == 'v':
            verts.append([float(x) for x in t[1:4]])
            colours.append([float(x) for x in t[4:7]] if len(t) >= 7 else None)
        elif k == 'vt':
            uvs.append((float(t[1]), float(t[2]) if len(t) > 2 else 0.0))
        elif k == 'f':
            idx = []
            for w in t[1:]:
                p = w.split('/')
                v = int(p[0])
                v = v - 1 if v > 0 else len(verts) + v
                vt = -1
                if len(p) > 1 and p[1]:
                    vt = int(p[1])
                    vt = vt - 1 if vt > 0 else len(uvs) + vt
                idx.append((v, vt))
            if len(idx) >= 3:
                faces.append((mat, idx, obj))
        elif k == 'usemtl':
            mat = ' '.join(t[1:])
        elif k in ('o', 'g'):
            obj = ' '.join(t[1:])
        elif k == 'mtllib':
            mtl.update(_read_mtl(os.path.join(os.path.dirname(path), ' '.join(t[1:]))))
    pts = np.array(verts, dtype=np.float64)
    sc = Scene(up='y', scale=1.0, label=os.path.basename(path))
    markers = {}
    for m, idx, name in faces:
        if name.lower().startswith(MARKER_PREFIXES):
            markers.setdefault(name, []).extend(pts[v] for v, _ in idx)
            continue
        if m not in sc.mats:
            info = mtl.get(m, {})
            mp = info.get('map')
            loader = (lambda p=mp: Image.open(p)) if mp and os.path.exists(mp) else None
            sc.mats[m] = Material(m, loader, info.get('kd', (0.8, 0.8, 0.8)), translucent=info.get('d', 1.0) < 0.99)
        p = [pts[v] for v, _ in idx]
        # (an OBJ's v grows upwards, a picture's t downwards)
        uv = [(uvs[vt][0], 1.0 - uvs[vt][1]) if 0 <= vt < len(uvs) else (0.0, 0.0) for _, vt in idx]
        col = [tuple(colours[v][:3]) if colours[v] else (1.0, 1.0, 1.0) for v, _ in idx]
        sc.add(p, uv, col, m)
    for name, ps in markers.items():
        sc.markers.append((name, sum(ps) / len(ps)))
    return sc
