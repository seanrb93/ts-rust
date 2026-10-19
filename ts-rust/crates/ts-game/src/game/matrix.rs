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
    d.set_x(a.w() * b.x() + a.x() * b.w() + a.y() * b.z() - a.z() * b.y());
    d.set_y(a.w() * b.y() + a.y() * b.w() + a.z() * b.x() - a.x() * b.z());
    d.set_z(a.w() * b.z() + a.z() * b.w() + a.x() * b.y() - a.y() * b.x());
    d.set_w(a.w() * b.w() - a.x() * b.x() - a.y() * b.y() - a.z() * b.z());
}

/// Add the translation of matrix `m` to the vector `v` of three floats (@ 0x2b5660).
pub fn matrixVecTrans(m: i32, v: i32) {
    for i in 0..3 {
        swc1(v + 4 * i, lwc1(v + 4 * i) + lwc1(m + 48 + 4 * i));
    }
}

/// `out` = `a` * (1 - t) + `b` * t, element by element (@ 0x2b5d80).  The elements go column by column.
pub fn matrixLerp(a: i32, b: i32, out: i32, t: f32) {
    let s = 1.0_f32 - t;
    for col in 0..4 {
        for row in 0..4 {
            let at = row * 16 + col * 4;
            swc1(out + at, s * lwc1(a + at) + t * lwc1(b + at));
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
    let depth = near - far;
    m.set(0, 0, 2.0 / (right - left));
    m.set(1, 1, 2.0 / (top - bottom));
    m.set(3, 2, (far + near) / depth);
    m.set(3, 0, (right + left) / (left - right));
    m.set(3, 1, (top + bottom) / (bottom - top));
    m.set(2, 2, 2.0 / depth);
}

/// Scale the x axis of a matrix: the first column (@ 0x2b4ee8).
pub fn matrixDoScaleX(m: i32, s: f32) {
    let (y, z, t) = (lwc1(m + 16) * s, lwc1(m + 32) * s, lwc1(m + 48) * s);
    swc1(m, lwc1(m) * s);
    swc1(m + 16, y);
    swc1(m + 32, z);
    swc1(m + 48, t);
}

/// `out` = `a` * (1 - t) + `b` * t for quaternions, not renormalised (@ 0x2b4608).
pub fn quaternionLerp(a: i32, b: i32, out: i32, t: f32) {
    let s = 1.0_f32 - t;
    for i in 0..4 {
        swc1(out + 4 * i, s * lwc1(a + 4 * i) + t * lwc1(b + 4 * i));
    }
}

/// Scale a quaternion to length 1 (@ 0x2b47b0).
pub fn quaternionNormalise(q: i32) {
    let (x, y, z, w) = (lwc1(q), lwc1(q + 4), lwc1(q + 8), lwc1(q + 12));
    let inv = 1.0_f32 / (x * x + y * y + z * z + w * w).abs().sqrt();
    swc1(q + 12, w * inv);
    swc1(q, x * inv);
    swc1(q + 4, y * inv);
    swc1(q + 8, z * inv);
}

/// Transform the vector `v` in place by the matrix `m` (three axes and the translation) (@ 0x2b5428).
pub fn matrixVecMul(m: i32, v: i32) {
    let (x, y, z) = (lwc1(v), lwc1(v + 4), lwc1(v + 8));
    let mut out = [0.0_f32; 3];
    for (i, o) in out.iter_mut().enumerate() {
        let i = i as i32;
        *o = x * lwc1(m + 4 * i) + y * lwc1(m + 16 + 4 * i) + z * lwc1(m + 32 + 4 * i) + lwc1(m + 48 + 4 * i);
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
    let trace = at(0, 0) + at(1, 1) + at(2, 2);
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
        let s = (at(i, i) - (at(j, j) + at(k, k)) + 1.0).abs().sqrt();
        let scale = 0.5_f32 / s;
        let mut out = [0.0_f32; 3];
        out[i as usize] = s * 0.5;
        out[j as usize] = (at(i, j) + at(j, i)) * scale;
        out[k as usize] = (at(i, k) + at(k, i)) * scale;
        swc1(q, out[0]);
        swc1(q + 4, out[1]);
        swc1(q + 8, out[2]);
        swc1(q + 12, (at(j, k) - at(k, j)) * scale);
    } else {
        let root = (trace + 1.0).abs().sqrt();
        let scale = 0.5_f32 / root;
        swc1(q + 12, (trace + 1.0).abs().sqrt() * 0.5);
        swc1(q, (at(1, 2) - at(2, 1)) * scale);
        swc1(q + 4, (at(2, 0) - at(0, 2)) * scale);
        swc1(q + 8, (at(0, 1) - at(1, 0)) * scale);
    }
}
