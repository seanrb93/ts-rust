//! The drawing of explosions and of the groups of sparks, in batches of 23 quads.

use super::particle_ray_and_pulse_drawing::{packed_corner, put_words};
use crate::*;

/// Start a batch: take the 32 bytes of its header and add the tag that will point at the packets after it.
pub(super) fn begin_batch() -> i32 {
    let header = memdbAlloc(32);
    sb(lw(g::gfxtag) + 3, 48);
    let tag = lw(g::gfxtag);
    sh(tag, 2);
    sw(tag + 4, header);
    sw(g::gfxtag, tag + 16);
    header
}

/// Close a full batch of 23 quads: fill in its header (the data count 0x8017) and add the tag of the pass that draws it; returns the place of the next header.
pub(super) fn close_full_batch(header: i32, last_tag: i32) -> i32 {
    sw(header, 0);
    sw(header + 4, 0);
    sw(header + 8, 0);
    sw(header + 12, 0x6cd08000);
    sw(header + 16, 0x8017);
    sw(header + 20, -0x6f51c000);
    sw(header + 24, -0x11111112);
    sw(header + 28, 14);
    sb(last_tag + 3, 48);
    let tag = lw(g::gfxtag);
    sw(tag + 4, g::vif_entry_pass);
    sh(tag, 1);
    sw(g::gfxtag, tag + 16);
    header + 32
}

/// Close the last batch of `count` quads.
pub(super) fn close_last_batch(header: i32, count: i32) {
    sw(header, 0);
    sw(header + 4, 0);
    sw(header + 8, 0);
    sw(header + 12, ((count << 3) + count + 1) << 16 | 0x6c008000);
    sw(header + 16, count | 0x8000);
    let last_tag = lw(g::gfxtag);
    sw(header + 20, -0x6f51c000);
    sw(header + 28, 14);
    sw(header + 24, -0x11111112);
    sb(last_tag + 3, 48);
    let tag = lw(g::gfxtag);
    sw(tag + 4, g::vif_entry_pass);
    sh(tag, 1);
    sw(g::gfxtag, tag + 16);
}

/// Write the packet of one quad of a batch (144 bytes): `first` the first word, the four corners (as the points at sp+32/36, 64/68, 48/52 and 80/84 and their depths) and the
/// markers `markers` for the second and third corner (13 or 5). Returns the tag of the packet.
fn write_quad_packet(sp: i32, first: i32, markers: [i32; 2]) -> i32 {
    let packet = memdbAlloc(144);
    sb(lw(g::gfxtag) + 3, 48);
    sw(packet, first);
    let tag = lw(g::gfxtag);
    sw(tag + 4, packet);
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
        markers[0],
        0,
        0x4000000,
        0,
        3,
        0,
        packed_corner(lwc1(sp + 64), lwc1(sp + 68)),
        lwc1(sp + 56) as i32,
        markers[1],
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
    sw(g::gfxtag, tag + 16);
    sh(tag, 9);
    tag
}

/// Draw the explosion particle `particle` (`count` flashes from +24, each with its start time at +1796, its place at +968 in groups of 12 bytes and the size from the explosion
/// definition of the number at +928) with the colour `colour` through the matrix `matrix`: the flashes that are in the view and in the first half-second of their time (twice
/// the age from 0 to 1, fading in the second half) are drawn as squares, 23 to a batch (@ 0x29d558).
pub fn particleExplosionGfx(particle: i32, colour: i32, matrix: i32) {
    let frame = Frame::new(272);
    let sp = frame.sp();
    let mut i = 0;
    let mut batched = 0;
    let mut count = lw(particle + 24);
    let zoom = (6e+01_f32).pdiv(lwc1(lw(g::currentplayer) + 260));
    let mut header = 0;
    if count > 0 {
        loop {
            let age = ps2float::sub(lwc1(particle + 12), lwc1(particle + 1796 + (i << 2)));
            let doubled = ps2float::add(age, age);
            if 0.0_f32 < doubled && doubled <= 1.0_f32 {
                let mut fade = 1.0_f32;
                if 0.5_f32 < doubled {
                    let down = ps2float::sub(1.0_f32, doubled);
                    fade = ps2float::add(down, down);
                }
                let fade = ps2float::mul(fade, 0.75_f32);
                let at = particle + i * 12;
                let z = lwc1(at + 976);
                let y = lwc1(at + 972);
                swc1(sp, lwc1(at + 968));
                swc1(sp + 4, y);
                swc1(sp + 8, z);
                matrixVec3Mul4Aligned(&[matrix as i64, sp as i64, (sp + 16) as i64]);
                let w = lwc1(sp + 28);
                let x = lwc1(sp + 16);
                let low = -w;
                if low <= x {
                    let y = lwc1(sp + 20);
                    if x <= w && low <= y && y <= w {
                        let z = lwc1(sp + 24);
                        if low <= z && z <= w {
                            let mut w = w;
                            let mut x = x;
                            let mut y = y;
                            let mut z = z;
                            if batched == 0 {
                                header = begin_batch();
                                w = lwc1(sp + 28);
                                x = lwc1(sp + 16);
                                y = lwc1(sp + 20);
                                z = lwc1(sp + 24);
                            }
                            let inverse = (1.0_f32).pdiv(w);
                            let window = g::window;
                            let scale = lwc1(0x3ad528);
                            let centre_x = lw(window + 8) as f32;
                            let down = 0 - lw(window + 36);
                            let size_f = ps2float::mul(ps2float::mul(doubled, zoom), lwc1((lw(particle + 928) << 3) + (g::explosiondefs + 4))).pdiv(z);
                            let screen_x = ps2float::mul(x, inverse);
                            batched += 1;
                            let screen_y = ps2float::mul(y, inverse);
                            let depth = ps2float::mul(z, inverse);
                            let across = ps2float::mul(lw(window + 16) as f32, screen_x);
                            swc1(sp + 16, screen_x);
                            swc1(sp + 20, screen_y);
                            let size = size_f as i32;
                            swc1(sp + 24, depth);
                            let px = ps2float::add(centre_x, across);
                            let full = size as f32;
                            let py = ps2float::add(lw(window + 28) as f32, ps2float::mul(down as f32, screen_y));
                            swc1(sp, px);
                            let half = ((size + ((size as u32) >> 31) as i32) >> 1) as f32;
                            let right = ps2float::add(px, full);
                            swc1(sp + 4, py);
                            let z_out = ps2float::add(ps2float::mul(depth, scale), 32768.0_f32);
                            let bottom = ps2float::add(py, half);
                            let left = ps2float::sub(px, full);
                            swc1(sp + 80, right);
                            let top = ps2float::sub(py, half);
                            swc1(sp + 88, z_out);
                            swc1(sp + 84, bottom);
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
                            let first = colour | (ps2float::mul(fade, 127.0_f32) as i32 & 255) << 24;
                            let tag = write_quad_packet(sp, first, [13, 13]);
                            if batched == 23 {
                                header = close_full_batch(header, tag + 16);
                                batched = 0;
                            }
                        }
                    }
                }
                count = lw(particle + 24);
            }
            i += 1;
            if i >= count {
                break;
            }
        }
    }
    if batched != 0 {
        close_last_batch(header, batched);
    }
}

/// Draw the group of sparks `particle` (as `particleGunSparkGroupGfx`, with gravity of 2.5 and the constant at 0x3ad510 for the life) with the colour `colour` through the matrix
/// `matrix`, `delta` ago to now, as quads of the width `width`, 23 to a batch (@ 0x29c078).
pub fn particleSparkGroupGfx(particle: i32, colour: i32, matrix: i32, delta: f32, width: f32) {
    let frame = Frame::new(336);
    let sp = frame.sp();
    let life = lwc1(0x3ad510);
    let mut i = 0;
    let mut batched = 0;
    let mut header = 0;
    let gravity = -2.5_f32;
    let mut age = lwc1(particle + 12);
    sw(sp + 128, colour);
    let later = ps2float::add(age, delta);
    let count = lw(particle + 24);
    let drop = ps2float::mul(ps2float::mul(age, gravity), age);
    sw(sp + 132, ps2float::mul((ps2float::sub(life, age)).pdiv(life), 127.0_f32) as i32 & 255);
    if count > 0 {
        sw(sp + 136, particle + 940);
        sw(sp + 140, particle + 944);
        loop {
            let y = lwc1(particle + 932);
            let x = lwc1(particle + 928);
            let z = lwc1(particle + 936);
            let vy = lwc1(lw(sp + 140) + i * 12);
            let at = particle + 948 + i * 12;
            let vz = lwc1(at);
            let vx = lwc1(lw(sp + 136) + i * 12);
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
                                        let (mut x1, mut y1, mut z1, mut w1) = (x1, y1, z1, w1);
                                        let (mut x2, mut y2, mut z2, mut w2) = (x2, y2, z2, w2);
                                        if batched == 0 {
                                            header = begin_batch();
                                            w1 = lwc1(sp + 44);
                                            x1 = lwc1(sp + 32);
                                            y1 = lwc1(sp + 36);
                                            z1 = lwc1(sp + 40);
                                            w2 = lwc1(sp + 60);
                                            x2 = lwc1(sp + 48);
                                            y2 = lwc1(sp + 52);
                                            z2 = lwc1(sp + 56);
                                        }
                                        let one = 1.0_f32;
                                        let window = g::window;
                                        let depth_scale = lwc1(0x3ad514);
                                        let inverse2 = (one).pdiv(w2);
                                        let inverse1 = (one).pdiv(w1);
                                        let across = lw(window + 16) as f32;
                                        let down = (0 - lw(window + 36)) as f32;
                                        let half = 0.5_f32;
                                        let centre_x = lw(window + 8) as f32;
                                        let centre_y = lw(window + 28) as f32;
                                        batched += 1;
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
                                        let packet = memdbAlloc(144);
                                        let first = lw(sp + 128) | lw(sp + 132) << 24;
                                        sb(lw(g::gfxtag) + 3, 48);
                                        sw(packet, first);
                                        let tag = lw(g::gfxtag);
                                        sw(tag + 4, packet);
                                        let tail = tag + 16;
                                        let shared = lwc1(sp + 72) as i32;
                                        let words: [i32; 33] = [
                                            0x3f800000,
                                            1,
                                            0,
                                            0,
                                            0,
                                            3,
                                            0,
                                            packed_corner(lwc1(sp + 64), lwc1(sp + 68)),
                                            shared,
                                            13,
                                            0,
                                            0x4000000,
                                            0,
                                            3,
                                            0,
                                            packed_corner(lwc1(sp + 96), lwc1(sp + 100)),
                                            shared,
                                            13,
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
    if batched != 0 {
        close_last_batch(header, batched);
    }
}
