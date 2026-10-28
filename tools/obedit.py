"""Make a changed copy of an object (model) file of OB.PAK, as a mod's file.

    python tools/obedit.py <OB.PAK> <name in the pack> <mods folder>/<mod> scale <factor>

    python tools/obedit.py <OB.PAK> <name in the pack> <mods folder>/<mod> box [part [size]]

    python tools/obedit.py <OB.PAK> <name> <mods folder>/<mod> mesh <part> <mesh.obj>

with `mesh` the geometry of a part is replaced by the triangles of an OBJ (in the file's own space, which `obdump.py --obj` writes), packed into chunks; it has to fit the room the part has.
With `box` the geometry of the model's part number `part` (the first by default) is replaced by a box made from nothing (a test that strips, positions, coordinates, colours and normals written by a tool are taken: the head of a
character becomes a box).  With `scale` it writes <mod>/OB.PAK/<name> with every position of the model multiplied by <factor> (a test that the game takes a model file made by a tool: the joints of the animations do not
scale with it, so a character is a little out of joint).  The pack is not changed; the mod loader puts the file in its place when the game starts.
"""
import os
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import obdump  # noqa: E402
import pakls  # noqa: E402


def scale_positions(f, factor):
    f = bytearray(f)
    tex, tab = struct.unpack_from('<2I', f, 0)
    n = struct.unpack_from('<I', f, tab)[0]
    bones = tab - n * 80
    done = set()
    for i in range(n):
        b = bones + 80 * i
        for list_at, arrays in ((struct.unpack_from('<I', f, b + 8)[0], b + 20), (struct.unpack_from('<I', f, b + 12)[0], b + 44)):
            if not obdump.ok(f, list_at):
                continue
            pos_at = struct.unpack_from('<I', f, arrays + 4)[0]
            for chunk in obdump.read_chunks(f, list_at):
                first, count = chunk[3], chunk[4]
                for k in range(first, first + count):
                    if (pos_at, k) in done:
                        continue
                    done.add((pos_at, k))
                    x, y, z = struct.unpack_from('<3f', f, pos_at + 12 * k)
                    struct.pack_into('<3f', f, pos_at + 12 * k, x * factor, y * factor, z * factor)
    return bytes(f), len(done)


def box_part(f, scale=0.8, part=0):
    """Replace the geometry of the first part of a model by a box (6 quads, 24 vertices) made from nothing: strips, positions, coordinates, colours and normals are written into
    the arrays the part already has, and its list of chunks becomes one chunk.  The box is as big as the part's own bounding box, times `scale`, with the texture of the first chunk."""
    f = bytearray(f)
    tex, tab = struct.unpack_from('<2I', f, 0)
    n = struct.unpack_from('<I', f, tab)[0]
    b = tab - n * 80 + 80 * part
    list_at = struct.unpack_from('<I', f, b + 8)[0]
    strips_at, pos_at, st_at, col_at, _bind, nrm_at = struct.unpack_from('<6I', f, b + 20)
    chunks = obdump.read_chunks(f, list_at)
    pts = [struct.unpack_from('<3f', f, pos_at + 12 * k) for c in chunks for k in range(c[3], c[3] + c[4])]
    lo = [min(p[i] for p in pts) for i in range(3)]
    hi = [max(p[i] for p in pts) for i in range(3)]
    c = [(lo[i] + hi[i]) / 2 for i in range(3)]
    h = [(hi[i] - lo[i]) / 2 * scale for i in range(3)]
    slot = chunks[0][0]
    # the six faces: a normal and four corners going round counter-clockwise seen from outside
    faces = []
    for axis in range(3):
        for sign in (1, -1):
            u, v = [(a, b_) for a, b_ in ((1, 2), (2, 0), (0, 1))][axis]
            nrm = [0.0, 0.0, 0.0]
            nrm[axis] = float(sign)
            corners = []
            for du, dv in ((-1, -1), (1, -1), (1, 1), (-1, 1)):
                p = list(c)
                p[axis] += sign * h[axis]
                p[u] += du * h[u]
                p[v] += dv * h[v]
                corners.append((p, (du + 1) / 2, (dv + 1) / 2))
            if sign < 0:
                corners.reverse()
            faces.append((nrm, corners))
    verts = 0
    for k, (nrm, q) in enumerate(faces):
        # a quad a, b, c, d going round is the strip a, b, d, c
        order = [q[0], q[1], q[3], q[2]]
        struct.pack_into('<4I', f, strips_at + 16 * k, 4 | (0x8000 if k == len(faces) - 1 else 0), 0x302e4000, 0x412, 0)
        for p, s_, t_ in order:
            struct.pack_into('<3f', f, pos_at + 12 * verts, *p)
            struct.pack_into('<4f', f, st_at + 16 * verts, s_, t_, 1.0, 1.0)
            struct.pack_into('<4B', f, col_at + 4 * verts, 0x7f, 0x7f, 0x7f, 0x7f)
            if nrm_at:
                struct.pack_into('<3f', f, nrm_at + 12 * verts, *nrm)
            verts += 1
    struct.pack_into('<6i', f, list_at, slot, 0, len(faces), 0, verts, chunks[0][5])
    if len(chunks) > 1:
        struct.pack_into('<i', f, list_at + 24 + 20, -1)
    struct.pack_into('<H', f, b + 70, 12)
    return bytes(f)


def read_obj(path):
    """The faces of a Wavefront OBJ: lists of three or four corners (position, (s, t), normal or None); faces with more corners are fanned into triangles."""
    v, vt, vn, tris = [], [], [], []
    for line in open(path):
        w = line.split()
        if not w:
            continue
        if w[0] == 'v':
            v.append(tuple(float(x) for x in w[1:4]))
        elif w[0] == 'vt':
            vt.append((float(w[1]), float(w[2]) if len(w) > 2 else 0.0))
        elif w[0] == 'vn':
            vn.append(tuple(float(x) for x in w[1:4]))
        elif w[0] == 'f':
            corners = []
            for c in w[1:]:
                i = [int(x) if x else 0 for x in (c.split('/') + ['', ''])[:3]]
                pos = v[i[0] - 1 if i[0] > 0 else i[0]]
                st = vt[i[1] - 1 if i[1] > 0 else i[1]] if i[1] else (0.0, 0.0)
                nrm = vn[i[2] - 1 if i[2] > 0 else i[2]] if i[2] else None
                corners.append((pos, st, nrm))
            if len(corners) <= 4:
                tris.append(corners)
            else:
                for k in range(1, len(corners) - 1):
                    tris.append([corners[0], corners[k], corners[k + 1]])
    return tris


def mesh_part(f, tris, part=0):
    """Replace the geometry of a part by the triangles `tris`: each triangle is a strip of three vertices, the chunks hold as many as the vector unit's memory allows (at most 64 vertices,
    24 strips and `3 * vertices + strips` of 212), the texture is that of the part's largest chunk.  The new geometry has to fit in the arrays the part has: the vertex arrays of the
    original and its strip table.  Normals are the triangle's own when the OBJ gives none."""
    f = bytearray(f)
    tex, tab = struct.unpack_from('<2I', f, 0)
    n = struct.unpack_from('<I', f, tab)[0]
    b = tab - n * 80 + 80 * part
    list_at = struct.unpack_from('<I', f, b + 8)[0]
    strips_at, pos_at, st_at, col_at, _bind, nrm_at = struct.unpack_from('<6I', f, b + 20)
    chunks = obdump.read_chunks(f, list_at)
    kind = chunks[0][5]
    slot = max(chunks, key=lambda c: c[4])[0]
    vertex_room = max(c[3] + c[4] for c in chunks)
    strip_room = max(c[1] + c[2] for c in chunks)
    chunk_room = len(chunks)
    # pack the faces into chunks (a triangle is a strip of three vertices, a quad a, b, c, d is the strip a, b, d, c)
    groups, cur, cv = [], [], 0
    for t in tris:
        nv = cv + len(t)
        if nv > 64 or len(cur) + 1 > 24 or 3 * nv + len(cur) + 1 > 212:
            groups.append(cur)
            cur, cv, nv = [], 0, len(t)
        cur.append(t)
        cv = nv
    if cur:
        groups.append(cur)
    # (a chunk's first vertex is a multiple of 4)
    first, need = [], 0
    for g in groups:
        first.append(need)
        need += (sum(len(t) for t in g) + 3) & ~3
    if need > vertex_room or len(tris) > strip_room or len(groups) > chunk_room:
        raise SystemExit(f'the mesh does not fit the part: it needs {need} vertices, {len(tris)} strips and {len(groups)} chunks; the part has room for {vertex_room}, {strip_room} and {chunk_room}')
    strip = 0
    for gi, g in enumerate(groups):
        v0 = first[gi]
        vi = v0
        for ti, t in enumerate(g):
            last = ti == len(g) - 1
            struct.pack_into('<4I', f, strips_at + 16 * (strip + ti), len(t) | (0x8000 if last else 0), 0x302e4000, 0x412, 0)
            a, bb, c = (t[0][0], t[1][0], t[2][0])
            fn = [(bb[1] - a[1]) * (c[2] - a[2]) - (bb[2] - a[2]) * (c[1] - a[1]), (bb[2] - a[2]) * (c[0] - a[0]) - (bb[0] - a[0]) * (c[2] - a[2]), (bb[0] - a[0]) * (c[1] - a[1]) - (bb[1] - a[1]) * (c[0] - a[0])]
            ln = sum(x * x for x in fn) ** 0.5 or 1.0
            fn = [x / ln for x in fn]
            corners = t if len(t) == 3 else [t[0], t[1], t[3], t[2]]
            for pos, st, nrm in corners:
                struct.pack_into('<3f', f, pos_at + 12 * vi, *pos)
                struct.pack_into('<4f', f, st_at + 16 * vi, st[0], st[1], 1.0, 1.0)
                struct.pack_into('<4B', f, col_at + 4 * vi, 0x7f, 0x7f, 0x7f, 0x7f)
                if nrm_at:
                    struct.pack_into('<3f', f, nrm_at + 12 * vi, *(nrm or fn))
                vi += 1
        struct.pack_into('<6i', f, list_at + 24 * gi, slot, strip, len(g), v0, vi - v0, kind)
        strip += len(g)
    # the list ends with a record of a negative kind (the room for it is the original's)
    end = list_at + 24 * len(groups)
    struct.pack_into('<6i', f, end, 0, 0, 0, 0, 0, -1)
    struct.pack_into('<H', f, b + 70, min(sum(len(t) - 2 for t in tris), 65535))
    return bytes(f), len(groups)


def main():
    if len(sys.argv) >= 7 and sys.argv[4] == 'mesh':
        d, es = pakls.entries(sys.argv[1])
        hit = [(n, o, s) for n, o, s in es if n == sys.argv[2]]
        if not hit:
            sys.exit('no such object: ' + sys.argv[2])
        n, o, s = hit[0]
        data, groups = mesh_part(d[o:o + s], read_obj(sys.argv[6]), int(sys.argv[5]))
        out = os.path.join(sys.argv[3], 'OB.PAK', n)
        os.makedirs(os.path.dirname(out), exist_ok=True)
        open(out, 'wb').write(data)
        open(os.path.join(sys.argv[3], 'mod.ini'), 'w').write('enabled = true\n')
        print(f'mesh written to {out} ({groups} chunks)')
        return
    if len(sys.argv) >= 5 and sys.argv[4] == 'box':
        d, es = pakls.entries(sys.argv[1])
        hit = [(n, o, s) for n, o, s in es if n == sys.argv[2]]
        if not hit:
            sys.exit('no such object: ' + sys.argv[2])
        n, o, s = hit[0]
        out = os.path.join(sys.argv[3], 'OB.PAK', n)
        os.makedirs(os.path.dirname(out), exist_ok=True)
        open(out, 'wb').write(box_part(d[o:o + s], float(sys.argv[6]) if len(sys.argv) > 6 else 0.8, int(sys.argv[5]) if len(sys.argv) > 5 else 0))
        open(os.path.join(sys.argv[3], 'mod.ini'), 'w').write('enabled = true\n')
        print('box written to', out)
        return
    if len(sys.argv) < 6 or sys.argv[4] != 'scale':
        sys.exit(__doc__)
    d, es = pakls.entries(sys.argv[1])
    hits = [(n, o, s) for n, o, s in es if n == sys.argv[2]]
    if not hits:
        sys.exit('no such object: ' + sys.argv[2])
    n, o, s = hits[0]
    new, count = scale_positions(d[o:o + s], float(sys.argv[5]))
    out = os.path.join(sys.argv[3], 'OB.PAK', n)
    os.makedirs(os.path.dirname(out), exist_ok=True)
    open(out, 'wb').write(new)
    open(os.path.join(sys.argv[3], 'mod.ini'), 'w').write('enabled = true\n')
    print(f'{count} positions scaled; written to {out}')


if __name__ == '__main__':
    main()
