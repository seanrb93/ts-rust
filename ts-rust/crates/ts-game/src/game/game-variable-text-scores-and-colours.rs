//! The text of a game variable, the score shown for a challenge, and a colour from a hue.

use crate::*;

/// The text of the value of the game variable `var` (value at +0, flags at +12 with 2 for a fixed point number, a table of names at +16 of pairs of the value and the number of
/// the name, ended by a name of 0, the digits to show at +24): the name from the table (a word of the language if under 1246, else the number is itself the text), or the value
/// written by the formats at 0x3ae7d8 to 0x3ae7f0 (@ 0x22c1d0).
pub fn gvGetStringValue(var: i32) -> i32 {
    let table = lw(var + 16);
    let flags;
    if table == 0 {
        flags = lw(var + 12);
    } else {
        let mut entry = table;
        let mut name = lw(entry + 4);
        if name == 0 {
            flags = lw(var + 12);
        } else {
            let value = lw(var);
            let mut current = lw(entry);
            let mut found = true;
            while current != value {
                entry += 8;
                name = lw(entry + 4);
                if name == 0 {
                    found = false;
                    break;
                }
                current = lw(entry);
            }
            if found {
                let language = lw(g::cur_language);
                if name < 1246 {
                    return lw((name << 2) + lw((language << 2) + g::languages));
                }
                return name;
            }
            flags = lw(var + 12);
        }
    }
    if (flags & 2) == 0 {
        let digits = lw(var + 24);
        if digits == 0 {
            vafmt(&[0x3ae7f0, lw(var) as i64, digits as i64])
        } else {
            vafmt(&[0x3ae7e8, lw(var) as i64, digits as i64])
        }
    } else {
        let shown = lw(var) as f32 * 1.5258789e-05_f32;
        if lw(var + 24) == 0 {
            vafmt(&[0x3ae7e0, fptodp(shown)])
        } else {
            let wide = fptodp(shown);
            let digits = lw(var + 24);
            vafmt(&[0x3ae7d8, wide, digits as i64])
        }
    }
}

/// Set the label of the score a challenge shows (word `out_label`: a word of the language) and its text (word `out_text`) by the game mode: the time left or the best of the target
/// (modes 0, 1 and 8), the glass left (9 to 11), the barrels not blown up (15), the team's score (17), nothing for the others (@ 0x21f380).
pub fn challengeGetDisplayScore(out_label: i32, out_text: i32) {
    sw(out_label, 0);
    let mode = lw(g::curgameparms + 68);
    if (mode as u32) >= 18 {
        return;
    }
    let language = |offset: i32| lw(lw((lw(g::cur_language) << 2) + g::languages) + offset);
    let value;
    match mode {
        0 | 1 | 8 => {
            let setup = challengeGetSetupNum(lw(g::curgameparms + 68));
            let mut i = 0;
            sw(out_label, language(2644));
            let count = lw(setup + 20);
            let entry;
            loop {
                if i >= count {
                    return;
                }
                let candidate = lw(setup + 16) + i * 20;
                i += 1;
                if lw(candidate + 4) == 7 {
                    entry = candidate;
                    break;
                }
            }
            let done = StatsGet(23, 0) as i32;
            let target = (lw(entry + 16) as f32 * 1.5258789e-05_f32) as i32;
            let text = vafmt(&[0x3ae5b8, done as i64, target as i64]);
            sw(out_text, text);
            return;
        }
        2 | 3 | 4 | 5 | 6 | 7 | 12 | 13 | 14 | 16 => return,
        9 | 10 | 11 => {
            sw(out_label, language(2648));
            value = bgAmountGlassLeft();
        }
        15 => {
            sw(out_label, language(2652));
            value = propNumBallelsNotBlownUp();
        }
        17 => {
            let team = lw(lw(g::chrinfo) + 0x1228);
            sw(out_label, language(2656));
            value = teamGetScore(team);
        }
        _ => unreachable!(),
    }
    let text = vafmt(&[0x3ae5b0, value as i64]);
    sw(out_text, text);
}

/// Make a colour (the bytes from the top: 0 to 127 each; the low byte 128 as alpha) from a hue in degrees (`hue`, wrapped to 0 to 360), a saturation and a volume (@ 0x22e610).
pub fn hsv2col(hue: f32, saturation: f32, volume: f32) -> i32 {
    let full = 360.0_f32;
    let mut hue = hue;
    if full <= hue {
        loop {
            hue -= full;
            if !(full <= hue) {
                break;
            }
        }
    }
    let sector_f = (hue).pdiv(60.0_f32);
    fptodp(sector_f);
    let wide = fptodp(sector_f);
    let floored = floor(wide);
    let whole = dptofp(floored);
    let one = 1.0_f32;
    let sector = whole as i32;
    let fraction = sector_f - whole;
    let top = 127.0_f32;
    let low = (volume * (one - saturation) * top) as i32 & 255;
    let high = (volume * top) as i32 & 255;
    let falling = (volume * (one - saturation * fraction) * top) as i32 & 255;
    let rising = (volume * (one - saturation * (one - fraction)) * top) as i32 & 255;
    if (sector as u32) >= 6 {
        return 0;
    }
    let (top_byte, second_byte, third_byte) = match sector {
        0 => (high, low, rising),
        1 => (falling, low, high),
        2 => (low, rising, high),
        3 => (low, high, falling),
        4 => (rising, high, low),
        5 => (high, falling, low),
        _ => unreachable!(),
    };
    top_byte << 24 | 128 | (second_byte << 8 | third_byte << 16)
}
