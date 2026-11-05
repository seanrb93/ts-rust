"""Make the first person hands mod: a hand and forearm (the ones of the game's character, `ob/chrs/chr01.raw`) are added to the model of a weapon as it is seen in the first person
(`ob/gunN/gunN.raw`), put where the weapon is held.

    python make-hands.py <OB.PAK> <mods folder>/first_person_hands_mod [weapon ...]      (the weapons are the names in WEAPONS; all of them when none is named).
The hands have the skin and sleeve of a character of the game: HANDS_CHARACTER=<number> (the character's number, 0 is Det. Harry Tipper, the default)

The tools it uses are `tools/obdump.py` (a model as an OBJ) and `tools/obedit.py meshes` (an OBJ back into a model).  The hand is put on a part of the gun that does not move on its own
(the body), or, for the forend of a pump gun, on the part that slides.  How each hand holds its weapon is in WEAPONS: the hand is turned to the weapon and its fingers bent round it, and the forearm comes in from where it would in real life.
"""
import math
import os
import shutil
import subprocess
import sys

import numpy as np
from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
TOOLS = os.path.join(HERE, '..', '..', 'tools')
TEX_BASE = 9400

# The weapons, held as they are in real life.  The space of a gun: +x is to the left on the screen, y up, z forward (to the muzzle).
# file: the first person model; part: the part the hands go on (the one that does not move on its own); kind: 'one' or 'two' hands.
# grip=(y, z): the middle of what the right hand holds; rake: how far the grip leans back from upright (degrees).  The hand lies on the right of it with its knuckles at its front, and the
# fingers curl round it (how far across it is, is measured from the model).
# fore=z: how far forward the left hand holds: palm up under the gun, the fingers up its far side, the thumb on the near side; fore_part: the part it rides on (the sliding forend of a pump gun).
# foregrip=(y, z): an upright grip for the left hand, held as the right hand holds its grip.
# handle=(x, y, z): a handle that sticks out to the left of the gun, for the left hand.
# arm: where the elbow of the right arm is, when not GRIP_ARM.
# palm=(x, y, z) with fingers=(x, y, z) and thumb=(x, y, z): something that is carried rather than gripped, lying on the upturned right palm.
CURL_LIMIT = 150.0  # how far round the fingers bend, degrees
THUMB_LIMIT = math.radians(90.0)  # and the thumb
GRIP_ARM = (-0.35, -0.3, -0.9)  # where the elbow of the right arm is from its wrist: back, down and to the right of the screen, nearly along the gun
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
    'rocket_launcher': dict(file='ob/gun9/gun9.raw', part='part0', kind='two', grip=(-0.02, -0.02), rake=10, handle=(0.15, 0.05, 0.08)),
    'minigun': dict(file='ob/gun11/gun11.raw', part='part0', kind='two', grip=(0.015, 0.035), rake=5, fore=0.19),
    'grenade_launcher': dict(file='ob/gun12/gun12.raw', part='part0', kind='two', grip=(-0.06, -0.07), rake=25, fore=0.27),
    'scifi_sniper_rifle': dict(file='ob/gun13/gun13.raw', part='part0', kind='two', grip=(-0.04, -0.06), rake=25, fore=0.3),
    'weapon_13': dict(file='ob/gun14/gun14.raw', part='part0', kind='two', grip=(-0.06, -0.07), rake=25, fore=0.27),
    'proximity_mine': dict(file='ob/gun15/gun15.raw', part='part1', kind='one', palm=(0.0, 0.0, 0.0), fingers=(0.5, 0.0, 0.85), thumb=(-0.85, 0.0, 0.5)),
    'remote_mine': dict(file='ob/gun16/gun16.raw', part='part0', kind='one', palm=(0.0, 0.0, 0.0), fingers=(0.5, 0.0, 0.85), thumb=(-0.85, 0.0, 0.5)),
    'timed_mine': dict(file='ob/gun17/gun17.raw', part='part2', kind='one', palm=(0.0, 0.02, 0.0), fingers=(0.5, 0.0, 0.85), thumb=(-0.85, 0.0, 0.5)),
    'weapon_17': dict(file='ob/gun18/gun18.raw', part='part0', kind='one', palm=(0.0, 0.0, 0.0), fingers=(0.5, 0.0, 0.85), thumb=(-0.85, 0.0, 0.5)),
    'tnt': dict(file='ob/gun19/gun19.raw', part='part0', kind='one', palm=(0.0, -0.04, 0.06), fingers=(1.0, 0.0, 0.25), thumb=(-0.25, 0.0, 1.0)),
    'tommy_gun': dict(file='ob/gun20/gun20.raw', part='part0', kind='two', grip=(-0.06, -0.09), rake=30, foregrip=(-0.05, 0.32)),
    'assault_shotgun': dict(file='ob/gun21/gun21.raw', part='part0', kind='two', grip=(-0.055, -0.04), rake=20, fore=0.42, fore_part='part2'),
    'blunderbuss': dict(file='ob/gun23/gun23.raw', part='part0', kind='two', grip_scale=0.75, grip=(0.01, -0.03), rake=30, arm=WRIST_ARM, fore=0.13),
    'shotgun': dict(file='ob/gun25/gun25.raw', part='part0', kind='two', grip_scale=0.85, grip=(0.0, -0.07), rake=30, arm=WRIST_ARM, fore=0.27),
    'brick': dict(file='ob/gun26/gun26.raw', part='part0', kind='one', palm=(0.0, -0.09, 0.0), fingers=(0.5, 0.0, 0.85), thumb=(-0.85, 0.0, 0.5)),
}


def gun_points(gun_obj):
    """The points of a gun's model, without its muzzle flash (a big shape of a few points)."""
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
            parts.setdefault(cur, set()).update(int(x.split('/')[0]) - 1 for x in w[1:])
    v = np.array(v)
    keep = [sorted(ids) for ids in parts.values() if not (len(ids) <= 12 and np.ptp(v[sorted(ids)], axis=0).max() > 0.15)]
    return v[sorted({i for ids in keep for i in ids})]


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


def around(points, y=None, z=None, dy=0.03, dz=0.03):
    """The points of a gun near a height and a distance forward (with a wider look when there are few)."""
    for grow in (1, 2, 4, 8):
        mask = np.ones(len(points), dtype=bool)
        if y is not None:
            mask &= abs(points[:, 1] - y) < dy * grow
        if z is not None:
            mask &= abs(points[:, 2] - z) < dz * grow
        if mask.sum() >= 6:
            return points[mask]
    return points


def side_pose(points, y, z, rake, side):
    """A hand on the side `side` (-1 the right of the screen, 1 the left) of a grip at (y, z), leaning back by `rake`: its knuckles are at the far edge of the grip, the way the fingers go."""
    near = around(points, y, z)
    t = math.radians(rake)
    along = np.array([0.0, -math.cos(t), -math.sin(t)])  # (down the grip)
    fingers = np.array([0.0, along[2], -along[1]])
    point = np.array([0.0, y, z])
    point = point + fingers * ((near @ fingers).max() - point @ fingers)
    point[0] = near[:, 0].min() if side < 0 else near[:, 0].max()
    return dict(kind='side', fingers=tuple(fingers), thumb=tuple(-along), knuckle=tuple(float(v) for v in point), width=float(np.ptp(near[:, 0])))


def handle_pose(points, x, y, z):
    """The left hand on a handle that sticks out to the left of the gun: from behind it, the fingers over the top and forward, the thumb towards the gun."""
    near = points[(abs(points[:, 0] - x) < 0.03) & (abs(points[:, 2] - z) < 0.05)]
    return dict(kind='side', fingers=(0.0, 0.85, 0.5), thumb=(-1.0, 0.0, 0.0), knuckle=(x, y + 0.03, z - 0.03), width=0.06, limit=170.0)


def under_pose(points, z):
    """The left hand under the gun at the distance `z`: palm up, the fingers across and forward, up the far side."""
    near = around(points, None, z)
    return dict(kind='under', fingers=(-0.75, 0.0, 0.66), thumb=(0.66, 0.0, 0.75), knuckle=(float(near[:, 0].min()), float(near[:, 1].min()), z), width=float(np.ptp(near[:, 1]) / 2.0), limit=100.0)


def hands_for(spec, gun_obj):
    """The hands of a weapon as (part, parts of the character, pose, scale, place, where the elbow is from the wrist, roll): the right hand holding it, and for a two handed gun the left hand under the front."""
    points = gun_points(gun_obj)
    if 'palm' in spec:
        pose = dict(kind='palm', fingers=spec['fingers'], thumb=spec['thumb'], under=(spec['palm'][0], float(points[:, 1].min()) + spec['palm'][1], spec['palm'][2]), width=float(min(np.ptp(points[:, 0]), np.ptp(points[:, 2])) / 2.0))
    else:
        pose = side_pose(points, spec['grip'][0], spec['grip'][1], spec['rake'], -1)
    hands = [(spec['part'], CHAINS['grip'], pose, spec.get('grip_scale', 1.0), (0.0, 0.0, 0.0), spec.get('arm', GRIP_ARM), 0)]
    if spec['kind'] == 'two':
        if 'handle' in spec:
            pose = handle_pose(points, *spec['handle'])
        elif 'foregrip' in spec:
            pose = side_pose(points, spec['foregrip'][0], spec['foregrip'][1], 5, 1)
        else:
            pose = under_pose(points, spec['fore'])
        hands.append((spec.get('fore_part', spec['part']), CHAINS['front'], pose, 1.0, (0.0, 0.0, 0.0), FORE_ARM, 0))
    return hands


def posed(pose, sources, scale, arm_from, chr_verts, chr_parts, mirror):
    """How the points of an arm go on a gun: the hand is turned to the pose with its fingers curled, the arm below the wrist is turned to `arm_from` about the wrist.  Gives a function of (point, whether it
    is of the hand)."""
    ids = lambda src: sorted({vi for f in chr_parts.get('part' + src, []) for vi, _ in f})
    # (a part of the chain can be empty in a character's model)
    sources = [src for src in sources if ids(src) and int(src) not in SEAM_PARTS]
    whole = len(sources) == 1
    verts = chr_verts[ids(sources[0])]
    if whole:
        # (an arm in one piece: the hand is its lowest sixth, the elbow its highest)
        by_y = verts[np.argsort(verts[:, 1])]
        k = max(1, len(by_y) // 6)
        hand, elbow = by_y[:k], by_y[-k:]
    else:
        hand, elbow = verts, chr_verts[ids(sources[-1])]
    middle, far = hand.mean(axis=0), elbow.mean(axis=0)
    down = (middle - far) / np.linalg.norm(middle - far)
    thumb_rest = np.array([0.0, 0.0, 1.0]) - down * down[2]
    thumb_rest /= np.linalg.norm(thumb_rest)
    side = 1.0 if middle[0] > 0 else -1.0      # (a left hand is at +x)
    palm_rest = np.cross(down, thumb_rest) * side
    along = (hand - middle) @ down
    wrist_u, tip_u = along.min(), along.max()
    wrist = middle + down * wrist_u
    knuckle_u = wrist_u + 0.55 * (tip_u - wrist_u)
    palm_side = (hand - middle) @ palm_rest
    palm_face = float(palm_side[along < knuckle_u].max())
    thumb_side = (hand - middle) @ thumb_rest
    fingers = np.array(pose['fingers'], dtype=float)
    fingers /= np.linalg.norm(fingers)
    thumb = np.array(pose['thumb'], dtype=float)
    thumb -= fingers * (thumb @ fingers)
    thumb /= np.linalg.norm(thumb)
    # (a turn takes down, the thumb and the palm of the rest to the fingers, the thumb and what follows of them)
    rest = np.column_stack([down, thumb_rest, np.cross(down, thumb_rest)])
    goal = np.column_stack([fingers, thumb, np.cross(fingers, thumb)])
    turn = goal @ rest.T
    radius = max(0.018, min(0.05, pose.get('width', 0.05) / 2.0)) / max(scale, 1e-6)
    knuckle_rest = middle + down * knuckle_u + palm_rest * palm_face
    if pose['kind'] == 'palm':
        # (the middle of the palm under the thing)
        target = np.array(pose['under']) - turn @ (palm_rest * palm_face) * scale
        hand_at = target
    else:
        hand_at = np.array(pose['knuckle']) - turn @ (knuckle_rest - middle) * scale
    arm_turn = rotation_between(far - wrist, arm_from)
    # (the arm is rolled about itself so that its thumb side is the hand's: no twist at the wrist)
    axis = np.array(arm_from, dtype=float) / np.linalg.norm(arm_from)
    got = arm_turn @ thumb_rest
    got -= axis * (got @ axis)
    want = thumb - axis * (thumb @ axis)
    if np.linalg.norm(got) > 1e-6 and np.linalg.norm(want) > 1e-6:
        angle = math.atan2(np.cross(got, want) @ axis, got @ want)
        k = np.array([[0, -axis[2], axis[1]], [axis[2], 0, -axis[0]], [-axis[1], axis[0], 0]])
        arm_turn = (np.eye(3) + math.sin(angle) * k + (1 - math.cos(angle)) * (k @ k)) @ arm_turn
    wrist_at = hand_at + turn @ (wrist - middle) * scale

    def curl(point):
        d = point - middle
        u, n, t = d @ down, d @ palm_rest, d @ thumb_rest
        # (the fingers fold: each is bent round what is held, the whole of its length on the arc, so that no tip stands out past it)
        new_u, new_n = u, n
        if u > knuckle_u:
            side_of = max(0.0, min(1.0, (t / max(thumb_side.max(), 1e-6) - 0.35) / 0.4))
            fade = 1.0 - side_of * side_of * (3.0 - 2.0 * side_of)
            angle = (u - knuckle_u) / max(tip_u - knuckle_u, 1e-6) * math.radians(pose.get('limit', CURL_LIMIT)) * fade
            reach = palm_face + radius - n
            new_u = knuckle_u + (u - knuckle_u) * (1.0 - fade) + reach * math.sin(angle)
            new_n = palm_face + radius - reach * math.cos(angle)
        # (and the thumb comes round the back of it)
        new_t = t
        thumb_from = 0.45 * thumb_side.max()
        if t > thumb_from:
            angle = (t - thumb_from) / max(thumb_side.max() - thumb_from, 1e-6) * THUMB_LIMIT
            reach = palm_face + radius - new_n
            new_t = thumb_from + reach * math.sin(angle)
            new_n = palm_face + radius - reach * math.cos(angle)
        return middle + down * new_u + palm_rest * new_n + thumb_rest * new_t

    def place(point, of_hand):
        if whole:
            out = hand_at + arm_turn @ (point - middle) * scale
        elif of_hand:
            out = hand_at + turn @ (curl(point) - middle) * scale
        else:
            out = wrist_at + arm_turn @ (point - wrist) * scale
        return out * np.array([-1.0, 1.0, 1.0]) if mirror else out

    return place


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
            mirror = len(arm_from) > 2 and arm_from[2] == 'mirror'
            pose_place = posed(rot, sources, scale, arm_from[0], chr_verts, chr_parts, mirror)
            hand_faces = []
            hv, hc, hf = [], [], []
            hand_nv = nv
            nv_before_t, nt_base = nv, nt
            hand_done = False
            for src in sources:
                if int(src) in SEAM_PARTS:
                    continue
                used, uv_of = {}, {}
                for f in chr_parts.get('part' + src, []):
                    for vi, ti in f:
                        if vi not in used:
                            used[vi] = len(used)
                            uv_of[vi] = ti
                if not used:
                    continue
                base = nv
                base_t = nt
                of_hand = not hand_done
                hand_done = True
                part_middle = pose_place(chr_verts[list(used)].mean(axis=0), of_hand)
                for vi in sorted(used, key=used.get):
                    p = pose_place(chr_verts[vi], of_hand)
                    hv.append(p)
                    hc.append(part_middle)
                    c = chr_cols[vi]
                    out.append('v %.6f %.6f %.6f %.3f %.3f %.3f' % (p[0], p[1], p[2], c[0], c[1], c[2]))
                    out.append('vt %.6f %.6f' % tuple(CHR_UVS[uv_of[vi]]))
                nv += len(used)
                nt += len(used)
                last_mat = None
                for fi, f in enumerate(chr_parts.get('part' + src, [])):
                    if os.environ.get('HANDS_DROP_SLOTS') and (CHR_MATS.get('part' + src, [None] * (fi + 1))[fi] or '').split('_')[0].replace('slot', '') in os.environ['HANDS_DROP_SLOTS'].split(','):
                        continue
                    # (each face has the texture of the slot the character's own model gives it: the sleeve and the skin are not one picture in every character)
                    mat = 'chr' + (CHR_MATS.get('part' + src, [None] * (fi + 1))[fi] or 'slot6').split('_')[0]
                    if mat != last_mat:
                        hf.append(mat)
                        last_mat = mat
                    hf.append([base + used[vi] for vi, _ in f][::-1] if mirror else [base + used[vi] for vi, _ in f])
            # (the normals are the average of the faces that meet at a point, so the arm is lit as a round thing and not as the flat pieces it is made of; one that points inward is turned)
            keys = [tuple(np.round(q, 3)) for q in hv]
            total, face_normals, flip = {}, {}, 0.0
            for k, f in enumerate(hf):
                if isinstance(f, str) or len(f) < 3:
                    continue
                a, b, c = (hv[i - hand_nv] for i in f[:3])
                n = np.cross(b - a, c - a)
                face_normals[k] = n
                flip += float(np.dot(n, (a + b + c) / 3 - hc[f[0] - hand_nv]))
                for i in f:
                    total[keys[i - hand_nv]] = total.get(keys[i - hand_nv], 0) + n
            sign = -1.0 if flip < 0 else 1.0
            for q in range(len(hv)):
                n = total.get(keys[q], np.zeros(3)) * sign
                norm = np.linalg.norm(n)
                n = n / norm if norm > 1e-9 else np.array([0.0, 1.0, 0.0])
                out.append('vn %.5f %.5f %.5f' % tuple(n))
            for f in hf:
                if isinstance(f, str):
                    out.append('usemtl ' + f)
                else:
                    out.append('f ' + ' '.join('%d/%d/%d' % (i + 1, (i - nv_before_t) + 1 + nt_base, (i - hand_nv) + 1 + nn) for i in f))
            nn += len(hv)
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
    import struct
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


def bake_character(pak, mod, character, chr_file, skin_number, names, work):
    """The first person models of `names` with the arms of `character` on them: written as `gunN.raw` for the first character (the models every character shares), as `gunN_cNN.raw` for the others."""
    global CHR_UVS
    import obedit
    skin_numbers = character_texture_numbers(pak, chr_file)
    import struct as _st
    sys.path.insert(0, TOOLS)
    import pakls as _pk
    _d, _es = _pk.entries(pak)
    _n, _o, _s = [e for e in _es if e[0] == chr_file][0]
    _f = _d[_o:_o + _s]
    _tab = _st.unpack_from('<I', _f, 4)[0]
    _cnt = _st.unpack_from('<I', _f, _tab)[0]
    SEAM_PARTS.clear()
    import obdump as _ob
    CHAINS['grip'], CHAINS['front'] = OTHER_ARMS.get(character, (HUMAN_CHAINS['grip'], HUMAN_CHAINS['front']))
    arm_parts = [int(x) for x in CHAINS['grip'] + CHAINS['front']]
    SEAM_PARTS.update(i for i in arm_parts if _f[_tab - _cnt * 80 + 80 * i] == 1 and character not in KEEP_SEAMS | set(int(x) for x in os.environ.get('HANDS_KEEP', '').split(',') if x))
    # (the pieces that are see-through, such as rags that hang from a sleeve, are drawn in a second pass that the model made here does not have: they would be white)
    for i in arm_parts:
        see_through = _st.unpack_from('<I', _f, _tab - _cnt * 80 + 80 * i + 12)[0]
        if _ob.ok(_f, see_through) and _ob.read_chunks(_f, see_through):
            SEAM_PARTS.add(i)
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
        if character in ARM_SCALE:
            spec = dict(spec, hands=[h[:3] + (h[3] * ARM_SCALE[character],) + h[4:] for h in spec['hands']])
        # (the model with the arms as the gun is held in both hands, and for a two handed gun the one of the left gun when a gun is held in each hand, with the hand on the grip only)
        variants = [(spec['hands'], spec['file'] if character == 0 else spec['file'].replace('.raw', '_c%02d.raw' % character), '')]
        if spec['kind'] == 'two':
            variants.append((spec['hands'][:1], spec['file'].replace('.raw', '_c%02d_l.raw' % character), '_l'))
        # (the gun of the left hand: the hand on its grip is a left hand, the right hand's mirror, its arm coming in from the lower left)
        grip = spec['hands'][0]
        variants.append(([grip[:5] + (grip[5], grip[6], 'mirror')], spec['file'].replace('.raw', '_c%02d_m.raw' % character), '_m'))
        for hands, out_name, tag in variants:
            add_hands(base + '.obj', base + '_hands%02d%s.obj' % (character, tag), hands, cv, cc, cp)
            body_faces = {k: len(v) for k, v in read_blocks(base + '.obj')[3].items()}
            groups = {}
            faces, _ = obedit.read_obj(base + '_hands%02d%s.obj' % (character, tag), groups)
            n, o, sz = [e for e in es if e[0] == spec['file']][0]
            data = d[o:o + sz]
            for part_name, fs in groups.items():
                if part_name and part_name.startswith('part') and part_name[4:].isdigit():
                    before = data
                    own = None if os.environ.get('HANDS_NOSMALL') else original_strip_faces(data, int(part_name[4:]))
                    raw = None
                    if own is not None and part_name in body_faces:
                        # (the gun's own strips, then the faces of the hands: the part is not turned into single triangles)
                        raw = own[1]
                        fs = own[0] + fs[body_faces[part_name]:]
                    data, _count = obedit.mesh_part(data, fs, int(part_name[4:]), {'chrslot%d' % k: n for k, n in enumerate(skin_numbers)}, raw_strips=raw)
                    small = data if os.environ.get('HANDS_NOSMALL') else drop_old_part_data(before, data, int(part_name[4:]))
                    if small is not data and same_geometry(data, small):
                        data = small
                    elif small is not data:
                        print('model not made smaller (the check failed):', out_name, part_name)
            out = os.path.join(mod, 'OB.PAK', out_name)
            os.makedirs(os.path.dirname(out), exist_ok=True)
            open(out, 'wb').write(data)


def original_strip_faces(f, part):
    """The strips of a part of a model as `obedit.mesh_part` takes them (a face with all the vertices of the strip, so the part keeps the size it has): [(corners, material)] and {id(corners): flip},
    or None when the part is not a plain part with normals (then it is made from the OBJ as triangles)."""
    import struct
    sys.path.insert(0, TOOLS)
    import obdump
    tab, = struct.unpack_from('<I', f, 4)
    cnt, = struct.unpack_from('<I', f, tab)
    bones = tab - cnt * 80
    lst, = struct.unpack_from('<I', f, bones + 80 * part + 8)
    if not obdump.ok(f, lst):
        return None
    strips_at, pos_at, st_at, col_at, bind_at, nrm_at = struct.unpack_from('<6I', f, bones + 80 * part + 20)
    chunks = obdump.read_chunks(f, lst)
    if not chunks or any(c[5] != 3 for c in chunks) or not nrm_at or not col_at:
        return None
    faces, flips = [], {}
    for slot, first_strip, strips, first_vertex, vertices, kind in chunks:
        v = first_vertex
        mat = 'slot%d_k%d' % (slot, kind)
        for s in range(strips):
            word, = struct.unpack_from('<I', f, strips_at + 16 * (first_strip + s))
            count = word & 0x7fff
            corners = []
            for k in range(count):
                pos = struct.unpack_from('<3f', f, pos_at + 12 * (v + k))
                st = struct.unpack_from('<2f', f, st_at + 16 * (v + k))
                nrm = struct.unpack_from('<3f', f, nrm_at + 12 * (v + k))
                col = tuple((b << 1) & 0xff for b in f[col_at + 4 * (v + k):col_at + 4 * (v + k) + 3])
                corners.append((pos, st, nrm, col))
            faces.append((corners, mat))
            flips[id(corners)] = bool(word & 0x10000)
            v += count
    return faces, flips


OWN_SLOTS = (8, 12, 20, 24, 28, 32, 36, 40)


def _pointer_slots(f, bones_at, count):
    """The words of the bone records that are places in the file: (part, position of the word, its value, is it one of the places of the arrays and chunk lists of the part).  The other places
    (such as the data behind the chunk list of a part) are data that stays."""
    import struct
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
    import struct
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
    import struct
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
    # (the mod's own mod.ini and script say how the models are used: they are kept, and written when the folder is new)
    ini = os.path.join(mod, 'mod.ini')
    if not os.path.exists(ini) or 'script' not in open(ini).read():
        open(ini, 'w').write('name = First Person Hands' + chr(10) + 'enabled = true' + chr(10) + 'script = hands.rhai' + chr(10))
    scripts = os.path.join(mod, 'scripts')
    if not os.path.exists(os.path.join(scripts, 'hands.rhai')):
        os.makedirs(scripts, exist_ok=True)
        shutil.copy(os.path.join(HERE, '..', 'first_person_hands_mod', 'scripts', 'hands.rhai'), scripts)


main()
