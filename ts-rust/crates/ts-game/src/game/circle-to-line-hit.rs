//! A moving circle against a line segment.

use crate::*;

/// How far along its move (0 to 1) a circle of radius `radius` at (`x`, `z`) moving by (`dx`, `dz`) first touches the line segment from (`x1`, `z1`) to (`x2`, `z2`); the last
/// number, `z2`, is the eleventh float and is put on the caller's stack. 1 if it does not touch within the move, 2 if it is moving away from the line (or is already
/// inside), 0 if it starts touching. The normal of the line (a unit vector in x and z) goes to `out` (not 0) when the circle meets the side of the segment; the ends are tested as points (@ 0x209150).
pub fn hittestMoveCircleToLine(out: i32, x: f32, z: f32, radius: f32, dx: f32, dz: f32, x1: f32, z1: f32, x2: f32) -> f32 {
    let frame = Frame::new(160);
    let z2 = lwc1(frame.sp() + 160);
    let speed = (dx * dx + dz * dz).abs().sqrt();
    if speed == 0.0_f32 {
        return 2.0_f32;
    }
    let line_x = x2 - x1;
    let line_z = z2 - z1;
    let inv_speed = 1.0_f32 / speed;
    let line_len2 = line_x * line_x + line_z * line_z;
    let reach;
    if !(0.0_f32 < line_len2) {
        reach = hittestMoveCircleToPoint(out, x, z, radius, dx * inv_speed, dz * inv_speed, x2, z2);
    } else {
        if 0.0_f32 <= line_z * dx - line_x * dz {
            return 2.0_f32;
        }
        let rel_x = x - x1;
        let rel_z = z - z1;
        if (line_z * rel_x - line_x * rel_z) < 0.0_f32 {
            return 1.0_f32;
        }
        let along = rel_x * line_x + rel_z * line_z;
        let point_x;
        let point_z;
        if !(along < 0.0_f32) {
            if !(line_len2 < along) {
                let inv = 1.0_f32 / line_len2.abs().sqrt();
                let nx = line_z * inv;
                let nz = -line_x * inv;
                if out != 0 {
                    swc1(out + 4, nz);
                    swc1(out, nx);
                }
                let start_side = nx * rel_x + nz * rel_z;
                let end_side = nx * (x + dx - x1) + nz * (z + dz - z1);
                let mut found = None;
                if !(start_side == end_side) {
                    if !(start_side < end_side) {
                        if radius <= end_side {
                            found = Some(speed);
                        } else if start_side <= -radius {
                            found = Some(speed);
                        } else if radius <= start_side {
                            found = Some(start_side - radius);
                        }
                    } else if radius <= start_side {
                        found = Some(speed);
                    } else if end_side <= -radius {
                        found = Some(speed);
                    } else if start_side <= -radius {
                        found = Some(-start_side - radius);
                    }
                }
                match found {
                    Some(distance) => reach = distance,
                    None => return 2.0_f32,
                }
                return finish(reach, speed, inv_speed);
            }
            point_x = x2;
            point_z = z2;
        } else {
            point_x = x1;
            point_z = z1;
        }
        reach = hittestMoveCircleToPoint(out, x, z, radius, dx * inv_speed, dz * inv_speed, point_x, point_z);
    }
    finish(reach, speed, inv_speed)
}

/// The result of the test: the distance `reach` moved before touching, as part of the whole move of length `speed` (0 if it starts touching), or 1 if it is not reached.
fn finish(reach: f32, speed: f32, inv_speed: f32) -> f32 {
    if !(speed <= reach) {
        if !(reach <= 0.0_f32) {
            return reach * inv_speed;
        }
        return 0.0_f32;
    }
    1.0_f32
}
