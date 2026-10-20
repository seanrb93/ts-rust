//! The doors, holograms and lifts of the levels.

use crate::*;

/// The tick of a door: its state (+20 the lock, +24 whether it is open, +28 how long it has been fully open, +32 how far it is open from 0 to 1, +36 how far it moves, +4 its kind:
/// 0 slides left, 1 right, 2 two halves, 3 up) follows the use flag; it moves at the game's speeds; a door that is not locked shuts again after 180 ticks fully open; its matrix is
/// moved by the amount (@ 0x26be78).
pub fn doorPropTick(prop: i32) {
    let state = lw(prop + 352);
    let mut open = lw(state + 24);
    if (lw(prop + 28) & 1) != 0 && lw(state + 20) == 0 {
        sw(state + 24, 1 - lw(state + 24));
        open = lw(state + 24);
    }
    let mut position = lwc1(state + 32);
    if open == 0 {
        let frame = lwc1(g::updateframesf);
        if 0.0_f32 < position {
            position -= frame * lwc1(0x3acc30);
            swc1(state + 32, position);
        }
        if position < 0.0_f32 {
            swc1(state + 32, 0.0_f32);
            position = 0.0_f32;
        }
    } else {
        let one = 1.0_f32;
        let frame = lwc1(g::updateframesf);
        if position < one {
            position += frame * lwc1(0x3acc2c);
            swc1(state + 32, position);
        }
        if one < position {
            position = one;
            swc1(state + 32, one);
        }
    }
    if !(position == 1.0_f32) {
        sw(state + 28, 0);
    } else {
        let mut time = lw(state + 28);
        if lw(state + 20) == 0 {
            sw(state + 28, time + lw(g::updateframesi));
            time = lw(state + 28);
        }
        if !(time < 181) {
            sw(state + 24, 0);
        }
    }
    let kind = lw(state + 4);
    let travel = lwc1(state + 36);
    match kind {
        1 => matrixTrans(lw(lw(prop + 32) + 4), travel * position, 0.0_f32, 0.0_f32),
        0 => matrixTrans(lw(lw(prop + 32) + 4), -travel * position, 0.0_f32, 0.0_f32),
        3 => matrixTrans(lw(lw(prop + 32) + 4), 0.0_f32, travel * position, 0.0_f32),
        2 => {
            let model = lw(prop + 32);
            let skeleton = lw(model);
            let matrices = lw(model + 4);
            let bones = skeleton - lw(skeleton) * 80;
            let first = lb(bones + 3);
            let second = lb(first * 80 + bones + 4);
            sceVu0UnitMatrix(matrices);
            matrixTrans(lw(lw(prop + 32) + 4) + (first << 6), -lwc1(state + 36) * lwc1(state + 32), 0.0_f32, 0.0_f32);
            matrixTrans(lw(lw(prop + 32) + 4) + (second << 6), lwc1(state + 36) * lwc1(state + 32), 0.0_f32, 0.0_f32);
        }
        _ => {}
    }
}

/// Put the alpha (0 to 127, `alpha`) into the top byte of every vertex colour of the first chunk list of each bone of the prop's model (the colours made by `getOverrideChunk`).
fn set_alpha_of_colours(prop: i32, sp: i32, alpha: i32) {
    let skeleton = lw(lw(prop + 32));
    let bones = lw(skeleton);
    let mut i = 0;
    if bones > 0 {
        loop {
            allocateOverrideChunk(prop, 0, 1, 4);
            let bone = i * 80 + (skeleton - bones * 80);
            let out = getOverrideChunk(prop, 0, 1, 4, sp);
            let mut source = lw(bone + 56);
            let mut k = 0;
            let next = i + 1;
            if lw(sp) > 0 {
                let mut at = out;
                loop {
                    let colour = lw(source);
                    k += 1;
                    source += 4;
                    sw(at, colour & 0xffffff | (colour & 255) + (alpha << 24));
                    at += 4;
                    if k >= lw(sp) {
                        break;
                    }
                }
            }
            i = next;
            if i >= lw(lw(lw(prop + 32))) {
                break;
            }
        }
    }
}

/// The hologram: its alpha (`hologramAlpha`) rises when a character is within 8 of the prop (in the state of neither 0 nor 0x2000) and falls otherwise (at the game's speeds, held
/// to 0 and 1), is put in every vertex colour, and a sine of the game's time kept at +72 and +76 of the prop (@ 0x27b1d8).
pub fn hologramPropTick(prop: i32) {
    let frame = Frame::new(160);
    let sp = frame.sp();
    let players = lw(g::lvnumplayers);
    let active = lw(g::numactivechars);
    let mut index = 0;
    let mut near = false;
    if (players + active) > 0 {
        let mut offset = 0;
        loop {
            let record = offset + lw(g::chrinfo);
            let state = lw(record + 2716);
            if state != 0x2000 && state != 0 {
                let other = lw(record + 3020);
                let dx = lwc1(other + 48) - lwc1(prop + 48);
                let dz = lwc1(other + 56) - lwc1(prop + 56);
                if (dx * dx + dz * dz).abs().sqrt() < 8.0_f32 {
                    near = true;
                }
            }
            index += 1;
            offset += 0x1210;
            if index >= players + active {
                break;
            }
        }
    }
    let alpha = lwc1(g::hologramAlpha);
    let limit;
    let mut now;
    let beyond;
    if !near {
        limit = 0.0_f32;
        now = alpha - lwc1(0x3acf84);
        beyond = now < limit;
        swc1(g::hologramAlpha, now);
    } else {
        limit = 1.0_f32;
        now = alpha + lwc1(0x3acf80);
        beyond = limit < now;
        swc1(g::hologramAlpha, now);
    }
    if beyond {
        now = limit;
        swc1(g::hologramAlpha, limit);
    }
    set_alpha_of_colours(prop, sp, fptoui(now * 127.0_f32));
    let wave = lw(g::gameframe) as f32 * lwc1(0x3acf88);
    swc1(prop + 76, wave);
    swc1(prop + 72, wave);
}

/// The tick of a lift: it waits (+72 is 0) until a player's character is within 1 of it, then runs a timer (+44, seconds) through the stages: before 0 it is at its start place
/// (+36 to +44 of the prop), from 0 to 4 it moves to the end place (+60 to +68 of its state) by a cosine ease, from 4 to 7 it stays there, from 7 to 11 it moves back,
/// and past 11 the timer starts again at -3 (@ 0x27b698).
pub fn liftPropTick(prop: i32) {
    let frame = Frame::new(144);
    let sp = frame.sp();
    let state = lw(prop + 352);
    let seconds = lwc1(g::updateframesf) / 60.0_f32;
    if lw(state + 72) == 0 {
        let players = lw(g::lvnumplayers);
        let mut i = 0;
        if players > 0 {
            let mut offset = 0;
            let mut count = players;
            loop {
                let record = offset + lw(g::chrinfo);
                let char_state = lw(record + 2716);
                if char_state != 0x2000 && char_state != 0 {
                    let other = lw(record + 3020);
                    let dx = lwc1(other + 48) - lwc1(prop + 48);
                    let dz = lwc1(other + 56) - lwc1(prop + 56);
                    if (dx * dx + dz * dz).abs().sqrt() < 1.0_f32 {
                        sw(state + 72, 1);
                    }
                    count = lw(g::lvnumplayers);
                }
                i += 1;
                offset += 0x1210;
                if i >= count {
                    break;
                }
            }
        }
        return;
    }
    let end_x = lwc1(state + 60);
    swc1(state + 44, lwc1(state + 44) + seconds);
    let start_y = lwc1(prop + 40);
    let start_z = lwc1(prop + 44);
    swc1(sp, end_x - lwc1(prop + 36));
    swc1(sp + 4, lwc1(state + 64) - start_y);
    swc1(sp + 8, lwc1(state + 68) - start_z);
    let mut timer = lwc1(state + 44);
    if 11.0_f32 < timer {
        swc1(state + 44, -3.0_f32);
        timer = -3.0_f32;
    }
    if !(timer < 0.0_f32) {
        if 4.0_f32 <= timer && timer < 7.0_f32 {
            swc1(prop + 48, lwc1(state + 60));
            swc1(prop + 52, lwc1(state + 64));
            swc1(prop + 56, lwc1(state + 68));
            return;
        }
        if 0.0_f32 <= timer && timer < 4.0_f32 {
            let angle = dptofp(dpmul(fptodp(timer * 0.25_f32), ld(0x3a8610)));
            let ease = 1.0_f32 - (cosf(angle) + 1.0_f32) * 0.5_f32;
            let x = lwc1(prop + 36) + lwc1(sp) * ease;
            let y = lwc1(prop + 40) + lwc1(sp + 4) * ease;
            swc1(prop + 56, lwc1(prop + 44) + lwc1(sp + 8) * ease);
            swc1(prop + 48, x);
            swc1(prop + 52, y);
            return;
        }
        if 7.0_f32 <= timer {
            let angle = dptofp(dpmul(fptodp((timer - 7.0_f32) * 0.25_f32), ld(0x3a8618)));
            let ease = (cosf(angle) + 1.0_f32) * 0.5_f32;
            let x = lwc1(prop + 36) + lwc1(sp) * ease;
            let y = lwc1(prop + 40) + lwc1(sp + 4) * ease;
            swc1(prop + 56, lwc1(prop + 44) + lwc1(sp + 8) * ease);
            swc1(prop + 48, x);
            swc1(prop + 52, y);
        }
    } else {
        let y = lwc1(prop + 40);
        let z = lwc1(prop + 44);
        swc1(prop + 48, lwc1(prop + 36));
        swc1(prop + 56, z);
        swc1(prop + 52, y);
    }
}
