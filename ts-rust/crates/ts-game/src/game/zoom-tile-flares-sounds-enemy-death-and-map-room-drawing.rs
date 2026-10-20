//! Functions: the zoomed tile page, the front end's flares, the level sounds, the tile set preload, the challenge character page, the map maker's rooms, the remote and enemy props
//! and the main in-game menu page's drawing.

use crate::*;

/// Draw the zoomed tile page: the box around the tile window, the tile, the button texts and the pad panel while only one page is up, the popup naming the item under the
/// cursor, and the pad page (@ 0x253018).
pub fn zoomtile_pageDraw(a0: i32) {
    dlSetZB(0);
    let width = lw(0x3b006c);
    let height = lw(0x3b0070);
    let y = lw(0x3b0064);
    let x = lw(0x3b0068);
    let half_width = (width + ((width as u32) >> 31) as i32) >> 1;
    let half_height = (height + ((height as u32) >> 31) as i32) >> 1;
    zoomboxDraw(y - half_width, x - half_height, y + half_width, x + half_height, 0x2880, 0x7f7f7f7f);
    zoomtileGfx();
    if lw(lw(g::front) + 20) < 2 {
        let current = lhu(g::padactcuritem);
        sw(g::button_text, 0);
        let language;
        let text;
        if current != 1 {
            language = lw(g::cur_language);
            text = lw(lw((language << 2) + g::languages) + 3548);
        } else {
            language = lw(g::cur_language);
            text = lw(lw((language << 2) + g::languages) + 3544);
        }
        'b3: {
            sw(g::button_text + 4, text);
            let languages = g::languages;
            let buttons = g::button_text;
            let pad = lw(g::aimpadinfo);
            sw(buttons + 8, lw(lw((language << 2) + languages) + 3552));
            if pad != 0 {
                let found = objFindItem(lw(g::zoomtileobj), lw(lw(pad)), -1);
                let language = lw(g::cur_language);
                if found != 0 {
                    sw(buttons + 12, lw(lw((language << 2) + languages) + 3556));
                    break 'b3;
                }
            }
            sw(g::button_text + 12, 0);
        }
        mmDrawPad(lw(g::w_x) + lw(g::w_w) - 128, 40);
    }
    'b5: {
        let pad = lw(g::aimpadinfo);
        if pad != 0 {
            let found = objFindItem(lw(g::zoomtileobj), lw(lw(pad)), -1);
            if found == 0 {
                break 'b5;
            }
            if (lhu(found + 2) as u32) >= 20 {
                break 'b5;
            }
            let text = lw((lh((lhu(found + 2) << 4) + g::itemdefs) << 2) + lw((lw(g::cur_language) << 2) + g::languages));
            mmDrawPopup(text, 40, 196, 430, 216, 0x283f, 0, 0);
        }
    }
    padact_pageDraw(a0);
}

/// Draw the front end's effects for the page kind `a0`: the clip is set, and (not for the kind 2) the background, then (not for the kind 5) the logo and the flares: each flare
/// is a textured rectangle in its colour at its place around the logo; the goodies page (kind 4) adds its own (@ 0x22ab90).
pub fn frontfxGfx(a0: i32) {
    let frame = Frame::new(208);
    let sp = frame.sp();
    let window = g::window;
    dlSetClip(lw(g::window), lw(window + 20), lw(window + 4), lw(window + 24));
    let blend = dlSetBlend(1);
    sw(sp + 4, blend);
    let dither = dlSetDitherMatrix(1);
    sw(sp, dither);
    if a0 != 2 {
        frontfxBgGfx();
        if a0 != 5 {
            frontfxLogoGfx(a0);
            dlSetBlend(2);
            let mut i = 0;
            if lw(g::flares) > 0 {
                loop {
                    let index = i << 2;
                    let flare = i * 12 + g::flares;
                    i += 1;
                    let intensity = lwc1(index + g::flareintensity) * 127.0_f32;
                    let green = lwc1(flare + 76) * 127.0_f32;
                    let blue = lwc1(flare + 80) * 127.0_f32;
                    let red = lwc1(flare + 84) * 127.0_f32;
                    let c0 = fptoui(intensity);
                    let c1 = fptoui(green);
                    let c2 = fptoui(red);
                    let c3 = fptoui(blue);
                    let other = index + g::flares;
                    let scale = lwc1(other + 4);
                    let width = lwc1(other + 28) * 256.0_f32;
                    let half = width * 0.5_f32;
                    let x = lw(g::logoposx) as f32 + lw(g::window + 16) as f32 * lwc1(g::flarescalarx) * scale;
                    let texture = lw((lw(other + 52) << 2) + 0x1fbd1b8);
                    let y = lw(g::logoposy) as f32 + lw(g::window + 36) as f32 * -lwc1(g::flarescalary) * scale;
                    dlSelectTextureKick(texture);
                    let right = x + width;
                    let colour = c1 << 24 | c0 | (c2 << 8 | c3 << 16);
                    let bottom = y + half;
                    dlTextureRectangleFloat(0, 0, 1024, 1024, colour, x - width, y - half, right, bottom);
                    if i >= lw(g::flares) {
                        break;
                    }
                }
            }
        }
    }
    if a0 == 4 {
        dlSetBlend(1);
        frontfxGoodiesGfx();
    }
    dlSetBlend(lw(sp + 4));
    dlSetDitherMatrix(lw(sp));
}

/// Start the looping sound `sound` at the position copied from `source` (12 bytes) to `dest` with the range `range` (@ lvSoundsRestart).
fn start_loop_at(sound: i32, source: i32, dest: i32, range: f32) {
    let tail = lw(source + 8);
    sd(dest, ld(source));
    sw(dest + 8, tail);
    soundStartLoopRange(sound, dest, range);
}

/// Restart the sounds of the level: the levels 11, 17 and 21 start their looping ambient sounds at fixed places (@ 0x226be8).
pub fn lvSoundsRestart() {
    let frame = Frame::new(96);
    let sp = frame.sp();
    let level = lw(g::lvlevelnum);
    if level == 17 {
        let range = 2e+01_f32;
        start_loop_at(1, 0x3a0ef8, sp, range);
        start_loop_at(1, 0x3a0f08, sp + 16, range);
    } else if level >= 18 {
        if level == 21 {
            let range = 15.0_f32;
            start_loop_at(16, 0x3a0ec8, sp, range);
            start_loop_at(16, 0x3a0ed8, sp + 16, range);
            start_loop_at(15, 0x3a0ee8, sp + 48, 5.0_f32);
        }
    } else if level == 11 {
        let range = 15.0_f32;
        start_loop_at(4, 0x3a0e98, sp, range);
        start_loop_at(4, 0x3a0ea8, sp + 16, range);
        start_loop_at(5, 0x3a0eb8, sp + 32, range);
    }
}

/// Preload for the map maker's tile set level: the map file in the temporary buffer is walked chunk by chunk (each is a tag and a size); the rooms chunk (tag 0x1324627) gives the
/// count and the objects chunk (0x1324628) adds the models of the things in each room to the preload list (@ 0x226e10).
pub fn lvTileSetLevelPreload() {
    let mut buffer = lw(g::mmtempbuffer);
    let s2 = buffer + 8;
    let mut count = 0;
    if (s2 as u32) < ((buffer + 0x1ffc) as u32) {
        let mut chunk = s2;
        let first = lw(buffer + 8);
        let mut data = buffer + 16;
        if first != 0x1324629 {
            let mut tag = first;
            loop {
                let size;
                'b3: {
                    if tag != 0x1324627 {
                        if tag != 0x1324628 {
                            size = lw(chunk + 4);
                            break 'b3;
                        }
                        let mut at = data;
                        let mut room = 0;
                        if count > 0 {
                            let mut items = lhu(at + 10);
                            loop {
                                let mut item = 0;
                                at += 36;
                                if items == 0 {
                                    room += 1;
                                } else {
                                    let next_room = room + 1;
                                    loop {
                                        let mut kind = lhu(at);
                                        let shifted = kind << 16;
                                        if ((kind - 5) as u32) < 15 {
                                            let definition = lw((shifted >> 12) + (g::itemdefs + 12));
                                            let number = lvGetPropNum(definition);
                                            if number > 0 {
                                                propAddPreLoad(number);
                                            }
                                            kind = lhu(at);
                                        }
                                        'b11: {
                                            let signed = (kind << 16) >> 16;
                                            let index = signed - 5;
                                            if (index as u32) < 4 {
                                                if lbu(index + (g::curgameparms + 100)) != 0 {
                                                    if lw(g::curgameparms + 72) != 2 {
                                                        item += 1;
                                                        break 'b11;
                                                    }
                                                    propAddPreLoad(signed + 315);
                                                }
                                            }
                                            item += 1;
                                        }
                                        at += 4;
                                        if item >= items {
                                            break;
                                        }
                                    }
                                    room = next_room;
                                }
                                if room >= count {
                                    break;
                                }
                                items = lhu(at + 10);
                            }
                            buffer = lw(g::mmtempbuffer);
                        }
                    } else {
                        count = lw(data + 4);
                    }
                    size = lw(chunk + 4);
                }
                let next = data + size;
                chunk = next;
                if (next as u32) >= ((buffer + 0x1ffc) as u32) {
                    break;
                }
                let next_tag = lw(chunk);
                data = chunk + 8;
                tag = next_tag;
                if next_tag == 0x1324629 {
                    break;
                }
            }
        }
    }
}

/// The tick of the challenge character page: the stage and the character of the challenge's set-up are made when the page comes up; the select effect ticks; at the state 5
/// the button 16 goes back and 64 or 2112 starts the game, at the state 2 16 leaves and 64 or 2112 starts the flash; the menu of the state ticks (@ 0x23c3d0).
pub fn challengechr_pageTick(a0: i32) {
    pageSet(0, g::challengechr_page);
    sw(g::allowedcheats, 0);
    frontfxTick(2);
    if lbu(lw(lw(g::front)) + 0x3aff28) == 1 {
        let setup = challengeGetSetupNum(lw(0x1fc1224));
        let character = lw(lw(setup + 48));
        fxSelectCharReset();
        setStates(2);
        fxCreateStage(0);
        fxSelectCharInit(0, character);
    }
    fxSelectCharTick(1);
    animTick(a0);
    let player = lw(lw(g::front));
    let state = lbu(player + 0x3aff28);
    let mut menu = 0;
    if state == 5 {
        let pressed = joyNewBut(player);
        let front = lw(g::front);
        if (pressed & 16) != 0 {
            sb(lw(front) + 0x3aff28, 2);
            fxSelectCharFlashInit(lw(front));
            actingsoundStop(lw(lw(g::front)));
            joyClearButs(lw(lw(g::front)), 16);
        }
        menu = g::chrselect_ready_go;
        if (joyNewBut(0) & 2112) != 0 {
            soundStart(143, 0);
            animStopImmediately();
            setStates(6);
            frontmpgStartGame();
        }
    } else if state < 6 {
        if state == 2 {
            let pressed = joyNewBut(player);
            let front = lw(g::front);
            if (pressed & 16) != 0 {
                pageOut();
                return;
            }
            let pressed = joyNewBut(lw(front));
            let front = lw(g::front);
            if (pressed & 2112) == 0 {
                menu = g::chrselect_main_menu;
            } else {
                fxSelectCharFlashInit(lw(front));
                sb(lw(lw(g::front)) + 0x3aff28, 5);
            }
        }
    }
    if menu != 0 {
        sh(menu + 6, 170);
        menutick_230238(a0, menu);
        let made = std_menumake(lw(a0 + 8), menu);
        sw(a0 + 12, made);
    }
}

/// Draw the map maker's rooms: the tiles of the other level (mode 6), then those of the edit level in the colours of the draw mode (highlighted or with the flag 4 when marked)
/// while the edit flag 0x800000 is set (@ 0x24aa08).
pub fn mmDrawRooms() {
    let mut object = lw(g::mapobjects);
    if object != 0 {
        let mut kind = lw(object);
        loop {
            'b4: {
                if kind != 1 {
                    object = lw(object + 56);
                } else {
                    let left = lw(g::m_x);
                    if ((lw(g::otherlevel) - lw(object + 20)) as u32) < (lw(lw(object + 36) + 8) as u32) {
                        let x = lw(object + 12);
                        let tile = lw(object + (lw(object + 24) << 2) + 36);
                        let y = lw(object + 16);
                        let right = x + (lw(tile) << 21);
                        let bottom = y + (lw(tile + 4) << 21);
                        if x < left + lw(g::m_w) {
                            let top = lw(g::m_y);
                            if y < top + lw(g::m_h) {
                                if right < left {
                                    object = lw(object + 56);
                                    break 'b4;
                                }
                                if bottom < top {
                                    object = lw(object + 56);
                                    break 'b4;
                                }
                                mmDrawTile(x, y, tile, 6, object);
                            }
                        }
                    }
                    object = lw(object + 56);
                }
            }
            if object == 0 {
                break;
            }
            kind = lw(object);
        }
        object = lw(g::mapobjects);
    }
    sw(g::editflags, lw(g::editflags) | 0x800000);
    if object != 0 {
        let mut kind = lw(object);
        loop {
            'b9: {
                if kind != 1 {
                    object = lw(object + 56);
                } else {
                    let left = lw(g::m_x);
                    if ((lw(g::editlevel) - lw(object + 20)) as u32) < (lw(lw(object + 36) + 8) as u32) {
                        let x = lw(object + 12);
                        let tile = lw(object + (lw(object + 24) << 2) + 36);
                        let y = lw(object + 16);
                        let right = x + (lw(tile) << 21);
                        let bottom = y + (lw(tile + 4) << 21);
                        if x < left + lw(g::m_w) {
                            let top = lw(g::m_y);
                            if y < top + lw(g::m_h) {
                                if right < left {
                                    object = lw(object + 56);
                                    break 'b9;
                                }
                                if bottom < top {
                                    object = lw(object + 56);
                                    break 'b9;
                                }
                                let mode = if (object ^ lw(g::highlightobj)) == 0 { kind } else { 0 };
                                let mode = if (lw(object + 4) & 1) == 0 { mode } else { mode | 4 };
                                mmDrawTile(x, y, tile, mode, object);
                            }
                        }
                    }
                    object = lw(object + 56);
                }
            }
            if object == 0 {
                break;
            }
            kind = lw(object);
        }
    }
    sw(g::editflags, lw(g::editflags) & -0x800001);
}

/// Make a remote controlled prop (a car or the like) at the position `a1`: the record gets its controls, the start state and the pivot position, the pivot's room is found,
/// and the angle to the pivot from the remote's start is set at +12, +16 and +24 (@ 0x2695c0).
pub fn remotePropNew(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32, f12: f32, f13: f32, f14: f32, f15: f32, f16: f32, f17: f32) {
    let frame = Frame::new(304);
    let sp = frame.sp();
    let prop = propNew(a0, lwc1(a1), lwc1(a1 + 4), lwc1(a1 + 8), f12);
    let model = lw(prop + 32);
    sw(prop + 88, 0);
    let skeleton = lw(model);
    let blend = lwc1(prop + 88);
    let record = lw(prop + 352);
    let skeleton_offset = lw(skeleton) * 80;
    sw(prop + 140, 2);
    obInstSetAnim(model, 1, blend);
    sw(record + 4, if (a3 & 32) == 0 { 1 } else { 4 });
    swc1(record + 60, f17);
    swc1(record + 40, f14);
    swc1(record + 44, f15);
    swc1(record + 52, 3.6e+02_f32);
    swc1(record + 20, f16);
    sw(record + 56, 0);
    sw(record + 64, 0);
    sw(record + 32, 0);
    swc1(record + 36, 3.6e+02_f32);
    sw(record + 48, 0);
    sw(record + 28, 0);
    let control = lw(record + 100);
    let players = lw(g::players);
    swc1(record + 68, lwc1(a2));
    sw(record, a3);
    let y = lwc1(a2 + 4);
    sw(record + 8, a5);
    swc1(record + 72, y);
    let player_chr = lw(players + 384);
    let z = lwc1(a2 + 8);
    sw(control, a4);
    sw(control + 68, 0);
    swc1(record + 92, f13);
    sw(record + 104, 0);
    sw(record + 96, player_chr);
    swc1(record + 76, z);
    sw(control + 64, 0);
    remoteGetPivotePos(prop, sp + 80);
    let reach = lwc1(0x3acbd4);
    let room = moveFindRoom(sp + 80, reach, reach);
    sw(record + 120, room);
    sw(record + 124, 0);
    sw(record + 128, 0);
    sw(record + 132, 0);
    sw(record + 136, 0);
    if lb(skeleton - skeleton_offset + 7) <= 0 {
        sw(record + 4, 0);
    } else {
        let degrees = lwc1(0x3acbd8);
        let offset = remoteGetPivoteOffset(prop);
        swc1(sp, lwc1(offset));
        swc1(sp + 4, lwc1(offset + 4));
        swc1(sp + 8, lwc1(offset + 8));
        matrixRotY(sp + 16, lwc1(prop + 76) * degrees / 1.8e+02_f32);
        matrixVecRotAligned(&[(sp + 16) as i64, sp as i64]);
        let dx = lwc1(a1) + lwc1(sp) - lwc1(record + 68);
        let dy = lwc1(a1 + 4) + lwc1(sp + 4) - lwc1(record + 72);
        let dz = lwc1(a1 + 8) + lwc1(sp + 8) - lwc1(record + 76);
        let sine = dy / (dx * dx + dy * dy + dz * dz).abs().sqrt();
        let angle = asinf(sine);
        swc1(record + 24, lwc1(record + 20));
        let result = angle * 1.8e+02_f32 / degrees;
        swc1(record + 16, result);
        swc1(record + 12, result);
    }
}

/// An enemy `a0` dies: its effect is freed, the kill is counted for the killer `a1` (a character), the death animation is chosen (unless the flag 0x1000 says it has one), the
/// move mode becomes 32, a gun it held is given a random amount of ammo, mines are detonated and the guns cleared (@ 0x2c6dc8).
pub fn enemyDie(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32, a6: i32) {
    let record = lw(a0 + 352);
    let effect = lw(record + 0x11a8);
    if effect != 0 {
        specialfxFreePtr(effect);
        sw(record + 0x11a8, 0);
    }
    obInstResetOverride(lw(a0 + 32));
    let mut killer = 0;
    if a1 != 0 {
        if lw(a1 + 8) == 8 {
            killer = lw(a1 + 352);
        }
    }
    let mut parms;
    'b3: {
        if lw(record + 8) != 1 {
            StatsKillMade(killer, record);
            parms = g::curgameparms;
            if lw(g::curgameparms + 72) != 6 {
                break 'b3;
            }
            if a1 != 0 {
                parms = g::curgameparms;
                if lw(a1 + 8) != 8 {
                    break 'b3;
                }
                let shooter = lw(a1 + 352);
                sw(shooter + 248, lw(shooter + 248) + 10);
            }
        }
        parms = g::curgameparms;
    }
    if lw(parms + 72) == 4 {
        if a1 != 0 {
            if lw(a1 + 8) == 8 {
                if a0 == lw(g::escort) {
                    let shooter = lw(a1 + 352);
                    sw(shooter + 248, lw(shooter + 248) + 1);
                }
            }
        }
    }
    let flags = lw(record + 2708);
    let old_state;
    if (flags & 0x1000) != 0 {
        old_state = lw(record + 2716);
    } else {
        chrSelectDeathAnim(a0, a2, a3, a4, a5, a6);
        old_state = lw(record + 2716);
    }
    sw(record + 2892, 0);
    sw(record + 2720, old_state);
    sw(record + 2716, 16);
    enemySetMoveMode(a0, 32);
    let held;
    if lw(record + 8) != 1 {
        held = lw(record + 440);
    } else {
        sw(record + 2920, 0);
        sw(record + 2924, 0);
        sw(record + 2928, 0);
        held = lw(record + 440);
    }
    if held != 0 {
        let roll = newrnd();
        let gun = lw(record + 376) * 400 + g::guns;
        let amount = (lw(gun + 24) as f32 * lwc1(0x3ad990)) as i32;
        if amount == 0 {
            trap("break");
        }
        sw(record + (lw(gun + 16) << 2) + 620, ((roll as i32 as u32) % (amount as u32)) as i32 + 5);
    }
    if gunCanDetonateMines(record) != 0 {
        gunDetonateRemoteMines(record);
    }
    chrClearGuns(record);
}

/// Draw the in-game main menu page: the highlight, a darkening of the window while the menu is open, then (in the game states under 5) the pause text, or the top menu
/// with the stats, rankings or awards page and the page's own draw functions (@ 0x2120b8).
pub fn main_pageGfx(a0: i32) {
    let menu = lw(lw(g::currentplayer)) * 92 + 0x1fb3240;
    dlSetBlend(1);
    menuitemHighlight(a0);
    'b1: {
        if lw(0x3afe78) < 0 {
            if (lhu(menu + 8) & 4) == 0 {
                break 'b1;
            }
        }
        let dither = dlSetDitherMatrix(1);
        let window = g::window;
        dlFillRectangle(lw(g::window), lw(window + 20), lw(window + 4), lw(window + 24), 64);
        dlSetDitherMatrix(dither);
    }
    if lw(g::gamestate) < 5 {
        let language = lw(g::cur_language);
        if (lhu(menu + 8) & 2) != 0 {
            let text = lw(lw((language << 2) + g::languages) + 1688);
            let old_font = textSelect(1);
            if lw(0x3afe78) >= 0 {
                drawPause();
            }
            let width = textWidth(text);
            let window = g::window;
            textOutline(lw(window + 8) - ((((width as u32) >> 31) as i32 + width) >> 1), lw(window + 24) - 30, text, lw(g::menuitem_fgcol), lw(g::menuitem_bgcol));
            textSelect(old_font);
            return;
        }
        topmenuGfx(a0);
        let selected = lhu(menu + 4);
        if selected >= 3 {
            if selected == 9 {
                drawAwards();
            }
        } else if selected != 0 {
            drawRankingsTable(selected);
        } else {
            drawStats();
        }
        let page = lw(menu + (lhu(menu + 4) << 2) + 12);
        if page != 0 {
            let first = lw(page + 4);
            let second;
            if first == 0 {
                second = lw(page + 12);
            } else {
                call_indirect(first, [a0, 0, 0, 0, 0, 0, 0, 0], [0.0; 8]);
                second = lw(page + 12);
            }
            if second != 0 {
                call_indirect(second, [a0, 0, 0, 0, 0, 0, 0, 0], [0.0; 8]);
            }
        }
    }
}

/// Create a non-player character in a free slot of the character table (or a new one when under 100): the previous prop of the slot is deleted, the character is made on the
/// pad of the external reference `a0` with the type `a1`/`a2`, its team chosen, guns cleared, the weapon (`a3`), flags (`a4`), values (`a5`, `a6`) set, and the pose chosen; the
/// slot is returned, or -1 when there is none (@ 0x2cc510).
pub fn enemyCreateNPC(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32, a6: i32) -> i32 {
    let players = lw(g::lvnumplayers);
    let chars = lw(g::numactivechars);
    let total = chars + players;
    let mut slot = -1;
    let mut i = players;
    if players < total {
        let info = lw(g::chrinfo);
        loop {
            slot = if lw(i * 0x1210 + info + 2716) == 0 { i } else { slot };
            i += 1;
            if i >= total {
                break;
            }
        }
    }
    if slot == -1 {
        if chars < 100 {
            slot = players + chars;
            sw(g::numactivechars, chars + 1);
        }
        if slot == -1 {
            return -1;
        }
    }
    let offset = slot * 0x1210;
    let old_prop = lw(slot * 0x1210 + lw(g::chrinfo) + 3020);
    if old_prop != 0 {
        propDelete(old_prop);
        sw(offset + lw(g::chrinfo) + 3020, 0);
    }
    let pad = padNumFromExtref(a0);
    let chr = chrNew(a1, a2, 0, slot, pad, 1, 0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32);
    sw(((slot - lw(g::lvnumplayers)) << 2) + g::chr, chr);
    chrSelectTeam(lw(chr + 352));
    chrClearGuns(lw(g::chrinfo) + offset);
    let info = lw(g::chrinfo);
    let record = offset + info;
    sw(record + 2704, lw(record + 2704) | 16);
    if a3 == -1 {
        sw(record + 260, 0);
    } else {
        sw(record + 260, a3);
    }
    let other = slot * 0x1210 + info;
    sw(other + 2724, a4);
    if (a4 & 0x69c000) != 0 {
        sw(other + 2728, a5);
        if a6 != -1 {
            sw(other + 2704, a6 | 16);
        }
        sw(other + 2768, -1);
    }
    let mut players_now = lw(g::lvnumplayers);
    if a2 == 31 {
        players_now = lw(g::lvnumplayers);
        if (a4 & 0x2000000) != 0 {
            obInstSetAnim(lw(lw(((slot - players_now) << 2) + g::chr) + 32), 476, 0.0_f32);
            return slot;
        }
    }
    obInstSetAnim(lw(lw(((slot - players_now) << 2) + g::chr) + 32), 3, 1.0_f32);
    slot
}
