//! Team names, sign-on names, a pad made at the player, a variable set to a value, a zoom step, player messages, and freeing memory.

use crate::*;

/// The name of team `team`: in the game mode 4 the words of the language for teams 1 and 2, else the team's colour name from the story data in the team format (@ 0x21c948).
pub fn gameGetTeamName(team: i32) -> i32 {
    if lw(g::curgameparms + 72) == 4 {
        if team == 1 {
            return lw(lw((lw(g::cur_language) << 2) + g::languages) + 1692);
        }
        if team == 2 {
            return lw(lw((lw(g::cur_language) << 2) + g::languages) + 1696);
        }
    }
    let language = lw((lw(g::cur_language) << 2) + g::languages);
    let format = lw(language + 2824);
    let colour = lw((lh((team << 3) + (g::storydata + 64)) << 2) + language);
    vafmt(&[0x3ae568, format as i64, colour as i64])
}

/// Look for the sign-on called `name` (the result is not used) (@ 0x2203c8).
pub fn signonGetName(name: i32) {
    let count = signonCount();
    let mut i = 0;
    let mut at = 0;
    if count > 0 {
        loop {
            if strcmp(lw(g::signonarray) + at, name) == 0 {
                break;
            }
            i += 1;
            at += 2936;
            if i >= count {
                break;
            }
        }
    }
}

/// The number (0 to 15) of the sign-on in the list at `list` (records of 2936 bytes) that is called `name`, or -1 (@ 0x220360).
pub fn signonFileGetName(list: i32, name: i32) -> i32 {
    let mut i = 0;
    let mut at = list;
    loop {
        if strcmp(at, name) == 0 {
            return i;
        }
        i += 1;
        at += 2936;
        if i >= 16 {
            return -1;
        }
    }
}

/// Make the pad `pad` at the current player's place, in the current hall, turned the way the player faces (by `atan2f` of two of the player's direction floats, in degrees)
/// (@ 0x2645f8).
pub fn generatepad(pad: i32) {
    let player = lw(g::currentplayer);
    let hall = lhu(g::curhall);
    swc1(pad + 12, lwc1(player + 832));
    sw(pad, -1);
    let y = lwc1(player + 836);
    sw(pad + 4, 1);
    swc1(pad + 16, y);
    sh(pad + 8, hall);
    swc1(pad + 20, lwc1(player + 840));
    let up = lwc1(player + 848);
    if up == 1.0_f32 {
        sw(pad + 24, 0);
    } else if !(up == -1.0_f32) {
        let z = lwc1(player + 852);
        let x = lwc1(player + 844);
        swc1(pad + 24, ps2float::mul(atan2f(x, z), 180.0_f32).pdiv(lwc1(0x3acba0)));
    } else {
        sw(pad + 24, 0);
    }
}

/// Set the game variable `var` (the value at +0, the least at +4, the most at +8, flag 1 of +12 to wrap round instead of stopping at the end, a function at +20 called when it
/// changed) to `value` (@ 0x22c308).
pub fn gvSetValue(var: i32, value: i32) {
    let old = lw(var);
    let most = lw(var + 8);
    let mut least = 0;
    let mut value = value;
    if most >= value {
        least = lw(var + 4);
        if !(value >= least) {
            value = if (lw(var + 12) & 1) == 0 { most } else { least };
        }
    } else {
        value = most;
        if (lw(var + 12) & 1) == 0 {
            value = lw(var + 4);
        }
    }
    let callback = lw(var + 20);
    sw(var, value);
    if callback != 0 {
        call_indirect(callback, [var, old, least, old, 0, 0, 0, 0], [0.0; 8]);
    }
}

/// Move the zoom of the map maker tile window a step on (the number of steps left at 0x3b0074): the four numbers at 0x3b0064 to 0x3b006c approach the goal at 0x1fc40f8 and
/// the fourth is the third scaled by 224 over 640 (@ 0x250558).
pub fn zoomtileWindowTick() {
    let steps = lw(0x3b0074);
    if steps > 0 {
        let a = lw(0x3b0064);
        let goal_a = lw(0x1fc40f8);
        let b = lw(0x3b0068);
        let step_b = (lw(0x1fc40f8 + 4) - b).wrapping_div(steps);
        sw(0x3b0064, a + (goal_a - a).wrapping_div(steps));
        let goal_c = lw(0x1fc40f8 + 8);
        let c = lw(0x3b006c);
        sw(0x3b0068, b + step_b);
        let c_now = c + (goal_c - c).wrapping_div(steps);
        sw(0x3b006c, c_now);
        sw(0x3b0074, steps - 1);
        sw(0x3b0070, (c_now * 224).wrapping_div(640));
    }
}

/// Show the message `text` to the player whose number is at `player_ptr` (the hud records of 352 bytes: the text up to 256 bytes, its timer at +272 of 180), or clear it (@ 0x280690).
pub fn playerMessage(player_ptr: i32, text: i32) {
    if text != 0 {
        let mut length = 0;
        while lbu(text + length) != 0 {
            length += 1;
        }
        let player = lw(player_ptr);
        let size = if 255 >= length { length + 1 } else { 256 };
        memcpy(player * 352 + g::hudparms, text, size);
        sb(lw(player_ptr) * 352 + g::hudparms + 255, 0);
        sw(lw(player_ptr) * 352 + g::hudparms + 272, 180);
    } else {
        sw(lw(player_ptr) * 352 + (g::hudparms + 272), 0);
    }
}

/// Write the `count` lowest bits of `flags` as characters (from the highest) to the buffer `out` (a format each for 0 and for 1) (@ 0x264c38).
pub fn padFlagsToString(flags: i32, count: i32, out: i32) {
    let mut at = out;
    let mut bit = count - 1;
    if bit >= 0 {
        loop {
            let set = (flags >> (bit & 31)) & 1;
            sprintf(&[at as i64, if set == 0 { 0x3aeb20 } else { 0x3aeb18 }, out as i64]);
            at += 1;
            bit -= 1;
            if bit < 0 {
                break;
            }
        }
    }
}

/// Give back the block of local memory at `block` (16 bytes of header: two magic words and where the block starts and how long it is) when it is the last one taken; any
/// other block stops the program (@ 0x201dd0).
pub fn memFree(block: i32) {
    let header = block - 16;
    let mut start = 0;
    if lw(header) == 0x41434f4c && lw(header + 4) == 0x4d454d4c {
        start = lw(header + 12);
    } else {
        exit(1);
    }
    let size;
    if start == lw(g::mem_local_at) {
        size = lw(header + 8);
    } else {
        size = exit(1);
    }
    let level_free = lw(g::mem_level_free) + 16 + size;
    let end = block + size;
    sw(g::mem_local_at, start + (size + 16));
    sw(g::mem_level_free, level_free);
    memMark(header, end);
}
