"""What every importer gives obj2level.py: polygons with texture coordinates and colours, the materials they use, and marker points.

Points are in the axes of the source (`Scene.up` says which axis is up: 'y' or 'z'); `Scene.scale` is the factor from the source's units to
the game's (a player is about 1.8 units high there).  Texture coordinates are in the picture's own terms: (0, 0) is the top left corner of
the picture, t grows downwards, and the picture repeats outside 0 to 1.  A polygon is convex, with 3 or more points, and goes round counter-
clockwise seen from the front (the side that is drawn and that stops players).
"""
import numpy as np
from PIL import Image


class Material:
    """A surface: its picture (made when it is first asked for), or a colour when there is none."""

    def __init__(self, name, loader=None, color=(0.6, 0.6, 0.6), translucent=False):
        self.name = name
        self._loader = loader
        self._image = None
        self._done = False
        self.color = color
        self.translucent = translucent      # drawn after the solid surfaces (set from the picture's alpha when it has see-through pixels)

    def image(self):
        """The picture as RGBA, or None."""
        if not self._done:
            self._done = True
            if self._loader is not None:
                try:
                    self._image = self._loader()
                except Exception as e:      # (a picture that cannot be read: the surface gets its colour)
                    print(f'[import] {self.name}: {e}')
                    self._image = None
            if self._image is not None:
                self._image = self._image.convert('RGBA')
                a = np.asarray(self._image)[:, :, 3]
                if (a < 250).any() and (a < 250).mean() > 0.02:
                    self.translucent = True
                px = np.asarray(self._image).reshape(-1, 4)
                self.color = tuple(float(x) / 255.0 for x in px[:, :3].mean(axis=0))
        return self._image


class RawPoly:
    __slots__ = ('p', 'uv', 'col', 'mat', 'solid')

    def __init__(self, p, uv, col, mat, solid=True):
        self.p, self.uv, self.col, self.mat, self.solid = list(p), list(uv), list(col), mat, solid


class Scene:
    def __init__(self, up='y', scale=1.0, label=''):
        self.polys = []
        self.mats = {}
        self.markers = []       # (name, point)
        self.up = up
        self.scale = scale
        self.label = label
        self.collision = None     # polygons (lists of points) that collide, where the source has its own (brushes); None: the drawn ones that are solid (`RawPoly.solid`) and not see-through
        self.min_light = 0.0      # a floor of brightness for the faces (maps with lightmaps have dark ones)

    def add(self, pts, uvs, cols, mat, solid=True):
        """A convex polygon (any number of points): up to four stay as they are, more are fanned into triangles."""
        n = len(pts)
        if n < 3:
            return
        pts = [np.asarray(p, dtype=np.float64) for p in pts]
        if n <= 4:
            self.polys.append(RawPoly(pts, uvs, cols, mat, solid))
            return
        for i in range(1, n - 1):
            self.polys.append(RawPoly([pts[0], pts[i], pts[i + 1]], [uvs[0], uvs[i], uvs[i + 1]], [cols[0], cols[i], cols[i + 1]], mat, solid))


def parse_entities(text):
    """The entity lump of a BSP: a list of dicts of key -> value (`{ "classname" "info_player_start" "origin" "1 2 3" }`)."""
    out, cur, i, n = [], None, 0, len(text)
    while i < n:
        c = text[i]
        if c == '{':
            cur = {}
            i += 1
        elif c == '}':
            if cur is not None:
                out.append(cur)
            cur = None
            i += 1
        elif c == '"':
            j = text.find('"', i + 1)
            if j < 0:
                break
            k = j + 1
            while k < n and text[k] in ' \t\r\n':
                k += 1
            if k < n and text[k] == '"' and cur is not None:
                e = text.find('"', k + 1)
                if e < 0:
                    break
                cur[text[i + 1:j]] = text[k + 1:e]
                i = e + 1
            else:
                i = j + 1
        else:
            i += 1
    return out


SPAWN_CLASSES = ('info_player_start', 'info_player_deathmatch', 'info_player_teamspawn', 'info_player_terrorist', 'info_player_counterterrorist',
                 'info_player_combine', 'info_player_rebel', 'info_player_allies', 'info_player_axis', 'gmod_player_start', 'info_spawn',
                 'info_player_red', 'info_player_blue', 'info_player_coop', 'info_player_human', 'info_player_zombie')


def markers_from_entities(ents, scene):
    """Player starts become `spawn` markers, and weapons and items lying about `weapon1` .. `weapon5` (in turn)."""
    slot = 0
    for e in ents:
        cls = e.get('classname', '')
        try:
            o = np.array([float(x) for x in e.get('origin', '').split()[:3]])
        except ValueError:
            continue
        if len(o) != 3:
            continue
        if cls in SPAWN_CLASSES or cls.startswith('info_player'):
            scene.markers.append(('spawn', o))
        elif cls.startswith('weapon_') or cls.startswith('item_weapon') or cls.startswith('ent_weapon'):
            slot = slot % 5 + 1
            scene.markers.append((f'weapon{slot}', o))


def components(faces_vertices):
    """Groups of faces that are connected to each other (by a shared vertex): a list of lists of face numbers, the largest first.

    A level's playable space is one connected piece of brushes (or a few); a skybox, or a room of tools, is another."""
    parent = {}

    def find(a):
        while parent.setdefault(a, a) != a:
            parent[a] = parent[parent[a]]
            a = parent[a]
        return a
    for fv in faces_vertices:
        if not fv:
            continue
        r = find(fv[0])
        for v in fv[1:]:
            rv = find(v)
            if rv != r:
                parent[rv] = r
    groups = {}
    for i, fv in enumerate(faces_vertices):
        if fv:
            groups.setdefault(find(fv[0]), []).append(i)
    return sorted(groups.values(), key=len, reverse=True)


def keep_playable(faces_vertices, points, markers, drop_points=(), fraction=0.0):
    """The set of faces to keep: the connected groups of faces, leaving out the 3D skybox, which is the groups that contain one of the `drop_points`
    (the camera of a 3D skybox) or lie outside the box of the largest group (the skybox is out of the map's own space) unless they contain a marker
    (a player start).  The parts of the map that are not joined to the rest (tunnels, platforms, tables, the glass) are kept: `fraction` of the largest
    group is the least size of a group.  `points[v]` is the position of vertex v."""
    groups = components(faces_vertices)
    if not groups:
        return set()
    allv = np.array([points[v] for i in groups[0] for v in faces_vertices[i]])
    wlo, whi = allv.min(axis=0) - 512.0, allv.max(axis=0) + 512.0
    keep = set()
    for g in groups:
        if len(g) < fraction * len(groups[0]) and g is not groups[0]:
            continue
        vs = np.array([points[v] for i in g for v in faces_vertices[i]])
        lo, hi = vs.min(axis=0) - 1.0, vs.max(axis=0) + 1.0
        inside = lambda q: bool(np.all(q >= lo) and np.all(q <= hi))
        if g is not groups[0] and not any(inside(m) for _, m in markers):
            if any(inside(d) for d in drop_points) or np.any(vs.min(axis=0) < wlo) or np.any(vs.max(axis=0) > whi):
                continue
        keep.update(g)
    return keep


def clip_halfspace(poly, n, d):
    """The part of a convex polygon (a list of points) with n . x <= d."""
    out = []
    for k in range(len(poly)):
        a, b = poly[k], poly[(k + 1) % len(poly)]
        da, db = n @ a - d, n @ b - d
        if da <= 1e-6:
            out.append(a)
        if (da < -1e-6 and db > 1e-6) or (da > 1e-6 and db < -1e-6):
            out.append(a + (b - a) * (da / (da - db)))
    return out


def brush_polygons(planes, want=None):
    """A convex brush given by its planes `(n, d)` (n . x <= d inside): the polygon of each plane (None where there is none), going counter-
    clockwise seen from outside.  `want`: which planes' polygons to make (all by default)."""
    out = []
    for i, (n, d) in enumerate(planes):
        if want is not None and i not in want:
            out.append(None)
            continue
        a = np.array([1.0, 0, 0]) if abs(n[0]) < 0.9 else np.array([0, 1.0, 0])
        u = np.cross(n, a)
        u /= np.linalg.norm(u)
        v = np.cross(n, u)
        c = n * d
        poly = [c - u * 3e4 - v * 3e4, c + u * 3e4 - v * 3e4, c + u * 3e4 + v * 3e4, c - u * 3e4 + v * 3e4]
        if np.dot(np.cross(poly[1] - poly[0], poly[2] - poly[0]), n) < 0:
            poly.reverse()
        for j, (n2, d2) in enumerate(planes):
            if j != i and poly:
                poly = clip_halfspace(poly, n2, d2)
        if len(poly) >= 3:
            # (the pieces that are not a polygon at all: slivers and points)
            area = sum(np.linalg.norm(np.cross(poly[k] - poly[0], poly[k + 1] - poly[0])) for k in range(1, len(poly) - 1)) / 2
            out.append(poly if area > 1e-3 else None)
        else:
            out.append(None)
    return out


def newell(pts):
    """The normal of a polygon (not unit length) from all of its points: the first three may lie on a line (a map's faces have points along their
    edges), which makes a cross product of them useless."""
    n = np.zeros(3)
    k = len(pts)
    for i in range(k):
        a, b = pts[i], pts[(i + 1) % k]
        n[0] += (a[1] - b[1]) * (a[2] + b[2])
        n[1] += (a[2] - b[2]) * (a[0] + b[0])
        n[2] += (a[0] - b[0]) * (a[1] + b[1])
    return n


def brush_entity_wanted(e, opts, shapes, doors):
    """Is the brush entity `e` made part of the level: one of the classes in `shapes` that is drawn and starts enabled, or in `doors` when
    `opts['doors']` is `closed` (the default leaves doors out: a closed door would shut the level's rooms off)?  False when entities are off."""
    if opts.get('entities', True) is False:
        return False
    cn = e.get('classname', '')
    if cn not in shapes and not (cn in doors and opts.get('doors') == 'closed'):
        return False
    low = {k.lower(): v for k, v in e.items() if isinstance(v, str)}
    rm = low.get('rendermode', '0')
    return not (rm == '10' or (rm not in ('0', '') and low.get('renderamt', '255') == '0') or low.get('startdisabled') == '1')


MAX_ENTITY_SIZE = 4096.0       # (units: a brush entity wider than this is a helper that spans the map, not a shape in it)


def brush_entity_models(ents, nmodels, opts, shapes, doors, extent=None):
    """[(model number, stops players, class)] of the brush entities (those whose `model` is `*N`) that `brush_entity_wanted` takes; `extent(m)` gives the
    widest side of the model's box, and a model wider than `MAX_ENTITY_SIZE` is left out."""
    out, seen = [], set()
    for e in ents:
        low = {k.lower(): v for k, v in e.items()}
        model = low.get('model', '')
        if not model.startswith('*') or not model[1:].isdigit():
            continue
        m = int(model[1:])
        if m == 0 or m >= nmodels or m in seen or not brush_entity_wanted(e, opts, shapes, doors):
            continue
        if extent is not None and extent(m) > MAX_ENTITY_SIZE:
            continue
        seen.add(m)
        out.append((m, low.get('solidity', '0') != '1', e.get('classname', '')))
    return out
