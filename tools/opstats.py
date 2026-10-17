import struct, collections
from ps2elf import Elf
from r5900 import decode, MMI, MMI0, MMI1, MMI2, MMI3, VU_UP, VU_SP
e=Elf(); ram=e.image()
fs=e.functions()
mmi=set(MMI.values())|set(MMI0.values())|set(MMI1.values())|set(MMI2.values())|set(MMI3.values())
vu=set(VU_UP.values())|set(VU_SP.values())|{'qmfc2','qmtc2','cfc2','ctc2','lqc2','sqc2','bc2f','bc2t','bc2fl','bc2tl'}
cnt=collections.Counter(); unk=collections.Counter()
feat=collections.defaultdict(set)
LIB=0x2cb900  # approx start of sce libs (ilink... then text_data_split)
for f in fs:
    for a in range(f.addr,f.addr+f.size,4):
        i=decode(a,struct.unpack_from('<I',ram,a)[0])
        cnt[i.op]+=1
        if i.op=='???': unk[f'{i.w:08x}']+=1
        g='game' if f.addr<LIB else 'lib'
        if i.op in mmi: feat[(g,'mmi')].add(f.name)
        if i.op in vu: feat[(g,'vu0')].add(f.name)
        if i.op in('lq','sq'): feat[(g,'lq/sq')].add(f.name)
        if i.op=='jr' and i.rs!=31: feat[(g,'jr-nonra')].add(f.name)
        if i.op=='jalr': feat[(g,'jalr')].add(f.name)
        if i.op in('beql','bnel','blezl','bgtzl','bltzl','bgezl','bc1fl','bc1tl'): feat[(g,'likely')].add(f.name)
        if i.op in('ldl','ldr','lwl','lwr','swl','swr','sdl','sdr'): feat[(g,'unaligned')].add(f.name)
        if i.op in('mfc0','mtc0','syscall','cache','di','ei','eret','tlbr'): feat[(g,'sys')].add(f.name)
        if i.op in('mult','multu','div','divu','mfhi','mflo','mthi','mtlo','madd','maddu'): feat[(g,'hilo')].add(f.name)
        if i.op in('dadd','daddu','dsub','dsubu','dsll','dsrl','dsra','dsll32','dsrl32','dsra32','daddiu','ld','sd'): feat[(g,'64bit')].add(f.name)
print('total ins',sum(cnt.values()))
print('top ops',cnt.most_common(70))
print('unknown encodings',unk.most_common(20), sum(unk.values()))
for k,v in sorted(feat.items()): print(k,len(v))
print('vu0 game funcs:',sorted(n for (g,k),v in feat.items() if g=='game' and k=='vu0' for n in v))
print('mmi game funcs:',sorted(n for (g,k),v in feat.items() if g=='game' and k=='mmi' for n in v))
print('jalr game funcs:',sorted(n for (g,k),v in feat.items() if g=='game' and k=='jalr' for n in v))
