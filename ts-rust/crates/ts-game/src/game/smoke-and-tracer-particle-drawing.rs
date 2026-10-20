//! The drawing of the smoke and the tracer particles.

use super::particle_ray_and_pulse_drawing::{packed_corner, put_words};
use crate::*;

/// Draw the smoke particle `particle` (`count` puffs from +24, the age of each is the particle's time at +12 less the puff's number over 10; the puffs' turn angles at +948) with
/// the colour `colour` through the matrix `matrix`: every puff younger than 1.5 and inside the view is drawn as a turned square (opacity rising to 1 at half a second, falling to 0 at
/// 1.5, as the high byte of the colour) as a packet of 176 bytes (@ 0x29dc40).
pub fn particleSmokeGfx(particle: i32, colour: i32, matrix: i32) {
    let frame = Frame::new(304);
    let sp = frame.sp();
    let player = lw(g::currentplayer);
    sw(sp + 96, colour);
    let mut count = lw(particle + 24);
    let zoom = 6e+01_f32 / lwc1(player + 260);
    let mut i = 0;
    if count <= 0 {
        return;
    }
    let half = 0.5_f32;
    let window = g::window;
    let table = g::g_SinTable;
    let mut offset = 0;
    loop {
        let age = lwc1(particle + 12) - i as f32 * half / 5.0_f32;
        if 0.0_f32 <= age {
            let angles = particle + 948;
            if age < 1.5_f32 {
                let mut fade = 1.0_f32;
                let mut spread = lwc1(angles + offset);
                if !(age < half) {
                    if fade < age {
                        let down = 1.5_f32 - age;
                        fade = down + down;
                    }
                } else {
                    fade = age + age;
                }
                if spread < 0.0_f32 {
                    spread = -spread;
                }
                let y = lwc1(particle + 932);
                let lift = spread * age * lwc1(0x3ad52c);
                let z = lwc1(particle + 936);
                swc1(sp, lwc1(particle + 928));
                swc1(sp + 8, z);
                swc1(sp + 4, y + lift);
                matrixVec3Mul4Aligned(&[matrix as i64, sp as i64, (sp + 16) as i64]);
                let w = lwc1(sp + 28);
                let x = lwc1(sp + 16);
                let low = -w;
                if low <= x {
                    let y = lwc1(sp + 20);
                    if x <= w && low <= y {
                        let z = lwc1(sp + 24);
                        if y <= w && low <= z && z <= w {
                            let across = lw(window + 16) as f32;
                            let centre_x = lw(window + 8) as f32;
                            let one = 1.0_f32;
                            let centre_y = lw(window + 28) as f32;
                            let scale = lwc1(0x3ad538);
                            let inverse = one / w;
                            let down = (0 - lw(window + 36)) as f32;
                            let screen_x = x * inverse;
                            let turn = lwc1(angles + offset) * age * lwc1(0x3ad530) / 1.8e+02_f32 * lwc1(0x3ad534) + half;
                            let depth = z * inverse;
                            let screen_y = y * inverse;
                            swc1(sp + 16, screen_x);
                            let index = turn as i32;
                            let size = (zoom * 1.5e+02_f32 / z * (age + one)) as i32 as f32;
                            swc1(sp + 24, depth);
                            swc1(sp + 20, screen_y);
                            let px = centre_x + across * screen_x;
                            let dy = size * lwc1((((index + 512) & 2047) << 2) + table);
                            let dx = size * lwc1(((index & 2047) << 2) + table);
                            swc1(sp, px);
                            let py = centre_y + down * screen_y;
                            let half_dy = dy * half;
                            let half_dx = dx * half;
                            swc1(sp + 4, py);
                            swc1(sp + 80, px - dy);
                            swc1(sp + 32, px + dy);
                            swc1(sp + 84, py - half_dx);
                            let z_out = depth * scale + 32768.0_f32 + 1e+02_f32;
                            swc1(sp + 64, px - dx);
                            swc1(sp + 68, py + half_dy);
                            swc1(sp + 36, py + half_dx);
                            swc1(sp + 88, z_out);
                            swc1(sp + 48, px + dx);
                            swc1(sp + 52, py - half_dy);
                            let alpha = (fade * 4e+01_f32) as i32 & 255;
                            swc1(sp + 8, z_out);
                            swc1(sp + 40, z_out);
                            swc1(sp + 56, z_out);
                            swc1(sp + 72, z_out);
                            let packet = memdbAlloc(176);
                            sb(lw(g::gfxtag) + 3, 48);
                            sw(packet, 0x6c0a8000);
                            let tag = lw(g::gfxtag);
                            sw(tag + 4, packet);
                            let words: [i32; 36] = [
                                0x8001,
                                -0x6f51c000,
                                -0x11111112,
                                14,
                                lw(sp + 96) | alpha << 24,
                                0x3f800000,
                                1,
                                0,
                                0,
                                0,
                                3,
                                0,
                                packed_corner(lwc1(sp + 32), lwc1(sp + 36)),
                                lwc1(sp + 40) as i32,
                                5,
                                0,
                                0x4000000,
                                0,
                                3,
                                0,
                                packed_corner(lwc1(sp + 64), lwc1(sp + 68)),
                                lwc1(sp + 56) as i32,
                                5,
                                0,
                                2048,
                                0,
                                3,
                                0,
                                packed_corner(lwc1(sp + 48), lwc1(sp + 52)),
                                lwc1(sp + 72) as i32,
                                5,
                                0,
                                0x4000800,
                                0,
                                3,
                                0,
                            ];
                            put_words(packet + 4, &words);
                            put_words(
                                packet + 4 + 36 * 4,
                                &[packed_corner(lwc1(sp + 80), lwc1(sp + 84)), lwc1(sp + 88) as i32, 5, 0, 0x140007fc, 0x11000000, 0],
                            );
                            sw(g::gfxtag, tag + 16);
                            sh(tag, 11);
                        }
                    }
                }
                count = lw(particle + 24);
            }
        }
        i += 1;
        offset += 4;
        if i >= count {
            break;
        }
    }
}

/// Draw the tracer particle `particle` (a line from its place (+928) to its place plus the velocity (+940) times the constant at 0x3ad53c) with the colour `colour` through the
/// matrix `matrix`: both ends are put on the screen (when they are in the view) and a quad of the ends' width, thinner at the far end, is sent as a packet of 192 bytes
/// (@ 0x29e2c8).
pub fn particleTracerGfx(particle: i32, colour: i32, matrix: i32) {
    let frame = Frame::new(272);
    let sp = frame.sp();
    let reach = lwc1(0x3ad53c);
    let x0 = lwc1(particle + 928);
    let y0 = lwc1(particle + 932);
    let z0 = lwc1(particle + 936);
    let zoom = lwc1(lw(g::currentplayer) + 260);
    let y1 = y0 + lwc1(particle + 944) * reach;
    let z1 = z0 + lwc1(particle + 948) * reach;
    swc1(sp + 16, x0 + lwc1(particle + 940) * reach);
    swc1(sp + 20, y1);
    swc1(sp + 24, z1);
    swc1(sp, x0);
    swc1(sp + 4, y0);
    swc1(sp + 8, z0);
    matrixVec3Mul4Aligned(&[matrix as i64, sp as i64, (sp + 32) as i64]);
    matrixVec3Mul4Aligned(&[matrix as i64, (sp + 16) as i64, (sp + 48) as i64]);
    let w1 = lwc1(sp + 44);
    let a_x = lwc1(sp + 32);
    let limit_a = w1 + w1;
    let w2 = lwc1(sp + 60);
    let low_a = -limit_a;
    let limit_b = w2 + w2;
    if !(low_a <= a_x) {
        return;
    }
    let a_y = lwc1(sp + 36);
    if !(a_x <= limit_a) || !(low_a <= a_y) {
        return;
    }
    let a_z = lwc1(sp + 40);
    if !(a_y <= limit_a) || !(-w1 <= a_z) {
        return;
    }
    let b_x = lwc1(sp + 48);
    let low_b = -limit_b;
    if !(a_z <= w1) || !(low_b <= b_x) {
        return;
    }
    let b_y = lwc1(sp + 52);
    if !(b_x <= limit_b) || !(low_b <= b_y) {
        return;
    }
    let b_z = lwc1(sp + 56);
    if !(b_y <= limit_b) || !(-w2 <= b_z) || !(b_z <= w2) {
        return;
    }
    let scale = lwc1(0x3ad540);
    let inverse_a = 1.0_f32 / w1;
    let inverse_b = 1.0_f32 / w2;
    let across = lw(g::window + 16) as f32;
    let centre_x = lw(g::window + 8) as f32;
    let centre_y = lw(g::window + 28) as f32;
    let down = (0 - lw(g::window + 36)) as f32;
    let a_sx = a_x * inverse_a;
    let b_sy = b_y * inverse_b;
    let b_sx = b_x * inverse_b;
    let a_sy = a_y * inverse_a;
    swc1(sp + 32, a_sx);
    swc1(sp + 52, b_sy);
    swc1(sp + 48, b_sx);
    swc1(sp + 36, a_sy);
    let b_py = centre_y + down * b_sy;
    let b_px = centre_x + across * b_sx;
    let a_px = centre_x + across * a_sx;
    let a_py = centre_y + down * a_sy;
    swc1(sp + 84, b_py);
    let a_sz = a_z * inverse_a;
    swc1(sp + 80, b_px);
    let dx = b_px - a_px;
    swc1(sp + 64, a_px);
    let dy = b_py - a_py;
    swc1(sp + 68, a_py);
    let b_sz = b_z * inverse_b;
    swc1(sp + 40, a_sz);
    swc1(sp + 56, b_sz);
    let width = 6e+01_f32 / zoom * 3e+01_f32;
    swc1(sp + 88, b_sz * scale + 32768.0_f32);
    let width_b = width / (b_z + 5.0_f32);
    let width_a = width / (a_z + 5.0_f32);
    swc1(sp + 72, a_sz * scale + 32768.0_f32);
    let unit = 1.0_f32 / (dx * dx + dy * dy).abs().sqrt();
    let p_x = lwc1(sp + 64);
    let p_y = lwc1(sp + 68);
    let q_x = lwc1(sp + 80);
    let q_y = lwc1(sp + 84);
    let ny = dy * unit;
    let nx = dx * unit;
    let ney = -ny;
    let nex = -nx;
    let sum = width_b * (nx + ny);
    swc1(sp + 112, p_x + width_a * (nex - ny));
    swc1(sp + 116, p_y + width_a * (ney + nx) * 0.5_f32);
    swc1(sp + 144, q_x + width_b * (nx - ny));
    swc1(sp + 148, q_y + sum * 0.5_f32);
    swc1(sp + 96, p_x + width_a * (nex + ny));
    swc1(sp + 100, p_y + width_a * (ney - nx) * 0.5_f32);
    swc1(sp + 132, q_y + width_b * (ny - nx) * 0.5_f32);
    swc1(sp + 128, q_x + sum);
    let packet = memdbAlloc(192);
    sb(lw(g::gfxtag) + 3, 48);
    sw(packet, 0x6c0b8000);
    let tag = lw(g::gfxtag);
    sw(tag + 4, packet);
    let tinted = colour | -0x80000000;
    let zero_a = inverse_a * 0.0_f32;
    let words: [i32; 38] = [
        0x8001,
        -0x5fd1c000,
        -0x11111112,
        238,
        tinted,
        inverse_a.to_bits() as i32,
        1,
        0,
        zero_a.to_bits() as i32,
        zero_a.to_bits() as i32,
        2,
        0,
        packed_corner(lwc1(sp + 96), lwc1(sp + 100)),
        lwc1(sp + 72) as i32,
        5,
        0,
        zero_a.to_bits() as i32,
        inverse_a.to_bits() as i32,
        2,
        0,
        packed_corner(lwc1(sp + 112), lwc1(sp + 116)),
        lwc1(sp + 72) as i32,
        5,
        0,
        tinted,
        inverse_b.to_bits() as i32,
        1,
        0,
        inverse_b.to_bits() as i32,
        (inverse_b * 0.0_f32).to_bits() as i32,
        2,
        0,
        packed_corner(lwc1(sp + 128), lwc1(sp + 132)),
        lwc1(sp + 88) as i32,
        5,
        0,
        inverse_b.to_bits() as i32,
        inverse_b.to_bits() as i32,
    ];
    put_words(packet + 4, &words);
    put_words(
        packet + 4 + 38 * 4,
        &[2, 0, packed_corner(lwc1(sp + 144), lwc1(sp + 148)), lwc1(sp + 88) as i32, 5, 0, 0x140007fc, 0x11000000, 0],
    );
    sw(g::gfxtag, tag + 16);
    sh(tag, 12);
}
