//! Moving a circle along a line to the point where it touches another point.

use crate::*;

/// How far a circle of radius `radius` at (`x`, `z`) moving in the direction (`dx`, `dz`) can go before it touches the point (`px`, `pz`); the
/// direction to push it away goes to `out` (two floats, a unit vector) if it is not 0.  A constant of the game (0x3ae440) when it does not touch (@ 0x208f70).
pub fn hittestMoveCircleToPoint(out: i32, x: f32, z: f32, radius: f32, dx: f32, dz: f32, px: f32, pz: f32) -> f32 {
    let ox = px - x;
    let oz = pz - z;
    let side = oz * dx - ox * dz;
    if !(radius < side) && !(radius < -side) {
        let half = ((radius + side) * (radius - side)).abs().sqrt();
        let mut reach = ox * dx + oz * dz - half;
        if lwc1(0x3ac808) < reach && reach < 0.0 {
            reach = 0.0;
        }
        if !(reach < 0.0) {
            if out == 0 {
                return reach;
            }
            let y = -(half * dz) + side * -dx;
            let xx = -(half * dx) + side * dz;
            swc1(out + 4, y);
            swc1(out, xx);
            let inv = 1.0_f32 / (xx * xx + y * y).abs().sqrt();
            let ny = lwc1(out + 4) * inv;
            swc1(out, lwc1(out) * inv);
            swc1(out + 4, ny);
            return reach;
        }
        if out != 0 {
            swc1(out, -dx);
            swc1(out + 4, -dz);
        }
        let behind = reach * reach;
        if (behind + side * side) < radius * radius - lwc1(0x3ac80c) {
            return 0.0;
        }
    }
    lwc1(0x3ae440)
}
