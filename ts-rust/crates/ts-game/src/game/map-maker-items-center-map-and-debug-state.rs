//! Drawing the items of a tile in the map maker, fitting the map to the window, and the debug text about the characters.

use crate::*;

/// Draw the items of the tile view `view` (the tile at +36, its turn at +24, its levels from +20, the item list at +52 and the place of the tile at +12, +16): for each item of the
/// list (a halfword number at +0, the item kind at +2, the next at +4) look up its place in the tile's pad data (28-byte records), turn it by the tile's turn, and when it is in the
/// height range of the shown level (4 high) draw the item's picture (the 16 bytes records of `itemdefs` from +8) at the map's place scaled to the window (@ 0x249b90).
pub fn mmDrawItems(view: i32) {
    let frame = Frame::new(32);
    let sp = frame.sp();
    let mut across = 0;
    let mut down = 0;
    let tile = lw(view + 36);
    let turn = lw(view + 24);
    let pads = lw(((((tile - g::tileset1).wrapping_mul(0x4f72c235)) >> 2) << 2) + g::tilepaddata);
    let level = (lw(g::editlevel) - lw(view + 20)) as f32 * 4.0_f32 - 0.5_f32;
    let records = pads + 12;
    let count = lw(pads + 4);
    let mut item = lw(view + 52);
    if turn == 1 {
        across = lw(tile + 4);
    } else if turn >= 2 {
        if turn == 2 {
            down = lw(tile + 4);
            across = lw(tile);
        } else if turn == 3 {
            down = lw(tile);
        }
    }
    let across = across << 21;
    let down = down << 21;
    if item != 0 {
        let scale = lwc1(0x3aca58);
        let mut number = lhu(item);
        loop {
            let mut found = 0;
            if lw(records) != number && count > 0 {
                let mut at = records + 28;
                loop {
                    found += 1;
                    if lw(at) == number {
                        break;
                    }
                    at += 28;
                    if found >= count {
                        break;
                    }
                }
            }
            let record = found * 28 + records;
            swc1(sp, lwc1(record + 12));
            let height = lwc1(record + 16);
            swc1(sp + 4, height);
            swc1(sp + 8, lwc1(record + 20));
            if !(height < level) && !((level + 4.0_f32) < height) {
                let defs = g::itemdefs + 8;
                let picture = lw((lhu(item + 2) << 4) + defs);
                if lw(view + 24) != 0 {
                    let matrices = g::tilerotmtx;
                    matrixVecMulAligned(&[(((lw(view + 24)) << 6) + matrices) as i64, sp as i64]);
                }
                let scale_y = lw(g::scale_mtowy);
                let scale_x = lw(g::scale_mtowx);
                let x = lw(view + 12) + (lwc1(sp) * scale) as i32 + across - lw(g::m_x);
                let y = lw(view + 16) + (lwc1(sp + 8) * scale) as i32 + down - lw(g::m_y);
                let screen_x = (lw(g::window) + (lw(g::w_x) + ((FMul(x, scale_x) as i32) >> 16)) - 2) as f32;
                let screen_y = (lw(g::window + 20) + (lw(g::w_y) + ((FMul(y, scale_y) as i32) >> 16)) - 1) as f32;
                guiDrawImage(14, picture, screen_x, screen_y);
            }
            item = lw(item + 4);
            if item == 0 {
                break;
            }
            number = lhu(item);
        }
    }
}

/// Fit the map to the window: find the map's bounds, make the map 0x2000000 wide in the map's units and as tall as the window's proportion gives, put its middle in the window's
/// middle (`m_x`, `m_y`, `m_x2`, `m_y2`), and work out the scales between map and window units (`scale_mtowx`, `scale_mtowy` and their inverses) (@ 0x249628).
pub fn mmCenterMap() {
    mmFindBoundaries();
    let window_w = lw(g::w_w);
    let window_h = lw(g::w_h) << 17;
    sw(g::m_w, 0x2000000);
    let ratio = FDiv(window_h, window_w << 16);
    let map_w = lw(g::m_w);
    let wide = lw(g::w_w) << 16;
    let max_y = lw(g::max_y);
    let map_h = FMul(map_w, ratio as i32);
    let sum_y = lw(g::min_y) + max_y;
    let sum_x = lw(g::min_x) + lw(g::max_x);
    sw(g::m_h, map_h as i32);
    let y = ((sum_y + ((sum_y as u32) >> 31) as i32) >> 1) - ((map_h as i32 + ((map_h as i32 as u32) >> 31) as i32) >> 1);
    let x = ((sum_x + ((sum_x as u32) >> 31) as i32) >> 1) - ((map_w + ((map_w as u32) >> 31) as i32) >> 1);
    sw(g::m_x2, x + map_w);
    sw(g::m_y2, y + map_h as i32);
    sw(g::m_x, x);
    sw(g::m_y, y);
    let to_window_x = FDiv(wide, map_w);
    let map_h = lw(g::m_h);
    let high = lw(g::w_h) << 16;
    sw(g::scale_mtowx, to_window_x as i32);
    let to_window_y = FDiv(high, map_h);
    let scale_x = lw(g::scale_mtowx);
    sw(g::scale_mtowy, to_window_y as i32);
    let back_x = FDiv(0x10000, scale_x);
    let scale_y = lw(g::scale_mtowy);
    sw(g::scale_wtomx, back_x as i32);
    let back_y = FDiv(0x10000, scale_y);
    sw(g::scale_wtomy, back_y as i32);
}

/// Draw the debug text about each active character (not the dead ones or those with state 0x2000): at heights going down by 10 from 105 its number, state and name; at 195 its
/// place and angle; at 395 its flags, team and the kind (@ 0x2cc778).
pub fn debugPrintStateInfo() {
    let frame = Frame::new(128);
    let sp = frame.sp();
    let mut shown = 0;
    let mut i = 0;
    let old = dlSetZB(3);
    let mut players = lw(g::lvnumplayers);
    let mut active = lw(g::numactivechars);
    sw(sp + 120, old);
    if (players + active) > 0 {
        loop {
            let character = i * 0x1210 + lw(g::chrinfo);
            let state = lw(character + 2716);
            let prop = lw(character + 3020);
            if state != 0 && state != 0x2000 {
                let line = shown * 10;
                sw(sp + 112, 0);
                shown += 1;
                sw(sp + 116, line + 105);
                sprintf(&[sp as i64, 0x3a9f78, lw(character) as i64, lw(character + 2728) as i64, (character + 2736) as i64]);
                textPrint(sp + 112, sp + 116, sp, 0x7f7f7f7f);
                let x = fptodp(lwc1(prop + 48));
                let y = fptodp(lwc1(prop + 52));
                let z = fptodp(lwc1(prop + 56));
                let angle = fptodp(lwc1(prop + 80));
                sprintf(&[sp as i64, 0x3a9f88, x, y, z, angle]);
                sw(sp + 112, 195);
                textPrint(sp + 112, sp + 116, sp, 0x7f7f7f7f);
                sw(sp + 112, 395);
                let team = lw(character + 684);
                let kind = lw(prop + 12);
                let flag = ((0 as u32) < ((lw(prop + 16) & 0xf000000) as u32)) as i32;
                let counter = lw(character + 2772);
                let side = lw(character + 680);
                sprintf(&[sp as i64, 0x3a9fa0, kind as i64, flag as i64, counter as i64, side as i64, team as i64]);
                textPrint(sp + 112, sp + 116, sp, 0x7f7f7f7f);
                players = lw(g::lvnumplayers);
                active = lw(g::numactivechars);
            }
            i += 1;
            if i >= players + active {
                break;
            }
        }
    }
    dlSetZB(lw(sp + 120));
}
