//! Moving the particles of trail and stream groups.

use super::particle_spawners::random_as_float;
use crate::*;

/// A random number from -0.5 up to 0.5.
fn random_centred() -> f32 {
    ps2float::sub(ps2float::mul(random_as_float(), 2.3283064e-10_f32), 0.5_f32)
}

/// Add new points to a trail group (points of three floats at +928, +932 and +936 with a stride of 12, up to 100; the time of each point from +3728, and its spread at +2128, +2528,
/// +2532, +2536) as many as the time since the last one allows at the group's rate (+44), taking their spread from the type's info (104 bytes each); the group is freed when it
/// has no rate and is empty or its last point has lived out its life (@ 0x29ae88).
pub fn particleTrailTick(group: i32) {
    let mut todo = 0;
    let rate = lw(group + 44);
    let count = lw(group + 24);
    if rate == 0 {
        if count == 0 {
            particleFree(group);
            return;
        }
        if ps2float::add(lwc1(group + 3728), lwc1(group + 16)) <= lwc1(group + 12) {
            particleFree(group);
            return;
        }
    }
    if count != 0 {
        if count < 100 {
            todo = ((ps2float::sub(lwc1(group + 12), lwc1(group + 3728))).pdiv(((1.0_f32).pdiv(rate as f32)))) as i32;
        }
    } else {
        todo = 1;
    }
    if todo > 0 && count < 100 {
        let mut made = 0;
        loop {
            let count = lw(group + 24);
            swc1(group + 928 + count * 12, lwc1(group + 28));
            swc1(group + 932 + lw(group + 24) * 12, lwc1(group + 32));
            swc1(group + 936 + lw(group + 24) * 12, lwc1(group + 36));
            swc1(group + 3728, lwc1(group + 12));
            let r = random_centred();
            let at = group + 2128 + (lw(group + 24) << 2);
            let kind = lw(group + 4);
            let info = g::particleinfos;
            swc1(at, ps2float::mul(r, lwc1(info + kind * 104 + 36)));
            let r = random_centred();
            let at = lw(group + 24) * 12 + (group + 2528);
            let spread = lw(group + 4) * 104 + (info + 68);
            let base = lw(group + 4) * 104 + (info + 56);
            swc1(at, ps2float::add(lwc1(base), ps2float::mul(lwc1(spread), r)));
            let r = random_centred();
            let kind = lw(group + 4);
            let at = lw(group + 24) * 12 + (group + 2532);
            swc1(at, ps2float::add(ps2float::add(lwc1(kind * 104 + (info + 60)), ps2float::mul(lwc1(kind * 104 + (info + 72)), r)), lwc1(info + kind * 104 + 52)));
            let r = random_centred();
            let at = lw(group + 24) * 12 + (group + 2536);
            let spread = lw(group + 4) * 104 + (info + 76);
            let base = lw(group + 4) * 104 + (info + 64);
            made += 1;
            let more = made < todo;
            swc1(at, ps2float::add(lwc1(base), ps2float::mul(lwc1(spread), r)));
            let now = lw(group + 24) + 1;
            sw(group + 24, now);
            if !more || now >= 100 {
                break;
            }
        }
    }
}

/// Move a stream group: the head (a float at +928, the points in a ring of 100) moves on by the rate (+44) and the frame time in 60ths of a second, and each new place gets
/// a start point (+932 ...), a random speed (+2132 ...) and the time (+3332) and a spread (+3732) from the type's info (104 bytes each, at `info`); then all points move on by
/// their speed (a speed of the info is added to the second); the group is freed when it has no rate and the point last made has lived out its life (@ 0x29b410).
pub fn particleStreamTick(group: i32) {
    let head = lwc1(group + 928);
    let rate = lw(group + 44);
    let mut index = lwc1(group + 928) as i32;
    let before = index - 1;
    let info = lw(group + 4) * 104 + g::particleinfos;
    let step = (update_ticks_f()).pdiv(60.0_f32);
    let last = if -1 >= before { index + 99 } else { before };
    if rate == 0 {
        let count = lw(group + 24);
        if count == 0 || ps2float::add(lwc1(group + (last << 2) + 3332), lwc1(group + 16)) <= lwc1(group + 12) {
            particleFree(group);
            return;
        }
    }
    let count = lw(group + 24);
    let moved = ps2float::add(head, ps2float::mul(rate as f32, ((update_ticks_f()).pdiv(60.0_f32))));
    let whole = moved as i32;
    swc1(group + 928, moved);
    if count < whole {
        sw(group + 24, whole);
    }
    let mut position = lwc1(group + 928);
    if (position as i32) >= 100 {
        sw(group + 24, 100);
        let wrapped = ps2float::sub(position, 100.0_f32);
        swc1(group + 928, wrapped);
        position = wrapped;
    }
    if index != position as i32 {
        loop {
            let offset = index * 12;
            swc1(group + 932 + index * 12, lwc1(group + 28));
            swc1(group + 936 + index * 12, lwc1(group + 32));
            swc1(group + 940 + index * 12, lwc1(group + 36));
            let r = random_centred();
            swc1(group + 2132 + offset, ps2float::add(lwc1(info + 56), ps2float::mul(lwc1(info + 68), r)));
            let r = random_centred();
            swc1(group + 2136 + offset, ps2float::add(lwc1(info + 60), ps2float::mul(lwc1(info + 72), r)));
            let r = random_centred();
            let times = group + 3332 + (index << 2);
            swc1(group + 2140 + offset, ps2float::add(lwc1(info + 64), ps2float::mul(lwc1(info + 76), r)));
            swc1(times, lwc1(group + 12));
            let r = random_centred();
            let spreads = group + 3732 + (index << 2);
            let next = index + 1;
            index = if next >= 100 { 0 } else { next };
            swc1(spreads, ps2float::mul(r, lwc1(info + 36)));
            if index == lwc1(group + 928) as i32 {
                break;
            }
        }
    }
    let total = lw(group + 24);
    if total > 0 {
        let mut k = 0;
        loop {
            let offset = k * 12;
            k += 1;
            let y_speed = group + 2136 + offset;
            let x_at = group + 932 + offset;
            let y_at = group + 936 + offset;
            let z_at = group + 940 + offset;
            swc1(y_speed, ps2float::add(lwc1(y_speed), ps2float::mul(lwc1(info + 52), step)));
            swc1(x_at, ps2float::add(lwc1(x_at), ps2float::mul(lwc1(group + 2132 + offset), step)));
            swc1(y_at, ps2float::add(lwc1(y_at), ps2float::mul(lwc1(y_speed), step)));
            swc1(z_at, ps2float::add(lwc1(z_at), ps2float::mul(lwc1(group + 2140 + offset), step)));
            if k >= lw(group + 24) {
                break;
            }
        }
    }
}
