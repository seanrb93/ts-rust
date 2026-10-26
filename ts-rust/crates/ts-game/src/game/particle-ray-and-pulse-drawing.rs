//! The drawing of the ray and pulse particles.

use super::particle_spawners::random_as_float;
use crate::*;

/// The packed corner word of a drawing packet: the x and y of the point times 16, moved to the GS's origin (0x6c00, 0x7900), as two halves.
pub(super) fn packed_corner(x: f32, y: f32) -> i32 {
    ((x as i32) << 4) + 0x6c00 & 0xffff | (((y as i32) << 4) + 0x7900) << 16
}

/// Write the words `words` one after the other from `at`, and the closing words 0x11000000 and 0 of a packet of a quad when `close` is set.
pub(super) fn put_words(at: i32, words: &[i32]) {
    let mut put = at;
    for word in words {
        sw(put, *word);
        put += 4;
    }
}

/// Draw the ray particle `particle` (`count` points from +24 along its direction (+944 to +948) from its place (+928), 40 steps to a unit) with the colour `colour` through the
/// matrix `matrix`: each point that is in the view is drawn as a square turned by a random angle as a packet of 176 bytes (@ 0x29efb8).
pub fn particleRayGfx(particle: i32, colour: i32, matrix: i32) {
    let frame = Frame::new(240);
    let sp = frame.sp();
    let zoom = (6e+01_f32).pdiv(lwc1(lw(g::currentplayer) + 260));
    let mut i = 0;
    if lw(particle + 24) <= 0 {
        return;
    }
    loop {
        let step = i as f32;
        let forty = 4e+01_f32;
        let y = lwc1(particle + 932) + (lwc1(particle + 944) * step).pdiv(forty);
        let z = lwc1(particle + 936) + (lwc1(particle + 948) * step).pdiv(forty);
        swc1(sp, lwc1(particle + 928) + (lwc1(particle + 940) * step).pdiv(forty));
        swc1(sp + 4, y);
        swc1(sp + 8, z);
        matrixVec3Mul4Aligned(&[matrix as i64, sp as i64, (sp + 16) as i64]);
        let w = lwc1(sp + 28);
        let x = lwc1(sp + 16);
        let low = -w;
        let mut count = lw(particle + 24);
        if low <= x {
            let y = lwc1(sp + 20);
            if x <= w && low <= y {
                let z = lwc1(sp + 24);
                if y <= w && low <= z && z <= w {
                    let window = g::window;
                    let inverse = (1.0_f32).pdiv(w);
                    let scale = lwc1(0x3ad550);
                    let across = lw(window + 16) as f32;
                    let down = (0 - lw(window + 36)) as f32;
                    let centre_x = lw(window + 8) as f32;
                    let depth = z * inverse;
                    let screen_x = x * inverse;
                    let screen_y = y * inverse;
                    swc1(sp + 24, depth);
                    swc1(sp + 16, screen_x);
                    swc1(sp + 20, screen_y);
                    let py = lw(window + 28) as f32 + down * screen_y;
                    swc1(sp, centre_x + across * screen_x);
                    swc1(sp + 8, depth * scale + 32768.0_f32);
                    swc1(sp + 4, py);
                    let r = random_as_float();
                    let table = g::g_SinTable;
                    let size = (((step + 5e+01_f32) * zoom).pdiv(z)) as i32 as f32;
                    let turn_scale = lwc1(0x3ad558);
                    let half = 0.5_f32;
                    let z_out = lwc1(sp + 8);
                    let px = lwc1(sp);
                    let turn = r * 2.3283064e-10_f32 * 3.6e+02_f32 * lwc1(0x3ad554);
                    swc1(sp + 88, z_out);
                    swc1(sp + 40, z_out);
                    swc1(sp + 56, z_out);
                    swc1(sp + 72, z_out);
                    let py = lwc1(sp + 4);
                    let index = ((turn).pdiv(1.8e+02_f32) * turn_scale + half) as i32;
                    let dx = size * lwc1(((index & 2047) << 2) + table);
                    let dy = size * lwc1((((index + 512) & 2047) << 2) + table);
                    let half_dx = dx * half;
                    let half_dy = dy * half;
                    swc1(sp + 80, px - dy);
                    swc1(sp + 84, py - half_dx);
                    swc1(sp + 64, px - dx);
                    swc1(sp + 68, py + half_dy);
                    swc1(sp + 32, px + dy);
                    swc1(sp + 36, py + half_dx);
                    swc1(sp + 48, px + dx);
                    swc1(sp + 52, py - half_dy);
                    let packet = memdbAlloc(176);
                    sb(lw(g::gfxtag) + 3, 48);
                    sw(packet, 0x6c0a8000);
                    let tag = lw(g::gfxtag);
                    sw(tag + 4, packet);
                    let words: [i32; 38] = [
                        0x8001,
                        -0x6f51c000,
                        -0x11111112,
                        14,
                        colour | 0x7f000000,
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
                        1024,
                        0,
                        3,
                        0,
                        packed_corner(lwc1(sp + 48), lwc1(sp + 52)),
                        lwc1(sp + 72) as i32,
                        5,
                        0,
                        0x4000400,
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
                    count = lw(particle + 24);
                }
            }
        }
        i += 1;
        if i >= count {
            break;
        }
    }
}

/// Draw the pulse particle `particle` (the pulse number at +928, `count` points from +24, 20 steps to a unit, the angle of each at +956) through the matrix `matrix`: each point in
/// the view is drawn as a square turned by its angle, textured by the pulse's textures, as a packet of 176 bytes (@ 0x29e968).
pub fn particlePulseGfx(particle: i32, matrix: i32) {
    let frame = Frame::new(240);
    let sp = frame.sp();
    let mut i = 0;
    let mut pulse = lw(particle + 928);
    let zoom = (6e+01_f32).pdiv(lwc1(lw(g::currentplayer) + 260));
    let colour = lw((pulse << 3) + g::pulsedefs);
    if lw(particle + 24) <= 0 {
        return;
    }
    loop {
        let step = i as f32;
        let twenty = 2e+01_f32;
        let y = lwc1(particle + 936) + (lwc1(particle + 948) * step).pdiv(twenty);
        let z = lwc1(particle + 940) + (lwc1(particle + 952) * step).pdiv(twenty);
        swc1(sp, lwc1(particle + 932) + (lwc1(particle + 944) * step).pdiv(twenty));
        swc1(sp + 4, y);
        swc1(sp + 8, z);
        matrixVec3Mul4Aligned(&[matrix as i64, sp as i64, (sp + 16) as i64]);
        let w = lwc1(sp + 28);
        let x = lwc1(sp + 16);
        let low = -w;
        let mut count = lw(particle + 24);
        if low <= x {
            let y = lwc1(sp + 20);
            if x <= w && low <= y {
                let z = lwc1(sp + 24);
                if y <= w && low <= z && z <= w {
                    let offset = i << 2;
                    let window = g::window;
                    let half = 0.5_f32;
                    let inverse = (1.0_f32).pdiv(w);
                    let down = (0 - lw(window + 36)) as f32;
                    let table = g::g_SinTable;
                    let across = lw(window + 16) as f32;
                    let depth_scale = lwc1(0x3ad54c);
                    let centre_x = lw(window + 8) as f32;
                    let screen_x = x * inverse;
                    let angle = (lwc1(particle + offset + 956) * lwc1(0x3ad544)).pdiv(1.8e+02_f32) * lwc1(0x3ad548) + half;
                    let screen_y = y * inverse;
                    swc1(sp + 16, screen_x);
                    let size = (((step + 5e+01_f32) * zoom).pdiv(z)) as i32 as f32;
                    let index = angle as i32;
                    swc1(sp + 20, screen_y);
                    let depth = z * inverse;
                    let sine = lwc1(((index & 2047) << 2) + table);
                    let px = centre_x + across * screen_x;
                    let cosine = lwc1((((index + 512) & 2047) << 2) + table);
                    swc1(sp + 24, depth);
                    let dy = size * cosine;
                    let py = lw(window + 28) as f32 + down * screen_y;
                    let dx = size * sine;
                    swc1(sp, px);
                    let half_dy = dy * half;
                    swc1(sp + 4, py);
                    let half_dx = dx * half;
                    swc1(sp + 80, px - dy);
                    swc1(sp + 84, py - half_dx);
                    swc1(sp + 64, px - dx);
                    let z_out = depth * depth_scale + 32768.0_f32;
                    swc1(sp + 68, py + half_dy);
                    swc1(sp + 32, px + dy);
                    swc1(sp + 36, py + half_dx);
                    swc1(sp + 88, z_out);
                    swc1(sp + 48, px + dx);
                    swc1(sp + 52, py - half_dy);
                    swc1(sp + 8, z_out);
                    swc1(sp + 40, z_out);
                    swc1(sp + 56, z_out);
                    swc1(sp + 72, z_out);
                    let packet = memdbAlloc(176);
                    sb(lw(g::gfxtag) + 3, 48);
                    sw(packet, 0x6c0a8000);
                    let tag = lw(g::gfxtag);
                    sw(tag + 4, packet);
                    let textures = g::particletextures;
                    let pulse_def = (pulse << 3) + (g::pulsedefs + 4);
                    let first = lw((lw(pulse_def) << 4) + (textures + 8)) << 20;
                    let second = lw((lw(pulse_def) << 4) + (textures + 4)) << 4;
                    let both = lw((lw(pulse_def) << 4) + (textures + 4)) << 4 | lw((lw(pulse_def) << 4) + (textures + 8)) << 20;
                    let words: [i32; 38] = [
                        0x8001,
                        -0x6f51c000,
                        -0x11111112,
                        14,
                        colour | 0x7f000000,
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
                        first,
                        0,
                        3,
                        0,
                        packed_corner(lwc1(sp + 64), lwc1(sp + 68)),
                        lwc1(sp + 56) as i32,
                        5,
                        0,
                        second,
                        0,
                        3,
                        0,
                        packed_corner(lwc1(sp + 48), lwc1(sp + 52)),
                        lwc1(sp + 72) as i32,
                        5,
                        0,
                        both,
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
                    count = lw(particle + 24);
                }
            }
        }
        i += 1;
        if i >= count {
            break;
        }
        pulse = lw(particle + 928);
    }
}
