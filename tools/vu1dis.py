"""VU1 micro-instruction decoder / disassembler (dual-issue: upper = FMAC half, lower = integer/LS/branch/EFU half)."""
import sys
from ps2elf import Elf
from vu1 import extract
from r5900 import VU_UP, VU_SP

BC = 'xyzw'


def dest(u):
    return ''.join(c for c, b in zip('xyzw', (24, 23, 22, 21)) if u >> b & 1) or '-'


def upper(u):
    """Return (mnemonic, operand-string, meta) for the upper half."""
    fn = u & 63
    ft, fs, fd = (u >> 16) & 31, (u >> 11) & 31, (u >> 6) & 31
    d = dest(u)
    if fn >= 0x3c:
        idx = ((u >> 6) & 31) * 4 + (u & 3)
        name = VU_SP.get(idx, f'?sp{idx}')
        nm = name[1:].upper()
        if nm == 'NOP':
            return 'NOP', '', None
        if nm.startswith('ITOF') or nm.startswith('FTOI') or nm == 'ABS':
            return f'{nm}.{d}', f'vf{ft}, vf{fs}', None
        if nm == 'CLIPW':
            return 'CLIP.xyz', f'vf{fs}, vf{ft}w', None
        # accumulator forms: result into ACC
        bcc = ''
        if nm[-1] in 'XYZW' and nm not in ('OPMULA',):
            bcc = nm[-1].lower()
            nm = nm[:-1]
            return f'{nm}.{d}', f'ACC, vf{fs}, vf{ft}{bcc}', None
        if nm.endswith('Q'):
            return f'{nm[:-1]}Q.{d}', f'ACC, vf{fs}, Q', None
        if nm.endswith('I'):
            return f'{nm[:-1]}I.{d}', f'ACC, vf{fs}, I', None
        return f'{nm}.{d}', f'ACC, vf{fs}, vf{ft}', None
    name = VU_UP.get(fn, f'?up{fn:x}')
    nm = name[1:].upper()
    if nm == 'OPMSUB':
        return 'OPMSUB.xyz', f'vf{fd}, vf{fs}, vf{ft}', None
    for suffix, ops in (('X', f'vf{ft}x'), ('Y', f'vf{ft}y'), ('Z', f'vf{ft}z'), ('W', f'vf{ft}w')):
        if nm.endswith(suffix) and nm[:-1] in ('ADD', 'SUB', 'MADD', 'MSUB', 'MAX', 'MINI', 'MUL'):
            return f'{nm[:-1]}{suffix.lower()}.{d}', f'vf{fd}, vf{fs}, {ops}', None
    if nm.endswith('Q'):
        return f'{nm}.{d}', f'vf{fd}, vf{fs}, Q', None
    if nm.endswith('I') and nm in ('ADDI', 'SUBI', 'MULI', 'MADDI', 'MSUBI', 'MAXI', 'MINII'):
        return f'{nm}.{d}', f'vf{fd}, vf{fs}, I', None
    return f'{nm}.{d}', f'vf{fd}, vf{fs}, vf{ft}', None


def s11(v):
    return v - 0x800 if v & 0x400 else v


def sx11(l):
    v = l & 0x7ff
    return v - 0x800 if v & 0x400 else v


LOWER_A = {0x00: 'LQ', 0x01: 'SQ', 0x04: 'ILW', 0x05: 'ISW', 0x08: 'IADDIU', 0x09: 'ISUBIU', 0x10: 'FCEQ', 0x11: 'FCSET',
           0x12: 'FCAND', 0x13: 'FCOR', 0x14: 'FSEQ', 0x15: 'FSSET', 0x16: 'FSAND', 0x17: 'FSOR', 0x18: 'FMEQ',
           0x1a: 'FMAND', 0x1b: 'FMOR', 0x1c: 'FCGET', 0x20: 'B', 0x21: 'BAL', 0x24: 'JR', 0x25: 'JALR', 0x28: 'IBEQ',
           0x29: 'IBNE', 0x2c: 'IBLTZ', 0x2d: 'IBGTZ', 0x2e: 'IBLEZ', 0x2f: 'IBGEZ'}

# funct 0x3c..0x3f, row = bits 10:6
LOWER_SP = {
    (0x3c, 0x0c): 'MOVE', (0x3c, 0x0d): 'LQI', (0x3c, 0x0e): 'DIV', (0x3c, 0x0f): 'MTIR', (0x3c, 0x10): 'RNEXT',
    (0x3c, 0x19): 'MFP', (0x3c, 0x1a): 'XTOP', (0x3c, 0x1b): 'XGKICK', (0x3c, 0x1c): 'ESADD', (0x3c, 0x1d): 'EATANxy',
    (0x3c, 0x1e): 'ESQRT', (0x3c, 0x1f): 'ESIN',
    (0x3d, 0x0c): 'MR32', (0x3d, 0x0d): 'SQI', (0x3d, 0x0e): 'SQRT', (0x3d, 0x0f): 'MFIR', (0x3d, 0x10): 'RGET',
    (0x3d, 0x1a): 'XITOP', (0x3d, 0x1c): 'ERSADD', (0x3d, 0x1d): 'EATANxz', (0x3d, 0x1e): 'ERSQRT',
    (0x3d, 0x1f): 'EATAN',
    (0x3e, 0x0d): 'LQD', (0x3e, 0x0e): 'RSQRT', (0x3e, 0x0f): 'ILWR', (0x3e, 0x10): 'RINIT', (0x3e, 0x1c): 'ELENG',
    (0x3e, 0x1d): 'ESUM', (0x3e, 0x1e): 'ERCPR', (0x3e, 0x1f): 'EEXP',
    (0x3f, 0x0d): 'SQD', (0x3f, 0x0e): 'WAITQ', (0x3f, 0x0f): 'ISWR', (0x3f, 0x10): 'RXOR', (0x3f, 0x1c): 'ERLENG',
}


def lower(l, pc):
    """Return (mnemonic, operands, info dict)."""
    op = (l >> 25) & 0x7f
    ft, fs = (l >> 16) & 31, (l >> 11) & 31
    d = dest(l)
    imm11 = sx11(l)
    if op in LOWER_A:
        nm = LOWER_A[op]
        if nm == 'LQ':
            return f'LQ.{d}', f'vf{ft}, {imm11}(vi{fs})', {}
        if nm == 'SQ':
            return f'SQ.{d}', f'vf{fs}, {imm11}(vi{ft})', {}
        if nm in ('ILW', 'ISW'):
            return f'{nm}.{d}', f'vi{ft}, {imm11}(vi{fs})', {}
        if nm in ('IADDIU', 'ISUBIU'):
            imm = ((l >> 21) & 15) << 11 | (l & 0x7ff)
            return nm, f'vi{ft}, vi{fs}, {imm}', {}
        if nm in ('FCEQ', 'FCSET', 'FCAND', 'FCOR'):
            return nm, f'{l & 0xffffff:#x}', {}
        if nm in ('FSEQ', 'FSSET', 'FSAND', 'FSOR'):
            imm = ((l >> 21) & 1) << 11 | (l & 0x7ff)
            return nm, f'vi{ft}, {imm:#x}', {}
        if nm in ('FMEQ', 'FMAND', 'FMOR'):
            return nm, f'vi{ft}, vi{fs}', {}
        if nm == 'FCGET':
            return nm, f'vi{ft}', {}
        if nm == 'B':
            return 'B', f'{pc + 1 + imm11}', {'br': pc + 1 + imm11}
        if nm == 'BAL':
            return 'BAL', f'vi{ft}, {pc + 1 + imm11}', {'br': pc + 1 + imm11}
        if nm == 'JR':
            return 'JR', f'vi{fs}', {}
        if nm == 'JALR':
            return 'JALR', f'vi{ft}, vi{fs}', {}
        if nm in ('IBEQ', 'IBNE'):
            return nm, f'vi{ft}, vi{fs}, {pc + 1 + imm11}', {'br': pc + 1 + imm11}
        return nm, f'vi{fs}, {pc + 1 + imm11}', {'br': pc + 1 + imm11}
    if op == 0x40:
        fn = l & 63
        if fn in (0x30, 0x31, 0x34, 0x35):
            nm = {0x30: 'IADD', 0x31: 'ISUB', 0x34: 'IAND', 0x35: 'IOR'}[fn]
            fd = (l >> 6) & 31
            return nm, f'vi{fd}, vi{fs}, vi{ft}', {}
        if fn == 0x32:
            imm5 = (l >> 6) & 31
            imm5 = imm5 - 32 if imm5 & 16 else imm5
            return 'IADDI', f'vi{ft}, vi{fs}, {imm5}', {}
        if fn >= 0x3c:
            row = (l >> 6) & 31
            nm = LOWER_SP.get((fn, row))
            if nm is None:
                return f'?L{fn:x}.{row:x}', f'raw={l:08x}', {}
            fsf, ftf = (l >> 21) & 3, (l >> 23) & 3
            if nm in ('MOVE', 'MR32'):
                if d == '-':
                    return 'NOP', '', {}
                return f'{nm}.{d}', f'vf{ft}, vf{fs}', {}
            if nm in ('LQI', 'LQD'):
                return f'{nm}.{d}', f'vf{ft}, ' + (f'(vi{fs}++)' if nm == 'LQI' else f'(--vi{fs})'), {}
            if nm in ('SQI', 'SQD'):
                return f'{nm}.{d}', f'vf{fs}, ' + (f'(vi{ft}++)' if nm == 'SQI' else f'(--vi{ft})'), {}
            if nm in ('DIV', 'RSQRT'):
                return nm, f'Q, vf{fs}{BC[fsf]}, vf{ft}{BC[ftf]}', {}
            if nm == 'SQRT':
                return nm, f'Q, vf{ft}{BC[ftf]}', {}
            if nm == 'MTIR':
                return nm, f'vi{ft}, vf{fs}{BC[fsf]}', {}
            if nm == 'MFIR':
                return f'{nm}.{d}', f'vf{ft}, vi{fs}', {}
            if nm in ('ILWR',):
                return f'{nm}.{d}', f'vi{ft}, (vi{fs})', {}
            if nm in ('ISWR',):
                return f'{nm}.{d}', f'vi{ft}, (vi{fs})', {}
            if nm in ('RNEXT', 'RGET'):
                return f'{nm}.{d}', f'vf{ft}, R', {}
            if nm in ('RINIT', 'RXOR'):
                return nm, f'R, vf{fs}{BC[fsf]}', {}
            if nm == 'MFP':
                return f'{nm}.{d}', f'vf{ft}, P', {}
            if nm in ('XTOP', 'XITOP'):
                return nm, f'vi{ft}', {}
            if nm == 'XGKICK':
                return nm, f'vi{fs}', {'kick': True}
            if nm == 'WAITQ':
                return nm, '', {}
            return nm, f'P, vf{fs}', {}
    return f'?L{op:x}', f'raw={l:08x}', {}


def disasm_range(mem, start, end):
    out = []
    for pc in range(start, end):
        w = mem.get(pc)
        if w is None:
            out.append((pc, None))
            continue
        lo, hi = w & 0xffffffff, w >> 32
        flags = []
        if hi >> 31 & 1:
            flags.append('I')
        if hi >> 30 & 1:
            flags.append('E')
        if hi >> 29 & 1:
            flags.append('M')
        if hi >> 28 & 1:
            flags.append('D')
        if hi >> 27 & 1:
            flags.append('T')
        um, uo, _ = upper(hi)
        if hi >> 31 & 1:
            lm, lo_s = 'LOI', f'{__import__("struct").unpack("<f", __import__("struct").pack("<I", lo))[0]:.6g}'
            info = {}
        else:
            lm, lo_s, info = lower(lo, pc)
        out.append((pc, (um, uo, lm, lo_s, ''.join(flags), info)))
    return out


def show(mem, start, end, file=sys.stdout):
    targets = set()
    rows = disasm_range(mem, start, end)
    for pc, r in rows:
        if r and 'br' in r[5]:
            targets.add(r[5]['br'])
    for pc, r in rows:
        if r is None:
            continue
        um, uo, lm, lo_s, fl, info = r
        lbl = f'L{pc:03x}:' if pc in targets else ''
        file.write(f'{pc:04x} {lbl:6} {um:<10} {uo:<26} | {lm:<9} {lo_s:<30} {fl}\n')


if __name__ == '__main__':
    e = Elf()
    mem = extract(e.image())
    a = int(sys.argv[1], 0)
    b = int(sys.argv[2], 0)
    show(mem, a, b)
