"""Independent reference interpreter for the VU1 microcode (development tool only).

Written separately from vu1dec.py (the decompiler) so the two can be compared on real machine states captured from the
running game: usage

    python vu1ref.py <dump dir> [n]      # compare after_NNNN.bin against the interpreter run from before_NNNN.bin
"""
import struct, sys, os, math
import numpy as np
from ps2elf import Elf
from vu1 import extract

F32 = np.float32


def f32(x):
    return float(F32(x))


def bits_f(b):
    return struct.unpack('<f', struct.pack('<I', b & 0xffffffff))[0]


def f_bits(f):
    return struct.unpack('<I', struct.pack('<f', f))[0]


def clampf(x):
    """VU float result: no NaN, no infinity, denormals flushed."""
    if x != x:
        return 0.0
    if math.isinf(x):
        return math.copysign(3.4028234663852886e38, x)
    if x != 0.0 and abs(x) < 1.1754943508222875e-38:
        return math.copysign(0.0, x)
    return x


class State:
    def __init__(self, blob):
        o = 0
        self.mem = [list(struct.unpack_from('<4I', blob, o + 16 * i)) for i in range(1024)]
        o += 16384
        self.vf = [list(struct.unpack_from('<4I', blob, o + 16 * i)) for i in range(32)]
        o += 512
        self.vi = [struct.unpack_from('<I', blob, o + 4 * i)[0] & 0xffff for i in range(16)]
        o += 64
        self.acc = [bits_f(b) for b in struct.unpack_from('<4I', blob, o)]
        o += 16
        (q, qp, p, i, mac, clip, top, itop, entry) = struct.unpack_from('<9I', blob, o)
        o += 36
        self.q, self.q_pend, self.p, self.i = bits_f(q), bits_f(qp), bits_f(p), bits_f(i)
        self.mac, self.clip, self.top, self.itop, self.entry = mac & 0xffff, clip, top, itop, entry
        self.q_ready, self.cycle = struct.unpack_from('<QQ', blob, o)
        o += 16
        n = struct.unpack_from('<I', blob, o)[0]
        o += 4
        self.kicks = list(struct.unpack_from(f'<{n}I', blob, o)) if n else []
        self.vf[0] = [0, 0, 0, 0x3f800000]
        self.vi[0] = 0


def laneset(w):
    return [l for l, b in enumerate((24, 23, 22, 21)) if w >> b & 1]


def s16(v):
    v &= 0xffff
    return v - 65536 if v & 0x8000 else v


class Ref:
    def __init__(self, st, imem, itof_flags=False, flag_latency=0):
        self.itof_flags = itof_flags
        self.flag_latency = flag_latency
        self.flag_events = []
        self.s = st
        self.imem = imem
        self.kicks = []
        self.trace = []

    # ---------------------------------------------------------------- flag pipeline (optional latency)
    def set_mac(self, m):
        if self.flag_latency:
            self.flag_events.append((self.s.cycle + self.flag_latency, 'mac', m))
        else:
            self.s.mac = m

    def set_clip(self, bits):
        if self.flag_latency:
            self.flag_events.append((self.s.cycle + self.flag_latency, 'clip', bits))
        else:
            self.s.clip = ((self.s.clip << 6) | bits) & 0xffffff

    def retire_flags(self):
        if not self.flag_events:
            return
        keep = []
        for ev in self.flag_events:
            if ev[0] <= self.s.cycle:
                if ev[1] == 'mac':
                    self.s.mac = ev[2]
                else:
                    self.s.clip = ((self.s.clip << 6) | ev[2]) & 0xffffff
            else:
                keep.append(ev)
        self.flag_events = keep

    # ---------------------------------------------------------------- helpers
    def vfw(self, reg, lanes, vals, flags=None):
        s = self.s
        for l in lanes:
            if reg != 0:
                s.vf[reg][l] = vals[l]

    def mac_of(self, lanes, raws):
        m = 0
        for l in lanes:
            r = raws[l]
            v = clampf(r)
            sh = 3 - l
            if v == 0.0:
                m |= 1 << sh
            if math.copysign(1.0, r) < 0 and r != 0.0:
                m |= 0x10 << sh
            if math.isinf(r):
                m |= 0x100 << sh
        return m

    def mem_addr(self, base, imm):
        return (base + imm) & 1023

    # ---------------------------------------------------------------- upper
    def upper(self, u):
        """Returns a closure to commit the result after the lower half executes."""
        s = self.s
        fn = u & 63
        ft, fs, fd = (u >> 16) & 31, (u >> 11) & 31, (u >> 6) & 31
        lanes = laneset(u)
        A = [bits_f(b) for b in s.vf[fs]]
        T = [bits_f(b) for b in s.vf[ft]]
        ACC = list(s.acc)
        res_bits = None
        Q, I = s.q, s.i

        def vec(op, a, b):
            return [f32(op(F32(a[l]), F32(b[l]))) for l in range(4)]

        add = lambda x, y: x + y
        sub = lambda x, y: x - y
        mul = lambda x, y: x * y
        mx = lambda x, y: max(x, y)
        mn = lambda x, y: min(x, y)

        def bc(i):
            return [T[i]] * 4

        def bank(kind, i=None):
            return {'v': T, 'q': [Q] * 4, 'i': [I] * 4, 'b': bc(i) if i is not None else None}[kind]

        def madd(a, b, sgn):
            prod = vec(mul, a, b)
            return [f32(F32(ACC[l]) + F32(prod[l]) * sgn) for l in range(4)]

        res = None
        dest_acc = False
        no_flags = False
        if fn < 0x3c:
            table = {}
            for k in range(4):
                table[0x00 + k] = (add, 'b', k, False)
                table[0x04 + k] = (sub, 'b', k, False)
                table[0x08 + k] = ('madd', 'b', k, False)
                table[0x0c + k] = ('msub', 'b', k, False)
                table[0x10 + k] = (mx, 'b', k, False)
                table[0x14 + k] = (mn, 'b', k, False)
                table[0x18 + k] = (mul, 'b', k, False)
            table[0x1c] = (mul, 'q', None, False)
            table[0x1d] = (mx, 'i', None, False)
            table[0x1e] = (mul, 'i', None, False)
            table[0x1f] = (mn, 'i', None, False)
            table[0x20] = (add, 'q', None, False)
            table[0x21] = ('madd', 'q', None, False)
            table[0x22] = (add, 'i', None, False)
            table[0x23] = ('madd', 'i', None, False)
            table[0x24] = (sub, 'q', None, False)
            table[0x25] = ('msub', 'q', None, False)
            table[0x26] = (sub, 'i', None, False)
            table[0x27] = ('msub', 'i', None, False)
            table[0x28] = (add, 'v', None, False)
            table[0x29] = ('madd', 'v', None, False)
            table[0x2a] = (mul, 'v', None, False)
            table[0x2b] = (mx, 'v', None, False)
            table[0x2c] = (sub, 'v', None, False)
            table[0x2d] = ('msub', 'v', None, False)
            table[0x2f] = (mn, 'v', None, False)
            if fn == 0x2e:
                res = [f32(ACC[0] - A[1] * T[2]), f32(ACC[1] - A[2] * T[0]), f32(ACC[2] - A[0] * T[1]), 0.0]
            else:
                op, kind, i, _ = table[fn]
                B = bank(kind, i)
                if op == 'madd':
                    res = madd(A, B, 1.0)
                elif op == 'msub':
                    res = madd(A, B, -1.0)
                else:
                    res = vec(op, A, B)
        else:
            idx = ((u >> 6) & 31) * 4 + (u & 3)
            tab = {}
            for k in range(4):
                tab[0 + k] = (add, 'b', k)
                tab[4 + k] = (sub, 'b', k)
                tab[8 + k] = ('madd', 'b', k)
                tab[12 + k] = ('msub', 'b', k)
                tab[24 + k] = (mul, 'b', k)
            tab.update({28: (mul, 'q', None), 30: (mul, 'i', None), 32: (add, 'q', None), 33: ('madd', 'q', None),
                        34: (add, 'i', None), 35: ('madd', 'i', None), 36: (sub, 'q', None), 37: ('msub', 'q', None),
                        38: (sub, 'i', None), 39: ('msub', 'i', None), 40: (add, 'v', None), 41: ('madd', 'v', None),
                        42: (mul, 'v', None), 44: (sub, 'v', None), 45: ('msub', 'v', None)})
            if idx in tab:
                op, kind, i = tab[idx]
                B = bank(kind, i)
                if op == 'madd':
                    res = madd(A, B, 1.0)
                elif op == 'msub':
                    res = madd(A, B, -1.0)
                else:
                    res = vec(op, A, B)
                dest_acc = True
            elif 16 <= idx <= 19:
                sc = [1.0, 16.0, 4096.0, 32768.0][idx - 16]
                res = [f32(float(struct.unpack('<i', struct.pack('<I', s.vf[fs][l]))[0]) / sc) for l in range(4)]
                no_flags = True
            elif 20 <= idx <= 23:
                sc = [1.0, 16.0, 4096.0, 32768.0][idx - 20]
                out = []
                for l in range(4):
                    v = clampf(A[l]) * sc
                    iv = int(v) if abs(v) < 2 ** 31 else (2 ** 31 - 1 if v > 0 else -2 ** 31)
                    iv = max(-2 ** 31, min(2 ** 31 - 1, iv))
                    out.append(iv & 0xffffffff)
                res_bits = out
                res = [0.0] * 4
                no_flags = True
            elif idx == 29:
                res_bits = [b & 0x7fffffff for b in s.vf[fs]]
                res = [0.0] * 4
                no_flags = True
            elif idx == 31:
                w = abs(clampf(T[3]))
                b = 0
                if A[0] > w: b |= 1
                if A[0] < -w: b |= 2
                if A[1] > w: b |= 4
                if A[1] < -w: b |= 8
                if A[2] > w: b |= 16
                if A[2] < -w: b |= 32

                def commit_clip():
                    self.set_clip(b)
                return commit_clip
            elif idx == 46:
                res = [f32(A[1] * T[2]), f32(A[2] * T[0]), f32(A[0] * T[1]), 0.0]
                dest_acc = True
            elif idx == 47:
                return lambda: None
            else:
                raise ValueError(f'upper special {idx}')

        # FTOI / ITOF / ABS write the register named by the *ft* field (bits 10:6 are part of their opcode)
        if no_flags:
            fd = ft

        def commit():
            if dest_acc:
                for l in lanes:
                    s.acc[l] = clampf(res[l])
                self.set_mac(self.mac_of(lanes, res))
            else:
                for l in lanes:
                    if fd != 0:
                        if res_bits is not None:
                            s.vf[fd][l] = res_bits[l]
                        else:
                            s.vf[fd][l] = f_bits(res[l] if no_flags else clampf(res[l]))
                if not no_flags:
                    self.set_mac(self.mac_of(lanes, res))
                elif self.itof_flags and res_bits is None:
                    self.set_mac(self.mac_of(lanes, res))
        return commit

    # ---------------------------------------------------------------- lower
    def lower(self, l, pc):
        """Execute the lower half.  Returns (branch_kind, target_or_cond) or None."""
        s = self.s
        op = l >> 25 & 0x7f
        ft, fs = (l >> 16) & 31, (l >> 11) & 31
        lanes = laneset(l)
        imm11 = l & 0x7ff
        if imm11 & 0x400:
            imm11 -= 0x800
        vi = s.vi

        def setvi(r, v):
            if r:
                vi[r] = v & 0xffff

        if op == 0x00:
            a = self.mem_addr(vi[fs], imm11)
            for k in lanes:
                if ft:
                    s.vf[ft][k] = s.mem[a][k]
        elif op == 0x01:
            a = self.mem_addr(vi[ft], imm11)
            for k in lanes:
                s.mem[a][k] = s.vf[fs][k]
        elif op == 0x04:
            a = self.mem_addr(vi[fs], imm11)
            k = lanes[0]
            setvi(ft, s.mem[a][k])
        elif op == 0x05:
            a = self.mem_addr(vi[fs], imm11)
            for k in lanes:
                s.mem[a][k] = vi[ft]
        elif op in (0x08, 0x09):
            imm15 = ((l >> 21) & 15) << 11 | (l & 0x7ff)
            setvi(ft, vi[fs] + imm15 if op == 0x08 else vi[fs] - imm15)
        elif op == 0x10:
            setvi(1, 1 if (s.clip & 0xffffff) == (l & 0xffffff) else 0)
        elif op == 0x11:
            s.clip = l & 0xffffff
        elif op == 0x12:
            setvi(1, 1 if s.clip & (l & 0xffffff) else 0)
        elif op == 0x13:
            setvi(1, 1 if ((s.clip | (l & 0xffffff)) & 0xffffff) == 0xffffff else 0)
        elif op in (0x14, 0x15, 0x16, 0x17):
            setvi(ft, 0)
        elif op == 0x18:
            setvi(ft, 1 if s.mac == vi[fs] else 0)
        elif op == 0x1a:
            setvi(ft, s.mac & vi[fs])
        elif op == 0x1b:
            setvi(ft, s.mac | vi[fs])
        elif op == 0x1c:
            setvi(ft, s.clip & 0xfff)
        elif op in (0x20, 0x21):
            tgt = pc + 1 + imm11
            if op == 0x21:
                setvi(ft, pc + 2)
            return ('jmp', tgt)
        elif op in (0x24, 0x25):
            tgt = vi[fs]
            if op == 0x25:
                setvi(ft, pc + 2)
            return ('jmp', tgt)
        elif op in (0x28, 0x29, 0x2c, 0x2d, 0x2e, 0x2f):
            tgt = pc + 1 + imm11
            c = {0x28: vi[ft] == vi[fs], 0x29: vi[ft] != vi[fs], 0x2c: s16(vi[fs]) < 0, 0x2d: s16(vi[fs]) > 0,
                 0x2e: s16(vi[fs]) <= 0, 0x2f: s16(vi[fs]) >= 0}[op]
            return ('cond', tgt, c)
        elif op == 0x40:
            fn = l & 63
            if fn in (0x30, 0x31, 0x34, 0x35):
                fd = (l >> 6) & 31
                a, b = vi[fs], vi[ft]
                setvi(fd, {0x30: a + b, 0x31: a - b, 0x34: a & b, 0x35: a | b}[fn])
            elif fn == 0x32:
                imm5 = (l >> 6) & 31
                imm5 = imm5 - 32 if imm5 & 16 else imm5
                setvi(ft, vi[fs] + imm5)
            else:
                row = (l >> 6) & 31
                fsf, ftf = (l >> 21) & 3, (l >> 23) & 3
                key = (fn, row)
                if key == (0x3c, 0x0c):
                    v = list(s.vf[fs])
                    for k in lanes:
                        if ft:
                            s.vf[ft][k] = v[k]
                elif key == (0x3d, 0x0c):
                    v = list(s.vf[fs])
                    r = [v[1], v[2], v[3], v[0]]
                    for k in lanes:
                        if ft:
                            s.vf[ft][k] = r[k]
                elif key == (0x3c, 0x0d):  # LQI
                    a = vi[fs] & 1023
                    for k in lanes:
                        if ft:
                            s.vf[ft][k] = s.mem[a][k]
                    setvi(fs, vi[fs] + 1)
                elif key == (0x3e, 0x0d):  # LQD
                    setvi(fs, vi[fs] - 1)
                    a = vi[fs] & 1023
                    for k in lanes:
                        if ft:
                            s.vf[ft][k] = s.mem[a][k]
                elif key == (0x3d, 0x0d):  # SQI
                    a = vi[ft] & 1023
                    for k in lanes:
                        s.mem[a][k] = s.vf[fs][k]
                    setvi(ft, vi[ft] + 1)
                elif key == (0x3f, 0x0d):  # SQD
                    setvi(ft, vi[ft] - 1)
                    a = vi[ft] & 1023
                    for k in lanes:
                        s.mem[a][k] = s.vf[fs][k]
                elif key == (0x3c, 0x0e):  # DIV
                    a, b = bits_f(s.vf[fs][fsf]), bits_f(s.vf[ft][ftf])
                    if b == 0.0:
                        r = 0.0 if a == 0.0 else math.copysign(3.4028234663852886e38, a) * (-1 if math.copysign(1, b) < 0 else 1)
                    else:
                        r = clampf(f32(F32(a) / F32(b)))
                    s.q_pend, s.q_ready = r, s.cycle + 7
                elif key == (0x3d, 0x0e):  # SQRT
                    s.q_pend, s.q_ready = clampf(f32(math.sqrt(abs(bits_f(s.vf[ft][ftf]))))), s.cycle + 7
                elif key == (0x3e, 0x0e):  # RSQRT
                    a, b = bits_f(s.vf[fs][fsf]), bits_f(s.vf[ft][ftf])
                    d = math.sqrt(abs(b))
                    s.q_pend = math.copysign(3.4028234663852886e38, a) if d == 0 else clampf(f32(a / d))
                    s.q_ready = s.cycle + 13
                elif key == (0x3f, 0x0e):  # WAITQ
                    s.cycle = max(s.cycle, s.q_ready)
                    s.q = s.q_pend
                elif key == (0x3c, 0x0f):  # MTIR
                    setvi(ft, s.vf[fs][fsf])
                elif key == (0x3d, 0x0f):  # MFIR
                    v = s16(vi[fs]) & 0xffffffff
                    for k in lanes:
                        if ft:
                            s.vf[ft][k] = v
                elif key == (0x3e, 0x0f):  # ILWR
                    a = vi[fs] & 1023
                    setvi(ft, s.mem[a][lanes[0]])
                elif key == (0x3f, 0x0f):  # ISWR
                    a = vi[fs] & 1023
                    for k in lanes:
                        s.mem[a][k] = vi[ft]
                elif key == (0x3c, 0x1a):
                    setvi(ft, s.top)
                elif key == (0x3d, 0x1a):
                    setvi(ft, s.itop)
                elif key == (0x3c, 0x1b):
                    self.kicks.append(vi[fs] & 1023)
                else:
                    raise ValueError(f'lower special {key} at {pc:#x}')
        else:
            raise ValueError(f'lower op {op:#x} at {pc:#x}')
        return None

    # ---------------------------------------------------------------- pair / run
    def pair(self, pc):
        s = self.s
        w = self.imem[pc]
        hi, lo = w >> 32, w & 0xffffffff
        s.cycle += 1
        if s.cycle >= s.q_ready:
            s.q = s.q_pend
        self.retire_flags()
        if not (hi >> 31 & 1) and (lo >> 25) == 0x40 and (lo & 63) == 0x3f and ((lo >> 6) & 31) == 0x0e:
            s.cycle = max(s.cycle, s.q_ready)
            s.q = s.q_pend
        if self.trace is not None:
            self.trace.append(pc)
        if hi >> 31 & 1:
            s.i = bits_f(lo)
        commit = self.upper(hi)
        br = None
        if not hi >> 31 & 1:
            br = self.lower(lo, pc)
        commit()
        return br, bool(hi >> 30 & 1)

    def run(self, entry, limit=200000):
        pc = entry
        steps = 0
        while True:
            steps += 1
            if steps > limit:
                raise RuntimeError('step limit')
            br, end = self.pair(pc)
            if br is not None:
                # delay slot
                self.pair(pc + 1)[0]
                if br[0] == 'jmp':
                    pc = br[1]
                elif br[2]:
                    pc = br[1]
                else:
                    pc = pc + 2
                continue
            if end:
                self.pair(pc + 1)
                return
            pc += 1


def compare(dump_dir, n):
    imem = extract(Elf().image())
    bef = State(open(f'{dump_dir}/before_{n:04d}.bin', 'rb').read())
    aft = State(open(f'{dump_dir}/after_{n:04d}.bin', 'rb').read())
    ref = Ref(bef, imem)
    ref.run(bef.entry)
    diffs = []
    for q in range(1024):
        if ref.s.mem[q] != aft.mem[q]:
            diffs.append(q)
    rv = ref.s.vf
    av = aft.vf
    vfd = [i for i in range(32) if rv[i] != av[i]]
    vid = [i for i in range(16) if ref.s.vi[i] != aft.vi[i]]
    return bef, aft, ref, diffs, vfd, vid


if __name__ == '__main__':
    d = sys.argv[1]
    if len(sys.argv) > 2:
        ns = [int(sys.argv[2])]
    else:
        ns = sorted(int(f.split('_')[1].split('.')[0]) for f in os.listdir(d) if f.startswith('before_'))
    bad = 0
    for n in ns:
        try:
            bef, aft, ref, diffs, vfd, vid = compare(d, n)
        except Exception as e:
            print(f'#{n}: reference failed: {e}')
            bad += 1
            continue
        same_kicks = ref.kicks == aft.kicks
        ok = not diffs and not vfd and not vid and same_kicks
        if not ok:
            bad += 1
            print(f'#{n} entry={bef.entry:#x}: DIFF mem qw {diffs[:8]} ({len(diffs)}) vf {vfd} vi {vid} kicks ref={ref.kicks} rust={aft.kicks}')
    print(f'{len(ns)} runs compared, {bad} differ')
