//! The choice of what an enemy goes for.

use super::particle_spawners::random_as_float;
use crate::*;

/// The squared distance between the positions (+48, +52, +56) of two objects.
fn distance_squared(a: i32, b: i32) -> f32 {
    let dx = lwc1(a + 48) - lwc1(b + 48);
    let dy = lwc1(a + 52) - lwc1(b + 52);
    let dz = lwc1(a + 56) - lwc1(b + 56);
    dx * dx + dy * dy + dz * dz
}

/// Choose the target of the enemy `a0` by the game kind (`curgameparms + 72`): kind 1 takes the bag (or, with a team flag, the nearest chase of a mate), kinds 2 and 5 look for the
/// pickups and bags or the chase of the other team (and for a defend pad), kind 3 the escort or the nearest member of the escort team, kind 4 a base or a player, kind 6 asks
/// `suggestTarget`, and the rest keep none; with no target the enemy checks its health and picks the player with the best ratio of kills to distance (@ 0x2c5380). The move mode
/// is set from what the target is.
#[allow(unused_assignments)]
pub fn selectTarget(a0: i32) {
    let frame = Frame::new(608);
    let sp = frame.sp();
    let s2 = lw(a0 + 352);
    let old_target = lw(s2 + 2788);
    let kind = lw(g::curgameparms + 72) - 1;
    sw(s2 + 2788, 0);
    let mut target = 0;
    let mut v0_1 = 0;
    let mut v1_8 = 0;
    let mut v0_12 = 0;
    'b1: {
        'b2: {
            'b3: {
                'b4: {
                    'b5: {
                        'b6: {
                            'b7: {
                                if (kind as u32) >= 7 {
                                    target = 0;
                                } else {
                                    'b8: {
                                        'b9: {
                                            match kind {
                                                0 => {
                                                    let bag = lw(g::bagspawn + 4);
                                                    let chars = lw(g::numactivechars);
                                                    if (lw(lw(bag + 352) + 68) & 4) != 0 {
                                                        let players = lw(g::lvnumplayers);
                                                        let mut i = 0;
                                                        if chars + players > 0 {
                                                            let info = lw(g::chrinfo);
                                                            loop {
                                                                let chr = i * 0x1210 + info;
                                                                i += 1;
                                                                if lw(chr + 484) != 0 {
                                                                    sw(s2 + 2788, lw(chr + 3020));
                                                                }
                                                                if i >= chars + players {
                                                                    break;
                                                                }
                                                            }
                                                        }
                                                        target = lw(s2 + 2788);
                                                        if (lw(g::curgameparms + 80) & 1) == 0 {
                                                            break 'b7;
                                                        }
                                                        let chosen = target;
                                                        if target == 0 {
                                                            break 'b2;
                                                        }
                                                        let team = lw(s2 + 24);
                                                        if lw(lw(target + 352) + 24) != team {
                                                            break 'b7;
                                                        }
                                                        i = 0;
                                                        let mut best = lwc1(0x3afd74);
                                                        let mut nearest = 0;
                                                        if chars + players > 0 {
                                                            let info = lw(g::chrinfo);
                                                            loop {
                                                                let chr = i * 0x1210 + info;
                                                                if lw(chr + 24) == team {
                                                                    i += 1;
                                                                } else if (lw(chr + 2708) & 32) != 0 {
                                                                    i += 1;
                                                                } else {
                                                                    let theirs = lw(chr + 2788);
                                                                    if theirs != chosen {
                                                                        i += 1;
                                                                    } else {
                                                                        i += 1;
                                                                        if lw(chr + 2716) == 2 {
                                                                            let mate = lw(chr + 3020);
                                                                            let d = distance_squared(theirs, mate);
                                                                            if d < best {
                                                                                best = d;
                                                                                nearest = mate;
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                                if i >= chars + players {
                                                                    break;
                                                                }
                                                            }
                                                        }
                                                        if nearest == 0 {
                                                            break 'b7;
                                                        }
                                                        target = nearest;
                                                        sw(s2 + 2788, nearest);
                                                        break 'b7;
                                                    }
                                                    target = bag;
                                                    let flags = lw(s2 + 2704) | 64;
                                                    sw(s2 + 2788, bag);
                                                    sw(s2 + 2704, flags & -129);
                                                    break 'b7;
                                                }
                                                1 | 4 => {
                                                    let mut drops = lw(g::numPickupDrops);
                                                    if lw(s2 + 484) == 0 {
                                                        target = lw(s2 + 2788);
                                                        let mut best = lwc1(0x3afd74);
                                                        if target == 0 {
                                                            let team = lw(s2 + 24);
                                                            let mut bags = lw(g::numPickupBags);
                                                            if (lw((lw(s2 + 24) << 2) + g::teamflag) & 1) == 0 {
                                                                let mut i = 0;
                                                                if target < bags {
                                                                    let mut at = i << 3;
                                                                    loop {
                                                                        let bag = lw(at + (g::bagspawn + 4));
                                                                        let record = lw(bag + 352);
                                                                        if lw(record + 68) != 1 {
                                                                            i += 1;
                                                                        } else if lw(record + 16) == team {
                                                                            i += 1;
                                                                        } else {
                                                                            i += 1;
                                                                            if (lw(bag + 16) & 1) != 0 {
                                                                                let d = distance_squared(a0, bag);
                                                                                if d < best {
                                                                                    let flags = lw(s2 + 2704);
                                                                                    sw(s2 + 2788, bag);
                                                                                    best = d;
                                                                                    sw(s2 + 2704, (flags | 64) & -129);
                                                                                    bags = lw(g::numPickupBags);
                                                                                }
                                                                            }
                                                                        }
                                                                        at = i << 3;
                                                                        if i >= bags {
                                                                            break;
                                                                        }
                                                                    }
                                                                }
                                                                let roll = random_as_float();
                                                                target = 0;
                                                                if !(0.5_f32 < roll * 2.3283064e-10_f32) {
                                                                    target = lw(s2 + 2788);
                                                                } else {
                                                                    sw(s2 + 2788, 0);
                                                                }
                                                            }
                                                        }
                                                        let best = lwc1(0x3afd74);
                                                        if target != 0 {
                                                            break 'b6;
                                                        }
                                                        let team = lw(s2 + 24);
                                                        let players = lw(g::lvnumplayers);
                                                        if (lw((lw(s2 + 24) << 2) + g::teamflag) & 4) != 0 {
                                                            'b23: {
                                                                let chars = lw(g::numactivechars);
                                                                let mut i = 0;
                                                                if target < chars + players {
                                                                    let info = lw(g::chrinfo);
                                                                    let mut chr;
                                                                    loop {
                                                                        chr = i * 0x1210 + info;
                                                                        let holder = lw(chr + 484);
                                                                        if holder == 0 {
                                                                            i += 1;
                                                                        } else {
                                                                            if lw(lw(holder + 352) + 16) == team {
                                                                                break;
                                                                            }
                                                                            i += 1;
                                                                        }
                                                                        if i >= chars + players {
                                                                            break 'b23;
                                                                        }
                                                                    }
                                                                    let mate = lw(chr + 3020);
                                                                    if distance_squared(a0, mate) < best {
                                                                        sw(s2 + 2788, mate);
                                                                    }
                                                                }
                                                            }
                                                            let roll = random_as_float();
                                                            target = 0;
                                                            if !(0.5_f32 < roll * 2.3283064e-10_f32) {
                                                                target = lw(s2 + 2788);
                                                            } else {
                                                                sw(s2 + 2788, 0);
                                                            }
                                                        }
                                                        if target != 0 {
                                                            sw(s2 + 1688, 0);
                                                            break 'b5;
                                                        }
                                                        let mut i = lw(g::lvnumplayers);
                                                        let mut tries = 0;
                                                        let mut chars = lw(g::numactivechars);
                                                        let mut first = i;
                                                        if i < chars + i {
                                                            let mut at = i * 0x1210;
                                                            'b31: {
                                                                'l29: loop {
                                                                    'b32: {
                                                                        let info = lw(g::chrinfo);
                                                                        if i != lw(s2) {
                                                                            let chr = at + info;
                                                                            if lw(s2 + 24) != lw(chr + 24) {
                                                                                i += 1;
                                                                                break 'b32;
                                                                            }
                                                                            'b34: {
                                                                                let state;
                                                                                'b35: {
                                                                                    let other = at + info;
                                                                                    if lw(chr + 2716) == 8 {
                                                                                        let mine = lw(s2 + 2788);
                                                                                        if mine == 0 {
                                                                                            state = lw(other + 2716);
                                                                                            break 'b35;
                                                                                        }
                                                                                        if lw(mine + 8) == 64 {
                                                                                            break 'b34;
                                                                                        }
                                                                                    }
                                                                                    state = lw(at + info + 2716);
                                                                                }
                                                                                if state != 2 {
                                                                                    i += 1;
                                                                                    break 'b32;
                                                                                }
                                                                            }
                                                                            tries += 1;
                                                                            if (tries & 1) != 0 {
                                                                                i += 1;
                                                                                break 'b32;
                                                                            }
                                                                            let pad = enemyChooseDefendPad(a0);
                                                                            sw(s2 + 684, pad);
                                                                            if pad != -1 {
                                                                                break 'l29;
                                                                            }
                                                                            first = lw(g::lvnumplayers);
                                                                            chars = lw(g::numactivechars);
                                                                        }
                                                                        i += 1;
                                                                    }
                                                                    at += 0x1210;
                                                                    if i >= chars + first {
                                                                        break 'b31;
                                                                    }
                                                                }
                                                                v1_8 = lw(s2 + 2716);
                                                                v0_12 = 4;
                                                                break 'b4;
                                                            }
                                                            target = lw(s2 + 2788);
                                                        }
                                                        if target != 0 {
                                                            sw(s2 + 1688, 0);
                                                            break 'b5;
                                                        }
                                                        break 'b8;
                                                    }
                                                    let mut i = 0;
                                                    if drops <= 0 {
                                                        target = lw(s2 + 2788);
                                                        break 'b7;
                                                    }
                                                    let team = lw(s2 + 24);
                                                    let mut at = i << 3;
                                                    loop {
                                                        let drop = lw(at + (g::dropspawn + 4));
                                                        i += 1;
                                                        if lw(lw(drop + 352) + 16) == team {
                                                            let flags = lw(s2 + 2704);
                                                            sw(s2 + 2788, drop);
                                                            sw(s2 + 2704, flags & -65 | 128);
                                                            drops = lw(g::numPickupDrops);
                                                        }
                                                        at = i << 3;
                                                        if i >= drops {
                                                            break;
                                                        }
                                                    }
                                                    target = lw(s2 + 2788);
                                                    break 'b7;
                                                }
                                                2 => {
                                                    break 'b8;
                                                }
                                                3 => {
                                                    if (lw((lw(s2 + 24) << 2) + g::teamflag) & 16) == 0 {
                                                        let escort = lw(g::escort);
                                                        target = escort;
                                                        sw(s2 + 2788, escort);
                                                        break 'b7;
                                                    }
                                                    v0_1 = lw(g::escort);
                                                    target = v0_1;
                                                    if old_target != v0_1 {
                                                        break 'b9;
                                                    }
                                                    let players = lw(g::lvnumplayers);
                                                    let mut count = 0;
                                                    let chars = lw(g::numactivechars);
                                                    let mut i = 0;
                                                    let mut best;
                                                    if (chars + lw(g::lvnumplayers)) <= 0 {
                                                        best = lwc1(0x3afd78);
                                                    } else {
                                                        let info = lw(g::chrinfo);
                                                        loop {
                                                            let chr = i * 0x1210 + info;
                                                            i += 1;
                                                            if (lw((lw(chr + 24) << 2) + g::teamflag) & 32) != 0 {
                                                                if (lw(chr + 2708) & 32) == 0 {
                                                                    let slot = sp + (count << 2);
                                                                    count += 1;
                                                                    sw(slot, lw(chr + 3020));
                                                                }
                                                            }
                                                            if i >= chars + players {
                                                                break;
                                                            }
                                                        }
                                                        best = lwc1(0x3afd78);
                                                    }
                                                    if count <= 0 {
                                                        target = lw(s2 + 2788);
                                                    } else {
                                                        let mut left = count;
                                                        let mut slot = sp;
                                                        loop {
                                                            let mate = lw(slot);
                                                            let closeness = (1.0_f32).pdiv(distance_squared(a0, mate).abs().psqrt());
                                                            left -= 1;
                                                            if best < closeness {
                                                                best = closeness;
                                                                sw(s2 + 2788, lw(slot));
                                                            }
                                                            slot += 4;
                                                            if left == 0 {
                                                                break;
                                                            }
                                                        }
                                                        target = lw(s2 + 2788);
                                                    }
                                                    if target != 0 {
                                                        sw(s2 + 1688, 0);
                                                        break 'b5;
                                                    }
                                                    v0_1 = lw(g::escort);
                                                }
                                                5 => {
                                                    if lw(g::numBases) > 0 {
                                                        let base = lw((lw(g::currentbase) << 3) + (g::bases + 4));
                                                        target = base;
                                                        sw(s2 + 2788, base);
                                                        break 'b7;
                                                    }
                                                    let roll = newrnd();
                                                    let players = lw(g::lvnumplayers);
                                                    if players == 0 {
                                                        trap("break");
                                                    }
                                                    v0_1 = lw(((roll as i32 as u32) % (players as u32)) as i32 * 1820 + lw(g::players) + 384);
                                                }
                                                6 => {
                                                    let first = lw(lw(g::players) + 384);
                                                    v0_1 = suggestTarget(a0, first, 0);
                                                    target = v0_1;
                                                    break 'b9;
                                                }
                                                _ => unreachable!(),
                                            }
                                            target = v0_1;
                                        }
                                        sw(s2 + 2788, v0_1);
                                        break 'b7;
                                    }
                                    let health = enemyCheckHealth(a0);
                                    if health != -1 {
                                        v1_8 = lw(s2 + 2716);
                                        sw(s2 + 2716, 64);
                                        sw(s2 + 684, health);
                                        break 'b3;
                                    }
                                    let chars = lw(g::numactivechars);
                                    let mut count = 0;
                                    let players = lw(g::lvnumplayers);
                                    let mut i = 0;
                                    let mut limit = lw(g::lvnumplayers);
                                    let mut best;
                                    if (lw(g::numactivechars) + lw(g::lvnumplayers)) <= 0 {
                                        best = lwc1(0x3afd74);
                                    } else {
                                        let info = lw(g::chrinfo);
                                        loop {
                                            'b52: {
                                                let chr = i * 0x1210 + info;
                                                if lw(chr + 2708) == 32 {
                                                    limit = players;
                                                } else {
                                                    let is_player = (i < limit) as i32;
                                                    if lw(chr + 24) != lw(s2 + 24) {
                                                        if is_player != 0 {
                                                            limit = players;
                                                            if 4.0_f32 < lwc1(i * 1820 + lw(g::players) + 880) {
                                                                break 'b52;
                                                            }
                                                        }
                                                        let slot = sp + (count << 2);
                                                        count += 1;
                                                        sw(slot, lw(chr + 3020));
                                                    }
                                                    limit = players;
                                                }
                                            }
                                            i += 1;
                                            if i >= chars + limit {
                                                break;
                                            }
                                        }
                                        best = lwc1(0x3afd74);
                                    }
                                    if count <= 0 {
                                        target = lw(s2 + 2788);
                                    } else {
                                        let mut left = count;
                                        let mut slot = sp;
                                        loop {
                                            let enemy = lw(slot);
                                            let d = distance_squared(a0, enemy);
                                            let kills = chrCountKills(lw(enemy + 352));
                                            let ratio = ((if kills != 0 { kills } else { 1 }) as f32).pdiv(d.abs().psqrt());
                                            left -= 1;
                                            if ratio < best {
                                                best = ratio;
                                                sw(s2 + 2788, lw(slot));
                                            }
                                            slot += 4;
                                            if left == 0 {
                                                break;
                                            }
                                        }
                                        target = lw(s2 + 2788);
                                    }
                                }
                            }
                            if target == 0 {
                                break 'b2;
                            }
                        }
                        sw(s2 + 1688, 0);
                    }
                    sw(s2 + 2692, 0);
                    let object_kind = lw(target + 8);
                    if object_kind == 8 {
                        sw(s2 + 684, lw(lw(target + 352) + 680));
                    } else if object_kind >= 9 {
                        if object_kind == 64 {
                            sw(s2 + 684, lw(lw(target + 352) + 36));
                        } else if object_kind == 256 {
                            let record = lw(target + 352);
                            sw(s2 + 684, lw(record + 100));
                        }
                    } else {
                        let base = lw(g::currentbase);
                        if object_kind == 1 {
                            sw(s2 + 684, lw((base << 3) + g::bases));
                        }
                    }
                    v1_8 = lw(s2 + 2716);
                    v0_12 = 8;
                }
                sw(s2 + 2716, v0_12);
            }
            sw(s2 + 2720, v1_8);
            sw(s2 + 2892, 0);
            break 'b1;
        }
        enemySetMoveMode(a0, 8);
    }
}
