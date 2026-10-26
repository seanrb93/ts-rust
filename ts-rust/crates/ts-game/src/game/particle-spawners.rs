//! Starting particle groups: rays, tracers, trails, streams and manual ones.

use crate::*;

/// Start a ray: a group of kind 13 of life 5 with its start point `from` and end point `to` (points of three floats) at +928 and +940, owned by `owner` and tagged `tag` (@ 0x298100).
pub fn particleNewRay(owner: i32, from: i32, to: i32, tag: i32) {
    let group = getgroup(13);
    sw(group + 12, 0);
    swc1(group + 16, 5.0_f32);
    sw(group + 52, 0);
    sw(group + 20, owner);
    let x = lwc1(from);
    sw(group + 48, tag);
    swc1(group + 928, x);
    swc1(group + 932, lwc1(from + 4));
    swc1(group + 936, lwc1(from + 8));
    sw(group + 4, -1);
    let x = lwc1(to);
    sw(group + 8, 0);
    swc1(group + 940, x);
    sw(group + 24, 0);
    let y = lwc1(to + 4);
    sw(group + 44, 0);
    swc1(group + 944, y);
    swc1(group + 948, lwc1(to + 8));
}

/// Start a trail of the particle type `kind` at the point `at`: a group of kind 6 whose life is the sum of two floats of the type's info (104 bytes each) and whose count
/// is that of `count_kind` (@ 0x298260).
pub fn particleNewTrail(owner: i32, kind: i32, at: i32, count_kind: i32) -> i32 {
    let group = getgroup(6);
    sw(group + 12, 0);
    sw(group + 20, owner);
    sw(group + 4, kind);
    let info = kind * 104 + g::particleinfos;
    sw(group + 8, 0);
    swc1(group + 52, lwc1(info + 100));
    swc1(group + 16, ps2float::add(lwc1(info + 12), lwc1(info + 28)));
    let count = particleGetNum(count_kind);
    let x = lwc1(at);
    sw(group + 44, count);
    swc1(group + 28, x);
    sw(group + 24, 0);
    swc1(group + 32, lwc1(at + 4));
    swc1(group + 36, lwc1(at + 8));
    group
}

/// Start a tracer: like a ray, a group of kind 3 whose life is the game's constant at 0x3ad4bc (@ 0x2981b0).
pub fn particleNewTracer(owner: i32, from: i32, to: i32, tag: i32) -> i32 {
    let group = getgroup(3);
    let life = lwc1(0x3ad4bc);
    sw(group + 12, 0);
    swc1(group + 16, life);
    sw(group + 52, 0);
    sw(group + 20, owner);
    let x = lwc1(from);
    sw(group + 48, tag);
    swc1(group + 928, x);
    swc1(group + 932, lwc1(from + 4));
    swc1(group + 936, lwc1(from + 8));
    sw(group + 4, -1);
    let x = lwc1(to);
    sw(group + 8, 0);
    swc1(group + 940, x);
    sw(group + 24, 0);
    let y = lwc1(to + 4);
    sw(group + 44, 0);
    swc1(group + 944, y);
    swc1(group + 948, lwc1(to + 8));
    group
}

/// Start a stream of the particle type `kind` at the point `at` with the speed `speed`; `flag` (any value but 0) is kept as 1 at +8 (@ 0x298778).
pub fn particleNewStream(owner: i32, kind: i32, at: i32, flag: i32, count_kind: i32, speed: f32) -> i32 {
    let group = getgroup(5);
    sw(group + 12, 0);
    let infos = g::particleinfos;
    sw(group + 20, owner);
    sw(group + 4, kind);
    sw(group + 8, (flag != 0) as i32);
    swc1(group + 16, lwc1(infos + kind * 104 + 12));
    let count = particleGetNum(count_kind);
    let kind_now = lw(group + 4);
    sw(group + 44, count);
    sw(group + 24, 0);
    swc1(group + 52, lwc1(kind_now * 104 + infos + 100));
    swc1(group + 28, lwc1(at));
    swc1(group + 32, lwc1(at + 4));
    let z = lwc1(at + 8);
    swc1(group + 40, speed);
    swc1(group + 36, z);
    sw(group + 928, 0);
    group
}

/// Start a group of kind 11 that the game moves by hand, of the particle type `kind` at the point `at` (@ 0x298328).
pub fn particleNewManual(owner: i32, kind: i32, at: i32) -> i32 {
    let group = getgroup(11);
    sw(group + 12, 0);
    sw(group + 4, kind);
    sw(group + 8, 0);
    let info = kind * 104 + g::particleinfos;
    sw(group + 44, 0);
    let life = lwc1(info + 100);
    sw(group + 20, owner);
    swc1(group + 52, life);
    let count_kind = lw(info + 8);
    swc1(group + 16, lwc1(info + 12));
    let count = particleGetNum(count_kind);
    let x = lwc1(at);
    sw(group + 24, count);
    swc1(group + 28, x);
    swc1(group + 32, lwc1(at + 4));
    let z = lwc1(at + 8);
    sw(group + 40, 0);
    swc1(group + 36, z);
    group
}

/// The next random number as a float, from 0 up to 2^32 (the 32 bits as an unsigned number); the caller scales it.
pub(super) fn random_as_float() -> f32 {
    let r = newrnd();
    if r < 0 {
        let half = (r & 1 | ((r as i32 as u32) >> 1) as i32 as i64) as i32 as f32;
        ps2float::add(half, half)
    } else {
        r as i32 as f32
    }
}

/// Start a pulse (a group of kind 14 of life 5): the number kept at +928, a point at +932 and one at +944 (three floats each), and 20 random angles of 0 to 360 degrees from +956 (@ 0x297fa8).
pub fn particleNewPulse(owner: i32, number: i32, point: i32, direction: i32, tag: i32) -> i32 {
    let group = getgroup(14);
    let scale = 2.3283064e-10_f32;
    sw(group + 928, number);
    swc1(group + 16, 5.0_f32);
    sw(group + 12, 0);
    let mut angles = group + 956;
    sw(group + 52, 0);
    sw(group + 20, owner);
    let x = lwc1(point);
    sw(group + 4, -1);
    swc1(group + 932, x);
    sw(group + 24, 20);
    let y = lwc1(point + 4);
    sw(group + 48, tag);
    swc1(group + 936, y);
    sw(group + 8, 0);
    let z = lwc1(point + 8);
    sw(group + 44, 0);
    swc1(group + 940, z);
    swc1(group + 944, lwc1(direction));
    swc1(group + 948, lwc1(direction + 4));
    swc1(group + 952, lwc1(direction + 8));
    let mut i = 0;
    loop {
        let angle = ps2float::mul(random_as_float(), scale);
        i += 1;
        swc1(angles, ps2float::mul(angle, 360.0_f32));
        angles += 4;
        if i >= lw(group + 24) {
            break;
        }
    }
    group
}

/// Start smoke (a group of kind 2 of life 2) at the point `at`: a random number (from a table of the game's, kind 5) of puffs with random sideways speeds of -90 to 90, and the rooms
/// of a box round the point found for it (@ 0x297e30).
pub fn particleNewSmoke(owner: i32, at: i32) {
    let group = getgroup(2);
    sw(group + 20, owner);
    sw(group + 12, 0);
    sw(group + 52, 0);
    sw(group + 8, 4);
    swc1(group + 16, 2.0_f32);
    sw(group + 4, -1);
    let count = particleGetNum(5);
    let x = lwc1(at);
    sw(group + 24, count);
    swc1(group + 928, x);
    sw(group + 44, 0);
    swc1(group + 932, lwc1(at + 4));
    swc1(group + 936, lwc1(at + 8));
    let mut i = 0;
    if count > 0 {
        loop {
            let speeds = group + 948 + (i << 2);
            let r = random_as_float();
            i += 1;
            swc1(speeds, ps2float::mul(ps2float::sub(ps2float::mul(r, 2.3283064e-10_f32), 0.5_f32), 180.0_f32));
            if i >= lw(group + 24) {
                break;
            }
        }
    }
    let y = lwc1(at + 4);
    let two = 2.0_f32;
    let z = lwc1(at + 8);
    let top = ps2float::add(y, 1.5_f32);
    let x = lwc1(at);
    bgCalcRoomArray(group + 72, ps2float::sub(x, two), ps2float::add(x, two), ps2float::sub(y, two), ps2float::add(top, two), ps2float::sub(z, two), ps2float::add(z, two));
}
