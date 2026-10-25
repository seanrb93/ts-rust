"""Build a copy of the port that can capture real calls of decompiled functions (see `ts-rt/src/capture.rs`).

    python tools/capture-build.py [--out DIR]

Copies the Rust workspace to DIR (the temporary folder by default), puts a call to `ts_rt::capture::hook` at the start of every decompiled function of
`ts-game/src/game`, and builds `ts-port` there.  The source folders are not changed.  Copy `SLUS_200.90`, `PAK` and `MUSIC` next to the program, then, for
instance:

    TS_CAPTURE=obMergeChunkPolys:0+3 ts-port --headless --level Village --frames 10
    TS_VERIFY_CAPTURE=obMergeChunkPolys-0.cap ts-verify --elf SLUS_200.90 -v

The first call of a function is number 0.  The captured file holds the whole memory (the pages that are not zero) and the arguments, and `ts-verify` runs the original
code and the port on it.
"""
import os
import re
import shutil
import subprocess
import sys

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..')
WS = os.path.join(ROOT, 'ts-rust')


def main():
    out = os.path.join(os.environ.get('TEMP', '/tmp'), 'ts-capture-build')
    if '--out' in sys.argv:
        out = sys.argv[sys.argv.index('--out') + 1]
    for sub in ('crates',):
        shutil.rmtree(os.path.join(out, sub), ignore_errors=True)
        shutil.copytree(os.path.join(WS, sub), os.path.join(out, sub))
    for f in ('Cargo.toml', 'Cargo.lock'):
        shutil.copy(os.path.join(WS, f), os.path.join(out, f))
    count = 0
    game = os.path.join(out, 'crates', 'ts-game', 'src', 'game')
    for dp, _, fns in os.walk(game):
        for fn in fns:
            if not fn.endswith('.rs'):
                continue
            path = os.path.join(dp, fn)
            lines = open(path, encoding='utf8').read().split('\n')
            result = []
            for line in lines:
                result.append(line)
                m = re.match(r'^pub fn (\w+)\((.*)\)( -> [^{]+)? \{\s*$', line)
                if not m or '&' in m.group(2) or '<' in m.group(2):
                    continue
                ints, floats = [], []
                for p in [x for x in m.group(2).split(',') if x.strip()]:
                    name, _, ty = p.partition(':')
                    name = name.replace('mut ', '').strip()
                    ty = ty.strip()
                    if ty == 'f32':
                        floats.append(name)
                    elif ty in ('i32', 'i64', 'u32'):
                        ints.append(name + ' as i64')
                    else:
                        ints = None
                        break
                if ints is None:
                    continue
                result.append('    if ts_rt::capture::enabled() { ts_rt::capture::hook("%s", &[%s], &[%s]); }' % (m.group(1), ', '.join(ints), ', '.join(floats)))
                count += 1
            open(path, 'w', encoding='utf8').write('\n'.join(result))
    print('hooked %d functions' % count)
    env = dict(os.environ, CARGO_TARGET_DIR=os.path.join(out, 'target'))
    r = subprocess.run(['cargo', 'build', '--release', '-p', 'ts-port'], cwd=out, env=env)
    print('built: ' + os.path.join(out, 'target', 'release', 'ts-port.exe' if os.name == 'nt' else 'ts-port'))
    sys.exit(r.returncode)


main()
