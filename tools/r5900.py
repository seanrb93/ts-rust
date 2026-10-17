"""R5900 (PS2 Emotion Engine) instruction decoder, including MMI, COP1 and VU0 macro (COP2)."""

GPR = ['zero', 'at', 'v0', 'v1', 'a0', 'a1', 'a2', 'a3', 't0', 't1', 't2', 't3', 't4', 't5', 't6', 't7',
       's0', 's1', 's2', 's3', 's4', 's5', 's6', 's7', 't8', 't9', 'k0', 'k1', 'gp', 'sp', 'fp', 'ra']

class Ins:
    __slots__ = ('addr', 'w', 'op', 'rs', 'rt', 'rd', 'sa', 'imm', 'tgt', 'dest', 'bc', 'fmt')

    def __init__(s, addr, w):
        s.addr, s.w = addr, w
        s.op = '???'
        s.rs = (w >> 21) & 31
        s.rt = (w >> 16) & 31
        s.rd = (w >> 11) & 31
        s.sa = (w >> 6) & 31
        v = w & 0xFFFF
        s.imm = v - 0x10000 if v & 0x8000 else v     # sign-extended imm16
        s.tgt = (w & 0x3FFFFFF) << 2
        s.dest = (w >> 21) & 0xF    # VU dest mask (x=8 y=4 z=2 w=1)
        s.bc = None
        s.fmt = None

    @property
    def uimm(s):
        return s.w & 0xFFFF

    @property
    def is_branch(s):
        return s.op in BRANCHES

    @property
    def is_likely(s):
        return s.op in LIKELY

    @property
    def branch_target(s):
        if s.op in ('j', 'jal'):
            return (s.addr & 0xF0000000) | s.tgt
        return s.addr + 4 + (s.imm << 2)

    def __repr__(s):
        return f'{s.addr:08x}: {s.disasm()}'

    def disasm(s):
        g = lambda i: '$' + GPR[i]
        o = s.op
        f = s.fmt
        if f == 'nop': return 'nop'
        if f == 'rd,rs,rt': return f'{o} {g(s.rd)}, {g(s.rs)}, {g(s.rt)}'
        if f == 'rd,rt,sa': return f'{o} {g(s.rd)}, {g(s.rt)}, {s.sa}'
        if f == 'rd,rt,rs': return f'{o} {g(s.rd)}, {g(s.rt)}, {g(s.rs)}'
        if f == 'rt,rs,imm': return f'{o} {g(s.rt)}, {g(s.rs)}, {s.imm}'
        if f == 'rt,rs,uimm': return f'{o} {g(s.rt)}, {g(s.rs)}, {s.uimm:#x}'
        if f == 'rt,uimm': return f'{o} {g(s.rt)}, {s.uimm:#x}'
        if f == 'rt,imm(rs)': return f'{o} {g(s.rt)}, {s.imm}({g(s.rs)})'
        if f == 'ft,imm(rs)': return f'{o} $f{s.rt}, {s.imm}({g(s.rs)})'
        if f == 'vt,imm(rs)': return f'{o} $vf{s.rt}, {s.imm}({g(s.rs)})'
        if f == 'rs,rt,off': return f'{o} {g(s.rs)}, {g(s.rt)}, {s.branch_target:#x}'
        if f == 'rs,off': return f'{o} {g(s.rs)}, {s.branch_target:#x}'
        if f == 'off': return f'{o} {s.branch_target:#x}'
        if f == 'tgt': return f'{o} {s.branch_target:#x}'
        if f == 'rs': return f'{o} {g(s.rs)}'
        if f == 'rd': return f'{o} {g(s.rd)}'
        if f == 'rd,rs': return f'{o} {g(s.rd)}, {g(s.rs)}'
        if f == 'rs,rt': return f'{o} {g(s.rs)}, {g(s.rt)}'
        if f == 'rd,rs,rt?': return f'{o} {g(s.rd)}, {g(s.rs)}, {g(s.rt)}'
        if f == 'rt,rd': return f'{o} {g(s.rt)}, {g(s.rd)}'
        if f == 'rt,fs': return f'{o} {g(s.rt)}, $f{s.rd}'
        if f == 'fd,fs,ft': return f'{o} $f{s.sa}, $f{s.rd}, $f{s.rt}'
        if f == 'fd,fs': return f'{o} $f{s.sa}, $f{s.rd}'
        if f == 'fd,ft': return f'{o} $f{s.sa}, $f{s.rt}'
        if f == 'fs,ft': return f'{o} $f{s.rd}, $f{s.rt}'
        if f == 'vu':
            return f'{o}.{s.dest_str()} vf{s.sa}, vf{s.rd}, vf{s.rt}'
        if f == 'sys': return f'{o} {(s.w >> 6) & 0xFFFFF:#x}'
        return f'{o} <{f}> {s.w:08x}'

    def dest_str(s):
        d = s.dest
        return ''.join(c for c, b in zip('xyzw', (8, 4, 2, 1)) if d & b) or '-'

BRANCHES = {'beq', 'bne', 'blez', 'bgtz', 'bltz', 'bgez', 'bltzal', 'bgezal', 'beql', 'bnel', 'blezl', 'bgtzl',
            'bltzl', 'bgezl', 'bltzall', 'bgezall', 'j', 'jal', 'jr', 'jalr', 'bc0f', 'bc0t', 'bc0fl', 'bc0tl',
            'bc1f', 'bc1t', 'bc1fl', 'bc1tl', 'bc2f', 'bc2t', 'bc2fl', 'bc2tl'}
LIKELY = {'beql', 'bnel', 'blezl', 'bgtzl', 'bltzl', 'bgezl', 'bltzall', 'bgezall', 'bc0fl', 'bc0tl', 'bc1fl',
          'bc1tl', 'bc2fl', 'bc2tl'}

SPECIAL = {0: 'sll', 2: 'srl', 3: 'sra', 4: 'sllv', 6: 'srlv', 7: 'srav', 8: 'jr', 9: 'jalr', 10: 'movz', 11: 'movn',
           12: 'syscall', 13: 'break', 15: 'sync', 16: 'mfhi', 17: 'mthi', 18: 'mflo', 19: 'mtlo', 20: 'dsllv',
           22: 'dsrlv', 23: 'dsrav', 24: 'mult', 25: 'multu', 26: 'div', 27: 'divu', 32: 'add', 33: 'addu',
           34: 'sub', 35: 'subu', 36: 'and', 37: 'or', 38: 'xor', 39: 'nor', 40: 'mfsa', 41: 'mtsa', 42: 'slt',
           43: 'sltu', 44: 'dadd', 45: 'daddu', 46: 'dsub', 47: 'dsubu', 48: 'tge', 49: 'tgeu', 50: 'tlt',
           51: 'tltu', 52: 'teq', 54: 'tne', 56: 'dsll', 58: 'dsrl', 59: 'dsra', 60: 'dsll32', 62: 'dsrl32',
           63: 'dsra32'}
REGIMM = {0: 'bltz', 1: 'bgez', 2: 'bltzl', 3: 'bgezl', 16: 'bltzal', 17: 'bgezal', 18: 'bltzall', 19: 'bgezall',
          24: 'mtsab', 25: 'mtsah'}
MMI = {0: 'madd', 1: 'maddu', 4: 'plzcw', 16: 'mfhi1', 17: 'mthi1', 18: 'mflo1', 19: 'mtlo1', 24: 'mult1',
       25: 'multu1', 26: 'div1', 27: 'divu1', 32: 'madd1', 33: 'maddu1', 48: 'pmfhl', 49: 'pmthl', 52: 'psllh',
       54: 'psrlh', 55: 'psrah', 60: 'psllw', 62: 'psrlw', 63: 'psraw'}
MMI0 = {0: 'paddw', 1: 'psubw', 2: 'pcgtw', 3: 'pmaxw', 4: 'paddh', 5: 'psubh', 6: 'pcgth', 7: 'pmaxh',
        8: 'paddb', 9: 'psubb', 10: 'pcgtb', 16: 'paddsw', 17: 'psubsw', 18: 'pextlw', 19: 'ppacw',
        20: 'paddsh', 21: 'psubsh', 22: 'pextlh', 23: 'ppach', 24: 'paddsb', 25: 'psubsb', 26: 'pextlb',
        27: 'ppacb', 30: 'pext5', 31: 'ppac5'}
MMI1 = {1: 'pabsw', 2: 'pceqw', 3: 'pminw', 4: 'padsbh', 5: 'pabsh', 6: 'pceqh', 7: 'pminh', 10: 'pceqb',
        16: 'padduw', 17: 'psubuw', 18: 'pextuw', 20: 'padduh', 21: 'psubuh', 22: 'pextuh', 24: 'paddub',
        25: 'psubub', 26: 'pextub', 27: 'qfsrv'}
MMI2 = {0: 'pmaddw', 2: 'psllvw', 3: 'psrlvw', 4: 'pmsubw', 8: 'pmfhi', 9: 'pmflo', 10: 'pinth', 12: 'pmultw',
        13: 'pdivw', 14: 'pcpyld', 16: 'pmaddh', 17: 'phmadh', 18: 'pand', 19: 'pxor', 20: 'pmsubh',
        21: 'phmsbh', 26: 'pexeh', 27: 'prevh', 28: 'pmulth', 29: 'pdivbw', 30: 'pexew', 31: 'prot3w'}
MMI3 = {0: 'pmadduw', 3: 'psravw', 8: 'pmthi', 9: 'pmtlo', 10: 'pinteh', 12: 'pmultuw', 13: 'pdivuw',
        14: 'pcpyud', 18: 'por', 19: 'pnor', 26: 'pexch', 27: 'pcpyh', 30: 'pexcw'}

COP1_S = {0: 'add.s', 1: 'sub.s', 2: 'mul.s', 3: 'div.s', 4: 'sqrt.s', 5: 'abs.s', 6: 'mov.s', 7: 'neg.s',
          22: 'rsqrt.s', 24: 'adda.s', 25: 'suba.s', 26: 'mula.s', 28: 'madd.s', 29: 'msub.s', 30: 'madda.s',
          31: 'msuba.s', 36: 'cvt.w.s', 40: 'max.s', 41: 'min.s', 48: 'c.f.s', 50: 'c.eq.s', 52: 'c.lt.s',
          54: 'c.le.s'}

# VU upper ops with funct < 0x3c
_vu_bc = ['x', 'y', 'z', 'w']
VU_UP = {}
for i, c in enumerate(_vu_bc):
    VU_UP[0 + i] = 'vadd' + c; VU_UP[4 + i] = 'vsub' + c; VU_UP[8 + i] = 'vmadd' + c; VU_UP[12 + i] = 'vmsub' + c
    VU_UP[16 + i] = 'vmax' + c; VU_UP[20 + i] = 'vmini' + c; VU_UP[24 + i] = 'vmul' + c
VU_UP.update({28: 'vmulq', 29: 'vmaxi', 30: 'vmuli', 31: 'vminii', 32: 'vaddq', 33: 'vmaddq', 34: 'vaddi',
              35: 'vmaddi', 36: 'vsubq', 37: 'vmsubq', 38: 'vsubi', 39: 'vmsubi', 40: 'vadd', 41: 'vmadd',
              42: 'vmul', 43: 'vmax', 44: 'vsub', 45: 'vmsub', 46: 'vopmsub', 47: 'vmini', 48: 'viadd',
              49: 'visub', 50: 'viaddi', 52: 'viand', 53: 'vior', 56: 'vcallms', 57: 'vcallmsr'})
# SPECIAL2 table index = ((w>>6)&0x1f)*4 + (w&3)
VU_SP = {}
for i, c in enumerate(_vu_bc):
    VU_SP[0 + i] = 'vadda' + c; VU_SP[4 + i] = 'vsuba' + c; VU_SP[8 + i] = 'vmadda' + c; VU_SP[12 + i] = 'vmsuba' + c
    VU_SP[24 + i] = 'vmula' + c
VU_SP.update({16: 'vitof0', 17: 'vitof4', 18: 'vitof12', 19: 'vitof15', 20: 'vftoi0', 21: 'vftoi4',
              22: 'vftoi12', 23: 'vftoi15', 28: 'vmulaq', 29: 'vabs', 30: 'vmulai', 31: 'vclipw',
              32: 'vaddaq', 33: 'vmaddaq', 34: 'vaddai', 35: 'vmaddai', 36: 'vsubaq', 37: 'vmsubaq',
              38: 'vsubai', 39: 'vmsubai', 40: 'vadda', 41: 'vmadda', 42: 'vmula', 44: 'vsuba', 45: 'vmsuba',
              46: 'vopmula', 47: 'vnop', 48: 'vmove', 49: 'vmr32', 52: 'vlqi', 53: 'vsqi', 54: 'vlqd',
              55: 'vsqd', 56: 'vdiv', 57: 'vsqrt', 58: 'vrsqrt', 59: 'vwaitq', 60: 'vmtir', 61: 'vmfir',
              62: 'vilwr', 63: 'viswr', 64: 'vrnext', 65: 'vrget', 66: 'vrinit', 67: 'vrxor'})

PRIMARY = {2: 'j', 3: 'jal', 4: 'beq', 5: 'bne', 6: 'blez', 7: 'bgtz', 8: 'addi', 9: 'addiu', 10: 'slti',
           11: 'sltiu', 12: 'andi', 13: 'ori', 14: 'xori', 15: 'lui', 20: 'beql', 21: 'bnel', 22: 'blezl',
           23: 'bgtzl', 24: 'daddi', 25: 'daddiu', 26: 'ldl', 27: 'ldr', 30: 'lq', 31: 'sq', 32: 'lb',
           33: 'lh', 34: 'lwl', 35: 'lw', 36: 'lbu', 37: 'lhu', 38: 'lwr', 39: 'lwu', 40: 'sb', 41: 'sh',
           42: 'swl', 43: 'sw', 44: 'sdl', 45: 'sdr', 46: 'swr', 47: 'cache', 49: 'lwc1', 51: 'pref',
           54: 'lqc2', 55: 'ld', 57: 'swc1', 62: 'sqc2', 63: 'sd'}


def decode(addr, w):
    i = Ins(addr, w)
    op = w >> 26
    if w == 0:
        i.op = 'nop'; i.fmt = 'nop'; return i
    if op == 0:
        fn = w & 63
        n = SPECIAL.get(fn)
        i.op = n or '???'
        if n in ('sll', 'srl', 'sra', 'dsll', 'dsrl', 'dsra', 'dsll32', 'dsrl32', 'dsra32'): i.fmt = 'rd,rt,sa'
        elif n in ('sllv', 'srlv', 'srav', 'dsllv', 'dsrlv', 'dsrav'): i.fmt = 'rd,rt,rs'
        elif n == 'jr': i.fmt = 'rs'
        elif n == 'jalr': i.fmt = 'rd,rs'
        elif n in ('mfhi', 'mflo', 'mfsa'): i.fmt = 'rd'
        elif n in ('mthi', 'mtlo', 'mtsa'): i.fmt = 'rs'
        elif n in ('mult', 'multu', 'div', 'divu'): i.fmt = 'rd,rs,rt?' if n in ('mult', 'multu') else 'rs,rt'
        elif n in ('syscall', 'break', 'sync'): i.fmt = 'sys'
        elif n: i.fmt = 'rd,rs,rt'
        return i
    if op == 1:
        n = REGIMM.get((w >> 16) & 31)
        i.op = n or '???'
        if n in ('mtsab', 'mtsah'): i.fmt = 'rs,imm'
        elif n: i.fmt = 'rs,off'
        return i
    if op in PRIMARY:
        n = PRIMARY[op]; i.op = n
        if n in ('j', 'jal'): i.fmt = 'tgt'
        elif n in ('beq', 'bne', 'beql', 'bnel'): i.fmt = 'rs,rt,off'
        elif n in ('blez', 'bgtz', 'blezl', 'bgtzl'): i.fmt = 'rs,off'
        elif n in ('addi', 'addiu', 'slti', 'sltiu', 'daddi', 'daddiu'): i.fmt = 'rt,rs,imm'
        elif n in ('andi', 'ori', 'xori'): i.fmt = 'rt,rs,uimm'
        elif n == 'lui': i.fmt = 'rt,uimm'
        elif n in ('lwc1', 'swc1'): i.fmt = 'ft,imm(rs)'
        elif n in ('lqc2', 'sqc2'): i.fmt = 'vt,imm(rs)'
        else: i.fmt = 'rt,imm(rs)'
        return i
    if op == 28:   # MMI
        fn = w & 63
        sa = (w >> 6) & 31
        tbl = {8: MMI0, 40: MMI1, 9: MMI2, 41: MMI3}.get(fn)
        if tbl is not None:
            n = tbl.get(sa)
        else:
            n = MMI.get(fn)
        i.op = n or '???'
        i.fmt = 'rd,rs,rt'
        if n in ('mfhi1', 'mflo1', 'pmfhi', 'pmflo'): i.fmt = 'rd'
        elif n in ('mthi1', 'mtlo1', 'pmthi', 'pmtlo'): i.fmt = 'rs'
        elif n in ('div1', 'divu1'): i.fmt = 'rs,rt'
        elif n in ('psllh', 'psrlh', 'psrah', 'psllw', 'psrlw', 'psraw'): i.fmt = 'rd,rt,sa'
        elif n in ('plzcw', 'pabsw', 'pabsh', 'pexeh', 'prevh', 'pexew', 'prot3w', 'pexch', 'pcpyh', 'pexcw',
                   'pmfhl', 'pmthl', 'ppac5', 'pext5'):
            i.fmt = 'rd,rs' if n == 'plzcw' else 'rd,rt'
        return i
    if op == 16:   # COP0
        rs = (w >> 21) & 31
        if rs == 0: i.op = 'mfc0'; i.fmt = 'rt,rd'
        elif rs == 4: i.op = 'mtc0'; i.fmt = 'rt,rd'
        elif rs == 8:
            i.op = ['bc0f', 'bc0t', 'bc0fl', 'bc0tl'][(w >> 16) & 3]; i.fmt = 'off'
        elif rs == 16:
            fn = w & 63
            i.op = {1: 'tlbr', 2: 'tlbwi', 6: 'tlbwr', 8: 'tlbp', 24: 'eret', 56: 'ei', 57: 'di'}.get(fn, '???')
            i.fmt = 'nop'
        return i
    if op == 17:   # COP1
        rs = (w >> 21) & 31
        fn = w & 63
        if rs == 0: i.op = 'mfc1'; i.fmt = 'rt,fs'
        elif rs == 2: i.op = 'cfc1'; i.fmt = 'rt,fs'
        elif rs == 4: i.op = 'mtc1'; i.fmt = 'rt,fs'
        elif rs == 6: i.op = 'ctc1'; i.fmt = 'rt,fs'
        elif rs == 8:
            i.op = ['bc1f', 'bc1t', 'bc1fl', 'bc1tl'][(w >> 16) & 3]; i.fmt = 'off'
        elif rs == 16:
            n = COP1_S.get(fn)
            i.op = n or '???'
            i.fmt = 'fd,fs,ft'
            if n in ('abs.s', 'mov.s', 'neg.s', 'cvt.w.s'): i.fmt = 'fd,fs'
            elif n == 'sqrt.s': i.fmt = 'fd,ft'
            elif n and n.startswith('c.'): i.fmt = 'fs,ft'
            elif n in ('adda.s', 'suba.s', 'mula.s', 'madda.s', 'msuba.s'): i.fmt = 'fs,ft'
        elif rs == 20:
            if fn == 32: i.op = 'cvt.s.w'; i.fmt = 'fd,fs'
        return i
    if op == 18:   # COP2
        rs = (w >> 21) & 31
        if rs == 1: i.op = 'qmfc2'; i.fmt = 'rt,vd'
        elif rs == 2: i.op = 'cfc2'; i.fmt = 'rt,vd'
        elif rs == 5: i.op = 'qmtc2'; i.fmt = 'rt,vd'
        elif rs == 6: i.op = 'ctc2'; i.fmt = 'rt,vd'
        elif rs == 8:
            i.op = ['bc2f', 'bc2t', 'bc2fl', 'bc2tl'][(w >> 16) & 3]; i.fmt = 'off'
        elif rs >= 16:
            fn = w & 63
            if fn >= 0x3C:
                idx = ((w >> 6) & 0x1F) * 4 + (fn & 3)
                i.op = VU_SP.get(idx, '???')
            else:
                i.op = VU_UP.get(fn, '???')
            i.fmt = 'vu'
        return i
    return i


def decode_range(mem, start, end):
    """Decode instructions in [start,end) from a bytes-like guest memory image."""
    import struct
    out = []
    for a in range(start, end, 4):
        out.append(decode(a, struct.unpack_from('<I', mem, a)[0]))
    return out
