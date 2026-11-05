"""Make the first person hands mod: what the program needs to put the arms of each character on the first person guns when a gun is loaded (`ts-game/src/manual/armmerge.rs`).

    python make-hands.py <OB.PAK> <mods folder>/first_person_hands_mod

Writes the mod's folder `arms`: `c<NN>.arm` for each character, the triangles of its arms as they are in its model, and `guns.txt`, how each gun is held.  How each hand holds its weapon is in WEAPONS: the hand is turned to the weapon and its
fingers bent round it, and the forearm comes in from where it would in real life.  The gun is whichever the packs have when the game runs, so another mod's guns get the arms too.  HANDS_CHARACTER=<number> makes one character only.
"""
import math
import os
import shutil
import struct
import subprocess
import sys

import numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
TOOLS = os.path.join(HERE, '..', '..', 'tools')
WORK = os.path.join(HERE, 'work')
TEX_BASE = 9400

# The weapons, held as they are in real life.  The space of a gun: +x is to the left on the screen, y up, z forward (to the muzzle).
# file: the first person model; part: the part the hands go on (the one that does not move on its own); kind: 'one' or 'two' hands.
# grip=(y, z): the middle of what the right hand holds; rake: how far the grip leans back from upright (degrees).  The hand lies on the right of it with its knuckles at its front, and the
# fingers curl round it (how far across it is, is measured from the model).
# fore=z: how far forward the left hand holds: palm up under the gun, the fingers up its far side, the thumb on the near side; fore_part: the part it rides on (the sliding forend of a pump gun);
# fore_above: a height below which something hangs at that distance that the hand does not go under.
# foregrip=(y, z): an upright grip for the left hand, held as the right hand holds its grip.
# arm: where the elbow of the right arm is, when not GRIP_ARM.
# palm=(x, lift, z) with fingers=(x, y, z) and thumb=(x, y, z): something that is carried rather than gripped, lying on the upturned right palm, which is `lift` above its underside at (x, z).
CURL_LIMIT = 150.0  # how far round the fingers bend, degrees
PALM_SEAT = 0.015  # how far the palm of the left hand goes up into the gun, so that no gap shows under it
GRIP_REACH = 0.05  # how far from the middle of a grip counts as the grip, along the gun
WRIST_BEND = 0.14  # how much of the arm the bend at the wrist takes
GRIP_ARM = (-0.35, -0.2, -0.92)  # where the elbow of the right arm is from its wrist: back, down and to the right of the screen, nearly along the gun
WRIST_ARM = (-0.45, -0.65, -0.6)  # the right arm when the hand is on the wrist of a stock: the elbow is lower
FORE_ARM = (0.6, -0.45, -0.7)   # and of the left arm, which comes across from the lower left
WEAPONS = {
    'pistol': dict(file='ob/gun1/gun1.raw', part='part3', kind='one', grip_scale=0.9, grip=(-0.055, -0.055), rake=25),
    'raygun': dict(file='ob/gun5/gun5.raw', part='part0', kind='one', grip_scale=0.9, grip=(-0.06, -0.065), rake=25),
    'scifi_handgun': dict(file='ob/gun8/gun8.raw', part='part0', kind='one', grip_scale=0.9, grip=(-0.07, -0.065), rake=25),
    'mauser': dict(file='ob/gun10/gun10.raw', part='part0', kind='one', grip_scale=0.9, grip=(-0.11, 0.035), rake=15),
    'colt': dict(file='ob/gun24/gun24.raw', part='part0', kind='one', grip_scale=0.9, grip=(-0.11, 0.0), rake=30),
    'uzi': dict(file='ob/gun22/gun22.raw', part='part0', kind='one', grip_scale=0.9, grip=(-0.05, -0.03), rake=5),
    'm16': dict(file='ob/gun2/gun2.raw', part='part0', kind='two', grip=(-0.05, -0.07), rake=25, fore=0.26),
    'assault_rifle': dict(file='ob/gun3/gun3.raw', part='part0', kind='two', grip=(0.0, 0.05), rake=10, fore=0.25),
    'sniper_rifle': dict(file='ob/gun4/gun4.raw', part='part0', kind='two', grip=(-0.06, -0.06), rake=30, fore=0.38),
    'raygun_carbine': dict(file='ob/gun6/gun6.raw', part='part0', kind='two', grip=(-0.06, -0.035), rake=25, fore=0.2),
    'scifi_autorifle': dict(file='ob/gun7/gun7.raw', part='part0', kind='two', grip=(-0.05, -0.09), rake=30, fore=0.18),
    'rocket_launcher': dict(file='ob/gun9/gun9.raw', part='part0', kind='two', grip=(-0.02, -0.02), rake=10, fore=0.25),
    'minigun': dict(file='ob/gun11/gun11.raw', part='part0', kind='two', grip=(0.015, 0.035), rake=5, fore=0.19),
    'grenade_launcher': dict(file='ob/gun12/gun12.raw', part='part0', kind='two', grip=(-0.06, -0.07), rake=25, fore=0.27),
    'scifi_sniper_rifle': dict(file='ob/gun13/gun13.raw', part='part0', kind='two', grip=(-0.04, -0.06), rake=25, fore=0.3),
    'weapon_13': dict(file='ob/gun14/gun14.raw', part='part0', kind='two', grip=(-0.06, -0.07), rake=25, fore=0.27),
    'proximity_mine': dict(file='ob/gun15/gun15.raw', part='part1', kind='one', palm=(0.0, 0.0, 0.0), fingers=(0.5, 0.0, 0.85), thumb=(-0.85, 0.0, 0.5)),
    'remote_mine': dict(file='ob/gun16/gun16.raw', part='part0', kind='one', palm=(0.0, 0.0, 0.0), fingers=(0.5, 0.0, 0.85), thumb=(-0.85, 0.0, 0.5)),
    'timed_mine': dict(file='ob/gun17/gun17.raw', part='part2', kind='one', palm=(0.0, 0.02, 0.0), fingers=(0.5, 0.0, 0.85), thumb=(-0.85, 0.0, 0.5)),
    'weapon_17': dict(file='ob/gun18/gun18.raw', part='part0', kind='one', palm=(0.0, 0.0, 0.0), fingers=(0.5, 0.0, 0.85), thumb=(-0.85, 0.0, 0.5)),
    'tnt': dict(file='ob/gun19/gun19.raw', part='part0', kind='one', palm=(0.0, 0.02, 0.06), fingers=(1.0, 0.0, 0.25), thumb=(-0.25, 0.0, 1.0)),
    'tommy_gun': dict(file='ob/gun20/gun20.raw', part='part0', kind='two', grip=(-0.06, -0.09), rake=30, foregrip=(-0.05, 0.32)),
    'assault_shotgun': dict(file='ob/gun21/gun21.raw', part='part0', kind='two', grip=(-0.055, -0.04), rake=20, fore=0.42, fore_part='part2'),
    'blunderbuss': dict(file='ob/gun23/gun23.raw', part='part0', kind='two', grip_scale=0.75, grip=(0.01, -0.03), rake=30, arm=WRIST_ARM, fore=0.13),
    'shotgun': dict(file='ob/gun25/gun25.raw', part='part0', kind='two', grip_scale=0.85, grip=(0.0, -0.07), rake=30, arm=WRIST_ARM, fore=0.27),
    'brick': dict(file='ob/gun26/gun26.raw', part='part0', kind='one', palm=(0.0, 0.01, 0.0), fingers=(0.5, 0.0, 0.85), thumb=(-0.85, 0.0, 0.5)),
}


def gun_triangles(gun_obj):
    """The triangles of a gun's model, without its muzzle flash (a big shape of a few points)."""
    v, parts, cur = [], {}, None
    for line in open(gun_obj):
        w = line.split()
        if not w:
            continue
        if w[0] == 'v':
            v.append([float(x) for x in w[1:4]])
        elif w[0] == 'o':
            cur = w[1]
        elif w[0] == 'f':
            corners = [int(x.split('/')[0]) - 1 for x in w[1:]]
            parts.setdefault(cur, []).extend((corners[0], corners[k], corners[k + 1]) for k in range(1, len(corners) - 1))
    v = np.array(v)
    keep = []
    for faces in parts.values():
        ids = sorted({i for f in faces for i in f})
        if not (len(ids) <= 12 and np.ptp(v[ids], axis=0).max() > 0.15):
            keep += faces
    return v[np.array(keep)]


def section(triangles, axis, at):
    """Where the surface of a gun crosses a plane (a model has its points only at the ends of a long box, so the points near a place do not say where the surface is there): points on the plane."""
    out = []
    for a, b in ((0, 1), (1, 2), (2, 0)):
        p, q = triangles[:, a], triangles[:, b]
        d = q[:, axis] - p[:, axis]
        with np.errstate(divide='ignore', invalid='ignore'):
            t = (at - p[:, axis]) / d
        crossing = (t >= 0.0) & (t <= 1.0) & (abs(d) > 1.0e-9)
        out.append(p[crossing] + (q[crossing] - p[crossing]) * t[crossing][:, None])
    return np.concatenate(out)


def side_pose(triangles, y, z, rake, side):
    """A hand on the side `side` (-1 the right of the screen, 1 the left) of a grip at (y, z), leaning back by `rake`: its knuckles are at the far edge of the grip, the way the fingers go."""
    across = section(triangles, 1, y)
    near = across[abs(across[:, 2] - z) < GRIP_REACH]
    t = math.radians(rake)
    along = np.array([0.0, -math.cos(t), -math.sin(t)])  # (down the grip)
    fingers = np.array([0.0, along[2], -along[1]])
    point = np.array([0.0, y, z])
    point = point + fingers * ((near @ fingers).max() - point @ fingers)
    point[0] = near[:, 0].min() if side < 0 else near[:, 0].max()
    return dict(kind='side', fingers=tuple(fingers), thumb=tuple(-along), knuckle=tuple(float(v) for v in point), width=float(np.ptp(near[:, 0])))


def under_pose(triangles, z, above=None):
    """The left hand under the gun at the distance `z`: palm up, the fingers across and forward, up the far side.  `above`: the height below which something hangs that the hand does not go under (a magazine)."""
    across = section(triangles, 2, z)
    if above is not None:
        across = across[across[:, 1] > above]
    bottom = float(across[:, 1].min())
    low = across[across[:, 1] < bottom + 0.03]
    return dict(kind='under', fingers=(-0.75, 0.0, 0.66), thumb=(0.66, 0.0, 0.75), knuckle=(float(low[:, 0].min()), bottom + PALM_SEAT, z), width=float(np.ptp(across[:, 1]) / 2.0), limit=100.0)


def palm_pose(triangles, spec):
    """Something that lies on the upturned right palm: the palm is at the underside of it at the place given."""
    x, lift, z = spec['palm']
    # (the place can be past the side of a thin thing: the stick of TNT)
    points = triangles.reshape(-1, 3)
    across = section(triangles, 2, min(float(points[:, 2].max()) - 0.001, max(float(points[:, 2].min()) + 0.001, z)))
    near = across[abs(across[:, 0] - x) < 0.04]
    near = near if len(near) else across
    return dict(kind='palm', fingers=spec['fingers'], thumb=spec['thumb'], under=(x, float(near[:, 1].min()) + lift, z), width=float(min(np.ptp(points[:, 0]), np.ptp(points[:, 2])) / 2.0))


def hands_for(spec, gun_obj):
    """The hands of a weapon as (part, parts of the character, pose, scale, place, where the elbow is from the wrist, roll): the right hand holding it, and for a two handed gun the left hand under the front."""
    triangles = gun_triangles(gun_obj)
    pose = palm_pose(triangles, spec) if 'palm' in spec else side_pose(triangles, spec['grip'][0], spec['grip'][1], spec['rake'], -1)
    hands = [(spec['part'], 0, pose, spec.get('grip_scale', 1.0), (0.0, 0.0, 0.0), spec.get('arm', GRIP_ARM), 0)]
    if spec['kind'] == 'two':
        pose = side_pose(triangles, spec['foregrip'][0], spec['foregrip'][1], 5, 1) if 'foregrip' in spec else under_pose(triangles, spec['fore'], spec.get('fore_above'))
        hands.append((spec.get('fore_part', spec['part']), 1, pose, 1.0, (0.0, 0.0, 0.0), FORE_ARM, 0))
    return hands


CHR_MATS = {}
# the parts of the character that blend two joints (kind 1: a seam that has no joint of its own): their triangles are drawn between the joints of a pose, and in the rest pose of a model that has
# them they are flat flaps, so they are left out
SEAM_PARTS = set()
# the parts of the arm that holds the gun and of the other arm, from the hand up to the shoulder (strings, the numbers of the parts): the humanoid layout unless the character is one whose
# model numbers its parts otherwise
CHAINS = {'grip': ['20', '19', '18', '17'], 'front': ['12', '11', '10', '9']}
HUMAN_CHAINS = dict(CHAINS)
# character number -> (gun arm, other arm) for the models that are not in the humanoid layout; a chain of one part is a whole arm in one piece (hand at its bottom)
# (an arm in one piece is long: it is made smaller)
ARM_SCALE = {37: 0.55, 22: 0.55}
# characters whose in-between parts (kind 1) are pieces of the arm and are drawn: all but the TimeSplitters (70 to 72), whose are flat flaps
KEEP_SEAMS = set(range(80)) - {70, 71, 72}
OTHER_ARMS = {
    20: (['20', '19', '18', '17', '16', '15'], ['12', '11', '10', '9', '8', '7']),
    22: (['20', '19', '18', '17', '16', '15'], ['12', '11', '10', '9', '8', '7']),
    8: (['14', '13', '12', '11'], ['9', '8', '7', '6']),
    24: (['18', '17', '16', '15'], ['11', '10', '9', '8']),
    31: (['10', '9', '8', '7'], ['6', '5', '4']),
    37: (['17'], ['10']),
    44: (['23', '22', '21', '20', '19'], ['12', '11', '10', '9', '8']),
    46: (['19', '18', '17', '16'], ['11', '10', '9', '8']),
    49: (['20', '19', '18', '17'], ['12', '11', '10', '9']),
}


def load_obj(path):
    verts, colours, uvs, parts = [], [], [], {}
    cur, mat = None, None
    CHR_MATS.clear()
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
        elif w[0] == 'usemtl':
            mat = w[1]
        elif w[0] == 'f':
            corners = []
            for c in w[1:]:
                i = c.split('/')
                corners.append((int(i[0]) - 1, int(i[1]) - 1 if len(i) > 1 and i[1] else 0))
            parts.setdefault(cur, []).append(corners)
            CHR_MATS.setdefault(cur, []).append(mat)
    return np.array(verts), np.array(colours), np.array(uvs), parts


CHR_UVS = None


ARM_PARTS = (9, 10, 11, 12, 17, 18, 19, 20)


def character_files(pak):
    """The characters of the game whose model has the arms of the humanoid layout (the parts 9 to 12 and 17 to 20 of the same kinds and joints as the first character's): character
    number -> (file in the pack, number of the texture the hand has).  The others (aliens, robots) have no arms of their own here and keep the first character's."""
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
        # (the kind of a part and its joint differ between the skeletons of the characters, the tree that the arms hang on does not: the parts they are made of are the same numbers)
        parts = [tuple(f[bones + 80 * i + 2:bones + 80 * i + 6]) for i in range(cnt)] if cnt > 20 else None
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
            if c in OTHER_ARMS:
                out[c] = (name, 0)
                continue
            try:
                got = layout(name)
            except Exception:
                continue
            if got and got[0] == first[0]:
                out[c] = (name, got[1])
        if c in OTHER_ARMS and name in by_name and name.endswith('.raw') and '/chrs/' in name:
            out[c] = (name, 0)
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


def character_texture_numbers(pak, chr_file):
    """The numbers of the textures in a character's model (its table, by slot)."""
    sys.path.insert(0, TOOLS)
    import pakls
    d, es = pakls.entries(pak)
    n, o, sz = [e for e in es if e[0] == chr_file][0]
    data = d[o:o + sz]
    numbers, at = [], struct.unpack_from('<I', data, 0)[0]
    while struct.unpack_from('<i', data, at)[0] != -1:
        numbers.append(struct.unpack_from('<i', data, at)[0])
        at += 16
    return numbers


def write_arms(pak, mod, character, chr_file):
    """The arms of a character: `c<NN>.arm`, little-endian: its scale for the arms, the numbers of its textures, then for the arm that holds the grip and for the other, its pieces from the hand up, the triangles of each as
    the slot of their texture and three corners (position, s, t, colour)."""
    numbers = character_texture_numbers(pak, chr_file)
    sys.path.insert(0, TOOLS)
    import obdump
    import pakls
    d, es = pakls.entries(pak)
    _, o, s = [e for e in es if e[0] == chr_file][0]
    f = d[o:o + s]
    tab = struct.unpack_from('<I', f, 4)[0]
    count = struct.unpack_from('<I', f, tab)[0]
    bones = tab - count * 80
    grip, front = OTHER_ARMS.get(character, (HUMAN_CHAINS['grip'], HUMAN_CHAINS['front']))
    seams = {int(i) for i in grip + front if f[bones + 80 * int(i)] == 1 and character not in KEEP_SEAMS}
    # (the pieces that are see-through, such as rags that hang from a sleeve, are drawn in a second pass that a gun's part does not have: they would be white)
    for i in grip + front:
        see_through = struct.unpack_from('<I', f, bones + 80 * int(i) + 12)[0]
        if obdump.ok(f, see_through) and obdump.read_chunks(f, see_through):
            seams.add(int(i))
    obj = os.path.join(WORK, 'character%02d.obj' % character)
    subprocess.check_call([sys.executable, os.path.join(TOOLS, 'obdump.py'), pak, chr_file, '--obj', obj], stdout=subprocess.DEVNULL)
    verts, cols, uvs, parts = load_obj(obj)
    out = bytearray(struct.pack('<fI', ARM_SCALE.get(character, 1.0), len(numbers)) + struct.pack('<%di' % len(numbers), *numbers))
    for chain in (grip, front):
        pieces = [src for src in chain if int(src) not in seams and parts.get('part' + src)]
        out += struct.pack('<I', len(pieces))
        for src in pieces:
            faces = parts['part' + src]
            out += struct.pack('<I', len(faces))
            for k, face in enumerate(faces):
                mat = CHR_MATS['part' + src][k] or 'slot6'
                out += struct.pack('<I', int(mat.split('_')[0][4:]))
                for vi, ti in face:
                    colour = bytes(int(round(c * 255)) >> 1 for c in cols[vi]) + bytes(1)
                    out += struct.pack('<3f2f', *verts[vi], *uvs[ti]) + colour
    with open(os.path.join(mod, 'arms', 'c%02d.arm' % character), 'wb') as o:
        o.write(out)


def write_guns(pak, mod):
    """`guns.txt`: for each model of each gun (no tag; `_l` the gun of the right hand when one is held in each hand, the hand on its grip only; `_m` the gun of the left hand), a line for each hand."""
    lines = []
    for name, spec in WEAPONS.items():
        base = os.path.join(WORK, name + '.obj')
        if not os.path.exists(base):
            subprocess.check_call([sys.executable, os.path.join(TOOLS, 'obdump.py'), pak, spec['file'], '--obj', base], stdout=subprocess.DEVNULL)
        hands = hands_for(spec, base)
        variants = [('-', hands, 0)]
        if spec['kind'] == 'two':
            variants.append(('_l', hands[:1], 0))
        variants.append(('_m', hands[:1], 1))
        for tag, held, mirror in variants:
            for part, chain, pose, scale, _, arm, _ in held:
                at = pose['under'] if pose['kind'] == 'palm' else pose['knuckle']
                numbers = [scale, pose['kind'], *pose['fingers'], *pose['thumb'], *at, pose.get('width', 0.05), pose.get('limit', CURL_LIMIT), *arm]
                lines.append(' '.join([spec['file'], '' if tag == '-' else tag, part[4:], str(chain), str(mirror)] + [x if isinstance(x, str) else '%.6f' % x for x in numbers] + ['.']).replace('  ', ' - ', 1))
    with open(os.path.join(mod, 'arms', 'guns.txt'), 'w') as f:
        f.write(chr(10).join(lines) + chr(10))


OWN_SLOTS = (8, 12, 20, 24, 28, 32, 36, 40)


def _pointer_slots(f, bones_at, count):
    """The words of the bone records that are places in the file: (part, position of the word, its value, is it one of the places of the arrays and chunk lists of the part).  The other places
    (such as the data behind the chunk list of a part) are data that stays."""
    out = []
    for i in range(count):
        for k in range(8, 80, 4):
            at = bones_at + 80 * i + k
            v = struct.unpack_from('<I', f, at)[0]
            if 128 <= v < bones_at and v % 8 == 0:
                out.append((i, at, v, k in OWN_SLOTS))
    return out


def drop_old_part_data(before, after, part):
    """`obedit.mesh_part` writes the new arrays of a part after the others and leaves the old ones in the file: take them out (the data behind moves up, and the places in the file that point behind
    them are changed), so a model with hands on one of its parts is not twice the size."""
    tab_b, = struct.unpack_from('<I', before, 4)
    cnt_b, = struct.unpack_from('<I', before, tab_b)
    bones_b = tab_b - cnt_b * 80
    slots = _pointer_slots(before, bones_b, cnt_b)
    own = [v for i, at, v, mine in slots if i == part and mine]
    others = [v for i, at, v, mine in slots if not (i == part and mine)] + [bones_b]
    if not own:
        return after
    lo = min(own)
    above = [v for v in others if v > lo]
    hi = min(above) if above else bones_b
    if max(own) >= hi:
        return after
    delta = hi - lo
    tab_a, = struct.unpack_from('<I', after, 4)
    cnt_a, = struct.unpack_from('<I', after, tab_a)
    bones_a = tab_a - cnt_a * 80
    out = bytearray(after[:lo] + after[hi:])
    for at in (0, 4):
        v, = struct.unpack_from('<I', out, at)
        if v >= hi:
            struct.pack_into('<I', out, at, v - delta)
    new_bones = bones_a - delta
    for i in range(cnt_a):
        for k in range(8, 80, 4):
            at = new_bones + 80 * i + k
            v, = struct.unpack_from('<I', out, at)
            if hi <= v <= bones_a and v % 8 == 0:
                struct.pack_into('<I', out, at, v - delta)
    return bytes(out)


def same_geometry(a, b):
    """Do two models have the same vertices in every part (a check of `drop_old_part_data`)?"""
    sys.path.insert(0, TOOLS)
    import obdump
    out = []
    for f in (a, b):
        tab, = struct.unpack_from('<I', f, 4)
        cnt, = struct.unpack_from('<I', f, tab)
        bones = tab - cnt * 80
        verts = []
        for i in range(cnt):
            lst, = struct.unpack_from('<I', f, bones + 80 * i + 8)
            if not obdump.ok(f, lst):
                continue
            arrays = struct.unpack_from('<6I', f, bones + 80 * i + 20)
            for ch in obdump.read_chunks(f, lst):
                for k in range(ch[3], ch[3] + ch[4]):
                    verts.append((i, ch[0], struct.unpack_from('<3f', f, arrays[1] + 12 * k), struct.unpack_from('<2f', f, arrays[2] + 16 * k)))
        out.append(verts)
    return out[0] == out[1]


def pakls_entries(pak):
    sys.path.insert(0, TOOLS)
    import pakls
    return pakls.entries(pak)


def main():
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    pak, mod = sys.argv[1], sys.argv[2]
    os.makedirs(WORK, exist_ok=True)
    os.makedirs(os.path.join(mod, 'arms'), exist_ok=True)
    sys.path.insert(0, TOOLS)
    chars = character_files(pak)
    if os.environ.get('HANDS_CHARACTER'):
        chars = {int(os.environ['HANDS_CHARACTER']): chars[int(os.environ['HANDS_CHARACTER'])]}
    print(len(chars), 'characters with arms of their own:', sorted(chars))
    write_guns(pak, mod)
    # (the first character's arms are what a character without its own has)
    for c in sorted(chars):
        write_arms(pak, mod, c, chars[c][0])
    if 0 not in chars:
        write_arms(pak, mod, 0, character_files(pak)[0][0])
    # (the mod's own mod.ini and script say how the models are used: they are kept, and written when the folder is new)
    ini = os.path.join(mod, 'mod.ini')
    if not os.path.exists(ini) or 'script' not in open(ini).read():
        open(ini, 'w').write('name = First Person Hands' + chr(10) + 'enabled = true' + chr(10) + 'script = hands.rhai' + chr(10))
    scripts = os.path.join(mod, 'scripts')
    if not os.path.exists(os.path.join(scripts, 'hands.rhai')):
        os.makedirs(scripts, exist_ok=True)
        shutil.copy(os.path.join(HERE, '..', 'first_person_hands_mod', 'scripts', 'hands.rhai'), scripts)


if __name__ == '__main__':
    main()
