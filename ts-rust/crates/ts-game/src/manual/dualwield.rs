//! Dual versions of the weapons that have none: "Uzi (x2)" next to "Uzi", for the weapons that the game only has in one hand.
//!
//! The game's weapon combinations (`guncombos`, 8 bytes each: rank, left gun, right gun, name) name the gun of each hand; a combination with
//! a left gun is a pair ("Pistol (x2)").  Everything else in the game (what a player owns, the ammo, the statistics, the pickups in the
//! levels) is indexed by the 33 combinations, so the new pairs are not combinations of their own: they are entries of the weapon
//! lists of the custom weapon sets, with the numbers from 33 up (`weapons_list_extend` adds them to the list the weapon page shows,
//! `combo_name` gives the name).  When a match starts the game asks for the combinations of the set (`frontmpgGetGunComboNum`): an entry
//! that is a pair is given as its single weapon and the weapon is remembered; while a match is played the single weapon's combination
//! has a left gun too, so the weapon is dual wielded wherever it is: the pickups of the weapon in the level are the pair.

use crate::*;
use std::sync::Mutex;

/// the first number of the new entries
const FIRST: i32 = 33;
/// the number of combinations that are single or paired weapons (the entries after them are the game's own special ones)
const COMBOS: i32 = 33;
/// the game's list of the weapons of the weapon page and its count
const LIST: i32 = 0x1fc2a00;
const COUNT: i32 = 0x3aff4c;
const NAME_AT: i32 = 0x240_0000 + 0xb000;

struct State {
    /// the combinations as the game has them: (left gun, right gun, name) of each
    original: Vec<(i32, i32, i32)>,
    /// the single combinations that have no pair: entry `FIRST + k` is the pair of `singles[k]`
    singles: Vec<i32>,
    /// the combinations that are dual wielded in the match that is set up
    dual: [bool; COMBOS as usize],
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

fn with<R>(f: impl FnOnce(&mut State) -> R) -> R {
    let mut g = STATE.lock().unwrap();
    let st = g.get_or_insert_with(|| {
        let original: Vec<(i32, i32, i32)> = (0..COMBOS).map(|k| (lhu(g::guncombos + 8 * k + 2), lhu(g::guncombos + 8 * k + 4), lhu(g::guncombos + 8 * k + 6))).collect();
        let paired: Vec<i32> = original.iter().filter(|c| c.0 != 0xffff && c.0 == c.1).map(|c| c.1).collect();
        // (the fists have no gun; a single weapon whose pair is another combination has its pair already)
        let singles = (1..COMBOS).filter(|&k| original[k as usize].0 == 0xffff && original[k as usize].1 != 0xffff && !paired.contains(&original[k as usize].1)).collect();
        State { original, singles, dual: [false; COMBOS as usize] }
    });
    f(st)
}

/// The number of the weapon of a new entry (the single combination), if the entry is one.
fn single_of(id: i32) -> Option<i32> {
    with(|s| s.singles.get((id - FIRST) as usize).copied()).filter(|_| id >= FIRST)
}

/// The name of a combination or of a new entry ("Uzi (x2)"), as text in memory.
pub fn combo_name(id: i32) -> i32 {
    let lang = lw(g::cur_language);
    let table = lw(g::languages + 4 * lang);
    match single_of(id) {
        None => lw(table + 4 * lh(g::guncombos + 8 * id + 6)),
        Some(single) => {
            let base = lw(table + 4 * lh(g::guncombos + 8 * single + 6));
            let mut name = String::new();
            let mut at = base;
            while lbu(at) != 0 && name.len() < 40 {
                name.push(lbu(at) as u8 as char);
                at += 1;
            }
            let mut at = NAME_AT + 0x40 * (id - FIRST);
            put_str(&mut at, &format!("{name} (x2)"))
        }
    }
}

static WEAPONS_PAGE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Is the weapon page being shown (its tick ran in this frame or the one before)?
pub fn weapons_page_active() -> bool {
    let at = WEAPONS_PAGE.load(std::sync::atomic::Ordering::Relaxed);
    at != 0 && (ts_sdk::frame_count() as u64) <= at + 2
}

/// The name that the message of a pickup of the weapon `combo` shows: `name` (the game's) or, when the weapon is dual wielded in this
/// match, the name of its dual version.
pub fn pickup_name(combo: i32, name: i32) -> i32 {
    let dual = with(|s| (combo >= 0 && combo < COMBOS && s.dual[combo as usize]).then(|| s.singles.iter().position(|&c| c == combo)).flatten());
    match dual.filter(|_| ts_sdk::dual_all()) {
        Some(k) => combo_name(FIRST + k as i32),
        None => name,
    }
}

/// `weapons_pageTick`, after it made the list of the weapons that can be chosen: the new pairs follow the singles they are made from.
pub fn weapons_list_extend() {
    WEAPONS_PAGE.store(ts_sdk::frame_count() as u64, std::sync::atomic::Ordering::Relaxed);
    let n = lw(COUNT).clamp(0, 40);
    let have: Vec<i32> = (0..n).map(|i| lw(LIST + 4 * i)).collect();
    if !ts_sdk::dual_all() {
        return;
    }
    let extra: Vec<i32> = with(|s| (0..s.singles.len() as i32).filter(|&k| have.contains(&s.singles[k as usize])).map(|k| FIRST + k).collect());
    let mut count = n;
    for id in extra {
        if !have.contains(&id) && count < 64 {
            sw(LIST + 4 * count, id);
            count += 1;
        }
    }
    sw(COUNT, count);
}

/// `frontmpgGetGunComboNum`: the combination the set has at place `place`, `combo` as the game found it.  An entry that is a pair of a
/// single weapon is given as the single weapon (and the weapon is dual wielded in the match).
pub fn dual_combo(place: i32, combo: i32) -> i32 {
    with(|s| {
        if place == 0 {
            s.dual = [false; COMBOS as usize];
        }
        match (combo >= FIRST).then(|| s.singles.get((combo - FIRST) as usize).copied()).flatten() {
            Some(single) => {
                s.dual[single as usize] = ts_sdk::dual_all();
                single
            }
            None => combo,
        }
    })
}

/// A game starts (`gameReset`): no weapon is dual wielded yet.
pub fn dual_reset() {
    PUNCHES.lock().unwrap().clear();
    with(|s| s.dual = [false; COMBOS as usize]);
}

/// Called every frame: while a match is played the weapons that the set has as pairs have a left gun; otherwise the table is the game's own.
pub fn dual_wield_tick() {
    let playing = lw(g::gamestate) != 1;
    with(|s| {
        for k in 0..COMBOS as usize {
            let a = g::guncombos + 8 * k as i32;
            let want = if playing && s.dual[k] { s.original[k].1 } else { s.original[k].0 };
            if lhu(a + 2) != want {
                sh(a + 2, want);
            }
        }
    });
}

// ---- the left fist ------------------------------------------------------------------------------------------------------------
//
// Without a weapon the player attacks with one arm: a model of the arm (a prop of its own, the "puppet") plays a swipe.  The game has no
// animation for the left arm, so every second swipe is drawn mirrored left to right: the display list gets a LABEL write around the
// puppet's picture (`propGfxRoom`), which the graphics code takes as "mirror what follows".  The hit test is the game's own.

/// (puppet prop, the next swipe is the left fist's)
static PUNCHES: Mutex<Vec<(i32, bool)>> = Mutex::new(Vec::new());

/// The puppet `prop` starts a swipe (`gunUpdate`): the swipes take turns.
pub fn punch_swing(prop: i32) {
    if !ts_sdk::dual_all() {
        return;
    }
    let mut p = PUNCHES.lock().unwrap();
    match p.iter_mut().find(|e| e.0 == prop) {
        Some(e) => e.1 = !e.1,
        None => p.push((prop, false)),
    }
}

/// Is the picture of this prop mirrored (`propGfxRoom`)?
pub fn punch_mirrored(prop: i32) -> bool {
    ts_sdk::dual_all() && PUNCHES.lock().unwrap().iter().any(|e| e.0 == prop && e.1)
}

const SCRATCH: i32 = 0x240_0000 + 0xc000;

/// The hit test of a swipe (`gunUpdate`) uses two points of the puppet's arm: when the swipe is the left fist's they are put on the
/// other side of the plane through the player that goes along the way he looks (`prop` is the puppet, `chr` the player's).
pub fn punch_mirror_points(prop: i32, chr: i32, p1: i32, p2: i32) {
    if !punch_mirrored(prop) {
        return;
    }
    // the player's right-hand direction: the game's own rotation by his heading of the sideways unit vector
    matrixRotY(SCRATCH, lwc1(chr + 80) * lwc1(0x3ad3bc) / 1.8e+02_f32);
    for (k, v) in [1.0_f32, 0.0, 0.0, 0.0].iter().enumerate() {
        swc1(SCRATCH + 64 + 4 * k as i32, *v);
    }
    matrixVecMulAligned(&[SCRATCH as i64, (SCRATCH + 64) as i64]);
    let mut r = [lwc1(SCRATCH + 64), lwc1(SCRATCH + 68), lwc1(SCRATCH + 72)];
    let len = (r[0] * r[0] + r[1] * r[1] + r[2] * r[2]).sqrt();
    if len < 1.0e-4 {
        return;
    }
    for c in r.iter_mut() {
        *c /= len;
    }
    let origin = [lwc1(chr + 48), lwc1(chr + 52), lwc1(chr + 56)];
    for p in [p1, p2] {
        let q = [lwc1(p), lwc1(p + 4), lwc1(p + 8)];
        let d = (q[0] - origin[0]) * r[0] + (q[1] - origin[1]) * r[1] + (q[2] - origin[2]) * r[2];
        for k in 0..3 {
            swc1(p + 4 * k as i32, q[k] - 2.0 * d * r[k]);
        }
    }
}

// ---- bots without a weapon ----------------------------------------------------------------------------------------------------
//
// A bot that starts without a weapon looks for one; in a level that has none to pick up it used to stand still.  It now goes after its
// target (`findweapon` patch) and, bare handed, hits it when it is close (this function, called by `enemyTick` for every bot).

const HIT_AT: i32 = 0x240_0000 + 0xc100;
/// time (in frames of 60 Hz) a bot has to wait for its next punch, per bot
static NEXT_PUNCH: Mutex<Vec<(i32, f32)>> = Mutex::new(Vec::new());

pub fn bot_punch(prop: i32) {
    let chr = lw(prop + 352);
    // (a bot of the arcade modes, with no gun and the fists as its weapon)
    if !ts_sdk::bots_punch() || chr == 0 || lw(chr + 8) != 0 || lw(chr + 376) != -1 || lw(chr + 260) != 0 || lw(g::gamestate) != 3 {
        return;
    }
    // (not while it is dying or about to spawn, and not at a target that is dead)
    let state = lw(chr + 2716);
    if state == 0 || state & (16 | 32) != 0 {
        return;
    }
    let target = lw(chr + 2788);
    if target == 0 {
        return;
    }
    let mut is_player = false;
    for p in 0..lw(g::lvnumplayers).clamp(0, 4) {
        let player = lw(g::players) + p * 1820;
        if lw(player + 384) == target {
            is_player = true;
            if lw(player + 296) == 4 {
                return;
            }
        }
    }
    let tchr = lw(target + 352);
    if !is_player && tchr != 0 && lw(tchr + 2716) & (16 | 32) != 0 {
        return;
    }
    let d = [lwc1(target + 48) - lwc1(prop + 48), lwc1(target + 52) - lwc1(prop + 52), lwc1(target + 56) - lwc1(prop + 56)];
    let dist2 = d[0] * d[0] + d[1] * d[1] + d[2] * d[2];
    let dt = lwc1(g::updateframesf);
    let mut wait = NEXT_PUNCH.lock().unwrap();
    let slot = match wait.iter().position(|e| e.0 == prop) {
        Some(i) => i,
        None => {
            wait.push((prop, 0.0));
            wait.len() - 1
        }
    };
    wait[slot].1 = (wait[slot].1 - dt).max(0.0);
    // (close, and on the same floor)
    if dist2 > 6.25 || d[1].abs() > 1.5 || wait[slot].1 > 0.0 {
        return;
    }
    wait[slot].1 = 45.0;
    drop(wait);
    if std::env::var_os("TS_PUNCHLOG").is_some() {
        eprintln!("[punch] f{} bot ({:.1} {:.1} {:.1}) target ({:.1} {:.1} {:.1}) dist {:.2} state {:#x} yaw {:.0}", ts_sdk::frame_count(), lwc1(prop + 48), lwc1(prop + 52), lwc1(prop + 56), lwc1(target + 48), lwc1(target + 52), lwc1(target + 56), dist2.sqrt(), lw(chr + 2716), lwc1(prop + 80));
    }
    let len = dist2.sqrt().max(1.0e-3);
    for k in 0..3 {
        swc1(HIT_AT + 4 * k, lwc1(target + 48 + 4 * k)); // where it hits
        swc1(HIT_AT + 16 + 4 * k, lwc1(prop + 48 + 4 * k)); // where it comes from
        swc1(HIT_AT + 32 + 4 * k, d[k as usize] / len); // the way it goes
    }
    propDamage(target, prop, HIT_AT, 1, HIT_AT + 16, HIT_AT + 32, 0, 0, 0.5);
    soundStart(if below_coin() { 172 } else { 171 }, prop + 48);
}

fn below_coin() -> bool {
    (ts_sdk::frame_count() & 1) != 0
}
