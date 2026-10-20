//! The credits' text, formatted text in the middle of the window, the escort-completed message, taking a bullet out of its lists, and the fonts' sizes.

use crate::*;

/// Draw the text of the credits page that is current: the lines of the credits table (records of 28 bytes at `creditsText`: font at +0, outline colour at +8, kind at +12, string
/// number at +16, colour at +4, and for a kind 0 line the gap down at +0) from `creditsTextIndex` to the next kind 4, each at the height going down from `creditsStartPos`, aligned
/// by its kind (1 left at 64, 2 in the middle, 3 right at 580) and moved in x by the wave of the page (@ 0x2b0300).
pub fn creditsTextGfx() {
    let frame = Frame::new(16);
    let sp = frame.sp();
    dlSetDitherMatrix(0);
    if lwc1(g::creditspagetime) < lwc1(g::creditswaittime) {
        return;
    }
    let first = lw(g::creditsTextIndex) * 28;
    let table = g::creditsText;
    sw(sp + 4, lwc1(g::creditsStartPos) as i32);
    if lw(table + first + 12) == 4 {
        return;
    }
    let mut offset = first;
    let colours = table + 4;
    let mut kind_at = offset + 12 + table;
    let mut colour_at = first + colours;
    let mut line = offset + table;
    let mut x_base = 0;
    loop {
        let y = lw(sp + 4);
        if lw(kind_at) != 0 {
            textSelect(lw(line));
            let kind = lw(line + 12);
            let text = lw((lh(line + 16) << 2) + lw((lw(g::cur_language) << 2) + g::languages));
            if kind == 2 {
                let width = textWidth(text);
                x_base = 320 - ((((width as u32) >> 31) as i32 + width) >> 1);
            } else if kind >= 3 {
                if kind == 3 {
                    x_base = 580 - textWidth(text);
                }
            } else if kind == 1 {
                x_base = 64;
            }
            let height = textHeight(text);
            let wave = creditsCalcTextWave(lw(sp + 4) + ((((height as u32) >> 31) as i32 + height) >> 1));
            let x = x_base + wave;
            let outline = lw(offset + (table + 8));
            sw(sp, x);
            if outline == 0 {
                textPrint(sp, sp + 4, text, lw(colour_at));
            } else {
                textOutline(x, lw(sp + 4), text, lw(offset + colours), outline);
            }
        } else {
            sw(sp + 4, y + lw(line));
        }
        kind_at += 28;
        colour_at += 28;
        offset += 28;
        line += 28;
        if lw(kind_at) == 4 {
            break;
        }
    }
}

/// Draw the lines of the text `text` (ended by newlines) one under the other in the middle of the window from the height `y`, in the font `font`: a line starting with "~" is
/// in the dim colours, one starting with "`" is moved down half a line first; the other lines have `colour` and `outline` (the second argument is not used) (@ 0x22e930).
pub fn guiDrawFormattedText(text: i32, _unused: i32, y: i32, font: i32, colour: i32, outline: i32) {
    let frame = Frame::new(16);
    let sp = frame.sp();
    let mut y = y;
    let mut index = 0;
    let old = textSelect(font);
    sw(sp, old);
    let mut at = text + index;
    loop {
        let newline = strchr(at, 10);
        let mut fg = colour;
        let mut bg = outline;
        if newline != 0 {
            sb(newline, 0);
        }
        let first = lbu(at);
        let current;
        if first != 126 {
            if first != 96 {
                current = lbu(at);
            } else {
                let current_font = lw(g::curfont);
                let line = if (lbu(g::fontflags) & 1) == 0 {
                    lw(current_font + 12)
                } else {
                    lw(current_font + 12) >> 1
                };
                let sign = ((lw(current_font + 12) as u32) >> 31) as i32;
                y += (line + sign) >> 1;
                index += 1;
                at = text + index;
                current = lbu(at);
            }
        } else {
            index += 1;
            fg = 0x40787880;
            bg = 0x5080;
            at = text + index;
            current = lbu(at);
        }
        if current != 0 {
            let width = textWidth(at);
            textOutline(lw(g::window + 8) - ((((width as u32) >> 31) as i32 + width) >> 1), y, at, fg, bg);
        }
        index = index + 1 + strlen(at);
        if (lbu(g::fontflags) & 1) == 0 {
            y += lw(lw(g::curfont) + 12);
        } else {
            y += lw(lw(g::curfont) + 12) >> 1;
        }
        if newline != 0 {
            sb(newline, 10);
        }
        let length = strlen(text);
        at = text + index;
        if (index as u32) >= (length as u32) {
            break;
        }
    }
    textSelect(lw(sp));
}

/// Draw the message of the escort mission being over: the text of the language (+1820 or, for the game flag 0x10000000 not set, +1824 in red) in the middle at the top, and, when
/// the escort is dead, a second line (+1832) under it; and for the first player the retry-or-quit box (@ 0x211aa8).
pub fn drawEscortCompleted() {
    let players = ilinkGetNumLocalPlayers();
    ilinkGetNumLocalPlayers();
    let old = textSelect(if 1 < players { 0 } else { 2 });
    let window = g::window;
    let top = lw(window + 20);
    let split = lw(window + 92);
    let base = if split != 0 { top + 20 } else { top + 38 };
    let y = if split != 0 { base + 5 } else { base + 10 };
    let (colour, outline, text);
    if (lw(g::curgameparms + 80) & 0x10000000) == 0 {
        colour = 0x7f202080;
        outline = 80;
        text = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1824);
    } else {
        colour = 0x40787880;
        outline = 0x5080;
        text = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1820);
    }
    let width = textWidth(text);
    textOutline(lw(window + 8) - ((((width as u32) >> 31) as i32 + width) >> 1), y, text, colour, outline);
    let line = lw(lw(g::curfont) + 12) >> (lbu(g::fontflags) & 1 & 31);
    let below = (if lw(window + 92) == 0 { y + 16 } else { y + 2 }) + line;
    let mut player = lw(g::currentplayer);
    if (lw(g::curgameparms + 80) & 0x10000000) == 0 {
        let language = lw(g::cur_language);
        if lwc1(lw(g::escort) + 520) <= 0.0_f32 {
            let dead = lw(lw((language << 2) + g::languages) + 1832);
            textSelect(1);
            let width = textWidth(dead);
            let fg = lw(g::menuitem_fgcol);
            let bg = lw(g::menuitem_bgcol);
            textOutline(lw(g::window + 8) - ((((width as u32) >> 31) as i32 + width) >> 1), below, dead, fg, bg);
        }
        player = lw(g::currentplayer);
    }
    if lw(player) == 0 {
        let window = g::window;
        let bottom = lw(window + 24);
        drawRetryQuit(if lw(window + 92) == 0 { bottom - 56 } else { bottom - 30 });
    }
    textSelect(old);
}

/// Take the bullet `index` (records of 276 bytes at the memory kept at 0x3b0138) out of its character's list (20 words from +0x11c0 of the character's record) and delete its
/// props (at +96, +144, +140), stop its two particle groups (+204, +208: their rate set to 0 and the group's owner cleared) and free the two others (+112, +116); then take it
/// out of the list of bullets (the previous at +4 and the next at +8; `list` says which of the two list heads, at 0x3b0148 and 0x3b0140, it is of) (@ 0x2892d0).
pub fn disconnect(index: i32, list: i32) {
    let record = lw(0x3b0138) + index * 276;
    let next = lw(record + 8);
    let owner = lw(record + 108);
    let previous = lw(record + 4);
    if owner >= 0 {
        let mut left = 19;
        let mut at = lw(g::chrinfo) + owner * 0x1210 + 0x11c0;
        loop {
            if lw(at) == record {
                sw(at, 0);
            }
            left -= 1;
            at += 4;
            if left < 0 {
                break;
            }
        }
    }
    for slot in [96, 144, 140] {
        let prop = lw(record + slot);
        if prop != 0 {
            propDelete(prop);
            sw(record + slot, 0);
        }
    }
    for slot in [204, 208] {
        let particles = lw(record + slot);
        if particles != 0 {
            particleUpdateRate(particles, 0.0_f32);
            let particles = lw(record + slot);
            sw(record + slot, 0);
            sw(particles + 20, 0);
        }
    }
    for slot in [112, 116] {
        let particles = lw(record + slot);
        if particles != 0 {
            particleFree(particles);
            sw(record + slot, 0);
        }
    }
    if previous < 0 {
        sw((list << 2) + 0x3b0148, next);
    } else {
        sw(previous * 276 + lw(0x3b0138) + 8, next);
        sw(record + 8, -1);
    }
    if next < 0 {
        sw((list << 2) + 0x3b0140, previous);
    } else {
        sw(next * 276 + lw(0x3b0138) + 4, previous);
        sw(record + 4, -1);
    }
}

/// Turn the corner coordinates of `count` font entries of 10 bytes at `table` into sizes: the width at +4 and the height at +6 become the corners minus the start plus 1.
fn fonts_to_sizes(table: i32, count: i32) {
    let mut at = table;
    let mut i = 0;
    while i < count {
        let height = lhu(at + 6) + 1 - lhu(at + 2);
        sh(at + 4, lhu(at + 4) + 1 - lhu(at));
        sh(at + 6, height);
        at += 10;
        i += 1;
    }
}

/// Turn the corners of the entries of the five fonts into sizes (once) and restart the text (@ 0x2020b0).
pub fn textReset() {
    if lbu(g::done_96) == 0 {
        let h09 = lw(g::h09fontentries);
        let h07 = lw(g::h07fontentries);
        let news = lw(g::newsfontentries);
        let square = lw(g::squarefontentries);
        let itckabel = lw(g::itckabelfontentries);
        fonts_to_sizes(g::h09font, h09);
        fonts_to_sizes(g::h07font, h07);
        fonts_to_sizes(g::newsfont, news);
        fonts_to_sizes(g::squarefont, square);
        fonts_to_sizes(g::itckabelfont, itckabel);
        sb(g::done_96, 1);
    }
    textRestart();
}
