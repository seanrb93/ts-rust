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
    --pad-spacing D  distance of the pads bots walk by (default 5); --max-pads N makes it larger when there would be more pads (default 700)
    --modes LIST     the modes the level offers: dm,bagtag,ctb,escort,ko,ls (default dm,bagtag,ctb,ko)
    --tune N         the music track 1..27 of the game's (default: the level `like` plays)
    --music FILE     music of its own for the level: any audio file (needs ffmpeg unless it is .opus); it is made Opus and the level plays it
    --min-light X    how bright the darkest surfaces are, 0..1 (default 0.35 for maps of Source and GoldSrc, whose lightmaps are only averaged here; else 0)
    --max-polygon W  polygons wider than W units are cut into pieces (default 0: not); makes the file larger
    --texture-budget KB  the textures are made smaller until their files add up to this (default 640; the game's own levels use up to 670)
    --max-size MB    about how large the level's BG file may be (default 16; the game's own largest is 2.7): the smallest surfaces that are not floors are left out above it
    --light-detail N Source maps: faces whose light varies much are cut every N luxels of their lightmap (16 luxels is 6.4 game units; default 16; 0: one brightness
                     for each face) and every vertex gets the light at its place
    --no-entities    maps: leave out the brush entities that are only a shape (func_brush, func_wall, func_breakable, ...), which are part of the level otherwise
    --doors closed   maps: make the doors part of the level, closed (they are left out by default)
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
MAX_ROOM_CHUNKS = 96      # (the game's largest room has 104 chunks and 5246 vertices)
MAX_ROOM_VERTS = 4800
MAX_ROOMS = 480          # the tables hold 512 rooms (the game's own held 200)
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


def plan_sizes(images, areas, budget):
    """The size (w, h: powers of two) each texture is made: the pictures' own sizes (at most 256 a side), made smaller, the ones that cover least
    for their bytes first, until the files add up to `budget` bytes.  The game keeps a level's textures in a pool of its own: the levels of the game
    use at most 670 KB, and a level with more has surfaces that are not drawn."""
    size = {}
    for m, img in images.items():
        w, h = img.size
        nw, nh = pow2_at_most(w), pow2_at_most(h)
        while nw * nh > 65536:
            if nw >= nh:
                nw //= 2
            else:
                nh //= 2
        size[m] = (nw, nh)
    cost = lambda wh: 1024 + 40 + wh[0] * wh[1]
    while sum(cost(v) for v in size.values()) > budget:
        # (the texture that costs most for the area it covers)
        cand = [m for m, v in size.items() if v[0] * v[1] > 256]
        if not cand:
            break
        m = max(cand, key=lambda k: size[k][0] * size[k][1] / (1.0 + areas.get(k, 0.0)))
        w, h = size[m]
        size[m] = (w // 2, h) if w >= h and w > 8 else (w, max(8, h // 2))
    return size


def fit_texture(img, target=None):
    """The picture as the game takes it: power-of-two sides, 65536 pixels at the most."""
    w, h = img.size
    nw, nh = target if target else (pow2_at_most(w), pow2_at_most(h))
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
    m = args.min_light if args.min_light is not None else scene.min_light
    if m > 0:
        for p in polys:
            p.col = [tuple(m + (1.0 - m) * min(1.0, max(0.0, c)) for c in cc) for cc in p.col]
    markers = [(name, conv(pos)) for name, pos in scene.markers]
    coll = None
    if scene.collision is not None:
        coll = []
        for pts in scene.collision:
            q = [conv(x) for x in pts]
            if args.flip:
                q.reverse()
            coll.append(Poly(q, [(0.0, 0.0)] * len(q), [(1.0, 1.0, 1.0)] * len(q), ''))
        coll = [p for p in coll if np.linalg.norm(p.normal()) > 0.5]
    build_polys.coll = coll
    return [p for p in polys if np.linalg.norm(p.normal()) > 0.5], markers


def poly_area(p):
    a = 0.0
    for i in range(1, len(p.p) - 1):
        a += np.linalg.norm(np.cross(p.p[i] - p.p[0], p.p[i + 1] - p.p[0])) / 2
    return a


def shrink_to(polys, max_bytes):
    """Leave out the smallest polygons that are not floors (railings, trim, bits of walls) until the BG file is about `max_bytes` (an estimate).  The
    game keeps the level's file in a pool of memory it also needs for textures, characters and the like; with a file of more than about 3.5 MB (the
    largest of the game's own is 2.7) surfaces further on are not drawn, whatever the level looks like."""
    def estimate(ps):
        return sum(40 * len(p.p) + 66 for p in ps)
    if estimate(polys) <= max_bytes:
        return polys
    droppable = sorted((p for p in polys if p.normal()[1] <= FLOOR_NY), key=poly_area)
    keep = set(id(p) for p in polys)
    size = estimate(polys)
    dropped = 0
    for p in droppable:
        if size <= max_bytes:
            break
        keep.discard(id(p))
        size -= 40 * len(p.p) + 66
        dropped += 1
    print(f'{dropped} small polygons that are not floors were left out to keep the level near {max_bytes / 1e6:.1f} MB (about {size / 1e6:.1f} MB now)')
    return [p for p in polys if id(p) in keep]


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


def write_bg(polys, tex_numbers, args, rooms_polys, room_boxes, portals, rooms_coll=None):
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
        floors, walls = collision(rooms_coll[ri] if rooms_coll is not None else rp)
        # floors
        o.align(4)
        f_ptrs = []
        for f in floors:
            f_ptrs.append(o.put(struct.pack('<IHH', len(f), 0, 0) + b''.join(struct.pack('<3f', *v) for v in f)))
        f_at = o.put(b''.join(struct.pack('<I', p) for p in f_ptrs) + struct.pack('<I', 0))
        # the game works out a room's box from its floors and walls, and clips what it sees of the room with that: the box has to hold all of
        # what is drawn of it.  Two walls a thousandth of a unit long at the corners of what is drawn do that without being in the way.
        if rp:
            pts = np.array([q for p in rp for q in p.p])
            lo_, hi_ = pts.min(axis=0), pts.max(axis=0)
            walls = list(walls) + [(lo_[0], lo_[1], lo_[2], hi_[1], lo_[0] + 0.001, lo_[1], lo_[2], hi_[1]),
                                   (hi_[0], lo_[1], hi_[2], hi_[1], hi_[0] + 0.001, lo_[1], hi_[2], hi_[1])]
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


def make_strips(polys, longest=40):
    """Triangle strips for the polygons of one material: each triangle is a strip of three vertices and a quad one of four (a, b, d, c); quads that
    share an edge (with the same texture coordinates and colours at its ends: pieces of one cut polygon do) are joined into one long strip, which
    holds each vertex once instead of two or four times."""
    def key(pt, uv, col):
        return (round(float(pt[0]), 5), round(float(pt[1]), 5), round(float(pt[2]), 5), round(uv[0], 5), round(uv[1], 5),
                round(col[0], 3), round(col[1], 3), round(col[2], 3))
    quads = [p for p in polys if len(p.p) == 4]
    out = [[(p.p[i], p.uv[i], p.col[i]) for i in (0, 1, 2)] for p in polys if len(p.p) == 3]
    # for every quad and rotation k: the strip order (k, k+1, k+3, k+2); its first edge is (k, k+1)
    first = {}
    for qi, p in enumerate(quads):
        for k in range(4):
            first.setdefault((key(p.p[k], p.uv[k], p.col[k]), key(p.p[(k + 1) % 4], p.uv[(k + 1) % 4], p.col[(k + 1) % 4])), []).append((qi, k))
    used = [False] * len(quads)
    for qi, p in enumerate(quads):
        if used[qi]:
            continue
        used[qi] = True
        idx = [0, 1, 3, 2]
        strip = [(p.p[i], p.uv[i], p.col[i]) for i in idx]
        while len(strip) + 2 <= longest:
            a, b = strip[-2], strip[-1]
            nxt = None
            for cj, k in first.get((key(*a), key(*b)), []):
                if not used[cj]:
                    nxt = (cj, k)
                    break
            if nxt is None:
                break
            cj, k = nxt
            used[cj] = True
            q = quads[cj]
            for i in ((k + 3) % 4, (k + 2) % 4):
                strip.append((q.p[i], q.uv[i], q.col[i]))
        out.append(strip)
    return out


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
            for strip in make_strips(ps):
                if nv + len(strip) > CHUNK_VERTS:
                    flush()
                cur.append(strip)
                nv += len(strip)
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


def clean_floor(pts):
    """A floor's vertices without repeated or collinear ones, starting at the three that make the largest triangle: the game works out the
    height of a sloping floor from its first three vertices, so these must not lie on a line (clipping at the rooms' edges makes such vertices)."""
    v = [np.asarray(p, dtype=np.float64) for p in pts]
    changed = True
    while changed and len(v) >= 3:
        changed = False
        for i in range(len(v)):
            a, b, c = v[i - 1], v[i], v[(i + 1) % len(v)]
            if np.linalg.norm(b - a) < 1e-5 or np.linalg.norm(np.cross(b - a, c - b)) < 1e-6 * max(1.0, np.linalg.norm(c - a)):
                del v[i]
                changed = True
                break
    if len(v) < 3:
        return None
    best, at = -1.0, 0
    for i in range(len(v)):
        t = np.linalg.norm(np.cross(v[(i + 1) % len(v)] - v[i], v[(i + 2) % len(v)] - v[i]))
        if t > best:
            best, at = t, i
    if best < 1e-4:
        return None
    v = v[at:] + v[:at]
    return [tuple(float(c) for c in q) for q in v]


def collision(polys):
    """Floors (lists of vertices) and walls (8 floats) of the polygons of a room."""
    floors, walls = [], []
    for p in polys:
        n = p.normal()
        if n[1] > FLOOR_NY:
            f = clean_floor(p.p)
            if f:
                floors.append(f)
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
        pads, links = _pads(polys, args, d, markers)
        if len(pads) <= args.max_pads:
            break
        d *= max(1.1, (len(pads) / args.max_pads) ** 0.5)
    return pads, links


def _pads(polys, args, d, markers=()):
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

    def floor_at(x, z):
        """(height, distance to the nearest edge of the floor triangle) of every floor triangle that covers a point."""
        gx, gz = int((x - lo[0]) / d), int((z - lo[2]) / d)
        out = []
        for k in grid.get((gx, gz), []):
            a, b, c = tris[k]
            den = (b[2] - c[2]) * (a[0] - c[0]) + (c[0] - b[0]) * (a[2] - c[2])
            if abs(den) < 1e-9:
                continue
            l1 = ((b[2] - c[2]) * (x - c[0]) + (c[0] - b[0]) * (z - c[2])) / den
            l2 = ((c[2] - a[2]) * (x - c[0]) + (a[0] - c[0]) * (z - c[2])) / den
            l3 = 1 - l1 - l2
            if min(l1, l2, l3) < -1e-6:
                continue
            m = 1e9
            for (p, q) in ((a, b), (b, c), (c, a)):
                ex, ez = q[0] - p[0], q[2] - p[2]
                ln = math.hypot(ex, ez)
                if ln > 1e-9:
                    m = min(m, abs(ex * (z - p[2]) - ez * (x - p[0])) / ln)
            out.append((l1 * a[1] + l2 * b[1] + l3 * c[1], m))
        return out

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
    # a pad where each marker is (on the floor under it), linked to the pads near it: spawns and weapons go to the pad nearest, which on a large map
    # with a wide grid would be far away
    for name, pos in markers:
        gx, gz = int((pos[0] - lo[0]) / d), int((pos[2] - lo[2]) / d)
        hs = [h for h in height_at(pos[0], pos[2], gx, gz) if h <= pos[1] + 0.8]
        if not hs:
            continue
        h = max(hs)
        # (a map's spawns are on round numbers, which are the edges of its floors: the game's test of what floor a point is over is not sure about a point
        # on an edge, and the player falls to the floor below.  The pad is moved to where the floor is under it with some room all round.)
        best = None
        for r in (0.0137, 0.05, 0.1, 0.2, 0.4, 0.8):
            for ang in range(0, 360, 45) if r > 0.0137 else (30,):
                px, pz = pos[0] + r * math.cos(math.radians(ang)), pos[2] + r * math.sin(math.radians(ang))
                cands = [m for (hh, m) in floor_at(px, pz) if abs(hh - h) < 0.05]
                if cands and max(cands) >= 0.03:
                    best = (px, pz)
                    break
            if best:
                break
        if best:
            pos = np.array([best[0], pos[1], best[1]])
        if any((q[0] - pos[0]) ** 2 + (q[2] - pos[2]) ** 2 < 0.09 and abs(q[1] - h) < 0.3 for q in pads):
            continue
        near = sorted((((q[0] - pos[0]) ** 2 + (q[1] - h) ** 2 + (q[2] - pos[2]) ** 2, j) for j, q in enumerate(pads) if abs(q[1] - h) <= args.step and (q[0] - pos[0]) ** 2 + (q[2] - pos[2]) ** 2 <= (2.5 * d) ** 2))[:3]
        pads.append((pos[0], h, pos[2], gx, gz))
        for _, j in near:
            links.append((len(pads) - 1, j))
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


def cut_big(polys, lo, w):
    """Cut the polygons that are wider than `w` (along x or z) at the lines of a grid of `w`; the others are left whole."""
    for axis in (0, 2):
        out = []
        for p in polys:
            lo_v = min(v[axis] for v in p.p)
            hi_v = max(v[axis] for v in p.p)
            if hi_v - lo_v <= w:
                out.append(p)
                continue
            todo = [p]
            k = math.floor((lo_v - lo[axis]) / w) + 1
            while lo[axis] + k * w < hi_v - 1e-6:
                line = lo[axis] + k * w
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


def assign_collision(coll, boxes):
    """The collision polygons of each room: a polygon goes to the room whose box (seen from above) has its centre in it, else to the nearest."""
    rooms = [[] for _ in boxes]
    for p in coll:
        c = p.centre()
        best, bd = 0, 1e30
        for i, bx in enumerate(boxes):
            dx = max(bx[0] - c[0], 0.0, c[0] - bx[3])
            dz = max(bx[2] - c[2], 0.0, c[2] - bx[5])
            d = dx * dx + dz * dz
            if d < bd:
                best, bd = i, d
                if d == 0.0:
                    break
        rooms[best].append(p)
    return rooms


def split_rooms(polys, args, coll=None):
    allp = np.array([v for p in polys for v in p.p])
    lo, hi = allp.min(axis=0), allp.max(axis=0)
    ylo, yhi = lo[1] - 2.0, hi[1] + 10.0
    if args.room_size <= 0:
        return [polys], [(lo[0] - 1, ylo, lo[2] - 1, hi[0] + 1, yhi, hi[2] + 1)], []
    r = args.room_size
    if args.max_polygon <= 0 or args.max_polygon > r:
        # (a polygon is in the room of its centre: the ones that are wider than a room are cut at its edges; with a smaller `--max-polygon` they are cut already)
        polys = cut_to_grid(polys, lo, r)
    cells = {}
    for p in polys:
        c = p.centre()
        cells.setdefault((int((c[0] - lo[0]) // r), int((c[2] - lo[2]) // r)), []).append(p)
    # leaves of rooms: a cell that is too full for the game (about 100 chunks, 5000 vertices: the largest room of the game has 104 and 5246) is
    # split in four, again and again
    leaves = []

    def load(ps):
        by = {}
        for q in ps:
            by.setdefault(q.mat, 0)
            by[q.mat] += len(q.p)
        return sum(-(-v // CHUNK_VERTS) for v in by.values()), sum(by.values())

    def divide(x0, z0, x1, z1, ps):
        chunks, verts = load(ps)
        if (chunks > MAX_ROOM_CHUNKS or verts > MAX_ROOM_VERTS) and x1 - x0 > 6.0 and len(ps) > 1:
            xm, zm = (x0 + x1) / 2, (z0 + z1) / 2
            quad = {}
            for q in ps:
                c = q.centre()
                quad.setdefault((c[0] >= xm, c[2] >= zm), []).append(q)
            for (hx, hz), qs in quad.items():
                divide(xm if hx else x0, zm if hz else z0, x1 if hx else xm, z1 if hz else zm, qs)
        else:
            leaves.append((x0, z0, x1, z1, ps))
    for k in sorted(cells):
        divide(lo[0] + k[0] * r, lo[2] + k[1] * r, lo[0] + (k[0] + 1) * r, lo[2] + (k[1] + 1) * r, cells[k])
    rooms = [l[4] for l in leaves]
    # (each room's box is as tall as its own surfaces, a little more: a portal as tall as the whole level, over a plain, is a thing the game's
    # clipping of what is seen through it does not manage near the camera)
    yr = []
    for rp in rooms:
        ys = [q[1] for p in rp for q in p.p]
        yr.append((min(ys) - 1.0, max(ys) + 4.0))
    boxes = [(l[0], yr[i][0], l[1], l[2], yr[i][1], l[3]) for i, l in enumerate(leaves)]
    # floor heights of each room (bands 3 units apart): a portal is a low opening at a height both rooms have a floor at; one as tall as the whole room
    # (a tower over a plaza) is more than the game's clipping of what is seen through a portal can do near the camera
    def floor_bands(i):
        ps = [p for p in (coll_rooms[i] if coll_rooms is not None else rooms[i]) if p.normal()[1] > FLOOR_NY]
        hs = {}
        for p in ps:
            y = sum(q[1] for q in p.p) / len(p.p)
            hs[round(y / 3.0)] = hs.get(round(y / 3.0), 0.0) + poly_area(p)
        # (the three with most floor: the others, a building's upper floors, are seen from inside it)
        return sorted(k * 3.0 for k, a in sorted(hs.items(), key=lambda kv: -kv[1])[:3] if a > 0.5)
    coll_rooms = assign_collision(coll, [(l[0], 0, l[1], l[2], 0, l[3]) for l in leaves]) if coll is not None else None
    bands = [floor_bands(i) for i in range(len(leaves))]
    portals = []
    eps = 1e-4
    for i in range(len(leaves)):
        for j in range(i + 1, len(leaves)):
            A, B = boxes[i], boxes[j]
            for axis in (0, 2):
                other = 2 - axis
                lo_o, hi_o = max(A[other], B[other]), min(A[3 + other], B[3 + other])
                if hi_o - lo_o <= 0.05:
                    continue
                if abs(A[3 + axis] - B[axis]) < eps:
                    first, second, v = i, j, A[3 + axis]
                elif abs(B[3 + axis] - A[axis]) < eps:
                    first, second, v = j, i, B[3 + axis]
                else:
                    continue
                # the heights: where both rooms have a floor (else a band of the room that has any)
                hts = [h for h in bands[i] if any(abs(h - g) <= 3.0 for g in bands[j])]
                if not hts:
                    y0, y1 = max(A[1], B[1]), min(A[4], B[4])
                    if y1 - y0 < 1.0:
                        continue
                    hts = None
                for h in (hts if hts is not None else [None]):
                    if h is None:
                        yy0, yy1 = y0, min(y1, y0 + 14.0)
                    else:
                        yy0, yy1 = h - 2.0, h + 12.0
                    if axis == 0:
                        quad = [(v, yy0, lo_o), (v, yy1, lo_o), (v, yy1, hi_o), (v, yy0, hi_o)]
                        nrm = (1.0, 0.0, 0.0)
                    else:
                        quad = [(lo_o, yy0, v), (hi_o, yy0, v), (hi_o, yy1, v), (lo_o, yy1, v)]
                        nrm = (0.0, 0.0, 1.0)
                    portals.append((first + 1, second + 1, nrm, quad))
    return rooms, boxes, portals


# ---------------------------------------------------------------- main

def make_music(src, dst):
    """The audio file `src` as an Ogg Opus file `dst` (the port decodes and loops it)."""
    import shutil
    import subprocess
    os.makedirs(os.path.dirname(dst), exist_ok=True)
    if src.lower().endswith('.opus'):
        shutil.copyfile(src, dst)
        return
    exe = shutil.which('ffmpeg')
    if not exe:
        try:
            import imageio_ffmpeg
            exe = imageio_ffmpeg.get_ffmpeg_exe()
        except Exception:
            sys.exit('--music needs ffmpeg (put it on the PATH, or pip install imageio-ffmpeg), or an .opus file')
    subprocess.run([exe, '-y', '-loglevel', 'error', '-i', src, '-vn', '-ac', '2', '-ar', '48000', '-c:a', 'libopus', '-b:a', '112k', '-vbr', 'on', dst], check=True)


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
    ap.add_argument('--max-pads', type=int, default=700)
    ap.add_argument('--step', type=float, default=1.0, help='the highest step a link may climb')
    ap.add_argument('--modes', default='dm,bagtag,ctb,ko')
    ap.add_argument('--tune', type=int, default=0)
    ap.add_argument('--music', default=None, help='an audio file (mp3, ogg, wav, flac, opus, ...) the level plays')
    ap.add_argument('--min-light', type=float, default=None, help='how bright the darkest surfaces are, 0..1 (default 0.35 for maps with lightmaps, else 0)')
    ap.add_argument('--max-polygon', type=float, default=0.0, help='polygons are cut into pieces no wider than this (0: not)')
    ap.add_argument('--spawn-at', default=None, help='x,z (game units): players start only here (a test aid)')
    ap.add_argument('--no-translucent', action='store_true', help='leave out surfaces drawn see-through (a test aid)')
    ap.add_argument('--texture-budget', type=float, default=640.0, help='the most KB of texture files (the video memory of the console has room for about 400 beside what the characters and weapons use)')
    ap.add_argument('--max-size', type=float, default=16.0, help='about how large the BG file may be, in MB')
    ap.add_argument('--no-entities', action='store_true', help='Source maps: leave out the brush entities (func_brush, func_wall, func_breakable, ...) that are only a shape')
    ap.add_argument('--doors', choices=('skip', 'closed'), default='skip', help='Source maps: doors are left out (the default) or made part of the level, closed')
    ap.add_argument('--no-brushes', action='store_true', help='Source maps: make the collision from the drawn faces instead of the brushes')
    ap.add_argument('--light-detail', type=int, default=16, help='Source maps: polygons whose light varies are cut every this many lightmap luxels (0: one brightness for a face)')
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
    scene = lvimport.load(args.input, {'materials': args.materials, 'wad': args.wad, 'blender': args.blender, 'no_brushes': args.no_brushes, 'light_detail': args.light_detail, 'entities': not args.no_entities, 'doors': args.doors})
    polys, markers = build_polys(scene, args)
    coll = build_polys.coll
    if not polys:
        sys.exit('no faces')
    if args.spawn_at:
        sx, sz = (float(x) for x in args.spawn_at.split(','))
        markers = [(k, v) for k, v in markers if not k.lower().startswith('spawn')] + [('spawn', np.array([sx, 2.0, sz]))]
    print(f'{scene.label}: {len(polys)} polygons, {len(scene.mats)} materials, {len(markers)} markers')
    if args.no_translucent:
        polys = [p for p in polys if not scene.mats[p.mat].image() or not scene.mats[p.mat].translucent]
    if args.radius > 0:
        if args.center:
            cx, cz = (float(x) for x in args.center.split(','))
        else:
            sp = [v for k, v in markers if k.lower().startswith('spawn')]
            allp = np.array([q for p in polys for q in p.p])
            cx, cz = (sp[0][0], sp[0][2]) if sp else ((allp[:, 0].min() + allp[:, 0].max()) / 2, (allp[:, 2].min() + allp[:, 2].max()) / 2)
        # (a polygon that reaches into the square is cut to it: a big ground face has its middle far away)
        R = args.radius

        def crop(ps):
            cut = []
            for p in ps:
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
            return cut
        polys = crop(polys)
        if coll is not None:
            coll = crop(coll)
        markers = [(k, v) for k, v in markers if abs(v[0] - cx) <= args.radius and abs(v[2] - cz) <= args.radius]
        print(f'kept {len(polys)} polygons within {args.radius} (a square) of ({cx:.1f}, {cz:.1f})')
        if not polys:
            sys.exit('nothing is left')
    reduce_materials(polys, scene, args.max_textures)
    coll_bytes = 0
    for p in (coll if coll is not None else polys):
        n = p.normal()
        coll_bytes += (24 + 12 * len(p.p)) if n[1] > FLOOR_NY else (44 if abs(n[1]) <= FLOOR_NY else 0)
    if coll is not None:
        print(f'collision: {len(coll)} polygons, about {coll_bytes // 1000} KB')
    polys = shrink_to(polys, int(args.max_size * 1000000) - (coll_bytes if coll is not None else 0))
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
    images = {m: material_image(scene.mats[m]) for m in names}
    areas = {}
    for p in polys:
        a = 0.0
        for i in range(1, len(p.p) - 1):
            a += np.linalg.norm(np.cross(p.p[i] - p.p[0], p.p[i + 1] - p.p[0])) / 2
        areas[p.mat] = areas.get(p.mat, 0.0) + a
    sizes = plan_sizes(images, areas, int(args.texture_budget * 1024))
    total = 0
    for k, m in enumerate(names):
        mat = scene.mats[m]
        num = base + k
        tex_numbers[m] = (num, bool(mat.translucent))
        data = png2qpm.make_qpm(fit_texture(images[m], sizes[m]))
        total += len(data)
        open(os.path.join(out, 'MISC.PAK', 'textures', f'{num:04d}.qpm'), 'wb').write(data)
    print(f'textures: {len(names)}, {total // 1024} KB')
    if args.max_polygon > 0:
        allp = np.array([q for p in polys for q in p.p])
        before = len(polys)
        polys = cut_big(polys, allp.min(axis=0), args.max_polygon)
        print(f'polygons cut to {args.max_polygon} units: {before} -> {len(polys)}')
    rooms, boxes, portals = split_rooms(polys, args, coll)
    # (the game has room for about 200 rooms: the grid is made coarser until there are fewer)
    while len(rooms) > MAX_ROOMS and args.room_size > 0:
        args.room_size *= 1.25
        print(f'{len(rooms)} rooms are too many: room size {args.room_size:.1f}')
        rooms, boxes, portals = split_rooms(polys, args, coll)
    rooms_coll = assign_collision(coll, boxes) if coll is not None else None
    bg = write_bg(polys, tex_numbers, args, rooms, boxes, portals, rooms_coll)
    open(os.path.join(out, 'BG.PAK', 'bg', f'level{args.number}', f'level{args.number}.raw'), 'wb').write(bg)
    pads, links = make_pads([p for r in (rooms_coll if rooms_coll is not None else rooms) for p in r], boxes, args, markers)
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
    if args.music:
        # (the level's tune is a number the game turns into the file MUSIC<number>: a number above the game's 27 tracks is the mod's own)
        args.tune = args.number
        make_music(args.music, os.path.join(out, 'MUSIC', f'{1000 + args.tune}.opus'))
    open(os.path.join(out, 'setup.txt'), 'w').write('\n'.join(setup) + '\n')
    line = f'level {args.number} "{args.name}" like {args.like}' + (f' tune {args.tune}' if args.tune else '') + f' modes {args.modes} setup setup.txt\n'
    open(os.path.join(out, 'levels.txt'), 'w').write(line)
    open(os.path.join(out, 'mod.ini'), 'w').write('enabled = true\n')
    print(f'{len(polys)} polygons, {len(rooms)} room(s), {len(portals)} portal(s), {len(pads)} pads, {len(links)} links, {len(names)} textures from {base}')


if __name__ == '__main__':
    main()
