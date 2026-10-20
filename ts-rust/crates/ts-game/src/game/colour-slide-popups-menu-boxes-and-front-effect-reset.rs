//! The colour slide of the light editor, popups and the memory bar of the map maker, the bots of the bot set-up, making a menu item, plain text boxes, and resetting the front effects.

use crate::*;

/// Draw the colour slide of the box `slide` (x, y, width, height as halfwords): a strip of the light's colour (`lighthue`, `lightsaturation`) fading across, a bar at the light's
/// volume with a thin white line on it (@ 0x251600).
pub fn draw_colourslide(slide: i32) {
    let frame = Frame::new(48);
    let sp = frame.sp();
    let colour = hsv2col(lwc1(g::lighthue), lwc1(g::lightsaturation), 1.0_f32);
    let top = lhu(slide + 2);
    let right = lhu(slide) + lhu(slide + 4) - 2;
    let left = lhu(slide) + 2;
    let bottom = top + lhu(slide + 6) - 1;
    let first = top + 1;
    sw(sp + 12, colour);
    sw(sp + 20, first);
    sw(sp + 28, left);
    sw(sp + 36, 128);
    sw(sp + 40, right);
    sw(sp + 44, bottom);
    sw(sp, colour);
    sw(sp + 4, left);
    sw(sp + 8, first);
    sw(sp + 16, right);
    sw(sp + 24, 128);
    sw(sp + 32, bottom);
    dlDrawTriangleStrip(sp, 4, 0);
    let x0 = lhu(slide);
    let left_f = x0 as f32;
    let right_f = (x0 + lhu(slide + 4)) as f32;
    let one = 1.0_f32;
    let level = lhu(slide + 2) as f32 + (one - lwc1(g::lightvolume)) * lhu(slide + 6) as f32 - one;
    let level_bottom = level + 2.5_f32;
    dlFillRectangleFloat(128, left_f, level, right_f, level_bottom);
    let half = 0.5_f32;
    dlFillRectangleFloat(0x7f7f7f80, left_f + one, level + half, right_f - one, level_bottom - half);
}

/// Draw a popup of the map maker: a zoom box from (`x`, `y`) to (`right`, `bottom`), the text `text` at its top left and, when `button_text` is not 0, that text at the right with
/// the button image `image` after it (@ 0x24b398).
pub fn mmDrawPopup(text: i32, x: i32, y: i32, right: i32, bottom: i32, mode: i32, button_text: i32, image: i32) {
    let old = textSelect(1);
    zoomboxDraw(x, y, right, bottom, mode, 0x7f7f7f7f);
    let line = if (lbu(g::fontflags) & 1) == 0 {
        lw(lw(g::curfont) + 12)
    } else {
        lw(lw(g::curfont) + 12) >> 1
    };
    let spare = bottom - y - 5 - line;
    let text_y = y + ((spare + ((spare as u32) >> 31) as i32) >> 1) + 2;
    textOutline(x + 10, text_y, text, 0x7f7f7f80, 0x2880);
    if button_text != 0 {
        let left = right - textWidth(button_text);
        textOutline(left - 44, text_y, button_text, 0x7f7f7f80, 0x2880);
        let width = textWidth(button_text);
        guiDrawImage(image, -0x7f7f7f80, (left - 28 + width) as f32, text_y as f32);
    }
    textSelect(old);
}

/// Draw the bot set-up at (`x`, `y`): the text (the language's at +1452, or +1448 when there are bots) and, when there are bots and the page is not 6, the bots of each team after it
/// (of the 4 teams when team play is on, else all together) (@ 0x235cc8).
pub fn drawBotSet(x: i32, y: i32) {
    let frame = Frame::new(16);
    let sp = frame.sp();
    sw(sp, x);
    let bots = countBots(0);
    let language = lw(g::cur_language);
    let text = lw(lw((language << 2) + g::languages) + if bots == 0 { 1452 } else { 1448 });
    let at = lw(sp);
    if lw(0x1fc1228) != 6 {
        let fg = lw(g::menuitem_fgcol);
        let bg = lw(g::menuitem_bgcol);
        textOutline(at, y, text, fg, bg);
        if countBots(0) != 0 {
            let below = y + 2;
            let width = textWidth(text);
            let teamplay = lw(g::gv_teamplay);
            sw(sp, lw(sp) + 32 + width);
            if teamplay != 0 {
                let mut team = 1;
                loop {
                    drawTeamBots(team, sp, below, 1);
                    let more = team < 4;
                    team += 1;
                    if !more {
                        break;
                    }
                }
            } else {
                drawTeamBots(0, sp, below, 0);
            }
        }
    }
}

/// Make the menu box `menu` for the item `item` placed by `at` (x, y as halfwords): its size is that of its text (the string number at +0, below 1246 taken from the current
/// language; in the large font if the item has flag 256; 252 or 332 wide for an item with flag 4), the item and its text are kept at +16 and +20 of the box (@ 0x24bff8).
pub fn menuitemmake(menu: i32, at: i32, item: i32) {
    let frame = Frame::new(16);
    let sp = frame.sp();
    let y = lh(at + 2);
    let x = lh(at);
    let mut text = lw(item);
    sw(sp, 0);
    sw(sp + 4, 0);
    if text != 0 {
        let mut old = -1;
        if (lhu(item + 10) & 256) != 0 {
            old = textSelect(lw(g::menuitem_bigfont));
            text = lw(item);
        }
        let language = lw(g::cur_language);
        if text >= 1246 {
            textSize(sp, sp | 4, text);
        } else {
            textSize(sp, sp | 4, lw((text << 2) + lw((language << 2) + g::languages)));
        }
        if old != -1 {
            textSelect(old);
        }
    }
    let flags = lhu(item + 10);
    if (flags & 4) != 0 {
        sw(sp, if (flags & 0x8000) != 0 { 252 } else { 332 });
    }
    makebox(menu, x, y, lh(sp), (lhu(sp + 4) << 16) >> 16, lhu(item + 8), 0, 0x24bc00);
    let text = lw(item);
    sw(menu + 16, item);
    if text >= 1246 {
        sw(menu + 20, text);
    } else {
        sw(menu + 20, lw((text << 2) + lw((lw(g::cur_language) << 2) + g::languages)));
    }
}

/// Draw the bar of the map maker's memory used at (`x`, `y`): 100 wide, filled in proportion to the cost of the map (the cost over 50), red and flashing when the cost has passed
/// the limit of 5001; the text of the language (at +3580) is put before it (@ 0x24b508).
pub fn mmDrawMemFree(x: i32, y: i32) {
    let mut border = 128;
    let used = lw(g::mapcost) / 50;
    let mut free_colour = 0x5080;
    let mut used_colour = 0x646480;
    let edge = x - (used - 99);
    if (lw(g::editflags) & 0x400000) != 0 {
        border = if (lw(g::tickframenum) & 32) != 0 { 0x7f7f7f80 } else { border };
        free_colour = -0x7f7fff80;
        used_colour = -0x7fffff80;
    }
    dlFillRectangle(x - 2, y, x + 101, y + 9, border);
    if x < edge {
        dlFillRectangle(x, y + 1, edge - 1, y + 8, used_colour);
    }
    if edge < x + 100 {
        dlFillRectangle(edge, y + 1, x + 99, y + 8, free_colour);
    }
    let old = textSelect(1);
    let language = lw(g::cur_language);
    sb(g::fontflags, lbu(g::fontflags) | 1);
    let text = lw(lw((language << 2) + g::languages) + 3580);
    let width = textWidth(text);
    let colour = if (lw(g::mapcost) + lw(g::flyingobjcost)) >= 0x1389 { used_colour } else { 0x40787880 };
    textOutline(x - width - 8, y + 2, text, colour, 128);
    sb(g::fontflags, lbu(g::fontflags) & 254);
    textSelect(old);
}

/// Draw the text box `item` of the neat menu (the style at +20 of it: flags 1 centre, 2 large font, 4 the dim colours; a fill colour at +8; the text at +16): the box is filled, and
/// the text drawn with an outline in the middle of its height (@ 0x24fb00).
pub fn draw_neattext(item: i32) {
    let style = lw(item + 20);
    let mut font = lw(g::menuitem_bigfont);
    if (lw(style) & 2) == 0 {
        font = lw(g::menuitem_normalfont);
    }
    let old = textSelect(font);
    let fill = lw(style + 8);
    let mut x = lw(g::window) + lhu(item);
    let text = lw(item + 16);
    let y = lw(g::window + 20) + lhu(item + 2);
    if fill != 0 {
        dlFillRectangle(x, y, x + lhu(item + 4), y + lhu(item + 6), fill);
    }
    if text == 0 {
        return;
    }
    let mut flags = lw(style);
    let mut font_flags = lbu(g::fontflags);
    if (flags & 1) != 0 {
        let width = textWidth(lw(item + 16));
        flags = lw(style);
        x = x + ((lhu(item + 4) as u32) >> 1) as i32 - ((((width as u32) >> 31) as i32 + width) >> 1);
        font_flags = lbu(g::fontflags);
    }
    let height = lhu(item + 6);
    let line = if (font_flags & 1) == 0 {
        lw(lw(g::curfont) + 12)
    } else {
        lw(lw(g::curfont) + 12) >> 1
    };
    let spare = height - line;
    let dim = flags & 4;
    let mut colour = 0x40787880;
    if dim == 0 {
        colour = lw(style + 4);
    }
    let dither = dlSetDitherMatrix(2);
    textOutline(x, y + ((spare + ((spare as u32) >> 31) as i32) >> 1), text, colour, if dim != 0 { 0x5080 } else { 64 });
    dlSetDitherMatrix(dither);
    textSelect(old);
}

/// Find the texture numbers of the front end's effects (3 logo layers at 0x1fbd1a8, the two logo tops, 5 flare pictures at 0x1fbd1b8 and the two backgrounds) and reset the
/// effects' times, scales and the flare's intensities, then the goodies' effect (@ 0x22a440).
pub fn frontfxReset() {
    let mut put = 0x1fbd1a8;
    let mut names = g::logolayersname;
    let mut left = 2;
    loop {
        let name = lw(names);
        left -= 1;
        names += 4;
        let texture = preloadGetTexName(name);
        sw(put, lw(texture + 20));
        put += 4;
        if left < 0 {
            break;
        }
    }
    let mut left = 4;
    let top1 = preloadGetTexName(0x3a17d0);
    sw(g::logotop1num, lw(top1 + 20));
    let top2 = preloadGetTexName(0x3a17f8);
    let mut put = 0x1fbd1b8;
    sw(g::logotop2num, lw(top2 + 20));
    let mut names = g::flareTexturenames_352a78;
    loop {
        let name = lw(names);
        left -= 1;
        names += 4;
        let texture = preloadGetTexName(name);
        sw(put, lw(texture + 20));
        put += 4;
        if left < 0 {
            break;
        }
    }
    swc1(g::globalscaley, 1.0_f32);
    let mut intensity = g::flareintensity + 20;
    swc1(g::flarescalarx, -2.0_f32);
    let mut left = 5;
    sw(g::frontfxtime, 0);
    sw(g::fronttime, 0);
    sw(g::frontbgtime, 0);
    sw(g::globalscalex, 0);
    sw(g::flarescalary, 0);
    loop {
        sw(intensity, 0);
        left -= 1;
        intensity -= 4;
        if left < 0 {
            break;
        }
    }
    let background = preloadGetTexName(0x3a1820);
    sw(g::bgtexnum, lw(background + 20));
    let second = preloadGetTexName(0x3a1840);
    sw(g::bgtexnum2, lw(second + 20));
    swc1(g::frontbgtranslatex3, 2.0_f32);
    sw(g::frontstate, -1);
    sw(g::frontbgtranslatex1, 0);
    sw(g::frontbgtranslatex2, 0);
    sw(g::frontbgtranslatey1, 0);
    sw(g::frontbgtranslatey2, 0);
    sw(g::frontbgtranslatey3, 0);
    frontfxGoodiesReset();
}
