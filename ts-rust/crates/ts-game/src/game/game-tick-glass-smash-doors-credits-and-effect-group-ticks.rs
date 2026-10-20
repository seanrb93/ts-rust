//! Functions: the game tick, smashing glass, doors, the portal tick, the credits background, a dropping pickup, the groups of special effects, drawing an object instance, the
//! stats update page, a prop in range of a moving prop and pasting in the map maker.

use crate::*;

/// The tick of the stats update page: the sign-on update or the message of the player in turn is run; when it is done the next player with something to update is taken, and
/// at the end the last game mode page is shown (@ 0x240100).
pub fn statsupdate_pageTick() {
    'b1: {
        'b2: {
            pageRoot();
            pageSet(lw(lw((lw(g::cur_language) << 2) + g::languages) + 468), g::nplayers_page);
            frontfxTick(0);
            let flags = lw(0x1fc11fc);
            if (flags & 2) != 0 {
                if (flags & 1) != 0 {
                    if signonUpdateTick(lw(0x3aff38)) == 0 {
                        break 'b1;
                    }
                } else {
                    let state = splitMessageTick(lw(0x3aff38));
                    if state != 0 {
                        splitMessageClear(lw(0x3aff38));
                    }
                    if state == 2 {
                        signonUpdateStart();
                        return;
                    }
                    if state != 3 {
                        break 'b1;
                    }
                }
                let mut player = lw(0x3aff38);
                'b7: {
                    loop {
                        let next = player + 1;
                        let more = (next < lw(0x3aff20)) as i32;
                        sw(0x3aff38, next);
                        if more == 0 {
                            break 'b7;
                        }
                        let slot = signonGet(next);
                        player = lw(0x3aff38);
                        if lw((slot << 3) + g::signoninfo) != 8 {
                            break;
                        }
                    }
                    let state = 0x1fc11f8;
                    if lw(0x3aff38) < lw(0x3aff20) {
                        sw(state + 4, lw(state + 4) & -2);
                        break 'b2;
                    }
                }
                pageSet(0, g::nplayers_page);
                setlastgamemodepage();
                return;
            }
        }
        let player = lw(0x3aff38);
        let text = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1924);
        let message = GetPlayerSignOnMessage(player, text);
        splitMessageStart(lw(0x3aff38), 1, message);
        let state = 0x1fc11f8;
        sw(state + 4, lw(state + 4) | 2);
    }
}

/// Whether a moving prop (a character in a state without 0x2010, or a kind 256 prop) is within `f12` of the position `a0` and visible from the room `a1` (a line test with
/// -3): the props from the room count upwards are searched (@ 0x274240).
pub fn propMovingPropInRange(a0: i32, a1: i32, f12: f32) -> i32 {
    let frame = Frame::new(176);
    let sp = frame.sp();
    let mut count = lw(g::num_props);
    let mut i = lw(g::bgNumRooms);
    if i < count {
        let mut offset = i * 592;
        loop {
            'b5: {
                let prop = lw(g::levelprops) + offset;
                if (lw(prop + 16) & i32::MIN) != 0 {
                    i += 1;
                } else {
                    let x;
                    'b6: {
                        let kind = lw(prop + 8);
                        if kind == 8 {
                            let state = lw(lw(prop + 352) + 2716);
                            if (state & 0x2010) == 0 {
                                if state != 0 {
                                    x = lwc1(a0);
                                    break 'b6;
                                }
                            }
                        }
                        if kind != 256 {
                            i += 1;
                            break 'b5;
                        }
                        x = lwc1(a0);
                    }
                    let y = lwc1(a0 + 4);
                    let dx = lwc1(prop + 48) - x;
                    let z = lwc1(a0 + 8);
                    swc1(sp + 16, dx);
                    let dy = lwc1(prop + 52) + 1.0_f32 - y;
                    swc1(sp + 20, dy);
                    let dz = lwc1(prop + 56) - z;
                    swc1(sp + 24, dz);
                    if (dx * dx + dy * dy + dz * dz) < f12 * f12 {
                        sw(sp, 0);
                        sw(sp + 8, 0);
                        let hit = bgLineTest(a1, a0, sp + 16, 0, 0, 0, 0, -3);
                        count = lw(g::num_props);
                        if hit == 0 {
                            return 1;
                        }
                    }
                    i += 1;
                }
            }
            offset += 592;
            if i >= count {
                break;
            }
        }
    }
    0
}

/// Paste the held objects of the map maker at the cursor: each object of the chain is copied to a new one that is moved by the cursor's cell (tiles snap to the grid) and
/// placed; if one cannot be placed the pasted ones are freed again and the answer is 0 (@ 0x246140).
pub fn mmDoPaste() -> i32 {
    mmBackupMap();
    let scale_x = lw(g::scale_wtomx);
    let x_offset = (lh(lw(g::front) + 34) - lw(g::w_x)) << 16;
    let fx = FMul(x_offset, scale_x);
    let scale_y = lw(g::scale_wtomy);
    let origin_x = lw(g::m_x) + fx as i32 + 0x100000;
    let y_offset = (lh(lw(g::front) + 38) - lw(g::w_y)) << 16;
    let fy = FMul(y_offset, scale_y);
    let mut object = lw(g::usedobj);
    let origin_y = lw(g::m_y) + fy as i32 + 0x100000;
    if object != 0 {
        let mut copy;
        loop {
            copy = mmAllocObject();
            if copy == 0 {
                break;
            }
            let b = ld(object + 8);
            let c = ld(object + 16);
            let d = ld(object + 24);
            sd(copy, ld(object));
            sd(copy + 8, b);
            sd(copy + 16, c);
            sd(copy + 24, d);
            let e = ld(object + 40);
            let f = ld(object + 48);
            let last = lw(object + 56);
            sd(copy + 32, ld(object + 32));
            sd(copy + 40, e);
            sd(copy + 48, f);
            sw(copy + 56, last);
            let kind = lw(object);
            let x = lw(object + 12) + origin_x;
            sw(object + 52, 0);
            sw(copy + 12, x);
            let y = lw(object + 16) + origin_y;
            sw(copy + 16, y);
            if kind == 1 {
                sw(copy + 12, x & -0x200000);
                sw(copy + 16, y & -0x200000);
            }
            let level = lw(object + 20);
            let edit_level = lw(g::editlevel);
            sw(copy + 56, 0);
            sw(copy + 20, level + edit_level);
            if mmPlaceObject(copy) == 0 {
                mmFreeObject(copy);
                mmFreeObject(lw(g::usedobj));
                sw(g::usedobj, 0);
                return 0;
            }
            object = lw(object + 56);
            if object == 0 {
                break;
            }
        }
    }
    mmFreeObject(lw(g::usedobj));
    sw(g::usedobj, 0);
    1
}

/// Smash a pane of glass `a0` hit at the position `a1`: the glass is split into pieces (`particleNewGlass`) with the position inside the pane worked out as fractions along
/// its two edges, the pane's strength drops to -60 and one of four sounds plays (@ 0x259780).
pub fn bgSmashGlass(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, f12: f32) {
    let frame = Frame::new(240);
    let sp = frame.sp();
    let position = a0 + 16;
    let roll = newrnd();
    let origin_x = lwc1(a0 + 56);
    let sound = roll & 3_i64;
    let origin_y = lwc1(a0 + 60);
    let f6 = lwc1(a0 + 16) - origin_x;
    let f7 = lwc1(a0 + 20) - origin_y;
    let f8 = lwc1(a0 + 96) - origin_x;
    let f9 = lwc1(a0 + 100) - origin_y;
    let f11 = lwc1(a1) - origin_x;
    let f13 = lwc1(a1 + 4) - origin_y;
    let origin_z = lwc1(a0 + 64);
    let f10 = lwc1(a1 + 8) - origin_z;
    let texture = lw(a0 + 4);
    let f1 = lwc1(a0 + 24) - origin_z;
    let f3 = lwc1(a0 + 104) - origin_z;
    let textures = lw(g::leveltextures);
    sw(sp, sp + 80);
    sw(sp + 8, a3);
    sw(sp + 16, a4);
    sw(sp + 24, a2);
    let a2_1 = lw((texture << 4) + textures);
    let a1_1 = lw(a0 + 192);
    let a3_1 = lw(a0 + 8);
    swc1(sp + 32, f6);
    swc1(sp + 36, f7);
    swc1(sp + 40, f1);
    swc1(sp + 48, f8);
    swc1(sp + 52, f9);
    swc1(sp + 56, f3);
    swc1(sp + 64, f11);
    swc1(sp + 68, f13);
    swc1(sp + 72, f10);
    swc1(sp + 80, (f6 * f11 + f7 * f13 + f1 * f10) / (f6 * f6 + f7 * f7 + f1 * f1));
    swc1(sp + 84, (f8 * f11 + f9 * f13 + f3 * f10) / (f8 * f8 + f9 * f9 + f3 * f3));
    particleNewGlass(0, a1_1, a2_1, a3_1, a0 + 56, position, a0 + 96, a0 + 136, f12);
    swc1(a0, -6e+01_f32);
    if sound == 1_i64 {
        soundStart(202, position);
    } else if sound >= 2_i64 {
        if sound == 2_i64 {
            soundStart(203, position);
        } else if sound == 3_i64 {
            soundStart(204, position);
        }
    } else if sound == 0_i64 {
        soundStart(201, position);
    }
}

/// Make a door prop `a0` at the position, closed (animation 1 at 0) and with the door handler: the record gets the position, the door number `a1` (kept in the door table at
/// 0x1fd0da0 when not negative) and, by the door model, the kind (0 to 3) and the width at +36 (@ 0x269970).
pub fn doorNew(a0: i32, a1: i32, f12: f32, f13: f32, f14: f32) -> i32 {
    let prop = propNew(a0, f12, f13, f14, 0.0_f32);
    let record = lw(prop + 352);
    obInstSetAnim(lw(prop + 32), 1, 0.0_f32);
    sw(prop + 16, lw(prop + 16) | 4);
    propSetHandler(prop, 2, 0x269960, 0);
    swc1(record + 8, f12);
    swc1(record + 12, f13);
    swc1(record + 16, f14);
    swc1(record + 32, 0.0_f32);
    sw(record, a1);
    sw(record + 20, 0);
    sw(record + 24, 0);
    sw(record + 28, 0);
    if a1 >= 0 {
        sw((a1 << 2) + 0x1fd0da0, prop);
    }
    let model = lw(prop + 4) - 169;
    'b2: {
        'b3: {
            if (model as u32) < 114 {
                let width;
                let kind;
                match model {
                    0 | 4 => {
                        sw(record + 4, 0);
                        swc1(record + 36, 1.5_f32);
                        break 'b2;
                    }
                    1 => {
                        width = lwc1(0x3acbe0);
                        kind = 2;
                    }
                    2 => {
                        width = lwc1(0x3acbe4);
                        kind = 3;
                    }
                    3 => {
                        width = lwc1(0x3acbe8);
                        kind = 3;
                    }
                    5 | 6 | 10 | 11 | 12 | 13 | 14 => {
                        width = 2.75_f32;
                        kind = 3;
                    }
                    23 => {
                        width = 2.0_f32;
                        kind = 1;
                    }
                    24 => {
                        sw(record + 4, 0);
                        swc1(record + 36, 2.75_f32);
                        break 'b2;
                    }
                    111 => {
                        width = lwc1(0x3acbdc);
                        kind = 1;
                    }
                    113 => {
                        width = 3.0_f32;
                        kind = 3;
                    }
                    _ => break 'b3,
                }
                sw(record + 4, kind);
                swc1(record + 36, width);
                break 'b2;
            }
        }
        sw(record + 4, 0);
    }
    prop
}

/// The tick of the portals: the camera matrix of the current player is combined, the portal and back face data are set up from its room, each room's depth is cleared and
/// the rooms are visited by depth, each getting its clip and portal positions (@ 0x2586c0).
pub fn bgPortalTick() {
    let frame = Frame::new(160);
    let sp = frame.sp();
    let player = lw(g::currentplayer);
    let mut room = lw(player + 896);
    if room == 0 {
        room = lw(player + 900);
    }
    sceVu0MulMatrix(sp, lw(player + 1760), lw(player + 1764));
    bgPortalCalcInit();
    bgPortalCalcBackFace(room);
    let rooms = lw(g::bgNumRooms);
    let mut i = 0;
    if rooms > 0 {
        let table = lw(0x3b0084);
        loop {
            let at = (i << 2) + table;
            i += 1;
            sw(at + 4, 0);
            if i >= rooms {
                break;
            }
        }
    }
    bgPortalCalcRoomDepth(room, 0);
    let mut current = lw(g::currentplayer);
    let mut depth = 0;
    if lw((lw(current) << 2) + g::maxlevel) >= 0 {
        let mut rooms = lw(g::bgNumRooms);
        let mut r = 1;
        loop {
            let next_depth;
            let mut index;
            let table;
            if r >= rooms + 1 {
                next_depth = depth + 1;
                index = lw(current);
                table = g::maxlevel;
            } else {
                next_depth = depth + 1;
                index = lw(current);
                loop {
                    if lw(r * 20 + lw((index << 2) + g::roomdata)) != depth {
                        r += 1;
                    } else {
                        bgRoomCalcClip(r);
                        bgPortalCalcPos(r, sp);
                        current = lw(g::currentplayer);
                        rooms = lw(g::bgNumRooms);
                        r += 1;
                    }
                    index = lw(current);
                    if r >= rooms + 1 {
                        break;
                    }
                }
                table = g::maxlevel;
            }
            depth = next_depth;
            r = 1;
            if lw((index << 2) + table) < depth {
                break;
            }
        }
    }
}

/// Reset the credits' background: the tunnel textures' numbers are taken from the preload list, the tunnel restarts, the display list headers of the credits are written, and the
/// 54 rows of colours of the tunnel get random values from 155 to 254 (@ 0x2ae838).
pub fn creditsBgReset() {
    let mut numbers = g::tunneltexnum;
    let mut left = 2;
    let mut names = g::tunneltexturename;
    loop {
        let name = lw(names);
        left -= 1;
        names += 4;
        let texture = preloadGetTexName(name);
        sw(numbers, lw(texture + 20));
        numbers += 4;
        if left < 0 {
            break;
        }
    }
    creditsRestartTunnel();
    sw(g::credits_giftags, 0x8004);
    sw(g::credits_giftags + 4, 0x302e4000);
    let at = g::credits_giftags + 8;
    sw(at, 1042);
    sw(at + 4, 0);
    sw(g::credits_vif_load_mmtx, 0);
    sw(g::credits_vif_load_mmtx + 4, 0);
    let at = g::credits_vif_load_mmtx + 8;
    sw(at, 0x1000404);
    sw(at + 4, 0x6c048004);
    sw(g::credits_vif_entry_loadm, 0x140007a8);
    sw(g::credits_vif_entry_loadm + 4, 0);
    let at = g::credits_vif_entry_loadm + 8;
    sw(at, 0);
    sw(at + 4, 0x11000000);
    sw(g::credits_vif_load_giftags, 0);
    sw(g::credits_vif_load_giftags + 4, 0);
    let at = g::credits_vif_load_giftags + 8;
    sw(at, 0x1000404);
    sw(at + 4, lw(g::ucode_loadgif) | 0x6c018000);
    sw(g::credits_vif_load_xyzw, 0);
    sw(g::credits_vif_load_xyzw + 4, 0);
    let at = g::credits_vif_load_xyzw + 8;
    sw(at, 0x1000404);
    sw(at + 4, lw(g::ucode_loadxyzw) | 0x6c048000);
    sw(g::credits_vif_load_stnorm, 0);
    sw(g::credits_vif_load_stnorm + 4, 0);
    let at = g::credits_vif_load_stnorm + 8;
    sw(at, 0x1000404);
    sw(at + 4, lw(g::ucode_loadst) | 0x6c048000);
    sw(g::credits_vif_load_rgba, 0);
    sw(g::credits_vif_load_rgba + 4, 0);
    let at = g::credits_vif_load_rgba + 8;
    sw(at, 0x1000404);
    sw(at + 4, lw(g::ucode_loadrgba) | 0x6e048000);
    let mut row = 0;
    let mut offset = row * 48;
    loop {
        let next = row + 1;
        let mut left = 2;
        let mut colour = offset + (g::tunnel + 20);
        loop {
            left -= 1;
            let roll = newrnd();
            sw(colour - 8, ((roll as i32 as u32) % 100) as i32 + 155);
            let roll = newrnd();
            sw(colour - 4, ((roll as i32 as u32) % 100) as i32 + 155);
            let roll = newrnd();
            sw(colour, ((roll as i32 as u32) % 100) as i32 + 155);
            colour += 12;
            if left < 0 {
                break;
            }
        }
        row = next;
        if row >= 54 {
            break;
        }
        offset = row * 48;
    }
    sw(g::randomsinewavetime, 0);
    sw(g::randomsinewaveintensity, 0);
}

/// The tick of a pickup that is dropping (the record's mode 2): it moves by its velocity and spins until its time runs out, then it lands at its pad: a sound and (outside
/// the game kind 7) the pickup is made to wait; a bag drop clears the pad's flag and hands the drop to the character (@ 0x26ecf0).
pub fn droppingPickupPropTick(a0: i32) {
    let record = lw(a0 + 352);
    if lw(a0 + 188) == 0 {
        if lw(a0 + 196) == 0 {
            if record != 0 {
                let mode = lw(record + 68);
                if mode == 2 {
                    let ticks = lw(g::updateframesi);
                    if lw(lw(a0 + 32) + 4) != 0 {
                        let left = lw(record + 76) - ticks;
                        sw(record + 76, left);
                        if left > 0 {
                            let frames = lwc1(g::updateframesf);
                            let y = lwc1(a0 + 52) + lwc1(a0 + 64) * frames;
                            let spin_y = lwc1(a0 + 72);
                            let z = lwc1(a0 + 56) + lwc1(a0 + 68) * frames;
                            let spin_z = lwc1(a0 + 76);
                            swc1(a0 + 48, lwc1(a0 + 48) + lwc1(a0 + 60) * frames);
                            swc1(a0 + 52, y);
                            swc1(a0 + 56, z);
                            let spin_w = lwc1(a0 + 88);
                            swc1(a0 + 72, spin_y + lwc1(record + 80) * frames);
                            swc1(a0 + 76, spin_z + lwc1(record + 84) * frames);
                            swc1(a0 + 88, spin_w + lwc1(record + 88) * frames);
                        } else {
                            let mut kind;
                            let x;
                            'b2: {
                                kind = lw(record);
                                if kind != mode {
                                    if kind != 1 {
                                        x = lwc1(record + 52);
                                        break 'b2;
                                    }
                                    sw(a0 + 72, 0);
                                    sw(a0 + 88, 0);
                                } else {
                                    soundStart(196, a0 + 48);
                                    if lw(g::curgameparms + 72) != 7 {
                                        sw(a0 + 488, 1);
                                        sw(a0 + 492, 720);
                                    }
                                    sw(a0 + 72, 0);
                                    swc1(a0 + 88, 9e+01_f32);
                                    kind = lw(record);
                                }
                                x = lwc1(record + 52);
                            }
                            let pad = lw(record + 64);
                            swc1(a0 + 48, x);
                            sw(record + 68, 1);
                            let y = lwc1(record + 56);
                            sw(record + 36, pad);
                            swc1(a0 + 52, y);
                            swc1(a0 + 56, lwc1(record + 60));
                            if kind == 1 {
                                let holder = enemyGetPadProp(pad);
                                if holder != 0 {
                                    let current_pad = lw(record + 64);
                                    if current_pad >= 0 {
                                        let pad_record = current_pad * 28 + lw(g::levelpads);
                                        sw(pad_record + 4, lw(pad_record + 4) & -0x10001);
                                    }
                                    let a1 = lw(holder + 4);
                                    let offset = lw(record + 72) * 0x1210;
                                    let info = lw(g::chrinfo);
                                    sw(record + 64, -1);
                                    propDropBagOnDrop(info + offset, a1, a0);
                                    return;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// The tick of the groups of special effects: the time (+48) of each of the 56 used groups moves on by the frame time over 60, and the group's own tick runs by its kind
/// (0 to 11) (@ 0x2ad9f0).
pub fn specialfxGroupsTick() {
    let step = lwc1(g::updateframesf) / 6e+01_f32;
    let mut i = 0;
    loop {
        let groups = g::fxgroups;
        let offset = i * 92;
        let kind = lw(i * 92 + groups);
        if kind != -1 {
            let time = offset + (groups + 48);
            swc1(time, lwc1(time) + step);
            if (kind as u32) < 12 {
                match kind {
                    0 => specialfxOccasionalRocketSmokeTick(i),
                    1 => specialfxRingSpawnTick(i),
                    2 | 3 => specialfxPlanetSpawnTick(i),
                    4 => specialfxPlanetSpawnGlowTick(i),
                    5 => specialfxRotateSpawnTick(i),
                    6 => specialfxSphereSpawnTick(i),
                    7 => specialfxBlastSpawnTick(i),
                    8 => specialfxBlastShardTick(i),
                    9 => specialfxPickupSpawnTick(i),
                    10 => specialfxTimesplitterHandsTick(i),
                    11 => specialfxTimesplitterRadiusTick(i),
                    _ => unreachable!(),
                }
            }
        }
        i += 1;
        if i >= 56 {
            break;
        }
    }
}

/// Draw an object instance for the current player if it is visible to it (the bit 0x1000000 shifted by the player's number in its flags): the texture buffer of the object is
/// selected, the blend, dither and depth modes of the instance are set (and put back), and the parts are drawn once or, for a blended instance when `a1` is set, in two
/// passes (@ 0x262a60).
pub fn obInstGfx(a0: i32, a1: i32) {
    let flags = lw(lw(a0 + 244) + 16);
    let model = lw(a0);
    if (0x1000000 << (lw(lw(g::currentplayer)) & 31) & flags) != 0 {
        if (flags & 256) == 0 {
            let buffer = lw(model + 40);
            let mut blend = -1;
            let mut dither = -1;
            let mut depth = -1;
            if buffer != 0 {
                emSelectTexBuffer(buffer);
            }
            let dither_mode;
            if lw(a0 + 296) == depth {
                dither_mode = lw(a0 + 300);
            } else {
                dlSetFixedRGBA(lw(a0 + 308));
                blend = dlSetBlend(lw(a0 + 296));
                dither_mode = lw(a0 + 300);
            }
            let depth_mode;
            if dither_mode == depth {
                depth_mode = lw(a0 + 304);
            } else {
                dither = dlSetDitherMatrix(dither_mode);
                depth_mode = lw(a0 + 304);
            }
            let blend_mode;
            if depth_mode == depth {
                blend_mode = lw(a0 + 296);
            } else {
                depth = dlSetZB(depth_mode);
                blend_mode = lw(a0 + 296);
            }
            'b6: {
                if ((blend_mode + 1) as u32) >= 2 {
                    if a1 != 0 {
                        partGfx(a0, 0, 0);
                        partGfx(a0, 0, 1);
                        break 'b6;
                    }
                }
                partGfx(a0, 0, a1);
            }
            if blend != -1 {
                dlSetBlend(blend);
            }
            if dither != -1 {
                dlSetDitherMatrix(dither);
            }
            if depth != -1 {
                dlSetZB(depth);
            }
            if lw(lw(a0) + 40) != 0 {
                emSelectTexBuffer(0);
                return;
            }
        }
    }
}

/// The tick of the game by its state: state 1 (the intro) goes on to 5 on a button in the level 105; state 3 runs the level unless paused; state 5 (leaving) sets up a deferred
/// reset to the menu, the credits or the story menu by the game flags; state 6 clears the pads and front ends and waits for the music to stop before the level ends (@ 0x21eeb8).
pub fn gameTick() {
    let frame = Frame::new(160);
    let sp = frame.sp();
    let state = lw(g::gamestate) - 1;
    if (state as u32) < 6 {
        match state {
            0 => {
                if lw(g::lvlevelnum) == 105 {
                    if (joyNewBut(0) & 2544) != 0 {
                        sw(g::gamestate, 5);
                    }
                }
            }
            1 | 3 => {}
            2 => {
                if timeGetPause() == 0 {
                    gameLevelTick();
                }
            }
            4 => {
                memset(sp, 0, 112);
                let flags = lw(g::curgameparms + 80);
                let mut level = 102;
                if (flags & 0x2000) == 0 {
                    level = 103;
                    if (flags & 0x40000) == 0 {
                        level = 102;
                        if (flags & 0x1000) == 0 {
                            let available = levelAvail(105);
                            level = 105;
                            if available == 0 {
                                level = 102;
                            }
                        }
                    }
                }
                sw(sp + 64, level);
                sw(sp, 1);
                musicEnd();
                gameDeferedReset(sp);
                sw(g::gamestate, 6);
            }
            5 => {
                let mut player = 0;
                let mut waiting;
                loop {
                    joyClearAll(player);
                    if lw(g::lvlevelnum) == 102 {
                        player += 1;
                    } else {
                        frontSet(player);
                        let front = lw(g::front);
                        sw(front + 12, 0);
                        sw(front + 20, 0);
                        player += 1;
                    }
                    waiting = lw(0x3afefc);
                    if player >= 4 {
                        break;
                    }
                }
                if waiting > 0 {
                    let stopped = musicDefinitelyStopped();
                    let left = lw(0x3afefc);
                    if stopped != 0 {
                        let left = left - 1;
                        sw(0x3afefc, left);
                        if left == 0 {
                            sw(g::lvend, 666);
                        }
                    }
                }
            }
            _ => unreachable!(),
        }
    }
}
