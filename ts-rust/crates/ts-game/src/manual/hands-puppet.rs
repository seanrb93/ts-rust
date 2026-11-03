//! First person arms (an experiment).
//!
//! Without a weapon the player attacks with an arm: the game makes a second copy of the player's own character (the "puppet", `remote_hand` in
//! `player-gun-update.rs`), puts it beside the view and plays a swipe on it, so the arm that is seen has the character's own skin and sleeve.  Here the puppet is
//! also made while a gun is held, with an animation of the character's own that has the arms up (`TS_HANDANIM=<animation number>`), so that the arms of the
//! character are seen holding the gun.  The puppet is made on the first call of the gun, put where the game puts the one of the fists, and kept for the game.

use crate::*;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::{Mutex, OnceLock};

/// The puppets of the players 0 to 3 (0: none yet).
static PUPPETS: [AtomicI32; 4] = [AtomicI32::new(0), AtomicI32::new(0), AtomicI32::new(0), AtomicI32::new(0)];
/// Scratch space for the matrix of the heading and the offset.
const SCRATCH: i32 = 0x240_0000 + 0x17_2800;

/// A game starts: the puppets of the one before are gone.
pub fn hands_reset() {
    for p in &PUPPETS {
        p.store(0, Ordering::Relaxed);
    }
}

fn animation() -> Option<i32> {
    static A: OnceLock<Option<i32>> = OnceLock::new();
    *A.get_or_init(|| std::env::var("TS_HANDANIM").ok().and_then(|v| v.parse().ok()))
}

/// At the end of the update of a gun (`gunUpdate`, the gun slot `a0` of the hand `a1`, `mind` the player's record): keep the puppet of the player's right hand.
pub fn hands_puppet_tick(a0: i32, a1: i32, mind: i32) {
    // (an experiment: the player's own body, which holds the gun in the hands of the character, is seen from the eyes, and the gun of the first person is hidden)
    if std::env::var_os("TS_FULLBODY").is_some() {
        let player = lw(g::currentplayer);
        let number = lw(player);
        if (0..4).contains(&number) && lw(a0) != 0 && (lw(a0) as u32) < 20 {
            let body = lw(player + 384);
            sw(body + 16, lw(body + 16) | view_bit(number));
            let gun = lw(a0 + 12);
            if gun != 0 && a1 == 0 {
                sw(gun + 16, lw(gun + 16) & !view_bit(number));
            }
        }
        return;
    }
    let Some(anim) = animation() else { return };
    // (only when a gun is held: the state of a gun below 20 and not 0, which is the arm of the fists)
    let state = lw(a0);
    if a1 != 0 || state == 0 || (state as u32) >= 20 {
        return;
    }
    let player = lw(g::currentplayer);
    let number = lw(player);
    if !(0..4).contains(&number) {
        return;
    }
    let body = lw(player + 384);
    let mut puppet = PUPPETS[number as usize].load(Ordering::Relaxed);
    if puppet == 0 {
        let kind = lh(mind + 4);
        puppet = chrNew(kind, kind, 0, number + 104, -1, 0, lwc1(body + 48), lwc1(body + 52), lwc1(body + 56), lwc1(body + 76));
        if puppet == 0 {
            return;
        }
        propRemovePropFromCollisionList(puppet);
        sw(puppet + 8, 16);
        let inst = lw(puppet + 32);
        let flags = lw(puppet + 16);
        let inst_flags = lhu(inst + 292);
        sw(puppet + 12, gun_room(number));
        sh(inst + 292, inst_flags & 0x3fff);
        sw(puppet + 16, flags | 0x40000000 | view_bit(number));
        obInstSetAnim(inst, anim, 1.0);
        PUPPETS[number as usize].store(puppet, Ordering::Relaxed);
        eprintln!("[hands] puppet {puppet:#x} for player {number}, kind {kind}, animation {anim}, state {state}");
    }
    if std::env::var_os("TS_HANDLOG").is_some() {
        eprintln!("[hands] puppet at ({:.2}, {:.2}, {:.2}) heading {:.1}", lwc1(puppet + 48), lwc1(puppet + 52), lwc1(puppet + 56), lwc1(puppet + 80));
    }
    let puppet_mind = lw(puppet + 352);
    swc1(puppet + 76, lwc1(body + 76));
    swc1(puppet + 80, lwc1(body + 80));
    swc1(puppet + 84, lwc1(body + 84));
    swc1(puppet_mind + 2968, lwc1(player + 284));
    swc1(puppet_mind + 2972, lwc1(player + 276));
    // where the puppet is: the offset of the kind of body, turned by the heading
    let model = lw(lw(lw(body + 32) + 88) + 12);
    let offset = if lh(mind + 4) == 24 {
        g::offsetvecjoy_rf_191
    } else if model == 0 {
        g::offsetvecjoy_m_189
    } else if model == 1 {
        g::offsetvecjoy_f_190
    } else {
        g::offsetvecjoy_m_189
    };
    // (a test aid: `TS_HANDOFF=<across>,<up>,<forward>` moves the arms from where the fists have them)
    let extra: Vec<f32> = std::env::var("TS_HANDOFF").map(|v| v.split(',').filter_map(|x| x.trim().parse().ok()).collect()).unwrap_or_default();
    for k in 0..3 {
        swc1(SCRATCH + 64 + 4 * k, lwc1(offset + 4 * k) + extra.get(k as usize).copied().unwrap_or(0.0));
    }
    matrixRotY(SCRATCH, ps2float::mul(lwc1(body + 80), lwc1(0x3ad3bc)).pdiv(1.8e+02_f32));
    matrixVecMulAligned(&[SCRATCH as i64, (SCRATCH + 64) as i64]);
    let data = lw(body + 356);
    swc1(puppet + 48, lwc1(data + 140) + lwc1(SCRATCH + 64));
    swc1(puppet + 52, lwc1(data + 156) - lwc1(data + 104) + lwc1(SCRATCH + 68));
    swc1(puppet + 56, lwc1(data + 148) + lwc1(SCRATCH + 72));
    // the character holds the gun of the player as the characters of the game do (`gunChrUpdate`): the model of the gun in the hand, from the combination the player has
    if std::env::var_os("TS_HANDGUN").is_some() {
        sw(puppet_mind + 260, lw(mind + 260));
        gunChrUpdate(puppet, 0);
        let held = lw(puppet_mind + 376 + 64);
        if held != 0 {
            sw(held + 16, lw(held + 16) | 0x40000000 | view_bit(number));
            sw(held + 12, gun_room(number));
        }
        // (and the gun of the first person is not shown)
        let fp = lw(a0 + 12);
        if fp != 0 && fp != puppet {
            sw(fp + 16, lw(fp + 16) & !view_bit(number));
        }
    }
    let inst = lw(puppet + 32);
    // (the flag 0x8000 of the model is what shows the arm: the fists set it for a swipe and take it off at rest)
    sh(inst + 292, lhu(inst + 292) | 0x8000);
    if curAnim(puppet, anim) == 0 {
        obInstSetAnim(inst, anim, 1.0);
    }
}

/// The parts of the player's own body that are not drawn when it is seen from the eyes (`TS_FULLBODY`; `TS_HIDEPARTS=22,21` to name others): the head, which the eyes are in.
pub fn fullbody_hides_part(owner: i32, part: i32) -> bool {
    static ON: OnceLock<Option<Vec<i32>>> = OnceLock::new();
    let hidden = ON.get_or_init(|| {
        std::env::var_os("TS_FULLBODY")?;
        Some(std::env::var("TS_HIDEPARTS").map(|v| v.split(',').filter_map(|x| x.trim().parse().ok()).collect()).unwrap_or_else(|_| vec![22]))
    });
    let Some(list) = hidden else { return false };
    let player = lw(g::currentplayer);
    owner != 0 && owner == lw(player + 384) && list.contains(&part)
}

/// The player's own body is drawn moved by `TS_BODYOFF=<x>,<y>,<z>` (in the view's own axes) so that the arms and the gun it holds are in the view (`sign` 1 moves it, -1 puts it back).
pub fn fullbody_shift(inst: i32, sign: f32) -> bool {
    static OFF: OnceLock<Option<[f32; 3]>> = OnceLock::new();
    let Some(off) = *OFF.get_or_init(|| {
        std::env::var_os("TS_FULLBODY")?;
        let v: Vec<f32> = std::env::var("TS_BODYOFF").ok()?.split(',').filter_map(|x| x.trim().parse().ok()).collect();
        (v.len() == 3).then(|| [v[0], v[1], v[2]])
    }) else {
        return false;
    };
    let player = lw(g::currentplayer);
    let owner = lw(inst + 244);
    if owner == 0 || owner != lw(player + 384) {
        return false;
    }
    let matrices = match lw(player) {
        1 => lw(inst + 12),
        2 => lw(inst + 16),
        3 => lw(inst + 20),
        _ => lw(inst + 8),
    };
    if matrices == 0 {
        return false;
    }
    // (the display list names the matrices and is run later: they are moved and left moved; they are made again for each picture, so a matrix that is already where it was moved to is left)
    static DONE: Mutex<Vec<(i32, [f32; 3])>> = Mutex::new(Vec::new());
    let now = [lwc1(matrices + 48), lwc1(matrices + 52), lwc1(matrices + 56)];
    let mut done = DONE.lock().unwrap();
    if let Some(e) = done.iter_mut().find(|e| e.0 == matrices) {
        if (0..3).all(|k| (e.1[k] - now[k]).abs() < 1e-6) {
            return true;
        }
        e.1 = [now[0] + off[0], now[1] + off[1], now[2] + off[2]];
    } else {
        done.push((matrices, [now[0] + off[0], now[1] + off[1], now[2] + off[2]]));
    }
    for joint in 0..40 {
        let m = matrices + (joint << 6);
        for k in 0..3 {
            swc1(m + 48 + 4 * k, lwc1(m + 48 + 4 * k) + off[k as usize]);
        }
    }
    true
}
