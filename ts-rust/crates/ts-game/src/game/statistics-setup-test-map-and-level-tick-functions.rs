//! Functions: the statistics, set-up, test map and challenge page ticks, the main in-game menu page tick, the crate's damage, the bag tag game, the room lights, the
//! ticks after the level's own and the glass hits of the background.

use super::particle_spawners::random_as_float;
use crate::*;

/// The tick of the statistics page: the players' entries are made in the table at 0x1fc31b0, the labels of the three menu items are taken from the language, the player item's
/// range follows the number of players, and the menu ticks and its items are made (@ 0x23db20).
pub fn statistics_pageTick(a0: i32) {
    let frame = Frame::new(176);
    let sp = frame.sp();
    let language_at = (lw(g::cur_language) << 2) + g::languages;
    sw(((lw(0x3aff2c) - 1) << 3) + 0x1fc1308, 0);
    let mut i = 0;
    pageSet(lw(lw(language_at) + 1228), g::statistics_page);
    frontfxTick(5);
    let mut players = lw(0x3aff20);
    sw(g::gv_statsplayer + 8, players - 1);
    let mut base = 33292288_u128;
    let language;
    if players <= 0 {
        language = lw(g::cur_language);
    } else {
        loop {
            let offset = i << 3;
            let table = 0x1fc31b0;
            let number = i;
            sw(offset + table, number);
            sq(sp, base);
            i += 1;
            let slot = signonGet(number);
            players = lw(0x3aff20);
            sw(table + offset + 4, lw(g::signonarray) + slot * 2936);
            base = lq(sp);
            if i >= players {
                break;
            }
        }
        language = lw(g::cur_language);
    }
    let table = 0x1fc31b0;
    let offset = i << 3;
    let texts = lw((language << 2) + g::languages);
    sw(table + offset + 4, 0);
    let items = 0x1fc31d8;
    let first = lw(texts + 1232);
    let in_range = (lw(g::gv_statsplayer) < players) as i32;
    sw(offset + table, 0);
    sw(items + 4, first);
    sw(0x1fc31d8, 0);
    let second_items = base as i32 + 0x3200;
    let second = lw(texts + 1236);
    sw(items + 24, 3);
    sw(items + 12, second);
    sw(items + 32, 4);
    sw(base as i32 + 0x3200, 0);
    let third = lw(texts + 1240);
    sw(second_items + 8, 1);
    sw(items + 20, third);
    sw(second_items + 16, 2);
    let fourth = lw(texts + 1244);
    sw(items + 36, 0);
    sw(items + 28, fourth);
    sw(second_items + 4, 0x3ae880);
    sw(second_items + 12, 0x3ae888);
    sw(second_items + 20, 0);
    sw(items + 8, 1);
    sw(items + 16, 2);
    if in_range == 0 {
        sw(g::gv_statsplayer, 0);
    }
    let menu = g::statistics_menu;
    sw(g::gv_statscat + 28, 570);
    sw(g::gv_statsplayer + 28, 570);
    sw(g::gv_statstype + 28, 570);
    menutick_230238(a0, menu);
    let made = std_menumake(lw(a0 + 8), menu);
    sw(a0 + 12, made);
}

/// A crate is damaged: when it has been hurt (+532 above 1) and has the model flag 2 it breaks into the first stage (flag 4), with an explosion and burst; above 2 and with the
/// flag 4 it breaks again (flag 8, and the prop flag 32 dropped) (@ 0x274998).
pub fn crateDamage(a0: i32, a1: i32, _a2: i32, a3: i32, a4: i32, a5: i32) {
    let frame = Frame::new(144);
    let sp = frame.sp();
    let mut health = lwc1(a0 + 532);
    if 1.0_f32 < health {
        if (lhu(lw(a0 + 32) + 292) & 2) != 0 {
            let z = lwc1(a0 + 516);
            let base_z = lwc1(a0 + 56);
            let y = lwc1(a0 + 512) + lwc1(a0 + 52);
            swc1(sp, lwc1(a0 + 508) + lwc1(a0 + 48));
            swc1(sp + 4, y);
            swc1(sp + 8, z + base_z);
            particleNewExplosion(0, 0, sp, a3, a4, a5, a1, 0);
            particleNewBurst(0, 15, sp, 0, 0.0_f32);
            let model = lw(a0 + 32);
            sh(model + 292, lhu(model + 292) & 0xfffd | 4);
            decalRemoveDecalsFromProp(a0);
            let roll = newrnd();
            soundStart(((roll as i32 as u32) % 6) as i32 + 98, sp);
            health = lwc1(a0 + 532);
        }
    }
    if 2.0_f32 < health {
        if (lhu(lw(a0 + 32) + 292) & 4) != 0 {
            let z = lwc1(a0 + 516);
            let base_z = lwc1(a0 + 56);
            let y = lwc1(a0 + 512) + lwc1(a0 + 52);
            swc1(sp + 16, lwc1(a0 + 508) + lwc1(a0 + 48));
            swc1(sp + 20, y);
            swc1(sp + 24, z + base_z);
            particleNewExplosion(0, 0, sp + 16, a3, a4, a5, a1, 0);
            particleNewBurst(0, 15, sp + 16, 0, 0.0_f32);
            let model = lw(a0 + 32);
            let flags = lhu(model + 292) & 0xfffb;
            sw(a0 + 16, lw(a0 + 16) & -33);
            sh(model + 292, flags | 8);
            decalRemoveDecalsFromProp(a0);
            let roll = newrnd();
            soundStart(((roll as i32 as u32) % 6) as i32 + 98, sp + 16);
        }
    }
}

/// The tick of the set-up page: the settings are read; the button 64 on the controls item (when the config is 0) opens the controls page and keeps a copy of the pad's settings
/// (68 bytes) for the player; the menu's row size follows the window; for three or more local players the items are made smaller; then the menu ticks and its items are made
/// (@ 0x20f5e0).
pub fn setup_pageTick(a0: i32) {
    setup_vars();
    if (joyNewBut(lw(lw(g::currentplayer))) & 64) != 0 {
        if lhu(a0 + 4) == 1 {
            if lw(g::gv_config) == 0 {
                inPage(lw(lw(g::currentplayer)), g::setupcontrols_page);
                let player = lw(g::currentplayer);
                let pad = lw(player + 24);
                let mut from = pad + 4;
                let mut to = lw(player) * 76 + 0x1fb33b0;
                let end = pad + 68;
                loop {
                    let b = ld(from + 8);
                    let c = ld(from + 16);
                    let d = ld(from + 24);
                    sd(to, ld(from));
                    sd(to + 8, b);
                    sd(to + 16, c);
                    sd(to + 24, d);
                    from += 32;
                    to += 32;
                    if from == end {
                        break;
                    }
                }
                let tail = lw(from + 8);
                sd(to, ld(from));
                sw(to + 8, tail);
            }
        }
    }
    let window = g::window;
    let shift = lw(window + 88);
    let rows = if lw(window + 12) < 500 { 24 } else { 128 };
    sh(g::setup_menu + 4, rows << (shift & 31));
    if ilinkGetNumLocalPlayers() >= 3 {
        let value = 16 >> (lw(g::window + 88) & 31);
        sw(g::gv_config + 36, value);
        sw(g::gv_lookahead + 36, value);
        sw(g::gv_invlook + 36, value);
        sw(g::gv_aimmode + 36, value);
        sw(g::gv_crouchmode + 36, value);
    }
    let menu = g::setup_menu;
    ingame_menutick(a0, menu);
    let made = std_menumake(lw(a0 + 8), menu);
    sw(a0 + 12, lw(a0 + 12) + made);
}

/// The tick of the map maker's test page: the four game modes are named and marked available by the map; the button 64 on a mode that is available starts the map in that
/// game (saved first, with the bots for the mode and the mode flags), the buttons 2304 leave the menu (@ 0x24cfd8).
pub fn testmap_pageTick(a0: i32) {
    let frame = Frame::new(176);
    let sp = frame.sp();
    let modes = mmCheckGameModes(0);
    let texts = lw((lw(g::cur_language) << 2) + g::languages);
    let mut item = g::m0400 + 32;
    let mut i = 0;
    loop {
        let mode = lbu(i + g::gamemodelist);
        let flag_at = i + 0x3b0040;
        i += 1;
        let text = lw((lh(mode * 12 + g::gamemodedata) << 2) + texts);
        sb(flag_at, ((0 as u32) < ((modes & lw(item + 12)) as u32)) as i32);
        sw(item, text);
        item += 32;
        if (i as u32) >= 4 {
            break;
        }
    }
    if (joyNewBut(0) & 2304) == 0 {
        if (joyNewBut(0) & 64) != 0 {
            let selected = lhu(a0 + 4);
            let mode = selected - 2;
            if (mode as u32) < 4 {
                if lbu(mode + 0x3b0040) == 0 {
                    mmCheckGameModes(lw(((selected - 1) << 5) + (g::m0400 + 12)));
                } else {
                    memset(sp, 0, 112);
                    let buffer = lw(g::mmtempbuffer);
                    sw(g::editflags, lw(g::editflags) | 512);
                    mmSaveMapFile(buffer, 1);
                    let bots = lbu(lhu(a0 + 4) + (g::gv_numbots + 38));
                    sw(sp + 64, 100);
                    sw(sp + 80, 0x40002);
                    sw(sp, 1);
                    sw(sp + 8, 0);
                    sw(sp + 72, bots);
                    sw(sp + 76, 1);
                    sw(sp + 88, 0);
                    sw(sp + 92, 0);
                    if bots == 2 || bots == 5 {
                        sw(sp + 80, 0x40003);
                        sb(sp + 17, 2);
                        sb(sp + 18, 3);
                        sb(sp + 19, 4);
                        sb(sp + 35, 1);
                        sw(sp + 12, 4);
                        sb(sp + 16, 1);
                    }
                    gameDeferedReset(sp);
                }
            }
        }
        let menu = g::testmap_menu;
        menutick_24c3a8(a0, menu);
        let made = menumake(lw(a0 + 8), menu);
        sw(a0 + 12, made);
    } else {
        mmExitMenu(a0);
    }
}

/// The tick of the bag tag game: the holder of the bag is the first character that has one; when it changes a message is made for the new holder and sent to the other players,
/// the holder's score grows by the ticks (and shows for a player), and the level completes when a combo unlocks (@ 0x21d788).
pub fn gmBagTagTick() {
    let players = lw(g::lvnumplayers);
    let chars = lw(g::numactivechars);
    let mut holder = -1;
    let previous;
    let mut a4 = 0;
    'b1: {
        if players + chars <= 0 {
            previous = lw(0x3afee8);
        } else {
            a4 = lw(g::chrinfo);
            previous = lw(0x3afee8);
            if lw(a4 + 484) == 0 {
                let mut i = 1;
                loop {
                    if i >= players + chars {
                        break 'b1;
                    }
                    if lw(i * 0x1210 + a4 + 484) != 0 {
                        break;
                    }
                    i += 1;
                }
                holder = i;
            } else {
                holder = 0;
            }
        }
    }
    let mut current = previous;
    'b4: {
        if holder != current {
            current = holder;
            sw(0x3afee8, current);
            if current >= 0 {
                let offset = current * 0x1210;
                let languages = g::languages;
                let entry = lw((lw(g::cur_language) << 2) + languages);
                let name = lw(offset + lw(g::chrinfo) + 16);
                let text = vafmt(&[0x3ae568_i64, name as i64, lw(entry + 2844) as i64, entry as i64, a4 as i64]);
                current = lw(0x3afee8);
                let mut count = lw(g::lvnumplayers);
                let language = lw(g::cur_language);
                if current < count {
                    hudMessage(current, lw(lw((language << 2) + languages) + 2848));
                    count = lw(g::lvnumplayers);
                    current = lw(0x3afee8);
                }
                let mut player = 0;
                if count > 0 {
                    let mut offset = 0;
                    loop {
                        if player != current {
                            playerMessage(lw(g::players) + offset, text);
                            count = lw(g::lvnumplayers);
                            current = lw(0x3afee8);
                        }
                        player += 1;
                        offset += 1820;
                        if player >= count {
                            break 'b4;
                        }
                    }
                }
            }
        }
    }
    if current >= 0 {
        let is_player = (current < lw(g::lvnumplayers)) as i32;
        let chr = current * 0x1210 + lw(g::chrinfo);
        sw(chr + 248, lw(chr + 248) + lw(g::updateframesi));
        if is_player != 0 {
            hudShowScore(current, 304);
        }
    }
    if UnlockCombo() == 0 {
        return;
    }
    gameLevelCompleted();
}

/// The tick of the room lights: each room's light (28 bytes from the second record) runs its pattern by the kind (0 fixed, 1 a sine, 2 a ramp, 3 flicker) to a strength that
/// blends its two colours, which is stored and given to the room's object (unless it is white) (@ 0x2a7d20).
pub fn roomlightTick() {
    let mut room = 0;
    let mut strength = 0.0_f32;
    if lw(g::bgNumRooms) > 0 {
        let sixty = 6e+01_f32;
        let half = 0.5_f32;
        loop {
            let light = lw(g::roomlights) + (room * 28 + 28);
            let kind = lw(light);
            let prop = lw((room << 2) + lw(g::roomprops) + 4);
            let first;
            'b3: {
                'b4: {
                    if kind == 1 {
                        let one = 1.0_f32;
                        let phase = lwc1(light + 8) + lwc1(g::updateframesf) * lwc1(light + 4) / sixty;
                        swc1(light + 8, phase);
                        if one <= phase {
                            swc1(light + 8, phase - one);
                        }
                        let angle = lwc1(light + 8) * lwc1(0x3ad624);
                        let wave = sinf(angle);
                        strength = wave * half + half;
                    } else {
                        'b6: {
                            if kind >= 2 {
                                let past;
                                if kind == 2 {
                                    let one = 1.0_f32;
                                    let phase = lwc1(light + 8) + lwc1(g::updateframesf) * lwc1(light + 4) / sixty;
                                    swc1(light + 8, phase);
                                    if one <= phase {
                                        swc1(light + 8, phase - one);
                                    }
                                    strength = 0.0_f32;
                                    past = half <= lwc1(light + 8);
                                } else {
                                    let ticks = lw(g::updateframesi);
                                    if kind != 3 {
                                        break 'b6;
                                    }
                                    if ticks == 0 {
                                        first = lw(light + 16);
                                        break 'b3;
                                    }
                                    let roll = random_as_float();
                                    strength = 0.0_f32;
                                    past = (roll * 2.3283064e-10_f32) < lwc1(light + 8);
                                }
                                if !past {
                                    first = lw(light + 16);
                                    break 'b3;
                                }
                            } else if kind == 0 {
                                strength = lwc1(light + 8);
                                break 'b4;
                            }
                        }
                        strength = 1.0_f32;
                    }
                }
                first = lw(light + 16);
            }
            let second = lw(light + 20);
            let a0_1 = ((first as u32) >> 24) as i32;
            let a5_1 = ((first as u32) >> 16) as i32 & 255;
            let a2_1 = ((first as u32) >> 8) as i32 & 255;
            let a4_2 = first & 255;
            let colour = (a0_1 + (strength * (((second as u32) >> 24) as i32 - a0_1) as f32) as i32) << 24
                | a4_2 + (strength * ((second & 255) - a4_2) as f32) as i32
                | ((a2_1 + (strength * ((((second as u32) >> 8) as i32 & 255) - a2_1) as f32) as i32) << 8
                    | (a5_1 + (strength * ((((second as u32) >> 16) as i32 & 255) - a5_1) as f32) as i32) << 16);
            sw(light + 24, colour);
            if colour != -1 {
                obInstLight(lw(prop + 32), light);
            }
            room += 1;
            if room >= lw(g::bgNumRooms) {
                break;
            }
        }
    }
}

/// The tick of the challenge page: the page is set; the shoulder buttons step the challenge number through the available ones (wrapping over 27) with a sound; the buttons
/// 2112 go on to the challenge character page; the menu ticks (@ 0x23c808).
pub fn challenge_pageTick_23c808(a0: i32) {
    let languages = g::languages;
    let language_at = (lw(g::cur_language) << 2) + languages;
    sw(((lw(0x3aff2c) - 1) << 3) + 0x1fc1308, 0);
    pageSet(lw(lw(language_at) + 488), g::challenge_page_354eb8);
    frontfxTick(0);
    let state = 0x1fc11f8;
    let setup = challengeGetSetupNum(lw(state + 44));
    let front = lw(g::front);
    let name = (lh(setup + 60) << 2) + lw((lw(g::cur_language) << 2) + languages);
    sw(state + 8, lw(state + 8) | 48);
    let flags = lw(front + 12);
    sw(g::i0700 + 32, lw(name));
    'b1: {
        if (flags & 16) == 0 {
            if (flags & 32) != 0 {
                soundStart(140, 0);
                let mut number = lw(state + 44);
                loop {
                    let next = number + 1;
                    sw(state + 44, next);
                    if next >= 27 {
                        sw(state + 44, 0);
                    }
                    if challengeAvail(lw(state + 44)) != 0 {
                        break 'b1;
                    }
                    number = lw(state + 44);
                }
            }
        } else {
            soundStart(140, 0);
            let mut number = lw(state + 44);
            loop {
                let previous = number - 1;
                sw(state + 44, previous);
                if previous < 0 {
                    sw(state + 44, 26);
                }
                if challengeAvail(lw(state + 44)) != 0 {
                    break 'b1;
                }
                number = lw(state + 44);
            }
        }
    }
    if (joyNewBut(0) & 2112) != 0 {
        frontPageSet(0, g::challengechr_page);
        setStates(1);
        pageIn(a0);
    }
    let menu = g::challenge_menu;
    menutick_230238(a0, menu);
    let made = std_menumake(lw(a0 + 8), menu);
    sw(a0 + 12, made);
}

/// The ticks after the level's own: a game level ticks the background, animations, room lights, props, character guns, sound, music, timers, statistics and the pause check;
/// the levels 101 to 105 set the window and tick what they need (@ 0x2263a0).
pub fn lvTickAfter() {
    let level = lw(g::lvlevelnum) - 101;
    if (level as u32) >= 5 {
        bgTick();
        animMtxTick();
        roomlightTick();
        propTickAfter();
        chrgunTick();
        aiEndTick();
        soundTick();
        musicTick();
        timerTick();
        StatsTick();
        lvSoundsTick();
        ingamemenuCheckUnpause();
        return;
    }
    windowTick();
    let player = lw(g::currentplayer);
    windowSet(lw(player + 4), lw(player + 8), lw(player + 12), lw(player + 16));
    match level {
        0 | 1 => {
            animMtxTick();
            propTickAfter();
            soundTick();
            musicTick();
            mcardTick();
        }
        2 => {
            mapmakerTick();
            camTick();
            animMtxTick();
            propTickAfter();
            soundTick();
            musicTick();
            mcardTick();
        }
        3 => {
            loadingTick();
        }
        4 => {
            creditsTick();
            animMtxTick();
            propTickAfter();
            soundTick();
            musicTick();
        }
        _ => unreachable!(),
    }
}

/// Process the glass hits of the background: for each of the `a1` hits (records of 44 bytes at 0x1fc4748) the glass's strength grows by `f12` and, by a random chance, the glass
/// smashes; otherwise a decal of the crack is added in the first free place of the pane's four (@ 0x25a090).
pub fn bgProcessGlasshits(a0: i32, a1: i32, a2: i32, f12: f32) {
    let frame = Frame::new(224);
    let sp = frame.sp();
    let mut i = 0;
    if a1 > 0 {
        loop {
            let record = i * 44 + 0x1fc4748;
            let slot = lw(record + 40) << 2;
            let list = lw(lw(record + 36) * 44 + lw(g::levelrooms) + 16);
            let glass = lw(slot + list);
            swc1(glass, lwc1(glass) + f12);
            let roll = random_as_float();
            let at = slot + list;
            let next;
            if !(lwc1(glass) < 0.5_f32 - roll * 2.3283064e-10_f32 * lwc1(0x3acacc)) {
                let base = 0x1fc4748;
                next = i + 1;
                bgSmashGlass(lw(at), i * 44 + base, i * 44 + (base + 24), i * 44 + (base + 12), a2, f12);
            } else {
                let mut place = -1;
                let pane = lw(at);
                let first = lw(pane + 176);
                swc1(sp, lwc1(pane + 16));
                swc1(sp + 4, lwc1(pane + 20));
                swc1(sp + 8, lwc1(pane + 24));
                swc1(sp + 12, lwc1(pane + 56));
                swc1(sp + 16, lwc1(pane + 60));
                swc1(sp + 20, lwc1(pane + 64));
                swc1(sp + 24, lwc1(pane + 96));
                swc1(sp + 28, lwc1(pane + 100));
                swc1(sp + 32, lwc1(pane + 104));
                swc1(sp + 36, lwc1(pane + 136));
                swc1(sp + 40, lwc1(pane + 140));
                swc1(sp + 44, lwc1(pane + 144));
                next = i + 1;
                'b5: {
                    if first != place {
                        let mut j = 1;
                        loop {
                            if j >= 4 {
                                break 'b5;
                            }
                            if lw(lw(slot + list) + (j << 2) + 176) == -1 {
                                break;
                            }
                            j += 1;
                        }
                        place = j;
                    } else {
                        place = 0;
                    }
                }
                if place >= 0 {
                    let base = 0x1fc4748;
                    let decal = decalNewGlassDecal(a0, lw(base + i * 44 + 36), i * 44 + base, i * 44 + (base + 24), sp, 4);
                    sw(lw(slot + list) + (place << 2) + 176, decal);
                }
            }
            i = next;
            if i >= a1 {
                break;
            }
        }
    }
}

/// The tick of the in-game main menu page of the current player: a page queued for it is made the current one, the page changes are followed, the top menu or the page's own
/// tick runs, and a button leaves the menu (2320 in the paused state, 2048 when the flag 8 says so) (@ 0x2122c0).
pub fn main_pageTick_2122c0(a0: i32) {
    let menu = lw(lw(g::currentplayer)) * 92 + 0x1fb3240;
    if lw(g::gamestate) < 5 {
        let offset = lhu(menu + 4) << 2;
        let slot = menu + 52 + offset;
        let queued = lw(slot);
        let base = menu + 12;
        if queued != 0 {
            let selected = lhu(menu + 4);
            sw(base + offset, queued);
            sw(slot, 0);
            sh(a0 + 4, lhu(lw(base + (selected << 2)) + 8));
        }
        let selected = lhu(menu + 4);
        if lhu(menu + 6) != selected {
            let page = lw(menu + ((selected & 0xffff) << 2) + 12);
            if page == 0 {
                sh(menu + 6, selected);
            } else {
                sh(a0 + 4, lhu(page + 8));
                sh(menu + 6, lhu(menu + 4));
            }
        }
        let state = lhu(menu + 8);
        let mut paused = state & 4;
        if (state & 2) == 0 {
            let current;
            if lh(menu + 2) != 0 {
                current = lhu(menu + 4);
            } else {
                topmenuTick();
                current = lhu(menu + 4);
            }
            let page = lw(menu + (current << 2) + 12);
            let now;
            if page == 0 {
                now = lhu(menu + 8);
            } else {
                let tick = lw(page);
                if tick == 0 {
                    now = lhu(menu + 8);
                } else {
                    call_indirect(tick, [a0, queued, slot, 92, 0, 0, 0, 0], [0.0; 8]);
                    now = lhu(menu + 8);
                }
            }
            paused = now & 4;
        }
        let mut sub = lhu(menu + 2);
        if paused == 0 {
            if lh(menu + 2) != 0 {
                return;
            }
            let player = lw(g::currentplayer);
            if lw(g::gamestate) == 3 {
                let pressed = joyNewBut(lw(player));
                let player_now = lw(g::currentplayer);
                if (pressed & 2320) != 0 {
                    let pressed = joyNewBut(lw(player_now));
                    let mut front = lw(g::front);
                    if (pressed & 16) != 0 {
                        if (lhu(menu + 8) & 2) != 0 {
                            return;
                        }
                        front = lw(g::front);
                    }
                    ingamemenuExit(lw(front));
                    return;
                }
                sub = lhu(menu + 2);
            }
        }
        if sub == 0 {
            if (joyNewBut(lw(lw(g::currentplayer))) & 2048) != 0 {
                if (lhu(menu + 8) & 8) != 0 {
                    ingamemenuExit(lw(lw(g::front)));
                    return;
                }
            }
        }
    }
}
