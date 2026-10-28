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
    """All the triangles of an object in the space of its file: (part, joint, slot, [three (position, (s, t))])."""
    tex, tab = struct.unpack_from('<2I', f, 0)
    n = u32(f, tab)
    bones = tab - n * 80
    out = []
    for i in range(n):
        b = bones + 80 * i
        for list_at, arrays in ((u32(f, b + 8), b + 20), (u32(f, b + 12), b + 44)):
            if not ok(f, list_at):
                continue
            strips_at, pos_at, st_at, col_at, nrm_at = u32(f, arrays), u32(f, arrays + 4), u32(f, arrays + 8), u32(f, arrays + 12), u32(f, arrays + 20)
            for tex_slot, first_strip, strips, first_vertex, vertices, kind in read_chunks(f, list_at):
                v = first_vertex
                for s in range(strips):
                    word = u32(f, strips_at + 16 * (first_strip + s))
                    count, flip = word & 0x7fff, bool(word & 0x10000)
                    pts = [(struct.unpack_from('<3f', f, pos_at + 12 * (v + k)), struct.unpack_from('<2f', f, st_at + 16 * (v + k)), tuple(f[col_at + 4 * (v + k):col_at + 4 * (v + k) + 3]) if ok(f, col_at) else None, struct.unpack_from('<3f', f, nrm_at + 12 * (v + k)) if kind in (3, 4) and ok(f, nrm_at) else None) for k in range(count)]
                    for k in range(count - 2):
                        tri = [pts[k], pts[k + 1], pts[k + 2]]
                        if (k % 2) ^ flip:
                            tri = [tri[0], tri[2], tri[1]]
                        out.append((i, f[b + 1], tex_slot, tri, kind))
                    v += count
    return out


def strips_of(f):
    """The strips of an object as they are in the file: (part, slot, kind, flip, [(position, (s, t), colour, normal)])."""
    tex, tab = struct.unpack_from('<2I', f, 0)
    n = u32(f, tab)
    bones = tab - n * 80
    out = []
    for i in range(n):
        b = bones + 80 * i
        for list_at, arrays in ((u32(f, b + 8), b + 20), (u32(f, b + 12), b + 44)):
            if not ok(f, list_at):
                continue
            strips_at, pos_at, st_at, col_at, nrm_at = u32(f, arrays), u32(f, arrays + 4), u32(f, arrays + 8), u32(f, arrays + 12), u32(f, arrays + 20)
            for chunk_no, (tex_slot, first_strip, strips, first_vertex, vertices, kind) in enumerate(read_chunks(f, list_at)):
                v = first_vertex
                for s in range(strips):
                    word = u32(f, strips_at + 16 * (first_strip + s))
                    count = word & 0x7fff
                    pts = [(struct.unpack_from('<3f', f, pos_at + 12 * (v + k)), struct.unpack_from('<2f', f, st_at + 16 * (v + k)), tuple(f[col_at + 4 * (v + k):col_at + 4 * (v + k) + 3]) if ok(f, col_at) else None, struct.unpack_from('<3f', f, nrm_at + 12 * (v + k)) if kind in (3, 4) and ok(f, nrm_at) else None) for k in range(count)]
                    out.append((i, tex_slot, kind, bool(word & 0x10000), pts, (list_at, chunk_no)))
                    v += count
    return out


def write_obj(f, path):
    """The model as an OBJ: an object `part<N>` for each part, a material `slot<K>` for each texture of the file's table (the names `obedit.py mesh` reads back), with the coordinates, the
    vertex colours (as the three numbers after a position) and the normals where the chunks have them."""
    tris = triangles(f)
    NL = chr(10)
    with open(path, 'w') as o:
        k = 1
        kn = 1
        last = (None, None, None)
        for part, joint, slot, tri, kind in sorted(tris, key=lambda t: (t[0], t[2], t[4])):
            if (part, slot, kind) != last:
                if part != last[0]:
                    o.write('o part%d' % part + NL)
                o.write('usemtl slot%d_k%d' % (slot, kind) + NL)
                last = (part, slot, kind)
            refs = []
            for pos, st, col, nrm in tri:
                if col:
                    o.write(('v %f %f %f %f %f %f' % (pos + tuple(min(1.0, c / 127.0) for c in col))) + NL)
                else:
                    o.write(('v %f %f %f' % pos) + NL)
                o.write(('vt %f %f' % st) + NL)
                if nrm:
                    o.write(('vn %f %f %f' % nrm) + NL)
                    refs.append('%d/%d/%d' % (k, k, kn))
                    kn += 1
                else:
                    refs.append('%d/%d' % (k, k))
                k += 1
            o.write('f ' + ' '.join(refs) + NL)
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
