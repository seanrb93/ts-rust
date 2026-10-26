//! Small functions: the escort and story ticks, a few menu pages, the gun and goodies drawing, prop set-up and the drum, door and enemy sound helpers.

use crate::*;

/// The tick of the escort game: the level is completed (flag 0x20000000) when a team has scored, when the escortee reached its destination (the escortee's flag at +248 is
/// set and the flag 0x10000000 added) or a combo is unlocked (@ 0x21de68).
pub fn gmEscortTick() {
    let parms = g::curgameparms;
    let mut team = 0;
    let record = lw(lw(g::escort) + 352);
    'b3: {
        loop {
            if team >= 5 {
                break 'b3;
            }
            let next = team + 1;
            let score = teamGetScore(next);
            team = next;
            if score > 0 {
                break;
            }
        }
        sw(parms + 80, lw(parms + 80) | 0x20000000);
        gameLevelCompleted();
    }
    if gmEscorteeReachedDest() != 0 {
        sw(record + 248, 1);
        let parms = g::curgameparms;
        sw(parms + 80, lw(parms + 80) | 0x10000000);
        gameLevelCompleted();
    }
    if UnlockCombo() == 0 {
        return;
    }
    let parms = g::curgameparms;
    sw(parms + 80, lw(parms + 80) | 0x10000000);
    gameLevelCompleted();
}

/// Draw the map maker's main page after the menu: the two images at the top (20 and 21), the tile zoom unless something is being edited, and the filled rectangle behind the
/// pages that are not the front one (@ 0x252b68).
pub fn mapmakermain_pageDrawAfter(a0: i32) {
    let height = 31.0_f32;
    guiDrawImage(20, 0x800080, 5e+01_f32, height);
    guiDrawImage(21, 0x800080, lwc1(0x3acaa0), height);
    let mut front = lw(g::front);
    if lw(g::editmode) == 0 {
        dlSetZB(0);
        zoomtileGfx();
        front = lw(g::front);
    }
    if ((a0 - lw(front + 4)) >> 4) == lw(front + 20) - 1 {
        return;
    }
    dlFillRectangle(50, 27, 589, 196, 0x2840);
}

/// The tick of the preferences page: the settings are read, the menu's row size follows the window's width and shift, the items for the pad are enabled by the pad's flag,
/// the slider values follow the window's shift, then the menu ticks and its items are made (@ 0x20f990).
pub fn prefs_pageTick(a0: i32) {
    prefs_vars();
    let window = g::window;
    let shift = lw(window + 88);
    let rows = if lw(window + 12) < 500 { 24 } else { 128 };
    let player = lw(g::currentplayer);
    sh(g::prefs_menu + 4, rows << (shift & 31));
    let items = g::prefsmenu_items;
    let flags = lhu(items + 106) & 0xffbf;
    let shift = lw(g::window + 88);
    sh(items + 106, if (lhu(lw(player + 24) + 2) & 16) != 0 { flags } else { flags | 64 });
    if shift != 0 {
        let value = 16 >> (shift & 31);
        sw(g::gv_vibmode + 36, value);
        sw(g::gv_autoaim + 36, value);
        sw(g::gv_wpchange + 36, value);
        sw(g::gv_vibration + 36, value);
    }
    let menu = g::prefs_menu;
    ingame_menutick(a0, menu);
    let made = std_menumake(lw(a0 + 8), menu);
    sw(a0 + 12, made);
}

/// The tick of the end game page: the first item's text depends on the flag 0x40000; the button 64 either opens the confirm page, or (with the flag) ends the game at once;
/// otherwise the menu ticks and its items are made (@ 0x20fb78).
pub fn endgame_pageTick(a0: i32) {
    let text = if (lw(g::curgameparms + 80) & 0x40000) == 0 {
        lw(lw((lw(g::cur_language) << 2) + g::languages) + 1784)
    } else {
        lw(lw((lw(g::cur_language) << 2) + g::languages) + 1788)
    };
    let player = lw(g::currentplayer);
    sw(g::endgamemenu_items, text);
    if (joyNewBut(lw(player)) & 64) == 0 {
        let menu = g::endgame_menu;
        ingame_menutick(a0, menu);
        let made = std_menumake(lw(a0 + 8), menu);
        sw(a0 + 12, lw(a0 + 12) + made);
    } else {
        if (lw(g::curgameparms + 80) & 0x40000) == 0 {
            inPage(lw(lw(g::currentplayer)), g::confirmendgame_page);
            return;
        }
        let menu = 0x1fb3240;
        let state = lhu(menu + 8);
        sw(0x3afe78, -1);
        sw(g::gamestate, 4);
        sh(menu + 8, state | 6);
    }
}

/// Draw the gun of the current player (when the gun is up, the value at +260 of the player at least 13.5): the gun prop is drawn three times with the depth buffer and the blend
/// modes of the original, then the modes are put back (@ 0x2949e0).
pub fn gunGfx() {
    if lwc1(lw(g::currentplayer) + 260) < 13.5_f32 {
        return;
    }
    dlSetBlend(0);
    let zb = dlSetZB(1);
    let dither = dlSetDitherMatrix(1);
    propGfxRoom(lw(lw(g::currentplayer)) + 201, 0);
    dlSetZB(3);
    propGfxRoom(lw(lw(g::currentplayer)) + 201, 0);
    dlSetBlend(1);
    dlSetZB(2);
    dlSetDitherMatrix(2);
    propGfxRoom(lw(lw(g::currentplayer)) + 201, 1);
    dlSetZB(zb);
    dlSetDitherMatrix(dither);
}

/// Start the menu `a0`: it goes on the stack of menus and becomes the current one, its items are initialised and the first is focused unless the menu has a handler; the pad
/// of the player is cleared (the debug menu in joy mode 1 switches the debug mode on first) (@ 0x20cce0).
pub fn menuStart(a0: i32) {
    let mode = debugmenuGetJoyMode();
    let mut position = lw(g::menu_stackpos);
    if mode == 1 {
        if lw(g::debugmode) == 0 {
            toggledebug();
            position = lw(g::menu_stackpos);
        }
    }
    let mut menu = a0;
    'b2: {
        sw((position << 2) + g::menustack, a0);
        sw(g::menu_stackpos, position + 1);
        sw(g::curmenu, menu);
        let mut i = 0;
        if lbu(menu + 4) != 0 {
            let mut offset = i << 5;
            loop {
                i += 1;
                menuitemInit(menu + (offset + 16));
                menu = lw(g::curmenu);
                offset = i << 5;
                if i >= lbu(menu + 4) {
                    break 'b2;
                }
            }
        }
    }
    let mut player = lw(g::currentplayer);
    if lw(menu + 8) == 0 {
        focusnext();
        player = lw(g::currentplayer);
    }
    joyClearAll(lw(player));
}

/// The tick of the story page: when the players are all dead (the first, and the second if there is one) the level is completed with the flag 0x20000000; the first player
/// can then restart with the button 64 or quit with 16 (@ 0x2119c8).
pub fn story_pageTick_2119c8() {
    let players = lw(g::players);
    let first_dead = (((lw(players + 296) ^ 4) as u32) < 1) as i32;
    let second_dead = if lw(g::lvnumplayers) < 2 { 1 } else { (((lw(players + 2116) ^ 4) as u32) < 1) as i32 };
    let current;
    if first_dead == 0 {
        current = lw(g::currentplayer);
    } else if second_dead == 0 {
        current = lw(g::currentplayer);
    } else {
        let parms = g::curgameparms;
        let flags = lw(parms + 80);
        current = lw(g::currentplayer);
        if (flags & 0x20000000) == 0 {
            sw(parms + 80, flags | 0x20000000);
            gameLevelCompleted();
            return;
        }
    }
    if lw(current) == 0 {
        if (lw(g::curgameparms + 80) & 0x20000000) != 0 {
            if (joyNewBut(0) & 64) != 0 {
                gameLevelRestart();
                return;
            }
            if (joyNewBut(0) & 16) != 0 {
                gameLevelQuit();
                return;
            }
        }
    }
}

/// Create the prop `a0` in front of the map maker's camera (the position at +832 of the player, the angle from its direction at +844 and +852 unless it looks straight up or
/// down); it gets the external reference `a1`, or a new one for -1 (@ 0x27c1c8).
pub fn propEditCreate(a0: i32, a1: i32) {
    let frame = Frame::new(80);
    let sp = frame.sp();
    let player = lw(g::currentplayer);
    let x = lwc1(player + 832);
    sw(sp + 4, a0);
    swc1(sp + 8, x);
    swc1(sp + 12, lwc1(player + 836));
    swc1(sp + 16, lwc1(player + 840));
    let up = lwc1(player + 848);
    if !(up == 1.0_f32) && !(up == -1.0_f32) {
        let z = lwc1(player + 852);
        let x = lwc1(player + 844);
        let angle = atan2f(x, z);
        swc1(sp + 20, ps2float::mul(angle, 1.8e+02_f32).pdiv(lwc1(0x3acfa0)));
    } else {
        sw(sp + 20, 0);
    }
    let prop = propNew(lw(sp + 4), lwc1(sp + 8), lwc1(sp + 12), lwc1(sp + 16), lwc1(sp + 20));
    if a1 == -1 {
        let reference = propGetNewExtref();
        sw(prop, reference);
    } else {
        sw(prop, a1);
    }
}

/// Start loading a save again for the player `a0`: the dialog gets its title, the current card is selected again if there is one, and the load of the file is started; the
/// front end's flags 1 is set and 4 and 8 dropped (@ 0x2316c8).
pub fn signonLoadRetry(a0: i32) {
    let languages = g::languages;
    let title = GetPlayerDialogTitle(a0, lw(lw((lw(g::cur_language) << 2) + languages) + 1916));
    let card = mcseqGetCurCard();
    let mut language = lw(g::cur_language);
    if card >= 0 {
        mcseqSetCurCard(mcseqGetCurCard());
        language = lw(g::cur_language);
    }
    let entry = lw((language << 2) + languages);
    let size = signonFileSize();
    mcseqLoad(10, title, lw(entry + 1888), 0x3a1960, lw(0x3aff3c), size, 0);
    mcseqController(a0);
    let state = 0x1fc11f8;
    let flags = lw(state + 4);
    sw(0x3aff44, 0);
    sw(state + 4, (flags | 1) & -5 & -9);
}

/// Draw the goodies page: the effects, the eight message boxes fading in order from the message counter, and (between the ticks 60 and 180) the text of the page's hint
/// (@ 0x240c30).
pub fn goodies_pageGfx() {
    frontfxGfx(4);
    dlSetBlend(2);
    let mut i = 0;
    let mut counter = lw(0x3affb0);
    loop {
        let delta = counter - i;
        i += 1;
        let box_number = if -1 >= delta { 0 } else { delta };
        goodieMessageGfx(box_number, 0x10102040);
        counter = lw(0x3affb0);
        if i >= 8 {
            break;
        }
    }
    dlSetBlend(1);
    if ((lw(0x3affb0) - 60) as u32) >= 120 {
        return;
    }
    let text = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1404);
    let width = textWidth(text);
    textOutline(lw(g::window + 8) - ((((width as u32) >> 31) as i32 + width) >> 1), 180, text, lw(g::menuitem_fgcol), lw(g::menuitem_bgcol));
}

/// Set up the prop of the record `a0` from a level file: a special prop (type 128), a water prop (1024) or an ordinary one that is added to the enemy list (@ 0x228ce0).
pub fn setupProp(a0: i32) {
    let kind = getPropType(lw(a0 + 4));
    if kind == 128 {
        specialPropNew(lw(a0 + 4), lw(a0 + 24), lwc1(a0 + 8), lwc1(a0 + 12), lwc1(a0 + 16), lwc1(a0 + 20));
        return;
    }
    let angle = lwc1(a0 + 20);
    if kind == 1024 {
        waterPropNew(lw(a0 + 4), lwc1(a0 + 8), lwc1(a0 + 12), lwc1(a0 + 16), angle);
        return;
    }
    let prop = propNew(lw(a0 + 4), lwc1(a0 + 8), lwc1(a0 + 12), lwc1(a0 + 16), angle);
    enemyAddToPropList(prop);
}

/// Start the line test of a bullet against the room's model: the model's last hit is kept aside, the line from +24 to +48 is tested, and a hit records the room prop and the
/// new last hit in the bullet before the old one is put back (@ 0x289c70).
pub fn bulletInitialiseStraightLineTest(a0: i32) {
    let frame = Frame::new(64);
    let sp = frame.sp();
    let last = obGetLastHitInst(sp, sp | 4, sp | 8);
    let room = lw(lw((lw(a0 + 100) << 2) + lw(g::roomprops)) + 32);
    let hit = obInstLineTest(room, a0 + 24, a0 + 48, 1, a0 + 160, a0 + 172);
    let props = lw(g::roomprops);
    if hit == 0 {
        sw(a0 + 184, 0);
    } else {
        sw(a0 + 184, lw((lw(a0 + 100) << 2) + props));
        obConfirmLastHitInst();
        let current = obGetLastHitInst(a0 + 192, a0 + 196, a0 + 200);
        let second = lw(sp + 4);
        let third = lw(sp + 8);
        let first = lw(sp);
        sw(a0 + 188, current);
        obSetLastHitInst(last, first, second, third);
    }
}

/// An oil drum is damaged: when it has been hurt (+532 above 1) and is not already blown (model flag 4), it explodes with a burst, loses the flag 4 for 8, its decals are removed
/// and one of six sounds (98 up) plays (@ 0x2748a8).
pub fn drumDamage(a0: i32, a1: i32, _a2: i32, a3: i32, a4: i32, a5: i32) {
    let frame = Frame::new(48);
    let sp = frame.sp();
    if 1.0_f32 < lwc1(a0 + 532) {
        if (lhu(lw(a0 + 32) + 292) & 4) != 0 {
            swc1(sp, ps2float::add(lwc1(a0 + 508), lwc1(a0 + 48)));
            swc1(sp + 4, ps2float::add(lwc1(a0 + 512), lwc1(a0 + 52)));
            swc1(sp + 8, ps2float::add(lwc1(a0 + 516), lwc1(a0 + 56)));
            particleNewExplosion(0, 0, sp, a3, a4, a5, a1, 0);
            particleNewBurst(0, 15, sp, 0, 0.0_f32);
            let model = lw(a0 + 32);
            sh(model + 292, lhu(model + 292) & 0xfffb | 8);
            decalRemoveDecalsFromProp(a0);
            let roll = newrnd();
            soundStart(((roll as i32 as u32) % 6) as i32 + 98, sp);
        }
    }
}

/// Preload the models of the special effects (props 294 to 307) and their two textures (@ 0x2a96e0).
pub fn specialfxPreload() {
    for prop in 294..=307 {
        propAddPreLoad(prop);
    }
    preloadAddTexName(0x3a9690, 0);
    preloadAddTexName(0x3a96b0, 0);
}

/// The tick of the challenge's messages: each message that is not done yet, has its flag 1 and is within 600 frames of its time shows the hurry message once (the sound 200,
/// and the flag 2 of the game parameters keeps it from repeating) (@ 0x21f808).
pub fn challengeMessagesTick() {
    let mut i = 0;
    let setup = challengeGetSetup();
    let mut count = lw(setup + 20);
    if count > 0 {
        let mut offset = 0;
        loop {
            'b3: {
                let message = lw(setup + 16) + offset;
                if lw(message + 4) != 0 {
                    i += 1;
                } else {
                    let frame = lw(g::gameframe);
                    if (lhu(message) & 1) != 0 {
                        let parms = g::curgameparms;
                        if (lw(message + 16) - frame) < 600 {
                            if (lw(parms + 84) & 2) != 0 {
                                i += 1;
                                break 'b3;
                            }
                            hudMessage(0, lw(lw((lw(g::cur_language) << 2) + g::languages) + 1864));
                            soundStart(200, 0);
                            sw(parms + 84, lw(parms + 84) | 2);
                            count = lw(setup + 20);
                        }
                    }
                    i += 1;
                }
            }
            offset += 20;
            if i >= count {
                break;
            }
        }
    }
}

/// The answer of the paste question without items: the button 64 frees the items of the objects to paste, restores the mouse and pastes (answer 1); the button 16 frees the
/// pasted object (answer 2); 0 while waiting (@ 0x246a40).
pub fn confirmPasteTileNoItems() -> i32 {
    let pressed = joyNewBut(0);
    let mut object = lw(g::usedobj);
    if (pressed & 64) == 0 {
        let other = joyNewBut(0);
        let mut answer = 0;
        if (other & 16) != 0 {
            mmFreeObject(lw(g::usedobj));
            sw(g::usedobj, 0);
            answer = 2;
        }
        answer
    } else {
        let mut front = lw(g::front);
        if object != 0 {
            let mut item = lw(object + 52);
            loop {
                if item != 0 {
                    mmFreeItem(object, lw(object + 52));
                    item = lw(object + 52);
                } else {
                    object = lw(object + 56);
                    if object == 0 {
                        break;
                    }
                    item = lw(object + 52);
                }
            }
            front = lw(g::front);
        }
        let mouse_y = lw(g::savemouse_y);
        sw(front + 32, lw(g::savemouse_x));
        sw(front + 36, mouse_y);
        mmDoPaste();
        1
    }
}

/// Start the zoomed view of the tile `a0` in the map maker: the page is set, the window and the tilt are set up, and the pad information for the tile is made (@ 0x253360).
pub fn mmStartZoomTileView(a0: i32) {
    let tile = lw(a0 + 36);
    sw(g::zoomtileobj, a0);
    sw(g::zoomtilenum, (tile - g::tileset1).wrapping_mul(0x4f72c235) >> 2);
    frontPageSet(0, g::zoomtile_page);
    let front = lw(g::front);
    sw(front + 12, lw(front + 12) & -2 | 2);
    sw(0x3b0060, 5);
    mmPushCursorPos();
    let front = lw(g::front);
    sw(front + 36, 0x700000);
    sw(front + 32, 0x1400000);
    let mut angle = ps2float::sub(0.0_f32, lwc1((lw(a0 + 24) << 2) + g::tilerotang));
    if angle < 0.0_f32 {
        angle = ps2float::add(angle, 3.6e+02_f32);
    }
    zoomtileWindow(544, 190, 172);
    let distance = zoomtileSizeToDist(lw(g::zoomtilenum));
    swc1(g::zoomtileroty, angle);
    swc1(g::zoomtiledist, distance);
    swc1(g::zoomtiletilt, 45.0_f32);
    sw(g::zoomtileflags, 3);
    zoomtileWindowDest(330, 115, 600, 15);
    mm_makepadinfo(lw(g::zoomtilenum));
    sw(g::editflags, lw(g::editflags) & -0x2001);
}

/// Make a door in the volume `a1` (the position from the volume): it is closed, its angles are the volume's two angles in degrees turned to radians (@ 0x269888).
pub fn doorNewInVolume(a0: i32, a1: i32, a2: i32) {
    let volume = bgGetVolume(a1);
    let degrees = 1.8e+02_f32;
    let divisor = ld(0x3a7c00);
    let door = doorNew(a0, a2, lwc1(volume + 8), lwc1(volume + 12), lwc1(volume + 16));
    sw(door + 88, 0);
    sw(door + 140, 2);
    swc1(door + 72, dptofp(dpdiv(fptodp(ps2float::mul(lwc1(volume + 32), degrees)), divisor)));
    swc1(door + 76, dptofp(dpdiv(fptodp(ps2float::mul(lwc1(volume + 36), degrees)), divisor)));
}

/// The sound an enemy makes when it wakes: by its type (78 and 79 play delayed sounds 816 and 655, 49 plays 655) or, for the others whose state at +8 is 0, the sound 188
/// (@ 0x2cb5b8).
pub fn enemyPlayActivationSound(a0: i32) {
    let record = lw(a0 + 352);
    if frames_gate() > 0.0_f32 {
        let kind = lh(record + 4);
        if kind == 78 {
            soundDelayStartEx(816, a0 + 48, 20, 0);
            return;
        }
        let state;
        if kind >= 79 {
            if kind == 79 {
                soundDelayStartEx(655, a0 + 48, 20, 0);
                return;
            }
            state = lw(record + 8);
        } else {
            if kind == 49 {
                soundStart(655, a0 + 48);
                return;
            }
            state = lw(record + 8);
        }
        if state == 0 {
            soundStart(188, a0 + 48);
            return;
        }
    }
}
