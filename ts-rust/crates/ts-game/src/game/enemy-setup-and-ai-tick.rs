//! Functions: setting up the computer characters of a level, the first part of the tick of the computer characters, the firing of the shots of a gun of a player and the tick of
//! the page that loads maps from the memory card.

use super::particle_spawners::random_as_float;
use crate::*;

/// A random number from 0 up to 1.
fn random_unit() -> f32 {
    ps2float::mul(random_as_float(), 2.3283064e-10_f32)
}

/// The prop for the flag of the team `team` (1 to 4), or -1 for another team.
fn team_flag_prop(team: i32) -> i32 {
    match team {
        2 => 429,
        3 => 428,
        4 => 430,
        1 => 427,
        _ => -1,
    }
}

/// Set up the computer characters of the level (@ 0x2ba140): the setup of the level is kept, the characters are made (for the story levels from the setup, for the other modes
/// as the characters chosen for the game), the doors and links of some levels are set for the mode, and the data for the Last Stand waves is set.
pub fn enemySetup() {
    let frame = Frame::new(192);
    let sp = frame.sp();
    let setup = lvGetLevelSetup(lw(g::lvlevelnum));
    sw(g::setup, setup);
    let mode = lw(g::curgameparms + 72);
    if mode == 4 {
        let flag = g::teamflag;
        let word = lw(flag + 8) | 32;
        sw(flag + 4, lw(flag + 4) | 16);
        sw(flag + 8, word);
    }
    'b2: {
        'b3: {
            let count = lw(g::numactivechars);
            if mode != 7 {
                if count > 0 {
                    if mode == 6 {
                        break 'b2;
                    }
                    let mut n = 0;
                    let mut level = lw(g::lvlevelnum);
                    loop {
                        let players = lw(g::lvnumplayers);
                        let count_now;
                        if level != 25 {
                            let parms = g::curgameparms;
                            let number = n + players;
                            let kind = lbu(parms + number + 21);
                            let chr = chrNew(kind, kind, 0, number, 10, 1, 0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32);
                            let slot = (n << 2) + g::chr;
                            sw(slot, chr);
                            chrSelectTeam(lw(chr + 352));
                            obInstSetAnim(lw(lw(slot) + 32), 3, 1.0_f32);
                            if (lw(parms + 80) & 1) != 0 {
                                let team = lw(lw(lw(slot) + 352) + 24);
                                let flag_prop = team_flag_prop(team);
                                let owner = lw(slot);
                                let z = lwc1(owner + 56);
                                let y = ps2float::add(lwc1(owner + 52), lwc1(0x3ad7d8));
                                let x = lwc1(owner + 48);
                                let prop = propNew(flag_prop, x, y, z, 0.0_f32);
                                let owner = lw(slot);
                                sw(prop + 16, lw(prop + 16) | 0x40000000);
                                sw(owner + 536, prop);
                                sw(prop + 12, lw(owner + 12));
                            }
                            count_now = lw(g::numactivechars);
                        } else {
                            let chr = chrNew(79, 79, 0, n + players, 10, 1, 0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32);
                            let slot = (n << 2) + g::chr;
                            sw(slot, chr);
                            chrSelectTeam(lw(chr + 352));
                            obInstSetAnim(lw(lw(slot) + 32), 1, 0.0_f32);
                            count_now = lw(g::numactivechars);
                        }
                        n += 1;
                        level = lw(g::lvlevelnum);
                        if n >= count_now {
                            break 'b3;
                        }
                    }
                }
            } else {
                let level_setup = lw(g::setup);
                let number = lw(level_setup + 44);
                sw(g::numactivechars, number);
                if number > 0 {
                    let mut n = 0;
                    loop {
                        let offset = n << 2;
                        let entry = lw(level_setup + 40) + n * 24;
                        lvGetChrTypeNumFromSetup(lw(entry + 8) & 15, sp, sp + 4);
                        let pad_extref = lw(entry);
                        let index = n + lw(g::lvnumplayers);
                        let kind = lw(sp);
                        let skin = lw(sp + 4);
                        let pad = padNumFromExtref(pad_extref);
                        let chr = chrNew(kind, skin, 0, index, pad, 1, -8.0_f32, 1.0_f32, lwc1(0x3ad7d4), 0.0_f32);
                        let slot = offset + g::chr;
                        sw(slot, chr);
                        chrSelectTeam(lw(chr + 352));
                        let mut special = false;
                        if lw(sp + 4) == 31 {
                            if (lw(entry + 12) & 0x2000000) != 0 {
                                obInstSetAnim(lw(lw(slot) + 32), 476, 0.0_f32);
                                special = true;
                            }
                        }
                        if !special {
                            obInstSetAnim(lw(lw(offset + g::chr) + 32), 3, 1.0_f32);
                        }
                        n += 1;
                        if n >= lw(lw(g::setup) + 44) {
                            break;
                        }
                    }
                }
            }
        }
    }
    if lw(g::curgameparms + 72) == 4 {
        let escorts = lw(g::numEscorts);
        sw(g::escort, 0);
        if escorts == 0 {
            let players = lw(g::lvnumplayers);
            let count = lw(g::numactivechars);
            let mut n = players;
            loop {
                if n >= count + players {
                    break;
                }
                let record = n * 0x1210 + lw(g::chrinfo);
                if lw(record + 24) == 1 {
                    enemyAddToEscortList(lw(record + 3020));
                    break;
                }
                n += 1;
            }
        }
    }
    'b17: {
        let parms = g::curgameparms;
        let mut mode_now = lw(parms + 72);
        let start_mode = mode_now;
        if mode_now == 7 {
            lvInitStoryState();
            mode_now = lw(parms + 72);
            if mode_now == start_mode {
                break 'b17;
            }
        }
        let level = lw(g::lvlevelnum);
        if level == 8 {
            if mode_now == 6 {
                for door in 1..=5 {
                    doorClose(door);
                }
                doorOpen(6);
                for door in 7..=14 {
                    doorClose(door);
                }
                for door in 1..=14 {
                    doorLock(door);
                }
                for (a, b) in [(1092, 1095), (1098, 1102), (1081, 1008), (1193, 1194), (1185, 1186), (1128, 1388), (1152, 1154), (1036, 1035), (1265, 1035), (1040, 1039), (1040, 1337), (1360, 1198), (1194, 1357), (1128, 1388)] {
                    let link = padGetLinkNum(a, b);
                    linkDisable(link);
                }
                bgPortalDisableFromRooms(92, 93);
                bgPortalDisableFromRooms(90, 69);
                bgPortalDisableFromRooms(71, 95);
                bgPortalDisableFromRooms(70, 95);
                bgPortalDisable(91);
                bgPortalDisable(92);
                bgPortalDisable(93);
                bgPortalDisableFromRooms(3, 18);
                bgPortalDisableFromRooms(5, 13);
                bgPortalDisableFromRooms(13, 14);
                bgPortalDisableFromRooms(86, 98);
                bgPortalDisableFromRooms(37, 31);
            }
        } else if level == 27 {
            if mode_now == 4 {
                for door in 1..=8 {
                    doorOpen(door);
                    doorLock(door);
                }
            } else {
                doorOpen(3);
                doorLock(3);
                doorOpen(4);
                doorLock(4);
                doorClose(1);
                doorLock(1);
                doorClose(2);
                doorLock(2);
            }
        }
        enemyPreCalcEnemyRoutes();
    }
    let wave_mode = lw(g::curgameparms + 72);
    if wave_mode == 6 {
        let level = lw(g::lvlevelnum);
        sw(g::numactivechars, 0);
        let phase;
        let sniper;
        let waves;
        if level != 8 {
            if level >= 9 {
                if level == 9 {
                    phase = g::planetphase;
                    sniper = g::planetsnipergrp;
                    waves = 7;
                } else if level == 12 {
                    phase = g::futurephase;
                    sniper = g::futuresnipergrp;
                    waves = wave_mode;
                } else {
                    exit(1);
                    return;
                }
            } else if level == 5 {
                phase = g::industrialphase;
                sniper = g::industrialsnipergrp;
                waves = wave_mode;
            } else {
                exit(1);
                return;
            }
        } else {
            phase = g::mansionphase;
            sniper = g::mansionsnipergrp;
            waves = 5;
        }
        sw(0x3b01e0, phase);
        sw(0x3b01b0, waves);
        sw(0x3b01e4, sniper);
    }
}

/// The first part of the tick of the computer characters (@ 0x2ba9c8): the event handlers of the level run, one of the characters is checked for seeing its target (or, every
/// 4th tick, one asleep is checked for waking), the viewing masks of the characters and their trigger pads are brought up to date and, in Last Stand, the overseer puts
/// characters that are far from the players back to sleep.
pub fn aiTickBefore() {
    let frame = Frame::new(192);
    let sp = frame.sp();
    let count = lw(g::numactivechars);
    let mut seen = 0;
    if lw(0x3b01a8) < count {
        sw(0x3b01a8, count);
    }
    let setup = lw(g::setup);
    let handler = lw(g::currAIEventHandler);
    if lw(setup + 60) > 0 {
        let routine = lw((handler << 2) + lw(setup + 56));
        sw(g::currAIEventHandler, handler + 1);
        call_indirect(routine, [routine, 0, 0, 0, 0, 0, 0, 0], [0.0; 8]);
        if lw(g::currAIEventHandler) >= lw(lw(g::setup) + 60) {
            sw(g::currAIEventHandler, 0);
        }
    }
    'b3: {
        updatePlayerPadPos();
        playerCampingDetection();
        let count = lw(g::numactivechars);
        let sleep_frame = lw(g::checksleepframe);
        if count != 0 {
            'b4: {
                let sleep_frame = sleep_frame - 1;
                sw(g::checksleepframe, sleep_frame);
                if sleep_frame > 0 {
                    let mut index = lw(g::updateplayer);
                    let mut found = false;
                    let mut mind;
                    let mut target;
                    loop {
                        mind = lw(lw((index << 2) + g::chr) + 352);
                        target = lw(mind + 2788);
                        if target == 0 {
                            index += 1;
                        } else if (lw(target + 8) & 329) == 0 {
                            index += 1;
                        } else {
                            let state = lw(mind + 2716);
                            if state == 0x2000 {
                                index += 1;
                            } else if state != 0 {
                                found = true;
                                break;
                            } else {
                                index += 1;
                            }
                        }
                        sw(g::updateplayer, index);
                        if index < count {
                            continue;
                        }
                        sw(g::updateplayer, 0);
                        index = 0;
                        break;
                    }
                    let chr_slot = index << 2;
                    if !found {
                        let wake = 1.0_f32 < lwc1(mind + 2912);
                        sw(mind + 2772, 0);
                        if wake {
                            sw(mind + 2912, 0);
                        }
                    } else {
                        'b12: {
                            let x = lwc1(mind + 3008);
                            swc1(sp + 48, x);
                            let chr = lw(chr_slot + g::chr);
                            let y = lwc1(mind + 3012);
                            let flags = lw(chr + 16);
                            swc1(sp + 52, y);
                            let z = lwc1(mind + 3016);
                            sw(chr + 16, flags & -33);
                            swc1(sp + 56, z);
                            let kind = lw(target + 8);
                            if kind == 8 {
                                let target_mind = lw(target + 352);
                                swc1(sp + 16, ps2float::sub(lwc1(target + 48), x));
                                swc1(sp + 20, ps2float::sub(ps2float::sub(lwc1(target_mind + 3012), lwc1(0x3ad7e4)), y));
                                let tz = lwc1(target + 56);
                                sw(sp, 1);
                                sw(sp + 8, 0);
                                swc1(sp + 24, ps2float::sub(tz, z));
                                seen = bgLineTest(lw(chr + 12), sp + 48, sp + 16, 0, 0, 0, 0, 112);
                                target = lw(mind + 2788);
                            } else {
                                let prop_kind;
                                if kind >= 9 {
                                    if kind != 64 {
                                        if kind == 256 {
                                            let tx = lwc1(target + 48);
                                            let dx = ps2float::sub(tx, x);
                                            let pos1 = g::gpos1;
                                            let pos2 = g::gpos2;
                                            swc1(sp + 16, dx);
                                            let dy = ps2float::sub(ps2float::add(lwc1(target + 52), 0.5_f32), y);
                                            swc1(sp + 20, dy);
                                            let tz = lwc1(target + 56);
                                            swc1(g::gpos2, ps2float::add(x, dx));
                                            let dz = ps2float::sub(tz, z);
                                            sw(sp, 0);
                                            sw(sp + 8, 0);
                                            swc1(g::gpos1, x);
                                            swc1(pos1 + 8, z);
                                            swc1(pos2 + 4, ps2float::add(y, dy));
                                            let room = lw(chr + 12);
                                            swc1(pos2 + 8, ps2float::add(z, dz));
                                            swc1(sp + 24, dz);
                                            swc1(pos1 + 4, y);
                                            seen = bgLineTest(room, sp + 48, sp + 16, 0, 0, 0, 0, 48);
                                            target = lw(mind + 2788);
                                            break 'b12;
                                        }
                                        break 'b12;
                                    }
                                    prop_kind = lw(target + 4);
                                } else {
                                    if kind != 1 {
                                        break 'b12;
                                    }
                                    prop_kind = lw(target + 4);
                                }
                                let x = lwc1(sp + 48);
                                let y = lwc1(sp + 52);
                                let z = lwc1(sp + 56);
                                let top;
                                if ((prop_kind - 201) as u32) >= (2 as u32) {
                                    swc1(sp + 16, ps2float::sub(ps2float::add(lwc1(target + 48), lwc1(target + 508)), x));
                                    top = ps2float::add(lwc1(target + 52), lwc1(target + 512));
                                } else {
                                    swc1(sp + 16, ps2float::sub(ps2float::add(lwc1(target + 48), lwc1(target + 508)), x));
                                    top = ps2float::add(lwc1(target + 52), 0.25_f32);
                                }
                                swc1(sp + 20, ps2float::sub(top, y));
                                swc1(sp + 24, ps2float::sub(ps2float::add(lwc1(target + 56), lwc1(target + 516)), z));
                                let end_x = ps2float::add(x, lwc1(sp + 16));
                                let shooter = lw((lw(g::updateplayer) << 2) + g::chr);
                                let end_z = ps2float::add(z, lwc1(sp + 24));
                                let dy = lwc1(sp + 20);
                                sw(sp, 0);
                                let pos2 = g::gpos2;
                                sw(sp + 8, 0);
                                swc1(g::gpos2, end_x);
                                let pos1 = g::gpos1;
                                swc1(pos2 + 8, end_z);
                                swc1(g::gpos1, x);
                                swc1(pos1 + 8, z);
                                swc1(pos2 + 4, ps2float::add(y, dy));
                                let room = lw(shooter + 12);
                                swc1(pos1 + 4, y);
                                seen = bgLineTest(room, sp + 48, sp + 16, 0, 0, 0, 0, 48);
                                target = lw(mind + 2788);
                            }
                        }
                        'b16: {
                            'b17: {
                                'b18: {
                                    if seen != target {
                                        let wake_time;
                                        'b19: {
                                            if seen == 0 {
                                                wake_time = lwc1(mind + 2912);
                                            } else {
                                                let seen_kind = lw(seen + 8);
                                                if seen_kind == 8 {
                                                    'b20: {
                                                        let parms = g::curgameparms;
                                                        let mode;
                                                        'b21: {
                                                            if lw(mind + 2716) == seen_kind {
                                                                let seen_mind = lw(seen + 352);
                                                                if lw(seen_mind + 24) != lw(mind + 24) {
                                                                    let m = lw(parms + 72);
                                                                    if m != 3 {
                                                                        mode = m;
                                                                        break 'b21;
                                                                    }
                                                                    let who = lw(seen_mind);
                                                                    if who >= lw(g::lvnumplayers) {
                                                                        break 'b20;
                                                                    }
                                                                    if 4.0_f32 < lwc1(who * 1820 + lw(g::players) + 880) {
                                                                        wake_time = lwc1(mind + 2912);
                                                                        break 'b19;
                                                                    }
                                                                    break 'b20;
                                                                }
                                                            }
                                                            mode = lw(parms + 72);
                                                        }
                                                        if mode == 7 {
                                                            break 'b17;
                                                        }
                                                        if (lw(mind + 2704) & 1) == 0 {
                                                            break 'b16;
                                                        }
                                                        if lw(lw(seen + 352) + 24) == lw(mind + 24) {
                                                            break 'b17;
                                                        }
                                                    }
                                                    sw(mind + 2788, seen);
                                                    sw(mind + 2772, 1);
                                                    break 'b18;
                                                }
                                                wake_time = lwc1(mind + 2912);
                                            }
                                        }
                                        sw(mind + 2772, 0);
                                        if 1.0_f32 < wake_time {
                                            sw(mind + 2912, 0);
                                        }
                                    } else {
                                        sw(mind + 2772, 1);
                                    }
                                }
                            }
                        }
                        let invisible = lw(g::invisible);
                        let shooter = lw((lw(g::updateplayer) << 2) + g::chr);
                        sw(shooter + 16, lw(shooter + 16) | 32);
                        sw(mind + 2704, lw(mind + 2704) | 0x4000);
                        if invisible != 0 {
                            sw(mind + 2772, 0);
                        }
                    }
                    let current = lw(g::updateplayer);
                    if found {
                        let count = lw(g::numactivechars);
                        let next = current + 1;
                        sw(g::updateplayer, next);
                        if next < count {
                            break 'b4;
                        }
                        sw(g::updateplayer, 0);
                        break 'b4;
                    }
                } else {
                    sw(g::checksleepframe, 4);
                    let mut index = lw(g::checksleepplayer);
                    let mut found = false;
                    let mut mind;
                    loop {
                        mind = lw(lw((index << 2) + g::chr) + 352);
                        if lw(mind + 2716) != 0x2000 {
                            index += 1;
                            sw(g::checksleepplayer, index);
                            if index < count {
                                continue;
                            }
                            sw(g::checksleepplayer, 0);
                            index = 0;
                        } else {
                            found = true;
                        }
                        break;
                    }
                    let _ = index;
                    let player = lw(g::players);
                    if !found {
                        break 'b4;
                    }
                    let prop = lw(mind + 3020);
                    let x = lwc1(prop + 48);
                    let flags = lw(prop + 16);
                    swc1(sp + 32, x);
                    let first_mind = lw(lw(player + 384) + 352);
                    let y = ps2float::add(lwc1(prop + 52), lwc1(0x3ad7dc));
                    sw(prop + 16, flags & -33);
                    let height = lwc1(0x3ad7e0);
                    swc1(sp + 36, y);
                    let z = lwc1(prop + 56);
                    swc1(sp + 40, z);
                    swc1(sp + 16, ps2float::sub(lwc1(player + 140), x));
                    swc1(sp + 20, ps2float::sub(ps2float::sub(lwc1(first_mind + 3012), height), y));
                    let tz = lwc1(player + 148);
                    sw(sp, 1);
                    sw(sp + 8, 0);
                    let room = lw(prop + 12);
                    swc1(sp + 24, ps2float::sub(tz, z));
                    let result = bgLineTest(room, sp + 32, sp + 16, 0, 0, 0, 0, 112);
                    let owner;
                    if result != lw(mind + 2788) {
                        let wake_time = lwc1(mind + 2912);
                        sw(mind + 2772, 0);
                        if 1.0_f32 < wake_time {
                            sw(mind + 2912, 0);
                        }
                        owner = lw(mind + 3020);
                    } else {
                        sw(mind + 2772, 1);
                        enemyFromSleepToAwake(lw(mind + 3020));
                        owner = lw(mind + 3020);
                    }
                    sw(owner + 16, flags | 32);
                }
            }
            let count = lw(g::numactivechars);
            let players_count = lw(g::lvnumplayers);
            let distance_index = lw(g::updatedistindex) + 1;
            sw(g::updatedistindex, distance_index);
            if distance_index >= count + players_count {
                sw(g::updatedistindex, 0);
            }
            let mut players_count = players_count;
            if count > 0 {
                let total = lw(g::numactivechars);
                let chr_table = g::chr;
                let players = lw(g::players);
                let clear = -0xf000001;
                let mut n = 0;
                loop {
                    let chr = lw((n << 2) + chr_table);
                    let mind = lw(chr + 352);
                    let aimed = lw(mind + 440);
                    sw(chr + 16, lw(chr + 16) & clear);
                    if aimed != 0 {
                        sw(aimed + 16, lw(aimed + 16) & clear);
                    }
                    let moving = lw(mind + 332);
                    if moving != 0 {
                        sw(moving + 16, lw(moving + 16) & clear);
                    }
                    players_count = lw(g::lvnumplayers);
                    if players_count <= 0 {
                        n += 1;
                    } else {
                        let next = n + 1;
                        let mut player = 0;
                        loop {
                            let next_player = player + 1;
                            if lw(player * 1820 + players + 908) > 0 {
                                let mut k = 0;
                                loop {
                                    let owner = lw((n << 2) + chr_table);
                                    if lw(player * 1820 + players + (k << 2) + 912) == lw(owner + 12) {
                                        let mask = 0x1000000 << (player & 31);
                                        sw(owner + 16, lw(owner + 16) | mask);
                                        if aimed != 0 {
                                            sw(aimed + 16, lw(aimed + 16) | mask);
                                        }
                                        let dependent;
                                        if moving == 0 {
                                            dependent = lw(owner + 188);
                                        } else {
                                            sw(moving + 16, lw(moving + 16) | mask);
                                            dependent = lw(owner + 188);
                                        }
                                        if dependent != 0 {
                                            let other = lw(dependent + 244);
                                            sw(other + 16, lw(other + 16) | mask);
                                        }
                                    }
                                    k += 1;
                                    if k >= lw(player * 1820 + players + 908) {
                                        break;
                                    }
                                }
                                players_count = lw(g::lvnumplayers);
                            }
                            player = next_player;
                            if player >= players_count {
                                break;
                            }
                        }
                        n = next;
                    }
                    if n >= total {
                        break;
                    }
                }
            }
            let mut found_pad = -1;
            let mut player = 0;
            if players_count > 0 {
                let mut pads = lw(g::numtriggerpads);
                loop {
                    let mut k = 0;
                    let next_player = player + 1;
                    if pads > 0 {
                        let chrinfo = lw(g::chrinfo);
                        loop {
                            let table = 0x1fedf00;
                            let offset = k << 3;
                            if lw(player * 0x1210 + chrinfo + 680) != lw(offset + table) {
                                k += 1;
                            } else {
                                let mask = 0x1000000 << (player & 31);
                                found_pad = k;
                                let mind = lw(table + offset + 4) * 0x1210 + chrinfo;
                                let prop = lw(mind + 3020);
                                let aimed = lw(mind + 440);
                                sw(prop + 16, lw(prop + 16) | mask);
                                if aimed != 0 {
                                    sw(aimed + 16, lw(aimed + 16) | mask);
                                }
                                let moving = lw(mind + 332);
                                let dependent;
                                if moving == 0 {
                                    dependent = lw(prop + 188);
                                } else {
                                    sw(moving + 16, lw(moving + 16) | mask);
                                    dependent = lw(prop + 188);
                                }
                                pads = lw(g::numtriggerpads);
                                if dependent != 0 {
                                    let other = lw(dependent + 244);
                                    sw(other + 16, lw(other + 16) | mask);
                                    pads = lw(g::numtriggerpads);
                                }
                                k += 1;
                            }
                            if k >= pads {
                                break;
                            }
                        }
                        players_count = lw(g::lvnumplayers);
                    }
                    player = next_player;
                    if player >= players_count {
                        break;
                    }
                }
            }
            let pads = lw(g::numtriggerpads);
            if found_pad != -1 {
                let last;
                if pads < 2 {
                    last = pads - 1;
                } else {
                    let table = 0x1fedf00;
                    let from = (pads - 1) << 3;
                    let to = found_pad << 3;
                    sw(to + table, lw(from + table));
                    sw(to + table + 4, lw(from + table + 4));
                    last = pads - 1;
                }
                sw(g::numtriggerpads, last);
            }
            if lw(g::curgameparms + 72) == 7 {
                let overseer = lvGetOverseerState();
                let mut n = lw(g::lvnumplayers);
                if overseer != 0 {
                    let mut count = lw(g::numactivechars);
                    let mut players = n;
                    if n < count + n {
                        let mut offset = n * 0x1210;
                        loop {
                            'b55: {
                                let mind = lw(g::chrinfo) + offset;
                                let state = lw(mind + 2716);
                                let target;
                                if (state & 0x69c84c) != 0 {
                                    target = lw(mind + 2788);
                                } else {
                                    if state != 0x1000000 {
                                        n += 1;
                                        break 'b55;
                                    }
                                    target = lw(mind + 2788);
                                }
                                if target == 0 {
                                    n += 1;
                                } else if lw(mind + 2772) != 0 {
                                    n += 1;
                                } else if (lw(lw(mind + 3020) + 16) & 0xf000000) != 0 {
                                    n += 1;
                                } else {
                                    let limit = lvGetOverseerBackSleepTime();
                                    count = lw(g::numactivechars);
                                    if limit < lwc1(mind + 2904) {
                                        let old = lw(mind + 2716);
                                        let flags = lw(mind + 2704) | 2048;
                                        sw(mind + 2716, 0x2000);
                                        sw(mind + 2704, flags);
                                        sw(mind + 2720, old);
                                        sw(mind + 2724, old);
                                        sw(mind + 2904, 0);
                                        sw(mind + 2892, 0);
                                        count = lw(g::numactivechars);
                                    }
                                    players = lw(g::lvnumplayers);
                                    n += 1;
                                }
                            }
                            offset += 0x1210;
                            if n >= count + players {
                                break 'b3;
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Fire the shots of the gun `a0` of a player (@ 0x28e360): the muzzle flash, the recoil and shake of the gun, the place of the muzzle, then every bullet of the shot is made
/// by `gunFire` (more than one for the shotguns and the like), cartridges are thrown, the sound is started and the state of the gun is set for the time until the next shot.
pub fn playerFireShots(a0: i32) {
    let frame = Frame::new(576);
    let sp = frame.sp();
    sw(sp + 376, sp + 16);
    memset(sp + 16, 0, 12);
    if a0 == 0 {
        return;
    }
    let gun = lw(a0 + 172);
    if gun == 0 {
        return;
    }
    let prop = lw(gun + 64);
    if prop == 0 {
        return;
    }
    let model_data = lw(prop + 32);
    if model_data == 0 {
        return;
    }
    let header = lw(model_data);
    if header == 0 {
        return;
    }
    let owner = lw(a0 + 12);
    if owner == 0 {
        return;
    }
    let owner_data = lw(owner + 32);
    if owner_data == 0 {
        return;
    }
    let gun_data = lw(gun) * 400 + GUNS_TABLE;
    let flags = lhu(owner_data + 292) | 1;
    let skeleton = header - lw(header) * 80;
    let second = lw(a0 + 52);
    sw(sp + 368, lw(lw(lw(g::currentplayer) + 384) + 352));
    let muzzle_bone = lw(gun_data + 364);
    let shot = if second == 0 { gun_data + 52 } else { gun_data + 188 };
    sw(sp + 372, skeleton);
    sh(owner_data + 292, flags);
    if muzzle_bone >= 0 {
        if lw(gun_data + 384) >= 0 {
            let random = random_unit();
            let kind = lw(lw(a0 + 172));
            let mut angle = ps2float::mul(random, 3.6e+02_f32);
            if kind == 1 || kind == 19 {
                let first = newrnd();
                let second = random_unit();
                let step = (((first as i32 as u32) % (6 as u32)) as i32) as f32;
                angle = ps2float::add(ps2float::mul(step, 6e+01_f32), ps2float::mul(ps2float::sub(second, 0.5_f32), 5.0_f32));
            }
            let owner_now = lw(a0 + 12);
            let data = lw(owner_now + 32);
            let header = lw(data);
            let matrix = lw(data + 4) + (lw(gun_data + 384) << 6);
            let pivot = lw(header - lw(header) * 80 + 72) + lw(gun_data + 364) * 12;
            matrixTrans(matrix, lwc1(pivot), lwc1(pivot + 4), lwc1(pivot + 8));
            matrixRotZXY(sp + 272, 0.0_f32, 0.0_f32, ps2float::mul(angle, lwc1(0x3ad2b0)).pdiv(1.8e+02_f32));
            sceVu0MulMatrix(matrix, matrix, sp + 272);
            matrixTrans(sp + 272, -lwc1(pivot), -lwc1(pivot + 4), -lwc1(pivot + 8));
            sceVu0MulMatrix(matrix, matrix, sp + 272);
        }
    }
    chrMuzzleFlash(gun);
    let gun_kind = lw(gun);
    let burst = lw(a0 + 184);
    if gun_kind == 11 || gun_kind == 13 {
        if burst != 0 {
            let data = lw(lw(a0 + 12) + 32);
            sh(data + 292, lhu(data + 292) & !(0 | 2 << (8 - burst & 31)));
        } else {
            let data = lw(lw(a0 + 12) + 32);
            sh(data + 292, lhu(data + 292) & 0xfffd);
        }
    }
    sw(a0 + 68, 0);
    sw(a0 + 72, 0);
    let spread = lwc1(a0 + 116);
    swc1(a0 + 76, lwc1(shot + 56));
    let strength = lwc1(shot + 60);
    let limited = spread < lwc1(0x3ad2b4);
    let recoil = ps2float::mul(lwc1(shot + 56), -2e+02_f32);
    sw(a0 + 184, burst + 1);
    swc1(a0 + 60, ps2float::mul(recoil, strength));
    let pull = lwc1(shot + 72);
    swc1(a0 + 120, 1.0_f32);
    swc1(a0 + 80, pull);
    if limited {
        sw(a0 + 92, 0);
        sw(a0 + 96, 0);
        sw(a0 + 100, 0);
        for place in [104, 108] {
            let random = random_unit();
            let centred = dpsub(fptodp(random), 0x3fe0000000000000_i64);
            let doubled = dpadd(centred, centred);
            let scaled = dpmul(doubled, fptodp(lwc1(shot + 96)));
            swc1(a0 + place, dptofp(scaled));
        }
        let random = random_unit();
        let size = lwc1(shot + 96);
        let wobble = ps2float::mul(random, lwc1(0x3ad2b8));
        swc1(a0 + 116, 1.0_f32);
        swc1(a0 + 112, ps2float::mul(wobble, size));
    }
    let to_radians = lwc1(0x3ad2bc);
    swc1(a0 + 188, lwc1(shot + 128));
    let ax = ps2float::mul(lwc1(prop + 72), to_radians).pdiv(1.8e+02_f32);
    let ay = ps2float::mul(lwc1(prop + 76), to_radians).pdiv(1.8e+02_f32);
    let az = ps2float::mul(lwc1(prop + 88), to_radians).pdiv(1.8e+02_f32);
    let mut pivot = lw(sp + 376);
    matrixRotZXY(sp + 48, ax, ay, az);
    let vy = lwc1(a0 + 72);
    let vz = lwc1(a0 + 76);
    swc1(sp + 256, lwc1(a0 + 68));
    swc1(sp + 260, vy);
    swc1(sp + 264, vz);
    matrixVecRot(sp + 48, sp + 256);
    let ry = lwc1(sp + 260);
    let rz = lwc1(sp + 264);
    let skeleton = lw(sp + 372);
    swc1(sp + 240, ps2float::add(lwc1(prop + 48), lwc1(sp + 256)));
    swc1(sp + 244, ps2float::add(lwc1(prop + 52), ry));
    swc1(sp + 248, ps2float::add(lwc1(prop + 56), rz));
    if lb(skeleton + 7) > 0 {
        pivot = lw(skeleton + 72);
    }
    swc1(sp + 32, lwc1(pivot));
    swc1(sp + 36, lwc1(pivot + 4));
    swc1(sp + 40, lwc1(pivot + 8));
    let turn = ps2float::mul(lwc1(prop + 76), to_radians).pdiv(1.8e+02_f32);
    matrixTransRotY(sp + 112, lwc1(prop + 48), lwc1(prop + 52), lwc1(prop + 56), turn);
    sceVu0MulMatrix(sp + 176, sp + 112, lw(lw(prop + 32) + 4));
    matrixTrans(sp + 112, lwc1(pivot), lwc1(pivot + 4), lwc1(pivot + 8));
    sceVu0MulMatrix(sp + 176, sp + 176, sp + 112);
    let owner = lw(a0 + 12);
    let owner_data = lw(owner + 32);
    swc1(gun + 72, lwc1(sp + 224));
    let owner_header = lw(owner_data);
    let skeleton_base = lw(owner_header);
    swc1(gun + 76, lwc1(sp + 228));
    let bone = lw(gun_data + 340);
    swc1(gun + 80, lwc1(sp + 232));
    let skeleton = owner_header - skeleton_base * 80;
    sw(sp + 372, skeleton);
    let pivot = if bone < 0 { lw(sp + 376) } else { lw(skeleton + 72) + bone * 12 };
    let to_radians2 = lwc1(0x3ad2c0);
    swc1(sp + 32, lwc1(pivot));
    swc1(sp + 36, lwc1(pivot + 4));
    swc1(sp + 40, lwc1(pivot + 8));
    matrixTransRotZXY(
        sp + 112,
        lwc1(owner + 48),
        lwc1(owner + 52),
        lwc1(owner + 56),
        ps2float::mul(lwc1(owner + 72), to_radians2).pdiv(1.8e+02_f32),
        ps2float::mul(lwc1(owner + 76), to_radians2).pdiv(1.8e+02_f32),
        ps2float::mul(lwc1(owner + 88), to_radians2).pdiv(1.8e+02_f32),
    );
    matrixVecMul(sp + 112, sp + 32);
    let muzzle_y = lwc1(sp + 36);
    let muzzle_z = lwc1(sp + 40);
    let view = lw(lw(g::currentplayer) + 1772);
    swc1(a0 + 16, lwc1(sp + 32));
    swc1(a0 + 20, muzzle_y);
    swc1(a0 + 24, muzzle_z);
    matrixVecMul(view, a0 + 16);
    let spark_bone = lw(gun_data + 360);
    if spark_bone >= 0 {
        let holder = lw(a0 + 168);
        let owner = lw(a0 + 12);
        swc1(sp + 352, ps2float::sub(lwc1(holder + 48), lwc1(holder + 112)));
        swc1(sp + 356, ps2float::sub(lwc1(holder + 52), lwc1(holder + 116)));
        swc1(sp + 360, ps2float::sub(lwc1(holder + 56), lwc1(holder + 120)));
        let word = lw(sp + 360);
        sd(sp + 336, ld(sp + 352));
        sw(sp + 344, word);
        let x = lwc1(sp + 240);
        let y = lwc1(sp + 244);
        let pivot = lw(lw(sp + 372) + 72) + spark_bone * 12;
        let z = lwc1(sp + 248);
        swc1(sp + 32, lwc1(pivot));
        swc1(sp + 36, lwc1(pivot + 4));
        swc1(sp + 40, lwc1(pivot + 8));
        let az = ps2float::mul(lwc1(owner + 88), to_radians2).pdiv(1.8e+02_f32);
        let ax = ps2float::mul(lwc1(owner + 72), to_radians2).pdiv(1.8e+02_f32);
        let ay = ps2float::mul(lwc1(owner + 76), to_radians2).pdiv(1.8e+02_f32);
        matrixTransRotZXY(sp + 112, x, y, z, ax, ay, az);
        matrixVecRotAligned(&[(sp + 112) as i64, (sp + 32) as i64]);
        let sy = lwc1(sp + 36);
        let player = lw(g::currentplayer);
        let sz = lwc1(sp + 40);
        swc1(sp + 32, ps2float::add(lwc1(sp + 32), lwc1(owner + 48)));
        let view = lw(player + 1772);
        swc1(sp + 36, ps2float::add(sy, lwc1(owner + 52)));
        swc1(sp + 40, ps2float::add(sz, lwc1(owner + 56)));
        matrixVecMulAligned(&[view as i64, (sp + 32) as i64]);
        let sparks = particleNewGunSparks(0, sp + 32, sp + 336);
        particleSetViewmask(sparks, 1 << (lw(lw(g::currentplayer)) & 31));
    }
    let bullet_kind = lw(shot);
    let mut bullets = 6;
    if bullet_kind != 4 {
        bullets = if (bullet_kind ^ 9) == 0 { 3 } else { 1 };
    }
    let count_f = bullets as f32;
    StatsAdd(8, lw(a0 + 180), count_f);
    StatsAddGun(0, lw(a0 + 180), lw(lw(sp + 368) + 260), count_f);
    let mut fired = 0;
    if bullets != 0 {
        loop {
            let random = random_unit();
            let mut side = ps2float::mul(ps2float::sub(ps2float::add(random, random), 1.0_f32), lwc1(shot + 32));
            let random = random_unit();
            let player = lw(g::currentplayer);
            let mut up = ps2float::mul(ps2float::sub(ps2float::add(random, random), 1.0_f32), lwc1(shot + 32));
            let aiming = lw(player + 828);
            if (lw(player + 20) & 2) != 0 {
                if bullets == 1 {
                    let scale = lwc1(0x3ad2c4);
                    up = ps2float::mul(up, scale);
                    side = ps2float::mul(side, scale);
                }
            }
            if aiming == 0 {
                let mind = lw(sp + 368);
                swc1(sp, ps2float::mul(ps2float::add(ps2float::add(lwc1(mind + 2976), ps2float::mul(side, lwc1(player + 240))), ps2float::mul(up, lwc1(player + 228))), lwc1(shot + 28)));
                swc1(sp + 4, ps2float::mul(ps2float::add(ps2float::add(lwc1(mind + 2980), ps2float::mul(side, lwc1(player + 244))), ps2float::mul(up, lwc1(player + 232))), lwc1(shot + 28)));
                swc1(sp + 8, ps2float::mul(ps2float::add(ps2float::add(lwc1(mind + 2984), ps2float::mul(side, lwc1(player + 248))), ps2float::mul(up, lwc1(player + 236))), lwc1(shot + 28)));
            } else {
                let y0 = lwc1(a0 + 20);
                let dx = ps2float::sub(lwc1(player + 832), lwc1(a0 + 16));
                let z0 = lwc1(a0 + 24);
                swc1(sp, dx);
                let dy = ps2float::sub(lwc1(player + 836), y0);
                swc1(sp + 4, dy);
                let dz = ps2float::sub(lwc1(player + 840), z0);
                swc1(sp + 8, dz);
                let inverse = (1.0_f32).pdiv(ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz)).abs().psqrt());
                let ny = ps2float::mul(lwc1(sp + 4), inverse);
                let nz = ps2float::mul(lwc1(sp + 8), inverse);
                swc1(sp, ps2float::mul(ps2float::add(ps2float::add(ps2float::mul(lwc1(sp), inverse), ps2float::mul(side, lwc1(player + 240))), ps2float::mul(up, lwc1(player + 228))), lwc1(shot + 28)));
                swc1(sp + 4, ps2float::mul(ps2float::add(ps2float::add(ny, ps2float::mul(side, lwc1(player + 244))), ps2float::mul(up, lwc1(player + 232))), lwc1(shot + 28)));
                swc1(sp + 8, ps2float::mul(ps2float::add(ps2float::add(nz, ps2float::mul(side, lwc1(player + 248))), ps2float::mul(up, lwc1(player + 236))), lwc1(shot + 28)));
            }
            let (extra, holder);
            if bullet_kind == 4 && fired >= 2 {
                extra = 0;
                holder = lw(a0 + 168);
            } else {
                extra = 1;
                holder = lw(a0 + 168);
            }
            fired += 1;
            gunFire(holder, lw(a0 + 172), a0, lw(a0 + 52), sp, lw(a0 + 176), lw(a0 + 180), extra);
            if fired >= bullets {
                break;
            }
        }
    }
    let mut gun_kind_now = lw(gun);
    if gun_kind_now != 20 {
        fireCartridges(a0, gun, gun_data, shot);
        gun_kind_now = lw(gun);
    }
    'sound: {
        if gun_kind_now == 18 && lw(a0) == 17 && lw(a0 + 52) == 0 {
            if lwc1(0x3ad2c8) < lwc1(a0 + 4) {
                break 'sound;
            }
        }
        gunFireSfx(gun, lw(a0 + 52), 0);
    }
    let player = lw(g::currentplayer);
    sw(a0 + 156, 1);
    let mind = lw(sp + 368);
    sw(mind + 2708, lw(mind + 2708) | 16);
    sw(player + 792, 60);
    let mut kind_now = lw(gun);
    if ((kind_now - 14) as u32) < (4 as u32) || kind_now == 25 {
        sw(a0, 5);
        swc1(a0 + 4, lwc1(0x3ad2cc));
        kind_now = lw(gun);
    }
    if kind_now == 18 {
        sw(a0, kind_now);
        swc1(a0 + 4, lwc1(0x3ad2d0));
    }
}

/// The tick of the page that loads maps from the memory card or the built-in list (@ 0x239ed8) for the page `a0`: by its state (0 to 5) it starts the load of the directory of
/// maps, reads the answer of the card, builds the list of up to 10 names to show, lets the player move in the list and picks a map to load.
pub fn arcade_mmload_pageTick(a0: i32) {
    let frame = Frame::new(192);
    let sp = frame.sp();
    let state = lbu(0x3aff78);
    sw(sp, a0);
    'b1: {
        'b2: {
            if (state as u32) < (6 as u32) {
                'b3: {
                    'b4: {
                        'b5: {
                            match state {
                                0 => {
                                    if lw(lw(g::mapdir1) + 4) != 0 {
                                        break 'b3;
                                    }
                                    mcseqUnvalidateCards();
                                    let languages = g::languages;
                                    let strings = lw((lw(g::cur_language) << 2) + languages);
                                    let size = mmfileDirSize();
                                    mcseqLoad(1, lw(strings + 1944), lw(strings + 1888), 0x3a1d28, lw(g::mapdir1), size, 0);
                                    mcseqAddOption(lw(lw((lw(g::cur_language) << 2) + languages) + 1952));
                                    let word = lw(0x1fc11f8 + 4);
                                    sb(0x3aff78, 1);
                                    sw(0x3aff80, -1);
                                    sw(0x1fc11f8 + 4, word & -17);
                                    sw(g::firstmap_3ae840, 0);
                                    break 'b1;
                                }
                                1 => {
                                    if mcseqGetState() != 1_i64 {
                                        break 'b1;
                                    }
                                    let directory;
                                    let value;
                                    'b7: {
                                        mcseqWindowClose();
                                        let result = mcseqGetResult();
                                        let index = result - 1;
                                        if (index as u32) < (9 as u32) {
                                            match index {
                                                0 => {
                                                    let data = lw(g::mapdir1) + 4;
                                                    let size = mmfileDirSize();
                                                    if crcCheck(data, size - 4, lw(lw(g::mapdir1))) != 0 {
                                                        break 'b3;
                                                    }
                                                    frontStartMessage(0, lw(lw((lw(g::cur_language) << 2) + g::languages) + 956), 0x7f3c3c7f, 0);
                                                    mmfileDirReset();
                                                    sw(lw(g::mapdir1) + 4, 1);
                                                    sb(0x3aff78, 2);
                                                    break 'b1;
                                                }
                                                1 | 2 | 5 | 6 => {}
                                                3 => {
                                                    let table = (lw(g::cur_language) << 2) + g::languages;
                                                    sw(lw(g::front) + 20, 1);
                                                    frontStartMessage(0, lw(lw(table) + 960), 0x6464647f, 0);
                                                    return;
                                                }
                                                4 => {
                                                    mmfileDirReset();
                                                    directory = lw(g::mapdir1);
                                                    value = 1;
                                                    break 'b7;
                                                }
                                                7 => {
                                                    break 'b4;
                                                }
                                                _ => {
                                                    mmfileDirBuiltinMaps();
                                                    sb(0x3aff78, 2);
                                                    sw(0x1fc11f8 + 4, lw(0x1fc11f8 + 4) | 16);
                                                    break 'b1;
                                                }
                                            }
                                        }
                                        mmfileDirReset();
                                        directory = lw(g::mapdir1);
                                        value = -666;
                                    }
                                    sw(directory + 4, value);
                                    sb(0x3aff78, 2);
                                    break 'b1;
                                }
                                2 | 3 => {
                                    let mut directory = lw(g::mapdir1);
                                    if state == 2 {
                                        if (lw(0x1fc11fc) & 16) == 0 {
                                            mcseqPollLoop();
                                            directory = lw(g::mapdir1);
                                        }
                                        let selected = lw(0x3aff80);
                                        if lw(directory + 8) <= 0 {
                                            let page = lw(sp);
                                            if lw(directory + 4) == -666 {
                                                sh(lw(sp) + 4, 666);
                                            } else {
                                                sh(page + 4, 1);
                                            }
                                        } else {
                                            let half = lhu(0x3aff80);
                                            let shown = if selected < 0 { 1 } else { half - lhu(g::firstmap_3ae840) + 1 };
                                            sh(lw(sp) + 4, shown);
                                        }
                                    }
                                    let visible = lw(directory + 8) - lw(g::firstmap_3ae840);
                                    sw(0x3aff74, visible);
                                    if visible >= 11 {
                                        sw(0x3aff74, 10);
                                    }
                                    let mut item = g::mfmi_354850;
                                    sw(item - 16, 0);
                                    let language = lw(g::cur_language);
                                    if (lw(0x1fc11fc) & 16) == 0 {
                                        let long_name = mcseqSlotGetLongName();
                                        let language = lw(g::cur_language);
                                        if long_name != 0 {
                                            let strings = lw((language << 2) + g::languages);
                                            let short_name = mcseqSlotGetShortName();
                                            let text = vafmt(&[0x3ae848, lw(strings + 964) as i64, short_name as i64]);
                                            strcpy(0x1fc2ff0, text);
                                        }
                                    } else {
                                        strcpy(0x1fc2ff0, lw(lw((language << 2) + g::languages) + 1952));
                                    }
                                    'b14: {
                                        if (lw(0x1fc11fc) & 16) == 0 {
                                            if mcseqSlotGetLongName() == 0 {
                                                break 'b14;
                                            }
                                        }
                                        copy_menu_item(item, g::mi_dirtop_354a10);
                                        item += 32;
                                        sw(0x354840, lw(0x354840) + 1);
                                    }
                                    let mut line = 0;
                                    let mut height = 25;
                                    if lw(0x3aff74) > 0 {
                                        loop {
                                            copy_menu_item(item, g::mi_dirfile_354a30);
                                            let directory = lw(g::mapdir1);
                                            let mut length = 0;
                                            let map = lw(g::firstmap_3ae840) + line;
                                            sh(item + 6, height);
                                            let name = line * 34 + 0x1fc3030;
                                            sq(sp + 16, 0x350000);
                                            strcpy(name, directory + (map * 40 + 12));
                                            let next_item = item + 32;
                                            let next_height = height + 10;
                                            let next_line = line + 1;
                                            let base = line * 34;
                                            let text = line * 34 + 0x1fc3030;
                                            loop {
                                                sq(sp + 16, 0x350000);
                                                let width = textWidth(text);
                                                if width < 261 {
                                                    break;
                                                }
                                                length = strlen(text);
                                                if length == 1 {
                                                    break;
                                                }
                                                sb(length + (base - 1) + 0x1fc3030, 0);
                                            }
                                            if length != 0 {
                                                sb(length + (base - 1) + 0x1fc3030, 46);
                                                sb(length + base + 0x1fc3030, 46);
                                                sb(length + (base + 1) + 0x1fc3030, 46);
                                                sb(length + (base + 2) + 0x1fc3030, 0);
                                            }
                                            let shown = lw(0x3aff74);
                                            sw(item, text);
                                            sh(item + 8, next_line);
                                            sh(item + 16, line);
                                            let following = if line >= shown - 1 { 0 } else { line + 2 };
                                            let counted = lw(0x354840);
                                            line = next_line;
                                            sh(item + 18, following);
                                            height = next_height;
                                            item = next_item;
                                            sw(0x354840, counted + 1);
                                            if line >= shown {
                                                break;
                                            }
                                        }
                                    }
                                    let directory = lw(g::mapdir1);
                                    let counted;
                                    if lw(directory + 4) != -666 && lw(directory + 8) != 0 {
                                        copy_menu_item(item, g::mi_dirback_354a70);
                                        counted = lw(0x354840);
                                    } else {
                                        copy_menu_item(item, g::mi_dirempty_354a50);
                                        sw(0x354840, lw(0x354840) + 1);
                                        copy_menu_item(item + 32, g::mi_dirback_354a70);
                                        counted = lw(0x354840);
                                    }
                                    sw(0x354840, counted + 1);
                                    sb(0x3aff78, 4);
                                }
                                4 => {
                                    break 'b5;
                                }
                                _ => {
                                    // state 5: the answer of the card to the load of the map
                                    let state = mcseqGetState();
                                    if state != 1_i64 {
                                        break 'b2;
                                    }
                                    mcseqWindowClose();
                                    let result = mcseqGetResult();
                                    let buffer = lw(g::mmtempbuffer);
                                    if (result as i64) != state {
                                        let front = lw(g::front);
                                        sw(buffer, 0);
                                        sw(front + 20, 1);
                                        sq(sp + 16, 0x350000);
                                        frontStartMessage(0, lw(lw((lw(g::cur_language) << 2) + g::languages) + 972), 0x6464647f, 0);
                                        sb(0x3aff78, 2);
                                        break 'b2;
                                    }
                                    let size = lhu(0x1fc31ae);
                                    let expected = lw(buffer + size - 4);
                                    let good = crcCheck(buffer, size - 4, expected);
                                    let buffer = lw(g::mmtempbuffer);
                                    if good != 0 {
                                        let mut front = lw(g::front);
                                        if lw(buffer) == 0x1324626 {
                                            front = lw(g::front);
                                            if lw(buffer + 4) == 114 {
                                                sw(lw(g::front) + 20, 1);
                                                sw(0x1fc11f8 + 4, lw(0x1fc11f8 + 4) | 128);
                                                break 'b1;
                                            }
                                        }
                                        sw(buffer, 0);
                                        sw(front + 20, 1);
                                        frontStartMessage(0, 0x3a1d58, 0x7f3c3c7f, 0);
                                        return;
                                    }
                                    let front = lw(g::front);
                                    sw(lw(g::mmtempbuffer), 0);
                                    sw(front + 20, 1);
                                    frontStartMessage(0, lw(lw((lw(g::cur_language) << 2) + g::languages) + 968), 0x6464647f, 0);
                                    return;
                                }
                            }
                        }
                        'b24: {
                            if (lw(0x1fc11fc) & 16) == 0 {
                                let card = mcseqGetState();
                                if card >= 6_i64 {
                                    if card < 8_i64 {
                                        break 'b24;
                                    }
                                    let directory = lw(g::mapdir1);
                                    if card == 8_i64 {
                                        if lw(directory + 8) != 0 {
                                            break 'b24;
                                        }
                                    }
                                }
                                if mcseqPollCount() == 0 {
                                    break 'b2;
                                }
                            }
                            let pressed = joyNewBut(0);
                            let page = lw(sp);
                            if (pressed & 128) == 0 {
                                'b27: {
                                    let shown = lhu(page + 4);
                                    sw(0x3aff7c, shown + (lw(g::firstmap_3ae840) - 1));
                                    if shown == 1 {
                                        if (lw(lw(g::front) + 12) & 4) != 0 {
                                            soundStart(140, 0);
                                            let first = lw(g::firstmap_3ae840) - 1;
                                            sw(g::firstmap_3ae840, first);
                                            if first >= 0 {
                                                break 'b27;
                                            }
                                            let total = lw(lw(g::mapdir1) + 8);
                                            let page = lw(sp);
                                            sw(g::firstmap_3ae840, if total < 10 { 0 } else { total - 10 });
                                            sh(page + 4, if total < 11 { total } else { 10 });
                                            break 'b27;
                                        }
                                    }
                                    let front = lw(g::front);
                                    if shown == lw(0x3aff74) {
                                        if (lw(front + 12) & 8) != 0 {
                                            soundStart(140, 0);
                                            let first = lw(g::firstmap_3ae840) + 1;
                                            let wrap = ((lw(lw(g::mapdir1) + 8) - 10) < first) as i32;
                                            sw(g::firstmap_3ae840, first);
                                            if wrap == 0 {
                                                break 'b27;
                                            }
                                            let page = lw(sp);
                                            sw(g::firstmap_3ae840, 0);
                                            sh(page + 4, 1);
                                            break 'b27;
                                        }
                                    }
                                    if (joyNewBut(0) & 16) == 0 {
                                        let pressed = joyNewBut(0);
                                        let directory = lw(g::mapdir1);
                                        if (pressed & 64) != 0 {
                                            let selected = lw(0x3aff7c);
                                            if (selected as u32) < (lw(directory + 8) as u32) {
                                                let status = 0x1fc11f8;
                                                if (lw(status + 4) & 16) == 0 {
                                                    let offset = mmfileOffset(lhu(selected * 40 + directory + 48));
                                                    let strings = lw((lw(g::cur_language) << 2) + g::languages);
                                                    let entry = lw(0x3aff7c) * 40 + lw(g::mapdir1);
                                                    let copy = 0x1fc3188;
                                                    sd(copy, ld(entry + 12));
                                                    sd(copy + 8, ld(entry + 20));
                                                    sd(copy + 16, ld(entry + 28));
                                                    sd(copy + 24, ld(entry + 36));
                                                    sd(copy + 32, ld(entry + 44));
                                                    mcseqLoad(1, lw(strings + 1888), 0, 0x3a1d28, lw(g::mmtempbuffer), lhu(0x1fc31ae), offset);
                                                    sb(0x3aff78, 5);
                                                    break 'b1;
                                                }
                                                if mmfileLoadBuiltinMap(selected) != 0 {
                                                    let word = lw(status + 4);
                                                    let front = lw(g::front);
                                                    sw(status + 4, word | 128);
                                                    sw(front + 20, 1);
                                                    break 'b1;
                                                }
                                                sw(lw(g::front) + 20, 1);
                                                frontStartMessage(0, 0x3a1d58, 0x7f3c3c7f, 0);
                                                return;
                                            }
                                        }
                                        let page = lw(sp);
                                        sq(sp + 16, 0x350000);
                                        menutick_230238(page, g::mapfile_menu_354840);
                                        break 'b2;
                                    }
                                    soundStart(142, 0);
                                    break 'b4;
                                }
                                let front = lw(g::front);
                                sw(front + 12, lw(front + 12) & -61);
                                sb(0x3aff78, 3);
                                break 'b2;
                            }
                        }
                        mmfileDirReset();
                        sb(0x3aff78, 0);
                        break 'b1;
                    }
                    sw(lw(g::front) + 20, 1);
                    break 'b1;
                }
                sb(0x3aff78, 2);
                break 'b1;
            }
        }
        let made = std_menumake(lw(lw(sp) + 8), 0x350000 + 0x4840);
        sw(lw(sp) + 12, made);
    }
}

/// Copy a menu item of 32 bytes from `from` to `to`.
fn copy_menu_item(to: i32, from: i32) {
    let b = ld(from + 8);
    let c = ld(from + 16);
    sd(to, ld(from));
    sd(to + 8, b);
    sd(to + 16, c);
    sd(to + 24, ld(from + 24));
}
