//! Matrices and quaternions (the parts of the game's maths that are plain scalar code).

use super::types::*;


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
