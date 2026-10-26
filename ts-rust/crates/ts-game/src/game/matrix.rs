//! Matrices and quaternions (the parts of the game's maths that are plain scalar code).

use super::types::*;
use crate::*;


/// Translation matrix: identity with the offset in the last row (@ 0x2b4e40).
pub fn matrixTrans(m: i32, x: f32, y: f32, z: f32) {
    let m = Mat4(m);
    m.set_identity();
    m.set_translation(x, y, z);
}

/// Set the translation of a matrix and leave the rest (@ 0x2b4e98).
pub fn matrixSetTrans(m: i32, x: f32, y: f32, z: f32) {
    Mat4(m).set_translation(x, y, z);
}

/// Uniform scale matrix (@ 0x2b4ea8).
pub fn matrixScale(m: i32, s: f32) {
    let m = Mat4(m);
    m.set_identity();
    m.set_diagonal(s, s, s);
}

/// Scale matrix with a factor for each axis (@ 0x2b4f20).
pub fn matrixScaleSet(m: i32, x: f32, y: f32, z: f32) {
    let m = Mat4(m);
    m.set_identity();
    m.set_diagonal(x, y, z);
}

/// `dst` = the inverse rotation of the unit quaternion `src` (the vector part negated) (@ 0x2b4780).
///
/// Like the original, each component is read from `src` just before it is written to `dst`, so overlapping quaternions give
/// what the console gave.
pub fn quaternionInvert(dst: i32, src: i32) {
    let (d, s) = (Quat(dst), Quat(src));
    d.set_x(-s.x());
    d.set_y(-s.y());
    d.set_z(-s.z());
    d.set_w(s.w());
}

/// `dst` = `a` * `b`, the Hamilton product (@ 0x2b4678).
///
/// The components are computed and stored one after another, reading `a` and `b` afresh each time (as the original does), so
/// `dst` must not be one of the operands.
pub fn quaternionMul(dst: i32, a: i32, b: i32) {
    let (d, a, b) = (Quat(dst), Quat(a), Quat(b));
    d.set_x(ps2float::sub(ps2float::add(ps2float::add(ps2float::mul(a.w(), b.x()), ps2float::mul(a.x(), b.w())), ps2float::mul(a.y(), b.z())), ps2float::mul(a.z(), b.y())));
    d.set_y(ps2float::sub(ps2float::add(ps2float::add(ps2float::mul(a.w(), b.y()), ps2float::mul(a.y(), b.w())), ps2float::mul(a.z(), b.x())), ps2float::mul(a.x(), b.z())));
    d.set_z(ps2float::sub(ps2float::add(ps2float::add(ps2float::mul(a.w(), b.z()), ps2float::mul(a.z(), b.w())), ps2float::mul(a.x(), b.y())), ps2float::mul(a.y(), b.x())));
    d.set_w(ps2float::sub(ps2float::sub(ps2float::sub(ps2float::mul(a.w(), b.w()), ps2float::mul(a.x(), b.x())), ps2float::mul(a.y(), b.y())), ps2float::mul(a.z(), b.z())));
}

/// Add the translation of matrix `m` to the vector `v` of three floats (@ 0x2b5660).
pub fn matrixVecTrans(m: i32, v: i32) {
    for i in 0..3 {
        swc1(v + 4 * i, ps2float::add(lwc1(v + 4 * i), lwc1(m + 48 + 4 * i)));
    }
}

/// `out` = `a` * (1 - t) + `b` * t, element by element (@ 0x2b5d80).  The elements go column by column.
pub fn matrixLerp(a: i32, b: i32, out: i32, t: f32) {
    let s = ps2float::sub(1.0_f32, t);
    for col in 0..4 {
        for row in 0..4 {
            let at = row * 16 + col * 4;
            swc1(out + at, ps2float::add(ps2float::mul(s, lwc1(a + at)), ps2float::mul(t, lwc1(b + at))));
        }
    }
}

/// An orthographic projection matrix (@ 0x2b5368).  Right-handed like OpenGL, with `near` and `far` giving z from -1 to 1 the other way round.
pub fn matrixOrtho(m: i32, left: f32, bottom: f32, right: f32, top: f32, near: f32, far: f32) {
    let m = Mat4(m);
    m.set(3, 3, 1.0);
    for (r, c) in [(0, 1), (0, 2), (0, 3), (1, 0), (1, 2), (1, 3), (2, 0), (2, 1), (2, 3)] {
        m.set(r, c, 0.0);
    }
    let depth = ps2float::sub(near, far);
    m.set(0, 0, (2.0_f32).pdiv(ps2float::sub(right, left)));
    m.set(1, 1, (2.0_f32).pdiv(ps2float::sub(top, bottom)));
    m.set(3, 2, (ps2float::add(far, near)).pdiv(depth));
    m.set(3, 0, (ps2float::add(right, left)).pdiv(ps2float::sub(left, right)));
    m.set(3, 1, (ps2float::add(top, bottom)).pdiv(ps2float::sub(bottom, top)));
    m.set(2, 2, (2.0_f32).pdiv(depth));
}

/// Scale the x axis of a matrix: the first column (@ 0x2b4ee8).
pub fn matrixDoScaleX(m: i32, s: f32) {
    let (y, z, t) = (ps2float::mul(lwc1(m + 16), s), ps2float::mul(lwc1(m + 32), s), ps2float::mul(lwc1(m + 48), s));
    swc1(m, ps2float::mul(lwc1(m), s));
    swc1(m + 16, y);
    swc1(m + 32, z);
    swc1(m + 48, t);
}

/// `out` = `a` * (1 - t) + `b` * t for quaternions, not renormalised (@ 0x2b4608).
pub fn quaternionLerp(a: i32, b: i32, out: i32, t: f32) {
    let s = ps2float::sub(1.0_f32, t);
    for i in 0..4 {
        swc1(out + 4 * i, ps2float::add(ps2float::mul(s, lwc1(a + 4 * i)), ps2float::mul(t, lwc1(b + 4 * i))));
    }
}

/// Scale a quaternion to length 1 (@ 0x2b47b0).
pub fn quaternionNormalise(q: i32) {
    let (x, y, z, w) = (lwc1(q), lwc1(q + 4), lwc1(q + 8), lwc1(q + 12));
    let inv = (1.0_f32).pdiv(ps2float::add(ps2float::add(ps2float::add(ps2float::mul(x, x), ps2float::mul(y, y)), ps2float::mul(z, z)), ps2float::mul(w, w)).abs().psqrt());
    swc1(q + 12, ps2float::mul(w, inv));
    swc1(q, ps2float::mul(x, inv));
    swc1(q + 4, ps2float::mul(y, inv));
    swc1(q + 8, ps2float::mul(z, inv));
}

/// Transform the vector `v` in place by the matrix `m` (three axes and the translation) (@ 0x2b5428).
pub fn matrixVecMul(m: i32, v: i32) {
    let (x, y, z) = (lwc1(v), lwc1(v + 4), lwc1(v + 8));
    let mut out = [0.0_f32; 3];
    for (i, o) in out.iter_mut().enumerate() {
        let i = i as i32;
        *o = ps2float::add(ps2float::add(ps2float::add(ps2float::mul(x, lwc1(m + 4 * i)), ps2float::mul(y, lwc1(m + 16 + 4 * i))), ps2float::mul(z, lwc1(m + 32 + 4 * i))), lwc1(m + 48 + 4 * i));
    }
    for (i, o) in out.iter().enumerate() {
        swc1(v + 4 * i as i32, *o);
    }
}

/// The quaternion (x, y, z, w) of the rotation matrix `m`: from the trace when it is positive, else from the largest diagonal element (the next
/// indices are the game's table of three words at 0x3a9a80) (@ 0x2b56d8).
pub fn matrixToQuaternion(q: i32, m: i32) {
    let at = |row: i32, col: i32| lwc1(m + 16 * row + 4 * col);
    let next = [lw(0x3a9a80), lw(0x3a9a84), lw(0x3a9a88)];
    let trace = ps2float::add(ps2float::add(at(0, 0), at(1, 1)), at(2, 2));
    if !(0.0 < trace) {
        let mut i = 0;
        if at(0, 0) < at(1, 1) {
            i = 1;
        }
        if at(i, i) < at(2, 2) {
            i = 2;
        }
        let j = next[i as usize];
        let k = next[j as usize];
        let s = ps2float::add(ps2float::sub(at(i, i), ps2float::add(at(j, j), at(k, k))), 1.0).abs().psqrt();
        let scale = (0.5_f32).pdiv(s);
        let mut out = [0.0_f32; 3];
        out[i as usize] = ps2float::mul(s, 0.5);
        out[j as usize] = ps2float::mul(ps2float::add(at(i, j), at(j, i)), scale);
        out[k as usize] = ps2float::mul(ps2float::add(at(i, k), at(k, i)), scale);
        swc1(q, out[0]);
        swc1(q + 4, out[1]);
        swc1(q + 8, out[2]);
        swc1(q + 12, ps2float::mul(ps2float::sub(at(j, k), at(k, j)), scale));
    } else {
        let root = ps2float::add(trace, 1.0).abs().psqrt();
        let scale = (0.5_f32).pdiv(root);
        swc1(q + 12, ps2float::mul(ps2float::add(trace, 1.0).abs().psqrt(), 0.5));
        swc1(q, ps2float::mul(ps2float::sub(at(1, 2), at(2, 1)), scale));
        swc1(q + 4, ps2float::mul(ps2float::sub(at(2, 0), at(0, 2)), scale));
        swc1(q + 8, ps2float::mul(ps2float::sub(at(0, 1), at(1, 0)), scale));
    }
}
