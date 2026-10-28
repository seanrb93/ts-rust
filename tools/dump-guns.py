"""List the game's table of weapons with the names of the fields that weapons.txt knows (see ts-port/src/moddata.rs).

    python tools/dump-guns.py <game folder or SLUS_200.90>      prints one line for each weapon
    python tools/dump-guns.py <game folder> --raw               prints every word of every entry

The values are the game's starting values, read from the program file (the table is 30 entries of 400 bytes at 0x366218).
"""
import os
import struct
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import ps2elf  # noqa: E402

GUNS, SIZE, COUNT = 0x366218, 400, 30
INT = {'ammo_kind': 16, 'clip': 20, 'ammo_reserve': 24, 'flags': 28, 'bullet_type': 52, 'alt_bullet_type': 188}
FLOAT = {'view_x': 32, 'view_y': 36, 'view_z': 40, 'fov_a': 44, 'fov_b': 48, 'bullet_speed': 80, 'damage': 72, 'ricochet': 68, 'spread': 84, 'recoil': 108, 'kick': 112, 'pull': 124}


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    path = sys.argv[1]
    if os.path.isdir(path):
        path = os.path.join(path, 'SLUS_200.90')
    if not os.path.exists(path):
        sys.exit(f'{path} does not exist')
    ram = ps2elf.Elf(path).image()
    raw = '--raw' in sys.argv
    for n in range(COUNT):
        at = GUNS + n * SIZE
        if raw:
            words = struct.unpack_from('<100I', ram, at)
            print(f'weapon {n}: ' + ' '.join(f'+{4 * i}={w:#x}' for i, w in enumerate(words) if w))
            continue
        ints = ' '.join(f'{k}={struct.unpack_from("<i", ram, at + o)[0]}' for k, o in INT.items())
        floats = ' '.join(f'{k}={struct.unpack_from("<f", ram, at + o)[0]:.3g}' for k, o in FLOAT.items())
        print(f'weapon {n:2d}: {ints}  {floats}')


if __name__ == '__main__':
    main()
