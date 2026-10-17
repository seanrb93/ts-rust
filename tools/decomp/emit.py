"""Rust code emission for structured, typed functions."""
import re
import struct
import bisect
from .ir import *
from .analysis import St, ARG_G, ARG_F, RET_REGS

ATOM, UNARY, CAST, MUL, ADD, SHIFT, BAND, BXOR, BOR, CMP, LAND, LOR = 100, 90, 80, 70, 60, 50, 45, 44, 43, 40, 30, 20

GPR_NAMES = ['zero', 'at', 'v0', 'v1', 'a0', 'a1', 'a2', 'a3', 'a4', 'a5', 'a6', 'a7', 't4', 't5', 't6', 't7',
             's0', 's1', 's2', 's3', 's4', 's5', 's6', 's7', 't8', 't9', 'k0', 'k1', 'gp', 'sp', 'fp', 'ra']
RUST_KW = set('as break const continue crate else enum extern false fn for if impl in let loop match mod move mut pub '
              'ref return self Self static struct super trait true type unsafe use where while async await dyn abstract '
              'become box do final macro override priv typeof unsized virtual yield try main'.split())


def rs_name(sym):
    n = re.sub(r'[^A-Za-z0-9_]', '_', sym)
    if not n or n[0].isdigit():
        n = '_' + n
    if n in RUST_KW:
        n += '_'
    return n


def reg_base(reg):
    if reg[0] == 'r':
        return GPR_NAMES[int(reg[1:])]
    return reg


def f32_lit(bits):
    f = bits2f(bits)
    if f != f or f in (float('inf'), float('-inf')):
        return f'f32::from_bits({bits:#x})'
    if bits == 0x80000000:
        return '-0.0_f32'
    for n in range(1, 10):
        s = f'{f:.{n}g}'
        if struct.unpack('<f', struct.pack('<f', float(s)))[0] == f:
            break
    if 'e' not in s and '.' not in s:
        s += '.0'
    return s + '_f32'


# functions whose 2D sprites are placed by projecting world positions (not UI): the display-list range they write is
# recorded so the renderer does not treat them as HUD (see ts-game/src/manual/hooks.rs)
WORLD_SPRITE_FUNCS = {'zbtestGfx'}

class SymTab:
    """Maps constants to data-symbol expressions."""

    def __init__(self, elf):
        objs = sorted([s for s in elf.symbols if s.kind == 1 and s.addr >= 0x2f0000 and s.name],
                      key=lambda s: (s.addr, -s.size))
        uniq = []
        for s in objs:                      # several symbols may alias one address: keep the largest
            if not uniq or uniq[-1].addr != s.addr:
                uniq.append(s)
        objs = uniq
        self.objs = objs
        self.starts = [s.addr for s in objs]
        import collections
        cnt = collections.Counter(rs_name(s.name) for s in objs)
        self.uname = {}
        for s in objs:
            n = rs_name(s.name)
            self.uname[s.addr] = n if cnt[n] == 1 else f'{n}_{s.addr:x}'
        self.funcs = {f.addr: f for f in elf.functions()}

    def lookup(self, v):
        if v < 0x2f0000 or v >= 0x2000000:
            return None
        k = bisect.bisect_right(self.starts, v) - 1
        if k < 0:
            return None
        s = self.objs[k]
        if s.addr <= v < s.addr + max(s.size, 1):
            return s
        # one-past-end pointers
        if v == s.addr + s.size and s.size:
            return s
        return None


class Emitter:
    def __init__(self, func, sigs, rettypes, symtab, funcnames):
        self.f = func
        self.sigs = sigs
        self.rettypes = rettypes
        self.sym = symtab
        self.webs = func.webs
        self.names = {}
        self.lines = []
        self.ind = 1
        self.uses_sp = False
        self.counter = {}
        self.loops = []       # stack of loop labels
        self.frames = []      # stack of ('loop'|'block', label)
        self.assign_ref = {}
        self.funcnames = funcnames
        self.assign_names()

    # ---------------------------------------------------------------- names / types
    def assign_names(self):
        webs = self.webs
        used = set()
        # parameters first
        for r in self.f.entry_params:
            for w in webs.values():
                if w.param and w.reg == r and w.live:
                    self.names[w.id] = reg_base(r)
        cnt = {}
        for w in sorted(webs.values(), key=lambda w: w.id):
            if w.id in self.names or not (w.live or w.param):
                continue
            base = reg_base(w.reg)
            n = cnt.get(base, 0) + 1
            cnt[base] = n
            nm = f'{base}_{n}'
            self.names[w.id] = nm

    def wty(self, wid):
        return self.webs[wid].ty or 'i32'

    def tystr(self, t):
        return {'i32': 'i32', 'i64': 'i64', 'f32': 'f32', 'bool': 'bool', 'u128': 'u128'}[t]

    # ---------------------------------------------------------------- expressions
    def paren(self, c, minprec):
        t, p = c
        return f'({t})' if p < minprec else t

    def coerce(self, c, frm, to):
        if frm == to:
            return c
        t, p = c
        pc = self.paren(c, CAST)
        if to == 'bool':
            if frm in ('i32', 'i64', 'u128'):
                return (f'{self.paren(c, CMP + 1)} != 0', CMP)
            if frm == 'f32':
                return (f'{self.paren(c, CMP + 1)} != 0.0', CMP)
        if frm == 'bool':
            if to == 'f32':
                return (f'(({t}) as i32 as f32)', ATOM)
            return (f'{pc} as {self.tystr(to)}', CAST)
        if to == 'f32':
            return (f'f32::from_bits({self.coerce(c, frm, "i32")[0]} as u32)', ATOM) if frm != 'i32' else \
                (f'f32::from_bits({pc} as u32)', ATOM)
        if frm == 'f32':
            return (f'{self.paren(c, ATOM)}.to_bits() as {self.tystr(to)}', CAST)
        return (f'{pc} as {self.tystr(to)}', CAST)

    def lit(self, v, want):
        if want == 'f32':
            return (f32_lit(v & M32), ATOM)
        if want == 'bool':
            return ('true' if v else 'false', ATOM)
        s = f'{v:#x}' if (v >= 4096) else (f'-{-v:#x}' if v <= -4096 else str(v))
        if want == 'i64':
            s += '_i64'
        elif want == 'u128':
            s = f'{v & ((1 << 128) - 1)}_u128'
        return (s, UNARY if v < 0 else ATOM)

    def const_sym(self, v):
        s = self.sym.lookup(v & M32)
        if s is None:
            return None
        off = (v & M32) - s.addr
        nm = f'g::{self.sym.uname[s.addr]}'
        if off == 0:
            return (nm, ATOM)
        return (f'{nm} + {off}', ADD)

    def var(self, wid):
        return (self.names[wid], ATOM)

    def ex(self, e, want=None):
        """Emit expression; coerce to `want` if given."""
        if want in ('i32', 'i64', 'u128') and e[0] == 'c':
            if want == 'i32' and e[2] == 'i32' and e[1] >= 0x2f0000:
                cs = self.const_sym(e[1])
                if cs:
                    return cs
            return self.lit(e[1], want)
        c, t = self.ex2(e)
        if want is not None and want != t:
            c = self.coerce(c, t, want)
        return c

    def ext(self, e, want):
        return self.paren(self.ex(e, want), 0)

    def ety(self, e):
        return ty(e, self.wty)

    def recv(self, e, want):
        """Expression used as a method-call receiver (integer literals need an explicit type)."""
        c = self.paren(self.ex(e, want), ATOM)
        if e[0] == 'c':
            c = f'({c}_{want})' if not c.endswith(('_i64', '_u128')) else f'({c})'
        return c

    def binop(self, sym, prec, a, b, want, rassoc_extra=1):
        ca = self.paren(self.ex(a, want), prec)
        cb = self.paren(self.ex(b, want), prec + rassoc_extra)
        return (f'{ca} {sym} {cb}', prec)

    def ex2(self, e):
        t = e[0]
        if t == 'c':
            if e[2] == 'i32':
                cs = self.const_sym(e[1]) if e[1] >= 0x2f0000 else None
                if cs:
                    return cs, 'i32'
            return self.lit(e[1], e[2]), e[2]
        if t == 'cf':
            return (f32_lit(e[1]), ATOM), 'f32'
        if t == 'cb':
            return ('true' if e[1] else 'false', ATOM), 'bool'
        if t == 'v':
            return self.var(e[1]), self.wty(e[1])
        if t == 'sp':
            self.uses_sp = True
            return (('sp' if e[1] == 0 else f'sp + {e[1]}'), ADD if e[1] else ATOM), 'i32'
        if t == 'ld':
            kind = e[1]
            fn = {'b': 'lb', 'bu': 'lbu', 'h': 'lh', 'hu': 'lhu', 'w': 'lw', 'wu': 'lwu', 'd': 'ld', 'f': 'lwc1',
                  'q': 'lq'}[kind]
            return (f'{fn}({self.ext(e[2], "i32")})', ATOM), LOAD_TY[kind]
        rt = self.ety(e)
        if t in ('add32', 'add'):
            w = rt
            a, b = e[1], e[2]
            if b[0] == 'c' and b[1] < 0 and b[1] > -0x80000000:
                nb = ('c', -b[1], b[2])
                return self.binop('-', ADD, a, nb, w), w
            return self.binop('+', ADD, a, b, w), w
        if t in ('sub32', 'sub'):
            return self.binop('-', ADD, e[1], e[2], rt), rt
        if t == 'mul32':
            return self.binop('*', MUL, e[1], e[2], 'i32'), 'i32'
        if t == 'and':
            return self.binop('&', BAND, e[1], e[2], rt), rt
        if t == 'or':
            return self.binop('|', BOR, e[1], e[2], rt), rt
        if t == 'xor':
            return self.binop('^', BXOR, e[1], e[2], rt), rt
        if t == 'nor':
            a = self.paren(self.ex(('or', e[1], e[2]), rt), UNARY)
            return (f'!{a}', UNARY), rt
        if t in ('shl32', 'shl64'):
            w = 'i32' if t == 'shl32' else 'i64'
            ca = self.paren(self.ex(e[1], w), CAST + 1)
            cb = self.paren(self.ex(e[2], 'i32'), SHIFT + 1)
            return (f'{ca} << {cb}', SHIFT), w
        if t == 'sar32' or t == 'sar64':
            w = 'i32' if t == 'sar32' else 'i64'
            ca = self.paren(self.ex(e[1], w), CAST + 1)
            cb = self.paren(self.ex(e[2], 'i32'), SHIFT + 1)
            return (f'{ca} >> {cb}', SHIFT), w
        if t == 'shr32' or t == 'shr64':
            w, u = ('i32', 'u32') if t == 'shr32' else ('i64', 'u64')
            ca = self.paren(self.ex(e[1], w), CAST)
            cb = self.paren(self.ex(e[2], 'i32'), SHIFT + 1)
            return (f'(({ca} as {u}) >> {cb}) as {w}', CAST), w
        if t == 'mulhi':
            ca, cb = self.paren(self.ex(e[1], 'i32'), CAST), self.paren(self.ex(e[2], 'i32'), CAST)
            return (f'((({ca} as i64) * ({cb} as i64)) >> 32) as i32', CAST), 'i32'
        if t == 'mulhiu':
            ca, cb = self.paren(self.ex(e[1], 'i32'), CAST), self.paren(self.ex(e[2], 'i32'), CAST)
            return (f'((({ca} as u32 as u64) * ({cb} as u32 as u64)) >> 32) as i32', CAST), 'i32'
        if t in ('maddhi', 'maddhiu'):
            hi, lo, a, b = [self.paren(self.ex(x, 'i32'), CAST) for x in e[1:5]]
            fn = 'maddhi' if t == 'maddhi' else 'maddhiu'
            return (f'{fn}({hi}, {lo}, {a}, {b})', ATOM), 'i32'
        if t == 'div32':
            return (f'{self.recv(e[1], "i32")}.wrapping_div({self.ext(e[2], "i32")})', ATOM), 'i32'
        if t == 'rem32':
            return (f'{self.recv(e[1], "i32")}.wrapping_rem({self.ext(e[2], "i32")})', ATOM), 'i32'
        if t == 'divu32':
            return (f'(({self.paren(self.ex(e[1], "i32"), CAST)} as u32) / ({self.paren(self.ex(e[2], "i32"), CAST)} '
                    f'as u32)) as i32', CAST), 'i32'
        if t == 'remu32':
            return (f'(({self.paren(self.ex(e[1], "i32"), CAST)} as u32) % ({self.paren(self.ex(e[2], "i32"), CAST)} '
                    f'as u32)) as i32', CAST), 'i32'
        if t == 'sel':
            w = rt
            return (f'if {self.ext(e[1], "bool")} {{ {self.ext(e[2], w)} }} else {{ {self.ext(e[3], w)} }}', 0), w
        if t == 'b2i':
            return (f'{self.paren(self.ex(e[1], "bool"), CAST)} as i32', CAST), 'i32'
        # comparisons
        if t in ('eq', 'ne', 'lt', 'le', 'gt', 'ge'):
            ta, tb = self.ety(e[1]), self.ety(e[2])
            # (a 128-bit register is compared by its low 64 bits, signed, like any 64-bit one)
            w = 'i64' if ('i64' in (ta, tb) or 'u128' in (ta, tb)) else 'i32'
            if w == 'i32' and 'bool' in (ta, tb) or (ta == 'bool' and tb == 'bool'):
                w = 'bool' if (ta == 'bool' and tb == 'bool') else 'i32'
            if ta == 'f32' and tb == 'f32':
                w = 'f32'
            sym = {'eq': '==', 'ne': '!=', 'lt': '<', 'le': '<=', 'gt': '>', 'ge': '>='}[t]
            ca = self.paren(self.ex(e[1], w), CAST + 1)
            cb = self.paren(self.ex(e[2], w), CMP + 1)
            return (f'{ca} {sym} {cb}', CMP), 'bool'
        if t in ('ltu', 'leu', 'gtu', 'geu'):
            ta, tb = self.ety(e[1]), self.ety(e[2])
            w = 'i64' if 'i64' in (ta, tb) else 'i32'
            u = 'u64' if w == 'i64' else 'u32'
            sym = {'ltu': '<', 'leu': '<=', 'gtu': '>', 'geu': '>='}[t]
            ca = self.paren(self.ex(e[1], w), CAST)
            cb = self.paren(self.ex(e[2], w), CAST)
            return (f'({ca} as {u}) {sym} ({cb} as {u})', CMP), 'bool'
        if t == 'not':
            c = self.ex(e[1], 'bool')
            return (f'!{self.paren(c, UNARY)}', UNARY), 'bool'
        if t in ('feq', 'flt', 'fle'):
            sym = {'feq': '==', 'flt': '<', 'fle': '<='}[t]
            ca = self.paren(self.ex(e[1], 'f32'), CAST + 1)
            cb = self.paren(self.ex(e[2], 'f32'), CMP + 1)
            return (f'{ca} {sym} {cb}', CMP), 'bool'
        if t in ('fadd', 'fsub', 'fmul', 'fdiv'):
            sym = {'fadd': '+', 'fsub': '-', 'fmul': '*', 'fdiv': '/'}[t]
            p = ADD if t in ('fadd', 'fsub') else MUL
            return self.binop(sym, p, e[1], e[2], 'f32'), 'f32'
        if t == 'fneg':
            return (f'-{self.paren(self.ex(e[1], "f32"), UNARY)}', UNARY), 'f32'
        if t == 'fabs':
            return (f'{self.paren(self.ex(e[1], "f32"), ATOM)}.abs()', ATOM), 'f32'
        if t == 'fsqrt':
            return (f'{self.paren(self.ex(e[1], "f32"), ATOM)}.abs().sqrt()', ATOM), 'f32'
        if t == 'frsqrt':
            return (f'{self.paren(self.ex(e[1], "f32"), MUL)} / {self.paren(self.ex(e[2], "f32"), ATOM)}.abs().sqrt()',
                    MUL), 'f32'
        if t == 'fmax':
            return (f'{self.paren(self.ex(e[1], "f32"), ATOM)}.max({self.ext(e[2], "f32")})', ATOM), 'f32'
        if t == 'fmin':
            return (f'{self.paren(self.ex(e[1], "f32"), ATOM)}.min({self.ext(e[2], "f32")})', ATOM), 'f32'
        if t == 'w2f':
            return (f'{self.paren(self.ex(e[1], "i32"), CAST)} as f32', CAST), 'f32'
        if t == 'f2w':
            return (f'{self.paren(self.ex(e[1], "f32"), CAST)} as i32', CAST), 'i32'
        if t == 'mtc1':
            return (f'f32::from_bits({self.paren(self.ex(e[1], "i32"), CAST)} as u32)', ATOM), 'f32'
        if t == 'mfc1':
            return (f'{self.paren(self.ex(e[1], "f32"), ATOM)}.to_bits() as i32', CAST), 'i32'
        if t == 'vop':
            args = ', '.join(self.ext(x, 'u128') if x[0] != 'c' or True else '' for x in e[3:])
            if e[1] in ('psllh', 'psrlh', 'psrah', 'psllw', 'psrlw', 'psraw'):
                args = self.ext(e[3], 'u128') + ', ' + self.ext(e[4], 'i32')
            return (f'{e[1]}({args})', ATOM), 'u128'
        raise KeyError(t)

    # ---------------------------------------------------------------- statements
    def w(self, s):
        self.lines.append('    ' * self.ind + s)

    def store_val(self, kind, v):
        if kind in ('b', 'h', 'w'):
            return self.ext(v, 'i32')
        if kind == 'd':
            return self.ext(v, 'i64')
        if kind == 'q':
            return self.ext(v, 'u128')
        return self.ext(v, 'f32')

    def callee_name(self, tgt):
        sg = self.sigs.get(tgt)
        return rs_name(sg.name) if sg else f'sub_{tgt:x}'

    def emit_call(self, s):
        sig = s.sig
        args = []
        if s.tgt is not None and sig is not None and sig.dynamic:
            lst = []
            for r, wid in s.argwebs:
                lst.append(self.ext(('v', wid), 'i64') if r[0] == 'r' else
                           f'{self.paren(self.ex(("v", wid), "f32"), ATOM)}.to_bits() as i64')
            call = f'{self.callee_name(s.tgt)}(&[{", ".join(lst)}])'
            rt = {'r2': 'i32', 'f0': 'f32'}
        elif s.tgt is not None:
            for (r, wid), pr in zip(s.argwebs, s.args):
                want = 'f32' if r[0] == 'f' else 'i32'
                args.append(self.ext(('v', wid), want))
            # callee arg types follow its own param types (declared i32/f32 except promoted ones)
            pts = getattr(sig, 'ptypes', None)
            if pts:
                args = []
                for (r, wid), pt in zip(s.argwebs, pts):
                    args.append(self.ext(('v', wid), pt))
            call = f'{self.callee_name(s.tgt)}({", ".join(args)})'
            rt = self.rettypes.get(s.tgt) or {}
        else:
            ints = [a for a in s.argwebs if a[0][0] == 'r']
            fl = [a for a in s.argwebs if a[0][0] == 'f']
            ia = [self.ext(('v', w_), 'i32') for r, w_ in ints] + ['0'] * (8 - len(ints))
            fa = [self.ext(('v', w_), 'f32') for r, w_ in fl] + ['0.0'] * (8 - len(fl))
            call = f'call_indirect({self.ext(s.ind, "i32")}, [{", ".join(ia)}], [{", ".join(fa)}])'
            rt = {'r2': 'i32', 'f0': 'f32'}
        rets = s.retwebs or []
        if not rets:
            self.w(call + ';')
            return
        got = {r: wid for r, wid in rets}
        if s.tgt is not None and s.sig is not None and not s.sig.dynamic:
            order = list(s.sig.rets)
        elif s.tgt is not None and s.sig is not None and s.sig.extern:
            order = ['r2']
        else:
            order = [r for r in ('r2', 'f0') if r in got] if s.tgt is not None else ['r2', 'f0']
        if len(order) == 1:
            r = order[0]
            if r in got:
                wid = got[r]
                frm = rt.get(r, 'f32' if r[0] == 'f' else 'i32')
                c = self.coerce((call, ATOM), frm, self.wty(wid))
                self.w(f'{self.names[wid]} = {c[0]};')
            else:
                self.w(call + ';')
        else:
            tmp = 'ret_' + str(abs(hash(call)) % 1000)
            self.w(f'let {tmp} = {call};')
            for i, r in enumerate(order):
                if r in got:
                    wid = got[r]
                    frm = rt.get(r, 'f32' if r[0] == 'f' else 'i32')
                    c = self.coerce((f'{tmp}.{i}', ATOM), frm, self.wty(wid))
                    self.w(f'{self.names[wid]} = {c[0]};')

    def emit_stmt(self, s):
        k = s.k
        if k == 'set':
            wid = s.dweb
            self.w(f'{self.names[wid]} = {self.ext(s.e, self.wty(wid))};')
        elif k == 'st':
            fn = {'b': 'sb', 'h': 'sh', 'w': 'sw', 'd': 'sd', 'q': 'sq', 'f': 'swc1'}[s.sk]
            self.w(f'{fn}({self.ext(s.a, "i32")}, {self.store_val(s.sk, s.v)});')
        elif k == 'call':
            self.emit_call(s)
        elif k == 'trap':
            self.w(f'trap("{s.what}");')
        else:
            raise KeyError(k)

    def ret_expr(self, t):
        rets = self.sigs[self.f.addr].rets
        parts = []
        rt = self.my_rettypes
        got = {r: wid for r, wid in (t.argwebs or [])}
        for r in rets:
            ty_ = rt.get(r, 'i32' if r[0] == 'r' else 'f32')
            if r in got:
                parts.append(self.ext(('v', got[r]), ty_))
            else:
                parts.append('0' if ty_ != 'f32' else '0.0')
        if not parts:
            return None
        return parts[0] if len(parts) == 1 else '(' + ', '.join(parts) + ')'

    def emit_body(self, body):
        for it in body:
            if isinstance(it, St):
                self.emit_stmt(it)
                continue
            k = it[0]
            if k == 'if':
                self.emit_if(it)
            elif k == 'block':
                self.w(f"'b{it[1]}: {{")
                self.ind += 1
                self.frames.append(('block', it[1]))
                self.emit_body(it[2])
                self.frames.pop()
                self.ind -= 1
                self.w('}')
            elif k == 'loop':
                self.w(f"'l{it[1]}: loop {{")
                self.loops.append(it[1])
                self.frames.append(('loop', it[1]))
                self.ind += 1
                self.w('#[cfg(feature = "calltrace")]')
                self.w('ts_rt::calltrace::poll();')
                self.emit_body(it[2])
                self.ind -= 1
                self.frames.pop()
                self.loops.pop()
                self.w('}')
            elif k == 'while':
                c = self.ext(it[2], 'bool')
                self.w(f"'l{it[1]}: while {c} {{")
                self.loops.append(it[1])
                self.frames.append(('loop', it[1]))
                self.ind += 1
                self.w('#[cfg(feature = "calltrace")]')
                self.w('ts_rt::calltrace::poll();')
                self.emit_body(it[3])
                self.ind -= 1
                self.frames.pop()
                self.loops.pop()
                self.w('}')
            elif k == 'break':
                self.w(self.jump('break', it[1]))
            elif k == 'continue':
                self.w(self.jump('continue', it[1]))
            elif k == 'ret':
                t = it[1]
                if t.k == 'tail':
                    self.emit_tail(t)
                else:
                    e = self.ret_expr(t)
                    self.w(f'return {e};' if e else 'return;')
            elif k == 'switch':
                self.w(f'match {self.ext(it[1], "i32")} {{')
                self.ind += 1
                for vals, bd in it[2]:
                    self.w(' | '.join(str(v) for v in vals) + ' => {')
                    self.ind += 1
                    self.emit_body(bd)
                    self.ind -= 1
                    self.w('}')
                self.w('_ => unreachable!(),')
                self.ind -= 1
                self.w('}')
            elif k == 'dispatch':
                self.w(f'let mut pc: u32 = {it[1]:#x};')
                self.w("'disp: loop {")
                self.ind += 1
                self.w('match pc {')
                self.ind += 1
                for a, bd in it[2]:
                    self.w(f'{a & 0xffffffffff:#x} => {{')
                    self.ind += 1
                    self.emit_body(bd)
                    self.ind -= 1
                    self.w('}')
                self.w('_ => unreachable!(),')
                self.ind -= 1
                self.w('}')
                self.ind -= 1
                self.w('}')
            elif k == 'goto':
                self.w(f"pc = {it[1] & 0xffffffffff:#x}; continue 'disp;")
            else:
                raise KeyError(k)

    def emit_tail(self, t):
        # tail call: call then return the callee's result
        sig = t.sig
        if t.tgt is not None and sig is not None and sig.dynamic:
            lst = []
            for r, wid in t.argwebs:
                lst.append(self.ext(('v', wid), 'i64') if r[0] == 'r' else
                           f'{self.paren(self.ex(("v", wid), "f32"), ATOM)}.to_bits() as i64')
            call = f'{self.callee_name(t.tgt)}(&[{", ".join(lst)}])'
            mine = self.sigs[self.f.addr].rets
            if mine == ['r2']:
                self.w(f'return {call};')
            else:
                self.w(f'{call};')
                self.w('return;')
        elif t.tgt is not None:
            args = ', '.join(self.ext(('v', wid), 'f32' if r[0] == 'f' else 'i32') for r, wid in t.argwebs)
            call = f'{self.callee_name(t.tgt)}({args})'
            mine = self.sigs[self.f.addr].rets
            if mine:
                self.w(f'return {call};')
            else:
                self.w(f'{call};')
                self.w('return;')
        else:
            ints = [a for a in t.argwebs if a[0][0] == 'r']
            fl = [a for a in t.argwebs if a[0][0] == 'f']
            ia = [self.ext(('v', w_), 'i32') for r, w_ in ints] + ['0'] * (8 - len(ints))
            fa = [self.ext(('v', w_), 'f32') for r, w_ in fl] + ['0.0'] * (8 - len(fl))
            self.w(f'let _r = call_indirect({self.ext(t.ind, "i32")}, [{", ".join(ia)}], [{", ".join(fa)}]);')
            mine = self.sigs[self.f.addr].rets
            if mine == ['r2']:
                self.w('return _r.0;')
            elif mine == ['f0']:
                self.w('return _r.1;')
            else:
                self.w('return;')

    def jump(self, kind, label):
        top = self.frames[-1] if self.frames else None
        if kind == 'continue':
            if top == ('loop', label):
                return 'continue;'
            return f"continue 'l{label};"
        if label in self.loops:
            if top == ('loop', label):
                return 'break;'
            return f"break 'l{label};"
        return f"break 'b{label};"

    def emit_if(self, it, head='if'):
        c = self.ext(it[1], 'bool')
        t, e = it[2], it[3]
        self.w(f'{head} {c} {{')
        self.ind += 1
        self.emit_body(t)
        self.ind -= 1
        if e:
            if len(e) == 1 and not isinstance(e[0], St) and e[0][0] == 'if':
                return self.emit_if(e[0], '} else if')
            self.w('} else {')
            self.ind += 1
            self.emit_body(e)
            self.ind -= 1
        self.w('}')

    # ---------------------------------------------------------------- whole function
    def emit_function(self, body, my_rettypes, export_name=None):
        self.my_rettypes = my_rettypes
        sig = self.sigs[self.f.addr]
        # parameter list
        params = []
        ptypes = []
        pw = {}
        for w in self.webs.values():
            if w.param:
                pw[w.reg] = w
        for r in sig.params:
            w = pw.get(r)
            t = self.tystr(w.ty) if (w is not None and w.ty) else ('f32' if r[0] == 'f' else 'i32')
            if w is None or not w.live:
                params.append(f'_{reg_base(r)}: {t}')
            else:
                params.append(f'mut {self.names[w.id]}: {t}')
            ptypes.append(t)
        sig.ptypes = ptypes
        rets = sig.rets
        rts = [self.tystr(my_rettypes.get(r, 'i32' if r[0] == 'r' else 'f32')) for r in rets]
        sig.rtypes = rts
        rtype = '' if not rets else (f' -> {rts[0]}' if len(rets) == 1 else f' -> ({", ".join(rts)})')
        # body first (determines sp usage)
        self.ind = 1
        main_lines = []
        saved = self.lines
        self.lines = main_lines
        self.emit_body(body)
        self.lines = saved
        hdr = [f'pub fn {rs_name(self.f.name)}({", ".join(params)}){rtype} {{',
               '    #[cfg(feature = "calltrace")]',
               f'    let _ct = ts_rt::calltrace::enter("{self.f.name}");']
        if sig.params:
            names = [(f'{self.names[pw[r].id]}' if (r in pw and pw[r].live) else f'_{reg_base(r)}') for r in sig.params]
            hdr += ['    #[cfg(feature = "calltrace")]',
                    f'    ts_rt::calltrace::trace_args("{self.f.name}", &[{", ".join(("0" if n.startswith("_") else f"{n} as i64") for n in names)}]);']
        # differential-testing hook (feature `shadow`): hands the entry arguments to the shadow interpreter
        ints, flts = ['0'] * 8, ['0.0'] * 8
        for r, nm in zip(sig.params, names if sig.params else []):
            if nm.startswith('_'):
                continue
            if r[0] == 'r' and 4 <= int(r[1:]) < 12:
                ints[int(r[1:]) - 4] = f'{nm} as i64'
            elif r[0] == 'f' and 12 <= int(r[1:]) < 20:
                flts[int(r[1:]) - 12] = f'{nm}'
        if self.f.name in WORLD_SPRITE_FUNCS:
            hdr += ['    let _hk = crate::manual::hooks::enter();']
        hdr += ['    #[cfg(feature = "shadow")]',
                f'    let _sh = ts_rt::shadow::enter({self.f.addr:#x}, "{self.f.name}", [{", ".join(ints)}], [{", ".join(flts)}]);']
        decl = []
        if self.uses_sp:
            decl.append(f'    let _frame = Frame::new({max(self.f.frame, 16)});')
            decl.append('    let sp = _frame.sp();')
        for w in sorted(self.webs.values(), key=lambda w: w.id):
            if (w.live) and not w.param and w.id in self.names:
                t = w.ty or 'i32'
                init = {'i32': '0', 'i64': '0', 'f32': '0.0', 'bool': 'false', 'u128': '0'}[t]
                decl.append(f'    let mut {self.names[w.id]}: {self.tystr(t)} = {init};')
        self.lines = hdr + decl + main_lines + ['}']
        return '\n'.join(self.lines)
