"""Characters: make one from a 3D model, or get a game character's model to start from."""
import hashlib
import os
import re

from . import ui
from .modfile import pick_number


def show_list(game):
    ui.heading('The characters of the game')
    rows = game.characters()
    for i in range(0, len(rows), 2):
        left = rows[i]
        right = rows[i + 1] if i + 1 < len(rows) else None
        text = f'{left["number"]:>3}  {left["name"]:<26}'
        if right:
            text += f'{right["number"]:>3}  {right["name"]}'
        ui.say(text)
    ui.pause()


def pick_character(game, prompt, default=0):
    return ui.ask_number(prompt + ' (number from 0 to 79)', default=default, kind=int, low=0, high=79)


def menu(ctx):
    game = ctx.game()
    if not game:
        return
    while True:
        what = ui.choose('Characters', [
            ('Make a character from a 3D model', 'make'),
            ('Get a game character as a 3D model to start from', 'export'),
            ('See the game\'s characters', 'list'),
            ('Remove a character of mine', 'remove'),
        ])
        if what is None:
            return
        if what == 'list':
            show_list(game)
        elif what == 'export':
            export(ctx, game)
        elif what == 'make':
            make(ctx, game)
        elif what == 'remove':
            remove(ctx)


def export(ctx, game):
    import obdump
    import pakls
    ui.heading('Get a character as a 3D model')
    show = ui.yes('Show the list of characters first?', False)
    if show:
        show_list(game)
    n = pick_character(game, 'Which character')
    d, es = pakls.entries(os.path.join(ctx.setup.game_dir, 'PAK', 'OB.PAK'))
    name = f'ob/chrs/chr{n + 1:02d}.raw'
    hit = [(o, s) for nm, o, s in es if nm == name]
    if not hit:
        ui.fail(f'{name} is not in the game.')
        return
    folder = os.path.join(ctx.setup.mods_dir, '_models')
    os.makedirs(folder, exist_ok=True)
    out = os.path.join(folder, f'character-{n}.obj')
    obdump.write_obj(d[hit[0][0]:hit[0][0] + hit[0][1]], out)
    ui.say(f'\nWritten to {out}')
    ui.say('Open it in Blender (or any 3D program), change it and keep the figure standing in the same pose, about 1.95 high, then make a character from it here.')


def make(ctx, game):
    import obchar
    mod = ctx.need_mod()
    if not mod:
        return
    ui.heading('Make a character from a 3D model')
    ui.say('The model is an OBJ file of a whole figure standing as the game\'s characters stand (arms down, feet on the ground).')
    ui.say('Textures come from the materials of the OBJ (any picture size).')
    mesh = ui.pick_file('Choose the model (.obj)', [('OBJ model', '*.obj'), ('All files', '*.*')])
    if not mesh:
        return
    name = ui.ask('Name of the character in the game', os.path.splitext(os.path.basename(mesh))[0].replace('_', ' ').replace('-', ' ').title()[:30])
    base = pick_character(game, 'Start as a copy of which game character? (its stats, animations, front end pose)', default=0)
    skeleton = 'female' if ui.yes('Does the model have a female build (the female skeleton)?', False) else 'male'
    scale = ui.ask_number('Size (1 = as the game\'s characters, 1.2 = taller)', default=1.0, low=0.5, high=3.0)
    voice = ui.ask_number('Voice of which game character?', default=base, kind=int, low=0, high=79)
    slug = re.sub(r'[^a-z0-9]+', '', name.lower())[:14] or 'char'
    tag = hashlib.md5((mod.name + '/' + name).encode()).hexdigest()
    number = pick_number(ctx.setup.mods_dir, mod, 'characters.txt', 'character', 80, 127, name)
    model_file = f'ob/chrs/c_{slug}{tag[:4]}.raw'
    tex_base = 9000 + (int(tag[4:8], 16) % 1500) * 20
    try:
        entry = obchar.make_character(ctx.setup.game_dir, mesh, mod.folder, name, base, number, scale, 1.95, skeleton, tex_base, model_file, voice, write_entry=False)
    except SystemExit as e:
        ui.fail(str(e))
        return
    cfile = mod.block_file('characters.txt')
    header, *body = entry.rstrip('\n').split('\n')
    # (a character of the same name made before is replaced)
    for h, b in list(cfile.blocks):
        if any(l.strip() == f'name "{name}"' for l in b):
            cfile.remove(h)
    cfile.set(header, body)
    cfile.save()
    ui.say(f'\nThe character "{name}" is in the mod "{mod.name}".')
    ui.say('In the game it is on the character select page after the game\'s characters (press left from the first one), and in the bot lists.')


def remove(ctx):
    mod = ctx.need_mod()
    if not mod:
        return
    cfile = mod.block_file('characters.txt')
    items = []
    for h, b in cfile.blocks:
        label = next((l.strip()[5:].strip('" ') for l in b if l.strip().startswith('name ')), h)
        items.append((label, h))
    if not items:
        ui.say('This mod has no characters.')
        return
    h = ui.choose('Remove which?', items)
    if h:
        cfile.remove(h)
        cfile.save()
        ui.say('Removed (its model file stays in the mod folder).')
