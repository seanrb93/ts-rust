//! Functions: putting every player back at the start of a level, drawing the statistics of a player at the end of a level, the tick of the character page of the multiplayer game
//! and adding a decal to the sides of a thing it is on.

use crate::*;

/// Put the players back at the start of a level (@ 0x27f808): each player gets a window, the sign-on record to play with, a new character (and, when the level has a sky, a
/// sky prop and, with teams, a flag prop) and has all of its state cleared.
pub fn playerRestart() {
    #[cfg(feature = "extensions")]
    crate::manual::harness::harness_player();
    let mut player = 0;
    if lw(g::lvnumplayers) > 0 {
        loop {
            let players = lw(g::players);
            sw(g::currentplayer, 0);
            let p = players + player * 1820;
            sw(p, player);
            playerSetWindow(p);
            let next = player + 1;
            if ((lw(g::lvlevelnum) - 101) as u32) >= (5 as u32) {
                if (lw(g::curgameparms + 80) & 0x2000) == 0 {
                    signonGet(player);
                    let signon = signonGet(player);
                    sw(p + 24, lw(g::signonarray) + signon * 2936 + 32);
                } else {
                    let prefs = 0x1fe9950;
                    playerprefsNew(prefs);
                    let flags = lhu(prefs + 2);
                    sh(0x1fe9950, 2);
                    sw(p + 24, prefs);
                    sh(prefs + 2, flags & 0xffaf);
                    let header = demoGetHeader();
                    sw(lw(p + 24) + 84, lbu(header + player + 21));
                }
                playerSetControls(p);
                let parms = g::curgameparms;
                let kind = if (lw(parms + 80) & 32) == 0 { lw(lw(p + 24) + 84) } else { lbu(parms + 21) };
                let set = kind * 44 + CHRS_TABLE;
                let chr = chrNew(kind, kind, 0, player, -1, 1, 0.0_f32, 0.0_f32, 0.0_f32, 1.8e+02_f32);
                let flags = lw(chr + 16);
                let all_players = (0x1000000 << (lw(g::lvnumplayers) & 31)) - 1;
                sw(chr + 12, 1);
                let inst = lw(chr + 32);
                sw(chr + 16, flags | all_players & 0xf000000);
                sw(p + 384, chr);
                sw(chr + 356, p);
                obInstSetAnim(inst, 4, 2.0_f32);
                let x = lwc1(set + 20);
                swc1(p + 104, x);
                swc1(p + 108, lwc1(set + 24));
                let y = lwc1(set + 28);
                swc1(p + 124, 0.0_f32);
                let height = ps2float::sub(x, y);
                swc1(p + 112, y);
                swc1(p + 120, height);
                swc1(p + 116, height);
                swc1(p + 132, lwc1(set + 32));
                let z = lwc1(set + 36);
                swc1(p + 1776, 0.0_f32);
                swc1(p + 1780, 0.0_f32);
                swc1(p + 136, z);
                let sky = lvGetSkyPropNum();
                if sky < 0 {
                    sw(p + 388, 0);
                } else {
                    let prop = propNew(sky, 0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32);
                    let flags = lw(prop + 16);
                    let number = lw(p) + 205;
                    sw(p + 388, prop);
                    sw(prop + 12, number);
                    sw(prop + 16, flags | 0x40000000);
                }
                let record = lw(lw(p + 384) + 352);
                let mut count = 32;
                let mut at = record + 616;
                loop {
                    sw(at, 0);
                    count -= 1;
                    at -= 4;
                    if count < 0 {
                        break;
                    }
                }
                sw(record + 260, 0);
                chrResetAmmo(record);
                let chr_now = lw(p + 384);
                sw(record + 256, 0);
                swc1(chr_now + 520, 1.0_f32);
                sw(record + 12, 0);
                sw(record + 28, 0);
                let signon = signonGet(player);
                sw(record + 0x11b0, 0);
                let mode = lw(g::curgameparms + 72) - 1;
                sw(record + 16, lw(g::signonarray) + signon * 2936);
                if (mode as u32) < (7 as u32) {
                    match mode {
                        0 | 2 => {
                            let parms = g::curgameparms;
                            if (lw(parms + 80) & 1) == 0 {
                                sw(record + 24, next);
                                sw(record + 20, lw((player << 2) + g::charcolours));
                            } else {
                                let team = lbu(parms + player + 35);
                                sw(record + 24, team);
                                sw(record + 20, lw((team << 3) + (g::storydata + 64) + 4));
                            }
                        }
                        1 | 3 | 4 => {
                            let team = lbu(player + (g::curgameparms + 35));
                            sw(record + 24, team);
                            sw(record + 20, lw((team << 3) + (g::storydata + 64) + 4));
                        }
                        _ => {
                            sw(record + 24, 1);
                            sw(record + 20, lw(g::teamdata + 4));
                        }
                    }
                }
                if (lw(g::curgameparms + 80) & 1) != 0 {
                    let team = lw(record + 24);
                    let flag_prop = match team {
                        2 => 429,
                        3 => 428,
                        4 => 430,
                        1 => 427,
                        _ => -1,
                    };
                    let chr_now = lw(p + 384);
                    let y = lwc1(chr_now + 56);
                    let x = ps2float::add(lwc1(chr_now + 52), lwc1(0x3acfdc));
                    let w = lwc1(chr_now + 48);
                    let prop = propNew(flag_prop, w, x, y, 0.0_f32);
                    let mask = match player {
                        1 => Some(-0x2010000),
                        2 => Some(-0x4010000),
                        3 => Some(-0x8010000),
                        0 => Some(-0x1010000),
                        _ => None,
                    };
                    if let Some(mask) = mask {
                        let flags = lw(prop + 16);
                        sw(prop + 16, flags & (mask | 0xffff));
                    }
                    let flags = lw(prop + 16);
                    let chr_now = lw(p + 384);
                    sw(prop + 16, flags | 0x40000000);
                    sw(chr_now + 536, prop);
                    sw(prop + 12, lw(chr_now + 12));
                }
                sw(p + 420, 0);
                sw(record + 332, 0);
                sw(p + 548, 0);
                let hud = player * 352 + g::hudparms;
                sw(p + 552, 0);
                sw(hud + 272, 0);
                sw(hud + 268, 0);
                sw(p + 20, 0);
                sw(record + 268, -1);
                sw(p + 416, -1);
                sw(p + 408, 0);
                sw(p + 556, 0);
                sw(p + 612, 0);
                sw(record + 376, -1);
                sw(record + 440, 0);
                sw(p + 412, 0);
                sw(p + 600, 0);
                sw(p + 604, 0);
                sw(p + 748, 0);
                sw(p + 740, 0);
                sw(p + 744, 0);
                sw(p + 608, -1);
                playerClearGuns(p);
                sw(p + 792, 0);
                sw(p + 908, 0);
                sw(record + 0x11a8, 0);
                sw(p + 804, 0);
                sw(p + 808, 0);
                sw(p + 812, 0);
                sw(p + 816, 0);
                sw(record + 0x11a4, 0);
                sw(record + 0x11ac, 0);
            } else {
                sw(p + 104, 0);
                sw(p + 384, 0);
                let zero = lwc1(p + 104);
                sw(p + 388, 0);
                swc1(p + 364, zero);
                sw(p + 420, 0);
                sw(p + 548, 0);
                sw(p + 552, 0);
                sw(p + 612, 0);
                sw(p + 740, 0);
                sw(p + 744, 0);
                sw(p + 108, 0);
                sw(p + 112, 0);
                sw(p + 116, 0);
                sw(p + 120, 0);
                sw(p + 124, 0);
                sw(p + 132, 0);
                sw(p + 136, 0);
                sw(p + 140, 0);
                sw(p + 144, 0);
                sw(p + 148, 0);
                swc1(p + 152, zero);
                swc1(p + 156, zero);
                swc1(p + 160, zero);
                swc1(p + 344, zero);
                swc1(p + 348, zero);
                swc1(p + 352, zero);
                swc1(p + 356, zero);
                swc1(p + 360, zero);
                sw(p + 24, 0);
            }
            let one = 1.0_f32;
            let sixty = 6e+01_f32;
            let chr_now = lw(p + 384);
            swc1(p + 272, sixty);
            sw(p + 392, 0);
            sw(p + 396, 0);
            sw(p + 400, 0);
            sw(p + 404, 0);
            sw(p + 164, 0);
            swc1(p + 168, one);
            sw(p + 172, 0);
            sw(p + 228, 0);
            swc1(p + 232, one);
            sw(p + 236, 0);
            swc1(p + 240, one);
            sw(p + 244, 0);
            sw(p + 248, 0);
            swc1(p + 252, one);
            swc1(p + 256, one);
            swc1(p + 260, sixty);
            swc1(p + 264, sixty);
            swc1(p + 268, sixty);
            sw(p + 276, 0);
            sw(p + 280, 0);
            sw(p + 284, 0);
            sw(p + 288, 0);
            sw(p + 296, 0);
            sw(p + 300, 0);
            sw(p + 304, 0);
            sw(p + 308, 0);
            if chr_now != 0 {
                let record = lw(chr_now + 352);
                sw(record + 680, -1);
                sw(record + 2864, 0);
            }
            let mut count = 107;
            let table = lw(p + 820);
            swc1(p + 316, lwc1(0x3acfe0));
            swc1(p + 848, one);
            let mut at = table + 1284;
            swc1(p + 320, one);
            swc1(p + 312, one);
            swc1(p + 184, one);
            swc1(p + 196, one);
            swc1(p + 208, one);
            swc1(p + 220, one);
            sw(p + 332, 0);
            sw(p + 336, 0);
            sw(p + 324, 0);
            sw(p + 328, 0);
            sw(p + 176, 0);
            sw(p + 180, 0);
            sw(p + 188, 0);
            sw(p + 192, 0);
            sw(p + 200, 0);
            sw(p + 204, 0);
            sw(p + 212, 0);
            sw(p + 216, 0);
            sw(p + 1784, 0);
            sw(p + 1788, 0);
            sw(p + 828, 0);
            sw(p + 832, 0);
            sw(p + 836, 0);
            sw(p + 840, 0);
            sw(p + 844, 0);
            sw(p + 852, 0);
            sw(p + 824, 0);
            let eye_x = lwc1(p + 152);
            let eye_y = lwc1(p + 156);
            let eye_z = lwc1(p + 160);
            loop {
                sw(at, 0);
                count -= 1;
                at -= 12;
                if count < 0 {
                    break;
                }
            }
            let view = lw(p + 796);
            sw(p + 904, 0);
            sw(p + 128, 0);
            swc1(p + 884, eye_x);
            swc1(p + 888, eye_y);
            swc1(p + 892, eye_z);
            sw(p + 900, view);
            sw(p + 896, view);
            let third_person = bossCheckParm(0x3a87e0);
            let boss_mode = lw(g::bosscammode);
            if third_person == 0 {
                if boss_mode != 0 {
                    camSetMode(boss_mode);
                    let prop = propGetNum(lw(g::bosscampropnum));
                    sw(p + 904, prop);
                }
            } else {
                camSetMode(1);
            }
            player = next;
            sw(p + 1768, 0);
            sw(p + 1760, 0);
            sw(p + 1764, 0);
            if player >= lw(g::lvnumplayers) {
                break;
            }
        }
    }
    sw(g::currentplayer, lw(g::players));
    playerAutoAimRestart();
}

/// Draw the statistics of the current player at the end of a level (@ 0x20fca0): the lines of the kills and deaths (or, in the mode 2, the lines of the numbers of the stories
/// that are there), each with its figure at the right of the window, and then the percentages of the shots that hit and the accuracy.
pub fn drawStats() {
    let frame = Frame::new(192);
    let sp = frame.sp();
    let local = ilinkGetNumLocalPlayers();
    let saved = textSelect(((local < 2) as i32) ^ 1);
    sw(sp, saved);
    let left_edge = lw(g::window);
    let window = g::window;
    let local = ilinkGetNumLocalPlayers();
    let right_edge = lw(window + 4);
    let x = if local >= 3 { left_edge + 14 } else { left_edge + 120 };
    let local = ilinkGetNumLocalPlayers();
    let top = lw(window + 20);
    let narrow = lw(window + 92);
    let right = if local >= 3 { right_edge - 10 } else { right_edge - 120 };
    let line = if narrow == 0 { 18 } else { 12 };
    let first = if narrow != 0 { top + 20 } else { top + 38 };
    let mut y = if narrow != 0 { first + 3 } else { first + line };
    StatsCalc();
    let language = lw(g::cur_language);
    let languages = g::languages;
    let colour = 0x7f7f7f80;
    if lw(g::curgameparms + 72) != 2 {
        y += line;
        textOutline(x, y, lw(lw((language << 2) + languages) + 0x1150), colour, 80);
        let value = StatsGet(5, lw(lw(g::currentplayer)));
        let text = vafmt(&[(g::lastgfxendtime + 4) as i64, (value as i32) as i64]);
        textOutline(right - textWidth(text), y, text, colour, 80);
        y += line;
        let local = ilinkGetNumLocalPlayers();
        let language = lw(g::cur_language);
        let label = if local < 3 { lw(lw((language << 2) + languages) + 0x1158) } else { lw(lw((language << 2) + languages) + 0x1154) };
        textOutline(x, y, label, colour, 80);
        let format = g::lastgfxendtime + 4;
        let value = StatsGet(6, lw(lw(g::currentplayer)));
        let text = vafmt(&[format as i64, (value as i32) as i64]);
        textOutline(right - textWidth(text), y, text, colour, 80);
        y += line;
        textOutline(x, y, lw(lw((lw(g::cur_language) << 2) + g::languages) + 0x115c), colour, 80);
        let value = StatsGet(7, lw(lw(g::currentplayer)));
        let text = vafmt(&[format as i64, (value as i32) as i64]);
        textOutline(right - textWidth(text), y, text, colour, 80);
    } else {
        y += line;
        textOutline(x, y, lw(lw((language << 2) + languages) + 0x112c), colour, 80);
        let value = StatsGet(39, lw(lw(g::currentplayer)));
        let text;
        if (value as i32) != 1 {
            let value = StatsGet(39, lw(lw(g::currentplayer)));
            text = vafmt(&[0x3ae518, (value as i32) as i64, lw(lw((lw(g::cur_language) << 2) + languages) + 0x1134) as i64]);
        } else {
            let value = StatsGet(39, lw(lw(g::currentplayer)));
            text = vafmt(&[0x3ae518, (value as i32) as i64, lw(lw((lw(g::cur_language) << 2) + languages) + 0x1130) as i64]);
        }
        textOutline(right - textWidth(text), y, text, colour, 80);
        y += line;
        textOutline(x, y, lw(lw((lw(g::cur_language) << 2) + languages) + 0x1138), colour, 80);
        let value = StatsGet(37, lw(lw(g::currentplayer)));
        let text;
        if (value as i32) != 1 {
            let value = StatsGet(37, lw(lw(g::currentplayer)));
            text = vafmt(&[0x3ae518, (value as i32) as i64, lw(lw((lw(g::cur_language) << 2) + languages) + 0x1140) as i64]);
        } else {
            let value = StatsGet(37, lw(lw(g::currentplayer)));
            text = vafmt(&[0x3ae518, (value as i32) as i64, lw(lw((lw(g::cur_language) << 2) + languages) + 0x113c) as i64]);
        }
        textOutline(right - textWidth(text), y, text, colour, 80);
        y += line;
        textOutline(x, y, lw(lw((lw(g::cur_language) << 2) + languages) + 0x1144), colour, 80);
        let value = StatsGet(38, lw(lw(g::currentplayer)));
        let text;
        if (value as i32) != 1 {
            let value = StatsGet(38, lw(lw(g::currentplayer)));
            text = vafmt(&[0x3ae518, (value as i32) as i64, lw(lw((lw(g::cur_language) << 2) + languages) + 0x114c) as i64]);
        } else {
            let value = StatsGet(38, lw(lw(g::currentplayer)));
            text = vafmt(&[0x3ae518, (value as i32) as i64, lw(lw((lw(g::cur_language) << 2) + languages) + 0x1148) as i64]);
        }
        textOutline(right - textWidth(text), y, text, colour, 80);
    }
    y += line;
    let accuracy = StatsGet(10, lw(lw(g::currentplayer)));
    let percent = ps2float::mul(accuracy, 1e+02_f32);
    textOutline(x, y, lw(lw((lw(g::cur_language) << 2) + languages) + 0x1160), colour, 80);
    let whole = percent as i32;
    let shots = StatsGet(8, lw(lw(g::currentplayer)));
    let text;
    if !(1.0_f32 <= shots) {
        text = lw(lw((lw(g::cur_language) << 2) + languages) + 0x1164);
    } else if whole != 0 {
        text = vafmt(&[0x3ae528, whole as i64]);
    } else {
        text = vafmt(&[0x3ae520, fptodp(percent)]);
    }
    textOutline(right - textWidth(text), y, text, colour, 80);
    y += line;
    let hits = StatsGet(18, lw(lw(g::currentplayer)));
    textOutline(x, y, lw(lw((lw(g::cur_language) << 2) + languages) + 0x1168), colour, 80);
    let total = StatsGet(13, lw(lw(g::currentplayer)));
    let text;
    if !(1.0_f32 <= total) {
        text = lw(lw((lw(g::cur_language) << 2) + languages) + 0x116c);
    } else {
        text = vafmt(&[0x3ae528, ps2float::mul(hits, 1e+02_f32) as i32 as i64]);
    }
    textOutline(right - textWidth(text), y, text, colour, 80);
    textSelect(lw(sp));
}

/// The tick of the character page of the multiplayer game (@ 0x237600) for the page `a0`. The player of the page (`lw(front)`) goes from the main menu (state 2) to the
/// handicap menu (state 3), the team menu (state 4) and then waits to be ready (state 5); the game starts when every player is ready and the first player pushes start.
pub fn chrplayer_pageTick(a0: i32) {
    #[cfg(feature = "extensions")]
        char_randomise(a0);
    let mut menu = 0;
    let signon = signonGet(lw(lw(g::front)));
    let record = lw(g::signonarray) + signon * 2936 + 32;
    fxSelectCharTick(0);
    animTick(a0);
    let front = lw(g::front);
    let state = lbu(lw(front) + 0x3aff28) - 2;
    let mut cleared = 0;
    'done: {
        if (state as u32) < (5 as u32) {
            'b2: {
                'b3: {
                    'b4: {
                        'b5: {
                            'b6: {
                                'b7: {
                                    match state {
                                        0 => {
                                            sw((lw(front) << 2) + 0x1fc2c88, -1);
                                            let clear_player;
                                            if lw(front) != 0 {
                                                clear_player = lw(front);
                                            } else {
                                                let pressed = joyNewBut(0);
                                                let now = lw(g::front);
                                                if (pressed & 16) != 0 {
                                                    pageOut();
                                                    return;
                                                }
                                                clear_player = lw(now);
                                            }
                                            let mut now = 0;
                                            let mut halfword = 0;
                                            let mut owner = 0;
                                            'b9: {
                                                'b10: {
                                                    joyClearButs(clear_player, 16);
                                                    let pressed = joyNewBut(lw(lw(g::front)));
                                                    now = lw(g::front);
                                                    if (pressed & 2112) != 0 {
                                                        halfword = lhu(a0 + 4);
                                                    } else {
                                                        halfword = lhu(a0 + 4);
                                                        if lw((lw(now) << 2) + 0x1fc12a8) == 0 {
                                                            break 'b10;
                                                        }
                                                    }
                                                    let shown = halfword & 0xffff;
                                                    owner = lw(now);
                                                    if shown != 1 {
                                                        break 'b9;
                                                    }
                                                    let slot = owner << 2;
                                                    let player_now;
                                                    if lw(slot + 0x1fc1288) != lw(slot + 0x1fc1298) {
                                                        player_now = lw(now);
                                                    } else {
                                                        sw(slot + 0x1fc12a8, shown);
                                                        player_now = lw(now);
                                                    }
                                                    if lw((player_now << 2) + 0x1fc1268) == 0 {
                                                        fxSelectCharFlashInit(player_now);
                                                        let front_now = lw(g::front);
                                                        let handicaps = lw(g::gv_handicaps);
                                                        sw((lw(front_now) << 2) + 0x1fc12a8, 0);
                                                        if handicaps == 0 {
                                                            break 'b7;
                                                        }
                                                        menu = g::chrselect_handicap_menu;
                                                        sb(lw(front_now) + 0x3aff28, 3);
                                                        joyClearAll(0);
                                                        break 'done;
                                                    }
                                                }
                                                if halfword != 1 {
                                                    owner = lw(now);
                                                } else {
                                                    let character = lw(record + 84);
                                                    if lbu(g::characterorder) != character {
                                                        let mut n = 1;
                                                        loop {
                                                            if (n as u32) >= (64 as u32) {
                                                                break;
                                                            }
                                                            if lbu(n + g::characterorder) == character {
                                                                break;
                                                            }
                                                            n += 1;
                                                        }
                                                    }
                                                    'b15: {
                                                        let front_now;
                                                        'b16: {
                                                            if (lw(now + 12) & 16) != 0 {
                                                                if lw((lw(now) << 2) + 0x1fc1268) != -1 {
                                                                    soundStart(140, 0);
                                                                    fxSelectCharLeft(0);
                                                                    front_now = lw(g::front);
                                                                    break 'b16;
                                                                }
                                                            }
                                                            if (lw(now + 12) & 32) == 0 {
                                                                break 'b15;
                                                            }
                                                            if lw((lw(now) << 2) + 0x1fc1268) == 1 {
                                                                owner = lw(now);
                                                                break 'b9;
                                                            }
                                                            soundStart(140, 0);
                                                            fxSelectCharRight(0);
                                                            front_now = lw(g::front);
                                                        }
                                                        fxSelectCharInit(lw(front_now), lw(record + 84));
                                                        now = lw(g::front);
                                                    }
                                                    owner = lw(now);
                                                }
                                            }
                                            if owner <= 0 {
                                                menu = g::chrselect_main_menu;
                                                break 'done;
                                            }
                                            menu = g::chrselect_main_menu_p234;
                                            break 'done;
                                        }
                                        1 => {
                                            menu = g::chrselect_handicap_menu;
                                            if (lw(front + 12) & 48) != 0 {
                                                let mut handicap = lw(record + 92);
                                                soundStart(140, 0);
                                                let buttons = lw(lw(g::front) + 12);
                                                if (buttons & 16) == 0 {
                                                    if (buttons & 32) != 0 {
                                                        soundStart(140, 0);
                                                        let up = handicap + 1;
                                                        handicap = if up >= 7 { 0 } else { up };
                                                    }
                                                } else {
                                                    let down = handicap - 1;
                                                    handicap = if -1 >= down { 6 } else { down };
                                                }
                                                if lw(record + 92) != handicap {
                                                    sw(record + 92, handicap);
                                                    break 'done;
                                                }
                                                break 'done;
                                            }
                                            let pressed = joyNewBut(lw(front));
                                            if (pressed & 2112) == 0 {
                                                let pressed = joyNewBut(lw(lw(g::front)));
                                                if (pressed & 16) == 0 {
                                                    break 'done;
                                                }
                                                let front_now = lw(g::front);
                                                cleared = front_now;
                                                break 'b2;
                                            }
                                            soundStart(140, 0);
                                        }
                                        2 => {
                                            let mut front_now = front;
                                            let mut base = 0;
                                            'l19: loop {
                                                let mut player_now = lw(front_now);
                                                'b22: {
                                                    loop {
                                                        base = 0x1fc2c88;
                                                        if selectableTeam(lw((player_now << 2) + base) + 1) != 0 {
                                                            break;
                                                        }
                                                        front_now = lw(g::front);
                                                        let at = (lw(front_now) << 2) + base;
                                                        let team = lw(at) + 1;
                                                        sw(at, team);
                                                        player_now = lw(front_now);
                                                        if team >= 4 {
                                                            break 'b22;
                                                        }
                                                    }
                                                    'b23: {
                                                        front_now = lw(g::front);
                                                        if lhu(a0 + 4) == 1 {
                                                            let buttons = lw(front_now + 12);
                                                            if (buttons & 16) == 0 {
                                                                if (buttons & 32) != 0 {
                                                                    soundStart(140, 0);
                                                                    front_now = lw(g::front);
                                                                    loop {
                                                                        let at = (lw(front_now) << 2) + base;
                                                                        let team = lw(at) + 1;
                                                                        sw(at, team);
                                                                        let mut chosen = lw(front_now);
                                                                        if team >= 4 {
                                                                            sw((chosen << 2) + base, 0);
                                                                            chosen = lw(front_now);
                                                                        }
                                                                        let selectable = selectableTeam(lw((chosen << 2) + base) + 1);
                                                                        front_now = lw(g::front);
                                                                        if selectable != 0 {
                                                                            break 'b23;
                                                                        }
                                                                    }
                                                                }
                                                            } else {
                                                                soundStart(140, 0);
                                                                front_now = lw(g::front);
                                                                loop {
                                                                    let at = (lw(front_now) << 2) + base;
                                                                    let team = lw(at) - 1;
                                                                    sw(at, team);
                                                                    if team < 0 {
                                                                        sw((lw(front_now) << 2) + base, 3);
                                                                    }
                                                                    let selectable = selectableTeam(lw((lw(front_now) << 2) + base) + 1);
                                                                    front_now = lw(g::front);
                                                                    if selectable != 0 {
                                                                        break 'b23;
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                    let pressed = joyNewBut(lw(front_now));
                                                    if (pressed & 2112) == 0 {
                                                        let pressed = joyNewBut(lw(lw(g::front)));
                                                        if (pressed & 16) == 0 {
                                                            menu = g::chrselect_team_menu;
                                                            break 'done;
                                                        }
                                                        break 'b4;
                                                    }
                                                    soundStart(140, 0);
                                                    let front_now = lw(g::front);
                                                    sw(record + 80, lw((lw(front_now) << 2) + 0x1fc2c88));
                                                    sb(lw(front_now) + 0x3aff28, 5);
                                                    cleared = lw(front_now);
                                                    break 'b6;
                                                }
                                                sw((player_now << 2) + base, 0);
                                                continue 'l19;
                                            }
                                        }
                                        3 => {
                                            break 'b5;
                                        }
                                        _ => {
                                            break 'done;
                                        }
                                    }
                                }
                                let mut front_now = lw(g::front);
                                if lw(g::gv_teamplay) != 0 {
                                    let only = onlyoneselectableteam();
                                    front_now = lw(g::front);
                                    if only == 0 {
                                        let now = lw(g::front);
                                        menu = g::chrselect_team_menu;
                                        sb(lw(now) + 0x3aff28, 4);
                                        sh(a0 + 4, 1);
                                        sw((lw(now) << 2) + 0x1fc2c88, lw(record + 80));
                                        break 'b3;
                                    }
                                    sw((lw(front_now) << 2) + 0x1fc2c88, only - 1);
                                }
                                sb(lw(front_now) + 0x3aff28, 5);
                                cleared = lw(front_now);
                            }
                            joyClearAll(cleared);
                        }
                        let front_now = lw(g::front);
                        sh(a0 + 4, 0);
                        let pressed = joyNewBut(lw(front_now));
                        if (pressed & 16) == 0 {
                            menu = g::chrselect_ready_waiting;
                            if lw(lw(g::front)) != 0 {
                                break 'done;
                            }
                            let players = lw(0x3aff20);
                            let mut ready = 0;
                            'b29: {
                                if players > 0 {
                                    if lbu(0x3aff28) == 5 {
                                        ready += 1;
                                        loop {
                                            if ready >= players {
                                                break 'b29;
                                            }
                                            if lbu(ready + 0x3aff28) != 5 {
                                                break 'b29;
                                            }
                                            ready += 1;
                                        }
                                    }
                                }
                            }
                            if ready != players {
                                break 'done;
                            }
                            let pressed = joyNewBut(0);
                            if (pressed & 2112) != 0 {
                                soundStart(143, 0);
                                animStopImmediately();
                                frontmpgStartGame();
                                setStates(6);
                            }
                            menu = g::chrselect_ready_go;
                            break 'done;
                        }
                        if lw(g::gv_teamplay) != 0 {
                            let only = onlyoneselectableteam();
                            if only != 0 {
                                break 'b2;
                            }
                            soundStart(142, 0);
                            let front_now = lw(g::front);
                            sb(lw(front_now) + 0x3aff28, 4);
                            sh(a0 + 4, 1);
                            joyClearButs(lw(front_now), 16);
                            return;
                        }
                    }
                    if lw(g::gv_handicaps) == 0 {
                        break 'b2;
                    }
                    soundStart(142, 0);
                    let front_now = lw(g::front);
                    menu = g::chrselect_handicap_menu;
                    sb(lw(front_now) + 0x3aff28, 3);
                    joyClearAll(lw(front_now));
                    break 'done;
                }
                let front_now = lw(g::front);
                joyClearAll(lw(front_now));
                break 'done;
            }
            let front_now = lw(g::front);
            sb(lw(front_now) + 0x3aff28, 2);
            sh(a0 + 4, 1);
            fxSelectCharFlashInit(lw(front_now));
            actingsoundStop(lw(lw(g::front)));
            return;
        }
    }
    let players = lw(0x3aff20);
    if menu != 0 {
        if players == 1 {
            sh(menu + 6, 170);
        } else if players == 2 {
            sh(menu + 6, 300);
        } else {
            sh(menu + 6, 150);
        }
        menutick_230238(a0, menu);
        let made = std_menumake(lw(a0 + 8), menu);
        sw(a0 + 12, made);
    }
}

/// One of the side tests of `decalNewDecal`: a line of the size 0.5 in the direction `direction` is tested, and when it hits a thing that has not been hit by the earlier
/// lines (`found` things, in the lists at sp+64, +96, +128 and +160) a decal is added there. Returns the new number of things hit.
fn decal_side(sp: i32, direction: [f32; 3], flag: i32, found: i32, last: bool) -> i32 {
    sw(sp + 16, 0);
    sw(sp + 20, 0);
    sw(sp + 24, 0);
    for (i, value) in direction.iter().enumerate() {
        if *value != 0.0_f32 {
            swc1(sp + 16 + i as i32 * 4, *value);
        }
    }
    sw(sp, 0);
    sw(sp + 8, 0);
    let hit = bgLineTest(lw(sp + 196), lw(sp + 200), sp + 16, 0, lw(sp + 204), lw(sp + 208), 0, 56);
    let offset = found << 2;
    let mut found = found;
    if hit != 0 {
        let part = sp + 96 + offset;
        let chunk = sp + 128 + offset;
        let poly = sp + 160 + offset;
        let inst = obGetLastHitInst(part, chunk, poly);
        sw(sp + flag, 1);
        sw(sp + 64 + offset, inst);
        if found > 0 {
            let mut n = 0;
            let mut poly_at = sp + 160;
            let mut chunk_at = sp + 128;
            let mut part_at = sp + 96;
            let mut inst_at = sp + 64;
            loop {
                n += 1;
                if inst == lw(inst_at) {
                    if lw(part) == lw(part_at) {
                        if lw(chunk) == lw(chunk_at) {
                            if lw(poly) == lw(poly_at) {
                                sw(sp + flag, 0);
                                break;
                            }
                        }
                    }
                }
                poly_at += 4;
                chunk_at += 4;
                part_at += 4;
                inst_at += 4;
                if n >= found {
                    break;
                }
            }
        }
        if lw(sp + flag) != 0 {
            if !last {
                found += 1;
            }
            decalNewDecalMain(lw(sp + 192), hit, lw(sp + 204), lw(sp + 208));
        }
    }
    found
}

/// Add a decal of the kind `a0` on the thing `a1` hit at the place `a3` (@ 0x2a4028): the kinds from 8 to 18 are put on every side of the place where something is (at most
/// 7 times, but never twice on the same part of the same thing); `a2` makes the place move a little along the direction `a4` first. Other kinds go straight to
/// `decalNewDecalMain`.
pub fn decalNewDecal(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32) {
    let frame = Frame::new(400);
    let sp = frame.sp();
    let mut side_kind = 0;
    sw(sp + 192, a0);
    sw(sp + 196, a1);
    sw(sp + 200, a3);
    if a0 < 19 {
        side_kind = ((a0 < 8) as i32) ^ 1;
    }
    if side_kind == 0 {
        decalNewDecalMain(lw(sp + 192), a2, lw(sp + 200), a4);
        return;
    }
    let mut found = 0;
    if a2 != 0 {
        let scale = lwc1(0x3ad5d8);
        let place = lw(sp + 200);
        let y = lwc1(place + 4);
        let z = lwc1(place + 8);
        swc1(place, ps2float::add(lwc1(place), ps2float::mul(lwc1(a4), scale)));
        swc1(place + 4, ps2float::add(y, ps2float::mul(lwc1(a4 + 4), scale)));
        swc1(place + 8, ps2float::add(z, ps2float::mul(lwc1(a4 + 8), scale)));
    }
    sw(sp + 204, sp + 32);
    sw(sp + 208, sp + 48);
    sw(sp + 20, 0);
    swc1(sp + 16, 0.5_f32);
    sw(sp + 24, 0);
    sw(sp, 0);
    sw(sp + 8, 0);
    let hit = bgLineTest(lw(sp + 196), lw(sp + 200), sp + 16, 0, sp + 32, sp + 48, 0, 56);
    if hit != 0 {
        let inst = obGetLastHitInst(sp + 96, sp + 128, sp + 160);
        sw(sp + 64, inst);
        found = 1;
        decalNewDecalMain(lw(sp + 192), hit, lw(sp + 204), lw(sp + 208));
    }
    found = decal_side(sp, [-0.5_f32, 0.0_f32, 0.0_f32], 212, found, false);
    found = decal_side(sp, [0.0_f32, 0.5_f32, 0.0_f32], 216, found, false);
    found = decal_side(sp, [0.0_f32, -0.5_f32, 0.0_f32], 220, found, false);
    found = decal_side(sp, [0.0_f32, 0.0_f32, 0.5_f32], 224, found, false);
    decal_side(sp, [0.0_f32, 0.0_f32, -0.5_f32], 228, found, true);
}
