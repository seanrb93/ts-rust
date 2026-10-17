import sys, time, collections, struct
from ps2elf import Elf
from decomp import lifter
from decomp.lifter import lift_function, Unsupported
from decomp.analysis import *
from decomp.externs import EXTERNS, DYNAMIC

def load_all():
    e=Elf(); ram=e.image()
    lifter.GP_VALUE=[s for s in e.symbols if s.name=='_gp'][0].addr
    fs=e.functions()
    split=[k for k,f in enumerate(fs) if f.name=='text_data_split'][0]
    funcs={}; failed={}
    for f in fs[:split]:
        try:
            fn=lift_function(f,ram,None); convert(fn); build_cfg(fn); funcs[f.addr]=fn
        except Unsupported as ex: failed[f.addr]=(f.name,str(ex))
    sigs={}
    for f in fs[split:]:
        if f.name in EXTERNS:
            ni,nf,rf=EXTERNS[f.name]
            sigs[f.addr]=Sig(f.name,ARG_G[:ni]+ARG_F[:nf],[],True,False,rf)
        else:
            sigs[f.addr]=Sig(f.name,[],[],True,f.name in DYNAMIC or True,False)
    for f in fs[:split]:
        if f.addr not in funcs and f.addr not in sigs:
            sigs[f.addr]=Sig(f.name,[],[],True,True,False)   # manual functions: unknown sig for now
    return e,ram,fs,split,funcs,failed,sigs

if __name__=='__main__':
    t=time.time()
    e,ram,fs,split,funcs,failed,sigs=load_all()
    print('lifted',len(funcs),'in',round(time.time()-t,1),'s')
    # address-taken: words in data referencing function starts
    starts={f.addr for f in fs[:split]}
    import numpy
