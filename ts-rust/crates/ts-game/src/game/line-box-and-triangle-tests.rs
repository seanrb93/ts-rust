//! A line against a box (the point where it enters) and against a triangle.

use crate::*;

/// Whether the line from `start` along `direction` (the whole segment, as 1 unit of the direction) meets the box `min` to `max` (points of three floats): the point where it enters
/// goes to `out` (or only the start, if it begins inside), and the normal of the face it enters by to `normal` (when not 0) (@ 0x20aa80).
pub fn HitTestLineBox(start: i32, direction: i32, min: i32, max: i32, out: i32, normal: i32) -> i32 {
    let frame = Frame::new(64);
    let sp = frame.sp();
    let mut inside = true;
    let out = if out != 0 { out } else { sp + 48 };
    for k in 0..3 {
        let s = lwc1(start + 4 * k);
        let low = lwc1(min + 4 * k);
        if !(s < low) {
            let high = lwc1(max + 4 * k);
            if !(high < s) {
                sw(sp + 4 * k, 2);
            } else {
                swc1(sp + 32 + 4 * k, high);
                inside = false;
                sw(sp + 4 * k, 0);
            }
        } else {
            swc1(sp + 32 + 4 * k, low);
            inside = false;
            sw(sp + 4 * k, 1);
        }
    }
    if inside {
        swc1(out, lwc1(start));
        swc1(out + 4, lwc1(start + 4));
        swc1(out + 8, lwc1(start + 8));
        return 1;
    }
    for k in 0..3 {
        if lw(sp + 4 * k) == 2 {
            swc1(sp + 16 + 4 * k, -1.0_f32);
        } else {
            let d = lwc1(direction + 4 * k);
            if d == 0.0_f32 {
                swc1(sp + 16 + 4 * k, -1.0_f32);
            } else {
                swc1(sp + 16 + 4 * k, (ps2float::sub(lwc1(sp + 32 + 4 * k), lwc1(start + 4 * k))).pdiv(d));
            }
        }
    }
    let mut best = 0;
    let mut best_at = 0;
    for k in 1..3 {
        if lwc1(sp + 16 + best_at) < lwc1(sp + 16 + 4 * k) {
            best = k;
            best_at = k << 2;
        }
    }
    let t = sp + 16 + best_at;
    if lwc1(t) < 0.0_f32 {
        return 0;
    }
    for k in 0..3 {
        if best == k {
            let plane = lwc1(sp + 32 + 4 * k);
            swc1(out + 4 * k, plane);
            if normal != 0 {
                swc1(normal + 8, 0.0_f32);
                swc1(normal + 4, 0.0_f32);
                swc1(normal, 0.0_f32);
                if !(lwc1(sp + 32 + 4 * k) == lwc1(min + 4 * k)) {
                    swc1(normal + 4 * k, 1.0_f32);
                } else {
                    swc1(normal + 4 * k, -1.0_f32);
                }
            }
        } else {
            let v = ps2float::add(lwc1(start + 4 * k), ps2float::mul(lwc1(t), lwc1(direction + 4 * k)));
            swc1(out + 4 * k, v);
            if v < lwc1(min + 4 * k) {
                return 0;
            }
            if lwc1(max + 4 * k) < v {
                return 0;
            }
        }
    }
    1
}

/// Whether the line from `start` along `direction` meets the triangle `a`, `b`, `c` (points of three floats): the plane must be met in front of the start, and the point
/// must be on the inside of the three edges; the point goes to `hit` and the normal of the triangle (not made one unit long) to `normal`, when not 0 (@ 0x209818).
pub fn hittestLineTri(start: i32, direction: i32, a: i32, b: i32, c: i32, hit: i32, normal: i32) -> i32 {
    let ax = lwc1(a);
    let ay = lwc1(a + 4);
    let az = lwc1(a + 8);
    let bx = lwc1(b);
    let by = lwc1(b + 4);
    let bz = lwc1(b + 8);
    let cz = lwc1(c + 8);
    let cx = lwc1(c);
    let cy = lwc1(c + 4);
    let abx = ps2float::sub(bx, ax);
    let aby = ps2float::sub(by, ay);
    let abz = ps2float::sub(bz, az);
    let acz = ps2float::sub(cz, az);
    let acx = ps2float::sub(cx, ax);
    let acy = ps2float::sub(cy, ay);
    let dz = lwc1(direction + 8);
    let dy = lwc1(direction + 4);
    let dx = lwc1(direction);
    let ny = ps2float::sub(ps2float::mul(abz, acx), ps2float::mul(abx, acz));
    let nx = ps2float::sub(ps2float::mul(aby, acz), ps2float::mul(abz, acy));
    let nz = ps2float::sub(ps2float::mul(abx, acy), ps2float::mul(aby, acx));
    let facing = ps2float::add(ps2float::add(ps2float::mul(nx, dx), ps2float::mul(ny, dy)), ps2float::mul(nz, dz));
    if facing == 0.0_f32 {
        return 0;
    }
    let sx = lwc1(start);
    let sy = lwc1(start + 4);
    let sz = lwc1(start + 8);
    let t = (ps2float::sub(ps2float::add(ps2float::add(ps2float::mul(nx, ax), ps2float::mul(ny, ay)), ps2float::mul(nz, az)), ps2float::add(ps2float::add(ps2float::mul(nx, sx), ps2float::mul(ny, sy)), ps2float::mul(nz, sz)))).pdiv(facing);
    if t < 0.0_f32 {
        return 0;
    }
    let hz = ps2float::add(sz, ps2float::mul(t, dz));
    let hx = ps2float::add(sx, ps2float::mul(t, dx));
    let hy = ps2float::add(sy, ps2float::mul(t, dy));
    // the edge from b to c
    let bcx = ps2float::sub(cx, bx);
    let bcz = ps2float::sub(cz, bz);
    let chx = ps2float::sub(cx, hx);
    let chz = ps2float::sub(cz, hz);
    let bcy = ps2float::sub(cy, by);
    let chy = ps2float::sub(cy, hy);
    let bhz = ps2float::sub(bz, hz);
    let bhx = ps2float::sub(bx, hx);
    let bhy = ps2float::sub(by, hy);
    let e1_y = ps2float::sub(ps2float::mul(abz, bhx), ps2float::mul(abx, bhz));
    let e1_z_part = ps2float::sub(ps2float::mul(bcz, chx), ps2float::mul(bcx, chz));
    let e1_x_part = ps2float::sub(ps2float::mul(bcy, chz), ps2float::mul(bcz, chy));
    let e1_x = ps2float::sub(ps2float::mul(aby, bhz), ps2float::mul(abz, bhy));
    let e2_y = ps2float::sub(ps2float::mul(abx, bhy), ps2float::mul(aby, bhx));
    let e2_z = ps2float::sub(ps2float::mul(bcx, chy), ps2float::mul(bcy, chx));
    if ps2float::add(ps2float::add(ps2float::mul(e1_x, e1_x_part), ps2float::mul(e1_y, e1_z_part)), ps2float::mul(e2_y, e2_z)) < 0.0_f32 {
        return 0;
    }
    // the edge from c to a
    let cax = ps2float::sub(ax, cx);
    let caz = ps2float::sub(az, cz);
    let ahx = ps2float::sub(ax, hx);
    let ahz = ps2float::sub(az, hz);
    let ahy = ps2float::sub(ay, hy);
    let cay = ps2float::sub(ay, cy);
    let f_z = ps2float::sub(ps2float::mul(caz, ahx), ps2float::mul(cax, ahz));
    let f_x = ps2float::sub(ps2float::mul(cay, ahz), ps2float::mul(caz, ahy));
    let f_y = ps2float::sub(ps2float::mul(cax, ahy), ps2float::mul(cay, ahx));
    if !(ps2float::add(ps2float::add(ps2float::mul(e1_x, f_x), ps2float::mul(e1_y, f_z)), ps2float::mul(e2_y, f_y)) < 0.0_f32) {
        if hit != 0 {
            swc1(hit + 8, hz);
            swc1(hit, hx);
            swc1(hit + 4, hy);
        }
        if normal != 0 {
            swc1(normal, nx);
            swc1(normal + 8, nz);
            swc1(normal + 4, ny);
        }
        return 1;
    }
    0
}
