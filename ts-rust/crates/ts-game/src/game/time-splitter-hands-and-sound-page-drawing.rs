//! The flashes of the time splitter's hands and the drawing of the sound test page.

use crate::*;

/// Draw the hands of the time splitter for the effects group `group`: between 1.5 and the constant at 0x3ad6b4 on its time (+48) one fading quad at the group's two points
/// (+24 and +36), and between the constants at 0x3ad6c0 and 0x3ad6c4 two flares (at the points raised by 0.25) put on the screen by the player's matrices and drawn as particles
/// when they are in view; blending and the z buffer are off while drawing and put back at the end (@ 0x2ab870).
pub fn specialfxTimeSplitterHandsGfx(group: i32) {
    let frame = Frame::new(432);
    let sp = frame.sp();
    let groups = g::fxgroups;
    let fade = lwc1(group * 92 + (groups + 52));
    let blend = dlSetBlend(2);
    let zbuffer = dlSetZB(2);
    let record = group * 92 + groups;
    let time = lwc1(record + 48);
    if 1.5_f32 <= time && time < lwc1(0x3ad6b4) {
        let rise = lwc1(0x3ad6b8);
        let past = (ps2float::sub(time, 1.5_f32)).pdiv(lwc1(0x3ad6bc));
        let y0 = lwc1(record + 28);
        let y1 = lwc1(record + 40);
        let x0 = lwc1(record + 24);
        let x1 = lwc1(record + 36);
        let z0 = lwc1(record + 32);
        let player = lw(g::currentplayer);
        let texture = lw(0x3b0188);
        let z1 = lwc1(record + 44);
        swc1(sp + 20, ps2float::add(y0, rise));
        swc1(sp + 36, ps2float::add(y1, rise));
        swc1(sp + 48, x0);
        swc1(sp + 52, ps2float::add(y0, 0.0_f32));
        swc1(sp + 56, z0);
        swc1(sp + 64, x1);
        swc1(sp + 68, ps2float::add(y1, 0.0_f32));
        swc1(sp + 72, z1);
        let colour = (ps2float::mul(ps2float::sub(1.0_f32, ps2float::mul(past, past)), 9e+01_f32) as i32) << 24 | 0x7f7f7f;
        swc1(sp + 16, x0);
        sw(sp, colour);
        let matrix = lw(player + 1764);
        swc1(sp + 24, z0);
        swc1(sp + 32, x1);
        swc1(sp + 40, z1);
        quadRenderAligned(texture, matrix, sp + 16, 128, 64, colour, colour, colour, fade, 0.0_f32);
    }
    let start = lwc1(0x3ad6c0);
    let time = lwc1(group * 92 + (groups + 48));
    if start <= time {
        let player = lw(g::currentplayer);
        if time < lwc1(0x3ad6c4) {
            let projection = lw(player + 1764);
            let view = lw(player + 1760);
            let age = (ps2float::sub(time, start)).pdiv(lwc1(0x3ad6c8));
            let mut left = 1;
            sceVu0MulMatrix(sp + 80, view, projection);
            let record = group * 92 + g::fxgroups;
            let y0 = ps2float::add(lwc1(record + 28), 0.25_f32);
            let y1 = ps2float::add(lwc1(record + 40), 0.25_f32);
            let x1 = lwc1(record + 36);
            let z0 = lwc1(record + 32);
            let z1 = lwc1(record + 44);
            let texture = lw(0x3b018c);
            swc1(sp + 144, lwc1(record + 24));
            swc1(sp + 148, y0);
            swc1(sp + 152, z0);
            swc1(sp + 160, x1);
            swc1(sp + 164, y1);
            swc1(sp + 168, z1);
            dlSelectTextureKick(texture);
            let mut point = sp + 144;
            loop {
                matrixVec3Mul4Aligned(&[(sp + 80) as i64, point as i64, (sp + 176) as i64]);
                let w = lwc1(sp + 188);
                let x = lwc1(sp + 176);
                let low = -w;
                if low <= x {
                    let y = lwc1(sp + 180);
                    if x <= w && low <= y {
                        let z = lwc1(sp + 184);
                        if y <= w && low <= z {
                            let player = lw(g::currentplayer);
                            if z <= w {
                                let zoom = (6e+01_f32).pdiv(lwc1(player + 260));
                                let across = lw(g::window + 16) as f32;
                                let inverse = (1.0_f32).pdiv(w);
                                let centre_x = lw(g::window + 8) as f32;
                                let down = (0 - lw(g::window + 36)) as f32;
                                let centre_y = lw(g::window + 28) as f32;
                                let size = ps2float::mul(ps2float::mul(age.abs().psqrt(), 6e+02_f32), zoom).pdiv(z);
                                let depth = ps2float::mul(z, inverse);
                                let screen_x = ps2float::mul(x, inverse);
                                let screen_y = ps2float::mul(y, inverse);
                                let far = ps2float::mul(depth, lwc1(0x3ad6cc));
                                swc1(sp + 184, depth);
                                swc1(sp + 176, screen_x);
                                swc1(sp + 180, screen_y);
                                let px = ps2float::add(centre_x, ps2float::mul(across, screen_x));
                                let py = ps2float::add(centre_y, ps2float::mul(down, screen_y));
                                let half = ps2float::mul(size, 0.5_f32);
                                swc1(sp + 192, px);
                                swc1(sp + 196, py);
                                let right = ps2float::add(px, size);
                                let bottom = ps2float::add(py, half);
                                let z_out = ps2float::add(ps2float::add(far, 32768.0_f32), 1e+02_f32);
                                let left_x = ps2float::sub(px, size);
                                let colour = (ps2float::sub(127.0_f32, ps2float::mul(age, 127.0_f32)) as i32) << 24 | 0x7f3f3f;
                                let top = ps2float::sub(py, half);
                                swc1(sp + 256, right);
                                swc1(sp + 260, bottom);
                                swc1(sp + 240, left_x);
                                swc1(sp + 228, top);
                                swc1(sp + 264, z_out);
                                swc1(sp + 200, z_out);
                                swc1(sp + 208, left_x);
                                swc1(sp + 212, top);
                                swc1(sp + 216, z_out);
                                swc1(sp + 224, right);
                                swc1(sp + 232, z_out);
                                swc1(sp + 244, bottom);
                                swc1(sp + 248, z_out);
                                particleRender(128, 128, colour, sp + 208, sp + 224, sp + 240, sp + 256);
                            }
                        }
                    }
                }
                left -= 1;
                point += 16;
                if left < 0 {
                    break;
                }
            }
        }
    }
    dlSetBlend(blend);
    dlSetZB(zbuffer);
}

/// Draw the sound test page `page`: the background with the title made from the current sound's name, and, when the page has its box 5, the five envelope values (boxes 5 to 9), the
/// note of the sound's pitch (octave and step, with the octave offsets 0 and 3 as numbers with a sign) at box 10, the volume at box 11 and the pitch-bend per mille at box 4
/// (@ 0x207000).
pub fn main_pageDraw(page: i32) {
    let frame = Frame::new(160);
    let sp = frame.sp();
    dlSetBlend(1);
    let title = vafmt(&[0x399b88, lw(lw(g::snd) * 36 + g::sounds) as i64]);
    pagedrawbg(page, title);
    if frontGetBox(page, 5) == 0 {
        return;
    }
    let mut i = 0;
    let mut id = i + 5;
    loop {
        let item = frontGetBox(page, id & 0xffff);
        i += 1;
        adsrvalue_draw(item);
        id = i + 5;
        if i >= 5 {
            break;
        }
    }
    let note = getNote(lhu(lw(g::snd) * 36 + (g::sounds + 18)) as i32);
    let box10 = frontGetBox(page, 10);
    let y = lhu(box10 + 2);
    sw(sp, lhu(box10) + 224);
    sw(sp + 4, y);
    if note != 288 {
        let past = note - 288;
        let step = note - (((if -1 < note { note } else { note + 7 }) >> 3) << 3);
        let mut octave = (if -1 < past { past } else { note - 281 }) >> 3;
        if past < 0 {
            octave = if step != 0 { octave - 1 } else { octave };
        }
        let mut text = vafmt(&[0x399b98, step as i64]);
        let width = textWidth(text);
        sw(sp, lw(sp) - width);
        textPrint(sp, sp + 4, text, 0x64646480);
        if octave <= 0 {
            if octave < 0 {
                text = vafmt(&[0x3ae3c0, octave as i64]);
            }
        } else {
            text = vafmt(&[0x3ae3b8, octave as i64]);
        }
        if octave != 0 {
            let width = textWidth(text);
            sw(sp, lhu(box10) - (width - 324));
            textPrint(sp, sp + 4, text, 0x64646480);
        }
        let mut turns = past.wrapping_div(96);
        if past < 0 {
            turns = if note.wrapping_rem(96) != 0 { turns - 1 } else { turns };
        }
        if turns <= 0 {
            if turns < 0 {
                text = vafmt(&[0x3ae3d0, turns as i64]);
            }
        } else {
            text = vafmt(&[0x3ae3c8, turns as i64]);
        }
        if turns != 0 {
            let width = textWidth(text);
            sw(sp, lhu(box10) - (width - 424));
            textPrint(sp, sp + 4, text, 0x64646480);
        }
    } else {
        let width = textWidth(0x3ae3b0);
        sw(sp, lw(sp) - width);
        textPrint(sp, sp + 4, 0x3ae3b0, 0x64646480);
    }
    let box11 = frontGetBox(page, 11);
    let volume = lw(g::gv_volume);
    let text = vafmt(&[0x399ba8, volume as i64]);
    let x = lhu(box11) + lhu(box11 + 4) + 32;
    sw(sp + 4, lhu(box11 + 2));
    sw(sp, x);
    textPrint(sp, sp + 4, text, 0x64646480);
    let box4 = frontGetBox(page, 4);
    let record = lw(g::snd) * 36 + g::sounds;
    let per_mille = (lw(record + 32) * 1000).wrapping_div(0x5800);
    let text = vafmt(&[0x399bb8, per_mille as i64, record as i64, 1000]);
    let y = lhu(box4 + 2) + 20;
    sw(sp, lhu(box4));
    sw(sp + 4, y);
    textPrint(sp, sp + 4, text, 0x64646480);
}
