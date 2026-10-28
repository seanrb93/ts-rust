"""Write the game's initialised data and its symbol names out of the original executable, into the repository, so that the port and the tools need no copy of `SLUS_200.90`.

    python tools/extract-game-data.py <SLUS_200.90>

writes (in ts-rust/crates/ts-rt/data/):

    game-data.bin    the bytes of the executable's memory image from DATA_START to DATA_END: the read-only data, the data and the small data (strings, tables, defaults, the
                     graphics microcode).  The code of the executable is not in it: every function of the game is Rust.
    symbols.tsv      the names and addresses of the executable's symbols (what `patches.txt` of a mod may name)

and checks that the program's loader constants (`ts-rt/src/loader.rs`) agree with them.  Run it again only if the decompilation needs more of the image.
"""
import os
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import ps2elf  # noqa: E402

DATA_START = 0x2eeff0
DATA_END = 0x3afe00    # the end of the initialised small data; what follows (.sbss, .bss) is zero when the game starts
OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'ts-rust', 'crates', 'ts-rt', 'data')


def main():
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    elf = ps2elf.Elf(sys.argv[1])
    ram = elf.image()
    os.makedirs(OUT, exist_ok=True)
    blob = bytes(ram[DATA_START:DATA_END])
    open(os.path.join(OUT, 'game-data.bin'), 'wb').write(blob)
    lines = []
    for s in elf.symbols:
        if s.name and s.kind in (1, 2) and s.addr:
            lines.append(f'{s.name}\t{s.addr:#x}\t{s.size}\t{"f" if s.kind == 2 else "d"}')
    open(os.path.join(OUT, 'symbols.tsv'), 'w', encoding='utf8').write('\n'.join(lines) + '\n')
    print(f'game-data.bin: {len(blob)} bytes ({DATA_START:#x}..{DATA_END:#x}); symbols.tsv: {len(lines)} symbols')


if __name__ == '__main__':
    main()
