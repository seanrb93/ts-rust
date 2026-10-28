"""Where the game is (its PAK and MUSIC folders and the program), where the mods go, and which mod is being worked on: found by itself, asked once, remembered."""
import json
import os
import sys

from . import ui


def app_dir():
    """The folder of the program (or, when run from the sources, the repository's folder)."""
    if getattr(sys, 'frozen', False):
        return os.path.dirname(os.path.abspath(sys.executable))
    return os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


def _settings_paths():
    yield os.path.join(app_dir(), 'modkit.json')
    yield os.path.join(os.environ.get('APPDATA') or os.path.expanduser('~'), 'ts-modkit', 'modkit.json')


def load():
    for p in _settings_paths():
        if os.path.exists(p):
            try:
                return json.load(open(p, encoding='utf8'))
            except Exception:
                pass
    return {}


def save(data):
    for p in _settings_paths():
        try:
            os.makedirs(os.path.dirname(p), exist_ok=True)
            json.dump(data, open(p, 'w', encoding='utf8'), indent=1)
            return
        except OSError:
            continue


def has_game_files(folder):
    return bool(folder) and os.path.isfile(os.path.join(folder, 'PAK', 'OB.PAK'))


def port_exe(folder):
    for name in ('ts-port.exe', 'ts-port'):
        p = os.path.join(folder, name)
        if folder and os.path.isfile(p):
            return p
    return ''


def _candidates():
    base = app_dir()
    out = [base, os.path.dirname(base), os.getcwd(), os.path.join(base, 'ts-rust', 'target', 'release'), os.path.join(base, 'game')]
    seen = []
    for c in out:
        c = os.path.normpath(c)
        if c not in seen:
            seen.append(c)
    return seen


class Setup:
    """The game folder (the one with `PAK`, `MUSIC` and the program, which is also where `mods` is), asked for once."""

    def __init__(self):
        self.data = load()
        self.port_dir = self.data.get('port_dir', '')
        self.game_dir = self.data.get('game_dir', '')
        if not (self.port_dir and os.path.isdir(self.port_dir)):
            self.port_dir = next((c for c in _candidates() if port_exe(c)), '')
        if not (self.game_dir and has_game_files(self.game_dir)):
            self.game_dir = next((c for c in [self.port_dir] + _candidates() if has_game_files(c)), '')

    def ensure(self):
        """Make sure the game folder is known; asks when it cannot be found.  Returns False when there is none."""
        if self.port_dir and self.game_dir:
            self._remember()
            return True
        ui.heading('Where is the game?')
        ui.say('I need the folder that has the game program (ts-port.exe) and the game\'s PAK and MUSIC folders.')
        folder = ui.pick_file('Choose the game folder', folder=True)
        if not folder:
            return False
        self.port_dir = folder if port_exe(folder) or not self.port_dir else self.port_dir
        if not self.port_dir:
            self.port_dir = folder
        self.game_dir = folder if has_game_files(folder) else self.game_dir
        if not has_game_files(self.game_dir):
            ui.fail('That folder has no PAK/OB.PAK, so it is not the game folder.')
            self.game_dir = ''
            return False
        self._remember()
        return True

    def _remember(self):
        self.data['port_dir'] = self.port_dir
        self.data['game_dir'] = self.game_dir
        save(self.data)

    @property
    def mods_dir(self):
        d = os.path.join(self.port_dir, 'mods')
        os.makedirs(d, exist_ok=True)
        return d

    @property
    def current_mod(self):
        return self.data.get('current_mod', '')

    @current_mod.setter
    def current_mod(self, name):
        self.data['current_mod'] = name
        save(self.data)

    def mod_names(self):
        d = self.mods_dir
        return sorted(n for n in os.listdir(d) if os.path.isdir(os.path.join(d, n)) and not n.startswith(('.', '_')))
