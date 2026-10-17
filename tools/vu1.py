"""VU1 microcode tools: extract the program image from the executable's MPG upload stream and disassemble it."""
import struct, sys
from ps2elf import Elf

UCODE_ADDR, UCODE_END = 0x2eeff0, 0x2f2bc0

def extract(ram):
    """Parse the VIF stream at UCODE_ADDR: returns dict addr -> 64-bit instruction (micro-memory is 2048 x 64-bit)."""
    mem = {}
    p = UCODE_ADDR
    while p < UCODE_END:
        code = struct.unpack_from('<I', ram, p)[0]
        cmd = (code >> 24) & 0x7f
        imm = code & 0xffff
        num = (code >> 16) & 0xff
        p += 4
        if cmd == 0x4a:
            n = num or 256
            for k in range(n):
                mem[imm + k] = struct.unpack_from('<Q', ram, p + 8 * k)[0]
            p += 8 * n
        elif cmd == 0x00:
            continue
        elif cmd in (0x14, 0x15, 0x17, 0x01, 0x03, 0x02, 0x04, 0x05, 0x07):
            continue
        else:
            print(f'unexpected VIF cmd {cmd:#x} at {p-4:#x}', file=sys.stderr)
            break
    return mem

if __name__ == '__main__':
    e = Elf(); ram = e.image()
    mem = extract(ram)
    print(len(mem), 'instructions; range', min(mem), max(mem))
    missing = [a for a in range(max(mem) + 1) if a not in mem]
    print('gaps:', len(missing), missing[:20])
