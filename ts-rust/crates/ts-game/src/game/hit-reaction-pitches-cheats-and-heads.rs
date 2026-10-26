//! The flinch of a character's body after a hit, the table of pitches, the cheat codes typed on the controller, and putting a head model into the body's place.

use crate::*;

/// Let the flinch after a hit die away (the time at +200 of the animation state goes down by the frame time in seconds; at 0 the two extra animations at +336 and +340 stop) and
/// turn the two matrices of the flinch (at +208 and +272) by angles worked out from the time left, from the game's constants (@ 0x215b50).
pub fn animHitReactUpdate(state: i32) {
    let left = lwc1(state + 200) - (lwc1(g::updateframesf)).pdiv(60.0_f32);
    swc1(state + 200, left);
    if left < 0.0_f32 {
        swc1(state + 200, 0.0_f32);
        sw(state + 340, -1);
        sw(state + 336, -1);
    }
    let fraction = (lwc1(state + 200)).pdiv(lwc1(0x3ac848));
    let scale = lwc1(0x3ac84c);
    sw(state + 336, lw(lw(lw(state + 32) + 92) + 16));
    matrixRotZ(state + 208, fraction * scale);
    let time = lwc1(state + 200);
    let switch_at = lwc1(0x3ac850);
    if !(switch_at < time) {
        let model = lw(state + 32);
        let rising = (time).pdiv(switch_at);
        let angle = rising * lwc1(0x3ac860);
        sw(state + 340, lw(lw(model + 92) + 20));
        matrixRotZ(state + 272, angle * rising * rising);
        return;
    }
    let half = 0.5_f32;
    let part = ((time - switch_at)).pdiv(lwc1(0x3ac854));
    sw(state + 340, lw(lw(lw(state + 32) + 92) + 20));
    if !(half < part) {
        let down = half - part;
        matrixRotZ(state + 272, (down + down) * lwc1(0x3ac85c));
        return;
    }
    matrixRotZ(state + 272, (part + part - 1.0_f32) * lwc1(0x3ac858));
}

/// Make the table of 576 pitches (halfwords at 0x1fb20a8): for the six octaves -3 to 2, the 12 notes and eighths of a note, two to the power of the note over 12 times the game's
/// base (a double) (@ 0x205e88).
pub fn calcPitches() {
    let mut octave = -3;
    let mut at = 0x1fb20a8;
    let mut base = octave * 12;
    loop {
        let mut note = 0;
        let next_octave = octave + 1;
        let mut value = base + note;
        loop {
            let next_note = note + 1;
            let mut step = value as f32;
            let mut left = 7;
            loop {
                let exponent = (step).pdiv(12.0_f32);
                left -= 1;
                step += 0.125_f32;
                let wide = fptodp(exponent);
                let power = pow(0x4000000000000000_i64, wide);
                let scaled = dpmul(power, ld(0x3998a8));
                sh(at, dptoul(scaled));
                at += 2;
                if left < 0 {
                    break;
                }
            }
            note = next_note;
            value = base + note;
            if note >= 12 {
                break;
            }
        }
        octave = next_octave;
        base = octave * 12;
        if octave >= 3 {
            break;
        }
    }
}

/// Look for a cheat code in the ring of the last 20 button presses (`cycbuf`, the next place at `cycbufpos`): each of the 6 cheats (records of 48 bytes at `cheats`: the code as
/// halfwords, its length at +40, its function at +44) whose code matches the last presses is run, and the ring is cleared (@ 0x2036b0).
pub fn cheatTick() {
    let mut cheat = 0;
    let mut position = lw(g::cycbufpos);
    let mut last_index = 0;
    let mut checked = 0;
    loop {
        let mut matches = 1;
        let record = cheat * 48 + g::cheats;
        let length = lw(record + 40);
        let mut index = (position - (length - 20)).wrapping_rem(20);
        let mut k = 0;
        let next = cheat + 1;
        if length > 0 {
            let mut following = index + 1;
            loop {
                let wrapped = following.wrapping_rem(20);
                let pressed = lhu((index << 1) + g::cycbuf);
                let wanted = lhu(record + (k << 1));
                k += 1;
                matches = if (pressed ^ wanted) != 0 { 0 } else { matches };
                last_index = wrapped;
                index = wrapped;
                checked = k;
                if k >= length {
                    break;
                }
                following = index + 1;
                if matches == 0 {
                    break;
                }
            }
        }
        let mut current = next;
        if matches != 0 {
            call_indirect(lw(record + 44), [current, last_index, checked, length, record, matches, position, 0], [0.0; 8]);
            position = lw(g::cycbufpos);
            let mut slot = g::cycbuf + 38;
            let mut left = 19;
            loop {
                sh(slot, 0);
                left -= 1;
                slot -= 2;
                if left < 0 {
                    break;
                }
            }
            current = next;
        }
        cheat = current;
        if (cheat as u32) >= 6 {
            break;
        }
    }
}

/// Put the points of a head model (chunks at +8 of `chunks`, their vertices at +4 of the part at +20 and normals at +20 after) from the world into the space of the body with matrix
/// `matrix`: each vertex has the matrix's place taken off and is turned by the matrix's transpose, each normal turned by it too (@ 0x2634c8).
pub fn transformHead(_unused: i32, chunks: i32, matrix: i32) {
    let frame = Frame::new(240);
    let sp = frame.sp();
    let list = lw(chunks + 8);
    if list == 0 {
        return;
    }
    let mut chunk = list;
    let part = chunks + 20;
    if lw(chunk + 20) < 0 {
        return;
    }
    let mut first = lw(chunk + 12);
    loop {
        let normals = lw(part + 20);
        let offset = first * 12;
        let mut vertex = lw(part + 4) + offset;
        let mut normal = if normals != 0 { normals + offset } else { 0 };
        let mut k = 0;
        if lw(chunk + 16) == 0 {
            chunk += 24;
        } else {
            let next_chunk = chunk + 24;
            loop {
                sceVu0TransposeMatrix(sp, matrix);
                let y = lwc1(matrix + 52);
                let z = lwc1(matrix + 56);
                swc1(sp + 64, lwc1(vertex) - lwc1(matrix + 48));
                swc1(sp + 68, lwc1(vertex + 4) - y);
                swc1(sp + 72, lwc1(vertex + 8) - z);
                matrixVecRotAligned(&[sp as i64, (sp + 64) as i64]);
                swc1(vertex, lwc1(sp + 64));
                swc1(vertex + 4, lwc1(sp + 68));
                swc1(vertex + 8, lwc1(sp + 72));
                vertex += 12;
                if normal != 0 {
                    matrixVecRot(sp, normal);
                    normal += 12;
                }
                k += 1;
                if (k as u32) >= (lw(chunk + 16) as u32) {
                    break;
                }
            }
            chunk = next_chunk;
        }
        if lw(chunk + 20) < 0 {
            return;
        }
        first = lw(chunk + 12);
    }
}
