"""The menu of the program, and what can be typed after its name (`modkit weapons`, `modkit character`, ...)."""
import os
import sys

from . import VERSION, assets, characters, config, levels, modes, play, share, ui, weapons
from .modfile import Mod, safe_name


class Context:
    def __init__(self):
        self.setup = config.Setup()
        self._game = None

    # -- the game
    def game(self):
        if self._game:
            return self._game
        from .gamedata import GameData
        self._game = GameData()
        return self._game

    def exe(self):
        exe = config.port_exe(self.setup.port_dir)
        if not exe:
            ui.fail(f'ts-rust.exe is not in {self.setup.port_dir}.')
        return exe

    # -- the mod being made
    def current(self):
        name = self.setup.current_mod
        if name and os.path.isdir(os.path.join(self.setup.mods_dir, name)):
            return Mod(os.path.join(self.setup.mods_dir, name))
        return None

    def need_mod(self):
        mod = self.current()
        if mod:
            return mod
        ui.say('\nYou need a mod to put your work in.')
        return new_mod(self)


def new_mod(ctx):
    name = safe_name(ui.ask('Name for the new mod', 'My-Mod'))
    mod = Mod(os.path.join(ctx.setup.mods_dir, name))
    mod.create()
    ctx.setup.current_mod = name
    ui.say(f'The mod "{name}" is made and is the one you are working on.')
    return mod


def mods_menu(ctx):
    while True:
        names = ctx.setup.mod_names()
        cur = ctx.setup.current_mod
        options = [('Make a new mod', '__new')]
        for n in names:
            m = Mod(os.path.join(ctx.setup.mods_dir, n))
            options.append((f'{n}{"  <- working on this" if n == cur else ""}  [{"on" if m.is_on() else "off"}]', n))
        pick = ui.choose('Mods in the game', options)
        if pick is None:
            return
        if pick == '__new':
            new_mod(ctx)
            continue
        m = Mod(os.path.join(ctx.setup.mods_dir, pick))
        act = ui.choose(f'The mod "{pick}"', [('Work on this mod', 'work'), ('Turn it ' + ('off' if m.is_on() else 'on') + ' in the game', 'toggle'), ('Open its folder', 'open')])
        if act == 'work':
            ctx.setup.current_mod = pick
        elif act == 'toggle':
            m.set_on(not m.is_on())
        elif act == 'open':
            _open_folder(m.folder)


def _open_folder(path):
    try:
        if sys.platform.startswith('win'):
            os.startfile(path)
        else:
            import subprocess
            subprocess.Popen(['xdg-open', path])
    except Exception:
        ui.say(path)


def help_text():
    ui.heading('Help')
    ui.say('''A mod is a folder of changes to the game.  This program makes the folder for you; the game reads it when it starts.

  1. Choose or make a mod (the first thing you do makes one).
  2. Make things: levels from maps and 3D models, characters from 3D models, new or changed weapons,
     ways to play the game modes, your own sounds, music and pictures.
  3. Try it in the game, then share the mod as a zip file.

Everything you make goes into the mod you are working on (shown at the top of the menu).  You can make as many mods as you like and turn them on and off.
Adding weapons, characters and mode names needs the game build with the extended tables.''')
    ui.pause()


def main(argv=None):
    argv = list(sys.argv[1:] if argv is None else argv)
    ctx = Context()
    if argv and argv[0] in ('-h', '--help', 'help'):
        print(__doc__)
        print('Start it with no words to get the menu.  Or: weapons, character, level, modes, sounds, play, share, mods.')
        return
    ui.say(f'TimeSplitters mod kit {VERSION}')
    try:
        found = ctx.setup.ensure()
    except (KeyboardInterrupt, EOFError):
        return
    if not found:
        ui.say('\nWithout the game folder I cannot go on.')
        ui.pause()
        return
    shortcuts = {'weapons': weapons.menu, 'weapon': weapons.menu, 'character': characters.menu, 'characters': characters.menu, 'level': levels.make, 'levels': levels.make,
                 'modes': modes.menu, 'mode': modes.menu, 'sounds': assets.menu, 'play': play.menu, 'share': share.pack, 'mods': mods_menu}
    try:
        if argv and argv[0] in shortcuts:
            shortcuts[argv[0]](ctx)
            return
        loop(ctx)
    except (KeyboardInterrupt, EOFError):
        print()
    except SystemExit:
        raise
    except Exception:
        import traceback
        text = traceback.format_exc()
        log = os.path.join(config.app_dir(), 'modkit-error.txt')
        try:
            open(log, 'w').write(text)
        except OSError:
            log = '(could not be written)'
        ui.fail('Something went wrong.  The details are in ' + log + ' - please send that file to whoever made this program.')
        print(text.strip().splitlines()[-1])
        ui.pause()


def loop(ctx):
    while True:
        cur = ctx.setup.current_mod
        ui.say('\n' + '=' * 60)
        ui.say(f'Working on the mod: {cur if cur else "(none yet)"}')
        pick = ui.choose('What do you want to make?', [
            ('A level (from a map or a 3D model)', levels.make),
            ('A character (from a 3D model)', characters.menu),
            ('Weapons (change or add)', weapons.menu),
            ('Game modes (ways to play)', modes.menu),
            ('Sounds, music and pictures', assets.menu),
            ('Try it in the game', play.menu),
            ('Share this mod as a zip / install a mod from a zip', share_menu),
            ('Choose, make or switch off mods', mods_menu),
            ('Help', lambda c: help_text()),
        ], back='Quit', default=1)
        if pick is None:
            return
        pick(ctx)


def share_menu(ctx):
    what = ui.choose('Share', [('Make a zip file of this mod to send to others', 'pack'), ('Install a mod from a zip file', 'install')])
    if what == 'pack':
        share.pack(ctx)
    elif what == 'install':
        share.install(ctx)
