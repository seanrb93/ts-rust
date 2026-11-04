//! The Virus mode (from the later games): a way to play Deathmatch, `rule virus 1` in a variant of `modes.txt`.
//!
//! Everyone is a survivor (team 1) but one, who starts infected (team 2).  A kill that an infected character makes of a survivor infects the survivor: he is on the infected team from then on, and
//! respawns as one.  The infected win when no survivor is left; the survivors win when the time limit of the game runs out.  The teams are the ones the game keeps for its team games (the team
//! word of a character's record, +24), so who the bots fight, the friendly fire switch and the team scores are the game's own.

use crate::*;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

const SURVIVORS: i32 = 1;
const INFECTED: i32 = 2;

static SETUP: AtomicBool = AtomicBool::new(false);
/// the game frame at which the teams were made
static SETUP_FRAME: AtomicI32 = AtomicI32::new(0);

/// Show a message to a player of this machine (the text is kept in the port's tables).
fn tell(player: i32, text: &str) {
    if !(0..lw(g::lvnumplayers)).contains(&player) {
        return;
    }
    let at = 0x240_0000 + 0x17_2c00;
    for (i, b) in text.bytes().take(60).enumerate() {
        sb(at + i as i32, b as i32);
    }
    sb(at + text.len().min(60) as i32, 0);
    hudMessage(player, at);
}

/// A game starts: the teams are made again when the level has its characters.
pub fn virus_reset() {
    SETUP.store(false, Ordering::Relaxed);
}

fn virus_on() -> bool {
    active_rules(lw(g::curgameparms + 72)).iter().any(|r| r.0 == Rule::Virus)
}

fn characters() -> (i32, i32) {
    (lw(g::chrinfo), lw(g::lvnumplayers) + lw(g::numactivechars))
}

/// Every game frame while the Virus rule is on (`custom_rules_tick`): the teams are made when the level has started, and the game is over when there is no survivor.
pub fn virus_tick(infected_at_start: i32) {
    if lw(g::gamestate) != 3 {
        return;
    }
    let (info, total) = characters();
    if info == 0 || total < 2 {
        return;
    }
    let frame = ts_sdk::frame_count() as i32;
    if std::env::var_os("TS_VIRUSLOG").is_some() && frame % 300 == 0 {
        eprintln!("[virus] f{frame} players {} active {} 484s {:?} teams {:?}", lw(g::lvnumplayers), lw(g::numactivechars), (0..total).map(|i| lw(i * 0x1210 + info + 484)).collect::<Vec<_>>(), (0..total).map(|i| lw(i * 0x1210 + info + 24)).collect::<Vec<_>>());
    }
    if !SETUP.load(Ordering::Relaxed) {
        // (a few frames after the level starts, when all the characters are made)
        if lw(g::lvnumplayers) <= 0 {
            return;
        }
        let present: Vec<i32> = (0..total).collect();
        if present.len() < 2 {
            return;
        }
        for &i in &present {
            sw(i * 0x1210 + info + 24, SURVIVORS);
        }
        let mut seed = frame.wrapping_mul(1103515245).wrapping_add(12345) as u32;
        let mut pool = present.clone();
        for _ in 0..infected_at_start.clamp(1, (present.len() as i32 - 1).max(1)) {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            let k = std::env::var("TS_VIRUSZERO").ok().and_then(|v| v.parse::<usize>().ok()).unwrap_or((seed >> 8) as usize) % pool.len();
            let zero = pool.remove(k);
            sw(zero * 0x1210 + info + 24, INFECTED);
            tell(zero, "You are infected! Infect them all");
            if std::env::var_os("TS_VIRUSLOG").is_some() {
                eprintln!("[virus] f{frame} character {zero} starts infected");
            }
        }
        SETUP.store(true, Ordering::Relaxed);
        SETUP_FRAME.store(frame, Ordering::Relaxed);
        return;
    }
    // (the game is over when no survivor is left)
    let survivors = (0..total).filter(|&i| lw(i * 0x1210 + info + 24) == SURVIVORS).count();
    if survivors == 0 && lw(g::gamestate) != 4 {
        if std::env::var_os("TS_VIRUSLOG").is_some() {
            eprintln!("[virus] f{frame} nobody is left uninfected");
        }
        gameLevelCompleted();
    }
}

/// A kill is counted (`StatsKillMade`): an infected character that kills a survivor infects him.
pub fn virus_kill(killer: i32, victim: i32) {
    if killer == 0 || victim == 0 || killer == victim || !SETUP.load(Ordering::Relaxed) || !virus_on() {
        return;
    }
    if lw(killer + 24) == INFECTED && lw(victim + 24) == SURVIVORS {
        sw(victim + 24, INFECTED);
        tell(lw(victim), "You are infected!");
        if std::env::var_os("TS_VIRUSLOG").is_some() {
            eprintln!("[virus] f{} character {} infects character {}", ts_sdk::frame_count(), lw(killer), lw(victim));
        }
    }
}
