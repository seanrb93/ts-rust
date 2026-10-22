//! Setting the pieces of a pane of glass that breaks.

use super::particle_spawners::random_as_float;
use crate::*;

/// The point of the pane with the texture coordinates `(u, v)`: the pane is the four corners `a`, `b`, `c` and `d` (each has the position at +0 and the texture coordinates at +12); the two sides `side_1` and `side_2` are added to the points of the shard.
fn pane_point(a: i32, b: i32, c: i32, d: i32, side_1: i32, side_2: i32, u: f32, v: f32) -> [f32; 3] {
    let mut point = [0.0_f32; 3];
    for axis in 0..3 {
        let offset = axis * 4;
        let s1 = lwc1(side_1 + offset);
        let s2 = lwc1(side_2 + offset);
        let across_c = (lwc1(c + offset) + s1) - lwc1(d + offset);
        let across_b = (lwc1(b + offset) + s2) - lwc1(d + offset);
        point[axis as usize] = lwc1(a + offset) + s1 * u + s2 * v - (across_c * v + across_b * u);
    }
    point
}

/// The texture coordinates of the shard's corner: by an affine map of the pane's (when the pane's mode, the word at +0x1184 of the glass record, is 0) or interpolated between its four corners.
fn corner_texture(a: i32, b: i32, c: i32, d: i32, affine: bool, uv: i32, at: i32) {
    let u = lwc1(uv);
    let v = lwc1(uv + 4);
    if affine {
        let base_u = lwc1(a + 12);
        let base_v = lwc1(a + 16);
        let b_u = lwc1(b + 12) - base_u;
        let c_u = lwc1(c + 12) - base_u;
        let b_v = lwc1(b + 16) - base_v;
        let c_v = lwc1(c + 16) - base_v;
        swc1(at, base_u + b_u * u + c_u * v);
        swc1(at + 4, lwc1(a + 16) + b_v * u + c_v * v);
        swc1(at + 8, 1.0_f32);
        swc1(at + 12, 1.0_f32);
    } else {
        for component in 0..3 {
            let offset = 12 + component * 4;
            let one_minus_u = 1.0_f32 - u;
            let one_minus_v = 1.0_f32 - v;
            let value = (lwc1(a + offset) * one_minus_u + lwc1(b + offset) * u) * one_minus_v
                + (lwc1(c + offset) * one_minus_u + lwc1(d + offset) * u) * v;
            swc1(at + component * 4, value);
        }
        swc1(at + 12, 1.0_f32);
    }
}

/// Make one shard of a breaking pane of glass: from the pane's corners (`a5`, `a6`, `a7` and, on the caller's stack from 688 bytes above this function's frame in slots of 8: the fourth corner, the two sides, the texture coordinates of the shard's four corners, the point of impact, the direction of the blow, the offset of the texture, and two flags) the matrix of the shard (by `matrixTrans`, into `a0` + 64 bytes a shard) and its four vertices with their texture coordinates (into `a1` and `a2`), and the shard's speed, spin and fade (into the arrays of the glass record `a3`), with a random scatter scaled by the distance from the point of impact; `a4` is the number of the shard and `f12` the force.
pub fn glassSetQuad(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32, a6: i32, a7: i32, f12: f32) {
    let frame = Frame::new(688);
    let sp = frame.sp();
    let args = sp + 688;
    let corner_d = lw(args);
    let side_1 = lw(args + 8);
    let side_2 = lw(args + 16);
    let uv_0 = lw(args + 24);
    let uv_1 = lw(args + 32);
    let uv_2 = lw(args + 40);
    let uv_3 = lw(args + 48);
    let impact = lw(args + 56);
    let direction = lw(args + 64);
    let texture_offset = lw(args + 72);
    let first_flag = lw(args + 80);
    let flags = lw(args + 88);
    let shard = a4 << 6;

    // The middle of the shard in the texture coordinates of the pane, and its corners moved by the offset of the texture.
    let u0 = lwc1(uv_0);
    let v0 = lwc1(uv_0 + 4);
    let u1 = lwc1(uv_1);
    let v1 = lwc1(uv_1 + 4);
    let u2 = lwc1(uv_2);
    let v2 = lwc1(uv_2 + 4);
    let centre_u = (u0 + u1 + u2) / 3.0_f32;
    let centre_v = (v0 + v1 + v2) / 3.0_f32;
    let scale = lwc1(0x3ad4c0);
    let offset_u = lwc1(texture_offset) * scale;
    let offset_v = lwc1(texture_offset + 4) * scale;
    let corners = [
        (u0 + offset_u, v0 + offset_v),
        (u1 + offset_u, v1 + offset_v),
        (u2 + offset_u, v2 + offset_v),
        (lwc1(uv_3) + offset_u, lwc1(uv_3 + 4) + offset_v),
    ];

    // The matrix of the shard moves it to the middle, and its vertices are round it.
    let middle = pane_point(a5, a6, a7, corner_d, side_1, side_2, centre_u, centre_v);
    matrixTrans(a0 + shard, middle[0], middle[1], middle[2]);
    let vertices = a1 + shard;
    for (index, (u, v)) in corners.iter().enumerate() {
        let point = pane_point(a5, a6, a7, corner_d, side_1, side_2, *u, *v);
        let at = vertices + index as i32 * 16;
        swc1(at, point[0] - middle[0]);
        swc1(at + 4, point[1] - middle[1]);
        swc1(at + 8, point[2] - middle[2]);
        swc1(at + 12, 1.0_f32);
    }
    let affine = lw(a3 + 0x1184) == 0;
    let texture = a2 + shard;
    let uvs = [uv_0, uv_1, uv_2, uv_3];
    for (index, uv) in uvs.iter().enumerate() {
        corner_texture(a5, a6, a7, corner_d, affine, *uv, texture + index as i32 * 16);
    }

    // The speed and the spin: they are bigger the nearer the shard is to the point of impact.
    let from_impact_u = centre_u - lwc1(impact);
    let from_impact_v = centre_v - lwc1(impact + 4);
    let distance = (from_impact_u * from_impact_u + from_impact_v * from_impact_v).abs().sqrt() / 2.0_f32.abs().sqrt();
    let near = 1.0_f32 - distance;
    let power = near * near * near;
    let push = power * 14.0_f32 * f12;
    let velocities = a3 + 1280;
    let rotations = a3 + 2240;
    let spins = a3 + 3200;
    let fades = a3 + 0x1040 + (a4 << 2);
    let shard_slot = a4 * 12;
    let random_x = random_as_float();
    swc1(
        velocities + shard_slot,
        lwc1(direction) * push + (random_x * 2.3283064e-10_f32 - 0.5_f32) * lwc1(0x3ad4c4) * distance,
    );
    let random_y = random_as_float();
    swc1(
        velocities + 4 + shard_slot,
        lwc1(direction + 4) * push + (random_y * 2.3283064e-10_f32 - 0.5_f32) * lwc1(0x3ad4c8) * distance,
    );
    let random_z = random_as_float();
    let scatter_z = (random_z * 2.3283064e-10_f32 - 0.5_f32) * lwc1(0x3ad4cc);
    let fade = distance * lwc1(0x3ad4d0);
    swc1(velocities + 8 + shard_slot, lwc1(direction + 8) * push + scatter_z * distance);
    swc1(rotations + shard_slot, 0.0_f32);
    swc1(rotations + 4 + shard_slot, 0.0_f32);
    swc1(rotations + 8 + shard_slot, 0.0_f32);
    swc1(fades, -fade);
    let mut around_u = 1.0_f32 - from_impact_u;
    if from_impact_u < 0.0_f32 {
        around_u = -1.0_f32 - from_impact_u;
    }
    let mut around_v = 1.0_f32 - from_impact_v;
    if from_impact_v < 0.0_f32 {
        around_v = -1.0_f32 - from_impact_v;
    }
    let twist = f12 * 18.0_f32 * power;
    let twist_v = around_v * twist;
    let twist_u = around_u * twist;
    swc1(spins + shard_slot, twist_v * lwc1(direction + 4) + twist_v * lwc1(direction + 8));
    swc1(spins + 4 + shard_slot, twist_u * lwc1(direction) + twist_u * lwc1(direction + 8));
    swc1(spins + 8 + shard_slot, twist_v * lwc1(direction) + twist_u * lwc1(direction + 4));
    if first_flag != 0 && flags & 512 == 0 && newrnd() & 7 == 0 {
        // Some shards just drop.
        let random = random_as_float();
        swc1(fades, -(random * 2.3283064e-10_f32 * lwc1(0x3ad4d4) + 0.5_f32));
        for axis in 0..3 {
            sw(velocities + axis * 4 + shard_slot, 0);
            sw(rotations + axis * 4 + shard_slot, 0);
            sw(spins + axis * 4 + shard_slot, 0);
        }
    }
}
