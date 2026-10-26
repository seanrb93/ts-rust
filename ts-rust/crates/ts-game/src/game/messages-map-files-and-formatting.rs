//! Hud messages, the directory of map files, map maker objects, formatted times and names, the sine table and a fixed point divide.

use crate::*;

/// Set the message of the hud of player `player` (records of 352 bytes at `hudparms`): the text `text` (up to 64 characters, at +284) and its flag at +348, or no message when
/// `text` is 0 (@ 0x2b0990).
pub fn hudMessage(player: i32, text: i32) {
    let record = player * 352 + g::hudparms;
    if text != 0 {
        strncpy(record + 284, text, 64);
        sb(record + 347, 0);
        sw(record + 348, 1);
    } else {
        sw(record + 348, 0);
    }
}

/// Clear the directory of map files and its two counts (@ 0x253918).
pub fn mmfileDirReset() {
    let size = mmfileDirSize();
    memset(lw(g::mapdir1), 0, size);
    let directory = lw(g::mapdir1);
    sw(directory + 8, 0);
    sw(directory + 4, 0);
}

/// Take entry `entry` (40 bytes each, from +12) out of the directory of map files: the entries after it move down (@ 0x253b20).
pub fn mmfileEntryDelete(entry: i32) {
    let mut directory = lw(g::mapdir1);
    let count = lw(directory + 8);
    if (entry as u32) < (count as u32) {
        if entry < count - 1 {
            let at = entry * 40 + directory;
            memcpy(at + 12, at + 52, (count - entry - 1) * 40);
            directory = lw(g::mapdir1);
        }
        sw(directory + 8, lw(directory + 8) - 1);
    }
}

/// Take a map maker object from the free list (0x3b0014, linked at +56): it is cleared (60 bytes) and the number of free ones goes down; 0 if none (@ 0x248650).
pub fn mmAllocObject() -> i32 {
    let object = lw(0x3b0014);
    if object != 0 {
        sw(0x3b0014, lw(object + 56));
        memset(object, 0, 60);
        sw(g::numfreeobjs, lw(g::numfreeobjs) - 1);
    }
    object
}

/// A string is looked at to its end (the length is taken and counted down to 0); nothing is done with it (@ 0x201368).
pub fn checkstring(text: i32) {
    if text != 0 {
        let mut length = strlen(text);
        if length > 0 {
            loop {
                length -= 1;
                if length == 0 {
                    break;
                }
            }
        }
    }
}

/// The time of a bag tag, in 60ths of a second, as text hours (up to 99), minutes, seconds and tenths (format at 0x3a0790); the word of the language for no time when it is 0 (@ 0x21ca10).
pub fn gameFormatBagTagTime(ticks: i32) -> i32 {
    let rest = ticks.wrapping_rem(60);
    let tenths = rest.wrapping_div(6);
    if ticks != 0 {
        let hours = ticks.wrapping_div(60).wrapping_div(60);
        let hours = if hours >= 100 { 99 } else { hours };
        let minutes = ticks.wrapping_div(60).wrapping_rem(60);
        vafmt(&[0x3a0790, hours as i64, minutes as i64, tenths as i64, rest as i64])
    } else {
        lw(lw((lw(g::cur_language) << 2) + g::languages) + 1760)
    }
}

/// The title of a player's dialog: the title `title` and, with two players or more, the player's number (`player` + 1) in the language's format (@ 0x2315b0).
pub fn GetPlayerDialogTitle(player: i32, title: i32) -> i32 {
    let buffer = 0x1fc0a98;
    strcpy(buffer, title);
    if lw(0x3aff20) >= 2 {
        let format = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1416);
        let suffix = vafmt(&[0x3a1940, format as i64, (player + 1) as i64]);
        strcat(buffer, suffix);
    }
    buffer
}

/// Make the table of 2048 sines of a whole turn (the angle in steps of 1/2048 of a turn times the game's constant) (@ 0x2b6930).
pub fn trigMake() {
    let mut at = g::g_SinTable;
    let mut i = 0;
    let turn = lwc1(0x3ad7cc);
    loop {
        let angle = i as f32;
        i += 1;
        let radians = ps2float::mul(ps2float::mul(angle, 0.00048828125_f32), turn);
        swc1(at, sinf(radians));
        at += 4;
        if i >= 2048 {
            break;
        }
    }
}

/// Divide the fixed point number `a` by `b` (16 bits of fraction): the quotient, or the largest of the sign of the answer when it would not fit (@ 0x24b9a0).
pub fn FDiv(a: i32, b: i32) -> i64 {
    let mut high = a;
    if a < 0 {
        high = 0 - high;
    }
    let mut low = b;
    if b < 0 {
        low = 0 - low;
    }
    if (high >> 14) < low {
        let wide = __divdi3((a as i64) << 16, b as i64);
        (wide << 32) >> 32
    } else {
        (if -1 < a ^ b { 0x7fffffff } else { -0x80000000_i64 }) as i64
    }
}
