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

# The weapons: the file of the gun as it is seen in the first person, the part of it that is the body (the part the hands are put on: it does not move on its own), the kind
# ('one' a hand on the grip, 'two' a hand on the grip and one under the front) and, where the first try is not right, the places (x, y, z in the gun's own space) of the hands:
# 'grip' and 'fore', and 'fore_part' when the front hand rides on another part (the sliding forend of a pump gun).  The first try is made from the box of the body.
WEAPONS = {
    'pistol': dict(file='ob/gun1/gun1.raw', part='part3', kind='one', grip=(-0.022, -0.03, -0.045)),
    'scifi_handgun': dict(file='ob/gun8/gun8.raw', part='part0', kind='one'),
    'mauser': dict(file='ob/gun10/gun10.raw', part='part0', kind='one'),
    'colt': dict(file='ob/gun24/gun24.raw', part='part0', kind='one'),
    'uzi': dict(file='ob/gun22/gun22.raw', part='part0', kind='one'),
    'm16': dict(file='ob/gun2/gun2.raw', part='part0', kind='two', grip=(-0.03, -0.14, -0.09), fore=(0.0, -0.065, 0.17)),
    'assault_rifle': dict(file='ob/gun3/gun3.raw', part='part0', kind='two'),
    'tommy_gun': dict(file='ob/gun20/gun20.raw', part='part0', kind='two'),
    'shotgun': dict(file='ob/gun25/gun25.raw', part='part0', kind='two', grip=(-0.03, -0.07, -0.04), fore=(0.0, 0.0, 0.22), fore_part='part2'),
    'assault_shotgun': dict(file='ob/gun21/gun21.raw', part='part0', kind='two'),
    'minigun': dict(file='ob/gun11/gun11.raw', part='part0', kind='two'),
}


def body_box(gun_obj, part):
    v, parts = [], {}
    cur = None
    for line in open(gun_obj):
        w = line.split()
        if not w:
            continue
        if w[0] == 'v':
            v.append([float(x) for x in w[1:4]])
        elif w[0] == 'o':
            cur = w[1]
        elif w[0] == 'f':
            parts.setdefault(cur, []).extend(int(x.split('/')[0]) - 1 for x in w[1:])
    pts = np.array(v)[np.unique(parts[part])]
    return pts.min(axis=0), pts.max(axis=0)


def hands_for(spec, gun_obj):
    """The hands of a weapon as (part, parts of the character, rotation, scale, place): from `spec`, with the places that it does not give made from the box of the body."""
    lo, hi = body_box(gun_obj, spec['part'])
    size = hi - lo
    grip = spec.get('grip') or (-0.025, lo[1] + 0.085, lo[2] + 0.26 * size[2] if spec['kind'] == 'one' else lo[2] + 0.22 * size[2])
    hands = [(spec['part'], ['20', '19'], (-90, 0, 0), 1.0, tuple(grip))]
    if spec['kind'] == 'two':
        fore = spec.get('fore') or (0.0, lo[1] + 0.07, lo[2] + 0.62 * size[2])
        hands.append((spec.get('fore_part', spec['part']), ['20', '19'], (-90, 0, 90), 1.0, tuple(fore)))
    return hands


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
        base = os.path.join(work, name)
        subprocess.check_call([sys.executable, os.path.join(TOOLS, 'obdump.py'), pak, spec['file'], '--obj', base + '.obj'], stdout=subprocess.DEVNULL)
        spec = dict(spec, hands=hands_for(spec, base + '.obj'))
        if os.environ.get('HANDS_JSON'):
            import json
            spec = dict(spec, hands=[tuple(h) for h in json.loads(os.environ['HANDS_JSON'])])
        add_hands(base + '.obj', base + '_hands.obj', spec['hands'], cv, cc, cp)
        subprocess.check_call([sys.executable, os.path.join(TOOLS, 'obedit.py'), pak, spec['file'], mod, 'meshes', base + '_hands.obj', '--tex-base', str(TEX_BASE)])
    open(os.path.join(mod, 'mod.ini'), 'w').write('enabled = true\n')


main()
