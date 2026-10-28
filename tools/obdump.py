"""Print the structure of an object (model) file of OB.PAK: its header, texture table, parts (bones) and chunks.

    python tools/obdump.py <OB.PAK> <name in the pack, or part of it>      e.g. ob/chrs/chr0.raw
    python tools/obdump.py <OB.PAK> --list [filter]                         lists the objects
    python tools/obdump.py <OB.PAK> <name> --obj out.obj                    writes the triangles (in the file's own space) as a Wavefront OBJ

The layout is the one of `docs/level-formats.md` ("A room's model") with what a character adds (a skeleton: the parts are bones).  Pointers in the file are offsets from its start.
"""
import os
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import pakls  # noqa: E402


def u32(f, o):
    return struct.unpack_from('<I', f, o)[0]


def ok(f, p):
    return 0 < p < len(f)


def dump(f, verbose=True):
    tex, tab = struct.unpack_from('<2I', f, 0)
    n = u32(f, tab)
    print(f'file {len(f)} bytes; texture table at {tex}; object at {tab} with {n} parts; header words {struct.unpack_from("<8I", f, 0)}')
    print(f'object record: {struct.unpack_from("<12I", f, tab)}')
    # the textures
    k, textures = 0, []
    while tex + 16 * k + 16 <= len(f):
        t = struct.unpack_from('<4I', f, tex + 16 * k)
        if t[0] == 0xffffffff:
            break
        textures.append(t)
        k += 1
    print(f'{len(textures)} textures: {[(t[0], hex(t[3])) for t in textures[:12]]}')
    bones = tab - n * 80
    kinds = {}
    for i in range(n):
        b = bones + 80 * i
        kind, matrix = f[b], f[b + 1]
        links = struct.unpack_from('<4b', f, b + 2)
        lamps, vecs = f[b + 6], f[b + 7]
        a_chunks, b_chunks = u32(f, b + 8), u32(f, b + 12)
        mask, tris = struct.unpack_from('<2H', f, b + 68)
        kinds[kind] = kinds.get(kind, 0) + 1
        chunks = lambda p: [] if not ok(f, p) else read_chunks(f, p)
        ca, cb = chunks(a_chunks), chunks(b_chunks)
        if verbose:
            print(f' part {i:3d}: kind {kind} joint {matrix} links {links} lamps {lamps} vectors {vecs} mask {mask:#x} triangles {tris} chunks {len(ca)}+{len(cb)}')
            if kind != 0 or vecs:
                vp = u32(f, b + 72)
                if vecs and ok(f, vp):
                    print('    vectors', [struct.unpack_from('<3f', f, vp + 12 * j) for j in range(min(vecs, 4))])
    print('kinds of parts:', kinds)


def read_chunks(f, p):
    out = []
    while p + 24 <= len(f):
        c = struct.unpack_from('<6i', f, p)
        if c[5] < 0:
            break
        out.append(c)
        p += 24
    return out


def triangles(f):
    """All the triangles of an object in the space of its file: (part, joint, [three positions])."""
    tex, tab = struct.unpack_from('<2I', f, 0)
    n = u32(f, tab)
    bones = tab - n * 80
    out = []
    for i in range(n):
        b = bones + 80 * i
        for list_at, arrays in ((u32(f, b + 8), b + 20), (u32(f, b + 12), b + 44)):
            if not ok(f, list_at):
                continue
            strips_at, pos_at = u32(f, arrays), u32(f, arrays + 4)
            for tex_slot, first_strip, strips, first_vertex, vertices, kind in read_chunks(f, list_at):
                v = first_vertex
                for s in range(strips):
                    count = u32(f, strips_at + 16 * (first_strip + s)) & 0x7fff
                    pts = [struct.unpack_from('<3f', f, pos_at + 12 * (v + k)) for k in range(count)]
                    for k in range(count - 2):
                        tri = [pts[k], pts[k + 1], pts[k + 2]]
                        if k % 2:
                            tri = [tri[0], tri[2], tri[1]]
                        out.append((i, f[b + 1], tri))
                    v += count
    return out


def write_obj(f, path):
    tris = triangles(f)
    with open(path, 'w') as o:
        k = 1
        for part, joint, tri in tris:
            for p in tri:
                o.write('v %f %f %f\n' % p)
            o.write('f %d %d %d\n' % (k, k + 1, k + 2))
            k += 3
    print(len(tris), 'triangles written to', path)


def main():
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    d, es = pakls.entries(sys.argv[1])
    if sys.argv[2] == '--list':
        flt = sys.argv[3] if len(sys.argv) > 3 else ''
        for n, o, s in es:
            if flt in n:
                print(f'{s:8d} {n}')
        return
    hits = [(n, o, s) for n, o, s in es if sys.argv[2] in n]
    if not hits:
        sys.exit('no such object')
    n, o, s = hits[0]
    print(n)
    if '--obj' in sys.argv:
        write_obj(d[o:o + s], sys.argv[sys.argv.index('--obj') + 1])
        return
    dump(d[o:o + s])


if __name__ == '__main__':
    main()
