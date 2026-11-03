"""A character in a pose of one of its animations, as an OBJ.

    python pose-character.py <OB.PAK> <chr file in the pack, e.g. ob/chrs/chr01.raw> <matrices.txt> <out.obj> [part ...]

`matrices.txt` is what the port writes for `TS_DUMPARM=<file>` (with `TS_HANDANIM=<animation>`): the matrices of the joints of the character at one moment of the animation (40 lines of 16
numbers, a joint each; row vectors, the translation in the last row).  Each vertex of the model (`tools/obdump.py` reads them in the rest pose) is moved with the matrix of the joint it
follows: the joint of its part for a part that is rigid, the joint its bind says for a part that joins joints.  The OBJ has the parts named `part<N>` as `obdump.py` writes them
(only the parts named on the command line when some are).
"""
import os
import struct
import sys

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, '..', '..', 'tools'))
import pakls  # noqa: E402
from obdump import read_chunks, ok, u32  # noqa: E402


def load_matrices(path):
    rows = []
    for l in open(path):
        if l.strip() == 'rest':
            break
        if l.strip():
            rows.append([float(x) for x in l.split()])
    return [np.array(r).reshape(4, 4) for r in rows]


def posed_triangles(f, mats, rest=None):
    """The triangles of the model with each vertex moved by its joint's matrix: (part, slot, kind, [(position, (s, t), colour)] * 3)."""
    tex, tab = struct.unpack_from('<2I', f, 0)
    n = u32(f, tab)
    bones = tab - n * 80
    out = []
    for i in range(n):
        b = bones + 80 * i
        part_joint = f[b + 1]
        for list_at, arrays in ((u32(f, b + 8), b + 20),):
            if not ok(f, list_at):
                continue
            strips_at, pos_at, st_at, col_at, bind_at, nrm_at = struct.unpack_from('<6I', f, arrays)
            for tex_slot, first_strip, strips, first_vertex, vertices, kind in read_chunks(f, list_at):
                v = first_vertex
                for s in range(strips):
                    word = u32(f, strips_at + 16 * (first_strip + s))
                    count, flip = word & 0x7fff, bool(word & 0x10000)
                    pts = []
                    for k in range(count):
                        pos = np.array(struct.unpack_from('<3f', f, pos_at + 12 * (v + k)) + (1.0,))
                        joint = f[bind_at + 4 * (v + k)] if kind == 4 and ok(f, bind_at) else part_joint
                        if joint >= len(mats):
                            joint = min(part_joint, len(mats) - 1)
                        moved = (pos @ (np.linalg.inv(rest[joint]) @ mats[joint] if rest else mats[joint]))[:3]
                        st = struct.unpack_from('<2f', f, st_at + 16 * (v + k))
                        col = tuple(f[col_at + 4 * (v + k):col_at + 4 * (v + k) + 3]) if ok(f, col_at) else None
                        pts.append((tuple(moved), st, col))
                    for k in range(count - 2):
                        tri = [pts[k], pts[k + 1], pts[k + 2]]
                        if (k % 2) ^ flip:
                            tri = [tri[0], tri[2], tri[1]]
                        out.append((i, tex_slot, kind, tri))
                    v += count
    return out


def main():
    if len(sys.argv) < 5:
        sys.exit(__doc__)
    pak, name, mats_path, out = sys.argv[1:5]
    only = set(sys.argv[5:])
    d, es = pakls.entries(pak)
    n, o, s = [e for e in es if e[0] == name][0]
    f = d[o:o + s]
    rest = load_matrices(os.environ['REST_MATRICES']) if os.environ.get('REST_MATRICES') else None
    tris = posed_triangles(f, load_matrices(mats_path), rest)
    nl = chr(10)
    with open(out, 'w') as w:
        k, last = 1, None
        for part, slot, kind, tri in sorted(tris, key=lambda t: (t[0], t[1], t[2])):
            if only and str(part) not in only:
                continue
            if part != last:
                w.write('o part%d' % part + nl)
                last = part
            w.write('usemtl slot%d_k%d' % (slot, kind) + nl)
            refs = []
            for pos, st, col in tri:
                w.write('v %f %f %f' % pos + nl)
                w.write('vt %f %f' % st + nl)
                refs.append('%d/%d' % (k, k))
                k += 1
            w.write('f ' + ' '.join(refs) + nl)
    print(len(tris), 'triangles')


main()
