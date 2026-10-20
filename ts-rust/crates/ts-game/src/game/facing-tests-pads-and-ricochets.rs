//! Tests of whether a point is in front of a character or a pad, making a pad at the player, taking a prop's attachment away, a ricochet sound, and level names.

use crate::*;

/// Whether the point `pos` is to the side of the character `chr` (the direction to it, not made a unit vector, against the way the character faces rotated by the game's
/// constant, more than `threshold`) (@ 0x2c6cc8).
pub fn checkSide(chr: i32, pos: i32, threshold: f32) -> i32 {
    let frame = Frame::new(160);
    let sp = frame.sp();
    let z = lwc1(pos + 8);
    let angle = lwc1(chr + 80) * lwc1(0x3ad98c) / 180.0_f32;
    let dx = lwc1(pos) - lwc1(chr + 48);
    let chr_z = lwc1(chr + 56);
    let dy = lwc1(pos + 4) - lwc1(chr + 52);
    swc1(sp, dx);
    swc1(sp + 4, dy);
    swc1(sp + 8, z - chr_z);
    let tail = lw(0x3a9e60 + 8);
    sd(sp + 16, ld(0x3a9e60));
    sw(sp + 24, tail);
    matrixRotY(sp + 32, angle);
    matrixVecRotAligned(&[(sp + 32) as i64, (sp + 16) as i64]);
    if threshold < lwc1(sp) * lwc1(sp + 16) + lwc1(sp + 4) * lwc1(sp + 20) + lwc1(sp + 8) * lwc1(sp + 24) {
        1
    } else {
        0
    }
}

/// Whether the point `pos` is in front of the character `chr`: the unit vector to it against the forward vector turned by the character's direction (+80) (@ 0x2c6808).
pub fn checkInFront(chr: i32, pos: i32, threshold: f32) -> i32 {
    let frame = Frame::new(176);
    let sp = frame.sp();
    let dx = lwc1(pos) - lwc1(chr + 48);
    let chr_z = lwc1(chr + 56);
    let pos_z = lwc1(pos + 8);
    let dy = lwc1(pos + 4) - lwc1(chr + 52);
    let tail = lw(0x3a9d38);
    sd(sp + 16, ld(0x3a9d30));
    sw(sp + 24, tail);
    let dz = pos_z - chr_z;
    swc1(sp, dx);
    swc1(sp + 4, dy);
    swc1(sp + 8, dz);
    let inv = 1.0_f32 / (dx * dx + dy * dy + dz * dz).abs().sqrt();
    let angle = lwc1(chr + 80) * lwc1(0x3ad980) / 180.0_f32;
    let nx = lwc1(sp) * inv;
    let ny = lwc1(sp + 4) * inv;
    swc1(sp + 8, lwc1(sp + 8) * inv);
    swc1(sp, nx);
    swc1(sp + 4, ny);
    matrixRotY(sp + 32, angle);
    matrixVecRotAligned(&[(sp + 32) as i64, (sp + 16) as i64]);
    if threshold < lwc1(sp) * lwc1(sp + 16) + lwc1(sp + 4) * lwc1(sp + 20) + lwc1(sp + 8) * lwc1(sp + 24) {
        1
    } else {
        0
    }
}

/// Whether the character `other` is aiming at `chr` in front: the unit vector from `other` to `chr` against the forward vector turned by `other`'s aim (+2972 of its state) (@ 0x2c6b50).
pub fn checkAimingFront(chr: i32, other: i32, threshold: f32) -> i32 {
    let frame = Frame::new(176);
    let sp = frame.sp();
    let dx = lwc1(chr + 48) - lwc1(other + 48);
    let other_z = lwc1(other + 56);
    let chr_z = lwc1(chr + 56);
    let dy = lwc1(chr + 52) - lwc1(other + 52);
    let tail = lw(0x3a9d38);
    sd(sp + 16, ld(0x3a9d30));
    sw(sp + 24, tail);
    let dz = chr_z - other_z;
    swc1(sp, dx);
    swc1(sp + 4, dy);
    swc1(sp + 8, dz);
    let inv = 1.0_f32 / (dx * dx + dy * dy + dz * dz).abs().sqrt();
    let state = lw(other + 352);
    let scale = lwc1(0x3ad988);
    let nx = lwc1(sp) * inv;
    let ny = lwc1(sp + 4) * inv;
    swc1(sp + 8, lwc1(sp + 8) * inv);
    swc1(sp, nx);
    swc1(sp + 4, ny);
    let angle = lwc1(state + 2972) * scale / 180.0_f32;
    matrixRotY(sp + 32, angle);
    matrixVecRotAligned(&[(sp + 32) as i64, (sp + 16) as i64]);
    if threshold < lwc1(sp) * lwc1(sp + 16) + lwc1(sp + 4) * lwc1(sp + 20) + lwc1(sp + 8) * lwc1(sp + 24) {
        1
    } else {
        0
    }
}

/// Whether the point `pos` is in front of pad `pad`: the unit vector from the pad to it against the forward vector turned by the pad's direction (+24) (@ 0x2c6980).
pub fn checkInFrontOfPad(pad: i32, pos: i32, threshold: f32) -> i32 {
    let frame = Frame::new(160);
    let sp = frame.sp();
    let pads = lw(g::levelpads);
    let tail = lw(0x3a9d38);
    sd(sp + 16, ld(0x3a9d30));
    sw(sp + 24, tail);
    let record = pad * 28 + pads;
    let y = lwc1(pos + 4);
    let dx = lwc1(pos) - lwc1(record + 12);
    let z = lwc1(pos + 8);
    swc1(sp, dx);
    let dy = y - lwc1(record + 16);
    swc1(sp + 4, dy);
    let dz = z - lwc1(record + 20);
    swc1(sp + 8, dz);
    let inv = 1.0_f32 / (dx * dx + dy * dy + dz * dz).abs().sqrt();
    let scale = lwc1(0x3ad984);
    let nx = lwc1(sp) * inv;
    let ny = lwc1(sp + 4) * inv;
    swc1(sp + 8, lwc1(sp + 8) * inv);
    swc1(sp, nx);
    swc1(sp + 4, ny);
    let angle = lwc1(pad * 28 + pads + 24) * scale / 180.0_f32;
    matrixRotY(sp + 32, angle);
    matrixVecRotAligned(&[(sp + 32) as i64, (sp + 16) as i64]);
    if threshold < lwc1(sp) * lwc1(sp + 16) + lwc1(sp + 4) * lwc1(sp + 20) + lwc1(sp + 8) * lwc1(sp + 24) {
        1
    } else {
        0
    }
}

/// Add a pad at the current player (the next number after the last pad's, 1001 for the first) and count it (@ 0x2646b0).
pub fn createpad() {
    let count = lw(g::numpads);
    let pads = lw(g::levelpads);
    sw(g::numpads, count + 1);
    let pad = pads + count * 28;
    generatepad(pad);
    let now = lw(g::numpads);
    let number = if now != 1 { lw(now * 28 + lw(g::levelpads) - 56) + 1 } else { 1001 };
    sw(pad, number);
    sw(0x3b00a0, lw(0x3b00a0) + 1);
    printpadinfo();
}

/// Take a prop off what it was attached to: its attachment (+192 is -1, +188 and +196 are 0), flag 0x40000000 is cleared, and all the matrices of its model are made unit
/// matrices again (@ 0x26f460).
pub fn propUnattach(prop: i32) {
    sw(prop + 192, -1);
    let model = lw(prop + 32);
    let flags = lw(prop + 16) & -0x40000001;
    sw(prop + 188, 0);
    sw(prop + 16, flags);
    sw(prop + 196, 0);
    if model != 0 && lw(model + 4) != 0 {
        let mut i = 0;
        if lw(lw(model) + 4) > 0 {
            let mut matrices = lw(model + 4);
            loop {
                let at = matrices + (i << 6);
                i += 1;
                sceVu0UnitMatrix(at);
                let model_now = lw(prop + 32);
                if i >= lw(lw(model_now) + 4) {
                    break;
                }
                matrices = lw(model_now + 4);
            }
        }
    }
}

/// Play a ricochet sound at `pos`: one of the sounds of the list for kind `kind` (`ricochetsoundarrays`, a list ended by 0) at random, louder for kind 0 (@ 0x28a830).
pub fn ricochetsfx(kind: i32, pos: i32) {
    let list = lw((kind << 2) + g::ricochetsoundarrays);
    let mut count = 0;
    if lw(list) != 0 {
        let mut at = list;
        loop {
            at += 4;
            count += 1;
            if lw(at) == 0 {
                break;
            }
        }
    }
    let random = newrnd();
    if count == 0 {
        trap("break");
        return;
    }
    let pick = ((random as i32 as u32) % (count as u32)) as i32;
    let volume = if kind == 0 { 15 } else { 5 };
    soundDelayStartEx(lw((pick << 2) + list), pos, volume, 0);
}

/// The number of the level called `name` in the current language (case does not matter), or else the number the name is written as (@ 0x2257e8).
pub fn lvGetNumFromName(name: i32) -> i32 {
    let language = lw(g::cur_language);
    let languages = g::languages;
    if lw((lh(LVINFO) << 2) + lw((language << 2) + languages)) != 0 {
        let mut entry = LVINFO;
        let mut offset = language << 2;
        loop {
            let level_name = lw((lh(entry) << 2) + lw(offset + languages));
            if strcasecmp(name, level_name) == 0 {
                return lw(entry + 4);
            }
            let language_now = lw(g::cur_language);
            entry += 24;
            offset = language_now << 2;
            if lw((lh(entry) << 2) + lw((language_now << 2) + languages)) == 0 {
                break;
            }
        }
    }
    atoi(name)
}
