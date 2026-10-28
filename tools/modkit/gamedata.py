"""What the game's own tables say, read from the game's program file: weapons, characters, their names."""
import os
import struct

GUNS, GUN_BYTES, GAME_GUNS = 0x366218, 400, 30
CHRS, CHR_BYTES, GAME_CHARS = 0x3650e0, 44, 80
GUNCOMBOS, GAME_COMBOS = 0x3690f8, 51 - 2
LANGUAGES = 0x2f2c18
AMMOINFO = 0x3661a0
AMMO_NAMES = ['none', 'pistol', 'rifle', 'shotgun', 'plasma', 'energy', 'explosive', 'special']


class GameData:
    def __init__(self, elf_path):
        import ps2elf
        self.ram = ps2elf.Elf(elf_path).image()

    def u32(self, at):
        return struct.unpack_from('<I', self.ram, at)[0]

    def i32(self, at):
        return struct.unpack_from('<i', self.ram, at)[0]

    def f32(self, at):
        return struct.unpack_from('<f', self.ram, at)[0]

    def u16(self, at):
        return struct.unpack_from('<H', self.ram, at)[0]

    def text(self, n):
        table = self.u32(LANGUAGES)
        p = self.u32(table + 4 * n)
        out = bytearray()
        while 0 < p < len(self.ram) and self.ram[p] and len(out) < 60:
            out.append(self.ram[p])
            p += 1
        return out.decode('latin-1')

    def weapon_name(self, n):
        """The name the game shows for weapon `n` (from its single combination)."""
        best = None
        for k in range(1, 60):
            at = GUNCOMBOS + 8 * k
            left, right, text = struct.unpack_from('<hhH', self.ram, at + 2)
            if right == n and (left == -1 or (left == right and best is None)):
                if left == -1:
                    return self.text(text)
                best = text
        return self.text(best) if best is not None else f'weapon {n}'

    def weapons(self):
        out = []
        for n in range(GAME_GUNS):
            at = GUNS + n * GUN_BYTES
            out.append({
                'number': n, 'name': self.weapon_name(n),
                'ammo_kind': self.i32(at + 16), 'clip': self.i32(at + 20), 'ammo_reserve': self.i32(at + 24),
                'damage': self.f32(at + 72), 'spread': self.f32(at + 84), 'recoil': self.f32(at + 108),
                'bullet_speed': self.f32(at + 80), 'fire_sound': self.i32(at + 96),
            })
        return out

    def character_name(self, n):
        return self.text(self.u16(CHRS + n * CHR_BYTES))

    def characters(self):
        return [{'number': n, 'name': self.character_name(n)} for n in range(GAME_CHARS)]
