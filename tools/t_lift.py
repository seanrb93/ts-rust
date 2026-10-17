import sys, collections
from ps2elf import Elf
from decomp import lifter
from decomp.lifter import lift_function, Unsupported
e=Elf(); ram=e.image()
lifter.GP_VALUE = [s for s in e.symbols if s.name=='_gp'][0].addr
fs=e.functions()
split=[k for k,f in enumerate(fs) if f.name=='text_data_split'][0]
game=fs[:split]
ok=0; bad=collections.defaultdict(list)
for f in game:
    try:
        lift_function(f, ram, None); ok+=1
    except Unsupported as ex:
        bad[str(ex)].append(f.name)
    except Exception as ex:
        bad['EXC '+type(ex).__name__+':'+str(ex)[:60]].append(f.name)
print('ok',ok,'of',len(game))
for k,v in sorted(bad.items(), key=lambda kv:-len(kv[1])): print(len(v),k,v[:8])
