"""Build a copy of the port that can capture real calls of decompiled functions (see `ts-rt/src/capture.rs`).

    python tools/capture-build.py [--out DIR] [--shadow]

Copies the Rust workspace to DIR (the temporary folder by default), puts a call to `ts_rt::capture::hook` at the start of every decompiled function of
`ts-game/src/game`, and builds the program there.  The source folders are not changed.  Copy `SLUS_200.90`, `PAK` and `MUSIC` next to the program, then, for
instance:

    TS_CAPTURE=obMergeChunkPolys:0+3 ts-rust --headless --level Village --frames 10
    TS_VERIFY_CAPTURE=obMergeChunkPolys-0.cap ts-verify --elf SLUS_200.90 -v

With `--shadow` the port is also built with live checking: run it with `TS_SHADOW=*` (or a list of names) and every function is run as its original machine code first, on the same
memory, and what each changes is compared (`TS_SHADOW_FIRST`, `TS_SHADOW_EVERY` choose which calls; see `ts-verify/src/shadow.rs`).

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


def function_addresses():
    """name -> address of the original function, from the generated table of the verifier."""
    table = os.path.join(WS, 'crates', 'ts-game', 'src', 'dec', 'verify_table.rs')
    found = {}
    for line in open(table, encoding='utf8'):
        m = re.match(r'\s*\((0x[0-9a-f]+), "(\w+)"', line)
        if m:
            found[m.group(2)] = m.group(1)
    return found


def hook_platform(native):
    """Put a call to `ts_rt::shadow::extern_hook` at the start of every routine of the SDK replacement that takes a slice of arguments or only 32-bit numbers, so that a live check can answer them
    with made-up values and log them (`TS_SHADOW_EXTERN`, see `ts-verify/src/shadow.rs`)."""
    count = 0
    for fn in os.listdir(native):
        if not fn.endswith('.rs'):
            continue
        path = os.path.join(native, fn)
        result = []
        for line in open(path, encoding='utf8').read().split(chr(10)):
            result.append(line)
            m = re.match(r'^pub fn (\w+)\((.*)\) -> i32 \{\s*$', line)
            if not m:
                continue
            params = [x.strip() for x in m.group(2).split(',') if x.strip()]
            if len(params) == 1 and params[0].endswith(': &[i64]'):
                args = '&' + params[0].split(':')[0].strip()
            elif all(re.match(r'^_?\w+: i32$', x) for x in params):
                args = '&[%s]' % ', '.join(x.split(':')[0].strip() + ' as i64' for x in params)
            else:
                continue
            result.append('    #[cfg(feature = "shadow")]' + chr(10) + '    if let Some(r) = ts_rt::shadow::extern_hook("%s", %s) { return r as i32; }' % (m.group(1), args))
            count += 1
        open(path, 'w', encoding='utf8').write(chr(10).join(result))
    # the generated stubs of routines that the port does not implement (the graphics chip, threads, interrupts) are logged too
    path = os.path.join(native, '..', 'stubs.rs')
    text = open(path, encoding='utf8').read()
    text, n = re.subn(r'pub fn (\w+)\(args: &\[i64\]\) -> i32 \{ ts_rt::stub_called', lambda m: 'pub fn %s(args: &[i64]) -> i32 { #[cfg(feature = "shadow")] if let Some(r) = ts_rt::shadow::extern_hook("%s", args) { return r as i32; } ts_rt::stub_called' % (m.group(1), m.group(1)), text)
    open(path, 'w', encoding='utf8').write(text)
    return count + n


def main():
    out = os.path.join(os.environ.get('TEMP', '/tmp'), 'ts-capture-build')
    if '--out' in sys.argv:
        out = sys.argv[sys.argv.index('--out') + 1]
    for sub in ('crates',):
        shutil.rmtree(os.path.join(out, sub), ignore_errors=True)
        shutil.copytree(os.path.join(WS, sub), os.path.join(out, sub))
    for f in ('Cargo.toml', 'Cargo.lock'):
        shutil.copy(os.path.join(WS, f), os.path.join(out, f))
    addresses = function_addresses()
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
                if m.group(1) in addresses and len(ints) <= 8 and len(floats) <= 8:
                    # for `--features shadow`: the original code is run on the same memory before the function and the memory they change is compared
                    result.append('    #[cfg(feature = "shadow")]\n    let _shadow = ts_rt::shadow::enter(%s, "%s", [%s], [%s]);' % (addresses[m.group(1)], m.group(1), ', '.join((ints + ['0'] * 4 + ['0x7000_0000'] * 4)[:8][:max(len(ints), 4)] + ['0x7000_0000'] * (8 - max(len(ints), 4))), ', '.join(floats + ['0.0'] * (8 - len(floats)))))
                count += 1
            open(path, 'w', encoding='utf8').write('\n'.join(result))
    print('hooked %d functions' % count)
    print('hooked %d platform routines' % hook_platform(os.path.join(out, 'crates', 'ts-sdk', 'src', 'native')))
    env = dict(os.environ, CARGO_TARGET_DIR=os.path.join(out, 'target'))
    features = ['--features', 'shadow'] if '--shadow' in sys.argv else []
    r = subprocess.run(['cargo', 'build', '--release', '-p', 'ts-port'] + features, cwd=out, env=env)
    print('built: ' + os.path.join(out, 'target', 'release', 'ts-rust.exe' if os.name == 'nt' else 'ts-rust'))
    if r.returncode == 0 and '--shadow' in sys.argv:
        # the verifier too, so that a run of whole frames on a snapshot can check the functions one by one (`TS_SHADOW=name,...`) and show which one differs
        r = subprocess.run(['cargo', 'build', '--release', '-p', 'ts-verify', '--features', 'shadow-hook'], cwd=out, env=env)
        print('built: ' + os.path.join(out, 'target', 'release', 'ts-verify.exe' if os.name == 'nt' else 'ts-verify'))
    sys.exit(r.returncode)


main()
