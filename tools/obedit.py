"""Make a changed copy of an object (model) file of OB.PAK, as a mod's file.

    python tools/obedit.py <OB.PAK> <name in the pack> <mods folder>/<mod> scale <factor>

    python tools/obedit.py <OB.PAK> <name in the pack> <mods folder>/<mod> box [part [size]]

with `box` the geometry of the model's part number `part` (the first by default) is replaced by a box made from nothing (a test that strips, positions, coordinates, colours and normals written by a tool are taken: the head of a
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


def main():
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
