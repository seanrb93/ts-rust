//! Functions: the preload of the hud's textures, resetting the game from the game parameters, the knock out game mode's tick, making a decal, the story character select page's
//! tick, the in game menu's tick, drawing the player character select page and the tick of the sound test page.

use crate::*;

/// Preload the textures of the hud (@ 0x2b0530): the two hud texture names, the ammo icon of each gun combo the players carry (and the challenge's required gun) and the
/// icon of the bag that the game mode uses, picked from the level (or from the arcade setup's first bag object).
pub fn hudPreload() {
    let icons = 0x1feafd0;
    let mut left = 14;
    sw(icons + 4, 1);
    sw(icons + 8, 1);
    preloadAddTexName(0x3a9920, 0x1feafc0);
    preloadAddTexName(0x3a9940, icons);
    let combos = lvGetGunComboNums();
    let mut ammo = 0x1feaf80;
    let mut slots = 0x1feaf40;
    loop {
        sw(slots, -1);
        left -= 1;
        sw(ammo, 0);
        slots += 4;
        ammo += 4;
        if left < 0 {
            break;
        }
    }
    let mut combo = combos;
    let mut i = 0;
    loop {
        let mut id = -1;
        if i >= 5 {
            if (lw(g::curgameparms + 80) & 32) != 0 {
                let setup = challengeGetSetup();
                id = lw(setup + 56);
            }
        } else {
            id = lw(combo);
        }
        let offset = id << 3;
        if id > 0 {
            let gun = lw(lh(offset + (GUNCOMBOS + 4)) * 400 + (GUNS_TABLE + 16));
            let texture = lw((gun << 3) + (g::ammoinfo + 4));
            if texture != 0 {
                sw((gun << 2) + 0x1feaf80, texture);
                preloadAddTexName(texture, 0x1feafd0);
            }
        }
        i += 1;
        combo += 4;
        if i >= 6 {
            break;
        }
    }
    let mode = lw(g::curgameparms + 72);
    sw(0x3b01a0, -1);
    'icon: {
        if mode != 7 {
            if mode != 5 {
                if mode != 1 {
                    if mode != 2 {
                        break 'icon;
                    }
                }
            }
            let level = lw(g::lvlevelnum);
            sw(0x3b01a0, 0);
            if level != 100 {
                let mut i = 0;
                let setup = lvGetARCADESetup(level);
                if lw(setup + 20) <= 0 {
                    break 'icon;
                }
                loop {
                    let object = lw(setup + 16) + i * 48;
                    if lvCheckCreateFlags(lw(object)) != 0 {
                        if lw(object + 8) == 1 {
                            let kind = lw(object + 4) - 324;
                            if (kind as u32) < (14 as u32) {
                                let icon = match kind {
                                    0 => Some(0),
                                    1 => Some(5),
                                    2 => Some(4),
                                    3 => Some(2),
                                    7 => Some(7),
                                    10 => Some(8),
                                    11 => Some(3),
                                    13 => Some(6),
                                    _ => None,
                                };
                                if let Some(icon) = icon {
                                    sw(0x3b01a0, icon);
                                }
                            }
                        }
                    }
                    i += 1;
                    if i >= lw(setup + 20) {
                        break;
                    }
                }
            }
        } else {
            let level = lw(g::lvlevelnum) - 2;
            if (level as u32) < (26 as u32) {
                let icon = match level {
                    0 => Some(1),
                    6 => Some(3),
                    7 => Some(8),
                    12 => Some(7),
                    21 => Some(4),
                    25 => Some(6),
                    _ => None,
                };
                if let Some(icon) = icon {
                    sw(0x3b01a0, icon);
                    break 'icon;
                }
            }
            sw(0x3b01a0, 0);
        }
    }
    let icon = lw(0x3b01a0);
    if icon < 0 {
        return;
    }
    let name = hudGetBagIconFileName(icon);
    preloadAddTexName(name, 0x1feafd0);
}

/// Set up the game from the game parameters (@ 0x21ea90): the parameters are copied from the pending ones or, when a demo is playing, from the demo's header; the story
/// levels and the arcade modes get their fixed settings, the number of players and the level are published and the game restarts.
pub fn gameReset() {
    dual_reset_hook();
    let mut src = 0x1fbc0e8;
    let params;
    if (lw(src + 80) & 0x2000) == 0 {
        let mut dst = g::curgameparms;
        let end = src + 96;
        loop {
            let word1 = ld(src + 8);
            let word2 = ld(src + 16);
            let word3 = ld(src + 24);
            sd(dst, ld(src));
            sd(dst + 8, word1);
            sd(dst + 16, word2);
            sd(dst + 24, word3);
            src += 32;
            dst += 32;
            if src == end {
                break;
            }
        }
        let last = ld(src + 8);
        sd(dst, ld(src));
        sd(dst + 8, last);
        replayReset(0, 0, 0, 0);
        let mut current = g::curgameparms;
        if signonGet(0) >= 0 {
            let available = levelAvail(105);
            let now = g::curgameparms;
            if available != 0 {
                sw(now + 80, lw(now + 80) | 0x1000);
            }
            current = g::curgameparms;
        }
        params = current;
    } else {
        let level = lw(src + 64);
        let mut dst = g::curgameparms;
        demoLoad(level);
        let mut header = demoGetHeader();
        memset(dst, 0, 112);
        let end = header + 96;
        loop {
            let word1 = ld(header + 8);
            let word2 = ld(header + 16);
            let word3 = ld(header + 24);
            sd(dst, ld(header));
            sd(dst + 8, word1);
            sd(dst + 16, word2);
            sd(dst + 24, word3);
            header += 32;
            dst += 32;
            if header == end {
                break;
            }
        }
        let last = ld(header + 8);
        sd(dst, ld(header));
        sd(dst + 8, last);
        let now = g::curgameparms;
        sw(now + 80, lw(now + 80) & -3 | 0x2000);
        statsCombine(0);
        params = g::curgameparms;
    }
    let level = lw(params + 64);
    let reset = if level >= 104 { level == 105 } else if level < 101 { level == 6 } else { true };
    if reset {
        let now = g::curgameparms;
        sw(now + 4, 1);
        sw(now + 72, 0);
        sw(g::curgameparms, 1);
        sw(now + 8, 0);
    }
    let mode = lw(g::curgameparms + 72);
    let index = mode - 1;
    if (index as u32) < (6 as u32) {
        match index {
            0 | 2 => {}
            _ => {
                let now = g::curgameparms;
                sw(now + 80, lw(now + 80) | 1);
            }
        }
    }
    let now = g::curgameparms;
    let players = lw(g::curgameparms);
    sw(g::lvlevelnum, lw(now + 64));
    sw(g::lvnumplayers, players);
    ilinkSetNumLocalPlayers();
    let target = if lw(now + 88) <= 0 {
        lw(now + 92)
    } else {
        sw(now + 80, lw(now + 80) | 0x4000);
        lw(now + 92)
    };
    if target > 0 {
        sw(now + 80, lw(now + 80) | 0x8000);
    }
    cheatsReset();
    gameRestart();
}

/// Tick of the knock out game mode (@ 0x21dab0): the level ends when a team reaches the kill limit; while fewer than two bags are in play, after a countdown a new bag is
/// put on a random pad that no active bag is using (a bag that was dropped and is free is moved there instead of making a new one).
pub fn gmKnockOutTick() {
    let params = g::curgameparms;
    let mut free_bag = -1;
    let mut bags = 0;
    'limit: {
        if lw(params + 96) != 0 {
            let mut team = 0;
            loop {
                bags = lw(g::numPickupBags);
                if team >= 5 {
                    break 'limit;
                }
                let next = team + 1;
                let score = teamGetScore(next);
                team = next;
                if score == lw(params + 96) {
                    break;
                }
            }
            gameLevelCompleted();
        }
        bags = lw(g::numPickupBags);
    }
    let mut active = 0;
    let mut i = 0;
    if bags > 0 {
        loop {
            if (lw(lw((i << 3) + (g::bagspawn + 4)) + 16) & 1) == 0 {
                free_bag = i;
            } else {
                active += 1;
            }
            i += 1;
            if i >= bags {
                break;
            }
        }
    }
    'spawn: {
        let countdown = lw(0x3afef0);
        if active < 2 {
            let remaining = countdown - 1;
            sw(0x3afef0, remaining);
            if remaining == -1 {
                let mut pad = -1;
                sw(0x3afef0, 180);
                let random = newrnd();
                let modulo = lw(0x3afeec);
                if modulo == 0 {
                    if no_bag_spawns() {
                        return;
                    }
                    trap("break");
                }
                let mut pick = ((random as i32 as u32) % (modulo as u32)) as i32;
                let start = pick;
                'found: {
                    loop {
                        let offset = pick << 2;
                        if pad != -1 {
                            break 'found;
                        }
                        'tested: {
                            let count = lw(g::numPickupBags);
                            let mut i = 0;
                            pad = lw(offset + 0x1fbb9d8);
                            if count > 0 {
                                'scan: loop {
                                    'bag: {
                                        let bag = lw((i << 3) + (g::bagspawn + 4));
                                        let data = lw(bag + 352);
                                        if (lw(bag + 16) & 1) != 0 {
                                            let next;
                                            'conflict: {
                                                let kind = lw(data + 68);
                                                if kind == 1 {
                                                    if lw(data + 36) == pad {
                                                        next = pick + 1;
                                                        break 'conflict;
                                                    }
                                                }
                                                if kind != 2 {
                                                    i += 1;
                                                    break 'bag;
                                                }
                                                if lw(data + 64) != pad {
                                                    i += 1;
                                                    break 'bag;
                                                }
                                                next = pick + 1;
                                            }
                                            pick = if (next ^ modulo) == 0 { 0 } else { next };
                                            pad = -1;
                                            if pick == start {
                                                break 'scan;
                                            }
                                        }
                                        i += 1;
                                    }
                                    if i >= count {
                                        break 'tested;
                                    }
                                }
                                sw(0x3afef0, 0);
                            }
                        }
                        if pad != -1 {
                            break 'found;
                        }
                        if pick == start {
                            break;
                        }
                    }
                    if pad == -1 {
                        break 'spawn;
                    }
                }
                let bag;
                if free_bag >= 0 {
                    bag = lw((free_bag << 3) + (g::bagspawn + 4));
                } else {
                    let kind = lw(0x3afef4);
                    let mut scale = lwc1(0x3ac864);
                    if kind == 325 {
                        scale = lwc1(0x3ac868);
                    }
                    let at = lw(pad * 28 + lw(g::levelpads));
                    bag = propNewPickupOnPadExtref(kind, 1, kind, 5, at, 0, 0, 0, 0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32, scale);
                    enemyAddToBagList(bag, pad);
                }
                let data = lw(bag + 352);
                let position = pad * 28 + lw(g::levelpads);
                sw(bag + 16, lw(bag + 16) | 1);
                let x = lwc1(position + 12);
                sw(data + 36, pad);
                swc1(bag + 48, x);
                swc1(bag + 52, ps2float::add(lwc1(position + 16), lwc1(data + 20)));
                swc1(bag + 56, lwc1(position + 20));
                soundStart(200, 0);
            }
        }
    }
    if UnlockCombo() == 0 {
        return;
    }
    gameLevelCompleted();
}

/// Make a decal on the object `a1` at the position `a2` with the normal `a3` (@ 0x2a3bd8), for the decal source `a0`. The decal goes to the ring of pending decals: its
/// position and normal are moved into the space of the hit instance (undoing the object's rotation and, for an animated one, the bone's matrix), and it is kept
/// only if the line test from just in front of the surface to the surface does hit something.
pub fn decalNewDecalMain(a0: i32, a1: i32, a2: i32, a3: i32) {
    let frame = Frame::new(336);
    let sp = frame.sp();
    if a1 == 0 {
        return;
    }
    if (lw(a1 + 16) & 1024) != 0 {
        return;
    }
    let kind = lw(a1 + 8);
    if !(kind == 1 || kind == 2 || kind == 4 || kind == 128 || kind == 256 || kind == 512) {
        return;
    }
    let write = lw(g::s_DecalCreateListWrite);
    let next = (write + 1).wrapping_rem(15);
    if next == lw(g::s_DecalCreateListRead) {
        sw(g::s_DecalCreateListRead, (next + 1).wrapping_rem(15));
    }
    let decal = write * 44 + 0x1fe9a40;
    sw(decal, a0);
    let hit = obGetLastHitInst(decal + 8, decal + 12, decal + 16);
    let hit_object = lw(hit);
    swc1(decal + 20, lwc1(a2));
    let part = lw(decal + 8);
    let base = lw(hit_object) * 80;
    swc1(decal + 24, lwc1(a2 + 4));
    let object_kind = lw(a1 + 8);
    let z = lwc1(a2 + 8);
    sw(decal + 4, hit);
    swc1(decal + 28, z);
    swc1(decal + 32, lwc1(a3));
    swc1(decal + 36, lwc1(a3 + 4));
    swc1(decal + 40, lwc1(a3 + 8));
    if object_kind != 2 {
        let x;
        if lw(a1 + 140) != 2 {
            let angle = ps2float::mul(lwc1(a1 + 76), lwc1(0x3ad5cc)).pdiv(1.8e+02_f32);
            matrixRotY(sp + 48, angle);
            x = lwc1(a1 + 48);
        } else {
            let scale = lwc1(0x3ad5c8);
            let rx = ps2float::mul(lwc1(a1 + 72), scale).pdiv(1.8e+02_f32);
            let ry = ps2float::mul(lwc1(a1 + 76), scale).pdiv(1.8e+02_f32);
            let rz = ps2float::mul(lwc1(a1 + 88), scale).pdiv(1.8e+02_f32);
            matrixRotZXY(sp + 48, rx, ry, rz);
            x = lwc1(a1 + 48);
        }
        let y = lwc1(decal + 24);
        let z = lwc1(decal + 28);
        let position = decal + 20;
        let normal = decal + 32;
        swc1(decal + 20, ps2float::sub(lwc1(decal + 20), x));
        swc1(decal + 24, ps2float::sub(y, lwc1(a1 + 52)));
        swc1(decal + 28, ps2float::sub(z, lwc1(a1 + 56)));
        sceVu0InversMatrix(sp + 112, sp + 48);
        matrixVecMul(sp + 112, position);
        matrixVecMul(sp + 112, normal);
        let bones = lw(lw(decal + 4) + 4);
        let nx;
        if bones == 0 {
            nx = lwc1(decal + 32);
        } else {
            let bone = lb(hit_object - base + part * 80 + 1);
            if bone >= 0 {
                sceVu0CopyMatrix(sp + 48, bones + (bone << 6));
                let z = lwc1(decal + 28);
                let bz = lwc1(sp + 104);
                let y = ps2float::sub(lwc1(decal + 24), lwc1(sp + 100));
                swc1(decal + 20, ps2float::sub(lwc1(decal + 20), lwc1(sp + 96)));
                sw(sp + 96, 0);
                swc1(decal + 24, y);
                sw(sp + 100, 0);
                swc1(decal + 28, ps2float::sub(z, bz));
                sw(sp + 104, 0);
                sceVu0InversMatrix(sp + 112, sp + 48);
                matrixVecMul(sp + 112, position);
                matrixVecMul(sp + 112, normal);
            }
            nx = lwc1(decal + 32);
        }
        let ny = lwc1(decal + 36);
        let nz = lwc1(decal + 40);
        let length = ps2float::add(ps2float::add(ps2float::mul(nx, nx), ps2float::mul(ny, ny)), ps2float::mul(nz, nz)).abs().psqrt();
        let inverse = (1.0_f32).pdiv(length);
        swc1(decal + 40, ps2float::mul(nz, inverse));
        swc1(decal + 32, ps2float::mul(nx, inverse));
        swc1(decal + 36, ps2float::mul(ny, inverse));
    }
    let nz = lwc1(a3 + 8);
    let nx = lwc1(a3);
    let out = lwc1(0x3ad5d0);
    let ny = lwc1(a3 + 4);
    let back = lwc1(0x3ad5d4);
    let from_x = ps2float::add(lwc1(a2), ps2float::mul(nx, out));
    let from_y = ps2float::add(lwc1(a2 + 4), ps2float::mul(ny, out));
    swc1(sp + 24, ps2float::add(lwc1(a2 + 8), ps2float::mul(nz, out)));
    swc1(sp + 16, from_x);
    swc1(sp + 40, ps2float::mul(nz, back));
    swc1(sp + 20, from_y);
    swc1(sp + 32, ps2float::mul(nx, back));
    swc1(sp + 36, ps2float::mul(ny, back));
    sw(sp, 0);
    sw(sp + 8, 0);
    let result = bgLineTest(lw(a1 + 12), sp + 16, sp + 32, 0, 0, 0, 0, 8);
    let write = lw(g::s_DecalCreateListWrite);
    if result != 0 {
        sw(g::s_DecalCreateListWrite, (write + 1).wrapping_rem(15));
    }
}

/// Tick of the story character select page (@ 0x23b630). The player's state (the byte at 0x3aff28 + the player) is 2 while choosing, 5 when ready: left and right change the
/// character, accept locks it in, back unlocks it or leaves the page, and once every player is ready accept starts the game.
pub fn storychrsel_pageTick(a0: i32) {
    let mut menu = 0;
    let signon = signonGet(lw(lw(g::front)));
    let slot = lw(g::signonarray) + signon * 2936 + 32;
    fxSelectCharTick(1);
    animTick(a0);
    let player = lw(lw(g::front));
    let state = lbu(player + 0x3aff28);
    'tick: {
        if state == 5 {
            let pressed = joyNewBut(player);
            let front = lw(g::front);
            if (pressed & 16) != 0 {
                sb(lw(front) + 0x3aff28, 2);
                sh(a0 + 4, 1);
                fxSelectCharFlashInit(lw(front));
                actingsoundStop(lw(lw(g::front)));
                joyClearButs(lw(lw(g::front)), 16);
            }
            menu = g::chrselect_ready_waiting;
            if lw(lw(g::front)) == 0 {
                let players = lw(0x3aff20);
                let mut ready = 0;
                'count: {
                    if players > 0 {
                        if lbu(0x3aff28) == state {
                            ready += 1;
                            loop {
                                if ready >= players {
                                    break 'count;
                                }
                                if lbu(ready + 0x3aff28) != 5 {
                                    break 'count;
                                }
                                ready += 1;
                            }
                        }
                    }
                }
                if ready == players {
                    if (joyNewBut(0) & 2112) != 0 {
                        soundStart(143, 0);
                        animStopImmediately();
                        frontmpgStartGame();
                        setStates(6);
                    }
                    menu = g::chrselect_ready_go;
                }
            }
        } else if state < 6 {
            if state == 2 {
                let mut who = player;
                if who == 0 {
                    let pressed = joyNewBut(0);
                    let front = lw(g::front);
                    if (pressed & 16) != 0 {
                        pageOut();
                        return;
                    }
                    who = lw(front);
                }
                'choose: {
                    joyClearButs(who, 16);
                    let pressed = joyNewBut(lw(lw(g::front)));
                    let item;
                    if (pressed & 2112) != 0 {
                        item = lhu(a0 + 4);
                    } else {
                        let front = lw(g::front);
                        let offset = lw(front) << 2;
                        if lw(offset + 0x1fc12a8) == 0 {
                            if lhu(a0 + 4) != 1 {
                                break 'choose;
                            }
                            if (lw(front + 12) & 16) != 0 {
                                if lw(offset + 0x1fc1268) != -1 {
                                    soundStart(140, 0);
                                    fxSelectCharLeft(1);
                                    fxSelectCharInit(lw(lw(g::front)), lw(slot + 84));
                                    break 'choose;
                                }
                            }
                            if (lw(front + 12) & 32) == 0 {
                                break 'choose;
                            }
                            if lw((lw(front) << 2) + 0x1fc1268) == 1 {
                                break 'choose;
                            }
                            soundStart(140, 0);
                            fxSelectCharRight(1);
                            fxSelectCharInit(lw(lw(g::front)), lw(slot + 84));
                            break 'choose;
                        }
                        item = lhu(a0 + 4);
                    }
                    if item == 1 {
                        let front = lw(g::front);
                        let offset = lw(front) << 2;
                        if lw(offset + 0x1fc1288) == lw(offset + 0x1fc1298) {
                            sw(offset + 0x1fc12a8, item);
                        }
                        let who = lw(front);
                        if lw((who << 2) + 0x1fc1268) != 0 {
                            break 'tick;
                        }
                        fxSelectCharFlashInit(who);
                        let front = lw(g::front);
                        sw((lw(front) << 2) + 0x1fc12a8, 0);
                        sb(lw(front) + 0x3aff28, 5);
                        sh(a0 + 4, 0);
                        return;
                    }
                }
                menu = g::storychrsel_menu;
            }
        }
    }
    let players = lw(0x3aff20);
    if menu != 0 {
        if players == 1 {
            sh(menu + 6, 170);
        } else if players != 2 {
            sh(menu + 6, 150);
        } else {
            sh(menu + 6, 300);
        }
        menutick_230238(a0, menu);
        let made = std_menumake(lw(a0 + 8), menu);
        sw(a0 + 12, made);
    }
}

/// Tick of the menus of the game while playing (@ 0x20e240), the in game version of the front end menu tick: move the cursor off a disabled item, scroll the list, change a
/// value item with left and right, and open or leave a page on accept and back.
pub fn ingame_menutick(a0: i32, a1: i32) {
    let id = lhu(a0 + 4);
    if id == 0 {
        return;
    }
    let mut item = menuitem(a1, id);
    let front = lw(g::front);
    if item == 0 {
        return;
    }
    let mut index = 0;
    let mut offset = 0;
    let mut cell = 0;
    let mut direction = 0;
    let mut found = false;
    loop {
        cell = item + 16;
        if (lw(front + 12) & (4 << (index & 31))) != 0 {
            direction = offset;
            if lhu(cell + offset) != 0 {
                found = true;
                break;
            }
        }
        index += 1;
        offset += 2;
        if index >= 4 {
            break;
        }
    }
    if found {
        let mut at = cell + direction;
        loop {
            let value = lhu(at);
            sh(a0 + 4, value);
            item = menuitem(a1, value & 0xffff);
            let cells = item + 16;
            if (lhu(item + 10) & 64) == 0 {
                break;
            }
            at = cells + direction;
            if lhu(cells + direction) == 0 {
                break;
            }
        }
        soundStart(140, 0);
        let front = lw(g::front);
        sw(front + 12, lw(front + 12) & -61);
    }
    let list = isinlist(a1, lhu(a0 + 4));
    let mut front = lw(g::front);
    let mut finished = false;
    let mut buttons = 0;
    if list != 0 {
        let offset = list + 12;
        let row = (item - (list + 16)) >> 5;
        let scroll = offset + lw(front);
        let relative = row - lbu(scroll);
        let rows;
        'scroll: {
            if relative >= 0 {
                let visible = lhu(list + 2);
                if relative < visible {
                    rows = lhu(list);
                    break 'scroll;
                }
                sb(scroll, row - lbu(list + 2) + 1);
            } else {
                sb(scroll, if -1 < row { row } else { 0 });
            }
            front = lw(g::front);
            rows = lhu(list);
        }
        if row < rows - 1 {
            buttons = lw(front + 12);
            finished = true;
        } else {
            sb(offset + lw(front), lbu(list) - lbu(list + 2));
        }
    }
    if !finished {
        front = lw(g::front);
        buttons = lw(front + 12);
    }
    if (buttons & 48) != 0 {
        let flags = lhu(item + 10);
        if (flags & 4) != 0 {
            let moved;
            if (flags & 64) != 0 {
                soundStart(142, 0);
                front = lw(g::front);
                moved = lw(front + 12);
            } else {
                let mut step = lw(front + 16);
                if (lw(lw(item + 12) + 12) & 2) != 0 {
                    step = step * 655;
                }
                let delta = step >> 5;
                if delta != 0 {
                    soundStart(141, 0);
                    let value = lw(item + 12);
                    gvSetValue(value, lw(value) + delta);
                    return;
                }
                moved = lw(front + 12);
            }
            let current = lw(g::currentplayer);
            sw(front + 12, moved & -61);
            joyClearAll(lw(current));
        }
    }
    let pressed = joyNewBut(lw(lw(g::currentplayer)));
    if (pressed & 64) != 0 {
        if (lhu(item + 10) & 1) != 0 {
            let current = lw(g::currentplayer);
            if lhu(item + 8) != 0x8000 {
                inPage(lw(current), lw(item + 12));
            } else {
                outPage(lw(current), lw(item + 12));
            }
            joyClearAll(lw(lw(g::currentplayer)));
            return;
        }
    }
    if (joyNewBut(lw(lw(g::currentplayer))) & 16) != 0 {
        let back = menuitem(a1, 0x8000);
        if back != 0 {
            outPage(lw(lw(g::currentplayer)), lw(back + 12));
            joyClearAll(lw(lw(g::currentplayer)));
            return;
        }
    }
}

/// Draw the player's character select page (@ 0x237030): the highlight of the page's item and the character's flash, then, by the state of the player, the name of the
/// character picked (state 2), the handicap with its arrows (state 3) or the team name on a gradient bar with arrows (state 4).
pub fn chrplayer_pageDraw(a0: i32) {
    let frame = Frame::new(192);
    let sp = frame.sp();
    let signon = signonGet(lw(lw(g::front)));
    let slot = lw(g::signonarray) + signon * 2936 + 32;
    let character = lw(slot + 84);
    if lw(lw(g::front)) > 0 {
        menuitemHighlight(a0);
    }
    fxSelectCharGfx();
    let found = frontGetBox(a0, lhu(a0 + 4));
    let mut front = lw(g::front);
    if found != 0 {
        if lw(found + 20) != 0 {
            let window = g::window;
            let top = lhu(found + 2);
            let wide = lw(window + 12);
            let window_top = lw(window + 20);
            let centre = lw(window + 8);
            let quarter = (if -1 < wide { wide } else { wide + 3 }) >> 2;
            let right = centre + quarter;
            let y0 = window_top + top + 1;
            let y1 = window_top + top + lhu(found + 6) - 1;
            let left = centre - quarter;
            let colour = lbu(g::flashalpha) | 0x8000;
            sw(sp, left);
            sw(sp + 4, y0);
            sw(sp + 8, right);
            sw(sp + 12, y1);
            dlFillRectangle(left, y0, right, y1, colour);
        }
        front = lw(g::front);
    }
    let player = lw(front);
    let state = lbu(player + 0x3aff28);
    if state == 3 {
        let languages = g::languages;
        let text = lw((lh((lw(slot + 92) << 1) + g::handicapstrings) << 2) + lw((lw(g::cur_language) << 2) + languages));
        let width = textWidth(text);
        let window = g::window;
        let x = lw(window + 8) - ((((width as u32) >> 31) as i32 + width) >> 1);
        let row = lw(window + 20) + lhu(frontGetBox(a0, 1) + 2);
        let colour = if (lw(g::framenum) & 32) == 0 { 0x40787880 } else { 0x7f7f7f80 };
        textOutline(x, row, text, colour, 0x5080);
        drawSelectCharArrows(row, text);
        let hint = lw(lw((lw(g::cur_language) << 2) + languages) + 1684);
        let width = textWidth(hint);
        let fg = lw(g::menuitem_fgcol);
        let bg = lw(g::menuitem_bgcol);
        textOutline(lw(window + 8) - ((((width as u32) >> 31) as i32 + width) >> 1), row - 12, hint, fg, bg);
    } else if state >= 4 {
        if state == 4 {
            if lw(a0 + 12) != 0 {
                let name = lw((lh((lw((player << 2) + 0x1fc2c88) << 3) + g::teamdata) << 2) + lw((lw(g::cur_language) << 2) + g::languages));
                let blink = if (lw(g::tickframenum) & 32) != 0 { 0x7f7f7f80 } else { 0x40787880 };
                let select = textSelect(2);
                let found = frontGetBox(a0, 1);
                let window = g::window;
                let shift = lw(window + 88);
                let row = lw(window + 20) + lhu(found + 2);
                let left = lw(window + 8) - (180 >> (shift & 31));
                let bar_bottom = if (lbu(g::fontflags) & 1) == 0 { row + lw(lw(g::curfont) + 12) } else { row + (lw(lw(g::curfont) + 12) >> 1) };
                let right = left + (360 >> (shift & 31));
                let team_colour = lw((lw((lw(lw(g::front)) << 2) + 0x1fc2c88) << 3) + (g::teamdata + 4));
                guiHGradient(left, row - 1, right, bar_bottom, team_colour, 128, 0);
                let item = lhu(a0 + 4) ^ 1;
                let fg = if item == 0 { blink } else { lw(g::menuitem_fgcol) };
                let bg = if item != 0 { lw(g::menuitem_bgcol) } else { 0x5080 };
                let width = textWidth(name);
                textOutline(lw(window + 8) - ((((width as u32) >> 31) as i32 + width) >> 1), row, name, fg, bg);
                textSelect(select);
                if lhu(a0 + 4) == 1 {
                    let y = (row + 3) as f32;
                    guiDrawImage(20, 0x800080, (lw(window + 8) - (200 >> (lw(window + 88) & 31)) - 18) as f32, y);
                    guiDrawImage(21, 0x800080, (lw(window + 8) + (200 >> (lw(window + 88) & 31))) as f32, y);
                }
            }
        }
    } else if state == 2 {
        let name = lw((lh(character * 44 + CHRS_TABLE) << 2) + lw((lw(g::cur_language) << 2) + g::languages));
        drawCharName(a0, 1, name, 1);
    }
}

/// Tick of the sound test page (@ 0x2073c0): left and right (or the shoulder buttons) step through the 1215 sounds, accept plays the sound, and the buttons on the
/// parameter items change the sound's envelope values, volume and note; the sound's table entry is edited in place.
pub fn main_pageTick_2073c0(a0: i32) {
    let sound;
    'pick: {
        let previous;
        let mut step_down = false;
        if (lw(lw(g::front) + 12) & 16) != 0 && lhu(a0 + 4) == 2 {
            previous = lw(g::snd);
            step_down = true;
        } else {
            let pressed = joyNewBut(0);
            previous = lw(g::snd);
            if (pressed & 4) != 0 {
                step_down = true;
            }
        }
        if step_down {
            let value = previous - 1;
            sw(g::snd, value);
            if value < 0 {
                sw(g::snd, 1214);
            }
            setcursound(lw(g::snd));
            sound = lw(g::snd);
            break 'pick;
        }
        let current;
        if (lw(lw(g::front) + 12) & 32) != 0 && lhu(a0 + 4) == 2 {
            current = lw(g::snd);
        } else {
            let pressed = joyNewBut(0);
            current = lw(g::snd);
            if (pressed & 8) == 0 {
                sound = lw(g::snd);
                break 'pick;
            }
        }
        let value = current + 1;
        sw(g::snd, value);
        if value >= 1215 {
            sw(g::snd, 0);
        }
        setcursound(lw(g::snd));
        sound = lw(g::snd);
    }
    let table = g::sounds;
    let note_id = lhu(sound * 36 + table + 18);
    let mut note = getNote(note_id);
    gvSetValue(g::gv_volume, lhu(lw(g::snd) * 36 + table + 20));
    let pressed = joyNewBut(0);
    let front = lw(g::front);
    if (pressed & 256) == 0 {
        'change: {
            let pressed = joyNewBut(0);
            let current = lw(g::snd);
            if (pressed & 2048) == 0 {
                if (joyNewBut(0) & 64) == 0 {
                    let buttons = lw(lw(g::front) + 12);
                    let delta;
                    let item_xor;
                    if (buttons & 16) == 0 {
                        if (buttons & 32) == 0 {
                            break 'change;
                        }
                        let item = lhu(a0 + 4);
                        let param = item - 5;
                        if (param as u32) < (5 as u32) {
                            let cell = (param << 1) + lw(g::snd + 4);
                            let value = lhu(cell) + 1;
                            let over = (lw((param << 2) + g::envmax) < (value << 16) >> 16) as i32;
                            sh(cell, value);
                            if over == 0 {
                                break 'change;
                            }
                            sh(cell, 0);
                            break 'change;
                        }
                        item_xor = item ^ 10;
                        delta = note + 1;
                    } else {
                        let item = lhu(a0 + 4);
                        let param = item - 5;
                        item_xor = item ^ 10;
                        if (param as u32) < (5 as u32) {
                            let cell = (param << 1) + lw(g::snd + 4);
                            let value = lhu(cell) - 1;
                            sh(cell, value);
                            if (value << 16) >= 0 {
                                break 'change;
                            }
                            sh(cell, lhu((param << 2) + g::envmax));
                            break 'change;
                        }
                        delta = note - 1;
                    }
                    note = if item_xor == 0 { delta } else { note };
                } else {
                    let item = lhu(a0 + 4);
                    if item != 12 {
                        if item == 2 {
                            sw(g::first_sound, -0x80000000_i64 as i32);
                        }
                    } else {
                        sounddataSave();
                        startMessage(0x399bc8, 0x7f7f7f80, 0);
                    }
                }
            } else {
                soundStart(current, 0);
            }
        }
        let actual = getNote(note_id);
        let below = (note < 576) as i32;
        if note != actual {
            let clamped = if below != 0 { if -1 >= note { 0 } else { note } } else { 575 };
            sh(lw(g::snd) * 36 + (g::sounds + 18), lhu((clamped << 1) + 0x1fb20a8));
        }
        let menu = g::main_menu;
        menutick_2062c0(a0, menu);
        let table = g::sounds;
        let y = ((lhu(menu + 6) + 30) << 16) >> 16;
        let end = lw(a0 + 8) + lw(a0 + 12) * 24;
        sh(lw(g::snd) * 36 + table + 20, lhu(g::gv_volume));
        makebox(end, 300, y, 200, 50, 0, 1, 0x206d18);
        let items = lw(a0 + 8);
        let count = ((end + 24 - items).wrapping_mul(-0x55555555)) >> 3;
        sw(a0 + 12, count);
        sw(g::i0000 + 32, lw(lw(g::snd) * 36 + table + 4));
        let made = std_menumake(count * 24 + items, menu);
        sw(a0 + 12, lw(a0 + 12) + made);
    } else {
        sw(front + 12, 0);
        sw(front + 20, 0);
    }
}
