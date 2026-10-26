//! Matrices from turns about three axes, back to angles, a turn about any axis, the perspective matrix, and the matrices of a skeleton's bones.

use crate::*;

/// Make the matrix `m` that turns by `z` about the z axis, then `x` about the x and `y` about the y (the first argument is the turn about the axis whose sine is at +36) (@ 0x2b4b18).
pub fn matrixRotZXY(m: i32, a: f32, b: f32, c: f32) {
    let sa = sinf(a);
    let ca = cosf(a);
    let sb_ = sinf(b);
    let cb = cosf(b);
    let sc = sinf(c);
    let cc = cosf(c);
    let sa_sb = sa * sb_;
    let sa_cb = sa * cb;
    sw(m + 12, 0);
    swc1(m + 60, 1.0_f32);
    sw(m + 28, 0);
    sw(m + 44, 0);
    sw(m + 48, 0);
    sw(m + 52, 0);
    sw(m + 56, 0);
    swc1(m + 40, ca * cb);
    swc1(m + 32, ca * sb_);
    swc1(m + 4, sc * ca);
    swc1(m + 20, cc * ca);
    swc1(m + 16, -sc * cb + cc * sa_sb);
    swc1(m, cc * cb + sc * sa_sb);
    swc1(m + 8, cc * -sb_ + sc * sa_cb);
    swc1(m + 24, sc * sb_ + cc * sa_cb);
    swc1(m + 36, -sa);
}

/// The same matrix with the place (`x`, `y`, `z`) as well (@ 0x2b4ce0).
pub fn matrixTransRotZXY(m: i32, x: f32, y: f32, z: f32, a: f32, b: f32, c: f32) {
    let sa = sinf(a);
    let ca = cosf(a);
    let sb_ = sinf(b);
    let cb = cosf(b);
    let sc = sinf(c);
    let cc = cosf(c);
    let sa_sb = sa * sb_;
    let sa_cb = sa * cb;
    swc1(m + 48, x);
    swc1(m + 60, 1.0_f32);
    swc1(m + 52, y);
    swc1(m + 56, z);
    sw(m + 12, 0);
    sw(m + 28, 0);
    sw(m + 44, 0);
    swc1(m + 40, ca * cb);
    swc1(m + 32, ca * sb_);
    swc1(m + 4, sc * ca);
    swc1(m + 20, cc * ca);
    swc1(m + 16, -sc * cb + cc * sa_sb);
    swc1(m, cc * cb + sc * sa_sb);
    swc1(m + 8, cc * -sb_ + sc * sa_cb);
    swc1(m + 24, sc * sb_ + cc * sa_cb);
    swc1(m + 36, -sa);
}

/// Take the angles (three floats) out of a matrix, by arcsines (with a double precision square root when the first angle is a right angle) (@ 0x2b5f10).
pub fn matrix2Euler(m: i32, angles: i32) {
    let negated = -lwc1(m + 28);
    let first = asinf(negated);
    let one = 1.0_f32;
    swc1(angles, first);
    if !(negated == one) {
        let second = asinf(-lwc1(m + 56));
        swc1(angles + 4, second);
        sw(angles + 8, 0);
    } else {
        let rest = one - negated * negated;
        fptodp(rest);
        let wide = fptodp(rest);
        let root = sqrt(wide);
        let inverse = dpdiv(0x3ff0000000000000_i64, root);
        let inv = dptofp(inverse);
        swc1(angles + 4, asinf(lwc1(m + 44) * inv));
        swc1(angles + 8, asinf(lwc1(m + 24) * inv));
    }
}

/// Make the matrix `m` that turns by `degrees` about the axis `axis` (a unit vector of three floats) (@ 0x2b5c28).
pub fn matrixroll(m: i32, axis: i32, degrees: f32) {
    let angle = (degrees * lwc1(0x3ad7bc)).pdiv(180.0_f32);
    let s = sinf(angle);
    let one = 1.0_f32;
    let c = cosf(angle);
    let ay = lwc1(axis + 4);
    let one_less_c = one - c;
    let az = lwc1(axis + 8);
    let ax = lwc1(axis);
    sceVu0UnitMatrix(m);
    let t = one_less_c * ax;
    swc1(m + 60, one);
    let t_az = t * az;
    let t_ay_az = one_less_c * ay * az;
    let s_ax = s * ax;
    let s_az = s * az;
    let s_ay = s * ay;
    let t_ay = t * ay;
    swc1(m + 40, one_less_c * (az * az) + c);
    swc1(m + 36, t_ay_az + s_ax);
    swc1(m + 16, t_ay + s_az);
    swc1(m + 32, t_az - s_ay);
    swc1(m, one_less_c * (ax * ax) + c);
    swc1(m + 20, one_less_c * (ay * ay) + c);
    swc1(m + 4, t_ay - s_az);
    swc1(m + 8, t_az + s_ay);
    swc1(m + 24, t_ay_az - s_ax);
    swc1(m + 48, 0.0_f32);
    swc1(m + 44, 0.0_f32);
    swc1(m + 28, 0.0_f32);
    swc1(m + 12, 0.0_f32);
    swc1(m + 56, 0.0_f32);
    swc1(m + 52, 0.0_f32);
}

/// Make the perspective matrix `m` for the field of view `fov`, the aspect ratio `aspect` (the first float, divided into the x scale), and the near and far planes (@ 0x2b5258).
pub fn matrixPerspective(m: i32, aspect: f32, fov: f32, near: f32, far: f32) {
    let half = fov * 0.5_f32;
    let depth = far - near;
    let c = cosf(half);
    let s = sinf(half);
    let near_far = near * far;
    let scale = (c).pdiv(s);
    sw(m + 60, 0);
    swc1(m + 44, -1.0_f32);
    sw(m + 4, 0);
    sw(m + 8, 0);
    sw(m + 12, 0);
    sw(m + 16, 0);
    swc1(m + 20, scale);
    sw(m + 24, 0);
    sw(m + 28, 0);
    sw(m + 32, 0);
    sw(m + 36, 0);
    sw(m + 48, 0);
    swc1(m, (scale).pdiv(aspect));
    swc1(m + 40, (-(near + far)).pdiv(depth));
    swc1(m + 56, (-(near_far + near_far)).pdiv(depth));
    sw(m + 52, 0);
}

/// Make the quaternion `q` (x, y, z, w) of the turns about three axes (three floats at `angles`) (@ 0x2b3e88).
pub fn rot2fquat(angles: i32, q: i32) {
    let half = 0.5_f32;
    let a = lwc1(angles) * half;
    let b = lwc1(angles + 4) * half;
    let c = lwc1(angles + 8) * half;
    let sa = sinf(a);
    let ca = cosf(a);
    let sb_ = sinf(b);
    let cb = cosf(b);
    let sc = sinf(c);
    let cc = cosf(c);
    let sa_sb = sa * sb_;
    let ca_sb = ca * sb_;
    let sa_cb = sa * cb;
    let ca_cb = ca * cb;
    swc1(q + 8, ca_sb * cc + sa_cb * sc);
    swc1(q, ca_cb * cc + sa_sb * sc);
    swc1(q + 4, sa_cb * cc - ca_sb * sc);
    swc1(q + 12, ca_cb * sc - sa_sb * cc);
}

/// Copy the matrices of the bones of the skeleton `skeleton` (from `source`, one for each bone in turn, the count kept in the word `count`) into the matrix table of the model
/// (+4 of `skeleton`), and note the bones of kind 0 by number in the list at +92; kind 1 bones are skipped and kind 3 ones are not copied (@ 0x262f80).
pub fn matrixAlign(skeleton: i32, source: i32, bone: i32, count: i32) {
    let mut bone = bone;
    loop {
        let top = lw(skeleton);
        let this = top - lw(top) * 80 + bone * 80;
        let kind = lb(this);
        if kind != 1 && kind != 3 {
            let target = lw(skeleton + 4) + (lb(this + 1) << 6);
            let from = source + (lw(count) << 6);
            sceVu0CopyMatrix(target, from);
            if lb(this) == 0 {
                sw((lw(count) << 2) + lw(lw(skeleton + 92)), lb(this + 1));
                sw(count, lw(count) + 1);
            }
        }
        let child = lb(this + 3);
        if child >= 0 && lb(this) != 3 {
            matrixAlign(skeleton, source, child, count);
        }
        bone = lb(this + 4);
        if bone < 0 {
            break;
        }
    }
}
