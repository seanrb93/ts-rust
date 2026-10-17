import struct, collections, bisect
from ps2elf import Elf
from r5900 import decode
e=Elf(); ram=e.image()
fs=e.functions()
split=[k for k,f in enumerate(fs) if f.name=='text_data_split'][0]
starts=[f.addr for f in fs]
byaddr={f.addr:f for f in fs}
ext=collections.Counter(); fp=collections.Counter()
for f in fs[:split]:
    for a in range(f.addr,f.addr+f.size,4):
        i=decode(a,struct.unpack_from('<I',ram,a)[0])
        if i.op in('jal','j'):
            t=i.branch_target
            g=byaddr.get(t)
            if g is None: ext['?%x'%t]+=1
            elif fs.index(g)>=split if False else g.addr>=fs[split].addr: ext[g.name]+=1
print(len(ext), sum(ext.values()))
print(sorted(ext.items(), key=lambda kv:-kv[1]))
