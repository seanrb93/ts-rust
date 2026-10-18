"""Blender files (.blend): Blender itself is run in the background to export the scene as glTF, which is then read.

Blender is looked for in the BLENDER environment variable, `--blender`, the PATH, and the places it installs to on Windows, macOS and Linux.
Modifiers are applied, and Empties are kept (an Empty named `spawn`, `weapon1` .. `weapon5` or `pickup<type>` is a marker).
"""
import glob
import os
import shutil
import subprocess
import sys
import tempfile

from . import gltf

SCRIPT = r'''
import bpy, sys
out = sys.argv[sys.argv.index("--") + 1]
kw = dict(filepath=out, export_format="GLB", export_apply=True, export_yup=True)
try:
    bpy.ops.export_scene.gltf(export_cameras=False, export_lights=False, **kw)
except TypeError:
    bpy.ops.export_scene.gltf(**kw)
'''


def find_blender(opts):
    for c in (opts.get('blender'), os.environ.get('BLENDER'), shutil.which('blender')):
        if c and os.path.exists(c):
            return c
    pats = [r'C:\Program Files\Blender Foundation\Blender*\blender.exe', r'C:\Program Files (x86)\Steam\steamapps\common\Blender\blender.exe',
            '/Applications/Blender.app/Contents/MacOS/Blender', '/usr/bin/blender', '/snap/bin/blender', '/opt/blender*/blender']
    for p in pats:
        hits = sorted(glob.glob(p))
        if hits:
            return hits[-1]
    return None


def load(path, opts):
    exe = find_blender(opts)
    if not exe:
        sys.exit('Blender was not found: install it, set the BLENDER environment variable or give --blender <path>, or export the scene as .glb from Blender (File > Export > glTF 2.0) and convert that')
    with tempfile.TemporaryDirectory() as d:
        script = os.path.join(d, 'export.py')
        out = os.path.join(d, 'scene.glb')
        open(script, 'w').write(SCRIPT)
        r = subprocess.run([exe, '--background', path, '--python', script, '--', out], capture_output=True, text=True)
        if not os.path.exists(out):
            sys.exit('Blender could not export the file:\n' + (r.stdout + r.stderr)[-1500:])
        sc = gltf.load(out, opts)
        # (the pictures are read now: the temporary folder goes away)
        for m in sc.mats.values():
            m.image()
        sc.label = os.path.basename(path)
        return sc
