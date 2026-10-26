//! Small functions: the room props, the level reset and preload, the death of a player, the clock props and a few sound and menu helpers.

use crate::*;

/// Make a prop for each room of the level (the props 513 up): the array of room props starts with an empty first slot, and each prop is numbered with its room and has no
/// links (@ 0x256548).
pub fn bgCreateRoomProps() {
    let mut room = 0;
    let rooms = lw(g::bgNumRooms);
    sw(lw(g::roomprops), 0);
    if rooms > 0 {
        loop {
            let prop = propNew(room + 513, 0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32);
            let next = room + 1;
            let count = lw(g::bgNumRooms);
            let at = (room << 2) + lw(g::roomprops);
            room = next;
            sw(at + 4, prop);
            sw(prop + 136, 0);
            sw(prop + 12, next);
            sw(prop + 140, 0);
            if room >= count {
                break;
            }
        }
    }
}

/// Reset the background: the room data and the level's tables are emptied and the level is loaded (the file named from the level number, or the tile set's level for the
/// level 100) (@ 0x256750).
pub fn bgReset() {
    let frame = Frame::new(80);
    let sp = frame.sp();
    let max = g::maxlevel;
    let level = lw(g::lvlevelnum);
    sw(max + 12, -1);
    let room = g::roomdata;
    sw(g::roomdata, 0);
    sw(g::maxlevel, -1);
    sw(g::bgGetRoomsFirstTick, 0);
    sw(room + 12, 0);
    sw(g::roombb, 0);
    sw(room + 4, 0);
    sw(room + 8, 0);
    sw(max + 4, -1);
    sw(max + 8, -1);
    sw(g::levelrooms, 0);
    sw(g::levelportals, 0);
    sw(g::levelvolumes, 0);
    sw(g::roomlightinitial, 0);
    if level != 100 {
        sprintf(&[sp as i64, 0x3a2640_i64, level as i64, level as i64, level as i64]);
        bgLoad(sp);
    } else {
        bgTileLoadLevel();
    }
    bgRestart();
}

/// Preload for the level: the fonts first; then for a game level every part of the game, for the levels 101 to 105 the part that level needs (the front end, the map maker, the
/// loading screen or the credits), and the level set-up's last (@ 0x225d98).
pub fn lvPreload() {
    textPreload();
    let level = lw(g::lvlevelnum) - 101;
    if (level as u32) >= 5 {
        particlePreload();
        decalPreload();
        weatherPreload();
        starfieldPreload();
        specialfxPreload();
        specialPreload();
        zbtestPreload();
        playerPreload();
        bgPreload();
        padPreload();
        hudPreload();
        gunPreload();
        enemyPreload();
        ingamemenuPreload();
        pickupposPreload();
    } else {
        match level {
            0 => {}
            1 => frontmpgPreload(),
            2 => mapmakerPreload(),
            3 => loadingPreload(),
            4 => creditsPreload(),
            _ => unreachable!(),
        }
    }
    lvLevelSetupPreload();
}

/// The player `a0` dies: its special effect is freed, the kill is counted for the killer (`a1`, when it is a character) and unless the player is already in the mode 2 the
/// death is made with the remaining arguments (@ 0x280488).
pub fn playerDie(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32, a6: i32) {
    let mut chr = lw(a0 + 384);
    let record = lw(chr + 352);
    let effect = lw(record + 0x11a8);
    if effect != 0 {
        specialfxFreePtr(effect);
        sw(record + 0x11a8, 0);
        chr = lw(a0 + 384);
    }
    let mut killer = 0;
    if a1 != 0 {
        if lw(a1 + 8) == 8 {
            killer = lw(a1 + 352);
        }
    }
    StatsKillMade(killer, lw(chr + 352));
    if lw(a0 + 296) == 2 {
        return;
    }
    playerOnGroundDie(a0, a2, a3, a4, a5, a6);
}

/// The player `a0` dies on the ground: the mode becomes 4, the flag 32 of the prop is dropped, the death animation is chosen, the message is made, the guns are cleared, the
/// camera goes to the mode 7 and (in the game kind 7) the in-game menu opens (@ 0x280588).
pub fn playerOnGroundDie(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32) {
    sw(a0 + 296, 4);
    obInstResetOverride(lw(lw(a0 + 384) + 32));
    sw(a0 + 20, lw(a0 + 20) & -33);
    chrSelectDeathAnim(lw(a0 + 384), a1, a2, a3, a4, a5);
    sw(lw(lw(a0 + 384) + 352) + 2708, 32);
    playerDeathMessage(a0);
    chrClearGuns(lw(lw(a0 + 384) + 352));
    playerClearGuns(a0);
    camSetMode(7);
    if lw(g::curgameparms + 72) != 7 {
        return;
    }
    ingamemenuStart(lw(a0));
}

/// Draw the sign-on page's last part: the messages of the players, or the zoom (@ 0x23f380).
pub fn signon_pageDrawAfter() {
    SetWindow(0, 1, 1);
    let first = lbu(0x3aff48);
    if lw(0x3aff98) < -1 {
        let second;
        if first == 0 {
            second = lbu(0x3aff94);
        } else {
            let player = first - 1;
            second = lbu(0x3aff94);
            if lw(player * 260 + 0x1fc1488) != 0 {
                splitMessageDraw(player);
                return;
            }
        }
        if second != 0 {
            let player = second - 1;
            if lw(player * 260 + 0x1fc1488) != 0 {
                splitMessageDraw(player);
                return;
            }
        }
        return;
    }
    soZoomGfx();
}

/// The tick of a clock prop: while the game runs the time is worked out; the hands (matrices at +64, +128 and +192 of the model's matrices) are turned about X by the angles,
/// and a sound plays when the minutes and seconds are both 0 (the hour) once (@ 0x279328).
pub fn clockPropTick(a0: i32) {
    if lw(g::updateframesi) != 0 {
        calculateClockTime();
    }
    sceVu0UnitMatrix(lw(lw(a0 + 32) + 4));
    matrixRotX(lw(lw(a0 + 32) + 4) + 64, lwc1(0x3b00f8));
    matrixRotX(lw(lw(a0 + 32) + 4) + 128, lwc1(0x3b00fc));
    matrixRotX(lw(lw(a0 + 32) + 4) + 192, lwc1(0x3b0100));
    let minutes = lw(0x3b0108);
    if lw(g::updateframesi) != 0 {
        if minutes != 0 {
            sw(0x3b0110, 0);
        } else if lw(0x3b010c) != 0 {
            sw(0x3b0110, 0);
        } else if lw(0x3b0110) == 0 {
            sw(0x3b0110, 1);
            soundStart(28, a0 + 48);
            return;
        }
    }
}

/// Load the texture numbered `a0` (described by the record `a1`) unless it is loaded already: a slot is taken from the end of the current texture buffer, its name is made
/// from the number and the kind, and the slot is recorded at `a2` when that is not null (@ 0x2b7900).
pub fn emLoadTextureNum(a0: i32, a1: i32, a2: i32) {
    let frame = Frame::new(1072);
    let sp = frame.sp();
    let slot = (a0 << 1) + 0x1feb3e0;
    let current = lh(slot);
    if current < 0 {
        let buffer = lw(g::thistexbuf);
        let kind = lbu(a1 + 12);
        sb(buffer + lw(buffer + 0x1a08) + 0x1808, kind);
        let buffer = lw(g::thistexbuf);
        let top = lw(buffer + 0x1a08);
        sh(slot, top);
        sw(buffer + 0x1a08, top - 1);
        sprintf(&[sp as i64, 0x3a9b20_i64, a0 as i64, kind as i64]);
        emLoadTexture(lh(slot), sp, a1);
        if a2 != 0 {
            sw(a2, lh(slot));
        }
    } else if a2 != 0 {
        sw(a2, current);
    }
}

/// The sounds of a gun being fired: the gun `a0` (the second kind of the pair when `a1`) plays its sound (the cheat version when the cheat 32 is on) at the position `a2`, and
/// its delayed sound 40 ticks later, while the game is running (@ 0x28de68).
pub fn gunFireSfx(a0: i32, a1: i32, a2: i32) {
    let base = if a1 != 0 { g::guns + 188 } else { g::guns + 52 };
    let gun = lw(a0) * 400 + base;
    let sound = if (lw(g::allowedcheats) & 32 & (lw(g::unlockedcheats) & lw(g::selectedcheats))) == 0 { lw(gun + 44) } else { lw(gun + 48) };
    let delayed = lw(gun + 52);
    if frames_gate() > 0.0_f32 {
        if sound >= 0 {
            soundStart(sound, a2);
        }
        if delayed >= 0 {
            soundDelayStartEx(delayed, a2, 40, 0);
            return;
        }
    }
}

/// Make or move the stage prop `a0` of the character select to the front of the current player's view (@ 0x238028).
pub fn fxCreateStage(a0: i32) {
    let at = (a0 << 2) + g::stage;
    let mut prop = lw(at);
    let mut player = lw(g::currentplayer);
    if prop == 0 {
        prop = propNew(497, 0.0_f32, 0.0_f32, 0.0_f32, 1.8e+02_f32);
        sw(at, prop);
        player = lw(g::currentplayer);
    }
    let distance = lwc1(0x3ac9cc);
    let x = lwc1(player + 152);
    let dx = ps2float::mul(lwc1(player + 176), distance);
    let flags = lw(prop + 16);
    sw(prop + 576, a0);
    swc1(prop + 48, ps2float::add(x, dx));
    let y = lwc1(player + 156);
    let dy = ps2float::mul(lwc1(player + 180), distance);
    sw(prop + 16, flags | 0x1000000);
    swc1(prop + 52, ps2float::sub(ps2float::add(y, dy), 1.0_f32));
    swc1(prop + 56, ps2float::add(lwc1(player + 160), ps2float::mul(lwc1(player + 184), distance)));
}

/// The tick of a clock prop of the mall: the hands (matrices at +64, +128 and +192) are turned about Z, and the sound 30 plays when the seconds change (@ 0x2793e8).
pub fn mallclockPropTick(a0: i32) {
    if lw(g::updateframesi) != 0 {
        calculateClockTime();
    }
    sceVu0UnitMatrix(lw(lw(a0 + 32) + 4));
    matrixRotZ(lw(lw(a0 + 32) + 4) + 64, lwc1(0x3b00f8));
    matrixRotZ(lw(lw(a0 + 32) + 4) + 128, lwc1(0x3b00fc));
    matrixRotZ(lw(lw(a0 + 32) + 4) + 192, lwc1(0x3b0100));
    let last = lw(0x3b0110);
    if lw(g::updateframesi) != 0 {
        let seconds = lw(0x3b010c);
        if last != seconds {
            sw(0x3b0110, seconds);
            soundStartRange(30, a0 + 48, 0, 8.0_f32);
            return;
        }
    }
}

/// Start a looping sound `a0` at the position `a1` (or everywhere, when 0) with the range `f12`: one of the six loop slots is taken and its channel (5 up) is returned, or 0
/// when all are in use (@ 0x2058f0).
pub fn soundStartLoopRange(a0: i32, a1: i32, f12: f32) -> i32 {
    let slot = lw(0x3afe28);
    let entry = slot * 24 + 0x1fb17e0;
    if slot < 6 {
        if a1 == 0 {
            sw(entry + 4, 0);
        } else {
            let x = lwc1(a1);
            sw(entry + 4, 1);
            swc1(entry + 8, x);
            swc1(entry + 12, lwc1(a1 + 4));
            swc1(entry + 16, lwc1(a1 + 8));
        }
        let channel = slot + 5;
        swc1(entry + 20, f12);
        sw(entry, a0 * 36 + g::sounds);
        soundSetChannel(0, channel);
        soundStart(a0, a1);
        soundSetChannel(-1, -1);
        sw(0x3afe28, lw(0x3afe28) + 1);
        channel
    } else {
        0
    }
}

/// Pause the game from the in-game menu of the player `a0`: the sound 222 plays, the player is recorded, the menu of each player that has none opens (the others are paused),
/// the front end is set and the time is paused (@ 0x2129e0).
pub fn ingamemenuPause(a0: i32) {
    let mut player = 0;
    soundStart(222, 0);
    let mut players = lw(g::lvnumplayers);
    sw(0x3afe78, a0);
    if players > 0 {
        loop {
            let menu = player * 92 + 0x1fb3240;
            if (lhu(menu + 8) & 1) != 0 {
                player += 1;
            } else {
                ingamemenuStart(player);
                players = lw(g::lvnumplayers);
                if player != a0 {
                    sh(menu + 8, lhu(menu + 8) | 2);
                }
                player += 1;
            }
            if player >= players {
                break;
            }
        }
    }
    frontSet(a0);
    timePause(1);
}

/// The tick of a message page: the page's own tick (or the default one) works out the answer, which is kept at +1036; when the page is done the sound 140 plays, the front
/// end's page count drops and the pad is cleared, otherwise the zoom box ticks (@ 0x2313b0).
pub fn message_pageTick_2313b0() {
    let message = lw(lw(g::front)) * 1040 + 0x1fc19c0;
    let tick = lw(message + 1024);
    if tick == 0 {
        let answer = defaultmessagetick_231388(lw(message + 1032));
        sw(message + 1036, answer);
    } else {
        let ret = call_indirect(tick, [lw(message + 1032), 1040, 0, 0, 0, 0, 0, 0], [0.0; 8]);
        sw(message + 1036, ret.0);
    }
    let state = messageGetState_2312c0(lw(lw(g::front)));
    let front = lw(g::front);
    if state == 0 {
        zoomboxTick(lw(front));
        return;
    }
    soundStart(140, 0);
    let front = lw(g::front);
    sw(front + 20, lw(front + 20) - 1);
    joyClearAll(lw(message + 1032));
}

/// Make a prop for each room of a tile set level (the props 513 up): the room's flags come from the table at 0x1fc4828 (@ 0x2565d0).
pub fn bgTileSetCreateRoomProps() {
    let mut room = 0;
    let rooms = lw(g::bgNumRooms);
    sw(lw(g::roomprops), 0);
    if rooms > 0 {
        loop {
            let prop = propNew(room + 513, 0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32);
            let flags = lhu((room << 1) + 0x1fc4828);
            let count = lw(g::bgNumRooms);
            let next = room + 1;
            let model = lw(prop + 32);
            let at = (room << 2) + lw(g::roomprops);
            room = next;
            sw(at + 4, prop);
            sh(model + 292, flags);
            sw(prop + 12, next);
            sw(prop + 140, 0);
            sw(prop + 136, 0);
            if room >= count {
                break;
            }
        }
    }
}

/// Make a pickup on the pad given by an external reference: the position and the angle come from the pad and the pickup remembers the pad (@ 0x272960).
pub fn propNewPickupOnPadExtref(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32, a6: i32, a7: i32, f12: f32, f13: f32, f14: f32, f15: f32, f16: f32) -> i32 {
    let pad = padNumFromExtref(a4);
    let at = pad * 28 + lw(g::levelpads);
    let f15_1 = lwc1(at + 20);
    let f13_1 = lwc1(at + 12);
    let f14_1 = lwc1(at + 16);
    let prop = propNewPickup(a0, a1, a2, a3, a5, a6, a7, f12, f13_1, f14_1, f15_1, f13, f14, f15, f16);
    let record = lw(prop + 352);
    sw(record + 40, pad);
    sw(record + 36, pad);
    prop
}

/// Update the directory of the maps: its count rises, its checksum is made and it is saved to the card with the text for a new (0) or a replaced map; the map maker's state
/// is set (@ 0x24d2d8).
pub fn updateDir(a0: i32) {
    let dir = lw(g::mapdir1);
    sw(dir + 4, lw(dir + 4) + 1);
    let size = mmfileDirSize();
    let checksum = crcCalc(dir + 4, size - 4);
    sw(lw(g::mapdir1), checksum);
    let title = if a0 == 0 {
        lw(lw((lw(g::cur_language) << 2) + g::languages) + 1892)
    } else {
        lw(lw((lw(g::cur_language) << 2) + g::languages) + 1896)
    };
    let size = mmfileDirSize();
    mcseqSave(3, title, 0, 0x3a2100, lw(g::mapdir1), size, 0);
    let flags = lw(g::editflags) & -0x20001;
    sb(0x3b0038, 14);
    sw(g::editflags, flags);
    if a0 != 0 {
        sw(g::editflags, flags | 0x20000);
    }
}
