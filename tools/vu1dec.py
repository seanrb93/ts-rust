"""VU1 microcode -> Rust.

Translates every instruction pair of the VU1 program image extracted from the executable into straight Rust
(one state machine over basic blocks) operating on ts-gfx's VU register file / data memory.  It is a decompiler,
not an interpreter: the output contains no instruction decoding.

usage: python vu1dec.py   (writes ts-rust/crates/ts-gfx/src/vu1_gen.rs and vu1_gen_ref.rs)
"""
import os, sys, struct
from ps2elf import Elf
from vu1 import extract

ROOT = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..')
OUT = os.path.join(ROOT, 'ts-rust', 'crates', 'ts-gfx', 'src', 'vu1_gen.rs')

# extra entry points: MPG chunk starts and every MSCAL target seen at run time
EXTRA_ENTRIES = {0x000, 0x100, 0x1a4, 0x2a4, 0x35c, 0x45c, 0x500, 0x600, 0x6a4, 0x75f, 0x76a, 0x775, 0x780, 0x78b,
                 0x796, 0x79d, 0x7a8, 0x7c4, 0x7e0, 0x7fc}


def dmask(w):
    """4-bit lane mask from the dest field (bit n = lane n; x=lane 0)."""
    return (w >> 24 & 1) | (w >> 23 & 1) << 1 | (w >> 22 & 1) << 2 | (w >> 21 & 1) << 3


def lanes(mask):
    return [l for l in range(4) if mask >> l & 1]


def sx11(l):
    v = l & 0x7ff
    return v - 0x800 if v & 0x400 else v


class Pair:
    def __init__(self, pc, w):
        self.pc = pc
        self.hi, self.lo = w >> 32, w & 0xffffffff
        self.E = bool(self.hi >> 30 & 1)
        self.I = bool(self.hi >> 31 & 1)
        # control-flow classification of the lower half
        self.kind = None
        self.target = None
        if not self.I:
            op = self.lo >> 25 & 0x7f
            imm = sx11(self.lo)
            if op in (0x20, 0x21):
                self.kind, self.target = ('B' if op == 0x20 else 'BAL'), pc + 1 + imm
            elif op in (0x28, 0x29, 0x2c, 0x2d, 0x2e, 0x2f):
                self.kind, self.target = 'COND', pc + 1 + imm
            elif op in (0x24, 0x25):
                self.kind = 'JR' if op == 0x24 else 'JALR'


UP_BC_OPS = {0x00: 'add', 0x04: 'sub', 0x08: 'madd', 0x0c: 'msub', 0x10: 'max', 0x14: 'mini', 0x18: 'mul'}
UP_Q = {0x1c: 'mul', 0x20: 'add', 0x21: 'madd', 0x24: 'sub', 0x25: 'msub'}
UP_I = {0x1d: 'max', 0x1e: 'mul', 0x1f: 'mini', 0x22: 'add', 0x23: 'madd', 0x26: 'sub', 0x27: 'msub'}
UP_VV = {0x28: 'add', 0x29: 'madd', 0x2a: 'mul', 0x2b: 'max', 0x2c: 'sub', 0x2d: 'msub', 0x2f: 'mini'}


def arith(op, a, b):
    return {'add': f'{a} + {b}', 'sub': f'{a} - {b}', 'mul': f'{a} * {b}', 'max': f'{a}.max({b})',
            'min': f'{a}.min({b})', 'mini': f'{a}.min({b})'}[op]


def emit_upper(p, out):
    """Compute the upper half now (reads old state); returns the commit statement to run after the lower half."""
    u = p.hi
    fn = u & 63
    ft, fs, fd = (u >> 16) & 31, (u >> 11) & 31, (u >> 6) & 31
    mask = dmask(u)
    n = f'u{p.pc:x}'
    if fn < 0x3c:
        op = bsrc = None
        acc_dest = False
        if fn in range(0x00, 0x1c):
            base = fn & 0x1c
            op = UP_BC_OPS[base]
            bsrc = ('bc', fn & 3)
        elif fn in UP_Q:
            op, bsrc = UP_Q[fn], ('q',)
        elif fn in UP_I:
            op, bsrc = UP_I[fn], ('i',)
        elif fn in UP_VV:
            op, bsrc = UP_VV[fn], ('vv',)
        elif fn == 0x2e:
            out.append(f'        let {n}a = r.vf[{fs}]; let {n}b = r.vf[{ft}]; let {n}c = r.acc;')
            out.append(f'        let {n} = [{n}c[0] - {n}a[1] * {n}b[2], {n}c[1] - {n}a[2] * {n}b[0], '
                       f'{n}c[2] - {n}a[0] * {n}b[1], 0.0];')
            return f'r.commit({fd}, {mask & 7}, {n}, true);'
        else:
            raise ValueError(f'upper {fn:#x} at {p.pc:#x}')
        out.append(f'        let {n}a = r.vf[{fs}];')
        if bsrc[0] == 'bc':
            out.append(f'        let {n}b = r.vf[{ft}][{bsrc[1]}];')
            bl = [f'{n}b'] * 4
        elif bsrc[0] == 'q':
            out.append(f'        let {n}b = r.q_now();')
            bl = [f'{n}b'] * 4
        elif bsrc[0] == 'i':
            out.append(f'        let {n}b = r.i;')
            bl = [f'{n}b'] * 4
        else:
            out.append(f'        let {n}b = r.vf[{ft}];')
            bl = [f'{n}b[{l}]' for l in range(4)]
        if op in ('madd', 'msub'):
            out.append(f'        let {n}c = r.acc;')
            sign = '+' if op == 'madd' else '-'
            exprs = [f'{n}c[{l}] {sign} {n}a[{l}] * {bl[l]}' for l in range(4)]
        else:
            exprs = [arith(op, f'{n}a[{l}]', bl[l]) for l in range(4)]
        out.append(f'        let {n} = [{", ".join(exprs)}];')
        return f'r.commit({fd}, {mask}, {n}, true);'
    idx = ((u >> 6) & 31) * 4 + (u & 3)
    # accumulator forms
    acc_ops = {}
    for i in range(4):
        acc_ops[0 + i] = ('add', ('bc', i))
        acc_ops[4 + i] = ('sub', ('bc', i))
        acc_ops[8 + i] = ('madd', ('bc', i))
        acc_ops[12 + i] = ('msub', ('bc', i))
        acc_ops[24 + i] = ('mul', ('bc', i))
    acc_ops.update({28: ('mul', ('q',)), 30: ('mul', ('i',)), 32: ('add', ('q',)), 33: ('madd', ('q',)),
                    34: ('add', ('i',)), 35: ('madd', ('i',)), 36: ('sub', ('q',)), 37: ('msub', ('q',)),
                    38: ('sub', ('i',)), 39: ('msub', ('i',)), 40: ('add', ('vv',)), 41: ('madd', ('vv',)),
                    42: ('mul', ('vv',)), 44: ('sub', ('vv',)), 45: ('msub', ('vv',))})
    if idx in acc_ops:
        op, bsrc = acc_ops[idx]
        out.append(f'        let {n}a = r.vf[{fs}];')
        if bsrc[0] == 'bc':
            out.append(f'        let {n}b = r.vf[{ft}][{bsrc[1]}];')
            bl = [f'{n}b'] * 4
        elif bsrc[0] == 'q':
            out.append(f'        let {n}b = r.q_now();')
            bl = [f'{n}b'] * 4
        elif bsrc[0] == 'i':
            out.append(f'        let {n}b = r.i;')
            bl = [f'{n}b'] * 4
        else:
            out.append(f'        let {n}b = r.vf[{ft}];')
            bl = [f'{n}b[{l}]' for l in range(4)]
        if op in ('madd', 'msub'):
            out.append(f'        let {n}c = r.acc;')
            sign = '+' if op == 'madd' else '-'
            exprs = [f'{n}c[{l}] {sign} {n}a[{l}] * {bl[l]}' for l in range(4)]
        else:
            exprs = [arith(op, f'{n}a[{l}]', bl[l]) for l in range(4)]
        out.append(f'        let {n} = [{", ".join(exprs)}];')
        return f'r.commit_acc({mask}, {n});'
    if idx == 46:  # OPMULA
        out.append(f'        let {n}a = r.vf[{fs}]; let {n}b = r.vf[{ft}];')
        out.append(f'        let {n} = [{n}a[1] * {n}b[2], {n}a[2] * {n}b[0], {n}a[0] * {n}b[1], 0.0];')
        return f'r.commit_acc({mask & 7}, {n});'
    if 16 <= idx <= 19:  # ITOF0/4/12/15
        sc = ['1.0', '16.0', '4096.0', '32768.0'][idx - 16]
        out.append(f'        let {n}a = r.vf[{fs}];')
        out.append(f'        let {n} = [{", ".join(f"({n}a[{l}].to_bits() as i32 as f32) / {sc}" for l in range(4))}];')
        return f'r.put({ft}, {mask}, {n});'
    if 20 <= idx <= 23:  # FTOI0/4/12/15
        sc = ['1.0', '16.0', '4096.0', '32768.0'][idx - 20]
        out.append(f'        let {n}a = r.vf[{fs}];')
        out.append(f'        let {n} = [{", ".join(f"ftoi({n}a[{l}], {sc})" for l in range(4))}];')
        return f'r.put_bits({ft}, {mask}, {n});'
    if idx == 29:  # ABS
        out.append(f'        let {n}a = r.vf[{fs}];')
        out.append(f'        let {n} = [{n}a[0].abs(), {n}a[1].abs(), {n}a[2].abs(), {n}a[3].abs()];')
        return f'r.put({ft}, {mask}, {n});'
    if idx == 31:  # CLIP
        return f'r.clip_op(r.vf[{fs}], r.vf[{ft}][3]);'
    if idx == 47:  # NOP
        return ''
    raise ValueError(f'upper special {idx} at {p.pc:#x}')


def emit_lower(p, out, delay=False):
    """Emit the lower half.  Returns ('seq'|'cond'|'jump'|..., expr) for branches (condition evaluated here)."""
    if p.I:
        out.append(f'        r.i = f32::from_bits({p.lo:#x});')
        return None
    l = p.lo
    op = l >> 25 & 0x7f
    ft, fs = (l >> 16) & 31, (l >> 11) & 31
    mask = dmask(l)
    imm = sx11(l)
    n = f'l{p.pc:x}'
    if op == 0x00:  # LQ
        out.append(f'        {{ let q = qw(r.vi[{fs}], {imm}); r.put_bits({ft}, {mask}, vu.mem[q]); }}')
    elif op == 0x01:  # SQ
        out.append(f'        {{ let q = qw(r.vi[{ft}], {imm}); let v = r.vf[{fs}];')
        for k in lanes(mask):
            out.append(f'          vu.mem[q][{k}] = v[{k}].to_bits();')
        out.append('        }')
    elif op == 0x04:  # ILW
        k = lanes(mask)[0] if mask else 0
        out.append(f'        {{ let q = qw(r.vi[{fs}], {imm}); r.vi_set({ft}, vu.mem[q][{k}] as u16); }}')
    elif op == 0x05:  # ISW
        out.append(f'        {{ let q = qw(r.vi[{fs}], {imm}); let v = r.vi[{ft}] as u32;')
        for k in lanes(mask):
            out.append(f'          vu.mem[q][{k}] = v;')
        out.append('        }')
    elif op in (0x08, 0x09):
        imm15 = ((l >> 21) & 15) << 11 | (l & 0x7ff)
        fn = 'wrapping_add' if op == 0x08 else 'wrapping_sub'
        out.append(f'        r.vi_set({ft}, r.vi[{fs}].{fn}({imm15}));')
    elif op == 0x10:
        out.append(f'        r.vi_set(1, (r.clip & 0xffffff == {l & 0xffffff:#x}) as u16);')
    elif op == 0x11:
        out.append(f'        r.clip = {l & 0xffffff:#x};')
    elif op == 0x12:
        out.append(f'        r.vi_set(1, (r.clip & {l & 0xffffff:#x} != 0) as u16);')
    elif op == 0x13:
        out.append(f'        r.vi_set(1, ((r.clip | {l & 0xffffff:#x}) & 0xffffff == 0xffffff) as u16);')
    elif op in (0x14, 0x15, 0x16, 0x17):
        out.append(f'        r.vi_set({ft}, 0); // status flag ops are not tracked')
    elif op == 0x18:
        out.append(f'        r.vi_set({ft}, (r.mac == r.vi[{fs}]) as u16);')
    elif op == 0x1a:
        out.append(f'        r.vi_set({ft}, r.mac & r.vi[{fs}]);')
    elif op == 0x1b:
        out.append(f'        r.vi_set({ft}, r.mac | r.vi[{fs}]);')
    elif op == 0x1c:
        out.append(f'        r.vi_set({ft}, (r.clip & 0xfff) as u16);')
    elif op == 0x20:
        return ('B', p.target, None)
    elif op == 0x21:
        out.append(f'        r.vi_set({ft}, {p.pc + 2});')
        return ('B', p.target, None)
    elif op == 0x24:
        out.append(f'        let {n}t = r.vi[{fs}] as usize;')
        return ('JR', f'{n}t', None)
    elif op == 0x25:
        out.append(f'        let {n}t = r.vi[{fs}] as usize;')
        out.append(f'        r.vi_set({ft}, {p.pc + 2});')
        return ('JR', f'{n}t', None)
    elif op in (0x28, 0x29, 0x2c, 0x2d, 0x2e, 0x2f):
        cond = {0x28: f'r.vi[{ft}] == r.vi[{fs}]', 0x29: f'r.vi[{ft}] != r.vi[{fs}]',
                0x2c: f'(r.vi[{fs}] as i16) < 0', 0x2d: f'(r.vi[{fs}] as i16) > 0',
                0x2e: f'(r.vi[{fs}] as i16) <= 0', 0x2f: f'(r.vi[{fs}] as i16) >= 0'}[op]
        out.append(f'        let {n}c = {cond};')
        return ('COND', p.target, f'{n}c')
    elif op == 0x40:
        fn = l & 63
        if fn in (0x30, 0x31, 0x34, 0x35):
            fd = (l >> 6) & 31
            e = {0x30: f'r.vi[{fs}].wrapping_add(r.vi[{ft}])', 0x31: f'r.vi[{fs}].wrapping_sub(r.vi[{ft}])',
                 0x34: f'r.vi[{fs}] & r.vi[{ft}]', 0x35: f'r.vi[{fs}] | r.vi[{ft}]'}[fn]
            out.append(f'        r.vi_set({fd}, {e});')
        elif fn == 0x32:
            imm5 = (l >> 6) & 31
            imm5 = imm5 - 32 if imm5 & 16 else imm5
            out.append(f'        r.vi_set({ft}, r.vi[{fs}].wrapping_add({imm5} as u16));')
        elif fn >= 0x3c:
            row = (l >> 6) & 31
            fsf, ftf = (l >> 21) & 3, (l >> 23) & 3
            key = (fn, row)
            if key == (0x3c, 0x0c) or key == (0x3d, 0x0c):  # MOVE / MR32
                if mask == 0:
                    return None
                if fn == 0x3c:
                    out.append(f'        {{ let v = r.vf[{fs}]; r.put({ft}, {mask}, v); }}')
                else:
                    out.append(f'        {{ let v = r.vf[{fs}]; r.put({ft}, {mask}, [v[1], v[2], v[3], v[0]]); }}')
            elif key in ((0x3c, 0x0d), (0x3e, 0x0d)):  # LQI / LQD
                if key == (0x3e, 0x0d):
                    out.append(f'        r.vi_set({fs}, r.vi[{fs}].wrapping_sub(1));')
                out.append(f'        {{ let q = qw(r.vi[{fs}], 0); r.put_bits({ft}, {mask}, vu.mem[q]); }}')
                if key == (0x3c, 0x0d):
                    out.append(f'        r.vi_set({fs}, r.vi[{fs}].wrapping_add(1));')
            elif key in ((0x3d, 0x0d), (0x3f, 0x0d)):  # SQI / SQD
                if key == (0x3f, 0x0d):
                    out.append(f'        r.vi_set({ft}, r.vi[{ft}].wrapping_sub(1));')
                out.append(f'        {{ let q = qw(r.vi[{ft}], 0); let v = r.vf[{fs}];')
                for k in lanes(mask):
                    out.append(f'          vu.mem[q][{k}] = v[{k}].to_bits();')
                out.append('        }')
                if key == (0x3d, 0x0d):
                    out.append(f'        r.vi_set({ft}, r.vi[{ft}].wrapping_add(1));')
            elif key == (0x3c, 0x0e):  # DIV
                out.append(f'        r.div(r.vf[{fs}][{fsf}], r.vf[{ft}][{ftf}]);')
            elif key == (0x3d, 0x0e):  # SQRT
                out.append(f'        r.sqrt(r.vf[{ft}][{ftf}]);')
            elif key == (0x3e, 0x0e):  # RSQRT
                out.append(f'        r.rsqrt(r.vf[{fs}][{fsf}], r.vf[{ft}][{ftf}]);')
            elif key == (0x3f, 0x0e):  # WAITQ
                out.append('        r.waitq();')
            elif key == (0x3c, 0x0f):  # MTIR
                out.append(f'        r.vi_set({ft}, r.vf[{fs}][{fsf}].to_bits() as u16);')
            elif key == (0x3d, 0x0f):  # MFIR
                out.append(f'        {{ let v = r.vi[{fs}] as i16 as i32 as u32; r.put_bits({ft}, {mask}, [v; 4]); }}')
            elif key == (0x3e, 0x0f):  # ILWR
                k = lanes(mask)[0] if mask else 0
                out.append(f'        {{ let q = qw(r.vi[{fs}], 0); r.vi_set({ft}, vu.mem[q][{k}] as u16); }}')
            elif key == (0x3f, 0x0f):  # ISWR
                out.append(f'        {{ let q = qw(r.vi[{fs}], 0); let v = r.vi[{ft}] as u32;')
                for k in lanes(mask):
                    out.append(f'          vu.mem[q][{k}] = v;')
                out.append('        }')
            elif key == (0x3c, 0x1a):  # XTOP
                out.append(f'        r.vi_set({ft}, vu.top as u16);')
            elif key == (0x3d, 0x1a):  # XITOP
                out.append(f'        r.vi_set({ft}, vu.itop as u16);')
            elif key == (0x3c, 0x1b):  # XGKICK
                out.append(f'        crate::programs::xgkick(&*vu, gif, frame, r.vi[{fs}] as usize);')
            else:
                raise ValueError(f'lower special {key} at {p.pc:#x}')
        else:
            raise ValueError(f'lower 0x40/{fn:#x} at {p.pc:#x}')
    else:
        raise ValueError(f'lower op {op:#x} at {p.pc:#x}')
    return None


MAC_LIVE = None


def mac_liveness(pairs, writers, leaders):
    """pcs whose MAC flag update can still be read (FMEQ/FMAND/FMOR) before another update replaces it; the others skip the
    flag computation.  Every flag-setting instruction replaces the whole MAC register, so it kills the older value.  The
    MAC register at the end of a program run is kept (the next run might read it)."""
    readers = {pc for pc, p in pairs.items() if not p.I and (p.lo >> 25 & 0x7f) in (0x18, 0x1a, 0x1b)}
    outcome = {}
    for pc, p in pairs.items():
        if p.kind in ('B', 'BAL'):
            outcome[pc] = {p.target}
        elif p.kind == 'COND':
            outcome[pc] = {p.target, pc + 2}
        elif p.kind in ('JR', 'JALR'):
            outcome[pc] = set(leaders)
    succ = {}
    exit_slot = set()
    for pc, p in pairs.items():
        t = {pc + 1}
        if pc - 1 in outcome:
            t |= outcome[pc - 1]  # delay slot: the branch decides where it goes on
        succ[pc] = {x for x in t if x in pairs}
        if pc - 1 in pairs and pairs[pc - 1].E and (pc - 1) not in outcome:
            exit_slot.add(pc)
        if p.E and pc not in outcome:
            exit_slot.add(pc + 1)
    live_in = set()
    live_out = set(exit_slot)
    changed = True
    while changed:
        changed = False
        for pc in pairs:
            lo = pc in exit_slot or any(s in live_in for s in succ[pc])
            if lo and pc not in live_out:
                live_out.add(pc)
                changed = True
            li = pc in readers or (lo and pc not in writers)
            if li and pc not in live_in:
                live_in.add(pc)
                changed = True
    return {pc for pc in writers if pc in live_out}


def pair_code(p, delay=False):
    out = [f'        // --- {p.pc:#05x}', '        r.tick();', f'        #[cfg(feature = "vutrace")] {{ if crate::trace::vu_enabled() {{ crate::trace::line(format!("      vu pc={p.pc:#x} vi={{:?}}", &r.vi[1..16])); }} }}']
    if p.I:
        out.append(f'        r.i = f32::from_bits({p.lo:#x});')
    elif (p.lo >> 25) == 0x40 and (p.lo & 63) == 0x3f and ((p.lo >> 6) & 31) == 0x0e:
        # WAITQ stalls the whole pair: the upper op (MULQ/ADDQ...) must see the finished divide
        out.append('        r.waitq();')
    commit = emit_upper(p, out)
    if commit and MAC_LIVE is not None and p.pc not in MAC_LIVE:
        commit = commit.replace(', true);', ', false);').replace('r.commit_acc(', 'r.commit_acc_nf(')
    br = None
    if not p.I:
        br = emit_lower(p, out)
    if commit:
        out.append('        ' + commit)
    return out, br


def generate():
    e = Elf()
    mem = extract(e.image())
    pairs = {pc: Pair(pc, w) for pc, w in mem.items()}
    global MAC_LIVE
    leaders = set(EXTRA_ENTRIES)
    for pc, p in pairs.items():
        if p.kind in ('B', 'BAL', 'COND'):
            leaders.add(p.target)
        if p.kind is not None:
            leaders.add(pc + 2)
    leaders = {a for a in leaders if a in pairs}
    # which pairs update the MAC flags (found by emitting them once), and which of those updates are ever read
    MAC_LIVE = None
    writers = set()
    for pc, p in pairs.items():
        try:
            code, _ = pair_code(p)
        except (ValueError, KeyError, TypeError):
            continue
        if any(l.strip().startswith(('r.commit(', 'r.commit_acc(')) and l.rstrip().endswith('true);') or l.strip().startswith('r.commit_acc(') for l in code):
            writers.add(pc)
    MAC_LIVE = mac_liveness(pairs, writers, leaders)
    print(f'MAC flags: {len(writers)} updating pairs, {len(MAC_LIVE)} of them read')
    out = []
    out.append('// @generated by tools/vu1dec.py from the VU1 microcode in SLUS_200.90 -- do not edit.')
    out.append('#![allow(unused_variables, unused_mut, unused_assignments, unused_parens, dead_code, '
               'clippy::all)]')
    out.append('use crate::gif::Gif;')
    out.append('use crate::scene::Frame;')
    out.append('use crate::vif::Vu1;')
    out.append('use crate::vu::*;')
    out.append('')
    out.append('/// Run the microprogram that starts at `entry` (an address in 64-bit instruction units).')
    out.append('pub fn run(vu: &mut Vu1, r: &mut Regs, gif: &mut Gif, frame: &mut Frame, entry: usize) {')
    out.append('    let mut pc = entry;')
    out.append('    loop {')
    out.append('        match pc {')
    emitted = 0
    for start in sorted(leaders):
        body = [f'        {start:#x} => {{']
        pc = start
        while True:
            p = pairs.get(pc)
            if p is None:
                body.append(f'            panic!("VU1 program ran off the end at {pc:#x}");')
                break
            code, br = pair_code(p)
            body += ['    ' + c for c in code]
            emitted += 1
            if br is not None or p.kind in ('B', 'BAL', 'COND', 'JR', 'JALR'):
                dp = pairs.get(pc + 1)
                if dp is not None:
                    dcode, _ = pair_code(dp, True)
                    body += ['    ' + c for c in dcode]
                    emitted += 1
                kind, tgt, cond = br
                if kind == 'B':
                    body.append(f'            pc = {tgt:#x}; continue;')
                elif kind == 'COND':
                    body.append(f'            if {cond} {{ pc = {tgt:#x}; }} else {{ pc = {pc + 2:#x}; }} continue;')
                else:
                    body.append(f'            pc = {tgt}; continue;')
                if p.E:
                    body.insert(len(body) - 1, '            // (E bit on a branch pair: program continues through the branch)')
                break
            if p.E:
                dp = pairs.get(pc + 1)
                if dp is not None:
                    dcode, _ = pair_code(dp, True)
                    body += ['    ' + c for c in dcode]
                    emitted += 1
                body.append('            return;')
                break
            pc += 1
            if pc in leaders:
                body.append(f'            pc = {pc:#x}; continue;')
                break
        body.append('        }')
        out += body
    out.append('        _ => panic!("VU1 microprogram entry {:#x} is not a known block start", pc),')
    out.append('        }')
    out.append('    }')
    out.append('}')
    open(OUT, 'w', encoding='utf8').write('\n'.join(out) + '\n')
    print(f'{len(leaders)} blocks, {emitted} instruction pairs emitted, {len(out)} lines -> {OUT}')


if __name__ == '__main__':
    generate()
