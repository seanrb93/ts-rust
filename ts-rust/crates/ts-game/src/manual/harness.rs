//! A harness for looking at what a mod makes: a level that starts with the characters, the place and the camera the test asks for (`--character`, `--bots`, `--spawn`, `--cam` of
//! the port, which set the `TS_` variables below).  It works for a level started with `--level` and for a match started from the menus.
//!
//! * `TS_PLAYERCHR=<n>`: player 1 is character `n` (the character of the sign on that plays is changed before the level makes the player).
//! * `TS_BOTCHRS=<a>,<b>,...`: the bots of the match are these characters, as many as are named (the number the arcade page gave is replaced), put in before the level makes them.
//! * `TS_SPAWN=<x>,<y>,<z>`: player 1 is put there a few frames after the level starts.
//! * `TS_CAM=<back>,<up>`: from a few frames after the level starts the third person camera is on, that far behind the player and that high.
//!
//! The first two are applied by `harness_player` and `harness_bots`, called at the start of `playerRestart` and `enemyRestart` (a build with the `extensions` feature); the rest is in
//! `harness_tick`, called every tick.

use crate::*;
use std::sync::atomic::{AtomicI64, Ordering};

fn numbers(var: &str) -> Vec<f32> {
    std::env::var(var).map(|v| v.split(',').filter_map(|x| x.trim().parse().ok()).collect()).unwrap_or_default()
}

/// Make player 1 the character `TS_PLAYERCHR` names (at the start of `playerRestart`).
pub fn harness_player() {
    let Ok(n) = std::env::var("TS_PLAYERCHR").map(|v| v.trim().parse::<i32>()) else { return };
    let Ok(n) = n else { return };
    let id = signonGet(0);
    if id >= 0 && (0..128).contains(&n) {
        sw(lw(g::signonarray) + id * 2936 + 32 + 84, n);
    }
}

/// Make the bots the characters `TS_BOTCHRS` names (at the start of `enemyRestart`, which reads their number from the game's parameters; `enemySetup` reads who they are).
pub fn harness_bots() {
    let bots = numbers("TS_BOTCHRS");
    if bots.is_empty() {
        return;
    }
    let parms = g::curgameparms;
    let players = lw(g::lvnumplayers).max(1);
    let n = bots.len().min(10);
    sw(parms + 8, n as i32);
    for (i, b) in bots.iter().take(n).enumerate() {
        sb(parms + 21 + players + i as i32, *b as i32);
    }
}

/// the port's frame at which the level in play started (-1: none)
static STARTED: AtomicI64 = AtomicI64::new(-1);
static DONE: AtomicI64 = AtomicI64::new(0);

/// Every tick: note when a level starts and, a few frames later, put the player where `TS_SPAWN` says and the camera as `TS_CAM` says.
pub fn harness_tick() {
    let now = ts_sdk::frame_count() as i64;
    if lw(g::gamestate) != 3 {
        STARTED.store(-1, Ordering::Relaxed);
        DONE.store(0, Ordering::Relaxed);
        return;
    }
    if STARTED.load(Ordering::Relaxed) < 0 {
        STARTED.store(now, Ordering::Relaxed);
    }
    let since = now - STARTED.load(Ordering::Relaxed);
    let player = lw(g::players);
    if player == 0 || lw(player + 384) == 0 {
        return;
    }
    let spawn = numbers("TS_SPAWN");
    if spawn.len() == 3 && since == 20 {
        for k in 0..3 {
            swc1(player + 140 + 4 * k, spawn[k as usize]);
        }
    }
    let cam = numbers("TS_CAM");
    if cam.len() == 2 && since >= 25 {
        camSetMode(2);
        swc1(0x3b0128, cam[0]);
        swc1(0x3b0130, cam[1]);
    }
}
