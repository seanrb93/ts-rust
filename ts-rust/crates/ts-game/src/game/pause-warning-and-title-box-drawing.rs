//! Small pieces of the menus and the HUD that draw a text with a box or an arrow, and the width of a menu.

use crate::*;

/// Draw the word "paused" (the text at +0x1184 of the current language) in the middle of the window, 10 above its middle in y, with an outline; the font is the small one when
/// more than one player is local, else the next size (@ 0x211018).
pub fn drawPause() {
    let players = ilinkGetNumLocalPlayers();
    let old = textSelect(if 1 < players { 0 } else { 2 });
    let text = lw(lw((lw(g::cur_language) << 2) + g::languages) + 0x1184);
    let width = textWidth(text);
    let window = g::window;
    let y = lw(window + 28) - 10;
    let x = lw(window + 8) - ((((width as u32) >> 31) as i32 + width) >> 1);
    textOutline(x, y, text, 0x40787880, 0x5080);
    textSelect(old);
}

/// Draw a warning: the warning picture (image 36) at (`x`, `y` + 2) and the text `text` after it, both in a colour whose alpha is `flashalpha` (it flashes) (@ 0x2361c0).
pub fn drawWarning(x: i32, y: i32, text: i32) {
    let alpha = lbu(g::flashalpha);
    let colour = alpha << 24 | alpha << 16 | 128;
    let old = textSelect(1);
    guiDrawImage(36, colour, x as f32, (y + 2) as f32);
    textOutline(x + 16, y, text, colour, 128);
    textSelect(old);
}

/// Draw the two arrows (images 20 and 21) at the sides of the character select's name at height `y` + 3: they stand as far from the middle of the window as half of the width
/// of the text `text` (or half of 400 less the window's shift, when there is no text or it is narrower) (@ 0x236e60).
pub fn drawSelectCharArrows(y: i32, text: i32) {
    let mut width = 400 >> (lw(g::window + 88) & 31);
    if text != 0 && width < textWidth(text) {
        width = textWidth(text);
    }
    let window = g::window;
    let half = (width + ((width as u32) >> 31) as i32) >> 1;
    let top = (y + 3) as f32;
    guiDrawImage(20, 0x800080, (lw(window + 8) - half - 18) as f32, top);
    guiDrawImage(21, 0x800080, (lw(window + 8) + half) as f32, top);
}

/// Draw the two lines of the retry-or-quit box (the texts at +1780 and +1776 of the current language) in the middle of the window, the first at height `y` and the second one
/// line (half of it in the small font) and a bit below, with an outline of the menu's colours (@ 0x210770).
pub fn drawRetryQuit(y: i32) {
    let window = g::window;
    let old = textSelect(if lw(window + 92) == 0 { 2 } else { 1 });
    let first = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1780);
    let width = textWidth(first);
    let fg = lw(g::menuitem_fgcol);
    let bg = lw(g::menuitem_bgcol);
    textOutline(lw(window + 8) - ((((width as u32) >> 31) as i32 + width) >> 1), y, first, fg, bg);
    let gap = 2 >> (lw(window + 92) & 31);
    let line = if (lbu(g::fontflags) & 1) == 0 {
        lw(lw(g::curfont) + 12)
    } else {
        lw(lw(g::curfont) + 12) >> 1
    };
    let below = y + (line + gap);
    let second = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1776);
    let width = textWidth(second);
    let fg = lw(g::menuitem_fgcol);
    let bg = lw(g::menuitem_bgcol);
    textOutline(lw(g::window + 8) - ((((width as u32) >> 31) as i32 + width) >> 1), below, second, fg, bg);
    textSelect(old);
}

/// Draw a title box with the text `text`: from `x` across `width` (the width of the text and 8 when -1) and `height` down from `y`, filled in `colour`, with a gradient under the
/// first line when the box is taller than a line, and the text in the middle with an outline (the font is the next size, small flag set while it is measured) (@ 0x2b0fc0).
pub fn drawTitleBox(text: i32, x: i32, y: i32, width: i32, height: i32, colour: i32) {
    let frame = Frame::new(16);
    let sp = frame.sp();
    let mut width = width;
    let old = textSelect(2);
    sw(sp, old);
    let flags = lbu(g::fontflags) | 1;
    sb(g::fontflags, flags);
    let mut flags = flags;
    if width == -1 {
        let measured = textWidth(text);
        flags = lbu(g::fontflags);
        width = measured + 8;
    }
    let right = x + width;
    sw(sp + 4, y + height - 1);
    let line = if (flags & 1) == 0 {
        lw(lw(g::curfont) + 12)
    } else {
        lw(lw(g::curfont) + 12) >> 1
    };
    let bottom = y + line + 1;
    sb(g::fontflags, flags & 254);
    dlFillRectangle(x, y, right, bottom - 1, colour);
    let mut flags = lbu(g::fontflags);
    let font = lw(g::curfont);
    let low = colour & 255;
    let short = if (flags & 1) == 0 { lw(font + 12) >= height } else { (lw(font + 12) >> 1) >= height };
    if !short {
        let top = low + 8;
        let end = lw(sp + 4);
        guiVGradient(x, bottom, right + 1, end, colour, ((top as u32) >> 2) as i32, ((top as u32) >> 1) as i32);
        flags = lbu(g::fontflags);
    }
    sb(g::fontflags, flags | 1);
    let measured = textWidth(text);
    let spare = width - measured;
    textOutline(x + ((spare + ((spare as u32) >> 31) as i32) >> 1), y + 1, text, low | 0x7f7f7f00, 32);
    textSelect(lw(sp));
    sb(g::fontflags, lbu(g::fontflags) & 254);
}

/// Draw the box of the message in `in_message` (nothing when it is 0) in the middle of the window, sized by the text, in the game's colours, with the text (@ 0x20d458).
pub fn menuTextBoxGfx() {
    let frame = Frame::new(32);
    let sp = frame.sp();
    let message = lw(g::in_message);
    if message == 0 {
        return;
    }
    textSize(sp, sp | 4, message);
    let width = lw(sp);
    let window = g::window;
    let height = lw(sp + 4);
    let x = lw(g::window) + ((lw(window + 12) - width - 12) >> 1);
    let y = lw(window + 20) + ((lw(window + 32) - height - 4) >> 1);
    sw(sp + 8, x);
    sw(sp + 12, y);
    guiDrawBox(x, y, x + width + 11, y + height + 3, 0x7f0070, 0x200060);
    let message = lw(g::in_message);
    let below = lw(sp + 12) + 2;
    sw(sp + 8, lw(sp + 8) + 6);
    sw(sp + 12, below);
    textPrint(sp | 8, sp | 12, message, 0x7f0070);
}

/// Work out the margins of the `count` menu items from `items` (records of 32 bytes: the text number or address at +0, the x at +4, the flags at +10) placed at `base`: the left
/// edge of the first item goes into the halfword at `out` and the right edge of the last into the one at `out` + 2; an item's width is its text's, 100 for none and at least
/// 300 for an item with flag 4 (@ 0x22d698).
pub fn std_menugetmargins(items: i32, count: i32, base: i32, out: i32) {
    let mut item = items;
    sh(out, 0x7fff);
    sh(out + 2, -0x8000);
    if count > 0 {
        let mut left = count;
        loop {
            let mut width = 100;
            let mut text = lw(item);
            let from = ((((base << 16) >> 16) + lhu(item + 4)) << 16) >> 16;
            if text != 0 {
                let language = lw(g::cur_language);
                if text < 1246 {
                    text = lw((text << 2) + lw((language << 2) + g::languages));
                }
                width = (textWidth(text) << 16) >> 16;
            }
            if (lhu(item + 10) & 4) != 0 {
                width = if !(299 < width) { 300 } else { width };
            }
            let to = ((from + width) << 16) >> 16;
            if from < lh(out) {
                sh(out, from);
            }
            if lh(out + 2) < to {
                sh(out + 2, to);
            }
            left -= 1;
            item += 32;
            if left == 0 {
                break;
            }
        }
    }
}

/// Make the text of the time `time` (in sixtieths of a second) in the buffer at 0x1fbb9c0: hours (up to 99) if any, minutes (when there are hours or minutes), seconds and tenths,
/// and return its address; the text of a zero time is the language's text at +1760 (@ 0x21cab0).
pub fn gameFormatTime(time: i32) -> i32 {
    if time == 0 {
        return lw(lw((lw(g::cur_language) << 2) + g::languages) + 1760);
    }
    let seconds = time / 60;
    let hours = seconds / 60 / 60;
    let hours = if hours >= 100 { 99 } else { hours };
    sb(0x1fbb9c0, 0);
    if hours > 0 {
        let text = vafmt(&[0x3ae570, hours as i64, seconds as i64]);
        strcat(0x1fbb9c0, text);
    }
    if !(hours == 0 && seconds / 60 <= 0) {
        let text = vafmt(&[0x3ae578, (seconds / 60 % 60) as i64, seconds as i64]);
        strcat(0x1fbb9c0, text);
    }
    let tenths = time % 60 / 6;
    let seconds_text = lw(lw((lw(g::cur_language) << 2) + g::languages) + 2828);
    let text = vafmt(&[0x3ae580, (seconds % 60) as i64, tenths as i64, seconds_text as i64]);
    strcat(0x1fbb9c0, text);
    0x1fbb9c0
}
