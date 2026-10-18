"""Source engine maps (Half-Life 2, Garry's Mod, Counter-Strike: Source, Team Fortress 2, Portal, Left 4 Dead, ...): compiled `.bsp` files (version
17 to 21, also as `.bsp.bz2`) and `.vmf` files of Hammer.

From a BSP the world model (model 0) is read: faces with their texture coordinates and brightness (from the lightmap), and displacements (terrain).
Props, brush entities (doors, lifts) and the 3D skybox are left out.  Player starts become `spawn` markers and weapons `weapon1` .. `weapon5`.

Pictures: a surface's material (`.vmt`) names a texture (`.vtf`).  They are looked for in the map's own pack (custom materials are inside the
BSP), then in the `materials` folders given with `--materials` (the game's own can be unpacked with a tool such as GCFScape or Crowbar), and next to
the map (a `materials` folder in a parent folder, as Crowbar unpacks workshop maps).  A surface whose picture is not found is drawn in the average
colour the map recorded for it.
"""
import bz2
import io
import lzma
import os
import struct
import zipfile

import numpy as np
from PIL import Image

from . import vtf
from .common import Material, Scene, keep_playable, markers_from_entities, parse_entities

SKIP_FLAGS = 0x2 | 0x4 | 0x40 | 0x80 | 0x100 | 0x200        # sky2d, sky, trigger, nodraw, hint, skip
WARP, TRANS = 0x8, 0x10


class Materials:
    """Finds a material's picture in the BSP's pack and in the folders given."""

    def __init__(self, bsp_path, pak, dirs):
        self.pak = pak
        self.dirs = list(dirs)
        d = os.path.dirname(os.path.abspath(bsp_path))
        for up in range(4):
            c = os.path.join(d, 'materials')
            if os.path.isdir(c) and c not in self.dirs:
                self.dirs.append(c)
            d = os.path.dirname(d)
        self.pakmap = {n.lower(): n for n in pak.namelist()} if pak else {}

    def read(self, rel):
        rel = rel.replace('\\', '/').lower()
        if rel in self.pakmap:
            return self.pak.read(self.pakmap[rel])
        for d in self.dirs:
            p = os.path.join(d, rel[len('materials/'):] if rel.startswith('materials/') else rel)
            if os.path.exists(p):
                return open(p, 'rb').read()
        return None

    def material(self, name):
        """(loader, flags dict) of a material by name like `concrete/wall01`."""
        name = name.replace('\\', '/').lower().strip('/')
        raw = self.read(f'materials/{name}.vmt')
        params = vtf.parse_vmt(raw.decode('latin1')) if raw else {}
        # (a patch material names the one it patches)
        if 'include' in params and not params.get('$basetexture'):
            inc = self.read(params['include'].replace('\\', '/').lower())
            if inc:
                params = {**vtf.parse_vmt(inc.decode('latin1')), **params}
        base = params.get('$basetexture')
        loader = None
        if base:
            data = self.read(f'materials/{base.replace(chr(92), "/").lower()}.vtf')
            if data:
                loader = lambda data=data: vtf.read_vtf(data)
        return loader, params


def load_bsp_bytes(path):
    d = open(path, 'rb').read()
    if path.lower().endswith('.bz2') or d[:3] == b'BZh':
        d = bz2.decompress(d)
    return d


def lump_data(d, i):
    off, ln, ver, four = struct.unpack_from('<iiii', d, 8 + 16 * i)
    data = d[off:off + ln]
    if data[:4] == b'LZMA' and len(data) >= 17:
        actual, comp = struct.unpack_from('<II', data, 4)
        props = data[12:17]
        data = lzma.decompress(props + struct.pack('<Q', actual) + data[17:17 + comp], format=lzma.FORMAT_ALONE)
    return data, ver


def load(path, opts):
    if path.lower().endswith('.vmf'):
        from . import vmf
        return vmf.load(path, opts)
    d = load_bsp_bytes(path)
    if d[:4] != b'VBSP':
        raise SystemExit(f'{path}: not a Source map')
    version = struct.unpack_from('<i', d, 4)[0]
    L = lambda i: lump_data(d, i)[0]
    ents = parse_entities(L(0).decode('latin1', 'replace'))
    planes = [struct.unpack_from('<3ff', L(1), 20 * i) for i in range(len(L(1)) // 20)]
    texdata_raw = L(2)
    verts = np.frombuffer(L(3), dtype='<f4').reshape(-1, 3).astype(np.float64)
    texinfo_raw = L(6)
    faces_raw, face_ver = lump_data(d, 7)
    light_raw = L(8) or L(53)
    edges = np.frombuffer(L(12), dtype='<u2').reshape(-1, 2)
    surfedges = np.frombuffer(L(13), dtype='<i4')
    models = L(14)
    first, count = struct.unpack_from('<2i', models, 40)
    strdata = L(43)
    strtab = np.frombuffer(L(44), dtype='<i4')
    dispinfo_raw, dispverts_raw = L(26), L(33)
    try:
        pak = zipfile.ZipFile(io.BytesIO(L(40)))
    except Exception:
        pak = None
    mats = Materials(path, pak, opts.get('materials') or [])
    sc = Scene(up='z', scale=0.025, label=os.path.basename(path))
    sc.min_light = 0.35
    markers_from_entities(ents, sc)
    sky = []
    for e in ents:
        if e.get('classname') == 'sky_camera':
            try:
                sky.append(np.array([float(x) for x in e['origin'].split()]))
            except (KeyError, ValueError):
                pass

    texinfo = [struct.unpack_from('<16f2i', texinfo_raw, 72 * i) for i in range(len(texinfo_raw) // 72)]
    texdata = [struct.unpack_from('<3f4i', texdata_raw, 32 * i) for i in range(len(texdata_raw) // 32)]

    def tex_name(td):
        o = int(strtab[texdata[td][3]])
        return strdata[o:strdata.index(b'\0', o)].decode('latin1').lower()

    FS = 56
    face_ids = []
    for fi in range(first, first + count):
        fe, ne = struct.unpack_from('<ih', faces_raw, FS * fi + 4)
        vi = []
        for k in range(ne):
            se = int(surfedges[fe + k])
            e = edges[abs(se)]
            vi.append(int(e[0] if se >= 0 else e[1]))
        face_ids.append(vi)
    keep = keep_playable(face_ids, verts, sc.markers, drop_points=sky)
    cache = {}

    def material(name, flags):
        if name in cache:
            return cache[name]
        loader, params = mats.material(name)
        water = bool(flags & WARP) or name.startswith('water') or 'water' in name.split('/')[:2]
        translucent = params.get('$translucent') in ('1', 'true') or params.get('$alphatest') in ('1', 'true') or bool(flags & TRANS) or water
        alphatest = params.get('$alphatest') in ('1', 'true')
        inner = loader
        if loader is not None and not (translucent or alphatest):
            # (the alpha of an ordinary texture is a mask for something else: the surface is solid)
            loader = lambda inner=inner: _opaque(inner())
        m = Material(name, loader, (0.5, 0.5, 0.5), translucent=translucent)
        if water and inner is not None:
            m._loader = lambda inner=inner: _water(inner())
        cache[name] = m
        sc.mats[name] = m
        return m

    for n, fi in enumerate(range(first, first + count)):
        if n not in keep:
            continue
        o = FS * fi
        planenum, side = struct.unpack_from('<HB', faces_raw, o)
        ti, dispinfo = struct.unpack_from('<hh', faces_raw, o + 10)
        styles = faces_raw[o + 16:o + 20]
        lightofs = struct.unpack_from('<i', faces_raw, o + 20)[0]
        lsize = struct.unpack_from('<2i', faces_raw, o + 36)
        if ti < 0 or ti >= len(texinfo):
            continue
        tinfo = texinfo[ti]
        flags, td = tinfo[16], tinfo[17]
        if flags & SKIP_FLAGS or not 0 <= td < len(texdata):
            continue
        name = tex_name(td)
        if name.startswith('tools/'):
            continue
        pts = verts[face_ids[n]]
        if len(pts) < 3:
            continue
        refl = texdata[td][0:3]
        wtex, htex = texdata[td][4], texdata[td][5]
        wtex, htex = max(wtex, 1), max(htex, 1)
        sv, so = np.array(tinfo[0:3]), tinfo[3]
        tv, to = np.array(tinfo[4:7]), tinfo[7]
        pn = np.array(planes[planenum][0:3]) * (-1.0 if side else 1.0)

        # the brightness: the average of the lightmap's first style (r, g, b and an exponent for each luxel)
        shade = 1.0
        if lightofs >= 0 and styles[0] == 0:
            nlux = (lsize[0] + 1) * (lsize[1] + 1)
            if 0 < nlux and lightofs + 4 * nlux <= len(light_raw):
                a = np.frombuffer(light_raw, dtype=np.uint8, count=4 * nlux, offset=lightofs).reshape(-1, 4)
                ex = a[:, 3].astype(np.int8).astype(np.float64)
                lin = (a[:, :3].astype(np.float64) / 255.0) * (2.0 ** ex)[:, None]
                v = float(lin.mean())
                shade = min(1.0, max(0.15, v ** (1 / 2.2) * 1.3))
        m = material(name, flags)
        if m._loader is None:
            # (no picture found: the colour the map recorded; shaded in the colour, there is no picture)
            m.color = tuple(float(min(1.0, max(0.05, c ** (1 / 2.2)))) for c in refl)

        polys = []
        if dispinfo >= 0 and dispinfo * 176 + 176 <= len(dispinfo_raw):
            polys = displacement(pts, dispinfo, dispinfo_raw, dispverts_raw)
        else:
            polys = [pts]
        for poly in polys:
            q = np.asarray(poly)
            cn = np.cross(q[1] - q[0], q[2] - q[0])
            order = list(range(len(q)))
            if np.dot(cn, pn) < 0:
                order.reverse()
            s_ = q @ sv + so
            t_ = q @ tv + to
            uv = [(float(a) / wtex, float(b) / htex) for a, b in zip(s_, t_)]
            sc.add([q[i] for i in order], [uv[i] for i in order], [(shade, shade, shade)] * len(q), name)
    return sc


def displacement(corners, di, dispinfo_raw, dispverts_raw):
    """The triangles of a displaced face: a grid of (2^power + 1)^2 points moved along their vectors."""
    o = di * 176
    start = np.array(struct.unpack_from('<3f', dispinfo_raw, o))
    vstart, _tri, power = struct.unpack_from('<3i', dispinfo_raw, o + 12)
    n = (1 << power) + 1
    c = [np.asarray(p) for p in corners[:4]]
    if len(c) != 4:
        return []
    k = int(np.argmin([np.linalg.norm(p - start) for p in c]))
    c = c[k:] + c[:k]
    dv = np.frombuffer(dispverts_raw, dtype='<f4', count=n * n * 5, offset=vstart * 20).reshape(n * n, 5) if (vstart + n * n) * 20 <= len(dispverts_raw) else None
    if dv is None:
        return []
    grid = np.empty((n, n, 3))
    for y in range(n):
        for x in range(n):
            ty, tx = y / (n - 1), x / (n - 1)
            left = c[0] + (c[1] - c[0]) * ty
            right = c[3] + (c[2] - c[3]) * ty
            base = left + (right - left) * tx
            v = dv[y * n + x]
            grid[y, x] = base + v[0:3].astype(np.float64) * float(v[3])
    out = []
    for y in range(n - 1):
        for x in range(n - 1):
            a, b, cc, dd = grid[y, x], grid[y, x + 1], grid[y + 1, x + 1], grid[y + 1, x]
            out.append([a, b, cc])
            out.append([a, cc, dd])
    return _orient(out, corners)


def _orient(tris, corners):
    """Triangles wound the way the face is (the game's faces go clockwise: they are turned to the original face's winding afterwards)."""
    c = np.asarray(corners)
    fn = np.cross(c[1] - c[0], c[2] - c[0])
    out = []
    for t in tris:
        tn = np.cross(t[1] - t[0], t[2] - t[0])
        out.append(t if np.dot(tn, fn) >= 0 else [t[0], t[2], t[1]])
    return out


def _opaque(im):
    a = np.asarray(im.convert('RGBA')).copy()
    a[:, :, 3] = 255
    return Image.fromarray(a, 'RGBA')


def _water(im):
    a = np.asarray(im.convert('RGBA')).copy()
    a[:, :, 3] = 150
    return Image.fromarray(a, 'RGBA')
