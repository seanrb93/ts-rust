//! Functions: the player auto aim tick, the audio and video page's drawing, the default enemy spawn handler, saving the props of a level, the map maker's menu tick, the
//! deathmatch game mode's tick, the preload of the background files and the drop of a bag.

use crate::*;

/// Tick of the auto aim of the players (@ 0x280858). On every fourth frame one (player, character) pair per call is tested, as many as there are pairs at most: the character
/// that is not on the player's team and is alive is aimed at (`playerAutoAimCalc`) and, when the aim is good enough and a line to the character is clear, the pair is
/// marked as visible; otherwise the character's flags are set so it is no longer a target.
pub fn playerAutoAimTick() {
    let frame = Frame::new(320);
    let sp = frame.sp();
    let counter = lw(0x3b0134) + 1;
    let mut found = 0;
    let mut tested = 0;
    sw(0x3b0134, counter);
    if (counter & 3) == 0 {
        let mut players = lw(g::lvnumplayers);
        let mut chars = lw(g::numactivechars);
        if (players * (players + chars)) > 0 {
            let mut to = lw(g::nexttestto);
            loop {
                tested += 1;
                to += 1;
                sw(g::nexttestto, to);
                let mut from;
                if to < players + chars {
                    from = lw(g::nexttestfrom);
                } else {
                    let previous = lw(g::nexttestfrom);
                    to = 0;
                    sw(g::nexttestto, 0);
                    from = previous + 1;
                    sw(g::nexttestfrom, from);
                    if from >= players {
                        from = 0;
                        sw(g::nexttestfrom, 0);
                    }
                }
                'pair: {
                    if from != to {
                        let player = lw(g::players) + from * 1820;
                        let chr = lw(g::chrinfo) + to * 0x1210;
                        let result = lw(player + 820) + to * 12;
                        let prop = lw(chr + 3020);
                        let owner = lw(lw(player + 384) + 352);
                        sw(result, 0);
                        let count = lw(g::lvnumplayers);
                        if lw(owner + 24) != lw(chr + 24) {
                            let heading;
                            if to < count {
                                heading = lwc1(prop + 76);
                            } else {
                                let mode = lw(chr + 2716);
                                if mode == 0 {
                                    break 'pair;
                                }
                                if mode == 0x2000 {
                                    break 'pair;
                                }
                                let flags = lw(prop + 16);
                                if (flags & 128) != 0 {
                                    break 'pair;
                                }
                                if (flags & 0xf000000) == 0 {
                                    break 'pair;
                                }
                                if (flags & 256) != 0 {
                                    break 'pair;
                                }
                                heading = lwc1(prop + 76);
                            }
                            let z = lwc1(prop + 56);
                            let x = lwc1(prop + 48);
                            let angle = ps2float::mul(heading, lwc1(0x3acfe4)).pdiv(1.8e+02_f32);
                            let y = lwc1(prop + 52);
                            matrixTransRotY(sp + 16, x, y, z, angle);
                            let model = lw(lw(chr + 3020) + 32);
                            let bone = lw(model + 4) + (lw(lw(model + 92) + 20) << 6);
                            sceVu0MulMatrix(sp + 80, sp + 16, bone);
                            let bx = lwc1(sp + 128);
                            let by = ps2float::sub(lwc1(sp + 132), lwc1(0x3acfe8));
                            let bz = lwc1(sp + 136);
                            swc1(sp + 144, bx);
                            swc1(sp + 152, bz);
                            swc1(sp + 148, by);
                            swc1(sp + 160, ps2float::sub(bx, lwc1(player + 152)));
                            swc1(sp + 164, ps2float::sub(by, lwc1(player + 156)));
                            swc1(sp + 168, ps2float::sub(bz, lwc1(player + 160)));
                            playerAutoAimCalc(player + 188, sp + 160, result);
                            if 0.5_f32 < lwc1(result + 4) {
                                sw(sp, 1);
                                sw(sp + 8, 0);
                                let hit = bgLineTest(lw(player + 796), player + 152, sp + 160, 0, 0, 0, 0, 120);
                                found = 1;
                                if hit == 0 {
                                    let from_now = lw(g::nexttestfrom);
                                    let chrs = lw(g::chrinfo);
                                    let target = lw(g::nexttestto) * 0x1210 + chrs;
                                    let current = lw(g::currentplayer);
                                    if lw(target + 24) != lw(from_now * 0x1210 + chrs + 24) {
                                        if lw(current + 296) != 4 {
                                            sw(target + 2776, from_now);
                                        }
                                    }
                                    sw(result, 1);
                                    break 'pair;
                                }
                                if lw(hit + 8) != 2 {
                                    break 'pair;
                                }
                            }
                            let target = lw(g::nexttestto) * 0x1210 + lw(g::chrinfo);
                            let flags = lw(target + 2704);
                            sw(target + 2776, -1);
                            sw(target + 2704, flags | 8);
                        }
                    }
                }
                if found != 0 {
                    break;
                }
                players = lw(g::lvnumplayers);
                chars = lw(g::numactivechars);
                to = lw(g::nexttestto);
                if tested >= players * (players + chars) {
                    break;
                }
            }
        }
    }
}

/// Draw the audio and video page (@ 0x236268): the background, then, on the page of the screen position, a frame of eight filled rectangles and four lines drawn around the
/// preview area of the window.
pub fn audiovideo_pageDraw(a0: i32) {
    let frame = Frame::new(192);
    let sp = frame.sp();
    pagebackgroundGfx_230688(a0);
    if lhu(a0 + 4) != 5 {
        return;
    }
    let window = g::window;
    let colour = lbu(g::quickflash) | 0x800000;
    let alpha = lbu(g::flashalpha);
    sw(sp + 4, lw(g::window));
    let top = lw(window + 20);
    let left = lw(sp + 4) + 2;
    sw(sp + 16, alpha | 0x7f7f7f00);
    sw(sp, left);
    let line_top = top + 2;
    let line_bottom_a = top + 11;
    let right_a = lw(sp + 4) + 202;
    sw(sp + 20, right_a);
    let bottom_a = top + 52;
    let window_bottom = lw(window + 24);
    let below = top + 12;
    let above = window_bottom - 12;
    sw(sp + 12, lw(window + 4));
    dlFillRectangle(lw(sp), line_top, right_a, line_bottom_a, colour);
    let inner_left = lw(sp + 4) + 22;
    sw(sp + 24, inner_left);
    sw(sp + 8, lw(sp + 12) - 2);
    dlFillRectangle(lw(sp), below, inner_left, bottom_a, colour);
    let width = lw(sp + 12);
    let right_left = width - 202;
    let right_inner = width - 21;
    dlFillRectangle(right_left, line_top, lw(sp + 8), line_bottom_a, colour);
    let line_bottom_b = window_bottom - 2;
    let bottom_b = window_bottom - 52;
    dlFillRectangle(right_inner, below, lw(sp + 8), bottom_a, colour);
    let line_end = window_bottom - 3;
    dlFillRectangle(right_inner, bottom_b, lw(sp + 8), line_bottom_b, colour);
    dlFillRectangle(right_left, above, lw(sp + 12) - 22, line_bottom_b, colour);
    dlFillRectangle(lw(sp), bottom_b, lw(sp + 4) + 21, line_bottom_b, colour);
    dlFillRectangle(lw(sp + 24), above, lw(sp + 20), line_bottom_b, colour);
    dlDrawLine(lw(sp), line_top, lw(sp + 8), line_top, lw(sp + 16));
    dlDrawLine(lw(sp), line_bottom_b, lw(sp + 8), line_bottom_b, lw(sp + 16));
    dlDrawLine(lw(sp), line_top, lw(sp), line_end, lw(sp + 16));
    dlDrawLine(lw(sp + 8), line_top, lw(sp + 8), line_end, lw(sp + 16));
}

/// The default handler of the spawn points of the level (@ 0x2cbbc8): every spawn entry whose create flags are met keeps a countdown and, when it runs out and fewer than 10
/// characters are alive from spawns and the entry has not spawned its limit, a character of the entry's type is made at the entry's pad. An entry that waits for a
/// player to come near (`+36` flag 4 or 2) is armed once such a player is found.
pub fn enemyAIDefaultSpawnHandler1() {
    let frame = Frame::new(176);
    let sp = frame.sp();
    let mut index = 0;
    let guncombos = lvGetGunComboNums();
    let setup = lw(g::setup);
    if lw(setup + 68) > 0 {
        loop {
            let next = index + 1;
            'entry: {
                let spawn = lw(lw(g::setup) + 64) + index * 52;
                let valid = lvCheckCreateFlags(lw(spawn));
                if valid != 0 {
                    let mut flags = lw(spawn + 36);
                    let mut armed = flags & 1;
                    let limit;
                    if armed != 0 {
                        limit = lhu(spawn + 28);
                    } else {
                        let mut count = lw(g::lvnumplayers);
                        let mut i = 0;
                        if count > 0 {
                            let mut wait = flags & 4;
                            loop {
                                'player: {
                                    if wait != 0 {
                                        if lw(i * 0x1210 + lw(g::chrinfo) + 484) == 0 {
                                            i += 1;
                                            break 'player;
                                        }
                                    }
                                    'arm: {
                                        let chr = i * 0x1210;
                                        if (flags & 2) != 0 {
                                            let chr_ptr = chr + lw(g::chrinfo);
                                            let pad = padNumFromExtref(lw(spawn + 16));
                                            flags = lw(spawn + 36);
                                            if lw(chr_ptr + 680) != pad {
                                                break 'arm;
                                            }
                                        }
                                        let armed_flags = flags | 1;
                                        sw(spawn + 36, armed_flags);
                                        flags = armed_flags;
                                    }
                                    count = lw(g::lvnumplayers);
                                    i += 1;
                                }
                                wait = flags & 4;
                                if i >= count {
                                    break;
                                }
                            }
                            armed = flags & 1;
                        }
                        if armed == 0 {
                            break 'entry;
                        }
                        limit = lhu(spawn + 28);
                    }
                    let spawned = lw(spawn + 40);
                    let reached = spawned >= limit;
                    let active = lw(g::totactivespawnedchars);
                    if !reached {
                        if active >= 10 {
                            break 'entry;
                        }
                        if lw(spawn + 44) >= lhu(spawn + 24) {
                            break 'entry;
                        }
                        let mut ready = 0;
                        'timer: {
                            if spawned != 0 {
                                let remaining = ps2float::sub(lwc1(spawn + 48), (lwc1(g::updateframesf)).pdiv(6e+01_f32));
                                swc1(spawn + 48, remaining);
                                if !(remaining <= 0.0_f32) {
                                    break 'timer;
                                }
                            }
                            ready = 1;
                        }
                        if ready == 0 {
                            break 'entry;
                        }
                        let pad = lvGetPadExtrefFromData(lw(spawn + 20));
                        if pad != 0 {
                            let combo = lw(spawn + 8);
                            let gun = if combo == -1 { -1 } else { lw((combo << 2) + guncombos) };
                            lvGetChrTypeNumFromSetup(lw(spawn + 4), sp, sp + 4);
                            let chr_type = lw(sp);
                            let chr_num = lw(sp + 4);
                            let start_pad = padNumFromExtref((((lw(spawn + 28) & -0x10000) as u32) >> 16) as i32);
                            let npc = enemyCreateNPC(pad, chr_type, chr_num, gun, lw(spawn + 12), start_pad, (((lw(spawn + 24) & -0x10000) as u32) >> 16) as i32);
                            if npc != 0 {
                                let total = lw(spawn + 44);
                                let made = lw(spawn + 40) + 1;
                                let chrs = lw(g::chrinfo);
                                sw(g::totactivespawnedchars, lw(g::totactivespawnedchars) + 1);
                                sw(spawn + 40, made);
                                sw(npc * 0x1210 + chrs + 3032, spawn);
                                sw(spawn + 44, total + 1);
                            }
                        }
                        swc1(spawn + 48, lwc1(spawn + 32));
                    }
                }
            }
            index = next;
            if index >= lw(lw(g::setup) + 68) {
                break;
            }
        }
    }
}

/// Save the props of the level to the prop file (@ 0x269078): the header, the number of used props and one line for each (its index, kind, position and heading) go to a
/// buffer that is written to the file of the level (the old file is copied away first) and the key file is written.
pub fn propEditSave() {
    let frame = Frame::new(1248);
    let sp = frame.sp();
    sprintf(&[sp as i64, 0x3a7b00, lw(g::lvlevelnum) as i64]);
    let buffer = malloc(0xf4240);
    let count = lw(g::num_props);
    let mut levelprops = 0;
    let mut used = 0;
    sw(sp + 1072, sp + 64);
    if count > 0 {
        levelprops = lw(g::levelprops);
        let mut i = 0;
        loop {
            let prop = i * 592;
            i += 1;
            used = if !(0 | lw(prop + levelprops)) != 0 { used + 1 } else { used };
            if i >= count {
                break;
            }
        }
    }
    sprintf(&[buffer as i64, 0x3a7b28, levelprops as i64, count as i64, 0x3a0000]);
    let mut end = buffer + strlen(buffer);
    sprintf(&[end as i64, 0x3a7b68]);
    end += strlen(end);
    sprintf(&[end as i64, 0x3a7b90, 1, used as i64]);
    end += strlen(end);
    sprintf(&[end as i64, 0x3a7bc0]);
    let length = strlen(end);
    let mut remaining = lw(g::num_props);
    end += length;
    if remaining > 0 {
        let mut i = 0;
        let mut offset = 0;
        loop {
            let prop = lw(g::levelprops) + offset;
            if lw(prop) != -1 {
                let kind = lw(prop + 4) * 28 + (PROPTABLE + 8);
                let x = fptodp(lwc1(prop + 48));
                let y = fptodp(lwc1(prop + 52));
                let z = fptodp(lwc1(prop + 56));
                let heading = fptodp(lwc1(prop + 76));
                sprintf(&[end as i64, 0x3a7be0, lw(prop) as i64, lw(kind) as i64, x, y, z, heading]);
                let line = strlen(end);
                remaining = lw(g::num_props);
                end += line;
            }
            i += 1;
            offset += 592;
            if i >= remaining {
                break;
            }
        }
    }
    sprintf(&[lw(sp + 1072) as i64, 0x3af918, sp as i64]);
    let size = fileSize(sp);
    let backup = lw(sp + 1072);
    if size <= 0 {
        fileSave(sp, buffer, end - buffer);
    } else {
        fileCopy(sp, backup);
        fileSave(sp, buffer, end - buffer);
    }
    free(buffer);
    WritePropKeyFile();
}

/// Tick of the map maker's menus (@ 0x24c3a8): move the cursor off a disabled item, scroll the list to keep the item in view, change a value item with left and right and
/// act on the accept and back buttons.
pub fn menutick_24c3a8(a0: i32, a1: i32) {
    let id = lhu(a0 + 4);
    if id == 0 {
        return;
    }
    let item = menuitem(a1, id);
    if item == 0 {
        return;
    }
    let mut cell = item + 16;
    let mut i = 0;
    let mut moved = false;
    loop {
        let front = lw(g::front);
        if i >= 4 {
            break;
        }
        if (lw(front + 12) & (4 << (i & 31))) == 0 {
            cell += 2;
        } else {
            if lhu(cell) != 0 {
                moved = true;
                break;
            }
            cell += 2;
        }
        i += 1;
    }
    if moved {
        soundStart(140, 0);
        sh(a0 + 4, lhu(cell));
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
        let low = row - 2;
        let rows;
        'scroll: {
            if relative >= 2 {
                if relative < lhu(list + 2) - 2 {
                    rows = lhu(list);
                    break 'scroll;
                }
                sb(scroll, row - lbu(list + 2) + 3);
            } else {
                sb(scroll, if -1 >= low { 0 } else { low });
            }
            front = lw(g::front);
            rows = lhu(list);
        }
        if row < rows - 2 {
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
    let mut item_flags = lhu(item + 10);
    let mut plain = 0;
    'change: {
        if (buttons & 48) != 0 {
            plain = item_flags & 64;
            if (item_flags & 4) != 0 {
                if plain == 0 {
                    let mut step = lw(front + 16);
                    if (lw(lw(item + 12) + 12) & 2) != 0 {
                        step = step * 655;
                    }
                    let delta = step >> 5;
                    plain = item_flags & 64;
                    if delta == 0 {
                        break 'change;
                    }
                    soundStart(141, 0);
                    let value = lw(item + 12);
                    gvSetValue(value, lw(value) + delta);
                    return;
                }
                soundStart(142, 0);
                item_flags = lhu(item + 10);
            }
        }
        plain = item_flags & 64;
    }
    if plain == 0 {
        if (joyNewBut(0) & 64) != 0 {
            if lhu(item + 8) != 0x8000 {
                let flags = lhu(item + 10);
                if (flags & 1) == 0 {
                    return;
                }
                soundStart(140, 0);
                frontPageSet(lw(lw(g::front) + 20), lw(item + 12));
                sh(lw(a0) + 8, lhu(a0 + 4));
                return;
            }
            soundStart(142, 0);
            mmExitMenu(a0);
            return;
        }
    }
    if (joyNewBut(0) & 16) != 0 {
        if menuitem(a1, 0x8000) != 0 {
            soundStart(142, 0);
            mmExitMenu(a0);
            return;
        }
    }
}

/// Tick of the deathmatch game mode (@ 0x21d468): announce the first kill, and when the game is over (flag 0x8000) find the best team score; end the level when a team has
/// the target score. Otherwise warn when a team is three or two points short of the target and complete the level when a combo is unlocked.
pub fn gmDeathmatchTick() {
    let frame = Frame::new(272);
    let sp = frame.sp();
    let params = g::curgameparms;
    let mut best = i32::MIN;
    'first: {
        if (lw(params + 80) & 0x8000000) == 0 {
            let mut i = 0;
            if (lw(g::lvnumplayers) + lw(g::numactivechars)) > 0 {
                let mut offset = i * 0x1210;
                let mut language;
                loop {
                    let kills = chrCountKills(lw(g::chrinfo) + offset);
                    language = lw(g::cur_language);
                    if kills > 0 {
                        break;
                    }
                    i += 1;
                    offset = i * 0x1210;
                    if i >= lw(g::lvnumplayers) + lw(g::numactivechars) {
                        break 'first;
                    }
                }
                let strings = lw((language << 2) + g::languages);
                let name = lw(offset + lw(g::chrinfo) + 16);
                let format = lw(strings + 1772);
                let text = vafmt(&[0x3ae588, format as i64, name as i64, strings as i64]);
                let flags;
                if lw(g::lvnumplayers) <= 0 {
                    flags = lw(params + 80);
                } else {
                    let mut player = 0;
                    loop {
                        hudMessage(player, text);
                        player += 1;
                        if player >= lw(g::lvnumplayers) {
                            break;
                        }
                    }
                    flags = lw(params + 80);
                }
                sw(params + 80, flags | 0x8000000);
            }
        }
    }
    'over: {
        if (lw(g::curgameparms + 80) & 0x8000) != 0 {
            memset(sp, 0, 108);
            let mut players = lw(g::lvnumplayers);
            let mut chars = lw(g::numactivechars);
            let mut i = 0;
            loop {
                if i >= players + chars {
                    break 'over;
                }
                let team = lw(i * 0x1210 + lw(g::chrinfo) + 24);
                let seen = sp + team;
                if lbu(seen) != 0 {
                    i += 1;
                } else {
                    sb(seen, 1);
                    let score = teamGetScore(team);
                    best = if best < score { score } else { best };
                    players = lw(g::lvnumplayers);
                    if score >= lw(g::curgameparms + 92) {
                        break;
                    }
                    chars = lw(g::numactivechars);
                    i += 1;
                }
            }
            gameLevelCompleted();
        }
    }
    'warn: {
        let params = g::curgameparms;
        if best != i32::MIN {
            let target = lw(params + 92);
            let short = target - best;
            if best < target {
                let mut message = 0;
                if short >= 4 {
                    sw(params + 80, lw(params + 80) & -0x1000001);
                }
                if short >= 3 {
                    sw(params + 80, lw(params + 80) & -0x2000001);
                }
                'announce: {
                    if short == 3 {
                        let flags = lw(params + 80);
                        if (flags & 0x1000000) == 0 {
                            let language = lw(g::cur_language) << 2;
                            sw(params + 80, flags | 0x1000000);
                            message = lw(lw(language + g::languages) + 2832);
                            soundStart(199, 0);
                            break 'announce;
                        }
                    }
                    let params = g::curgameparms;
                    if short == 2 {
                        let flags = lw(params + 80);
                        let language = lw(g::cur_language);
                        if (flags & 0x2000000) == 0 {
                            sw(params + 80, flags | 0x2000000);
                            message = lw(lw((language << 2) + g::languages) + 2836);
                            soundStart(199, 0);
                        }
                    }
                }
                let players = lw(g::lvnumplayers);
                if message != 0 {
                    let mut player = 0;
                    if players > 0 {
                        loop {
                            hudMessage(player, message);
                            player += 1;
                            if player >= lw(g::lvnumplayers) {
                                break 'warn;
                            }
                        }
                    }
                }
            }
        }
    }
    if UnlockCombo() != 0 {
        gameLevelCompleted();
    }
}

/// Preload the files of the background of the level (@ 0x2561e8). For the level 100 (the map maker's level) the tile files used by the loaded map are found by walking the
/// chunks of the temporary buffer (0x1324627 gives the number of tiles, 0x1324628 lists the tile ids of an object) and each used tile gets its object and pad file
/// preloaded; any other level preloads the single background file of the level.
pub fn bgPreload() {
    let level = lw(g::lvlevelnum);
    if level != 100 {
        let buffer = 0x1fc4708;
        sprintf(&[buffer as i64, 0x3a2640, level as i64, level as i64]);
        preloadAddFile(buffer, 0);
        return;
    }
    let base = lw(g::mmtempbuffer);
    memset(0x1fc49b8, 0, 84);
    let mut cursor = base + 8;
    let mut tiles = 0;
    memset(g::tilepaddata, 0, 84);
    let buffer_start = lw(g::mmtempbuffer);
    let mut variants = 0;
    'scan: {
        if (cursor as u32) < ((buffer_start + 0x1ffc) as u32) {
            let mut chunk = cursor;
            let kind = lw(base + 8);
            let mut data = base + 16;
            if kind != 0x1324629 {
                let mut kind = kind;
                loop {
                    if kind == 0x1324627 {
                        variants = lhu(data + 8);
                        tiles = lw(data + 4);
                    } else if kind == 0x1324628 {
                        let mut entry = data;
                        if tiles > 0 {
                            let mut left = tiles;
                            loop {
                                left -= 1;
                                let counter = (lw(entry) << 2) + 0x1fc49b8;
                                entry = entry + 36 + (lhu(entry + 10) << 2);
                                sw(counter, lw(counter) + 1);
                                if left == 0 {
                                    break;
                                }
                            }
                        }
                    }
                    let size = lw(chunk + 4);
                    let next = data + size;
                    if (next as u32) >= ((buffer_start + 0x1ffc) as u32) {
                        break 'scan;
                    }
                    chunk = next;
                    let next_kind = lw(chunk);
                    data = chunk + 8;
                    kind = next_kind;
                    if next_kind == 0x1324629 {
                        break 'scan;
                    }
                }
            }
        }
    }
    let count = variants + 1;
    let pad_names = g::tilepadfilenames;
    let mut tile = 0;
    loop {
        if lw((tile << 2) + 0x1fc49b8) == 0 {
            tile += 1;
        } else {
            let number = tile + 1;
            let offset = tile << 6;
            let name = offset + g::tilefilenames;
            let object_name = vafmt(&[0x3a25f8, count as i64, number as i64, count as i64, number as i64]);
            let format = 0x3a2620;
            strcpy(name, object_name);
            preloadAddOb(name, 0);
            let mut pad_file = vafmt(&[format, count as i64, number as i64]);
            let size = fileSize(pad_file);
            let slot = offset + pad_names;
            if size <= 0 {
                pad_file = vafmt(&[format, 1, number as i64]);
            }
            strcpy(slot, pad_file);
            preloadAddFile(slot, 0);
            tile = number;
        }
        if tile >= 21 {
            break;
        }
    }
}

/// Drop the bag `a2` that the character `a0` was carrying, taken from the holder `a1` (@ 0x26b0f0). When the dropper is on the holder's team and the bag was not secured the
/// team is credited and told; when it is on another team the opposing characters of the holder's team are given the score. The bag moves back to its home position unless
/// the game mode is the bag mode that hides it.
pub fn propDropBagOnDrop(a0: i32, a1: i32, a2: i32) {
    let frame = Frame::new(176);
    let sp = frame.sp();
    let chr = lw(a0 + 3020);
    let holder = lw(a1 + 352);
    let bag = lw(a2 + 352);
    sw(a2 + 16, (lw(a2 + 16) | 2) & -65);
    let mut team = lw(holder + 16);
    let mut secured = 0;
    if lw(a0 + 24) != team {
        let players = lw(g::lvnumplayers);
        if lw(bag + 16) != team {
            let mut credited = 0;
            let mut chars = lw(g::numactivechars);
            let mut i = 0;
            let mut players = players;
            if (players + chars) > 0 {
                loop {
                    let other = i * 0x1210 + lw(g::chrinfo);
                    if lw(other + 24) != team {
                        i += 1;
                    } else {
                        let strings = lw((lw(g::cur_language) << 2) + g::languages);
                        sw(other + 248, lw(other + 248) + 1);
                        credited = 1;
                        hudMessage(i, lw(strings + 0x1210));
                        StatsAdd(39, i, 1.0_f32);
                        players = lw(g::lvnumplayers);
                        chars = lw(g::numactivechars);
                        i += 1;
                    }
                    if i >= players + chars {
                        break;
                    }
                    if credited != 0 {
                        break;
                    }
                    team = lw(holder + 16);
                }
            }
        }
    } else if lw(bag + 16) != team {
        StatsAdd(39, lw(a0), 1.0_f32);
        sw(a0 + 248, lw(a0 + 248) + 1);
        let mut count = lw(g::lvnumplayers);
        let mut id;
        if count <= 0 {
            id = lw(a0);
        } else {
            let mut i = 0;
            loop {
                let players = lw(g::players);
                let offset = i * 1820;
                let player;
                if lw(lw(lw(offset + players + 384) + 352) + 24) != lw(a0 + 24) {
                    player = offset + players;
                } else {
                    hudShowScore(i, 304);
                    player = offset + lw(g::players);
                }
                if lw(lw(lw(player + 384) + 352) + 24) != lw(bag + 16) {
                    count = lw(g::lvnumplayers);
                } else {
                    let language = lw(g::cur_language);
                    let letter = lw(a0 + 24) - 1;
                    sd(sp, ld(0x3af928));
                    let strings = lw((language << 2) + g::languages);
                    let name = lw(strings + 0x11f4);
                    let team_name = lw((lh(sp + (letter << 1)) << 2) + strings);
                    let text = vafmt(&[0x3af920, team_name as i64, name as i64, 0, 0, 0x3af928]);
                    playerMessage(player, text);
                    count = lw(g::lvnumplayers);
                }
                i += 1;
                if i >= count {
                    break;
                }
            }
            id = lw(a0);
        }
        if (id < count) as i32 != 0 {
            hudShowScore(id, 304);
            id = lw(a0);
            hudMessage(id, lw(lw((lw(g::cur_language) << 2) + g::languages) + 0x11f8));
        }
    } else {
        bagSecuredMessage(a0);
        secured = 1;
    }
    soundStart(233, chr + 48);
    if lw(g::curgameparms + 72) != 5 {
        let x = lwc1(bag + 24);
        let flag = lw(bag + 40);
        swc1(a2 + 48, x);
        sw(bag + 36, flag);
        let y = lwc1(bag + 28);
        swc1(a2 + 52, y);
        let z = lwc1(bag + 32);
        swc1(a2 + 112, x);
        swc1(a2 + 116, y);
        swc1(a2 + 120, z);
        swc1(a2 + 56, z);
        if secured == 0 {
            propMakeVisible(a2);
        }
    } else {
        sw(a2 + 16, lw(a2 + 16) & -2);
    }
}
