"""Check against the machine code only the game functions that a change touched.

    python tools/verify-changed.py [<commit>]   (default: the changes not committed yet, against HEAD)

The functions are those of `ts-game/src/game/` whose lines the change touched, and the game functions that use a constant or a function whose definition
it touched elsewhere (in `dec/g.rs`, `manual/`, ...): a table moved to another address changes every function that names it.  They are passed to
`tools/verify-game.py --only`, which builds the checker and runs them.  `--list` prints the functions without checking them.
"""
import os
import re
import subprocess
import sys

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..')
GAME = os.path.join('ts-rust', 'crates', 'ts-game', 'src', 'game')
SRC = os.path.join('ts-rust', 'crates', 'ts-game', 'src')
FN = re.compile(r'^(?:pub(?:\(crate\))? )?fn (\w+)')
DEF = re.compile(r'^\s*(?:pub(?:\(crate\))? )?(?:const|fn|static) (\w+)')


def git(*args):
    return subprocess.run(['git', *args], cwd=ROOT, capture_output=True, text=True, encoding='utf8', errors='replace').stdout


def changed_lines(ref):
    """{file: set of line numbers of the new version that the change touched (for a removed line, the line where it was)}"""
    out = {}
    current = None
    for line in git('diff', '-U0', ref, '--', 'ts-rust/crates/ts-game/src').splitlines():
        if line.startswith('+++ '):
            path = line[4:]
            current = None if path == '/dev/null' else path[2:]
            if current:
                out.setdefault(current, set())
        elif line.startswith('@@') and current:
            m = re.search(r'\+(\d+)(?:,(\d+))?', line)
            start, count = int(m.group(1)), int(m.group(2) or 1)
            out[current].update(range(start, start + max(count, 1)))
    return out


def functions_at(path, lines):
    """The functions of a file that hold the given lines: (the public function a line is in or under, ...), and the names it defines on those lines."""
    text = open(os.path.join(ROOT, path), encoding='utf8').read().splitlines()
    owner = None
    owners = set()
    defined = set()
    for no, line in enumerate(text, 1):
        m = FN.match(line)
        if m:
            owner = m.group(1)
        if no in lines:
            if owner:
                owners.add(owner)
            d = DEF.match(line)
            if d:
                defined.add(d.group(1))
    return owners, defined


def game_functions():
    """{name: (file, the text of its body)} of every public function of the game's files."""
    out = {}
    base = os.path.join(ROOT, GAME)
    for fn in sorted(os.listdir(base)):
        if not fn.endswith('.rs'):
            continue
        name = None
        body = []
        for line in open(os.path.join(base, fn), encoding='utf8'):
            m = re.match(r'^pub fn (\w+)', line)
            if m:
                if name:
                    out[name] = (fn, ''.join(body))
                name, body = m.group(1), []
            body.append(line)
        if name:
            out[name] = (fn, ''.join(body))
    return out


def main():
    args = [a for a in sys.argv[1:] if not a.startswith('--')]
    ref = args[0] if args else 'HEAD'
    lines = changed_lines(ref)
    game = game_functions()
    wanted = set()
    names = set()
    for path, ls in lines.items():
        if not os.path.exists(os.path.join(ROOT, path)):
            continue
        owners, defined = functions_at(path, ls)
        if os.path.normpath(path).startswith(os.path.normpath(GAME)):
            # (a private helper of a game file: the public functions of the file that use it)
            wanted |= {o for o in owners if o in game}
            names |= {o for o in owners if o not in game} | defined
        else:
            names |= owners | defined
    # the game functions that name a constant or function whose definition changed
    for n in sorted(names):
        pat = re.compile(r'\b' + re.escape(n) + r'\b')
        wanted |= {g for g, (_, body) in game.items() if g != n and pat.search(body)}
    wanted = sorted(wanted)
    print(f'{len(wanted)} game functions touched by the change against {ref}' + (f' (through {", ".join(sorted(names))})' if names else ''))
    if '--list' in sys.argv or not wanted:
        print(' '.join(wanted))
        return
    r = subprocess.run([sys.executable, os.path.join(ROOT, 'tools', 'verify-game.py'), '--only', *wanted], cwd=ROOT)
    sys.exit(r.returncode)


if __name__ == '__main__':
    main()
