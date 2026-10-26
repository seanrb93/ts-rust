//! Starting sparks.

use super::particle_spawners::random_as_float;
use crate::*;

/// Find the rooms of the box round sparks at `at` for a group's room list `out`: the box runs from the point by the spread vector `spread` (three floats) scaled by a constant of
/// the game, `scale`, with `lift` added to the bottom (the end of both spark starters).
fn spark_rooms(out: i32, at: i32, spread: i32, scale: f32, lift: f32) {
    let sy = lwc1(spread + 4);
    let sz = lwc1(spread + 8);
    let one = 1.0_f32;
    let three = 3.0_f32;
    let half = 0.5_f32;
    let one_and_half = 1.5_f32;
    let sx = lwc1(spread);
    let y = lwc1(at + 4);
    let z = lwc1(at + 8);
    let x = lwc1(at);
    let max_z = ps2float::add(ps2float::add(z, ps2float::mul(ps2float::sub(ps2float::mul(ps2float::add(sz, one), three), one_and_half), scale)), one);
    let min_x = ps2float::sub(ps2float::add(x, ps2float::mul(ps2float::sub(ps2float::mul(ps2float::add(sx, 0.0_f32), three), one_and_half), scale)), one);
    let max_x = ps2float::add(ps2float::add(x, ps2float::mul(ps2float::sub(ps2float::mul(ps2float::add(sx, one), three), one_and_half), scale)), one);
    let min_y = ps2float::sub(ps2float::add(ps2float::add(y, ps2float::mul(ps2float::sub(ps2float::mul(ps2float::add(sy, 0.0_f32), three), half), scale)), lift), one);
    let max_y = ps2float::add(ps2float::add(y, ps2float::mul(ps2float::sub(ps2float::mul(ps2float::add(sy, one), three), half), scale)), one);
    let min_z = ps2float::sub(ps2float::add(z, ps2float::mul(ps2float::sub(ps2float::mul(ps2float::add(sz, 0.0_f32), three), one_and_half), scale)), one);
    bgCalcRoomArray(out, min_x, max_x, min_y, max_y, min_z, max_z);
}

/// Write the speeds of a group's sparks (12 bytes each from `speeds`): on each axis a random number plus the spread vector's, times 3, less 1.5 (0.5 for the second axis).
fn spark_speeds(group: i32, speeds: i32, spread: i32) {
    let mut i = 0;
    loop {
        let at = i * 12 + speeds;
        let r = random_as_float();
        swc1(at, ps2float::sub(ps2float::mul(ps2float::add(ps2float::mul(r, 2.3283064e-10_f32), lwc1(spread)), 3.0_f32), 1.5_f32));
        let at = i * 12 + (speeds + 4);
        let r = random_as_float();
        swc1(at, ps2float::sub(ps2float::mul(ps2float::add(ps2float::mul(r, 2.3283064e-10_f32), lwc1(spread + 4)), 3.0_f32), 0.5_f32));
        let at = i * 12 + (speeds + 8);
        let r = random_as_float();
        i += 1;
        swc1(at, ps2float::sub(ps2float::mul(ps2float::add(ps2float::mul(r, 2.3283064e-10_f32), lwc1(spread + 8)), 3.0_f32), 1.5_f32));
        if i >= lw(group + 24) {
            break;
        }
    }
}

/// Start sparks (a group of kind 0): the game's number of them (kind 46), each with a random speed round the spread vector `spread`; `at` is the point (@ 0x296ec0).
pub fn particleNewSparks(owner: i32, at: i32, spread: i32) {
    let group = getgroup(0);
    let life = lwc1(0x3ad488);
    sw(group + 20, owner);
    sw(group + 12, 0);
    sw(group + 52, 0);
    sw(group + 8, 4);
    swc1(group + 16, life);
    sw(group + 4, -1);
    let count = particleGetNum(46);
    let x = lwc1(at);
    sw(group + 24, count);
    swc1(group + 928, x);
    sw(group + 44, 0);
    swc1(group + 932, lwc1(at + 4));
    swc1(group + 936, lwc1(at + 8));
    if count > 0 {
        spark_speeds(group, group + 940, spread);
    }
    let scale = lwc1(0x3ad48c);
    let lift = lwc1(0x3ad490);
    spark_rooms(group + 72, at, spread, scale, lift);
}

/// Start sparks that are sprites (a group of kind 1): the game's number of them (kind 30), the number `number` kept at +928 (@ 0x297598).
pub fn particleNewSpriteSparks(owner: i32, number: i32, at: i32, spread: i32) {
    let group = getgroup(1);
    let life = lwc1(0x3ad4a0);
    sw(group + 20, owner);
    sw(group + 12, 0);
    sw(group + 52, 0);
    sw(group + 8, 4);
    swc1(group + 16, life);
    sw(group + 4, -1);
    let count = particleGetNum(30);
    sw(group + 928, number);
    sw(group + 24, count);
    let x = lwc1(at);
    sw(group + 44, 0);
    swc1(group + 932, x);
    swc1(group + 936, lwc1(at + 4));
    swc1(group + 940, lwc1(at + 8));
    if count > 0 {
        spark_speeds(group, group + 944, spread);
    }
    let scale = lwc1(0x3ad4a4);
    let lift = lwc1(0x3ad4a8);
    spark_rooms(group + 72, at, spread, scale, lift);
}
