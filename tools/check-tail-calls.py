"""List the tail calls of the original (a `j` to the start of another function) that the hand-written function of the same name does not mention.  The lifted listing of the
generated code leaves a tail call out (the function just returns), so a transcription can lose it: `specialPropTick` lost the call for the fan shadow that way.  A name in the list
may still be right (the call is in a helper); look at each.  Run from `tools` with `Game Files/SLUS_200.90` in place.
"""
import sys,re,glob,struct
sys.argv=['x']
from ps2elf import Elf
e=Elf(); ram=e.image(); fs=sorted(e.functions(),key=lambda f:f.addr)
starts={f.addr:f for f in fs}
def word(a): return struct.unpack('<I',ram[a:a+4])[0] if not hasattr(ram,'read') else None
import os
src={}
for p in glob.glob('../ts-rust/crates/ts-game/src/game/*.rs'):
    t=open(p,encoding='utf8').read()
    for m in re.finditer(r'^pub fn (\w+)\(.*?^}\n',t,re.M|re.S):
        src[m.group(1)]=m.group(0)
res=[]
for f in fs:
    if f.name not in src: continue
    body=src[f.name]
    for a in range(f.addr,f.addr+f.size,4):
        try: w=struct.unpack('<I',bytes(ram[a:a+4]))[0]
        except Exception: continue
        if w>>26==2:
            t=((w&0x3ffffff)<<2)|(a&0xf0000000)
            if (t<f.addr or t>=f.addr+f.size) and t in starts:
                cal=starts[t].name
                if cal not in body: res.append((f.name,hex(a),cal))
print(len(res))
for r in res: print(*r)
