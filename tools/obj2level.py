"""Make a level for the port from a 3D model or a map of another game.

    python tools/obj2level.py <file> --out mods/MyLevel --number 33 --name "My Level" --like 25 [options]

Formats (by the file's extension):
    .obj                 Wavefront OBJ with its MTL (Blender, Maya, 3ds Max, ...)
    .glb .gltf           glTF 2.0 (Blender: File > Export > glTF 2.0; most tools)
    .blend               a Blender file: Blender (found by itself, or --blender <exe>) is run to export it
    .bsp                 a compiled Source map (Half-Life 2, Garry's Mod, CS:S, TF2, Portal, ...; also .bsp.bz2) or GoldSrc map (Half-Life, CS 1.6, ...)
    .vmf                 a Hammer map source (Source)

The result is a mod folder (see examples/levels_sample) holding what the game reads for a level, in the formats of docs/level-formats.md:

    mods/MyLevel/mod.ini
    mods/MyLevel/levels.txt                          one `level` line, with `setup setup.txt`
    mods/MyLevel/setup.txt                           the spawns, weapons and pickups, by the numbers of the pads
    mods/MyLevel/BG.PAK/bg/level33/level33.raw       the world: rooms, floors, walls, portals
    mods/MyLevel/MISC.PAK/pad/data/level33.raw       the pads and links bots walk by
    mods/MyLevel/MISC.PAK/propsetup/data/level33.raw no props
    mods/MyLevel/MISC.PAK/textures/<n>.qpm           the level's own textures

Faces that point up (less than 60 degrees from level) become floors players stand on, steep ones become walls that stop them (from the side
the face points to); ceilings are only drawn.  A material whose picture has see-through pixels is drawn after the solid ones.  The game
holds about 90 textures for a level: when a map has more materials, the ones that cover least are drawn with the most alike colour of the others.

Markers: in an OBJ an object or group (`o` / `g` line), in glTF and Blender a node or Empty, whose name starts with `spawn` marks where players
and bots start (the faces of a marker are not part of the level), `weapon1` .. `weapon5` where a weapon of that slot lies, `pickup<type>` a
pickup of that number (see docs/level-formats.md).  In a map of another game the player starts and weapons are the markers.  Without markers the
converter picks spots by itself (`--spawns`, `--weapons`).

Options:
    --scale S        multiply every coordinate (the game's rooms are about 20 units wide, a player about 1.8 high); the default is 1 for
                     models and 0.025 for maps of Source and GoldSrc (a player is 72 units there)
    --up y|z         which axis is up (default: the format's; z is turned into the game's y)
    --flip           reverse the winding of faces (when the level is inside out)
    --room-size R    split the level into rooms of R x R units, joined by portals (default 0: one room; large levels draw faster split)
    --pad-spacing D  distance of the pads bots walk by (default 5); --max-pads N makes it larger when there would be more pads (default 450)
    --modes LIST     the modes the level offers: dm,bagtag,ctb,escort,ko,ls (default dm,bagtag,ctb,ko)
    --tune N         the music track 1..27 (default: the level `like` plays)
    --max-textures N the most textures (default 90)
    --materials DIR  a `materials` folder (Source: the game's own pictures; may be given more than once); --wad FILE|DIR for GoldSrc
    --blender EXE    the Blender to use for .blend files
    --radius R       keep only what is within R units (a square) of --center x,z (default: the first spawn): large maps do not fit in the game
    --tex-base N     the number of the first texture of this level's own (default 4000 + 100 * (number - 31))
"""
import argparse
import math
import os
import re
import struct
import sys

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import png2qpm  # noqa: E402
import lvimport  # noqa: E402

CHUNK_VERTS = 64          # the most vertices one chunk can draw (the VU1 buffer)
FLOOR_NY = 0.5            # a face whose normal points up more than this is a floor
MAX_TEX = 256             # the largest side of a texture file (65536 pixels in all)
MAX_ROOMS = 180          # the game's tables hold about 200 rooms (a crash on more)
MODES = {'dm': 1, 'bagtag': 2, 'ctb': 4, 'escort': 8, 'ko': 16, 'ls': 32}


# ---------------------------------------------------------------- textures

def pow2_at_most(n, limit=MAX_TEX):
    p = 1
    while p * 2 <= min(n, limit):
        p *= 2
    # (the nearer power of two)
    if p * 2 <= limit and n - p > p * 2 - n:
        p *= 2
    return max(p, 1)


def material_image(m):
    """The picture of a material (a `lvimport.common.Material`) as RGBA; a plain square of its colour when it has none."""
    img = m.image()
    if img is None:
        img = Image.new('RGBA', (16, 16), tuple(int(max(0, min(1, c)) * 255) for c in m.color) + (255,))
    return img


def fit_texture(img):
    """The picture as the game takes it: power-of-two sides, 65536 pixels at the most."""
    w, h = img.size
    nw, nh = pow2_at_most(w), pow2_at_most(h)
    while nw * nh > 65536:
        if nw >= nh:
            nw //= 2
        else:
            nh //= 2
    if (nw, nh) != (w, h):
        img = img.resize((nw, nh), Image.LANCZOS)
    return img


# ---------------------------------------------------------------- the model

class Poly:
    __slots__ = ('p', 'uv', 'col', 'mat')       # p: list of 3 or 4 points (np arrays), uv: list of (s, t), col: list of (r, g, b)

    def __init__(self, p, uv, col, mat):
        self.p, self.uv, self.col, self.mat = p, uv, col, mat

    def centre(self):
        return sum(self.p) / len(self.p)

    def normal(self):
        n = np.zeros(3)
        for i in range(1, len(self.p) - 1):
            n += np.cross(self.p[i] - self.p[0], self.p[i + 1] - self.p[0])
        l = np.linalg.norm(n)
        return n / l if l > 1e-12 else n


def build_polys(scene, args):
    """The scene's polygons in the game's axes and units (3 or 4 points each), and its markers as a list of (name, point)."""
    up = args.up or scene.up
    scale = args.scale if args.scale is not None else scene.scale

    def conv(p):
        p = np.asarray(p, dtype=np.float64)
        if up == 'z':
            p = np.array([p[0], p[2], -p[1]])
        return p * scale
    polys = []
    for rp in scene.polys:
        pts = [conv(q) for q in rp.p]
        uv, col = list(rp.uv), list(rp.col)
        if args.flip:
            pts.reverse()
            uv.reverse()
            col.reverse()
        if len(pts) == 4:
            # (a quad has to be flat to be drawn as one strip: else it is two triangles)
            n = Poly(pts, uv, col, rp.mat).normal()
            if not (abs(np.dot(pts[2] - pts[0], np.cross(pts[1] - pts[0], pts[3] - pts[0]))) < 1e-3 * max(1.0, np.linalg.norm(n))):
                polys.append(Poly([pts[0], pts[1], pts[2]], uv[:3], col[:3], rp.mat))
                polys.append(Poly([pts[0], pts[2], pts[3]], [uv[0], uv[2], uv[3]], [col[0], col[2], col[3]], rp.mat))
                continue
        polys.append(Poly(pts, uv, col, rp.mat))
    markers = [(name, conv(pos)) for name, pos in scene.markers]
    return [p for p in polys if np.linalg.norm(p.normal()) > 0.5], markers


def reduce_materials(polys, scene, limit):
    """At most `limit` materials: the ones that cover least are changed for the one of the rest whose colour is most like theirs."""
    area = {}
    for p in polys:
        a = 0.0
        for i in range(1, len(p.p) - 1):
            a += np.linalg.norm(np.cross(p.p[i] - p.p[0], p.p[i + 1] - p.p[0])) / 2
        area[p.mat] = area.get(p.mat, 0.0) + a
    if len(area) <= limit:
        return
    ranked = sorted(area, key=area.get, reverse=True)
    keep, drop = ranked[:limit], ranked[limit:]
    for m in ranked:
        scene.mats[m].image()
    remap = {}
    for m in drop:
        c = np.array(scene.mats[m].color)
        best = min(keep, key=lambda k: float(np.linalg.norm(np.array(scene.mats[k].color) - c)) + (0.0 if scene.mats[k].translucent == scene.mats[m].translucent else 1.0))
        remap[m] = best
    for p in polys:
        p.mat = remap.get(p.mat, p.mat)
    print(f'{len(drop)} of {len(area)} materials were merged into the others')


class Out:
    """The file being written: offsets are where the bytes go."""

    def __init__(self):
        self.b = bytearray()

    def tell(self):
        return len(self.b)

    def align(self, n=16):
        while len(self.b) % n:
            self.b.append(0)

    def put(self, data):
        at = len(self.b)
        self.b += data
        return at

    def u32(self, at, v):
        struct.pack_into('<I', self.b, at, v & 0xffffffff)


def write_bg(polys, tex_numbers, args, rooms_polys, room_boxes, portals):
    """The BG file.  `tex_numbers`: material -> (texture number, translucent).  `rooms_polys`: polygons of each room."""
    o = Out()
    o.put(bytes(16))
    # the textures this level uses, in order of first use
    used = []
    for rp in rooms_polys:
        for p in rp:
            if p.mat not in used:
                used.append(p.mat)
    o.align()
    tex_at = o.tell()
    for m in used:
        o.put(struct.pack('<4i', tex_numbers[m][0], 0, 0, 0))
    o.put(struct.pack('<4i', -1, 0, 0, 0))
    slot = {m: i for i, m in enumerate(used)}
    nrooms = len(rooms_polys)
    o.align()
    rooms_at = o.tell()
    o.put(bytes(44 * (nrooms + 2)))
    o.align()
    portals_at = o.tell()
    o.put(bytes(4 * (len(portals) + 2)))
    portal_ptrs = []
    for (a, b, nrm, quad) in portals:
        o.align(4)
        at = o.put(struct.pack('<II3fHH', a, b, *nrm, len(quad), 0) + b''.join(struct.pack('<3f', *q) for q in quad))
        portal_ptrs.append(at)
    for i, at in enumerate(portal_ptrs):
        o.u32(portals_at + 4 * (i + 1), at)
    o.align()
    vols_at = o.put(bytes(16))
    for ri, rp in enumerate(rooms_polys):
        rec = rooms_at + 44 * (ri + 1)
        # neighbours
        o.align(4)
        nb = [i + 1 for i, (a, b, _, _) in enumerate(portals) if a == ri + 1 or b == ri + 1]
        nb_at = o.put(struct.pack('<I', len(nb)) + b''.join(struct.pack('<I', n) for n in nb))
        floors, walls = collision(rp)
        # floors
        o.align(4)
        f_ptrs = []
        for f in floors:
            f_ptrs.append(o.put(struct.pack('<IHH', len(f), 0, 0) + b''.join(struct.pack('<3f', *v) for v in f)))
        f_at = o.put(b''.join(struct.pack('<I', p) for p in f_ptrs) + struct.pack('<I', 0))
        w_ptrs = []
        for w in walls:
            w_ptrs.append(o.put(struct.pack('<I8f', 0, *w)))
        w_at = o.put(b''.join(struct.pack('<I', p) for p in w_ptrs) + struct.pack('<I', 0))
        ob = write_model(o, rp, slot, tex_numbers)
        box = room_boxes[ri]
        struct.pack_into('<5I6f', o.b, rec, ob, nb_at, f_at, w_at, 0, *box)
    struct.pack_into('<4I', o.b, 0, tex_at, rooms_at, portals_at, vols_at)
    o.align()
    return bytes(o.b)


def write_model(o, polys, slot, tex_numbers):
    """A room's model: one part, a chunk for every run of up to 64 vertices of one texture.  Returns the offset of the model block."""
    sets = []
    for translucent in (False, True):
        by_tex = {}
        for p in polys:
            if tex_numbers[p.mat][1] == translucent:
                by_tex.setdefault(p.mat, []).append(p)
        desc, pos, st, rgba, chunks = [], [], [], [], []
        for mat, ps in by_tex.items():
            cur = []          # strips of the chunk being made: lists of (p, uv, col)
            nv = 0

            def flush():
                nonlocal cur, nv
                if not cur:
                    return
                v0 = len(pos)
                d0 = len(desc)
                for k, strip in enumerate(cur):
                    desc.append(len(strip) | (0x8000 if k == len(cur) - 1 else 0))
                    for (pp, uv, col) in strip:
                        pos.append(pp)
                        st.append(uv)
                        rgba.append(col)
                chunks.append((slot[mat], d0, len(cur), v0, nv, 0))
                while len(pos) % 4:       # (the next chunk starts on a 16 byte boundary)
                    pos.append((0.0, 0.0, 0.0))
                    st.append((0.0, 0.0))
                    rgba.append((1.0, 1.0, 1.0))
                cur, nv = [], 0
            for p in ps:
                n = len(p.p)
                if nv + n > CHUNK_VERTS:
                    flush()
                # (a quad is the strip a, b, d, c)
                order = [0, 1, 2] if n == 3 else [0, 1, 3, 2]
                cur.append([(p.p[i], p.uv[i], p.col[i]) for i in order])
                nv += n
            flush()
        sets.append((desc, pos, st, rgba, chunks))
    arrays = []
    for (desc, pos, st, rgba, chunks) in sets:
        if not chunks:
            arrays.append(None)
            continue
        o.align()
        d_at = o.put(b''.join(struct.pack('<4I', c, 0x302e4000, 0x412, 0) for c in desc))
        o.align()
        p_at = o.put(b''.join(struct.pack('<3f', *x) for x in pos))
        o.align()
        s_at = o.put(b''.join(struct.pack('<4f', s, t, 1.0, 1.0) for s, t in st))
        o.align()
        c_at = o.put(b''.join(bytes(max(0, min(255, int(round(v * 127)))) for v in col) + b'\x7f' for col in rgba))
        o.align()
        ck_at = o.put(b''.join(struct.pack('<6i', *c) for c in chunks) + struct.pack('<6i', 0, 0, 0, 0, 0, -1))
        arrays.append((d_at, p_at, s_at, c_at, ck_at))
    # the strips' first-triangle flag stays 0: a strip of a polygon in order is drawn the way the game's own are
    o.align()
    tris = sum(max(0, (c & 0x7fff) - 2) for s in sets for c in s[0])
    part_at = o.tell()
    part = bytearray(80)
    struct.pack_into('<BBbbbbBB', part, 0, 0, 0, -1, -1, -1, -1, 0, 0)
    for k, a in enumerate(arrays):
        if a is None:
            continue
        d_at, p_at, s_at, c_at, ck_at = a
        base = 8 + 4 * k
        struct.pack_into('<I', part, base, ck_at)
        struct.pack_into('<4I', part, 20 + 24 * k, d_at, p_at, s_at, c_at)
    struct.pack_into('<HH', part, 68, 0, min(tris, 65535))
    o.put(bytes(part))
    ob_at = o.put(struct.pack('<I', 1) + bytes(60))
    return ob_at


def collision(polys):
    """Floors (lists of vertices) and walls (8 floats) of the polygons of a room."""
    floors, walls = [], []
    for p in polys:
        n = p.normal()
        if n[1] > FLOOR_NY:
            floors.append([tuple(float(c) for c in v) for v in p.p])
        elif abs(n[1]) <= FLOOR_NY:
            xz = [(v[0], v[2]) for v in p.p]
            best, pair = -1.0, (0, 0)
            for i in range(len(xz)):
                for j in range(i + 1, len(xz)):
                    d = (xz[i][0] - xz[j][0]) ** 2 + (xz[i][1] - xz[j][1]) ** 2
                    if d > best:
                        best, pair = d, (i, j)
            if best < 1e-6:
                continue
            lo, hi = min(v[1] for v in p.p), max(v[1] for v in p.p)
            a, b = p.p[pair[0]], p.p[pair[1]]
            # (a wall stops what comes from one side only: the open side is to the right of the edge's direction seen from above, which is
            # the side the polygon faces)
            if (b[0] - a[0]) * n[2] - (b[2] - a[2]) * n[0] > 0:
                a, b = b, a
            walls.append((a[0], lo, a[2], hi, b[0], lo, b[2], hi))
    return floors, walls


# ---------------------------------------------------------------- pads, spawns, pickups

def make_pads(polys, boxes, args, markers):
    """Pads on a grid over the floors (the grid is made wider when it would give more than `--max-pads`); links between neighbours."""
    d = args.pad_spacing
    for _ in range(6):
        pads, links = _pads(polys, args, d)
        if len(pads) <= args.max_pads:
            break
        d *= max(1.1, (len(pads) / args.max_pads) ** 0.5)
    return pads, links


def _pads(polys, args, d):
    """Pads on a grid over the floors: (x, y, z, room); links between neighbours."""
    floors = [(i, p) for i, p in enumerate(polys) if p.normal()[1] > FLOOR_NY]
    if not floors:
        return [], []
    allp = np.array([v for _, p in floors for v in p.p])
    lo, hi = allp.min(axis=0), allp.max(axis=0)
    nx, nz = int((hi[0] - lo[0]) / d) + 1, int((hi[2] - lo[2]) / d) + 1
    # a grid of the heights at every point (a point can be over floors of several heights)
    cells = {}
    tris = []
    for _, p in floors:
        for i in range(1, len(p.p) - 1):
            tris.append((p.p[0], p.p[i], p.p[i + 1]))
    bbs = [(min(t[0][0], t[1][0], t[2][0]), max(t[0][0], t[1][0], t[2][0]), min(t[0][2], t[1][2], t[2][2]), max(t[0][2], t[1][2], t[2][2])) for t in tris]
    grid = {}
    for k, bb in enumerate(bbs):
        for gx in range(int((bb[0] - lo[0]) / d), int((bb[1] - lo[0]) / d) + 1):
            for gz in range(int((bb[2] - lo[2]) / d), int((bb[3] - lo[2]) / d) + 1):
                grid.setdefault((gx, gz), []).append(k)

    def height_at(x, z, gx, gz):
        hs = []
        for k in grid.get((gx, gz), []):
            a, b, c = tris[k]
            den = (b[2] - c[2]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[2] - c[2])
            if abs(den) < 1e-9:
                continue
            l1 = ((b[2] - c[2]) * (x - c[0]) + (c[0] - b[0]) * (z - c[2])) / den
            l2 = ((c[2] - a[2]) * (x - c[0]) + (a[0] - c[0]) * (z - c[2])) / den
            l3 = 1 - l1 - l2
            if min(l1, l2, l3) >= -1e-6:
                hs.append(l1 * a[1] + l2 * b[1] + l3 * c[1])
        return hs

    pads = []
    at = {}
    for gx in range(nx):
        for gz in range(nz):
            x, z = lo[0] + (gx + 0.5) * d, lo[2] + (gz + 0.5) * d
            for h in height_at(x, z, gx, gz):
                at[(gx, gz, len(at))] = None
                pads.append((x, h, z, gx, gz))
    index = {}
    for i, p in enumerate(pads):
        index.setdefault((p[3], p[4]), []).append(i)
    links = []
    for i, p in enumerate(pads):
        for dx, dz in ((1, 0), (0, 1)):
            for j in index.get((p[3] + dx, p[4] + dz), []):
                if abs(pads[j][1] - p[1]) <= args.step:
                    links.append((i, j))
    return pads, links


def room_of(point, boxes):
    for i, b in enumerate(boxes):
        if b[0] <= point[0] <= b[3] and b[2] <= point[2] <= b[5]:
            return i + 1
    return 1


def assign_halls(npads, links, target=16):
    """Groups of connected pads (the game finds routes inside a group, and between groups by the links that cross): about `target` pads each,
    numbered from 0.  The game's own levels have 20 to 45 groups, of 7 to 25 pads on average."""
    nb = [[] for _ in range(npads)]
    for a, b in links:
        nb[a].append(b)
        nb[b].append(a)
    hall = [-1] * npads
    count = 0
    sizes = []
    for seed in range(npads):
        if hall[seed] >= 0:
            continue
        queue = [seed]
        hall[seed] = count
        n = 0
        while queue and n < target:
            cur = queue.pop(0)
            n += 1
            for o in nb[cur]:
                if hall[o] < 0 and n + len(queue) < target:
                    hall[o] = count
                    queue.append(o)
        # (pads put in the queue but not reached are kept: the group is what was claimed)
        sizes.append(sum(1 for h in hall if h == count))
        count += 1
    # small groups join the neighbouring group they are linked to
    changed = True
    while changed:
        changed = False
        sz = {}
        for h in hall:
            sz[h] = sz.get(h, 0) + 1
        for g in [g for g, n in sz.items() if n < 4]:
            members = [i for i in range(npads) if hall[i] == g]
            near = {}
            for i in members:
                for o in nb[i]:
                    if hall[o] != g:
                        near[hall[o]] = near.get(hall[o], 0) + 1
            if near:
                tgt = max(near, key=lambda k: (near[k] > 0, -sz[k]))
                for i in members:
                    hall[i] = tgt
                changed = True
                break
    # numbered from 0 without gaps
    remap = {g: i for i, g in enumerate(sorted(set(hall)))}
    return [remap[h] for h in hall]


def write_pads(pads, links, boxes, extra):
    out = struct.pack('<3I', 1, len(pads), len(links))
    halls = assign_halls(len(pads), links)
    for i, (x, y, z, *_r) in enumerate(pads):
        # (the u16 at +8 is the pad's hall: a group of pads the bots' routes are worked out by; one hall for the whole level is what a level
        # without corridors has.  The u16 after it is the room, which the game sets itself)
        out += struct.pack('<iIHHffff', 1001 + i, 1, halls[i], 0, x, y + 0.01, z, 0.0)
    for a, b in links:
        out += struct.pack('<IiiII', 1, 1001 + a, 1001 + b, 0, 0)
    return out


def pick_spread(pads, count, taken=()):
    """Pads far from each other (and from `taken`)."""
    if not pads:
        return []
    chosen = list(taken)
    out = []
    pts = np.array([[p[0], p[2]] for p in pads])
    if not chosen:
        chosen = [int(np.argmin(pts[:, 0]))]
        out.append(chosen[0])
    while len(out) < count and len(out) + len(taken) < len(pads):
        dist = np.min(np.linalg.norm(pts[:, None, :] - pts[chosen][None, :, :], axis=2), axis=1)
        nxt = int(np.argmax(dist))
        if dist[nxt] <= 0:
            break
        chosen.append(nxt)
        out.append(nxt)
    return out


def nearest_pad(pads, point):
    best, bi = 1e30, 0
    for i, p in enumerate(pads):
        d = (p[0] - point[0]) ** 2 + (p[1] - point[1]) ** 2 + (p[2] - point[2]) ** 2
        if d < best:
            best, bi = d, i
    return bi


# ---------------------------------------------------------------- rooms and portals

def clip_poly(poly, axis, value, keep_below):
    """The part of a convex polygon on one side of the plane `axis` = `value`, as polygons of 3 or 4 points (None when nothing is left)."""
    pts = list(zip(poly.p, poly.uv, poly.col))
    out = []
    n = len(pts)

    def inside(q):
        return q[0][axis] <= value if keep_below else q[0][axis] >= value
    for i in range(n):
        a, b = pts[i], pts[(i + 1) % n]
        ia, ib = inside(a), inside(b)
        if ia:
            out.append(a)
        if ia != ib:
            t = (value - a[0][axis]) / (b[0][axis] - a[0][axis])
            out.append((a[0] + (b[0] - a[0]) * t, (a[1][0] + (b[1][0] - a[1][0]) * t, a[1][1] + (b[1][1] - a[1][1]) * t),
                        tuple(a[2][k] + (b[2][k] - a[2][k]) * t for k in range(3))))
    if len(out) < 3:
        return []
    if len(out) <= 4:
        return [Poly([o[0] for o in out], [o[1] for o in out], [o[2] for o in out], poly.mat)]
    return [Poly([out[0][0], out[i][0], out[i + 1][0]], [out[0][1], out[i][1], out[i + 1][1]], [out[0][2], out[i][2], out[i + 1][2]], poly.mat)
            for i in range(1, len(out) - 1)]


def cut_to_grid(polys, lo, r):
    """Cut the polygons along the lines of the grid of rooms, so that each lies in one room."""
    for axis in (0, 2):
        out = []
        for p in polys:
            todo = [p]
            lo_v = min(v[axis] for v in p.p)
            hi_v = max(v[axis] for v in p.p)
            k = math.floor((lo_v - lo[axis]) / r) + 1
            while lo[axis] + k * r < hi_v - 1e-6:
                line = lo[axis] + k * r
                nxt = []
                for q in todo:
                    if min(v[axis] for v in q.p) < line - 1e-6 and max(v[axis] for v in q.p) > line + 1e-6:
                        nxt += clip_poly(q, axis, line, True) + clip_poly(q, axis, line, False)
                    else:
                        nxt.append(q)
                todo = nxt
                k += 1
            out += todo
        polys = out
    return [p for p in polys if np.linalg.norm(p.normal()) > 1e-6]


def split_rooms(polys, args):
    allp = np.array([v for p in polys for v in p.p])
    lo, hi = allp.min(axis=0), allp.max(axis=0)
    ylo, yhi = lo[1] - 2.0, hi[1] + 10.0
    if args.room_size <= 0:
        return [polys], [(lo[0] - 1, ylo, lo[2] - 1, hi[0] + 1, yhi, hi[2] + 1)], []
    r = args.room_size
    polys = cut_to_grid(polys, lo, r)
    cells = {}
    for p in polys:
        c = p.centre()
        cells.setdefault((int((c[0] - lo[0]) // r), int((c[2] - lo[2]) // r)), []).append(p)
    keys = sorted(cells)
    rooms = [cells[k] for k in keys]
    boxes = [(lo[0] + k[0] * r, ylo, lo[2] + k[1] * r, lo[0] + (k[0] + 1) * r, yhi, lo[2] + (k[1] + 1) * r) for k in keys]
    number = {k: i + 1 for i, k in enumerate(keys)}
    portals = []
    for k in keys:
        for dx, dz in ((1, 0), (0, 1)):
            nb = (k[0] + dx, k[1] + dz)
            if nb not in number:
                continue
            a, b = number[k], number[nb]
            bx = boxes[a - 1]
            if dx:
                x = bx[3]
                quad = [(x, ylo, bx[2]), (x, yhi, bx[2]), (x, yhi, bx[5]), (x, ylo, bx[5])]
                nrm = (1.0, 0.0, 0.0)
            else:
                z = bx[5]
                quad = [(bx[0], ylo, z), (bx[3], ylo, z), (bx[3], yhi, z), (bx[0], yhi, z)]
                nrm = (0.0, 0.0, 1.0)
            portals.append((a, b, nrm, quad))
    return rooms, boxes, portals


# ---------------------------------------------------------------- main

def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('input')
    ap.add_argument('--out', required=True)
    ap.add_argument('--number', type=int, required=True)
    ap.add_argument('--name', required=True)
    ap.add_argument('--like', type=int, default=25)
    ap.add_argument('--scale', type=float, default=None)
    ap.add_argument('--up', choices=('y', 'z'), default=None)
    ap.add_argument('--flip', action='store_true')
    ap.add_argument('--room-size', type=float, default=0.0)
    ap.add_argument('--pad-spacing', type=float, default=5.0)
    ap.add_argument('--max-pads', type=int, default=450)
    ap.add_argument('--step', type=float, default=1.0, help='the highest step a link may climb')
    ap.add_argument('--modes', default='dm,bagtag,ctb,ko')
    ap.add_argument('--tune', type=int, default=0)
    ap.add_argument('--max-textures', type=int, default=90)
    ap.add_argument('--materials', action='append', default=[])
    ap.add_argument('--wad', action='append', default=[])
    ap.add_argument('--blender', default=None)
    ap.add_argument('--tex-base', type=int, default=0)
    ap.add_argument('--radius', type=float, default=0.0, help='keep only the part of the level within this distance (game units) of --center')
    ap.add_argument('--center', default=None, help='x,z of the middle of the part kept by --radius (default: the first spawn, else the middle of the level)')
    ap.add_argument('--spawns', type=int, default=16)
    ap.add_argument('--weapons', type=int, default=3, help='how many of each weapon slot are laid out without markers')
    args = ap.parse_args()
    if not 31 <= args.number <= 99:
        sys.exit('--number has to be 31 to 99')
    scene = lvimport.load(args.input, {'materials': args.materials, 'wad': args.wad, 'blender': args.blender})
    polys, markers = build_polys(scene, args)
    if not polys:
        sys.exit('no faces')
    print(f'{scene.label}: {len(polys)} polygons, {len(scene.mats)} materials, {len(markers)} markers')
    if args.radius > 0:
        if args.center:
            cx, cz = (float(x) for x in args.center.split(','))
        else:
            sp = [v for k, v in markers if k.lower().startswith('spawn')]
            allp = np.array([q for p in polys for q in p.p])
            cx, cz = (sp[0][0], sp[0][2]) if sp else ((allp[:, 0].min() + allp[:, 0].max()) / 2, (allp[:, 2].min() + allp[:, 2].max()) / 2)
        # (a polygon that reaches into the square is cut to it: a big ground face has its middle far away)
        R = args.radius
        cut = []
        for p in polys:
            xs = [q[0] for q in p.p]
            zs = [q[2] for q in p.p]
            if max(xs) < cx - R or min(xs) > cx + R or max(zs) < cz - R or min(zs) > cz + R:
                continue
            parts = [p]
            for axis, value, below in ((0, cx + R, True), (0, cx - R, False), (2, cz + R, True), (2, cz - R, False)):
                nxt = []
                for q in parts:
                    nxt += clip_poly(q, axis, value, below)
                parts = nxt
            cut += [q for q in parts if np.linalg.norm(q.normal()) > 0.5]
        polys = cut
        markers = [(k, v) for k, v in markers if abs(v[0] - cx) <= args.radius and abs(v[2] - cz) <= args.radius]
        print(f'kept {len(polys)} polygons within {args.radius} (a square) of ({cx:.1f}, {cz:.1f})')
        if not polys:
            sys.exit('nothing is left')
    reduce_materials(polys, scene, args.max_textures)
    base = args.tex_base or 4000 + 100 * (args.number - 31)
    out = args.out
    os.makedirs(os.path.join(out, 'BG.PAK', 'bg', f'level{args.number}'), exist_ok=True)
    os.makedirs(os.path.join(out, 'MISC.PAK', 'pad', 'data'), exist_ok=True)
    os.makedirs(os.path.join(out, 'MISC.PAK', 'propsetup', 'data'), exist_ok=True)
    os.makedirs(os.path.join(out, 'MISC.PAK', 'textures'), exist_ok=True)
    # textures
    names = []
    for p in polys:
        if p.mat not in names:
            names.append(p.mat)
    tex_numbers = {}
    for k, m in enumerate(names):
        mat = scene.mats[m]
        img = material_image(mat)
        num = base + k
        tex_numbers[m] = (num, bool(mat.translucent))
        open(os.path.join(out, 'MISC.PAK', 'textures', f'{num:04d}.qpm'), 'wb').write(png2qpm.make_qpm(fit_texture(img)))
    rooms, boxes, portals = split_rooms(polys, args)
    # (the game has room for about 200 rooms: the grid is made coarser until there are fewer)
    while len(rooms) > MAX_ROOMS and args.room_size > 0:
        args.room_size *= 1.25
        print(f'{len(rooms)} rooms are too many: room size {args.room_size:.1f}')
        rooms, boxes, portals = split_rooms(polys, args)
    bg = write_bg(polys, tex_numbers, args, rooms, boxes, portals)
    open(os.path.join(out, 'BG.PAK', 'bg', f'level{args.number}', f'level{args.number}.raw'), 'wb').write(bg)
    pads, links = make_pads([p for r in rooms for p in r], boxes, args, markers)
    if not pads:
        sys.exit('no floors: nothing for bots and players to stand on (faces that point up)')
    open(os.path.join(out, 'MISC.PAK', 'pad', 'data', f'level{args.number}.raw'), 'wb').write(write_pads(pads, links, boxes, markers))
    open(os.path.join(out, 'MISC.PAK', 'propsetup', 'data', f'level{args.number}.raw'), 'wb').write(struct.pack('<2I', 1, 0))
    # the set-up: spawns, weapons, pickups
    setup = ['modes ' + args.modes]
    spawn_pads = sorted({nearest_pad(pads, v) for k, v in markers if k.lower().startswith('spawn')})
    if not spawn_pads:
        spawn_pads = pick_spread(pads, args.spawns)
    spawn_pads = spawn_pads[:64]
    for i in spawn_pads:
        setup.append(f'spawn {1001 + i}')
    weapons = []
    for k, v in markers:
        m = re.match(r'weapon(\d)', k.lower())
        if m and 1 <= int(m.group(1)) <= 5:
            weapons.append((int(m.group(1)), nearest_pad(pads, v)))
    if weapons:
        for slot, i in weapons[:100]:
            setup.append(f'weapon {slot} {1001 + i}')
    else:
        taken = list(spawn_pads)
        for slot in range(1, 6):
            for i in pick_spread(pads, args.weapons, taken):
                taken.append(i)
                setup.append(f'weapon {slot} {1001 + i}')
    for k, v in markers:
        m = re.match(r'pickup(-?\d+)', k.lower())
        if m:
            setup.append(f'pickup {m.group(1)} {1001 + nearest_pad(pads, v)}')
    open(os.path.join(out, 'setup.txt'), 'w').write('\n'.join(setup) + '\n')
    line = f'level {args.number} "{args.name}" like {args.like}' + (f' tune {args.tune}' if args.tune else '') + f' modes {args.modes} setup setup.txt\n'
    open(os.path.join(out, 'levels.txt'), 'w').write(line)
    open(os.path.join(out, 'mod.ini'), 'w').write('enabled = true\n')
    print(f'{len(polys)} polygons, {len(rooms)} room(s), {len(portals)} portal(s), {len(pads)} pads, {len(links)} links, {len(names)} textures from {base}')


if __name__ == '__main__':
    main()
