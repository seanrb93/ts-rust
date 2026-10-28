"""Make a changed copy of an object (model) file of OB.PAK, as a mod's file.

    python tools/obedit.py <OB.PAK> <name in the pack> <mods folder>/<mod> scale <factor>

    python tools/obedit.py <OB.PAK> <name in the pack> <mods folder>/<mod> box [part [size]]

    python tools/obedit.py <OB.PAK> <name> <mods folder>/<mod> mesh <part> <mesh.obj> [--tex-base N]

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


def read_mtl(path):
    """material name -> the picture file of its `map_Kd`, from a Wavefront MTL."""
    out, cur = {}, None
    if not os.path.exists(path):
        return out
    for line in open(path):
        w = line.split(None, 1)
        if len(w) == 2 and w[0] == 'newmtl':
            cur = w[1].strip()
        elif len(w) == 2 and w[0].lower() == 'map_kd' and cur:
            out[cur] = os.path.join(os.path.dirname(path), w[1].strip().split()[-1].replace('\\', '/'))
    return out


def read_obj(path):
    """The faces of a Wavefront OBJ: (corners, material) where corners are three or four (position, (s, t), normal or None) and the material is the name of the last `usemtl`
    (or None); faces with more corners are fanned into triangles.  Also gives the pictures of the materials, from the `mtllib` files."""
    v, vt, vn, tris, mat, pictures = [], [], [], [], None, {}
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
        elif w[0] == 'mtllib':
            pictures.update(read_mtl(os.path.join(os.path.dirname(path), line.split(None, 1)[1].strip())))
        elif w[0] == 'usemtl':
            mat = line.split(None, 1)[1].strip()
        elif w[0] == 'f':
            corners = []
            for c in w[1:]:
                i = [int(x) if x else 0 for x in (c.split('/') + ['', ''])[:3]]
                pos = v[i[0] - 1 if i[0] > 0 else i[0]]
                st = vt[i[1] - 1 if i[1] > 0 else i[1]] if i[1] else (0.0, 0.0)
                nrm = vn[i[2] - 1 if i[2] > 0 else i[2]] if i[2] else None
                corners.append((pos, st, nrm))
            if len(corners) <= 4:
                tris.append((corners, mat))
            else:
                for k in range(1, len(corners) - 1):
                    tris.append(([corners[0], corners[k], corners[k + 1]], mat))
    return tris, pictures


def power_of_two_up(n):
    return n if n & (n - 1) == 0 else 1 << n.bit_length()


def add_pictures(mod, pictures, base):
    """Put the pictures into the mod as textures of the game: numbers from `base`, each as a small .qpm (what the game's loader takes: at most 256 by 256, a power of two on each side) and a .png of the
    picture's own size beside it, which the renderer shows in its place at any size.  Gives material -> texture number."""
    import png2qpm
    from PIL import Image
    numbers, seen = {}, {}
    for mat, path in pictures.items():
        if path in seen:
            numbers[mat] = seen[path]
            continue
        num = base + len(seen)
        seen[path] = numbers[mat] = num
        img = Image.open(path).convert('RGBA')
        folder = os.path.join(mod, 'MISC.PAK', 'textures')
        os.makedirs(folder, exist_ok=True)
        w, h = power_of_two_up(img.width), power_of_two_up(img.height)
        while w * h > 65536 or max(w, h) > 256:
            w, h = max(1, w // 2), max(1, h // 2)
        small = img.resize((w, h), Image.LANCZOS)
        open(os.path.join(folder, f'{num:04d}.qpm'), 'wb').write(png2qpm.make_qpm(small))
        img.save(os.path.join(folder, f'{num:04d}.png'))
    return numbers


def mesh_part(f, faces, part=0, numbers=None):
    """Replace the geometry of a part by the faces `faces` (`read_obj`'s): a triangle is a strip of three vertices, a quad a, b, c, d the strip a, b, d, c, and the faces are packed into
    chunks, one texture each, within what the vector unit's memory holds (at most 64 vertices, 24 strips and `3 * vertices + strips` of 212: the size of one draw, not a limit of the
    mesh).  There is no limit on the size of the mesh: the new arrays and chunk list are written after the file's other data, the bones and the object record move behind them, and
    the pointers (of the part, and the offsets of the object and of the texture table in the file's header) are changed.  `numbers` is material -> texture number (`add_pictures`): each
    gets a place in the file's table of textures after the ones it has; a face with another material (or none) has the texture of the part's largest chunk.  Normals are the face's own
    when the OBJ gives none."""
    f = bytes(f)
    tex, tab = struct.unpack_from('<2I', f, 0)
    n = struct.unpack_from('<I', f, tab)[0]
    bones_at = tab - n * 80
    b = 80 * part
    bones = bytearray(f[bones_at:tab])
    tail = f[tab:]
    list_at = struct.unpack_from('<I', bones, b + 8)[0]
    chunks = obdump.read_chunks(f, list_at)
    kind = chunks[0][5]
    old_slot = max(chunks, key=lambda c: c[4])[0]
    # the table of textures: the file's own, and a place for each new picture
    entries = []
    while True:
        e = struct.unpack_from('<4I', f, tex + 16 * len(entries))
        if e[0] == 0xffffffff:
            break
        entries.append(e)
    slots = {}
    for mat, num in (numbers or {}).items():
        if num not in slots:
            entries.append((num, 0, 0, 0))
            slots[num] = len(entries) - 1
    slot_of = lambda mat: slots[numbers[mat]] if numbers and mat in numbers else old_slot
    # the faces by texture, packed into chunks
    by_slot = {}
    for corners, mat in faces:
        by_slot.setdefault(slot_of(mat), []).append(corners)
    groups = []
    for slot, fs in by_slot.items():
        cur, cv = [], 0
        for t in fs:
            nv = cv + len(t)
            if nv > 64 or len(cur) + 1 > 24 or 3 * nv + len(cur) + 1 > 212:
                groups.append((slot, cur))
                cur, cv, nv = [], 0, len(t)
            cur.append(t)
            cv = nv
        if cur:
            groups.append((slot, cur))
    # the arrays, with a chunk's first vertex a multiple of 4
    strips, positions, coords, colours, normals, chunk_list = bytearray(), bytearray(), bytearray(), bytearray(), bytearray(), bytearray()
    strip_no = vertex_no = 0
    for slot, g in groups:
        while vertex_no % 4:
            positions += bytes(12)
            coords += bytes(16)
            colours += bytes(4)
            normals += bytes(12)
            vertex_no += 1
        v0 = vertex_no
        for ti, t in enumerate(g):
            last = ti == len(g) - 1
            strips += struct.pack('<4I', len(t) | (0x8000 if last else 0), 0x302e4000, 0x412, 0)
            a, bb, c = (t[0][0], t[1][0], t[2][0])
            fn = [(bb[1] - a[1]) * (c[2] - a[2]) - (bb[2] - a[2]) * (c[1] - a[1]), (bb[2] - a[2]) * (c[0] - a[0]) - (bb[0] - a[0]) * (c[2] - a[2]), (bb[0] - a[0]) * (c[1] - a[1]) - (bb[1] - a[1]) * (c[0] - a[0])]
            ln = sum(x * x for x in fn) ** 0.5 or 1.0
            fn = [x / ln for x in fn]
            for pos, st, nrm in (t if len(t) == 3 else [t[0], t[1], t[3], t[2]]):
                positions += struct.pack('<3f', *pos)
                coords += struct.pack('<4f', st[0], st[1], 1.0, 1.0)
                colours += bytes((0x7f, 0x7f, 0x7f, 0x7f))
                normals += struct.pack('<3f', *(nrm or fn))
                vertex_no += 1
        chunk_list += struct.pack('<6i', slot, strip_no, len(g), v0, vertex_no - v0, kind)
        strip_no += len(g)
    chunk_list += struct.pack('<6i', 0, 0, 0, 0, 0, -1)
    table = b''.join(struct.pack('<4I', *e) for e in entries) + struct.pack('<4I', 0xffffffff, 0, 0, 0)
    # the file: what was before the bones, the new arrays and table (each on a 16 byte boundary), the bones and the object record
    out = bytearray(f[:bones_at])
    place = {}
    for name, data in (('strips', strips), ('positions', positions), ('coords', coords), ('colours', colours), ('normals', normals), ('chunks', chunk_list), ('table', table)):
        while len(out) % 16:
            out.append(0)
        place[name] = len(out)
        out += data
    while len(out) % 16:
        out.append(0)
    new_tab = len(out) + n * 80
    struct.pack_into('<I', bones, b + 8, place['chunks'])
    for k, name in enumerate(('strips', 'positions', 'coords', 'colours')):
        struct.pack_into('<I', bones, b + 20 + 4 * k, place[name])
    struct.pack_into('<I', bones, b + 36, 0)
    if kind in (3, 4):
        struct.pack_into('<I', bones, b + 40, place['normals'])
    struct.pack_into('<H', bones, b + 70, min(sum(len(t) - 2 for t, _ in faces), 65535))
    out += bones + tail
    struct.pack_into('<I', out, 0, place['table'])
    struct.pack_into('<I', out, 4, new_tab)
    return bytes(out), len(groups)


def main():
    if len(sys.argv) >= 7 and sys.argv[4] == 'mesh':
        d, es = pakls.entries(sys.argv[1])
        hit = [(n, o, s) for n, o, s in es if n == sys.argv[2]]
        if not hit:
            sys.exit('no such object: ' + sys.argv[2])
        n, o, s = hit[0]
        faces, pictures = read_obj(sys.argv[6])
        base = int(sys.argv[sys.argv.index('--tex-base') + 1]) if '--tex-base' in sys.argv else 9000
        numbers = add_pictures(sys.argv[3], pictures, base) if pictures else None
        data, groups = mesh_part(d[o:o + s], faces, int(sys.argv[5]), numbers)
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
