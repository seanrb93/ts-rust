//! First person arms (an experiment).
//!
//! Without a weapon the player attacks with an arm: the game makes a second copy of the player's own character (the "puppet", `remote_hand` in
//! `player-gun-update.rs`), puts it beside the view and plays a swipe on it, so the arm that is seen has the character's own skin and sleeve.  Here the puppet is
//! also made while a gun is held, with an animation of the character's own that has the arms up (`TS_HANDANIM=<animation number>`), so that the arms of the
//! character are seen holding the gun.  The puppet is made on the first call of the gun, put where the game puts the one of the fists, and kept for the game.

use crate::*;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::OnceLock;

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
    let inst = lw(puppet + 32);
    // (the flag 0x8000 of the model is what shows the arm: the fists set it for a swipe and take it off at rest)
    sh(inst + 292, lhu(inst + 292) | 0x8000);
    if curAnim(puppet, anim) == 0 {
        obInstSetAnim(inst, anim, 1.0);
    }
}
