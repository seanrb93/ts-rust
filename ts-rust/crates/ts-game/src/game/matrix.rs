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
