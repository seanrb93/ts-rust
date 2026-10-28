"""Make a character for the game from a mesh: a mod folder with the model and the line of `characters.txt`.

    python tools/obchar.py <game folder> <mesh.obj> --out mods/Robo --name "Robo" [--base 0] [--number 80] [--skeleton male|female] [--height 1.95]

The mesh is an OBJ of a whole figure standing in the pose of the game's skeleton (arms down at the sides, feet on the ground, facing the way the game's characters face; `python tools/obdump.py
<OB.PAK> ob/chrs/chr01.raw --obj out.obj` writes a game character in the pose, to model on).  It is scaled to the height of a game character (`--height`, 1.95 by default) and put on the ground.  Each triangle
is given to the bone of the game's skeleton nearest to it (the bone is the stretch from a joint to the next one, or to a point past the end of a limb), so every part of the mesh follows the
joint that is nearest to it, rigidly: the game's characters have separate parts that join two joints at an elbow or a knee (kinds 1 to 3 of `docs/model-format.md`), which this does not make, so a bent
joint shows a gap.  The materials of the OBJ with a `map_Kd` picture (any size) become textures of the game (`--tex-base`, 9000 by default).

The model is made from a character of the game (`--base`, 0 by default): its file is the pattern, so the bones, the places of the hands and the head and the animations are those of the skeleton.  The result:

    <out>/mod.ini
    <out>/characters.txt                 character <number>: copy <base>, the name, the model file
    <out>/OB.PAK/ob/chrs/<file>.raw     the model
    <out>/MISC.PAK/textures/<n>.qpm, .png   the pictures

A build with extended-tables is needed.  The character is in the table of characters and can be used wherever a character number is given (it is not in the select page yet).
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
    parts = [(model[b0 + 80 * i], struct.unpack_from('<4b', model, b0 + 80 * i + 2)) for i in range(n)]
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


def bone_segments(kind0, world, parent):
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
            b = a + up / max(np.linalg.norm(up), 1e-6) * (0.35 if a[1] > 1.4 else 0.2)
        segs[p] = (a, b)
    return segs


def distance_to_segment(p, a, b):
    ab = b - a
    t = np.clip(np.dot(p - a, ab) / max(np.dot(ab, ab), 1e-12), 0.0, 1.0)
    return float(np.linalg.norm(p - (a + t * ab)))


def main():
    if len(sys.argv) < 3 or '--out' not in sys.argv:
        sys.exit(__doc__)
    game, mesh = sys.argv[1], sys.argv[2]
    arg = lambda k, d=None: sys.argv[sys.argv.index(k) + 1] if k in sys.argv else d
    out, name = arg('--out'), arg('--name', 'Custom')
    base, number = int(arg('--base', 0)), int(arg('--number', 80))
    height = float(arg('--height', 1.95))
    skeleton_file = 'anim/data/newanims/fskel.raw' if arg('--skeleton', 'male') == 'female' else 'anim/data/male_skeleton.raw'
    tex_base = int(arg('--tex-base', 9000))
    model_file = arg('--model-file', f'ob/chrs/chr{90 + number - 80:02d}.raw')
    ob_path = os.path.join(game, 'PAK', 'OB.PAK')
    d, es = pakls.entries(ob_path)
    base_name = f'ob/chrs/chr{base + 1:02d}.raw'
    hit = [(o, s) for n, o, s in es if n == base_name]
    if not hit:
        sys.exit('no such character model: ' + base_name)
    model = d[hit[0][0]:hit[0][0] + hit[0][1]]
    kind0, world, parent, parts = skeleton_joints(game, model, skeleton_file)
    segs = bone_segments(kind0, world, parent)
    faces, pictures = obedit.read_obj(mesh)
    if not faces:
        sys.exit('the mesh has no faces')
    # the mesh scaled to the height of a character, on the ground, centred
    pts = np.array([c[0] for corners, _ in faces for c in corners])
    lo, hi = pts.min(axis=0), pts.max(axis=0)
    scale = height / max(hi[1] - lo[1], 1e-6)
    shift = np.array([-(lo[0] + hi[0]) / 2, -lo[1], -(lo[2] + hi[2]) / 2])
    place = lambda p: tuple(float(x) for x in (np.array(p) + shift) * scale)
    moved = []
    for corners, mat in faces:
        if mat and mat.startswith('slot'):
            # (a mesh written by obdump.py: the parts here are all rigid, whose chunks are of kind 3)
            mat = 'slot%s_k3' % mat[4:].partition('_k')[0]
        moved.append(([(place(c[0]), c[1], c[2], c[3]) for c in corners], mat))
    # each face to the bone nearest its middle
    by_part = {p: [] for p in kind0}
    for corners, mat in moved:
        mid = np.mean([c[0] for c in corners], axis=0)
        best = min(kind0, key=lambda p: distance_to_segment(mid, *segs[p]))
        by_part[best].append((corners, mat))
    numbers = obedit.add_pictures(out, pictures, tex_base) if pictures else None
    data = model
    only = {int(x) for x in os.environ['OBONLY'].split(',')} if os.environ.get('OBONLY') else None
    for p in range(len(parts)):
        fs = by_part.get(p, [])
        if only is not None and p not in only:
            continue
        if parts[p][0] == 0 or fs:
            data, _ = obedit.mesh_part(data, fs, p, numbers)
        elif not os.environ.get('OBKEEP'):
            # (a part that joins two joints is left without surfaces)
            data, _ = obedit.mesh_part(data, [], p, numbers)
    target = os.path.join(out, 'OB.PAK', model_file)
    os.makedirs(os.path.dirname(target), exist_ok=True)
    open(target, 'wb').write(data)
    open(os.path.join(out, 'mod.ini'), 'w').write('enabled = true' + chr(10))
    nl = chr(10)
    open(os.path.join(out, 'characters.txt'), 'w').write(f'character {number}{nl}  copy {base}{nl}  name "{name}"{nl}  model_file {model_file}{nl}')
    used = sum(1 for p in kind0 if by_part[p])
    print(f'{len(moved)} faces on {used} of the {len(kind0)} bones; model {target}')
    print(f'characters.txt: character {number} "{name}", a copy of character {base}')


if __name__ == '__main__':
    main()
