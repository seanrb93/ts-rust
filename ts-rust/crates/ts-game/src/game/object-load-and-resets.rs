//! Functions: loading an object file, the resets of the map maker, the depth tests, the sound and the textures, the top menu, the skeleton binding, special effect groups,
//! enemy preloading, the character select gfx, the glass decals and the damage of the auto gun and the deer gun.

use crate::*;

/// Load the object file `a0` (from the preload list or the file), relocate it and its texture info, load the textures unless the flag 0x10000000 says they are shared,
/// precalculate its drawing data and return the object (@ 0x25ee00).
pub fn obLoad(a0: i32) -> i32 {
    let mut memory;
    let table;
    'b1: {
        let preloaded = preloadGetOb(a0);
        if preloaded == 0 {
            memory = fileGetMem(a0);
            if memory != 0 {
                table = lw(memory + 4);
                break 'b1;
            }
            memory = fileLoad(a0, 1, 0);
        } else {
            memory = lw(preloaded + 16);
        }
        table = lw(memory + 4);
    }
    let object = table + memory;
    let textures = lw(memory) + memory;
    sw(object + 12, textures);
    texinfoRelocate(textures, memory);
    obRelocate(object, memory);
    if (lw(lw(object + 12) + 12) & 0x10000000) == 0 {
        obLoadTextures(object);
    }
    obPrecalcGfxData(object);
    object
}

/// Preload for the tile zoom: the 21 tiles' object files and pad files are named and added to the preload list, and the models 234 to 245 (@ 0x2500c8).
pub fn zoomtilePreload() {
    memset(0x1fc4108, 0, 1536);
    let mut i = 0;
    let mut offset = i << 6;
    loop {
        let next = i + 1;
        let name = offset + g::tilefilenames;
        let pad_name = offset + g::tilepadfilenames;
        sprintf(&[name as i64, 0x3a2318_i64, next as i64, next as i64]);
        i = next;
        preloadAddOb(name, 0);
        sprintf(&[pad_name as i64, 0x3a2338_i64, 1_i64, next as i64]);
        preloadAddFile(pad_name, 0);
        offset = i << 6;
        if i >= 21 {
            break;
        }
    }
    for n in 0..12 {
        propAddPreLoad(n + 234);
    }
}

/// Explode a remote mine `a0`: the owner is found (a player or a character, by the kind at +104), a sound and an explosion with smoke and a decal are made, and the mine is
/// freed (@ 0x28a730).
pub fn bulletExplodeRemoteMine(a0: i32) {
    let kind = lw(a0 + 104);
    let mut owner = 0;
    'b1: {
        if kind == 1 {
            let number = lw(a0 + 108);
            if number >= 0 {
                owner = lw(number * 1820 + lw(g::players) + 384);
                break 'b1;
            }
        }
        if kind == 2 {
            let number = lw(a0 + 108);
            if number >= 0 {
                owner = lw(number * 0x1210 + lw(g::chrinfo) + 3020);
            }
        }
    }
    let position = a0 + 24;
    let roll = newrnd();
    soundStart(((roll as i32 as u32) % 6) as i32 + 98, position);
    particleNewExplosion(0, 0, position, lw(a0), position, a0 + 48, owner, 0);
    particleNewSmoke(0, position);
    decalNewDecal(lw(a0), lw(a0 + 100), 0, position, 0);
    bulletFree(a0);
}

/// Reset the map maker: its parts reset, the colours and fonts of the menu are set, memory for the objects (0xf000) and items (2064) is taken, the map is cleared (or loaded
/// from the backup when it is valid) and the camera mode is 8 (@ 0x248410).
pub fn mapmakerReset() {
    mmfileReset();
    zoomboxReset();
    guiReset();
    sw(g::menuitem_fgcol, 0x64646480);
    sw(g::menuitem_bgcol, 0x14141480);
    sw(g::menuitem_normalfont, 1);
    sw(g::menuitem_bigfont, 2);
    zoomtileReset();
    memset(g::button_text, 0, 20);
    mmResetTiles();
    mmClearTiles();
    mmLoadTiles();
    let objects = memAlloc(0xf000, 1);
    sw(g::objects, objects);
    let items = memAlloc(2064, 1);
    sw(g::itemspool, items);
    mmClearMap();
    mmInitView();
    let flags = lw(g::editflags);
    let backup = lw(g::mmtempbuffer);
    sw(g::editflags, flags & 0x1200);
    if lw(backup) == 0x1324626 {
        mmLoadMapFile(backup, 0);
        mmInitView();
        mmCenterMap();
        sw(g::editflags, lw(g::editflags) & -513 | flags & 0x1000);
    }
    mmResetCursorPos();
    let front = lw(g::front);
    let state = lw(front + 12);
    sw(front + 40, 0x24fe68);
    sw(g::usedobj, 0);
    sw(front + 12, state | 3);
    sw(g::otherlevel, -1);
    sw(g::editmode, 0);
    sw(g::editlevel, 0);
    mmDialogsReset();
    mmDebugObjects();
    textSelect(1);
    camSetMode(8);
}

/// Make a prop visible: the flags 3 are set and, for a character that is not placed (+92 is -1) of the kinds 1, 2, 4, 8 and 32 in a room a player sees, the model is
/// overridden and a special effect spawns at its position (@ 0x273ac0).
pub fn propMakeVisible(a0: i32) {
    let frame = Frame::new(48);
    let sp = frame.sp();
    let kind = lw(a0 + 8);
    let record = lw(a0 + 352);
    sw(a0 + 16, lw(a0 + 16) | 3);
    if kind == 64 {
        if lw(record + 92) == -1 {
            let mode = lw(record);
            let x;
            if ((mode - 1) as u32) < 2 {
                x = lwc1(record + 24);
            } else {
                if mode != 4 {
                    if mode != 8 {
                        if mode != 32 {
                            return;
                        }
                    }
                }
                x = lwc1(record + 24);
            }
            swc1(sp, x);
            let y = lwc1(record + 28);
            swc1(sp + 4, y);
            swc1(sp + 8, lwc1(record + 32));
            if mode == 8 {
                swc1(sp + 4, ps2float::add(y, 0.5_f32));
            }
            if isRoomVisibleByAnyPlayer(lw(a0 + 12)) != 0 {
                obInstSetOverrideStates(lw(a0 + 32), 3, -1, -1, 0);
                let roll = newrnd();
                specialfxNewSpawn(6, sp, (roll & 3_i64) as i32, 0, a0);
            }
        }
    }
}

/// Reset the depth test: the buffer of 0x46040 bytes (aligned to 64) is taken, in the levels but the menu level three arrays of 0x3cc0 bytes are taken and cleared, and the glow
/// texture is loaded unless the level is 104 (@ 0x2a6640).
pub fn zbtestReset() {
    let memory = memAlloc(0x46040, 1);
    let mut level = lw(g::lvlevelnum);
    sw(g::zbuffer, memory + 63 & -64);
    if level != 102 {
        let first = memAlloc(0x3cc0, 1);
        sw(g::zbval, first);
        let values = g::zbval;
        memset(first, 0, 0x3cc0);
        let second = memAlloc(0x3cc0, 1);
        sw(values + 4, second);
        memset(second, 0, 0x3cc0);
        let third = memAlloc(0x3cc0, 1);
        sw(values + 8, third);
        memset(third, 0, 0x3cc0);
        level = lw(g::lvlevelnum);
    }
    if level != 104 {
        let glow = lvGetGlowTexNum();
        emLoadTextureName(lw((glow << 2) + g::glowtextures), 0, g::glowtexnum);
    }
    zbtestRestart();
}

/// Reset the sound: the buffers of the sound memory are created (three for the level 102, four otherwise) and the flag 0x80 of every sound is cleared with its link (@ 0x204650).
pub fn soundReset() {
    if lw(g::lvlevelnum) != 102 {
        soundCreateBuffer(0, 0x5010, 0x1a8000);
        soundCreateBuffer(1, 0x1a8000, 0x1b0000);
        soundCreateBuffer(2, 0x1b0000, 0x1b8000);
        soundCreateBuffer(3, 0x168000, 0x1a8000);
    } else {
        soundCreateBuffer(0, 0x5010, 0x138000);
        soundCreateBuffer(1, 0x138000, 0x178000);
        soundCreateBuffer(2, 0x178000, 0x1b8000);
    }
    let mut sound = g::sounds;
    let end = sound + 0xaadc;
    loop {
        let flags = lhu(sound + 22);
        sw(sound + 24, 0);
        sh(sound + 22, flags & 0xff7f);
        sound += 36;
        if sound >= end {
            break;
        }
    }
    soundRestart();
}

/// The tick of the top menu: the time on the page counts up; the buttons 0x8000 and 0x2000 move the selection up or down through the entries that are not set in the mask at
/// +0 (wrapping over ten) and clear a direction flag of the front end; a change plays the sound 140 (@ 0x20dc48).
pub fn topmenuTick() {
    let player = lw(lw(g::currentplayer));
    let menu = player * 92 + 0x1fb3240;
    let mut selected = lhu(menu + 4);
    sh(menu + 10, lhu(menu + 10) + lhu(g::tickframesi));
    let pressed = joyNewBut(player);
    let old;
    'b1: {
        if (pressed & 0x8000) == 0 {
            if (joyNewBut(lw(lw(g::currentplayer))) & 0x2000) == 0 {
                old = lhu(menu + 4);
                break 'b1;
            }
            sh(menu + 10, 0);
            old = lhu(menu + 4);
            let front = lw(g::front);
            let mask = lhu(menu);
            loop {
                let next = selected + 1;
                selected = if next >= 10 { 0 } else { next };
                if (mask >> (selected & 31) & 1) == 0 {
                    break;
                }
            }
            let flags = lw(front + 12);
            sw(front + 12, flags & -33);
        } else {
            sh(menu + 10, 0);
            old = lhu(menu + 4);
            let front = lw(g::front);
            let mask = lhu(menu);
            loop {
                let next = selected - 1;
                selected = if -1 >= next { 9 } else { next };
                if (mask >> (selected & 31) & 1) == 0 {
                    break;
                }
            }
            let flags = lw(front + 12);
            sw(front + 12, flags & -17);
        }
    }
    if old != selected {
        soundStart(140, 0);
        sh(menu + 4, selected);
    }
}

/// Bind a skeleton to the object instance `a0` from the object `a1`: the bone data of the object is allocated the first time (48 bytes with the bone arrays and two matrices),
/// the skeleton is found by its number, the parts are relocated and the bone matrix numbers are made (@ 0x260940).
pub fn obBindSkeleton(a0: i32, a1: i32, a2: i32) {
    let mut bones = lw(a1 + 20);
    if bones == 0 {
        let record = memAlloc(48, 1);
        let object = lw(a1 + 4);
        sw(a1 + 20, record);
        let mask = -16;
        let first = memAlloc((lw(object + 4) << 2) + 15 & mask, 1);
        let count = lw(lw(a1 + 4));
        sw(lw(a1 + 20), first);
        let second = memAlloc((count << 2) + 15 & mask, 1);
        sw(lw(a1 + 20) + 32, second);
        let third = memAlloc(64, 1);
        sw(lw(a1 + 20) + 36, third);
        let fourth = memAlloc(64, 1);
        bones = lw(a1 + 20);
        sw(bones + 40, fourth);
    }
    let number = lw(a1 + 16);
    let skeleton = if number == -1 { 0 } else { number * 20 + g::skeletons };
    let parts = lw(skeleton + 16);
    sw(a0 + 92, bones);
    let flag = ((a2 as u32) < 1) as i32;
    sw(a0 + 88, skeleton);
    relocateParts(a0, parts, flag);
    getBoneMatrixNum(lw(a1 + 4), lw(a1 + 20));
}

/// Make a special effect: the first free one of the 56 groups is found (a group's first word is -1 when it is free), cleared and given to `specialfxCreate`; the group is freed
/// again if that fails (@ 0x2aad90).
pub fn specialfxNew(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32, a6: i32) -> i32 {
    let mut at = g::fxgroups;
    let mut result = 0;
    let mut free = -1;
    let mut i = 0;
    loop {
        if free >= 0 {
            i += 1;
        } else {
            free = if !(0 | lw(at)) == 0 { i } else { free };
            i += 1;
        }
        at += 92;
        if i >= 56 {
            break;
        }
    }
    if free >= 0 {
        let mut link = free * 92 + (g::fxgroups + 68);
        for _ in 0..4 {
            sw(link, 0);
            link -= 4;
        }
        let mut link = free * 92 + (g::fxgroups + 76);
        for _ in 0..2 {
            sw(link, 0);
            link -= 4;
        }
        result = specialfxCreate(a0, a1, a2, a3, a4, a5, a6, free);
        if result == 0 {
            specialfxFree(free);
        }
    }
    result
}

/// Preload the enemy characters: for the game kinds 6 and 7 the eleven character types of the set-up; otherwise the characters of the game parameters (the character 79 in
/// the level 25) (@ 0x2ba048).
pub fn enemyPreload() {
    let frame = Frame::new(96);
    let sp = frame.sp();
    let parms = g::curgameparms;
    if ((lw(parms + 72) - 6) as u32) >= 2 {
        if lw(g::numactivechars) > 0 {
            let mut i = 0;
            if lw(parms + 8) > 0 {
                let mut level = lw(g::lvlevelnum);
                loop {
                    let players = lw(g::lvnumplayers);
                    if level != 25 {
                        let character = lbu(i + players + (parms + 21));
                        chrPreload(character, character);
                    } else {
                        chrPreload(79, 79);
                    }
                    i += 1;
                    let count = lw(parms + 8);
                    level = lw(g::lvlevelnum);
                    if !(i < count) {
                        break;
                    }
                }
            }
        }
    } else {
        let mut i = 0;
        loop {
            lvGetChrTypeNumFromSetup(i, sp, sp + 4);
            i += 1;
            chrPreload(lw(sp), lw(sp + 4));
            if i >= 11 {
                break;
            }
        }
    }
}

/// Draw the character select effect: the character, and (unless the game state is 5 or 6) the flash that fades in and out over the window, with the black bars growing from
/// the top and the bottom (@ 0x238c60).
pub fn fxSelectCharGfx() {
    drawChar(lw((lw(lw(g::front)) << 2) + 0x1fc1238));
    let index = lw(lw(g::front)) << 2;
    let state = lw(g::gamestate);
    if lw(index + 0x1fc12b8) != 0 {
        if state != 5 {
            if state != 6 {
                let time = lwc1(index + 0x1fc12c8);
                let mut fade = ps2float::add(time, time);
                if 1.0_f32 < fade {
                    fade = ps2float::sub(2.0_f32, fade);
                }
                let window = g::window;
                let height = lw(window + 32) as f32;
                let colour = ps2float::mul(fade, 127.0_f32) as i32;
                let bar = ps2float::mul(ps2float::mul(height, 0.5_f32), fade);
                dlSetBlend(1);
                dlFillRectangle(lw(g::window), lw(window + 20), lw(window + 4), lw(window + 24), colour | 0x7f7f7f00);
                dlSetBlend(0);
                let top = lw(window + 20);
                dlFillRectangle(lw(g::window), top, lw(window + 4), ps2float::add(top as f32, bar) as i32, 0x7f000000);
                dlFillRectangle(lw(g::window), ps2float::add(lw(window + 20) as f32, ps2float::sub(height, bar)) as i32, lw(window + 4), lw(window + 24), 0x7f000000);
                dlSetBlend(1);
            }
        }
    }
}

/// Reset the texture memory: the 5000 texture slots are emptied and the texture buffers are created (one of 512 for a level, five for the level 102) (@ 0x2b6ad0).
pub fn emReset() {
    let level = lw(g::lvlevelnum);
    let mut at = texnum_table() + texnum_count() * 2 - 2;
    let mut left = texnum_count() - 1;
    loop {
        sh(at, -1);
        left -= 1;
        at -= 2;
        if left < 0 {
            break;
        }
    }
    if level != 102 {
        emCreateTexBuffer(0, 160, 352, 512);
    } else {
        emCreateTexBuffer(0, 160, 288, 200);
        emCreateTexBuffer(1, 448, 16, 16);
        emCreateTexBuffer(2, 464, 16, 16);
        emCreateTexBuffer(3, 480, 16, 16);
        emCreateTexBuffer(4, 496, 16, 16);
    }
    emRestart();
}

/// Check whether the game can leave the pause: when every player's menu has been left (or is in its closing state) the game quits from the end screen (state 4), or the
/// menus that are open exit and the time runs again (@ 0x212b98).
pub fn ingamemenuCheckUnpause() {
    let wanted = lw(0x3afe78);
    if wanted < 0 {
        if lw(g::gamestate) != 4 {
            return;
        }
    }
    let mut i = 0;
    let mut players = lw(g::lvnumplayers);
    loop {
        let state = lw(g::gamestate);
        if i >= players {
            break;
        }
        if state == 3 {
            if (lhu(i * 92 + 0x1fb3248) & 4) != 0 {
                i += 1;
                continue;
            }
        }
        if (lhu(i * 92 + 0x1fb3248) & 2) == 0 {
            break;
        }
        i += 1;
    }
    if i == players {
        if wanted < 0 {
            if lw(g::gamestate) == 4 {
                gameLevelQuit();
                return;
            }
        } else {
            let mut player = 0;
            sw(0x3afe78, -1);
            let mut exited = 0;
            if players > 0 {
                loop {
                    if (lhu(player * 92 + 0x1fb3248) & 4) != 0 {
                        player += 1;
                    } else {
                        exited += 1;
                        ingamemenuExit(player);
                        players = lw(g::lvnumplayers);
                        player += 1;
                    }
                    if player >= players {
                        break;
                    }
                }
            }
            if exited != 0 {
                timePause(0);
                return;
            }
        }
    }
}

/// Load the sounds of the level: in the game levels every sound that is not loaded and is flagged for the level (flag 8, not 16) is loaded; then the sounds flagged 4 are
/// loaded for every level but 104 (@ 0x2047c8).
pub fn soundPreload() {
    let level = lw(g::lvlevelnum);
    if level != 104 {
        if ((level - 101) as u32) >= 3 {
            if level != 105 {
                let mut i = 0;
                let mut sound = g::sounds;
                loop {
                    let mut flags = lhu(sound + 22);
                    let mut bit8 = flags & 8;
                    if (flags & 27) == 0 {
                        soundLoad(i);
                        flags = lhu(sound + 22);
                        bit8 = flags & 8;
                    }
                    'b5: {
                        let clear = bit8 == 0;
                        let bit16 = flags & 16;
                        if !clear {
                            if bit16 != 0 {
                                i += 1;
                                break 'b5;
                            }
                            if (((flags as u32) >> 8) as i32) != lw(g::lvlevelnum) {
                                i += 1;
                                break 'b5;
                            }
                            soundLoad(i);
                        }
                        i += 1;
                    }
                    sound += 36;
                    if i >= 1215 {
                        return;
                    }
                }
            }
        }
        let mut i = 0;
        let mut sound = g::sounds + 22;
        loop {
            if (lhu(sound) & 4) == 0 {
                i += 1;
            } else {
                soundLoad(i);
                i += 1;
            }
            sound += 36;
            if i >= 1215 {
                return;
            }
        }
    }
}

/// Update an animated object's matrices: when the player sees it and its animation is in use the matrices are worked out (`calMatrices`) for the characters (not in the state
/// 3) and the kinds 16 and 0x1000 under 80; otherwise, unless the object is of the kind 1, they are set to the identity (@ 0x212cf8).
pub fn animUpdate(a0: i32) {
    let animation = lw(a0);
    let player = lw(g::currentplayer);
    if animation != 0 {
        if lw(player + 1764) != 0 {
            if lw(animation + 4) != 0 {
                let mut kind = 0;
                'b2: {
                    let matrices;
                    'b3: {
                        let owner = lw(a0 + 244);
                        let owner_kind = lw(owner + 8);
                        if owner_kind == 8 {
                            if lw(lw(owner + 352) + 8) != 3 {
                                matrices = lw(a0 + 4);
                                break 'b3;
                            }
                        }
                        if owner_kind == 16 {
                            if lw(owner + 4) < 80 {
                                matrices = lw(a0 + 4);
                                break 'b3;
                            }
                        }
                        if owner_kind != 0x1000 {
                            kind = lw(a0 + 96);
                            break 'b2;
                        }
                        if lw(owner + 4) >= 80 {
                            kind = lw(a0 + 96);
                            break 'b2;
                        }
                        matrices = lw(a0 + 4);
                    }
                    if matrices != 0 {
                        calMatrices(a0);
                        return;
                    }
                    kind = lw(a0 + 96);
                }
                if kind != 1 {
                    let mut i = 0;
                    if lw(animation + 4) > 0 {
                        let mut matrices = lw(a0 + 4);
                        loop {
                            let offset = i << 6;
                            i += 1;
                            sceVu0UnitMatrix(matrices + offset);
                            if i >= lw(lw(a0) + 4) {
                                return;
                            }
                            matrices = lw(a0 + 4);
                        }
                    }
                }
            }
        }
    }
}

/// Make a decal for broken glass: the size is 5 to 9, the quadrant (0 to 3) and the flip come from random numbers, the texture, scale and colour default and are replaced by the
/// bullet's with the cheat 16; the decal is added (@ 0x2a4938).
pub fn decalNewGlassDecal(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32) -> i32 {
    let frame = Frame::new(240);
    let sp = frame.sp();
    let roll = newrnd();
    sw(sp + 64, ((roll as i32 as u32) % 5) as i32 + 5);
    let roll = newrnd();
    let quadrant = roll & 3_i64;
    let flip;
    if quadrant != 0_i64 && quadrant != 2_i64 {
        let roll = newrnd();
        flip = ((roll & 1_i64) as i32) << 1;
    } else {
        let roll = newrnd();
        flip = (((roll & 1_i64) as i32) << 1) + 1;
    }
    let allowed = lw(g::allowedcheats);
    let cheats = lw(g::unlockedcheats) & lw(g::selectedcheats);
    let first = lb(0x3a93c0);
    let second = lb(0x3a93c1);
    sd(sp + 48, ld(0x3a93b8));
    sb(sp + 56, first);
    sb(sp + 57, second);
    swc1(sp + 68, 1.0_f32);
    sw(sp + 72, 0xffffff);
    if (allowed & 16 & cheats) != 0 {
        decalGetBulletTexScaleCol(a0, sp + 64, sp + 68, sp + 72);
    }
    let size = lw(sp + 64);
    let scale = lwc1(sp + 68);
    let colour = lw(sp + 72);
    sw(sp, a3);
    sw(sp + 8, a4);
    sw(sp + 16, sp + 48);
    sw(sp + 24, if a5 >= 11 { 10 } else { a5 });
    sw(sp + 32, 0);
    decalAddNewDecal(size, quadrant as i32, flip, colour, 0, 0, a1, a2, scale)
}

/// The tick of the map description page: the first time the string editor starts with the current description; each tick it runs, and when it is accepted the description is
/// kept (and the map gets a default name if it has none) and the page closes, when cancelled the page closes (@ 0x24ccb0).
pub fn mmenterdesc_pageTick(_a0: i32) {
    let state = lbu(0x3b0038);
    if state == 0 {
        strcpy(0x1fc3e00, g::mapdesc);
        stringeditStart(0, 0x1fc3e00, 127, 3, 320, lw(g::window) + 160, lw(g::window + 20) + 40);
        stringeditSet(0);
        sw(lw(g::textbox) + 56, lw(lw((lw(g::cur_language) << 2) + g::languages) + 1868));
        sb(0x3b0038, 1);
    } else if state != 1 {
        return;
    }
    stringeditTick(0);
    stringeditSet(0);
    let textbox = lw(g::textbox);
    let front;
    let pages;
    if lw(textbox + 32) != 0 {
        strcpy(g::mapdesc, 0x1fc3e00);
        if lbu(g::mapname) != 0 {
            front = lw(g::front);
        } else {
            let source = 0x3a20f0;
            let tail = lw(source + 8);
            sd(g::mapname, ld(source));
            sw(g::mapname + 8, tail);
            front = lw(g::front);
        }
        pages = lw(front + 20);
    } else {
        front = lw(g::front);
        if lw(textbox + 36) == 0 {
            return;
        }
        pages = lw(front + 20);
    }
    sw(front + 20, pages - 1);
}

/// The tick of the story character page: the page is set, the cheats allowed follow the story and character cheats, the front end ticks; the first time the page is shown the
/// selection effects are reset and each player's character is chosen from the story data and started (@ 0x23ba38).
pub fn storychr_pageTick(a0: i32) {
    pageSet(0, g::storychr_page);
    let story = cheatsStory();
    let select = cheatsChrSelect();
    sw(g::allowedcheats, story & select);
    frontfxTick(2);
    if lbu(lw(lw(g::front)) + 0x3aff28) == 1 {
        fxSelectCharReset();
        splitpageInit(g::storychrsel_page);
        setStates(2);
        if lw(0x3aff20) > 0 {
            let mut i = 0;
            loop {
                let slot = signonGet(i);
                let story = (lw(0x1fc1218) << 3) + g::storydata;
                let first = lhu(story + 4);
                let entry = lw(g::signonarray) + slot * 2936 + 32;
                let current = lw(entry + 84);
                if current != first {
                    let second = lhu(story + 6);
                    let front = lw(g::front);
                    if current != second {
                        sw(entry + 84, if (lw(front) & 1) == 0 { first } else { second });
                    }
                }
                fxCreateStage(i);
                let character = lw(entry + 84);
                let player = i;
                i += 1;
                fxSelectCharInit(player, character);
                if i >= lw(0x3aff20) {
                    break;
                }
            }
        }
    }
    splitpageTick(a0, g::storychrsel_page);
}

/// The auto gun is damaged: when it has been hurt (+532 above 1) it explodes (a point in front of it, turned by its angle), with a burst, it drops the flags 33 and 2 and its
/// linked prop gets 33, its decals are removed and one of five sounds (98 up) plays (@ 0x274e50).
pub fn autogunDamage(a0: i32, a1: i32, _a2: i32, a3: i32, a4: i32, a5: i32) {
    let frame = Frame::new(208);
    let sp = frame.sp();
    if 1.0_f32 < lwc1(a0 + 532) {
        let angle = ps2float::mul(lwc1(a0 + 76), lwc1(0x3acda8));
        sw(sp + 72, 0);
        let zero = lwc1(sp + 72);
        let turn = (angle).pdiv(1.8e+02_f32);
        swc1(sp + 64, 1.0_f32);
        swc1(sp + 68, -0.5_f32);
        matrixRotZXY(sp, zero, turn, zero);
        matrixVecMul(sp, sp + 64);
        let y = ps2float::add(lwc1(sp + 68), lwc1(a0 + 52));
        let z = ps2float::add(lwc1(sp + 72), lwc1(a0 + 56));
        swc1(sp + 64, ps2float::add(lwc1(sp + 64), lwc1(a0 + 48)));
        swc1(sp + 68, y);
        swc1(sp + 72, z);
        particleNewExplosion(0, 1, sp + 64, a3, a4, a5, a1, 0);
        particleNewBurst(0, 15, sp + 64, 0, zero);
        let linked = lw(a0 + 536);
        sw(a0 + 16, lw(a0 + 16) & -34);
        sw(linked + 16, lw(linked + 16) | 33);
        decalRemoveDecalsFromProp(a0);
        let roll = newrnd();
        soundStart(((roll as i32 as u32) % 5) as i32 + 98, sp + 64);
    }
}

/// The deer gun is damaged: the same as the auto gun with its own offset, no flag 1 in the explosion and one of six sounds (@ 0x274fe0).
pub fn deergunDamage(a0: i32, a1: i32, _a2: i32, a3: i32, a4: i32, a5: i32) {
    let frame = Frame::new(208);
    let sp = frame.sp();
    if 1.0_f32 < lwc1(a0 + 532) {
        let angle = ps2float::mul(lwc1(a0 + 76), lwc1(0x3acdac));
        sw(sp + 68, 0);
        let zero = lwc1(sp + 68);
        let turn = (angle).pdiv(1.8e+02_f32);
        swc1(sp + 64, lwc1(0x3acdb0));
        sw(sp + 72, 0);
        matrixRotZXY(sp, zero, turn, zero);
        matrixVecMul(sp, sp + 64);
        let y = ps2float::add(lwc1(sp + 68), lwc1(a0 + 52));
        let z = ps2float::add(lwc1(sp + 72), lwc1(a0 + 56));
        swc1(sp + 64, ps2float::add(lwc1(sp + 64), lwc1(a0 + 48)));
        swc1(sp + 68, y);
        swc1(sp + 72, z);
        particleNewExplosion(0, 0, sp + 64, a3, a4, a5, a1, 0);
        particleNewBurst(0, 15, sp + 64, 0, zero);
        let linked = lw(a0 + 536);
        sw(a0 + 16, lw(a0 + 16) & -34);
        sw(linked + 16, lw(linked + 16) | 33);
        decalRemoveDecalsFromProp(a0);
        let roll = newrnd();
        soundStart(((roll as i32 as u32) % 6) as i32 + 98, sp + 64);
    }
}
