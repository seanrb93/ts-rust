"""Variable recovery: reaching definitions -> webs (one variable each), dead-code elimination, expression inlining
and type inference.  Operates on functions already converted by analysis.convert()."""
from collections import defaultdict
from .ir import *
from .analysis import *

ENTRY, UNDEF = 0, 1


class Web:
    __slots__ = ('id', 'reg', 'defs', 'uses', 'ty', 'param', 'undef', 'live', 'inl', 'is_f')

    def __init__(self, wid, reg):
        self.id = wid
        self.reg = reg
        self.defs = []       # list of St, or 'entry'
        self.uses = 0
        self.ty = None
        self.param = False
        self.undef = False
        self.live = False
        self.inl = False
        self.is_f = reg[0] == 'f' and reg != 'fcc'


class DSU:
    def __init__(self):
        self.p = {}

    def find(self, x):
        p = self.p
        if x not in p:
            p[x] = x
            return x
        r = x
        while p[r] != r:
            r = p[r]
        while p[x] != r:
            p[x], x = r, p[x]
        return r

    def union(self, a, b):
        a, b = self.find(a), self.find(b)
        if a != b:
            self.p[b] = a


def call_defs(s, sigs):
    """(real def regs, clobber regs) for a call statement."""
    if s.sig is None:
        sig = sigs.get(s.tgt) if s.tgt is not None else None
        s.sig = sig
    sig = s.sig
    if sig is None or sig.dynamic:
        real = ['r2', 'f0'] if (sig is None or not sig.extern or True) else []
    else:
        real = list(sig.rets)
    clob = [r for r in CLOBBER if r not in real]
    return real, clob


def build_webs(func, sigs):
    blocks = func.blocks
    rpo = func.rpo
    params = set(func_params(func, sigs))
    # ---------- collect, per register, defs and effects per block
    # block_effects[a] = list of (reg, kind, key) in order: kind 'def' (key=(id(s),reg)) or 'kill'
    regs_seen = set(ARG_G + ARG_F)
    beff = {}
    for a in rpo:
        b = blocks[a]
        eff = []
        for s in b.stmts + [b.term]:
            for ex in s.exprs():
                regs_seen.update(regs_in(ex))
        for s in b.stmts:
            if s.k == 'set':
                eff.append((s.dst, 'def', (id(s), s.dst)))
                regs_seen.add(s.dst)
            elif s.k == 'call':
                real, clob = call_defs(s, sigs)
                for r in real:
                    eff.append((r, 'def', (id(s), r)))
                    regs_seen.add(r)
                for r in clob:
                    eff.append((r, 'kill', None))
        beff[a] = eff
    # per reg local def numbering
    defid = defaultdict(lambda: {'entry': ENTRY, 'undef': UNDEF})   # reg -> {key: idx}
    for reg in regs_seen:
        defid[reg]
    for a in rpo:
        for reg, kind, key in beff[a]:
            if kind == 'def':
                d = defid[reg]
                if key not in d:
                    d[key] = len(d)
    # gen/kill per block per reg
    gen = {}
    killed = {}
    for a in rpo:
        g = {}
        for reg, kind, key in beff[a]:
            g[reg] = (1 << UNDEF) if kind == 'kill' else (1 << defid[reg][key])
        gen[a] = g
    IN = {}
    for reg in regs_seen:
        inm = {a: 0 for a in rpo}
        out = {a: 0 for a in rpo}
        inm[func.entry] = 1 << ENTRY
        changed = True
        while changed:
            changed = False
            for a in rpo:
                b = blocks[a]
                i = 1 << ENTRY if a == func.entry else 0
                for p in b.preds:
                    i |= out[p]
                o = gen[a].get(reg, i)
                if i != inm[a] or o != out[a]:
                    inm[a] = i
                    out[a] = o
                    changed = True
        IN[reg] = inm
    # ---------- uses: reaching sets
    dsu = DSU()
    undef_uses = set()
    all_uses = []     # (St, reg)

    def reaching(a, pos_stmt_index, reg):
        """Reaching def keys for reg right before statement index (in block a)."""
        b = blocks[a]
        eff_before = None
        # scan stmts before pos backwards for last effect on reg
        for s in reversed(b.stmts[:pos_stmt_index]):
            if s.k == 'set' and s.dst == reg:
                return [(id(s), reg)], False
            if s.k == 'call':
                real, clob = call_defs(s, sigs)
                if reg in real:
                    return [(id(s), reg)], False
                if reg in clob:
                    return [], True
        m = IN[reg][a]
        keys = []
        un = False
        d = defid[reg]
        inv = None
        if m & 1:
            keys.append(('entry', reg))
        if m & 2:
            un = True
        if m >> 2:
            inv = {v: k for k, v in d.items() if isinstance(k, tuple)}
            mm = m >> 2
            idx = 2
            while mm:
                if mm & 1:
                    keys.append(inv[idx])
                mm >>= 1
                idx += 1
        return keys, un

    for a in rpo:
        b = blocks[a]
        for n, s in enumerate(b.stmts + [b.term]):
            # dynamic call / tail argument determination
            if s.k in ('call', 'tail'):
                sig = sigs.get(s.tgt) if s.tgt is not None else None
                s.sig = sig
                if sig is None or sig.dynamic:
                    args = []
                    for grp in (ARG_G, ARG_F):
                        cut = 0
                        for i, r in enumerate(grp):
                            keys, un = reaching(a, n, r)
                            ok = False
                            for k in keys:
                                if k[0] == 'entry':
                                    ok = ok or r in params
                                else:
                                    ok = True
                            if ok:
                                cut = i + 1
                        args += grp[:cut]
                    s.args = args
                else:
                    s.args = list(sig.params)
            elif s.k == 'ret':
                s.args = list(s.rets or [])
            regs = []
            for e in s.exprs():
                regs += regs_in(e)
            if s.k in ('call', 'tail', 'ret'):
                regs += s.args
            uw = {}
            for reg in dict.fromkeys(regs):
                keys, un = reaching(a, n, reg)
                if not keys:
                    keys = [('entry', reg)]
                    un = True
                for k in keys[1:]:
                    dsu.union(keys[0], k)
                dsu.find(keys[0])
                uw[reg] = keys[0]
                if un or any(k[0] == 'entry' and reg not in params for k in keys):
                    undef_uses.add(keys[0])
            s.uweb = uw
    # ---------- create webs
    webs = {}
    func.webs = webs

    rep_ids = {}

    def web_of(key):
        r = dsu.find(key)
        wid = rep_ids.get(r)
        if wid is None:
            wid = rep_ids[r] = len(rep_ids)
            webs[wid] = Web(wid, key[1])
        return webs[wid]

    stmt_by_id = {}
    for a in rpo:
        b = blocks[a]
        for s in b.stmts:
            stmt_by_id[id(s)] = s
    # defs: each real def key belongs to a web (singleton if never used)
    for a in rpo:
        b = blocks[a]
        for s in b.stmts:
            if s.k == 'set':
                w = web_of((id(s), s.dst))
                w.defs.append(s)
                s.dweb = w.id
            elif s.k == 'call':
                real, _ = call_defs(s, sigs)
                s.retwebs = []
                for r in real:
                    w = web_of((id(s), r))
                    w.defs.append(s)
                    s.retwebs.append((r, w.id))
    for a in rpo:
        b = blocks[a]
        for s in b.stmts + [b.term]:
            s.uweb = {r: web_of(k).id for r, k in s.uweb.items()} if s.uweb else {}
            for reg, wid in s.uweb.items():
                pass
    # entry defs
    for key in list(dsu.p):
        if key[0] == 'entry':
            w = web_of(key)
            if 'entry' not in w.defs:
                w.defs.append('entry')
                if key[1] in params:
                    w.param = True
    for key in undef_uses:
        web_of(key).undef = True
    # rewrite leaves
    for a in rpo:
        b = blocks[a]
        for s in b.stmts + [b.term]:
            if s.uweb is None:
                continue
            uw = s.uweb

            def fn(leaf, uw=uw):
                if leaf[0] == 'r':
                    return ('v', uw[leaf[1]])
                return None
            if s.exprs():
                s.set_exprs([subst(x, fn) for x in s.exprs()])
            if s.k in ('call', 'tail', 'ret'):
                s.argwebs = [(r, uw[r]) for r in s.args]
    func.entry_params = [r for r in (ARG_G + ARG_F) if r in params]
    return webs


def func_params(func, sigs):
    return sigs[func.addr].params


def count_uses(func):
    for w in func.webs.values():
        w.uses = 0
    for b in func.blocks.values():
        for s in b.stmts + [b.term]:
            for e in s.exprs():
                for x in walk(e):
                    if x[0] == 'v':
                        func.webs[x[1]].uses += 1
            if s.k in ('call', 'tail', 'ret') and s.argwebs:
                for r, wid in s.argwebs:
                    func.webs[wid].uses += 1


def dce(func):
    """Mark-sweep dead code elimination: keep statements with side effects and everything they depend on."""
    webs = func.webs
    live_webs = set()
    work = []

    def need_expr(e):
        for x in walk(e):
            if x[0] == 'v' and x[1] not in live_webs:
                live_webs.add(x[1])
                work.append(x[1])

    for b in func.blocks.values():
        for s in b.stmts + [b.term]:
            if s.k != 'set':
                for e in s.exprs():
                    need_expr(e)
                if s.k in ('call', 'tail', 'ret') and s.argwebs:
                    for r, wid in s.argwebs:
                        if wid not in live_webs:
                            live_webs.add(wid)
                            work.append(wid)
    while work:
        wid = work.pop()
        for d in webs[wid].defs:
            if d != 'entry' and d.k == 'set' and not getattr(d, 'dead_mark', False):
                for e in d.exprs():
                    need_expr(e)
    for b in func.blocks.values():
        keep = []
        for s in b.stmts:
            if s.k == 'set' and s.dweb not in live_webs:
                continue
            keep.append(s)
        b.stmts = keep
        for p, s in enumerate(b.stmts):
            s.pos = p
        b.term.pos = len(b.stmts)
    for w in webs.values():
        w.live = w.id in live_webs
    # drop dead ret defs
    for b in func.blocks.values():
        for s in b.stmts:
            if s.k == 'call' and s.retwebs:
                s.retwebs = [(r, w) for r, w in s.retwebs if w in live_webs]
    count_uses(func)


def has_load(e):
    return any(x[0] == 'ld' for x in walk(e))


def inline_exprs(func):
    """Forward-substitute single-use definitions and re-materialise constants / copies."""
    webs = func.webs
    count_uses(func)
    ndefs = {w.id: len(w.defs) for w in webs.values()}
    single = {}      # wid -> defining St, single def, single/any use, non-const
    remat = {}       # wid -> expression valid at every use

    def stable_leaf(x):
        return ndefs[x[1]] == 1

    CONSTLIKE = ('c', 'cf', 'sp', 'cb')
    for w in webs.values():
        if not w.live or w.undef or w.param or not w.defs or 'entry' in w.defs:
            continue
        ds = w.defs
        if any(d.k != 'set' for d in ds):
            continue
        if len(ds) == 1:
            e2 = ds[0].e
            if e2[0] in CONSTLIKE or (e2[0] == 'v' and ndefs[e2[1]] == 1 and e2[1] != w.id):
                remat[w.id] = e2
            else:
                single[w.id] = ds[0]
        elif all(d.e[0] in CONSTLIKE for d in ds) and all(d.e == ds[0].e for d in ds):
            remat[w.id] = ds[0].e

    def resolve(e):
        """Apply remat repeatedly."""
        for _ in range(8):
            ch = [False]

            def fn(leaf):
                if leaf[0] == 'v' and leaf[1] in remat:
                    ch[0] = True
                    return remat[leaf[1]]
                return None
            e = subst(e, fn)
            if not ch[0]:
                break
        return e

    for w, e2 in list(remat.items()):
        remat[w] = e2
    for a in func.rpo:
        b = func.blocks[a]
        for s in b.stmts + [b.term]:
            ex = s.exprs()
            if ex:
                s.set_exprs([resolve(x) for x in ex])
    for w in list(remat):
        remat[w] = resolve(remat[w])

    for rnd in range(6):
        changed_any = False
        for a in func.rpo:
            b = func.blocks[a]
            for p, s in enumerate(b.stmts + [b.term]):
                exprs = s.exprs()
                if not exprs:
                    continue
                touched = [False]

                def fn(leaf, p=p, s=s):
                    if leaf[0] != 'v':
                        return None
                    wid = leaf[1]
                    d = single.get(wid)
                    if d is None or d is s or d.dead:
                        return None
                    w = webs[wid]
                    if w.uses != 1:
                        return None
                    e2 = d.e
                    ok = True
                    same_block = d.blk is s.blk and d.pos < p
                    for x in walk(e2):
                        if x[0] == 'v' and not stable_leaf(x):
                            if not same_block:
                                return None
                            reg = webs[x[1]].reg
                            for t in b.stmts[d.pos + 1:p]:
                                if (t.k == 'set' and t.dst == reg) or (t.k == 'call' and (
                                        reg in CLOBBER_SET or reg in [r for r, _ in (t.retwebs or [])])):
                                    return None
                    if has_load(e2):
                        if not same_block:
                            return None
                        for t in b.stmts[d.pos + 1:p]:
                            if t.k in ('st', 'call', 'trap'):
                                return None
                    touched[0] = True
                    return e2
                new_exprs = [subst(ex, fn) for ex in exprs]
                if touched[0]:
                    s.set_exprs(new_exprs)
                    changed_any = True
        count_uses(func)
        for b in func.blocks.values():
            keep = []
            for s in b.stmts:
                if s.k == 'set' and (s.dweb in remat or (s.dweb in single and single[s.dweb] is s))                         and webs[s.dweb].uses == 0:
                    s.dead = True
                    webs[s.dweb].live = False
                    continue
                keep.append(s)
            b.stmts = keep
            for pp, s in enumerate(b.stmts):
                s.pos = pp
            b.term.pos = len(b.stmts)
        count_uses(func)
        if not changed_any:
            break


def simplify_cfg(func):
    """Fold constant branches, drop unreachable blocks.  Returns True if anything changed."""
    changed = False
    for b in func.blocks.values():
        t = b.term
        if t.k == 'br':
            c = t.cond
            if c[0] == 'cb':
                t.k = 'jmp'
                t.T = t.T if c[1] else t.F
                t.cond = None
                changed = True
            elif t.T == t.F:
                t.k = 'jmp'
                t.cond = None
                changed = True
        elif t.k == 'sw' and t.idx[0] == 'c':
            pass
    if changed:
        from .analysis import build_cfg
        build_cfg(func)
        live_blocks = set(func.blocks)
        for w in func.webs.values():
            w.defs = [d for d in w.defs if d == 'entry' or d.blk.addr in live_blocks]
    return changed


def run_passes(func, sigs):
    build_webs(func, sigs)
    for _ in range(6):
        dce(func)
        inline_exprs(func)
        if not simplify_cfg(func):
            break
    dce(func)


RANK = {'bool': 0, 'i32': 1, 'i64': 2, 'u128': 3, 'f32': 9}


def join_ty(a, b):
    if a is None:
        return b
    if b is None:
        return a
    if a == b:
        return a
    if 'f32' in (a, b):
        return 'f32'
    return a if RANK[a] >= RANK[b] else b


def infer_types(func, sigs, rettypes):
    """Fixpoint of web types.  rettypes: {callee_addr: {reg: type}}."""
    webs = func.webs
    for w in webs.values():
        if not w.live and not w.param:
            continue
        w.ty = 'f32' if w.is_f else ('bool' if w.reg == 'fcc' else 'i32')
        if w.reg in ('bc',):
            w.ty = 'bool'
    vt = lambda wid: webs[wid].ty or 'i32'
    for _ in range(8):
        changed = False
        for b in func.blocks.values():
            for s in b.stmts:
                if s.k == 'set':
                    w = webs[s.dweb]
                    t = ty(s.e, vt)
                    if w.reg in ('fcc', 'bc'):
                        t = 'bool'
                    elif w.is_f and t != 'f32':
                        t = 'f32'
                    n = join_ty(w.ty, t) if not (w.is_f or w.reg in ('fcc', 'bc')) else w.ty
                    if n != w.ty:
                        w.ty = n
                        changed = True
                elif s.k == 'call' and s.retwebs:
                    rt = rettypes.get(s.tgt) or {}
                    for r, wid in s.retwebs:
                        w = webs[wid]
                        t = rt.get(r)
                        if t and not w.is_f:
                            n = join_ty(w.ty, t)
                            if n != w.ty:
                                w.ty = n
                                changed = True
        if not changed:
            break
    # return types of this function: join of web types feeding ret
    out = {}
    for b in func.blocks.values():
        t = b.term
        if t.k == 'ret' and t.argwebs:
            for r, wid in t.argwebs:
                out[r] = join_ty(out.get(r), webs[wid].ty)
    return out
