//! A look-at matrix, and a textured rectangle of the display list: both take one argument on the caller's stack.

use crate::*;

/// Make the view matrix `m` that looks from the point (`eye_x`, `eye_y`, `eye_z`) along the direction (`dir_x`, `dir_y`, `dir_z`) with the up direction (`up_x`, `up_y`,
/// `up_z`); the last of the up direction is the ninth floating point argument, which the caller puts on its stack, at the place just above this function's own frame (@ 0x2b4f78).
pub fn matrixLook(m: i32, eye_x: f32, eye_y: f32, eye_z: f32, dir_x: f32, dir_y: f32, dir_z: f32, up_x: f32, up_y: f32) {
    let frame = Frame::new(128);
    let up_z = lwc1(frame.sp() + 128);
    let inv = (-1.0_f32).pdiv(ps2float::add(ps2float::add(ps2float::mul(dir_x, dir_x), ps2float::mul(dir_y, dir_y)), ps2float::mul(dir_z, dir_z)).abs().psqrt());
    let fz = ps2float::mul(dir_z, inv);
    let fx = ps2float::mul(dir_x, inv);
    let fy = ps2float::mul(dir_y, inv);
    let rx = ps2float::sub(ps2float::mul(up_y, fz), ps2float::mul(up_z, fy));
    let ry = ps2float::sub(ps2float::mul(up_z, fx), ps2float::mul(up_x, fz));
    let rz = ps2float::sub(ps2float::mul(up_x, fy), ps2float::mul(up_y, fx));
    let inv = (1.0_f32).pdiv(ps2float::add(ps2float::add(ps2float::mul(rx, rx), ps2float::mul(ry, ry)), ps2float::mul(rz, rz)).abs().psqrt());
    let rz = ps2float::mul(rz, inv);
    let rx = ps2float::mul(rx, inv);
    let ry = ps2float::mul(ry, inv);
    let ux = ps2float::sub(ps2float::mul(fz, rx), ps2float::mul(fx, rz));
    let uy = ps2float::sub(ps2float::mul(fy, rz), ps2float::mul(fz, ry));
    let uz = ps2float::sub(ps2float::mul(fx, ry), ps2float::mul(fy, rx));
    let inv = (1.0_f32).pdiv(ps2float::add(ps2float::add(ps2float::mul(uy, uy), ps2float::mul(ux, ux)), ps2float::mul(uz, uz)).abs().psqrt());
    swc1(m, rx);
    swc1(m + 16, ry);
    swc1(m + 32, rz);
    let uy = ps2float::mul(uy, inv);
    swc1(m + 60, 1.0_f32);
    let ux = ps2float::mul(ux, inv);
    swc1(m + 8, fx);
    let uz = ps2float::mul(uz, inv);
    swc1(m + 24, fy);
    swc1(m + 4, uy);
    swc1(m + 20, ux);
    swc1(m + 36, uz);
    swc1(m + 40, fz);
    sw(m + 12, 0);
    sw(m + 28, 0);
    sw(m + 44, 0);
    swc1(m + 56, -ps2float::add(ps2float::add(ps2float::mul(eye_x, fx), ps2float::mul(eye_y, fy)), ps2float::mul(eye_z, fz)));
    swc1(m + 52, -ps2float::add(ps2float::add(ps2float::mul(eye_x, uy), ps2float::mul(eye_y, ux)), ps2float::mul(eye_z, uz)));
    swc1(m + 48, -ps2float::add(ps2float::add(ps2float::mul(eye_x, rx), ps2float::mul(eye_y, ry)), ps2float::mul(eye_z, rz)));
}

/// Draw a rectangle of the current texture: from (`x0`, `y0`) to (`x1`, `y1`) on the screen (in 1/16 pixel steps from the screen's offset) and from the texture coordinates
/// (`u0`, `v0`) to (`u1`, `v1`) (as the number `a7` and the three others, with 8 added); the colour (0xaarrggbb) is the eighth argument on the caller's stack. The depth test is
/// turned off for it (@ 0x2b8c08).
pub fn dlTextureRectangle(x0: i32, y0: i32, x1: i32, y1: i32, u0: i32, v0: i32, u1: i32, v1: i32) {
    let frame = Frame::new(176);
    let colour = lw(frame.sp() + 176);
    let packet = memdbAlloc(80);
    let depth = dlSetZB(0);
    sb(lw(g::gfxtag) + 3, 48);
    let tag = lw(g::gfxtag);
    sw(tag + 4, g::texrect_begin);
    sw(g::gfxtag, tag + 16);
    sh(tag, 2);
    sb(tag + 16 + 3, 48);
    sw(packet, ((colour as u32) >> 24) as i32);
    let second = lw(g::gfxtag);
    sw(second + 4, packet);
    let mut at = packet + 4;
    sw(at, colour >> 16 & 255);
    at += 4;
    sw(at, colour >> 8 & 255);
    at += 4;
    sw(at, colour & 255);
    at += 4;
    sw(at, u0 + 8);
    at += 4;
    sh(second, 5);
    sw(at, v0 + 8);
    at += 4;
    sw(g::gfxtag, second + 16);
    sw(at, 0);
    at += 4;
    sw(at, 0);
    at += 4;
    sw(at, (x0 << 4) + 0x6c00);
    at += 4;
    sw(at, (y0 << 4) + 0x7900);
    at += 4;
    sw(at, -1);
    at += 4;
    sw(at, 0);
    at += 4;
    sw(at, u1 + 8);
    at += 4;
    sw(at, v1 + 8);
    at += 4;
    sw(at, 0);
    at += 4;
    sw(at, 0);
    at += 4;
    sw(at, (x1 << 4) + 0x6c00);
    at += 4;
    sw(at, (y1 << 4) + 0x7900);
    at += 4;
    sw(at, -1);
    sw(at + 4, 0);
    sb(second + 16 + 3, 48);
    let third = lw(g::gfxtag);
    sw(third + 4, g::texrect_end);
    sh(third, 1);
    sw(g::gfxtag, third + 16);
    dlSetZB(depth);
}
