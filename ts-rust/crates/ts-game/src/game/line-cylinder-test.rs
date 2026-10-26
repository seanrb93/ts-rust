//! A line against the side of an upright cylinder.

use crate::*;

/// Whether the line from the start point (the three floats on the caller's stack, just above this function's frame: x, y, z in slots 8 bytes apart) along the direction
/// (`dir_x`, `dir_y`, `dir_z`) meets the side of the cylinder that stands on the point (`base_x`, `base_y`, `base_z`), `height` high with the radius `radius`: the point where
/// it enters goes to `out` and the normal (the direction turned round, not a unit vector) to `normal`. Nothing is written when the line misses or enters through an end (@ 0x209dc8).
pub fn hittestLineCyl(out: i32, normal: i32, base_x: f32, base_y: f32, base_z: f32, radius: f32, height: f32, dir_x: f32, dir_y: f32, dir_z: f32) {
    let frame = Frame::new(144);
    let sp = frame.sp();
    let start_x = lwc1(sp + 144);
    let off_x = ps2float::sub(start_x, base_x);
    let start_z = lwc1(sp + 160);
    let off_z = ps2float::sub(start_z, base_z);
    let start_y = lwc1(sp + 152);
    let inv = (1.0_f32).pdiv(ps2float::add(ps2float::mul(dir_x, dir_x), ps2float::mul(dir_z, dir_z)).abs().psqrt());
    let across = ps2float::mul(ps2float::sub(ps2float::mul(off_z, dir_x), ps2float::mul(off_x, dir_z)), inv);
    if !(radius < across) && !(radius < -across) {
        let half = ps2float::mul(ps2float::add(radius, across), ps2float::sub(radius, across)).abs().psqrt();
        let along = ps2float::mul(ps2float::add(ps2float::mul(off_x, dir_x), ps2float::mul(off_z, dir_z)), -inv);
        let near = ps2float::sub(along, half);
        let far = ps2float::add(along, half);
        if near < 0.0_f32 && ps2float::mul(radius, radius) < ps2float::add(ps2float::mul(near, near), ps2float::mul(across, across)) {
            return;
        }
        let y_near;
        if !(far < 0.0_f32) {
            y_near = ps2float::mul(near, dir_y);
        } else {
            if ps2float::mul(radius, radius) < ps2float::add(ps2float::mul(far, far), ps2float::mul(across, across)) {
                return;
            }
            y_near = ps2float::mul(near, dir_y);
        }
        let top = ps2float::add(base_y, height);
        let y_enter = ps2float::add(start_y, y_near);
        let mut enter_side = 1;
        if !(top < y_enter) {
            enter_side = 2;
            if y_enter < base_y {
                enter_side = 3;
            }
        }
        let y_leave = ps2float::add(start_y, ps2float::mul(far, dir_y));
        let mut leave_side = 1;
        if !(top < y_leave) {
            leave_side = 2;
            if y_leave < base_y {
                leave_side = 3;
            }
        }
        if enter_side != leave_side {
            let below_three = enter_side < 3;
            if enter_side != 2 {
                if !below_three {
                    if enter_side != 3 {
                        return;
                    }
                } else if enter_side != 1 {
                    return;
                }
            }
        } else if enter_side != 2 {
            return;
        }
        swc1(out + 4, ps2float::add(start_y, y_near));
        swc1(out + 8, ps2float::add(start_z, ps2float::mul(near, dir_z)));
        swc1(out, ps2float::add(start_x, ps2float::mul(near, dir_x)));
        swc1(normal, -dir_x);
        swc1(normal + 4, -dir_y);
        swc1(normal + 8, -dir_z);
    }
}
