"""Debug printer for analysed IR (pseudo-code, not Rust)."""
from .ir import *

def fe(e, webs):
    t = e[0]
    if t == 'c': return f'{e[1]:#x}' if abs(e[1]) > 255 else str(e[1])
    if t == 'cf': return repr(round(bits2f(e[1]), 6)) + 'f'
    if t == 'cb': return str(e[1])
    if t == 'sp': return f'sp+{e[1]}' if e[1] else 'sp'
    if t == 'v':
        w = webs[e[1]]
        return f'{w.reg}#{e[1] }'
    if t == 'r': return e[1]
    if t == 'ld': return f'ld.{e[1]}[{fe(e[2], webs)}]'
    if t == 'vop': return f'{e[1]}(' + ', '.join(fe(x, webs) for x in e[3:]) + ')'
    return f'{t}(' + ', '.join(fe(x, webs) if isinstance(x, tuple) else str(x) for x in e[1:]) + ')'

def dump(func, sigs=None, out=None):
    import sys
    out = out or sys.stdout
    webs = getattr(func, 'webs', None)
    out.write(f'fn {func.name} @ {func.addr:#x}  frame={func.frame}\n')
    for a in func.rpo:
        b = func.blocks[a]
        out.write(f' B{a & 0xffffff:x}:  preds={[hex(p & 0xffffff) for p in b.preds]}\n')
        for s in b.stmts:
            out.write('    ' + fs(s, webs, sigs) + '\n')
        out.write('    ' + fs(b.term, webs, sigs) + '\n')

def fs(s, webs, sigs):
    k = s.k
    if k == 'set':
        d = f'{webs[s.dweb].reg}#{s.dweb }' if webs else s.dst
        return f'{d} = {fe(s.e, webs)}'
    if k == 'st': return f'st.{s.sk}[{fe(s.a, webs)}] = {fe(s.v, webs)}'
    if k == 'call':
        tgt = s.tgt if s.tgt is not None else fe(s.ind, webs)
        name = sigs[s.tgt].name if sigs and s.tgt in sigs else (hex(tgt) if isinstance(tgt, int) else tgt)
        args = ', '.join(f'{webs[w].reg}#{w }' for r, w in (s.argwebs or []))
        rets = ', '.join(f'{r}#{w }' for r, w in (s.retwebs or []))
        return f'{("(" + rets + ") = ") if rets else ""}call {name}({args})'
    if k == 'trap': return f'trap {s.what}'
    if k == 'br': return f'br {fe(s.cond, webs)} ? B{s.T & 0xffffff:x} : B{s.F & 0xffffff:x}'
    if k == 'jmp': return f'jmp B{s.T & 0xffffff:x}'
    if k == 'ret': return 'ret ' + ', '.join(f'{webs[w].reg}#{w }' for r, w in (s.argwebs or []))
    if k == 'sw': return f'switch {fe(s.idx, webs)} -> ' + ', '.join(f'B{t & 0xffffff:x}' for t in s.targets)
    if k == 'tail': return f'tail {s.tgt if s.tgt is not None else fe(s.ind, webs)}'
    return k
