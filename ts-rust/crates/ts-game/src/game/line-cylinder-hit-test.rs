//! The hit test of a line against a cylinder.

use crate::*;

/// Test the line from `from` along the direction `dir` (three floats each) against the cylinder with its bottom end `bottom` and top end `top` (three floats each) and the
/// squared radius `radius_squared`: the result is 1 for a hit and 0 for none; `hit` (when not 0) gets the place where the line first meets the cylinder (the side, or an end cap)
/// and `normal` (when not 0) the surface normal there. The line parallel to the axis is tested against the caps and the radius alone (@ 0x20a080).
pub fn HitTestLineCylinder(from: i32, dir: i32, bottom: i32, top: i32, hit: i32, normal: i32, radius_squared: f32) -> i32 {
    let frame = Frame::new(352);
    let sp = frame.sp();
    let bx = lwc1(bottom);
    let by = lwc1(bottom + 4);
    let dx = ps2float::sub(lwc1(top), bx);
    let tz = lwc1(top + 8);
    let dy = ps2float::sub(lwc1(top + 4), by);
    let bz = lwc1(bottom + 8);
    swc1(sp + 64, dx);
    let dz = ps2float::sub(tz, bz);
    swc1(sp + 68, dy);
    swc1(sp + 72, dz);
    let inverse = (1.0_f32).pdiv(ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz)).abs().psqrt());
    let dir_z = lwc1(dir + 8);
    let dir_x = lwc1(dir);
    let dir_y = lwc1(dir + 4);
    let from_x = lwc1(from);
    let az = ps2float::mul(lwc1(sp + 72), inverse);
    let from_y = lwc1(from + 4);
    let ax = ps2float::mul(lwc1(sp + 64), inverse);
    let from_z = lwc1(from + 8);
    let ay = ps2float::mul(lwc1(sp + 68), inverse);
    swc1(sp + 72, az);
    swc1(sp + 64, ax);
    swc1(sp + 68, ay);
    let cross_y = ps2float::sub(ps2float::mul(dir_z, ax), ps2float::mul(dir_x, az));
    let cross_x = ps2float::sub(ps2float::mul(dir_y, az), ps2float::mul(dir_z, ay));
    swc1(sp + 20, cross_y);
    swc1(sp + 16, cross_x);
    let cross_z = ps2float::sub(ps2float::mul(dir_x, ay), ps2float::mul(dir_y, ax));
    swc1(sp + 24, cross_z);
    swc1(sp, ps2float::sub(from_x, bx));
    swc1(sp + 4, ps2float::sub(from_y, by));
    swc1(sp + 8, ps2float::sub(from_z, bz));
    let rel_x = lwc1(sp);
    let length = ps2float::add(ps2float::add(ps2float::mul(cross_x, cross_x), ps2float::mul(cross_y, cross_y)), ps2float::mul(cross_z, cross_z)).abs().psqrt();
    if !(length == 0.0_f32) {
        let unit = (1.0_f32).pdiv(ps2float::add(ps2float::add(ps2float::mul(cross_x, cross_x), ps2float::mul(cross_y, cross_y)), ps2float::mul(cross_z, cross_z)).abs().psqrt());
        let rx = lwc1(sp);
        let ry = lwc1(sp + 4);
        let rz = lwc1(sp + 8);
        let ny = ps2float::mul(lwc1(sp + 20), unit);
        let nx = ps2float::mul(lwc1(sp + 16), unit);
        let nz = ps2float::mul(lwc1(sp + 24), unit);
        swc1(sp + 20, ny);
        swc1(sp + 16, nx);
        let mut distance = ps2float::add(ps2float::add(ps2float::mul(rx, nx), ps2float::mul(ry, ny)), ps2float::mul(rz, nz));
        swc1(sp + 24, nz);
        if !(0.0_f32 <= distance) {
            distance = -distance;
        }
        let distance_squared = ps2float::mul(distance, distance);
        if radius_squared < distance_squared {
            return 0;
        }
        let ax = lwc1(sp + 64);
        if hit == 0 && normal == 0 {
            return 1;
        }
        let ay = lwc1(sp + 68);
        let az = lwc1(sp + 72);
        let side_y = ps2float::sub(ps2float::mul(nz, ax), ps2float::mul(nx, az));
        let side_x = ps2float::sub(ps2float::mul(ny, az), ps2float::mul(nz, ay));
        swc1(sp + 52, side_y);
        swc1(sp + 48, side_x);
        let side_z = ps2float::sub(ps2float::mul(nx, ay), ps2float::mul(ny, ax));
        swc1(sp + 56, side_z);
        swc1(sp + 144, -ax);
        swc1(sp + 148, -ay);
        swc1(sp + 152, -az);
        let along = ps2float::mul(-ps2float::add(ps2float::add(ps2float::mul(ps2float::sub(ps2float::mul(ry, az), ps2float::mul(rz, ay)), nx), ps2float::mul(ps2float::sub(ps2float::mul(rz, ax), ps2float::mul(rx, az)), ny)), ps2float::mul(ps2float::sub(ps2float::mul(rx, ay), ps2float::mul(ry, ax)), nz)), unit);
        let side_unit = (1.0_f32).pdiv(ps2float::add(ps2float::add(ps2float::mul(side_x, side_x), ps2float::mul(side_y, side_y)), ps2float::mul(side_z, side_z)).abs().psqrt());
        let remaining = ps2float::sub(radius_squared, distance_squared);
        let unit_x = ps2float::mul(lwc1(sp + 48), side_unit);
        let unit_y = ps2float::mul(lwc1(sp + 52), side_unit);
        swc1(sp + 56, ps2float::mul(lwc1(sp + 56), side_unit));
        swc1(sp + 48, unit_x);
        swc1(sp + 52, unit_y);
        fptodp(remaining);
        let root = sqrt(fptodp(remaining));
        let slope = ps2float::add(ps2float::add(ps2float::mul(lwc1(dir), lwc1(sp + 48)), ps2float::mul(lwc1(dir + 4), lwc1(sp + 52))), ps2float::mul(lwc1(dir + 8), lwc1(sp + 56)));
        let quotient = dpdiv(root, fptodp(slope));
        let half_chord;
        if dpcmp(quotient, 0_i64) < 0 {
            fptodp(remaining);
            let root = sqrt(fptodp(remaining));
            let slope = ps2float::add(ps2float::add(ps2float::mul(lwc1(dir), lwc1(sp + 48)), ps2float::mul(lwc1(dir + 4), lwc1(sp + 52))), ps2float::mul(lwc1(dir + 8), lwc1(sp + 56)));
            half_chord = dpsub(0_i64, dpdiv(root, fptodp(slope)));
        } else {
            fptodp(remaining);
            let root = sqrt(fptodp(remaining));
            let slope = ps2float::add(ps2float::add(ps2float::mul(lwc1(dir), lwc1(sp + 48)), ps2float::mul(lwc1(dir + 4), lwc1(sp + 52))), ps2float::mul(lwc1(dir + 8), lwc1(sp + 56)));
            half_chord = dpdiv(root, fptodp(slope));
        }
        let (dir_x, dir_y, dir_z) = (lwc1(dir), lwc1(dir + 4), lwc1(dir + 8));
        let spread = dptofp(half_chord);
        let near = ps2float::sub(along, spread);
        let px = lwc1(from);
        let py = lwc1(from + 4);
        let far = ps2float::add(along, spread);
        let pz = lwc1(from + 8);
        let tx = lwc1(top);
        let ty = lwc1(top + 4);
        let tz = lwc1(top + 8);
        let first_x = ps2float::add(px, ps2float::mul(near, dir_x));
        let neg_x = lwc1(sp + 144);
        let first_y = ps2float::add(py, ps2float::mul(near, dir_y));
        let neg_y = lwc1(sp + 148);
        let neg_z = lwc1(sp + 152);
        swc1(sp + 80, first_x);
        let first_z = ps2float::add(pz, ps2float::mul(near, dir_z));
        swc1(sp + 84, first_y);
        let to_top_x = ps2float::sub(first_x, tx);
        let to_top_y = ps2float::sub(first_y, ty);
        let second_y = ps2float::add(py, ps2float::mul(far, dir_y));
        swc1(sp + 88, first_z);
        let second_x = ps2float::add(px, ps2float::mul(far, dir_x));
        swc1(sp + 112, to_top_x);
        swc1(sp + 116, to_top_y);
        let to_top_z = ps2float::sub(first_z, tz);
        swc1(sp + 100, second_y);
        swc1(sp + 96, second_x);
        let second_z = ps2float::add(pz, ps2float::mul(far, dir_z));
        swc1(sp + 120, to_top_z);
        let far_top_x = ps2float::sub(second_x, tx);
        let far_top_y = ps2float::sub(second_y, ty);
        swc1(sp + 104, second_z);
        swc1(sp + 128, far_top_x);
        let far_top_z = ps2float::sub(second_z, tz);
        swc1(sp + 132, far_top_y);
        swc1(sp + 136, far_top_z);
        if ps2float::add(ps2float::add(ps2float::mul(to_top_x, neg_x), ps2float::mul(to_top_y, neg_y)), ps2float::mul(to_top_z, neg_z)) < 0.0_f32 {
            if ps2float::add(ps2float::add(ps2float::mul(far_top_x, neg_x), ps2float::mul(far_top_y, neg_y)), ps2float::mul(far_top_z, neg_z)) < 0.0_f32 {
                return 0;
            }
        }
        let b_x = lwc1(bottom);
        let b_y = lwc1(bottom + 4);
        let hit_x = lwc1(sp + 80);
        let hit_y = lwc1(sp + 84);
        let rel_x = ps2float::sub(hit_x, b_x);
        let b_z = lwc1(bottom + 8);
        let rel_y = ps2float::sub(hit_y, b_y);
        let axis_x = lwc1(sp + 64);
        let axis_y = lwc1(sp + 68);
        let hit_z = lwc1(sp + 88);
        let rel_z = ps2float::sub(hit_z, b_z);
        let second_rel_x = ps2float::sub(lwc1(sp + 96), b_x);
        let axis_z = lwc1(sp + 72);
        let second_rel_y = ps2float::sub(lwc1(sp + 100), b_y);
        let second_z = lwc1(sp + 104);
        swc1(sp + 120, rel_z);
        swc1(sp + 116, rel_y);
        let second_rel_z = ps2float::sub(second_z, b_z);
        swc1(sp + 128, second_rel_x);
        swc1(sp + 112, rel_x);
        swc1(sp + 132, second_rel_y);
        let above = ps2float::add(ps2float::add(ps2float::mul(rel_x, axis_x), ps2float::mul(rel_y, axis_y)), ps2float::mul(rel_z, axis_z));
        swc1(sp + 136, second_rel_z);
        if above < 0.0_f32 {
            if ps2float::add(ps2float::add(ps2float::mul(second_rel_x, axis_x), ps2float::mul(second_rel_y, axis_y)), ps2float::mul(second_rel_z, axis_z)) < 0.0_f32 {
                return 0;
            }
        }
        if hit != 0 {
            swc1(hit + 8, hit_z);
            swc1(hit, hit_x);
            swc1(hit + 4, hit_y);
        }
        if normal == 0 {
            return 1;
        }
        let centre_z = ps2float::sub(lwc1(bottom + 8), ps2float::mul(above, axis_z));
        let centre_x = ps2float::sub(lwc1(bottom), ps2float::mul(above, axis_x));
        let centre_y = ps2float::sub(lwc1(bottom + 4), ps2float::mul(above, axis_y));
        swc1(sp + 168, centre_z);
        swc1(sp + 160, centre_x);
        swc1(sp + 164, centre_y);
        swc1(normal + 8, ps2float::sub(hit_z, centre_z));
        swc1(normal, ps2float::sub(hit_x, centre_x));
        swc1(normal + 4, ps2float::sub(hit_y, centre_y));
        return 1;
    }
    let axis_x = lwc1(sp + 64);
    let rel_y = lwc1(sp + 4);
    let axis_y = lwc1(sp + 68);
    let rel_z = lwc1(sp + 8);
    let axis_z = lwc1(sp + 72);
    let along = ps2float::add(ps2float::add(ps2float::mul(rel_x, axis_x), ps2float::mul(rel_y, axis_y)), ps2float::mul(rel_z, axis_z));
    let off_x = ps2float::sub(rel_x, ps2float::mul(along, axis_x));
    let off_y = ps2float::sub(rel_y, ps2float::mul(along, axis_y));
    let off_z = ps2float::sub(rel_z, ps2float::mul(along, axis_z));
    swc1(sp + 32, off_x);
    swc1(sp + 36, off_y);
    swc1(sp + 40, off_z);
    if radius_squared < ps2float::add(ps2float::add(ps2float::mul(off_x, off_x), ps2float::mul(off_y, off_y)), ps2float::mul(off_z, off_z)) {
        return 0;
    }
    let top_y;
    if hit != 0 {
        top_y = lwc1(top + 4);
    } else {
        if normal == 0 {
            return 1;
        }
        top_y = lwc1(top + 4);
    }
    let neg_y = -axis_y;
    let from_x = lwc1(from);
    let neg_x = -axis_x;
    let neg_z = -axis_z;
    let to_top_x = ps2float::sub(from_x, lwc1(top));
    let top_z = lwc1(top + 8);
    let to_top_y = ps2float::sub(lwc1(from + 4), top_y);
    let from_z = lwc1(from + 8);
    swc1(sp + 84, neg_y);
    let to_top_z = ps2float::sub(from_z, top_z);
    swc1(sp + 88, neg_z);
    swc1(sp + 80, neg_x);
    swc1(sp, to_top_x);
    let mut abs_along = along;
    swc1(sp + 4, to_top_y);
    swc1(sp + 8, to_top_z);
    let past_top = ps2float::add(ps2float::add(ps2float::mul(to_top_x, neg_x), ps2float::mul(to_top_y, neg_y)), ps2float::mul(to_top_z, neg_z));
    if !(0.0_f32 <= along) {
        abs_along = -along;
    }
    let side;
    if !(0.0_f32 <= past_top) {
        side = abs_along < -past_top;
    } else {
        side = abs_along < past_top;
    }
    if side {
        if hit != 0 {
            swc1(hit, ps2float::sub(from_x, ps2float::mul(along, axis_x)));
            swc1(hit + 4, ps2float::sub(lwc1(from + 4), ps2float::mul(along, axis_y)));
            swc1(hit + 8, ps2float::sub(lwc1(from + 8), ps2float::mul(along, axis_z)));
        }
        let first = lwc1(sp + 80);
        if normal == 0 {
            return 1;
        }
        let second = lwc1(sp + 84);
        let third = lwc1(sp + 88);
        swc1(normal, first);
        swc1(normal + 8, third);
        swc1(normal + 4, second);
        return 1;
    }
    let first = lwc1(sp + 80);
    if hit != 0 {
        let second = ps2float::mul(past_top, lwc1(sp + 84));
        let third = ps2float::mul(past_top, lwc1(sp + 88));
        swc1(hit, ps2float::sub(from_x, ps2float::mul(past_top, first)));
        swc1(hit + 4, ps2float::sub(lwc1(from + 4), second));
        swc1(hit + 8, ps2float::sub(lwc1(from + 8), third));
    }
    if normal == 0 {
        return 1;
    }
    swc1(normal + 8, axis_z);
    swc1(normal, axis_x);
    swc1(normal + 4, axis_y);
    1
}
