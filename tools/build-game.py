"""Build the game and put it in the play folder.

    python tools/build-game.py             the build with everything for mods (extended tables, extensions)
    python tools/build-game.py --original  the build that behaves as the original game
    python tools/build-game.py --kit       also build modkit.exe (needs pyinstaller) and put it there too

Cargo builds in `build/cargo/` (set in `ts-rust/.cargo/config.toml`), so its many files stay out of the play folder, `ts-rust/target/release/`, which then holds only the program (`ts-port.exe`),
`modkit.exe` if you want it, the game's `PAK/` and `MUSIC/` (yours to put there), and what the game makes there (`ts-port.ini`, `saves/`, `mods/`).
"""
import os
import shutil
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
WS = os.path.join(ROOT, 'ts-rust')
PLAY = os.path.join(WS, 'target', 'release')
EXE = 'ts-port.exe' if os.name == 'nt' else 'ts-port'


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
    if '--kit' in sys.argv:
        if subprocess.run([sys.executable, os.path.join(ROOT, 'tools', 'build-modkit.py')]).returncode:
            sys.exit(1)
        kit = 'modkit.exe' if os.name == 'nt' else 'modkit'
        shutil.copyfile(os.path.join(ROOT, 'dist', kit), os.path.join(PLAY, kit))
        print(f'the mod kit: {os.path.join(PLAY, kit)}')
    for name in ('PAK', 'MUSIC'):
        if not os.path.isdir(os.path.join(PLAY, name)):
            print(f'note: the game needs its {name}/ folder in {PLAY}')


if __name__ == '__main__':
    main()
