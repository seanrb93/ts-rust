//! Making the tile set setup of a map maker level, a quad of the credits tunnel, and the drawing of a menu with its box.

use crate::*;

/// Make the tile set setup of the map maker level `level` from the map file in the temporary buffer: the chunks of the file (kinds 0x1324627 with the number of tiles at +4,
/// 0x1324628 the tiles) are read, and for each item of each tile (found among the pad's items by its number) a record of 16 bytes (the pad kinds 0 to 4 with the flags 127, 7, 11, 19
/// and 35) is made at +32 of the level or one of 48 bytes (the other kinds 5 to 19) at +16, with the flags, the size and the number of the prop of the item; an item that is
/// not found ends the program with -1 (@ 0x227158).
pub fn lvCreateTileSetSetup(level: i32) {
    let frame = Frame::new(208);
    let sp = frame.sp();
    sd(sp, ld(0x3ae770));
    sd(sp + 16, ld(0x3ae770));
    sw(sp + 32, 0);
    lvAllocTileSetSetup(level);
    let mut buffer = lw(g::mmtempbuffer);
    sw(sp + 36, 1001);
    let mut data = buffer + 8;
    let mut pads = lw(level + 32);
    let mut things = lw(level + 16);
    if (data as u32) >= ((buffer + 0x1ffc) as u32) {
        return;
    }
    sw(sp + 40, data);
    let first = lw(buffer + 8);
    data = buffer + 16;
    if first == 0x1324629 {
        return;
    }
    let mut chunk = first;
    loop {
        if chunk != 0x1324627 {
            if chunk == 0x1324628 && lw(sp + 32) > 0 {
                let mut at = data;
                let mut tile = 0;
                let mut number = lw(at);
                loop {
                    let count = lhu(at + 10);
                    let mut i = 0;
                    at += 36;
                    let pad = lw((number << 2) + g::tilepaddata);
                    let mut size;
                    if count == 0 {
                        size = lw(pad + 4);
                    } else {
                        size = lw(pad + 4);
                        loop {
                            let kind = lhu(at);
                            let mut found = 0;
                            let item = kind;
                            let definition = lw((((kind << 16) >> 12) as i32) + (g::itemdefs + 12));
                            let mut ok = false;
                            let base;
                            if size > 0 {
                                let wanted = lh(at + 2);
                                let mut in_range = true;
                                if lw(pad + 12) != wanted {
                                    let mut entry = pad + 12;
                                    found += 1;
                                    loop {
                                        entry += 28;
                                        if found >= size {
                                            in_range = false;
                                            break;
                                        }
                                        if lw(entry) == wanted {
                                            break;
                                        }
                                        found += 1;
                                    }
                                }
                                base = lw(sp + 36);
                                if in_range && found < size {
                                    ok = true;
                                }
                            } else {
                                base = 0;
                            }
                            if !ok {
                                exit(-1);
                                return;
                            }
                            let index = base + found;
                            if (kind & 0xffff) >= 5 {
                                let relative = item as i32 - 5;
                                if ((relative & 0xffff) as u32) < 15 {
                                    let sub = (relative << 16) >> 16;
                                    sw(things, 63);
                                    swc1(things + 40, lwc1(0x3ac8c4));
                                    sw(things + 4, definition);
                                    sw(things + 12, 0);
                                    sw(things + 16, 0);
                                    sw(things + 20, 0);
                                    sw(things + 28, 0);
                                    sw(things + 32, 0);
                                    sw(things + 36, 0);
                                    if (sub as u32) < 15 {
                                        let mut prop = false;
                                        match sub {
                                            0..=3 => {
                                                sw(things, 18);
                                                sw(things + 8, 16);
                                                sw(things + 20, ((kind << 16) >> 16) - 4);
                                            }
                                            4..=8 => {
                                                sw(things + 8, 2);
                                            }
                                            9 => {
                                                sw(things + 8, 1);
                                                sw(things, 1);
                                                prop = true;
                                            }
                                            10 => {
                                                sw(things, 16);
                                                sw(things + 8, 1);
                                                prop = true;
                                            }
                                            11 | 12 => {
                                                sw(things + 8, 4);
                                                let value = lwc1(sp + ((((kind << 16) >> 16) - 16) << 2));
                                                swc1(things + 12, value);
                                            }
                                            _ => {
                                                sw(things + 8, 8);
                                                let value = lwc1(sp + ((((kind << 16) >> 16) - 18) << 2) + 16);
                                                swc1(things + 12, value);
                                            }
                                        }
                                        if prop {
                                            let number = lvGetPropNum(definition);
                                            sw(things + 4, number);
                                        }
                                    }
                                    if lw(things + 8) != 1 {
                                        sw(things + 24, index);
                                    } else {
                                        let number = lvGetPropNum(lw((lh(at) << 4) + (g::itemdefs + 12)));
                                        if number != 325 {
                                            swc1(things + 40, lwc1(0x3ac8cc));
                                        } else {
                                            swc1(things + 40, lwc1(0x3ac8c8));
                                        }
                                        sw(things + 24, index);
                                    }
                                    things += 48;
                                }
                            } else {
                                let which = (item << 16) >> 16;
                                sw(pads, 63);
                                sw(pads + 4, index);
                                if (which as u32) < 5 {
                                    let flags = match which {
                                        0 => 127,
                                        1 => 7,
                                        2 => 11,
                                        3 => 19,
                                        _ => 35,
                                    };
                                    sw(pads + 8, flags);
                                }
                                pads += 16;
                            }
                            size = lw(pad + 4);
                            i += 1;
                            at += 4;
                            if i >= count {
                                break;
                            }
                        }
                    }
                    let _ = size;
                    tile += 1;
                    let tiles = lw(sp + 32);
                    sw(sp + 36, lw(sp + 36) + size);
                    if tile >= tiles {
                        break;
                    }
                    number = lw(at);
                }
                buffer = lw(g::mmtempbuffer);
            }
        } else {
            sw(sp + 32, lw(data + 4));
        }
        let current = lw(sp + 40);
        data += lw(current + 4);
        if (data as u32) >= ((buffer + 0x1ffc) as u32) {
            return;
        }
        sw(sp + 40, data);
        chunk = lw(data);
        data += 8;
        if chunk == 0x1324629 {
            return;
        }
    }
}

/// Make the points of a quad of the credits tunnel: the sections `a1` and `a2` of the tunnel (`getCreditTunnelSection`) at the angle given by `step` (the section number, in twelfths
/// of the turn with the constant at 0x3ad774, taken from the table of sines), and the rows of the record `record` (its lists of points, colours and texture coordinates at the
/// offsets from +4 to +0x120c) filled in three passes with the sine and cosine of the angle for the x and y of each, the z of 1.0 and the colour from the section's numbers
/// (@ 0x2af230).
pub fn creditsSetQuad(record: i32, section_a: i32, section_b: i32, step: i32) {
    let frame = Frame::new(528);
    let sp = frame.sp();
    let twelve = 12.0_f32;
    let step_f = step as f32;
    let one = 1.0_f32;
    let full = 3.6e+02_f32;
    let factor = lwc1(0x3ad774);
    let half_turn = 1.8e+02_f32;
    let limit = ld(0x3a98a8);
    let mut angle = (((step_f + one)).pdiv(twelve) * full * factor).pdiv(half_turn);
    if dpcmp(fptodp(angle), limit) >= 0 {
        angle = dptofp(dpsub(fptodp(angle), limit));
    }
    getCreditTunnelSection(sp, section_a);
    getCreditTunnelSection(sp + 48, section_b);
    sw(sp + 168, step << 4);
    let scale = lwc1(0x3ad778);
    sw(sp + 296, record + 4);
    sw(sp + 300, record + 8);
    sw(sp + 172, record + 12);
    sw(sp + 176, record + 16);
    sw(sp + 180, record + 20);
    sw(sp + 184, record + 24);
    sw(sp + 188, record + 28);
    sw(sp + 192, record + 32);
    let half = 0.5_f32;
    sw(sp + 196, record + 36);
    sw(sp + 200, record + 40);
    sw(sp + 204, record + 44);
    sw(sp + 208, record + 48);
    sw(sp + 212, record + 52);
    sw(sp + 220, record + 56);
    sw(sp + 228, record + 60);
    sw(sp + 232, record + 2312);
    let five = 5.0_f32;
    sw(sp + 240, record + 2316);
    sw(sp + 244, record + 2328);
    sw(sp + 248, record + 2332);
    sw(sp + 252, record + 2344);
    sw(sp + 256, record + 2348);
    sw(sp + 272, record + 2360);
    sw(sp + 284, record + 2364);
    sw(sp + 268, record + 0x1200);
    let y_b = section_b as f32 * five;
    sw(sp + 236, sp + 20);
    let y_a = section_a as f32 * five;
    sw(sp + 216, sp + 12);
    sw(sp + 224, sp + 16);
    sw(sp + 280, record + 0x1204);
    let z_b = lwc1(sp + 56);
    let index_b = (angle * scale + half) as i32;
    sw(sp + 288, record + 0x1208);
    sw(sp + 260, sp + 60);
    sw(sp + 276, sp + 68);
    sw(sp + 264, sp + 64);
    let index_a = (((step_f).pdiv(twelve) * full * factor).pdiv(half_turn) * scale + half) as i32;
    sw(sp + 292, record + 0x120c);
    let table = g::g_SinTable;
    let x_a = lwc1(sp);
    let y0 = lwc1(sp + 4);
    let x_b = lwc1(sp + 48);
    let y1 = lwc1(sp + 52);
    sw(sp + 132, (((index_a + 512) & 2047) << 2) + table);
    let z_a = lwc1(sp + 8);
    sw(sp + 320, step << 6);
    sw(sp + 144, ((index_b & 2047) << 2) + table);
    sw(sp + 136, ((index_a & 2047) << 2) + table);
    sw(sp + 140, (((index_b + 512) & 2047) << 2) + table);
    swc1(sp + 96, x_a);
    sw(sp + 304, g::layerMultiplier);
    swc1(sp + 100, y0);
    sw(sp + 128, 0);
    swc1(sp + 104, y_a);
    swc1(sp + 112, x_b);
    swc1(sp + 116, y1);
    swc1(sp + 120, y_b);
    loop {
        let layer = lwc1(lw(sp + 304));
        let pass = lw(sp + 128);
        let a = z_a * layer;
        let b = z_b * layer;
        let offset = lw(sp + 320) + pass * 768;
        let cos_a = lwc1(lw(sp + 132)) * a;
        sw(sp + 304, lw(sp + 304) + 4);
        let list_a = lw(sp + 172) + offset;
        let list_b = lw(sp + 176) + offset;
        let list_c = lw(sp + 180) + offset;
        let list_d = lw(sp + 184) + offset;
        let list_e = lw(sp + 188);
        let list_f = lw(sp + 192);
        let list_g = lw(sp + 196);
        swc1(record + offset, x_a + cos_a);
        let list_h = lw(sp + 204) + offset;
        let list_i = lw(sp + 208) + offset;
        let list_j = lw(sp + 200) + offset;
        let sin_a = lwc1(lw(sp + 136));
        let list_k = lw(sp + 228) + offset;
        let list_l = lw(sp + 212);
        sw(sp + 152, lw(sp + 220) + offset);
        sw(sp + 156, list_k);
        let list_m = lw(sp + 248);
        let list_n = lw(sp + 244) + offset;
        let list_o = lw(sp + 232);
        sw(sp + 148, list_l + offset);
        let list_p = lw(sp + 240);
        sw(sp + 160, list_n);
        sw(sp + 164, list_m + offset);
        let blue = lw(lw(sp + 224) + pass * 12);
        let list_q = lw(sp + 252) + offset;
        let colours = lw(sp + 216);
        let green = lw(lw(sp + 236) + pass * 12) << 16;
        swc1(lw(sp + 296) + offset, y0 + sin_a * a);
        let rows = lw(sp + 168);
        let red = lw(colours + pass * 12);
        let pass_now = lw(sp + 128);
        swc1(lw(sp + 300) + offset, y_a);
        let list_r = lw(sp + 256);
        swc1(list_a, one);
        let list_s = lw(sp + 272) + offset;
        let list_t = lw(sp + 284) + offset;
        let colour_a = red | 0x7f000000 | (green | blue << 8);
        let colour_ptr_a = lw(sp + 268) + (rows + pass_now * 192);
        let u0 = lw(sp + 264) + pass * 12;
        let cos_b = x_a + lwc1(lw(sp + 140)) * a;
        let u1 = lw(sp + 276) + pass * 12;
        let u2 = lw(sp + 260) + pass * 12;
        let colour_ptr_b = lw(sp + 280) + (rows + pass_now * 192);
        let colour_ptr_c = lw(sp + 288) + (rows + pass_now * 192);
        let colour_ptr_d = lw(sp + 292) + (rows + pass_now * 192);
        sw(sp + 128, lw(sp + 128) + 1);
        swc1(list_b, cos_b);
        let more = (lw(sp + 128) < 3) as i32;
        swc1(list_c, y0 + lwc1(lw(sp + 144)) * a);
        swc1(list_d, y_a);
        swc1(list_e + offset, one);
        swc1(list_f + offset, x_b + lwc1(lw(sp + 132)) * b);
        swc1(list_g + offset, y1 + lwc1(lw(sp + 136)) * b);
        swc1(list_j, y_b);
        swc1(list_h, one);
        swc1(list_i, x_b + lwc1(lw(sp + 140)) * b);
        swc1(lw(sp + 148), y1 + lwc1(lw(sp + 144)) * b);
        swc1(lw(sp + 152), y_b);
        swc1(lw(sp + 156), one);
        swc1(list_o + offset, one);
        swc1(list_p + offset, one);
        swc1(lw(sp + 160), one);
        swc1(lw(sp + 164), one);
        swc1(list_q, one);
        swc1(list_r + offset, one);
        swc1(list_s, one);
        swc1(list_t, one);
        sw(colour_ptr_a, colour_a);
        let shifted = lw(u1) << 16;
        let low = lw(u2);
        let mid = lw(u0) << 8;
        sw(colour_ptr_b, colour_a);
        let colour_b = low | 0x7f000000 | (shifted | mid);
        sw(colour_ptr_c, colour_b);
        sw(colour_ptr_d, colour_b);
        if more == 0 {
            break;
        }
    }
}

/// Draw the current menu: the cross hair of the player's cursor when it is set (+800), and then the box round the menu (its items measured with `menuitemCalcSize` to find the
/// corners, the title text sized with `textSize`), with a double edge, the title and every item (`menuitemGfx`) (@ 0x20cff8).
pub fn menuGfx() {
    let frame = Frame::new(224);
    let sp = frame.sp();
    let player = lw(g::currentplayer);
    if lw(player + 800) != 0 {
        let scale = lwc1(0x3ac820);
        let window = g::window;
        let height = lw(window + 36);
        let width = lw(window + 16);
        let y = lw(window + 20) + height + (lwc1(player + 816) * (height as f32 * scale) as i32 as f32) as i32;
        let x = lw(g::window) + width + (lwc1(player + 812) * (width as f32 * scale) as i32 as f32) as i32;
        dlFillRectangle(x, y - 8, x, y - 3, 0xff0080);
        dlFillRectangle(x, y + 3, x, y + 8, 0xff0080);
        dlFillRectangle(x - 15, y, x - 5, y, 0xff0080);
        dlFillRectangle(x + 5, y, x + 15, y, 0xff0080);
    }
    let menu = lw(g::curmenu);
    if menu == 0 {
        return;
    }
    let mut left = 1000;
    let mut top = 1000;
    let mut right = 0;
    let mut bottom = 0;
    let mut count = 0;
    let mut menu_now = menu;
    let mut register: u128;
    sw(sp + 32, sp + 24);
    register = 3342336_u128;
    sw(sp + 36, sp + 28);
    let title;
    if lbu(menu + 4) == 0 {
        title = lw(menu);
    } else {
        let mut last;
        loop {
            sq(sp + 48, register);
            let item = menu_now + ((count << 5) + 16);
            count += 1;
            menuitemCalcSize(item, sp);
            let item_x = lhu(item + 6);
            let item_y = lhu(item + 14);
            left = if item_x < left { item_x } else { left };
            last = lw(g::curmenu);
            let far_x = item_x + lw(sp);
            let far_y = item_y + lw(sp + 4);
            bottom = if bottom < far_y { far_y } else { bottom };
            right = if right < far_x { far_x } else { right };
            top = if item_y < top { item_y } else { top };
            menu_now = last;
            register = lq(sp + 48);
            if count >= lbu(last + 4) {
                break;
            }
        }
        title = lw(last);
    }
    sq(sp + 48, register);
    textSize(sp + 16, sp + 20, title);
    let mut item_number = 0;
    register = lq(sp + 48);
    let text_width = lw(sp + 16);
    let line = lw(register as i32 - 0x65fc);
    let across = (left + right - text_width) >> 1;
    let text_y = top - lw(sp + 20) - 4;
    sw(sp + 24, across);
    sw(sp + 28, text_y);
    let text_bottom = text_y + line;
    let text_right = across + text_width;
    let margin = lw(register as i32 - 0x6610);
    sw(sp + 28, text_bottom);
    sw(sp + 24, across + margin);
    let box_bottom = line + top;
    let box_far = line + bottom;
    let lower = text_bottom + 1;
    let box_top = box_bottom - 3;
    let wide = margin + (if right < text_right { text_right } else { right });
    let near = margin + (if across < left { across } else { left });
    let outer_right = wide + 5;
    let outer_left = near - 5;
    let frame_left = near - 6;
    let frame_right = wide + 6;
    dlFillRectangle(outer_left, lower, outer_right, box_top, 96);
    let lower_edge = box_bottom - 1;
    let low_bottom = box_far + 5;
    let frame_bottom = box_far + 6;
    dlFillRectangle(outer_left, lower_edge, outer_right, low_bottom, 96);
    dlFillRectangle(frame_left, text_bottom, frame_right, text_bottom, 0x7f0070);
    dlFillRectangle(frame_left, box_bottom - 2, frame_right, box_bottom - 2, 0x7f0070);
    dlFillRectangle(frame_left, frame_bottom, frame_right, frame_bottom, 0x7f0070);
    dlFillRectangle(frame_left, text_bottom, frame_left, frame_bottom, 0x7f0070);
    dlFillRectangle(frame_right, text_bottom, frame_right, frame_bottom, 0x7f0070);
    textPrint(lw(sp + 32), lw(sp + 36), lw(lw(g::curmenu)), 0x7f0070);
    let mut current = lw(g::curmenu);
    let mut offset = 16;
    if lbu(current + 4) != 0 {
        loop {
            menuitemGfx(current + offset);
            item_number += 1;
            current = lw(g::curmenu);
            offset += 32;
            if item_number >= lbu(current + 4) {
                break;
            }
        }
    }
}
