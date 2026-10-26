//! Functions: reloading a gun, the cheats page's tick, a bullet's line test for a shot that does not travel straight, the damage stages of a destructible prop, the main page of
//! the map maker, the z-buffer test drawing of flares, the drawing of a tile brick in the map maker's tile selection and the check that an enemy's pad is still the nearest valid one.

use crate::*;

/// Reload the gun combo `a1` of the player `a0` into the gun state `a2` (@ 0x28db90). When the clip is empty and cannot be filled, the combo is changed to one that has ammo
/// unless `a3` is set; `a4` chooses which of the two hands' ammo counts is checked.
pub fn gunReload(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32) {
    let gun = g::guns + lw(a1) * 400;
    let slot = lw(gun + 16);
    let limit = lw(a2 + 152);
    if limit != -1 {
        if lw(gun + 396) >= limit {
            return;
        }
    }
    let clip = lw(gun + 20);
    if clip != -1 {
        let offset = slot << 2;
        if clip != 0 {
            if lw(a0 + offset + 620) != 0 {
                if gunReloadCore(a0, a1) != 0 {
                    if a2 != 0 {
                        if lw(a1) != 24 {
                            sw(a2, 7);
                            sw(a2 + 4, 0);
                        } else {
                            sw(a2, 9);
                            let first = propNew(222, 0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32);
                            let player = lw(g::currentplayer);
                            sw(first + 140, 2);
                            sw(first + 136, 2);
                            let id = lw(lw(player));
                            let flags = lw(first + 16) | 0x40000000;
                            sw(a2 + 140, first);
                            sw(first + 16, flags);
                            sw(first + 12, id + 201);
                            let second = propNew(222, 0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32);
                            let player = lw(g::currentplayer);
                            sw(second + 136, 2);
                            sw(second + 140, 2);
                            let id = lw(player);
                            let flags = lw(second + 16) | 0x40000000;
                            sw(a2 + 144, second);
                            sw(second + 16, flags);
                            sw(second + 12, id + 201);
                            updateShotgunReload(a2);
                            sw(a2 + 4, 0);
                        }
                        soundStart(138, 0);
                    }
                }
            } else {
                let mut empty = 0;
                if gunBothHandsActive(a0) != 0 {
                    let ammo = if a4 == 0 { lw(a0 + offset + 272) } else { lw(a0 + offset + 380) };
                    empty = (ammo < 1) as i32;
                }
                if a3 == 0 {
                    let both = gunBothHandsActive(a0);
                    if both != 0 {
                        if empty == 0 {
                            return;
                        }
                    }
                    changeToComboWithAmmo(a0);
                }
            }
        }
    } else if lw(a2 + 160) != 0 {
        if lw(a0 + (slot << 2) + 620) != 0 {
            sw(a2 + 4, 0);
            sw(a2, 7);
            if lw(a1) != 8 {
                soundStart(138, 0);
                return;
            }
            soundStart(215, 0);
            return;
        }
        if a3 == 0 {
            changeToComboWithAmmo(a0);
        }
    }
}

/// Tick of the cheats page: list the cheats that are unlocked (and allowed in the current mode) as menu items with a check box for each selected one, then run the menu
/// (@ 0x2398c0).
pub fn cheats_pageTick(a0: i32) {
    let frame = Frame::new(176);
    let sp = frame.sp();
    let allowed = if (lw(0x1fc11fc) & 64) == 0 { cheatsStory() } else { cheatsArcade() };
    let language = lw(g::cur_language);
    let title = if (lw(0x1fc11fc) & 64) == 0 {
        lw(lw((language << 2) + g::languages) + 936)
    } else {
        lw(lw((language << 2) + g::languages) + 932)
    };
    let mut items = g::cheats_mi;
    sw(((lw(0x3aff2c) - 1) << 3) + 0x1fc1308, 0);
    pageSet(title, g::cheats_page);
    frontfxTick(0);
    memset(items, 0, 480);
    let unlocked = lw(g::unlockedcheats);
    let mut bit = 0;
    let mut count = 0;
    loop {
        if (allowed & (unlocked & (1 << (bit & 31)))) != 0 {
            sh(items + 8, count + 1000);
            count += 1;
            items += 32;
        }
        bit += 1;
        if bit >= 15 {
            break;
        }
    }
    let total = count;
    let mut items = g::cheats_mi;
    let last = total - 1;
    sw(sp + 4, (total < 16) as i32);
    sw(sp, unlocked);
    let first_item = items;
    let last_item = (last << 5) + items;
    let mut defs = g::cheatdefs;
    let mut boxes = 0x1fc2d98;
    let mut bit = 0;
    let mut index = 0;
    loop {
        let language = lw(g::cur_language);
        if (allowed & (lw(sp) & (1 << (bit & 31)))) != 0 {
            let name_offset = lh(defs) << 2;
            let names = lw((language << 2) + g::languages);
            let chosen = lw(g::selectedcheats) & unlocked;
            let template = g::gv_cheat;
            let word1 = ld(template + 8);
            let word2 = ld(template + 16);
            sd(boxes, ld(template));
            sd(boxes + 8, word1);
            sd(boxes + 16, word2);
            let word4 = ld(template + 32);
            sd(boxes + 24, ld(template + 24));
            sd(boxes + 32, word4);
            let name = lw(name_offset + names);
            sw(boxes, if ((chosen >> (bit & 31)) & 1) == 0 { 0 } else { 1 });
            sw(items, name);
            sh(items + 4, 0);
            sh(items + 6, 0);
            sh(items + 10, 4);
            sw(items + 12, boxes);
            let previous_limit;
            if index <= 0 {
                previous_limit = last;
                sh(items + 16, lhu(last_item + 8));
            } else {
                previous_limit = total - 1;
                sh(items + 16, lhu(((index - 1) << 5) + first_item + 8));
            }
            let has_next = (index < previous_limit) as i32;
            let next = index + 1;
            if has_next == 0 {
                sh(items + 18, lhu(first_item + 8));
            } else {
                sh(items + 18, lhu((next << 5) + first_item + 8));
            }
            index = next;
            items += 32;
        }
        bit += 1;
        defs += 12;
        boxes += 40;
        if bit >= 15 {
            break;
        }
    }
    let menu = g::cheats_menu;
    sh(g::cheats_list + 2, if lw(sp + 4) != 0 { total } else { 15 });
    sh(g::cheats_list, total);
    menutick_230238(a0, menu);
    let made = std_menumake(lw(a0 + 8), menu);
    sw(a0 + 12, made);
}

/// Line test of a bullet `a0` that moves in more than one segment (@ 0x28a1d0). When the bullet was just fired from a gun (`+216` set) the first segment starts from the
/// muzzle: the line is tested from the gun's position to the bullet's, with the owner's prop made invisible to the test; later segments test from the bullet's
/// previous position to the destination `a1`. Returns the result of the last `bgLineTest`.
pub fn bulletNonStraightLineTest(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32) -> i32 {
    let frame = Frame::new(240);
    let sp = frame.sp();
    sw(sp + 64, a1);
    let mut owner = lw(a0 + 220);
    let z = lwc1(a0 + 28);
    let x = lwc1(a0 + 24);
    let y = lwc1(a0 + 32);
    let kind = lw(a0);
    sw(sp + 68, lw(owner + 16));
    swc1(sp + 16, x);
    swc1(sp + 24, y);
    swc1(sp + 20, z);
    if kind == 19 {
        swc1(sp + 20, ps2float::sub(z, lwc1(0x3ad1e8)));
    }
    let flags = if 2 < ((kind - 14) as u32) { 0 } else { 4 };
    let s2 = a0 + 100;
    let room;
    if lw(a0 + 216) == 0 {
        room = lw(a0 + 100);
    } else {
        let source = lw(a0 + 104);
        let mut hidden = 0;
        let mut hidden_flags = 0;
        let start_room;
        sw(a0 + 216, 0);
        'start: {
            if source != 1 {
                if source != 2 {
                    if source == 3 {
                        remoteGetPivotePos(lw(a0 + 220), sp + 32);
                        owner = lw(a0 + 220);
                        let remote = lw(owner + 352);
                        hidden = lw(remote + 96);
                        start_room = lw(remote + 120);
                        hidden_flags = lw(hidden + 16);
                        sw(hidden + 16, hidden_flags & -33);
                        break 'start;
                    }
                    let px = lwc1(owner + 48);
                    start_room = lw(owner + 12);
                    swc1(sp + 32, px);
                    swc1(sp + 36, lwc1(owner + 52));
                    let pz = lwc1(owner + 56);
                    swc1(sp + 40, pz);
                } else {
                    let player = lw(owner + 352);
                    start_room = lw(owner + 12);
                    swc1(sp + 32, lwc1(player + 2808));
                    swc1(sp + 36, lwc1(player + 2812));
                    let pz = lwc1(player + 2816);
                    swc1(sp + 40, pz);
                }
            } else {
                let chr = lw(owner + 356);
                start_room = lw(chr + 796);
                swc1(sp + 32, lwc1(chr + 152));
                swc1(sp + 36, lwc1(chr + 156));
                swc1(sp + 40, lwc1(chr + 160));
            }
        }
        let dy = ps2float::sub(lwc1(sp + 20), lwc1(sp + 36));
        let owner_flags = lw(owner + 16);
        let dz = ps2float::sub(lwc1(sp + 24), lwc1(sp + 40));
        swc1(sp + 48, ps2float::sub(lwc1(sp + 16), lwc1(sp + 32)));
        swc1(sp + 52, dy);
        sw(owner + 16, owner_flags & -33);
        swc1(sp + 56, dz);
        sw(sp, flags);
        sw(sp + 8, a4);
        let hit = bgLineTest(start_room, sp + 32, sp + 48, 0, a2, a3, s2, 112);
        if hidden != 0 {
            sw(hidden + 16, hidden_flags);
        }
        if hit != 0 {
            sw(lw(a0 + 220) + 16, lw(sp + 68));
            return hit;
        }
        if a4 == 0 {
            room = lw(a0 + 100);
        } else {
            let glass = lw(a4);
            if glass == 0 {
                room = lw(a0 + 100);
            } else {
                bulletProcessGlasshits(a0, glass);
                room = lw(a0 + 100);
            }
        }
    }
    sw(sp, flags);
    sw(sp + 8, a4);
    let hit = bgLineTest(room, sp + 16, lw(sp + 64), 0, a2, a3, s2, 112);
    let owner_prop = lw(a0 + 96);
    sw(lw(a0 + 220) + 16, lw(sp + 68));
    if owner_prop != 0 {
        sw(owner_prop + 12, lw(a0 + 100));
    }
    hit
}

/// Advance the damage stage of a destructible prop `a0` from how much damage it has taken and, when the stage changed, explode it (@ 0x274bb8): the first stage flag set in the
/// definition's stage word moves up one place at each of the damage thresholds. A prop of the kind 201 explodes with a bang and a burst, one of the kind 202 with a random
/// boom. The arguments `a3`, `a4`, `a5` and `a1` are passed on to the explosion particle.
pub fn baseDamage(a0: i32, a1: i32, _a2: i32, a3: i32, a4: i32, a5: i32) {
    let frame = Frame::new(128);
    let sp = frame.sp();
    let mut damage = lwc1(a0 + 532);
    let mut changed = 0;
    if 1.0_f32 < damage {
        let flags = lw(a0 + 16);
        swc1(a0 + 532, 1.0_f32);
        damage = 1.0_f32;
        sw(a0 + 16, flags & -33);
    }
    let mut stage: Option<(i32, i32)> = None;
    if lwc1(0x3acda0) < damage {
        let def = lw(a0 + 32);
        let bits = lhu(def + 292);
        if (bits & 2) != 0 {
            stage = Some((def, bits & 0xfffd | 4));
        }
    }
    if stage.is_none() {
        if 0.5_f32 < damage {
            let def = lw(a0 + 32);
            let bits = lhu(def + 292);
            if (bits & 4) != 0 {
                stage = Some((def, bits & 0xfffb | 8));
            }
        }
    }
    if stage.is_none() {
        if lwc1(0x3acda4) < damage {
            let def = lw(a0 + 32);
            let bits = lhu(def + 292);
            if (bits & 8) != 0 {
                stage = Some((def, bits & 0xfff7 | 16));
            }
        }
    }
    if stage.is_none() {
        if 1.0_f32 <= damage {
            let def = lw(a0 + 32);
            let bits = lhu(def + 292);
            if (bits & 16) != 0 {
                stage = Some((def, bits & 0xffef | 32));
            }
        }
    }
    if let Some((def, bits)) = stage {
        sh(def + 292, bits);
        decalRemoveDecalsFromProp(a0);
        changed = 1;
    }
    if changed != 0 {
        let y = ps2float::add(lwc1(a0 + 512), lwc1(a0 + 52));
        let kind = lw(a0 + 4);
        let z = ps2float::add(lwc1(a0 + 516), lwc1(a0 + 56));
        swc1(sp, ps2float::add(lwc1(a0 + 508), lwc1(a0 + 48)));
        swc1(sp + 4, y);
        swc1(sp + 8, z);
        if kind != 201 {
            if kind == 202 {
                let random = newrnd();
                soundStart(((random as i32 as u32) % (6 as u32)) as i32 + 98, sp);
                particleNewExplosion(0, 5, sp, a3, a4, a5, a1, 0);
                if lw(g::curgameparms + 72) == 6 {
                    particleNewBurst(0, 18, sp, 1, lwc1(a0 + 52));
                }
            }
        } else {
            soundStart(24, 0);
            soundDelayStartEx(25, 0, 10, 0);
            particleNewExplosion(0, 2, sp, a3, a4, a5, a1, 0);
            if lw(g::curgameparms + 72) == 6 {
                particleNewBurst(0, 19, sp, 1, lwc1(a0 + 52));
            }
        }
    }
}

/// Tick of the main page of the map maker (@ 0x24ef40). Items are greyed out while the map is empty or nothing can be undone; a button press on one of the enabled items
/// undoes the map, centres it, asks for a confirmation of the exit or the new map, toggles the directory flag or opens the test run.
pub fn mmmain_pageTick(a0: i32) {
    let page = g::mmmain_menu;
    let item = menuitem(page, lhu(a0 + 4));
    let flags = lw(g::editflags);
    let objects = lw(g::nummapobjects);
    if (lw(g::editflags) & 0x8000) == 0 {
        let menu = g::m0100;
        let (first, second) = if objects <= 0 {
            (lhu(menu + 42) | 64, lhu(menu + 234) | 64)
        } else {
            (lhu(menu + 42) & 0xffbf, lhu(menu + 234) & 0xffbf)
        };
        sh(menu + 42, first);
        sh(menu + 234, second);
        let menu = g::m0100;
        let third = if (flags & 256) == 0 { lhu(menu + 74) | 64 } else { lhu(menu + 74) & 0xffbf };
        sh(menu + 74, third);
        if (flags & 8) != 0 {
            if (joyNewBut(0) & 2320) != 0 {
                mmExitMenu(a0);
                return;
            }
        }
        if (joyNewBut(0) & 64) == 0 {
        } else if (lhu(item + 10) & 64) != 0 {
        } else {
            let action = lhu(a0 + 4) - 3;
            if (action as u32) < (8 as u32) {
                let mut new_flags = 0;
                match action {
                    0 => {
                        mmUndoMap();
                        mmExitMenu(a0);
                        return;
                    }
                    1 => {
                        mmCenterMap();
                        mmExitMenu(a0);
                        return;
                    }
                    2 => {
                        let message = lw(lw((lw(g::cur_language) << 2) + g::languages) + 3988);
                        mmStartMessage(message, 0x647f7f7f, 0x24ee98);
                    }
                    3 => {}
                    4 => {
                        mmfileDirReset();
                        let current = lw(g::editflags);
                        sb(0x3b0038, 0);
                        new_flags = current | 2048;
                        sw(g::editflags, new_flags);
                    }
                    5 => {
                        mmfileDirReset();
                        let current = lw(g::editflags);
                        sb(0x3b0038, 0);
                        new_flags = current & -2049;
                        sw(g::editflags, new_flags);
                    }
                    6 => {
                        let message = lw(lw((lw(g::cur_language) << 2) + g::languages) + 3992);
                        mmStartMessage(message, 0x6464647f, 0);
                    }
                    _ => {
                        let current = lw(g::editflags);
                        if lw(g::nummapobjects) > 0 && (current & 0x1000) != 0 {
                            let message = lw(lw((lw(g::cur_language) << 2) + g::languages) + 3996);
                            mmStartMessage(message, 0x647f7f7f, 0x24ee50);
                        } else {
                            new_flags = current | 0x8000;
                            sw(g::editflags, new_flags);
                        }
                    }
                }
                let _ = new_flags;
            }
        }
        let menu = g::mmmain_menu;
        menutick_24c3a8(a0, menu);
        let made = menumake(lw(a0 + 8), menu);
        sw(a0 + 12, made);
        return;
    }
    let front = lw(g::front);
    sw(front + 12, 0);
    sw(front + 20, 0);
    exitmapmaker();
}

/// Draw the flares, glow sprites and torches of the current player with the z-buffer test: first the glow sprite of every light of the player (a second, smaller one for a
/// tinted light), then the flares and the torches (@ 0x2a6b78).
pub fn zbtestGfx() {
    let _hk = world_sprite_guard();
    let texture = lw(g::glowtexnum);
    let mut i = 0;
    let info = texture * 40 + lw(lw(g::thistexbuf) + 4);
    let tex_h = lhu(info + 2);
    let tex_w = lhu(info);
    dlSelectTextureKick(texture);
    dlSetZB(0);
    dlSetDitherMatrix(2);
    let mut count = lw(0x3b017c);
    if count > 0 {
        loop {
            let light = i * 28 + 0x1fea0b0;
            if lw(light) != lw(lw(g::currentplayer)) {
                i += 1;
            } else {
                let radius = lw(light + 20) as f32;
                let x = lwc1(light + 12);
                let u1 = tex_w << 4;
                let y = lwc1(light + 16);
                let v1 = tex_h << 4;
                let high = ps2float::mul(radius, lwc1(0x3ad61c));
                let colour = lw(light + 24);
                let alpha = colour & 255;
                dlTextureRectangleFloat(0, 0, u1, v1, colour, ps2float::sub(x, radius), ps2float::sub(y, high), ps2float::add(x, radius), ps2float::add(y, high));
                let tint = 0x7f7f7f00;
                let inner_colour = alpha | tint;
                if (lw(light + 24) - alpha) != tint {
                    let scale = 0.75_f32;
                    let y = lwc1(light + 16);
                    let inner_high = ps2float::mul(high, scale);
                    let x = lwc1(light + 12);
                    let inner_radius = ps2float::mul(lw(light + 20) as f32, scale);
                    dlTextureRectangleFloat(0, 0, u1, v1, inner_colour, ps2float::sub(x, inner_radius), ps2float::sub(y, inner_high), ps2float::add(x, inner_radius), ps2float::add(y, inner_high));
                }
                count = lw(0x3b017c);
                i += 1;
            }
            if i >= count {
                break;
            }
        }
    }
    i = 0;
    dlSetBlend(2);
    let mut count = lw(0x3b0180);
    if count > 0 {
        loop {
            let flare = i * 24 + 0x1fea740;
            if lw(flare) != lw(lw(g::currentplayer)) {
                i += 1;
            } else {
                flareGfx(flare);
                count = lw(0x3b0180);
                i += 1;
            }
            if i >= count {
                break;
            }
        }
    }
    i = 0;
    dlSetDitherMatrix(1);
    dlSetZB(3);
    let mut count = lw(0x3b0184);
    if count > 0 {
        loop {
            if lw((i << 5) + 0x1fea860) != lw(lw(g::currentplayer)) {
                i += 1;
            } else {
                torchGfx();
                count = lw(0x3b0184);
                i += 1;
            }
            if i >= count {
                break;
            }
        }
    }
}

/// Draw one brick of the tile selection of the map maker (@ 0x251be0): the window rectangle of the brick `a0` is filled, a grid is drawn over the area of the tile and the
/// tile itself on top of it, with the map scales temporarily set to a fixed zoom. The brick that is the current selection is drawn offset and highlighted.
pub fn draw_selectbrick(a0: i32) {
    let frame = Frame::new(256);
    let sp = frame.sp();
    let front = lw(g::front);
    let tile_id = lhu(a0 + 8);
    let saved_mtowy = lw(g::scale_mtowy);
    let saved_wtomx = lw(g::scale_wtomx);
    let saved_wtomy = lw(g::scale_wtomy);
    let selected = ((lhu((lw(front + 20) << 4) + lw(front + 4) - 12) ^ tile_id) as u32 == 0) as i32;
    sw(sp, lw(g::scale_mtowx));
    sw(sp + 84, selected);
    sw(sp + 4, saved_mtowy);
    sw(sp + 8, saved_wtomx);
    sw(sp + 12, saved_wtomy);
    sw(g::scale_mtowx, 0x8000);
    sw(g::scale_mtowy, 0x4000);
    let back_x = FDiv(0x10000, 0x8000);
    let scale_y = lw(g::scale_mtowy);
    sw(g::scale_wtomx, back_x as i32);
    let back_y = FDiv(0x10000, scale_y);
    let saved_objects = lw(g::mapobjects);
    sw(g::scale_wtomy, back_y as i32);
    sw(sp + 80, saved_objects);
    sw(g::mapobjects, 0);
    let left = lhu(a0);
    let top = lhu(a0 + 2);
    dlFillRectangle(left, top, left + lhu(a0 + 4), top + lhu(a0 + 6), 0xa0a1480);
    let scale_x = lw(g::scale_wtomx);
    let wx = (lhu(a0) - lw(g::w_x)) << 16;
    let moved_x = FMul(wx, scale_x);
    let scale_y = lw(g::scale_wtomy);
    let mut x = lw(g::m_x).wrapping_add(moved_x as i32);
    let wy = (lhu(a0 + 2) - lw(g::w_y)) << 16;
    let moved_y = FMul(wy, scale_y);
    let mut y = lw(g::m_y).wrapping_add(moved_y as i32);
    if lw(sp + 84) != 0 {
        x = x.wrapping_sub(10);
        y = y.wrapping_sub(4);
    }
    let table = (tile_id - 200) << 2;
    let tile = lw(table + g::tiles);
    let tile_w = lw(tile);
    let tile_h = lw(tile + 4);
    let x_end = x.wrapping_add((if 2 >= tile_w { 3 } else { tile_w }) << 21);
    let height = (if 2 >= tile_h { 3 } else { tile_h }) << 21;
    let mut line;
    let y_end;
    if x_end < x {
        y_end = y.wrapping_add(height);
        line = y;
    } else {
        y_end = y.wrapping_add(height);
        let mut at = x;
        loop {
            mmDrawLine(at, y, at, y_end, 0x64003f);
            at = at.wrapping_add(0x200000);
            if x_end < at {
                break;
            }
        }
        line = y;
    }
    let mut tiles = g::tiles;
    if y_end >= y {
        loop {
            mmDrawLine(x, line, x_end, line, 0x64003f);
            line = line.wrapping_add(0x200000);
            if y_end < line {
                break;
            }
        }
        tiles = g::tiles;
    }
    let tile = lw(table + tiles);
    sw(sp + 36, lw(g::editlevel));
    mmDrawTile(x, y, tile, lw(sp + 84) << 1, sp + 16);
    let mtowx = lw(sp);
    let mtowy = lw(sp + 4);
    let wtomx = lw(sp + 8);
    let wtomy = lw(sp + 12);
    sw(g::mapobjects, lw(sp + 80));
    sw(g::scale_mtowx, mtowx);
    sw(g::scale_mtowy, mtowy);
    sw(g::scale_wtomx, wtomx);
    sw(g::scale_wtomy, wtomy);
}

/// Check that the pad an enemy `a0` is using is still the nearest valid one (@ 0x2bf920): the nearest pad to the enemy is compared to its current pad and, when it is further
/// than 2.0 closer and the enemy has a clear line to it, the enemy switches to it.
pub fn enemyCheckValidPad(a0: i32) {
    let frame = Frame::new(208);
    let sp = frame.sp();
    let chr = lw(g::chrinfo) + a0 * 0x1210;
    let prop = lw(chr + 3020);
    if lw(chr + 2716) != 0x610000 {
        let x = lwc1(prop + 48);
        let y;
        if a0 >= lw(g::lvnumplayers) {
            swc1(sp + 32, x);
            y = lwc1(chr + 0x1198);
        } else {
            swc1(sp + 32, x);
            y = lwc1(prop + 52);
        }
        swc1(sp + 36, y);
        swc1(sp + 40, lwc1(prop + 56));
        let nearest = closestpad(sp + 32, 0.0_f32);
        let mut next = -1;
        let current;
        if lw(chr + 1688) <= 0 {
            current = lw(chr + 680);
        } else {
            next = findNextPad(chr + 680);
            current = lw(chr + 680);
        }
        if nearest != current {
            if nearest != next {
                if nearest > 0 {
                    let pads = lw(g::levelpads);
                    let px = lwc1(sp + 32);
                    let py = lwc1(sp + 36);
                    let pz = lwc1(sp + 40);
                    let pad = nearest * 28 + pads;
                    let dy = ps2float::sub(py, lwc1(pad + 16));
                    let dx = ps2float::sub(px, lwc1(pad + 12));
                    let dz = ps2float::sub(pz, lwc1(pad + 20));
                    let old = current * 28 + pads;
                    let old_dy = ps2float::sub(py, lwc1(old + 16));
                    let old_dx = ps2float::sub(px, lwc1(old + 12));
                    let old_dz = ps2float::sub(pz, lwc1(old + 20));
                    let gain = ps2float::sub(ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz)).abs().psqrt(), ps2float::add(ps2float::add(ps2float::mul(old_dx, old_dx), ps2float::mul(old_dy, old_dy)), ps2float::mul(old_dz, old_dz)).abs().psqrt());
                    let mut gain = fptodp(gain);
                    if dpcmp(gain, 0_i64) < 0 {
                        gain = dpsub(0_i64, gain);
                    }
                    let over = dpcmp(gain, 0x4000000000000000_i64);
                    let pads = lw(g::levelpads);
                    if over > 0 {
                        let pad = nearest * 28 + pads;
                        let py = lwc1(sp + 36);
                        let pz = lwc1(sp + 40);
                        let is_player = (a0 < lw(g::lvnumplayers)) as i32;
                        swc1(sp + 16, ps2float::sub(lwc1(pad + 12), lwc1(sp + 32)));
                        swc1(sp + 20, ps2float::sub(lwc1(pad + 16), py));
                        swc1(sp + 24, ps2float::sub(lwc1(pad + 20), pz));
                        let blocked;
                        if is_player != 0 {
                            let players = lw(g::players);
                            sw(sp, 0);
                            sw(sp + 8, 0);
                            blocked = bgLineTest(lw(prop + 12), players + a0 * 1820 + 152, sp + 16, 0, 0, 0, 0, 56);
                        } else {
                            swc1(sp + 32, lwc1(prop + 48));
                            swc1(sp + 36, ps2float::add(lwc1(prop + 52), 0.5_f32));
                            let z = lwc1(prop + 56);
                            sw(sp, 0);
                            sw(sp + 8, 0);
                            swc1(sp + 40, z);
                            blocked = bgLineTest(lw(prop + 12), sp + 32, sp + 16, 0, 0, 0, 0, 56);
                        }
                        if blocked == 0 {
                            sw(chr + 680, nearest);
                            sw(chr + 2692, 0);
                            sw(chr + 1688, 0);
                        }
                    }
                }
            }
        }
    }
}
