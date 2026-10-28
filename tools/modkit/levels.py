"""Levels: the level converter (a map or a 3D model becomes a level of the game) put into the mod being made."""
import os
import re
import shutil
import sys

from . import ui


def make(ctx):
    mod = ctx.need_mod()
    if not mod:
        return
    ui.heading('Make a level from a map or a 3D model')
    ui.say('Works with Source maps (.bsp, .vmf: Half-Life 2, Garry\'s Mod, CS:S, ...), GoldSrc maps (.bsp), OBJ, glTF/GLB and Blender (.blend) files.')
    path = ui.pick_file('Choose the map or model', [('Maps and models', '*.bsp *.vmf *.obj *.glb *.gltf *.blend *.bz2'), ('All files', '*.*')])
    if not path:
        return
    stem = os.path.basename(path)
    for ext in ('.bz2', '.bsp', '.vmf', '.obj', '.glb', '.gltf', '.blend'):
        if stem.lower().endswith(ext):
            stem = stem[:-len(ext)]
    name = ui.ask('Name of the level in the game', re.sub(r'\s+', ' ', re.sub(r'[_\-]+', ' ', stem)).strip().title() or 'My Level')
    args = [path, '--name', name]
    if ui.yes('Add your own music to the level?', False):
        music = ui.pick_file('Choose the music (mp3, ogg, wav, flac, opus)', [('Audio', '*.mp3 *.ogg *.wav *.flac *.opus *.m4a'), ('All files', '*.*')])
        if music:
            args += ['--music', music]
    if ui.yes('Keep to the limits of the original game build (smaller levels, for a game without the extended tables)?', False):
        args += ['--original-limits']
    tmp = os.path.join(ctx.setup.mods_dir, '.level-work')
    shutil.rmtree(tmp, ignore_errors=True)
    ui.say('\nMaking the level. This can take a few minutes for a big map.\n')
    import obj2level
    old = sys.argv
    sys.argv = ['obj2level'] + args + ['--out', tmp]
    try:
        obj2level.main_run()
    except SystemExit as e:
        if e.code not in (None, 0):
            ui.fail(str(e.code))
            shutil.rmtree(tmp, ignore_errors=True)
            return
    finally:
        sys.argv = old
    merge(tmp, mod)
    shutil.rmtree(tmp, ignore_errors=True)
    ui.say(f'\nThe level "{name}" is in the mod "{mod.name}".  In the game: Arcade, choose the level after the game\'s own.')


def merge(tmp, mod):
    """Move what the converter made into the mod, keeping the levels the mod has: the `level` line is added to `levels.txt`, the set-up file gets the number in its name."""
    lines = open(os.path.join(tmp, 'levels.txt'), encoding='utf8').read().splitlines()
    out = []
    for line in lines:
        m = re.match(r'level\s+(\d+)', line)
        if m and ' setup setup.txt' in line:
            n = m.group(1)
            line = line.replace(' setup setup.txt', f' setup setup{n}.txt')
            os.replace(os.path.join(tmp, 'setup.txt'), os.path.join(tmp, f'setup{n}.txt'))
        out.append(line)
    for root, _, files in os.walk(tmp):
        for f in files:
            if f in ('mod.ini', 'levels.txt'):
                continue
            src = os.path.join(root, f)
            rel = os.path.relpath(src, tmp)
            dst = mod.path(rel)
            os.makedirs(os.path.dirname(dst), exist_ok=True)
            shutil.copyfile(src, dst)
    existing = mod.path('levels.txt')
    text = open(existing, encoding='utf8').read().rstrip('\n') + '\n' if os.path.exists(existing) else ''
    # (a level of the same number made before is replaced)
    for line in out:
        n = re.match(r'level\s+(\d+)', line).group(1)
        text = ''.join(l + '\n' for l in text.splitlines() if not re.match(rf'level\s+{n}\b', l))
        text += line + '\n'
    open(existing, 'w', encoding='utf8').write(text)
