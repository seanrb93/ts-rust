"""Build the game and put it in the play folder.

    python tools/build-game.py             the build with everything for mods (extended tables, extensions)
    python tools/build-game.py --original  the build that behaves as the original game
    python tools/build-game.py --kit       also build modkit.exe (needs pyinstaller) in the root of the repository (it is not put in the play folder)

Cargo builds in `build/cargo/` (set in `ts-rust/.cargo/config.toml`), so its many files stay out of the play folder, `ts-rust/target/release/`, which then holds only the program (`ts-rust.exe`),
the game's `PAK/` and `MUSIC/` (yours to put there), and what the game makes there (`ts-rust.ini`, `saves/`, `mods/`).
"""
import os
import shutil
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
WS = os.path.join(ROOT, 'ts-rust')
PLAY = os.path.join(WS, 'target', 'release')
EXE = 'ts-rust.exe' if os.name == 'nt' else 'ts-rust'


def main():
    cmd = ['cargo', 'build', '--release', '-p', 'ts-port']
    if '--original' not in sys.argv:
        cmd += ['--features', 'extended-tables,extensions']
    if subprocess.run(cmd, cwd=WS).returncode:
        sys.exit(1)
    os.makedirs(PLAY, exist_ok=True)
    built = os.path.join(ROOT, 'build', 'cargo', 'release', EXE)
    shutil.copyfile(built, os.path.join(PLAY, EXE))
    print(f'the game: {os.path.join(PLAY, EXE)}')
    # (the program was called ts-port before: an old copy would only confuse)
    for old in ('ts-port.exe', 'ts-port'):
        if os.path.exists(os.path.join(PLAY, old)):
            os.remove(os.path.join(PLAY, old))
    if '--kit' in sys.argv:
        if subprocess.run([sys.executable, os.path.join(ROOT, 'tools', 'build-modkit.py')]).returncode:
            sys.exit(1)
        kit = 'modkit.exe' if os.name == 'nt' else 'modkit'
        print(f'the mod kit: {os.path.join(ROOT, kit)}')
    for name in ('PAK', 'MUSIC'):
        if not os.path.isdir(os.path.join(PLAY, name)):
            print(f'note: the game needs its {name}/ folder in {PLAY}')


if __name__ == '__main__':
    main()
