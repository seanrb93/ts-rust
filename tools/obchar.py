"""Make a character for the game from a mesh: a mod folder with the model and the line of `characters.txt`.

    python tools/obchar.py <game folder> <mesh.obj> --out mods/Robo --name "Robo" [--base 0] [--number 80] [--skeleton male|female] [--voice 2] [--scale 1.4] [--height 1.95] [--arm-bias 0.12]

The mesh is an OBJ of a whole figure standing in the pose of the game's skeleton (arms down at the sides, feet on the ground, facing the way the game's characters face; `python tools/obdump.py
<OB.PAK> ob/chrs/chr01.raw --obj out.obj` writes a game character in the pose, to model on).  It is scaled to the height of a game character (`--height`, 1.95 by default) and put on the ground.  Keep it so: the skeleton does not change with the mesh, so a mesh of another height no longer fits its joints (a figure 1.4 times taller shows torn triangles at the hips).  For a larger or smaller character use `--scale 1.4`: the game scales the offsets of the animations' joints by the character's `scale` field (`characters.txt`), so the tool scales the mesh by the same number, which it writes as `scale`; that fits the joints (checked on the select page at 1.4).  Only the mesh, not the field, would tear the model.  Each triangle
is given to the bone of the game's skeleton nearest to it (the bone is the stretch from a joint to the next one, or to a point past the end of a limb), so every part of the mesh follows the
joint that is nearest to it.  The triangles near a joint between two bones go to the model's own seam part there (kinds 1 and 2 of `docs/model-format.md`, the ones the game's characters have at an elbow
or a knee), each corner following the joint of the nearer bone, and for a seam of kind 2 the corners right at the joint follow its own joint, so a bent joint stretches instead of opening.  `--arm-bias 0.12` keeps a torso that lies close to the arms from following them (the sides of a slim torso are nearer to an arm's bone than to the spine); it is for figures that are not modelled on the game's own.  The materials of the OBJ with a `map_Kd` picture (any size) become textures of the game (`--tex-base`, 9000 by default).

The model is made from a character of the game (`--base`, 0 by default): its file is the pattern, so the bones, the places of the hands and the head and the animations are those of the skeleton.  The result:

    <out>/mod.ini
    <out>/characters.txt                 character <number>: copy <base>, the name, the model file
    <out>/OB.PAK/ob/chrs/<file>.raw     the model
    <out>/MISC.PAK/textures/<n>.qpm, .png   the pictures

A build with extended-tables is needed.  The character is in the table of characters and can be used wherever a character number is given (the select page reaches it after the game's own characters).
"""
import os
import struct
import sys

import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import obdump  # noqa: E402
import obedit  # noqa: E402
import pakls  # noqa: E402


def quat_matrix(q):
    x, y, z, w = q
    return np.array([[1 - 2 * (y * y + z * z), 2 * (x * y - z * w), 2 * (x * z + y * w)],
                     [2 * (x * y + z * w), 1 - 2 * (x * x + z * z), 2 * (y * z - x * w)],
                     [2 * (x * z - y * w), 2 * (y * z + x * w), 1 - 2 * (x * x + y * y)]])


def skeleton_joints(game, model, skeleton_file):
    """The world positions of the kind 0 parts of a model (their order is the order of the skeleton's channels), the parent of each, and the parts."""
    d, es = pakls.entries(os.path.join(game, 'PAK', 'ANIM.PAK'))
    entry = [(o, s) for n, o, s in es if n == skeleton_file][0]
    sk = d[entry[0]:entry[0] + entry[1]]
    keys, channels = struct.unpack_from('<I', sk, 4)[0], struct.unpack_from('<I', sk, 28)[0]
    data_at = (((keys << 2) + 15) & ~15) + (channels << 5) + 51 & ~15
    poses = [struct.unpack_from('<7f', sk, data_at + 32 * k) for k in range(channels)]
    tex, tab = struct.unpack_from('<2I', model, 0)
    n = struct.unpack_from('<I', model, tab)[0]
    b0 = tab - n * 80
    parts = [(model[b0 + 80 * i], struct.unpack_from('<4b', model, b0 + 80 * i + 2), model[b0 + 80 * i + 1]) for i in range(n)]
    kind0 = [i for i, p in enumerate(parts) if p[0] == 0]

    def ancestor(i):
        p = parts[i][1][0]
        while p >= 0 and parts[p][0] != 0:
            p = parts[p][1][0]
        return p
    world, parent = {}, {}
    for ch, pi in enumerate(kind0):
        pos, rot = np.array(poses[ch][:3]), quat_matrix(poses[ch][3:])
        par = ancestor(pi)
        parent[pi] = par
        if par < 0:
            world[pi] = (pos, rot)
        else:
            world[pi] = (world[par][0] + world[par][1] @ pos, world[par][1] @ rot)
    return kind0, {k: v[0] for k, v in world.items()}, parent, parts


def bone_segments(kind0, world, parent, size=1.0):
    """For each kind 0 part the stretch of the body it moves: from its joint to the mean of its children's joints, or, for the end of a limb, a point past it."""
    children = {p: [c for c in kind0 if parent[c] == p] for p in kind0}
    segs = {}
    for p in kind0:
        a = world[p]
        if children[p]:
            b = np.mean([world[c] for c in children[p]], axis=0)
        else:
            # (the end of a limb: the head goes up, a hand or a foot goes on the way the limb pointed)
            up = world[p] - world[parent[p]] if parent[p] >= 0 else np.array([0, 0.1, 0])
            b = a + up / max(np.linalg.norm(up), 1e-6) * (0.35 if a[1] > 1.4 * size else 0.2) * size
        segs[p] = (a, b)
    return segs


def distance_to_segment(p, a, b):
    ab = b - a
    t = np.clip(np.dot(p - a, ab) / max(np.dot(ab, ab), 1e-12), 0.0, 1.0)
    return float(np.linalg.norm(p - (a + t * ab)))


def make_character(game, mesh, out, name='Custom', base=0, number=80, scale=1.0, height=1.95, skeleton='male', tex_base=9000, model_file=None, voice=None, write_entry=True, arm_bias=0.0):
    """Make the character: its model and pictures go into the mod folder `out`; the lines for `characters.txt` are returned (and, with `write_entry`, written with a `mod.ini`, replacing
    the file).  `scale` makes it larger or smaller (the mesh and the character's `scale` field together)."""
    size = float(scale)
    height = float(height) * size
    skeleton_file = 'anim/data/newanims/fskel.raw' if skeleton == 'female' else 'anim/data/male_skeleton.raw'
    model_file = model_file or f'ob/chrs/chr{90 + number - 80:02d}.raw'
    ob_path = os.path.join(game, 'PAK', 'OB.PAK')
    d, es = pakls.entries(ob_path)
    base_name = f'ob/chrs/chr{base + 1:02d}.raw'
    hit = [(o, s) for n, o, s in es if n == base_name]
    if not hit:
        sys.exit('no such character model: ' + base_name)
    model = d[hit[0][0]:hit[0][0] + hit[0][1]]
    kind0, world, parent, parts = skeleton_joints(game, model, skeleton_file)
    world = {k: v * size for k, v in world.items()}
    segs = bone_segments(kind0, world, parent, size)
    faces, pictures = obedit.read_obj(mesh)
    if not faces:
        sys.exit('the mesh has no faces')
    # the mesh scaled to the height of a character, on the ground, centred
    pts = np.array([c[0] for corners, _ in faces for c in corners])
    lo, hi = pts.min(axis=0), pts.max(axis=0)
    scale = height / max(hi[1] - lo[1], 1e-6)
    # (centred on the median of the vertices, which is near the body's axis; the middle of the box is not when an arm or a gun reaches out on one side)
    mid = np.median(pts, axis=0)
    # (the game's own characters have their median 0.045 in front of the skeleton's origin: measured on chr01)
    shift = np.array([-mid[0], -lo[1], -mid[2] + 0.045 * size])
    place = lambda p: tuple(float(x) for x in (np.array(p) + shift) * scale)
    moved = []
    for corners, mat in faces:
        if mat and mat.startswith('slot'):
            # (a mesh written by obdump.py: the parts here are all rigid, whose chunks are of kind 3)
            mat = 'slot%s_k3' % mat[4:].partition('_k')[0]
        moved.append(([(place(c[0]), c[1], c[2], c[3]) for c in corners], mat))
    # each face to the bone nearest its middle; a face near the joint between two bones goes to the seam of the model that joins them, each of its corners following the joint
    # (a seam of kind 2 has a joint of its own between them, which the corners at the joint follow) of the nearer bone
    by_part = {p: [] for p in kind0}
    seams = {}
    for p in range(len(parts)):
        if parts[p][0] in (1, 2) and parts[p][1][0] in segs and parts[p][1][1] in segs:
            x, y = parts[p][1][0], parts[p][1][1]
            lengths = [float(np.linalg.norm(segs[q][1] - segs[q][0])) for q in (x, y)]
            seams[p] = (x, y, max(0.04, 0.35 * min(lengths)))
    corner_binds = {}
    # (`arm_bias`: a distance added to the bones of the arms when a vertex picks the bone it follows, so that the sides of a torso that lies close to the arms stay with the spine)
    arm_bones = {q for q in kind0 if abs(world[q][0]) >= 0.2 * size}
    nearest = lambda v, among: min(among, key=lambda q: distance_to_segment(v, *segs[q]) + (arm_bias * size if q in arm_bones else 0.0))
    key = lambda c: tuple(round(float(x), 5) for x in c[0])
    # each vertex (by position) follows one bone, the nearest at first.  A triangle whose corners follow bones that no seam of the model joins cannot be drawn without a crack, so its
    # corners are all given the bone nearest its middle, and that changes the other triangles at those vertices; this goes on until nothing changes
    vertex_bone = {}
    for corners, mat in moved:
        for c in corners:
            vertex_bone.setdefault(key(c), nearest(np.array(c[0]), kind0))
    pairs = {frozenset((x, y)): sp for sp, (x, y, radius) in seams.items()}
    for _ in range(20):
        changed = False
        for corners, mat in moved:
            bones = {vertex_bone[key(c)] for c in corners}
            if len(bones) < 2 or len(corners) != 3 or any(bones <= pair for pair in pairs):
                continue
            mid = np.mean([c[0] for c in corners], axis=0)
            best = min(bones, key=lambda q: distance_to_segment(mid, *segs[q]))
            for c in corners:
                if vertex_bone[key(c)] != best:
                    vertex_bone[key(c)] = best
                    changed = True
        if not changed:
            break
    for corners, mat in moved:
        bones = {vertex_bone[key(c)] for c in corners}
        if len(corners) == 3 and len(bones) == 2:
            sp = pairs.get(frozenset(bones))
            if sp is None:
                sp = next((q for p2, q in pairs.items() if bones <= p2), None)
        else:
            sp = None
        at = None
        if sp is None and len(corners) == 3:
            # (a triangle on one bone with a corner at the joint of a seam that has a joint of its own: it is drawn by the seam, as the corner there follows the joint of its own)
            for q, (x, y, radius) in seams.items():
                if parts[q][0] == 2 and bones <= {x, y} and any(np.linalg.norm(np.array(c[0]) - world[y]) < radius / 2 for c in corners):
                    sp = q
                    break
        if sp is not None:
            x, y, radius = seams[sp]
            joint = world[y]
            binds = []
            for c in corners:
                v = np.array(c[0])
                if parts[sp][0] == 2 and np.linalg.norm(v - joint) < radius / 2:
                    binds.append(parts[sp][2])
                else:
                    binds.append(parts[vertex_bone[key(c)]][2])
            corner_binds[id(corners)] = binds
            # (the chunks of a part that blends joints are of kind 4)
            by_part.setdefault(sp, []).append((corners, mat[:-1] + '4' if mat and mat.endswith('_k3') else mat))
        else:
            mid = np.mean([c[0] for c in corners], axis=0)
            best = min(bones, key=lambda q: distance_to_segment(mid, *segs[q]))
            by_part[best].append((corners, mat))
    numbers = obedit.add_pictures(out, pictures, tex_base) if pictures else None
    data = model
    only = {int(x) for x in os.environ['OBONLY'].split(',')} if os.environ.get('OBONLY') else None
    for p in range(len(parts)):
        fs = by_part.get(p, [])
        if only is not None and p not in only:
            continue
        if parts[p][0] == 0 or fs:
            data, _ = obedit.mesh_part(data, fs, p, numbers, corner_binds=corner_binds)
        elif not os.environ.get('OBKEEP'):
            # (a part that joins two joints is left without surfaces)
            data, _ = obedit.mesh_part(data, [], p, numbers)
    target = os.path.join(out, 'OB.PAK', model_file)
    os.makedirs(os.path.dirname(target), exist_ok=True)
    open(target, 'wb').write(data)
    nl = chr(10)
    entry = f'character {number}{nl}  copy {base}{nl}  name "{name}"{nl}  model_file {model_file}{nl}' + (f'  voice {voice}{nl}' if voice not in (None, '') else '') + (f'  scale {size}{nl}' if size != 1.0 else '')
    if write_entry:
        open(os.path.join(out, 'mod.ini'), 'w').write('enabled = true' + nl)
        open(os.path.join(out, 'characters.txt'), 'w').write(entry)
    used = sum(1 for p in kind0 if by_part[p])
    print(f'{len(moved)} faces on {used} of the {len(kind0)} bones; model {target}')
    print(f'characters.txt: character {number} "{name}", a copy of character {base}')
    return entry


def main():
    if len(sys.argv) < 3 or '--out' not in sys.argv:
        sys.exit(__doc__)
    game, mesh = sys.argv[1], sys.argv[2]
    arg = lambda k, d=None: sys.argv[sys.argv.index(k) + 1] if k in sys.argv else d
    number = int(arg('--number', 80))
    make_character(game, mesh, arg('--out'), arg('--name', 'Custom'), int(arg('--base', 0)), number, float(arg('--scale', 1.0)), float(arg('--height', 1.95)), arg('--skeleton', 'male'),
                   int(arg('--tex-base', 9000)), arg('--model-file'), arg('--voice'), arm_bias=float(arg('--arm-bias', 0.0)))


if __name__ == '__main__':
    main()
