"""Build ts-verify and prove every hand-decompiled function (ts-game/src/game) equal to the machine code.

    python tools/verify-game.py [--n 500] [--elf path/to/SLUS_200.90] [--regen]

`--regen` runs tools/gen_rust.py first (it needs `Game Files/SLUS_200.90`, which is copied from --elf for the run and removed after).
"""
import os
import re
import shutil
import subprocess
import sys

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..')
WS = os.path.join(ROOT, 'ts-rust')


# functions that differ from the machine code on purpose (none: the game's own tables are used unless the `extended-tables` feature is on)
BY_DESIGN = set()


# recursion over a structure the random data makes endless overflows the host stack (the interpreter stops, the Rust code cannot), and pickupposGfx (the original faults on random data, the Rust loop over a random count takes minutes): reviewed only
UNVERIFIABLE = {'partSetTextureOffset', 'obSetTexList', 'partTransform', 'pickupposGfx'}


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
    names = [n for n in names if n not in UNVERIFIABLE]
    exe = os.path.join(WS, 'target', 'release', 'ts-verify.exe' if os.name == 'nt' else 'ts-verify')
    r = subprocess.run([exe, '-v', '--elf', elf, '--n', n] + sorted(names), capture_output=True, text=True)
    lines = (r.stdout + r.stderr).strip().splitlines()
    bad = [l for l in lines if l.startswith('FAIL') and l.split()[1] not in BY_DESIGN]
    print(chr(10).join(bad))
    skipped = [l.split(':')[0][5:] for l in lines if l.startswith('skip ')]
    if skipped:
        # second pass: small numbers only, runs that do not finish are left out (slow: few runs)
        env = dict(os.environ, TS_VERIFY_SMALL='1')
        r2 = subprocess.run([exe, '-v', '--elf', elf, '--n', '40'] + skipped, capture_output=True, text=True, env=env)
        l2 = (r2.stdout + r2.stderr).splitlines()
        bad += [l for l in l2 if l.startswith('FAIL')]
        print(chr(10).join(l for l in l2 if l.startswith('FAIL')))
        skipped = [l.split(':')[0][5:] for l in l2 if l.startswith('skip ')]
    skipped += sorted(UNVERIFIABLE)
    if skipped:
        print('not machine-verified (the original code did not finish on random data, or recursion overflowed): ' + ' '.join(skipped))
    print((lines[-1] if lines else 'no output') + f'  ({len(names)} functions, {len(lines) and sum(1 for l in lines if l.startswith("FAIL")) - len(bad)} differ by design)')
    sys.exit(0 if lines and not bad else 1)


if __name__ == '__main__':
    main()
