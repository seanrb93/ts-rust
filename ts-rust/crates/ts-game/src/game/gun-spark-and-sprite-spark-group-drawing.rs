//! The drawing of the groups of sparks of the guns and the sprite sparks.

use super::particle_ray_and_pulse_drawing::{packed_corner, put_words};
use crate::*;

/// Draw the group of gun sparks `particle` (`count` sparks from +24: their velocities at +940 in groups of 12 bytes, the age at +12) with the colour `colour` through the matrix
/// `matrix`: every spark is a streak from where it was `delta` ago to where it is, drawn as a quad of the streak's width `width` when both ends are in the view, with the opacity
/// falling over the life (the constant at 0x3ad518), as a packet of 176 bytes (@ 0x29c810).
pub fn particleGunSparkGroupGfx(particle: i32, colour: i32, matrix: i32, delta: f32, width: f32) {
    let frame = Frame::new(336);
    let sp = frame.sp();
    let life = lwc1(0x3ad518);
    let mut i = 0;
    let gravity = -1.5_f32;
    let mut age = lwc1(particle + 12);
    sw(sp + 128, colour);
    let later = ps2float::add(age, delta);
    let count = lw(particle + 24);
    let drop = ps2float::mul(ps2float::mul(age, gravity), age);
    sw(sp + 132, ps2float::mul((ps2float::sub(life, age)).pdiv(life), 127.0_f32) as i32 & 255);
    if count <= 0 {
        return;
    }
    loop {
        let y = lwc1(particle + 932);
        let x = lwc1(particle + 928);
        let z = lwc1(particle + 936);
        let at = particle + 948 + i * 12;
        let vy = lwc1(particle + 944 + i * 12);
        let vz = lwc1(at);
        let vx = lwc1(particle + 940 + i * 12);
        swc1(sp + 16, ps2float::add(x, ps2float::mul(vx, later)));
        swc1(sp + 24, ps2float::add(z, ps2float::mul(vz, later)));
        swc1(sp, ps2float::add(x, ps2float::mul(vx, age)));
        swc1(sp + 4, ps2float::add(ps2float::add(y, ps2float::mul(vy, age)), drop));
        swc1(sp + 8, ps2float::add(z, ps2float::mul(vz, age)));
        swc1(sp + 20, ps2float::add(ps2float::add(y, ps2float::mul(vy, later)), ps2float::mul(ps2float::mul(later, gravity), later)));
        matrixVec3Mul4Aligned(&[matrix as i64, sp as i64, (sp + 32) as i64]);
        matrixVec3Mul4Aligned(&[matrix as i64, (sp + 16) as i64, (sp + 48) as i64]);
        let w1 = lwc1(sp + 44);
        let x1 = lwc1(sp + 32);
        let low1 = -w1;
        let mut count = lw(particle + 24);
        if low1 <= x1 {
            let y1 = lwc1(sp + 36);
            if x1 <= w1 && low1 <= y1 {
                let z1 = lwc1(sp + 40);
                if y1 <= w1 && low1 <= z1 {
                    let w2 = lwc1(sp + 60);
                    if z1 <= w1 {
                        let x2 = lwc1(sp + 48);
                        let low2 = -w2;
                        if low2 <= x2 {
                            let y2 = lwc1(sp + 52);
                            if x2 <= w2 && low2 <= y2 {
                                let z2 = lwc1(sp + 56);
                                if y2 <= w2 && low2 <= z2 && z2 <= w2 {
                                    let one = 1.0_f32;
                                    let window = g::window;
                                    let depth_scale = lwc1(0x3ad51c);
                                    let inverse2 = (one).pdiv(w2);
                                    let inverse1 = (one).pdiv(w1);
                                    let across = lw(window + 16) as f32;
                                    let down = (0 - lw(window + 36)) as f32;
                                    let half = 0.5_f32;
                                    let centre_x = lw(window + 8) as f32;
                                    let centre_y = lw(window + 28) as f32;
                                    let sx2 = ps2float::mul(x2, inverse2);
                                    let sy2 = ps2float::mul(y2, inverse2);
                                    let sx1 = ps2float::mul(x1, inverse1);
                                    let sy1 = ps2float::mul(y1, inverse1);
                                    swc1(sp + 48, sx2);
                                    swc1(sp + 52, sy2);
                                    swc1(sp + 32, sx1);
                                    swc1(sp + 36, sy1);
                                    let px2 = ps2float::add(centre_x, ps2float::mul(across, sx2));
                                    let py2 = ps2float::add(centre_y, ps2float::mul(down, sy2));
                                    let px1 = ps2float::add(centre_x, ps2float::mul(across, sx1));
                                    let py1 = ps2float::add(centre_y, ps2float::mul(down, sy1));
                                    let sz2 = ps2float::mul(z2, inverse2);
                                    swc1(sp + 56, sz2);
                                    let sz1 = ps2float::mul(z1, inverse1);
                                    swc1(sp + 40, sz1);
                                    let ox = ps2float::mul(ps2float::mul(width, ps2float::sub(px2, px1)), half);
                                    let oy = ps2float::mul(ps2float::mul(width, ps2float::sub(py2, py1)), half);
                                    swc1(sp + 88, ps2float::add(ps2float::mul(sz2, depth_scale), 32768.0_f32));
                                    swc1(sp + 84, ps2float::add(py2, ox));
                                    swc1(sp + 80, ps2float::sub(px2, oy));
                                    swc1(sp + 64, ps2float::sub(px1, oy));
                                    swc1(sp + 68, ps2float::add(py1, ox));
                                    swc1(sp + 72, ps2float::add(ps2float::mul(sz1, depth_scale), 32768.0_f32));
                                    swc1(sp + 96, ps2float::add(px1, oy));
                                    swc1(sp + 100, ps2float::sub(py1, ox));
                                    swc1(sp + 112, ps2float::add(px2, oy));
                                    swc1(sp + 116, ps2float::sub(py2, ox));
                                    let packet = memdbAlloc(176);
                                    sb(lw(g::gfxtag) + 3, 48);
                                    sw(packet, 0x6c0a8000);
                                    let tag = lw(g::gfxtag);
                                    sw(tag + 4, packet);
                                    let alpha = lw(sp + 132) << 24;
                                    let tint = lw(sp + 128);
                                    let shared = lwc1(sp + 72) as i32;
                                    let words: [i32; 38] = [
                                        0x8001,
                                        -0x6f51c000,
                                        -0x11111112,
                                        14,
                                        tint | alpha,
                                        0x3f800000,
                                        1,
                                        0,
                                        0,
                                        0,
                                        3,
                                        0,
                                        packed_corner(lwc1(sp + 64), lwc1(sp + 68)),
                                        shared,
                                        5,
                                        0,
                                        0x4000000,
                                        0,
                                        3,
                                        0,
                                        packed_corner(lwc1(sp + 96), lwc1(sp + 100)),
                                        shared,
                                        5,
                                        0,
                                        2048,
                                        0,
                                        3,
                                        0,
                                        packed_corner(lwc1(sp + 80), lwc1(sp + 84)),
                                        shared,
                                        5,
                                        0,
                                        0x4000800,
                                        0,
                                        3,
                                        0,
                                        packed_corner(lwc1(sp + 112), lwc1(sp + 116)),
                                        shared,
                                    ];
                                    put_words(packet + 4, &words);
                                    put_words(packet + 4 + 38 * 4, &[5, 0, 0x140007fc, 0x11000000, 0]);
                                    sw(g::gfxtag, tag + 16);
                                    sh(tag, 11);
                                }
                            }
                        }
                    }
                }
            }
            count = lw(particle + 24);
        } else {
            count = lw(particle + 24);
        }
        i += 1;
        if i >= count {
            break;
        }
        age = lwc1(particle + 12);
    }
}

/// Draw the group of sprite sparks `particle` (`count` sparks from +24 at the velocities at +940, falling by the constant at 0x3ad520 over the time): the sparks that are in the
/// view are drawn as squares of the size 100 over the distance, 23 of them to a batch (the batch's header of 32 bytes is made when the first is, and filled in with its count when
/// the batch is full or the group is done) (@ 0x29ce70).
pub fn particleSpriteSparkGroupGfx(particle: i32, colour: i32, matrix: i32) {
    let frame = Frame::new(288);
    let sp = frame.sp();
    let life = lwc1(0x3ad520);
    let mut i = 0;
    let mut batched = 0;
    let mut header = 0;
    let mut age = lwc1(particle + 12);
    sw(sp + 96, colour);
    sw(sp + 100, matrix);
    let fall = ps2float::mul(ps2float::mul(age, -2.5_f32), age);
    let zoom = (6e+01_f32).pdiv(lwc1(lw(g::currentplayer) + 260));
    let opacity = ps2float::mul((ps2float::sub(life, age)).pdiv(life), 32.0_f32) as i32 & 255;
    if lw(particle + 24) > 0 {
        loop {
            let at = particle + 952 + i * 12;
            let z = ps2float::add(lwc1(particle + 940), ps2float::mul(lwc1(at), age));
            let y = ps2float::add(ps2float::add(lwc1(particle + 936), ps2float::mul(lwc1(particle + 948 + i * 12), age)), fall);
            swc1(sp, ps2float::add(lwc1(particle + 932), ps2float::mul(lwc1(particle + 944 + i * 12), age)));
            swc1(sp + 8, z);
            swc1(sp + 4, y);
            matrixVec3Mul4Aligned(&[lw(sp + 100) as i64, sp as i64, (sp + 16) as i64]);
            let w = lwc1(sp + 28);
            let x = lwc1(sp + 16);
            let low = -w;
            let mut count = lw(particle + 24);
            if low <= x {
                let y = lwc1(sp + 20);
                if x < w && low <= y && y < w {
                    let z = lwc1(sp + 24);
                    if low <= z && z < w {
                        let mut x = x;
                        let mut y = y;
                        let mut w = w;
                        if batched == 0 {
                            header = memdbAlloc(32);
                            sb(lw(g::gfxtag) + 3, 48);
                            let tag = lw(g::gfxtag);
                            w = lwc1(sp + 28);
                            sh(tag, 2);
                            sw(tag + 4, header);
                            sw(g::gfxtag, tag + 16);
                            x = lwc1(sp + 16);
                            y = lwc1(sp + 20);
                        }
                        let window = g::window;
                        let inverse = (1.0_f32).pdiv(w);
                        let scale = lwc1(0x3ad524);
                        let across = lw(window + 16) as f32;
                        let centre_x = lw(window + 8) as f32;
                        let centre_y = lw(window + 28) as f32;
                        batched += 1;
                        let down = (0 - lw(window + 36)) as f32;
                        let screen_x = ps2float::mul(x, inverse);
                        let screen_y = ps2float::mul(y, inverse);
                        let size = (ps2float::mul(zoom, 1e+02_f32).pdiv(z)) as i32;
                        let depth = ps2float::mul(z, inverse);
                        swc1(sp + 16, screen_x);
                        swc1(sp + 20, screen_y);
                        swc1(sp + 24, depth);
                        let px = ps2float::add(centre_x, ps2float::mul(across, screen_x));
                        let py = ps2float::add(centre_y, ps2float::mul(down, screen_y));
                        let half = ((size + ((size as u32) >> 31) as i32) >> 1) as f32;
                        let full = size as f32;
                        swc1(sp, px);
                        swc1(sp + 4, py);
                        let z_out = ps2float::add(ps2float::mul(depth, scale), 32768.0_f32);
                        let bottom = ps2float::add(py, half);
                        let right = ps2float::add(px, full);
                        let left = ps2float::sub(px, full);
                        swc1(sp + 88, z_out);
                        let top = ps2float::sub(py, half);
                        swc1(sp + 84, bottom);
                        swc1(sp + 80, right);
                        swc1(sp + 64, left);
                        swc1(sp + 52, top);
                        swc1(sp + 8, z_out);
                        swc1(sp + 32, left);
                        swc1(sp + 36, top);
                        swc1(sp + 40, z_out);
                        swc1(sp + 48, right);
                        swc1(sp + 56, z_out);
                        swc1(sp + 68, bottom);
                        swc1(sp + 72, z_out);
                        let packet = memdbAlloc(144);
                        let first = lw(sp + 96) | opacity << 24;
                        sb(lw(g::gfxtag) + 3, 48);
                        sw(packet, first);
                        let tag = lw(g::gfxtag);
                        sw(tag + 4, packet);
                        let tail = tag + 16;
                        let words: [i32; 33] = [
                            0x3f800000,
                            1,
                            0,
                            0,
                            0,
                            3,
                            0,
                            packed_corner(lwc1(sp + 32), lwc1(sp + 36)),
                            lwc1(sp + 40) as i32,
                            13,
                            0,
                            0x4000000,
                            0,
                            3,
                            0,
                            packed_corner(lwc1(sp + 64), lwc1(sp + 68)),
                            lwc1(sp + 56) as i32,
                            13,
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
                            packed_corner(lwc1(sp + 80), lwc1(sp + 84)),
                            lwc1(sp + 88) as i32,
                        ];
                        put_words(packet + 4, &words);
                        put_words(packet + 4 + 33 * 4, &[5, 0]);
                        sw(g::gfxtag, tail);
                        sh(tag, 9);
                        if batched == 23 {
                            sw(header, 0);
                            sw(header + 4, 0);
                            sw(header + 8, 0);
                            sw(header + 12, 0x6cd08000);
                            sw(header + 16, 0x8017);
                            sw(header + 20, -0x6f51c000);
                            batched = 0;
                            sw(header + 24, -0x11111112);
                            sw(header + 28, 14);
                            sb(tail + 3, 48);
                            header += 32;
                            let tag = lw(g::gfxtag);
                            sw(tag + 4, g::vif_entry_pass);
                            sh(tag, 1);
                            sw(g::gfxtag, tag + 16);
                        }
                    }
                }
                count = lw(particle + 24);
            } else {
                count = lw(particle + 24);
            }
            i += 1;
            if i >= count {
                break;
            }
            age = lwc1(particle + 12);
        }
    }
    let eight = batched << 3;
    if batched != 0 {
        sw(header, 0);
        sw(header + 4, 0);
        sw(header + 8, 0);
        sw(header + 12, (eight + batched + 1) << 16 | 0x6c008000);
        sw(header + 16, batched | 0x8000);
        let tag_last = lw(g::gfxtag);
        sw(header + 20, -0x6f51c000);
        sw(header + 28, 14);
        sw(header + 24, -0x11111112);
        sb(tag_last + 3, 48);
        let tag = lw(g::gfxtag);
        sw(tag + 4, g::vif_entry_pass);
        sh(tag, 1);
        sw(g::gfxtag, tag + 16);
    }
}
