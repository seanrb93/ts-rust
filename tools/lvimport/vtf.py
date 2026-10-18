"""Source engine textures (.vtf) and materials (.vmt): enough to get a surface's picture and its flags."""
import io
import re
import struct

import numpy as np
from PIL import Image

# image formats: number -> (name, bytes per pixel or None for the block formats)
BPP = {0: 4, 1: 4, 2: 3, 3: 3, 4: 2, 5: 1, 6: 2, 8: 1, 11: 4, 12: 4, 13: None, 14: None, 15: None}


def _size(fmt, w, h):
    if fmt in (13,):
        return max(1, (w + 3) // 4) * max(1, (h + 3) // 4) * 8
    if fmt in (14, 15):
        return max(1, (w + 3) // 4) * max(1, (h + 3) // 4) * 16
    return w * h * BPP[fmt]


def read_vtf(data):
    """The largest picture of a VTF file as RGBA."""
    if data[:4] != b'VTF\0':
        raise ValueError('not a VTF file')
    major, minor, header = struct.unpack_from('<III', data, 4)
    w, h, flags, frames, first = struct.unpack_from('<HHIHH', data, 16)
    fmt = struct.unpack_from('<i', data, 52)[0]
    mips = data[56]
    lowfmt = struct.unpack_from('<i', data, 57)[0]
    lw, lh = data[61], data[62]
    if fmt not in BPP:
        raise ValueError(f'image format {fmt} is not supported')
    faces = 6 if flags & 0x4000 else 1
    if (major, minor) >= (7, 3):
        count = struct.unpack_from('<I', data, 68)[0]
        start = None
        for i in range(count):
            tag = data[80 + 8 * i:83 + 8 * i]
            off = struct.unpack_from('<I', data, 84 + 8 * i)[0]
            if tag == b'\x30\x00\x00':
                start = off
        if start is None:
            raise ValueError('no image data')
    else:
        start = header + (_size(lowfmt, lw, lh) if lw and lh and lowfmt in BPP else 0)
    # the mips are stored smallest first; each holds every frame (and face)
    before = 0
    for m in range(mips - 1, 0, -1):
        mw, mh = max(1, w >> m), max(1, h >> m)
        before += _size(fmt, mw, mh) * frames * faces
    at = start + before
    n = _size(fmt, w, h)
    raw = data[at:at + n]
    if len(raw) < n:
        raise ValueError('the picture is cut short')
    return decode(fmt, raw, w, h)


def decode(fmt, raw, w, h):
    if fmt in (13, 14, 15):
        four = {13: b'DXT1', 14: b'DXT3', 15: b'DXT5'}[fmt]
        hdr = bytearray(128)
        hdr[0:4] = b'DDS '
        struct.pack_into('<7I', hdr, 4, 124, 0x1 | 0x2 | 0x4 | 0x1000 | 0x80000, h, w, len(raw), 0, 1)
        struct.pack_into('<2I4s', hdr, 76, 32, 4, four)
        struct.pack_into('<I', hdr, 108, 0x1000)
        return Image.open(io.BytesIO(bytes(hdr) + raw)).convert('RGBA')
    a = np.frombuffer(raw, dtype=np.uint8)
    if fmt == 0:
        px = a.reshape(h, w, 4)
    elif fmt == 1:      # ABGR
        px = a.reshape(h, w, 4)[:, :, ::-1]
    elif fmt == 12:     # BGRA
        px = a.reshape(h, w, 4)[:, :, [2, 1, 0, 3]]
    elif fmt == 11:     # BGRX
        px = a.reshape(h, w, 4)[:, :, [2, 1, 0, 3]].copy()
        px[:, :, 3] = 255
    elif fmt == 2:
        px = np.dstack([a.reshape(h, w, 3), np.full((h, w), 255, np.uint8)])
    elif fmt == 3:
        px = np.dstack([a.reshape(h, w, 3)[:, :, ::-1], np.full((h, w), 255, np.uint8)])
    elif fmt == 4:      # RGB565
        v = np.frombuffer(raw, dtype='<u2').reshape(h, w).astype(np.uint32)
        px = np.dstack([((v >> 11) & 31) * 255 // 31, ((v >> 5) & 63) * 255 // 63, (v & 31) * 255 // 31, np.full((h, w), 255)]).astype(np.uint8)
    elif fmt == 5:      # I8
        g = a.reshape(h, w)
        px = np.dstack([g, g, g, np.full((h, w), 255, np.uint8)])
    elif fmt == 6:      # IA88
        g = a.reshape(h, w, 2)
        px = np.dstack([g[:, :, 0], g[:, :, 0], g[:, :, 0], g[:, :, 1]])
    elif fmt == 8:      # A8
        g = a.reshape(h, w)
        px = np.dstack([np.full((h, w), 255, np.uint8)] * 3 + [g])
    else:
        raise ValueError(f'image format {fmt}')
    return Image.fromarray(np.ascontiguousarray(px).astype(np.uint8), 'RGBA')


def parse_vmt(text):
    """The parameters of a material as a dict (lower case keys): `$basetexture`, `$translucent`, `$alphatest`, `$color`, ..."""
    out = {}
    for m in re.finditer(r'"?(\$?[\w]+)"?\s+"([^"\n]*)"', text.replace('\r', '')):
        out.setdefault(m.group(1).lower(), m.group(2))
    for m in re.finditer(r'"?(\$?[\w]+)"?\s+(\S+)', text.replace('\r', '')):
        out.setdefault(m.group(1).lower(), m.group(2).strip('"'))
    return out
