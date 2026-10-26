//! Functions: drawing the props of a room, the map maker's zoom and used object, menu ticks, glass hits of bullets, a thrown grenade, the level reset and the head prop.

use crate::*;

/// The product of two 16.16 fixed point numbers as the game's code works it out from the 64 bit product (the high and low halves are put together and taken back by 16).
fn fixed_multiply(a: i32, b: i32) -> i32 {
    let high = (((a as i64) * (b as i64)) >> 32) as i32 as i64;
    let low = (a.wrapping_mul(b) as u32) as i64;
    ((((high << 32) | low) << 16) >> 32) as i32
}

/// Draw the props of the room `a0` that are visible and on: the clip is set for the prop 310 (a window prop), and each is drawn with `a1` (@ 0x26fe08).
pub fn propGfxRoom(a0: i32, a1: i32) {
    let count_at = a0 << 2;
    let mut i = 0;
    if lw(count_at + ROOM_PROP_COUNTS) > 0 {
        loop {
            let prop = lw((i << 2) + a0 * 400 + ROOM_PROP_LISTS);
            if (ld(prop + 16) & 0x80000001_i64) == 1_i64 {
                if lw(prop + 4) == 310 {
                    let window = g::window;
                    dlSetClip(lw(window), lw(window + 20), lw(window + 4), lw(window + 24));
                }
                let mirrored = punch_label_begin(prop);
                obInstGfx(lw(prop + 32), a1);
                punch_label_end(mirrored);
            }
            i += 1;
            if i >= lw(count_at + ROOM_PROP_COUNTS) {
                break;
            }
        }
    }
}

/// The tick of the map maker's zoomed tile page: the front page flag 2 is set on the top page, the page margins follow the tile window when nothing is busy, the mini tile
/// rotates; the button 16 leaves; otherwise the window and tile tick and the highlighted pad and the pad page are handled (@ 0x253220).
pub fn zoomtile_pageTick(a0: i32) {
    let front = lw(g::front);
    let top = ((((a0 - lw(front + 4)) >> 4 ^ lw(front + 20) - 1) as u32) < 1) as i32;
    if top != 0 {
        sw(front + 12, lw(front + 12) | 2);
    }
    let pressed = joyNewBut(0);
    let busy = lw(0x3b0074);
    if (pressed & 16) == 0 {
        if busy == 0 {
            let height = lw(0x3b0070);
            if top != 0 {
                let width = lw(0x3b006c);
                let margins = g::pagemargins;
                let x = lw(0x3b0068);
                let y = lw(0x3b0064);
                let half_height = (height + ((height as u32) >> 31) as i32) >> 1;
                let half_width = (width + ((width as u32) >> 31) as i32) >> 1;
                sw(g::pagemargins, (x - half_height) << 16);
                sw(margins + 4, (x + half_height) << 16);
                sw(margins + 12, (y + half_width) << 16);
                sw(margins + 8, (y - half_width) << 16);
                rotateMiniTile();
            }
        }
        zoomtileWindowTick();
        zoomtileTick();
        if lw(0x3b0074) == 0 {
            if top != 0 {
                findHighlightPad();
                padact_pageTick(a0);
                return;
            }
        }
        return;
    }
    sh(lw(a0) + 8, lhu(g::padactcuritem));
    exitDialogs();
}

/// A bullet hits glass: the strength is the bullet's own (+80) or 1 for the kinds 5 to 7 and 10 to 19; with the reverse flag the bullet's counter at +152 is cleared; a bullet
/// of a player (+104 is 1) with a shooter counts a statistic; the hit is handed to the background (@ 0x28d218).
pub fn bulletProcessGlasshits(a0: i32, a1: i32) {
    let kind = lw(a0);
    let mut strength = lwc1(a0 + 80);
    if kind >= 5 {
        if kind >= 8 {
            if kind < 20 && kind >= 10 {
                strength = 1.0_f32;
            }
        } else {
            strength = 1.0_f32;
        }
    }
    if lw(g::glasshitreverse) != 0 {
        sw(a0 + 152, 0);
    }
    if lw(a0 + 108) >= 0 {
        if lw(a0 + 104) == 1 {
            let shooter = lw(a0 + 220);
            if shooter != 0 {
                StatsAdd(32, lw(lw(shooter + 352)), 1.0_f32);
            }
        }
    }
    bgProcessGlasshits(lw(a0), a1, a0 + 84, strength);
}

/// The tick of the audio and video page: the page is set, the items for the music and the sound are dimmed when those are off, then the menu ticks and its items are made
/// (@ 0x2364c8).
pub fn audiovideo_pageTick(a0: i32) {
    let language = (lw(g::cur_language) << 2) + g::languages;
    if !av_ingame_x() {
        sw(((lw(0x3aff2c) - 1) << 3) + 0x1fc1308, 0);
    }
    let title = av_title_or(a0, lw(lw(language) + 480));
    if !av_ingame_x() {
        pageSet(title, g::audiovideo_page);
    }
    if !av_ingame_x() {
        frontfxTick(0);
    }
    let items = g::i04b00;
    let music = if lw(g::gv_playmusic) == 0 { lhu(items + 42) | 0x4040 } else { lhu(items + 42) & 0xbfbf };
    sh(items + 42, music);
    let items = g::i04b00;
    let sound = if lw(g::gv_playsound) == 0 { lhu(items + 106) | 0x4040 } else { lhu(items + 106) & 0xbfbf };
    sh(items + 106, sound);
    let menu = av_menu_or(a0, g::audiovideo_menu);
    if av_ingame_x() {
        ingame_menutick(a0, menu);
    } else {
        menutick_230238(a0, menu);
    }
    let made = std_menumake(lw(a0 + 8), menu);
    sw(a0 + 12, made);
}

/// The tick of the current menu: the pad moves the focus (0x1000 up, 0x4000 down), the button 64 runs the focused item (a function, a sub menu or an end), and the button 2048
/// ends the menu (@ 0x20ce18).
pub fn menuTick() {
    if lw(g::curmenu) != 0 {
        if (joyNewBut(lw(lw(g::currentplayer))) & 0x1000) != 0 {
            focusprev();
        }
        if (joyNewBut(lw(lw(g::currentplayer))) & 0x4000) != 0 {
            focusnext();
        }
        'b4: {
            let pressed = joyNewBut(lw(lw(g::currentplayer)));
            let menu = lw(g::curmenu);
            if (pressed & 64) != 0 {
                let item = lw(menu + 8);
                let flags = lbu(item + 20);
                if (flags & 4) != 0 {
                    call_indirect(lw(item + 24), [item, 0, 0, 0, 0, 0, 0, 0], [0.0; 8]);
                    break 'b4;
                }
                if (flags & 64) != 0 {
                    menuStart(lw(item + 24));
                    break 'b4;
                }
                if (flags & 128) == 0 {
                    break 'b4;
                }
                menuEnd();
            }
        }
        if (joyNewBut(lw(lw(g::currentplayer))) & 2048) != 0 {
            menuEnd();
            return;
        }
    }
}

/// Write the file of the prop keys: the line for each of the 713 props (its name and number) is made into a 1,000,000 byte buffer, and the file is saved, after the old one is
/// copied aside if there was one (@ 0x268f70).
pub fn WritePropKeyFile() {
    let frame = Frame::new(1120);
    let sp = frame.sp();
    let name = 0x3a7ae0;
    let buffer = malloc(0xf4240);
    let mut at = buffer;
    let mut i = 0;
    let mut entry = PROPTABLE + 8;
    let mut key = lw(entry);
    loop {
        sprintf(&[at as i64, 0x3af910_i64, key as i64, i as i64]);
        i += 1;
        entry += 28;
        at += strlen(at);
        if i >= 713 {
            break;
        }
        key = lw(entry);
    }
    sprintf(&[sp as i64, 0x3af918_i64, name as i64]);
    let size = fileSize(name);
    if size <= 0 {
        fileSave(name, buffer, at - buffer);
    } else {
        fileCopy(name, sp);
        fileSave(name, buffer, at - buffer);
    }
    free(buffer);
}

/// Draw the object the map maker is holding at the cursor (mode 2, or 3 when the edit flags say so): each tile object of the chain is copied and drawn at the cursor's cell
/// (@ 0x24ac00).
pub fn mmDrawUsedObject() {
    let frame = Frame::new(176);
    let sp = frame.sp();
    let used = lw(g::usedobj);
    let mode = if (lw(g::editflags) & 0x400004 ^ 4) == 0 { 2 } else { 3 };
    if used != 0 {
        let front = lw(g::front);
        let mut object = used;
        let scale_x = lw(g::scale_wtomx);
        let x_offset = (lh(front + 34) - lw(g::w_x)) << 16;
        let scale_y = lw(g::scale_wtomy);
        let y_offset = (lh(front + 38) - lw(g::w_y)) << 16;
        let origin_x = lw(g::m_x) + fixed_multiply(x_offset, scale_x) + 0x100000;
        let origin_y = lw(g::m_y) + fixed_multiply(y_offset, scale_y) + 0x100000;
        loop {
            let b = ld(object + 8);
            let c = ld(object + 16);
            let d = ld(object + 24);
            sd(sp, ld(object));
            sd(sp + 8, b);
            sd(sp + 16, c);
            sd(sp + 24, d);
            let e = ld(object + 40);
            let f = ld(object + 48);
            let last = lw(object + 56);
            sd(sp + 32, ld(object + 32));
            sd(sp + 40, e);
            sd(sp + 48, f);
            sw(sp + 56, last);
            let kind = lw(object);
            let level = lw(g::editlevel);
            let x = lw(object + 12) + origin_x & -0x200000;
            sw(sp + 12, x);
            let y = lw(object + 16) + origin_y & -0x200000;
            sw(sp + 16, y);
            sw(sp + 20, level + lw(object + 20));
            if kind == 1 {
                let tile = lw(object + (lw(object + 24) << 2) + 36);
                mmDrawTile(x, y, tile, mode, sp);
            }
            object = lw(object + 56);
            if object == 0 {
                break;
            }
        }
    }
}

/// A non-player character throws a grenade: the throw is made with `gunFire` from the hand bone's position, and the throw's speed `a1` is scaled by the distance to the
/// target `a2` (@ 0x290500).
pub fn npcThrowGrenade(a0: i32, a1: i32, a2: i32) {
    let frame = Frame::new(336);
    let sp = frame.sp();
    let flags = lw(a0 + 16);
    let record = lw(a0 + 352);
    sw(sp, 13);
    if (flags & 128) == 0 {
        if (flags & 0xf000000) != 0 {
            let angle = ps2float::mul(lwc1(a0 + 76), lwc1(0x3ad328)).pdiv(1.8e+02_f32);
            matrixTransRotY(sp + 176, lwc1(a0 + 48), lwc1(a0 + 52), lwc1(a0 + 56), angle);
            let model = lw(a0 + 32);
            sceVu0MulMatrix(sp + 112, sp + 176, lw(model + 4) + (lw(lw(model + 92) + 24) << 6));
            let hand_x = lwc1(sp + 160);
            let hand_y = lwc1(sp + 164);
            let dx = ps2float::sub(lwc1(a2 + 48), hand_x);
            let z = lwc1(a2 + 56);
            let dy = ps2float::sub(lwc1(a2 + 52), hand_y);
            let hand_z = lwc1(sp + 168);
            swc1(sp + 72, hand_x);
            let dz = ps2float::sub(z, hand_z);
            swc1(sp + 76, hand_y);
            swc1(sp + 80, hand_z);
            let factor = ps2float::add((ps2float::sub(ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz)).abs().psqrt(), 6.5_f32)).pdiv(62.0_f32), lwc1(0x3ad32c));
            let vx = ps2float::mul(lwc1(a1), factor);
            let vy = ps2float::mul(lwc1(a1 + 4), factor);
            swc1(a1 + 8, ps2float::mul(lwc1(a1 + 8), factor));
            swc1(a1, vx);
            swc1(a1 + 4, vy);
            gunFire(a0, sp, 0, 0, a1, 2, lw(record), 1);
        }
    }
}

/// Reset everything after the level's own reset, in the order the game does: a game level resets every part; the levels 101 to 105 reset the memory area, the players and
/// the part of their own (@ 0x225f68).
pub fn lvResetAfter() {
    textReset();
    soundResetAfter();
    let level = lw(g::lvlevelnum) - 101;
    if (level as u32) >= 5 {
        StatsReset();
        pickupposReset();
        particleReset();
        weatherReset();
        zbtestResetAfter();
        starfieldReset();
        specialfxReset();
        specialReset();
        memdbReset(2);
        ingamemenuReset();
        bgReset();
        roomlightReset();
        padReset();
        hudReset();
        gunReset();
        bulletReset();
        cartridgesReset();
        playerReset();
        musicReset();
        menuEndMessage();
        decalReset();
        lvSoundsReset();
        return;
    }
    match level {
        0 => {
            memdbReset(1);
            playerReset();
            frontredReset();
        }
        1 => {
            memdbReset(1);
            playerReset();
            frontmpgReset();
            musicReset();
        }
        2 => {
            memdbReset(2);
            playerReset();
            mapmakerReset();
            musicReset();
        }
        3 => {
            memdbReset(1);
            playerReset();
            musicReset();
            loadingReset();
        }
        4 => {
            memdbReset(2);
            playerReset();
            creditsReset();
            musicReset();
        }
        _ => unreachable!(),
    }
}

/// Test what the first player's aim hits and draw it: the line from the player's eye to the aim point is tested against the level (the player's own prop is out of the test),
/// and a line is drawn to the hit or to the end in a colour that says whether it hit what the player expected (@ 0x284e90).
pub fn debugtesthits() {
    let frame = Frame::new(144);
    let sp = frame.sp();
    let player = lw(g::players);
    let x = lwc1(player + 140);
    let chr = lw(player + 384);
    swc1(sp + 16, x);
    let y = ps2float::add(lwc1(player + 144), 1.5_f32);
    sw(chr + 16, lw(chr + 16) & -33);
    swc1(sp + 20, y);
    let z = lwc1(player + 148);
    swc1(sp + 24, z);
    let dx = ps2float::sub(lwc1(player + 1960), lwc1(player + 140));
    swc1(sp + 48, dx);
    let dy = ps2float::sub(lwc1(player + 1964), lwc1(player + 144));
    swc1(sp + 52, dy);
    let dz = ps2float::sub(lwc1(player + 1968), lwc1(player + 148));
    sw(sp, 1);
    sw(sp + 8, 0);
    swc1(sp + 32, ps2float::add(x, dx));
    let room = lw(player + 796);
    swc1(sp + 36, ps2float::add(y, dy));
    swc1(sp + 56, dz);
    swc1(sp + 40, ps2float::add(z, dz));
    let hit = bgLineTest(room, sp + 16, sp + 48, 0, sp + 64, sp + 80, sp + 96, 48);
    let player = lw(g::players);
    let chr = lw(player + 384);
    let expected = lw(player + 2204);
    sw(chr + 16, lw(chr + 16) | 32);
    if hit != expected {
        if hit != 0 {
            drawLine(sp + 16, sp + 32, 0x64000060);
        }
    } else {
        drawLine(sp + 16, sp + 64, 0x640060);
    }
}

/// The tick of the character select page: the page is set, the cheats allowed follow the arcade and character cheats, the front end ticks; the first time the page is shown the
/// effects reset and each player's character is moved on to one that is playable and started (@ 0x237ee8).
pub fn characterselect_pageTick(a0: i32) {
    pageSet(0, g::characterselect_page);
    let arcade = cheatsArcade();
    let select = cheatsChrSelect();
    sw(g::allowedcheats, arcade & select);
    frontfxTick(2);
    if lbu(lw(lw(g::front)) + 0x3aff28) == 1 {
        fxSelectCharReset();
        splitpageInit(g::chrplayer_page);
        setStates(2);
        if lw(0x3aff20) > 0 {
            let mut i = 0;
            loop {
                let slot = signonGet(i);
                let entry = lw(g::signonarray) + slot * 2936 + 32;
                let next = i + 1;
                loop {
                    if chrPlayable(lw(entry + 84)) != 0 {
                        break;
                    }
                    let character = lw(entry + 84) + 1;
                    sw(entry + 84, character);
                    if character >= 80 {
                        sw(entry + 84, 0);
                    }
                }
                fxCreateStage(i);
                let character = lw(entry + 84);
                let player = i;
                i = next;
                fxSelectCharInit(player, character);
                if i >= lw(0x3aff20) {
                    break;
                }
            }
        }
    }
    splitpageTick(a0, g::chrplayer_page);
}

/// Make a head prop for the character `a0` that was shot in the head: the head model (+3024 of its record) is made as a prop with the velocity `a1` over the frame time, the
/// head is hidden on the body once, a sound plays and the head prop is returned (0 if there is no head or it is gone already) (@ 0x26b538).
pub fn headPropNew(a0: i32, a1: i32) -> i32 {
    if (lw(a0 + 16) & 0x20000000) == 0 {
        let model = lw(lw(a0 + 352) + 3024);
        if model != -1 {
            let prop = propNew(model, lwc1(a0 + 48), lwc1(a0 + 52), lwc1(a0 + 56), 0.0_f32);
            let head = lw(prop + 32);
            let bones = lw(head);
            let hidden = 0x1fd0d50;
            let entry = lw(prop + 4) * 28 + PROPTABLE;
            let bone_table = bones - lw(bones) * 80;
            if lb(lh(lw(a0 + 352) + 4) + hidden) == 0 {
                transformHead(head, bone_table, lw(lw(lw(a0 + 32) + 92) + 36));
                let record = lw(a0 + 352);
                sw(entry + 16, 2);
                sb(lh(record + 4) + hidden, 1);
            }
            let record = lw(prop + 352);
            let frames = lwc1(g::updateframesf);
            swc1(record + 68, 4.0_f32);
            let vx = lwc1(a1);
            let flags = lw(a0 + 16) | 0x20000000;
            sw(record + 96, a0);
            sw(a0 + 16, flags);
            sw(record + 64, 0);
            let prop_flags = lw(prop + 16) & -2;
            swc1(record + 100, (vx).pdiv(frames));
            sw(prop + 16, prop_flags | 0x40000000);
            swc1(record + 104, (lwc1(a1 + 4)).pdiv(frames));
            let vz = lwc1(a1 + 8);
            swc1(record + 80, 0.0_f32);
            swc1(record + 84, 0.0_f32);
            swc1(record + 88, 0.0_f32);
            swc1(record + 92, 0.0_f32);
            swc1(record + 72, 0.0_f32);
            swc1(record + 76, 0.0_f32);
            swc1(record + 108, (vz).pdiv(frames));
            soundDelayStartEx(211, 0, 15, 0);
            sw(prop + 12, lw(lw(record + 96) + 12));
            return prop;
        }
    }
    0
}
