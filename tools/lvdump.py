"""Read and check the level files of the game (the formats are in docs/level-formats.md).

    python tools/lvdump.py <PAK folder> [level number ...]

For every level given (all of them by default) the BG model (BG.PAK bg/levelN/levelN.raw), the pad file (MISC.PAK pad/data/levelN.raw) and the
prop file (MISC.PAK propsetup/data/levelN.raw) are read with the layouts described in the document, and every offset, count and terminator that
the document promises is checked.  A summary is printed for each; problems are listed and make the exit status 1.
"""
import os
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from pakls import entries  # noqa: E402


class Bad(Exception):
    pass


def u32(b, o):
    if o < 0 or o + 4 > len(b):
        raise Bad(f'read at {o:#x} is outside the file ({len(b):#x} bytes)')
    return struct.unpack_from('<I', b, o)[0]


def s32(b, o):
    return struct.unpack_from('<i', b, o)[0] if 0 <= o <= len(b) - 4 else u32(b, o)


def u16(b, o):
    if o < 0 or o + 2 > len(b):
        raise Bad(f'read at {o:#x} is outside the file')
    return struct.unpack_from('<H', b, o)[0]


def f32(b, o):
    if o < 0 or o + 4 > len(b):
        raise Bad(f'read at {o:#x} is outside the file')
    return struct.unpack_from('<f', b, o)[0]


# ---------------------------------------------------------------- BG (the model of a level)

ROOM = 44          # bytes of a room record
PART = 80          # bytes of a part record
CHUNK = 24         # bytes of a chunk record
DESC = 16          # bytes of a strip descriptor


def read_ptr_list(b, at, what):
    """A list of offsets ended by 0 (an offset of 0 in the middle is a deleted entry only where the caller says so)."""
    out = []
    while True:
        v = u32(b, at + 4 * len(out))
        if v == 0:
            return out
        if v >= len(b):
            raise Bad(f'{what}: entry {len(out)} is {v:#x}, outside the file')
        out.append(v)
        if len(out) > 100000:
            raise Bad(f'{what}: no end')


def parse_chunks(b, at, what):
    out = []
    while True:
        c = struct.unpack_from('<6i', b, at + CHUNK * len(out)) if at + CHUNK * (len(out) + 1) <= len(b) else None
        if c is None:
            raise Bad(f'{what}: the chunks run off the file')
        if c[5] < 0:
            return out
        out.append(c)
        if len(out) > 10000:
            raise Bad(f'{what}: no end')


def parse_part(b, ob, k, n, what):
    at = ob - PART * n + PART * k
    w = struct.unpack_from('<20I', b, at)
    p = {'at': at, 'kind': b[at], 'flags': w[0] >> 8, 'w1': w[1], 'raw': w}
    sets = []
    for chunks_at, arrays_at in ((w[2], 5), (w[3], 11)):
        if chunks_at == 0:
            sets.append(None)
            continue
        chunks = parse_chunks(b, chunks_at, f'{what} part {k}')
        arr = [w[arrays_at + i] for i in range(6)]
        sets.append({'chunks': chunks, 'desc': arr[0], 'pos': arr[1], 'st': arr[2], 'rgba': arr[3], 'norm': arr[4], 'extra': arr[5]})
    p['sets'] = sets
    return p


def check_part_set(b, s, what, ntex):
    """The chunks of a set against the file: strip descriptors, vertices and arrays all inside it."""
    verts = 0
    for ci, c in enumerate(s['chunks']):
        tex, desc0, ndesc, v0, nv, kind = c
        if not 0 <= tex < ntex:
            raise Bad(f'{what}: chunk {ci} uses texture slot {tex} of {ntex}')
        # the strips of the chunk: the vertex counts add up to the chunk's
        total = 0
        d = desc0
        while True:
            w = u32(b, s['desc'] + DESC * d)
            total += w & 0x7fff
            d += 1
            if w & 0x8000:
                break
            if d - desc0 > 100000:
                raise Bad(f'{what}: chunk {ci} has no last strip')
        if d - desc0 != ndesc:
            raise Bad(f'{what}: chunk {ci} says {ndesc} descriptors, has {d - desc0}')
        if total != nv:
            raise Bad(f'{what}: chunk {ci} says {nv} vertices, its strips have {total}')
        for name, size in (('pos', 12), ('st', 16), ('rgba', 4)):
            end = s[name] + (v0 + nv) * size
            if s[name] == 0 or end > len(b):
                raise Bad(f'{what}: chunk {ci} {name} array ends at {end:#x}')
        if kind in (3, 4) and (s['norm'] == 0 or s['norm'] + (v0 + nv) * 12 > len(b)):
            raise Bad(f'{what}: chunk {ci} has no normals')
        verts += nv
    return verts


def parse_bg(b, report):
    """Parse a level's BG file; returns a summary dict."""
    tex_at, rooms_at, portals_at, vols_at = (u32(b, 4 * i) for i in range(4))
    # texture table: 16 bytes each, the texture number first, ended by -1
    tex = []
    while s32(b, tex_at + 16 * len(tex)) != -1:
        tex.append(struct.unpack_from('<4i', b, tex_at + 16 * len(tex)))
        if len(tex) > 4096:
            raise Bad('texture table has no end')
    rooms = []
    i = 1
    while u32(b, rooms_at + ROOM * i) != 0:
        r = struct.unpack_from('<5I6f', b, rooms_at + ROOM * i)
        rooms.append(r)
        i += 1
    if u32(b, rooms_at) != 0 or any(u32(b, rooms_at + 4 * k) for k in range(11)):
        report('room 0 is not empty')
    nparts = nverts = nfloors = nwalls = nglass = 0
    portals = read_ptr_list(b, portals_at + 4, 'portals') if u32(b, portals_at + 4) else []
    for ri, r in enumerate(rooms, 1):
        what = f'room {ri}'
        ob = r[0]
        n = u32(b, ob)
        if not 1 <= n <= 64 or ob - PART * n < 0:
            raise Bad(f'{what}: {n} parts')
        for k in range(n):
            p = parse_part(b, ob, k, n, what)
            nparts += 1
            for s in p['sets']:
                if s:
                    nverts += check_part_set(b, s, what, len(tex))
        # the neighbours: a count and portal numbers
        if r[1]:
            cnt = u32(b, r[1])
            for k in range(cnt):
                if not 1 <= u32(b, r[1] + 4 + 4 * k) <= len(portals):
                    raise Bad(f'{what}: neighbour {k} is portal {u32(b, r[1] + 4 + 4 * k)} of {len(portals)}')
        for slot, name in ((2, 'floors'), (3, 'walls'), (4, 'glass')):
            if r[slot] == 0:
                continue
            at = r[slot]
            count = 0
            while True:
                v = u32(b, at + 4 * count)
                count += 1
                if v == 0:
                    break
                if v >= len(b):
                    raise Bad(f'{what}: {name} entry {count - 1} is outside the file')
                if name == 'floors':
                    nfloors += 1
                    nv = u32(b, v)
                    if nv and v + 8 + 12 * nv > len(b):
                        raise Bad(f'{what}: floor with {nv} vertices runs off the file')
                elif name == 'walls':
                    nwalls += 1
                else:
                    nglass += 1
        # (the lists end with their first 0: a list can hold 0 entries in the middle)
    # portals: two rooms, a count of vertices at +20 (16 bits), the vertices from +24
    for pi, pa in enumerate(portals, 1):
        a, bb = u32(b, pa), u32(b, pa + 4)
        if not (1 <= a <= len(rooms) and 1 <= bb <= len(rooms)):
            raise Bad(f'portal {pi} joins rooms {a} and {bb} of {len(rooms)}')
        nv = u16(b, pa + 20)
        if nv < 3 or pa + 24 + 12 * nv > len(b):
            raise Bad(f'portal {pi} has {nv} vertices')
    vols = u32(b, vols_at) if vols_at else 0
    return {'textures': [t[0] for t in tex], 'rooms': len(rooms), 'parts': nparts, 'verts': nverts, 'floors': nfloors,
            'walls': nwalls, 'glass': nglass, 'portals': len(portals), 'volumes': vols}


# ---------------------------------------------------------------- pad (the points bots and players use)

def parse_pad(b):
    rev, npads, nlinks = struct.unpack_from('<3I', b, 0)
    if rev != 1:
        raise Bad(f'pad file revision {rev}')
    pads = [struct.unpack_from('<iIHHffff', b, 12 + 28 * i) for i in range(npads)]
    links = [struct.unpack_from('<IiiII', b, 12 + 28 * npads + 20 * i) for i in range(nlinks)]
    have = 12 + 28 * npads + 20 * nlinks
    note = ''
    if have != len(b):
        note = f'(the file is {len(b) - have:+d} bytes from 12+28*pads+20*links)'
    ids = {p[0] for p in pads}
    for k, l in enumerate(links):
        if l[1] not in ids or l[2] not in ids:
            note += f' link {k} joins {l[1]} and {l[2]}, not both pads'
            break
    return {'pads': npads, 'links': nlinks, 'flagged': sum(1 for p in pads if p[1] & 1), 'note': note.strip()}


# ---------------------------------------------------------------- props

def parse_props(b):
    rev, n = struct.unpack_from('<2I', b, 0)
    if rev != 1:
        raise Bad(f'prop file revision {rev}')
    if len(b) != 8 + 24 * n:
        raise Bad(f'prop file: {len(b)} bytes for {n} props')
    props = [struct.unpack_from('<iIffff', b, 8 + 24 * i) for i in range(n)]
    return {'props': n, 'types': sorted({p[1] for p in props})}


def main(argv):
    if not argv:
        print(__doc__)
        return 2
    folder = argv[0]
    wanted = {int(x) for x in argv[1:]}
    bg_d, bg_e = entries(os.path.join(folder, 'BG.PAK'))
    ms_d, ms_e = entries(os.path.join(folder, 'MISC.PAK'))
    bg = {n: bg_d[o:o + s] for n, o, s in bg_e}
    ms = {n: ms_d[o:o + s] for n, o, s in ms_e}
    bad = 0
    for lv in range(1, 31):
        if wanted and lv not in wanted:
            continue
        line = [f'level {lv:2d}']
        for label, files, fn in (('bg', (bg, f'bg/level{lv}/level{lv}.raw'), None), ('pad', (ms, f'pad/data/level{lv}.raw'), parse_pad),
                                 ('props', (ms, f'propsetup/data/level{lv}.raw'), parse_props)):
            src, name = files
            if name not in src:
                continue
            b = src[name]
            try:
                notes = []
                if label == 'bg':
                    r = parse_bg(b, notes.append)
                    r['textures'] = len(r['textures'])
                else:
                    r = fn(b)
                note = r.pop('note', '')
                line.append(label + ' ' + ' '.join(f'{k}={v}' for k, v in r.items()) + (' ' + note if note else '') + (' !' + '; '.join(notes) if notes else ''))
            except (Bad, struct.error) as e:
                bad += 1
                line.append(f'{label} PROBLEM: {e}')
        print(' | '.join(line))
    return 1 if bad else 0


if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))
