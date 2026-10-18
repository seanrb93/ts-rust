"""glTF 2.0 (.gltf with its files, or .glb): the format Blender exports to for games, and most other tools can too.

Meshes (triangles, strips and fans), their materials (base colour picture and factor, alpha mode), vertex colours (COLOR_0) and the node
tree's transforms are read.  A node without a mesh whose name begins with `spawn`, `weapon` or `pickup` is a marker (an Empty in Blender); so is a
mesh node with such a name (its origin is the spot, and it is not part of the level).
"""
import base64
import io
import json
import os
import struct

import numpy as np
from PIL import Image

from .common import Material, Scene

MARKER_PREFIXES = ('spawn', 'weapon', 'pickup')
COMPONENT = {5120: ('b', 1), 5121: ('B', 1), 5122: ('h', 2), 5123: ('H', 2), 5125: ('I', 4), 5126: ('f', 4)}
NCOMP = {'SCALAR': 1, 'VEC2': 2, 'VEC3': 3, 'VEC4': 4, 'MAT4': 16}


class Gltf:
    def __init__(self, path):
        self.base = os.path.dirname(path)
        data = open(path, 'rb').read()
        self.bin = None
        if data[:4] == b'glTF':
            at = 12
            while at + 8 <= len(data):
                ln, kind = struct.unpack_from('<II', data, at)
                body = data[at + 8:at + 8 + ln]
                if kind == 0x4E4F534A:
                    self.j = json.loads(body.decode('utf8'))
                elif kind == 0x004E4942 and self.bin is None:
                    self.bin = body
                at += 8 + ln + (-ln % 4)
        else:
            self.j = json.loads(data.decode('utf8'))
        self.buffers = {}

    def buffer(self, i):
        if i not in self.buffers:
            b = self.j['buffers'][i]
            uri = b.get('uri')
            if uri is None:
                self.buffers[i] = self.bin
            elif uri.startswith('data:'):
                self.buffers[i] = base64.b64decode(uri.split(',', 1)[1])
            else:
                self.buffers[i] = open(os.path.join(self.base, _unquote(uri)), 'rb').read()
        return self.buffers[i]

    def accessor(self, i):
        a = self.j['accessors'][i]
        fmt, size = COMPONENT[a['componentType']]
        n = NCOMP[a['type']]
        count = a['count']
        if 'bufferView' not in a:
            return np.zeros((count, n))
        v = self.j['bufferViews'][a['bufferView']]
        buf = self.buffer(v['buffer'])
        at = v.get('byteOffset', 0) + a.get('byteOffset', 0)
        stride = v.get('byteStride') or size * n
        out = np.empty((count, n), dtype=np.float64)
        if stride == size * n:
            arr = np.frombuffer(buf, dtype='<' + fmt if fmt != 'f' else '<f4', count=count * n, offset=at)
            out = arr.reshape(count, n).astype(np.float64)
        else:
            for k in range(count):
                out[k] = struct.unpack_from('<' + fmt * n, buf, at + k * stride)
        if a.get('normalized'):
            out = out / {'b': 127.0, 'B': 255.0, 'h': 32767.0, 'H': 65535.0}.get(fmt, 1.0)
        return out

    def image(self, i):
        im = self.j['images'][i]
        if 'uri' in im:
            uri = im['uri']
            if uri.startswith('data:'):
                return Image.open(io.BytesIO(base64.b64decode(uri.split(',', 1)[1])))
            return Image.open(os.path.join(self.base, _unquote(uri)))
        v = self.j['bufferViews'][im['bufferView']]
        buf = self.buffer(v['buffer'])
        at = v.get('byteOffset', 0)
        return Image.open(io.BytesIO(bytes(buf[at:at + v['byteLength']])))


def _unquote(s):
    from urllib.parse import unquote
    return unquote(s)


def _local(node):
    if 'matrix' in node:
        return np.array(node['matrix'], dtype=np.float64).reshape(4, 4).T
    t = np.array(node.get('translation', [0, 0, 0]), dtype=np.float64)
    q = node.get('rotation', [0, 0, 0, 1])
    s = np.array(node.get('scale', [1, 1, 1]), dtype=np.float64)
    x, y, z, w = q
    r = np.array([[1 - 2 * (y * y + z * z), 2 * (x * y - z * w), 2 * (x * z + y * w)],
                  [2 * (x * y + z * w), 1 - 2 * (x * x + z * z), 2 * (y * z - x * w)],
                  [2 * (x * z - y * w), 2 * (y * z + x * w), 1 - 2 * (x * x + y * y)]])
    m = np.eye(4)
    m[:3, :3] = r * s
    m[:3, 3] = t
    return m


def load(path, opts):
    g = Gltf(path)
    j = g.j
    sc = Scene(up='y', scale=1.0, label=os.path.basename(path))
    mats = j.get('materials', [])

    def material(i):
        name = (mats[i].get('name') or f'material{i}') if i is not None and i < len(mats) else 'default'
        key = f'{name}#{i}'
        if key in sc.mats:
            return key
        color, loader, translucent = (0.8, 0.8, 0.8), None, False
        if i is not None and i < len(mats):
            m = mats[i]
            pbr = m.get('pbrMetallicRoughness', {})
            f = pbr.get('baseColorFactor', [1, 1, 1, 1])
            color = tuple(f[:3])
            translucent = m.get('alphaMode') == 'BLEND' or f[3] < 0.99
            t = pbr.get('baseColorTexture')
            if t is not None and 'source' in j['textures'][t['index']]:
                src = j['textures'][t['index']]['source']
                tint = tuple(f[:3])

                def loader(src=src, tint=tint):
                    im = g.image(src).convert('RGBA')
                    if tint != (1.0, 1.0, 1.0):
                        a = np.asarray(im).astype(np.float64)
                        a[:, :, :3] *= np.array(tint)
                        im = Image.fromarray(np.clip(a, 0, 255).astype(np.uint8), 'RGBA')
                    return im
        sc.mats[key] = Material(key, loader, color, translucent)
        return key

    scene_nodes = j['scenes'][j.get('scene', 0)]['nodes'] if j.get('scenes') else range(len(j.get('nodes', [])))

    def walk(i, parent):
        node = j['nodes'][i]
        m = parent @ _local(node)
        name = node.get('name', '')
        is_marker = name.lower().startswith(MARKER_PREFIXES)
        if is_marker:
            sc.markers.append((name, m[:3, 3].copy()))
        elif 'mesh' in node:
            mesh(j['meshes'][node['mesh']], m)
        for c in node.get('children', []):
            walk(c, m)

    def mesh(me, m):
        for prim in me['primitives']:
            attrs = prim['attributes']
            if 'POSITION' not in attrs:
                continue
            pos = g.accessor(attrs['POSITION'])
            pos = (np.c_[pos, np.ones(len(pos))] @ m.T)[:, :3]
            uv = g.accessor(attrs['TEXCOORD_0']) if 'TEXCOORD_0' in attrs else np.zeros((len(pos), 2))
            col = g.accessor(attrs['COLOR_0'])[:, :3] if 'COLOR_0' in attrs else None
            if prim.get('indices') is not None:
                idx = g.accessor(prim['indices']).astype(np.int64).reshape(-1)
            else:
                idx = np.arange(len(pos))
            mode = prim.get('mode', 4)
            if mode == 4:
                tris = idx[:len(idx) // 3 * 3].reshape(-1, 3)
            elif mode == 5:
                tris = np.array([(idx[k], idx[k + 1], idx[k + 2]) if k % 2 == 0 else (idx[k + 1], idx[k], idx[k + 2]) for k in range(len(idx) - 2)])
            elif mode == 6:
                tris = np.array([(idx[0], idx[k], idx[k + 1]) for k in range(1, len(idx) - 1)])
            else:
                continue
            key = material(prim.get('material'))
            for a, b, c in tris:
                if a == b or b == c or a == c:
                    continue
                tri = (a, b, c)
                sc.add([pos[v] for v in tri], [tuple(uv[v][:2]) for v in tri], [tuple(col[v]) if col is not None else (1.0, 1.0, 1.0) for v in tri], key)

    for n in scene_nodes:
        walk(n, np.eye(4))
    return sc
