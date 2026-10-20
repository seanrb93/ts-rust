//! The goodie message sliding in, the glass of a room, the pad buttons of the map maker and the bars of the little team characters.

use crate::*;

/// Draw the goodie message (the text at 0x3affb4) for the time `time` (it slides in from the left for the first 60 ticks, stays, and slides out between 180 and 240; nothing at 0)
/// line by line in the big font from height 160, a line starting with ~ in the colour 0x40787880, a line starting with ` moved down by half a line, the others in `colour`
/// (@ 0x2408b0).
pub fn goodieMessageGfx(time: i32, colour: i32) {
    let frame = Frame::new(176);
    let sp = frame.sp();
    let text = lw(0x3affb4);
    if time == 0 {
        return;
    }
    let dither = dlSetDitherMatrix(2);
    sw(sp + 12, dither);
    let font = textSelect(4);
    let shown = lw(0x3affb4);
    sw(sp + 8, font);
    let mut length = 0;
    if lbu(shown) != 0 {
        loop {
            length += 1;
            if lbu(shown + length) == 0 {
                break;
            }
        }
    }
    let mut y = 160;
    let offset;
    if time >= 60 {
        let past = time - 180;
        if (past as u32) >= 60 {
            offset = 0;
        } else {
            let half = 0x3fe0000000000000_i64;
            let fraction = past as f32 / 6e+01_f32;
            let turned = dpmul(dpmul(fptodp(fraction), ld(0x3a1e60)), half);
            let index = dptoli(dpadd(dpmul(turned, ld(0x3a1e68)), half));
            offset = (-((1.0_f32 - lwc1((((index + 512) & 2047) << 2) + g::g_SinTable)) * lw(g::window + 12) as f32)) as i32;
        }
    } else {
        let half = 0x3fe0000000000000_i64;
        let fraction = time as f32 / 6e+01_f32;
        let turned = dpmul(dpmul(fptodp(fraction), ld(0x3a1e50)), half);
        let index = dptoli(dpadd(dpmul(turned, ld(0x3a1e58)), half));
        offset = ((1.0_f32 - lwc1(((index & 2047) << 2) + g::g_SinTable)) * lw(g::window + 12) as f32) as i32;
    }
    let mut at = 0;
    loop {
        let mut line = text + at;
        let mut paint = colour;
        let end = strchr(line, 10);
        if end != 0 {
            sb(end, 0);
        }
        let first = lbu(line);
        let mut next = first;
        if first != 126 {
            if first == 96 {
                let raw = lw(lw(g::curfont) + 12);
                let sign = ((raw as u32) >> 31) as i32;
                let height = if (lbu(g::fontflags) & 1) == 0 { raw } else { raw >> 1 };
                y += (height + sign) >> 1;
                at += 1;
                line = text + at;
                next = lbu(line);
            }
        } else {
            at += 1;
            paint = 0x40787880;
            line = text + at;
            next = lbu(line);
        }
        if next != 0 {
            let width = textWidth(line);
            let across = lw(g::window + 8);
            sw(sp + 4, y);
            sw(sp, across - ((((width as u32) >> 31) as i32 + width) >> 1) + offset);
            textPrint(sp, sp + 4, line, paint);
        }
        let skip = at + 1;
        let size = strlen(line);
        at = skip + size;
        if (lbu(g::fontflags) & 1) == 0 {
            y += lw(lw(g::curfont) + 12);
        } else {
            y += lw(lw(g::curfont) + 12) >> 1;
        }
        if end != 0 {
            sb(end, 10);
        }
        let total = strlen(text);
        if (at as u32) >= (total as u32) {
            break;
        }
    }
    textSelect(lw(sp + 8));
    dlSetDitherMatrix(lw(sp + 12));
}

/// Draw the panes of glass of the room `room` (a list of pointers ended by 0, from `bgGetGlass`): each one with a value at +0 of -2 or more is sent as a packet of 144 bytes (the
/// three points and texture coordinates and colours of its corners from +16 to +156, one of the ucodes by its kind at +8: 0, 2 or the one before) with the player's matrix and the
/// colour whose alpha follows the value (@ 0x258888).
pub fn bgGlassGfxRoom(room: i32) {
    let frame = Frame::new(192);
    let sp = frame.sp();
    sw(sp + 16, 0);
    let matrix = memdbAlloc(64);
    sceVu0CopyMatrix(matrix, lw(lw(g::currentplayer) + 1764));
    let list = bgGetGlass(room);
    let mut ucode = 0;
    if lw(list) == 0 {
        return;
    }
    let mut entry = list;
    loop {
        let glass = lw(entry);
        let mut count = lw(sp + 16);
        if -2.0_f32 <= lwc1(glass) {
            let packet = memdbAlloc(144);
            let kind = lw(glass + 8);
            if kind == 0 {
                ucode = 0;
            } else {
                ucode = if (kind ^ 1) == 0 { 2 } else { ucode };
            }
            let value = lwc1(glass);
            let colour = if !(0.0_f32 <= value) {
                let alpha = (value + 2.0_f32) * 0.5_f32 * 127.0_f32;
                fptoui(alpha) << 24 | 0x7f7f7f
            } else {
                0x7f7f7f7f
            };
            sw(sp + 12, colour);
            sw(sp, colour);
            sw(sp + 4, colour);
            sw(sp + 8, colour);
            let step = ucode << 2;
            sb(lw(g::gfxtag) + 3, 48);
            let tag = lw(g::gfxtag);
            sw(tag + 4, g::vif_load_mmtx_357350);
            sw(g::gfxtag, tag + 16);
            sh(tag, 1);
            sb(tag + 16 + 3, 48);
            let tag = lw(g::gfxtag);
            sh(tag, 4);
            sw(g::gfxtag, tag + 16);
            sw(tag + 4, matrix);
            sb(tag + 16 + 3, 48);
            let tag = lw(g::gfxtag);
            sw(tag + 4, g::vif_entry_loadm_357360);
            sw(g::gfxtag, tag + 16);
            sh(tag, 1);
            sb(tag + 16 + 3, 48);
            sw(packet, 0x6c018000);
            let tag = lw(g::gfxtag);
            sw(tag + 4, packet);
            let after = tag + 16;
            sh(tag, 9);
            sw(packet + 4, 0x8004);
            sw(packet + 8, 0x302e4000);
            sw(g::gfxtag, after);
            sw(packet + 12, 1042);
            sw(packet + 16, 0);
            sw(packet + 20, lw(step + g::ucode_loadxyzw) | 0x68048000);
            let mut put = packet + 24;
            for offset in [56, 60, 64, 16, 20, 24, 96, 100, 104, 136, 140, 144] {
                swc1(put, lwc1(glass + offset));
                put += 4;
            }
            sw(put, lw(step + g::ucode_loadst) | 0x68048000);
            put += 4;
            for offset in [68, 72, 76, 28, 32, 36, 108, 112, 116, 148, 152, 156] {
                swc1(put, lwc1(glass + offset));
                put += 4;
            }
            let texbuf = lw(lw(g::thistexbuf) + 4);
            let textures = lw(g::leveltextures);
            sw(put, lw(step + g::ucode_loadrgba) | 0x6e048000);
            sw(put + 4, lw(sp));
            sw(put + 8, lw(sp + 4));
            sw(put + 12, lw(sp + 8));
            sw(put + 16, lw(sp + 12));
            let picture = lw((lw(glass + 4) << 4) + textures);
            let texture = lw((if -1 >= picture { 0 } else { picture }) * 40 + texbuf + 32);
            sb(after + 3, 48);
            let tag = lw(g::gfxtag);
            sw(tag + 4, texture);
            sw(g::gfxtag, tag + 16);
            sh(tag, 8);
            sb(tag + 16 + 3, 48);
            let tag = lw(g::gfxtag);
            sw(tag + 4, (ucode << 5) + g::startucode);
            sh(tag, 1);
            sw(g::gfxtag, tag + 16);
            count = lw(sp + 16);
        }
        let next = count + 1;
        sw(sp + 16, next);
        entry = (next << 2) + list;
        if lw(entry) == 0 {
            break;
        }
    }
}

/// Draw the buttons of the map maker's pad at (`x`, `y`): the four direction buttons (at the top, right, left and bottom) with the texts of `button_text` beside the top, right and
/// left ones, and the bottom one's text centred under it, with the number of tiles and the colour of the light (hue, saturation, volume, and its phase picture) added when edit flag
/// 0x100000 is set (@ 0x24ade8).
pub fn mmDrawPad(x: i32, y: i32) {
    let frame = Frame::new(176);
    let sp = frame.sp();
    let buttons = g::button_text;
    let left = x - 12;
    let font = textSelect(1);
    sw(sp, font);
    guiDrawButton(left, y - 20, 2, (0 < (lw(buttons + 8) as u32)) as i32);
    let label = lw(buttons + 8);
    let mut right = lw(g::button_text);
    if label != 0 {
        let width = textWidth(label);
        let text = lw(buttons + 8);
        let fg = lw(g::menuitem_fgcol);
        textOutline(x - (width >> 1), y - 30, text, fg, 128);
        right = lw(g::button_text);
    }
    let row = y - 12;
    guiDrawButton(x + 12, row, 0, (0 < (right as u32)) as i32);
    let text = lw(g::button_text);
    if text != 0 {
        let fg = lw(g::menuitem_fgcol);
        textOutline(x + 46, row, text, fg, 128);
    }
    guiDrawButton(x - 36, row, 3, (0 < (lw(buttons + 12) as u32)) as i32);
    let back = lw(buttons + 12);
    let down;
    if back == 0 {
        down = lw(buttons + 4);
    } else {
        let width = textWidth(back);
        let text = lw(buttons + 12);
        let fg = lw(g::menuitem_fgcol);
        textOutline(x - (width + 46), row, text, fg, 128);
        down = lw(buttons + 4);
    }
    guiDrawButton(left, y - 4, 1, (0 < (down as u32)) as i32);
    let title = lw(buttons + 4);
    if title == 0 {
        textSelect(lw(sp));
        return;
    }
    let mut width = textWidth(title);
    let tiles = g::numtiles + 4;
    let colour_name = 0x3ae938;
    if (lw(g::editflags) & 0x100000) != 0 {
        width += textWidth(tiles);
        width += textWidth(colour_name);
        if (lbu(g::lighttype) as u32) >= 2 {
            width += 50;
        } else {
            width += 28;
        }
    }
    let buttons = g::button_text;
    let line = y + 6;
    let text = lw(buttons + 4);
    let mut at = x - ((width + ((width as u32) >> 31) as i32) >> 1);
    let fg = lw(g::menuitem_fgcol);
    textOutline(at, line, text, fg, 128);
    if (lw(g::editflags) & 0x100000) != 0 {
        let width = textWidth(lw(buttons + 4));
        at += width;
        let fg = lw(g::menuitem_fgcol);
        textOutline(at, line, tiles, fg, 128);
        let width = textWidth(tiles);
        at = at + 4 + width;
        dlFillRectangle(at, line, at + 24, y + 18, 128);
        let volume = lwc1(g::lightvolume);
        let hue = lwc1(g::lighthue);
        let saturation = lwc1(g::lightsaturation);
        let swatch = hsv2col(hue, saturation, volume);
        dlFillRectangle(at + 4, y + 8, at + 20, y + 16, swatch);
        let mut fg = lw(g::menuitem_fgcol);
        let mut next = at + 28;
        if (lbu(g::lighttype) as u32) >= 2 {
            guiDrawImage(lbu(g::lightphase) + 24, -0x7f7f7f80, next as f32, line as f32);
            next += 24;
            fg = lw(g::menuitem_fgcol);
        }
        textOutline(next, line, colour_name, fg, 128);
    }
    textSelect(lw(sp));
}

/// Draw the bars of the little characters of the teams along the bottom of the window: without team play one grey bar with all the players and bots; with team play one bar for
/// each allowed team in the colour of the team (from `teamdata`), the widths made to fit the number of characters (the first teams get as much as they need, the rest share what is
/// left), and in the mode 4 the name of the team's side above its characters (@ 0x235de0).
pub fn drawLittleGuys() {
    let frame = Frame::new(192);
    let sp = frame.sp();
    if lw(0x1fc1228) == 6 {
        return;
    }
    if lw(g::gv_teamplay) == 0 {
        let bottom = lw(g::window + 32);
        let row = bottom - 10;
        sw(sp, 20);
        guiVGradient(16, bottom - 9, lw(g::window + 12) - 17, bottom - 1, 0x50505080, 0, 128);
        drawTeamPlayers(0, sp, row);
        drawTeamBots(0, sp, row, 0);
        return;
    }
    if numAllowedTeams() == 0 {
        return;
    }
    let teams = numAllowedTeams();
    if teams == 0 {
        trap("break");
    }
    let width = lw(g::window + 12);
    sw(sp + 12, 0);
    let base = 0x1fc1234;
    let mut extra = 0;
    let mut i = 0;
    sw(sp + 16, -1);
    let mut address = i + base;
    let mut counted = 0;
    let mut full = true;
    loop {
        let next = i + 1;
        if lbu(address) == 0 {
            i = next;
        } else {
            sw(sp + 4, 0);
            drawTeamPlayers(next, sp + 4, -1);
            drawTeamBots(next, sp + 4, -1, 0);
            counted = lw(sp + 4) << 4;
            if (width - 32).wrapping_div(teams) < counted + 8 {
                full = false;
                break;
            }
            i = next;
        }
        address = i + base;
        if i >= 4 {
            break;
        }
    }
    if !full {
        extra = counted - ((width - 32).wrapping_div(teams) - 8);
        sw(sp + 16, i);
        if numAllowedTeams() >= 2 {
            let others = numAllowedTeams() - 1;
            if others == 0 {
                trap("break");
            }
            sw(sp + 12, extra.wrapping_div(others));
        }
    }
    let bottom = lw(g::window + 32);
    sw(sp + 20, (width - 32).wrapping_div(teams) + extra);
    let row = bottom - 10;
    let mut left = 16;
    let mut team = 0;
    loop {
        let special = lw(sp + 16);
        if lbu(team + 0x1fc1234) == 0 {
            team += 1;
        } else {
            let window_width = lw(g::window + 12);
            let mut right = left
                + (if (team ^ special) == 0 {
                    lw(sp + 20)
                } else {
                    (width - 32).wrapping_div(teams) - lw(sp + 12)
                });
            if (window_width - 16) < right {
                right = window_width - 16;
            }
            let colour = lw((team << 3) + (g::teamdata + 4));
            guiVGradient(left, row + 1, right - 1, row + 9, colour, 0, 128);
            let inner = left + 4;
            let mode = lw(0x1fc1228);
            sw(sp + 8, inner);
            let following = team + 1;
            if mode == 4 {
                sb(g::fontflags, lbu(g::fontflags) | 1);
                let title = if following != 1 {
                    lw(lw((lw(g::cur_language) << 2) + g::languages) + 1696)
                } else {
                    lw(lw((lw(g::cur_language) << 2) + g::languages) + 1692)
                };
                textOutline(inner, row + 2, title, 0x40787880, 0x5080);
                let across = lw(g::window + 12);
                sb(g::fontflags, lbu(g::fontflags) & 254);
                let at = lw(sp + 8);
                sw(sp + 4, 0);
                sw(sp + 8, at + (across - 32).wrapping_div(5));
                drawTeamPlayers(following, sp + 4, -1);
                drawTeamBots(following, sp + 4, -1, 0);
                let used = lw(sp + 4) << 4;
                if right < lw(sp + 8) + used + 4 {
                    sw(sp + 8, (right - 4) - used);
                }
            }
            drawTeamPlayers(following, sp + 8, row);
            drawTeamBots(following, sp + 8, row, 0);
            left = right;
            team = following;
        }
        if team >= 4 {
            break;
        }
    }
}
