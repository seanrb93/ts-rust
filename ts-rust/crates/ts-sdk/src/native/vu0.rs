//! `sceVu0*` vector / matrix helpers (Sony libvu0).  On the PS2 these ran on VU0 in macro mode; here they are plain
//! f32 code that reads and writes the game's 16-byte aligned vectors / 4x4 row-major matrices.
//! Accumulation order matches the VU0 multiply-accumulate sequences (no fused multiply-add).
#![allow(non_snake_case)]

use ts_rt::*;

type V4 = [f32; 4];
type M4 = [V4; 4];

fn rv(a: i32) -> V4 {
    [lwc1(a), lwc1(a + 4), lwc1(a + 8), lwc1(a + 12)]
}
fn wv(a: i32, v: V4) {
    for i in 0..4 {
        swc1(a + 4 * i as i32, v[i]);
    }
}
fn rm(a: i32) -> M4 {
    [rv(a), rv(a + 16), rv(a + 32), rv(a + 48)]
}
fn wm(a: i32, m: M4) {
    for i in 0..4 {
        wv(a + 16 * i as i32, m[i]);
    }
}

/// `v * m` for a row vector: x*r0 + y*r1 + z*r2 + w*r3 accumulated in VU0 order.
fn row_times(v: V4, m: &M4) -> V4 {
    let mut o = [0.0f32; 4];
    for j in 0..4 {
        o[j] = v[0] * m[0][j];
        o[j] = o[j] + v[1] * m[1][j];
        o[j] = o[j] + v[2] * m[2][j];
        o[j] = o[j] + v[3] * m[3][j];
    }
    o
}

/// `dst = m2 * m1` (each row of m2 is transformed by m1) -- the argument order of the Sony routine.
pub fn sceVu0MulMatrix(a0: i32, a1: i32, a2: i32) -> i32 {
    let m1 = rm(a1);
    let m2 = rm(a2);
    let out = [row_times(m2[0], &m1), row_times(m2[1], &m1), row_times(m2[2], &m1), row_times(m2[3], &m1)];
    wm(a0, out);
    0
}

pub fn sceVu0CopyMatrix(a0: i32, a1: i32) -> i32 {
    let m = rm(a1);
    wm(a0, m);
    0
}

pub fn sceVu0UnitMatrix(a0: i32) -> i32 {
    wm(a0, [[1.0, 0.0, 0.0, 0.0], [0.0, 1.0, 0.0, 0.0], [0.0, 0.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]]);
    0
}

pub fn sceVu0TransposeMatrix(a0: i32, a1: i32) -> i32 {
    let m = rm(a1);
    let mut o = [[0.0f32; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            o[i][j] = m[j][i];
        }
    }
    wm(a0, o);
    0
}

/// Inverse of a rigid transform (orthonormal 3x3 rotation + translation in row 3).
pub fn sceVu0InversMatrix(a0: i32, a1: i32) -> i32 {
    let m = rm(a1);
    let mut o = [[0.0f32; 4]; 4];
    for i in 0..3 {
        for j in 0..3 {
            o[i][j] = m[j][i];
        }
    }
    let t = [m[3][0], m[3][1], m[3][2]];
    for j in 0..3 {
        let mut acc = t[0] * o[0][j];
        acc = acc + t[1] * o[1][j];
        acc = acc + t[2] * o[2][j];
        o[3][j] = 0.0 - acc;
    }
    o[3][3] = m[3][3];
    wm(a0, o);
    0
}

// ---------------------------------------------------------------------------------------------------------------
// Game-side VU0 macro-mode helpers (originally hand-written inline-assembly leaf functions of the game).
// ---------------------------------------------------------------------------------------------------------------

fn dot4(a: V4, b: V4) -> f32 {
    let x = a[0] * b[0];
    let y = a[1] * b[1];
    let z = a[2] * b[2];
    let w = a[3] * b[3];
    ((x + y) + z) + w
}

/// Shared body of `quaternionSlerp` / `quaternionSlerp2`: spherical interpolation q0 -> q1 by `t`.
/// The original's formulas are reproduced as compiled (including its odd `sinf((1-t)*theta / sinf(theta))` weight).
fn quat_slerp(q0a: i32, q1a: i32, outa: i32, t: f32, t1_copies_q1: bool) {
    let q0 = rv(q0a);
    let q1 = rv(q1a);
    if t == 0.0 {
        wv(outa, q0);
        return;
    }
    if t == 1.0 {
        // quaternionSlerp copies q1; quaternionSlerp2 copied an uninitialised stack slot (clearly unintended)
        let _ = t1_copies_q1;
        wv(outa, q1);
        return;
    }
    let sq = |v: V4| v[0] * v[0] + v[1] * v[1] + v[2] * v[2] + v[3] * v[3];
    let diff = sq([q0[0] - q1[0], q0[1] - q1[1], q0[2] - q1[2], q0[3] - q1[3]]);
    let sum = sq([q0[0] + q1[0], q0[1] + q1[1], q0[2] + q1[2], q0[3] + q1[3]]);
    // take the shorter arc
    let qn: V4 = if sum < diff { [0.0 - q1[0], 0.0 - q1[1], 0.0 - q1[2], 0.0 - q1[3]] } else { q1 };
    let cos = dot4(q0, qn);
    let eps = f32::from_bits(0x3400_0000);
    let (w0, w1): (f32, f32);
    if !(eps < cos + 1.0) {
        // nearly opposite: quarter-turn blend
        let k = std::f32::consts::FRAC_PI_2;
        w0 = crate::native::math::sinf((1.0 - t) * k);
        w1 = crate::native::math::sinf(t * k);
        wv(outa, [w0 * q0[0] - w1 * qn[0], w0 * q0[1] - w1 * qn[1], w0 * q0[2] - w1 * qn[2], w0 * q0[3] - w1 * qn[3]]);
        return;
    }
    if !(eps < 1.0 - cos) {
        // nearly identical: linear blend
        w0 = 1.0 - t;
        w1 = t;
    } else {
        let theta = crate::native::math::acosf(cos);
        let s = crate::native::math::sinf(theta);
        w0 = crate::native::math::sinf(((1.0 - t) * theta) / s);
        w1 = crate::native::math::sinf(t * theta) / s;
    }
    wv(outa, [w0 * q0[0] + w1 * qn[0], w0 * q0[1] + w1 * qn[1], w0 * q0[2] + w1 * qn[2], w0 * q0[3] + w1 * qn[3]]);
}

/// quaternionSlerp(q0, q1, out, t)
pub fn quaternionSlerp(args: &[i64]) -> i32 {
    #[cfg(feature = "shadow")]
    let _sh = shadow_args(0x2b4080, "quaternionSlerp", args);
    let t = f32::from_bits(args[3] as u32);
    quat_slerp(args[0] as i32, args[1] as i32, args[2] as i32, t, true);
    0
}

pub fn quaternionSlerp2(args: &[i64]) -> i32 {
    #[cfg(feature = "shadow")]
    let _sh = shadow_args(0x2b4340, "quaternionSlerp2", args);
    let t = f32::from_bits(args[3] as u32);
    quat_slerp(args[0] as i32, args[1] as i32, args[2] as i32, t, false);
    0
}

/// Shared by the three matrix * vector routines: ACC = row0*v.x + row1*v.y [+ row2*v.z] ...
fn acc3(m: &M4, v: V4, with_z: bool) -> V4 {
    let mut acc = [0.0f32; 4];
    for j in 0..4 {
        acc[j] = m[0][j] * v[0];
        acc[j] = acc[j] + m[1][j] * v[1];
        if with_z {
            acc[j] = acc[j] + m[2][j] * v[2];
        }
    }
    acc
}

/// v.xyz = (v.x*row0 + v.y*row1 + v.z*row2 + row3).xyz, in place; w untouched.   (matrix = a0, vector = a1)
pub fn matrixVecMulAligned(args: &[i64]) -> i32 {
    #[cfg(feature = "shadow")]
    let _sh = shadow_args(0x2b54c8, "matrixVecMulAligned", args);
    let (ma, va) = (args[0] as i32, args[1] as i32);
    let m = rm(ma);
    let v = rv(va);
    let acc = acc3(&m, v, true);
    let o = [acc[0] + m[3][0] * 1.0, acc[1] + m[3][1] * 1.0, acc[2] + m[3][2] * 1.0, v[3]];
    wv(va, o);
    0
}

/// out = v.x*row0 + v.y*row1 + v.z*row2 + row3 (all four lanes).   (matrix = a0, vector = a1, out = a2)
pub fn matrixVec3Mul4Aligned(args: &[i64]) -> i32 {
    #[cfg(feature = "shadow")]
    let _sh = shadow_args(0x2b5570, "matrixVec3Mul4Aligned", args);
    let (ma, va, oa) = (args[0] as i32, args[1] as i32, args[2] as i32);
    let m = rm(ma);
    let v = rv(va);
    let acc = acc3(&m, v, true);
    wv(oa, [acc[0] + m[3][0] * 1.0, acc[1] + m[3][1] * 1.0, acc[2] + m[3][2] * 1.0, acc[3] + m[3][3] * 1.0]);
    0
}

/// Rotation only: v.xyz = (v.x*row0 + v.y*row1 + v.z*row2).xyz, in place; w untouched.
pub fn matrixVecRotAligned(args: &[i64]) -> i32 {
    #[cfg(feature = "shadow")]
    let _sh = shadow_args(0x2b5638, "matrixVecRotAligned", args);
    let (ma, va) = (args[0] as i32, args[1] as i32);
    let m = rm(ma);
    let v = rv(va);
    let acc = acc3(&m, v, false);
    wv(va, [acc[0] + m[2][0] * v[2], acc[1] + m[2][1] * v[2], acc[2] + m[2][2] * v[2], v[3]]);
    0
}

/// Rotation matrix (a1, 4x4 row-major) -> quaternion (a0), Shoemake's method as compiled in the game.
pub fn matrixToQuaternionNew(args: &[i64]) -> i32 {
    #[cfg(feature = "shadow")]
    let _sh = shadow_args(0x2b5970, "matrixToQuaternionNew", args);
    let (qa, ma) = (args[0] as i32, args[1] as i32);
    let m = rm(ma);
    let sqrt = |x: f32| x.abs().sqrt();
    let mut q = [0.0f32; 4];
    let tr = (m[0][0] + m[1][1]) + m[2][2];
    if tr > 0.0 {
        let s = sqrt(tr + 1.0);
        q[3] = s * 0.5;
        let s = 0.5 / s;
        q[0] = (m[1][2] - m[2][1]) * s;
        q[1] = (m[2][0] - m[0][2]) * s;
        q[2] = (m[0][1] - m[1][0]) * s;
    } else {
        let nxt = [lw(0x3a9a80) as usize, lw(0x3a9a84) as usize, lw(0x3a9a88) as usize];
        let mut i = if m[0][0] < m[1][1] { 1 } else { 0 };
        if m[i][i] < m[2][2] {
            i = 2;
        }
        let j = nxt[i] % 3;
        let k = nxt[j] % 3;
        let s = sqrt((m[i][i] - (m[j][j] + m[k][k])) + 1.0);
        q[i] = s * 0.5;
        let s = 0.5 / s;
        q[3] = (m[j][k] - m[k][j]) * s;
        q[j] = (m[i][j] + m[j][i]) * s;
        q[k] = (m[i][k] + m[k][i]) * s;
    }
    wv(qa, q);
    0
}

#[cfg(feature = "shadow")]
fn shadow_args(addr: u32, name: &'static str, args: &[i64]) -> ts_rt::shadow::Guard {
    let mut ints = [0i64; 8];
    let mut floats = [0f32; 8];
    for (i, a) in args.iter().take(4).enumerate() {
        ints[i] = *a;
    }
    // the slerp routines take their blend factor in f12 (passed here as raw bits in args[3])
    if name.starts_with("quaternionSlerp") {
        floats[0] = f32::from_bits(args[3] as u32);
    }
    ts_rt::shadow::enter(addr, name, ints, floats)
}
