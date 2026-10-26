//! A bot looking for a weapon, the walls a prop carries with it, and the start of the map maker's view.

use crate::*;

/// The weapon search of the bot `enemy` (its record is at +352). The target at +2788 counts as chasing when it is a prop of kind 64 whose record starts with 4. A bot that is not
/// chasing, is not in a vehicle (+8) and has no weapon pad it holds (+2792, +684) picks one with `selectWeaponPad`; if there is none it stands (or, with the bots' bare-hand
/// patch off, plays its idle animation). Then it sets its move mode to 1 and looks for the next pad on its route (`findNextPad`): none left (-2) puts it in the state 256 with the
/// old state kept at +2720; a pad is made the next goal (+2796 to +2804) and the bot goes on to state 256 or 128 (the state before it is kept at +2720) (@ 0x2c4d70).
pub fn findweapon(enemy: i32) {
    let record = lw(enemy + 352);
    let target = lw(record + 2788);
    let mut chasing = false;
    if target != 0 && lw(target + 8) == 64 {
        chasing = lw(lw(target + 352)) == 4;
    }
    let mut new_state = 0;
    let mut old_state = 0;
    'done: {
        'written: {
            'state: {
                if !chasing {
                    new_state = 8;
                    if lw(record + 8) != 0 {
                        old_state = lw(record + 2716);
                        break 'state;
                    }
                    if lw(record + 440) == 0 {
                        let mut skip = false;
                        if lw(record + 684) >= 0 {
                            let weapon_pad = lw(record + 2792);
                            if weapon_pad == 0 || (lw(weapon_pad + 16) & 1) != 0 {
                                skip = true;
                            }
                        }
                        if !skip {
                            let pad = selectWeaponPad(enemy);
                            sw(record + 2792, pad);
                            if pad == 0 {
                                enemySetMoveMode(enemy, 8);
                                enemySetMoveMode(enemy, 512);
                                if bot_idle_punch(record) {
                                    return;
                                }
                                enemySetIdleAnimation(enemy);
                                return;
                            }
                            sw(record + 684, lw(lw(pad + 352) + 36));
                        }
                    }
                }
                enemySetMoveMode(enemy, 1);
                swc1(record + 2872, lwc1(record + 2856));
                if !chasing {
                    let mut go = true;
                    if lw(record + 332) != 0 {
                        old_state = lw(record + 2716);
                    } else if lw(record + 440) == 0 {
                        go = false;
                    } else {
                        old_state = lw(record + 2716);
                    }
                    if go {
                        new_state = 128;
                        break 'state;
                    }
                }
                let next = findNextPad(record + 680);
                if next == -2 {
                    let old = lw(record + 2716);
                    sw(record + 2716, 256);
                    sw(record + 2720, old);
                    sw(record + 2792, 0);
                    break 'written;
                }
                if next != -1 {
                    let range = lwc1(0x3ad968);
                    let pad = next * 28 + lw(g::levelpads);
                    let px = lwc1(pad + 12);
                    swc1(record + 2796, px);
                    let py = lwc1(pad + 16);
                    swc1(record + 2800, py);
                    let pz = lwc1(pad + 20);
                    swc1(record + 2804, pz);
                    let dx = ps2float::sub(lwc1(enemy + 48), px);
                    let dy = ps2float::sub(lwc1(enemy + 52), py);
                    let dz = ps2float::sub(lwc1(enemy + 56), pz);
                    if ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz)) < range {
                        let count = lw(record + 1688);
                        sw(record + 680, next);
                        sw(record + 1688, count - 1);
                    }
                    let flags = lw(record + 2704);
                    new_state = 256;
                    if (flags & 1) == 0 {
                        break 'done;
                    }
                    old_state = lw(record + 2716);
                } else {
                    new_state = 128;
                    if !chasing {
                        sw(record + 684, next);
                        break 'done;
                    }
                    old_state = lw(record + 2716);
                }
            }
            sw(record + 2716, new_state);
            sw(record + 2720, old_state);
        }
        sw(record + 2892, 0);
    }
}

/// Make the walls of the prop `prop` (its table at +484, from the model's original walls) in the world: each wall's two corner points (and their heights) are moved to the
/// prop's place by the matrix of its place and angle, and when `by_bones` is set first by the matrix of its bone; for the prop model 311 the lowest and highest heights of all
/// the walls are then put on all of them (@ 0x267338).
pub fn propCalculateWalls(prop: i32, by_bones: i32) {
    let frame = Frame::new(112);
    let sp = frame.sp();
    let mut lowest = lwc1(0x3af8f8);
    let mut highest = lwc1(0x3af8fc);
    if lw(prop + 484) == 0 {
        return;
    }
    propGetOriginalWalls(lw(prop + 4), sp + 96, sp + 100);
    let mut index = 0;
    matrixTransRotY(sp, lwc1(prop + 48), lwc1(prop + 52), lwc1(prop + 56), ps2float::mul(lwc1(prop + 76), lwc1(0x3acbc4)).pdiv(1.8e+02_f32));
    let mut originals = lw(sp + 96);
    let mut original = lw(originals);
    let mut offset = 0;
    let kind;
    if original == 0 {
        kind = lw(prop + 4);
    } else {
        let mut first = lwc1(original + 4);
        loop {
            let low = lwc1(original + 8);
            let high = lwc1(original + 16);
            swc1(sp + 64, first);
            let walls = lw(prop + 484);
            swc1(sp + 68, lwc1(original + 8));
            let wall = lw(offset + walls);
            swc1(sp + 72, lwc1(original + 12));
            swc1(sp + 80, lwc1(original + 20));
            swc1(sp + 84, lwc1(original + 8));
            swc1(sp + 88, lwc1(original + 28));
            if by_bones != 0 {
                let model = lw(prop + 32);
                matrixVecMulAligned(&[(lw(model + 4) + (lw(offset + lw(sp + 100)) << 6)) as i64, (sp + 64) as i64]);
                let model = lw(prop + 32);
                matrixVecMulAligned(&[(lw(model + 4) + (lw(offset + lw(sp + 100)) << 6)) as i64, (sp + 80) as i64]);
            }
            matrixVecMulAligned(&[sp as i64, (sp + 64) as i64]);
            matrixVecMulAligned(&[sp as i64, (sp + 80) as i64]);
            let number = lhu(original);
            swc1(wall + 4, lwc1(sp + 64));
            let current_kind = lw(prop + 4);
            let bottom = lwc1(sp + 68);
            sh(wall, number);
            swc1(wall + 24, bottom);
            swc1(wall + 8, bottom);
            swc1(wall + 12, lwc1(sp + 72));
            swc1(wall + 20, lwc1(sp + 80));
            let top = ps2float::add(lwc1(sp + 84), ps2float::sub(high, low));
            swc1(wall + 32, top);
            swc1(wall + 16, top);
            swc1(wall + 28, lwc1(sp + 88));
            if current_kind == 311 {
                if bottom < lowest {
                    lowest = bottom;
                }
                if highest < top {
                    highest = top;
                }
            }
            index += 1;
            originals = lw(sp + 96);
            offset = index << 2;
            original = lw(offset + originals);
            if original == 0 {
                kind = current_kind;
                break;
            }
            first = lwc1(original + 4);
        }
    }
    if kind == 311 {
        let mut i = 0;
        if lw(originals) != 0 {
            let walls = lw(prop + 484);
            let mut at = 0;
            loop {
                i += 1;
                let wall = lw(at + walls);
                at = i << 2;
                swc1(wall + 16, highest);
                let next = lw(at + originals);
                swc1(wall + 24, lowest);
                swc1(wall + 8, lowest);
                swc1(wall + 32, highest);
                if next == 0 {
                    break;
                }
            }
        }
    }
}

/// Start the map maker's view: take the window's size and place (`w_w`, `w_x`, `w_y`, `w_h`), find the map's bounds, make the map 0x2000000 wide in the map's units, as tall as
/// the window's proportion gives, centred on 0, and work out the scales between the map's and the window's units (@ 0x2498c8).
pub fn mmInitView() {
    let window = g::window;
    let x = lw(g::window);
    let height = lw(window + 32);
    let y = lw(window + 20);
    sw(g::w_w, lw(window + 12));
    sw(g::w_x, x);
    sw(g::w_y, y);
    sw(g::w_h, height);
    mmFindBoundaries();
    let window_w = lw(g::w_w);
    let window_h = lw(g::w_h) << 17;
    sw(g::m_w, 0x2000000);
    let ratio = FDiv(window_h, window_w << 16);
    let map_w = lw(g::m_w);
    let wide = lw(g::w_w) << 16;
    let map_h = FMul(map_w, ratio as i32);
    let left = 0 - ((map_w + ((map_w as u32) >> 31) as i32) >> 1);
    sw(g::m_h, map_h as i32);
    sw(g::m_x2, left + map_w);
    sw(g::m_x, left);
    let top = 0 - ((map_h as i32 + ((map_h as i32 as u32) >> 31) as i32) >> 1);
    sw(g::m_y, top);
    sw(g::m_y2, top + map_h as i32);
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
