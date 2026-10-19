//! The display list packet that draws a particle sprite.

use crate::*;

/// The word of a vertex of the packet: the x and y of a point (floats) in 1/16 units of the screen offset, packed in two halves.
fn vertex_word(x: f32, y: f32) -> i32 {
    ((x as i32) << 4) + 0x6c00 & 0xffff | (((y as i32) << 4) + 0x7900) << 16
}

/// Put a sprite of a particle in the display list: a packet of 176 bytes (a GIF tag and four vertices of a textured quad): the texture setting `a2`, the corners in
/// four points of three floats (`corner_a`, `corner_b`, `corner_c`, `corner_d`; the flipped `x` and `y` of the sprite by the bits of `flip_x` and `flip_y` of the
/// second number `a1` and the first `a0`) (@ 0x2a0f68).
pub fn particleRender(a0: i32, a1: i32, a2: i32, corner_a: i32, corner_b: i32, corner_c: i32, corner_d: i32) {
    let packet = memdbAlloc(176);
    sb(lw(g::gfxtag) + 3, 48);
    sw(packet, 0x6c0a8000);
    let tag = lw(g::gfxtag);
    sw(tag + 4, packet);
    let mut at = packet + 4;
    let mut put = |value: i32| {
        sw(at, value);
        at += 4;
    };
    put(0x8001);
    let f1 = lwc1(corner_a);
    put(-0x6f51c000);
    let f0 = lwc1(corner_a + 4);
    put(-0x11111112);
    put(14);
    put(a2);
    put(0x3f800000);
    let f1_2 = lwc1(corner_c);
    put(1);
    put(0);
    let f0_2 = lwc1(corner_c + 4);
    put(0);
    let f2 = lwc1(corner_a + 8);
    put(0);
    let s1 = a1 << 20;
    put(3);
    put(0);
    put(vertex_word(f1, f0));
    put(f2 as i32);
    put(5);
    sw(g::gfxtag, tag + 16);
    put(0);
    let s3 = a0 << 4;
    let f0_3 = lwc1(corner_c + 8);
    put(s1);
    sh(tag, 11);
    put(0);
    put(3);
    put(0);
    put(vertex_word(f1_2, f0_2));
    put(f0_3 as i32);
    let f1_3 = lwc1(corner_b);
    put(5);
    let f0_4 = lwc1(corner_b + 4);
    put(0);
    put(s3);
    put(0);
    put(3);
    put(0);
    put(vertex_word(f1_3, f0_4));
    let f1_4 = lwc1(corner_d);
    put(lwc1(corner_b + 8) as i32);
    let f2_2 = lwc1(corner_d + 4);
    put(5);
    put(0);
    put(s3 | s1);
    put(0);
    put(3);
    put(0);
    let f0_5 = lwc1(corner_d + 8);
    put(vertex_word(f1_4, f2_2));
    put(f0_5 as i32);
    put(5);
    put(0);
    put(0x140007fc);
    put(0x11000000);
    put(0);
}
