"""Hammer map sources (.vmf): the brushes of the world are turned into polygons (each face of a brush is its plane cut by the planes of the others),
with the texture coordinates of the face's `uaxis` and `vaxis`.  Displacements are made too.  Entities' brushes are left out; player starts and weapons
become markers.  Pictures are found as for a BSP (`--materials`, or a `materials` folder near the file).
"""
import os
import re
import struct

import numpy as np

from . import vtf
from .common import Material, Scene, markers_from_entities
from .source import Materials, _opaque, _water

TOKEN = re.compile(r'"((?:[^"\\]|\\.)*)"|([{}])|//[^\n]*|([A-Za-z_]\w*)')


def parse(text):
    """KeyValues: a block is (name, [(key, value) or block ...])."""
    toks = []
    for m in TOKEN.finditer(text):
        if m.group(1) is not None:
            toks.append(('s', m.group(1)))
        elif m.group(2):
            toks.append((m.group(2), m.group(2)))
        elif m.group(3):
            toks.append(('s', m.group(3)))
    pos = 0

    def block():
        nonlocal pos
        items = []
        while pos < len(toks):
            kind, val = toks[pos]
            if kind == '}':
                pos += 1
                return items
            pos += 1
            if kind == 's':
                nk, nv = toks[pos]
                if nk == '{':
                    pos += 1
                    items.append((val, block()))
                else:
                    pos += 1
                    items.append((val, nv))
        return items
    root = []
    while pos < len(toks):
        kind, val = toks[pos]
        pos += 1
        if kind == 's' and pos < len(toks) and toks[pos][0] == '{':
            pos += 1
            root.append((val, block()))
    return root


def vec(s):
    return np.array([float(x) for x in re.findall(r'-?[\d.]+(?:e-?\d+)?', s)])


def plane_polygons(sides):
    """For every side of a brush `(p1, p2, p3)`, the polygon of the side: a large square on its plane cut by the other sides' planes."""
    planes = []
    for p1, p2, p3 in sides:
        n = np.cross(p3 - p1, p2 - p1)
        l = np.linalg.norm(n)
        if l < 1e-9:
            planes.append(None)
            continue
        n = n / l
        planes.append((n, float(n @ p1)))
    out = []
    for i, pl in enumerate(planes):
        if pl is None:
            out.append(None)
            continue
        n, d = pl
        # a square on the plane, counter-clockwise seen from outside
        a = np.array([1.0, 0, 0]) if abs(n[0]) < 0.9 else np.array([0, 1.0, 0])
        u = np.cross(n, a)
        u /= np.linalg.norm(u)
        v = np.cross(n, u)
        c = n * d
        poly = [c - u * 1e5 - v * 1e5, c + u * 1e5 - v * 1e5, c + u * 1e5 + v * 1e5, c - u * 1e5 + v * 1e5]
        # (u x v = n for u, v as above up to sign: make the order go counter-clockwise about n)
        if np.dot(np.cross(poly[1] - poly[0], poly[2] - poly[0]), n) < 0:
            poly.reverse()
        for j, other in enumerate(planes):
            if j == i or other is None or not poly:
                continue
            poly = clip(poly, other[0], other[1])
        out.append(poly if len(poly) >= 3 else None)
    return out


def clip(poly, n, d):
    """Keep the part of a polygon with n . x <= d."""
    out = []
    for k in range(len(poly)):
        a, b = poly[k], poly[(k + 1) % len(poly)]
        da, db = n @ a - d, n @ b - d
        if da <= 1e-6:
            out.append(a)
        if (da < -1e-6 and db > 1e-6) or (da > 1e-6 and db < -1e-6):
            t = da / (da - db)
            out.append(a + (b - a) * t)
    return out


def load(path, opts):
    root = parse(open(path, encoding='utf8', errors='replace').read())
    sc = Scene(up='z', scale=0.025, label=os.path.basename(path))
    mats = Materials(path, None, opts.get('materials') or [])
    cache, sizes = {}, {}

    def size_of(name):
        if name not in sizes:
            loader, params = mats.material(name)
            base = params.get('$basetexture')
            w = h = 256
            if base:
                data = mats.read(f'materials/{base.replace(chr(92), "/").lower()}.vtf')
                if data and data[:4] == b'VTF\0':
                    w, h = struct.unpack_from('<HH', data, 16)
            sizes[name] = (max(w, 1), max(h, 1))
        return sizes[name]

    def material(name):
        if name in cache:
            return name
        loader, params = mats.material(name)
        translucent = params.get('$translucent') in ('1', 'true') or params.get('$alphatest') in ('1', 'true') or 'water' in name
        inner = loader
        if loader is not None and not translucent:
            loader = lambda inner=inner: _opaque(inner())
        m = Material(name, loader, (0.55, 0.55, 0.55), translucent=translucent)
        if 'water' in name and inner is not None:
            m._loader = lambda inner=inner: _water(inner())
        cache[name] = m
        sc.mats[name] = m
        return name

    ents = []
    for name, items in root:
        if name == 'world':
            for k, v in items:
                if k == 'solid':
                    solid(v, sc, material, size_of)
        elif name == 'entity':
            e = {k: v for k, v in items if isinstance(v, str)}
            ents.append(e)
    markers_from_entities(ents, sc)
    return sc


def solid(items, sc, material, size_of):
    sides = [v for k, v in items if k == 'side']
    pl, info = [], []
    for s in sides:
        d = {k: v for k, v in s if isinstance(v, str)}
        pts = re.findall(r'\(([^)]*)\)', d.get('plane', ''))
        if len(pts) != 3:
            pl.append(None)
            info.append(None)
            continue
        pl.append(tuple(vec(p) for p in pts))
        info.append((d, next((v for k, v in s if k == 'dispinfo'), None)))
    ok = [p for p in pl if p is not None]
    if len(ok) < 4:
        return
    polys = plane_polygons(ok)
    it = iter(polys)
    for p, i in zip(pl, info):
        if p is None:
            continue
        poly = next(it)
        if poly is None:
            continue
        d, disp = i
        name = d.get('material', '').lower()
        if name.startswith('tools/'):
            continue
        key = material(name)
        w, h = size_of(name)
        u = vec(d.get('uaxis', '[1 0 0 0] 0.25'))
        v = vec(d.get('vaxis', '[0 -1 0 0] 0.25'))
        us, vs = (u[4] if len(u) > 4 and u[4] else 0.25), (v[4] if len(v) > 4 and v[4] else 0.25)

        def uv_of(q):
            return (float((q @ u[:3]) / us + u[3]) / w, float((q @ v[:3]) / vs + v[3]) / h)
        if disp is not None:
            for tri in displaced(poly, disp):
                sc.add(tri, [uv_of(q) for q in tri], [(1.0, 1.0, 1.0)] * 3, key)
        else:
            sc.add(poly, [uv_of(q) for q in poly], [(1.0, 1.0, 1.0)] * len(poly), key)


def displaced(poly, disp):
    """Triangles of a displacement of a brush side (the side has four corners)."""
    if len(poly) != 4:
        return []
    d = {k: v for k, v in disp if isinstance(v, str)}
    power = int(d.get('power', 2))
    n = (1 << power) + 1
    start = vec(d.get('startposition', '[0 0 0]'))
    c = list(poly)
    k = int(np.argmin([np.linalg.norm(p - start) for p in c]))
    c = c[k:] + c[:k]
    blocks = {k: v for k, v in disp if isinstance(v, list)}
    normals = [vec(v) for kk, v in blocks.get('normals', [])] if 'normals' in blocks else None
    dists = [vec(v) for kk, v in blocks.get('distances', [])] if 'distances' in blocks else None
    if not normals or not dists or len(normals) < n or len(dists) < n:
        return []
    grid = np.empty((n, n, 3))
    for y in range(n):
        for x in range(n):
            ty, tx = y / (n - 1), x / (n - 1)
            left = c[0] + (c[1] - c[0]) * ty
            right = c[3] + (c[2] - c[3]) * ty
            base = left + (right - left) * tx
            nv = normals[y][3 * x:3 * x + 3]
            grid[y, x] = base + nv * dists[y][x]
    fn = np.cross(c[1] - c[0], c[2] - c[0])
    out = []
    for y in range(n - 1):
        for x in range(n - 1):
            a, b, cc, dd = grid[y, x], grid[y, x + 1], grid[y + 1, x + 1], grid[y + 1, x]
            for t in ([a, b, cc], [a, cc, dd]):
                tn = np.cross(t[1] - t[0], t[2] - t[0])
                out.append(t if np.dot(tn, fn) >= 0 else [t[0], t[2], t[1]])
    return out
