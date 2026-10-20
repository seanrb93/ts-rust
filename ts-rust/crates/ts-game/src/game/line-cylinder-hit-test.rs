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
    let dx = lwc1(top) - bx;
    let tz = lwc1(top + 8);
    let dy = lwc1(top + 4) - by;
    let bz = lwc1(bottom + 8);
    swc1(sp + 64, dx);
    let dz = tz - bz;
    swc1(sp + 68, dy);
    swc1(sp + 72, dz);
    let inverse = 1.0_f32 / (dx * dx + dy * dy + dz * dz).abs().sqrt();
    let dir_z = lwc1(dir + 8);
    let dir_x = lwc1(dir);
    let dir_y = lwc1(dir + 4);
    let from_x = lwc1(from);
    let az = lwc1(sp + 72) * inverse;
    let from_y = lwc1(from + 4);
    let ax = lwc1(sp + 64) * inverse;
    let from_z = lwc1(from + 8);
    let ay = lwc1(sp + 68) * inverse;
    swc1(sp + 72, az);
    swc1(sp + 64, ax);
    swc1(sp + 68, ay);
    let cross_y = dir_z * ax - dir_x * az;
    let cross_x = dir_y * az - dir_z * ay;
    swc1(sp + 20, cross_y);
    swc1(sp + 16, cross_x);
    let cross_z = dir_x * ay - dir_y * ax;
    swc1(sp + 24, cross_z);
    swc1(sp, from_x - bx);
    swc1(sp + 4, from_y - by);
    swc1(sp + 8, from_z - bz);
    let rel_x = lwc1(sp);
    let length = (cross_x * cross_x + cross_y * cross_y + cross_z * cross_z).abs().sqrt();
    if !(length == 0.0_f32) {
        let unit = 1.0_f32 / (cross_x * cross_x + cross_y * cross_y + cross_z * cross_z).abs().sqrt();
        let rx = lwc1(sp);
        let ry = lwc1(sp + 4);
        let rz = lwc1(sp + 8);
        let ny = lwc1(sp + 20) * unit;
        let nx = lwc1(sp + 16) * unit;
        let nz = lwc1(sp + 24) * unit;
        swc1(sp + 20, ny);
        swc1(sp + 16, nx);
        let mut distance = rx * nx + ry * ny + rz * nz;
        swc1(sp + 24, nz);
        if !(0.0_f32 <= distance) {
            distance = -distance;
        }
        let distance_squared = distance * distance;
        if radius_squared < distance_squared {
            return 0;
        }
        let ax = lwc1(sp + 64);
        if hit == 0 && normal == 0 {
            return 1;
        }
        let ay = lwc1(sp + 68);
        let az = lwc1(sp + 72);
        let side_y = nz * ax - nx * az;
        let side_x = ny * az - nz * ay;
        swc1(sp + 52, side_y);
        swc1(sp + 48, side_x);
        let side_z = nx * ay - ny * ax;
        swc1(sp + 56, side_z);
        swc1(sp + 144, -ax);
        swc1(sp + 148, -ay);
        swc1(sp + 152, -az);
        let along = -((ry * az - rz * ay) * nx + (rz * ax - rx * az) * ny + (rx * ay - ry * ax) * nz) * unit;
        let side_unit = 1.0_f32 / (side_x * side_x + side_y * side_y + side_z * side_z).abs().sqrt();
        let remaining = radius_squared - distance_squared;
        let unit_x = lwc1(sp + 48) * side_unit;
        let unit_y = lwc1(sp + 52) * side_unit;
        swc1(sp + 56, lwc1(sp + 56) * side_unit);
        swc1(sp + 48, unit_x);
        swc1(sp + 52, unit_y);
        fptodp(remaining);
        let root = sqrt(fptodp(remaining));
        let slope = lwc1(dir) * lwc1(sp + 48) + lwc1(dir + 4) * lwc1(sp + 52) + lwc1(dir + 8) * lwc1(sp + 56);
        let quotient = dpdiv(root, fptodp(slope));
        let half_chord;
        if dpcmp(quotient, 0_i64) < 0 {
            fptodp(remaining);
            let root = sqrt(fptodp(remaining));
            let slope = lwc1(dir) * lwc1(sp + 48) + lwc1(dir + 4) * lwc1(sp + 52) + lwc1(dir + 8) * lwc1(sp + 56);
            half_chord = dpsub(0_i64, dpdiv(root, fptodp(slope)));
        } else {
            fptodp(remaining);
            let root = sqrt(fptodp(remaining));
            let slope = lwc1(dir) * lwc1(sp + 48) + lwc1(dir + 4) * lwc1(sp + 52) + lwc1(dir + 8) * lwc1(sp + 56);
            half_chord = dpdiv(root, fptodp(slope));
        }
        let (dir_x, dir_y, dir_z) = (lwc1(dir), lwc1(dir + 4), lwc1(dir + 8));
        let spread = dptofp(half_chord);
        let near = along - spread;
        let px = lwc1(from);
        let py = lwc1(from + 4);
        let far = along + spread;
        let pz = lwc1(from + 8);
        let tx = lwc1(top);
        let ty = lwc1(top + 4);
        let tz = lwc1(top + 8);
        let first_x = px + near * dir_x;
        let neg_x = lwc1(sp + 144);
        let first_y = py + near * dir_y;
        let neg_y = lwc1(sp + 148);
        let neg_z = lwc1(sp + 152);
        swc1(sp + 80, first_x);
        let first_z = pz + near * dir_z;
        swc1(sp + 84, first_y);
        let to_top_x = first_x - tx;
        let to_top_y = first_y - ty;
        let second_y = py + far * dir_y;
        swc1(sp + 88, first_z);
        let second_x = px + far * dir_x;
        swc1(sp + 112, to_top_x);
        swc1(sp + 116, to_top_y);
        let to_top_z = first_z - tz;
        swc1(sp + 100, second_y);
        swc1(sp + 96, second_x);
        let second_z = pz + far * dir_z;
        swc1(sp + 120, to_top_z);
        let far_top_x = second_x - tx;
        let far_top_y = second_y - ty;
        swc1(sp + 104, second_z);
        swc1(sp + 128, far_top_x);
        let far_top_z = second_z - tz;
        swc1(sp + 132, far_top_y);
        swc1(sp + 136, far_top_z);
        if (to_top_x * neg_x + to_top_y * neg_y + to_top_z * neg_z) < 0.0_f32 {
            if (far_top_x * neg_x + far_top_y * neg_y + far_top_z * neg_z) < 0.0_f32 {
                return 0;
            }
        }
        let b_x = lwc1(bottom);
        let b_y = lwc1(bottom + 4);
        let hit_x = lwc1(sp + 80);
        let hit_y = lwc1(sp + 84);
        let rel_x = hit_x - b_x;
        let b_z = lwc1(bottom + 8);
        let rel_y = hit_y - b_y;
        let axis_x = lwc1(sp + 64);
        let axis_y = lwc1(sp + 68);
        let hit_z = lwc1(sp + 88);
        let rel_z = hit_z - b_z;
        let second_rel_x = lwc1(sp + 96) - b_x;
        let axis_z = lwc1(sp + 72);
        let second_rel_y = lwc1(sp + 100) - b_y;
        let second_z = lwc1(sp + 104);
        swc1(sp + 120, rel_z);
        swc1(sp + 116, rel_y);
        let second_rel_z = second_z - b_z;
        swc1(sp + 128, second_rel_x);
        swc1(sp + 112, rel_x);
        swc1(sp + 132, second_rel_y);
        let above = rel_x * axis_x + rel_y * axis_y + rel_z * axis_z;
        swc1(sp + 136, second_rel_z);
        if above < 0.0_f32 {
            if (second_rel_x * axis_x + second_rel_y * axis_y + second_rel_z * axis_z) < 0.0_f32 {
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
        let centre_z = lwc1(bottom + 8) - above * axis_z;
        let centre_x = lwc1(bottom) - above * axis_x;
        let centre_y = lwc1(bottom + 4) - above * axis_y;
        swc1(sp + 168, centre_z);
        swc1(sp + 160, centre_x);
        swc1(sp + 164, centre_y);
        swc1(normal + 8, hit_z - centre_z);
        swc1(normal, hit_x - centre_x);
        swc1(normal + 4, hit_y - centre_y);
        return 1;
    }
    let axis_x = lwc1(sp + 64);
    let rel_y = lwc1(sp + 4);
    let axis_y = lwc1(sp + 68);
    let rel_z = lwc1(sp + 8);
    let axis_z = lwc1(sp + 72);
    let along = rel_x * axis_x + rel_y * axis_y + rel_z * axis_z;
    let off_x = rel_x - along * axis_x;
    let off_y = rel_y - along * axis_y;
    let off_z = rel_z - along * axis_z;
    swc1(sp + 32, off_x);
    swc1(sp + 36, off_y);
    swc1(sp + 40, off_z);
    if radius_squared < off_x * off_x + off_y * off_y + off_z * off_z {
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
    let to_top_x = from_x - lwc1(top);
    let top_z = lwc1(top + 8);
    let to_top_y = lwc1(from + 4) - top_y;
    let from_z = lwc1(from + 8);
    swc1(sp + 84, neg_y);
    let to_top_z = from_z - top_z;
    swc1(sp + 88, neg_z);
    swc1(sp + 80, neg_x);
    swc1(sp, to_top_x);
    let mut abs_along = along;
    swc1(sp + 4, to_top_y);
    swc1(sp + 8, to_top_z);
    let past_top = to_top_x * neg_x + to_top_y * neg_y + to_top_z * neg_z;
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
            swc1(hit, from_x - along * axis_x);
            swc1(hit + 4, lwc1(from + 4) - along * axis_y);
            swc1(hit + 8, lwc1(from + 8) - along * axis_z);
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
        let second = past_top * lwc1(sp + 84);
        let third = past_top * lwc1(sp + 88);
        swc1(hit, from_x - past_top * first);
        swc1(hit + 4, lwc1(from + 4) - second);
        swc1(hit + 8, lwc1(from + 8) - third);
    }
    if normal == 0 {
        return 1;
    }
    swc1(normal + 8, axis_z);
    swc1(normal, axis_x);
    swc1(normal + 4, axis_y);
    1
}
