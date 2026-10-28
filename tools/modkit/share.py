"""Share a mod (a zip file) and install one."""
import os
import zipfile

from . import ui
from .modfile import safe_name


def pack(ctx):
    mod = ctx.need_mod()
    if not mod:
        return
    out_dir = os.path.join(ctx.setup.port_dir, 'shared-mods')
    os.makedirs(out_dir, exist_ok=True)
    out = os.path.join(out_dir, mod.name + '.zip')
    count = 0
    with zipfile.ZipFile(out, 'w', zipfile.ZIP_DEFLATED) as z:
        for root, _, files in os.walk(mod.folder):
            for f in files:
                full = os.path.join(root, f)
                z.write(full, os.path.join(mod.name, os.path.relpath(full, mod.folder)))
                count += 1
    ui.say(f'\nMade {out}  ({count} files).')
    ui.say('Send that file to others: they choose "Install a mod from a zip" in this program, or unpack it into the mods folder of their game.')
    ui.say('Do not put the game\'s own files (pictures, sounds, models taken from it) in a mod you share.')


def install(ctx):
    path = ui.pick_file('Choose the mod\'s zip file', [('Zip', '*.zip'), ('All files', '*.*')])
    if not path:
        return
    with zipfile.ZipFile(path) as z:
        names = z.namelist()
        tops = {n.split('/')[0] for n in names if n.strip('/')}
        if len(tops) == 1:
            dest = ctx.setup.mods_dir
            name = next(iter(tops))
        else:
            name = safe_name(os.path.splitext(os.path.basename(path))[0])
            dest = os.path.join(ctx.setup.mods_dir, name)
        target = os.path.join(ctx.setup.mods_dir, name)
        if os.path.exists(target) and not ui.yes(f'The mod "{name}" is already installed.  Replace it?', False):
            return
        for n in names:
            if n.startswith('/') or '..' in n.split('/'):
                ui.fail('That zip has unsafe file names; not installed.')
                return
        z.extractall(dest)
    ui.say(f'\nInstalled the mod "{name}" (the game reads it when it starts).')
