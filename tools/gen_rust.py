"""Full pipeline: ELF -> decompiled Rust sources in ts-rust/crates/ts-game/src/gen.

usage: python gen_rust.py [--only name,name] [--dump name]
"""
import sys, os, re, struct, time, collections, traceback
from ps2elf import Elf
from decomp import lifter
from decomp.lifter import lift_function, Unsupported
from decomp.analysis import *
from decomp.webs import *
from decomp.structure import structure
from decomp.emit import Emitter, SymTab, rs_name
from decomp.externs import EXTERNS, DYNAMIC
from decomp.dump import dump
import patches

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..')
OUT = os.path.join(ROOT, 'ts-rust', 'crates', 'ts-game', 'src', 'dec')
SDK = os.path.join(ROOT, 'ts-rust', 'crates', 'ts-sdk', 'src')
MANUAL_DIR = os.path.join(ROOT, 'ts-rust', 'crates', 'ts-game', 'src', 'manual')
GAME_DIR = os.path.join(ROOT, 'ts-rust', 'crates', 'ts-game', 'src', 'game')   # the hand-decompiled game, see docs/decompilation.md

# functions hand-written in ts-game/src/manual (never generated); name -> (param regs, rets)
MANUAL_DYNAMIC = {'vafmt'}

# soft-float / libm externs that return 64-bit values or take 64-bit args
EXT_TYPES = {
    'fptodp': ([], 'i64'), 'dptofp': (['i64'], 'f32'), 'dpadd': (['i64', 'i64'], 'i64'),
    'dpsub': (['i64', 'i64'], 'i64'), 'dpmul': (['i64', 'i64'], 'i64'), 'dpdiv': (['i64', 'i64'], 'i64'),
    'dpcmp': (['i64', 'i64'], 'i32'), 'dptoli': (['i64'], 'i32'), 'dptoul': (['i64'], 'i32'),
    'fptoui': ([], 'i32'), '__divdi3': (['i64', 'i64'], 'i64'),
    'sqrt': (['i64'], 'i64'), 'atan': (['i64'], 'i64'), 'floor': (['i64'], 'i64'), 'atan2': (['i64', 'i64'], 'i64'),
    'pow': (['i64', 'i64'], 'i64'),
}


def load_all():
    e = Elf()
    ram = e.image()
    lifter.GP_VALUE = [s for s in e.symbols if s.name == '_gp'][0].addr
    fs = e.functions()
    split = [k for k, f in enumerate(fs) if f.name == 'text_data_split'][0]
    # unique rust names
    cnt = collections.Counter(rs_name(f.name) for f in fs)
    for f in fs:
        if cnt[rs_name(f.name)] > 1:
            f.name = f'{f.name}_{f.addr:x}'
    funcs, failed, sigs = {}, {}, {}
    for f in fs[:split]:
        if f.name in MANUAL_DYNAMIC:
            sigs[f.addr] = Sig(f.name, [], [], True, True, False)
            failed[f.addr] = (f.name, 'manual')
            continue
        try:
            fn = lift_function(f, ram, None)
            convert(fn)
            build_cfg(fn)
            funcs[f.addr] = fn
        except Unsupported as ex:
            failed[f.addr] = (f.name, str(ex))
    for f in fs[split:]:
        if f.name in EXTERNS:
            ni, nf, rf = EXTERNS[f.name]
            sigs[f.addr] = Sig(f.name, ARG_G[:ni] + ARG_F[:nf], [], True, False, rf)
        else:
            sigs[f.addr] = Sig(f.name, [], [], True, True, False)
    for f in fs[:split]:
        if f.addr not in funcs and f.addr not in sigs:
            sigs[f.addr] = Sig(f.name, [], [], True, True, False)
    return e, ram, fs, split, funcs, failed, sigs


def find_addr_taken(ram, fs, funcs):
    """Functions whose address is stored in data or materialised by lui/addiu|ori pairs in code."""
    from r5900 import decode
    starts = {f.addr for f in fs}
    at = set()
    for a in range(0x2eeff0, 0x3b0200, 4):
        w = struct.unpack_from('<I', ram, a)[0]
        if w in starts:
            at.add(w)
    for f in fs:
        regs = {}
        for a in range(f.addr, f.addr + f.size, 4):
            i = decode(a, struct.unpack_from('<I', ram, a)[0])
            o = i.op
            if o == 'lui':
                regs[i.rt] = (i.uimm << 16) & 0xFFFFFFFF
                continue
            if o in ('addiu', 'ori', 'daddiu') and i.rs in regs:
                v = (regs[i.rs] + (i.imm if o != 'ori' else i.uimm)) & 0xFFFFFFFF
                if v in starts:
                    at.add(v)
                regs[i.rt] = v
                continue
            # any other write kills the tracked value
            d = None
            if o in ('addu', 'daddu', 'or', 'subu', 'and', 'xor', 'nor', 'slt', 'sltu', 'sll', 'srl', 'sra', 'mflo',
                     'mfhi', 'movz', 'movn', 'dsll', 'dsll32', 'dsra32', 'dsrl32', 'dsubu'):
                d = i.rd
            elif o in ('lw', 'lb', 'lbu', 'lh', 'lhu', 'ld', 'lwu', 'slti', 'sltiu', 'andi', 'xori', 'lq', 'mfc1'):
                d = i.rt
            elif o in ('jal', 'jalr'):
                regs.clear()
            if d is not None:
                regs.pop(d, None)
    # Second, looser pass: a `lui` followed within a few instructions by an `addiu`/`ori` of the same register pair,
    # even when the compiler interleaved other code between them (the strict pass above gives up at any write).
    for f in fs:
        words = [decode(a, struct.unpack_from('<I', ram, a)[0]) for a in range(f.addr, f.addr + f.size, 4)]
        for k, i in enumerate(words):
            if i.op != 'lui':
                continue
            hi = (i.uimm << 16) & 0xFFFFFFFF
            for j in words[k + 1:k + 64]:
                if j.op in ('addiu', 'ori', 'daddiu') and j.rs == i.rt:
                    v = (hi + (j.imm if j.op != 'ori' else j.uimm)) & 0xFFFFFFFF
                    if v in starts:
                        at.add(v)
    return at


def main():
    args = sys.argv[1:]
    only = None
    dumpn = None
    if '--only' in args:
        only = set(args[args.index('--only') + 1].split(','))
    if '--dump' in args:
        dumpn = set(args[args.index('--dump') + 1].split(','))
    t0 = time.time()
    e, ram, fs, split, funcs, failed, sigs = load_all()
    print(f'lifted {len(funcs)} functions, {len(failed)} manual/unsupported  ({time.time() - t0:.1f}s)')
    at = find_addr_taken(ram, fs, funcs)
    for a in at:
        if a in sigs:
            sigs[a].addr_taken = True
    infer_signatures(funcs, sigs, at)
    print(f'signatures done ({time.time() - t0:.1f}s)')
    # passes
    for a, f in funcs.items():
        try:
            run_passes(f, sigs)
        except Exception as ex:
            print('PASS FAIL', f.name, repr(ex))
            traceback.print_exc()
            raise
    print(f'webs done ({time.time() - t0:.1f}s)')
    # types: two rounds so callers see callee return types
    rettypes = {a: {} for a in funcs}
    for a, sg in sigs.items():
        if sg.extern:
            t = EXT_TYPES.get(sg.name)
            rettypes[a] = {r: (t[1] if (t and r == 'r2') else ('f32' if r == 'f0' else 'i32')) for r in ('r2', 'r3', 'f0')}
            if t:
                sg.ptypes = t[0] + ['i32' if r[0] == 'r' else 'f32' for r in sg.params[len(t[0]):]]
    for rnd in range(3):
        for a, f in funcs.items():
            rettypes[a] = infer_types(f, sigs, rettypes) or rettypes[a]
    print(f'types done ({time.time() - t0:.1f}s)')
    symtab = SymTab(e)
    out_funcs = {}
    nerr = 0
    irr = 0
    for a, f in sorted(funcs.items()):
        if only and f.name not in only:
            continue
        try:
            body = structure(f)
            if f.irreducible:
                irr += 1
            em = Emitter(f, sigs, rettypes, symtab, None)
            code = em.emit_function(body, rettypes[a])
            out_funcs[a] = (f, code)
        except Exception as ex:
            nerr += 1
            print('EMIT FAIL', f.name, repr(ex))
            if nerr < 4:
                traceback.print_exc()
    print(f'emitted {len(out_funcs)} functions, {nerr} errors, {irr} irreducible ({time.time() - t0:.1f}s)')
    if dumpn:
        for a, (f, code) in out_funcs.items():
            if f.name in dumpn:
                print(code)
        return
    write_outputs(e, fs, split, funcs, failed, sigs, rettypes, out_funcs, at, symtab)
    patches.check()
    print(f'written ({time.time() - t0:.1f}s)')


def write_outputs(e, fs, split, funcs, failed, sigs, rettypes, out_funcs, at, symtab):
    os.makedirs(OUT, exist_ok=True)
    for fn in os.listdir(OUT):
        os.remove(os.path.join(OUT, fn))
    manual_names = set()
    if os.path.isdir(MANUAL_DIR):
        for fn in os.listdir(MANUAL_DIR):
            if fn.endswith('.rs'):
                src = open(os.path.join(MANUAL_DIR, fn), encoding='utf8').read()
                manual_names |= set(re.findall(r'pub fn (\w+)', src))
    # decompiled by hand: the functions at the top level of a file (methods of types are indented and are not the game's)
    for dp, _, fns in os.walk(GAME_DIR):
        for fn in fns:
            if fn.endswith('.rs'):
                src = open(os.path.join(dp, fn), encoding='utf8').read()
                manual_names |= set(re.findall(r'^pub fn (\w+)', src, re.M))
    items = sorted(out_funcs.items())
    per = 60
    mods = []
    for i in range(0, len(items), per):
        chunk = items[i:i + per]
        name = f'part_{i // per:03d}'
        mods.append(name)
        with open(os.path.join(OUT, name + '.rs'), 'w', encoding='utf8') as fh:
            fh.write('// @generated by tools/gen_rust.py - decompiled from SLUS_200.90.  Do not edit by hand;\n'
                     '// hand-written replacements live in ../manual and win over generated code of the same name.\n')
            fh.write('#![allow(unused_variables, unused_mut, unused_assignments, unreachable_code, non_snake_case, '
                     'dead_code, unused_parens, unused_braces, unused_labels, arithmetic_overflow, clippy::all)]\n')
            fh.write('use crate::*;\n\n')
            for a, (f, code) in chunk:
                if rs_name(f.name) in manual_names:
                    continue
                fh.write(f'// {f.name} @ {f.addr:#x}\n')
                fh.write(patches.apply(f.name, code) + '\n\n')
    with open(os.path.join(OUT, 'mod.rs'), 'w') as fh:
        fh.write('pub mod g;\npub mod fnptr;\npub use fnptr::*;\npub mod verify_table;\npub use verify_table::{call_any, FUNC_INFO};\n')
        for m in mods:
            fh.write(f'pub mod {m};\npub use {m}::*;\n')
    # ---- g.rs : data symbol address constants
    with open(os.path.join(OUT, 'g.rs'), 'w') as fh:
        fh.write('// @generated: addresses of named data objects in the game address space.\n')
        fh.write('#![allow(non_upper_case_globals, dead_code)]\n')
        for s in symtab.objs:
            fh.write(f'pub const {symtab.uname[s.addr]}: i32 = {s.addr:#x};\n')
        for a in sorted(at):
            if a in sigs:
                fh.write(f'pub const fn_{rs_name(sigs[a].name)}: i32 = {a:#x};\n')
    # ---- indirect call table
    with open(os.path.join(OUT, 'fnptr.rs'), 'w', encoding='utf8') as fh:
        fh.write('// @generated: dispatch for indirect (function pointer) calls.\n')
        fh.write('#![allow(unused_variables, dead_code)]\nuse crate::*;\n\n')
        fh.write('pub fn call_indirect(addr: i32, ints: [i32; 8], floats: [f32; 8]) -> (i32, f32) {\n    match addr {\n')
        for a in sorted(at):
            sg = sigs.get(a)
            if sg is None or sg.extern and sg.dynamic:
                continue
            if sg.extern and a not in funcs:
                pass
            args = []
            ni = nf = 0
            ptypes = getattr(sg, 'ptypes', None) or ['f32' if r[0] == 'f' else 'i32' for r in sg.params]
            for r, pt in zip(sg.params, ptypes):
                if r[0] == 'r':
                    v = f'ints[{ni}]'
                    ni += 1
                    if pt == 'i64':
                        v += ' as i64'
                else:
                    v = f'floats[{nf}]'
                    nf += 1
                args.append(v)
            call = f'{rs_name(sg.name)}({", ".join(args)})'
            rts = getattr(sg, 'rtypes', None) or []
            if not sg.rets:
                body = f'{{ {call}; (0, 0.0) }}'
            elif sg.rets == ['r2']:
                body = f'({call} as i32, 0.0)'
            elif sg.rets == ['f0']:
                body = f'(0, {call})'
            else:
                body = f'{{ let r = {call}; (r.0 as i32, 0.0) }}'
            fh.write(f'        {a:#x} => {body},\n')
        fh.write('        _ => panic!("indirect call to unknown address {:#x}", addr),\n    }\n}\n')
    # ---- call_any: every decompiled function by address (used by the differential verifier)
    with open(os.path.join(OUT, 'verify_table.rs'), 'w', encoding='utf8') as fh:
        fh.write('// @generated: address -> decompiled function dispatch for tools/verify.\n')
        fh.write('#![allow(unused_variables, dead_code)]\nuse crate::*;\n\n')
        fh.write('/// (address, name, int params, float params, return kind: 0 none / 1 int / 2 float)\n')
        fh.write('pub static FUNC_INFO: &[(u32, &str, u8, u8, u8)] = &[\n')
        for a, (f, code) in sorted(out_funcs.items()):
            sg = sigs[a]
            ni = sum(1 for r in sg.params if r[0] == 'r')
            nf = sum(1 for r in sg.params if r[0] == 'f')
            rk = 0 if not sg.rets else (2 if sg.rets[0] == 'f0' else 1)
            fh.write(f'    ({a:#x}, "{f.name}", {ni}, {nf}, {rk}),\n')
        fh.write('];\n\n')
        fh.write('/// Returns (int result, float result).  Panics for addresses without a decompiled function.\n')
        fh.write('pub fn call_any(addr: u32, ints: [i64; 8], floats: [f32; 8]) -> (i64, f32) {\n    match addr {\n')
        for a, (f, code) in sorted(out_funcs.items()):
            sg = sigs[a]
            args = []
            ni = nf = 0
            ptypes = getattr(sg, 'ptypes', None) or ['f32' if r[0] == 'f' else 'i32' for r in sg.params]
            for r, pt in zip(sg.params, ptypes):
                if r[0] == 'r':
                    args.append(f'ints[{ni}]' + (' as i32' if pt != 'i64' else ''))
                    ni += 1
                else:
                    args.append(f'floats[{nf}]')
                    nf += 1
            call = f'{rs_name(sg.name)}({", ".join(args)})'
            if not sg.rets:
                body = f'{{ {call}; (0, 0.0) }}'
            elif sg.rets == ['r2']:
                body = f'({call} as i64, 0.0)'
            elif sg.rets == ['f0']:
                body = f'(0, {call})'
            else:
                body = f'{{ let r = {call}; (r.0 as i64, 0.0) }}'
            fh.write(f'        {a:#x} => {body},\n')
        fh.write('        _ => panic!("no decompiled function at {:#x}", addr),\n    }\n}\n')
    # ---- extern stubs
    called = collections.OrderedDict()
    for f, code in out_funcs.values():
        for m in re.finditer(r'\b([A-Za-z_][A-Za-z0-9_]*)\(', code):
            called[m.group(1)] = 1
    native = set()
    nd = os.path.join(SDK, 'native')
    if os.path.isdir(nd):
        for fn in os.listdir(nd):
            if fn.endswith('.rs'):
                native |= set(re.findall(r'pub fn (\w+)', open(os.path.join(nd, fn), encoding='utf8').read()))
    with open(os.path.join(SDK, 'stubs.rs'), 'w', encoding='utf8') as fh:
        fh.write('// @generated: stubs for SDK / libc functions called by the decompiled game that have no native\n'
                 '// implementation yet.  Implement them in native/*.rs (they are then skipped here).\n')
        fh.write('#![allow(unused_variables, non_snake_case, dead_code)]\n\n')
        for a, sg in sorted(sigs.items()):
            if not sg.extern:
                continue
            n = rs_name(sg.name)
            if n not in called and not sg.addr_taken:
                continue
            if n in native:
                continue
            if sg.dynamic:
                fh.write(f'pub fn {n}(args: &[i64]) -> i32 {{ ts_rt::stub_called("{sg.name}"); 0 }}\n')
                continue
            pts = getattr(sg, 'ptypes', None) or ['f32' if r[0] == 'f' else 'i32' for r in sg.params]
            ps = ', '.join(f'{reg_name(r)}: {t}' for r, t in zip(sg.params, pts))
            rt = rettypes.get(a, {})
            if not sg.rets:
                rtype = ''
                rty = None
            else:
                rty = rt.get(sg.rets[0], 'i32' if sg.rets[0][0] == 'r' else 'f32')
                rtype = f' -> {rty}'
            fh.write(f'pub fn {n}({ps}){rtype} {{ ts_rt::stub_called("{sg.name}"); Default::default() }}\n')
    # ---- failures report
    with open(os.path.join(ROOT, 'ts-rust', 'MANUAL_FUNCTIONS.txt'), 'w') as fh:
        for a, (n, why) in sorted(failed.items()):
            fh.write(f'{a:#x} {n}: {why}\n')


def reg_name(r):
    from decomp.emit import reg_base
    return reg_base(r)


if __name__ == '__main__':
    main()
