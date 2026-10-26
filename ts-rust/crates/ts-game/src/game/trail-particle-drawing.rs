//! The drawing of the trail particles.

use super::particle_ray_and_pulse_drawing::{packed_corner, put_words};
use crate::*;

/// Draw the trail particle `particle` (the kind of particle at +4, `count` points from +24) through the matrix `matrix`: each point whose age (the particle's time at +12 less its
/// start along the trail) is inside the life of the kind (+12 of the info record of 104 bytes) and that is in the view is drawn as a turned, textured square whose size, spin and
/// opacity follow the age, as a packet of 176 bytes (@ 0x29f5e8).
pub fn particleTrailGfx(particle: i32, matrix: i32) {
    let frame = Frame::new(352);
    let sp = frame.sp();
    let infos = g::particleinfos;
    let mut i = 0;
    let zoom_raw = lwc1(lw(g::currentplayer) + 260);
    let offset = lw(particle + 4) * 104;
    sw(sp + 96, matrix);
    let info = offset + infos;
    sw(sp + 100, lw(info + 32));
    let life = lwc1(info + 12);
    let fade_in = lwc1(info + 24);
    let fade_out = lwc1(info + 28);
    dlSelectTextureKick(lw((lw(info) << 4) + (g::particletextures + 12)));
    dlSetDitherMatrix(2);
    let mut count = lw(particle + 24);
    if count <= 0 {
        return;
    }
    let one = 1.0_f32;
    let half = 0.5_f32;
    let window = g::window;
    loop {
        let age = lwc1(particle + 12) - (i as f32 * fade_in).pdiv(count as f32);
        if 0.0_f32 <= age {
            if age < life {
                let ratio = (age).pdiv(lwc1(info + 12));
                let spin = lwc1(particle + (i << 2) + 2128) * age;
                let mut opacity = 1.0_f32;
                let turn = (spin * lwc1(0x3ad55c)).pdiv(1.8e+02_f32);
                let mut spin_abs = spin;
                if !(age < fade_in) {
                    if (life - fade_out) < age {
                        opacity = ((life - age)).pdiv(fade_out);
                    }
                } else {
                    opacity = (age).pdiv(fade_in);
                }
                if spin < 0.0_f32 {
                    spin_abs = -spin;
                }
                let at = particle + i * 12;
                let y = lwc1(at + 932) + lwc1(info + 84) * spin_abs + lwc1(at + 2532) * age;
                let z = lwc1(at + 936) + lwc1(info + 88) * spin + lwc1(at + 2536) * age;
                swc1(sp, lwc1(at + 928) + lwc1(info + 80) * spin + lwc1(at + 2528) * age);
                swc1(sp + 4, y);
                swc1(sp + 8, z);
                matrixVec3Mul4Aligned(&[lw(sp + 96) as i64, sp as i64, (sp + 16) as i64]);
                let w = lwc1(sp + 28);
                let x = lwc1(sp + 16);
                let low = -w;
                if low <= x {
                    let y = lwc1(sp + 20);
                    if x <= w && low <= y {
                        let z = lwc1(sp + 24);
                        if y <= w && low <= z && z <= w {
                            let inverse = (one).pdiv(w);
                            let across = lw(window + 16) as f32;
                            let centre_x = lw(window + 8) as f32;
                            let grow_x = ratio * lwc1(info + 48);
                            let down = lw(window + 36);
                            let grow_y = ratio * lwc1(info + 44);
                            let depth = z * inverse;
                            let texture = lw(info) << 4;
                            let angle = turn * lwc1(0x3ad560) + half;
                            let screen_x = x * inverse;
                            let table = g::g_SinTable;
                            let base_size = (lwc1(info + 40)).pdiv(depth);
                            swc1(sp + 24, depth);
                            let screen_y = y * inverse;
                            let index = angle as i32;
                            let tex_a = lw(texture + (g::particletextures + 8));
                            let scaled = base_size * ((60.0_f32).pdiv(zoom_raw));
                            let sine = lwc1(((index & 2047) << 2) + table);
                            let cosine = lwc1((((index + 512) & 2047) << 2) + table);
                            let tex_b = lw(texture + (g::particletextures + 4));
                            swc1(sp + 16, screen_x);
                            swc1(sp + 20, screen_y);
                            let dx = (scaled * (grow_x + one)) as i32 as f32 * sine;
                            let dy = (scaled * (grow_y + one)) as i32 as f32 * cosine;
                            let depth_out = depth * lwc1(0x3ad564);
                            let px = centre_x + across * screen_x;
                            let py = lw(window + 28) as f32 + (0 - down) as f32 * screen_y;
                            let half_dx = dx * half;
                            let half_dy = dy * half;
                            swc1(sp, px);
                            swc1(sp + 4, py);
                            swc1(sp + 80, px - dy);
                            swc1(sp + 84, py - half_dx);
                            swc1(sp + 64, px - dx);
                            let z_out = depth_out + 32768.0_f32 + 1e+02_f32;
                            swc1(sp + 68, py + half_dy);
                            let alpha_f = opacity * 3e+02_f32;
                            swc1(sp + 32, px + dy);
                            swc1(sp + 36, py + half_dx);
                            swc1(sp + 88, z_out);
                            swc1(sp + 8, z_out);
                            swc1(sp + 48, px + dx);
                            swc1(sp + 52, py - half_dy);
                            swc1(sp + 40, z_out);
                            swc1(sp + 56, z_out);
                            swc1(sp + 72, z_out);
                            let product = dpmul(fptodp(alpha_f), dpmul(fptodp(lwc1(sp + 8)), 0x3f00000000000000_i64));
                            let mut alpha = 127;
                            if dpcmp(product, ld(0x3a9050)) <= 0 {
                                let product = dpmul(fptodp(alpha_f), dpmul(fptodp(lwc1(sp + 8)), 0x3f00000000000000_i64));
                                alpha = dptoul(product) as i32 & 255;
                            }
                            let packet = memdbAlloc(176);
                            sb(lw(g::gfxtag) + 3, 48);
                            sw(packet, 0x6c0a8000);
                            let tag = lw(g::gfxtag);
                            sw(tag + 4, packet);
                            let tex_a = tex_a << 20;
                            let tex_b = tex_b << 4;
                            let words: [i32; 38] = [
                                0x8001,
                                -0x6f51c000,
                                -0x11111112,
                                14,
                                lw(sp + 100) | alpha << 24,
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
                                tex_a,
                                0,
                                3,
                                0,
                                packed_corner(lwc1(sp + 64), lwc1(sp + 68)),
                                lwc1(sp + 56) as i32,
                                5,
                                0,
                                tex_b,
                                0,
                                3,
                                0,
                                packed_corner(lwc1(sp + 48), lwc1(sp + 52)),
                                lwc1(sp + 72) as i32,
                                5,
                                0,
                                tex_b | tex_a,
                                0,
                                3,
                                0,
                                packed_corner(lwc1(sp + 80), lwc1(sp + 84)),
                                lwc1(sp + 88) as i32,
                            ];
                            put_words(packet + 4, &words);
                            put_words(packet + 4 + 38 * 4, &[5, 0, 0x140007fc, 0x11000000, 0]);
                            sw(g::gfxtag, tag + 16);
                            sh(tag, 11);
                        }
                    }
                }
                count = lw(particle + 24);
            }
        }
        i += 1;
        if i >= count {
            break;
        }
    }
}
