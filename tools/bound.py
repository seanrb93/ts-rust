from ps2elf import Elf
e=Elf(); fs=e.functions()
i=[k for k,f in enumerate(fs) if f.name=='text_data_split'][0]
print('text_data_split idx',i,hex(fs[i].addr),'of',len(fs))
after=fs[i:]
print('after split count',len(after),'bytes',sum(f.size for f in after))
print('before split bytes',sum(f.size for f in fs[:i]))
print(' '.join(f.name for f in after[:400]))
for s in e.symbols:
    if s.name in('_gp','_ftext','_etext','_fdata','_edata','_fbss','_end','_stack','_stack_size','_heap_size','__bss_start','_SDA_BASE_'): print(s)
