"""List the files of a P4CK pack: python tools/pakls.py <pack> [filter]"""
import struct, sys, collections

def entries(path):
    d = open(path, 'rb').read()
    assert d[:4] == b'P4CK', 'not a P4CK pack'
    dir_at, dir_len = struct.unpack_from('<II', d, 4)
    out = []
    for o in range(dir_at, dir_at + dir_len, 60):
        e = d[o:o + 60]
        if len(e) < 60:
            break
        name = e[:48].split(b'\0')[0].decode('latin1')
        off, size = struct.unpack_from('<II', e, 48)
        out.append((name, off, size))
    return d, out

if __name__ == '__main__':
    d, es = entries(sys.argv[1])
    flt = sys.argv[2] if len(sys.argv) > 2 else ''
    ext = collections.Counter(n.rsplit('.', 1)[-1] if '.' in n else '' for n, _, _ in es)
    print(len(es), 'files', dict(ext))
    for n, o, s in es:
        if flt in n:
            print(f'{o:9d} {s:9d} {n}'.encode('ascii','replace').decode())
