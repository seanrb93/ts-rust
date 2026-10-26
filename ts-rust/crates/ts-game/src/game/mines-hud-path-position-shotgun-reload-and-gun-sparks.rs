//! The mines of the HUD, moving along a path, the shotgun reload's shells, and the sparks of a gun.

use crate::*;
use super::random_as_float;

/// Draw the pictures of the mines stuck to the prop `prop` (counts for three kinds from `bulletGetMinesAttachedToProp`) in a column at the right of the window starting at the
/// height `y`: each picture present pulses (its brightness is 27 plus a half of 100 times a sine of the game's time, plus 1) (@ 0x2b1e98).
pub fn hudMinesGfx(prop: i32, y: i32) {
    let frame = Frame::new(32);
    let sp = frame.sp();
    sw(sp + 16, 0);
    sw(sp + 20, 0);
    sw(sp + 24, 0);
    let players = ilinkGetNumLocalPlayers();
    let big = ((players < 3) as i32) ^ 1;
    bulletGetMinesAttachedToProp(prop, sp + 16, sp + 20, sp + 24);
    let window = g::window;
    let edge = lw(window + 4);
    let mut left = if big != 0 { edge - 36 } else { edge - 62 };
    let players = ilinkGetNumLocalPlayers();
    let edge = lw(window + 4);
    if players == 2 {
        left = edge - 50;
    }
    let right = if big != 0 { edge - 6 } else { edge - 14 };
    let height = if lw(window + 92) != 0 { 16 } else { 22 };
    let pulse = |scale_a: i32, scale_b: i32| -> i32 {
        let wave = ps2float::add(lwc1((((ps2float::add(ps2float::mul(ps2float::mul(gameframe_x(), lwc1(scale_a)), lwc1(scale_b)), 0.5_f32) as i32) & 2047) << 2) + g::g_SinTable), 1.0_f32);
        dptoli(dpmul(dpmul(fptodp(wave), 0x4059000000000000_i64), 0x3fe0000000000000_i64)) + 27
    };
    if lw(sp + 16) > 0 {
        let brightness = pulse(0x3ad784, 0x3ad788);
        let texture = lw(0x1feaf74);
        let entry = texture * 40 + lw(lw(g::thistexbuf) + 4);
        let picture_h = lhu(entry + 2);
        let picture_w = lhu(entry);
        dlSelectTextureKick(texture);
        let bottom = y + height;
        sw(sp, brightness << 8 | 127 | (brightness << 16 | 0x7f000000));
        dlTextureRectangle(left, y, right, bottom, 0, 0, picture_w << 4, picture_h << 4);
    }
    let second = y + 2 + height;
    if lw(sp + 20) > 0 {
        let brightness = pulse(0x3ad78c, 0x3ad790);
        let texture = lw(0x1feaf6c);
        let entry = texture * 40 + lw(lw(g::thistexbuf) + 4);
        let picture_h = lhu(entry + 2);
        let picture_w = lhu(entry);
        dlSelectTextureKick(texture);
        let bottom = second + height;
        sw(sp, brightness << 8 | 127 | (brightness << 16 | 0x7f000000));
        dlTextureRectangle(left, second, right, bottom, 0, 0, picture_w << 4, picture_h << 4);
    }
    let third = second + 2 + height;
    if lw(sp + 24) > 0 {
        let brightness = pulse(0x3ad794, 0x3ad798);
        let texture = lw(0x1feaf70);
        let entry = texture * 40 + lw(lw(g::thistexbuf) + 4);
        let picture_h = lhu(entry + 2);
        let picture_w = lhu(entry);
        dlSelectTextureKick(texture);
        sw(sp, brightness << 8 | 127 | (brightness << 16 | 0x7f000000));
        dlTextureRectangle(left, third, right, third + height, 0, 0, picture_w << 4, picture_h << 4);
    }
}

/// Move the prop `prop` the distance `distance` along its path (the node at +12 of its record; nodes are 44 bytes: kind at +0, the place at +4 to +12, for the arcs the angle at
/// +16 and the radius at +20): kind 0 a straight line to the node's place, 1 and 2 an arc about the node's centre one way or the other (its length the radius times the game's
/// constant times the part of the circle left to go), 3 the end. A node that is passed uses up the distance and the next is taken (@ 0x2756d0).
pub fn pathPosUpdateAbs(prop: i32, distance: f32) {
    let frame = Frame::new(16);
    let sp = frame.sp();
    let mut left = distance;
    let state = lw(prop + 352);
    if !(0.0_f32 < left) {
        return;
    }
    let full = lwc1(0x3acddc);
    loop {
        let node = lw(state + 12);
        let kind = lw(node);
        let mut passed: Option<f32> = None;
        if kind == 1 || kind == 2 {
            let turning = if kind == 1 { ps2float::sub(lwc1(node + 16), lwc1(state + 56)) } else { ps2float::sub(lwc1(state + 56), lwc1(node + 16)) };
            let mut turn = turning;
            if turn < 0.0_f32 {
                turn = ps2float::add(turn, 3.6e+02_f32);
            }
            let arc = ps2float::mul(ps2float::mul(lwc1(node + 20), full), ((turn).pdiv(3.6e+02_f32)));
            if arc < left {
                passed = Some(arc);
            } else {
                let to_radians = lwc1(if kind == 1 { 0x3acde0 } else { 0x3acde4 });
                let start = lwc1(node + 16);
                let sine = sinf(ps2float::mul(start, to_radians).pdiv(1.8e+02_f32));
                let cosine = cosf(ps2float::mul(lwc1(lw(state + 12) + 16), to_radians).pdiv(1.8e+02_f32));
                let centre = lw(state + 12);
                let radius = lwc1(centre + 20);
                let part = ps2float::mul(left, turn).pdiv(arc);
                let (centre_z, centre_x);
                let angle;
                if kind == 1 {
                    centre_z = ps2float::add(lwc1(centre + 12), ps2float::mul(radius, -sine));
                    centre_x = ps2float::add(lwc1(centre + 4), ps2float::mul(radius, cosine));
                    let moved = ps2float::add(lwc1(state + 56), part);
                    swc1(state + 56, moved);
                    if 3.6e+02_f32 <= moved {
                        swc1(state + 56, ps2float::sub(moved, 3.6e+02_f32));
                    }
                    left = 0.0_f32;
                    let back = ps2float::mul(ps2float::sub(lwc1(state + 56), 9e+01_f32), to_radians).pdiv(1.8e+02_f32);
                    let sine_now = sinf(back);
                    swc1(prop + 48, ps2float::add(centre_x, ps2float::mul(lwc1(centre + 20), sine_now)));
                    angle = ps2float::sub(lwc1(state + 56), 9e+01_f32);
                } else {
                    centre_z = ps2float::add(lwc1(centre + 12), ps2float::mul(radius, sine));
                    centre_x = ps2float::add(lwc1(centre + 4), ps2float::mul(radius, -cosine));
                    let moved = ps2float::sub(lwc1(state + 56), part);
                    swc1(state + 56, moved);
                    if moved < 0.0_f32 {
                        swc1(state + 56, ps2float::add(moved, 3.6e+02_f32));
                    }
                    let ahead = ps2float::mul(ps2float::add(lwc1(state + 56), 9e+01_f32), to_radians).pdiv(1.8e+02_f32);
                    left = 0.0_f32;
                    let sine_now = sinf(ahead);
                    swc1(prop + 48, ps2float::add(centre_x, ps2float::mul(lwc1(centre + 20), sine_now)));
                    angle = ps2float::add(lwc1(state + 56), 9e+01_f32);
                }
                let cosine_now = cosf(ps2float::mul(angle, to_radians).pdiv(1.8e+02_f32));
                swc1(prop + 56, ps2float::add(centre_z, ps2float::mul(lwc1(lw(state + 12) + 20), cosine_now)));
            }
        } else if kind == 0 {
            let x = lwc1(prop + 48);
            let y = lwc1(prop + 52);
            let dx = ps2float::sub(lwc1(node + 4), x);
            let z = lwc1(prop + 56);
            swc1(sp, dx);
            let dy = ps2float::sub(lwc1(node + 8), y);
            swc1(sp + 4, dy);
            let dz = ps2float::sub(lwc1(node + 12), z);
            let length = ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz)).abs().psqrt();
            swc1(sp + 8, dz);
            let first = lwc1(sp);
            if !(length < left) {
                let step_x = ps2float::mul(first, left).pdiv(length);
                let step_y = ps2float::mul(lwc1(sp + 4), left).pdiv(length);
                let step_z = ps2float::mul(lwc1(sp + 8), left).pdiv(length);
                left = 0.0_f32;
                let new_y = ps2float::add(lwc1(prop + 52), step_y);
                let new_z = ps2float::add(lwc1(prop + 56), step_z);
                swc1(prop + 48, ps2float::add(lwc1(prop + 48), step_x));
                swc1(prop + 52, new_y);
                swc1(prop + 56, new_z);
            } else {
                let current = lw(state + 12);
                left = ps2float::sub(left, length);
                let next_x = lwc1(current + 4);
                sw(state + 12, current + 44);
                swc1(prop + 48, next_x);
                swc1(prop + 52, lwc1(current + 8));
                swc1(prop + 56, lwc1(current + 12));
            }
        } else if kind == 3 {
            left = 0.0_f32;
        }
        if let Some(arc) = passed {
            let next_x = lwc1(node + 4);
            left = ps2float::sub(left, arc);
            sw(state + 12, node + 44);
            swc1(prop + 48, next_x);
            swc1(prop + 52, lwc1(node + 8));
            swc1(prop + 56, lwc1(node + 12));
        }
        if !(0.0_f32 < left) {
            break;
        }
    }
}

/// Move the shells of the shotgun reload `reload` (the model at +12, the two shells at +140 and +144, the angle at +136): the pump bone's matrix is turned by the angle about the
/// bone's centre; a reload that is not of kind 11 puts each shell where the bone has turned it (its place, angle and the model's other angles added), and one of kind 11 moves the
/// shells by their speeds (@ 0x291d38).
pub fn updateShotgunReload(reload: i32) {
    let frame = Frame::new(240);
    let sp = frame.sp();
    let to_radians = lwc1(0x3ad3a8);
    let half_turn = 1.8e+02_f32;
    let model = lw(lw(reload + 12) + 32);
    let skeleton = lw(model);
    let matrix = lw(model + 4) + 128;
    let bones = skeleton - lw(skeleton) * 80;
    let centre = lw(bones + 72);
    let centre_y = centre + 36;
    matrixTrans(matrix, lwc1(centre + 36), lwc1(centre_y + 4), lwc1(centre_y + 8));
    matrixRotZXY(sp, ps2float::mul(lwc1(reload + 136), to_radians).pdiv(half_turn), 0.0_f32, 0.0_f32);
    sceVu0MulMatrix(matrix, matrix, sp);
    matrixTrans(sp, -lwc1(centre + 36), -lwc1(centre_y + 4), -lwc1(centre_y + 8));
    sceVu0MulMatrix(matrix, matrix, sp);
    let first = lw(reload + 140);
    if first == 0 {
        return;
    }
    let second = lw(reload + 144);
    if second == 0 {
        return;
    }
    if lw(reload) != 11 {
        let owner = lw(reload + 12);
        matrixRotZXY(sp + 64, ps2float::mul(lwc1(reload + 136), to_radians).pdiv(half_turn), 0.0_f32, 0.0_f32);
        let model_now = lw(reload + 12);
        let roll = ps2float::mul(lwc1(model_now + 88), to_radians).pdiv(half_turn);
        let yaw = ps2float::mul(lwc1(model_now + 72), to_radians).pdiv(half_turn);
        let pitch = ps2float::mul(lwc1(model_now + 76), to_radians).pdiv(half_turn);
        matrixRotZXY(sp + 128, yaw, pitch, roll);
        let bone = lw(bones + 72);
        let pivot_x = lwc1(bone + 36);
        let pivot = bone + 36;
        swc1(sp + 208, pivot_x);
        let pivot_y = lwc1(pivot + 4);
        swc1(sp + 212, pivot_y);
        let pivot_z = lwc1(pivot + 8);
        swc1(sp + 216, pivot_z);
        swc1(sp + 192, ps2float::sub(lwc1(bone), pivot_x));
        swc1(sp + 196, ps2float::sub(lwc1(bone + 4), pivot_y));
        swc1(sp + 200, ps2float::sub(lwc1(bone + 8), pivot_z));
        matrixVecRotAligned(&[(sp + 64) as i64, (sp + 192) as i64]);
        matrixVecRotAligned(&[(sp + 128) as i64, (sp + 208) as i64]);
        matrixVecRotAligned(&[(sp + 128) as i64, (sp + 192) as i64]);
        let shell = lw(reload + 140);
        let model_now = lw(reload + 12);
        let bone = lw(bones + 72);
        let pivot = bone + 36;
        swc1(shell + 48, ps2float::add(ps2float::add(lwc1(sp + 192), lwc1(sp + 208)), lwc1(owner + 48)));
        swc1(shell + 52, ps2float::add(ps2float::add(lwc1(sp + 196), lwc1(sp + 212)), lwc1(owner + 52)));
        swc1(shell + 56, ps2float::add(ps2float::add(lwc1(sp + 200), lwc1(sp + 216)), lwc1(owner + 56)));
        swc1(shell + 72, ps2float::add(lwc1(model_now + 72), lwc1(reload + 136)));
        swc1(shell + 76, lwc1(model_now + 76));
        swc1(shell + 88, lwc1(model_now + 88));
        let pivot_x = lwc1(bone + 36);
        swc1(sp + 208, pivot_x);
        let pivot_y = lwc1(pivot + 4);
        swc1(sp + 212, pivot_y);
        let pivot_z = lwc1(pivot + 8);
        let other = bone + 12;
        swc1(sp + 216, pivot_z);
        swc1(sp + 192, ps2float::sub(lwc1(bone + 12), pivot_x));
        swc1(sp + 196, ps2float::sub(lwc1(other + 4), pivot_y));
        swc1(sp + 200, ps2float::sub(lwc1(other + 8), pivot_z));
        matrixVecRotAligned(&[(sp + 64) as i64, (sp + 192) as i64]);
        matrixVecRotAligned(&[(sp + 128) as i64, (sp + 208) as i64]);
        matrixVecRotAligned(&[(sp + 128) as i64, (sp + 192) as i64]);
        let shell = lw(reload + 144);
        let model_now = lw(reload + 12);
        swc1(shell + 48, ps2float::add(ps2float::add(lwc1(sp + 192), lwc1(sp + 208)), lwc1(owner + 48)));
        swc1(shell + 52, ps2float::add(ps2float::add(lwc1(sp + 196), lwc1(sp + 212)), lwc1(owner + 52)));
        swc1(shell + 56, ps2float::add(ps2float::add(lwc1(sp + 200), lwc1(sp + 216)), lwc1(owner + 56)));
        swc1(shell + 72, ps2float::add(lwc1(model_now + 72), lwc1(reload + 136)));
        swc1(shell + 76, lwc1(model_now + 76));
        swc1(shell + 88, lwc1(model_now + 88));
    } else {
        let y1 = ps2float::add(lwc1(first + 52), per_update(lwc1(first + 64)));
        let z1 = ps2float::add(lwc1(first + 56), per_update(lwc1(first + 68)));
        swc1(first + 48, ps2float::add(lwc1(first + 48), per_update(lwc1(first + 60))));
        swc1(first + 52, y1);
        swc1(first + 56, z1);
        let y2 = ps2float::add(lwc1(second + 52), per_update(lwc1(second + 64)));
        let z2 = ps2float::add(lwc1(second + 56), per_update(lwc1(second + 68)));
        swc1(second + 48, ps2float::add(lwc1(second + 48), per_update(lwc1(second + 60))));
        swc1(second + 52, y2);
        swc1(second + 56, z2);
    }
}

/// Start the sparks of a gun hit: a particle group of kind 15 owned by `owner` at the place `at`, with as many sparks as the game's number for kind 10, each spark's speed the
/// direction `direction` plus a random bit (up to 1.5 in each axis and, for x and z, 0.75 less); the rooms the group can be seen from are worked out from the box the sparks
/// can reach (the group's room array at +72) (@ 0x2971e0).
pub fn particleNewGunSparks(owner: i32, at: i32, direction: i32) -> i32 {
    let group = getgroup(15);
    let k = lwc1(0x3ad494);
    sw(group + 20, owner);
    sw(group + 12, 0);
    sw(group + 8, 4);
    swc1(group + 16, k);
    sw(group + 4, -1);
    let count = particleGetNum(10);
    sw(group + 24, count);
    swc1(group + 928, lwc1(at));
    sw(group + 44, 0);
    swc1(group + 932, lwc1(at + 4));
    swc1(group + 936, lwc1(at + 8));
    let rooms = group + 72;
    let scale = 2.3283064e-10_f32;
    if count > 0 {
        let mut i = 0;
        loop {
            swc1(i * 12 + (group + 940), ps2float::sub(ps2float::add(lwc1(direction), ps2float::mul(ps2float::mul(random_as_float(), scale), 1.5_f32)), 0.75_f32));
            swc1(i * 12 + (group + 944), ps2float::add(lwc1(direction + 4), ps2float::mul(ps2float::mul(random_as_float(), scale), 1.5_f32)));
            let unit = random_as_float();
            i += 1;
            swc1(i * 12 + (group + 936), ps2float::sub(ps2float::add(lwc1(direction + 8), ps2float::mul(ps2float::mul(unit, scale), 1.5_f32)), 0.75_f32));
            if i >= lw(group + 24) {
                break;
            }
        }
    }
    let dir_x = lwc1(direction);
    let one_and_half = 1.5_f32;
    let dir_y = lwc1(direction + 4);
    let dir_x_plus = ps2float::add(dir_x, 0.0_f32);
    let three_quarters = 0.75_f32;
    let time = lwc1(0x3ad498);
    let y = lwc1(at + 4);
    let x = lwc1(at);
    let one = 1.0_f32;
    let z = lwc1(at + 8);
    let half_wide = 0x3fe8000000000000_i64;
    let factor = ld(0x3a9018);
    let one_wide = 0x3ff0000000000000_i64;
    let max_x = ps2float::add(ps2float::add(x, ps2float::mul(ps2float::sub(ps2float::add(dir_x, one_and_half), three_quarters), time)), one);
    let max_y = ps2float::sub(ps2float::add(ps2float::add(y, ps2float::mul(ps2float::add(dir_y, 0.0_f32), time)), lwc1(0x3ad49c)), one);
    let low_z = dptofp(dpsub(dpadd(fptodp(z), dpmul(dpsub(fptodp(ps2float::add(lwc1(direction + 8), 0.0_f32)), half_wide), factor)), one_wide));
    let high_z = dptofp(dpadd(dpadd(fptodp(lwc1(at + 8)), dpmul(dpsub(fptodp(ps2float::add(lwc1(direction + 8), one_and_half)), half_wide), factor)), one_wide));
    bgCalcRoomArray(
        rooms,
        ps2float::sub(ps2float::add(x, ps2float::mul(ps2float::sub(dir_x_plus, three_quarters), time)), one),
        max_x,
        max_y,
        ps2float::add(ps2float::add(y, ps2float::mul(ps2float::add(dir_y, one_and_half), time)), one),
        low_z,
        high_z,
    );
    group
}
