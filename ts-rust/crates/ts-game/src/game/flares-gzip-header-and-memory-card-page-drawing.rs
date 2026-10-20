//! The lens flares of the lights, the reading of a gzip header, and the memory card page's drawing.

use crate::*;

/// Draw the lens flare `flare` (kind at +4, place at +8, brightness at +20): the place is put on the screen by the player's matrices and, when it is in front and inside the screen,
/// the flare's parts (the count at +0 of the kind's record of 244 bytes in the load buffer, each with a colour, a place factor, a size and a texture) are drawn as rectangles
/// centred on the place moved by the part's factor, the brightness fading towards the edge of the screen; on level 13 the kind is 2 and the brightness is scaled by the constant
/// at 0x3ad610 (@ 0x2a67b8).
pub fn flareGfx(flare: i32) {
    let frame = Frame::new(304);
    let sp = frame.sp();
    let mut scale = 1.0_f32;
    let mut kind = lw(flare + 4);
    if lw(g::lvlevelnum) == 13 {
        scale = lwc1(0x3ad610);
        kind = 2;
    }
    let player = lw(g::currentplayer);
    let projection = lw(player + 1764);
    let view = lw(player + 1760);
    sceVu0MulMatrix(sp + 32, view, projection);
    matrixVec3Mul4(sp + 32, flare + 8, sp);
    let mut visible = false;
    let depth = lwc1(sp + 8);
    let inverse = 1.0_f32 / lwc1(sp + 12);
    let screen_z = depth * inverse;
    let screen_x = lwc1(sp) * inverse;
    let screen_y = lwc1(sp + 4) * inverse;
    swc1(sp + 24, screen_z);
    swc1(sp + 16, screen_x);
    swc1(sp + 20, screen_y);
    if 0.0_f32 <= depth
        && 0.0_f32 <= screen_z
        && -1.0_f32 <= screen_x
        && screen_x <= 1.0_f32
        && -1.0_f32 <= screen_y
        && screen_y <= 1.0_f32
    {
        visible = true;
    }
    if !visible {
        return;
    }
    let x = lwc1(sp + 16);
    let y = lwc1(sp + 20);
    let fade = lwc1(0x3ad614) - (x * x + y * y).abs().sqrt();
    let mut brightness = scale;
    if !(1.0_f32 < fade) {
        brightness = fade * brightness;
    }
    let intensity = lw(flare + 20);
    let brightness = if intensity < 0 {
        let half = (intensity & 1 | ((intensity as u32) >> 1) as i32) as f32;
        brightness * (half + half)
    } else {
        brightness * intensity as f32
    };
    let alpha = fptoui(brightness);
    sw(sp + 96, alpha);
    let base = g::loadbuf + 576;
    let records = base;
    if lw(kind * 244 + (base - 0x5d50)) > 0 {
        let mut i = 0;
        loop {
            let start = base - 0x5d50;
            let word = i << 2;
            let part = start + (i * 12 + kind * 244);
            let red = lwc1(part + 124) * 127.0_f32;
            let blue = lwc1(part + 132) * 127.0_f32;
            let green = lwc1(part + 128) * 127.0_f32;
            let top = fptoui(red);
            let colour_high = top << 24 | lw(sp + 96);
            let low = fptoui(blue);
            let mid = fptoui(green);
            let entry = start + (word + kind * 244);
            let along = lwc1(entry + 4);
            let centre_x = lw(g::window + 8) as f32 + lw(g::window + 16) as f32 * lwc1(sp + 16) * along;
            let size = lwc1(entry + 44) * 256.0_f32 * (6e+01_f32 / lwc1(lw(g::currentplayer) + 260));
            let centre_y = lw(g::window + 28) as f32 + lw(g::window + 36) as f32 * -lwc1(sp + 20) * along;
            let players = ilinkGetNumLocalPlayers();
            let mut wide = size;
            if players >= 2 {
                wide = size * 0.5_f32;
            }
            let table = records - 0x5d50;
            i += 1;
            let high = wide * lwc1(0x3ad618);
            let texture = lw((lw(table + (word + kind * 244) + 84) << 2) + g::flareTextureNum);
            dlSelectTextureKick(texture);
            let colour = colour_high | (low << 8 | mid << 16);
            dlTextureRectangleFloat(0, 0, 1024, 1024, colour, centre_x - wide, centre_y - high, centre_x + wide, centre_y + high);
            if i >= lw(kind * 244 + table) {
                break;
            }
        }
    }
}

/// Read the header of a gzip stream at the buffer 0x3afeb8 (position at 0x3afebc, size at 0x3afec0): the two magic bytes 31 and 139, the method 8, the flags (a byte past the end
/// reads as 255), the six bytes of time, extra flags and system skipped, the extra field of flag 4 (its length first), the name of flag 8 up to its 0 byte; the program exits with
/// 1 for a wrong header or for the flags 2, 16 or 32 (@ 0x21bed0).
pub fn GetHeader() {
    let frame = Frame::new(32);
    let sp = frame.sp();
    let size = lw(0x3afec0);
    let mut pos = lw(0x3afebc);
    let first;
    if (pos as u32) >= (size as u32) {
        first = 255;
    } else {
        first = lbu(lw(0x3afeb8) + pos);
        pos += 1;
        sw(0x3afebc, pos);
    }
    sb(sp, first);
    let buffer = lw(0x3afeb8);
    let second;
    if (pos as u32) >= (size as u32) {
        second = 255;
    } else {
        second = lbu(buffer + pos);
        sw(0x3afebc, pos + 1);
    }
    let magic = lbu(sp);
    sb(sp + 1, second);
    let at;
    if magic == 31 && second == 139 {
        at = lw(0x3afebc);
    } else {
        exit(1);
        return;
    }
    let buffer = lw(0x3afeb8);
    let next;
    if (at as u32) < (size as u32) {
        next = at + 1;
        let method = lbu(buffer + at);
        sw(0x3afebc, next);
        if method != 8 {
            exit(1);
            return;
        }
    } else {
        exit(1);
        return;
    }
    let flags;
    if (next as u32) >= (size as u32) {
        flags = 255;
    } else {
        flags = lbu(buffer + next);
        sw(0x3afebc, at + 2);
    }
    let reserved = flags & 2;
    if (flags & 32) != 0 {
        exit(1);
        return;
    }
    let pos = lw(0x3afebc);
    if reserved != 0 {
        exit(1);
        return;
    }
    if (pos as u32) < (size as u32) {
        sw(0x3afebc, pos + 1);
        if ((pos + 1) as u32) < (size as u32) {
            sw(0x3afebc, pos + 2);
            if ((pos + 2) as u32) < (size as u32) {
                sw(0x3afebc, pos + 3);
                if ((pos + 3) as u32) < (size as u32) {
                    sw(0x3afebc, pos + 4);
                    if ((pos + 4) as u32) < (size as u32) {
                        sw(0x3afebc, pos + 5);
                        if ((pos + 5) as u32) < (size as u32) {
                            sw(0x3afebc, pos + 6);
                        }
                    }
                }
            }
        }
    }
    let mut pos = lw(0x3afebc);
    if (flags & 4) != 0 {
        let low;
        if (pos as u32) >= (size as u32) {
            low = -1;
        } else {
            low = lbu(buffer + pos);
            pos += 1;
            sw(0x3afebc, pos);
        }
        let length;
        if (pos as u32) >= (size as u32) {
            length = low | -256;
        } else {
            let high = lbu(buffer + pos);
            sw(0x3afebc, pos + 1);
            length = low | high << 8;
        }
        let mut left = length - 1;
        if left != -1 {
            let mut at = lw(0x3afebc);
            loop {
                let inside = (at as u32) < (size as u32);
                if inside {
                    sw(0x3afebc, at + 1);
                }
                left -= 1;
                at = lw(0x3afebc);
                if left == -1 {
                    break;
                }
            }
        }
    }
    if (flags & 8) != 0 {
        let mut at = lw(0x3afebc);
        loop {
            if (at as u32) < (size as u32) {
                let byte = lbu(buffer + at);
                at += 1;
                sw(0x3afebc, at);
                if byte == 0 {
                    break;
                }
            }
        }
    }
    if (flags & 16) != 0 {
        exit(1);
    }
}

/// Draw the memory card page `page`: the window over the whole screen with its zoom box; in the saving and loading states (3 and 10) the two message texts (at 0x3affc4 and
/// 0x3affc8) centred in the zoom box, one under the other; otherwise the flashing marker over the page's chosen box and the menu item highlight (@ 0x243fa0).
pub fn mcseq_pageDraw(page: i32) {
    let frame = Frame::new(80);
    let sp = frame.sp();
    windowSet(0, 0, 639, 223);
    let window = g::window;
    dlSetClip(lw(g::window), lw(window + 20), lw(window + 4), lw(window + 24));
    windowGfx();
    zoomboxGfx(0);
    let state = lw(g::mcseq_state);
    if state != 3 && state != 10 {
        let id = lhu(page + 4);
        if id != 0 {
            let boxed = frontGetBox(page, id);
            if boxed != 0 {
                let top = lw(window + 20);
                let zoom = g::zoomboxes;
                let left = lw(zoom + 20) + 2;
                let right = left + lw(zoom + 28) - 4;
                let upper = top + lhu(boxed + 2) + 1;
                let lower = top + lhu(boxed + 2) + lhu(boxed + 6) - 1;
                let colour = lbu(g::flashalpha) | 0x8000;
                sw(sp + 16, left);
                sw(sp + 20, upper);
                sw(sp + 24, right);
                sw(sp + 28, lower);
                dlFillRectangle(left, upper, right, lower, colour);
            }
        }
        menuitemHighlight(page);
        return;
    }
    let first = lw(0x3affc4);
    let mut first_height = 0;
    let mut second_height = 0;
    if first != 0 {
        first_height = textHeight(first);
    }
    let second = lw(0x3affc8);
    if second != 0 {
        second_height = textHeight(second);
    }
    let zoom = g::zoomboxes;
    let base = lhu(zoom + 24);
    let spare = lw(zoom + 32) - (first_height + (second_height + 8));
    sh(sp, 0);
    let first_now = lw(0x3affc4);
    let y = base + ((spare + ((spare as u32) >> 31) as i32) >> 1);
    sh(sp + 2, y);
    if first_now != 0 {
        let fg = lw(g::menuitem_fgcol);
        let bg = lw(g::menuitem_bgcol);
        guiDrawFormattedText(first_now, 0, (y << 16) >> 16, 1, fg, bg);
        sh(sp + 2, lhu(sp + 2) + 8 + first_height);
    }
    let second_now = lw(0x3affc8);
    let y = lhu(sp + 2);
    if second_now != 0 {
        let fg = lw(g::menuitem_fgcol);
        let bg = lw(g::menuitem_bgcol);
        guiDrawFormattedText(second_now, 0, (y << 16) >> 16, 1, fg, bg);
        sh(sp + 2, lhu(sp + 2) + 20 + second_height);
    }
}
