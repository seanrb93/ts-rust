//! The drawing of the particles whose points are moved by the game's own code.

use super::explosion_and_spark_group_drawing::{begin_batch, close_full_batch, close_last_batch};
use super::particle_ray_and_pulse_drawing::{packed_corner, put_words};
use crate::*;

/// Draw the manual particle `particle` (the kind of particle at +4, `count` points from +24 with their places at +992, colours at +2192, turn speeds at +2592) through the matrix
/// `matrix`: the particle's own matrix (at +928) is put with the view's, and each point that is in view and younger than the life of the kind (+12 of the info record of 104
/// bytes) is drawn as a turned square of the sizes of the kind, 23 to a batch, with opacity rising over the fade in and falling over the fade out (@ 0x2a0728).
pub fn particleManualGfx(particle: i32, matrix: i32) {
    let frame = Frame::new(400);
    let sp = frame.sp();
    let mut batched = 0;
    let mut i = 0;
    let mut header = 0;
    let zoom = (6e+01_f32).pdiv(lwc1(lw(g::currentplayer) + 260));
    let info = lw(particle + 4) * 104 + g::particleinfos;
    let textures = (lw(info) << 4) + g::particletextures;
    let tex_b = lw(textures + 4);
    let tex_a = lw(textures + 8);
    dlSelectTextureKick(lw(textures + 12));
    dlSetDitherMatrix(2);
    sceVu0MulMatrix(sp, matrix, particle + 928);
    let mut count = lw(particle + 24);
    if count > 0 {
        let tex_b = tex_b << 4;
        let tex_a = tex_a << 20;
        let window = g::window;
        sw(sp + 164, tex_b);
        sw(sp + 160, tex_a);
        let one = 1.0_f32;
        let half = 0.5_f32;
        sw(sp + 168, tex_b | tex_a);
        let mut age = lwc1(particle + 12);
        loop {
            if 0.0_f32 <= age {
                let life = lwc1(info + 12);
                if age < life {
                    let mut fade = 1.0_f32;
                    let at = particle + (i << 2);
                    let colour_word = fptoui(lwc1(at + 2192));
                    let turn = lwc1(at + 2592) * age;
                    let fade_in = lwc1(info + 24);
                    let turn = (turn * lwc1(0x3ad580)).pdiv(1.8e+02_f32);
                    if !(age < fade_in) {
                        let fade_out = lwc1(info + 28);
                        if (life - fade_out) < age {
                            fade = ((life - age)).pdiv(fade_out);
                        }
                    } else {
                        fade = (age).pdiv(fade_in);
                    }
                    let place = particle + i * 12;
                    let z = lwc1(place + 1000);
                    let y = lwc1(place + 996);
                    swc1(sp + 64, lwc1(place + 992));
                    swc1(sp + 68, y);
                    swc1(sp + 72, z);
                    matrixVec3Mul4Aligned(&[sp as i64, (sp + 64) as i64, (sp + 80) as i64]);
                    let w = lwc1(sp + 92);
                    let x = lwc1(sp + 80);
                    let low = -w;
                    if low <= x {
                        let y = lwc1(sp + 84);
                        if x <= w && low <= y {
                            let z = lwc1(sp + 88);
                            if y <= w && low <= z && z <= w {
                                let mut w = w;
                                let mut x = x;
                                let mut y = y;
                                let mut z = z;
                                if batched == 0 {
                                    header = begin_batch();
                                    w = lwc1(sp + 92);
                                    x = lwc1(sp + 80);
                                    y = lwc1(sp + 84);
                                    z = lwc1(sp + 88);
                                }
                                let size_raw = lwc1(info + 40);
                                let inverse = (one).pdiv(w);
                                let across = lw(window + 16) as f32;
                                let scale = lwc1(0x3ad588);
                                let down = 0 - lw(window + 36);
                                let size = (size_raw).pdiv(z) * zoom;
                                let depth = z * inverse;
                                let width = size * (age * lwc1(info + 48) + one);
                                batched += 1;
                                let height = size * (age * lwc1(info + 44) + one);
                                let index = (turn * lwc1(0x3ad584) + half) as i32;
                                swc1(sp + 88, depth);
                                let screen_x = x * inverse;
                                let screen_y = y * inverse;
                                let table = g::g_SinTable;
                                let cosine = lwc1((((index + 512) & 2047) << 2) + table);
                                let sine = lwc1(((index & 2047) << 2) + table);
                                swc1(sp + 80, screen_x);
                                let dy = (height as i32) as f32 * cosine;
                                swc1(sp + 84, screen_y);
                                let dx = (width as i32) as f32 * sine;
                                let py = lw(window + 28) as f32 + down as f32 * screen_y;
                                let px = lw(window + 8) as f32 + across * screen_x;
                                swc1(sp + 68, py);
                                let half_dx = dx * half;
                                let half_dy = dy * half;
                                swc1(sp + 64, px);
                                let first = colour_word | ((fade * 4e+01_f32) as i32) << 24;
                                swc1(sp + 144, px - dy);
                                swc1(sp + 148, py - half_dx);
                                swc1(sp + 128, px - dx);
                                let z_out = depth * scale + 32768.0_f32 + 1e+02_f32;
                                swc1(sp + 132, py + half_dy);
                                swc1(sp + 96, px + dy);
                                swc1(sp + 100, py + half_dx);
                                swc1(sp + 152, z_out);
                                swc1(sp + 112, px + dx);
                                swc1(sp + 116, py - half_dy);
                                swc1(sp + 72, z_out);
                                swc1(sp + 104, z_out);
                                swc1(sp + 120, z_out);
                                swc1(sp + 136, z_out);
                                let packet = memdbAlloc(144);
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
                                    packed_corner(lwc1(sp + 96), lwc1(sp + 100)),
                                    lwc1(sp + 104) as i32,
                                    13,
                                    0,
                                    lw(sp + 160),
                                    0,
                                    3,
                                    0,
                                    packed_corner(lwc1(sp + 128), lwc1(sp + 132)),
                                    lwc1(sp + 120) as i32,
                                    13,
                                    0,
                                    lw(sp + 164),
                                    0,
                                    3,
                                    0,
                                    packed_corner(lwc1(sp + 112), lwc1(sp + 116)),
                                    lwc1(sp + 136) as i32,
                                    5,
                                    0,
                                    lw(sp + 168),
                                    0,
                                    3,
                                    0,
                                    packed_corner(lwc1(sp + 144), lwc1(sp + 148)),
                                    lwc1(sp + 152) as i32,
                                ];
                                put_words(packet + 4, &words);
                                put_words(packet + 4 + 33 * 4, &[5, 0]);
                                sw(g::gfxtag, tail);
                                sh(tag, 9);
                                if batched == 23 {
                                    header = close_full_batch(header, tail);
                                    batched = 0;
                                }
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
            age = lwc1(particle + 12);
        }
    }
    if batched != 0 {
        close_last_batch(header, batched);
    }
}
