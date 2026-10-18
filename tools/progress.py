"""How much of the game is decompiled by hand.

    python tools/progress.py [-v]

Counts the functions of ts-game/src/game (readable, hand-decompiled) against all functions of the game (those still machine-translated in
ts-game/src/dec, the hand-written ones in ts-game/src/manual, and those in game).  `dec/` is made by tools/gen_rust.py from your own copy of
the game; without it only the hand-decompiled functions are counted.
"""
import os
import re
import sys

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..')
SRC = os.path.join(ROOT, 'ts-rust', 'crates', 'ts-game', 'src')


def names(folder, pattern):
    found = {}
    for dp, _, fns in os.walk(os.path.join(SRC, folder)):
        for fn in sorted(fns):
            if not fn.endswith('.rs') or fn in ('mod.rs', 'g.rs', 'fnptr.rs', 'verify_table.rs'):
                continue
            text = open(os.path.join(dp, fn), encoding='utf8').read()
            for n in re.findall(pattern, text, re.M):
                found[n] = os.path.relpath(os.path.join(dp, fn), SRC).replace(os.sep, '/')
    return found


def main():
    done = names('game', r'^pub fn (\w+)')
    translated = names('dec', r'^pub fn (\w+)')
    manual = names('manual', r'^pub fn (\w+)')
    total = len(translated) + len(done)
    print(f'hand-decompiled: {len(done)} of {total or "?"} functions' + (f' ({100 * len(done) / total:.1f}%)' if total else ''))
    print(f'still machine-translated: {len(translated)}   port code in manual/: {len(manual)}')
    by = {}
    for n, f in done.items():
        by.setdefault(f, []).append(n)
    for f in sorted(by):
        print(f'  {f}: {len(by[f])}' + (('  ' + ' '.join(sorted(by[f]))) if '-v' in sys.argv else ''))


if __name__ == '__main__':
    main()
