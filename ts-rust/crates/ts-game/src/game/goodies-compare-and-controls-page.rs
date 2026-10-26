//! Comparing the goodies with the stored ones, the front end's wavy background, and the controls page.

use crate::*;

/// Whether the goodies have changed since they were stored (1) or not (0): the unlocked cheats, the levels, the bot sets, the misc flag and the 12-byte lists of the available and
/// the playable characters are each compared with the stored values (@ 0x240558).
pub fn frontmpgNewGoodies() -> i32 {
    let frame = Frame::new(48);
    let sp = frame.sp();
    if lw(0x3affa0) == lw(g::unlockedcheats) {
        let levels = goodiesGetLevels();
        if lw(0x3affa4) != levels {
            return 1;
        }
        let bots = goodiesGetBotSets();
        if lw(0x3affa8) != bots {
            return 1;
        }
        let misc = goodiesGetMisc();
        if lbu(0x3affac) != misc {
            return 1;
        }
        goodiesGetAvailableChrs(sp);
        let stored = 0x1fc33b0;
        if lw(sp) == lw(0x1fc33b0) {
            if lw(sp + 4) != lw(stored + 4) {
                return 1;
            }
            if lw(sp + 8) != lw(stored + 8) {
                return 1;
            }
            goodiesGetPlayableChrs(sp + 16);
            let stored = 0x1fc33c0;
            if lw(sp + 16) == lw(0x1fc33c0) {
                if lw(sp + 20) != lw(stored + 4) {
                    return 1;
                }
                if lw(sp + 24) == lw(stored + 8) {
                    return 0;
                }
            }
        }
    }
    1
}

/// The text to show for the first thing that is new among the goodies (the language's text at +1732 to +1756 for the cheats, levels, bot sets, the two misc flags, the available
/// characters and the playable characters), the stored value being brought up to date for it; 0 when nothing is new (@ 0x240630).
pub fn getNextGoodieString() -> i32 {
    let frame = Frame::new(112);
    let sp = frame.sp();
    let cheats = lw(g::unlockedcheats);
    let levels = goodiesGetLevels();
    let misc = goodiesGetMisc();
    let bots = goodiesGetBotSets();
    goodiesGetAvailableChrs(sp);
    goodiesGetPlayableChrs(sp + 16);
    let stored_misc = lbu(0x3affac);
    let first = misc & 1;
    let language = lw(g::cur_language);
    if first != stored_misc & 1 {
        sb(0x3affac, stored_misc & 254 | first);
        return lw(lw((language << 2) + g::languages) + 1752);
    }
    let second = misc & 2;
    if second != stored_misc & 2 {
        sb(0x3affac, stored_misc & 253 | second);
        return lw(lw((language << 2) + g::languages) + 1748);
    }
    if cheats != lw(0x3affa0) {
        sw(0x3affa0, cheats);
        return lw(lw((lw(g::cur_language) << 2) + g::languages) + 1732);
    }
    if levels != lw(0x3affa4) {
        sw(0x3affa4, levels);
        return lw(lw((lw(g::cur_language) << 2) + g::languages) + 1736);
    }
    if bots != lw(0x3affa8) {
        sw(0x3affa8, bots);
        return lw(lw((lw(g::cur_language) << 2) + g::languages) + 1756);
    }
    let available = 0x1fc33b0;
    if lw(sp) == lw(0x1fc33b0) && (lw(sp + 4) ^ lw(available + 4) | lw(sp + 8) ^ lw(available + 8)) == 0 {
        let playable = 0x1fc33c0;
        if lw(sp + 16) == lw(0x1fc33c0) && (lw(sp + 20) ^ lw(playable + 4) | lw(sp + 24) ^ lw(playable + 8)) == 0 {
            return 0;
        }
        let language = lw(g::cur_language);
        let last = lw(sp + 24);
        sd(playable, ld(sp + 16));
        sw(playable + 8, last);
        return lw(lw((language << 2) + g::languages) + 1744);
    }
    let language = lw(g::cur_language);
    let last = lw(sp + 8);
    sd(available, ld(sp));
    sw(available + 8, last);
    lw(lw((language << 2) + g::languages) + 1740)
}

/// Draw the front end's background: two wavy pictures (the textures `bgtexnum` and `bgtexnum2`, in grids of 9 by 4 and 16 by 8) with the strengths and offsets passed on the
/// stack, then the blending and dither state put as the front end expects (@ 0x22bd10).
pub fn frontfxBgGfx() {
    let frame = Frame::new(112);
    let sp = frame.sp();
    let shift_x = lwc1(g::frontbgtranslatex1);
    let a = lwc1(0x3ac94c);
    let b = lwc1(0x3ac950);
    let c = lwc1(0x3ac954);
    let d = lwc1(0x3ac958);
    let e = lwc1(0x3ac95c);
    let shift_y = lwc1(g::frontbgtranslatey1);
    let texture = lw(g::bgtexnum);
    let first = lwc1(0x3ac960);
    swc1(sp, 7.0_f32);
    swc1(sp + 8, 5.0_f32);
    swc1(sp + 16, shift_x);
    swc1(sp + 24, shift_y);
    frontfxWaveyTexture(texture, 0x1515256f, 9, 4, first, a, b, c, 1.0_f32, d, 1.5_f32, e);
    let shift_x = lwc1(g::frontbgtranslatex2);
    let shift_y = lwc1(g::frontbgtranslatey2);
    let texture = lw(g::bgtexnum2);
    let other = lwc1(0x3ac964);
    swc1(sp, 4.0_f32);
    swc1(sp + 8, 2.0_f32);
    swc1(sp + 16, shift_x);
    swc1(sp + 24, shift_y);
    frontfxWaveyTexture(texture, 0x1f1f2f5f, 16, 8, a, c, other, b, 1.0_f32, d, 1.5_f32, e);
    dlSetBlend(2);
    dlSetDitherMatrix(2);
    dlSetBlend(1);
}

/// Draw the controls page `page`: the rows of the controls list (from the item of the scroll position, `controls_list` + 2 of them), each a box that is lit when it is the page's
/// chosen item (and then the highlighted item and boxed with a flashing edge), the name of the control, and the value's menu item drawn through a made box (@ 0x20eef0).
pub fn setupcontrols_pageGfx(page: i32) {
    let frame = Frame::new(208);
    let sp = frame.sp();
    sw(sp + 32, page);
    let font = textSelect(1);
    sw(sp + 36, font);
    customcontrols_makemenu();
    let window_x = lw(g::window);
    let window = g::window;
    let shift = lw(window + 88);
    let mut item = (lbu(lw(lw(g::front)) + (g::controls_list + 12)) << 5) + g::controlsmenu_items;
    let mut margin = 24;
    if lw(window + 12) >= 500 {
        margin = 128;
    }
    let left = window_x + (margin >> (shift & 31));
    let window = g::window;
    let width = lw(window + 12);
    let shift_now = lw(window + 88);
    let mut y = lw(window + 20) + (lh(g::controls_list + 6) >> (lw(window + 92) & 31));
    let mut margin = 24;
    if width >= 500 {
        margin = 128;
    }
    sw(sp + 40, width - ((margin >> (shift_now & 31)) << 1));
    let font_record = lw(g::curfont);
    let step = if (lbu(g::fontflags) & 1) == 0 {
        lw(lw(g::curfont) + 12) << 1
    } else {
        (lw(font_record + 12) >> 1) << 1
    };
    let mut count = 0;
    if lhu(g::controls_list + 2) != 0 {
        let name_x = left + 4;
        let right = left + lw(sp + 40);
        let mut chosen_page = lw(sp + 32);
        loop {
            let bottom = y + step - 1;
            let edge = right - 1;
            let colour = if (lhu(item + 8) ^ lhu(chosen_page + 4)) != 0 { 0x2840 } else { 0x5040 };
            dlFillRectangle(left, y, edge, bottom, colour);
            let mut item_id = lhu(item + 8);
            let mut page_id = lhu(lw(sp + 32) + 4);
            if item_id != page_id {
                sw((lw(lw(g::front)) << 2) + g::highlightitem, 0);
            } else {
                let edge_colour = lbu(g::flashalpha) << 24 | 0x808080;
                guiDrawBox(left, y, edge, bottom, edge_colour, 0);
                item_id = lhu(item + 8);
                page_id = lhu(lw(sp + 32) + 4);
                sw((lw(lw(g::front)) << 2) + g::highlightitem, item);
            }
            let differs = page_id ^ item_id;
            let text_id = lw(item);
            let fg = if differs != 0 { lw(g::menuitem_fgcol) } else { 0x40787880 };
            let bg = if differs != 0 { lw(g::menuitem_bgcol) } else { 0x5080 };
            let next = y + 1;
            if text_id >= 1246 {
                textOutline(name_x, next, text_id, fg, bg);
            } else {
                let text = lw((text_id << 2) + lw((lw(g::cur_language) << 2) + g::languages));
                textOutline(name_x, next, text, fg, bg);
            }
            if (lhu(item + 10) & 4) != 0 {
                let value = lw(item + 12);
                let offset = 0 - lw(sp + 40) + 36;
                sw(value + 28, right - 32);
                sw(value + 36, 12);
                sw(value + 32, offset);
            }
            let rel_y = y - lw(g::window + 20);
            let rel_x = ((left - lhu(g::window)) << 16) >> 16;
            let height = if (lbu(g::fontflags) & 1) == 0 { lw(lw(g::curfont) + 12) } else { lw(lw(g::curfont) + 12) >> 1 };
            let rel_y = ((rel_y + height - 1) << 16) >> 16;
            count += 1;
            makebox(sp, rel_x, rel_y, 0, 0, 0, 0, 0);
            y = next + step;
            sw(sp + 16, item);
            sw(sp + 20, 0);
            item += 32;
            draw_menuitem(sp);
            chosen_page = lw(sp + 32);
            if count >= lhu(g::controls_list + 2) {
                break;
            }
        }
    }
    textSelect(lw(sp + 36));
}
