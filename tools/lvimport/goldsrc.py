"""GoldSrc maps (Half-Life 1 and its mods, Counter-Strike 1.6, Team Fortress Classic, ...): compiled `.bsp` files, version 30.

The world model (model 0) is read: its faces with their texture coordinates, the textures inside the map or in the `.wad` files given with `--wad`
(Half-Life's own are in `halflife.wad` and the game folder's `.wad` files; the map's `wad` entity names the ones it uses and they are looked for
next to the map and in the folders given), and a brightness for each face from the map's light.  Entities: player starts become `spawn`
markers and weapons `weapon1` .. `weapon5`.  Skies, clips, triggers and other tool textures are left out; water is drawn see-through.
"""
import os
import struct

import numpy as np
from PIL import Image

from .common import Material, Scene, newell, keep_playable, markers_from_entities, parse_entities

SKIP = ('sky', 'clip', 'null', 'aaatrigger', 'trigger', 'origin', 'hint', 'skip', 'noclip', 'bevel', 'contentwater', 'blocklight', 'black_hidden',
        'env/', 'solidhint')


def lump(d, i):
    o, ln = struct.unpack_from('<ii', d, 4 + 8 * i)
    return d[o:o + ln]


def read_miptex(buf, at):
    """(name, width, height, RGBA picture or None) of a miptex at `at` in `buf`; None as the picture when its pixels are not in it."""
    name = buf[at:at + 16].split(b'\0')[0].decode('latin1')
    w, h = struct.unpack_from('<II', buf, at + 16)
    offs = struct.unpack_from('<4I', buf, at + 24)
    if offs[0] == 0 or w == 0 or h == 0 or w > 4096 or h > 4096:
        return name, w, h, None
    pix = np.frombuffer(buf, dtype=np.uint8, count=w * h, offset=at + offs[0])
    pal_at = at + offs[3] + (w // 8) * (h // 8) + 2
    pal = np.frombuffer(buf, dtype=np.uint8, count=768, offset=pal_at).reshape(256, 3)
    rgba = np.empty((w * h, 4), dtype=np.uint8)
    rgba[:, :3] = pal[pix]
    rgba[:, 3] = 255
    if name.startswith('{'):
        rgba[pix == 255] = (0, 0, 0, 0)
    return name, w, h, Image.fromarray(rgba.reshape(h, w, 4), 'RGBA')


def read_wad(path):
    """name (lower case) -> (width, height, picture) of the textures of a WAD3 file."""
    d = open(path, 'rb').read()
    if d[:4] != b'WAD3':
        return {}
    n, tab = struct.unpack_from('<II', d, 4)
    out = {}
    for i in range(n):
        pos, disk, size, typ, comp = struct.unpack_from('<iiibb', d, tab + 32 * i)
        if typ != 0x43 or comp != 0:
            continue
        name, w, h, img = read_miptex(d, pos)
        if img is not None:
            out[name.lower()] = (w, h, img)
    return out


def load(path, opts):
    d = open(path, 'rb').read()
    if struct.unpack_from('<i', d, 0)[0] != 30:
        raise SystemExit(f'{path}: not a GoldSrc map (version {struct.unpack_from("<i", d, 0)[0]}, 30 is expected)')
    ents = parse_entities(lump(d, 0).decode('latin1', 'replace'))
    plane_raw = lump(d, 1)
    planes = [struct.unpack_from('<4fi', plane_raw, 20 * i) for i in range(len(plane_raw) // 20)]
    tex_lump = lump(d, 2)
    verts = np.frombuffer(lump(d, 3), dtype='<f4').reshape(-1, 3).astype(np.float64)
    texinfo_raw = lump(d, 6)
    faces_raw = lump(d, 7)
    light = lump(d, 8)
    edges = np.frombuffer(lump(d, 12), dtype='<u2').reshape(-1, 2)
    surfedges = np.frombuffer(lump(d, 13), dtype='<i4')
    model0 = struct.unpack_from('<9f7i', lump(d, 14), 0)
    first, count = model0[14], model0[15]

    # textures: in the map, or in the wads
    miptex = []
    if len(tex_lump) >= 4:
        n = struct.unpack_from('<i', tex_lump, 0)[0]
        for i in range(n):
            off = struct.unpack_from('<i', tex_lump, 4 + 4 * i)[0]
            miptex.append(read_miptex(tex_lump, off) if off >= 0 else ('', 0, 0, None))
    wad_names = []
    for e in ents:
        if e.get('classname') == 'worldspawn':
            wad_names = [os.path.basename(w.replace('\\', '/')) for w in e.get('wad', '').split(';') if w]
    wad_files = list(opts.get('wad') or [])
    search = [os.path.dirname(os.path.abspath(path))] + [x for x in wad_files if os.path.isdir(x)]
    for w in wad_names:
        for s in search:
            c = os.path.join(s, w)
            if os.path.exists(c):
                wad_files.append(c)
    wads = {}
    for w in wad_files:
        if os.path.isfile(w):
            for k, v in read_wad(w).items():
                wads.setdefault(k, v)

    sc = Scene(up='z', scale=0.025, label=os.path.basename(path))
    sc.min_light = 0.35
    markers_from_entities(ents, sc)
    texinfo = [struct.unpack_from('<8f2i', texinfo_raw, 40 * i) for i in range(len(texinfo_raw) // 40)]

    face_pts, face_ids = [], []
    for fi in range(first, first + count):
        plane, side, fe, ne, ti = struct.unpack_from('<HHiHH', faces_raw, 20 * fi)
        vi = []
        for k in range(ne):
            se = int(surfedges[fe + k])
            e = edges[abs(se)]
            vi.append(int(e[0] if se >= 0 else e[1]))
        face_ids.append(vi)
    keep = keep_playable(face_ids, verts, sc.markers, drop_points=())
    n_used = 0
    for n, fi in enumerate(range(first, first + count)):
        if n not in keep:
            continue
        plane, side, fe, ne, ti = struct.unpack_from('<HHiHH', faces_raw, 20 * fi)
        styles = faces_raw[20 * fi + 12:20 * fi + 16]
        lightofs = struct.unpack_from('<i', faces_raw, 20 * fi + 16)[0]
        if ti >= len(texinfo):
            continue
        tinfo = texinfo[ti]
        mt = tinfo[8]
        if not 0 <= mt < len(miptex):
            continue
        name, w, h, img = miptex[mt]
        lname = name.lower()
        if lname.startswith(SKIP):
            continue
        if img is None and lname in wads:
            w, h, img = wads[lname]
        if w == 0 or h == 0:
            w = h = 64
        pts = verts[face_ids[n]]
        if len(pts) < 3:
            continue
        sv, so, tv, to = np.array(tinfo[0:3]), tinfo[3], np.array(tinfo[4:7]), tinfo[7]
        s = pts @ sv + so
        t = pts @ tv + to
        uv = [(float(a) / w, float(b) / h) for a, b in zip(s, t)]
        # (the faces go clockwise seen from the front: turn them round, checking against the plane's normal)
        pn = np.array(planes[plane][:3]) * (-1.0 if side else 1.0)
        cn = newell(pts)
        order = list(range(len(pts)))
        if np.dot(cn, pn) < 0:
            order.reverse()
        # the brightness of the face: the average of its lightmap (the first style), else full
        shade = 1.0
        if lightofs >= 0 and styles[0] == 0 and light:
            smin, smax = np.floor(s.min() / 16), np.ceil(s.max() / 16)
            tmin, tmax = np.floor(t.min() / 16), np.ceil(t.max() / 16)
            lw_, lh_ = int(smax - smin) + 1, int(tmax - tmin) + 1
            nb = lw_ * lh_ * 3
            if 0 < nb and lightofs + nb <= len(light):
                shade = float(np.frombuffer(light, dtype=np.uint8, count=nb, offset=lightofs).mean()) / 255.0
                shade = min(1.0, max(0.18, shade * 1.8))
        key = name or f'texture{mt}'
        if key not in sc.mats:
            water = key.startswith(('!', '*'))
            sc.mats[key] = Material(key, (lambda im=img: im) if img is not None else None, (0.5, 0.5, 0.5), translucent=water or key.startswith('{'))
            if water:
                sc.mats[key]._loader = (lambda im=img: _tint(im)) if img is not None else None
        sc.add([pts[i] for i in order], [uv[i] for i in order], [(shade, shade, shade)] * len(pts), key)
        n_used += 1
    return sc


def _tint(im):
    a = np.asarray(im.convert('RGBA')).copy()
    a[:, :, 3] = 140
    return Image.fromarray(a, 'RGBA')
