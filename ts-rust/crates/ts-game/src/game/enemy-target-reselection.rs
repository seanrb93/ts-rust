//! Choosing a new target for a computer character.

use crate::*;

/// Send the enemy to state 128 (the old state kept at +2720, the timer at +2892 cleared); returns 1.
fn go_to_state_128(state: i32) -> i32 {
    let old = lw(state + 2716);
    sw(state + 2716, 128);
    sw(state + 2720, old);
    sw(state + 2892, 0);
    1
}

/// The enemy has taken a new target: its two timers (+2692 and +1688) are cleared; returns 1.
fn new_target_chosen(state: i32) -> i32 {
    sw(state + 2692, 0);
    sw(state + 1688, 0);
    1
}

/// The last choice of a target: the character whose number is kept at +2776 (the one that last hurt it) becomes the target when it is on another team, not in state 32, a
/// player (flag at +612 of its player record) and either near (the float at +880 of that record under 4) or the current target is not a player's character that is in the
/// game; else, when it was hurt by a character (+0x11b8 not 0) of another team (+0x11bc), that one becomes the target with kind 2 and the flag at +2772 is set (@ part of 0x2c0ea8).
fn reselect_by_attacker(chr: i32, state: i32) -> i32 {
    let attacker = lw(state + 2776);
    if attacker >= 0 {
        let target = lw(state + 2788);
        let record = attacker * 0x1210 + lw(g::chrinfo);
        let prop = lw(record + 3020);
        if target != prop && lw(record + 24) != lw(state + 24) && lw(record + 2708) != 32 {
            let player = attacker * 1820 + lw(g::players);
            if lw(player + 612) != 0 {
                let players = lw(g::lvnumplayers);
                let target_is_player = lw(target + 8) == 8 && lw(lw(target + 352)) < players;
                if !target_is_player && lwc1(player + 880) < 4.0_f32 {
                    let new = suggestTarget(chr, prop, 1);
                    sw(state + 2788, new);
                    return new_target_chosen(state);
                }
            }
        }
    }
    if lw(state + 0x11b8) == 0 {
        return 0;
    }
    let hurter = lw(state + 0x11bc) * 0x1210 + lw(g::chrinfo);
    if lw(state + 24) == lw(hurter + 24) {
        return 0;
    }
    let new = suggestTarget(chr, lw(hurter + 3020), 2);
    sw(state + 2788, new);
    sw(state + 2772, 1);
    1
}

/// The part for the game modes 7 and 6: in the mode 7, when the target is in state 32 it is chosen again from the first player's character; then the last choice above.
fn reselect_mode_7(chr: i32, state: i32) -> i32 {
    if lw(g::curgameparms + 72) == 7 {
        let target = lw(state + 2788);
        if target != 0 && lw(lw(target + 352) + 2708) == 32 {
            let new = suggestTarget(chr, lw(lw(g::players) + 384), 0);
            sw(state + 2788, new);
            sw(state + 1688, 0);
            sw(state + 2692, 0);
        }
    }
    reselect_by_attacker(chr, state)
}

/// Choose a new target for the computer character `chr`, by the game mode (the number at +72 of the game parameters, 1 to 7); returns 1 if it changed what it is doing (its
/// target, its drop, or its state), else 0. Mode 1: the character goes to state 128 when its target (a prop of kind 64 that is not what it is, or of kind 8) is not worth
/// following, by the team of the player's characters and who is already chasing it. Modes 2 and 5: a character that carries a player's flag chooses a drop; a target of kind
/// 64 that holds a flag of its own makes it go to state 128. Mode 6: it walks through the bases while their health is at least 1 and takes each as its target. Modes 6 and
/// 7, and 3: the choice by the last attacker (@ 0x2c0ea8).
pub fn enemyReselectTarget(chr: i32) -> i32 {
    let mode = lw(g::curgameparms + 72) - 1;
    let state = lw(chr + 352);
    if (mode as u32) >= 7 {
        return 0;
    }
    match mode {
        0 => {
            if lw(state + 484) != 0 {
                return 0;
            }
            let target = lw(state + 2788);
            let wants_to_leave: bool;
            if lw(target + 8) == 64 && lw(lw(target + 352) + 68) != 1 {
                wants_to_leave = true;
            } else if lw(target + 8) != 8 {
                wants_to_leave = false;
            } else {
                let active = lw(g::numactivechars);
                if (lw(g::curgameparms + 80) & 1) == 0 {
                    wants_to_leave = (lw(lw(target + 352) + 484) as u32) < 1;
                } else {
                    let total = active + lw(g::lvnumplayers);
                    let mut leader = 0;
                    let mut same_team_leader = 0;
                    let mut chased = 0;
                    let mut i = 0;
                    if total > 0 {
                        let chars = lw(g::chrinfo);
                        loop {
                            let record = i * 0x1210 + chars;
                            if lw(record + 484) != 0 {
                                leader = lw(record + 3020);
                                same_team_leader = if (lw(record + 24) ^ lw(state + 24)) == 0 { 1 } else { same_team_leader };
                            }
                            i += 1;
                            if !(i < total) {
                                break;
                            }
                        }
                    }
                    if leader != 0 {
                        let team = lw(state + 24);
                        if lw(lw(leader + 352) + 24) == team && total > 0 {
                            let chars = lw(g::chrinfo);
                            let mut j = 0;
                            loop {
                                let record = if j == 0 { chars } else { j * 0x1210 + chars };
                                if lw(record + 24) != team && lw(record + 2788) == leader && lw(record + 2716) == 2 {
                                    chased = 1;
                                    break;
                                }
                                j += 1;
                                if j >= total {
                                    break;
                                }
                            }
                        }
                    }
                    if target == leader && same_team_leader != 0 && chased != 0 {
                        wants_to_leave = true;
                    } else if (lw(lw(target + 352) + 2708) & 32) != 0 {
                        wants_to_leave = true;
                    } else if leader == 0 {
                        wants_to_leave = true;
                    } else if target == leader {
                        wants_to_leave = false;
                    } else {
                        wants_to_leave = same_team_leader == 0;
                    }
                }
            }
            if !wants_to_leave {
                return 0;
            }
            go_to_state_128(state)
        }
        1 | 4 => {
            let controlled = lw(state + 484);
            let target = lw(state + 2788);
            if controlled == 0 && lw(target + 8) == 64 && (lw(lw(target + 352) + 68) & 4) != 0 {
                return go_to_state_128(state);
            }
            if controlled != 0 {
                if (lw(state + 2704) & 64) == 0 {
                    return 0;
                }
                let drop = selectDrop(chr);
                sw(state + 684, drop);
                return new_target_chosen(state);
            }
            if lw(target + 8) == 64 {
                return 0;
            }
            reselect_mode_7(chr, state)
        }
        2 => reselect_by_attacker(chr, state),
        3 => 0,
        5 => {
            let bases = lw(g::numBases);
            if bases > 0 {
                if !(1.0_f32 <= lwc1(lw(state + 2788) + 532)) {
                    return 0;
                }
                let mut i = 0;
                loop {
                    sw(g::currentbase, i);
                    let base = lw((i << 3) + (g::bases + 4));
                    i += 1;
                    let alive = 1.0_f32 <= lwc1(base + 532);
                    sw(state + 2788, base);
                    if !alive || !(i < bases) {
                        return 0;
                    }
                }
            }
            let target = lw(state + 2788);
            if target == 0 {
                return 0;
            }
            if lw(lw(target + 352) + 2708) != 32 {
                return 0;
            }
            go_to_state_128(state)
        }
        6 => reselect_mode_7(chr, state),
        _ => unreachable!(),
    }
}
