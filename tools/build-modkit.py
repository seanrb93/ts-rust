"""Build `modkit.exe`: one program with all the mod making tools, which needs no Python on the machine it runs on.

    python tools/build-modkit.py            makes tools/modkit.exe (not in the play folder)

Needs `pip install pyinstaller numpy pillow imageio-ffmpeg` once.  Put modkit.exe in the game's folder (next to ts-rust.exe), or anywhere: it asks where the game is the first time.
"""
import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
TOOLS = os.path.join(ROOT, 'tools')


def main():
    cmd = [sys.executable, '-m', 'PyInstaller', '--onefile', '--console', '--noconfirm', '--clean', '--name', 'modkit',
           '--distpath', TOOLS, '--workpath', os.path.join(ROOT, 'build', 'modkit'), '--specpath', os.path.join(ROOT, 'build'),
           '--paths', TOOLS, '--collect-submodules', 'lvimport', '--collect-submodules', 'modkit', '--collect-all', 'imageio_ffmpeg',
           '--hidden-import', 'tkinter', '--add-data', os.path.join(ROOT, 'ts-rust', 'crates', 'ts-rt', 'data', 'game-data.bin') + os.pathsep + '.', os.path.join(TOOLS, 'modkit-entry.py')]
    print(' '.join(cmd))
    sys.exit(subprocess.run(cmd, cwd=ROOT).returncode)


if __name__ == '__main__':
    main()
