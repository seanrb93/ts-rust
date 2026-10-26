//! Whether something is unlocked, and choosing a pad for a character to appear at.

use super::particle_spawners::random_as_float;
use crate::*;

/// The statistics record (36 bytes) of level `level` of the saved (flag 0) or the current (flag not 0) set; three numbers, one for each difficulty.
fn level_record(stats: i32, level: i32, current: i32) -> i32 {
    level * 36 + stats + if current == 0 { 328 } else { 340 }
}

/// Count the levels of a story's list (halfwords 8 bytes apart from `list`) that have any of the three numbers set, starting from `from` and stopping at the first that has
/// none or at `to`; whether all were done.
fn story_done(list: i32, from: i32, to: i32, current: i32) -> i32 {
    let mut done = from;
    let mut at = list;
    let mut rest;
    loop {
        let record = if current == 0 {
            let stats = statsGet();
            lh(at) * 36 + stats + 328
        } else {
            let stats = statsGet();
            lh(at) * 36 + stats + 340
        };
        if lw(record) != 0 {
            done += 1;
        } else if lw(record + 4) != 0 {
            done += 1;
        } else {
            rest = done ^ to;
            if lw(record + 8) == 0 {
                return ((rest as u32) < 1) as i32;
            }
            done += 1;
        }
        at += 8;
        if done >= to {
            break;
        }
    }
    rest = done ^ to;
    ((rest as u32) < 1) as i32
}

/// Whether the item `item` (a record of four words: the kind, and what it needs) is unlocked, for the saved (`current` 0) or the current statistics: kind 0 never, 1 always,
/// 2 a level of the story (the halfword pair at +4: the level in the upper half and the difficulty in the lower, 2 meaning any of the ones from the difficulty up) with a time
/// or score set and compared with the number at +12 (by the mode at +8: 0 at least, 1 at most), 3, 4 and 5 all the levels of the first, second or third part of the story
/// done, 6 an entry of the table at +1404 compared the same way; everything is unlocked when the test flag `val1` is on (@ 0x2251a8).
pub fn unlockedEx(item: i32, current: i32) -> i32 {
    let kind = lw(item);
    if kind != 0 && lw(g::val1) != 0 {
        return 1;
    }
    if (kind as u32) >= 7 {
        return 0;
    }
    match kind {
        0 => 0,
        1 => 1,
        2 => {
            let packed = lw(item + 4);
            let level = packed >> 16;
            let mut difficulty = packed & 0xffff;
            let record = if current == 0 {
                let stats = statsGet();
                level * 36 + stats + 328
            } else {
                let stats = statsGet();
                level * 36 + stats + 340
            };
            let mode = lw(item + 8);
            let offset = difficulty << 2;
            if mode == 2 {
                if difficulty >= 3 {
                    return 0;
                }
                let mut at = offset + record;
                loop {
                    difficulty += 1;
                    if lw(at) != 0 {
                        return 1;
                    }
                    at += 4;
                    if difficulty >= 3 {
                        break;
                    }
                }
                return 0;
            }
            let value = lw(offset + record);
            if value == 0 {
                return 0;
            }
            if mode == 0 {
                if lw(item + 12) >= value {
                    return 1;
                }
            }
            if mode != 1 {
                return 0;
            }
            if value < lw(item + 12) {
                return 0;
            }
            1
        }
        3 => story_done(g::storydata + 2, 0, 3, current),
        4 => story_done(g::storydata + 26, 3, 6, current),
        5 => story_done(g::storydata + 50, 6, 9, current),
        6 => {
            let index = lw(item + 4);
            let stats = statsGet();
            let value = lw(stats + (index << 3) + 1404);
            if value == 0 {
                return 0;
            }
            let mode = lw(item + 8);
            if mode == 2 {
                return 1;
            }
            if mode == 0 && lw(item + 12) >= value {
                return 1;
            }
            if mode != 1 {
                return 0;
            }
            if value >= lw(item + 12) {
                return 1;
            }
            0
        }
        _ => unreachable!(),
    }
}

/// Test a pad as the place for a character to appear: the pad (the character's own pad number moved by `step` in the order 0, 1, -2, 3, -4 ... and wrapped round the pads)
/// must be no farther than the square root of `max2` and no nearer than that of `min2` from the player at `player` (positions at +140), and by the flags `flags` the direction
/// from the player to the pad must have a dot product with the player's facing (at +188) that is at least `facing` (flags 0x1c000000) or at most `facing` (0x2000000).
/// Returns the pad and whether it was accepted.
fn try_pad(flags: i32, own_pad: i32, step: i32, player: i32, facing: f32, min2: f32, max2: f32) -> (i32, bool) {
    let pads = lw(g::numpads);
    let moved = own_pad + if step & 1 != 0 { step } else { 0 - step };
    let wrapped = if moved >= pads { moved - pads } else { moved };
    let pad = lw(g::levelpads) + (if -1 >= wrapped { wrapped + pads } else { wrapped }) * 28;
    let mut ok = true;
    let x = ps2float::sub(lwc1(pad + 12), lwc1(player + 140));
    let y = ps2float::sub(lwc1(pad + 16), lwc1(player + 144));
    let z = ps2float::sub(lwc1(pad + 20), lwc1(player + 148));
    let d2 = ps2float::add(ps2float::add(ps2float::mul(x, x), ps2float::mul(y, y)), ps2float::mul(z, z));
    if max2 < d2 {
        ok = false;
    }
    if ok {
        if d2 < min2 {
            ok = false;
        }
        if ok {
            if flags & 0x1c000000 != 0 {
                let inv = (1.0_f32).pdiv(d2.abs().psqrt());
                let (nx, nz, ny) = (ps2float::mul(x, inv), ps2float::mul(z, inv), ps2float::mul(y, inv));
                if ps2float::add(ps2float::add(ps2float::mul(nx, lwc1(player + 188)), ps2float::mul(ny, lwc1(player + 192))), ps2float::mul(nz, lwc1(player + 196))) < facing {
                    ok = false;
                }
            }
            if ok && flags & 0x2000000 != 0 {
                let inv = (1.0_f32).pdiv(ps2float::add(ps2float::add(ps2float::mul(x, x), ps2float::mul(y, y)), ps2float::mul(z, z)).abs().psqrt());
                let (nz, nx, ny) = (ps2float::mul(z, inv), ps2float::mul(x, inv), ps2float::mul(y, inv));
                if facing < ps2float::add(ps2float::add(ps2float::mul(nx, lwc1(player + 188)), ps2float::mul(ny, lwc1(player + 192))), ps2float::mul(nz, lwc1(player + 196))) {
                    ok = false;
                }
            }
        }
    }
    (pad, ok)
}

/// Choose a pad for a character to be put at, by the flags: 0x40000000 a least distance (the byte at 0 of the flags), 0x20000000 a greatest one (the byte at 8), 0x10000000
/// facing the player's way (the first constant), 0x8000000 (the second), 0x4000000 and 0x2000000 facing away; with other than one player and a random number up to a half
/// the pads near the second player's character are tried first, then those near the first; the pads tried are the first half of the order of steps; returns the first word
/// of the pad chosen or 0 (@ 0x228590).
pub fn lvGetPseudoRandomPadExtref(flags: i32) -> i32 {
    let mut min2 = 0.0_f32;
    let half_pads = ps2float::mul(lw(g::numpads) as f32, 0.5_f32) as i32;
    let mut pad = 0;
    let mut accepted = false;
    let mut max2 = min2;
    let mut facing = min2;
    if flags & 0x40000000 != 0 {
        let least = (flags & 255) as f32;
        min2 = ps2float::mul(least, least);
    }
    if flags & 0x20000000 != 0 {
        let greatest = ((flags & 0xff00) >> 8) as f32;
        max2 = ps2float::mul(greatest, greatest);
    }
    if flags & 0x10000000 != 0 {
        facing = lwc1(0x3ac8d0);
    }
    if flags & 0x8000000 != 0 {
        facing = lwc1(0x3ac8d4);
    }
    if flags & 0x4000000 != 0 {
        facing = 0.0_f32;
    }
    let players = lw(g::lvnumplayers);
    if flags & 0x2000000 != 0 {
        facing = 0.0_f32;
    }
    let mut result_known = false;
    let mut result = false;
    if players != 1 {
        let unit = ps2float::mul(random_as_float(), 2.3283064e-10_f32);
        if !(0.5_f32 < unit) {
            result_known = true;
            if half_pads > 0 {
                let players_at = lw(g::players);
                let mut chr = lw(g::chrinfo);
                let mut step = 0;
                loop {
                    let (p, ok) = try_pad(flags, lw(chr + 0x14b8), step, players_at + 1820, facing, min2, max2);
                    pad = p;
                    accepted = ok;
                    step = (if ok { half_pads } else { step }) + 1;
                    chr = lw(g::chrinfo);
                    if step >= half_pads {
                        break;
                    }
                }
            }
            result = accepted;
        }
    }
    if !result_known {
        if half_pads > 0 {
            let players_at = lw(g::players);
            let mut chr = lw(g::chrinfo);
            let mut step = 0;
            loop {
                let (p, ok) = try_pad(flags, lw(chr + 680), step, players_at, facing, min2, max2);
                pad = p;
                accepted = ok;
                step = (if ok { half_pads } else { step }) + 1;
                chr = lw(g::chrinfo);
                if step >= half_pads {
                    break;
                }
            }
            result = accepted;
        } else {
            result = accepted;
        }
    }
    if result {
        return lw(pad);
    }
    0
}
