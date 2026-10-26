//! Moving a circle along a line to the point where it touches another point.

use crate::*;

/// How far a circle of radius `radius` at (`x`, `z`) moving in the direction (`dx`, `dz`) can go before it touches the point (`px`, `pz`); the
/// direction to push it away goes to `out` (two floats, a unit vector) if it is not 0.  A constant of the game (0x3ae440) when it does not touch (@ 0x208f70).
pub fn hittestMoveCircleToPoint(out: i32, x: f32, z: f32, radius: f32, dx: f32, dz: f32, px: f32, pz: f32) -> f32 {
    let ox = ps2float::sub(px, x);
    let oz = ps2float::sub(pz, z);
    let side = ps2float::sub(ps2float::mul(oz, dx), ps2float::mul(ox, dz));
    if !(radius < side) && !(radius < -side) {
        let half = ps2float::mul(ps2float::add(radius, side), ps2float::sub(radius, side)).abs().psqrt();
        let mut reach = ps2float::sub(ps2float::add(ps2float::mul(ox, dx), ps2float::mul(oz, dz)), half);
        if lwc1(0x3ac808) < reach && reach < 0.0 {
            reach = 0.0;
        }
        if !(reach < 0.0) {
            if out == 0 {
                return reach;
            }
            let y = ps2float::add(-ps2float::mul(half, dz), ps2float::mul(side, -dx));
            let xx = ps2float::add(-ps2float::mul(half, dx), ps2float::mul(side, dz));
            swc1(out + 4, y);
            swc1(out, xx);
            let inv = (1.0_f32).pdiv(ps2float::add(ps2float::mul(xx, xx), ps2float::mul(y, y)).abs().psqrt());
            let ny = ps2float::mul(lwc1(out + 4), inv);
            swc1(out, ps2float::mul(lwc1(out), inv));
            swc1(out + 4, ny);
            return reach;
        }
        if out != 0 {
            swc1(out, -dx);
            swc1(out + 4, -dz);
        }
        let behind = ps2float::mul(reach, reach);
        if ps2float::add(behind, ps2float::mul(side, side)) < ps2float::sub(ps2float::mul(radius, radius), lwc1(0x3ac80c)) {
            return 0.0;
        }
    }
    lwc1(0x3ae440)
}

/// Move a circle (x, z, radius) in the direction (dx, dz) until it touches the circle (px, pz, radius2): the distance as a fraction of the length of the
/// direction, from 0 to 1 (@ 0x2094a0).
pub fn hittestMoveCircleToCircle(out: i32, x: f32, z: f32, radius: f32, dx: f32, dz: f32, px: f32, pz: f32, radius2: f32) -> f32 {
    let combined = ps2float::add(radius, radius2);
    let inv = (1.0_f32).pdiv(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dz, dz)).abs().psqrt());
    let reach = hittestMoveCircleToPoint(out, x, z, combined, ps2float::mul(dx, inv), ps2float::mul(dz, inv), px, pz);
    if !(1.0 <= reach) {
        if !(reach <= 0.0) { ps2float::mul(reach, inv) } else { 0.0 }
    } else {
        1.0
    }
}

/// Whether the ray from `origin` in the direction `dir` (a unit vector) hits the sphere at `center` (radius squared in `radius2`): 1 or 0; the first hit point is
/// written to `hit` and the vector from the center to it to `normal`, if they are not 0 (@ 0x20a8d8).
pub fn HitTestLineSphere(origin: i32, dir: i32, center: i32, hit: i32, normal: i32, radius2: f32) -> i32 {
    let (ox, oy, oz) = (lwc1(origin), lwc1(origin + 4), lwc1(origin + 8));
    let vx = ps2float::sub(ox, lwc1(center));
    let vy = ps2float::sub(oy, lwc1(center + 4));
    let vz = ps2float::sub(oz, lwc1(center + 8));
    let (dx, dy, dz) = (lwc1(dir), lwc1(dir + 4), lwc1(dir + 8));
    let along = ps2float::add(ps2float::add(ps2float::mul(vx, dx), ps2float::mul(vy, dy)), ps2float::mul(vz, dz));
    let discriminant = ps2float::sub(ps2float::mul(along, along), ps2float::sub(ps2float::add(ps2float::add(ps2float::mul(vx, vx), ps2float::mul(vy, vy)), ps2float::mul(vz, vz)), radius2));
    if !(0.0 <= discriminant) {
        return 0;
    }
    if hit == 0 && normal == 0 {
        return 1;
    }
    let t = ps2float::sub(-along, discriminant.abs().psqrt());
    let hz = ps2float::add(oz, ps2float::mul(t, dz));
    let hx = ps2float::add(ox, ps2float::mul(t, dx));
    let hy = ps2float::add(oy, ps2float::mul(t, dy));
    if hit != 0 {
        swc1(hit + 8, hz);
        swc1(hit, hx);
        swc1(hit + 4, hy);
    }
    if normal != 0 {
        swc1(normal, ps2float::sub(hx, lwc1(center)));
        swc1(normal + 4, ps2float::sub(hy, lwc1(center + 4)));
        swc1(normal + 8, ps2float::sub(hz, lwc1(center + 8)));
    }
    1
}

/// The squared distance of the point `p` from the line segment `a` to `b`; the nearest point of the segment goes to `nearest` if it is not 0 (@ 0x20add0).
pub fn HitTestDistancePointToLineSegmentSquared(p: i32, a: i32, b: i32, nearest: i32) -> f32 {
    let ax = lwc1(a);
    let ay = lwc1(a + 4);
    let vx = ps2float::sub(lwc1(p), ax);
    let vy = ps2float::sub(lwc1(p + 4), ay);
    let dx = ps2float::sub(lwc1(b), ax);
    let az = lwc1(a + 8);
    let dy = ps2float::sub(lwc1(b + 4), ay);
    let vz = ps2float::sub(lwc1(p + 8), az);
    let dz = ps2float::sub(lwc1(b + 8), az);
    let along = ps2float::add(ps2float::add(ps2float::mul(vx, dx), ps2float::mul(vy, dy)), ps2float::mul(vz, dz));
    let (mut rx, mut ry, mut rz) = (vx, vy, vz);
    let t;
    if !(along <= 0.0) {
        let length2 = ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz));
        if !(length2 <= along) {
            t = (along).pdiv(length2);
            rx = ps2float::sub(vx, ps2float::mul(t, dx));
            ry = ps2float::sub(vy, ps2float::mul(t, dy));
            rz = ps2float::sub(vz, ps2float::mul(t, dz));
        } else {
            rx = ps2float::sub(vx, dx);
            t = 1.0;
            ry = ps2float::sub(vy, dy);
            rz = ps2float::sub(vz, dz);
        }
    } else {
        t = 0.0;
    }
    if nearest != 0 {
        swc1(nearest, ps2float::add(ax, ps2float::mul(t, dx)));
        swc1(nearest + 4, ps2float::add(lwc1(a + 4), ps2float::mul(t, dy)));
        swc1(nearest + 8, ps2float::add(lwc1(a + 8), ps2float::mul(t, dz)));
    }
    ps2float::add(ps2float::add(ps2float::mul(rx, rx), ps2float::mul(ry, ry)), ps2float::mul(rz, rz))
}
