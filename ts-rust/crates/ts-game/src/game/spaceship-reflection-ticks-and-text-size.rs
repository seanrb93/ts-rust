//! The first spaceship's moving surfaces, the reflections, and the size of a text.

use crate::*;
use super::random_as_float;

/// The first spaceship prop: its position follows three sines of the game's time (the amount at +32 of its state scales them) and the sine of the third (kept at +88, times that
/// amount) is the level of a glow; when the sine falls below 0 and the next one (one frame of time later) rises above it the amount is given a new random value. The x of every
/// vertex of the first chunk list of the second part of the model moves by a sine of the game's time (@ 0x27ab20).
pub fn spaceship1PropTick(prop: i32) {
    let frame = Frame::new(48);
    let sp = frame.sp();
    let now = lw(g::gameframe);
    let time = gameframe_x();
    let k = lwc1(0x3acf28);
    let half = 0.5_f32;
    let table = g::g_SinTable;
    let state = lw(prop + 352);
    let k3 = lwc1(0x3acf34);
    let model = lw(lw(prop + 32));
    let k2 = lwc1(0x3acf38);
    let y_at = ((ps2float::add(ps2float::mul(ps2float::mul(time, lwc1(0x3acf2c)), k), half) as i32 & 2047) << 2) + table;
    let base_y = lwc1(prop + 40);
    let bones = model - lw(model) * 80;
    let later = ps2float::add(ps2float::mul(ps2float::mul(gameframe_before(now), k3), k), half) as i32;
    let base_z = lwc1(prop + 44);
    swc1(prop + 48, ps2float::add(lwc1(prop + 36), ps2float::mul(ps2float::mul(lwc1(((ps2float::add(ps2float::mul(ps2float::mul(time, lwc1(0x3acf24)), k), half) as i32 & 2047) << 2) + table), lwc1(0x3acf30)), lwc1(state + 32))));
    let z_at = ((ps2float::add(ps2float::mul(ps2float::mul(time, k3), k), half) as i32 & 2047) << 2) + table;
    swc1(prop + 52, ps2float::add(base_y, ps2float::mul(ps2float::mul(lwc1(y_at), k2), lwc1(state + 32))));
    swc1(prop + 56, ps2float::add(base_z, ps2float::mul(ps2float::mul(lwc1(z_at), 1.5_f32), lwc1(state + 32))));
    let glow = lwc1(z_at);
    swc1(prop + 88, glow);
    let next = lwc1(((later & 2047) << 2) + table);
    if glow <= 0.0_f32 && 0.0_f32 < next {
        swc1(state + 32, ps2float::mul(random_as_float(), 2.3283064e-10_f32));
    }
    let amount = lwc1(state + 32);
    swc1(prop + 88, ps2float::mul(lwc1(prop + 88), amount));
    allocateOverrideChunk(prop, 0, 1, 2);
    let out = getOverrideChunk(prop, 0, 1, 2, sp);
    if out != 0 {
        let count = lw(sp);
        let mut at = out;
        let mut source = lw(bones + 52);
        let mut i = 0;
        if count > 0 {
            let tick = lw(g::gameframe);
            loop {
                i += 1;
                swc1(at, ps2float::add(lwc1(source), lwc1(((ps2float::add(ps2float::mul(ps2float::mul(tick as f32, lwc1(0x3acf3c)), lwc1(0x3acf40)), half) as i32 & 2047) << 2) + g::g_SinTable)));
                swc1(at + 4, lwc1(source + 4));
                swc1(at + 8, lwc1(source + 8));
                let w = lwc1(source + 12);
                source += 16;
                swc1(at + 12, w);
                at += 16;
                if i >= count {
                    break;
                }
            }
        }
    }
}

/// The reflections prop: the x and z of every vertex of the first chunk list's override array move by sines of the game's time and of the vertex's y (@ 0x278ef8).
pub fn reflectionsPropTick(prop: i32) {
    let frame = Frame::new(48);
    let sp = frame.sp();
    let model = lw(lw(prop + 32));
    let bones = model - lw(model) * 80;
    allocateOverrideChunk(prop, 0, 1, 1);
    let out = getOverrideChunk(prop, 0, 1, 1, sp);
    if out != 0 {
        let count = lw(sp);
        let mut at = out;
        let mut source = lw(bones + 48);
        let mut i = 0;
        if count > 0 {
            let tick = lw(g::gameframe);
            loop {
                let table = g::g_SinTable;
                let eight = 8.0_f32;
                i += 1;
                let time = ps2float::mul(gameframe_tick_x(tick), lwc1(0x3ace98));
                let k = lwc1(0x3acea0);
                let half = 0.5_f32;
                let k_z = lwc1(0x3aceac);
                let time_z = ps2float::mul(time, lwc1(0x3acea8));
                swc1(at, ps2float::add(lwc1(source), ps2float::mul(lwc1(((ps2float::add(ps2float::mul(ps2float::add(ps2float::mul(lwc1(source + 4), eight), ps2float::mul(time, lwc1(0x3ace9c))), k), half) as i32 & 2047) << 2) + table), lwc1(0x3acea4))));
                swc1(at + 4, lwc1(source + 4));
                let z = lwc1(source + 8);
                let y8 = ps2float::mul(lwc1(source + 4), eight);
                source += 12;
                swc1(at + 8, ps2float::add(z, ps2float::mul(lwc1(((ps2float::add(ps2float::mul(ps2float::add(y8, time_z), k), half) as i32 & 2047) << 2) + table), k_z)));
                at += 12;
                if i >= count {
                    break;
                }
            }
        }
    }
}

/// The size of the text `text` in the current font (the width into `*width`, the height into `*height`; both 0 for no text): lines end at a newline; "^" and a digit is a button
/// picture; "`" moves down half a line; a character above 127 is turned to the 7-bit one (or a capital's letter); the width of a character is the font's, or in the number fonts
/// (flag 2) 6 for "/", 5 for "." and ":", else the font's number width (@ 0x202a88).
pub fn textSize(width: i32, height: i32, text: i32) {
    let text = button_text_or(text);
    let frame = Frame::new(64);
    let sp = frame.sp();
    let font = lw(g::curfont);
    let mut max_x = 0;
    let mut max_y = 0;
    let mut x = 0;
    let mut cursor = text;
    sw(sp, width);
    sw(sp + 4, height);
    let shift = lbu(g::fontflags) & 1;
    let mut base = lw(font) as u128;
    sw(sp + 8, lw(font + 12) >> (shift & 31));
    if cursor != 0 {
        let mut c = lbu(cursor);
        let line = lw(sp + 8);
        if c != 0 {
            let mut half_line = 0;
            let mut lines = 0;
            sw(sp + 12, line >> 1);
            let mut glyph = 0;
            loop {
                let ch = lbu(cursor);
                'next: {
                    if ch == 94 {
                        cursor += 1;
                        let picture = lw(((lbu(cursor) - 49) << 2) + g::texnum_text_gfx) * 40 + lw(lw(g::thistexbuf) + 4);
                        x += lhu(picture) >> (shift & 31);
                        let bottom = lines + half_line + ((lhu(picture + 2) >> 1) >> (shift & 31));
                        let right = x - 1;
                        max_y = if max_y < bottom { bottom } else { max_y };
                        max_x = if max_x < right { right } else { max_x };
                    } else {
                        if ch >= 95 {
                            if ch == 96 {
                                half_line += lw(sp + 12);
                                break 'next;
                            }
                        } else if ch == 10 {
                            x = 0;
                            lines += lw(sp + 8);
                            break 'next;
                        }
                        let code = if (c & 128) == 0 { ch } else { lbu((c & 127) + g::ascii8to7) };
                        'found: {
                            if (c & 128) != 0 && code == 32 {
                                glyph = base as i32 + (lbu(((lbu((c & 127) + g::asciicaps) - 1) << 2) + (g::capsinfo + 3)) * 10 - 320);
                                break 'found;
                            }
                            glyph = base as i32 + (code * 10 - 320);
                        }
                        if (lbu(g::fontflags) & 2) == 0 {
                            sq(sp + 16, base);
                            let offset = GetXOffset(cursor);
                            base = lq(sp + 16);
                            x += (lhu(glyph + 4) + offset) >> (shift & 31);
                        } else if ch == 47 {
                            x += 6;
                        } else if ch >= 48 {
                            if ch != 58 {
                                x += lw(lw(g::curfont) + 20) >> (shift & 31);
                            } else {
                                x += 5;
                            }
                        } else if ch != 46 {
                            x += lw(lw(g::curfont) + 20) >> (shift & 31);
                        } else {
                            x += 5;
                        }
                        let right = x - 1;
                        max_x = if max_x < right { right } else { max_x };
                        let bottom = lines + half_line + ((lb(glyph + 8) + lhu(glyph + 6)) >> (shift & 31));
                        max_y = if max_y < bottom { bottom } else { max_y };
                    }
                }
                cursor += 1;
                c = lbu(cursor);
                if c == 0 {
                    break;
                }
            }
        }
        sw(lw(sp), max_x);
        sw(lw(sp + 4), max_y);
    } else {
        sw(width, 0);
        sw(lw(sp + 4), 0);
    }
}
