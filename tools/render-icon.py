"""Render the save-icon model of the game (`tssave.ico.obj` with its texture, from ps2iodb) as seen from the front: a transparent PNG and a Windows icon.

    python tools/render-icon.py <tssave.ico.obj> <out folder> [--view front|back|top] [--size 1024]

writes <out>/icon-<size>.png and <out>/ts-port.ico (16 to 256 pixels).  A plain software renderer: orthographic, the vertex colours times the texture, a depth buffer, drawn large and
scaled down for smooth edges.
"""
import os
import sys

import numpy as np
from PIL import Image


def load(path):
    base = os.path.dirname(path)
    tex_name = None
    mtl = os.path.join(base, os.path.splitext(os.path.basename(path))[0] + '.mtl')
    for f in (mtl, path.replace('.obj', '.mtl')):
        if os.path.exists(f):
            for l in open(f):
                if l.lower().startswith('map_kd'):
                    tex_name = l.split(None, 1)[1].strip()
            break
    tex = Image.open(os.path.join(base, tex_name)).convert('RGBA') if tex_name else None
    v, col, vt, faces = [], [], [], []
    for l in open(path):
        w = l.split()
        if not w:
            continue
        if w[0] == 'v':
            v.append([float(x) for x in w[1:4]])
            col.append([float(x) for x in w[4:7]] if len(w) >= 7 and not w[4].startswith('#') else [1, 1, 1])
        elif w[0] == 'vt':
            vt.append([float(w[1]), float(w[2])])
        elif w[0] == 'f':
            idx = []
            for c in w[1:]:
                p = c.split('/')
                idx.append((int(p[0]) - 1, int(p[1]) - 1 if len(p) > 1 and p[1] else -1))
            for k in range(1, len(idx) - 1):
                faces.append((idx[0], idx[k], idx[k + 1]))
    return np.array(v), np.array(col), np.array(vt), faces, tex


def render(v, col, vt, faces, tex, view, size):
    # the model: y up.  The logo reads the right way round from -z (the side the game's own icon shows), which is the `front` here; `back` is the other side
    if view == 'front':
        p = np.stack([-v[:, 0], v[:, 1], -v[:, 2]], 1)
    elif view == 'top':
        p = np.stack([v[:, 0], -v[:, 2], v[:, 1]], 1)
    else:
        p = v.copy()
    lo, hi = p[:, :2].min(0), p[:, :2].max(0)
    span = (hi - lo).max()
    pad = 0.06
    scale = size * (1 - 2 * pad) / span
    off = np.array([size / 2, size / 2]) - (lo + hi) / 2 * scale
    sx = p[:, 0] * scale + off[0]
    sy = size - (p[:, 1] * scale + off[1])
    sz = p[:, 2]
    img = np.zeros((size, size, 4), np.float32)
    depth = np.full((size, size), -1e9, np.float32)
    texa = np.asarray(tex, np.float32) / 255.0 if tex is not None else None
    for tri in faces:
        ids = [t[0] for t in tri]
        uvs = [vt[t[1]] if t[1] >= 0 else np.zeros(2) for t in tri]
        xs, ys, zs = sx[ids], sy[ids], sz[ids]
        x0, x1 = int(max(0, np.floor(xs.min()))), int(min(size - 1, np.ceil(xs.max())))
        y0, y1 = int(max(0, np.floor(ys.min()))), int(min(size - 1, np.ceil(ys.max())))
        if x1 < x0 or y1 < y0:
            continue
        d = (ys[1] - ys[2]) * (xs[0] - xs[2]) + (xs[2] - xs[1]) * (ys[0] - ys[2])
        if abs(d) < 1e-9:
            continue
        gx, gy = np.meshgrid(np.arange(x0, x1 + 1) + 0.5, np.arange(y0, y1 + 1) + 0.5)
        l0 = ((ys[1] - ys[2]) * (gx - xs[2]) + (xs[2] - xs[1]) * (gy - ys[2])) / d
        l1 = ((ys[2] - ys[0]) * (gx - xs[2]) + (xs[0] - xs[2]) * (gy - ys[2])) / d
        l2 = 1 - l0 - l1
        inside = (l0 >= -1e-6) & (l1 >= -1e-6) & (l2 >= -1e-6)
        if not inside.any():
            continue
        z = l0 * zs[0] + l1 * zs[1] + l2 * zs[2]
        region = depth[y0:y1 + 1, x0:x1 + 1]
        win = inside & (z > region)
        if not win.any():
            continue
        c = (l0[..., None] * col[ids[0]] + l1[..., None] * col[ids[1]] + l2[..., None] * col[ids[2]]) * 2.0   # (vertex colours are 0.5 = unchanged on the console)
        rgba = np.ones(c.shape[:2] + (4,), np.float32)
        if texa is not None:
            u = l0 * uvs[0][0] + l1 * uvs[1][0] + l2 * uvs[2][0]
            w = l0 * uvs[0][1] + l1 * uvs[1][1] + l2 * uvs[2][1]
            tx = np.clip((u % 1.0) * texa.shape[1], 0, texa.shape[1] - 1).astype(int)
            ty = np.clip((w % 1.0) * texa.shape[0], 0, texa.shape[0] - 1).astype(int)
            rgba = texa[ty, tx].copy()
        rgba[..., :3] = np.clip(rgba[..., :3] * c, 0, 1)
        region[win] = z[win]
        sub = img[y0:y1 + 1, x0:x1 + 1]
        sub[win] = rgba[win]
    return Image.fromarray((np.clip(img, 0, 1) * 255).astype(np.uint8), 'RGBA')


def main():
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    path, out = sys.argv[1], sys.argv[2]
    arg = lambda k, d=None: sys.argv[sys.argv.index(k) + 1] if k in sys.argv else d
    view, size = arg('--view', 'front'), int(arg('--size', 1024))
    os.makedirs(out, exist_ok=True)
    v, col, vt, faces, tex = load(path)
    big = render(v, col, vt, faces, tex, view, size)
    big.save(os.path.join(out, f'icon-{view}-{size}.png'))
    sizes = [16, 24, 32, 48, 64, 128, 256]
    big.save(os.path.join(out, 'ts-port.ico'), sizes=[(s, s) for s in sizes])
    print(f'{len(faces)} triangles; written to {out}')


if __name__ == '__main__':
    main()
