"""Source engine maps (Half-Life 2, Garry's Mod, Counter-Strike: Source, Team Fortress 2, Portal, Left 4 Dead, ...): compiled `.bsp` files (version
17 to 21, also as `.bsp.bz2`) and `.vmf` files of Hammer.

From a BSP the world model (model 0) is read: faces with their texture coordinates and brightness (from the lightmap), and displacements (terrain).
Props (the models that the map places) are made part of the level; the 3D skybox is left out (see `props.py`).  The brush entities that are only a shape (`func_brush`, `func_wall`, `func_illusionary`, `func_breakable`, buttons, and
the starting place of lifts and rotating parts) are part of the level, with their collision; doors only with `--doors closed`.  Player starts become `spawn`
markers and weapons `weapon1` .. `weapon5`.

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
from . import props as props_mod
from .common import Material, Scene, newell, brush_polygons, clip_halfspace, keep_playable, markers_from_entities, parse_entities, brush_entity_models

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
    light_detail = int(opts.get('light_detail', 16))
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

    headnode = struct.unpack_from('<i', models, 36)[0]
    ent_models = brush_entity_models(ents, len(models) // 48, opts, SHAPE_ENTITIES, DOOR_ENTITIES,
                                     lambda m: max(abs(b - a) for a, b in zip(struct.unpack_from('<3f', models, 48 * m), struct.unpack_from('<3f', models, 48 * m + 12))))
    if ent_models:
        names = {}
        for _m, _solid, cn in ent_models:
            names[cn] = names.get(cn, 0) + 1
        print('brush entities: ' + ', '.join(f'{n} {c}' for c, n in sorted(names.items())))
    if not opts.get('no_brushes'):
        coll, nb = world_brushes(d, L, planes, headnode)
        for m, solid, _cn in ent_models:
            if solid:
                more, nb2 = world_brushes(d, L, planes, struct.unpack_from('<i', models, 48 * m + 36)[0])
                coll += more
                nb += nb2
        sc.collision = coll
        print(f'collision: {nb} brushes, {len(coll)} polygons')
    texinfo = [struct.unpack_from('<16f2i', texinfo_raw, 72 * i) for i in range(len(texinfo_raw) // 72)]
    texdata = [struct.unpack_from('<3f4i', texdata_raw, 32 * i) for i in range(len(texdata_raw) // 32)]

    def tex_name(td):
        o = int(strtab[texdata[td][3]])
        return strdata[o:strdata.index(b'\0', o)].decode('latin1').lower()

    FS = 56
    face_ids = []
    # (the faces of the world, then those of the brush entities that are part of the level)
    all_faces = list(range(first, first + count))
    for m, _solid, _cn in ent_models:
        f0, c0 = struct.unpack_from('<2i', models, 48 * m + 40)
        all_faces += range(f0, f0 + c0)
    for fi in all_faces:
        fe, ne = struct.unpack_from('<ih', faces_raw, FS * fi + 4)
        vi = []
        for k in range(ne):
            se = int(surfedges[fe + k])
            e = edges[abs(se)]
            vi.append(int(e[0] if se >= 0 else e[1]))
        face_ids.append(vi)
    keep = keep_playable(face_ids[:count], verts, sc.markers, drop_points=sky)
    keep = set(keep) | set(range(count, len(all_faces)))
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

    for n, fi in enumerate(all_faces):
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

        # the light: the lightmap's first style (r, g, b and an exponent for each luxel), seen at every vertex (the polygon is cut along the lightmap's
        # grid where it varies much: `light_detail` luxels between cuts; 0: one brightness for the face)
        lum = None
        if lightofs >= 0 and styles[0] == 0:
            nlux = (lsize[0] + 1) * (lsize[1] + 1)
            if 0 < nlux and lightofs + 4 * nlux <= len(light_raw):
                a = np.frombuffer(light_raw, dtype=np.uint8, count=4 * nlux, offset=lightofs).reshape(-1, 4)
                ex = a[:, 3].astype(np.int8).astype(np.float64)
                lin = (a[:, :3].astype(np.float64) / 255.0) * (2.0 ** ex)[:, None]
                lum = np.clip(lin.mean(axis=1) ** (1 / 2.2) * 1.3, 0.15, 1.0).reshape(lsize[1] + 1, lsize[0] + 1)
        m = material(name, flags)
        if m._loader is None:
            # (no picture found: the colour the map recorded; shaded in the colour, there is no picture)
            m.color = tuple(float(min(1.0, max(0.05, c ** (1 / 2.2)))) for c in refl)
        lvs, lvt = np.array(tinfo[8:11]), np.array(tinfo[12:15])
        lo_s, lo_t = tinfo[11] - struct.unpack_from('<i', faces_raw, o + 28)[0], tinfo[15] - struct.unpack_from('<i', faces_raw, o + 32)[0]

        def shade_at(pt):
            if lum is None:
                return 1.0
            fs_, ft_ = float(pt @ lvs) + lo_s, float(pt @ lvt) + lo_t
            fs_ = min(max(fs_, 0.0), lum.shape[1] - 1.0)
            ft_ = min(max(ft_, 0.0), lum.shape[0] - 1.0)
            s0, t0 = int(fs_), int(ft_)
            s1, t1 = min(s0 + 1, lum.shape[1] - 1), min(t0 + 1, lum.shape[0] - 1)
            a_, b_ = fs_ - s0, ft_ - t0
            return float((lum[t0, s0] * (1 - a_) + lum[t0, s1] * a_) * (1 - b_) + (lum[t1, s0] * (1 - a_) + lum[t1, s1] * a_) * b_)
        varied = lum is not None and light_detail > 0 and float(lum.std()) > 0.05 and max(lum.shape) > light_detail
        flat = float(lum.mean()) if lum is not None else 1.0

        polys = []
        if dispinfo >= 0 and dispinfo * 176 + 176 <= len(dispinfo_raw):
            polys = displacement(pts, dispinfo, dispinfo_raw, dispverts_raw)
        elif varied:
            # cut along the grid of the lightmap, every `light_detail` luxels
            pieces = [list(pts)]
            for vec, off, size in ((lvs, lo_s, lum.shape[1]), (lvt, lo_t, lum.shape[0])):
                for k in range(light_detail, int(size), light_detail):
                    nxt = []
                    for pc in pieces:
                        vals = [float(q @ vec) + off for q in pc]
                        if min(vals) < k - 1e-6 and max(vals) > k + 1e-6:
                            # (the line s = k: points with s <= k, and those with s >= k)
                            nrm = vec / np.linalg.norm(vec)
                            dd = (k - off) / np.linalg.norm(vec)
                            nxt += [x for x in (clip_halfspace(pc, nrm, dd), clip_halfspace(pc, -nrm, -dd)) if len(x) >= 3]
                        else:
                            nxt.append(pc)
                    pieces = nxt
            polys = pieces
        else:
            polys = [pts]
        for poly in polys:
            q = np.asarray(poly)
            cn = newell(q)
            order = list(range(len(q)))
            if np.dot(cn, pn) < 0:
                order.reverse()
            s_ = q @ sv + so
            t_ = q @ tv + to
            uv = [(float(a) / wtex, float(b) / htex) for a, b in zip(s_, t_)]
            if varied or dispinfo >= 0 and lum is not None and light_detail > 0:
                cols = [(c, c, c) for c in (shade_at(x) for x in q)]
            else:
                cols = [(flat, flat, flat)] * len(q)
            sc.add([q[i] for i in order], [uv[i] for i in order], [cols[i] for i in order], name)
    if not opts.get('no_props'):
        placed = props_mod.static_props(d, lump_data(d, 35)[0], version) + props_mod.entity_props(ents)
        if placed:
            models = props_mod.Models(pak, mats.dirs, opts.get('models'))

            def prop_material(path):
                material(path, 0)
                return path
            props_mod.add_props(sc, placed, models, prop_material, lambda p: mats.read(p) is not None, opts)
    return sc


# brush entities that are only a shape: they are drawn, and stop players, where they start
SHAPE_ENTITIES = ('func_brush', 'func_wall', 'func_wall_toggle', 'func_illusionary', 'func_breakable', 'func_button', 'func_rotating', 'func_movelinear',
                  'func_physbox', 'func_tracktrain')
DOOR_ENTITIES = ('func_door', 'func_door_rotating', 'func_movelinear_door')

SOLID_CONTENTS = 0x1 | 0x2 | 0x8 | 0x4000 | 0x10000        # solid, window, grate, moveable, player clip


def world_brushes(d, L, planes, headnode):
    """The collision of the world: the polygons of the sides of the brushes that stop players (solid, window, grate, player clip), the invisible
    ones included, with the ones that are inside other brushes left out.  Returns (polygons, number of brushes); each polygon is a list of points."""
    nodes = L(5)
    leafs, _ = lump_data(d, 10)
    version = struct.unpack_from('<i', d, 4)[0]
    leaf_size = 56 if version <= 19 else 32
    leafbrushes = np.frombuffer(L(17), dtype='<u2')
    brushes = np.frombuffer(L(18), dtype='<i4').reshape(-1, 3)
    sides = np.frombuffer(L(19), dtype=[('plane', '<u2'), ('texinfo', '<i2'), ('disp', '<i2'), ('bevel', '<i2')])
    # the brushes of the world model: those of the leaves of its tree
    seen, stack, mine = set(), [headnode], set()
    while stack:
        n = stack.pop()
        if n < 0:
            lf = -n - 1
            if lf in seen:
                continue
            seen.add(lf)
            first, num = struct.unpack_from('<2H', leafs, lf * leaf_size + 24)
            mine.update(int(x) for x in leafbrushes[first:first + num])
        else:
            c0, c1 = struct.unpack_from('<2i', nodes, n * 32 + 4)
            stack += [c0, c1]
    solids = []
    for bi in sorted(mine):
        first, num, contents = (int(x) for x in brushes[bi])
        if not contents & SOLID_CONTENTS:
            continue
        sd = sides[first:first + num]
        if (sd['disp'] > 0).any():
            continue            # (a displacement's collision is its triangles)
        pl = [(np.array(planes[int(s['plane'])][0:3], dtype=np.float64), float(planes[int(s['plane'])][3])) for s in sd]
        solids.append((pl, [i for i, s in enumerate(sd) if s['bevel'] == 0]))
    # polygons of every real side
    boxes, polys = [], []
    for k, (pl, want) in enumerate(solids):
        ps = brush_polygons(pl, set(want))
        pts = [q for poly in ps if poly for q in poly]
        if not pts:
            boxes.append(None)
            continue
        arr = np.array(pts)
        boxes.append((arr.min(axis=0) - 1.0, arr.max(axis=0) + 1.0))
        for poly in ps:
            if poly:
                polys.append((k, poly))
    # inside another brush: not a surface (a floor inside a wall must not be stood on)
    cell = 512.0
    grid = {}
    for k, bx in enumerate(boxes):
        if bx is None:
            continue
        for gx in range(int(bx[0][0] // cell), int(bx[1][0] // cell) + 1):
            for gy in range(int(bx[0][1] // cell), int(bx[1][1] // cell) + 1):
                grid.setdefault((gx, gy), []).append(k)

    def inside_other(k, pt):
        for j in grid.get((int(pt[0] // cell), int(pt[1] // cell)), []):
            if j == k or boxes[j] is None:
                continue
            lo, hi = boxes[j]
            if not (np.all(pt >= lo) and np.all(pt <= hi)):
                continue
            if all(n @ pt <= dd - 0.01 for n, dd in solids[j][0]):
                return True
        return False
    out = []
    for k, poly in polys:
        c = sum(poly) / len(poly)
        n = np.cross(poly[1] - poly[0], poly[2] - poly[0])
        ln = np.linalg.norm(n)
        if ln < 1e-9:
            continue
        n /= ln
        # (a point a little out of the side, and one in the middle of the side: both inside another brush means it is covered)
        if inside_other(k, c + n * 0.5) and inside_other(k, c - n * 0.5 + n * 0.51):
            continue
        out.append(poly)
    return out, len(solids)


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
