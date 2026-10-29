"""Extract every texture of the game as PNG files, sorted into folders by what they belong to.

    python tools/extract_textures.py <PAK folder> <output folder>

The game's textures are `.qpm` files in MISC.PAK: `textures/NNNN.qpm` (numbered; used by the models and the levels) and `textures/misc/*.qpm`
(named; effects, fonts, menus, ...).  A `.qpm` is a text header of three lines (`Q8`, `width height`, a line of numbers), then the palette
(256 entries of red, green, blue, alpha; the alpha is 0..128, 128 = opaque) and the width * height palette indices (`M8` has the smaller
mip levels after them, `Q6` has no palette: width * height entries of red, green, blue, alpha).  The files are decoded at their own size
(the PNG has the same pixels as the game's picture, with the alpha scaled to 0..255).

Which model or level uses a numbered texture is read from the models: a model (`.raw` in OB.PAK, FRONTEND.PAK or BG.PAK) starts with
the offset of its table of textures (16 bytes per entry, the texture's number first, ended by -1).  A texture is stored once, in the folder of
its first owner in the order characters, weapons, objects, frontend, levels; `index.csv` lists all of its owners.

The pictures are the game's own: keep them to yourself (do not put them in a repository).
"""
import csv
import os
import re
import struct
import sys
from collections import defaultdict

import numpy as np
from PIL import Image

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from pakls import entries  # noqa: E402

def level_names():
    """The names of the levels by number, from the game's own table of them (the pointers at 0x314410, level 1 first: the names the command line takes), such as 'Prison' or
    'ChemicalPlant'; the placeholders of the game ('spare1') are left out."""
    import struct
    import ps2elf
    ram = ps2elf.data_image()
    names = {}
    for n in range(1, 31):
        at = struct.unpack_from('<I', ram, 0x314410 + 4 * (n - 1))[0]
        if not 0 < at < len(ram) - 40:
            continue
        end = at
        while ram[end] and end - at < 40:
            end += 1
        name = ''.join(c for c in bytes(ram[at:end]).decode('latin-1') if c.isalnum())
        if name and not name.lower().startswith('spare'):
            names[n] = name
    return names


LEVEL_NAMES = level_names()


def tileset_names():
    """The names of the five tile sets of the map maker by number (1 to 5), from the game's own table of texts (five pointers at 0x3145b4): Virtual, Industrial, Gothic, Alien and
    Spaceport."""
    import struct
    import ps2elf
    ram = ps2elf.data_image()
    names = {}
    for n in range(1, 6):
        at = struct.unpack_from('<I', ram, 0x3145b4 + 4 * (n - 1))[0]
        end = at
        while ram[end] and end - at < 40:
            end += 1
        names[n] = ''.join(c for c in bytes(ram[at:end]).decode('latin-1') if c.isalnum())
    return names


TILESET_NAMES = tileset_names()


def decode(b):
    """A .qpm file -> (kind, width, height, RGBA image)."""
    i = 0
    lines = []
    for _ in range(3):
        j = b.index(b'\n', i)
        lines.append(b[i:j].decode('latin1'))
        i = j + 1
    kind = lines[0].strip()
    w, h = (int(x) for x in lines[1].split())
    if kind in ('Q8', 'M8'):
        pal = np.frombuffer(b, dtype=np.uint8, count=1024, offset=i).reshape(256, 4)
        idx = np.frombuffer(b, dtype=np.uint8, count=w * h, offset=i + 1024).reshape(h, w)
        rgba = pal[idx].copy()
    elif kind == 'Q6':
        rgba = np.frombuffer(b, dtype=np.uint8, count=w * h * 4, offset=i).reshape(h, w, 4).copy()
    else:
        raise ValueError('unknown kind ' + kind)
    rgba[..., 3] = np.minimum(255, (rgba[..., 3].astype(np.int32) * 255 + 64) // 128)
    return kind, w, h, Image.fromarray(rgba, 'RGBA')


def model_textures(b):
    """The numbers of the textures in a model's table (None if the file has no such table)."""
    if len(b) < 32:
        return None
    start = struct.unpack_from('<I', b, 0)[0]
    if start != 16:
        return None
    out = []
    o = start
    while o + 16 <= len(b):
        n = struct.unpack_from('<i', b, o)[0]
        if n == -1:
            return out or None
        if not 0 <= n < 4000:
            return None
        out.append(n)
        o += 16
    return None


def category_of(path):
    """(priority, folder) of a model or level file."""
    p = path.lower()
    m = re.match(r'ob/chrs/([^/]+)\.raw$', p)
    if m:
        return 0, 'characters/' + m.group(1)
    m = re.match(r'ob/((?:chr)?gun\d+|ammo|cartridge\d+|grenade\d+|rocket\d+|remotemine|proximitymine|timedmine|tnt|brick)/', p)
    if m:
        return 1, 'weapons/' + m.group(1)
    m = re.match(r'ob/([^/]+)/', p)
    if m:
        return 2, 'objects/' + m.group(1)
    m = re.match(r'front/data/([^/]+)\.raw$', p)
    if m:
        return 3, 'frontend/' + m.group(1)
    m = re.match(r'bg/level(\d+)/', p)
    if m:
        n = int(m.group(1))
        return 4, 'levels/level%02d%s' % (n, '_' + LEVEL_NAMES[n] if n in LEVEL_NAMES else '')
    return 5, 'other/' + os.path.dirname(p).replace('/', '_')


MISC_RULES = [
    (r'font', 'fonts'), (r'^(mm_|mapmaker)', 'mapmaker'), (r'^(logo|loading|back|tunnel|tselec|tsblast|clouds)', 'menus'),
    (r'^ammo_', 'ammo'), (r'^bag_', 'bags'), (r'(bullethole|burnmark|splat|blood)', 'decals'),
    (r'(flare|glow|explo|smoke|star|beam|plasma|ellipse|fade|radar|spark|fire)', 'effects'), (r'^seaframe', 'sea'),
]


def misc_folder(name):
    n = name.lower()
    for rx, folder in MISC_RULES:
        if re.search(rx, n):
            return 'misc/' + folder
    return 'misc/other'


def main():
    if len(sys.argv) != 3:
        print(__doc__)
        sys.exit(1)
    pak_dir, out_dir = sys.argv[1], sys.argv[2]
    misc, misc_entries = entries(os.path.join(pak_dir, 'MISC.PAK'))
    qpm = {n: misc[o:o + s] for n, o, s in misc_entries if n.lower().endswith('.qpm')}
    owners = defaultdict(list)  # texture number -> [(priority, folder, model path)]
    for pack in ('OB', 'FRONTEND', 'BG'):
        d, es = entries(os.path.join(pak_dir, pack + '.PAK'))
        for n, o, s in es:
            if not n.lower().endswith('.raw'):
                continue
            t = model_textures(d[o:o + s])
            if t:
                pri, folder = category_of(n)
                for k in t:
                    owners[k].append((pri, folder, n))
    rows = []
    done = 0
    for name, data in sorted(qpm.items()):
        try:
            kind, w, h, img = decode(data)
        except Exception as e:  # noqa: BLE001
            print('cannot decode', name, e)
            continue
        base = os.path.basename(name)[:-4]
        if name.lower().startswith('textures/misc/'):
            folder, used = misc_folder(base), ''
        else:
            num = int(base)
            own = sorted(owners.get(num, []))
            folder = own[0][1] if own else 'unreferenced'
            used = ' | '.join(sorted({f'{f}' for _, f, _ in own}))
        path = os.path.join(out_dir, folder, base + '.png')
        os.makedirs(os.path.dirname(path), exist_ok=True)
        img.save(path, optimize=True)
        rows.append([name, kind, w, h, folder, used])
        done += 1
    with open(os.path.join(out_dir, 'index.csv'), 'w', newline='', encoding='utf-8') as f:
        wr = csv.writer(f)
        wr.writerow(['texture', 'format', 'width', 'height', 'folder', 'used by'])
        wr.writerows(rows)
    by = defaultdict(int)
    for r in rows:
        by[r[4].split('/')[0]] += 1
    print(done, 'textures written to', out_dir, dict(by))


if __name__ == '__main__':
    main()
