//! Restarting the enemies, setting up the surface of water for a prop, the lobster's claws, and what an enemy does with no route.

use crate::*;
use super::random_as_float;

/// Restart the enemies: the count of active characters is the game parameters' (+8), or 0 in the levels 6 and 101 to 104; the tables of the pickup spawns, bags, health, drops,
/// bases and escorts and the explosion data are cleared with their counts; every character's record has its guns, route and wait fields cleared, flag 2 of its flags
/// taken off and its state numbers (+0x11a4 to +0x11ac) reset; the level's triggers (records of 52 bytes at +64 of the set-up) have theirs cleared; and the AI's
/// working values are put to their starts (@ 0x2b9d70).
pub fn enemyRestart() {
    let level = lw(g::lvlevelnum);
    sw(g::totactivespawnedchars, 0);
    if level != 6 && level != 101 && level != 102 && level != 103 && level != 104 {
        sw(g::numactivechars, lw(g::curgameparms + 8));
    } else {
        sw(g::numactivechars, 0);
    }
    memset(0x1fedf00, 0, 800);
    sw(g::checksleepframe, 4);
    sw(g::numtriggerpads, 0);
    sw(g::updateplayer, 0);
    sw(g::checksleepplayer, 0);
    sw(g::updatedistindex, 0);
    sw(g::currAIEventHandler, 0);
    memset(g::weaponspawn, 0, 280);
    sw(g::numPickupWeapons, 0);
    memset(g::bagspawn, 0, 48);
    sw(g::numPickupBags, 0);
    memset(g::healthspawn, 0, 200);
    sw(g::numPickupHealth, 0);
    memset(g::dropspawn, 0, 160);
    sw(g::numPickupDrops, 0);
    memset(g::bases, 0, 80);
    sw(g::numBases, 0);
    memset(g::escort, 0, 20);
    sw(g::numEscorts, 0);
    sw(g::setup, 0);
    sw(g::currentbase, 0);
    sw(g::framecheck, 0);
    sw(g::updatechr, 0);
    memset(g::aiexplosiondata, 0, 320);
    let players = lw(g::lvnumplayers);
    sw(0x3b01ac, 0);
    let mut flag = g::teamflag + 16;
    sw(g::totactivespawnedchars, 0);
    sw(0x3b01a8, 0);
    let mut left = 4;
    let active = lw(g::numactivechars);
    loop {
        sw(flag, 0);
        left -= 1;
        flag -= 4;
        if left < 0 {
            break;
        }
    }
    let mut i = 0;
    if (active + players) > 0 {
        let characters = lw(g::chrinfo);
        loop {
            let next = i + 1;
            let mut words = 32;
            let record = i * 0x1210 + characters;
            sw(record + 260, 0);
            sw(record + 440, 0);
            let mut at = record + 616;
            sw(record + 332, 0);
            sw(record + 484, 0);
            sw(record + 264, 0);
            loop {
                sw(at, 0);
                words -= 1;
                at -= 4;
                if words < 0 {
                    break;
                }
            }
            let record = i * 0x1210 + characters;
            i = next;
            let more = i < lw(g::numactivechars) + lw(g::lvnumplayers);
            let flags = lw(record + 2704);
            sw(record + 0x11ac, 1);
            sw(record + 0x11a4, 0);
            sw(record + 2704, flags & -3);
            sw(record + 0x11a8, 0);
            if !more {
                break;
            }
        }
    }
    let setup = lvGetLevelSetup(lw(g::lvlevelnum));
    sw(g::setup, setup);
    if setup != 0 {
        let mut i = 0;
        if lw(setup + 68) > 0 {
            loop {
                let record = i * 52 + lw(setup + 64);
                i += 1;
                let flags = lw(record + 36);
                sw(record + 40, 0);
                sw(record + 44, 0);
                sw(record + 36, flags & -2);
                sw(record + 48, 0);
                if i >= lw(setup + 68) {
                    break;
                }
            }
        }
    }
    let start = lwc1(0x3ad7d0);
    sw(0x3b01d8, 1);
    swc1(0x3b01cc, start);
    sw(0x3b01dc, 15);
    sw(0x3b01e0, 0);
    sw(0x3b01b0, 0);
    sw(0x3b01e4, 0);
    sw(0x3b01b4, 0);
    sw(0x3b01b8, 0);
    sw(0x3b01bc, 0);
    swc1(0x3b01c0, start);
    sw(0x3b01c4, 0);
    sw(0x3b01c8, 0);
    sw(0x3b01d0, 0);
    sw(0x3b01d4, 0);
}

/// Set up the water of the prop `prop` (kind 315 to 317, the water of the first, second or third surface) for the part `part` (0 or 1): the table of the water's numbers (ten
/// vectors at 0x3a8528 to 0x3a8598 of which one coordinate by the prop's kind is used) gives the periods and speeds in x and z, the amplitude and the reduction
/// (`waterperiodx` and the rest); then for each vertex of the part's last chunk list (`vtxlistWaterxyzPre`: the number of vertices is the first chunk's count at +12 plus the
/// last one's start at +16) a factor is made into the memory `waterMemory[part]` (taken when not restarting): 1 less the distance from the middle over 80 for the kind 316, 1 for 317,
/// and for 315 the product of two ramps over the height and an edge falling off over the width (@ 0x278400).
pub fn vtxlistWaterxyzPre(part: i32, prop: i32) {
    let frame = Frame::new(160);
    let sp = frame.sp();
    let mut count = 0;
    let skeleton = lw(lw(prop + 32));
    let kind = lw(prop + 4);
    let bone_count = lw(skeleton);
    let tables = [0x3a8528, 0x3a8528, 0x3a8538, 0x3a8538, 0x3a8548, 0x3a8558, 0x3a8568, 0x3a8578, 0x3a8588, 0x3a8598];
    for (n, table) in tables.iter().enumerate() {
        let at = sp + (n as i32) * 16;
        sd(at, ld(*table));
        sw(at + 8, lw(*table + 8));
    }
    let index = kind - 315;
    let row = sp + (index << 2);
    let bones = skeleton - bone_count * 80;
    let period_z = lwc1(row + 16);
    let low = lwc1(row + 128);
    let high = lwc1(row + 144);
    let width = lwc1(row + 112) - lwc1(row + 96);
    let speed_x = lwc1(row + 32);
    swc1(g::waterperiodx, lwc1(row));
    let height = high - low;
    swc1(g::waterperiodz, period_z);
    swc1(g::waterspeedx, speed_x);
    let reduction = lwc1(row + 80);
    let speed_z = lwc1(row + 48);
    swc1(g::wateramplitude, lwc1(row + 64));
    swc1(g::waterspeedz, speed_z);
    swc1(g::waterreduction, reduction);
    let mut points = 0;
    'found: {
        let mut chunk;
        let first_points;
        if part == 1 {
            chunk = lw(bones + 12);
            if chunk != 0 {
                first_points = lw(bones + 48);
                if lw(chunk + 20) >= 0 && lw(chunk + 44) >= 0 {
                    loop {
                        chunk += 24;
                        if lw(chunk + 44) < 0 {
                            break;
                        }
                    }
                }
                points = first_points;
                count = lw(chunk + 12) + lw(chunk + 16);
                break 'found;
            }
        }
        if part != 0 {
            break 'found;
        }
        chunk = lw(bones + 8);
        if chunk == 0 {
            break 'found;
        }
        first_points = lw(bones + 24);
        if lw(chunk + 20) >= 0 && lw(chunk + 44) >= 0 {
            loop {
                chunk += 24;
                if lw(chunk + 44) < 0 {
                    break;
                }
            }
        }
        points = first_points;
        count = lw(chunk + 12) + lw(chunk + 16);
    }
    let restarting = lw(g::restart);
    if count != 0 && restarting == 0 {
        let memory = memAlloc((count << 2) + 15 & -16, 1);
        sw((part << 2) + g::waterMemory, memory);
    }
    let mut i = 0;
    if count > 0 {
        let slot = (part << 2) + g::waterMemory;
        loop {
            let x = lwc1(points);
            let z = lwc1(points + 8);
            if index == 1 {
                let at = (i << 2) + lw(slot);
                swc1(at, 1.0_f32 - ((x * x + z * z).abs().psqrt()).pdiv(8e+01_f32));
            } else if index >= 2 {
                if index == 2 {
                    swc1((i << 2) + lw(slot), 1.0_f32);
                }
            } else if index == 0 {
                let low_z = lwc1(sp + 128);
                let along = z - low_z;
                let mut ramp_up = 1.0_f32;
                let share = (along).pdiv(height);
                if share < 0.5_f32 {
                    ramp_up = (along).pdiv((height * 0.5_f32));
                }
                let mut ramp_down = 1.0_f32;
                let top = lwc1(sp + 144);
                if 0.5_f32 < share {
                    ramp_down = ((top - z)).pdiv((height * 0.5_f32));
                }
                swc1((i << 2) + lw(slot), ramp_up * ramp_down * (((x - lwc1(sp + 96))).pdiv(width)));
            }
            i += 1;
            points += 12;
            if i >= count {
                break;
            }
        }
    }
}

/// The tick of the lobster: the angles of its claw (+108) and its two pincers (+112, +116) follow sines of the game's time (the 45 degrees of the claw by a half and a half of a
/// sine, the pincers by 35 degrees of one); unless the prop is flagged 256 the matrices of its bones are made unit and then the three bones (at 192, 320 and 512 bytes in
/// the model's matrices) are turned about their centres by those angles, and the matrices at 320 and 512 are copied to 384 and 576 (@ 0x269fc8).
pub fn lobsterPropTick(prop: i32) {
    let frame = Frame::new(80);
    let sp = frame.sp();
    let time = lw(g::gameframe) as f32;
    let table = g::g_SinTable;
    let state = lw(prop + 352);
    let phase = lwc1(state + 104);
    let scale = lwc1(0x3acbf8);
    let half = 0.5_f32;
    let model = lw(prop + 32);
    let pincer = ((((phase + time * lwc1(0x3acbf4)) * scale + half) as i32 & 2047) << 2) + table;
    swc1(state + 108, (lwc1(((((phase + time * lwc1(0x3acbf0)) * scale + half) as i32 & 2047) << 2) + table) * half + half) * 45.0_f32);
    swc1(state + 112, lwc1(pincer) * 35.0_f32);
    swc1(state + 116, lwc1(pincer) * -35.0_f32);
    obInstSetAnim(model, 1, 0.0_f32);
    if (lw(prop + 16) & 256) != 0 {
        return;
    }
    let mut owner = lw(prop + 32);
    let mut i = 0;
    let mut cursor = owner;
    if lw(lw(owner) + 4) > 0 {
        loop {
            sceVu0UnitMatrix(lw(cursor + 4) + (i << 6));
            i += 1;
            owner = lw(prop + 32);
            cursor = owner;
            if i >= lw(lw(owner) + 4) {
                break;
            }
        }
    }
    let skeleton = lw(owner);
    let first = lw(owner + 4) + 192;
    let to_radians = lwc1(0x3acbfc);
    let half_turn = 1.8e+02_f32;
    let bones = skeleton - lw(skeleton) * 80;
    let centre = lw(bones + 72);
    matrixTrans(first, lwc1(centre), lwc1(centre + 4), lwc1(centre + 8));
    matrixRotZXY(sp, (lwc1(state + 108) * to_radians).pdiv(half_turn), 0.0_f32, 0.0_f32);
    sceVu0MulMatrix(first, first, sp);
    matrixTrans(sp, -lwc1(centre), -lwc1(centre + 4), -lwc1(centre + 8));
    sceVu0MulMatrix(first, first, sp);
    let centre = lw(bones + 72);
    let second_centre = centre + 60;
    let second = lw(lw(prop + 32) + 4) + 320;
    matrixTrans(second, lwc1(centre + 60), lwc1(second_centre + 4), lwc1(second_centre + 8));
    matrixRotZXY(sp, 0.0_f32, (lwc1(state + 112) * to_radians).pdiv(half_turn), 0.0_f32);
    sceVu0MulMatrix(second, second, sp);
    matrixTrans(sp, -lwc1(centre + 60), -lwc1(second_centre + 4), -lwc1(second_centre + 8));
    sceVu0MulMatrix(second, second, sp);
    let centre = lw(bones + 72);
    let third_centre = centre + 48;
    let third = lw(lw(prop + 32) + 4) + 512;
    matrixTrans(third, lwc1(centre + 48), lwc1(third_centre + 4), lwc1(third_centre + 8));
    matrixRotZXY(sp, 0.0_f32, (lwc1(state + 116) * to_radians).pdiv(half_turn), 0.0_f32);
    sceVu0MulMatrix(third, third, sp);
    matrixTrans(sp, -lwc1(centre + 48), -lwc1(third_centre + 4), -lwc1(third_centre + 8));
    sceVu0MulMatrix(third, third, sp);
    let matrices = lw(lw(prop + 32) + 4);
    sceVu0CopyMatrix(matrices + 384, matrices + 320);
    let matrices = lw(lw(prop + 32) + 4);
    sceVu0CopyMatrix(matrices + 576, matrices + 512);
}

/// What the enemy `enemy` does when it has no route (its state at +2728): its goal is set to its target's place (when it has a target) and its move mode to 8. State 0 goes to 3 for
/// a random time of 0.5 to 1.5 seconds and finds the nearest pad; state 1 (following) goes to 2 or 4 when it is no more than 5 away, state 2 looks for a bag to go for in the modes 1, 2
/// and 5, or else a target to follow (state 1 and then a route to the target's pad), state 3 waits for the time and goes to 4, and state 4 goes back to 3 with a new time (@ 0x2caf78).
pub fn enemyNoRoute(enemy: i32) {
    let record = lw(enemy + 352);
    let target = lw(record + 2788);
    if target != 0 {
        swc1(record + 2796, lwc1(target + 48));
        swc1(record + 2800, lwc1(target + 52));
        swc1(record + 2804, lwc1(target + 56));
    }
    enemySetMoveMode(enemy, 8);
    let state = lw(record + 2728);
    if (state as u32) >= 5 {
        return;
    }
    let scale = 2.3283064e-10_f32;
    match state {
        0 => {
            sw(record + 2728, 3);
            let unit = random_as_float() * scale;
            sw(record + 2892, 0);
            let time = lwc1(record + 2892);
            swc1(record + 2732, unit + unit + 0.5_f32);
            let pad = closestpad(enemy + 48, time);
            sw(record + 680, pad);
            return;
        }
        1 => {
            let moved = fptodp(lwc1(record + 2780));
            let mut distance = moved;
            if dpcmp(distance, 0_i64) < 0 {
                distance = dpsub(0_i64, distance);
            }
            if dpcmp(distance, 0x4014000000000000_i64) < 0 {
                sw(record + 2728, if lw(record + 2772) == 0 { 2 } else { 4 });
            }
            if lw(record) != lw(g::updatedistindex) {
                return;
            }
            if lw(record + 2772) != 0 {
                return;
            }
        }
        2 => {
            let mode = lw(g::curgameparms + 72);
            if mode == 1 || mode == 5 || mode == 2 {
                let bags = lw(g::numPickupBags);
                let mut i = 0;
                if bags > 0 {
                    loop {
                        let bag = lw((i << 3) + (g::bagspawn + 4));
                        let bag_record = lw(bag + 352);
                        i += 1;
                        if lw(bag_record + 68) == 1 && lw(bag_record + 16) != lw(record + 24) && (lw(bag + 16) & 1) != 0 {
                            let old = lw(record + 2716);
                            sw(record + 2716, 128);
                            sw(record + 2720, old);
                            sw(record + 2892, 0);
                            return;
                        }
                        if i >= bags {
                            break;
                        }
                    }
                }
            }
            let found = enemyCycleThroughTargets(enemy);
            if found != 0 {
                sw(record + 2788, found);
            }
            let target = lw(record + 2788);
            if target == 0 {
                return;
            }
            let target_record = lw(target + 352);
            sw(record + 2728, 1);
            let pad = lw(target_record + 680);
            sw(record + 684, pad);
            if pad < 0 {
                return;
            }
            sw(record + 1688, 0);
            sw(record + 2692, 0);
            let next = findNextPad(record + 680);
            if next < 0 {
                return;
            }
            let old = lw(record + 2716);
            sw(record + 2716, 8);
            sw(record + 2720, old);
            sw(record + 2892, 0);
            return;
        }
        3 => {
            if lw(record + 2788) != 0 && lw(record + 2772) != 0 {
                if !(lwc1(record + 2732) < lwc1(record + 2892)) {
                    return;
                }
                let flags = lw(record + 2704);
                sw(record + 2728, 4);
                sw(record + 2704, flags | 2);
                return;
            }
        }
        _ => {
            if (lw(record + 2704) & 2) != 0 {
                return;
            }
            sw(record + 2728, 3);
            let unit = random_as_float() * scale;
            sw(record + 2892, 0);
            swc1(record + 2732, unit * 0.5_f32 + lwc1(0x3ada48));
            return;
        }
    }
    sw(record + 2728, 2);
}
