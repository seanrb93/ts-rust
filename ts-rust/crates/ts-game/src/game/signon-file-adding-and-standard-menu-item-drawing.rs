//! Adding a save file to the sign-on list, and drawing the value of an item of the standard menu.

use crate::*;

/// The colour of an arrow of a menu item: dark when the item is not the highlighted one of the page, white when the page's flag `flag` (a button being held) is set, else flashing.
fn arrow_colour(entry: i32, flag: i32) -> i32 {
    let front = lw(g::front);
    if entry != lw((lw(front) << 2) + g::highlightitem) {
        0x400030
    } else if (lw(front + 12) & flag) != 0 {
        0x7f7f7f80
    } else {
        ((lbu(g::flashalpha) as u32) >> 1) as i32 + 64 | 0x800000
    }
}

/// Add the files of the sign-on data `data` (up to 16 records of 2936 bytes from +16, each starting with a name, the next one only while the name's first byte is not 0) to the
/// sign-on slots: where a slot of the same name and the same key (+4 of `data`) exists its number at `signoninfo` is set to `value`; otherwise a free slot (`signonGetFree`) gets a
/// copy of the record, the key and `value` (@ 0x2204f8).
pub fn signonAddFile(data: i32, value: i32) {
    let mut done = 0;
    let mut name = data + 16;
    signonCheckSlot(data, value);
    if lbu(data + 16) == 0 {
        return;
    }
    let info = g::signoninfo;
    let keys = info + 4;
    loop {
        let mut slot = 0;
        let count = signonCount();
        done += 1;
        let mut found = false;
        let mut at = 0;
        loop {
            if slot >= count {
                break;
            }
            let same = strcmp(lw(g::signonarray) + slot * 2936, name);
            if same != 0 {
                slot += 1;
            } else {
                at = slot << 3;
                slot += 1;
                if lw(at + keys) == lw(data + 4) {
                    found = true;
                    break;
                }
            }
        }
        if found {
            sw(at + info, value);
        } else {
            let free = signonGetFree();
            if free >= 0 {
                let mut to = free * 2936 + lw(g::signonarray);
                let mut from = name;
                let end = name + 2912;
                loop {
                    let b = ld(from + 8);
                    let c = ld(from + 16);
                    let d = ld(from + 24);
                    sd(to, ld(from));
                    sd(to + 8, b);
                    sd(to + 16, c);
                    sd(to + 24, d);
                    from += 32;
                    to += 32;
                    if from == end {
                        break;
                    }
                }
                let b = ld(from + 8);
                let c = ld(from + 16);
                sd(to, ld(from));
                sd(to + 8, b);
                sd(to + 16, c);
                sw((free << 3) + keys, lw(data + 4));
                sw((free << 3) + info, value);
            }
        }
        if done >= 16 {
            return;
        }
        name = data + (done * 2936 + 16);
        if lbu(name) == 0 {
            return;
        }
    }
}

/// Draw the value of the menu item record `item` of the standard menu (the entry at +16, its text at +20): the text of the item, then, when the entry has flag 4 (and not 512),
/// a slider (flag 8) or the text of the value right-aligned (by the value's own place at +28 or by the window, wide windows from 500 getting more room; 24 more on the weapons
/// page for the weapon set) with the two arrows (images 20 and 21) at the ends of the widest of the choices, unless flag 64 is set (@ 0x22d230).
pub fn draw_menuitem(item: i32) {
    let frame = Frame::new(192);
    let sp = frame.sp();
    let entry = lw(item + 16);
    let flags = lhu(entry + 10);
    if (flags & 512) != 0 {
        return;
    }
    let mut flags = flags;
    if lw(item + 20) != 0 {
        std_drawmenuitemtext(item);
        flags = lhu(entry + 10);
    }
    if (flags & 4) == 0 {
        return;
    }
    let value = lw(entry + 12);
    if (flags & 8) != 0 {
        let low = lw(value + 4);
        let thumb = (((flags as u32) >> 6) as i32 ^ 1) & 1;
        let high = lw(value + 8) - low;
        let x = pane_x(lhu(item) as i32);
        let y = pane_y(lhu(item + 2) as i32 + 14);
        let width = lw(value) - low;
        draw_slider(x, y, width, high, thumb);
        slider_arrows_hook(item, x, y);
        return;
    }
    let dim = flags & 64;
    let fg = if dim == 0 { lw(g::menuitem_fgcol) } else { 0x1e1e1e80 };
    let bg = if dim == 0 { lw(g::menuitem_bgcol) } else { 64 };
    sw(sp + 4, 0);
    let text = gvGetStringValue(value);
    sw(sp, text);
    let place;
    if (lhu(entry + 10) & 256) == 0 {
        place = value_place(lw(value + 28));
    } else {
        let font = textSelect(lw(g::menuitem_bigfont));
        sw(sp + 4, font);
        place = value_place(lw(value + 28));
    }
    let mut right = place;
    if place == 0 {
        let x = lhu(item) as i32;
        if (lhu(entry + 10) & 256) == 0 {
            let window = lw(g::window);
            right = if lw(g::window + 12) >= 500 { x + 384 + window } else { x + 256 + window };
        } else {
            let window = lw(g::window);
            right = if lw(g::window + 12) >= 500 { x + 448 + window } else { x + 320 + window };
        }
    }
    let right = weapons_set_shift(value, right);
    let window = g::window;
    let line = right;
    let y = lw(window + 20) + lhu(item + 2) as i32;
    let width = textWidth(lw(sp));
    textOutline(line - width, y, lw(sp), fg, bg);
    let flags_now = lhu(entry + 10);
    let mut big = flags_now & 256;
    if (flags_now & 64) == 0 {
        let mut matches = 0;
        let extra = lw(value + 32);
        let row_y = lw(window + 20) + lhu(item + 2) as i32;
        let row = if (flags_now & 256) != 0 { row_y + 4 } else { row_y + 2 };
        let left;
        let second_x;
        if extra != 0 {
            left = line + extra;
            second_x = right + 16;
        } else {
            let mut widest = 8;
            let list = lw(value + 16);
            if list != 0 {
                let mut n = 0;
                if lw(list + 4) != 0 {
                    let mut offset = 0;
                    loop {
                        n += 1;
                        let text = gvGetStringValue(value);
                        let width = textWidth(text);
                        let list_now = lw(value + 16);
                        let current = lw(value);
                        widest = if widest < width { width } else { widest };
                        let key = lw(offset + list_now);
                        offset = n << 3;
                        matches |= (((key ^ current) as u32) < 1) as i32;
                        if lw(offset + list_now + 4) == 0 {
                            break;
                        }
                    }
                }
            }
            let mut edge = right - widest;
            if matches == 0 {
                let format = 0x3ae808;
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
            second_x = line + 16;
        }
        let colour = arrow_colour(entry, 256);
        guiDrawImage(20, colour, left as f32, row as f32);
        let colour = arrow_colour(entry, 512);
        let right_extra = lw(value + 36);
        let across = if right_extra == 0 { second_x } else { right + right_extra };
        guiDrawImage(21, colour, across as f32, row as f32);
        big = lhu(entry + 10) & 256;
    }
    if big != 0 {
        textSelect(lw(sp + 4));
    }
}
