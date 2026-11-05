"""Make the Smooth Weapons mod: the models of the guns with more polygons where they are curved.

    python tools/make-smooth-weapons.py <OB.PAK> <mods folder> [name in the pack ...]

Writes <mods folder>/smooth_weapons_mod/OB.PAK/<name> for the first person guns (`ob/gunN/gunN.raw`) and those the characters hold (`ob/chrgunN/chrgunN.raw`), or the ones named, and a mod.ini.  The game's models stay out of
the repository: the mod is written next to the game, and this script is what is kept.

Each triangle becomes a curved patch, as in "curved PN triangles" (Vlachos and others, 2001): a cubic surface through its corners, from the normals there.  The normals are worked out from the faces: faces that meet at less than
CREASE degrees are one smooth surface and share a normal, and an edge sharper than that (or with no face on the other side) stays a straight line.  So a barrel or a drum goes round, and a box keeps its corners and flat sides, which
get no more polygons.  First Person Hands replaces these files when its arms are wanted (its script asks for its own names first), so with that mod on the guns are only smoothed in the third person, unless it is
made from these with HANDS_BASE=<this mod's folder>.  A patch is split only where the surface leaves the flat triangle by more than TOLERANCE, and that is decided edge by edge from the curve of the edge alone, so that the two faces at an edge split it alike and
no crack opens between them.
"""
import contextlib
import importlib.util
import io
import math
import os
import sys
import tempfile

import numpy as np

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import obdump  # noqa: E402
import obedit  # noqa: E402
import pakls  # noqa: E402

# (the taking out of a part's old data, which the first person hands need as well)
_spec = importlib.util.spec_from_file_location('make_hands', os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'examples', 'first_person_hands_src', 'make-hands.py'))
make_hands = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(make_hands)

CREASE = 62.0       # degrees: a sharper edge between two faces is kept
TOLERANCE = 0.0012  # how far the surface can be from its triangles, in the model's units (a gun is about 1 long)
MAX_DEPTH = 3       # an edge is cut in two at most so many times over
WELD = 1.0e-4       # points nearer than this are one point
FLASH_POINTS = 12   # a part of so few points that is large is the flash of the muzzle: it is left alone


def key_of(p):
    return tuple(int(round(c / WELD)) for c in p)


def smooth_normals(faces):
    """For each face its normal, and the normal of each corner: the faces around a corner that are joined by soft edges share one.  Also whether each edge of the face is soft."""
    points = [[np.array(c[0], dtype=float) for c in corners] for corners, _ in faces]
    normals, areas = [], []
    for a, b, c in points:
        n = np.cross(b - a, c - a)
        size = np.linalg.norm(n)
        areas.append(size)
        normals.append(n / size if size > 1.0e-12 else None)
    edges = {}
    for fi, ps in enumerate(points):
        if normals[fi] is None:
            continue
        keys = [key_of(p) for p in ps]
        for i in range(3):
            edges.setdefault(frozenset((keys[i], keys[(i + 1) % 3])), []).append((fi, i))
    limit = math.cos(math.radians(CREASE))
    soft = set()
    for pair in edges.values():
        if len(pair) == 2 and float(normals[pair[0][0]] @ normals[pair[1][0]]) > limit:
            soft.add((pair[0][0], pair[0][1]))
            soft.add((pair[1][0], pair[1][1]))
    # (the corners that are joined over soft edges)
    parent = {}

    def find(x):
        parent.setdefault(x, x)
        while parent[x] != x:
            parent[x] = parent[parent[x]]
            x = parent[x]
        return x

    for pair in edges.values():
        if len(pair) != 2 or (pair[0][0], pair[0][1]) not in soft:
            continue
        (f0, e0), (f1, e1) = pair
        keys0 = [key_of(p) for p in points[f0]]
        keys1 = [key_of(p) for p in points[f1]]
        for k in (e0, (e0 + 1) % 3):
            parent[find((f0, k))] = find((f1, keys1.index(keys0[k])))
    total = {}
    for fi, ps in enumerate(points):
        if normals[fi] is None:
            continue
        for k in range(3):
            u, v = ps[(k + 1) % 3] - ps[k], ps[(k + 2) % 3] - ps[k]
            angle = math.acos(max(-1.0, min(1.0, float(u @ v) / max(np.linalg.norm(u) * np.linalg.norm(v), 1.0e-12))))
            root = find((fi, k))
            total[root] = total.get(root, np.zeros(3)) + normals[fi] * angle
    corner_normals = {}
    for fi in range(len(points)):
        if normals[fi] is None:
            continue
        for k in range(3):
            n = total[find((fi, k))]
            corner_normals[(fi, k)] = n / max(np.linalg.norm(n), 1.0e-12)
    return points, normals, corner_normals, soft


def patch_of(p, n, soft):
    """The control points of the cubic patch of a triangle: the corners, two on each edge (on the line for a hard edge, so the edge stays straight) and the middle."""
    c = {}
    for i in range(3):
        j = (i + 1) % 3
        if i in soft:
            c[(i, j)] = (2.0 * p[i] + p[j] - float((p[j] - p[i]) @ n[i]) * n[i]) / 3.0
            c[(j, i)] = (2.0 * p[j] + p[i] - float((p[i] - p[j]) @ n[j]) * n[j]) / 3.0
        else:
            c[(i, j)] = (2.0 * p[i] + p[j]) / 3.0
            c[(j, i)] = (2.0 * p[j] + p[i]) / 3.0
    e = sum(c.values()) / 6.0
    v = (p[0] + p[1] + p[2]) / 3.0
    return c, e + (e - v) / 2.0


def surface(bary, p, c, middle):
    """The point of the patch at the barycentric place `bary`."""
    u, v, w = bary
    return (p[0] * u ** 3 + p[1] * v ** 3 + p[2] * w ** 3 + 6.0 * middle * u * v * w
            + 3.0 * (c[(0, 1)] * u * u * v + c[(1, 0)] * v * v * u + c[(1, 2)] * v * v * w + c[(2, 1)] * w * w * v + c[(2, 0)] * w * w * u + c[(0, 2)] * u * u * w))


def tessellate(face, p, n, soft_edges):
    """The triangles (corners as `obedit.read_obj` gives them) of a face made curved."""
    corners, _ = face
    c, middle = patch_of(p, n, soft_edges)
    st = [np.array(x[1], dtype=float) for x in corners]
    colour = [np.array(x[3], dtype=float) if x[3] else None for x in corners]
    out = []

    def corner(b):
        bary = np.array(b, dtype=float)
        pos = surface(bary, p, c, middle)
        nrm = bary[0] * n[0] + bary[1] * n[1] + bary[2] * n[2]
        nrm = nrm / max(np.linalg.norm(nrm), 1.0e-12)
        tex = bary[0] * st[0] + bary[1] * st[1] + bary[2] * st[2]
        col = None if colour[0] is None else tuple(int(round(v)) for v in bary[0] * colour[0] + bary[1] * colour[1] + bary[2] * colour[2])
        return (tuple(float(x) for x in pos), (float(tex[0]), float(tex[1])), tuple(float(x) for x in nrm), col)

    def must_split(a, b):
        # (the curve between two places on an edge: the same for the two faces at the edge)
        mid = (np.array(a) + np.array(b)) / 2.0
        chord = (surface(np.array(a), p, c, middle) + surface(np.array(b), p, c, middle)) / 2.0
        return float(np.linalg.norm(surface(mid, p, c, middle) - chord)) > TOLERANCE

    def split(a, b, cc, depth):
        if depth >= MAX_DEPTH:
            out.append([corner(a), corner(b), corner(cc)])
            return
        ab, bc, ca = must_split(a, b), must_split(b, cc), must_split(cc, a)
        half = lambda x, y: tuple((np.array(x) + np.array(y)) / 2.0)
        m_ab, m_bc, m_ca = half(a, b), half(b, cc), half(cc, a)
        count = ab + bc + ca
        if count == 0:
            centre = tuple((np.array(a) + np.array(b) + np.array(cc)) / 3.0)
            flat = (surface(np.array(a), p, c, middle) + surface(np.array(b), p, c, middle) + surface(np.array(cc), p, c, middle)) / 3.0
            if float(np.linalg.norm(surface(np.array(centre), p, c, middle) - flat)) > TOLERANCE:
                for x, y in ((a, b), (b, cc), (cc, a)):
                    out.append([corner(x), corner(y), corner(centre)])
            else:
                out.append([corner(a), corner(b), corner(cc)])
        elif count == 3:
            split(a, m_ab, m_ca, depth + 1)
            split(m_ab, b, m_bc, depth + 1)
            split(m_ca, m_bc, cc, depth + 1)
            split(m_ab, m_bc, m_ca, depth + 1)
        elif count == 1:
            if ab:
                split(a, m_ab, cc, depth + 1)
                split(m_ab, b, cc, depth + 1)
            elif bc:
                split(a, b, m_bc, depth + 1)
                split(a, m_bc, cc, depth + 1)
            else:
                split(a, b, m_ca, depth + 1)
                split(b, cc, m_ca, depth + 1)
        else:
            if not ab:
                split(a, b, m_bc, depth + 1)
                split(a, m_bc, m_ca, depth + 1)
                split(m_ca, m_bc, cc, depth + 1)
            elif not bc:
                split(m_ab, b, cc, depth + 1)
                split(a, m_ab, m_ca, depth + 1)
                split(m_ab, cc, m_ca, depth + 1)
            else:
                split(a, m_ab, m_bc, depth + 1)
                split(m_ab, b, m_bc, depth + 1)
                split(a, m_bc, cc, depth + 1)

    split((1.0, 0.0, 0.0), (0.0, 1.0, 0.0), (0.0, 0.0, 1.0), 0)
    return out


def smooth_part(faces):
    """The faces of a part of a model, made curved."""
    faces = [(c, m) for corners, m in faces for c in ([corners] if len(corners) == 3 else [[corners[0], corners[1], corners[2]], [corners[0], corners[2], corners[3]]])]
    points, normals, corner_normals, soft = smooth_normals(faces)
    out = []
    for fi, face in enumerate(faces):
        if normals[fi] is None:
            continue
        n = [corner_normals[(fi, k)] for k in range(3)]
        soft_edges = {i for i in range(3) if (fi, i) in soft}
        if not soft_edges:
            # (a face with no soft edge stays as it is, with the game's own normals)
            out.append(face)
            continue
        out += [(t, face[1]) for t in tessellate(face, points[fi], n, soft_edges)]
    return out


def smooth_model(pak, name, data):
    """The file of a model with its parts made curved, and the number of triangles before and after."""
    with tempfile.TemporaryDirectory() as tmp:
        path = os.path.join(tmp, 'model.obj')
        with contextlib.redirect_stdout(io.StringIO()):
            obdump.write_obj(data, path)
        groups = {}
        obedit.read_obj(path, groups)
    before = after = 0
    for part, faces in sorted(groups.items()):
        if not (part and part.startswith('part') and part[4:].isdigit()):
            continue
        number = int(part[4:])
        points = np.array([c[0] for corners, _ in faces for c in corners])
        keys = {key_of(p) for p in points}
        if len(keys) <= FLASH_POINTS and np.ptp(points, axis=0).max() > 0.15:
            continue
        if any(not (m or '').endswith('_k3') for _, m in faces):
            continue
        made = smooth_part(faces)
        before += len(faces)
        after += len(made)
        previous = data
        data, _ = obedit.mesh_part(data, made, number)
        small = make_hands.drop_old_part_data(previous, data, number)
        if small is not data and make_hands.same_geometry(data, small):
            data = small
    return data, before, after


def main():
    if len(sys.argv) < 3:
        print(__doc__)
        sys.exit(1)
    pak, mods = sys.argv[1], sys.argv[2]
    d, entries = pakls.entries(pak)
    wanted = sys.argv[3:]
    mod = os.path.join(mods, 'smooth_weapons_mod')
    for name, o, size in entries:
        if wanted and name not in wanted:
            continue
        if not wanted and not ((name.startswith('ob/gun') or name.startswith('ob/chrgun')) and name.endswith('.raw')):
            continue
        data, before, after = smooth_model(pak, name, d[o:o + size])
        path = os.path.join(mod, 'OB.PAK', name)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, 'wb') as f:
            f.write(data)
        print(f'{name}: {before} to {after} triangles, {size} to {len(data)} bytes')
    with open(os.path.join(mod, 'mod.ini'), 'w', encoding='utf-8') as f:
        f.write('name = Smooth Weapons\nenabled = true\n')


if __name__ == '__main__':
    main()
