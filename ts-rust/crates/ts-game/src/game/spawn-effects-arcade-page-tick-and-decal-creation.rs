//! Functions: starting the special effect of a spawning character, the tick of the arcade page and adding a decal to the decal ring.

use super::particle_spawners::random_as_float;
use crate::*;

/// Start the effect of a spawn (@ 0x2adb70): one of the 14 spawn groups is taken, the sound of the spawn is played and the kind of effect `a0` (0 to 6) makes its special
/// effects at the position `a1`; for a character the effect's colour comes from its team (or a random one). Returns the group, or 0 when the group is only a
/// shared one (the flag at +20).
pub fn specialfxNewSpawn(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32) -> i32 {
    let frame = Frame::new(208);
    let sp = frame.sp();
    let mut result = 0;
    let mut free = -1;
    let mut i = 0;
    let mut at = g::spawngroups;
    sw(sp + 32, a4);
    loop {
        if free >= 0 {
            i += 1;
        } else {
            free = if !(0 | lw(at)) == 0 { i } else { free };
            i += 1;
        }
        at += 32;
        if i >= 14 {
            break;
        }
    }
    let offset = free << 5;
    if free >= 0 {
        let group = offset + g::spawngroups;
        sw(offset + g::spawngroups, a0);
        let owner = offset + (g::spawngroups + 12);
        sw(group + 8, 1);
        sw(group + 24, 0);
        sw(owner, 0);
        if a3 == 0 {
            soundStart(214, a1);
            let value = lw(sp + 32);
            sw(offset + (g::spawngroups + 4), 1);
            sw(owner, value);
        } else if a0 != 5 {
            soundStart(213, a1);
            let prop = lw(a3 + 3020);
            sw(offset + (g::spawngroups + 4), 1);
            sw(owner, prop);
        } else {
            let corpse = propNewCorpse(lw(a3 + 3020));
            sw(owner, corpse);
            sw(offset + (g::spawngroups + 4), 0);
        }
        let prop = lw(offset + (g::spawngroups + 12));
        if prop != 0 {
            let radius = lwc1(0x3ad730);
            swc1(sp, lwc1(prop + 48) - radius);
            swc1(sp + 12, lwc1(prop + 48) + radius);
            swc1(sp + 4, lwc1(prop + 52));
            swc1(sp + 16, lwc1(prop + 52) + radius);
            swc1(sp + 8, lwc1(prop + 56) - radius);
            swc1(sp + 20, lwc1(prop + 56) + radius);
            if bgRoomBBIntersectionNumRooms(sp) >= 2 {
                sw(offset + (g::spawngroups + 24), 1);
            }
        }
        let colour_at = offset + (g::spawngroups + 28);
        sw(offset + (g::spawngroups + 16), 0);
        sw(colour_at, 2);
        if a0 >= 5 {
            sw(offset + (g::spawngroups + 20), 1);
            sw(colour_at, a2);
        } else {
            sw(offset + (g::spawngroups + 20), 0);
            if a2 != -1 {
                sw(colour_at, a2);
            } else {
                let random = newrnd();
                let teams = lw(g::curgameparms + 80) & 1;
                sw(colour_at, (random & 3_i64) as i32);
                if teams != 0 {
                    if a3 != 0 {
                        let team = lw(a3 + 24);
                        if ((team - 1) as u32) < (4 as u32) {
                            if team == 2 {
                                sw(colour_at, team);
                            } else if team >= 3 {
                                if team == 3 {
                                    sw(colour_at, 1);
                                } else if team == 4 {
                                    sw(colour_at, 3);
                                }
                            } else if team == 1 {
                                sw(colour_at, 0);
                            }
                        }
                    }
                }
            }
            if ((a0 - 1) as u32) < (2 as u32) {
                sw(offset + (g::spawngroups + 28), -1);
            }
        }
        'effects: {
            let kind = lw(offset + g::spawngroups);
            if (kind as u32) < (7 as u32) {
                let mut single = None;
                match kind {
                    0 => single = Some(1),
                    3 => single = Some(6),
                    1 | 2 => {
                        let planet = if kind == 1 { 2 } else { 3 };
                        let colour = offset + (g::spawngroups + 28);
                        let random = newrnd();
                        let first = specialfxNew(planet, a1, 0, 0, 1, lw(colour), ((random as i32 as u32) % (3 as u32)) as i32 * 24 + g::planetspawncolours);
                        let random = newrnd();
                        let second = specialfxNew(planet, a1, 0, 0, -1, lw(colour), ((random as i32 as u32) % (3 as u32)) as i32 * 24 + g::planetspawncolours);
                        let random = newrnd();
                        let pick = ((random as i32 as u32) % (3 as u32)) as i32;
                        let third = specialfxNew(planet, a1, 0, 0, 1, lw(colour), pick * 24 + g::planetspawncolours);
                        specialfxNew(4, a1, 0, 0, -1, lw(colour), 0);
                        if first == 0 {
                            break 'effects;
                        }
                        if second == 0 {
                            break 'effects;
                        }
                        if third != 0 {
                            let b = lwc1(0x3ad738);
                            swc1(first + 48, lwc1(0x3ad734));
                            let c = lwc1(0x3ad73c);
                            swc1(second + 48, b);
                            swc1(third + 48, c);
                        }
                        break 'effects;
                    }
                    4 => {
                        let colour = offset + (g::spawngroups + 28);
                        let first = specialfxNew(7, a1, 0, 0, 1, lw(colour), 0);
                        let second = specialfxNew(7, a1, 0, 0, -1, lw(colour), 0);
                        let third = specialfxNew(7, a1, 0, 0, 1, lw(colour), 0);
                        'pair: {
                            if first != 0 {
                                if second == 0 {
                                    break 'pair;
                                }
                                if third == 0 {
                                    break 'pair;
                                }
                                let b = lwc1(0x3ad744);
                                swc1(second + 48, lwc1(0x3ad740));
                                swc1(third + 48, b);
                            }
                        }
                        let last = specialfxNew(5, a1, 0, 0, -1, lw(offset + (g::spawngroups + 28)), 0);
                        if last != 0 {
                            swc1(last + 48, lwc1(0x3ad748));
                        }
                        break 'effects;
                    }
                    5 => break 'effects,
                    _ => {
                        specialfxNew(9, a1, 0, 0, 1, lw(offset + (g::spawngroups + 28)), 0);
                        break 'effects;
                    }
                }
                if let Some(effect) = single {
                    specialfxNew(effect, a1, 0, 0, -1, lw(offset + (g::spawngroups + 28)), 0);
                }
            }
        }
        result = if lw(offset + g::spawngroups + 20) != 0 { 0 } else { offset + g::spawngroups };
    }
    result
}

/// Tick of the arcade page (@ 0x23add8): the level to play is picked with left and right among the available arcade levels (the next and previous are found by
/// skipping those that are locked), the items for the game modes the level offers are added to the menu, and the picked item opens the options page of that game mode, the
/// map loading page or starts the map maker's test game.
pub fn arcade_pageTick(a0: i32) {
    let frame = Frame::new(304);
    let sp = frame.sp();
    let table = (lw(g::cur_language) << 2) + g::languages;
    sw(((lw(0x3aff2c) - 1) << 3) + 0x1fc1308, 0);
    pageSet(lw(lw(table) + 484), g::arcade_page);
    frontfxTick(0);
    let front = lw(g::front);
    if lw(front + 20) >= 2 {
        if lw(lw(front + 4) + 16) == g::arcade_mmload_page {
            return;
        }
    }
    let state = 0x1fc11f8;
    let current_level = lw(state + 36);
    sw(state + 8, lw(state + 8) & -49);
    let mut next = current_level + 1;
    let next_level;
    loop {
        let candidate = if next >= 19 { 0 } else { next };
        if levelAvail(lw((candidate << 2) + g::arcadelevels)) != 0 {
            next_level = candidate;
            break;
        }
        next = candidate + 1;
    }
    let state = 0x1fc11f8;
    let current_level = lw(state + 36);
    sw(state + 8, lw(state + 8) | 32);
    let mut previous = current_level - 1;
    let previous_level;
    loop {
        let candidate = if -1 >= previous { 18 } else { previous };
        if levelAvail(lw((candidate << 2) + g::arcadelevels)) != 0 {
            previous_level = candidate;
            break;
        }
        previous = candidate - 1;
    }
    'sound: {
        let state = 0x1fc11f8;
        let low_item = ((lhu(a0 + 4) as u32) < (2 as u32)) as i32;
        let flags = lw(state + 8) | 16;
        sw(state + 8, flags);
        if low_item == 0 {
            let name;
            let buttons = lw(lw(g::front) + 12);
            if (buttons & 16) == 0 {
                if (buttons & 32) == 0 {
                    break 'sound;
                }
                if (flags & 32) == 0 {
                    soundStart(142, 0);
                    break 'sound;
                }
                sw(state + 36, next_level);
                name = 140;
            } else {
                if (flags & 16) == 0 {
                    soundStart(142, 0);
                    break 'sound;
                }
                sw(state + 36, previous_level);
                name = 140;
            }
            soundStart(name, 0);
        }
    }
    let state = 0x1fc11f8;
    let level = lw(state + 36);
    sw(state + 40, lw((level << 2) + g::arcadelevels));
    let page_items;
    'maker: {
        let items = g::i0400;
        if level != 18 {
            sh(items + 10, lhu(items + 10) | 0x4200);
        } else {
            let buffer = lw(g::mmtempbuffer);
            if lw(buffer) == 0x1324626 {
                let info = mmfileGetInfo(buffer);
                let items = g::i0400;
                let flags = lhu(items + 10);
                sw(g::setup_tileset, lw(info));
                sh(items + 10, flags & 0xbdff);
                break 'maker;
            }
            sh(items + 10, lhu(items + 10) | 0x4200);
        }
    }
    'loaded: {
        if (lw(state + 4) & 128) != 0 {
            if lw(state + 36) != 18 {
                page_items = g::i0400;
                break 'loaded;
            }
            let buffer = lw(g::mmtempbuffer);
            if lw(buffer) != 0x1324626 {
                page_items = g::i0400;
                break 'loaded;
            }
            let info = mmfileGetInfo(buffer);
            let flags = lw(state + 4) & -129;
            sw(g::gv_tileset_354b00, lhu(info + 8));
            sw(state + 4, flags);
            sh(a0 + 4, 3);
        }
        page_items = g::i0400;
    }
    let state = 0x1fc11f8;
    let mut text = 0;
    let level_id = lw(state + 40);
    let mut y = ((lhu(page_items + 38) + 15) << 16) >> 16;
    let setup = lvGetARCADESetup(level_id);
    let mut item = page_items + 64;
    sw(state + 56, 0);
    let mut clear = item;
    let mut left = 5;
    loop {
        left -= 1;
        sh(clear + 10, lhu(clear + 10) | 0x4240);
        clear += 32;
        if left < 0 {
            break;
        }
    }
    'modes: {
        let state = 0x1fc11f8;
        let mut language = lw(g::cur_language);
        if lw(state + 36) == 18 {
            if lw(lw(g::mmtempbuffer)) != 0x1324626 {
                break 'modes;
            }
            language = lw(g::cur_language);
        }
        let names = (language << 2) + g::languages;
        let font = lw(g::curfont);
        let few_players = (lw(0x3aff20) < 3) as i32;
        let half = lbu(g::fontflags) & 1;
        let mut mode = g::gmtab;
        let mut i = 0;
        loop {
            'mode: {
                if lbu(mode) == 6 {
                    if few_players == 0 {
                        i += 1;
                        break 'mode;
                    }
                }
                if setup == 0 {
                    i += 1;
                } else {
                    let mask = lbu(mode + 1);
                    let allowed = lw(setup);
                    if (allowed & mask) != 0 {
                        let kind = lbu(mode);
                        sh(item + 6, y);
                        let strings = lw(names);
                        let flags = lhu(item + 10) & 0xbdbf;
                        sh(item + 10, flags);
                        sw(item, lw((lh(kind * 12 + g::gamemodedata) << 2) + strings));
                        if (allowed & (mask << 7)) != 0 {
                            sh(item + 10, flags | 64);
                        }
                        let count = lw(state + 56);
                        item += 32;
                        sw(sp + (count << 2), i);
                        sw(state + 56, count + 1);
                        let line = if half == 0 { lw(font + 12) } else { lw(font + 12) >> 1 };
                        y = (((y - 1) + line) << 16) >> 16;
                    }
                    i += 1;
                }
            }
            mode += 2;
            if i >= 6 {
                break;
            }
        }
    }
    let state = 0x1fc11f8;
    let mut count = lw(state + 56);
    if lw(state + 36) == 18 {
        if count > 0 {
            y = ((y + 9) << 16) >> 16;
        }
        let language = lw(g::cur_language);
        let flags = lbu(g::fontflags);
        let flags_item = lhu(item + 10);
        sh(item + 6, y);
        let strings = lw((language << 2) + g::languages);
        sh(item + 10, flags_item & 0xbdbf);
        sw(item, lw(strings + 1632));
        let second = item + 32;
        let row = if (flags & 1) == 0 { y - 1 + lw(lw(g::curfont) + 12) } else { y - 1 + (lw(lw(g::curfont) + 12) >> 1) };
        sh(second + 6, (row << 16) >> 16);
        count += 2;
        let label = lw(lw((language << 2) + g::languages) + 1636);
        sh(second + 10, lhu(second + 10) & 0xbdbf);
        sw(second, label);
    }
    let picked = if lhu(a0 + 4) < count + 3 {
        lhu(a0 + 4)
    } else {
        sh(a0 + 4, count + 2);
        lhu(a0 + 4)
    };
    let mut items = g::i0400;
    if (picked as u32) >= (3 as u32) {
        let language = lw(g::cur_language);
        if (lhu(((lhu(a0 + 4) - 1) << 5) + (g::i0400 + 10)) & 64) == 0 {
            text = lw(lw((language << 2) + g::languages) + 512);
            items = g::i0400;
        }
    }
    'input: {
        sw(items + 288, text);
        let cheats = cheatsArcade();
        let option = cheatOption(cheats);
        sw(items + 320, option);
        if (joyNewBut(0) & 32) != 0 {
            let cheats = cheatsArcade();
            let state = 0x1fc11f8;
            if cheats != 0 {
                sw(state + 4, lw(state + 4) | 64);
                frontPageSet(0, g::cheats_page);
                pageIn(a0);
                break 'input;
            }
        }
        let pressed = joyNewBut(0);
        let state = 0x1fc11f8;
        if (pressed & 2112) != 0 {
            let selected = lhu(a0 + 4);
            let modes = lw(state + 56);
            if ((selected - 3) as u32) >= (modes as u32) {
                if selected != modes + 3 {
                    if selected == modes + 4 {
                        memset(sp + 32, 0, 112);
                        sw(sp + 96, 103);
                        sw(sp + 36, 1);
                        sw(sp + 32, 1);
                        sw(sp + 40, 0);
                        sw(sp + 104, 0);
                        sw(sp + 108, 0);
                        sw(sp + 112, 0);
                        gameDeferedReset(sp + 32);
                        sw(0x1fc11f8, 5);
                        soundStart(140, 0);
                    }
                } else {
                    mmfileDirReset();
                    frontPageSet(lw(lw(g::front) + 20), g::arcade_mmload_page);
                    sb(0x3aff78, 0);
                    soundStart(140, 0);
                }
            } else {
                let found = menuitem(g::arcade_menu1, selected);
                if (lhu(found + 10) & 64) == 0 {
                    sw((lw(0x3aff2c) << 3) + 0x1fc1308, lw(lw((lw(g::cur_language) << 2) + g::languages) + 1008));
                    frontPageSet(0, g::arcadeoptions_page);
                    pageIn(a0);
                    let mode = lbu((lw(sp + ((lhu(a0 + 4) - 3) << 2)) << 1) + g::gmtab);
                    sw(state + 48, mode);
                    getGameModeOptions(mode);
                }
            }
        }
    }
    let menu = g::arcade_menu1;
    menutick_230238(a0, menu);
    let made = std_menumake(lw(a0 + 8), menu);
    sw(a0 + 12, made);
}

/// Add a decal (@ 0x2a3270) of the kind `a0` and the group `a6` to the ring of decals: the decal quad is built round the hit position `a7` from the surface normal in
/// `s6` (the stack argument at 480) and the up direction (a fixed vector, or another when the normal is near it), sized by the texture and `f12`, and clipped to the
/// polygon it lies on (`decalClipToPoly`). When it is on a moving object the object and bone are noted; the four corners are checked with line tests so decals on glass
/// can be rearranged. Returns the decal number, or -1 when it was clipped away.
pub fn decalAddNewDecal(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32, a6: i32, a7: i32, f12: f32) -> i32 {
    let frame = Frame::new(480);
    let sp = frame.sp();
    let group_size = lw(g::s_DecalsToGroup);
    let group = (if -1 < a6 { a6 } else { a6 + 7 }) >> 3;
    sw(sp + 220, group << 2);
    let normal = lw(sp + 480);
    sw(sp + 216, group * group_size + lw(lw(sp + 220) + 0x1fe9cd8));
    let scale_base = lwc1(a0 * 12 + (g::s_DecalTextures + 4));
    let offset = lw(sp + 216) * 240;
    sw(sp + 212, a4);
    let decals = lw(g::s_Decals);
    let scale = scale_base * f12;
    sw(sp + 208, a2);
    let decal = decals + offset;
    let word = lw(0x3a9310);
    sd(sp, ld(0x3a9308));
    sw(sp + 8, word);
    decalRemoveFromDrawList(decal);
    'object: {
        let owner = lw(sp + 212);
        if owner == 0 {
            sw(decal + 212, -1);
        } else {
            if lw(owner + 8) != 2 {
                sw(decal + 212, a5);
                sw(decal + 216, lw(sp + 212));
                break 'object;
            }
            sw(decal + 212, -1);
        }
        sw(decal + 216, 0);
    }
    swc1(decal + 220, 1.0_f32);
    swc1(decal + 224, 1.0_f32);
    let mut ux = lwc1(sp);
    let mut uy = lwc1(sp + 4);
    let nx = lwc1(normal);
    let ny = lwc1(normal + 4);
    let uz = lwc1(sp + 8);
    let nz = lwc1(normal + 8);
    let texture = lw(a0 * 12 + g::s_DecalTextures);
    sw(decal + 228, a3);
    sw(decal + 204, texture);
    let mut alignment = ux * nx + uy * ny + uz * nz;
    sw(decal + 208, a6);
    if !(0.0_f32 <= alignment) {
        alignment = -alignment;
    }
    if dpcmp(fptodp(alignment), ld(0x3a9318)) > 0 {
        swc1(sp, 1.0_f32);
        ux = 1.0_f32;
        swc1(sp + 4, 0.0_f32);
        uy = 0.0_f32;
    }
    let cx = ny * uz - nz * uy;
    let cy = nz * ux - nx * uz;
    swc1(sp + 32, cx);
    swc1(sp + 36, cy);
    let cz = nx * uy - ny * ux;
    swc1(sp + 40, cz);
    let inverse = 1.0_f32 / (cx * cx + cy * cy + cz * cz).abs().sqrt();
    let rz = lwc1(sp + 40) * inverse;
    let rx = lwc1(sp + 32) * inverse;
    let ry = lwc1(sp + 36) * inverse;
    swc1(sp + 40, rz);
    swc1(sp + 32, rx);
    swc1(sp + 36, ry);
    let fx = nz * rx - nx * rz;
    let fy = ny * rz - nz * ry;
    swc1(sp + 68, fx);
    swc1(sp + 64, fy);
    let fz = nx * ry - ny * rx;
    swc1(sp + 72, fz);
    let inverse = 1.0_f32 / (fy * fy + fx * fx + fz * fz).abs().sqrt();
    let rx = lwc1(sp + 32);
    let ry = lwc1(sp + 36);
    let rz = lwc1(sp + 40);
    let fy = lwc1(sp + 64) * inverse;
    let fx = lwc1(sp + 68) * inverse;
    let fz = lwc1(sp + 72) * inverse;
    let sum_x = rx + fy;
    swc1(sp + 64, fy);
    let sum_y = ry + fx;
    swc1(sp + 68, fx);
    let sum_z = rz + fz;
    swc1(sp + 72, fz);
    swc1(sp + 16, sum_x);
    swc1(sp + 20, sum_y);
    swc1(sp + 24, sum_z);
    let inverse = 1.0_f32 / (sum_x * sum_x + sum_y * sum_y + sum_z * sum_z).abs().sqrt();
    let dx = rx - lwc1(sp + 64);
    let dy = ry - lwc1(sp + 68);
    let px = lwc1(sp + 16);
    let dz = rz - lwc1(sp + 72);
    let py = lwc1(sp + 20);
    swc1(sp + 48, dx);
    let pz = lwc1(sp + 24);
    swc1(sp + 52, dy);
    swc1(sp + 56, dz);
    swc1(sp + 24, pz * inverse);
    swc1(sp + 16, px * inverse);
    swc1(sp + 20, py * inverse);
    let inverse = 1.0_f32 / (dx * dx + dy * dy + dz * dz).abs().sqrt();
    let by = lwc1(sp + 52);
    let qy = lwc1(sp + 20);
    let qz = lwc1(sp + 24);
    let qx = lwc1(sp + 16) * scale;
    let bx = lwc1(sp + 48) * inverse;
    let hit_x = lwc1(a7);
    let bz = lwc1(sp + 56) * inverse;
    sw(decal + 200, 4);
    swc1(sp + 16, qx);
    let bx_scaled = bx * scale;
    swc1(sp + 48, bx_scaled);
    swc1(sp + 56, bz * scale);
    let corner_x = hit_x - bx_scaled;
    swc1(sp + 20, qy * scale);
    swc1(sp + 24, qz * scale);
    swc1(sp + 52, by * inverse * scale);
    swc1(decal, corner_x);
    swc1(sp + 80, corner_x);
    let corner_y = lwc1(a7 + 4) - lwc1(sp + 52);
    swc1(decal + 4, corner_y);
    swc1(sp + 84, corner_y);
    let corner_z = lwc1(a7 + 8) - lwc1(sp + 56);
    swc1(decal + 8, corner_z);
    swc1(sp + 88, corner_z);
    if (a1 as u32) >= (2 as u32) {
        swc1(decal + 128, 1.0_f32);
    } else {
        swc1(decal + 128, 0.0_f32);
    }
    let low = ((lw(sp + 208) as u32) < (2 as u32)) as i32;
    swc1(sp + 128, lwc1(decal + 128));
    if low == 0 {
        swc1(decal + 132, 1.0_f32);
    } else {
        sw(decal + 132, 0);
    }
    swc1(sp + 132, lwc1(decal + 132));
    let x = lwc1(a7) - lwc1(sp + 16);
    swc1(decal + 16, x);
    swc1(sp + 92, x);
    let y = lwc1(a7 + 4) - lwc1(sp + 20);
    swc1(decal + 20, y);
    swc1(sp + 96, y);
    let z = lwc1(a7 + 8) - lwc1(sp + 24);
    swc1(decal + 24, z);
    swc1(sp + 100, z);
    if ((a1 - 1) as u32) >= (2 as u32) {
        swc1(decal + 136, 1.0_f32);
    } else {
        sw(decal + 136, 0);
    }
    let low = (((lw(sp + 208) - 1) as u32) < (2 as u32)) as i32;
    swc1(sp + 136, lwc1(decal + 136));
    if low == 0 {
        swc1(decal + 140, 1.0_f32);
    } else {
        sw(decal + 140, 0);
    }
    swc1(sp + 140, lwc1(decal + 140));
    let x = lwc1(a7) + lwc1(sp + 48);
    swc1(decal + 32, x);
    swc1(sp + 104, x);
    let y = lwc1(a7 + 4) + lwc1(sp + 52);
    swc1(decal + 36, y);
    swc1(sp + 108, y);
    let z = lwc1(a7 + 8) + lwc1(sp + 56);
    swc1(decal + 40, z);
    swc1(sp + 112, z);
    if ((a1 - 2) as u32) >= (2 as u32) {
        swc1(decal + 144, 1.0_f32);
    } else {
        sw(decal + 144, 0);
    }
    let low = (((lw(sp + 208) - 2) as u32) < (2 as u32)) as i32;
    swc1(sp + 144, lwc1(decal + 144));
    if low == 0 {
        swc1(decal + 148, 1.0_f32);
    } else {
        sw(decal + 148, 0);
    }
    'high: {
        swc1(sp + 148, lwc1(decal + 148));
        let x = lwc1(a7) + lwc1(sp + 16);
        swc1(decal + 48, x);
        swc1(sp + 116, x);
        let y = lwc1(a7 + 4) + lwc1(sp + 20);
        swc1(decal + 52, y);
        swc1(sp + 120, y);
        let z = lwc1(a7 + 8) + lwc1(sp + 24);
        swc1(decal + 56, z);
        swc1(sp + 124, z);
        if a1 != 0 {
            if a1 != 3 {
                swc1(decal + 152, 1.0_f32);
                break 'high;
            }
        }
        sw(decal + 152, 0);
    }
    'last: {
        let owner_kind = lw(sp + 208);
        swc1(sp + 152, lwc1(decal + 152));
        if owner_kind != 0 {
            if owner_kind != 3 {
                swc1(decal + 156, 1.0_f32);
                break 'last;
            }
        }
        sw(decal + 156, 0);
    }
    swc1(sp + 156, lwc1(decal + 156));
    let clipped = decalClipToPoly(decal, normal, a1, lw(sp + 208), lw(sp + 488), lw(sp + 496), lw(sp + 504), lw(sp + 512));
    let hits = lw(sp + 512);
    let result;
    if clipped == 0 {
        result = -1;
        sw(decal + 204, -1);
    } else {
        let mut groups = lw(g::s_DecalsToGroup);
        'reorder: {
            if hits != 0 {
                if lw(hits) >= 0 {
                    let mut vx = lwc1(normal);
                    let mut vy = lwc1(normal + 4);
                    let mut all = 1;
                    let mut vz = lwc1(normal + 8);
                    let factor = lwc1(0x3ad5c0);
                    let mut i = 0;
                    sw(sp + 224, sp + 160);
                    swc1(sp + 176, -vx * factor);
                    swc1(sp + 180, -vy * factor);
                    swc1(sp + 184, -vz * factor);
                    loop {
                        let step = lwc1(0x3ad5c4);
                        let origin = lw(sp + 224);
                        let object = lw(lw(sp + 212) + 32);
                        swc1(sp + 160, lwc1(sp + 80 + i * 12) + vx * step);
                        swc1(sp + 164, lwc1(sp + 84 + i * 12) + vy * step);
                        swc1(sp + 168, lwc1(sp + 88 + i * 12) + vz * step);
                        let hit = obInstLineTest(object, origin, sp + 176, 0, 0, 0);
                        all = if hit == 0 { 0 } else { all };
                        let at = sp + 192 + (i << 2);
                        i += 1;
                        sw(at, hit);
                        if i >= 4 {
                            break;
                        }
                        vx = lwc1(normal);
                        vy = lwc1(normal + 4);
                        vz = lwc1(normal + 8);
                    }
                    let hits = lw(sp + 512);
                    if all == 0 {
                        groups = lw(g::s_DecalsToGroup);
                    } else {
                        sw(hits, -1);
                        let mut j = 0;
                        groups = lw(g::s_DecalsToGroup);
                        let mut v = decal + 132;
                        let mut u = decal + 128;
                        let mut z_at = decal + 8;
                        let mut y_at = decal + 4;
                        let mut x_at = decal;
                        loop {
                            let sign = ((j as u32) >> 31) as i32;
                            let corner;
                            if (j & 1) == 0 {
                                let value = 4 - ((j + sign) >> 1);
                                corner = value - (((if -1 < value { value } else { value + 3 }) >> 2) << 2);
                            } else {
                                corner = ((j + sign) >> 1) + 1;
                            }
                            let uv = corner << 3;
                            j += 1;
                            swc1(x_at, lwc1(sp + 80 + corner * 12));
                            x_at += 16;
                            swc1(y_at, lwc1(sp + 84 + corner * 12));
                            y_at += 16;
                            swc1(z_at, lwc1(sp + 88 + corner * 12));
                            z_at += 16;
                            swc1(u, lwc1(sp + 128 + uv));
                            u += 8;
                            swc1(v, lwc1(sp + 132 + uv));
                            v += 8;
                            if j >= 4 {
                                break 'reorder;
                            }
                        }
                    }
                }
            }
        }
        if groups == 0 {
            trap("break");
        }
        let counter = lw(sp + 220) + 0x1fe9cd8;
        sw(counter, (lw(counter) + 1).wrapping_rem(groups));
        decalAddToDrawList(decal, lw(decal + 208));
        result = lw(sp + 216);
    }
    result
}
