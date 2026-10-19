//! A computer character following its target's pad.

use crate::*;

/// Put the character into the state `new` (the old state kept at +2720, the timer at +2892 cleared).
fn enter_state(state: i32, new: i32) {
    let old = lw(state + 2716);
    sw(state + 2716, new);
    sw(state + 2720, old);
    sw(state + 2892, 0);
}

/// The character is at the pad it wants (the target pad `wanted`, over 0): by the game mode (1 to 7) it picks another pad (modes 1 and 4 a drop, or a random pad the character can
/// go to; the other modes keep still), clears its two timers; returns true when it did so (false means the caller must stop).
fn at_target_pad(chr: i32, state: i32, path: i32, wanted: i32, mode: i32) -> Option<bool> {
    match mode {
        0 => {
            if lw(state + 484) == 0 {
                enter_state(state, 128);
                return Some(false);
            }
            let position = lw(state + 3020) + 48;
            let mut extref = lvGetRandomPadExtref(-0x1fffd7ec, position);
            if extref == 0 {
                extref = lvGetRandomPadExtref(-0x1fffafd8, lw(state + 3020) + 48);
                if extref == 0 {
                    extref = lvGetRandomPadExtref(-0x1fff9c00, lw(state + 3020) + 48);
                }
            }
            sw(state + 684, padNumFromExtref(extref));
            if findNextPad(path) >= 0 {
                sw(state + 2692, 0);
            } else {
                let mut tries = 1;
                loop {
                    let found = lvGetRandomPadExtref(-0x1ffff600, lw(state + 3020) + 48);
                    if found == 0 {
                        tries += 1;
                    } else {
                        sw(state + 684, padNumFromExtref(found));
                        let next = findNextPad(path);
                        tries = (if next >= 0 { 10 } else { tries }) + 1;
                    }
                    if tries >= 10 {
                        break;
                    }
                }
                if findNextPad(path) < 0 {
                    sw(state + 684, wanted);
                }
                sw(state + 2692, 0);
            }
            sw(state + 1688, 0);
            Some(true)
        }
        1 | 4 => {
            let controller = lw(state + 484);
            if controller == 0 {
                enter_state(state, 128);
                return Some(false);
            }
            if lw(lw(controller + 352) + 16) != lw(state + 24) {
                let drop = selectDrop(chr);
                sw(state + 684, drop);
                sw(state + 2692, 0);
                sw(state + 1688, 0);
                return Some(true);
            }
            enter_state(state, 128);
            Some(false)
        }
        2 | 3 | 5 | 6 => {
            enemySetMoveMode(chr, 8);
            None
        }
        _ => unreachable!(),
    }
}

/// Make the character follow the pad of its target (as `followTarget` does for the pad on the way): when the target pad changed on the tick of the character's own turn its
/// timers are cleared; with a flag at +2772 set the character goes to state 2 (move mode 1, or the escort tests of the game mode 4) unless the mode 1 conditions or
/// `enemyReselectTarget` find something else; with no route the state is 0x4000000, or 128 when it has no next pad; at the target pad `at_target_pad` picks another; otherwise it
/// moves to the next pad and, after waiting near it, goes to state 256 (@ 0x2c0510).
pub fn trackTarget(chr: i32) {
    let state = lw(chr + 352);
    let target_pad = enemyGetTargetPad(chr);
    let turn = lw(g::updatedistindex);
    if lw(state + 684) != target_pad && turn == lw(state) {
        sw(state + 1688, 0);
        sw(state + 2692, 0);
    }
    sw(state + 684, target_pad);
    let path = state + 680;
    let next = findNextPad(path);
    if next == -2 {
        let old = lw(state + 2716);
        sw(state + 2716, 0x4000000);
        sw(state + 2720, old);
        sw(state + 2892, 0);
        return;
    }
    if next != -1 {
        let pad = next * 28 + lw(g::levelpads);
        swc1(state + 2796, lwc1(pad + 12));
        swc1(state + 2800, lwc1(pad + 16));
        swc1(state + 2804, lwc1(pad + 20));
    }
    if lw(state + 2772) != 0 {
        let params = g::curgameparms;
        let mode = lw(params + 72);
        let mut skip_to_reselect = false;
        if mode != 1 {
            if mode != 4 {
                enemySetMoveMode(chr, 1);
                enter_state(state, 2);
                return;
            }
            if (lw((lw(state + 24) << 2) + g::teamflag) & 32) == 0 {
                let new = if lw(state + 2788) != lw(g::escort) { 2 } else { 128 };
                enter_state(state, new);
                return;
            }
            enter_state(state, 2);
            return;
        }
        let target;
        if lw(state + 484) == 0 {
            target = lw(state + 2788);
        } else if (lw(params + 80) & 2048) != 0 {
            skip_to_reselect = true;
            target = 0;
        } else {
            target = lw(state + 2788);
        }
        if !skip_to_reselect && lw(state + 24) != lw(lw(target + 352) + 24) {
            enter_state(state, 2);
            return;
        }
    }
    if enemyReselectTarget(chr) != 0 {
        return;
    }
    if next == -1 {
        let wanted = lw(state + 684);
        if lw(state + 680) == wanted && wanted > 0 {
            let mode = lw(g::curgameparms + 72) - 1;
            if (mode as u32) >= 7 {
                return;
            }
            let _ = at_target_pad(chr, state, path, wanted, mode);
            return;
        }
        enter_state(state, 128);
        return;
    }
    enemySetMoveMode(chr, 1);
    let x = lwc1(state + 2796);
    swc1(state + 2872, lwc1(state + 2856));
    let dx = lwc1(chr + 48) - x;
    let dy = lwc1(chr + 52) - lwc1(state + 2800);
    let dz = lwc1(chr + 56) - lwc1(state + 2804);
    let waited;
    if !((dx * dx + dy * dy + dz * dz) < lwc1(0x3ad8a4)) {
        waited = lwc1(state + 2892);
    } else {
        let count = lw(state + 1688);
        sw(state + 680, next);
        sw(state + 1688, count - 1);
        waited = lwc1(state + 2892);
    }
    if !(lwc1(0x3ad8a8) < waited) {
        return;
    }
    if (lw(state + 2704) & 1) == 0 {
        return;
    }
    enter_state(state, 256);
}
