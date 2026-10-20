//! The drawing of the string editor (the on-screen keyboard and the text being typed).

use crate::*;

/// Take half of `width` off, rounding the half towards 0, as the centred text is placed.
fn half_width(width: i32) -> i32 {
    (((width as u32) >> 31) as i32 + width) >> 1
}

/// The height of a line of the current font, halved when the font flag 1 is set.
fn line_height() -> i32 {
    if (lbu(g::fontflags) & 1) == 0 {
        lw(lw(g::curfont) + 12)
    } else {
        lw(lw(g::curfont) + 12) >> 1
    }
}

/// Draw the string editor `edit` (`active` is 0 when it is not the one being typed into): the box of the keyboard (the 55 keys in rows of 11, each 24 wide, the special keys named
/// from the language's texts and wider keys by `specialcharw`), the key under the cursor lit, the title bar, and the text being typed in a box under the keyboard, or, when the
/// editor has two lines or more, a box with the prompt and the text; the text cursor flashes (@ 0x22f0f0).
pub fn stringeditDraw(edit: i32, _unused: i32, active: i32) {
    let frame = Frame::new(224);
    let sp = frame.sp();
    let old_font = textSelect(1);
    sw(sp + 40, old_font);
    stringeditSet(edit);
    let textbox = lw(g::textbox);
    let left = lw(g::window) + lw(textbox);
    let height = lw(textbox + 12);
    let top = lw(g::window + 20) + lw(textbox + 4);
    sw(sp + 56, left + 8);
    let colour = if active == 0 { 0x20202080 } else { 0x7f7f7f80 };
    zoomboxDraw(left, top, left + lw(textbox + 8), top + height, 0x2880, colour);
    let textbox = lw(g::textbox);
    let row = lw(textbox + 44);
    let column = lw(textbox + 40);
    let key = lbu(lw(textbox + 28) + (row * 11 + column));
    let key_y = top + (row * 12 + 4);
    let start = lw(sp + 56);
    let key_x;
    let key_width;
    if key < 252 {
        key_width = 24;
        key_x = start + column * 24;
    } else {
        let special = (key - 252) << 2;
        key_x = lw(sp + 56) + lw(special + g::specialcharcol) * 24;
        key_width = lw(special + g::specialcharw) * 24;
    }
    let row_offset = lw(textbox + 44) * 12;
    let key_right = key_x + key_width;
    let column_offset = lw(textbox + 40) * 24;
    let flash = lbu(g::flashalpha);
    sw(sp + 44, key_right);
    sw(sp + 48, key_y + 11);
    let key_colour = flash | 0x8000;
    sw(sp + 36, top + (row_offset + 4));
    sw(sp + 32, left + (column_offset + 8));
    dlFillRectangle(key_x, key_y, key_right - 1, lw(sp + 48), key_colour);
    let mut across = 0;
    let mut label = 0;
    sw(sp + 32, left + 20);
    let raw_height = lw(lw(g::curfont) + 12);
    let sign = ((raw_height as u32) >> 31) as i32;
    let font_height = if (lbu(g::fontflags) & 1) == 0 { raw_height } else { raw_height >> 1 };
    sw(sp + 36, top + 10 - ((font_height + sign) >> 1));
    sb(sp + 1, 0);
    let languages = g::languages;
    sw(sp + 52, top - 14);
    let mut i = 0;
    let mut textbox_now = lw(g::textbox);
    loop {
        'key: {
            let character = lbu(lw(textbox_now + 28) + i);
            let x;
            if (character as u32) < 252 {
                sb(sp, character);
                let width = textWidth(sp);
                let y = lw(sp + 36);
                sw(sp + 16, lw(sp + 32) - half_width(width));
                sw(sp + 20, y);
                textPrint(sp + 16, sp + 20, sp, 0x64646480);
            } else {
                let special = ((character & 255) - 252) << 2;
                if across != lw(special + g::specialcharcol) {
                    x = lw(sp + 32);
                    let _ = x;
                    break 'key;
                }
                let spans = lw(special + g::specialcharw) * 12;
                let at = lw(sp + 32);
                sb(g::fontflags, lbu(g::fontflags) | 1);
                let keys = lw(textbox_now + 28);
                sw(sp + 16, at + (spans - 12));
                let which = lbu(keys + i);
                if which == 253 {
                    label = lw(lw((lw(g::cur_language) << 2) + languages) + 1972);
                } else if which >= 254 {
                    if which == 254 {
                        label = lw(lw((lw(g::cur_language) << 2) + languages) + 1968);
                    } else if which == 255 {
                        label = lw(lw((lw(g::cur_language) << 2) + languages) + 1984);
                    }
                } else if which == 252 {
                    if keys != g::chartab1 {
                        label = lw(lw((lw(g::cur_language) << 2) + languages) + 1980);
                    } else {
                        label = lw(lw((lw(g::cur_language) << 2) + languages) + 1976);
                    }
                }
                let width = textWidth(label);
                let y = lw(sp + 36) + 3;
                sw(sp + 16, lw(sp + 16) - half_width(width));
                sw(sp + 20, y);
                textPrint(sp + 16, sp + 20, label, 0x407f4080);
                sb(g::fontflags, lbu(g::fontflags) & 254);
            }
        }
        let current = lw(sp + 32);
        across += 1;
        sw(sp + 32, current + 24);
        if across == 11 {
            let y = lw(sp + 36);
            sw(sp + 32, current - 240);
            across = 0;
            sw(sp + 36, y + 12);
        }
        i += 1;
        if (i as u32) >= 55 {
            break;
        }
        textbox_now = lw(g::textbox);
    }
    let key_right = lw(sp + 44);
    if active != 0 {
        zoomboxDraw(key_x, key_y, key_right, lw(sp + 48), 0, 0x7f7f80);
    }
    let textbox = lw(g::textbox);
    let lines = lw(textbox + 16);
    let language;
    if lines >= 2 {
        let box_width = lw(textbox + 20);
        let lines_height = lines * line_height();
        textSize(sp + 24, sp + 28, lw(textbox + 48));
        let right = left + box_width + 32;
        let box_top = top + lw(lw(g::textbox) + 12);
        sw(sp + 32, left);
        sw(sp + 36, box_top + 8);
        let bottom = box_top + 16 + (lines_height + line_height());
        let colour = if active == 0 { 0x20202080 } else { 0x7f7f7f80 };
        zoomboxDraw(lw(sp + 32), lw(sp + 36), right, bottom, 0x2880, colour);
        let y = lw(sp + 36);
        let x = lw(sp + 32) + 16;
        let fg = lw(g::menuitem_fgcol);
        let text = lw(lw(g::textbox) + 56);
        let bg = lw(g::menuitem_bgcol);
        sw(sp + 36, y + 4);
        sw(sp + 32, x);
        textOutline(x, y + 3, text, fg, bg);
        let y = lw(sp + 36) + line_height();
        sw(sp + 36, y);
        let x = lw(sp + 32);
        dlFillRectangle(x, y, x + box_width - 1, y + lines_height - 1, 0x5080);
        textPrint(sp + 32, sp + 36, lw(lw(g::textbox) + 48), 0x40787880);
        let frame_number = lw(g::framenum);
        if active != 0 && (frame_number & 16) != 0 {
            textOutline(lw(sp + 32), lw(sp + 36), 0x3ae820, 0x40787880, 0x5080);
        }
        language = frame_number;
    } else {
        let box_width = lw(textbox + 20);
        let spare = lw(textbox + 8) - box_width;
        let y = top - 26 + lw(textbox + 12);
        sw(sp + 36, y);
        let x = left + ((spare + ((spare as u32) >> 31) as i32) >> 1);
        sw(sp + 32, x);
        dlFillRectangle(x, y, x + box_width, y + 10, 0x5080);
        let text = lw(lw(g::textbox) + 48);
        let mut text_width = 0;
        if text != 0 {
            text_width = textWidth(text);
        }
        let textbox_last = lw(g::textbox);
        if text_width != 0 {
            textOutline(lw(sp + 32), lw(sp + 36), lw(textbox_last + 48), 0x40787880, 0x5080);
        }
        let tick = lw(g::tickframenum);
        if active != 0 && (tick & 16) != 0 {
            textOutline(lw(sp + 32) + text_width, lw(sp + 36), 0x3ae820, 0x40787880, 0x5080);
        }
        language = lw(g::cur_language);
    }
    let fg = lw(g::menuitem_fgcol);
    let bg = lw(g::menuitem_bgcol);
    let y = lw(sp + 52) + lw(lw(g::textbox) + 12);
    let text = lw(lw((language << 2) + g::languages) + 1988);
    textOutline(lw(sp + 56), y, text, fg, bg);
    textSelect(lw(sp + 40));
    if active == 0 {
        let textbox = lw(g::textbox);
        let corner_x = left + lw(textbox + 8) - 1;
        let corner_y = top + lw(textbox + 12) - 1;
        dlFillRectangle(left, top, corner_x, corner_y, 0x2860);
    }
}
