"""Build ts-verify and prove every hand-decompiled function (ts-game/src/game) equal to the machine code.

    python tools/verify_game.py [--n 500] [--elf path/to/SLUS_200.90] [--regen]

`--regen` runs tools/gen_rust.py first (it needs `Game Files/SLUS_200.90`, which is copied from --elf for the run and removed after).
"""
import os
import re
import shutil
import subprocess
import sys

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..')
WS = os.path.join(ROOT, 'ts-rust')


def arg(name, default):
    return sys.argv[sys.argv.index(name) + 1] if name in sys.argv else default


def main():
    elf = os.path.abspath(arg('--elf', os.path.join(WS, 'target', 'release', 'SLUS_200.90')))
    n = arg('--n', '500')
    if '--regen' in sys.argv:
        gf = os.path.join(ROOT, 'Game Files')
        os.makedirs(gf, exist_ok=True)
        shutil.copy(elf, os.path.join(gf, 'SLUS_200.90'))
        try:
            subprocess.run([sys.executable, os.path.join(ROOT, 'tools', 'gen_rust.py')], check=True)
        finally:
            shutil.rmtree(gf, ignore_errors=True)
    if subprocess.run(['cargo', 'build', '--release', '-p', 'ts-verify', '-q'], cwd=WS, stderr=subprocess.DEVNULL).returncode:
        sys.exit(1)
    names = []
    for dp, _, fns in os.walk(os.path.join(WS, 'crates', 'ts-game', 'src', 'game')):
        for fn in fns:
            if fn.endswith('.rs'):
                names += re.findall(r'^pub fn (\w+)', open(os.path.join(dp, fn), encoding='utf8').read(), re.M)
    exe = os.path.join(WS, 'target', 'release', 'ts-verify.exe' if os.name == 'nt' else 'ts-verify')
    r = subprocess.run([exe, '--elf', elf, '--n', n] + sorted(names), capture_output=True, text=True)
    lines = (r.stdout + r.stderr).strip().splitlines()
    print('\n'.join(l for l in lines if l.startswith('FAIL') or l.startswith('==')))
    print(f'{len(names)} functions')
    sys.exit(0 if ' fail 0,' in (lines[-1] if lines else '') else 1)


if __name__ == '__main__':
    main()
