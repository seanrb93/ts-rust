"""Importers: each reads a level or model in some format and gives a `common.Scene` (polygons, materials, markers)."""
import os
import struct


def detect(path):
    """The importer's name for a file."""
    low = path.lower()
    if low.endswith('.obj'):
        return 'obj'
    if low.endswith(('.glb', '.gltf')):
        return 'gltf'
    if low.endswith('.blend'):
        return 'blend'
    if low.endswith('.vmf'):
        return 'vmf'
    if low.endswith(('.bsp', '.bsp.bz2')):
        head = open(path, 'rb').read(8)
        if head[:3] == b'BZh':
            return 'source'
        if head[:4] == b'VBSP':
            return 'source'
        if len(head) >= 4 and struct.unpack_from('<i', head, 0)[0] in (29, 30):
            return 'goldsrc'
        raise SystemExit(f'{path}: a BSP of a kind that is not known (Quake 1 and 2, Source and GoldSrc maps are the ones to try)')
    raise SystemExit(f'{path}: a file of a kind that is not known (.obj .glb .gltf .blend .bsp .vmf)')


def load(path, opts):
    kind = detect(path)
    if kind == 'obj':
        from . import obj as m
    elif kind == 'gltf':
        from . import gltf as m
    elif kind == 'blend':
        from . import blend as m
    elif kind == 'goldsrc':
        from . import goldsrc as m
    else:
        from . import source as m
    return m.load(path, opts)
