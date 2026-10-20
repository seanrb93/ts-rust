//! Drawing the text of menu items, the loading screen, the bases bar of the HUD and a box with a value, and the tick of the blast shard effect.

use crate::*;

/// The text of a menu item `item`: the string number (below 1246, taken from the current language) or the address at +0 of the item, or the text a function makes from it
/// when it is not a text.
fn menu_item_text(item: i32) -> i32 {
    let mut text = lw(item);
    let is_text = istext(text);
    let mut numbered = (text < 1246) as i32;
    if is_text == 0 {
        text = gettextfromfunction(text, item);
        numbered = (text < 1246) as i32;
    }
    let language = lw(g::cur_language);
    if numbered != 0 {
        text = lw((text << 2) + lw((language << 2) + g::languages));
    }
    text
}

/// Draw the text of the menu item `item` at its place (x at +6, y at +14 from the window): in green when it is the menu's current item (or the item it follows, with flag 2 at
/// +20), else in the plain colours (@ 0x20cf00).
pub fn menuitemGfx(item: i32) {
    let frame = Frame::new(16);
    let sp = frame.sp();
    let top = lw(g::window + 20);
    let up = lhu(item + 14);
    let menu = lw(g::curmenu);
    sw(sp, lw(g::window) + lhu(item + 6));
    let current = lw(menu + 8);
    sw(sp + 4, top + up);
    let colour = if item != current {
        if (lbu(item + 20) & 2) == 0 {
            0x7f0070
        } else if lw(item + 24) != current {
            0x7f0070
        } else {
            0x557f557f
        }
    } else {
        0x557f557f
    };
    let text = menu_item_text(item);
    textPrint(sp, sp | 4, text, colour);
}

/// Draw the text of the menu item box `item` (at +0, +2 from the window; the item at +16, its flags at +10, its text at +20) with an outline: grey for flag 128, dim for flags
/// 0x4040, flashing when it is the highlighted item of the front end, else the menu's colours; an item with the value 0x8000 also gets the cursor's text before it. The font is the
/// large one when flag 256 is set (@ 0x24ba78).
pub fn drawmenuitemtext(item: i32) {
    let frame = Frame::new(16);
    let sp = frame.sp();
    let entry = lw(item + 16);
    let mut font = lw(g::menuitem_bigfont);
    if (lhu(entry + 10) & 256) == 0 {
        font = lw(g::menuitem_normalfont);
    }
    let old = textSelect(font);
    let flags = lhu(entry + 10);
    let x = lw(g::window) + lhu(item);
    sw(sp + 4, lw(g::window + 20) + lhu(item + 2));
    sw(sp, x);
    let (colour, outline);
    if (flags & 128) == 0 {
        if (flags & 0x4040) == 0 {
            let mut fg = lw(g::menuitem_fgcol);
            if entry != lw((lw(lw(g::front)) << 2) + g::highlightitem) {
                outline = lw(g::menuitem_bgcol);
            } else {
                fg = -0x7f7f7f80;
                outline = (((lbu(g::menuitemflash) as u32) >> 1) as i32) << 16 | 0x5080;
            }
            colour = fg;
        } else {
            outline = 128;
            colour = 0x1e1e1e80;
        }
    } else {
        colour = -0x7f7fd780;
        outline = 0x20200a80;
    }
    textOutline(lw(sp), lw(sp + 4), lw(item + 20), colour, outline);
    if lhu(entry + 8) == 0x8000 {
        let cursor_colour = if (lhu(entry + 10) & 0x4000) == 0 { 0x7f7f7f80 } else { 0x3f3f3f80 };
        sw(sp, lw(sp) - ((36 >> (lw(g::window + 88) & 31)) - 6));
        textPrint(sp, sp | 4, g::cursor_savepos + 8, cursor_colour);
    }
    if (lhu(entry + 10) & 256) != 0 {
        textSelect(old);
    }
}

/// Draw the loading screen: the whole screen filled dark blue and the language's loading text (at +1456) in the middle with an outline; the blend, dither matrix and z test are
/// put back (@ 0x22ebb0).
pub fn guiLoadingGfx() {
    let frame = Frame::new(16);
    let sp = frame.sp();
    let blend = dlSetBlend(1);
    let dither = dlSetDitherMatrix(2);
    let zb = dlSetZB(0);
    let text = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1456);
    let font = textSelect(4);
    textSize(sp, sp | 4, text);
    let spare_x = 640 - lw(sp);
    let spare_y = 224 - lw(sp + 4);
    dlFillRectangle(0, 0, 639, 223, 0x2850);
    textOutline((spare_x + ((spare_x as u32) >> 31) as i32) >> 1, (spare_y + ((spare_y as u32) >> 31) as i32) >> 1, text, 0x40787880, 0x5080);
    textSelect(font);
    dlSetDitherMatrix(dither);
    dlSetBlend(blend);
    dlSetZB(zb);
}

/// Draw the bases of the HUD: for each base (up to the number of bases) a title box with its number (the text at +3236 of the language) and a bar under it, as wide as its
/// health (+532 of the base, from 0 to 1) in 50, going from green to red and flashing when low (@ 0x2b1180).
pub fn hudBaseGfx() {
    let window = g::window;
    let mut i = 0;
    let top = lw(window + 20);
    let x = lw(g::window) + (32 >> (lw(window + 88) & 31));
    let mut y = if lw(window + 92) != 0 { top + 38 } else { top + 50 };
    loop {
        let next = i + 1;
        let count = enemyGetNumBases();
        let index = i;
        if i >= count {
            break;
        }
        i = next;
        let base = enemyGetBase(index);
        let bar_top = y + 9;
        let text = lw(lw((lw(g::cur_language) << 2) + g::languages) + 3236);
        let lost = 128 - (lwc1(base + 532) * 128.0_f32) as i32;
        let positive = (-1 < lost * 50) as i32;
        let label = vafmt(&[0x3afba0, text as i64, next as i64, positive as i64]);
        let colour = lw(lw(lw(lw(g::currentplayer) + 384) + 352) + 20);
        drawTitleBox(label, x, y, 50, 16, colour);
        let bar_bottom = y + 16;
        let right = x + ((if positive != 0 { lost * 50 } else { lost * 50 + 127 }) >> 7);
        let bar_colour = (128 - lost) << 24 | ((lost >> 1) + 64 & 255) << 16 | 128;
        y += 17;
        if lost < 41 && (lw(g::tickframenum) & 16) == 0 {
            continue;
        }
        dlFillRectangle(x, bar_top, right, bar_bottom, bar_colour);
    }
}

/// The tick of the blast shard effect for the effects group `group`: its particle group (at +72) is moved to the group's place plus its speed times the time, with a sine of the
/// time added in y, and its rate set to 60 times what is left of the first second; the group is freed after 2 (@ 0x2ac370).
pub fn specialfxBlastShardTick(group: i32) {
    let frame = Frame::new(16);
    let sp = frame.sp();
    let groups = g::fxgroups;
    let slot = group * 92 + (groups + 72);
    let times = groups + 48;
    if lw(slot) != 0 {
        let record = groups + group * 92;
        let time_at = group * 92 + times;
        let time = lwc1(time_at);
        swc1(sp, lwc1(record + 12) + lwc1(record + 80) * time);
        let wide = fptodp(time);
        let turned = dpadd(dpmul(dpmul(wide, ld(0x3a9728)), ld(0x3a9730)), 0x3fe0000000000000_i64);
        let index = dptoli(turned);
        let record = groups + group * 92;
        let speed_z = lwc1(record + 88);
        let base_z = lwc1(record + 20);
        let particles = lw(slot);
        swc1(sp + 4, lwc1(group * 92 + (groups + 16)) + lwc1(group * 92 + (groups + 84)) * lwc1(((index & 2047) << 2) + g::g_SinTable));
        swc1(sp + 8, base_z + speed_z * lwc1(time_at));
        particleUpdatePos(particles, sp);
        let particles = lw(slot);
        particleUpdateRate(particles, (1.0_f32 - lwc1(time_at)) * 6e+01_f32);
    }
    if 2.0_f32 < lwc1(group * 92 + times) {
        specialfxFree(group);
    }
}

/// Draw the text of the menu item box `item` of the standard menu: like `drawmenuitemtext` but flashing by the flash alpha and with the value mark "0x3ae800" before an item with the
/// value 0x8000 (@ 0x22cf70).
pub fn std_drawmenuitemtext(item: i32) {
    let frame = Frame::new(16);
    let sp = frame.sp();
    let entry = lw(item + 16);
    let mut font = lw(g::menuitem_bigfont);
    if (lhu(entry + 10) & 256) == 0 {
        font = lw(g::menuitem_normalfont);
    }
    let old = textSelect(font);
    let window = lw(g::window);
    let flags = lhu(entry + 10);
    let across = lhu(item);
    sw(sp + 4, lw(g::window + 20) + lhu(item + 2));
    let mut x = window + across;
    let (colour, outline);
    if (flags & 128) == 0 {
        if (flags & 64) == 0 {
            let mut fg = lw(g::menuitem_fgcol);
            if entry != lw((lw(lw(g::front)) << 2) + g::highlightitem) {
                outline = lw(g::menuitem_bgcol);
            } else {
                outline = 0x5080;
                fg = lbu(g::flashalpha) << 24 | 0x787880;
            }
            colour = fg;
        } else {
            outline = 64;
            colour = 0x1e1e1e80;
        }
    } else {
        colour = -0x7f7fd780;
        outline = 0x20200a80;
    }
    let mut y = lw(sp + 4);
    if lhu(entry + 8) == 0x8000 {
        sw(sp, x);
        x += 32;
        textPrint(sp, sp | 4, 0x3ae800, 0x7f7f7f80);
        y = lw(sp + 4);
    }
    textOutline(x, y, lw(item + 20), colour, outline);
    if (lhu(entry + 10) & 256) != 0 {
        textSelect(old);
    }
}

/// Draw a box with a title and a value: the title box of `title` (at (`x`, `y`) of `width` by `height`, in `colour`) and under its first line the text `value` in the middle of
/// the rest of the box, with an outline in `value_colour`; the ninth argument (on the caller's stack) not 0 sets font flag 2 for the value (@ 0x2b1420).
pub fn drawValueBox(title: i32, value: i32, x: i32, y: i32, width: i32, height: i32, colour: i32, value_colour: i32) {
    let frame = Frame::new(176);
    let sp = frame.sp();
    let narrow = lw(sp + 176);
    sw(sp, value);
    sw(sp + 4, value_colour);
    let old = textSelect(3);
    let flags = lbu(g::fontflags) | 1;
    sw(sp + 8, old);
    sb(g::fontflags, flags);
    let line = if (flags & 1) == 0 {
        lw(lw(g::curfont) + 12)
    } else {
        lw(lw(g::curfont) + 12) >> 1
    };
    let below = y + line;
    sb(g::fontflags, flags & 254);
    drawTitleBox(title, x, y, width, height, colour);
    let current = lbu(g::fontflags);
    if narrow != 0 {
        sb(g::fontflags, current | 2);
    }
    let measured = textWidth(lw(sp));
    let rest = y + height - below;
    let line = if (lbu(g::fontflags) & 1) == 0 {
        lw(lw(g::curfont) + 12)
    } else {
        lw(lw(g::curfont) + 12) >> 1
    };
    let text_colour = lw(sp + 4);
    let text_x = x + ((width - measured) >> 1);
    let text = lw(sp);
    let text_y = below + ((rest - line) >> 1) + 2;
    textOutline(text_x, text_y, text, text_colour, text_colour & 255);
    let current = lbu(g::fontflags);
    if narrow != 0 {
        sb(g::fontflags, current & 253);
    }
    textSelect(lw(sp + 8));
}
