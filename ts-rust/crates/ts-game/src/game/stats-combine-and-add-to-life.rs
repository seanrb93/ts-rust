//! Combining the stats of the signed-on players, and adding a game's stats to the player's life totals.

use crate::*;

/// Add `value` to the whole number kept in the float slot at `at` (the slot is read as an integer, made a float, added to and made an integer again).
fn int_add(at: i32, value: f32) {
    swc1(at, f32::from_bits((lwc1(at).to_bits() as i32 as f32 + value) as i32 as u32));
}

/// Make the stats of the current game shown (the pointer at 0x3aff04): with flag 0x2000 of the game's options new stats (`statsNew`, at 0x1fbc168); for two players or more the stats
/// of the first `count` signed-on players added together at 0x1fbc168 (the 80 words, the 30 level records of 36 bytes with their lowest three times, and the 27 challenge records
/// whose best value is kept by the challenge's condition); for one player that player's stats (or 0x1fbc168 when there is no one signed on); the cheats are updated after the last two
/// cases (@ 0x21fc88).
pub fn statsCombine(count: i32) {
    let shown;
    if (lw(g::curgameparms + 80) & 0x2000) == 0 {
        if count >= 2 {
            memset(0x1fbc168, 0, 2804);
            let mut player = 0;
            if count > 0 {
                loop {
                    let next = player + 1;
                    let slot = signonGet(player);
                    let stats = lw(g::signonarray) + slot * 2936 + 132;
                    let mut left = 79;
                    let mut from = stats;
                    let mut to = 0x1fbc168;
                    loop {
                        left -= 1;
                        let value = lw(from);
                        from += 4;
                        sw(to, lw(to) + value);
                        to += 4;
                        if left < 0 {
                            break;
                        }
                    }
                    let mut level = 0;
                    let mut offset = level * 36;
                    loop {
                        let after = level + 1;
                        let total = offset + 0x1fbc2a8;
                        let record = stats + (offset + 320);
                        let kills = lw(total + 32);
                        let first = lw(total + 4);
                        sw(total, lw(total) + lw(record));
                        sw(total + 32, kills + lw(record + 32));
                        sw(total + 4, first + lw(record + 4));
                        let mut best_from = record + 8;
                        let mut best_to = total + 8;
                        let mut i = 0;
                        loop {
                            let value = lw(best_from);
                            if value != 0 {
                                let old = lw(best_to);
                                if !(old != 0 && (value as u32) >= (old as u32)) {
                                    sw(best_to, value);
                                }
                            }
                            i += 1;
                            best_to += 4;
                            best_from += 4;
                            if i >= 3 {
                                break;
                            }
                        }
                        let mut best_to = total + 20;
                        let mut best_from = record + 20;
                        let mut i = 0;
                        loop {
                            let value = lw(best_from);
                            if value != 0 {
                                let old = lw(best_to);
                                if !(old != 0 && (value as u32) >= (old as u32)) {
                                    sw(best_to, value);
                                }
                            }
                            i += 1;
                            best_to += 4;
                            best_from += 4;
                            if i >= 3 {
                                break;
                            }
                        }
                        level = after;
                        offset = level * 36;
                        if level >= 30 {
                            break;
                        }
                    }
                    let mut challenge = 0;
                    loop {
                        let step = challenge << 3;
                        let total = step + 0x1fbc6e0;
                        let mut sum = lw(total);
                        let record = stats + (step + 1400);
                        if sum != 0 {
                            let added = lw(record);
                            sum += added;
                            if added != 0 {
                                let theirs = lw(record + 4);
                                let mut best = lw(total + 4);
                                let condition = challengeGetScoreCondition(challenge);
                                let kind = lw(condition + 12);
                                if (kind as u32) < 5 {
                                    match kind {
                                        0 | 1 => {
                                            best = if theirs < best { theirs } else { best };
                                        }
                                        2 => {}
                                        _ => {
                                            best = if best < theirs { theirs } else { best };
                                        }
                                    }
                                }
                                sw(total + 4, best);
                                sum = lw(total) + lw(record);
                            }
                        } else {
                            sw(total + 4, lw(record + 4));
                            sum = lw(record) + sum;
                        }
                        challenge += 1;
                        sw(total, sum);
                        if challenge >= 27 {
                            break;
                        }
                    }
                    player = next;
                    if player >= count {
                        break;
                    }
                }
            }
            shown = 0x1fbc168;
        } else {
            let slot = signonGet(0);
            if slot < 0 {
                sw(0x3aff04, 0x1fbc168);
                return;
            }
            shown = lw(g::signonarray) + slot * 2936 + 132;
        }
        sw(0x3aff04, shown);
        cheatsUpdate();
        return;
    }
    statsNew(0x1fbc168);
    sw(0x3aff04, 0x1fbc168);
}

/// Add the stats of the player `slot` for the game just played to the life totals `life`: the time (+4), games and kinds of whole numbers and maximums (+8 to +60), the 33 guns'
/// two stats from +64 and +200, a few more (+196, +332, +336, +344) and the 48 medal flags from +348, each set (unless the game is of mode 6 or 7 or has option flag 32) when the
/// medal list of the slot (3 numbers of `medals`) has its number (@ 0x2244d0).
pub fn StatsAddToLife(life: i32, slot: i32) {
    sw(life + 4, lw(life + 4) + lw(g::gameframe));
    let value = StatsGet(0, slot);
    int_add(life + 8, value);
    let value = StatsGet(1, slot);
    int_add(life + 12, value);
    let value = StatsGet(5, slot);
    if (lwc1(life + 16).to_bits() as i32 as f32) < value {
        let value = StatsGet(5, slot);
        swc1(life + 16, f32::from_bits(value as i32 as u32));
    }
    let value = StatsGet(6, slot);
    if (lwc1(life + 20).to_bits() as i32 as f32) < value {
        let value = StatsGet(6, slot);
        swc1(life + 20, f32::from_bits(value as i32 as u32));
    }
    let value = StatsGet(7, slot);
    if (lwc1(life + 24).to_bits() as i32 as f32) < value {
        let value = StatsGet(7, slot);
        swc1(life + 24, f32::from_bits(value as i32 as u32));
    }
    let value = StatsGet(8, slot);
    int_add(life + 28, value);
    let value = StatsGet(9, slot);
    int_add(life + 32, value);
    let value = StatsGet(13, slot);
    swc1(life + 36, lwc1(life + 36) + value);
    let value = StatsGet(15, slot);
    swc1(life + 40, lwc1(life + 40) + value);
    let value = StatsGet(16, slot);
    swc1(life + 44, lwc1(life + 44) + value);
    let value = StatsGet(17, slot);
    swc1(life + 48, lwc1(life + 48) + value);
    let high = StatsGet(23, slot);
    let low = StatsGet(24, slot);
    int_add(life + 52, high - low);
    let value = StatsGet(24, slot);
    int_add(life + 56, value);
    let value = StatsGet(32, slot);
    int_add(life + 60, value);
    let mut gun = 0;
    let mut first = life + 64;
    let mut second = life + 200;
    loop {
        let value = StatsGetGun(0, slot, gun);
        int_add(first, value);
        gun += 1;
        let other = StatsGetGun(3, slot, gun - 1);
        first += 4;
        swc1(second, lwc1(second) + other);
        second += 4;
        if gun >= 33 {
            break;
        }
    }
    let value = StatsGet(62, slot);
    int_add(life + 196, value);
    let value = StatsGet(63, slot);
    swc1(life + 332, lwc1(life + 332) + value);
    let value = StatsGet(64, slot);
    int_add(life + 336, value);
    let value = StatsGet(46, slot);
    swc1(life + 344, lwc1(life + 344) + value);
    let mut medal = 0;
    loop {
        let flag = life + 348 + medal;
        let mut at = slot * 12 + g::medals;
        let mut left = 2;
        loop {
            if lw(at) == medal {
                let parms = g::curgameparms;
                if !(((lw(parms + 72) - 6) as u32) < 2) && (lw(parms + 80) & 32) == 0 {
                    sb(flag, 1);
                }
            }
            left -= 1;
            at += 4;
            if left < 0 {
                break;
            }
        }
        medal += 1;
        if medal >= 48 {
            break;
        }
    }
}
