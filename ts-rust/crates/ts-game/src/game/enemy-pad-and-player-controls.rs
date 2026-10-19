//! An enemy that goes to its pad, and the button layouts of the players.

use crate::*;

/// Put an enemy into state `new` (the old one kept at +2720) and clear its timer at +2892.
fn enter_state(state: i32, new: i32) {
    let old = lw(state + 2716);
    sw(state + 2716, new);
    sw(state + 2720, old);
    sw(state + 2892, 0);
}

/// Send an enemy to its pad (at +680 of its state): its move mode is set to 1 and the pad's position is kept at +2796. Near the pad (closer than the game's limit) it takes the
/// state kept at +2724, or else goes to state 8 (alone in the story with a player in the game, or when it has a target) or 128, or when it has no flag 440 and no flag at +8 to
/// state 64. Far from the pad, after half a second of waiting, it goes to state 2 with a target of a kind 1 or 8 (a different team) that it is allowed to hit, or to 256 if
/// bit 1 of +2704 is set (@ 0x2c64c8).
pub fn findmypad(chr: i32) {
    let state = lw(chr + 352);
    swc1(state + 2872, lwc1(state + 2860));
    enemySetMoveMode(chr, 1);
    let near = lwc1(0x3ad97c);
    let pad = lw(state + 680) * 28 + lw(g::levelpads);
    let x = lwc1(pad + 12);
    swc1(state + 2796, x);
    let y = lwc1(pad + 16);
    swc1(state + 2800, y);
    let z = lwc1(pad + 20);
    swc1(state + 2804, z);
    let dx = lwc1(chr + 48) - x;
    let dy = lwc1(chr + 52) - y;
    let dz = lwc1(chr + 56) - z;
    if !((dx * dx + dy * dy + dz * dz) < near) {
        let target = lw(state + 2788);
        let waited;
        if target == 0 {
            waited = lwc1(state + 2892);
        } else {
            let mut done = false;
            let mut timer = 0.0_f32;
            if lw(state + 2772) != 0 {
                let kind = lw(target + 8);
                if kind != 1 {
                    if kind != 8 {
                        timer = lwc1(state + 2892);
                        done = true;
                    } else if lw(state + 24) == lw(lw(target + 352) + 24) {
                        timer = lwc1(state + 2892);
                        done = true;
                    }
                }
                if !done && lw(state + 440) != 0 {
                    enter_state(state, 2);
                    return;
                }
            }
            waited = if done { timer } else { lwc1(state + 2892) };
        }
        if !(0.5_f32 < waited) {
            return;
        }
        if lw(state + 2704) & 1 == 0 {
            return;
        }
        enter_state(state, 256);
        return;
    }
    let pending = lw(state + 2724);
    if pending != 0 {
        let old = lw(state + 2716);
        sw(state + 2716, pending);
        sw(state + 2720, old);
        sw(state + 2724, 0);
        sw(state + 2892, 0);
        return;
    }
    let guard;
    if lw(g::curgameparms + 72) != 1 {
        guard = lw(state + 440);
    } else {
        if lw(state + 484) != 0 {
            enter_state(state, 8);
            return;
        }
        guard = lw(state + 440);
    }
    let target;
    if guard != 0 {
        target = lw(state + 2788);
    } else {
        if lw(state + 8) == 0 {
            let timer = lwc1(state + 2856);
            sw(state + 2720, lw(state + 2716));
            swc1(state + 2872, timer);
            sw(state + 2716, 64);
            sw(state + 2692, 0);
            sw(state + 2892, 0);
            sw(state + 1688, 0);
            return;
        }
        target = lw(state + 2788);
    }
    enter_state(state, if target == 0 { 128 } else { 8 });
}

/// Set the button layout of a player record: by the kind of its controller (the halfword at the start of the record held at +24; 2 when there is none): kinds 1 and 2 and 3
/// are written out; kind 0 copies 76 bytes from the controller's record; any other kind changes nothing (@ 0x27f5a8).
pub fn playerSetControls(player: i32) {
    let controller = lw(player + 24);
    let mut kind = 2;
    if controller != 0 {
        kind = lhu(controller);
    }
    if kind == 1 || kind == 2 {
        sw(player + 96, 2);
        sw(player + 32, 8);
        sw(player + 40, 16);
        sw(player + 56, 4);
        sw(player + 48, 0x1000);
        sw(player + 52, 0x4000);
        sw(player + 36, 64);
        sw(player + 60, 0x2000);
        sw(player + 64, 0x8000);
        sw(player + 84, 1);
        sw(player + 92, 3);
        sw(player + 28, 2);
        sw(player + 44, 1);
        sw(player + 68, if kind == 1 { 1 } else { 3 });
        sw(player + 76, if kind == 1 { 3 } else { 1 });
        sw(player + 72, 0);
        sw(player + 80, 2);
        sw(player + 88, 0);
        return;
    }
    if kind == 3 {
        sw(player + 32, 4);
        sw(player + 40, 0x1000);
        sw(player + 56, 8);
        sw(player + 48, 16);
        sw(player + 52, 64);
        sw(player + 36, 0x8000);
        sw(player + 60, 32);
        sw(player + 64, 128);
        sw(player + 84, kind);
        sw(player + 92, 1);
        sw(player + 88, 2);
        sw(player + 96, 0);
        sw(player + 28, 1);
        sw(player + 44, 2);
        sw(player + 68, kind);
        sw(player + 76, 1);
        sw(player + 72, 2);
        sw(player + 80, 0);
        return;
    }
    if kind == 0 {
        let mut src = controller + 4;
        let mut dst = player + 28;
        for _ in 0..2 {
            let b = ld(src + 8);
            let c = ld(src + 16);
            let d = ld(src + 24);
            sd(dst, ld(src));
            sd(dst + 8, b);
            sd(dst + 16, c);
            sd(dst + 24, d);
            src += 32;
            dst += 32;
        }
        let word = lw(src + 8);
        sd(dst, ld(src));
        sw(dst + 8, word);
    }
}
