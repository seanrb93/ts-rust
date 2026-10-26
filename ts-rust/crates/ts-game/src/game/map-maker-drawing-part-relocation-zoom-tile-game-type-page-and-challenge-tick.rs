//! Functions: drawing the map maker, relocating the parts of a character, drawing the zoomed tile, the game type page's tick, drawing the arcade map load page and the map file
//! page, ticking the challenge conditions and updating the data of a camera prop that follows a path.

use crate::*;

/// Draw the map maker (@ 0x24b688): the 2D view, the level box, a popup that describes the highlighted object (with the zoomed tile for a tile object), the title and the
/// pad.
pub fn mapmakerGfx() {
    dlSetZB(2);
    mmDraw2DView();
    if (lw(g::editflags) & 0x4008) == 0 {
        if lw(lw(g::front) + 20) == 0 {
            mmDrawLevelBox();
            let object = lw(g::highlightobj);
            let mut extra = -1;
            if object != 0 {
                let mut text = 0;
                if lw(object) == 1 {
                    if lw(g::editmode) == 2 {
                        let walls = PWFTAB + 5160;
                        let mut index = 0;
                        let language = lw(g::cur_language);
                        if lbu(g::lighttypes + 2) != lbu(object + 28) {
                            index += 1;
                            loop {
                                if index >= 11 {
                                    break;
                                }
                                if lbu((index << 2) + (g::lighttypes + 2)) == lbu(object + 28) {
                                    break;
                                }
                                index += 1;
                            }
                        }
                        let offset = index << 2;
                        if index >= 2 {
                            extra = lbu(object + 29);
                        }
                        let strings = lw((language << 2) + g::languages);
                        let format = lw(strings + 3584);
                        let name = lw((lh(offset + g::lighttypes) << 2) + strings);
                        text = vafmt(&[0x3ae940, format as i64, name as i64, index as i64, object as i64, walls as i64, 0x3b0000]);
                    }
                }
                let x = lw(g::w_x);
                if text != 0 {
                    let y = lw(g::w_y) + lw(g::w_h) - 28;
                    let right = x + lw(g::w_w) - 220;
                    let label = if extra < 0 { 0 } else { lw(lw((lw(g::cur_language) << 2) + g::languages) + 3588) };
                    mmDrawPopup(text, x + 40, y, right, 219, 0x2850, label, extra + 24);
                }
                let highlighted = lw(g::highlightobj);
                if highlighted != 0 {
                    if lw(highlighted) == 1 {
                        zoomboxDraw(458, 160, 629, 219, 0x1e64, 0x7f7f7f7f);
                        zoomtileGfx();
                    }
                }
            }
            let used = lw(g::usedobj);
            if used == 0 {
                guiDrawImage(20, 0x800080, 8e+01_f32, 1e+01_f32);
            }
            let fg = lw(g::menuitem_fgcol);
            let title = lw((lh((lw(g::editmode) << 1) + g::editmode_desc) << 2) + lw((lw(g::cur_language) << 2) + g::languages));
            textOutline(112, 8, title, fg, 128);
            if lw(g::usedobj) == 0 {
                guiDrawImage(21, 0x800080, 172.0_f32, 1e+01_f32);
            }
            mmDrawMemFree(272, 10);
            if lw(lw(g::front) + 20) == 0 {
                mmDrawPad(lw(g::w_x) + lw(g::w_w) - 128, 38);
            }
        }
    }
}

/// Relocate the parts of the character `a0` to the pose of the first frame of the animation file `a1` (@ 0x262cc0): every bone is set from the animation data, scaled by the
/// character's scale (a file whose name ends in the nine characters at 0x3a2710 gets a different scale), the matrices are rebuilt and, unless `a2` is set, the parts are
/// relocated.
pub fn relocateParts(a0: i32, a1: i32, a2: i32) {
    let frame = Frame::new(288);
    let sp = frame.sp();
    let mut i = 0;
    sw(sp + 116, a2);
    loadAnimation(a1, 2);
    let anim = getAnimation(2);
    let mut skeleton = lw(a0 + 88);
    sw(sp + 120, lw(skeleton + 12));
    sw(skeleton + 12, 0);
    if lw(anim + 8) > 0 {
        loop {
            let length = strlen(a1);
            if strcasecmp(a1 + (length - 9), 0x3a2710) != 0 {
                let scale = chrGetScale(lw(g::chrnew_lastchrnum));
                extractAnimData(a0, anim, i, 0, sp, scale);
            } else {
                let scale = chrGetScale(lw(g::chrnew_lastchrnum));
                extractAnimData(a0, anim, i, 0, sp, (scale).pdiv(lwc1(0x3acb90)));
            }
            let bone = i * 28;
            let matrix = (i << 6) + 0x70000000;
            i += 1;
            let target = bone + lw(lw(a0 + 88) + 4);
            let word1 = ld(sp + 8);
            let word2 = ld(sp + 16);
            let word3 = lw(sp + 24);
            sd(target, ld(sp));
            sd(target + 8, word1);
            sd(target + 16, word2);
            sw(target + 24, word3);
            let qy = lwc1(sp + 16);
            let qz = lwc1(sp + 20);
            let qw = lwc1(sp + 24);
            swc1(sp + 32, lwc1(sp + 12));
            swc1(sp + 36, qy);
            swc1(sp + 40, qz);
            swc1(sp + 44, qw);
            quaternionToMatrix(matrix, sp + 32);
            swc1(matrix + 48, lwc1(sp));
            swc1(matrix + 52, lwc1(sp + 4));
            swc1(matrix + 56, lwc1(sp + 8));
            if i >= lw(anim + 8) {
                break;
            }
        }
        skeleton = lw(a0 + 88);
    }
    sw(skeleton + 12, lw(sp + 120));
    sw(sp + 112, 0);
    matrixAlign(a0, 0x70000000, 0, sp + 112);
    calGlobalMatrices(a0, lw(a0 + 4), 0, -1);
    anim_SetMatrixArray(lw(a0 + 4));
    calDoubleFillets(a0, 0, 0.5_f32);
    let model = lw(a0 + 92);
    sceVu0CopyMatrix(lw(model + 36), lw(a0 + 4) + (lw(lw(model) + 44) << 6));
    let model = lw(a0 + 92);
    sceVu0CopyMatrix(lw(model + 40), lw(a0 + 4) + (lw(lw(model) + 8) << 6));
    sceVu0UnitMatrix(sp + 48);
    anim_SetModelview(sp + 48, 1);
    swc1(lw(a0 + 92) + 4, lwc1(lw(a0 + 4) + 52));
    if lw(sp + 116) == 0 {
        partRelocate(a0, 0);
    }
    anim_PopModelview();
}

/// Draw the tile that is being zoomed in the map maker (@ 0x250d88) into its own window: every part of the tile set is drawn, the one at the level of the edit marked in a
/// different blend, then the lights, the aim pad and the circle prop.
pub fn zoomtileGfx() {
    if (lw(g::zoomtilenum) as u32) >= (21 as u32) {
        return;
    }
    windowSave();
    let high = lw(0x3b0070);
    let wide = lw(0x3b006c);
    let x = lw(0x3b0064);
    let y = lw(0x3b0068);
    let half_w = (wide + ((wide as u32) >> 31) as i32) >> 1;
    let half_h = (high + ((high as u32) >> 31) as i32) >> 1;
    windowSet(x - half_w + 1, y - half_h + 1, x + half_w, y + half_h);
    windowGfx();
    let dither = dlSetDitherMatrix(2);
    dlSetZB(0);
    let object = lw(g::zoomtileobj);
    let marked = if (lw(g::zoomtileflags) & 8) == 0 { -1 } else { lw(g::editlevel) - lw(object + 20) };
    let mut tile = lw(g::zoomtilenum);
    let set = tile * 116 + g::tileset1;
    let mut part = 0;
    if lw(set + 8) > 0 {
        loop {
            let props = g::tileprops;
            sh(lw(lw((tile << 2) + props) + 32) + 292, 1 << (part & 31));
            let count;
            if part == marked {
                dlSetFixedRGBA(lbu(g::quickflash));
                dlSetBlend(4);
                obInstGfx(lw(lw((lw(g::zoomtilenum) << 2) + props) + 32), 0);
                dlSetBlend(2);
                dlSetFBMSK(-0xff01);
                obInstGfx(lw(lw((lw(g::zoomtilenum) << 2) + props) + 32), 0);
                dlSetFBMSK(0);
                count = lw(set + 8);
            } else {
                let before = (part < marked) as i32;
                let slot;
                if (lw(g::zoomtileflags) & 8) == 0 {
                    dlSetBlend(2);
                    slot = (lw(g::zoomtilenum) << 2) + props;
                } else {
                    if before == 0 {
                        dlSetFixedRGBA(48);
                    } else {
                        dlSetFixedRGBA(64);
                    }
                    dlSetBlend(4);
                    slot = (lw(g::zoomtilenum) << 2) + g::tileprops;
                }
                obInstGfx(lw(lw(slot) + 32), 0);
                count = lw(set + 8);
            }
            part += 1;
            tile = lw(g::zoomtilenum);
            if part >= count {
                break;
            }
        }
    }
    dlSetBlend(2);
    dlSetDitherMatrix(dither);
    if (lw(g::zoomtileflags) & 3) != 0 {
        let mut i = 0;
        dlSetZB(3);
        let mut count = lw(0x3b0078);
        if count > 0 {
            loop {
                let light = lw(i * 24 + 0x1fc410c);
                if (lw(light + 16) & 1) == 0 {
                    i += 1;
                } else {
                    obInstGfx(lw(light + 32), 0);
                    count = lw(0x3b0078);
                    i += 1;
                }
                if i >= count {
                    break;
                }
            }
        }
        let circle = lw(g::circleprop);
        if lw(g::aimpadinfo) != 0 {
            obInstGfx(lw(circle + 32), 0);
        }
    }
    dlSetBlend(1);
    windowSwitch();
    windowGfx();
}

/// Tick of the game type page (@ 0x23dd70): the items of the challenge mode and the next level are hidden or shown by what is available, the picked item sets the page the menu
/// goes to, and a button press on the item 4 starts the level 105 directly.
pub fn gametype_pageTick(a0: i32) {
    let frame = Frame::new(208);
    let sp = frame.sp();
    let languages = g::languages;
    let table = (lw(g::cur_language) << 2) + languages;
    sw(((lw(0x3aff2c) - 1) << 3) + 0x1fc1308, 0);
    pageSet(lw(lw(table) + 476), g::gametype_page);
    let mut text = 0;
    frontfxTick(3);
    let items = g::i0300;
    let players = lw(0x3aff20);
    let flags42 = lhu(items + 42);
    let flags74 = lhu(items + 74);
    sh(items + 10, lhu(items + 10) & 0xffbf);
    let cleared74 = flags74 & 0xffbf;
    sh(items + 42, flags42 & 0xffbf);
    sh(items + 74, cleared74);
    if players >= 2 {
        sh(items + 74, cleared74 | 64);
    }
    let page_item;
    if players < story_players() {
        page_item = lhu(a0 + 4);
    } else {
        sh(items + 10, lhu(items + 10) | 64);
        page_item = lhu(a0 + 4);
    }
    if page_item != 0 {
        let language = lw(g::cur_language);
        let mut skip = false;
        if !(page_item < 4) {
            if gt_item_far(page_item) {
                skip = true;
            } else if page_item < 5 {
                skip = true;
            }
        }
        if !skip {
            text = lw(lw((language << 2) + languages) + 512);
        }
    }
    let challenge = challengeModeAvail();
    let items = g::i0300;
    if challenge == 0 {
        if lhu(a0 + 4) == 3 {
            sh(a0 + 4, 2);
        }
        let flags = lhu(items + 74);
        sh(items + 38, gt_y(70));
        sh(items + 6, gt_y(50));
        sh(items + 74, flags | 0x4200);
    } else {
        let flags = lhu(items + 74);
        sh(items + 6, gt_y(40));
        sh(items + 70, gt_y(80));
        sh(items + 74, flags & 0xbdff);
        sh(items + 38, gt_y(60));
    }
    let level = levelAvail(105);
    let language = lw(g::cur_language);
    if level == 0 {
        let items = g::i0300;
        if lhu(a0 + 4) == 4 {
            sh(a0 + 4, 2);
        }
        sh(items + 82, 5);
        sh(items + 144, 3);
        sw(items + 96, 0);
    } else {
        let items = g::i0300;
        sh(items + 144, 4);
        sh(items + 82, 4);
        sw(items + 96, lw(lw((language << 2) + g::languages) + 1288));
    }
    let item = lhu(a0 + 4);
    sw(g::i0300 + 224, text);
    if item == 2 {
        sw(0x1fc1214, 3);
    } else if item >= 3 {
        if item == 3 {
            sw(0x1fc1214, 4);
        } else if item == 4 {
            if (joyNewBut(0) & 64) != 0 {
                frontmpgStoreGoodies();
                memset(sp, 0, 112);
                sw(sp + 64, 105);
                sw(sp + 4, 1);
                sw(sp, 1);
                sw(sp + 72, 0);
                gameDeferedReset(sp);
                sw(0x1fc11f8, 6);
                return;
            }
        }
    } else if item == 1 {
        sw(0x1fc1214, 2);
    }
    if (joyNewBut(0) & 16) != 0 {
        menusToSettings();
        setStates(1);
        pageOut();
        joyClearAll(0);
    }
    if av_gametype_exit_x(a0) {
        return;
    }
    let menu = av_gamemode_menu_or(g::gamemode_menu);
    menutick_230238(a0, menu);
    let made = std_menumake(lw(a0 + 8), menu);
    sw(a0 + 12, made);
}

/// Draw the arcade map load page (@ 0x239be8): the frame of the list, the highlight of the picked row, the scroll arrows and the date of every map file in the list.
pub fn arcade_mmload_pageDraw(a0: i32) {
    let frame = Frame::new(128);
    let sp = frame.sp();
    frontGetPageSize(a0, sp);
    let menu = g::mapfile_menu_354840;
    let front = lw(g::front);
    let y = lh(menu + 6);
    let x = lh(menu + 4);
    let bottom = y + 160;
    let right = x + 510;
    let top = y - 5;
    let left = x - 20;
    sw(sp + 8, right);
    sw(sp, left);
    sw(sp + 4, top);
    sw(sp + 12, bottom);
    let colour = if ((a0 - lw(front + 4)) >> 4 ^ lw(front + 20) - 1) != 0 { 0x3f3f3f7f } else { 0x7f7f7f7f };
    zoomboxDraw(left, top, right, bottom, 0x2880, colour);
    if ((lbu(0x3aff78) - 3) as u32) < (2 as u32) {
        let mut alpha = lbu(g::flashalpha);
        if (lw(0x1fc11fc) & 16) == 0 {
            if mcseqPollCount() <= 0 {
                return;
            }
            alpha = lbu(g::flashalpha);
        }
        let front = lw(g::front);
        let tint = ((((alpha as u32) >> 1) as i32) << 24) | 128 | (alpha << 8 | alpha << 16);
        if ((a0 - lw(front + 4)) >> 4) == lw(front + 20) - 1 {
            let found = frontGetBox(a0, lhu(a0 + 4));
            if found != 0 {
                let row = lw(g::window + 20) + lhu(found + 2);
                dlFillRectangle(lw(sp) + 20, row + 1, lw(sp + 8), row + lhu(found + 6) - 1, lbu(g::flashalpha) | 0x8000);
            }
            menuitemHighlight(a0);
        }
        let mut first = lw(g::firstmap_3ae840);
        if first > 0 {
            guiDrawImage(6, tint, lw(g::window + 8) as f32, (lh(g::mapfile_menu_354840 + 6) + 16) as f32);
            first = lw(g::firstmap_3ae840);
        }
        let mut shown = lw(0x3aff74);
        if (lw(lw(g::mapdir1) + 8) - (first + shown)) > 0 {
            guiDrawImage(5, tint, lw(g::window + 8) as f32, (lh(g::mapfile_menu_354840 + 6) + 127) as f32);
            shown = lw(0x3aff74);
        }
        if shown > 0 {
            if (lw(0x1fc11fc) & 16) == 0 {
                let mut row_box = frontGetBox(a0, 1);
                if row_box != 0 {
                    let mut i = 0;
                    if lw(0x3aff74) > 0 {
                        loop {
                            let entry = (lw(g::firstmap_3ae840) + i) * 40;
                            i += 1;
                            let time = rtcFormatTime(lw(lw(g::mapdir1) + entry + 44));
                            sb(time + 8, 0);
                            let clock = time + 9;
                            let row_y = lhu(row_box + 2);
                            row_box += 24;
                            sb(g::fontflags, lbu(g::fontflags) | 2);
                            let width = textWidth(clock);
                            let fg = lw(g::menuitem_fgcol);
                            let bg = lw(g::menuitem_bgcol);
                            textOutline(lw(sp + 8) - (width + 10), row_y, clock, fg, bg);
                            let width = textWidth(time);
                            let fg = lw(g::menuitem_fgcol);
                            let bg = lw(g::menuitem_bgcol);
                            textOutline(lw(sp + 8) - (width + 110), row_y, time, fg, bg);
                            let more = (i < lw(0x3aff74)) as i32;
                            sb(g::fontflags, lbu(g::fontflags) & 253);
                            if more == 0 {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Test the conditions of the challenge (@ 0x21f908): each condition reads a value (the game frame, a rank, a score, a kill count, a statistic or the result of a call), compares
/// it to the target and, depending on its flags, fails the challenge, stores the value, completes the level or ends the challenge as lost.
pub fn challengeTick() {
    let mut s5 = 0;
    let mut s4 = 0;
    let mut s3 = 1;
    let mut index = 0;
    let setup = challengeGetSetup();
    DisplayChallengeDebug();
    let mut count = lw(setup + 20);
    let mut a1_2 = 0;
    let mut a2 = 0;
    let mut value = 0;
    'scan: {
        if count > 0 {
            loop {
                let entry = lw(setup + 16) + index * 20;
                let source = lw(entry + 4);
                if (source as u32) < (9 as u32) {
                    let mut scaled = None;
                    match source {
                        0 => {
                            value = lw(g::gameframe);
                        }
                        1 => {
                            value = gameGetRankChr(lw(lw(entry + 8) * 0x1210 + lw(g::chrinfo)));
                        }
                        2 => {
                            value = gameGetRankTeam(lw(lw(entry + 8) * 0x1210 + lw(g::chrinfo) + 24));
                        }
                        3 => {
                            scaled = Some(lwc1(lw(lw(entry + 8) * 0x1210 + lw(g::chrinfo) + 3020) + 520));
                        }
                        4 => {
                            value = chrCountKills(lw(g::chrinfo) + lw(entry + 8) * 0x1210);
                        }
                        5 => {
                            value = chrCountLosses(lw(g::chrinfo) + lw(entry + 8) * 0x1210);
                        }
                        6 => {
                            value = teamGetScore(lw(lw(entry + 8) * 0x1210 + lw(g::chrinfo) + 24));
                        }
                        7 => {
                            let packed = lw(entry + 8);
                            let slot = ((packed as u32) >> 16) as i32;
                            let stat = packed & 0xffff;
                            scaled = Some(StatsGet(stat, slot));
                        }
                        _ => {
                            let ret = call_indirect(lw(entry + 8), [count, a1_2, a2, value, 0, 0, 0, 0], [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]);
                            value = ret.0;
                        }
                    }
                    if let Some(float) = scaled {
                        value = ps2float::mul(float, 65536.0_f32) as i32;
                    }
                    let compare = lw(entry + 12);
                    if (compare as u32) < (5 as u32) {
                        match compare {
                            0 => {
                                a2 = (value < lw(entry + 16)) as i32;
                            }
                            1 => {
                                let greater = (lw(entry + 16) < value) as i32;
                                a2 = greater ^ 1;
                            }
                            2 => {
                                a2 = ((value ^ lw(entry + 16)) as u32 == 0) as i32;
                            }
                            3 => {
                                a2 = (lw(entry + 16) < value) as i32;
                            }
                            _ => {
                                let less = (value < lw(entry + 16)) as i32;
                                a2 = less ^ 1;
                            }
                        }
                        a1_2 = lhu(entry);
                        s3 = if (a1_2 & 2) != 0 { if a2 != 0 { s3 } else { 0 } } else { s3 };
                        if (a1_2 & 8) != 0 {
                            sw(0x3aff00, value);
                        }
                        if (a1_2 & 1) != 0 {
                            if a2 != 0 {
                                s4 = 1;
                                s3 = 0;
                                break 'scan;
                            }
                        }
                        if (a1_2 & 4) != 0 {
                            s4 = a2;
                            s5 = 1;
                            count = lw(setup + 20);
                        } else {
                            count = lw(setup + 20);
                        }
                    } else {
                        count = lw(setup + 20);
                    }
                }
                index += 1;
                if index >= count {
                    break 'scan;
                }
            }
        }
    }
    if s3 != 0 {
        if !(s5 != 0 && s4 == 0) {
            sw(g::curgameparms + 80, lw(g::curgameparms + 80) | 0x10000000);
            gameLevelCompleted();
        }
    } else if s4 != 0 {
        sw(g::curgameparms + 80, lw(g::curgameparms + 80) | 0x20000000);
        gameLevelCompleted();
    }
    challengeMessagesTick();
}

/// Draw the map file page (@ 0x24d3d0), the map maker's version of the arcade map load page: the list scrolls over the files of the directory and every row shows the date
/// of its file.
pub fn mapfile_pageDraw(a0: i32) {
    let frame = Frame::new(128);
    let sp = frame.sp();
    let front = lw(g::front);
    let menu = g::mapfile_menu_356ba8;
    let y = lh(menu + 6);
    let x = lh(menu + 4);
    let bottom = y + 160;
    let right = x + 510;
    let top = y - 5;
    let left = x - 20;
    sw(sp + 8, right);
    sw(sp, left);
    sw(sp + 4, top);
    sw(sp + 12, bottom);
    let colour = if ((a0 - lw(front + 4)) >> 4 ^ lw(front + 20) - 1) != 0 { 0x3f3f3f7f } else { 0x7f7f7f7f };
    zoomboxDraw(left, top, right, bottom, 0x2880, colour);
    let flags = lw(g::editflags);
    if ((lbu(0x3b0038) - 3) as u32) < (2 as u32) {
        let mut alpha = lbu(g::flashalpha);
        if (flags & 0x10000) == 0 {
            let polled = mcseqPollCount();
            alpha = lbu(g::flashalpha);
            if polled <= 0 {
                return;
            }
        }
        let front = lw(g::front);
        let tint = ((((alpha as u32) >> 1) as i32) << 24) | 128 | (alpha << 8 | alpha << 16);
        if ((a0 - lw(front + 4)) >> 4) == lw(front + 20) - 1 {
            let found = frontGetBox(a0, lhu(a0 + 4));
            if found != 0 {
                let row = lw(g::window + 20) + lhu(found + 2);
                dlFillRectangle(lw(sp) + 20, row + 1, lw(sp + 8), row + lhu(found + 6) - 1, lbu(g::flashalpha) | 0x8000);
            }
            menuitemHighlight(a0);
        }
        let mut first = lw(g::firstmap_3ae9bc);
        if first > 0 {
            guiDrawImage(6, tint, lw(g::window + 8) as f32, (lh(g::mapfile_menu_356ba8 + 6) + 16) as f32);
            first = lw(g::firstmap_3ae9bc);
        }
        let mut shown = lw(0x3b0050);
        if first < lw(0x3b0058) - shown {
            guiDrawImage(5, tint, lw(g::window + 8) as f32, (lh(g::mapfile_menu_356ba8 + 6) + 127) as f32);
            shown = lw(0x3b0050);
        }
        let flags = lw(g::editflags);
        if shown > 0 {
            if (flags & 0x10000) == 0 {
                let mut row_box = frontGetBox(a0, 1);
                if row_box != 0 {
                    let mut rows = lw(0x3b0050);
                    let mut i = 0;
                    if rows > 0 {
                        loop {
                            let entry = lw(g::firstmap_3ae9bc) + i - lw(0x3b0054);
                            if entry >= 0 {
                                let time = rtcFormatTime(lw(lw(g::mapdir1) + entry * 40 + 44));
                                sb(time + 8, 0);
                                let clock = time + 9;
                                let row_y = lhu(row_box + 2);
                                sb(g::fontflags, lbu(g::fontflags) | 2);
                                let width = textWidth(clock);
                                let fg = lw(g::menuitem_fgcol);
                                let bg = lw(g::menuitem_bgcol);
                                textOutline(lw(sp + 8) - (width + 10), row_y, clock, fg, bg);
                                let width = textWidth(time);
                                let fg = lw(g::menuitem_fgcol);
                                let bg = lw(g::menuitem_bgcol);
                                textOutline(lw(sp + 8) - (width + 110), row_y, time, fg, bg);
                                rows = lw(0x3b0050);
                                sb(g::fontflags, lbu(g::fontflags) & 253);
                            }
                            i += 1;
                            row_box += 24;
                            if i >= rows {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Update the data of the camera or path-following prop `a0` (@ 0x276270): the state in `+352` holds a pointer into a table of 44-byte path steps, whose kind (1 to 8) says
/// whether to orbit a point, hold the angle, jump the pointer on or back, or snap to a position.
pub fn updateData(a0: i32) {
    let mut state = lw(a0 + 352);
    loop {
        let step = lw(state + 12);
        swc1(state + 40, lwc1(a0 + 48));
        let kind = lw(step) - 1;
        swc1(state + 44, lwc1(a0 + 52));
        swc1(state + 48, lwc1(a0 + 56));
        if (kind as u32) >= (8 as u32) {
            return;
        }
        match kind {
            0 | 1 => {
                let angle;
                if kind == 0 {
                    let turn = 3.6e+02_f32;
                    let half = 1.8e+02_f32;
                    let mut value = ps2float::add(ps2float::add(ps2float::sub(turn, lwc1(step + 24)), lwc1(state + 72)), half);
                    if turn <= value {
                        value = ps2float::sub(value, turn);
                    }
                    angle = ps2float::mul(value, lwc1(0x3ace04)).pdiv(half);
                } else {
                    let turn = 3.6e+02_f32;
                    let mut value = ps2float::add(ps2float::sub(turn, lwc1(step + 24)), lwc1(state + 72));
                    if turn <= value {
                        value = ps2float::sub(value, turn);
                    }
                    angle = ps2float::mul(value, lwc1(0x3ace08)).pdiv(1.8e+02_f32);
                }
                let sine = sinf(angle);
                let current = lw(state + 12);
                swc1(state + 28, ps2float::add(lwc1(a0 + 48), ps2float::mul(lwc1(step + 20), sine)));
                swc1(state + 32, lwc1(a0 + 52));
                let cosine = cosf(angle);
                swc1(state + 36, ps2float::add(lwc1(a0 + 56), ps2float::mul(lwc1(current + 20), cosine)));
                return;
            }
            2 => {
                return;
            }
            3 => {
                swc1(state + 64, lwc1(step + 28));
                return;
            }
            4 => {
                sw(state + 12, lw(state + 4));
            }
            5 => {
                let first = lw(state);
                swc1(state + 40, lwc1(step + 4));
                swc1(state + 44, lwc1(step + 8));
                swc1(state + 48, lwc1(step + 12));
                swc1(a0 + 48, lwc1(step + 4));
                swc1(a0 + 52, lwc1(step + 8));
                swc1(a0 + 56, lwc1(step + 12));
                swc1(state + 52, lwc1(step + 32));
                swc1(state + 72, lwc1(step + 16));
                if first == 0 {
                    swc1(state + 56, lwc1(step + 16));
                }
                let height = lwc1(step + 36);
                sw(state + 12, step + 44);
                swc1(state + 60, height);
                sw(state + 84, 0);
                swc1(state + 76, lwc1(step + 40));
                swc1(state + 68, lwc1(step + 28));
            }
            6 => {
                sw(state + 12, lw(state + 4) + lwc1(step + 40) as i32 * 44);
            }
            _ => {
                let mut next = step + 44;
                if lw(a0 + 4) == 313 {
                    soundStart(3, 0);
                    next = lw(state + 12) + 44;
                }
                sw(state + 12, next);
                state = lw(a0 + 352);
            }
        }
    }
}
