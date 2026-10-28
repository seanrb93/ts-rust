"""Print the structure of an animation file of ANIM.PAK: its frames, key frame times and, for each channel (a bone), the kind of its data and its first poses.

    python tools/anidump.py <ANIM.PAK> <name in the pack, or part of it>      e.g. anim/data/rf_m0.raw
    python tools/anidump.py <ANIM.PAK> --list [filter]

The layout is the one `loadAnimation` lays out in memory (see docs/model-format.md, "Animations"): a header, the table of the key frames' times (in 60ths of a second),
a table of channels of 32 bytes (one for each bone) and the data of the channels, each part starting on a 16 byte boundary.  A pose is seven floats: a position (x, y, z) and a rotation as
a quaternion (x, y, z, w).
"""
import math
import os
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import pakls  # noqa: E402


def align(n):
    return (n + 15) & ~15


def parse(f):
    frames, keys, bones, flags = struct.unpack_from('<4I', f, 0)
    channels = struct.unpack_from('<I', f, 28)[0]
    times_at = 48
    table_at = ((keys << 2) + 63) & ~15
    data_at = (((keys << 2) + 15) & ~15) + (channels << 5) + 51 & ~15
    return {'frames': frames, 'keys': keys, 'bones': bones, 'flags': flags, 'channels': channels, 'times_at': times_at, 'table_at': table_at, 'data_at': data_at}


def channel_sizes(kind, count):
    """The bytes of a channel's data, as `loadAnimation` counts them (a kind that is not known takes none)."""
    if kind == 2:
        return (count << 4) | 12
    if kind == 4:
        return (count - 1) * 12 + 28
    if kind == 8:
        return count * 28
    if kind == 0:
        return 28
    return 0


def pose(f, at):
    return struct.unpack_from('<7f', f, at)


def poses(f, kind, count, at):
    """The poses of a channel at each key frame: position and quaternion, the way `extractAnimData` reads them."""
    out = []
    for k in range(count):
        if kind == 8:
            out.append(pose(f, at + 28 * k))
        elif kind == 2:
            pos = struct.unpack_from('<3f', f, at)
            rot = struct.unpack_from('<4f', f, at + (k << 4) + 12) if True else None
            out.append(pos + rot)
        elif kind == 4:
            rot = struct.unpack_from('<4f', f, at + 12)
            pos = struct.unpack_from('<3f', f, at + 28 + (k - 1) * 12) if k else struct.unpack_from('<3f', f, at)
            out.append(pos + rot)
        else:
            out.append(pose(f, at))
    return out


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
        sys.exit('no such animation')
    n, o, s = hits[0]
    f = d[o:o + s]
    print(n, s, 'bytes')
    h = parse(f)
    print(h)
    times = struct.unpack_from(f'<{h["keys"]}I', f, h['times_at'])
    print('key frame times (60ths of a second):', times[:12])
    at = h['data_at']
    for i in range(h['channels']):
        c = struct.unpack_from('<8I', f, h['table_at'] + 32 * i)
        kind, count = c[1], c[3]
        ps = poses(f, kind, count, at) if kind in (0, 2, 4, 8) else []
        norms = [round(math.sqrt(sum(x * x for x in p[3:])), 3) for p in ps]
        print(f' channel {i}: bone {c[0]} kind {kind} key frames {count} data at {at} ({channel_sizes(kind, count)} bytes); quaternion lengths {sorted(set(norms))[:3]}')
        if ps:
            print('    first pose', [round(x, 3) for x in ps[0]])
        at = (at + channel_sizes(kind, count) + 15) & ~15
    print('data ends at', at, 'of', len(f))


if __name__ == '__main__':
    main()
