"""Make a changed copy of an object (model) file of OB.PAK, as a mod's file.

    python tools/obedit.py <OB.PAK> <name in the pack> <mods folder>/<mod> scale <factor>

writes <mod>/OB.PAK/<name> with every position of the model multiplied by <factor> (a test that the game takes a model file made by a tool: the joints of the animations do not
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


def main():
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
