//! Enemy behaviours: patrolling a route, the pillar alien that wakes up, and the ambush from the left.

use crate::*;

/// The patrol of the enemy `enemy` (its record at +352): its route is a list of pad references (from the level's set-up: by the enemy's own number in the arcade modes, the list
/// at +2728 for an enemy made by a pad, the first in mode 4); -1 at the place in the list (+3028) means the route starts again; the bot goes to the next pad on its way
/// (`findNextPad`), a target it is aware of makes it stop and look (state 2) or turn (animation 450); at the end of a route it goes on to the next route in the list (@ 0x2c8408).
pub fn enemyPatrol(enemy: i32) {
    let record = lw(enemy + 352);
    let route;
    if lw(g::curgameparms + 72) != 4 {
        let setup = lw(g::setup);
        if (lw(record + 2704) & 16) == 0 {
            let again = lw(g::setup);
            route = lw((lw((lw(record) - lw(g::lvnumplayers)) * 24 + lw(again + 40) + 20) << 3) + lw(again + 48));
        } else {
            route = lw((lw(record + 2728) << 3) + lw(setup + 48));
        }
    } else {
        route = lw(lw(lw(g::setup) + 48));
    }
    enemySetMoveMode(enemy, 1024);
    let entry = lw((lw(record + 3028) << 2) + route);
    let pad_at = record + 680;
    if entry == -1 {
        sw(record + 1688, 0);
        sw(record + 2692, 0);
        let pad = padNumFromExtref(lw(route));
        sw(record + 684, pad);
        sw(record + 3028, 0);
        sw(record + 2724, 0);
    }
    let next = findNextPad(pad_at);
    if lw(g::curgameparms + 72) != 4 {
        let aware = awareOfTarget(enemy);
        if aware == 3 {
            if curAnim(enemy, 450) == 0 {
                setAnim(enemy, 450, 0.25_f32, 1.0_f32);
            }
        } else if aware != 0 {
            let old = lw(record + 2716);
            let speed = lwc1(record + 2856);
            sw(record + 2716, 2);
            swc1(record + 2872, speed);
            sw(record + 2720, old);
            sw(record + 2892, 0);
            return;
        }
    }
    if next == -2 {
        enemySetMoveMode(enemy, 8);
        return;
    }
    if next != -1 {
        let pads = lw(g::levelpads);
        swc1(record + 2872, lwc1(0x3ad9d8));
        let range = lwc1(0x3ad9dc);
        let pad = next * 28 + pads;
        let px = lwc1(pad + 12);
        swc1(record + 2796, px);
        let py = lwc1(pad + 16);
        swc1(record + 2800, py);
        let pz = lwc1(pad + 20);
        swc1(record + 2804, pz);
        let dx = ps2float::sub(lwc1(enemy + 48), px);
        let dy = ps2float::sub(lwc1(enemy + 52), py);
        let dz = ps2float::sub(lwc1(enemy + 56), pz);
        if ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz)) < range {
            let count = lw(record + 1688);
            sw(record + 680, next);
            sw(record + 1688, count - 1);
        }
    } else {
        let place = lw(record + 3028);
        sw(record + 1688, 0);
        let following = place + 1;
        sw(record + 2692, 0);
        sw(record + 3028, following);
        if lw((following << 2) + route) == next {
            sw(record + 3028, 0);
        }
        let pad = padNumFromExtref(lw((lw(record + 3028) << 2) + route));
        sw(record + 684, pad);
        findNextPad(pad_at);
    }
}

/// The pillar alien `enemy`: while asleep (state 0 at +2728) it plays animation 476 and wakes (state 1, target the first player, or the first within 10 squared units of the
/// distances counted) when a player comes within 10; when waking (state 1) it plays 475 with the first gun combination of the level, and when that ends goes to animation 3
/// and the state 8 (@ 0x2ca478).
pub fn enemyPillarAlien(enemy: i32) {
    let frame = Frame::new(32);
    let sp = frame.sp();
    let record = lw(enemy + 352);
    memset(sp, 0, 8);
    let guns = lvGetGunComboNums();
    let players = lw(g::lvnumplayers);
    let mut i = 0;
    let state = lw(record + 2728);
    if players > 0 {
        let list = lw(g::players);
        let x = lwc1(enemy + 48);
        let y = lwc1(enemy + 52);
        let z = lwc1(enemy + 56);
        loop {
            let slot = i << 2;
            let player = i * 1820 + list;
            i += 1;
            let dx = ps2float::sub(lwc1(player + 140), x);
            let dy = ps2float::sub(lwc1(player + 144), y);
            let dz = ps2float::sub(lwc1(player + 148), z);
            swc1(sp + slot, ps2float::add(ps2float::add(ps2float::mul(dx, dx), ps2float::mul(dy, dy)), ps2float::mul(dz, dz)));
            if i >= players {
                break;
            }
        }
    }
    if state == 0 {
        if curAnim(enemy, 476) == 0 {
            setAnim(enemy, 476, 0.0_f32, 0.0_f32);
        }
        let count = lw(g::lvnumplayers);
        if count > 0 {
            let mut near = 1;
            if !(lwc1(sp) <= 1e+02_f32) {
                loop {
                    if near >= count {
                        return;
                    }
                    if lwc1(sp + (near << 2)) <= 1e+02_f32 {
                        break;
                    }
                    near += 1;
                }
                let list = lw(g::players);
                sw(record + 2728, 1);
                sw(record + 2732, 0);
                sw(record + 2892, 0);
                sw(record + 2788, lw(near * 1820 + list + 384));
            } else {
                let list = lw(g::players);
                sw(record + 2728, 1);
                let target = lw(list + 384);
                sw(record + 2732, 0);
                sw(record + 2788, target);
                sw(record + 2892, 0);
            }
        }
    } else if state == 1 {
        if curAnim(enemy, 475) != 0 {
            if curAnimComplete(lw(enemy + 32)) != 0 {
                setAnim(enemy, 3, 0.0_f32, 1.0_f32);
                let old = lw(record + 2716);
                sw(record + 2716, 8);
                swc1(record + 2892, 0.0_f32);
                sw(record + 2720, old);
            }
        } else {
            sw(record + 260, lw(guns));
            setAnim(enemy, 475, 0.0_f32, 1.0_f32);
        }
    }
}

/// The ambush from the left: like `enemyAmbushRight`, but the enemy steps out with one of four animations (608 and 609, or 611 and 612 for a model whose flag at +12 of its second
/// bone record is set; the second of each when its gun has flag 2); when the animation is not done flags 6 are added to +2704 (@ 0x2c8828).
pub fn enemyAmbushLeft(enemy: i32) {
    let record = lw(enemy + 352);
    enemySetMoveMode(enemy, 8);
    swc1(record + 2796, lwc1(enemy + 48));
    swc1(record + 2800, lwc1(enemy + 52));
    let z = lwc1(enemy + 56);
    sw(record + 2780, 0);
    swc1(record + 2804, z);
    if enemyTriggered(enemy) != 0 && curAnim(enemy, 609) == 0 && curAnim(enemy, 608) == 0 && curAnim(enemy, 612) == 0 && curAnim(enemy, 611) == 0 {
        let gun = lw(record + 376);
        if lw(lw(lw(enemy + 32) + 88) + 12) != 0 {
            if (lw(gun * 400 + (GUNS_TABLE + 28)) & 2) == 0 {
                setAnim(enemy, 611, 0.0_f32, 1.0_f32);
            } else {
                setAnim(enemy, 612, 0.0_f32, 1.0_f32);
            }
        } else if (lw(gun * 400 + (GUNS_TABLE + 28)) & 2) == 0 {
            setAnim(enemy, 608, 0.0_f32, 1.0_f32);
        } else {
            setAnim(enemy, 609, 0.0_f32, 1.0_f32);
        }
        return;
    }
    let old;
    if curAnim(enemy, 609) == 0 && curAnim(enemy, 608) == 0 && curAnim(enemy, 612) == 0 && curAnim(enemy, 611) == 0 {
        enemySetMoveMode(enemy, 8);
        enemySetIdleAnimation(enemy);
        if awareOfTarget(enemy) == 0 {
            return;
        }
        old = lw(record + 2716);
    } else {
        if curAnimComplete(lw(enemy + 32)) == 0 {
            sw(record + 2704, lw(record + 2704) | 6);
            return;
        }
        old = lw(record + 2716);
    }
    sw(record + 2716, 2);
    sw(record + 2720, old);
    sw(record + 2892, 0);
}
