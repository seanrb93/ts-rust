from ps2elf import Elf
import collections
e=Elf()
print('entry',hex(e.entry))
for s in e.segments: print('seg va=%x off=%x fsz=%x msz=%x fl=%x'%s)
fs=e.functions()
print('funcs',len(fs),'code bytes',sum(f.size for f in fs))
objs=[s for s in e.symbols if s.kind==1 and s.addr]
print('objects',len(objs),'data bytes',sum(o.size for o in objs))
print('kinds',collections.Counter(s.kind for s in e.symbols))
print('first/last func',hex(fs[0].addr),hex(fs[-1].addr+fs[-1].size))
print([f.name for f in fs if 0x2cb000<=f.addr<=0x2cf000][:70])
print('data syms sample',[ (hex(o.addr),o.size,o.name) for o in sorted(objs,key=lambda o:o.addr)[::120]])
for i,s in enumerate(e.sections): print(i,'type',s[1],'addr %x off %x size %x'%(s[3],s[4],s[5]))
