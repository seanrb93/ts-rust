"""Minimal PS2 EE ELF loader: program image, symbols."""
import struct, os

GAME_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'Game Files')
ELF_PATH = os.path.join(GAME_DIR, 'SLUS_200.90')

class Sym:
    __slots__ = ('name', 'addr', 'size', 'kind', 'bind')
    def __init__(s, name, addr, size, kind, bind):
        s.name, s.addr, s.size, s.kind, s.bind = name, addr, size, kind, bind
    def __repr__(s):
        return f'Sym({s.name!r},{s.addr:#x},{s.size},{s.kind})'

class Elf:
    def __init__(self, path=ELF_PATH):
        d = self.data = open(path, 'rb').read()
        (_, typ, mach, _, self.entry, phoff, shoff, _, _, phentsize, phnum, shentsize, shnum, shstrndx) = \
            struct.unpack('<16sHHIIIIIHHHHHH', d[:52])
        self.segments = []
        for i in range(phnum):
            t, off, va, pa, fsz, msz, fl, al = struct.unpack('<8I', d[phoff + i * 32:phoff + i * 32 + 32])
            if t == 1:
                self.segments.append((va, off, fsz, msz, fl))
        secs = [struct.unpack('<10I', d[shoff + i * 40:shoff + i * 40 + 40]) for i in range(shnum)]
        self.sections = secs
        # symtab is type 2, strtab (type 3) linked to it
        self.symbols = []
        for s in secs:
            if s[1] == 2:
                symoff, symsz, link = s[4], s[5], s[6]
                stroff = secs[link][4]
                for i in range(symsz // 16):
                    n, v, sz, info, oth, sh = struct.unpack('<IIIBBH', d[symoff + i * 16:symoff + i * 16 + 16])
                    e = d.index(b'\0', stroff + n)
                    self.symbols.append(Sym(d[stroff + n:e].decode('latin1'), v, sz, info & 15, info >> 4))
        self.text = [s for s in secs[1:2]][0]  # first section is .text-like
        # image: sparse dict of bytes via segment list
    def image(self, ramsize=0x2000000):
        """Return bytearray of guest RAM with segments loaded."""
        ram = bytearray(ramsize)
        for va, off, fsz, msz, fl in self.segments:
            ram[va:va + fsz] = self.data[off:off + fsz]
        return ram
    def functions(self):
        fs = [s for s in self.symbols if s.kind == 2 and s.addr]
        fs.sort(key=lambda s: s.addr)
        return fs

if __name__ == '__main__':
    e = Elf()
    print(hex(e.entry), e.segments)
    print(len(e.functions()))
