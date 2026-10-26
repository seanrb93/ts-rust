//! Functions of the enemies' reaction to hits and mines, the preload lists, animation loading, the credits tunnel, the level end and the weather tick.

use crate::*;

/// An enemy `a0` was hit by `a1`: unless it is the escort of the escort game, an enemy that is idle or in one of the four fight states and of another team than the shooter goes
/// after the shooter (state 8), and the others nearby are told (@ 0x2c7000).
pub fn enemyInformHit(a0: i32, a1: i32) {
    let record = lw(a0 + 352);
    if lw(record + 8) == 0 {
        let state;
        if lw(g::curgameparms + 72) != 4 {
            state = lw(record + 2716);
        } else {
            if a0 == lw(g::escort) {
                return;
            }
            state = lw(record + 2716);
        }
        if state == 8 || state == 128 || state == 0x10000 || state == 0x20000 {
            if lw(record + 24) != lw(lw(a1 + 352) + 24) {
                let target = suggestTarget(a0, a1, 3);
                sw(record + 1688, 0);
                let target_record = lw(target + 352);
                sw(record + 2692, 0);
                let old_state = lw(record + 2716);
                let pad = lw(target_record + 680);
                sw(record + 2716, 8);
                sw(record + 684, pad);
                sw(record + 2720, old_state);
                sw(record + 2788, target);
                sw(record + 2908, 0);
                sw(record + 2892, 0);
            }
        }
        enemyInformOthers(a0);
    }
}

/// An enemy detonates the remote mines it can see: each of its twenty mines that is armed (+72 above the limit), at rest and either within 7 or past 6 time units of its
/// own is exploded (@ 0x2c03e0).
pub fn enemyDetonateMines(a0: i32) {
    let mut left = 19;
    let limit = lwc1(0x3ad8a0);
    let mut x = lwc1(a0 + 48);
    let mut y = lwc1(a0 + 52);
    let mut at = lw(a0 + 352) + 0x11c0;
    let mut z = lwc1(a0 + 56);
    let mut mine = lw(at);
    loop {
        'b3: {
            left -= 1;
            at += 4;
            if mine != 0 {
                let age = lwc1(mine + 72);
                if limit < age {
                    if lwc1(mine + 48) == 0.0_f32 {
                        if lwc1(mine + 52) == 0.0_f32 {
                            if lwc1(mine + 56) == 0.0_f32 {
                                let dx = ps2float::sub(lwc1(mine + 24), x);
                                let dy = ps2float::sub(lwc1(mine + 28), y);
                                let dz = ps2float::sub(lwc1(mine + 32), z);
                                if !(49.0_f32 < ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz))) {
                                    if !(6.0_f32 < age) {
                                        break 'b3;
                                    }
                                }
                                bulletExplodeRemoteMine(mine);
                                x = lwc1(a0 + 48);
                                y = lwc1(a0 + 52);
                                z = lwc1(a0 + 56);
                            }
                        }
                    }
                }
            }
        }
        if left < 0 {
            break;
        }
        mine = lw(at);
    }
}

/// Preload the front end's effects: the three logo layers, two textures, the five flares and two more (@ 0x22a390).
pub fn frontfxPreload() {
    let mut at = g::logolayersname;
    for _ in 0..3 {
        let name = lw(at);
        at += 4;
        preloadAddTexName(name, 0);
    }
    preloadAddTexName(0x3a17d0, 0);
    preloadAddTexName(0x3a17f8, 0);
    let mut at = g::flareTexturenames_352a78;
    for _ in 0..5 {
        let name = lw(at);
        at += 4;
        preloadAddTexName(name, 0);
    }
    preloadAddTexName(0x3a1820, 0);
    preloadAddTexName(0x3a1840, 0);
}

/// Preload for the multiplayer front end: the effects, the interface, the zoom box, the models of the front end's props and the built-in maps (@ 0x2410a0).
pub fn frontmpgPreload() {
    frontfxPreload();
    guiPreload();
    zoomboxPreload();
    for prop in [361, 366, 367, 379, 370, 360, 381, 159, 160, 161, 162, 163, 164, 165, 166, 167, 168, 497] {
        propAddPreLoad(prop);
    }
    mmfilePreloadBuiltinMaps();
}

/// The tick of the pub sign: it sways in the wind (the strength along a direction, times a quarter of the frames) and gets a swing each whole tick; its matrix is turned about X
/// by the angle (@ 0x279bf0).
pub fn pubsignPropTick(a0: i32) {
    let frame = Frame::new(80);
    let sp = frame.sp();
    let mut i = 0;
    let model = lw(a0 + 32);
    let source = 0x3a85b8;
    let tail = lw(source + 8);
    sd(sp, ld(source));
    sw(sp + 8, tail);
    sceVu0UnitMatrix(lw(model + 4));
    let record = lw(a0 + 352);
    let wind = weatherCalcWindStrengthInDir(a0 + 48, lw(a0 + 12), sp);
    let ticks = lw(g::updateframesi);
    swc1(record + 20, ps2float::add(lwc1(record + 20), ps2float::mul(ps2float::mul(wind, 0.25_f32), lwc1(g::updateframesf))));
    if ticks > 0 {
        loop {
            i += 1;
            swingProp(a0, 0, 3.0_f32, lwc1(0x3aceb8), 0.0_f32);
            if i >= lw(g::updateframesi) {
                break;
            }
        }
    }
    let angle = ps2float::mul(lwc1(lw(a0 + 352) + 8), lwc1(0x3acebc)).pdiv(1.8e+02_f32);
    matrixRotX(lw(lw(a0 + 32) + 4) + 64, angle);
}

/// Set up the animation file `a0` as the animation number `a1`: its tables are laid out in the file's memory (the channels' data after the 16-byte aligned headers, with the sizes
/// by the channel kind) and the length in seconds is worked out from the frame count at 60 a second (@ 0x2147e0).
pub fn loadAnimation(a0: i32, a1: i32) {
    let memory = fileGetMem(a0);
    let mask = -16;
    let header = memory + 15 & mask;
    let slot = a1 << 2;
    let table = lw(header + 4);
    sw(slot + g::animRightTag, lw(header + 12));
    let channels = lw(header + 28);
    let length = (ps2float::sub(lw(header) as f32, 1.0_f32)).pdiv(6e+01_f32);
    let channel_table = (table << 2) + memory + 63 & mask;
    let mut data = ((lw(header + 4) << 2) + 15 & mask) + (memory + (channels << 5)) + 51 & mask;
    sw(header + 20, memory + 51 & mask);
    sw(slot + g::pAnimations, header);
    swc1(header + 16, length);
    sw(header + 32, channel_table);
    if channels > 0 {
        let mut i = 0;
        loop {
            let channel = channel_table + (i << 5);
            let kind = lw(channel + 4);
            sw(channel + 20, data);
            if kind == 2 {
                data += lw(channel + 12) << 4 | 12;
            } else if kind >= 3 {
                if kind == 4 {
                    data += (lw(channel + 12) - 1) * 12 + 28;
                } else if kind == 8 {
                    data = lw(channel + 12) * 28 + data;
                }
            } else if kind == 0 {
                data += 28;
            }
            i += 1;
            data = data + 15 & -16;
            sw(channel + 16, lw(header + 20));
            if i >= channels {
                break;
            }
        }
    }
}

/// Update the pad each player's character is going to: the pad closest among those linked to its pad is taken; every seventh frame one character's pad is checked (@ 0x2bf830).
pub fn updatePlayerPadPos() {
    let frame = Frame::new(160);
    let sp = frame.sp();
    let mut i = 0;
    if lw(g::lvnumplayers) > 0 {
        loop {
            let offset = i * 1820;
            let record = lw(lw(i * 1820 + lw(g::players) + 384) + 352);
            let linked = findpadslinked2pad(lw(record + 680), sp + 4, 19);
            let player = lw(g::players) + offset;
            sw(sp, lw(record + 680));
            let best = closestpadinlist(player + 140, linked + 1, sp);
            if best != lw(record + 680) {
                sw(record + 680, best);
            }
            i += 1;
            if i >= lw(g::lvnumplayers) {
                break;
            }
        }
    }
    let frames = lw(g::framecheck) + 1;
    sw(g::framecheck, frames);
    if frames >= 7 {
        enemyCheckValidPad(lw(g::updatechr));
        let players = lw(g::lvnumplayers);
        let next = lw(g::updatechr) + 1;
        sw(g::framecheck, 0);
        sw(g::updatechr, next);
        if next >= players {
            sw(g::updatechr, 0);
        }
    }
}

/// Restart the credits' tunnel: its 40 sections of 0x1450 bytes are linked into a ring from the head, and each gets its twelve quads made (@ 0x2ae750).
pub fn creditsRestartTunnel() {
    let mut data = lw(g::currenttunneldata);
    let mut link = data + 0x31820;
    let mut next = data + 0x31830;
    let base = data;
    let mut left = 38;
    sw(g::tunnelDataHead, data);
    loop {
        sw(link, next);
        left -= 1;
        link -= 0x1450;
        next -= 0x1450;
        if left < 0 {
            break;
        }
    }
    let mut section = 0;
    sw(base + 0x32c70, 0);
    loop {
        let following = section + 1;
        let mut quad = 0;
        let at = (39 - section) * 0x1450 + data;
        sw(at + 0x1444, section);
        loop {
            creditsSetQuad(at, section, following, quad);
            quad += 1;
            if quad >= 12 {
                break;
            }
        }
        section = following;
        data = lw(g::currenttunneldata);
        if section >= 40 {
            break;
        }
    }
    sw(g::currenttunnelsection, 0);
}

/// End the level: a game level ends every part of the game; the levels 101 to 105 end the music and players and the part of their own; then the parts every level ends, and the
/// memory card for the levels 102 and 103 (@ 0x226110).
pub fn lvEnd() {
    let level = lw(g::lvlevelnum) - 101;
    if (level as u32) >= 5 {
        musicEnd();
        cartridgesEnd();
        specialEnd();
        playerEnd();
        bulletEnd();
        gunEnd();
        hudEnd();
        padEnd();
        roomlightEnd();
        bgEnd();
        ingamemenuEnd();
        decalEnd();
    } else {
        match level {
            0 => {
                musicEnd();
                playerEnd();
                frontredEnd();
            }
            1 => {
                musicEnd();
                playerEnd();
                frontmpgEnd();
            }
            2 => {
                musicEnd();
                playerEnd();
                mapmakerEnd();
            }
            3 => {
                musicEnd();
                playerEnd();
            }
            4 => {
                musicEnd();
                playerEnd();
                creditsEnd();
            }
            _ => unreachable!(),
        }
    }
    enemyEnd();
    windowEnd();
    specialfxEnd();
    chrEnd();
    propEnd();
    particleEnd();
    timerEnd();
    zbtestEnd();
    soundEnd();
    if ((lw(g::lvlevelnum) - 102) as u32) < 2 {
        mcardEnd();
    }
    frontEnd();
    textEnd();
    packfileEnd();
    memEnd();
}

/// Preload for the players: the sky prop if the level has one, each player's character (the challenge's, the chosen one of the sign-on, or 0 for the forced game kinds), and the
/// four weapon props 427 to 430 (@ 0x27f4d8).
pub fn playerPreload() {
    let sky = lvGetSkyPropNum();
    if sky >= 0 {
        propAddPreLoad(sky);
    }
    let players = lw(g::lvnumplayers);
    let mut i = 0;
    if players > 0 {
        let mut parms = g::curgameparms;
        loop {
            let flags = lw(parms + 80);
            let mut character = 0;
            if (flags & 0x2000) == 0 {
                if (flags & 32) == 0 {
                    let player = signonGet(i);
                    character = lw(player * 2936 + lw(g::signonarray) + 116);
                } else {
                    let setup = challengeGetSetup();
                    character = lw(lw(setup + 48));
                }
            }
            i += 1;
            chrPreload(character, character);
            parms = g::curgameparms;
            if i >= lw(g::lvnumplayers) {
                break;
            }
        }
    }
    propAddPreLoad(427);
    propAddPreLoad(428);
    propAddPreLoad(429);
    propAddPreLoad(430);
}

/// The tick of a banner prop: it sways in the wind (the strength along a direction times a rate that depends on which half of the circle it faces) and gets a swing each whole
/// tick; its matrix is turned about X (@ 0x279e48).
pub fn bannerPropTick(a0: i32) {
    let frame = Frame::new(80);
    let sp = frame.sp();
    let record = lw(a0 + 352);
    let source = 0x3a85b8;
    let tail = lw(source + 8);
    sd(sp, ld(source));
    sw(sp + 8, tail);
    let wind = weatherCalcWindStrengthInDir(a0 + 48, lw(a0 + 12), sp);
    let sway = lwc1(record + 20);
    let push = if !(1.8e+02_f32 <= lwc1(a0 + 76)) {
        ps2float::mul(ps2float::mul(wind, lwc1(0x3aced4)), lwc1(g::updateframesf))
    } else {
        ps2float::mul(ps2float::mul(-wind, lwc1(0x3aced0)), lwc1(g::updateframesf))
    };
    let ticks = lw(g::updateframesi);
    let mut i = 0;
    swc1(record + 20, ps2float::add(sway, push));
    if ticks > 0 {
        loop {
            i += 1;
            swingProp(a0, 0, 2.0_f32, lwc1(0x3aced8), 7e+01_f32);
            if i >= lw(g::updateframesi) {
                break;
            }
        }
    }
    let angle = ps2float::mul(lwc1(lw(a0 + 352) + 8), lwc1(0x3acedc)).pdiv(1.8e+02_f32);
    matrixRotX(lw(lw(a0 + 32) + 4), angle);
}

/// The tick of the map maker's message page: the mouse is kept inside the zoom box, the page's own tick (or the default) gives the answer kept at 0x3b0030; with an answer the
/// sound 140 (answer 1) or 142 plays, the page count drops and the edit flag 0x4000 is cleared (@ 0x24c990).
pub fn message_pageTick_24c990() {
    let frame = Frame::new(32);
    let sp = frame.sp();
    let boxes = g::zoomboxes;
    let left = lw(boxes + 24);
    let top = lw(boxes + 20);
    let width = lw(boxes + 32);
    let bottom = top + lw(boxes + 28);
    sw(sp, left << 16);
    sw(sp + 4, (left + width) << 16);
    sw(sp + 8, top << 16);
    sw(sp + 12, bottom << 16);
    mmClampMouse(sp);
    let tick = lw(0x3b0028);
    let answer = if tick == 0 {
        defaultmessagetick_24c968()
    } else {
        call_indirect(tick, [0; 8], [0.0; 8]).0
    };
    sw(0x3b0030, answer);
    if answer == 0 {
        zoomboxTick(0);
    } else {
        if answer != 1 {
            soundStart(142, 0);
        } else {
            soundStart(140, 0);
        }
        let front = lw(g::front);
        let state = lw(0x3b0034);
        let flags = lw(g::editflags) & -0x4001;
        sw(front + 20, lw(front + 20) - 1);
        sw(g::editflags, flags);
        sw(front + 12, state);
    }
}

/// Whether a position is in free space of the kind `a0` (0 anywhere; 1 outdoors and a metre from a wall; 2 outdoors; 3 and 4 more than 1 and 1.5 away from a wall, squared)
/// (@ 0x2ae2e8).
pub fn specialfxIsSpace(a0: i32, a1: i32, a2: i32) -> i32 {
    let inside;
    'b2: {
        if a0 != 0 {
            if a0 == 2 {
                inside = specialfxIsOutdoors(a1, a2);
                break 'b2;
            }
            let distance = specialfxCalcDistSqFromWall(a1, a2);
            if a0 >= 3 {
                let limit;
                if a0 == 3 {
                    limit = 1.0_f32;
                } else {
                    if a0 != 4 {
                        return 0;
                    }
                    limit = 2.25_f32;
                }
                inside = if !(limit <= distance) { 0 } else { 1 };
                break 'b2;
            }
            if a0 != 1 {
                return 0;
            }
            if specialfxIsOutdoors(a1, a2) == 0 {
                return 0;
            }
            if !(1.0_f32 <= distance) {
                return 0;
            }
        }
        inside = 1;
    }
    inside
}

/// The tick of the confirm end game page: the button 64 on the first item leaves the player's menu, on the second ends the game for everyone (the level is completed with the
/// flag 0x20000000); otherwise the menu ticks and its items are made (@ 0x20fa70).
pub fn confirmendgame_pageTick(a0: i32) {
    if (joyNewBut(lw(lw(g::currentplayer))) & 64) == 0 {
        let menu = g::confirmendgame_menu;
        ingame_menutick(a0, menu);
        let made = std_menumake(lw(a0 + 8), menu);
        sw(a0 + 12, lw(a0 + 12) + made);
    } else {
        let choice = lhu(a0 + 4);
        if choice == 1 {
            ingamemenuExit(lw(lw(g::front)));
            return;
        }
        if choice == 2 {
            let players = lw(g::lvnumplayers);
            sw(0x3afe78, -1);
            let mut i = 0;
            let mut parms = g::curgameparms;
            if players > 0 {
                loop {
                    ingamemenuExit(i);
                    i += 1;
                    parms = g::curgameparms;
                    if i >= lw(g::lvnumplayers) {
                        break;
                    }
                }
            }
            sw(parms + 80, lw(parms + 80) | 0x20000000);
            gameLevelCompleted();
            return;
        }
    }
}

/// Make the list of pad icons for the tile `a0` of the map maker: each pad of the tile with the flag 16 (up to 64) gets a record of 24 bytes and a marker prop (235) (@ 0x250300).
pub fn mm_makepadinfo(a0: i32) {
    let mut i = 0;
    let tile = lw((a0 << 2) + g::tilepaddata);
    let count = lw(tile + 4);
    let mut pad = tile + 12;
    sw(0x3b0078, 0);
    if count > 0 {
        let mut flags = lw(pad + 4);
        loop {
            let mut slot = lw(0x3b0078);
            if (flags & 16) != 0 {
                let records = 0x1fc4108;
                let props = records + 4;
                let prop = lw(slot * 24 + props);
                sw(slot * 24 + records, pad);
                sw(slot * 24 + 0x1fc4110, 0);
                if prop == 0 {
                    let marker = propNew(235, 0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32);
                    slot = lw(0x3b0078);
                    sw(slot * 24 + props, marker);
                }
                mm_setpadicon(slot * 24 + records);
                sw(0x3b0078, lw(0x3b0078) + 1);
            }
            i += 1;
            pad += 28;
            if i >= count {
                break;
            }
            if lw(0x3b0078) >= 64 {
                break;
            }
            flags = lw(pad + 4);
        }
    }
    sw(g::aimpadinfo, 0);
}

/// Restart the background: the room props are made, and each room's list of objects gets every entry cleared and its four links set to -1 (@ 0x256670).
pub fn bgRestart() {
    if lw(g::lvlevelnum) != 100 {
        bgCreateRoomProps();
    } else {
        bgTileSetCreateRoomProps();
    }
    let rooms = lw(g::bgNumRooms);
    let mut room = 0;
    if rooms > 0 {
        let table = lw(g::levelrooms);
        let mut count = rooms;
        loop {
            let list = lw(room * 44 + table + 16);
            let next = room + 1;
            if list != 0 {
                if lw(list) != 0 {
                    let mut entry = lw(list);
                    let mut index = 1;
                    loop {
                        sw(entry, 0);
                        let mut link = entry + 188;
                        for _ in 0..4 {
                            sw(link, -1);
                            link -= 4;
                        }
                        let at = index;
                        entry = lw((at << 2) + list);
                        index = at + 1;
                        if entry == 0 {
                            break;
                        }
                    }
                    count = lw(g::bgNumRooms);
                }
            }
            room = next;
            if room >= count {
                break;
            }
        }
    }
}

/// Make a water prop: the prop gets the flag 0x40000000 and (for the water props 315 to 317) a model number, its pointers are allocated, the water texture info of the 16
/// textures is read from the preload list, and the vertex lists are prepared (@ 0x278aa0).
pub fn waterPropNew(a0: i32, f12: f32, f13: f32, f14: f32, f15: f32) {
    let prop = propNew(a0, f12, f13, f14, f15);
    sw(prop + 16, lw(prop + 16) | 0x40000000);
    if a0 == 316 {
        sw(prop + 12, 209);
    } else if a0 >= 317 {
        if a0 == 317 {
            sw(prop + 12, 210);
        }
    } else if a0 == 315 {
        sw(prop + 12, 209);
    }
    allocateOverridePointers(prop);
    sw(g::waterOffsets, 0);
    let mut info = g::watertexinfo;
    sw(g::waterOffsett, 0);
    let mut names = g::watertexnames;
    for _ in 0..16 {
        let name = lw(names);
        names += 4;
        let texture = preloadGetTexName(name);
        let value = lw(texture + 20);
        sw(info + 4, 0);
        sw(info, value);
        sw(info + 8, 0);
        sw(info + 12, 0);
        info += 16;
    }
    vtxlistWaterxyzPre(0, prop);
    vtxlistWaterxyzPre(1, prop);
}

/// Keep the settings of the next game in 0x1fbc0e8 (112 bytes copied from `a0`) and go to the loading state: unless the level is the credits (104) now or then, the music
/// ends and the state becomes 6; otherwise the level ends at once (@ 0x21e7a8).
pub fn gameDeferedReset(a0: i32) {
    let mut to = 0x1fbc0e8;
    let mut from = a0;
    loop {
        let b = ld(from + 8);
        let c = ld(from + 16);
        let d = ld(from + 24);
        sd(to, ld(from));
        sd(to + 8, b);
        sd(to + 16, c);
        sd(to + 24, d);
        from += 32;
        to += 32;
        if from == a0 + 96 {
            break;
        }
    }
    let tail = ld(from + 8);
    sd(to, ld(from));
    sd(to + 8, tail);
    let level = lw(g::lvlevelnum);
    if lw(a0 + 64) != 104 {
        if level != 104 {
            musicEnd();
            sw(0x3afefc, 5);
            sw(g::gamestate, 6);
            return;
        }
    }
    sw(g::lvend, 666);
}

/// Preload the interface's textures: five (numbered 5 down to 1) and then the other 37 except, in the level 103, those numbered 25 to 36 (@ 0x22e250).
pub fn guiPreload() {
    let mut name = 0x1fc0368;
    let mut number = 4;
    loop {
        sprintf(&[name as i64, 0x3a18d0_i64, (number + 1) as i64]);
        number -= 1;
        preloadAddTexName(name, 0);
        name -= 38;
        if number < 0 {
            break;
        }
    }
    let mut level = lw(g::lvlevelnum);
    let mut i = 0;
    let mut offset = 0;
    loop {
        'b5: {
            let next = i + 1;
            let name = offset + 0x1fc0390;
            let in_range = (((i - 24) as u32) < 12) as i32;
            if level != 103 {
                i = next;
                if in_range != 0 {
                    break 'b5;
                }
            }
            sprintf(&[name as i64, 0x3a18f8_i64, next as i64]);
            preloadAddTexName(name, 0);
            level = lw(g::lvlevelnum);
            i = next;
        }
        if i >= 37 {
            break;
        }
        offset = i * 38;
    }
}

/// The tick of the weather: the time moves on, the wind at that time is worked out and scaled by 14, and its sideways parts are pushed out to at least 9 either way (@ 0x2a84f0).
pub fn weatherTick() {
    let weather = g::weather;
    let time = ps2float::add(lwc1(weather + 60), ps2float::mul(lwc1(g::updateframesf), lwc1(0x3ad62c)));
    swc1(weather + 60, time);
    weatherCalcWindAtTime(weather + 48, time);
    let scale = 14.0_f32;
    let x = ps2float::mul(lwc1(weather + 48), scale);
    let z = ps2float::mul(lwc1(weather + 56), scale);
    swc1(weather + 48, x);
    swc1(weather + 56, z);
    let mut pushed = x;
    if !(0.0_f32 <= x) {
        let limit = -9.0_f32;
        if limit < x {
            pushed = limit;
        }
    } else {
        let limit = 9.0_f32;
        if x < limit {
            pushed = limit;
        }
    }
    swc1(weather + 48, pushed);
    let weather = g::weather;
    let nine = 9.0_f32;
    let up = lwc1(weather + 52);
    let mut up_pushed = up;
    if up < nine {
        up_pushed = nine;
    }
    let positive = 0.0_f32 <= lwc1(weather + 48);
    swc1(weather + 52, up_pushed);
    if !positive {
        let value = lwc1(weather + 56);
        let mut result = value;
        if -9.0_f32 < value {
            result = -9.0_f32;
        }
        swc1(weather + 56, result);
    } else {
        let mut value = lwc1(weather + 56);
        if value < nine {
            value = nine;
        }
        swc1(weather + 56, value);
    }
}
