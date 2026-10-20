//! Functions: the swinging chandelier and lantern, clearing a character's guns, the player's tick, the map maker's prop moves, preloading characters, the car prop, the goodies
//! page, the tile zoom reset, the guns held by a character and the preload execution.

use crate::*;

/// The tick of a chandelier: it is pushed by the wind at its position (a rate times the frames), swings on two axes each whole tick, and its model is turned by the angles
/// at +8 and +16 (@ 0x27a0d0).
pub fn chandelierPropTick(a0: i32) {
    let frame = Frame::new(112);
    let sp = frame.sp();
    let mut i = 0;
    let record = lw(a0 + 352);
    weatherGetWindAtPosition(a0 + 48, lw(a0 + 12), sp);
    let rate = lwc1(0x3acee8);
    let frames = lwc1(g::updateframesf);
    let y = lwc1(record + 24);
    let z = lwc1(record + 28);
    let ticks = lw(g::updateframesi);
    swc1(record + 20, lwc1(record + 20) + lwc1(sp) * rate * frames);
    swc1(record + 24, y + lwc1(sp + 4) * rate * frames);
    swc1(record + 28, z + lwc1(sp + 8) * rate * frames);
    if ticks > 0 {
        let a = lwc1(0x3aceec);
        let b = lwc1(0x3acef0);
        let c = 8e+01_f32;
        loop {
            i += 1;
            swingProp(a0, 0, a, b, c);
            swingProp(a0, 2, a, b, c);
            if i >= lw(g::updateframesi) {
                break;
            }
        }
    }
    let record = lw(a0 + 352);
    let factor = lwc1(0x3acef4);
    let half = 1.8e+02_f32;
    let roll = lwc1(record + 16) * factor / half;
    let pitch = lwc1(record + 8) * factor / half;
    matrixRotZXY(lw(lw(a0 + 32) + 4), pitch, 0.0_f32, roll);
}

/// Take away everything a character holds (not for the kinds 2 and 3): the held item, the left and right hand guns are dropped, the gun fields are reset, and the remote mines
/// are forgotten (@ 0x288928).
pub fn chrClearGuns(a0: i32) {
    if ((lw(a0 + 8) - 2) as u32) >= 2 {
        let held = lw(a0 + 484);
        let left_gun;
        if held == 0 {
            left_gun = lw(a0 + 440);
        } else {
            updateTeamFlags(lw(a0 + 3020), held, 0);
            propDrop(lw(a0 + 484), a0, 0);
            sw(a0 + 484, 0);
            left_gun = lw(a0 + 440);
        }
        if left_gun != 0 {
            propDrop(left_gun, a0, a0 + 376);
            sw(a0 + 440, 0);
        }
        let right_gun = lw(a0 + 332);
        if right_gun != 0 {
            propDrop(right_gun, a0, a0 + 268);
            sw(a0 + 332, 0);
        }
        sw(a0 + 260, 0);
        sw(a0 + 268, -1);
        sw(a0 + 376, -1);
        sw(a0 + 264, 0);
        let mut at = a0 + 616;
        let mut left = 32;
        loop {
            sw(at, 0);
            left -= 1;
            at -= 4;
            if left < 0 {
                break;
            }
        }
        sw(a0 + 2704, lw(a0 + 2704) & -7);
        let mut mine_at = a0 + 0x11c0;
        let mut left = 19;
        loop {
            let mine = lw(mine_at);
            if mine != 0 {
                sw(mine + 108, -1);
            }
            sw(mine_at, 0);
            left -= 1;
            mine_at += 4;
            if left < 0 {
                break;
            }
        }
    }
}

/// The tick of a lantern: the same as the chandelier with its own rates (@ 0x279f70).
pub fn lanternPropTick(a0: i32) {
    let frame = Frame::new(112);
    let sp = frame.sp();
    let mut i = 0;
    let record = lw(a0 + 352);
    weatherGetWindAtPosition(a0 + 48, lw(a0 + 12), sp);
    let rate = 0.25_f32;
    let frames = lwc1(g::updateframesf);
    let y = lwc1(record + 24);
    let z = lwc1(record + 28);
    let ticks = lw(g::updateframesi);
    swc1(record + 20, lwc1(record + 20) + lwc1(sp) * rate * frames);
    swc1(record + 24, y + lwc1(sp + 4) * rate * frames);
    swc1(record + 28, z + lwc1(sp + 8) * rate * frames);
    if ticks > 0 {
        let a = 3.0_f32;
        let b = lwc1(0x3acee0);
        let c = 8e+01_f32;
        loop {
            i += 1;
            swingProp(a0, 0, a, b, c);
            swingProp(a0, 2, a, b, c);
            if i >= lw(g::updateframesi) {
                break;
            }
        }
    }
    let record = lw(a0 + 352);
    let factor = lwc1(0x3acee4);
    let half = 1.8e+02_f32;
    let roll = lwc1(record + 16) * factor / half;
    let pitch = lwc1(record + 8) * factor / half;
    matrixRotZXY(lw(lw(a0 + 32) + 4), pitch, 0.0_f32, roll);
}

/// The tick of the current player in a level: its front end and window, the menus (the pause with the button 2048 or when the pad is gone, the in-game menu with 256), and for
/// a game level the player, hud, camera, gun, portals and props ticks; the other levels tick the camera only (@ 0x2265a8).
pub fn lvTickPlayer() {
    frontSet(lw(lw(g::currentplayer)));
    windowTick();
    let player = lw(g::currentplayer);
    windowSet(lw(player + 4), lw(player + 8), lw(player + 12), lw(player + 16));
    frontTick();
    let level = lw(g::lvlevelnum) - 101;
    if (level as u32) >= 5 {
        if lw(lw(g::currentplayer)) == 0 {
            menuTick();
        }
        'b2: {
            let mode = debugmenuGetJoyMode();
            if mode != 1 {
                let pressed = joyNewBut(lw(lw(g::currentplayer)));
                if (pressed & 2048) == 0 {
                    if lw(g::joy_justdisconnected) == 0 {
                        let pressed = joyNewBut(lw(lw(g::currentplayer)));
                        if (pressed & 256) == 0 {
                            break 'b2;
                        }
                        ingamemenuStart(lw(lw(g::currentplayer)));
                        joyClearButs(lw(lw(g::currentplayer)), 256);
                        break 'b2;
                    }
                }
                ingamemenuPause(lw(lw(g::currentplayer)));
                joyClearButs(lw(lw(g::currentplayer)), 2048);
            }
        }
        playerTick();
        hudTick();
        camTick();
        gunTick();
        bgPortalTick();
        propTickPlayer();
        return;
    }
    match level {
        0 | 1 | 3 | 4 => camTick(),
        _ => {}
    }
}

/// Move the prop being edited in the map maker to where the player looks: the player's own props are left out of a line test along the view, the hit position is the prop's, and
/// its angle follows the view direction (@ 0x27c2b0).
pub fn propEditMove(a0: i32) {
    let frame = Frame::new(64);
    let sp = frame.sp();
    let player = lw(g::currentplayer);
    let chr = lw(player + 384);
    sw(a0 + 16, lw(a0 + 16) & -33);
    let record = lw(chr + 352);
    let right = lw(record + 332);
    sw(chr + 16, lw(chr + 16) & -33);
    if right != 0 {
        sw(right + 16, lw(right + 16) & -33);
    }
    let left = lw(record + 440);
    if left != 0 {
        sw(left + 16, lw(left + 16) & -33);
    }
    let room = lw(player + 796);
    let room = if room == 0 { 1 } else { room };
    sw(sp, 0);
    sw(sp + 8, 0);
    let hit = bgLineTest(room, player + 152, lw(lw(player + 384) + 352) + 2976, 1, player + 832, player + 844, 0, 0);
    let player = lw(g::currentplayer);
    let right = lw(record + 332);
    let chr = lw(player + 384);
    sw(player + 828, hit);
    sw(chr + 16, lw(chr + 16) | 32);
    if right != 0 {
        sw(right + 16, lw(right + 16) | 32);
    }
    let left = lw(record + 440);
    if left != 0 {
        sw(left + 16, lw(left + 16) | 32);
    }
    swc1(a0 + 48, lwc1(player + 832));
    swc1(a0 + 52, lwc1(player + 836));
    swc1(a0 + 56, lwc1(player + 840));
    let angle = atan2f(lwc1(player + 844), lwc1(player + 852));
    let degrees = angle * 1.8e+02_f32 / lwc1(0x3acfa4);
    sw(a0 + 16, lw(a0 + 16) | 32);
    swc1(a0 + 76, degrees);
}

/// Preload the character type `a0`: its body model, up to 18 sounds of its first list and 6 of its second (each list ends at a 0), and the head prop (@ 0x2887e8).
pub fn chrPreload(a0: i32, a1: i32) {
    let entry = a0 * 44 + g::chrs;
    let sounds = lw(entry + 4);
    propAddPreLoad(lw(entry + 8));
    let mut i = 0;
    if lh(sounds) != 0 {
        let mut at = sounds;
        loop {
            i += 1;
            soundAddPreLoad(lh(at));
            let offset = i << 1;
            let more = i < 18;
            if lh(sounds + offset) == 0 {
                break;
            }
            at = sounds + offset;
            if !more {
                break;
            }
        }
    }
    let second = sounds + 36;
    let mut j = 0;
    if lh(sounds + 36) != 0 {
        let mut at = second;
        loop {
            j += 1;
            soundAddPreLoad(lh(at));
            let offset = j << 1;
            let more = j < 6;
            if lh(second + (j << 1)) == 0 {
                break;
            }
            at = second + offset;
            if !more {
                break;
            }
        }
    }
    let head = chrGetChrHeadProp(a0, a1);
    if head == -1 {
        return;
    }
    propAddPreLoad(head);
}

/// Choose the kind of space for a special effect to spawn in: a random kind among 3 (when `a0` is under the number of players) or 5 is tried downwards against the
/// distance to the wall and whether the spot is outdoors, giving 0 to 4 (@ 0x2ae400).
pub fn specialfxGetBestSpawnType(a0: i32, a1: i32, a2: i32) -> i32 {
    let limit = if a0 < lw(g::lvnumplayers) { 3 } else { 5 };
    let roll = newrnd();
    let mut best = -1;
    if limit == 0 {
        trap("break");
    }
    'b2: {
        let mut kind = ((roll as i32 as u32) % (limit as u32)) as i32;
        if lw(g::lvnumplayers) < 3 {
            if kind != 0 {
                let mut distance = 0.0_f32;
                loop {
                    let mut in_range = ((kind as u32) < 5) as i32;
                    if kind != 3 {
                        if distance == 0.0_f32 {
                            distance = specialfxCalcDistSqFromWall(a1, a2);
                            in_range = ((kind as u32) < 5) as i32;
                        }
                    }
                    'b6: {
                        if in_range != 0 {
                            match kind {
                                0 => {
                                    best = 0;
                                }
                                1 => {
                                    if !(1.0_f32 <= distance) {
                                        kind -= 1;
                                        break 'b6;
                                    }
                                    best = 3;
                                }
                                2 => {
                                    if !(2.25_f32 <= distance) {
                                        kind -= 1;
                                        break 'b6;
                                    }
                                    best = 4;
                                }
                                3 | 4 => {
                                    if specialfxIsOutdoors(a1, a2) == 0 {
                                        kind = 3;
                                    } else {
                                        best = 2;
                                        if 1.0_f32 <= distance {
                                            best = 1;
                                        }
                                    }
                                }
                                _ => unreachable!(),
                            }
                        }
                        kind -= 1;
                    }
                    if best != -1 {
                        break 'b2;
                    }
                }
            }
        }
        best = 0;
    }
    best
}

/// Make a car prop: the prop is made at the position of the record `a2` and gets the six values; the data is updated, the pose set, the closest pad found, and the cars 311 to
/// 313 get their own tick function (0x2775c0 or 0x277860), flags and (the car 312) a looping sound (@ 0x2764b0).
pub fn carPropNew(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32, a6: i32) -> i32 {
    let frame = Frame::new(176);
    let sp = frame.sp();
    let y = lwc1(a2 + 8);
    let z = lwc1(a2 + 12);
    let x = lwc1(a2 + 4);
    let prop = propNew(a0, x, y, z, 0.0_f32);
    let record = lw(prop + 352);
    sw(record, a1);
    sw(record + 12, a2);
    sw(record + 8, a3);
    sw(record + 16, a4);
    sw(record + 20, a5);
    sw(record + 24, a6);
    sw(record + 4, a2);
    swc1(record + 80, 0.0_f32);
    swc1(record + 88, 0.0_f32);
    swc1(record + 92, 0.0_f32);
    swc1(record + 96, 0.0_f32);
    updateData(prop);
    obInstSetAnim(lw(prop + 32), 1, 0.0_f32);
    swc1(sp, lwc1(prop + 508));
    swc1(sp + 4, lwc1(prop + 52));
    swc1(sp + 8, lwc1(prop + 516));
    let pad = closestpad(sp, 0.0_f32);
    sw(record + 100, pad);
    swc1(record + 104, 0.0_f32);
    let mut kind = lw(prop + 4);
    if kind == 311 {
        let flags = lw(prop + 16) | 16;
        sw(prop + 400, 0x2775c0);
        sw(prop + 16, flags);
    }
    if kind == 312 {
        let players = lw(g::lvnumplayers);
        let flags = lw(prop + 16) | 0x40000010;
        sw(prop + 400, 0x277860);
        sw(prop + 16, flags);
        if players >= 3 {
            sw(prop + 12, 0);
        } else {
            sw(prop + 12, 212);
        }
        sh(lw(prop + 32) + 292, 0);
        let sound = soundStartLoopRange(26, prop + 48, 1.8e+02_f32);
        sw(record + 108, sound);
        kind = lw(prop + 4);
    }
    if kind == 313 {
        let model = lw(prop + 32);
        let flags = lw(prop + 16) | 0x40000000;
        sw(prop + 12, 212);
        sw(prop + 16, flags);
        sh(model + 292, 0);
    }
    prop
}

/// The tick of the goodies page: the frame counter at 0x3affb0 runs through the message phases (60 to 240); the button 64 skips to the next phase; at the end the next goodie
/// message is shown and, when none is left, the stats update page or the last game page is shown (@ 0x240d18).
pub fn goodies_pageTick() {
    pageRoot();
    pageSet(0, g::nplayers_page);
    frontfxTick(4);
    let finished = frontfxFinished(4);
    let mut counter = lw(0x3affb0);
    if finished != 0 {
        'b1: {
            'b2: {
                let mut next;
                'b3: {
                    let under240 = counter < 240;
                    if ((counter - 68) as u32) >= 120 {
                        let under60 = counter < 60;
                        if !under240 {
                            break 'b2;
                        }
                        let mut under181 = counter < 181;
                        if under60 {
                            let pressed = joyNewBut(0);
                            next = 60;
                            if (pressed & 64) != 0 {
                                sw(0x3affb0, next);
                                break 'b3;
                            }
                            counter = lw(0x3affb0);
                            under181 = counter < 181;
                        }
                        next = counter + 2;
                        if !under181 {
                            let pressed = joyNewBut(0);
                            next = 241;
                            if (pressed & 64) != 0 {
                                sw(0x3affb0, next);
                                break 'b3;
                            }
                            next = lw(0x3affb0) + 2;
                        }
                        sw(0x3affb0, next);
                    } else {
                        let pressed = joyNewBut(0);
                        next = 188;
                        if (pressed & 64) == 0 {
                            next = lw(0x3affb0);
                        } else {
                            sw(0x3affb0, next);
                        }
                    }
                }
                if next < 240 {
                    break 'b1;
                }
            }
            let message = getNextGoodieString();
            goodieMessage(message);
            if lw(0x3affb0) == 0 {
                if (lw(0x1fc11fc) & 0x10000) != 0 {
                    if setStatsUpdatePage() != 0 {
                        break 'b1;
                    }
                }
                setlastgamemodepage();
                return;
            }
        }
    }
}

/// Reset the tile zoom: each of the 21 tiles has its object loaded, a prop made (498 up) and the pad data of its file taken from the preload list; the eleven icon objects
/// are loaded into the prop table's entries; and the circle prop (234) is made (@ 0x2501c8).
pub fn zoomtileReset() {
    let mut pad_data = g::tilepaddata;
    let mut pad_names = g::tilepadfilenames;
    let mut props = g::tileprops;
    let mut names = g::tilefilenames;
    let mut i = 0;
    loop {
        let object = obLoad(names);
        let number = i + 498;
        i += 1;
        propSetOb(number, object);
        names += 64;
        let prop = propNew(number, 0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32);
        sw(props, prop);
        let file = preloadGetFile(pad_names);
        pad_names += 64;
        props += 4;
        sw(pad_data, lw(file + 16));
        pad_data += 4;
        if i >= 21 {
            break;
        }
    }
    let mut icon = g::iconobs;
    let mut entry = g::proptable + 6556;
    let mut left = 11;
    loop {
        let object = lw(entry);
        if object != 0 {
            sw(icon, object);
        } else {
            let loaded = obLoad(lw(entry + 8));
            sw(entry, loaded);
            sw(icon, loaded);
        }
        entry += 28;
        left -= 1;
        icon += 4;
        if left < 0 {
            break;
        }
    }
    let circle = propNew(234, 0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32);
    sw(g::circleprop, circle);
}

/// Keep the gun a character holds in one hand (`a1` 0 for the right hand: the slot at +376, otherwise at +268) in line with the gun combo it has: the gun prop is deleted and
/// a new one made and attached to the hand bone when the gun changed; the prop's visible flag follows the view guns setting (@ 0x294528).
pub fn gunChrUpdate(a0: i32, a1: i32) {
    let record = lw(a0 + 352);
    let slot = if a1 != 0 { record + 268 } else { record + 376 };
    let combo = (lw(record + 260) << 3) + g::guncombos;
    let mut gun = if a1 == 0 { lh(combo + 4) } else { lh(combo + 2) };
    if a1 == 0 {
        if gunRightHandAble(record) == 0 {
            gun = -1;
        }
    } else if gunLeftHandAble(record) == 0 {
        gun = -1;
    }
    let current = lw(slot);
    let mut prop = lw(slot + 64);
    if current != gun {
        if prop != 0 {
            propDelete(prop);
            sw(slot + 64, 0);
        }
        sw(slot, gun);
        if gun == -1 {
            prop = lw(slot + 64);
        } else {
            let new_prop = propNew(lw(gun * 400 + (g::guns + 4)), lwc1(a0 + 48), lwc1(a0 + 52), lwc1(a0 + 56), 0.0_f32);
            sw(slot + 64, new_prop);
            let bone = if a1 == 0 { lw(lw(lw(a0 + 32) + 92) + 28) } else { lw(lw(lw(a0 + 32) + 92) + 24) };
            propSetAttached(lw(slot + 64), lw(a0 + 32), bone, if a1 == 0 { 1 } else { 2 });
            let owner = lw(record + 3020);
            prop = lw(slot + 64);
            swc1(prop + 564, lwc1(owner + 48));
            swc1(prop + 568, lwc1(owner + 52));
            swc1(prop + 572, lwc1(owner + 56));
        }
    }
    let view = lw(g::viewguns);
    if prop != 0 {
        if view == 0 {
            let flags = lw(prop + 16);
            sw(prop + 16, flags & -2);
        } else {
            sw(prop + 16, lw(prop + 16) | 1);
        }
    }
}

/// Run the preload list: each record is loaded by its kind (0 a file, 1 an object file and its textures, 2 a texture by name, 3 a texture by number) (@ 0x21c728).
pub fn preloadExec() {
    let mut record = lw(0x3afee0);
    if record != 0 {
        let mut kind = lw(record);
        loop {
            'b3: {
                let below_two = kind < 2;
                if kind == 1 {
                    let file = lw(record + 16);
                    let name = lw(record + 12);
                    if file == 0 {
                        let loaded = fileLoad(name, 1, record + 20);
                        sw(record + 16, loaded);
                    } else {
                        let size = fileLoadTo(name, file, 0x2000000);
                        sw(record + 20, size);
                    }
                    obPreloadTextures(lw(record + 12));
                    record = lw(record + 4);
                } else {
                    if !below_two {
                        if kind == 2 {
                            emLoadTextureName(lw(record + 12), lw(record + 16), record + 20);
                            record = lw(record + 4);
                            break 'b3;
                        }
                        if kind != 3 {
                            record = lw(record + 4);
                            break 'b3;
                        }
                        emLoadTextureNum(lw(record + 12), lw(record + 16), record + 20);
                    } else {
                        if kind != 0 {
                            record = lw(record + 4);
                            break 'b3;
                        }
                        let file = lw(record + 16);
                        let name = lw(record + 12);
                        if file == 0 {
                            let loaded = fileLoad(name, 1, record + 20);
                            sw(record + 16, loaded);
                        } else {
                            let size = fileLoadTo(name, file, 0x2000000);
                            sw(record + 20, size);
                        }
                    }
                    record = lw(record + 4);
                }
            }
            if record == 0 {
                break;
            }
            kind = lw(record);
        }
    }
}
