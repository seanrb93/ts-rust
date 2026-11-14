//! Drawing a text on the screen.

use crate::*;

/// Draw the text `text` in the current font with colour `colour` at the place held in `*x_at` and `*y_at` (which are updated to where the text ends). A newline goes back to the
/// start x and down a line; the text stops past x 640 or y 224. "^" and a digit draws a button picture (at the font's size, in white); a character above 127 is turned to the
/// 7-bit one, and a capital's accent (from the table `capsinfo`) is drawn over it in the accent font; the number fonts (flag 2) give "1" 4 less width and "/" 7 width with 1 less
/// before, "." and ":" 5. The dither matrix and the z buffer test are set for the drawing and put back (@ 0x2024f0).
pub fn textPrint(x_at: i32, y_at: i32, text: i32, colour: i32) {
    let text = button_text_or(text);
    let frame = Frame::new(128);
    let sp = frame.sp();
    let font = lw(g::curfont);
    let scale = text_scale();
    let sc = |v: i32| text_scaled(v, scale);
    let f16 = |v: i32| text_scaled16(v, scale);
    // (the place along the line in sixteenths of a pixel, in 64 bits: the pixel's place wraps as the original's 32-bit sum does)
    let w16 = |v: i32| -> i64 { if scale == 1000 { (v as i64) << 4 } else { text_scaled16(v, scale) as i64 } };
    let mut cursor = text;
    sw(sp + 16, x_at);
    sw(sp + 20, y_at);
    let flags = lbu(g::fontflags);
    let glyphs = lw(font);
    sw(sp + 24, colour);
    let shift = (flags & 1) & 31;
    // (a text of a size that is not the font's: the glyphs are drawn smaller or bigger about the middle of their line)
    let line_height = lw(font + 12) >> shift;
    let centre_lift = (line_height - sc(line_height)) >> 1;
    let centre_lift16 = ((line_height << 4) - f16(line_height)) >> 1;
    sw(sp + 32, glyphs);
    let mut x = lw(x_at);
    let mut x16: i64 = (x as i64) << 4;
    sw(sp + 28, lw(y_at));
    if cursor == 0 {
        return;
    }
    if (flags & 8) == 0 {
        let old = dlSetDitherMatrix(2);
        sw(sp + 40, old);
    }
    let old_zb = dlSetZB(0);
    sw(sp + 36, old_zb);
    dlSelectTextureKick(lw(lw(g::curfont) + 16));
    let mut ch = lbu(cursor);
    // (the text after a button picture is what the button does: the mouse can click it)
    let mut prompt: Option<(u8, i32, i32)> = None;
    let mut end_x_at = lw(sp + 16);
    if ch != 0 {
        loop {
            if ch != 10 {
                let y = lw(sp + 28);
                if !(x < 640) {
                    break;
                }
                if y >= 224 {
                    break;
                }
                if ch != 94 {
                    let code = if (ch & 128) == 0 { lbu(cursor) } else { lbu((ch & 127) + g::ascii8to7) };
                    let glyph = lw(sp + 32) + (code * 10 - 320);
                    let advance;
                    if code == 32 {
                        if (ch & 128) == 0 {
                            let current = lw(g::curfont);
                            if (lbu(g::fontflags) & 2) != 0 {
                                advance = lw(current + 20) >> shift;
                            } else {
                                advance = lhu(glyph + 4) >> shift;
                            }
                        } else {
                            advance = lhu(lw(sp + 32) + lbu(((lbu((ch & 127) + g::asciicaps) - 1) << 2) + (g::capsinfo + 3)) * 10 - 316) >> shift;
                        }
                    } else {
                        let width;
                        if (lbu(g::fontflags) & 2) == 0 {
                            let offset = GetXOffset(cursor);
                            width = lhu(glyph + 4);
                            advance = (width + offset) >> shift;
                        } else if code == 49 {
                            x16 += w16(4);
                            x = (x16 >> 4) as i32;
                            width = lhu(glyph + 4);
                            advance = (lw(lw(g::curfont) + 20) >> shift) - 4;
                        } else if code >= 50 {
                            let current = lw(g::curfont);
                            if code != 58 {
                                width = lhu(glyph + 4);
                                advance = lw(current + 20) >> shift;
                            } else {
                                width = lhu(glyph + 4);
                                advance = 5;
                            }
                        } else if code != 46 {
                            let current = lw(g::curfont);
                            if code == 47 {
                                x16 -= w16(1);
                                x = (x16 >> 4) as i32;
                                width = lhu(glyph + 4);
                                advance = 7;
                            } else {
                                width = lhu(glyph + 4);
                                advance = lw(current + 20) >> shift;
                            }
                        } else {
                            width = lhu(glyph + 4);
                            advance = 5;
                        }
                        let u = lhu(glyph);
                        let v = lhu(glyph + 2);
                        let height = lhu(glyph + 6);
                        let drop = lb(glyph + 8);
                        let top = lw(sp + 28);
                        sw(sp, lw(sp + 24));
                        dlTextureRectangleFine(
                            x16 as i32,
                            (top << 4) + centre_lift16 + f16(drop >> shift),
                            (x16 + w16(width >> shift)) as i32,
                            (top << 4) + centre_lift16 + f16((drop + height) >> shift),
                            u << 4,
                            v << 4,
                            (u + width) << 4,
                            (v + height - 1) << 4,
                        );
                    }
                    if lw(g::cur_language) == 0 {
                        x16 += w16(advance);
                        x = (x16 >> 4) as i32;
                    } else {
                        let first = lbu(cursor);
                        if (first & 128) != 0 {
                            let accent = lbu((first & 127) + g::asciicaps);
                            if accent != 0 {
                                let current = lw(g::curfont);
                                let index = accent - 1;
                                let record = index << 2;
                                let accent_u = lbu(record + g::capsinfo) as u128;
                                let accent_v = if lw(current + 12) >= 13 { 77 } else { 61 };
                                let lift = if ((code - 65) as u32) < 26 { -1 } else { 1 >> shift };
                                let table = 3250720_u128;
                                let record_at = record + (table as i32 + 1);
                                let mut accent_w = lbu(record_at);
                                let lift = if 1_u32 >= ((index - 5) as u32) { 2 } else { lift };
                                let spare = sc(advance) - sc(accent_w);
                                let accent_texture = lw(0x3afe18);
                                let mut accent_u = accent_u;
                                let mut table = table;
                                if lw(current + 16) != accent_texture {
                                    sq(sp + 48, accent_u);
                                    sq(sp + 64, table);
                                    dlSelectTextureKick(accent_texture);
                                    accent_w = lbu(record_at);
                                    table = lq(sp + 64);
                                    accent_u = lq(sp + 48);
                                }
                                let accent_h = lbu(table as i32 + record + 2);
                                let right_u = accent_u as i32 + accent_w;
                                let left = x + ((spare + ((spare as u32) >> 31) as i32) >> 1);
                                let top = lw(sp + 28) + lift + centre_lift;
                                sw(sp, lw(sp + 24));
                                dlTextureRectangle(
                                    left,
                                    top,
                                    left + sc(accent_w >> shift),
                                    top + sc(accent_h >> shift),
                                    (accent_u as i32) << 4,
                                    accent_v << 4,
                                    right_u << 4,
                                    (accent_v + accent_h) << 4,
                                );
                                let back = lw(lw(g::curfont) + 16);
                                if back != lw(0x3afe18) {
                                    dlSelectTextureKick(back);
                                }
                            }
                        }
                        x16 += w16(advance);
                        x = (x16 >> 4) as i32;
                    }
                } else {
                    cursor += 1;
                    #[cfg(feature = "extensions")]
                    {
                        let height = line_height;
                        if let Some((kind, x0, y0)) = prompt.take() {
                            zone_note(kind, x0, y0, x, y0 + height, 0);
                        }
                        prompt = match lbu(cursor) {
                            49 => Some((8, x, y)),
                            50 => Some((6, x, y)),
                            51 => Some((7, x, y)),
                            52 => Some((9, x, y)),
                            _ => None,
                        };
                    }
                    let picture = lw(((lbu(cursor) - 49) << 2) + g::texnum_text_gfx);
                    let top = if lw(lw(g::curfont) + 12) < 13 { y } else { y + 2 };
                    let entry = picture * 40 + lw(lw(g::thistexbuf) + 4);
                    let height = lhu(entry + 2);
                    let width = lhu(entry);
                    if (lbu(g::fontflags) & 4) != 0 {
                        x16 += w16(width >> shift);
                        x = (x16 >> 4) as i32;
                    } else {
                        let drawn = sc(width >> shift);
                        dlSelectTextureKick(picture);
                        sw(sp, 0x7f7f7f7f);
                        dlTextureRectangle(x, top + centre_lift, x + drawn, top + centre_lift + sc(height >> ((shift + 1) & 31)), 0, 0, width << 4, height << 4);
                        dlSelectTextureKick(lw(lw(g::curfont) + 16));
                        x16 += (drawn as i64) << 4;
                        x = (x16 >> 4) as i32;
                    }
                }
                cursor += 1;
            } else {
                #[cfg(feature = "extensions")]
                if let Some((kind, x0, y0)) = prompt.take() {
                    zone_note(kind, x0, y0, x, y0 + line_height, 0);
                }
                cursor += 1;
                x = lw(lw(sp + 16));
                x16 = (x as i64) << 4;
                sw(sp + 28, lw(sp + 28) + sc(lw(lw(g::curfont) + 12) >> shift));
            }
            ch = lbu(cursor);
            if ch == 0 {
                break;
            }
        }
        #[cfg(feature = "extensions")]
        if let Some((kind, x0, y0)) = prompt.take() {
            zone_note(kind, x0, y0, x, y0 + line_height, 0);
        }
        end_x_at = lw(sp + 16);
    }
    sw(end_x_at, x);
    sw(lw(sp + 20), lw(sp + 28));
    let mut old_zb = lw(sp + 36);
    if (lbu(g::fontflags) & 8) == 0 {
        dlSetDitherMatrix(lw(sp + 40));
        old_zb = lw(sp + 36);
    }
    dlSetZB(old_zb);
}
