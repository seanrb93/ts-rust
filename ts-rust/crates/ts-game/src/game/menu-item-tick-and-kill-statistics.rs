//! The tick of a menu item (moving the cursor and pressing), and the statistics of a kill.

use crate::*;

/// The tick of the menu `menu` (its chosen item number at +4) of the page `page`: the chosen item moves to the neighbour that the front end's direction flags (4, 8, 16 and 32 of
/// +12) ask for (the four numbers at +16 of the item); the scroll position of the page's list follows it; the buttons 32 and 16 change the value of a slider item (by the amount at +16
/// of the front end, ×655 for a flag 2 value, over 32); button 64 selects the item of a list or goes to its page (item 0x8000 goes back a page), and button 16 goes back a page
/// (@ 0x2062c0).
pub fn menutick_2062c0(menu: i32, page: i32) {
    let number = lhu(menu + 4);
    if number == 0 {
        return;
    }
    let item = menuitem(page, number);
    let front = lw(g::front);
    if item == 0 {
        return;
    }
    for direction in 0..4 {
        if (lw(front + 12) & (4 << direction)) != 0 {
            let candidate = lhu(item + (direction << 1) + 16);
            if candidate != 0 {
                sh(menu + 4, candidate);
                break;
            }
        }
    }
    let chosen = lhu(menu + 4);
    let list = isinlist(page, chosen);
    let player = lw(g::currentplayer);
    if list != 0 {
        let base = list + 12;
        let index = (item - (list + 16)) >> 5;
        let scroll_at = base + lw(player);
        let past_top = index - lbu(scroll_at);
        let before = index - 2;
        if past_top < 2 {
            sb(scroll_at, if -1 >= before { 0 } else { before });
        } else if !(past_top < lhu(list + 2) - 2) {
            sb(scroll_at, index - lbu(list + 2) + 3);
        }
        let count = lhu(list);
        if !(index < count - 2) {
            sb(base + lw(lw(g::currentplayer)), lbu(list) - lbu(list + 2));
        }
    }
    let front_now = lw(g::front);
    if (lw(front_now + 12) & 48) != 0 && (lw(item + 8) & 0x440000) == 0x40000 {
        let value = lw(item + 12);
        let mut amount = lw(front_now + 16);
        if (lw(value + 12) & 2) != 0 {
            amount *= 655;
        }
        let change = amount >> 5;
        if change != 0 {
            gvSetValue(value, lw(value) + change);
            return;
        }
    }
    if (lhu(item + 10) & 64) == 0 {
        if (joyNewBut(lw(g::menujoy)) & 64) != 0 {
            let found = isinlist(page, lhu(menu + 4));
            if found != 0 && (lhu(found + 8) & 1) != 0 {
                menuitemSelect(page, lhu(menu + 4));
                sh(found + 10, lhu(menu + 4));
                return;
            }
        }
        if (lhu(item + 10) & 64) == 0 && (joyNewBut(lw(g::menujoy)) & 64) != 0 && (lhu(item + 10) & 1) != 0 {
            let front_at = lw(g::front);
            if lhu(item + 8) == 0x8000 {
                sw(front_at + 20, lw(front_at + 20) - 1);
                return;
            }
            frontPageSet(lw(lw(g::front) + 20), lw(item + 12));
            return;
        }
    }
    let pad = joyNewBut(lw(g::menujoy));
    let front_at = lw(g::front);
    if (pad & 16) == 0 {
        return;
    }
    sw(front_at + 20, lw(front_at + 20) - 1);
}

/// Whether the statistics slot `id` is kept in this game: the slots below the number of players and characters, or below 4 in the modes 6 and 7 and in games with option flag 32.
fn stats_slot_kept(id: i32) -> bool {
    if ((lw(g::curgameparms + 72) - 6) as u32) >= 2 && (lw(g::curgameparms + 80) & 32) == 0 {
        id < lw(g::lvnumplayers) + lw(g::numactivechars)
    } else {
        id < 4
    }
}

/// Count a kill: the victim's deaths are counted and its streak records are cleared, and if the killer exists and is not the victim, the killer's kills, the kill by character
/// and by gun, the headshot-like count for kind 2 hits (stat 64), the kill streak (the kills in the last 3 seconds out of a ring of 20 times) and its records (stats 5, 6 and 7) are
/// kept; a victim who killed himself counts as a suicide (stat 2); the victim's own kill run (stats 65 to 67) is then closed (@ 0x220bc8).
pub fn StatsKillMade(killer: i32, victim: i32) {
    StatsAdd(1, lw(victim), 1.0_f32);
    let victim_slot;
    if killer == 0 {
        victim_slot = lw(victim);
    } else {
        StatsAdd(0, lw(killer), 1.0_f32);
        StatsAddChar(1, lw(victim), lw(killer), 1.0_f32);
        StatsAddChar(0, lw(killer), lw(victim), 1.0_f32);
        StatsAddGun(1, lw(killer), lw(killer + 260), 1.0_f32);
        if lw(victim + 8) == 2 {
            StatsAdd(64, lw(killer), 1.0_f32);
        }
        victim_slot = lw(victim);
    }
    if victim_slot < 14 && stats_slot_kept(victim_slot) {
        sw(victim_slot * 1368 + (g::matchstats + 1272), 0);
        sw(lw(victim) * 1368 + (g::matchstats + 1360), 0);
        sw(lw(victim) * 1368 + (g::matchstats + 1356), 0);
    }
    if killer != victim {
        StatsAdd(4, lw(victim), 1.0_f32);
        if killer != 0 {
            StatsAdd(3, lw(killer), 1.0_f32);
            let slot = lw(killer);
            if slot < 14 && stats_slot_kept(slot) {
                let times = g::matchstats + 1272;
                let ring = g::matchstats + 1356;
                let mut k = slot;
                if lw(k * 1368 + times) != 0 {
                    let at = k * 1368 + ring;
                    sw(at, (lw(at) + 1).wrapping_rem(20));
                    k = lw(killer);
                }
                let stamps = g::matchstats + 1276;
                let now = ps2float::mul(gameframe_x(), lwc1(0x3ac86c));
                let count_at = k * 1368 + times;
                let position = lw(k * 1368 + ring);
                sw(count_at, lw(count_at) + 1);
                let stamp_at = (position << 2) + k * 1368;
                k = lw(killer);
                swc1(stamp_at + stamps, now);
                let count_f = lw(k * 1368 + times) as f32;
                if lwc1(k * 1368 + (g::matchstats + 20)) < count_f {
                    StatsSet(5, k, count_f);
                    k = lw(killer);
                }
                let mut recent = 0;
                let mut step = 0;
                let mut position = lw(k * 1368 + ring);
                let total = lw(k * 1368 + times);
                let mut at = position << 2;
                let latest = lwc1(at + k * 1368 + stamps);
                if (if total >= 21 { 20 } else { total }) > 0 {
                    let base = k * 1368;
                    let counted = k * 1368 + (ring - 84);
                    loop {
                        if ps2float::sub(latest, 3.0_f32) < lwc1(at + base + stamps) {
                            recent += 1;
                        }
                        let previous = position + 19;
                        let count = lw(counted);
                        step += 1;
                        position = previous.wrapping_rem(20);
                        if step >= (if count >= 21 { 20 } else { count }) {
                            break;
                        }
                        at = previous.wrapping_rem(20) << 2;
                    }
                }
                let recent_f = recent as f32;
                if lwc1(k * 1368 + (g::matchstats + 24)) < recent_f {
                    StatsSet(6, k, recent_f);
                    k = lw(killer);
                }
                let run = g::matchstats + 1360;
                let run_at = k * 1368 + run;
                let base;
                if lw(run_at) != 0 {
                    base = k * 1368;
                } else {
                    sw(run_at, 1);
                    k = lw(killer);
                    base = k * 1368;
                }
                let last = g::matchstats + 1364;
                let factor = lwc1(0x3ac870);
                let quick = ps2float::mul(gameframe_x(), factor) < ps2float::add(lwc1(base + last), 2.0_f32);
                let streak_at = base + run;
                if quick {
                    sw(streak_at, lw(streak_at) + 1);
                    k = lw(killer);
                }
                let streak = lw(k * 1368 + run) as f32;
                let best = StatsGet(7, k);
                if best < streak {
                    let slot = lw(killer);
                    StatsSet(7, slot, lw(slot * 1368 + run) as f32);
                }
                let slot = lw(killer);
                swc1(slot * 1368 + last, ps2float::mul(gameframe_x(), factor));
            }
        }
    } else {
        StatsAdd(2, lw(victim), 1.0_f32);
    }
    let slot = lw(victim);
    if slot < 14 && stats_slot_kept(slot) {
        let run = StatsGet(65, lw(victim));
        let best = StatsGet(66, lw(victim));
        let mut at = lw(victim);
        if run < best {
            let value = StatsGet(65, at);
            StatsSet(66, lw(victim), value);
            at = lw(victim);
        }
        let run = StatsGet(65, at);
        let other = StatsGet(67, lw(victim));
        let mut at = lw(victim);
        if other < run {
            let value = StatsGet(65, at);
            StatsSet(67, lw(victim), value);
            at = lw(victim);
        }
        StatsSet(65, at, 0.0_f32);
    }
}
