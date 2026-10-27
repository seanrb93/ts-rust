"""Props of Source maps: the models (`.mdl` with its `.vvd` and `.dx90.vtx`) that a map places as `prop_static` (the static prop lump of a BSP, or the
entity of a VMF) and as `prop_dynamic`, `prop_physics` and the like (an entity with a `model`).  Each is put into the level as geometry, where the map
starts it, in its first skin and its most detailed level; bones, animation, physics and the models' own lights are not used.

The models are looked for in the BSP's own pack, then in the `models` folders given with `--models` and in the folders next to the `materials` folders
that are used (a `models` folder beside the map or in a parent folder, as Crowbar unpacks a workshop map).  The game's own models (`models/props_c17/...`) are
not in the map: unpack them with a tool such as GCFScape or Crowbar and give the `models` folder.  A prop whose model is not found is left out.
"""
import math
import os
import struct

import numpy as np

# classes of entities that place a model
PROP_ENTITIES = ('prop_static', 'prop_dynamic', 'prop_dynamic_override', 'prop_physics', 'prop_physics_override', 'prop_physics_multiplayer',
                 'prop_ragdoll', 'prop_detail', 'prop_door_rotating')


class Models:
    """Finds a model's files."""

    def __init__(self, pak, mats_dirs, extra_dirs):
        self.pak = pak
        self.pakmap = {n.lower(): n for n in pak.namelist()} if pak else {}
        self.dirs = []
        for d in list(extra_dirs or []) + [os.path.join(os.path.dirname(m), 'models') for m in mats_dirs]:
            if os.path.isdir(d) and d not in self.dirs:
                self.dirs.append(d)

    def read(self, rel):
        rel = rel.replace('\\', '/').lower()
        if rel in self.pakmap:
            return self.pak.read(self.pakmap[rel])
        sub = rel[len('models/'):] if rel.startswith('models/') else rel
        for d in self.dirs:
            p = os.path.join(d, sub)
            if os.path.exists(p):
                return open(p, 'rb').read()
        return None


def angle_matrix(pitch, yaw, roll):
    """The rotation of Source's angles (degrees)."""
    sp, cp = math.sin(math.radians(pitch)), math.cos(math.radians(pitch))
    sy, cy = math.sin(math.radians(yaw)), math.cos(math.radians(yaw))
    sr, cr = math.sin(math.radians(roll)), math.cos(math.radians(roll))
    return np.array([
        [cp * cy, sr * sp * cy + cr * -sy, cr * sp * cy + -sr * -sy],
        [cp * sy, sr * sp * sy + cr * cy, cr * sp * sy + -sr * cy],
        [-sp, sr * cp, cr * cp]])


def static_props(d, gamelump, version):
    """[(model name, origin, angles, skin, scale)] of the static prop lump (`sprp`) of a BSP; `gamelump` is the data of the game lump (lump 35)."""
    if len(gamelump) < 4:
        return []
    n = struct.unpack_from('<i', gamelump, 0)[0]
    for i in range(n):
        gid, flags, ver, ofs, ln = struct.unpack_from('<IHHii', gamelump, 4 + 16 * i)
        if gid != 0x73707270:                 # 'sprp'
            continue
        if flags & 1:
            return []                         # (compressed)
        data = d[ofs:ofs + ln]
        ndict = struct.unpack_from('<i', data, 0)[0]
        names = [data[4 + 128 * k:4 + 128 * (k + 1)].split(b'\0')[0].decode('latin1') for k in range(ndict)]
        at = 4 + 128 * ndict
        nleaf = struct.unpack_from('<i', data, at)[0]
        at += 4 + 2 * nleaf
        nprops = struct.unpack_from('<i', data, at)[0]
        at += 4
        if nprops <= 0:
            return []
        stride = (len(data) - at) // nprops
        out = []
        for k in range(nprops):
            o = at + k * stride
            ox, oy, oz, ax, ay, az = struct.unpack_from('<6f', data, o)
            ptype = struct.unpack_from('<H', data, o + 24)[0]
            skin = struct.unpack_from('<i', data, o + 32)[0] if stride >= 36 else 0
            if ptype < len(names):
                out.append((names[ptype], (ox, oy, oz), (ax, ay, az), skin, 1.0))
        return out
    return []


def entity_props(ents):
    """[(model name, origin, angles, skin, scale)] of the entities that place a model (`prop_dynamic`, `prop_physics`, ...; `prop_static` of a VMF)."""
    out = []
    for e in ents:
        if e.get('classname') not in PROP_ENTITIES:
            continue
        model = e.get('model', '')
        if not model.lower().endswith('.mdl'):
            continue
        low = {k.lower(): v for k, v in e.items() if isinstance(v, str)}
        rm = low.get('rendermode', '0')
        if rm == '10' or (rm not in ('0', '') and low.get('renderamt', '255') == '0') or low.get('startdisabled') == '1':
            continue
        try:
            org = tuple(float(x) for x in low.get('origin', '0 0 0').split())
            ang = tuple(float(x) for x in low.get('angles', '0 0 0').split())
            skin = int(low.get('skin', '0') or 0)
            scale = float(low.get('modelscale', '1') or 1)
        except ValueError:
            continue
        if len(org) == 3 and len(ang) == 3:
            out.append((model, org, ang, skin, scale))
    return out


def read_model(models, name):
    """[(material paths, [(position, normal, (u, v)) of each triangle's three points])] of the model `name`, or None when its files are not found.  The
    material paths are the places the model's material may be (`materials/<dir>/<name>`, without the extension); skin 0."""
    base = name.replace('\\', '/').lower()
    if base.endswith('.mdl'):
        base = base[:-4]
    mdl = models.read(base + '.mdl')
    vvd = models.read(base + '.vvd')
    vtx = models.read(base + '.dx90.vtx') or models.read(base + '.dx80.vtx') or models.read(base + '.sw.vtx')
    if not (mdl and vvd and vtx) or mdl[:4] != b'IDST' or vvd[:4] != b'IDSV':
        return None
    numtex, texidx, numcd, cdidx, numskinref, _numfam, skinidx, numbody, bodyidx = struct.unpack_from('<9i', mdl, 204)
    # (the textures' names and the directories they are looked for in)
    texnames = []
    for i in range(numtex):
        o = texidx + 64 * i
        nameofs = struct.unpack_from('<i', mdl, o)[0]
        s = o + nameofs
        texnames.append(mdl[s:mdl.index(b'\0', s)].decode('latin1').replace('\\', '/').lower())
    cds = []
    for i in range(numcd):
        ofs = struct.unpack_from('<i', mdl, cdidx + 4 * i)[0]
        cds.append(mdl[ofs:mdl.index(b'\0', ofs)].decode('latin1').replace('\\', '/').lower())

    def material_paths(ti):
        """The places a texture's material may be: each of the model's material directories, in turn."""
        if not 0 <= ti < len(texnames):
            return []
        return ['materials/' + (cd.strip('/') + '/' + texnames[ti]).strip('/') for cd in cds] or ['materials/' + texnames[ti]]
    # skin 0: the table says which texture each skin reference is
    skin = [struct.unpack_from('<h', mdl, skinidx + 2 * k)[0] for k in range(numskinref)] if numskinref else []

    # the vertices of the most detailed level (with the fixups of the file)
    nverts0 = struct.unpack_from('<i', vvd, 16)[0]          # (the number of vertices of the most detailed level)
    numfix, fixofs, vofs = struct.unpack_from('<3i', vvd, 48)
    raw = np.frombuffer(vvd, dtype=np.uint8, count=48 * nverts0 if numfix == 0 else None, offset=vofs)
    allv = raw[:len(raw) // 48 * 48].reshape(-1, 48)
    if numfix:
        picks = []
        for k in range(numfix):
            lod, src, cnt = struct.unpack_from('<3i', vvd, fixofs + 12 * k)
            if lod >= 0:
                picks.append(allv[src:src + cnt])
        verts = np.concatenate(picks) if picks else allv[:0]
    else:
        verts = allv[:nverts0]
    f = verts.copy().view('<f4') if len(verts) else np.zeros((0, 12), dtype='<f4')
    pos = f[:, 4:7].astype(np.float64)
    nrm = f[:, 7:10].astype(np.float64)
    uv = f[:, 10:12].astype(np.float64)

    out = {}
    # the meshes of the model: body parts of the .mdl and of the .vtx go together
    vtx_bodyparts, vtx_bodyofs = struct.unpack_from('<2i', vtx, 28)
    for bp in range(numbody):
        _bn, nmodels, _base, modelidx = struct.unpack_from('<4i', mdl, bodyidx + 16 * bp)
        if nmodels < 1:
            continue
        # (the first model of each body part)
        mo = bodyidx + 16 * bp + modelidx
        nmeshes, meshidx, nvert, vertidx = struct.unpack_from('<4i', mdl, mo + 72)
        model_vertex_start = vertidx // 48
        vb = vtx_bodyofs + 8 * bp
        vnm, vmo = struct.unpack_from('<2i', vtx, vb)
        if vnm < 1:
            continue
        vm = vb + vmo
        vnl, vlo = struct.unpack_from('<2i', vtx, vm)
        if vnl < 1:
            continue
        vl = vm + vlo
        vnmesh, vmeshofs = struct.unpack_from('<2i', vtx, vl)
        for mi in range(min(nmeshes, vnmesh)):
            mm = mo + meshidx + 116 * mi
            matref, _mdlidx, _nv, vertoffset = struct.unpack_from('<4i', mdl, mm)
            ti = skin[matref] if 0 <= matref < len(skin) else matref
            mat = tuple(material_paths(ti))
            vmh = vl + vmeshofs + 9 * mi
            nsg, sgofs = struct.unpack_from('<2i', vtx, vmh)
            for sg in range(nsg):
                g = vmh + sgofs + 25 * sg
                gnv, gvo, gni, gio, gns, gso = struct.unpack_from('<6i', vtx, g)
                gverts = [struct.unpack_from('<H', vtx, g + gvo + 9 * k + 4)[0] for k in range(gnv)]
                inds = struct.unpack_from('<%dH' % gni, vtx, g + gio) if gni else ()
                for st in range(gns):
                    s = g + gso + 27 * st
                    sni, sio = struct.unpack_from('<2i', vtx, s)
                    sflags = vtx[s + 18]
                    seq = inds[sio:sio + sni]
                    tris = []
                    if sflags & 2:                  # a strip
                        for k in range(len(seq) - 2):
                            tris.append((seq[k], seq[k + 1], seq[k + 2]) if k % 2 == 0 else (seq[k + 1], seq[k], seq[k + 2]))
                    else:                           # a list
                        tris = [tuple(seq[k:k + 3]) for k in range(0, len(seq) - 2, 3)]
                    for tri in tris:
                        ids = []
                        for ix in tri:
                            if ix >= len(gverts):
                                ids = None
                                break
                            vi = gverts[ix] + vertoffset + model_vertex_start
                            if vi >= len(pos):
                                ids = None
                                break
                            ids.append(vi)
                        if ids is None or len(set(ids)) < 3:
                            continue
                        out.setdefault(mat, []).append([(pos[v], nrm[v], (float(uv[v][0]), float(uv[v][1]))) for v in ids])
    return list(out.items())


def add_props(sc, props, models, material, exists, opts):
    """Puts the props into the scene `sc`; `material(path)` makes the scene's material of a Source material path (like `concrete/wall01`) and gives its name,
    `exists(path)` says whether the map or the folders have that material file."""
    cache = {}
    n_props = n_polys = missing = 0
    miss_names = set()
    shade = max(sc.min_light, 0.75)
    # (the props that are big for their polygons first: when they are more than the budget of polygons the rest are left out)
    budget = int(opts.get('prop_budget', 20000))
    sizes = {}

    def size_of(key, name):
        if key not in sizes:
            try:
                meshes = read_model(models, name)
            except Exception:
                meshes = None
            cache[key] = meshes
            if meshes:
                pts = np.array([p for _mat, tris in meshes for tri in tris for p, _n, _uv in tri])
                sizes[key] = (float(np.prod(pts.max(axis=0) - pts.min(axis=0) + 1.0)), sum(len(t) for _m, t in meshes))
            else:
                sizes[key] = (0.0, 0)
        return sizes[key]
    ordered = sorted(props, key=lambda pr: -(size_of(pr[0].lower(), pr[0])[0] * pr[4] ** 3) / max(size_of(pr[0].lower(), pr[0])[1], 1))
    used, dropped = 0, 0
    kept = []
    for pr in ordered:
        tris = size_of(pr[0].lower(), pr[0])[1]
        if budget > 0 and used + tris > budget:
            dropped += 1
            continue
        used += tris
        kept.append(pr)
    props = kept
    for name, org, ang, skin, scale in props:
        key = name.lower()
        if key not in cache:
            try:
                cache[key] = read_model(models, name)
            except Exception as e:                   # (a model that cannot be read: left out)
                print(f'[props] {name}: {e}')
                cache[key] = None
        meshes = cache[key]
        if meshes is None:
            missing += 1
            miss_names.add(key)
            continue
        m = angle_matrix(*ang) * scale
        o = np.array(org)
        n_props += 1
        for mat, tris in meshes:
            paths = [m_ for m_ in mat if exists(m_ + '.vmt')] or list(mat) or ['materials/tools/toolsnodraw']
            matname = material(paths[0][len('materials/'):])
            for tri in tris:
                pts = [m @ p + o for p, _n, _uv in tri]
                nr = [m @ nn for _p, nn, _uv in tri]
                face = np.cross(pts[1] - pts[0], pts[2] - pts[0])
                avg = nr[0] + nr[1] + nr[2]
                order = [0, 1, 2]
                if np.dot(face, avg) < 0:
                    order = [0, 2, 1]
                cols = [(shade, shade, shade)] * 3
                sc.add([pts[i] for i in order], [(tri[i][2][0], tri[i][2][1]) for i in order], cols, matname)
                n_polys += 1
    msg = f'props: {n_props} placed, {n_polys} polygons'
    if dropped:
        msg += f', {dropped} small ones left out to keep to {budget} polygons (--prop-budget)'
    if missing:
        msg += f', {missing} left out (no model files for {len(miss_names)} models: {", ".join(sorted(miss_names)[:3])}{" ..." if len(miss_names) > 3 else ""})'
    print(msg)
