//! The drawing of the map maker's main page.

use crate::*;

/// The text of a count: the text at 0x3ae9e0 for 0, else the number formatted by 0x3ae9d8.
fn count_text(count: i32) -> i32 {
    if count == 0 {
        0x3ae9e0
    } else {
        vafmt(&[0x3ae9d8, count as i64])
    }
}

/// The colour of a tick image of the map maker: 0x800080 for the filled box (image 18), else -0x7fffff80.
fn tick_colour(image: i32) -> i32 {
    if (image ^ 18) != 0 { -0x7fffff80 } else { 0x800080 }
}

/// Draw the main page `page` of the map maker: the zoom box (lit when the page's chosen item is the front end's current one) and, in the edit mode 1, the table of the
/// objects used: for each of the five tile kinds its name, the count used (in the box width) and the count of the matching items (named from the item definitions) with a tick for
/// the kinds in use, and the totals of the lights, seals and special things below (@ 0x2525a0).
pub fn mapmakermain_pageDraw(page: i32) {
    let frame = Frame::new(176);
    let sp = frame.sp();
    let front = lw(g::front);
    let colour = if (((page - lw(front + 4)) >> 4) ^ (lw(front + 20) - 1)) != 0 { 0x1f1f1f7f } else { 0x7f7f7f7f };
    zoomboxDraw(46, 25, 593, 198, 0x2864, colour);
    if lw(g::editmode) != 1 {
        return;
    }
    let big_font = lw(g::menuitem_bigfont);
    let fg = lw(g::menuitem_fgcol);
    let languages = g::languages;
    let bg = lw(g::menuitem_bgcol);
    let mut i = 0;
    let old = textSelect(big_font);
    let mut y = 71;
    sw(sp, old);
    textOutline(66, 50, lw(lw((lw(g::cur_language) << 2) + languages) + 3516), fg, bg);
    textOutline(198, 50, lw(lw((lw(g::cur_language) << 2) + languages) + 3520), fg, bg);
    textOutline(320, 50, lw(lw((lw(g::cur_language) << 2) + languages) + 3524), fg, bg);
    textSelect(lw(g::menuitem_normalfont));
    let mut names = g::disp_padactstart + 4;
    let mut colours = g::storydata + 68;
    loop {
        let next_y = y + 1;
        if i > 0 {
            guiHGradient(66, next_y, 262, y + 11, lw(colours), 128, 0);
        }
        let used = mmItemCount(i);
        let used_text = count_text(used);
        let name = lw(names);
        let language = lw(g::cur_language);
        if name >= 1246 {
            let fg = lw(g::menuitem_fgcol);
            let bg = lw(g::menuitem_bgcol);
            textOutline(66, y, name, fg, bg);
        } else {
            let fg = lw(g::menuitem_fgcol);
            let bg = lw(g::menuitem_bgcol);
            textOutline(66, y, lw((name << 2) + lw((language << 2) + g::languages)), fg, bg);
        }
        let other = i + 9;
        let width = textWidth(used_text);
        let fg = lw(g::menuitem_fgcol);
        let bg = lw(g::menuitem_bgcol);
        textOutline(198 - width, y, used_text, fg, bg);
        let items = mmItemCount(other);
        let items_text = count_text(items);
        let fg = lw(g::menuitem_fgcol);
        let bg = lw(g::menuitem_bgcol);
        textOutline(
            336,
            y,
            lw((lh((other << 4) + g::itemdefs) << 2) + lw((lw(g::cur_language) << 2) + g::languages)),
            fg,
            bg,
        );
        let width = textWidth(items_text);
        let fg = lw(g::menuitem_fgcol);
        let bg = lw(g::menuitem_bgcol);
        textOutline(570 - width, y, items_text, fg, bg);
        let mut flags = lbu(g::fontflags);
        if i > 0 {
            let marked = mmItemCount(i + 4);
            let image = if marked != 0 { 18 } else { 19 };
            guiDrawImage(image, tick_colour(image), 2.3e+02_f32, (y + 2) as f32);
            flags = lbu(g::fontflags);
        }
        names += 8;
        colours += 8;
        i += 1;
        y = if (flags & 1) == 0 { next_y + lw(lw(g::curfont) + 12) } else { next_y + (lw(lw(g::curfont) + 12) >> 1) };
        if i >= 5 {
            break;
        }
    }
    let lights = mmItemCount(16);
    let lights_more = mmItemCount(17);
    let total = lights + lights_more;
    let fg = lw(g::menuitem_fgcol);
    let bg = lw(g::menuitem_bgcol);
    textOutline(66, 145, lw(lw((lw(g::cur_language) << 2) + g::languages) + 3528), fg, bg);
    let total_text = count_text(total);
    let width = textWidth(total_text);
    let fg = lw(g::menuitem_fgcol);
    let bg = lw(g::menuitem_bgcol);
    textOutline(198 - (width - 48), 145, total_text, fg, bg);
    let seals = mmItemCount(18);
    let seals_more = mmItemCount(19);
    let total = seals + seals_more;
    let fg = lw(g::menuitem_fgcol);
    let bg = lw(g::menuitem_bgcol);
    textOutline(66, 159, lw(lw((lw(g::cur_language) << 2) + g::languages) + 3532), fg, bg);
    let total_text = count_text(total);
    let languages = g::languages;
    let width = textWidth(total_text);
    let fg = lw(g::menuitem_fgcol);
    let bg = lw(g::menuitem_bgcol);
    textOutline(198 - (width - 48), 159, total_text, fg, bg);
    let specials = mmItemCount(14);
    let fg = lw(g::menuitem_fgcol);
    let bg = lw(g::menuitem_bgcol);
    textOutline(320, 145, lw(lw((lw(g::cur_language) << 2) + languages) + 3536), fg, bg);
    let image = if specials < 1 { 19 } else { 18 };
    guiDrawImage(image, tick_colour(image), lwc1(0x3aca9c), 147.0_f32);
    let more = mmItemCount(15);
    let fg = lw(g::menuitem_fgcol);
    let bg = lw(g::menuitem_bgcol);
    textOutline(320, 159, lw(lw((lw(g::cur_language) << 2) + languages) + 3540), fg, bg);
    let more_text = count_text(more);
    let width = textWidth(more_text);
    let fg = lw(g::menuitem_fgcol);
    let bg = lw(g::menuitem_bgcol);
    textOutline(570 - width, 159, more_text, fg, bg);
    textSelect(lw(sp));
}
