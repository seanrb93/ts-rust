//! Moving a circle along a line to the point where it touches another point.

use crate::*;

/// How far a circle of radius `radius` at (`x`, `z`) moving in the direction (`dx`, `dz`) can go before it touches the point (`px`, `pz`); the
/// direction to push it away goes to `out` (two floats, a unit vector) if it is not 0.  A constant of the game (0x3ae440) when it does not touch (@ 0x208f70).
pub fn hittestMoveCircleToPoint(out: i32, x: f32, z: f32, radius: f32, dx: f32, dz: f32, px: f32, pz: f32) -> f32 {
    let ox = px - x;
    let oz = pz - z;
    let side = oz * dx - ox * dz;
    if !(radius < side) && !(radius < -side) {
        let half = ((radius + side) * (radius - side)).abs().psqrt();
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
            let inv = (1.0_f32).pdiv((xx * xx + y * y).abs().psqrt());
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

/// Move a circle (x, z, radius) in the direction (dx, dz) until it touches the circle (px, pz, radius2): the distance as a fraction of the length of the
/// direction, from 0 to 1 (@ 0x2094a0).
pub fn hittestMoveCircleToCircle(out: i32, x: f32, z: f32, radius: f32, dx: f32, dz: f32, px: f32, pz: f32, radius2: f32) -> f32 {
    let combined = radius + radius2;
    let inv = (1.0_f32).pdiv((dx * dx + dz * dz).abs().psqrt());
    let reach = hittestMoveCircleToPoint(out, x, z, combined, dx * inv, dz * inv, px, pz);
    if !(1.0 <= reach) {
        if !(reach <= 0.0) { reach * inv } else { 0.0 }
    } else {
        1.0
    }
}

/// Whether the ray from `origin` in the direction `dir` (a unit vector) hits the sphere at `center` (radius squared in `radius2`): 1 or 0; the first hit point is
/// written to `hit` and the vector from the center to it to `normal`, if they are not 0 (@ 0x20a8d8).
pub fn HitTestLineSphere(origin: i32, dir: i32, center: i32, hit: i32, normal: i32, radius2: f32) -> i32 {
    let (ox, oy, oz) = (lwc1(origin), lwc1(origin + 4), lwc1(origin + 8));
    let vx = ox - lwc1(center);
    let vy = oy - lwc1(center + 4);
    let vz = oz - lwc1(center + 8);
    let (dx, dy, dz) = (lwc1(dir), lwc1(dir + 4), lwc1(dir + 8));
    let along = vx * dx + vy * dy + vz * dz;
    let discriminant = along * along - (vx * vx + vy * vy + vz * vz - radius2);
    if !(0.0 <= discriminant) {
        return 0;
    }
    if hit == 0 && normal == 0 {
        return 1;
    }
    let t = -along - discriminant.abs().psqrt();
    let hz = oz + t * dz;
    let hx = ox + t * dx;
    let hy = oy + t * dy;
    if hit != 0 {
        swc1(hit + 8, hz);
        swc1(hit, hx);
        swc1(hit + 4, hy);
    }
    if normal != 0 {
        swc1(normal, hx - lwc1(center));
        swc1(normal + 4, hy - lwc1(center + 4));
        swc1(normal + 8, hz - lwc1(center + 8));
    }
    1
}

/// The squared distance of the point `p` from the line segment `a` to `b`; the nearest point of the segment goes to `nearest` if it is not 0 (@ 0x20add0).
pub fn HitTestDistancePointToLineSegmentSquared(p: i32, a: i32, b: i32, nearest: i32) -> f32 {
    let ax = lwc1(a);
    let ay = lwc1(a + 4);
    let vx = lwc1(p) - ax;
    let vy = lwc1(p + 4) - ay;
    let dx = lwc1(b) - ax;
    let az = lwc1(a + 8);
    let dy = lwc1(b + 4) - ay;
    let vz = lwc1(p + 8) - az;
    let dz = lwc1(b + 8) - az;
    let along = vx * dx + vy * dy + vz * dz;
    let (mut rx, mut ry, mut rz) = (vx, vy, vz);
    let t;
    if !(along <= 0.0) {
        let length2 = dx * dx + dy * dy + dz * dz;
        if !(length2 <= along) {
            t = (along).pdiv(length2);
            rx = vx - t * dx;
            ry = vy - t * dy;
            rz = vz - t * dz;
        } else {
            rx = vx - dx;
            t = 1.0;
            ry = vy - dy;
            rz = vz - dz;
        }
    } else {
        t = 0.0;
    }
    if nearest != 0 {
        swc1(nearest, ax + t * dx);
        swc1(nearest + 4, lwc1(a + 4) + t * dy);
        swc1(nearest + 8, lwc1(a + 8) + t * dz);
    }
    rx * rx + ry * ry + rz * rz
}
