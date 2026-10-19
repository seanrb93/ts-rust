//! Moving the vertices of a part of a model, and its parts below it, by a matrix.

use crate::*;

/// Move by the matrix `matrix` all the vertices of the bone `bone` of the skeleton `skeleton` (and those of the bones that follow it at the same level, and, by recursion, of its
/// children, unless the bone has kind 3): for the first list of chunks (`which` 0) the vertices of the chunks, for the second also the vertices of the bone's own morph targets
/// (records of 152 bytes with 5 rows of two vertices) and its extra vertices (@ 0x260258).
pub fn partTransform(skeleton: i32, bone: i32, which: i32, matrix: i32) {
    let mut bone_number = bone;
    loop {
        let this = skeleton - lw(skeleton) * 80 + bone_number * 80;
        let mut chunk = if which == 0 { lw(this + 8) } else { lw(this + 12) };
        let mut arrays = this + 20;
        if which != 0 {
            arrays = this + 44;
            let mut target = 0;
            if lb(this + 6) > 0 {
                loop {
                    let next = target + 1;
                    let mut row = 4;
                    let mut vertex = lw(this + 16) + target * 152 + 8;
                    loop {
                        matrixVecMul(matrix, vertex);
                        row -= 1;
                        matrixVecMul(matrix, vertex + 60);
                        vertex += 12;
                        if row < 0 {
                            break;
                        }
                    }
                    target = next;
                    if target >= lb(this + 6) {
                        break;
                    }
                }
            }
            let mut k = 0;
            if lb(this + 7) > 0 {
                let mut at = 0;
                loop {
                    k += 1;
                    matrixVecMul(matrix, lw(this + 72) + at);
                    at += 12;
                    if k >= lb(this + 7) {
                        break;
                    }
                }
            }
        }
        let child;
        if chunk == 0 || lw(chunk + 20) < 0 {
            child = lb(this + 3);
        } else {
            loop {
                let next = chunk + 24;
                let mut done = false;
                let mut strip = lw(arrays) + (lw(chunk + 4) << 4);
                let header = lw(strip);
                let mut vertex = lw(arrays + 4) + lw(chunk + 12) * 12;
                let mut last = header & 0x8000;
                let mut count = header & 0x7fff;
                loop {
                    for _ in 0..count {
                        matrixVecMul(matrix, vertex);
                        vertex += 12;
                    }
                    if last != 0 {
                        done = true;
                    } else {
                        strip += 16;
                        let header = lw(strip);
                        last = header & 0x8000;
                        count = header & 0x7fff;
                    }
                    if done {
                        break;
                    }
                }
                chunk = next;
                if lw(next + 20) < 0 {
                    break;
                }
            }
            child = lb(this + 3);
        }
        if child >= 0 && lb(this) != 3 {
            partTransform(skeleton, child, which, matrix);
        }
        bone_number = lb(this + 4);
        if bone_number < 0 {
            break;
        }
    }
}
