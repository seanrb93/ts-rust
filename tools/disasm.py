"""Disassembler CLI: python dis.py <func name|addr> [...]   with symbolized calls and lui/ori addresses."""
import sys, struct, bisect
from ps2elf import Elf
from r5900 import decode, GPR

e = Elf(); ram = e.image()
funcs = e.functions()
fstarts = [f.addr for f in funcs]
fbyname = {}
for f in funcs: fbyname.setdefault(f.name, f)
objs = sorted([s for s in e.symbols if s.kind == 1 and s.addr], key=lambda s: s.addr)
ostarts = [s.addr for s in objs]

def sym_for(a):
    k = bisect.bisect_right(ostarts, a) - 1
    if k >= 0:
        s = objs[k]
        if s.addr <= a < s.addr + max(s.size, 1):
            return s.name + (f'+{a - s.addr:#x}' if a != s.addr else '')
    k = bisect.bisect_right(fstarts, a) - 1
    if k >= 0 and funcs[k].addr <= a < funcs[k].addr + funcs[k].size:
        return funcs[k].name + (f'+{a - funcs[k].addr:#x}' if a != funcs[k].addr else '')
    return None

def dis(f, out=sys.stdout):
    out.write(f'; {f.name} @ {f.addr:#x} size {f.size}\n')
    regs = {}
    for a in range(f.addr, f.addr + f.size, 4):
        i = decode(a, struct.unpack_from('<I', ram, a)[0])
        cm = ''
        if i.op == 'lui': regs[i.rt] = (i.uimm << 16) & 0xFFFFFFFF
        elif i.op in ('addiu', 'ori') and i.rs in regs:
            v = (regs[i.rs] + (i.imm if i.op == 'addiu' else i.uimm)) & 0xFFFFFFFF
            s = sym_for(v)
            cm = f'  ; {v:#x}' + (f' <{s}>' if s else '')
            if i.rt: regs[i.rt] = v
        elif i.op in ('lw','sw','lwc1','swc1','lb','lbu','lh','lhu','sb','sh','ld','sd','lq','sq') and i.rs in regs:
            v = (regs[i.rs] + i.imm) & 0xFFFFFFFF
            s = sym_for(v)
            cm = f'  ; {v:#x}' + (f' <{s}>' if s else '')
        elif i.op in ('jal', 'j'):
            s = sym_for(i.branch_target)
            cm = f'  ; <{s}>' if s else ''
        elif i.op in ('beq','bne','blez','bgtz','bltz','bgez','beql','bnel'): pass
        out.write(f'{a:08x}: {i.w:08x}  {i.disasm()}{cm}\n')

if __name__ == '__main__':
    for n in sys.argv[1:]:
        f = fbyname.get(n)
        if f is None:
            a = int(n, 0)
            k = bisect.bisect_right(fstarts, a) - 1; f = funcs[k]
        dis(f)
        print()
