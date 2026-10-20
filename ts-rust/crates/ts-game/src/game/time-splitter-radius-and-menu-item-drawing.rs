//! The expanding ring of the time splitter and the drawing of a menu item's value.

use crate::*;

/// Draw the time splitter's ring for the effects group `group`: while its time (+48) is between the constants at 0x3ad690 and 0x3ad694 six curved quads (the arc 60 degrees apart,
/// fading with the time) are drawn about the group's place (+12) with blending and the z buffer off; the three blast rings (`specialfxBlastRadius`) at the group's place are always
/// drawn; the blending and the z buffer are put back (@ 0x2ab340).
pub fn specialfxTimeSplitterRadiusGfx(group: i32) {
    let frame = Frame::new(352);
    let sp = frame.sp();
    let groups = g::fxgroups;
    swc1(sp + 80, lwc1(group * 92 + (groups + 52)));
    let blend = dlSetBlend(2);
    sw(sp + 84, blend);
    let zbuffer = dlSetZB(2);
    let low = lwc1(0x3ad690);
    let time = lwc1(group * 92 + (groups + 48));
    if low <= time && time < lwc1(0x3ad694) {
        let table = g::g_SinTable;
        let half_time = (time - low) * 0.5_f32;
        let scale = lwc1(0x3ad698);
        let factor = lwc1(0x3ad69c);
        let spread = lwc1(0x3ad6a0);
        let root = half_time.abs().sqrt();
        let alpha = ((1.0_f32 - half_time * half_time) * 127.0_f32) as i32;
        swc1(sp + 88, half_time * 3.6e+02_f32 * 0.5_f32);
        let bright = alpha << 24 | 0x4f6a7f;
        let dark = ((alpha as f32 * 0.0_f32) as i32) << 24 | 0x4f6a7f;
        let mut i = 0;
        loop {
            let angle = i as f32 * 6e+01_f32 + lwc1(sp + 88);
            let half = 0x3fe0000000000000_i64;
            let index = dptoli(dpadd(dpmul(dpmul(fptodp(root), ld(0x3a9700)), ld(0x3a9708)), half));
            let swing = lwc1(((index & 2047) << 2) + table);
            let x = lwc1(group * 92 + (groups + 12));
            let z = lwc1(group * 92 + (groups + 20));
            let wide = swing * -1e+01_f32 + 25.0_f32;
            let y = lwc1(group * 92 + (groups + 16));
            let first = ((angle - 25.0_f32) * scale / 1.8e+02_f32 * factor + 0.5_f32) as i32;
            sw(sp, dark);
            let second = ((angle + 25.0_f32) * scale / 1.8e+02_f32 * factor + 0.5_f32) as i32;
            let matrix = lw(lw(g::currentplayer) + 1764);
            swc1(sp + 52, y);
            let top = y + swing * 2.5_f32;
            let first_cos = lwc1((((first + 512) & 2047) << 2) + table);
            let first_sin = lwc1(((first & 2047) << 2) + table);
            let second_sin = lwc1(((second & 2047) << 2) + table);
            let outer = root * 2.5_f32;
            let second_cos = lwc1((((second + 512) & 2047) << 2) + table);
            swc1(sp + 68, top);
            swc1(sp + 36, top);
            let third = ((angle + wide) * scale / 1.8e+02_f32 * factor + 0.5_f32) as i32;
            swc1(sp + 20, y);
            let fourth = ((angle - wide) * scale / 1.8e+02_f32 * factor + 0.5_f32) as i32;
            let texture = lw(0x3b0188);
            let fade = lwc1(sp + 80);
            let fourth_sin = lwc1(((fourth & 2047) << 2) + table) * outer;
            let fourth_cos = lwc1((((fourth + 512) & 2047) << 2) + table) * outer;
            let third_x = x + lwc1(((third & 2047) << 2) + table) * outer;
            let third_z = z + lwc1((((third + 512) & 2047) << 2) + table) * outer;
            swc1(sp + 16, x + first_sin * spread);
            swc1(sp + 24, z + first_cos * spread);
            swc1(sp + 64, third_x);
            swc1(sp + 72, third_z);
            i += 1;
            swc1(sp + 32, x + fourth_sin);
            swc1(sp + 40, z + fourth_cos);
            swc1(sp + 48, x + second_sin * spread);
            swc1(sp + 56, z + second_cos * spread);
            quadRenderAligned(texture, matrix, sp + 16, 128, 64, bright, dark, bright, fade, 0.0_f32);
            if i >= 6 {
                break;
            }
        }
    }
    let raise = lwc1(0x3ad6a4);
    let time_at = group * 92 + (groups + 48);
    let place = group * 92 + (groups + 12);
    specialfxBlastRadius(place, lwc1(time_at), lwc1(0x3ad6a8), 1.5_f32, raise);
    specialfxBlastRadius(place, lwc1(time_at), 1.0_f32, 1.5_f32, raise);
    specialfxBlastRadius(place, lwc1(time_at), lwc1(0x3ad6ac), lwc1(0x3ad6b0), raise);
    dlSetBlend(lw(sp + 84));
    dlSetZB(zbuffer);
}

/// The colour of an arrow of a menu item: dark when the item is not the highlighted one of the page, white when the page's flag `flag` (a button being held) is set, else flashing.
fn arrow_colour(item: i32, flag: i32) -> i32 {
    let front = lw(g::front);
    if item != lw((lw(front) << 2) + g::highlightitem) {
        0x400030
    } else if (lw(front + 12) & flag) != 0 {
        0x7f7f7f80
    } else {
        ((lbu(g::flashalpha) as u32) >> 1) as i32 + 64 | 0x800000
    }
}

/// Draw the value of the menu item record `item` (the entry at +16, its text at +20): the text of the item, then, when the entry has flag 4, either a slider (flag 8) or the text of
/// the value `gvGetStringValue` right-aligned (at +28 of the value, else by the window and the entry's flag 0x8000) with the two arrows (images 20 and 21) at the ends of the widest
/// of the value's choices (or the value's own text), unless flag 64 is set (@ 0x24bc00).
pub fn draw_mi(item: i32) {
    let frame = Frame::new(176);
    let sp = frame.sp();
    let entry = lw(item + 16);
    if lw(item + 20) != 0 {
        drawmenuitemtext(item);
    }
    let flags = lhu(entry + 10);
    if (flags & 4) == 0 {
        return;
    }
    let value = lw(entry + 12);
    if (flags & 8) != 0 {
        let low = lw(value + 4);
        let thumb = (((flags as u32) >> 6) as i32 ^ 1) & 1;
        let high = lw(value + 8) - low;
        let x = lhu(item) as i32;
        let y = lhu(item + 2) as i32 + 14;
        let width = lw(value) - low;
        draw_slider(x, y, width, high, thumb);
        slider_arrows(item, x, y);
        return;
    }
    let dim = flags & 64;
    let fg = if dim == 0 { lw(g::menuitem_fgcol) } else { 0x1e1e1e80 };
    let bg = if dim == 0 { lw(g::menuitem_bgcol) } else { 128 };
    sw(sp, 0);
    let text = gvGetStringValue(value);
    let right_at;
    if (lhu(entry + 10) & 256) == 0 {
        right_at = lw(value + 28);
    } else {
        let font = textSelect(lw(g::menuitem_bigfont));
        sw(sp, font);
        right_at = lw(value + 28);
    }
    let right;
    if right_at != 0 {
        right = right_at;
    } else {
        let window = lw(g::window);
        let x = lhu(item) as i32;
        if (lhu(entry + 10) & 0x8000) == 0 {
            right = x + 300 + window;
        } else {
            right = x + 252 + window;
        }
    }
    let y = lw(g::window + 20) + lhu(item + 2) as i32;
    let width = textWidth(text);
    textOutline(right - width, y, text, fg, bg);
    let flags_now = lhu(entry + 10);
    let mut big = flags_now & 256;
    if (flags_now & 64) == 0 {
        let mut matches = 0;
        let left;
        let extra = lw(value + 32);
        if extra != 0 {
            left = right + extra;
            sw(sp + 4, right + 16);
        } else {
            let mut widest = 8;
            let mut list = lw(value + 16);
            if list != 0 && lw(list + 4) != 0 {
                let mut n = 0;
                let mut offset = 0;
                sw(sp + 4, right + 16);
                loop {
                    n += 1;
                    let width = textWidth(lw(offset + list + 4));
                    list = lw(value + 16);
                    let key = lw(offset + list);
                    let current = lw(value);
                    offset = n << 3;
                    let following = lw(offset + list + 4);
                    widest = if widest < width { width } else { widest };
                    matches |= (((key ^ current) as u32) < 1) as i32;
                    if following == 0 {
                        break;
                    }
                }
            } else {
                sw(sp + 4, right + 16);
            }
            let mut edge = right - widest;
            if matches == 0 {
                let format = 0x3ae9b8;
                let own = vafmt(&[format, lw(value) as i64]);
                let own_width = textWidth(own);
                let used;
                if own_width < widest {
                    used = widest;
                } else {
                    let again = vafmt(&[format, lw(value) as i64]);
                    used = textWidth(again);
                }
                edge = right - used;
            }
            left = edge - 32;
        }
        let colour = arrow_colour(entry, 256);
        guiDrawImage(20, colour, left as f32, (lhu(item + 2) as i32 + 3) as f32);
        let colour = arrow_colour(entry, 512);
        let right_extra = lw(value + 36);
        let across = if right_extra == 0 { lw(sp + 4) } else { right + right_extra };
        guiDrawImage(21, colour, across as f32, (lhu(item + 2) as i32 + 3) as f32);
        big = lhu(entry + 10) & 256;
    }
    if big != 0 {
        textSelect(lw(sp));
    }
}
