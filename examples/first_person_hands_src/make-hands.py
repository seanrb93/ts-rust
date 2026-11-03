"""Make the first person hands mod: a hand and forearm (the ones of the game's character, `ob/chrs/chr01.raw`) are added to the model of a weapon as it is seen in the first person
(`ob/gunN/gunN.raw`), put where the weapon is held.

    python make-hands.py <OB.PAK> <mods folder>/first_person_hands_mod [weapon ...]      (the weapons are the names in WEAPONS; all of them when none is named)

The tools it uses are `tools/obdump.py` (a model as an OBJ) and `tools/obedit.py meshes` (an OBJ back into a model).  The hand is put on a part of the gun that does not move on its own
(the body), or, for the forend of a pump gun, on the part that slides.  Each hand is a rotation, a scale and a place: see WEAPONS.
"""
import math
import os
import subprocess
import sys

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
TOOLS = os.path.join(HERE, '..', '..', 'tools')
TEX_BASE = 9400

# For each weapon: the file, and its hands: (part of the gun that carries it, the part of the character that is added (12 the right hand, 11 and 10 the forearm and the elbow...),
# rotation about x, y, z in degrees, scale, place (x, y, z) in the gun's own space).
WEAPONS = {
    'pistol': {
        'file': 'ob/gun1/gun1.raw',
        'hands': [
            ('part3', ['20', '19'], (-90, 0, 0), 1.0, (-0.022, -0.03, -0.045)),
        ],
    },
    'm16': {
        'file': 'ob/gun2/gun2.raw',
        'hands': [
            ('part0', ['20', '19'], (-90, 0, 0), 1.0, (-0.03, -0.14, -0.09)),
            ('part0', ['20', '19'], (-90, 0, 90), 1.0, (0.0, -0.065, 0.17)),
        ],
    },
    'shotgun': {
        'file': 'ob/gun25/gun25.raw',
        'hands': [
            ('part0', ['20', '19'], (-90, 0, 0), 1.0, (-0.03, -0.07, -0.04)),
            ('part2', ['20', '19'], (-90, 0, 90), 1.0, (0.0, 0.0, 0.22)),
        ],
    },
}


def load_obj(path):
    verts, colours, uvs, parts = [], [], [], {}
    cur = None
    for line in open(path):
        w = line.split()
        if not w:
            continue
        if w[0] == 'v':
            verts.append([float(x) for x in w[1:4]])
            colours.append([float(x) for x in w[4:7]] if len(w) >= 7 else [1.0, 1.0, 1.0])
        elif w[0] == 'vt':
            uvs.append([float(w[1]), float(w[2])])
        elif w[0] in ('o', 'g'):
            cur = w[1]
        elif w[0] == 'f':
            corners = []
            for c in w[1:]:
                i = c.split('/')
                corners.append((int(i[0]) - 1, int(i[1]) - 1 if len(i) > 1 and i[1] else 0))
            parts.setdefault(cur, []).append(corners)
    return np.array(verts), np.array(colours), np.array(uvs), parts


def rotation(rx, ry, rz):
    a, b, c = (math.radians(d) for d in (rx, ry, rz))
    Rx = np.array([[1, 0, 0], [0, math.cos(a), -math.sin(a)], [0, math.sin(a), math.cos(a)]])
    Ry = np.array([[math.cos(b), 0, math.sin(b)], [0, 1, 0], [-math.sin(b), 0, math.cos(b)]])
    Rz = np.array([[math.cos(c), -math.sin(c), 0], [math.sin(c), math.cos(c), 0], [0, 0, 1]])
    return Rz @ Ry @ Rx


def skin_picture(path):
    img = Image.new('RGBA', (32, 32))
    px = img.load()
    for y in range(32):
        for x in range(32):
            shade = 1.0 - 0.06 * ((x * 7 + y * 3) % 5) / 4
            px[x, y] = (int(255 * shade), int(214 * shade), int(186 * shade), 255)
    img.save(path)


def read_blocks(path):
    """The objects of an OBJ written by obdump: name -> list of (material, faces) where a face is a list of (position, coordinate, normal or None) indices; and the arrays."""
    v, vt, vn, blocks = [], [], [], {}
    cur, mat = None, None
    for line in open(path):
        w = line.split()
        if not w:
            continue
        if w[0] == 'v':
            v.append(line.strip())
        elif w[0] == 'vt':
            vt.append(line.strip())
        elif w[0] == 'vn':
            vn.append(line.strip())
        elif w[0] in ('o', 'g'):
            cur = w[1]
        elif w[0] == 'usemtl':
            mat = w[1]
        elif w[0] == 'f':
            face = []
            for c in w[1:]:
                i = (c.split('/') + ['', ''])[:3]
                face.append((int(i[0]) - 1, int(i[1]) - 1 if i[1] else None, int(i[2]) - 1 if i[2] else None))
            blocks.setdefault(cur, []).append((mat, face))
    return v, vt, vn, blocks


def add_hands(gun_obj, out_obj, hands, chr_verts, chr_cols, chr_parts):
    """An OBJ with only the parts of the gun that carry a hand (the others are left as the game has them), each with its hands added."""
    v, vt, vn, blocks = read_blocks(gun_obj)
    out = ['mtllib hands.mtl']
    nv = nt = nn = 0
    carriers = []
    for h in hands:
        if h[0] not in carriers:
            carriers.append(h[0])
    for part in carriers:
        out.append('o ' + part)
        remap_v, remap_t, remap_n = {}, {}, {}
        lines, faces = [], []
        mat = None
        for m, face in blocks.get(part, []):
            if m != mat:
                faces.append('usemtl ' + m)
                mat = m
            corner = []
            for vi, ti, ni in face:
                if vi not in remap_v:
                    remap_v[vi] = nv = nv + 1
                    lines.append(v[vi])
                if ti is not None and ti not in remap_t:
                    remap_t[ti] = nt = nt + 1
                    lines.append(vt[ti])
                if ni is not None and ni not in remap_n:
                    remap_n[ni] = nn = nn + 1
                    lines.append(vn[ni])
                corner.append('%d/%s/%s' % (remap_v[vi], remap_t[ti] if ti is not None else '', remap_n[ni] if ni is not None else ''))
            faces.append('f ' + ' '.join(corner))
        out += lines + faces
        for hand_part, sources, rot, scale, place in hands:
            if hand_part != part:
                continue
            R = rotation(*rot) * scale
            out.append('usemtl ' + os.environ.get('HAND_MAT', 'skin'))
            out.append('vt 0.5 0.5')
            nt += 1
            hand_faces = []
            first, middle = True, None
            for src in sources:
                used = {}
                for f in chr_parts.get('part' + src, []):
                    for vi, _ in f:
                        if vi not in used:
                            used[vi] = len(used)
                base = nv
                # (the parts of the character are in the character's own space: the hand is turned about its own middle, which goes to `place`)
                middle = chr_verts[list(used)].mean(axis=0) if first else middle
                first = False
                for vi in sorted(used, key=used.get):
                    p = R @ (chr_verts[vi] - middle) + np.array(place)
                    c = chr_cols[vi]
                    out.append('v %.6f %.6f %.6f %.3f %.3f %.3f' % (p[0], p[1], p[2], c[0], c[1], c[2]))
                nv += len(used)
                for f in chr_parts.get('part' + src, []):
                    hand_faces.append('f ' + ' '.join('%d/%d' % (base + used[vi] + 1, nt) for vi, _ in f))
            out += hand_faces
    open(out_obj, 'w').write(chr(10).join(out) + chr(10))


def main():
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    pak, mod = sys.argv[1], sys.argv[2]
    names = sys.argv[3:] or list(WEAPONS)
    work = os.path.join(HERE, 'work')
    os.makedirs(work, exist_ok=True)
    skin_picture(os.path.join(work, 'skin.png'))
    open(os.path.join(work, 'hands.mtl'), 'w').write('newmtl skin\nmap_Kd skin.png\n')
    subprocess.check_call([sys.executable, os.path.join(TOOLS, 'obdump.py'), pak, 'ob/chrs/chr01.raw', '--obj', os.path.join(work, 'chr01.obj')], stdout=subprocess.DEVNULL)
    cv, cc, _, cp = load_obj(os.path.join(work, 'chr01.obj'))
    for name in names:
        spec = WEAPONS[name]
        if os.environ.get('HANDS_JSON'):
            import json
            spec = dict(spec, hands=[tuple(h) for h in json.loads(os.environ['HANDS_JSON'])])
        base = os.path.join(work, name)
        subprocess.check_call([sys.executable, os.path.join(TOOLS, 'obdump.py'), pak, spec['file'], '--obj', base + '.obj'], stdout=subprocess.DEVNULL)
        add_hands(base + '.obj', base + '_hands.obj', spec['hands'], cv, cc, cp)
        subprocess.check_call([sys.executable, os.path.join(TOOLS, 'obedit.py'), pak, spec['file'], mod, 'meshes', base + '_hands.obj', '--tex-base', str(TEX_BASE + 10 * list(WEAPONS).index(name))])
    open(os.path.join(mod, 'mod.ini'), 'w').write('enabled = true\n')


main()
