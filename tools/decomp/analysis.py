"""Dataflow analyses over lifted functions: CFG, liveness, interprocedural signatures, def-use webs,
dead-code elimination, expression inlining and type inference."""
from collections import defaultdict
from .ir import *

ARG_G = [f'r{i}' for i in range(4, 12)]
ARG_F = [f'f{i}' for i in range(12, 20)]
RET_REGS = ['r2', 'r3', 'f0']
CLOBBER = ([f'r{i}' for i in list(range(1, 16)) + [24, 25]] + [f'f{i}' for i in range(0, 20)] +
           ['fcc', 'hi', 'lo', 'hi1', 'lo1', 'acc'])
CLOBBER_SET = set(CLOBBER)
ALL_REGS = [f'r{i}' for i in range(1, 32)] + [f'f{i}' for i in range(32)] + \
           ['fcc', 'hi', 'lo', 'hi1', 'lo1', 'acc', 'bc', 'jt', 'swidx']
RIDX = {r: i for i, r in enumerate(ALL_REGS)}


def mask(regs):
    m = 0
    for r in regs:
        m |= 1 << RIDX[r]
    return m


ARG_MASK = mask(ARG_G + ARG_F)
RET_MASK = mask(RET_REGS)
CLOBBER_MASK = mask(CLOBBER)


class Sig:
    def __init__(self, name, params=(), rets=(), extern=False, dynamic=False, retf=False):
        self.name = name
        self.retf = retf
        self.params = list(params)
        self.rets = list(rets)
        self.extern = extern
        self.dynamic = dynamic       # arguments determined from the call site (variadic / unknown)
        self.addr_taken = False

    def __repr__(self):
        return f'Sig({self.name},{self.params}->{self.rets}{" dyn" if self.dynamic else ""})'


class St:
    """One statement.  kind: set st call trap | br jmp ret sw tail"""
    __slots__ = ('k', 'dst', 'e', 'a', 'v', 'sk', 'tgt', 'ind', 'args', 'rets', 'cond', 'T', 'F', 'targets',
                 'what', 'idx', 'blk', 'pos', 'uweb', 'dweb', 'dead', 'sig', 'retwebs', 'argwebs', 'live_after')

    def __init__(self, k, **kw):
        self.k = k
        self.dst = self.e = self.a = self.v = self.sk = self.tgt = self.ind = None
        self.args = self.rets = self.cond = self.T = self.F = self.targets = self.what = self.idx = None
        self.blk = self.pos = None
        self.uweb = None
        self.dweb = None
        self.dead = False
        self.sig = None
        self.retwebs = None
        self.argwebs = None
        self.live_after = 0
        for a, b in kw.items():
            setattr(self, a, b)

    def exprs(self):
        k = self.k
        if k == 'set':
            return [self.e]
        if k == 'st':
            return [self.a, self.v]
        if k == 'call' or k == 'tail':
            return [self.ind] if self.ind is not None else []
        if k == 'br':
            return [self.cond]
        if k == 'sw':
            return [self.idx]
        return []

    def set_exprs(self, new):
        k = self.k
        if k == 'set':
            self.e = new[0]
        elif k == 'st':
            self.a, self.v = new
        elif k == 'call' or k == 'tail':
            if self.ind is not None:
                self.ind = new[0]
        elif k == 'br':
            self.cond = new[0]
        elif k == 'sw':
            self.idx = new[0]

    def __repr__(self):
        return f'St({self.k},{self.dst},{self.e})'


def convert(func):
    """Turn lifter list-statements into St objects."""
    for b in func.blocks.values():
        out = []
        for s in b.stmts:
            k = s[0]
            if k == 'set':
                out.append(St('set', dst=s[1], e=s[2]))
            elif k == 'st':
                out.append(St('st', sk=s[1], a=s[2], v=s[3]))
            elif k == 'call':
                out.append(St('call', tgt=s[1], ind=s[2]))
            elif k == 'trap':
                out.append(St('trap', what=s[1]))
        b.stmts = out
        t = b.term
        if t[0] == 'br':
            b.term = St('br', cond=t[1], T=t[2], F=t[3])
        elif t[0] == 'jmp':
            b.term = St('jmp', T=t[1])
        elif t[0] == 'ret':
            b.term = St('ret')
        elif t[0] == 'sw':
            b.term = St('sw', idx=t[1], targets=t[2])
        elif t[0] == 'tail':
            b.term = St('tail', tgt=t[1], ind=t[2])
        for p, s in enumerate(b.stmts):
            s.blk = b
            s.pos = p
        b.term.blk = b
        b.term.pos = len(b.stmts)


def build_cfg(func):
    blocks = func.blocks
    for b in blocks.values():
        b.preds = []
        b.succs = []
    seen = set()
    order = []
    stack = [(func.entry, 0)]
    # iterative DFS for reverse post-order
    adj = {}
    for a, b in blocks.items():
        t = b.term
        if t.k == 'br':
            s = [t.T, t.F]
        elif t.k == 'jmp':
            s = [t.T]
        elif t.k == 'sw':
            s = list(dict.fromkeys(t.targets))
        else:
            s = []
        adj[a] = s
    post = []
    visited = {func.entry}
    st = [(func.entry, iter(adj[func.entry]))]
    while st:
        n, it = st[-1]
        for m in it:
            if m not in visited:
                visited.add(m)
                st.append((m, iter(adj[m])))
                break
        else:
            post.append(n)
            st.pop()
    for a in list(blocks):
        if a not in visited:
            del blocks[a]
    for a, b in blocks.items():
        b.succs = [x for x in adj[a]]
        for x in set(b.succs):
            blocks[x].preds.append(a)
    func.rpo = list(reversed(post))
    for i, a in enumerate(func.rpo):
        blocks[a].order = i


def stmt_uses(s, sigs):
    """Registers read by statement (for liveness / webs)."""
    k = s.k
    u = []
    for e in s.exprs():
        u += regs_in(e)
    if k == 'call' or k == 'tail':
        sig = sigs.get(s.tgt) if s.tgt is not None else None
        if sig is not None and not sig.dynamic:
            u += sig.params
    elif k == 'ret':
        u += s.rets or []
    return u


def block_masks(func, sigs):
    """Per block (use, def) masks for liveness.  Calls kill clobbers."""
    res = {}
    for a, b in func.blocks.items():
        # recompute properly in forward order: use = regs read before written
        use = 0
        dfn = 0
        for s in b.stmts + [b.term]:
            u = mask(stmt_uses(s, sigs))
            use |= u & ~dfn
            k = s.k
            if k == 'set':
                dfn |= 1 << RIDX[s.dst]
            elif k == 'call':
                dfn |= CLOBBER_MASK
        res[a] = (use, dfn)
    return res


def liveness(func, sigs):
    bm = block_masks(func, sigs)
    live_in = {a: 0 for a in func.blocks}
    live_out = {a: 0 for a in func.blocks}
    changed = True
    order = list(reversed(func.rpo))
    while changed:
        changed = False
        for a in order:
            b = func.blocks[a]
            lo = 0
            for s in set(b.succs):
                lo |= live_in[s]
            use, dfn = bm[a]
            li = use | (lo & ~dfn)
            if li != live_in[a] or lo != live_out[a]:
                live_in[a] = li
                live_out[a] = lo
                changed = True
    return live_in, live_out


def annotate_live_after(func, sigs, live_out):
    """Set st.live_after for call statements (mask of regs live right after the call)."""
    for a, b in func.blocks.items():
        live = live_out[a]
        # terminator
        live |= mask(stmt_uses(b.term, sigs))
        for s in reversed(b.stmts):
            if s.k == 'call':
                s.live_after = live
            k = s.k
            if k == 'set':
                live &= ~(1 << RIDX[s.dst])
            elif k == 'call':
                live &= ~CLOBBER_MASK
            live |= mask(stmt_uses(s, sigs))


def may_ret(func, sigs, poss=None):
    """Return registers `func` could possibly leave defined (own writes plus what callees could return)."""
    m = 0

    def callee_mask(tgt):
        sig = sigs.get(tgt) if tgt is not None else None
        if sig is None or sig.dynamic:
            return mask(['r2', 'f0'])
        if poss is not None and tgt in poss:
            return poss[tgt]
        return mask(sig.rets)

    for b in func.blocks.values():
        for s in b.stmts:
            if s.k == 'set' and s.dst in RET_REGS:
                m |= 1 << RIDX[s.dst]
            elif s.k == 'call':
                m |= callee_mask(s.tgt)
        t = b.term
        if t.k == 'tail':
            m |= callee_mask(t.tgt)
    return m


def possible_returns(funcs, sigs):
    """Least fixpoint of may_ret over the call graph, independent of what callers demand.  (Demand-driven
    propagation alone can never discover a value that is only produced by a callee of a callee.)"""
    poss = {a: 0 for a in funcs}
    for _ in range(30):
        changed = False
        for a, f in funcs.items():
            m = may_ret(f, sigs, poss)
            if m != poss[a]:
                poss[a] = m
                changed = True
        if not changed:
            break
    return poss


def prefix_params(regs_mask):
    """Complete int / float param registers to contiguous prefixes."""
    gi = [i for i, r in enumerate(ARG_G) if regs_mask >> RIDX[r] & 1]
    fi = [i for i, r in enumerate(ARG_F) if regs_mask >> RIDX[r] & 1]
    return ARG_G[:(max(gi) + 1) if gi else 0] + ARG_F[:(max(fi) + 1) if fi else 0]


RETURNING_CALLBACKS = {
    'getmoveview', 'getgameview', 'getthirdpersonview', 'getpropview', 'getviewportals', 'getviewportalrects',
    'getviewfloors', 'getviewwalls', 'getviewvolumes', 'getviewguns', 'getviewbg', 'getpadlinkedit', 'getpropedit',
    'getturtlecpu', 'getturtlegs', 'getdisplayspeed', 'getdisplayperfstats', 'getdisplayrooms', 'getdisplaypos',
    'getinvincible', 'getinvisible', 'getcollisions',
}


def infer_signatures(funcs, sigs, addr_taken):
    """Iterate to a fixpoint.  funcs: {addr: Func (converted, cfg built)}; sigs pre-seeded with externs."""
    for a, f in funcs.items():
        if a not in sigs:
            sigs[a] = Sig(f.name)
    # callbacks that are only ever called through a pointer by code that uses the returned value (the debug menu's text
    # getters return the text of their On / Off display): no direct caller demands their result
    for a, sg in sigs.items():
        if sg.name in RETURNING_CALLBACKS:
            sg.addr_taken = True
    demand = defaultdict(int)
    poss = possible_returns(funcs, sigs)
    for it in range(12):
        changed = False
        newdemand = defaultdict(int)
        for a, f in funcs.items():
            sig = sigs[a]
            for b in f.blocks.values():
                if b.term.k == 'ret':
                    b.term.rets = sig.rets
            li, lo = liveness(f, sigs)
            annotate_live_after(f, sigs, lo)
            params = prefix_params(li[f.entry] & ARG_MASK)
            if params != sig.params:
                sig.params = params
                changed = True
            for b in f.blocks.values():
                for s in b.stmts:
                    if s.k == 'call' and s.tgt in funcs or (s.k == 'call' and s.tgt in sigs):
                        newdemand[s.tgt] |= s.live_after & RET_MASK
        # tail calls: callee rets demanded if we return them
        for a, f in funcs.items():
            for b in f.blocks.values():
                t = b.term
                if t.k == 'tail' and t.tgt in sigs:
                    newdemand[t.tgt] |= mask(sigs[a].rets)
        for a, f in funcs.items():
            sig = sigs[a]
            d = newdemand[a]
            if sig.addr_taken:
                d |= mask(['r2'])
            m = poss[a]
            rets = [r for r in RET_REGS if (d & m) >> RIDX[r] & 1]
            if rets != sig.rets:
                sig.rets = rets
                changed = True
        for a, sg in sigs.items():
            if sg.extern:
                d = newdemand[a]
                rets = [r for r in RET_REGS if d >> RIDX[r] & 1 and r != 'r3' and (r != 'f0' or sg.retf)]
                if rets != sg.rets:
                    sg.rets = rets
                    changed = True
        if not changed:
            break
    return sigs
