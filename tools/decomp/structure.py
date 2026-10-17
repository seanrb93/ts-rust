"""Control-flow structuring: CFG -> nested if / loop / labelled-block AST ("Beyond Relooper" style), followed by
readability clean-ups.  Irreducible CFGs fall back to a pc-dispatch loop.

AST items (body = list):
  St                         plain statement (set / st / call / trap)
  ('if', cond, then, else)
  ('block', label, body)     labelled block, `break label` jumps to its end
  ('loop', label, body)      `loop`, `continue label` restarts, `break label` leaves
  ('while', label, cond, body)
  ('break', label) ('continue', label)
  ('ret', St)                return / tail call
  ('switch', idx, [(vals, body)])
  ('dispatch', entry, [(addr, body)])   /  ('goto', addr)
"""
import sys
from .ir import *
from .analysis import build_cfg, St

sys.setrecursionlimit(100000)


class Irreducible(Exception):
    pass


def cleanup_cfg(func):
    """Thread jumps through empty blocks and merge straight-line block chains."""
    blocks = func.blocks
    for _ in range(50):
        changed = False
        # --- 1. thread through empty `jmp` blocks
        def final(t):
            seen = set()
            while True:
                b = blocks[t]
                if t in seen or t == func.entry or b.stmts or b.term.k != 'jmp':
                    return t
                seen.add(t)
                t = b.term.T
        for b in list(blocks.values()):
            t = b.term
            if t.k == 'br':
                nt, nf = final(t.T), final(t.F)
                if (nt, nf) != (t.T, t.F):
                    t.T, t.F = nt, nf
                    changed = True
            elif t.k == 'jmp':
                nt = final(t.T)
                if nt != t.T:
                    t.T = nt
                    changed = True
            elif t.k == 'sw':
                nts = [final(x) for x in t.targets]
                if nts != t.targets:
                    t.targets = nts
                    changed = True
        build_cfg(func)
        blocks = func.blocks
        # --- 2. merge chains
        for a in list(func.rpo):
            b = blocks.get(a)
            if b is None:
                continue
            while b.term.k == 'jmp':
                x = b.term.T
                xb = blocks.get(x)
                if xb is None or x == b.addr or x == func.entry or len(xb.preds) != 1:
                    break
                for s in xb.stmts:
                    s.blk = b
                b.stmts += xb.stmts
                b.term = xb.term
                b.term.blk = b
                for p, s in enumerate(b.stmts):
                    s.pos = p
                b.term.pos = len(b.stmts)
                del blocks[x]
                changed = True
                build_cfg(func)
                blocks = func.blocks
        if not changed:
            break
    build_cfg(func)


def dominators(func):
    blocks = func.blocks
    rpo = func.rpo
    num = {a: i for i, a in enumerate(rpo)}
    idom = {func.entry: func.entry}

    def intersect(a, b):
        while a != b:
            while num[a] > num[b]:
                a = idom[a]
            while num[b] > num[a]:
                b = idom[b]
        return a
    changed = True
    while changed:
        changed = False
        for a in rpo[1:]:
            ps = [p for p in blocks[a].preds if p in idom]
            if not ps:
                continue
            n = ps[0]
            for p in ps[1:]:
                n = intersect(p, n)
            if idom.get(a) != n:
                idom[a] = n
                changed = True
    return idom, num


class Structurer:
    def __init__(self, func):
        self.f = func
        self.blocks = func.blocks
        self.idom, self.num = dominators(func)
        self.label = 0
        # ---- dominance tests / back edges
        self.back = set()
        self.loops = {}
        for a, b in self.blocks.items():
            for s in set(b.succs):
                if self.dominates(s, a):
                    self.back.add((a, s))
                elif self.num[s] <= self.num[a]:
                    raise Irreducible(f'{a:#x}->{s:#x}')
        for (u, h) in self.back:
            body = self.loops.setdefault(h, {h})
            stack = [u]
            while stack:
                n = stack.pop()
                if n not in body:
                    body.add(n)
                    stack.extend(self.blocks[n].preds)
        # ---- dominator tree (with loop-exit re-parenting)
        self.parent = dict(self.idom)
        self.forced = set()
        for h, L in self.loops.items():
            for n in list(self.blocks):
                if n in L:
                    continue
                if any(p in L for p in self.blocks[n].preds) and (
                        any((p, n) not in self.back for p in self.blocks[n].preds if p in L)):
                    # n is a loop exit target
                    if self.idom[n] in L or self.idom[n] == h:
                        self.parent[n] = h
                        self.forced.add(n)
        self.children = {a: [] for a in self.blocks}
        for a, p in self.parent.items():
            if a != self.f.entry:
                self.children[p].append(a)
        for a in self.children:
            self.children[a].sort(key=lambda x: self.num[x])
        # forward in-edge counts
        self.merge = set(self.forced)
        for a, b in self.blocks.items():
            fw = [p for p in set(b.preds) if (p, a) not in self.back]
            if len(fw) >= 2:
                self.merge.add(a)

    def dominates(self, a, b):
        while True:
            if a == b:
                return True
            if b == self.f.entry:
                return False
            b = self.idom[b]

    def newlabel(self):
        self.label += 1
        return self.label

    def run(self):
        return self.do_tree(self.f.entry, [])

    def do_tree(self, x, ctx):
        children = self.children[x]
        if x in self.loops:
            L = self.loops[x]
            inside = [c for c in children if c in L and c != x]
            exits = [c for c in children if c not in L]
            ys_in = [c for c in inside if c in self.merge]
            lbl = self.newlabel()
            elabels = [(self.newlabel(), e) for e in exits]
            frames = list(ctx)
            for lb, e in reversed(elabels):         # largest rpo outermost
                frames.append(('block', lb, e))
            frames.append(('loop', lbl, x))
            body = self.node_code(x, ys_in, frames)
            code = [('loop', lbl, body)]
            # wrap exits: layout  loop, e1, e2 ... ; innermost block is e1
            for i, (lb, e) in enumerate(elabels):
                code = [('block', lb, code)] + self.do_tree(e, ctx + [('block', l2, e2) for l2, e2 in
                                                                       reversed(elabels[i + 1:])])
            return code
        ys = [c for c in children if c in self.merge]
        return self.node_code(x, ys, ctx)

    def node_code(self, x, ys, ctx):
        if ys:
            y = ys[-1]
            lbl = self.newlabel()
            inner = self.node_code(x, ys[:-1], ctx + [('block', lbl, y)])
            return [('block', lbl, inner)] + self.do_tree(y, ctx)
        return self.translate(x, ctx)

    def do_branch(self, src, tgt, ctx):
        if (src, tgt) in self.back:
            for fr in reversed(ctx):
                if fr[0] == 'loop' and fr[2] == tgt:
                    return [('continue', fr[1])]
            raise Irreducible('no loop frame')
        for fr in reversed(ctx):
            if fr[0] == 'block' and fr[2] == tgt:
                return [('break', fr[1])]
        if tgt in self.merge:
            raise Irreducible(f'merge target {tgt:#x} not in context from {src:#x}')
        return self.do_tree(tgt, ctx)

    def translate(self, x, ctx):
        b = self.blocks[x]
        out = list(b.stmts)
        t = b.term
        k = t.k
        if k == 'br':
            out.append(('if', t.cond, self.do_branch(x, t.T, ctx), self.do_branch(x, t.F, ctx)))
        elif k == 'jmp':
            out += self.do_branch(x, t.T, ctx)
        elif k in ('ret', 'tail'):
            out.append(('ret', t))
        elif k == 'sw':
            groups = {}
            for i, tg in enumerate(t.targets):
                groups.setdefault(tg, []).append(i)
            arms = [(vals, self.do_branch(x, tg, ctx)) for tg, vals in groups.items()]
            out.append(('switch', t.idx, arms))
        else:
            raise Irreducible('term ' + k)
        return out


def dispatch_fallback(func):
    arms = []
    for a in func.rpo:
        b = func.blocks[a]
        out = list(b.stmts)
        t = b.term
        if t.k == 'br':
            out.append(('if', t.cond, [('goto', t.T)], [('goto', t.F)]))
        elif t.k == 'jmp':
            out.append(('goto', t.T))
        elif t.k in ('ret', 'tail'):
            out.append(('ret', t))
        elif t.k == 'sw':
            groups = {}
            for i, tg in enumerate(t.targets):
                groups.setdefault(tg, []).append(i)
            out.append(('switch', t.idx, [(v, [('goto', tg)]) for tg, v in groups.items()]))
        arms.append((a, out))
    return [('dispatch', func.entry, arms)]


# ----------------------------------------------------------------------------------------------------------------
# clean-ups
# ----------------------------------------------------------------------------------------------------------------

def count_jumps(body, counts):
    for it in body:
        if isinstance(it, St):
            continue
        k = it[0]
        if k in ('break', 'continue'):
            counts[(k, it[1])] = counts.get((k, it[1]), 0) + 1
        elif k == 'if':
            count_jumps(it[2], counts)
            count_jumps(it[3], counts)
        elif k in ('block', 'loop'):
            count_jumps(it[2], counts)
        elif k == 'while':
            count_jumps(it[3], counts)
        elif k == 'switch':
            for _, bd in it[2]:
                count_jumps(bd, counts)


def strip_tail(body, kind, label):
    """Remove `kind label` jumps in tail position of body.  Returns new body."""
    if not body:
        return body
    last = body[-1]
    if isinstance(last, tuple):
        k = last[0]
        if k == kind and last[1] == label:
            return strip_tail(body[:-1], kind, label)
        if k == 'if':
            return body[:-1] + [('if', last[1], strip_tail(last[2], kind, label), strip_tail(last[3], kind, label))]
        if k == 'block':
            return body[:-1] + [('block', last[1], strip_tail(last[2], kind, label))]
        if k == 'switch':
            return body[:-1] + [('switch', last[1], [(v, strip_tail(bd, kind, label)) for v, bd in last[2]])]
    return body


def simplify(body):
    out = []
    for it in body:
        if isinstance(it, St):
            out.append(it)
            continue
        k = it[0]
        if k == 'if':
            c, t, e = it[1], simplify(it[2]), simplify(it[3])
            if not t and not e:
                continue
            if not t:
                c, t, e = mk('not', c), e, t
            out.append(('if', c, t, e))
        elif k == 'block':
            lbl, bd = it[1], simplify(it[2])
            bd = strip_tail(bd, 'break', lbl)
            cnt = {}
            count_jumps(bd, cnt)
            if cnt.get(('break', lbl), 0) == 0:
                out += bd
            else:
                # block containing only a loop: break -> loop break
                if len(bd) == 1 and isinstance(bd[0], tuple) and bd[0][0] == 'loop':
                    out.append(('block', lbl, bd))
                else:
                    out.append(('block', lbl, bd))
        elif k == 'loop':
            lbl, bd = it[1], simplify(it[2])
            bd = strip_tail(bd, 'continue', lbl)
            out.append(('loop', lbl, bd))
        elif k == 'switch':
            out.append(('switch', it[1], [(v, simplify(bd)) for v, bd in it[2]]))
        else:
            out.append(it)
    return out


def rename_breaks(body, old, new):
    r = []
    for it in body:
        if isinstance(it, St):
            r.append(it)
            continue
        k = it[0]
        if k == 'break' and it[1] == old:
            r.append(('break', new))
        elif k == 'if':
            r.append(('if', it[1], rename_breaks(it[2], old, new), rename_breaks(it[3], old, new)))
        elif k in ('block', 'loop'):
            r.append((k, it[1], rename_breaks(it[2], old, new)))
        elif k == 'while':
            r.append(('while', it[1], it[2], rename_breaks(it[3], old, new)))
        elif k == 'switch':
            r.append(('switch', it[1], [(v, rename_breaks(bd, old, new)) for v, bd in it[2]]))
        else:
            r.append(it)
    return r


def hoist_loop_exits(body):
    """block E { loop L {...} }  ->  loop L {...}  with `break E` retargeted to L."""
    out = []
    for it in body:
        if isinstance(it, St):
            out.append(it)
            continue
        k = it[0]
        if k == 'block':
            bd = hoist_loop_exits(it[2])
            if len(bd) == 1 and isinstance(bd[0], tuple) and bd[0][0] == 'loop':
                lp = bd[0]
                out.append(('loop', lp[1], rename_breaks(lp[2], it[1], lp[1])))
            else:
                out.append(('block', it[1], bd))
        elif k == 'if':
            out.append(('if', it[1], hoist_loop_exits(it[2]), hoist_loop_exits(it[3])))
        elif k == 'loop':
            out.append(('loop', it[1], hoist_loop_exits(it[2])))
        elif k == 'switch':
            out.append(('switch', it[1], [(v, hoist_loop_exits(bd)) for v, bd in it[2]]))
        else:
            out.append(it)
    return out


def to_while(body):
    out = []
    for it in body:
        if isinstance(it, St):
            out.append(it)
            continue
        k = it[0]
        if k == 'loop':
            lbl, bd = it[1], to_while(it[2])
            if bd and isinstance(bd[0], tuple) and bd[0][0] == 'if':
                _, c, t, e = bd[0]
                brk = [('break', lbl)]
                if e == brk and len(bd) == 1:
                    out.append(('while', lbl, c, t))
                    continue
                if t == brk and len(bd) == 1:
                    out.append(('while', lbl, mk('not', c), e))
                    continue
            out.append(('loop', lbl, bd))
        elif k == 'if':
            out.append(('if', it[1], to_while(it[2]), to_while(it[3])))
        elif k == 'block':
            out.append(('block', it[1], to_while(it[2])))
        elif k == 'switch':
            out.append(('switch', it[1], [(v, to_while(bd)) for v, bd in it[2]]))
        else:
            out.append(it)
    return out


TERMINAL = ('break', 'continue', 'ret', 'goto')


def ends_terminal(body):
    if not body:
        return False
    l = body[-1]
    if isinstance(l, St):
        return False
    if l[0] in TERMINAL:
        return True
    if l[0] == 'if':
        return ends_terminal(l[2]) and ends_terminal(l[3])
    return False


def flatten_else(body):
    """if c { ...terminal } else { E }  ->  if c { ...terminal } E"""
    out = []
    for it in body:
        if isinstance(it, St):
            out.append(it)
            continue
        k = it[0]
        if k == 'if':
            c, t, e = it[1], flatten_else(it[2]), flatten_else(it[3])
            if e and ends_terminal(t):
                out.append(('if', c, t, []))
                out += e
            elif t and ends_terminal(e) and not ends_terminal(t):
                out.append(('if', mk('not', c), e, []))
                out += t
            else:
                out.append(('if', c, t, e))
        elif k in ('block', 'loop'):
            out.append((k, it[1], flatten_else(it[2])))
        elif k == 'while':
            out.append(('while', it[1], it[2], flatten_else(it[3])))
        elif k == 'switch':
            out.append(('switch', it[1], [(v, flatten_else(bd)) for v, bd in it[2]]))
        else:
            out.append(it)
    return out


def structure(func):
    cleanup_cfg(func)
    try:
        body = Structurer(func).run()
        body = simplify(body)
        body = hoist_loop_exits(body)
        body = simplify(body)
        body = to_while(body)
        body = simplify(body)
        body = flatten_else(body)
        body = simplify(body)
        func.irreducible = False
    except Irreducible:
        body = dispatch_fallback(func)
        func.irreducible = True
    return body
