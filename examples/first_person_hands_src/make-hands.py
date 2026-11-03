"""Make the first person hands mod: a hand and forearm (the ones of the game's character, `ob/chrs/chr01.raw`) are added to the model of a weapon as it is seen in the first person
(`ob/gunN/gunN.raw`), put where the weapon is held.

    python make-hands.py <OB.PAK> <mods folder>/first_person_hands_mod [weapon ...]      (the weapons are the names in WEAPONS; all of them when none is named).
The hands have the skin and sleeve of a character of the game: HANDS_CHARACTER=<number> (the character's number, 0 is Det. Harry Tipper, the default)

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
# where the elbow of each arm is from its hand, in the space of the gun (x right, y up, z forward): the arms come into the view from the bottom of the screen
GRIP_FROM = (0.5, -0.55, -0.65)
FORE_FROM = (-0.5, -0.45, -0.7)
GRIP_ROT = (-90, 0, 90)      # the hand of the grip: the palm is upright, the thumb up, as a hand holds a grip
FORE_ROT = (-90, 0, 180)     # the hand under the front: the palm up, under the gun, the fingers forward
WEAPONS = {
    'pistol': dict(file='ob/gun1/gun1.raw', part='part3', kind='one', grip=(-0.022, -0.04, -0.04)),
    'scifi_handgun': dict(file='ob/gun8/gun8.raw', part='part0', kind='one', grip=(-0.02, -0.06, -0.06)),
    'mauser': dict(file='ob/gun10/gun10.raw', part='part0', kind='one', grip=(-0.02, -0.11, 0.05)),
    'colt': dict(file='ob/gun24/gun24.raw', part='part0', kind='one', grip=(-0.03, -0.12, -0.04)),
    'uzi': dict(file='ob/gun22/gun22.raw', part='part0', kind='one', grip=(-0.02, -0.09, -0.04)),
    'm16': dict(file='ob/gun2/gun2.raw', part='part0', kind='two', grip=(-0.02, -0.10, -0.05), fore=(0.01, -0.035, 0.36)),
    'assault_rifle': dict(file='ob/gun3/gun3.raw', part='part0', kind='two', grip=(-0.02, -0.06, 0.0), fore=(0.01, -0.035, 0.27)),
    'tommy_gun': dict(file='ob/gun20/gun20.raw', part='part0', kind='two', grip=(-0.02, -0.08, -0.06), fore=(0.01, -0.09, 0.33)),
    'shotgun': dict(file='ob/gun25/gun25.raw', part='part0', kind='two', grip=(-0.02, -0.04, -0.03), fore=(0.01, -0.02, 0.3), fore_part='part2'),
    'assault_shotgun': dict(file='ob/gun21/gun21.raw', part='part0', kind='two', grip=(-0.02, -0.08, -0.05), fore=(0.01, -0.02, 0.3)),
    'minigun': dict(file='ob/gun11/gun11.raw', part='part0', kind='two', grip=(-0.02, -0.04, 0.08), fore=(0.01, -0.02, 0.3)),
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


def rotation_between(a, b):
    """The rotation that turns the direction `a` to the direction `b` (Rodrigues), as a 3 by 3 matrix."""
    a = np.array(a, dtype=float) / np.linalg.norm(a)
    b = np.array(b, dtype=float) / np.linalg.norm(b)
    v = np.cross(a, b)
    c = float(np.dot(a, b))
    if c > 0.9999999:
        return np.eye(3)
    if c < -0.9999999:
        return -np.eye(3)
    k = np.array([[0, -v[2], v[1]], [v[2], 0, -v[0]], [-v[1], v[0], 0]])
    return np.eye(3) + k + k @ k / (1.0 + c)


def hands_for(spec, gun_obj):
    if os.environ.get('HANDS_STYLE', 'rest') == 'rest':
        # (the hand of the character as its model has it, turned by hand to hold the gun: tidy, and the first version's look)
        hands = [(spec['part'], ['20', '19'], spec.get('grip_rot', GRIP_ROT), 1.0, tuple(spec['grip']), None)]
        if spec['kind'] == 'two':
            # (the front hand is on the side of the gun that the view sees, the arm reaching in from the lower left: in the widescreen picture the grip and what is under the gun are off the screen)
            lo, hi = body_box(gun_obj, spec['fore_part'] if 'fore_part' in spec else spec['part'])
            fore = (hi[0] + 0.03, spec['fore'][1] + 0.05, spec['fore'][2])
            hands = [h for h in hands]
            hands.append((spec.get('fore_part', spec['part']), ['12', '11', '10', '9'], (0, 0, 0), 1.0, fore, (0.6, -0.6, -0.5), 0))
        return hands
    """The hands of a weapon as (part, parts of the character, rotation, scale, place): the hand of the grip and, for a two handed gun, the hand under the front."""
    # (the arms are the character's own, in the pose of its animation for a gun held in both hands: the part 20 is the hand of the grip, 12 the hand under the front)
    hands = [(spec['part'], ['20', '19', '18', '17'], spec.get('grip_rot', (0, 0, 0)), 1.0, tuple(spec['grip']), spec.get('grip_from', GRIP_FROM), spec.get('grip_roll', 0))]
    if spec['kind'] == 'two':
        hands.append((spec.get('fore_part', spec['part']), ['12', '11', '10', '9'], spec.get('fore_rot', (0, 0, 0)), 1.0, tuple(spec['fore']), spec.get('fore_from', FORE_FROM), spec.get('fore_roll', 180)))
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


CHR_UVS = None


def skin_picture_old(path):
    img = Image.new('RGBA', (32, 32))
    px = img.load()
    for y in range(32):
        for x in range(32):
            shade = 1.0 - 0.06 * ((x * 7 + y * 3) % 5) / 4
            px[x, y] = (int(255 * shade), int(214 * shade), int(186 * shade), 255)
    img.save(path)


def skin_from_character(pak, chr_file, out_png):
    """The picture the hand of a character has: the texture of the slot the hand parts use (6) in the character's own table, taken from `MISC.PAK` (the game's own picture: it is
    written into the mod's folder to be used by the mod, not kept in the repository)."""
    import struct
    sys.path.insert(0, TOOLS)
    import pakls
    from extract_textures import decode
    d, es = pakls.entries(pak)
    n, o, sz = [e for e in es if e[0] == chr_file][0]
    data = d[o:o + sz]
    numbers, at = [], struct.unpack_from('<I', data, 0)[0]
    while struct.unpack_from('<i', data, at)[0] != -1:
        numbers.append(struct.unpack_from('<i', data, at)[0])
        at += 16
    slot = int(os.environ.get('HANDS_SLOT', '6'))
    number = numbers[slot]
    d2, es2 = pakls.entries(os.path.join(os.path.dirname(pak), 'MISC.PAK'))
    hit = [e for e in es2 if e[0] == 'textures/%04d.qpm' % number][0]
    _, _, _, img = decode(d2[hit[1]:hit[1] + hit[2]])
    img.save(out_png)
    print('hand skin: texture', number, 'of', chr_file, img.size)


def skin_picture(path):
    skin_picture_old(path)


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
        for hand_part, sources, rot, scale, place, *arm_from in hands:
            if hand_part != part:
                continue
            R = rotation(*rot) * scale
            if arm_from and arm_from[0] is not None:
                # the arm is turned about its hand so that the elbow is where `arm_from` says
                hand = chr_verts[sorted({vi for f in chr_parts.get('part' + sources[0], []) for vi, _ in f})].mean(axis=0)
                elbow = chr_verts[sorted({vi for f in chr_parts.get('part' + sources[-1], []) for vi, _ in f})].mean(axis=0)
                R = rotation_between(elbow - hand, arm_from[0]) @ R
                if len(arm_from) > 1 and arm_from[1]:
                    # (the roll of the hand about the arm, degrees)
                    t = math.radians(arm_from[1])
                    ax = np.array(arm_from[0], dtype=float) / np.linalg.norm(arm_from[0])
                    k = np.array([[0, -ax[2], ax[1]], [ax[2], 0, -ax[0]], [-ax[1], ax[0], 0]])
                    R = (np.eye(3) + math.sin(t) * k + (1 - math.cos(t)) * (k @ k)) @ R
            out.append('usemtl ' + os.environ.get('HAND_MAT', 'skin'))
            hand_faces = []
            first, middle = True, None
            for src in sources:
                used, uv_of = {}, {}
                for f in chr_parts.get('part' + src, []):
                    for vi, ti in f:
                        if vi not in used:
                            used[vi] = len(used)
                            uv_of[vi] = ti
                base = nv
                base_t = nt
                # (the parts of the character are in the character's own space: the hand is turned about its own middle, which goes to `place`)
                middle = chr_verts[list(used)].mean(axis=0) if first else middle
                first = False
                for vi in sorted(used, key=used.get):
                    p = R @ (chr_verts[vi] - middle) + np.array(place)
                    c = chr_cols[vi]
                    out.append('v %.6f %.6f %.6f %.3f %.3f %.3f' % (p[0], p[1], p[2], c[0], c[1], c[2]))
                    out.append('vt %.6f %.6f' % tuple(CHR_UVS[uv_of[vi]]))
                nv += len(used)
                nt += len(used)
                for f in chr_parts.get('part' + src, []):
                    hand_faces.append('f ' + ' '.join('%d/%d' % (base + used[vi] + 1, base_t + used[vi] + 1) for vi, _ in f))
            out += hand_faces
    open(out_obj, 'w').write(chr(10).join(out) + chr(10))


ARM_PARTS = (9, 10, 11, 12, 17, 18, 19, 20)


def character_files(pak):
    """The characters of the game whose model has the arms of the humanoid layout (the parts 9 to 12 and 17 to 20 of the same kinds and joints as the first character's): character
    number -> (file in the pack, number of the texture the hand has).  The others (aliens, robots) have no arms of their own here and keep the first character's."""
    import struct
    sys.path.insert(0, TOOLS)
    import pakls
    import ps2elf
    ram = ps2elf.data_image()
    d, es = pakls.entries(pak)
    by_name = {e[0]: e for e in es}
    base = 0x35bb74 - 390 * 28  # (the table of props: 28 bytes each, the pistol's first person model is the prop 390)

    def prop_file(n):
        w = struct.unpack_from('<7I', ram, base + 28 * n)
        return bytes(ram[w[0]:w[0] + 60]).split(bytes([0]))[0].decode('latin1').replace('host0:', '')

    def layout(name):
        n, o, sz = by_name[name]
        f = d[o:o + sz]
        tab = struct.unpack_from('<I', f, 4)[0]
        cnt = struct.unpack_from('<I', f, tab)[0]
        bones = tab - cnt * 80
        parts = [(f[bones + 80 * i], f[bones + 80 * i + 1]) for i in range(cnt)] if cnt > 20 else None
        if parts is None:
            return None
        # (the texture the hand part uses: the slot of its first chunk, and the table of the file)
        hand = bones + 80 * 20
        list_at = struct.unpack_from('<I', f, hand + 8)[0]
        slot = struct.unpack_from('<i', f, list_at)[0]
        numbers, at = [], struct.unpack_from('<I', f, 0)[0]
        while struct.unpack_from('<i', f, at)[0] != -1:
            numbers.append(struct.unpack_from('<i', f, at)[0])
            at += 16
        return tuple(parts[i] for i in ARM_PARTS), numbers[slot]

    first = layout('ob/chrs/chr01.raw')
    out = {}
    for c in range(80):
        model = struct.unpack_from('<i', ram, 0x3650e0 + 44 * c + 8)[0]
        if not 0 <= model < 1100:
            continue
        name = prop_file(model)
        if name in by_name and name.endswith('.raw') and '/chrs/' in name:
            try:
                got = layout(name)
            except Exception:
                continue
            if got and got[0] == first[0]:
                out[c] = (name, got[1])
    return out


def character_poses(pak, character):
    """The matrices of the joints of a character in the pose of the animation for a gun in both hands and in its standing pose (the one its model is skinned from): the files the port writes
    for `TS_DUMPARM` when the game is run with that character and that animation (kept in `poses/` after the first time; the game is `ts-rust.exe` in the folder of the pack)."""
    folder = os.path.join(HERE, 'poses', 'characters')
    os.makedirs(folder, exist_ok=True)
    out = []
    game = os.path.dirname(os.path.dirname(os.path.abspath(pak)))
    for animation, tag in ((478, 'two'), (3, 'rest')):
        path = os.path.join(folder, 'c%02d-%s.txt' % (character, tag))
        if not os.path.exists(path):
            env = dict(os.environ, TS_PLAYERCHR=str(character), TS_DUMPARM=path, TS_HANDANIM=str(animation), TS_GIVE='100:3', TS_EQUIP='1')
            subprocess.run([os.path.join(game, 'ts-rust.exe'), '--level', 'Prison', '--players', '1', '--frames', '270', '--headless'], cwd=game, env=env, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=300)
        out.append(path)
    return out


def bake_character(pak, mod, character, chr_file, skin_number, names, work):
    """The first person models of `names` with the arms of `character` on them: written as `gunN.raw` for the first character (the models every character shares), as `gunN_cNN.raw` for the others."""
    global CHR_UVS
    import obedit
    obj = os.path.join(work, 'character%02d.obj' % character)
    if os.environ.get('HANDS_STYLE', 'rest') == 'rest':
        subprocess.check_call([sys.executable, os.path.join(TOOLS, 'obdump.py'), pak, chr_file, '--obj', obj], stdout=subprocess.DEVNULL)
    else:
        two, rest = character_poses(pak, character)
        env = dict(os.environ, REST_MATRICES=rest)
        subprocess.check_call([sys.executable, os.path.join(HERE, 'pose-character.py'), pak, chr_file, two, obj], env=env, stdout=subprocess.DEVNULL)
    cv, cc, CHR_UVS, cp = load_obj(obj)
    d, es = pakls_entries(pak)
    for name in names:
        spec = WEAPONS[name]
        base = os.path.join(work, name)
        if not os.path.exists(base + '.obj'):
            subprocess.check_call([sys.executable, os.path.join(TOOLS, 'obdump.py'), pak, spec['file'], '--obj', base + '.obj'], stdout=subprocess.DEVNULL)
        spec = dict(spec, hands=hands_for(spec, base + '.obj'))
        if os.environ.get('HANDS_JSON'):
            import json
            spec = dict(spec, hands=[tuple(h) for h in json.loads(os.environ['HANDS_JSON'])])
        add_hands(base + '.obj', base + '_hands%02d.obj' % character, spec['hands'], cv, cc, cp)
        groups = {}
        faces, _ = obedit.read_obj(base + '_hands%02d.obj' % character, groups)
        n, o, sz = [e for e in es if e[0] == spec['file']][0]
        data = d[o:o + sz]
        for part_name, fs in groups.items():
            if part_name and part_name.startswith('part') and part_name[4:].isdigit():
                data, _count = obedit.mesh_part(data, fs, int(part_name[4:]), {'skin': skin_number})
        out_name = spec['file'] if character == 0 else spec['file'].replace('.raw', '_c%02d.raw' % character)
        out = os.path.join(mod, 'OB.PAK', out_name)
        os.makedirs(os.path.dirname(out), exist_ok=True)
        open(out, 'wb').write(data)


def pakls_entries(pak):
    sys.path.insert(0, TOOLS)
    import pakls
    return pakls.entries(pak)


def main():
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    pak, mod = sys.argv[1], sys.argv[2]
    names = sys.argv[3:] or list(WEAPONS)
    work = os.path.join(HERE, 'work')
    os.makedirs(work, exist_ok=True)
    sys.path.insert(0, TOOLS)
    chars = character_files(pak)
    if os.environ.get('HANDS_CHARACTER'):
        chars = {int(os.environ['HANDS_CHARACTER']): chars[int(os.environ['HANDS_CHARACTER'])]}
    print(len(chars), 'characters with arms of their own:', sorted(chars))
    # (the first character's arms are the models every character without arms of its own has)
    for c in sorted(chars):
        name, skin = chars[c]
        bake_character(pak, mod, c, name, skin, names, work)
    if 0 not in chars:
        name, skin = character_files(pak)[0]
        bake_character(pak, mod, 0, name, skin, names, work)
    os.makedirs(mod, exist_ok=True)
    open(os.path.join(mod, 'mod.ini'), 'w').write('enabled = true' + chr(10))


main()
