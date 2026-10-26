//! The planet that spawns characters, the rolling menu list and the last stand's end screen.

use crate::*;

/// The tick of the planet spawn effect for the effects group `group`: the prop in the first slot (+56) rises and swings by three sines of the group's time (still while the time is
/// below 0 and deleted past the constant at 0x3ad6e0), the particle group (+72) is turned about y and made smaller, and the group is freed after 4 (@ 0x2ac7b0).
pub fn specialfxPlanetSpawnTick(group: i32) {
    let frame = Frame::new(144);
    let sp = frame.sp();
    let groups = g::fxgroups;
    let slot = group * 92 + (groups + 56);
    let prop = lw(slot);
    let step = ps2float::mul(lwc1(g::updateframesf), lwc1(0x3ad6dc));
    if prop != 0 {
        let limit = lwc1(0x3ad6e0);
        let time_at = group * 92 + (groups + 48);
        let table = g::g_SinTable;
        let fraction = (lwc1(time_at)).pdiv(limit);
        let scale = ld(0x3a9748);
        let scale_two = ld(0x3a9750);
        let half = 0x3fe0000000000000_i64;
        let amplitude = lwc1(0x3ad6e4);
        let state = lw(prop + 352);
        let first = dptoli(dpadd(dpmul(dpadd(dpmul(fptodp(fraction), scale), scale), scale_two), half));
        swc1(state + 44, ps2float::add(ps2float::mul(lwc1(((first & 2047) << 2) + table), lwc1(0x3ad6e8)), lwc1(0x3ad6ec)));
        let fraction = (lwc1(time_at)).pdiv(limit);
        let second = dptoli(dpadd(dpmul(dpmul(fptodp(fraction), scale), scale_two), half));
        let shift = lwc1(0x3ad6f0);
        swc1(state + 52, ps2float::mul(lwc1(((second & 2047) << 2) + table), amplitude));
        let later = ps2float::add(lwc1(time_at), shift);
        let third = dptoli(dpadd(dpmul(dpmul(fptodp(later), scale), scale_two), half));
        swc1(state + 56, ps2float::mul(lwc1(((third & 2047) << 2) + table), amplitude));
        let mut now = lwc1(time_at);
        if now < 0.0_f32 {
            swc1(state + 44, 0.0_f32);
            now = lwc1(time_at);
        }
        if limit < now {
            specialPropDelete(lw(slot));
            sw(slot, 0);
        }
    }
    let groups = g::fxgroups;
    let particles = group * 92 + (groups + 72);
    if lw(particles) != 0 {
        matrixRotY(sp, ps2float::mul(ps2float::mul(ps2float::mul(step, 3.6e+02_f32), lwc1(0x3ad6f4)).pdiv(1.8e+02_f32), lw(groups + group * 92 + 4) as f32));
        let one = 1.0_f32;
        let shrink = ps2float::sub(one, ps2float::mul(step, lwc1(0x3ad6f8)));
        let shrink_y = ps2float::sub(one, ps2float::mul(step, lwc1(0x3ad6fc)));
        matrixScaleSet(sp + 64, shrink, shrink_y, shrink);
        let matrix = lw(particles) + 928;
        sceVu0MulMatrix(matrix, matrix, sp + 64);
        let matrix = lw(particles) + 928;
        sceVu0MulMatrix(matrix, matrix, sp);
    }
    if 4.0_f32 < lwc1(groups + group * 92 + 48) {
        specialfxFree(group);
    }
}

/// Make the items of a rolling menu list into `menus` (records of 24 bytes) around the item `item_number` of the page `page`: while the roll offset (`rolloffset`) is not 0 it
/// goes back towards 0 by 1 each time and the item before (or after) is made too, moved by the offset; unless `flag` is set the neighbours are made as well; the items of the page
/// then have flag 64 cleared for the item and set for the others; the number of the items made is returned (@ 0x24f208).
pub fn rollmenumake(menus: i32, page: i32, item_number: i32, flag: i32) -> i32 {
    let frame = Frame::new(16);
    let sp = frame.sp();
    let mut at = menus;
    let start = at;
    let current = menuitem(page, item_number & 0xffff);
    if current == 0 {
        return 0;
    }
    let before = menuitem(page, lhu(current + 16));
    let after = menuitem(page, lhu(current + 18));
    let mut offset = lw(g::rolloffset);
    if offset != 0 {
        if offset <= 0 {
            offset += 1;
        } else {
            offset -= 1;
        }
        sw(g::rolloffset, offset);
    }
    let top = lhu(page + 6) + lhu(g::rolloffset);
    sh(sp, lhu(page + 4));
    sh(sp + 2, top);
    if offset > 0 {
        let earlier = menuitem(page, lhu(before + 16));
        sh(sp + 2, lhu(sp + 2) - 10);
        if earlier != 0 {
            menuitemmake(at, sp, earlier);
            at += 24;
        }
        sh(sp + 2, lhu(sp + 2) + 10);
    }
    let mut y = lhu(sp + 2);
    if flag == 0 && before != 0 {
        menuitemmake(at, sp, before);
        at += 24;
        y = lhu(sp + 2);
    }
    let here = at;
    at += 24;
    sh(sp + 2, y + 10);
    menuitemmake(here, sp, current);
    sh(sp + 2, lhu(sp + 2) + 10);
    if flag == 0 && after != 0 {
        menuitemmake(at, sp, after);
        at += 24;
    }
    let count;
    if lw(g::rolloffset) >= 0 {
        count = lw(page);
    } else {
        let later = menuitem(page, lhu(after + 18));
        sh(sp + 2, lhu(sp + 2) + 10);
        if later != 0 {
            menuitemmake(at, sp, later);
            at += 24;
        }
        count = lw(page);
    }
    let made = at - start;
    let mut item = page + 16;
    let mut i = 0;
    if count > 0 {
        loop {
            let flags = lhu(item + 10);
            let new_flags = if item == current { flags & 0xffbf } else { flags | 64 };
            sh(item + 10, new_flags);
            i += 1;
            item += 32;
            if i >= count {
                break;
            }
        }
    }
    (made.wrapping_mul(-0x55555555)) >> 3
}

/// Draw the last stand's end screen: while the game state is 3 the language's text at +1836 in the middle at the top and, under it, the text at +0x11ac (level 8) or +0x11b0 as
/// formatted text; else the text at +1840 (in red), the text at +1852, the score of the player (`vafmt` of the format at 0x3b0000 + something) and, for the first player, the retry-or-quit
/// box (@ 0x211cf0).
pub fn drawLastStand() {
    textSelect(2);
    let window = g::window;
    let top = lw(window + 20);
    let split = lw(window + 92);
    let y = if split != 0 { top + 20 } else { top + 38 };
    let y1 = if split != 0 { y + 7 } else { y + 14 };
    if lw(g::gamestate) != 3 {
        let title = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1840);
        let width = textWidth(title);
        textOutline(lw(window + 8) - ((((width as u32) >> 31) as i32 + width) >> 1), y1, title, 0x7f202080, 80);
        let line = lw(lw(g::curfont) + 12) >> (lbu(g::fontflags) & 1 & 31);
        let y2 = (if lw(window + 92) == 0 { y1 + 16 } else { y1 + 4 }) + line;
        textSelect(1);
        let text = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1852);
        let width = textWidth(text);
        let fg = lw(g::menuitem_fgcol);
        let bg = lw(g::menuitem_bgcol);
        textOutline(lw(g::window + 8) - ((((width as u32) >> 31) as i32 + width) >> 1), y2, text, fg, bg);
        let font = lw(g::curfont);
        let y3 = y2 + 1 + if (lbu(g::fontflags) & 1) == 0 { lw(font + 12) } else { lw(font + 12) >> 1 };
        textSelect(2);
        let window = g::window;
        let score = vafmt(&[(g::lastgfxendtime + 4) as i64, lw(lw(lw(lw(g::currentplayer) + 384) + 352) + 248) as i64]);
        sb(g::fontflags, lbu(g::fontflags) | 2);
        let width = textWidth(score);
        textOutline(lw(window + 8) - ((((width as u32) >> 31) as i32 + width) >> 1), y3, score, 0x40787880, 0x5080);
        let player = lw(g::currentplayer);
        sb(g::fontflags, lbu(g::fontflags) & 253);
        if lw(player) != 0 {
            return;
        }
        let bottom = lw(window + 24);
        drawRetryQuit(if lw(window + 92) == 0 { bottom - 56 } else { bottom - 30 });
        return;
    }
    let title = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1836);
    let width = textWidth(title);
    textOutline(lw(window + 8) - ((((width as u32) >> 31) as i32 + width) >> 1), y1, title, 0x40787880, 0x5080);
    let line = lw(lw(g::curfont) + 12) >> (lbu(g::fontflags) & 1 & 31);
    let y2 = (if lw(window + 92) == 0 { y1 + 16 } else { y1 + 4 }) + line;
    let language = lw(g::cur_language);
    let text = if lw(g::lvlevelnum) != 8 {
        lw(lw((language << 2) + g::languages) + 0x11b0)
    } else {
        lw(lw((language << 2) + g::languages) + 0x11ac)
    };
    let fg = lw(g::menuitem_fgcol);
    let bg = lw(g::menuitem_bgcol);
    guiDrawFormattedText(text, 0, y2, 1, fg, bg);
}
