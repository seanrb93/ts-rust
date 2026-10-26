//! The drawing of a decal on a wall or an object.

use crate::*;

/// Draw the decal `decal` (the number of corners at +200, the corners as 16-byte points from +0, the texture coordinates at +128, the texture at +204, the matrix number at +212 of
/// the object at +216, the opacity at +220 and the colour at +228) with the opacity `alpha`: the corners are put on the screen (by the object's matrix with the view's, or by the
/// view's alone) and the decal is left out when all of them are behind the near plane, or all off to one side, or when the first two corners are in the same place; else it
/// is sent as a packet of 3 words a corner (@ 0x2a5b20).
pub fn decalDraw(decal: i32, alpha: f32) {
    let frame = Frame::new(560);
    let sp = frame.sp();
    let mut skip = 1;
    let mut behind = 0;
    memset(sp + 128, 0, 12);
    memset(sp + 144, 0, 12);
    let object = lw(decal + 216);
    let player = lw(g::currentplayer);
    let view;
    let count;
    if object != 0 && lw(decal + 212) >= 0 {
        if lw(object + 140) != 2 {
            let turn = ps2float::mul(lwc1(object + 76), lwc1(0x3ad604)).pdiv(1.8e+02_f32);
            matrixTransRotY(sp + 224, lwc1(object + 48), lwc1(object + 52), lwc1(object + 56), turn);
        } else {
            let radians = lwc1(0x3ad600);
            let half_turn = 1.8e+02_f32;
            let z = lwc1(object + 56);
            let a = ps2float::mul(lwc1(object + 72), radians).pdiv(half_turn);
            let x = lwc1(object + 48);
            let b = ps2float::mul(lwc1(object + 76), radians).pdiv(half_turn);
            let c = ps2float::mul(lwc1(object + 88), radians).pdiv(half_turn);
            let y = lwc1(object + 52);
            matrixTransRotZXY(sp + 224, x, y, z, a, b, c);
        }
        let object_now = lw(decal + 216);
        let model = lw(object_now + 32);
        let matrix = lw(model + 4) + (lw(decal + 212) << 6);
        sceVu0MulMatrix(sp + 288, sp + 224, matrix);
        sceVu0MulMatrix(sp + 160, lw(lw(g::currentplayer) + 1768), sp + 288);
        view = sp + 160;
        count = lw(decal + 200);
    } else {
        view = lw(lw(g::currentplayer) + 1768);
        let _ = player;
        count = lw(decal + 200);
    }
    let mut i = 0;
    if count > 0 {
        let mut offset = i << 4;
        loop {
            let out = sp + offset;
            matrixVec3Mul4Aligned(&[view as i64, (decal + offset) as i64, out as i64]);
            let w = lwc1(sp + 12 + offset);
            let z = lwc1(sp + 8 + offset);
            let low = -w;
            if z < low || w < z {
                behind = 1;
                break;
            }
            let x = lwc1(out);
            if !(x < low) {
                if !(w < x) {
                    sw(sp + 132, 1);
                } else {
                    sw(sp + 136, 1);
                }
            } else {
                sw(sp + 128, 1);
            }
            let w = lwc1(sp + 12 + offset);
            let y = lwc1(sp + 4 + offset);
            if !(y < -w) {
                if !(w < y) {
                    sw(sp + 148, 1);
                } else {
                    sw(sp + 152, 1);
                }
            } else {
                sw(sp + 144, 1);
            }
            i += 1;
            offset = i << 4;
            if i >= lw(decal + 200) {
                break;
            }
        }
    }
    let mut all_sides = lw(sp + 132) != 0 || (lw(sp + 128) != 0 && lw(sp + 136) != 0);
    if all_sides {
        let across = lw(sp + 148) != 0 || (lw(sp + 144) != 0 && lw(sp + 152) != 0);
        all_sides = across;
        if all_sides && behind == 0 {
            skip = 0;
        }
    }
    if skip != 0 {
        return;
    }
    let total = lw(decal + 200);
    let mut i = 0;
    if total > 0 {
        let mut offset = i << 4;
        loop {
            let one = 1.0_f32;
            let q_at = sp + 352 + (i << 2);
            let window = g::window;
            let a2 = sp + offset;
            let inverse = (one).pdiv(lwc1(sp + 12 + offset));
            let across = lw(window + 16) as f32;
            let centre_x = lw(window + 8) as f32;
            let y_at = sp + 4 + offset;
            let down = (0 - lw(window + 36)) as f32;
            let z_at = sp + 8 + offset;
            let centre_y = lw(window + 28) as f32;
            let depth_scale = lwc1(0x3ad608);
            swc1(q_at, inverse);
            let scale = lwc1(0x3ad60c);
            swc1(a2, ps2float::add(centre_x, ps2float::mul(ps2float::mul(across, lwc1(a2)), inverse)));
            swc1(y_at, ps2float::add(centre_y, ps2float::mul(ps2float::mul(down, lwc1(y_at)), lwc1(q_at))));
            swc1(z_at, ps2float::mul(ps2float::add(ps2float::mul(ps2float::mul(lwc1(z_at), depth_scale), lwc1(q_at)), 32768.0_f32), scale));
            let mut next = i + 1;
            if i == 1 {
                let dx = ps2float::sub(lwc1(sp), lwc1(sp + 16));
                let mut check_y = true;
                let y0;
                if !(0.0_f32 <= dx) {
                    y0 = lwc1(sp + 4);
                    if !(-dx <= one) {
                        check_y = false;
                    }
                } else {
                    y0 = lwc1(sp + 4);
                    if !(dx <= one) {
                        check_y = false;
                    }
                }
                if check_y {
                    let dy = ps2float::sub(y0, lwc1(sp + 20));
                    if !(0.0_f32 <= dy) {
                        if -dy <= one {
                            skip = 1;
                            break;
                        }
                    } else if dy <= one {
                        skip = 1;
                        break;
                    }
                }
                next = i + 1;
            }
            i = next;
            offset = i << 4;
            if i >= total {
                break;
            }
        }
    }
    if skip != 0 {
        return;
    }
    let mut i = 0;
    let colour = lw(decal + 228) | (ps2float::mul(ps2float::mul(alpha, 127.0_f32), lwc1(decal + 220)) as i32) << 24;
    dlSelectTextureKick(lw(decal + 204));
    let n = lw(decal + 200);
    let packet = memdbAlloc(((n << 1) + n + 2) << 4);
    let n2 = lw(decal + 200);
    sb(lw(g::gfxtag) + 3, 48);
    let n3 = lw(decal + 200);
    let tag = lw(g::gfxtag);
    sw(tag + 4, packet);
    sw(packet, ((n3 << 1) + n3 + 1) << 16 | 0x6c008000);
    let mut put = packet + 4;
    sh(tag, (n2 << 1) + n2 + 2);
    let n4 = lw(decal + 200);
    sw(g::gfxtag, tag + 16);
    sw(put, n4 | 0x8000);
    put += 4;
    sw(put, 0x302e4000);
    put += 4;
    sw(put, 3822);
    put += 4;
    sw(put, 0);
    put += 4;
    if lw(decal + 200) > 0 {
        loop {
            sw(put, colour);
            let q_at = sp + 352 + (i << 2);
            put += 4;
            let by_eight = i << 3;
            swc1(put, lwc1(q_at));
            put += 4;
            let by_sixteen = i << 4;
            let q = lwc1(q_at);
            let u = lwc1(decal + 128 + by_eight);
            sw(put, 1);
            put += 4;
            sw(put, 0);
            put += 4;
            let sixteen = 16.0_f32;
            swc1(put, ps2float::mul(u, q));
            put += 4;
            i += 1;
            swc1(put, ps2float::mul(lwc1(decal + 132 + by_eight), lwc1(q_at)));
            put += 4;
            let y = lwc1(sp + 4 + by_sixteen);
            let x = ps2float::mul(lwc1(sp + by_sixteen), sixteen);
            sw(put, 2);
            put += 4;
            sw(put, 0);
            put += 4;
            let z = lwc1(sp + 8 + by_sixteen);
            sw(put, (x as i32 + 0x6c00 & 0xffff) | ((ps2float::mul(y, sixteen) as i32 + 0x7900) << 16));
            put += 4;
            swc1(put, f32::from_bits(z as i32 as u32));
            put += 4;
            sw(put, 5);
            put += 4;
            sw(put, 0);
            put += 4;
            if i >= lw(decal + 200) {
                break;
            }
        }
    }
    sw(put, 0x140007fc);
    sw(put + 4, 0x11000000);
    sw(put + 8, 0);
}
