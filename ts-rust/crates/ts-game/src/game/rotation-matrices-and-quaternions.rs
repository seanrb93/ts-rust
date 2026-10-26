//! Matrices and quaternions for a turn about one axis, and the matrices of the bones of a skeleton.

use crate::*;

/// Make the matrix `m` that turns by `angle` about the x axis (@ 0x2b49f8).
pub fn matrixRotX(m: i32, angle: f32) {
    sceVu0UnitMatrix(m);
    let s = sinf(angle);
    let c = cosf(angle);
    swc1(m + 24, s);
    swc1(m + 40, c);
    swc1(m + 20, c);
    swc1(m + 36, -s);
}

/// Make the matrix `m` that turns by `angle` about the y axis (@ 0x2b4a58).
pub fn matrixRotY(m: i32, angle: f32) {
    sceVu0UnitMatrix(m);
    let s = sinf(angle);
    let c = cosf(angle);
    swc1(m + 32, s);
    swc1(m, c);
    swc1(m + 40, c);
    swc1(m + 8, -s);
}

/// Make the matrix `m` that turns by `angle` about the z axis (@ 0x2b4ab8).
pub fn matrixRotZ(m: i32, angle: f32) {
    sceVu0UnitMatrix(m);
    let s = sinf(angle);
    let c = cosf(angle);
    swc1(m + 4, s);
    swc1(m + 20, c);
    swc1(m, c);
    swc1(m + 16, -s);
}

/// Make the matrix `m` that turns by `angle` about the y axis and moves by (`x`, `y`, `z`) (@ 0x2b4c50).
pub fn matrixTransRotY(m: i32, x: f32, y: f32, z: f32, angle: f32) {
    sceVu0UnitMatrix(m);
    let s = sinf(angle);
    let c = cosf(angle);
    swc1(m + 56, z);
    swc1(m + 48, x);
    swc1(m + 52, y);
    swc1(m + 32, s);
    swc1(m, c);
    swc1(m + 8, -s);
    swc1(m + 40, c);
}

/// Make the quaternion `q` (w first) of a turn by `angle` about the x axis (@ 0x2b3d80).
pub fn rotx2fquat(q: i32, angle: f32) {
    let half = ps2float::mul(angle, 0.5_f32);
    swc1(q, cosf(half));
    swc1(q + 4, sinf(half));
    sw(q + 12, 0);
    sw(q + 8, 0);
}

/// The same about the y axis (@ 0x2b3dd8).
pub fn roty2fquat(q: i32, angle: f32) {
    let half = ps2float::mul(angle, 0.5_f32);
    let c = cosf(half);
    sw(q + 4, 0);
    swc1(q, c);
    swc1(q + 8, sinf(half));
    sw(q + 12, 0);
}

/// The same about the z axis (@ 0x2b3e30).
pub fn rotz2fquat(q: i32, angle: f32) {
    let half = ps2float::mul(angle, 0.5_f32);
    let c = cosf(half);
    sw(q + 4, 0);
    sw(q + 8, 0);
    swc1(q, c);
    swc1(q + 12, sinf(half));
}

/// Make the global matrices (64 bytes each, in `matrices`) of the bone `bone` of the skeleton `skeleton` and of its children and the bones after it: a bone of kind 0 has its own
/// matrix multiplied by its parent's (`parent`, when not negative); a bone's children are done unless it is of kind 3 (@ 0x2149b0).
pub fn calGlobalMatrices(skeleton: i32, matrices: i32, bone: i32, parent: i32) {
    let mut bone = bone;
    let mut parent = parent;
    loop {
        let top = lw(skeleton);
        let bones = top - lw(top) * 80;
        let this = bones + bone * 80;
        let saved_parent = parent;
        if lb(this) == 0 {
            let offset = parent * 80;
            if parent >= 0 {
                let own = matrices + (lb(this + 1) << 6);
                let parents = matrices + (lb(offset + bones + 1) << 6);
                sceVu0MulMatrix(own, parents, own);
            }
            parent = bone;
        }
        let child = lb(this + 3);
        if child >= 0 && lb(this) != 3 {
            calGlobalMatrices(skeleton, matrices, child, parent);
        }
        bone = lb(this + 4);
        if bone < 0 {
            break;
        }
        parent = saved_parent;
    }
}
