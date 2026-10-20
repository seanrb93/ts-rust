//! Small functions: the damage of the gong and the pub sign, the in-game menu start and exit, the decal drawing, the music fade and the end of a level.

use crate::*;

/// The gong is hit: it swings, away or towards the side the hit came from, by the damage times 9, and the sound 14 plays (@ 0x279858).
pub fn gongDamage(a0: i32, _a1: i32, _a2: i32, _a3: i32, _a4: i32, a5: i32, f12: f32) {
    let frame = Frame::new(112);
    let sp = frame.sp();
    let source = 0x3a85a8;
    let tail = lw(source + 8);
    sd(sp, ld(source));
    sw(sp + 8, tail);
    memset(sp + 16, 0, 12);
    let record = lw(a0 + 352);
    let multiplier = getSwingDirectionMultiplier(a5, sp, sp + 16);
    swc1(record + 20, lwc1(record + 20) + multiplier * f12 * 9.0_f32);
    soundStart(14, a0 + 48);
}

/// The pub sign is hit: the same as the gong with the damage times 24 and the sound 34 (@ 0x2796f8).
pub fn pubsignDamage(a0: i32, _a1: i32, _a2: i32, _a3: i32, _a4: i32, a5: i32, f12: f32) {
    let frame = Frame::new(112);
    let sp = frame.sp();
    let source = 0x3a85a8;
    let tail = lw(source + 8);
    sd(sp, ld(source));
    sw(sp + 8, tail);
    memset(sp + 16, 0, 12);
    let record = lw(a0 + 352);
    let multiplier = getSwingDirectionMultiplier(a5, sp, sp + 16);
    swc1(record + 20, lwc1(record + 20) + multiplier * f12 * 24.0_f32);
    soundStart(34, a0 + 48);
}

/// Take the page out of the stack of pages of the front end: the acting sounds of the four players stop and the previous page comes back, or the stack is empty (@ 0x230148).
pub fn pageOut() {
    let mut player = 0;
    let mut depth;
    loop {
        actingsoundStop(player);
        player += 1;
        depth = lw(0x3aff2c);
        if player >= 4 {
            break;
        }
    }
    let previous = depth - 1;
    sw(0x3aff2c, previous);
    if previous < 0 {
        sw(0x3aff2c, -1);
    } else {
        frontPageSet(0, lw((previous << 3) + 0x1fc130c));
    }
    soundStart(142, 0);
}

/// Leave the in-game menu of the player `a0`: the root pages are set, and unless the menu is cleared outright (when the flag at +0x3afe78 is negative and the bit 4 of its
/// state is not set) the pause bit of the state is toggled with the sound 222 and the root menu is made when it came on; the pad is cleared (@ 0x212ae8).
pub fn ingamemenuExit(a0: i32) {
    'b1: {
        setRootPages(a0);
        if lw(0x3afe78) >= 0 {
        } else if (lhu(a0 * 92 + 0x1fb3248) & 4) == 0 {
            ingamemenuClear(a0);
            break 'b1;
        }
        soundStart(222, 0);
        let menu = a0 * 92 + 0x1fb3240;
        let state = lhu(menu + 8) ^ 2;
        sh(menu + 8, state);
        if (state & 2) != 0 {
            ingamemenuRoot(a0);
        }
    }
    joyClearAll(a0);
}

/// The tick of the map maker's map settings page: the buttons 2304 leave the menu; otherwise the button 64 on the page 3 clears a flag, the menu ticks and its items are made
/// (@ 0x24ce08).
pub fn mapsettings_pageTick(a0: i32) {
    if (joyNewBut(0) & 2304) == 0 {
        if (joyNewBut(0) & 64) != 0 {
            if lhu(a0 + 4) == 3 {
                sb(0x3b0038, 0);
            }
        }
        let menu = g::mapsettings_menu;
        menutick_24c3a8(a0, menu);
        let made = menumake(lw(a0 + 8), menu);
        sw(a0 + 12, made);
        return;
    }
    mmExitMenu(a0);
}

/// Show the message that a save is corrupt for the player `a0`: the text is made from the language's strings and the slot's long name, and the front end's flag 8 is set
/// (@ 0x2317d8).
pub fn corruptLoadMessage(a0: i32) {
    let languages = g::languages;
    let entry = lw((lw(g::cur_language) << 2) + languages);
    let slot = mcseqSlotGetLongName();
    let tail = lw(lw((lw(g::cur_language) << 2) + languages) + 552);
    let text = vafmt(&[0x3a1990_i64, lw(entry + 544) as i64, lw(entry + 548) as i64, slot as i64, tail as i64]);
    let message = GetPlayerSignOnMessage(a0, text);
    mcseqMessage(6, message, lw(lw((lw(g::cur_language) << 2) + languages) + 1616));
    let state = 0x1fc11f8;
    sw(state + 4, lw(state + 4) | 8);
}

/// Draw the decals of the moving prop room: with the depth buffer in mode 2, blending and the dither matrix 2, each decal of the room's list is drawn (@ 0x2a6250).
pub fn decalGfxMoving() {
    let zb = dlSetZB(2);
    dlSetBlend(1);
    dlSetDitherMatrix(2);
    let mut decal = lw((lw(g::s_MovingPropRoom) << 2) + 0x1fe9d48);
    if decal != 0 {
        loop {
            decalDraw(decal, lwc1(decal + 224));
            decal = lw(decal + 192);
            if decal == 0 {
                break;
            }
        }
    }
    dlSetDitherMatrix(1);
    dlSetZB(zb);
}

/// The tick of the music volume: it moves by 256 a tick towards the wanted volume and is set on the stream (@ 0x205d10).
pub fn musicTick() {
    let current = lw(g::curvol_133);
    if lw(g::tunenum) != 0 {
        let wanted = lw(g::destvol_132);
        if current < wanted {
            let mut value = current + 256;
            sw(g::curvol_133, value);
            if wanted < value {
                value = wanted;
                sw(g::curvol_133, value);
            }
            musicSetVol(value << 16 | value, 1);
            return;
        }
        let mut value = current - 256;
        if wanted < current {
            sw(g::curvol_133, value);
            if value < wanted {
                value = wanted;
                sw(g::curvol_133, value);
            }
            musicSetVol(value << 16 | value, 1);
            return;
        }
    }
}

/// The level is completed: the flag 0x10000 is told to the front end when the status changed, the state becomes 4, the time is paused, the sound plays and, for the game kinds
/// 1 to 7, the in-game menu of each player opens (@ 0x21d348).
pub fn gameLevelCompleted() {
    if gameStatusUpdated() != 0 {
        frontmpgSetFlags(0x10000);
    }
    sw(g::gamestate, 4);
    timePause(1);
    gameEndLevelSound();
    let kind = lw(g::curgameparms + 72);
    if kind < 8 {
        let players = lw(g::lvnumplayers);
        if kind > 0 {
            let mut i = 0;
            if players > 0 {
                loop {
                    ingamemenuStart(i);
                    i += 1;
                    if i >= lw(g::lvnumplayers) {
                        break;
                    }
                }
            }
        }
    }
}

/// Detonate the remote mines of the character `a0` (twenty slots from +0x11c0): each is exploded and cleared, the sound 195 plays if there was one, and when the character has
/// no ammo left (+664) the gun changes to a combo that has some (@ 0x294458).
pub fn gunDetonateRemoteMines(a0: i32) {
    let mut any = 0;
    let mut left = 19;
    let mut at = a0 + 0x11c0;
    loop {
        let mine = lw(at);
        left -= 1;
        if mine != 0 {
            bulletExplodeRemoteMine(mine);
            sw(at, 0);
            any = 1;
        }
        at += 4;
        if left < 0 {
            break;
        }
    }
    if any != 0 {
        soundStart(195, 0);
    }
    if lw(a0 + 664) > 0 {
        return;
    }
    changeToComboWithAmmo(a0);
}

/// Open the in-game menu of the player `a0`: the sound 222 plays if it was already open and paused, the state is reset, the root menu is made and the front end starts at the
/// main page (@ 0x212930).
pub fn ingamemenuStart(a0: i32) {
    let menu = a0 * 92 + 0x1fb3240;
    if (lhu(menu + 8) & 3) == 3 {
        soundStart(222, 0);
    }
    sh(menu + 10, 0);
    sh(menu + 8, 1);
    ingamemenuRoot(a0);
    frontSet(a0);
    let front = lw(g::front);
    sw(front + 12, 0);
    sw(front + 20, 0);
    frontPageSet(0, g::main_page_32ab10);
    let front = lw(g::front);
    let flags = lw(front + 12) & -2;
    sw(front + 40, 0);
    sw(front + 12, flags);
    setRootPages(a0);
}

/// Draw the cheats page: the background and the arrows (images 6 and 5) at the left of the list, in the flashing colour, when the list can scroll up or down (@ 0x2397b0).
pub fn cheats_pageDraw(a0: i32) {
    let list = g::cheats_list;
    pagebackgroundGfx_230688(a0);
    let flash = lbu(g::flashalpha);
    if (lhu(list + 2) as u32) < (lhu(g::cheats_list) as u32) {
        let x = lh(list + 4) - 32;
        let colour = (((flash as u32) >> 1) as i32) << 24 | 128 | (flash << 8 | flash << 16);
        if lbu(list + 12) != 0 {
            guiDrawImage(6, colour, x as f32, (lh(list + 6) + 2) as f32);
        }
        let shown = lhu(list + 2);
        if lbu(list + 12) < lhu(g::cheats_list) - shown {
            guiDrawImage(5, colour, x as f32, (lh(list + 6) + (shown - 1) * 11 + 2) as f32);
            return;
        }
    }
}

/// Draw the decals of the room `a0`: with the depth buffer in mode 2, blending and the dither matrix 2, each decal of the room's list is drawn (@ 0x2a61c8).
pub fn decalGfx(a0: i32) {
    let zb = dlSetZB(2);
    dlSetBlend(1);
    dlSetDitherMatrix(2);
    let mut decal = lw((a0 << 2) + 0x1fe9d48);
    if decal != 0 {
        loop {
            decalDraw(decal, lwc1(decal + 224));
            decal = lw(decal + 192);
            if decal == 0 {
                break;
            }
        }
    }
    dlSetDitherMatrix(1);
    dlSetZB(zb);
}

/// The ticks before the rest of the level's tick: the ordinary levels tick the whole list of systems; the levels 101 to 105 tick only the props (all but the level 104)
/// (@ 0x2262c8).
pub fn lvTickBefore() {
    let level = lw(g::lvlevelnum) - 101;
    if (level as u32) >= 5 {
        ipTick();
        starfieldTick();
        weatherTick();
        bgGetRoomsTick();
        aiTickBefore();
        propTickBefore();
        bulletTick();
        decalTick();
        cartridgesTick();
        specialfxTick();
        particleTick();
        playerAutoAimTick();
        return;
    }
    match level {
        0 | 1 | 2 | 4 => propTickBefore(),
        _ => {}
    }
}

/// Reset the players: the array of the players is allocated and each gets 1296 bytes of its own at +820, then the players restart (@ 0x27ff80).
pub fn playerReset() {
    let players = memAlloc(lw(g::lvnumplayers) * 1820 + 15 & -16, 1);
    let count = lw(g::lvnumplayers);
    sw(g::players, players);
    if count > 0 {
        let mut i = 0;
        let mut offset = 0;
        loop {
            i += 1;
            let player = lw(g::players) + offset;
            let own = memAlloc(1296, 1);
            offset += 1820;
            let more = i < lw(g::lvnumplayers);
            sw(player + 820, own);
            if !more {
                break;
            }
        }
    }
    playerRestart();
}
