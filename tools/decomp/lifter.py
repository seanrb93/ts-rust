"""R5900 machine code -> register-level IR with explicit basic blocks.

Handles branch delay slots, branch-likely, prologue/epilogue removal, unaligned load/store pair merging and
jump-table (switch) recovery.

Statement forms (lists, so later passes may annotate them):
  ['set', reg, expr]
  ['st', kind, addr, val]
  ['call', target|None, indirect_expr|None]        (+ annotated .args/.rets later through a side dict)
  ['trap', what]
Terminators:
  ['br', cond, T, F]  ['jmp', T]  ['ret']  ['sw', idx_expr, [targets]]  ['tail', target|None, indirect_expr|None]
"""
import struct
from ps2elf import Elf
from r5900 import decode, GPR
from .ir import *

GP_VALUE = 0x3b47f0   # patched at load from the _gp symbol's value


class Unsupported(Exception):
    pass


class Block:
    __slots__ = ('addr', 'stmts', 'term', 'preds', 'succs', 'order')

    def __init__(self, addr):
        self.addr = addr
        self.stmts = []
        self.term = None
        self.preds = []
        self.succs = []

    def __repr__(self):
        return f'Block({self.addr:#x})'


class Func:
    def __init__(self, sym):
        self.name = sym.name
        self.addr = sym.addr
        self.size = sym.size
        self.blocks = {}
        self.entry = sym.addr
        self.frame = 0
        self.uses_frame = False
        self.manual = None


CALLEE_SAVED_G = set(range(16, 24)) | {30, 31}
CALLEE_SAVED_F = set(range(20, 32))
LIKELY_OPS = {'beql', 'bnel', 'blezl', 'bgtzl', 'bltzl', 'bgezl', 'bc1fl', 'bc1tl', 'bc0fl', 'bc0tl', 'bc2fl', 'bc2tl'}
COND_BRANCH = {'beq', 'bne', 'blez', 'bgtz', 'bltz', 'bgez', 'beql', 'bnel', 'blezl', 'bgtzl', 'bltzl', 'bgezl',
               'bc1f', 'bc1t', 'bc1fl', 'bc1tl'}
STORE_OPS = {'sb': 'b', 'sh': 'h', 'sw': 'w', 'sd': 'd', 'sq': 'q', 'swc1': 'f'}
LOAD_OPS = {'lb': 'b', 'lbu': 'bu', 'lh': 'h', 'lhu': 'hu', 'lw': 'w', 'lwu': 'wu', 'ld': 'd', 'lq': 'q', 'lwc1': 'f'}
UNALIGNED = {'ldl': ('ld', 7, 'l'), 'ldr': ('ld', 7, 'r'), 'lwl': ('lw', 3, 'l'), 'lwr': ('lw', 3, 'r'),
             'sdl': ('sd', 7, 'l'), 'sdr': ('sd', 7, 'r'), 'swl': ('sw', 3, 'l'), 'swr': ('sw', 3, 'r')}


def R(n):
    if n == 0:
        return ZERO
    if n == 28:
        return C(GP_VALUE)
    if n == 29:
        return ('sp', 0)
    return ('r', f'r{n}')


def F(n):
    return ('r', f'f{n}')


def W(n, e):
    if n == 0:
        return []
    if n == 29 or n == 28:
        raise Unsupported(f'write to {GPR[n]}')
    return [['set', f'r{n}', e]]


def FW(n, e):
    return [['set', f'f{n}', e]]


def frame_info(ins):
    """Return (skip addresses, frame size) for prologue/epilogue pieces."""
    skip = set()
    size = 0
    for i in ins:
        if i.op in ('addiu', 'daddiu') and i.rt == 29 and i.rs == 29:
            skip.add(i.addr)
            if i.imm < 0 and size == 0:
                size = -i.imm
    saves = set()
    written = set()
    stop_after = None
    for k, i in enumerate(ins):
        if stop_after is not None and k > stop_after:
            break
        if i.is_branch and i.op not in ('jal', 'jalr'):
            stop_after = k + 1          # still scan the delay slot
        if i.op in ('sd', 'sq', 'sw') and i.rs == 29 and i.rt in CALLEE_SAVED_G and ('g', i.rt) not in written:
            saves.add(('g', i.rt, i.imm)); skip.add(i.addr)
        elif i.op == 'swc1' and i.rs == 29 and i.rt in CALLEE_SAVED_F and ('f', i.rt) not in written:
            saves.add(('f', i.rt, i.imm)); skip.add(i.addr)
        # track writes to callee-saved regs
        if i.op in LOAD_OPS or i.op in ('addiu', 'daddu', 'addu', 'or', 'lui', 'ori', 'daddiu', 'sll', 'move'):
            if i.op in ('lwc1',):
                written.add(('f', i.rt))
            else:
                d = i.rt if i.op in LOAD_OPS or i.op in ('addiu', 'lui', 'ori', 'daddiu') else i.rd
                written.add(('g', d))
    for i in ins:
        if i.rs == 29:
            if i.op in ('ld', 'lq', 'lw') and ('g', i.rt, i.imm) in saves:
                skip.add(i.addr)
            elif i.op == 'lwc1' and ('f', i.rt, i.imm) in saves:
                skip.add(i.addr)
    return skip, size


def sym_for_ins(ins, k):
    return ins[k]


def find_def(ins, reg, before, window=40):
    """Index of the closest instruction before `before` writing GPR `reg`."""
    for k in range(before - 1, max(-1, before - window), -1):
        i = ins[k]
        o = i.op
        d = None
        if o in ('addiu', 'daddiu', 'addi', 'slti', 'sltiu', 'andi', 'ori', 'xori', 'lui') or o in LOAD_OPS and o != 'lwc1':
            d = i.rt
        elif o in ('addu', 'daddu', 'or', 'and', 'xor', 'nor', 'subu', 'dsubu', 'sll', 'srl', 'sra', 'sllv', 'srlv',
                   'srav', 'slt', 'sltu', 'mflo', 'mfhi', 'movz', 'movn', 'add', 'sub', 'dsll', 'dsrl', 'dsra',
                   'dsll32', 'mult', 'multu'):
            d = i.rd
        elif o == 'jal' or o == 'jalr':
            d = 2 if reg in (2, 3) else None
        if d == reg:
            return k
    return None


def const_of(ins, reg, before, depth=0):
    """Try to evaluate GPR value as a constant (lui/addiu/ori chains)."""
    if reg == 0:
        return 0
    if depth > 6:
        return None
    k = find_def(ins, reg, before, window=4000)
    if k is None:
        return None
    i = ins[k]
    if i.op == 'lui':
        return sx32(i.uimm << 16)
    if i.op in ('addiu', 'daddiu'):
        b = const_of(ins, i.rs, k, depth + 1)
        return None if b is None else sx32(b + i.imm)
    if i.op == 'ori':
        b = const_of(ins, i.rs, k, depth + 1)
        return None if b is None else sx32(b | i.uimm)
    if i.op in ('addu', 'daddu'):
        a = const_of(ins, i.rs, k, depth + 1)
        b = const_of(ins, i.rt, k, depth + 1)
        return None if a is None or b is None else sx32(a + b)
    return None


def find_jump_tables(ins, ram, faddr, fsize):
    """Return dict jr_addr -> (n, base, load_addr, load_off, load_base_reg)."""
    res = {}
    for k, i in enumerate(ins):
        if i.op != 'jr' or i.rs == 31:
            continue
        lk = find_def(ins, i.rs, k)
        if lk is None or ins[lk].op != 'lw':
            continue
        ld = ins[lk]
        # address register = ld.rs + ld.imm ; expect addu rA, rB
        ak = find_def(ins, ld.rs, lk)
        if ak is None or ins[ak].op not in ('addu', 'daddu'):
            continue
        a = ins[ak]
        base = None
        idxreg = None
        for x, y in ((a.rs, a.rt), (a.rt, a.rs)):
            c = const_of(ins, y, ak)
            if c is not None:
                sk = find_def(ins, x, ak)
                if sk is not None and ins[sk].op == 'sll' and ins[sk].sa == 2:
                    base, idxreg = c, ins[sk].rt
                    break
        if base is None:
            continue
        base = sx32(base + ld.imm)
        # bound: look for sltiu t, idxreg, N shortly before the sll
        n = None
        for j in range(ak, max(-1, ak - 40), -1):
            if ins[j].op == 'sltiu' and ins[j].rs == idxreg:
                n = ins[j].imm & 0xFFFF
                break
        targets = []
        off = base
        while n is None or len(targets) < n:
            t = struct.unpack_from('<I', ram, off)[0]
            if not (faddr <= t < faddr + fsize) or t & 3:
                break
            targets.append(t)
            off += 4
            if len(targets) > 512:
                break
        if not targets:
            continue
        res[i.addr] = (targets, base, ld.addr, ld.imm, ld.rs)
    return res


def dest_gpr(i):
    o = i.op
    if o in ('addiu', 'daddiu', 'addi', 'slti', 'sltiu', 'andi', 'ori', 'xori', 'lui', 'mfc1') or             (o in LOAD_OPS and o != 'lwc1') or o in ('ldl', 'ldr', 'lwl', 'lwr'):
        return i.rt
    if o in ('jal', 'jalr'):
        return -2          # clobbers many
    if o in ('sb', 'sh', 'sw', 'sd', 'sq', 'swc1', 'sdl', 'sdr', 'swl', 'swr', 'nop') or i.is_branch:
        return -1
    return i.rd


def merge_unaligned(ins, skip):
    """Merge ldl/ldr, lwl/lwr, sdl/sdr, swl/swr pairs into plain accesses.  Returns dict addr->replacement
    (op, rt, rs, imm) and adds the partner instructions to skip."""
    repl = {}
    by_addr = {i.addr: i for i in ins}
    for k, i in enumerate(ins):
        if i.op in UNALIGNED and i.addr not in skip and i.addr not in repl:
            base_op, span, side = UNALIGNED[i.op]
            # partner: next or previous instruction with the other side, same rt/rs
            for j in [k + d for d in (1, -1, 2, -2, 3, -3, 4, -4, 5, -5, 6, -6, 7, -7, 8, -8)]:
                if 0 <= j < len(ins):
                    p = ins[j]
                    if p.op in UNALIGNED and UNALIGNED[p.op][0] == base_op and UNALIGNED[p.op][2] != side \
                            and p.rt == i.rt and p.rs == i.rs and p.addr not in skip and p.addr not in repl:
                        lo = min(i.imm, p.imm)
                        hi = max(i.imm, p.imm)
                        a_, b_ = min(j, k), max(j, k)
                        between_ok = all(dest_gpr(ins[m]) not in (i.rt, i.rs, -2) or
                                         (m == b_ - 1 and ins[m].op in ('jal', 'jalr'))
                                         for m in range(a_ + 1, b_) if ins[m].addr not in skip)
                        if hi - lo == span and between_ok:
                            # the merged op executes at the position of the later of the two
                            later = i if i.addr > p.addr else p
                            earlier = p if later is i else i
                            repl[later.addr] = (base_op, i.rt, i.rs, lo)
                            skip.add(earlier.addr)
                            break
    return repl


def lift_ins(i, repl):
    """Lift one non-control instruction to a list of statements."""
    o = i.op
    if i.addr in repl:
        bo, rt, rs, imm = repl[i.addr]
        a = mk('add32', R(rs), C(imm))
        if bo in ('ld', 'lw'):
            return W(rt, ('ld', 'd' if bo == 'ld' else 'w', a))
        return [['st', 'd' if bo == 'sd' else 'w', a, R(rt)]]
    rs, rt, rd, sa, imm = i.rs, i.rt, i.rd, i.sa, i.imm
    uimm = i.uimm
    if o == 'nop' or o in ('sync', 'cache', 'pref', 'ei', 'di'):
        return []
    if o in ('addiu', 'addi'):
        return W(rt, mk('add32', R(rs), C(imm)))
    if o in ('daddiu', 'daddi'):
        return W(rt, mk('add', R(rs), C(imm)))
    if o in ('addu', 'add'):
        return W(rd, mk('add32', R(rs), R(rt)))
    if o in ('daddu', 'dadd'):
        return W(rd, mk('add', R(rs), R(rt)))
    if o in ('subu', 'sub'):
        return W(rd, mk('sub32', R(rs), R(rt)))
    if o in ('dsubu', 'dsub'):
        return W(rd, mk('sub', R(rs), R(rt)))
    if o in ('and', 'or', 'xor', 'nor'):
        return W(rd, mk(o, R(rs), R(rt)))
    if o == 'andi':
        return W(rt, mk('and', R(rs), C(uimm)))
    if o == 'ori':
        return W(rt, mk('or', R(rs), C(uimm)))
    if o == 'xori':
        return W(rt, mk('xor', R(rs), C(uimm)))
    if o == 'lui':
        return W(rt, C(uimm << 16))
    if o == 'slt':
        return W(rd, ('b2i', mk('lt', R(rs), R(rt))))
    if o == 'sltu':
        return W(rd, ('b2i', mk('ltu', R(rs), R(rt))))
    if o == 'slti':
        return W(rt, ('b2i', mk('lt', R(rs), C(imm))))
    if o == 'sltiu':
        return W(rt, ('b2i', mk('ltu', R(rs), C(imm))))
    if o == 'sll':
        return W(rd, mk('shl32', R(rt), C(sa)) if sa else mk('add32', R(rt), ZERO))
    if o == 'srl':
        return W(rd, mk('shr32', R(rt), C(sa)) if sa else mk('add32', R(rt), ZERO))
    if o == 'sra':
        return W(rd, mk('sar32', R(rt), C(sa)) if sa else mk('add32', R(rt), ZERO))
    if o == 'sllv':
        return W(rd, mk('shl32', R(rt), mk('and', R(rs), C(31))))
    if o == 'srlv':
        return W(rd, mk('shr32', R(rt), mk('and', R(rs), C(31))))
    if o == 'srav':
        return W(rd, mk('sar32', R(rt), mk('and', R(rs), C(31))))
    if o in ('dsll', 'dsll32'):
        return W(rd, mk('shl64', R(rt), C(sa + (32 if o == 'dsll32' else 0))))
    if o in ('dsrl', 'dsrl32'):
        return W(rd, mk('shr64', R(rt), C(sa + (32 if o == 'dsrl32' else 0))))
    if o in ('dsra', 'dsra32'):
        return W(rd, mk('sar64', R(rt), C(sa + (32 if o == 'dsra32' else 0))))
    if o == 'dsllv':
        return W(rd, mk('shl64', R(rt), mk('and', R(rs), C(63))))
    if o == 'dsrlv':
        return W(rd, mk('shr64', R(rt), mk('and', R(rs), C(63))))
    if o == 'dsrav':
        return W(rd, mk('sar64', R(rt), mk('and', R(rs), C(63))))
    if o in ('mult', 'multu'):
        hi = 'mulhi' if o == 'mult' else 'mulhiu'
        out = [['set', 'lo', mk('mul32', R(rs), R(rt))], ['set', 'hi', (hi, R(rs), R(rt))]]
        if rd:
            out += W(rd, ('r', 'lo'))
        return out
    if o in ('mult1', 'multu1'):
        hi = 'mulhi' if o == 'mult1' else 'mulhiu'
        out = [['set', 'lo1', mk('mul32', R(rs), R(rt))], ['set', 'hi1', (hi, R(rs), R(rt))]]
        if rd:
            out += W(rd, ('r', 'lo1'))
        return out
    if o in ('div', 'divu', 'div1', 'divu1'):
        sfx = '1' if o.endswith('1') else ''
        u = 'u' if o.startswith('divu') else ''
        return [['set', 'lo' + sfx, (f'div{u}32'.replace('divu32', 'divu32'), R(rs), R(rt))],
                ['set', 'hi' + sfx, (f'rem{u}32', R(rs), R(rt))]]
    if o in ('madd', 'maddu', 'madd1', 'maddu1'):
        sfx = '1' if o.endswith('1') else ''
        u = 'u' if o.startswith('maddu') else ''
        lo, hi = ('r', 'lo' + sfx), ('r', 'hi' + sfx)
        out = [['set', 'hi' + sfx, ('maddhi' + u, hi, lo, R(rs), R(rt))],
               ['set', 'lo' + sfx, mk('add32', lo, mk('mul32', R(rs), R(rt)))]]
        if rd:
            out += W(rd, ('r', 'lo' + sfx))
        return out
    if o in ('mfhi', 'mflo', 'mfhi1', 'mflo1'):
        return W(rd, ('r', o[2:]))
    if o in ('mthi', 'mtlo', 'mthi1', 'mtlo1'):
        return [['set', o[2:], R(rs)]]
    if o == 'movz':
        return W(rd, ('sel', mk('eq', R(rt), ZERO), R(rs), R(rd)))
    if o == 'movn':
        return W(rd, ('sel', mk('ne', R(rt), ZERO), R(rs), R(rd)))
    if o in LOAD_OPS:
        a = mk('add32', R(rs), C(imm))
        e = ('ld', LOAD_OPS[o], a)
        return FW(rt, e) if o == 'lwc1' else W(rt, e)
    if o in STORE_OPS:
        a = mk('add32', R(rs), C(imm))
        v = F(rt) if o == 'swc1' else R(rt)
        return [['st', STORE_OPS[o], a, v]]
    if o == 'break':
        return [['trap', 'break']]
    if o == 'syscall':
        return [['trap', 'syscall']]
    # ---- COP1
    if o == 'mfc1':
        return W(rt, ('mfc1', F(rd)))
    if o == 'mtc1':
        return FW(rd, ('mtc1', R(rt)))
    if o == 'cfc1':
        return W(rt, C(0))
    if o == 'ctc1':
        return []
    fs, ft, fd = rd, rt, sa
    if o in ('add.s', 'sub.s', 'mul.s', 'div.s'):
        return FW(fd, ({'add.s': 'fadd', 'sub.s': 'fsub', 'mul.s': 'fmul', 'div.s': 'fdiv'}[o], F(fs), F(ft)))
    if o == 'sqrt.s':
        return FW(fd, ('fsqrt', F(ft)))
    if o == 'rsqrt.s':
        return FW(fd, ('frsqrt', F(fs), F(ft)))
    if o == 'abs.s':
        return FW(fd, ('fabs', F(fs)))
    if o == 'neg.s':
        return FW(fd, ('fneg', F(fs)))
    if o == 'mov.s':
        return FW(fd, F(fs))
    if o == 'max.s':
        return FW(fd, ('fmax', F(fs), F(ft)))
    if o == 'min.s':
        return FW(fd, ('fmin', F(fs), F(ft)))
    if o == 'cvt.s.w':
        return FW(fd, ('w2f', ('mfc1', F(fs))))
    if o == 'cvt.w.s':
        return FW(fd, ('mtc1', ('f2w', F(fs))))
    if o == 'c.lt.s':
        return [['set', 'fcc', ('flt', F(fs), F(ft))]]
    if o == 'c.le.s':
        return [['set', 'fcc', ('fle', F(fs), F(ft))]]
    if o == 'c.eq.s':
        return [['set', 'fcc', ('feq', F(fs), F(ft))]]
    if o == 'c.f.s':
        return [['set', 'fcc', ('cb', False)]]
    if o == 'adda.s':
        return [['set', 'acc', ('fadd', F(fs), F(ft))]]
    if o == 'suba.s':
        return [['set', 'acc', ('fsub', F(fs), F(ft))]]
    if o == 'mula.s':
        return [['set', 'acc', ('fmul', F(fs), F(ft))]]
    if o == 'madda.s':
        return [['set', 'acc', ('fadd', ('r', 'acc'), ('fmul', F(fs), F(ft)))]]
    if o == 'msuba.s':
        return [['set', 'acc', ('fsub', ('r', 'acc'), ('fmul', F(fs), F(ft)))]]
    if o == 'madd.s':
        return FW(fd, ('fadd', ('r', 'acc'), ('fmul', F(fs), F(ft))))
    if o == 'msub.s':
        return FW(fd, ('fsub', ('r', 'acc'), ('fmul', F(fs), F(ft))))
    # ---- MMI / VU0 -> vector intrinsics (u128)
    from r5900 import MMI, MMI0, MMI1, MMI2, MMI3
    mmi = set(MMI.values()) | set(MMI0.values()) | set(MMI1.values()) | set(MMI2.values()) | set(MMI3.values())
    if o in mmi:
        if o in ('mfhi1', 'mflo1', 'mthi1', 'mtlo1'):
            raise Unsupported(o)
        if o in ('psllh', 'psrlh', 'psrah', 'psllw', 'psrlw', 'psraw'):
            return W(rd, ('vop', o, 'u128', R(rt), C(sa)))
        if o in ('pabsw', 'pabsh', 'pexeh', 'prevh', 'pexew', 'prot3w', 'pexch', 'pcpyh', 'pexcw', 'ppac5', 'pext5',
                 'plzcw'):
            return W(rd, ('vop', o, 'u128', R(rt if o != 'plzcw' else rs)))
        if o in ('pmfhl', 'pmthl', 'pmfhi', 'pmflo', 'pmthi', 'pmtlo', 'pmadduw',
                 'pmaddw', 'pmultw', 'pmultuw', 'pmaddh', 'phmadh', 'pmsubh', 'phmsbh', 'pmulth', 'pdivw',
                 'pdivuw', 'pdivbw', 'pmsubw', 'qfsrv'):
            raise Unsupported(o)
        return W(rd, ('vop', o, 'u128', R(rs), R(rt)))
    raise Unsupported(o)


def branch_cond(i):
    o = i.op
    rs, rt = i.rs, i.rt
    base = o.rstrip('l') if o.endswith('l') and o not in ('bltz', 'bgez') else o
    if o in ('beq', 'beql'):
        return mk('eq', R(rs), R(rt))
    if o in ('bne', 'bnel'):
        return mk('ne', R(rs), R(rt))
    if o in ('blez', 'blezl'):
        return mk('le', R(rs), ZERO)
    if o in ('bgtz', 'bgtzl'):
        return mk('gt', R(rs), ZERO)
    if o in ('bltz', 'bltzl'):
        return mk('lt', R(rs), ZERO)
    if o in ('bgez', 'bgezl'):
        return mk('ge', R(rs), ZERO)
    if o in ('bc1t', 'bc1tl'):
        return ('r', 'fcc')
    if o in ('bc1f', 'bc1fl'):
        return ('not', ('r', 'fcc'))
    raise Unsupported(o)


def lift_function(sym, ram, funcs_by_addr):
    f = Func(sym)
    n = sym.size // 4
    ins = [decode(sym.addr + 4 * k, struct.unpack_from('<I', ram, sym.addr + 4 * k)[0]) for k in range(n)]
    if not ins:
        raise Unsupported('empty')
    skip, f.frame = frame_info(ins)
    jt = find_jump_tables(ins, ram, sym.addr, sym.size)
    repl = merge_unaligned(ins, skip)
    end = sym.addr + sym.size
    inside = lambda a: sym.addr <= a < end

    for i in ins:
        if i.op in ('lqc2', 'sqc2', 'qmfc2', 'qmtc2', 'cfc2', 'ctc2', 'bc2f', 'bc2t', 'bc2fl', 'bc2tl') or \
                (i.fmt == 'vu'):
            raise Unsupported('vu0')

    def is_ctl(i):
        return i.op in ('j', 'jal', 'jr', 'jalr') or i.op in COND_BRANCH or i.op in ('bltzal', 'bgezal', 'bltzall',
                                                                                     'bgezall')

    leaders = {sym.addr}
    for k, i in enumerate(ins):
        if not is_ctl(i):
            continue
        if k + 2 <= n and inside(i.addr + 8):
            leaders.add(i.addr + 8)
        if i.op in COND_BRANCH or i.op == 'j':
            t = i.branch_target
            if inside(t):
                leaders.add(t)
        if i.op == 'jr' and i.addr in jt:
            for t in jt[i.addr][0]:
                leaders.add(t)

    idx_of = lambda a: (a - sym.addr) // 4
    sorted_leaders = sorted(leaders)

    def lift_slot(k):
        """Lift the instruction in the delay slot at index k."""
        if k >= n:
            return []
        d = ins[k]
        if d.addr in skip:
            return []
        if is_ctl(d):
            raise Unsupported('branch in delay slot')
        return lift_ins(d, repl)

    for L in sorted_leaders:
        b = Block(L)
        f.blocks[L] = b
        k = idx_of(L)
        while True:
            i = ins[k]
            if i.addr in skip:
                k += 1
            elif not is_ctl(i):
                st = None
                for jr_addr, (targets, base, ld_addr, ld_off, ld_base) in jt.items():
                    if ld_addr == i.addr:
                        # ld_base holds (table address - ld_off) + idx*4
                        st = [['set', 'swidx', mk('sar32', mk('add32', R(ld_base), C(ld_off - base)), C(2))]]
                if st is not None:
                    b.stmts += st
                else:
                    b.stmts += lift_ins(i, repl)
                k += 1
            else:
                o = i.op
                dk = k + 1
                if o == 'jal':
                    b.stmts += lift_slot(dk)
                    b.stmts.append(['call', i.branch_target, None])
                    k += 2
                elif o == 'jalr':
                    ind = R(i.rs)
                    tmp = None
                    slot = lift_slot(dk)
                    if any(s[0] == 'set' and s[1] == f'r{i.rs}' for s in slot):
                        b.stmts.append(['set', 'jt', ind])
                        ind = ('r', 'jt')
                    b.stmts += slot
                    b.stmts.append(['call', None, ind])
                    k += 2
                elif o == 'j':
                    b.stmts += lift_slot(dk)
                    t = i.branch_target
                    b.term = ['jmp', t] if inside(t) else ['tail', t, None]
                    break
                elif o == 'jr':
                    b.stmts += lift_slot(dk)
                    if i.rs == 31:
                        b.term = ['ret']
                    elif i.addr in jt:
                        b.term = ['sw', ('r', 'swidx'), list(jt[i.addr][0])]
                    else:
                        b.term = ['tail', None, R(i.rs)]
                    break
                elif o in COND_BRANCH:
                    cond = branch_cond(i)
                    t = i.branch_target
                    fall = i.addr + 8
                    likely = o in LIKELY_OPS
                    uncond = o == 'beq' and i.rs == i.rt
                    if not inside(t):
                        if uncond:
                            b.stmts += lift_slot(dk)
                            b.term = ['tail', t, None]
                            break
                        raise Unsupported('branch out of function')
                    if uncond:
                        b.stmts += lift_slot(dk)
                        b.term = ['jmp', t]
                        break
                    if likely:
                        sk = i.addr + 4 + (1 << 40)
                        sb = Block(sk)
                        sb.stmts = lift_slot(dk)
                        sb.term = ['jmp', t]
                        f.blocks[sk] = sb
                        b.term = ['br', cond, sk, fall]
                        break
                    slot = lift_slot(dk)
                    regs = set(regs_in(cond))
                    clob = {s[1] for s in slot if s[0] == 'set'}
                    if regs & clob:
                        b.stmts.append(['set', 'bc', cond])
                        cond = ('r', 'bc')
                    b.stmts += slot
                    b.term = ['br', cond, t, fall]
                    break
                else:
                    raise Unsupported(o)
            if k >= n:
                b.term = ['ret']
                break
            if ins[k].addr in leaders:
                b.term = ['jmp', ins[k].addr]
                break
    # tail-call 'jal' immediately followed by return?  (kept as call + ret)
    return f


def jt_loads(jt):
    return {v[2] for v in jt.values()}
