"""Try the mod in the game."""
import os
import re
import subprocess

from . import ui
from .modfile import BlockFile, Mod

GAME_CHARS = 80
GAME_GUNS = 30
GAME_COMBOS = 49


def enabled_mods(ctx):
    out = []
    for name in ctx.setup.mod_names():
        m = Mod(os.path.join(ctx.setup.mods_dir, name))
        if m.is_on():
            out.append(m)
    return out


def runtime_numbers(ctx, filename, keyword, game_count):
    """What the game will call the added things of the enabled mods: the numbers from the game's count on, in the order of the numbers each mod gave.  {(mod, given number): runtime number}"""
    items = []
    for m in enabled_mods(ctx):
        for n in BlockFile(m.path(filename)).numbers(keyword):
            items.append((n, m.name))
    items.sort()
    return {(mod, n): game_count + i for i, (n, mod) in enumerate(items)}, len(items)


def launch(ctx, args, env=None):
    exe = ctx.exe()
    if not exe:
        return
    full = dict(os.environ)
    full.update(env or {})
    cmd = [exe, '--game-dir', ctx.setup.game_dir] + args
    ui.say('\nStarting the game...  (close the game window to come back here)')
    try:
        subprocess.run(cmd, cwd=os.path.dirname(exe), env=full)
    except OSError as e:
        ui.fail(f'The game could not be started: {e}')


def menu(ctx):
    mod = ctx.current()
    while True:
        what = ui.choose('Try it in the game', [
            ('Start the game', 'start'),
            ('Look at one of my characters (walking in the Prison level)', 'char'),
            ('Try one of my weapons against bots', 'weapon'),
        ])
        if what is None:
            return
        if what == 'start':
            launch(ctx, [])
        elif what == 'char':
            look_at_character(ctx, mod)
        elif what == 'weapon':
            try_weapon(ctx, mod)


def look_at_character(ctx, mod):
    if not mod:
        ui.say('Choose a mod first.')
        return
    cfile = mod.block_file('characters.txt')
    mine = []
    for h, body in cfile.blocks:
        m = re.match(r'character\s+(\d+)', h, re.I)
        if m:
            label = next((l.strip()[5:].strip('" ') for l in body if l.strip().startswith('name ')), h)
            mine.append((label, int(m.group(1))))
    if not mine:
        ui.say('This mod has no characters yet.')
        return
    numbers, _ = runtime_numbers(ctx, 'characters.txt', 'character', GAME_CHARS)
    pick = ui.choose('Which character?', mine)
    if pick is None:
        return
    num = numbers.get((mod.name, pick))
    ui.say('You walk with the stick; the camera is behind you.  Close the window when you have seen enough.')
    launch(ctx, ['--level', 'Prison', '--character', str(num), '--cam', '4,1.2'])


def try_weapon(ctx, mod):
    if not mod:
        ui.say('Choose a mod first.')
        return
    wfile = mod.block_file('weapons.txt')
    mine = []
    for h, body in wfile.blocks:
        m = re.match(r'weapon\s+(\d+)', h, re.I)
        if m:
            label = next((l.strip()[5:].strip('" ') for l in body if l.strip().startswith('name ')), h)
            mine.append((label, int(m.group(1))))
    if not mine:
        ui.say('This mod has no new weapons yet (changes to the game\'s weapons show in any game).')
        return
    numbers, count = runtime_numbers(ctx, 'weapons.txt', 'weapon', GAME_GUNS)
    pick = ui.choose('Which weapon?', mine)
    if pick is None:
        return
    gun = numbers[(mod.name, pick)]
    j = gun - GAME_GUNS
    single = GAME_COMBOS + count + j
    ui.say('You have the weapon in your hands in the Prison level with some bots near you.  Fire with the left mouse button.')
    launch(ctx, ['--level', 'Prison', '--bots', '3,4,5', '--spawn', '12,0,-26'], env={'TS_GIVE': f'40:{single}', 'TS_EQUIP': '1'})
