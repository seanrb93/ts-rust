"""Expression IR for the TimeSplitters decompiler.

Expressions are immutable tuples:
  ('c', value, ty)        integer constant, ty in i32|i64
  ('cf', bits)            f32 constant (raw bits)
  ('cb', bool)            boolean constant
  ('r', 'r4')             register read (before web recovery)
  ('v', web_id)           variable read (after web recovery)
  ('sp', off)             address of the current stack frame + off
  ('ld', kind, addr)      memory load
  (op, a, b, ...)         operation
"""
import struct

M32 = 0xFFFFFFFF
M64 = 0xFFFFFFFFFFFFFFFF


def sx32(v):
    v &= M32
    return v - (1 << 32) if v & 0x80000000 else v


def sx64(v):
    v &= M64
    return v - (1 << 64) if v & (1 << 63) else v


def f2bits(f):
    return struct.unpack('<I', struct.pack('<f', f))[0]


def bits2f(b):
    return struct.unpack('<f', struct.pack('<I', b & M32))[0]


# result types of each op.  'poly' = i64 if either operand is i64 else i32
RES = {
    # 32-bit integer ops
    'add32': 'i32', 'sub32': 'i32', 'mul32': 'i32', 'mulhi': 'i32', 'mulhiu': 'i32', 'div32': 'i32', 'rem32': 'i32',
    'divu32': 'i32', 'remu32': 'i32', 'shl32': 'i32', 'shr32': 'i32', 'sar32': 'i32', 'clz': 'i32',
    'b2i': 'i32', 'f2w': 'i32', 'mfc1': 'i32',
    # polymorphic (64-bit capable)
    'add': 'poly', 'sub': 'poly', 'and': 'poly', 'or': 'poly', 'xor': 'poly', 'nor': 'poly', 'sel': 'poly2',
    # 64-bit
    'shl64': 'i64', 'shr64': 'i64', 'sar64': 'i64', 'mul64': 'i64', 'div64': 'i64', 'rem64': 'i64',
    # float
    'fadd': 'f32', 'fsub': 'f32', 'fmul': 'f32', 'fdiv': 'f32', 'fsqrt': 'f32', 'fabs': 'f32', 'fneg': 'f32',
    'fmax': 'f32', 'fmin': 'f32', 'frsqrt': 'f32', 'w2f': 'f32', 'mtc1': 'f32',
    # conditions
    'eq': 'bool', 'ne': 'bool', 'lt': 'bool', 'le': 'bool', 'gt': 'bool', 'ge': 'bool', 'ltu': 'bool', 'geu': 'bool',
    'leu': 'bool', 'gtu': 'bool', 'not': 'bool', 'feq': 'bool', 'flt': 'bool', 'fle': 'bool',
    'band': 'bool', 'bor': 'bool',
    'maddhi': 'i32', 'maddhiu': 'i32',
}

LOAD_TY = {'b': 'i32', 'bu': 'i32', 'h': 'i32', 'hu': 'i32', 'w': 'i32', 'wu': 'i64', 'd': 'i64', 'f': 'f32',
           'q': 'u128'}


def C(v, ty='i32'):
    return ('c', sx32(v) if ty == 'i32' else sx64(v), ty)


def CF(f):
    return ('cf', f2bits(f) if not isinstance(f, int) else f & M32)


ZERO = C(0)
LEAVES = ('c', 'cf', 'r', 'v', 'sp', 'cb')


def is_c(e):
    return e[0] == 'c'


def ty(e, vt=None):
    """Result type of expression e.  vt: callable web_id -> type for ('v', id) leaves."""
    t = e[0]
    if t == 'c':
        return e[2]
    if t == 'cf':
        return 'f32'
    if t == 'cb':
        return 'bool'
    if t == 'sp':
        return 'i32'
    if t == 'ld':
        return LOAD_TY[e[1]]
    if t == 'v':
        return vt(e[1]) if vt else 'i32'
    if t == 'r':
        return 'i32'
    if t == 'vop':
        return e[2]
    r = RES.get(t)
    if r is None:
        raise KeyError(t)
    if r == 'poly':
        a, b = ty(e[1], vt), ty(e[2], vt)
        return 'u128' if 'u128' in (a, b) else ('i64' if 'i64' in (a, b) else 'i32')
    if r == 'poly2':
        a, b = ty(e[2], vt), ty(e[3], vt)
        return 'u128' if 'u128' in (a, b) else ('i64' if 'i64' in (a, b) else 'i32')
    return r


def _fold_int(op, a, b):
    ta = 'i64' if a[2] == 'i64' else 'i32'
    tb = 'i64' if b[2] == 'i64' else 'i32'
    j = 'i64' if 'i64' in (ta, tb) else 'i32'
    x, y = a[1], b[1]
    if op in ('add32', 'sub32', 'mul32'):
        r = {'add32': x + y, 'sub32': x - y, 'mul32': x * y}[op]
        return C(r, 'i32')
    r = {'add': x + y, 'sub': x - y, 'and': x & y, 'or': x | y, 'xor': x ^ y, 'nor': ~(x | y)}.get(op)
    if r is not None:
        return C(r, j)
    if op == 'shl32':
        return C(x << (y & 31), 'i32')
    if op == 'shr32':
        return C((x & M32) >> (y & 31), 'i32')
    if op == 'sar32':
        return C(sx32(x) >> (y & 31), 'i32')
    if op == 'shl64':
        return C(x << (y & 63), 'i64')
    if op == 'shr64':
        return C((x & M64) >> (y & 63), 'i64')
    if op == 'sar64':
        return C(x >> (y & 63), 'i64')
    return None


CMP_NEG = {'eq': 'ne', 'ne': 'eq', 'lt': 'ge', 'ge': 'lt', 'le': 'gt', 'gt': 'le', 'ltu': 'geu', 'geu': 'ltu',
           'leu': 'gtu', 'gtu': 'leu'}
INT_BIN = ('add32', 'sub32', 'mul32', 'add', 'sub', 'and', 'or', 'xor', 'nor', 'shl32', 'shr32', 'sar32',
           'shl64', 'shr64', 'sar64')


def mk(op, *a):
    """Build an op node with algebraic simplification."""
    if op in INT_BIN:
        x, y = a
        if is_c(x) and is_c(y):
            r = _fold_int(op, x, y)
            if r:
                return r
        if op in ('add32', 'add'):
            if is_c(x) and not is_c(y):
                x, y = y, x  # constants on the right
            if is_c(y) and y[1] == 0:
                return x if (op == 'add' or ty(x) == 'i32') else ('add32', x, y)
            if x[0] == 'sp' and is_c(y):
                return ('sp', x[1] + y[1])
            if is_c(y) and x[0] == op and is_c(x[2]):
                return mk(op, x[1], _fold_int(op, x[2], y))
        elif op in ('sub32', 'sub'):
            if is_c(y):
                return mk('add32' if op == 'sub32' else 'add', x, C(-y[1], y[2]))
        elif op in ('or', 'xor', 'and'):
            if is_c(x) and not is_c(y):
                x, y = y, x
            if op != 'and' and is_c(y) and y[1] == 0:
                return x
        elif op in ('shl32', 'shr32', 'sar32', 'shl64', 'shr64', 'sar64'):
            if is_c(y) and y[1] == 0:
                return x
        return (op, x, y)
    if op == 'not':
        (c,) = a
        if c[0] == 'cb':
            return ('cb', not c[1])
        if c[0] == 'not':
            return c[1]
        if c[0] in CMP_NEG:
            return (CMP_NEG[c[0]], c[1], c[2])
        return ('not', c)
    if op in CMP_NEG:
        x, y = a
        if is_c(x) and is_c(y):
            xv, yv = x[1], y[1]
            if x[2] == 'i32' and y[2] == 'i32':
                ux, uy = xv & M32, yv & M32
            else:
                ux, uy = xv & M64, yv & M64
            r = {'eq': xv == yv, 'ne': xv != yv, 'lt': xv < yv, 'ge': xv >= yv, 'le': xv <= yv, 'gt': xv > yv,
                 'ltu': ux < uy, 'geu': ux >= uy, 'leu': ux <= uy, 'gtu': ux > uy}[op]
            return ('cb', r)
        if op in ('eq', 'ne') and is_c(y) and y[1] == 0 and x[0] == 'b2i':
            return x[1] if op == 'ne' else mk('not', x[1])
        return (op, x, y)
    # ---- float/int bit-pattern plumbing
    if op == 'mfc1':
        (x,) = a
        if x[0] == 'mtc1':
            return x[1]
        if x[0] == 'cf':
            return C(x[1], 'i32')
    elif op == 'mtc1':
        (x,) = a
        if x[0] == 'mfc1':
            return x[1]
        if is_c(x):
            return ('cf', x[1] & M32)
    elif op == 'w2f':
        (x,) = a
        if is_c(x):
            return CF(float(x[1]))
    elif op in ('feq', 'fle') and a[0] == a[1] and a[0][0] in ('r', 'v'):
        return ('cb', True)        # no NaNs on the PS2 FPU
    elif op == 'flt' and a[0] == a[1] and a[0][0] in ('r', 'v'):
        return ('cb', False)
    elif op == 'fneg':
        (x,) = a
        if x[0] == 'cf':
            return ('cf', x[1] ^ 0x80000000)
        if x[0] == 'fneg':
            return x[1]
    return (op,) + tuple(a)


def children(e):
    t = e[0]
    if t in LEAVES:
        return ()
    if t == 'ld':
        return (e[2],)
    if t == 'vop':
        return e[3:]
    return tuple(x for x in e[1:] if isinstance(x, tuple))


def walk(e):
    """Yield every sub-expression (pre-order)."""
    yield e
    for x in children(e):
        yield from walk(x)


def regs_in(e):
    return [x[1] for x in walk(e) if x[0] == 'r']


def subst(e, fn):
    """Rebuild e bottom-up, replacing leaves via fn(leaf)->expr|None."""
    t = e[0]
    if t in LEAVES:
        r = fn(e)
        return e if r is None else r
    if t == 'ld':
        return ('ld', e[1], subst(e[2], fn))
    if t == 'vop':
        return e[:3] + tuple(subst(x, fn) for x in e[3:])
    return mk(t, *[subst(x, fn) if isinstance(x, tuple) else x for x in e[1:]])
