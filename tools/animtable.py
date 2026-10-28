"""List which animation file is which animation number, and the skeleton files, from the game's own tables.

    python tools/animtable.py <game folder or SLUS_200.90> [--game-src ts-rust/crates/ts-game/src/game/animation-loading.rs]

`animMake` (animation-loading.rs) loads the animation files with tables of (the address of the file's name, the number it is loaded as); this reads those tables and the names from the program file.
Prints one line for each: `number  file`.  The numbers are what the game's code asks for: a character plays the animation `n`, shifted by the character's variant for the classes of 11 (see model-format.md).
"""
import os
import re
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import ps2elf  # noqa: E402

SRC = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'ts-rust', 'crates', 'ts-game', 'src', 'game', 'animation-loading.rs')


def cstr(ram, at):
    end = ram.index(b'\0', at)
    return bytes(ram[at:end]).decode('latin1')


def main():
    src = open(sys.argv[sys.argv.index('--game-src') + 1] if '--game-src' in sys.argv else SRC, encoding='utf-8').read()
    ram = ps2elf.data_image()
    rows = []
    for m in re.finditer(r'const ANIMATION_FILES_\d+: \[\(i32, i32\); \d+\] = \[(.*?)\];', src, re.S):
        for a, i in re.findall(r'\((0x[0-9a-f]+|\d+), (\d+)\)', m.group(1)):
            rows.append((int(i), cstr(ram, int(a, 0))))
    rows.sort()
    for n, name in rows:
        print(f'{n:5d}  {name}')
    print(f'{len(rows)} animation files', file=sys.stderr)
    for k, at in enumerate((0x39b590, 0x39b5b8)):
        print(f'skeleton {k}: {cstr(ram, at)}')


if __name__ == '__main__':
    main()
