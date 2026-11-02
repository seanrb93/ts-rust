//! Network play: the game's own link between machines (the `ilink` functions, which the console game used for two consoles joined by an i.Link
//! cable) made to work over the network.  Every machine runs the whole game; each tick the pads of all the players are exchanged (`ts_sdk::net`)
//! and written into the controller records 8 to 11, which are the records of the players on a link, and every machine draws the view of its
//! own player only.

use crate::*;

/// The controller record that is the pad of this machine (port 0, slot 0).
const LOCAL_PAD: i32 = 0;
/// The first controller record of the players of a network game.
const FIRST_NET_PAD: i32 = 8;

/// Is this a network game?
pub fn netlink_active() -> bool {
    ts_sdk::net::active()
}

/// The pad record of a controller as the 16 bytes that travel.
fn pad_bytes(c: i32) -> [u8; ts_sdk::net::PAD_BYTES] {
    let mut b = [0u8; ts_sdk::net::PAD_BYTES];
    b[0..2].copy_from_slice(&(lhu(c + 288) as u16).to_le_bytes());
    b[2..4].copy_from_slice(&(lhu(c + 340) as u16).to_le_bytes());
    b[4..6].copy_from_slice(&(lhu(c + 292) as u16).to_le_bytes());
    b[6..8].copy_from_slice(&(lhu(c + 296) as u16).to_le_bytes());
    b[8] = lbu(c + 300) as u8;
    b[9] = lbu(c + 304) as u8;
    b[10] = lbu(c + 308) as u8;
    b[11] = lbu(c + 312) as u8;
    b[12..16].copy_from_slice(&lwc1(c + 316).to_le_bytes());
    b
}

/// Write the 16 bytes into a controller record (the same fields as `GetControllerFromBuff`).
fn put_pad(c: i32, b: &[u8; ts_sdk::net::PAD_BYTES]) {
    let h = |i: usize| u16::from_le_bytes([b[i], b[i + 1]]) as i32;
    sw(c + 288, h(0));
    sw(c + 340, h(2));
    sw(c + 292, h(4));
    sw(c + 296, h(6));
    sw(c + 300, b[8] as i32);
    sw(c + 304, b[9] as i32);
    sw(c + 308, b[10] as i32);
    sw(c + 312, b[11] as i32);
    swc1(c + 316, f32::from_le_bytes([b[12], b[13], b[14], b[15]]));
}

/// A checksum of what the game's tick changes: the players, the characters, the random numbers and the frame.
fn state_sum() -> u32 {
    let mut h: u32 = 0x811c_9dc5;
    let mut feed = |x: u32| {
        h = (h ^ x).wrapping_mul(0x0100_0193);
    };
    let players = lw(g::lvnumplayers);
    let from = lw(g::players);
    for i in 0..(players * 1820 / 4) {
        // (the four words at 1760 are addresses of buffers the player was given when the level began; where the game's memory happens to give them differs between machines
        // that were in other menus before, and nothing that is played depends on it)
        if (1760..1776).contains(&(i * 4 % 1820)) {
            continue;
        }
        feed(lw(from + 4 * i) as u32);
    }
    let chars = lw(g::chrinfo);
    let count = players + lw(g::numactivechars);
    for i in 0..(count * 0x1210 / 4) {
        feed(lw(chars + 4 * i) as u32);
    }
    feed(ld(g::newrndseedval) as u32);
    feed((ld(g::newrndseedval) >> 32) as u32);
    feed(lw(g::rndcount) as u32);
    feed(lw(g::gameframe) as u32);
    // the match: the scores and the like
    for i in 0..(14 * 1368 / 4) {
        feed(lw(g::matchstats + 4 * i) as u32);
    }
    for i in 0..28 {
        feed(lw(g::curgameparms + 4 * i) as u32);
    }
    h
}

/// Where the players are (a line of the log, to see two machines agree).
fn where_everybody() -> String {
    let mut s = String::new();
    for i in 0..lw(g::lvnumplayers) {
        let prop = lw(lw(g::players) + i * 1820 + 384);
        if prop != 0 {
            s += &format!(" p{} c{} ({:.1} {:.1} {:.1})", i, lhu(lw(prop + 352) + 4), lwc1(prop + 48), lwc1(prop + 52), lwc1(prop + 56));
        }
    }
    // (the bots, in more digits: a test aid shows where two machines first part)
    let players = lw(g::lvnumplayers);
    for i in 0..lw(g::numactivechars).min(10) {
        let chr = lw(g::chrinfo) + (players + i) * 0x1210;
        let prop = lw(chr + 3020);
        if prop != 0 {
            s += &format!(" b{} ({:.3} {:.3} {:.3})", i, lwc1(prop + 48), lwc1(prop + 52), lwc1(prop + 56));
        }
    }
    s
}

/// A player's machine has gone: everybody sees a message and the player's character falls (the same on every machine, on the same tick).
fn player_left(p: i32) {
    const TEXT_AT: i32 = 0x240_0000 + 0xd600;
    let text = format!("Player {} has left", p + 1);
    for (i, c) in text.bytes().enumerate() {
        sb(TEXT_AT + i as i32, c as i32);
    }
    sb(TEXT_AT + text.len() as i32, 0);
    for i in 0..lw(g::lvnumplayers) {
        hudMessage(i, TEXT_AT);
    }
    let prop = lw(g::players) + p * 1820;
    if p < lw(g::lvnumplayers) && lw(prop + 384) != 0 && lw(prop + 296) != 4 {
        playerDie(prop, 0, -1, 0, 0, 0, 0);
    }
}

/// Called at the start of each frame (`ilinkTick`) of a network game once the pads have been read: in a level, this machine's pad is given
/// to the others, and the pads of every player for the tick are written into their controller records.
pub fn netlink_tick() {
    if !ts_sdk::net::active() || lw(g::gamestate) != 3 || ts_sdk::net::rollback() {
        return;
    }
    let local = local_pad();
    let Some(pads) = ts_sdk::net::step(local) else {
        ts_sdk::net::lost("The other players have stopped answering, so the network game ends.");
        return;
    };
    apply_pads(&pads, 0);
    check_out_of_step();
    for p in ts_sdk::net::newly_gone() {
        player_left(p as i32);
    }
    let t = ts_sdk::net::tick() - 1;
    if t % ts_sdk::net::SUM_EVERY == 0 {
        let sum = state_sum();
        ts_sdk::net::submit_sum(t, sum);
        if std::env::var_os("TS_NETLOG").is_some() {
            eprintln!("[net] tick {t} sum {sum:08x}{}", where_everybody());
        }
    }
}

/// The player on this machine has an in-game (pause) menu open.
pub fn netlink_local_menu_open() -> bool {
    let me = ts_sdk::net::local_player();
    lhu(me * 92 + 0x1fb3248) & 1 != 0
}

/// The player `a0` pauses: in a network game only that player's own in-game menu opens (the others carry on playing), and the game's own pause of the time is not used (see
/// `timeGetPause`).  True when it was done here; false for a game that is not a network game.
pub fn netlink_pause(a0: i32) -> bool {
    if !ts_sdk::net::active() {
        return false;
    }
    soundStart(222, 0);
    sw(0x3afe78, a0);
    if lhu(a0 * 92 + 0x1fb3248) & 1 == 0 {
        ingamemenuStart(a0);
    }
    frontSet(a0);
    timePause(1);
    true
}

/// In a network game the in-game menus of the players are their own: a player's menu is closed when that player has chosen to resume (the game marks it ready, flag 2), and
/// the time runs again when no menu is open.  The original waits for every menu and closes them all together.  True when it was done here.
pub fn netlink_unpause() -> bool {
    if !ts_sdk::net::active() {
        return false;
    }
    let wanted = lw(0x3afe78);
    let state = lw(g::gamestate);
    if wanted < 0 {
        // (nobody has paused: the original goes on at the end of a level)
        return state != 4;
    }
    if state != 3 {
        return false;
    }
    let players = lw(g::lvnumplayers);
    for p in 0..players {
        let flags = lhu(p * 92 + 0x1fb3248);
        if flags & 1 != 0 && flags & 4 == 0 && flags & 2 != 0 {
            // (the game closes a menu in `ingamemenuExit` when nobody has the pause)
            sw(0x3afe78, -1);
            ingamemenuExit(p);
            sw(0x3afe78, wanted);
        }
    }
    let any = (0..players).any(|p| lhu(p * 92 + 0x1fb3248) & 1 != 0);
    if !any {
        sw(0x3afe78, -1);
        timePause(0);
    }
    true
}

/// This machine's pad as it travels: the controller record, with the look sensitivity (the others have no way to know it) and the mouse's movement.
fn local_pad() -> [u8; ts_sdk::net::PAD_BYTES] {
    let mut local = pad_bytes(g::controllers + LOCAL_PAD * 384);
    // (the look sensitivity and the mouse are part of the pad: the others have no way to know them)
    let k = ts_sdk::pad::look_sensitivity();
    if k != 1.0 {
        for i in [8, 9] {
            local[i] = (128.0 + (local[i] as f32 - 128.0) * k).round().clamp(0.0, 255.0) as u8;
        }
    }
    let (mut dx, mut dy) = ts_sdk::pad::mouse_take();
    // test aid: TS_MOUSETEST=<x>,<y> moves the mouse by that much on every tick
    static TEST: std::sync::OnceLock<(f64, f64)> = std::sync::OnceLock::new();
    let t = TEST.get_or_init(|| std::env::var("TS_MOUSETEST").ok().and_then(|v| v.split_once(',').map(|(a, b)| (a.parse().unwrap_or(0.0), b.parse().unwrap_or(0.0)))).unwrap_or((0.0, 0.0)));
    dx += t.0;
    dy += t.1;
    if mouse_look_active() {
        local[16..18].copy_from_slice(&((dx * 8.0).round().clamp(-32000.0, 32000.0) as i16).to_le_bytes());
        local[18..20].copy_from_slice(&((dy * 8.0).round().clamp(-32000.0, 32000.0) as i16).to_le_bytes());
    }
    if netlink_local_menu_open() {
        // (the game goes on while this player's pause menu is open, and the menu reads the same pad: its buttons go on, the sticks do not move the character)
        local[8..12].fill(128);
    }
    local
}

/// The pads of the players for the tick are written into their controller records, and the mouse turns the players; the players in `gone` have left.
fn apply_pads(pads: &[[u8; ts_sdk::net::PAD_BYTES]], gone: u8) {
    for (p, b) in pads.iter().enumerate() {
        let record = FIRST_NET_PAD + p as i32;
        put_pad(g::controllers + record * 384, b);
        sw(g::playercontrollers + 4 * p as i32, record);
        let mx = i16::from_le_bytes([b[16], b[17]]);
        let my = i16::from_le_bytes([b[18], b[19]]);
        if (mx != 0 || my != 0) && (p as i32) < lw(g::lvnumplayers) {
            mouse_turn_player(lw(g::players) + p as i32 * 1820, mx as f32 / 8.0, my as f32 / 8.0);
        }
    }
    for p in 0..pads.len() {
        if gone & (1 << p) != 0 {
            player_left(p as i32);
        }
    }
}

/// The checksums say the machines have parted: the player is told, once, and the log has it.
fn check_out_of_step() {
    static TOLD: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    if ts_sdk::net::out_of_step() && TOLD.swap(1, std::sync::atomic::Ordering::Relaxed) == 0 {
        sw(0x240_0000 + 0xd700, 0);
        let text = b"Out of step with the others";
        for (i, c) in text.iter().enumerate() {
            sb(0x240_0000 + 0xd700 + i as i32, *c as i32);
        }
        sb(0x240_0000 + 0xd700 + text.len() as i32, 0);
        hudMessage(ts_sdk::net::local_player().min(lw(g::lvnumplayers) - 1), 0x240_0000 + 0xd700);
    }
}

// ---- rollback ------------------------------------------------------------------------------------------------------------------------------------------------

/// The ticks the journal goes back (more than the game may run ahead with guessed pads).
#[cfg(feature = "extensions")]
const KEEP: usize = 24;

#[cfg(feature = "extensions")]
struct Rollback {
    started: bool,
    /// the tick the game is at (the next frame to play)
    tick: u32,
    /// the first tick whose pads are all known and were what was played
    verified: u32,
    /// the pads that were played for each tick (all the players), and the players who left with it
    used: std::collections::BTreeMap<u32, (Vec<[u8; ts_sdk::net::PAD_BYTES]>, u8)>,
    /// the checksums of the state after a tick, waiting until the tick is certain
    pending: std::collections::BTreeMap<u32, u32>,
}

#[cfg(feature = "extensions")]
static ROLLBACK: std::sync::Mutex<Rollback> = std::sync::Mutex::new(Rollback { started: false, tick: 0, verified: 0, used: std::collections::BTreeMap::new(), pending: std::collections::BTreeMap::new() });

/// Between a tick's picture and the time of the next tick (a network game with rollback at more than 60 pictures a second): as many pictures as fit.  Each is the game's
/// frame for the part of a tick that has passed since the tick, played from the state the tick left and then undone with the journal, so that only the tick changes the game.
/// What moves smoothly is moved; what happens at a tick (the whole ticks) is not.  The pictures start no sounds.
#[cfg(feature = "extensions")]
/// A checksum of every 256 bytes of the game's memory (a test aid).
fn block_address(i: usize) -> usize {
    if i < 0x400_0000 / 256 { i * 256 } else { 0xfe0_0000 + (i - 0x400_0000 / 256) * 256 }
}

fn memory_blocks() -> Vec<u32> {
    // (the first 64 MB and the last 2 MB, where the stack is)
    (0..(0x400_0000 / 256)).chain((0xfe0_0000 / 256)..(0x1000_0000 / 256))
        .map(|page| {
            let at = if page < 0x400_0000 / 256 { page * 256 } else { page * 256 };
            let mut h: u32 = 0x811c_9dc5;
            for w in 0..64 {
                h = (h ^ lw(at + 4 * w) as u32).wrapping_mul(0x0100_0193);
            }
            h
        })
        .collect()
}

pub fn netlink_sub_frames() {
    use std::time::{Duration, Instant};
    static COST: std::sync::Mutex<f64> = std::sync::Mutex::new(0.004);
    if !ts_sdk::net::rollback() || std::env::var_os("TS_NOSUBFRAMES").is_some() {
        return;
    }
    let hz = ts_sdk::display_hz() as f64;
    if hz < 61.0 && std::env::var_os("TS_SUBFORCE").is_none() {
        return;
    }
    if !in_level() || netlink_local_menu_open() || !ROLLBACK.lock().unwrap().started {
        ts_sdk::net::note_subframes_wanted(0, true, false);
        return;
    }
    // (a test aid, TS_SUBFORCE=<every n ticks>: a picture between ticks at the boundary of every nth tick whatever the time, so that two machines can be made to differ in
    // them, and with TS_SUBCHECK=2 what the picture changed that was not put back is told)
    if let Some(n) = std::env::var("TS_SUBFORCE").ok().and_then(|v| v.parse::<u32>().ok()) {
        static TICKS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        if TICKS.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % n.max(1) == 0 {
            let blocks = if std::env::var("TS_SUBCHECK").map_or(false, |v| v == "2") { Some(memory_blocks()) } else { None };
            ts_rt::journal::begin_tick();
            ts_sdk::net::set_muted(true);
            sub_frame(0.5);
            ts_sdk::net::set_muted(false);
            ts_sdk::present_sync();
            ts_rt::journal::undo(1);
            if let Some(b) = blocks {
                let after = memory_blocks();
                let diff: Vec<String> = (0..b.len()).filter(|&i| b[i] != after[i]).take(24).map(|i| format!("{:x}", block_address(i))).collect();
                if !diff.is_empty() {
                    eprintln!("[sub] not put back: {} blocks, the first at {:?}", (0..b.len()).filter(|&i| b[i] != after[i]).count(), diff);
                }
            }
        }
        return;
    }
    let tick = 1.0 / 60.0;
    let real = Instant::now();
    let boundary = ts_sdk::next_vsync();
    // (the pictures a tick gets: `hz / 60` on average, so the whole numbers on either side of it in turn)
    static OWED: std::sync::Mutex<f64> = std::sync::Mutex::new(0.0);
    let extra = {
        let mut o = OWED.lock().unwrap();
        *o += hz / 60.0 - 1.0;
        let n = o.floor().max(0.0);
        *o -= n;
        n as u32
    };
    if extra == 0 {
        return;
    }
    let spacing = Duration::from_secs_f64(tick / (extra as f64 + 1.0));
    let mut shown = real;
    let mut count = 0u32;
    let check = std::env::var_os("TS_SUBCHECK").is_some();
    while count < extra {
        let cost = Duration::from_secs_f64(*COST.lock().unwrap());
        let start = (shown + spacing).max(Instant::now());
        if start + cost >= boundary {
            break;
        }
        if !ts_sdk::paced_by_present() {
            ts_sdk::sleep_until(start);
        }
        let f = (Instant::now() - real).as_secs_f64() / tick;
        let began = Instant::now();
        let before = if check { state_sum() } else { 0 };
        // (a test aid, TS_SUBCHECK=2: every 256 bytes of memory, to see what the picture changed that was not put back)
        static SCANS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let blocks = if check && std::env::var("TS_SUBCHECK").map_or(false, |v| v == "2") && SCANS.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % 2 == 1 { Some(memory_blocks()) } else { None };
        ts_rt::journal::begin_tick();
        ts_sdk::net::set_muted(true);
        sub_frame(f.clamp(0.02, 0.98) as f32);
        ts_sdk::net::set_muted(false);
        // (what is being drawn must be finished before the memory is put back)
        ts_sdk::present_sync();
        ts_rt::journal::undo(1);

        if check && state_sum() != before {
            eprintln!("[sub] the picture between ticks changed the game's state");
        }
        if let Some(b) = blocks {
            let after = memory_blocks();
            let diff: Vec<String> = (0..b.len()).filter(|&i| b[i] != after[i]).take(24).map(|i| format!("{:x}", block_address(i))).collect();
            if !diff.is_empty() {
                eprintln!("[sub] not put back: {} blocks, the first at {:?}", (0..b.len()).filter(|&i| b[i] != after[i]).count(), diff);
            }
        }
        shown = Instant::now();
        count += 1;
        let took = (shown - began).as_secs_f64();
        let mut c = COST.lock().unwrap();
        *c = *c * 0.8 + took * 0.2;
    }
    ts_sdk::net::note_subframes(count);
    ts_sdk::net::note_subframes_wanted(extra, false, count < extra);
}

/// Before the sound commands of a frame are sent: the ones a tick played again has already sent are taken out.
#[cfg(feature = "extensions")]
pub fn netlink_sound_filter() {
    if ts_sdk::net::muted() {
        sw(g::soundbatchlen, 0);
        return;
    }
    if !ts_sdk::net::rollback() {
        return;
    }
    let n = lw(g::soundbatchlen);
    if n <= 0 {
        return;
    }
    let base = 0x1fb19c0 | 0x20000000;
    let mut keep = 0;
    for i in 0..n {
        let e = base + i * 8;
        let (w0, w1) = (lw(e), lw(e + 4));
        if ts_sdk::net::sound_new([0xb47c, w0 as u32, w1 as u32, 0]) {
            if keep != i {
                sw(base + keep * 8, w0);
                sw(base + keep * 8 + 4, w1);
            }
            keep += 1;
        }
    }
    sw(g::soundbatchlen, keep);
}

/// The level of a network game from the front end is about to be set up: the random numbers and the counts of the game start as at the start of the program.
#[cfg(feature = "extensions")]
static RESEED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn netlink_reseed() {
    #[cfg(feature = "extensions")]
    if RESEED.swap(false, std::sync::atomic::Ordering::Relaxed) {
        newrndseed(0);
        sw(g::tickframenum, 0);
        sw(g::gameframe, 0);
    }
}

/// The sign-on profiles that the players had before a network game from the front end's pages (given back when it is over).
#[cfg(feature = "extensions")]
static PROFILES: std::sync::Mutex<Option<[i32; 4]>> = std::sync::Mutex::new(None);
/// The profiles made for the players of a network game from the front end's pages (their records go when it is over).
#[cfg(feature = "extensions")]
static TEMPS: std::sync::Mutex<Vec<i32>> = std::sync::Mutex::new(Vec::new());
/// A network level has been played since the game was started (the game stops being a network game when it is left).
#[cfg(feature = "extensions")]
static RAN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Start the network match of the lobby from the front end's pages: the players get profiles of their own (named as the lobby says), the level is started with the players'
/// number and the mode; the session (`ts_sdk::net::start`) and the match's setup (`set_setup`) have been made.  The game is a game of the machine again when the level is left.
#[cfg(feature = "extensions")]
pub fn netlink_start_match(level: &str, players: usize) {
    let Some(setup) = ts_sdk::net::setup() else { return };
    // (a test aid: no sounds at all)
    if std::env::var_os("TS_NOSND").is_some() {
        sw(g::gv_playsound, 0);
    }
    let before = [signonGet(0), signonGet(1), signonGet(2), signonGet(3)];
    *PROFILES.lock().unwrap() = Some(before);
    let names = if setup.names.is_empty() { (1..=players).map(|i| format!("Player {i}")).collect() } else { setup.names.clone() };
    let mut temps = Vec::new();
    for (player, n) in names.iter().take(4).enumerate() {
        let text = ts_sdk::malloc(16);
        for (i, b) in n.bytes().take(12).chain(std::iter::once(0)).enumerate() {
            sb(text + i as i32, b as i32);
        }
        let own = signonNew(text);
        if own >= 0 {
            signonSet(player as i32, own);
            temps.push(own);
        }
    }
    *TEMPS.lock().unwrap() = temps;
    let name = ts_sdk::malloc(level.len() as i32 + 1);
    for (i, b) in level.bytes().chain(std::iter::once(0)).enumerate() {
        sb(name + i as i32, b as i32);
    }
    let number = lvGetNumFromName(name);
    // (the block of the game's parameters as `main_` makes it for a level started with a number of players)
    let at = 0x240_0000 + 0xe400;
    for i in (0..112).step_by(4) {
        sw(at + i, 0);
    }
    if !setup.map.is_empty() {
        // (a Map Maker map: the same bytes on every machine, in the game's buffer for maps)
        netsetup_map_install(&setup.map);
        if setup.block.len() == 28 {
            netsetup_prepare_map(setup.block[18]);
        }
    }
    if setup.block.len() == 28 {
        // (the parameters as the game's pages made them on the host, and the variables they were made from)
        netsetup_apply(&setup.gvs, setup.block[18]);
        for (i, w) in setup.block.iter().enumerate() {
            sw(at + 4 * i as i32, *w);
        }
        sw(at, players as i32);
        sw(at + 4, players as i32);
        RESEED.store(true, std::sync::atomic::Ordering::Relaxed);
        gameDeferedReset(at);
        sw(0x1fc11f8, 6);
        return;
    }
    sw(at, players as i32);
    sw(at + 4, players as i32);
    sw(at + 64, number);
    sw(at + 72, setup.mode);
    sw(at + 76, 1);
    sw(at + 80, 2);
    // (the match's options are written into the game's records, which the level reads when it is set up; the random numbers and the counts the game draws from are made the
    // same on every machine when the level begins to be set up, `netlink_reseed`: the machines have been in the front end for different times)
    netlink_options();
    RESEED.store(true, std::sync::atomic::Ordering::Relaxed);
    gameDeferedReset(at);
    sw(0x1fc11f8, 6);
}

/// The statistics of the temporary profile `mine` are added to those of the profile `real` with the game's own combination (the counts added, the best times kept); the lives
/// that follow the combined part (+1616 on) stay as they were.
#[cfg(feature = "extensions")]
fn netlink_keep_stats(real: i32, mine: i32) {
    let (a, b) = (signonGet(0), signonGet(1));
    signonSet(0, real);
    signonSet(1, mine);
    statsCombine(2);
    let array = lw(g::signonarray);
    if std::env::var_os("TS_NETLOG").is_some() {
        eprintln!("[net] statistics: before {:?}, this match {:?}, after {:?}", (0..6).map(|i| lw(array + real * 2936 + 132 + 4 * i)).collect::<Vec<_>>(), (0..6).map(|i| lw(array + mine * 2936 + 132 + 4 * i)).collect::<Vec<_>>(), (0..6).map(|i| lw(0x1fbc168 + 4 * i)).collect::<Vec<_>>());
    }
    memcpy(array + real * 2936 + 132, 0x1fbc168, 1616);
    signonSet(0, a);
    signonSet(1, b);
}

/// A level is being played (not its first frames, nor the screens after it).
#[cfg(feature = "extensions")]
fn in_level() -> bool {
    lw(g::gamestate) == 3 && lw(g::lvfirstframe_130) == 0
}

/// The level of a network game with rollback is being played: the pads are exchanged between the frames.
#[cfg(feature = "extensions")]
pub fn netlink_rollback_level() -> bool {
    ts_sdk::net::rollback() && in_level()
}

/// The pads for the tick `t` as well as they are known: the real ones, and for a player whose pad has not come yet the last one before.
#[cfg(feature = "extensions")]
fn best_pads(t: u32) -> (Vec<[u8; ts_sdk::net::PAD_BYTES]>, u8) {
    let me = ts_sdk::net::local_player() as usize;
    let pads = (0..ts_sdk::net::num_players() as usize)
        .map(|p| ts_sdk::net::pad(p, t).unwrap_or_else(|| if p == me { ts_sdk::net::neutral() } else { ts_sdk::net::guess(p, t) }))
        .collect();
    (pads, ts_sdk::net::gone_mask(t))
}

/// The first tick from `from` before `to` whose pads were guessed wrong (or whose players who left changed).
#[cfg(feature = "extensions")]
fn first_wrong(rb: &Rollback, from: u32, to: u32) -> Option<u32> {
    let me = ts_sdk::net::local_player() as usize;
    for t in from..to {
        let Some((used, mask)) = rb.used.get(&t) else { continue };
        if ts_sdk::net::gone_mask(t) != *mask {
            return Some(t);
        }
        for (p, u) in used.iter().enumerate() {
            if p != me && ts_sdk::net::pad(p, t).map_or(false, |r| r != *u) {
                return Some(t);
            }
        }
    }
    None
}

/// Go back to the tick `e` and play the frames up to `k` (not including it) again with the pads as they are known now.  The journal is left with the record of the tick `k`.
#[cfg(feature = "extensions")]
fn go_back(rb: &mut Rollback, e: u32, k: u32) {
    let n = (k - e) as usize;
    if ts_rt::journal::depth() < n + 1 {
        ts_sdk::net::lost("The game cannot go back so far; the network game ends.");
        return;
    }
    // (what is being drawn must be finished before the memory changes)
    ts_sdk::present_sync();
    ts_rt::journal::undo(n + 1);
    ts_sdk::net::set_replaying(true);
    for t in e..k {
        ts_sdk::net::replay_tick(t);
        ts_rt::journal::begin_tick();
        let (pads, mask) = best_pads(t);
        apply_pads(&pads, mask);
        rb.used.insert(t, (pads, mask));
        sim_frame();
        if t % ts_sdk::net::SUM_EVERY == 0 {
            rb.pending.insert(t, state_sum());
        }
    }
    ts_sdk::net::set_replaying(false);
    ts_sdk::net::set_tick(k);
    ts_rt::journal::begin_tick();
    ts_sdk::net::note_rollback(n as u32);
}

/// Between two frames of the level: the pads of the others that have come are taken in; if one of those was guessed wrong the game goes back and plays the frames
/// again; then this machine's pad is read and sent, and the pads of the next tick (guessed where they have not come) are written for the game.
#[cfg(feature = "extensions")]
pub fn netlink_boundary() {
    if !ts_sdk::net::rollback() {
        return;
    }
    let mut rb = ROLLBACK.lock().unwrap();
    if std::env::var_os("TS_NETLOG").is_some() {
        static CALLS: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        if CALLS.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % 300 == 0 {
            eprintln!("[net] boundary: gamestate {} firstframe {} tick {} started {} ran {}", lw(g::gamestate), lw(g::lvfirstframe_130), rb.tick, rb.started, RAN.load(std::sync::atomic::Ordering::Relaxed));
        }
    }
    if !in_level() {
        if rb.started {
            ts_rt::journal::disable();
            rb.started = false;
            rb.used.clear();
            rb.pending.clear();
            RAN.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        // (the screen after the level: the machines do not exchange pads any more, and each player's own pad is the one that player's buttons read, so that each can go on)
        if RAN.load(std::sync::atomic::Ordering::Relaxed) && lw(g::gamestate) == 4 {
            let me = ts_sdk::net::local_player() as usize;
            let mut pads = vec![ts_sdk::net::neutral(); ts_sdk::net::num_players() as usize];
            if let Some(p) = pads.get_mut(me) {
                *p = local_pad();
            }
            apply_pads(&pads, 0);
        }
        // (the match is over when the level is left: the game is not a network game any more, and the players have their own profiles back)
        if RAN.load(std::sync::atomic::Ordering::Relaxed) && !matches!(lw(g::gamestate), 3 | 4) {
            RAN.store(false, std::sync::atomic::Ordering::Relaxed);
            let local = ts_sdk::net::local_player();
            ts_sdk::net::stop();
            let temps = std::mem::take(&mut *TEMPS.lock().unwrap());
            if let Some(before) = PROFILES.lock().unwrap().take() {
                // (the statistics this machine's player made are added to the profile that was signed in: the players played with profiles of their own, which have the game's
                // starting preferences on every machine, so the preferences of the real profile were never in the play)
                let real = before[0];
                if let Some(&mine) = temps.get(local as usize) {
                    if real >= 0 && real != mine {
                        netlink_keep_stats(real, mine);
                    }
                }
                for (p, id) in before.iter().enumerate() {
                    signonSet(p as i32, *id);
                }
                // (and the profiles that were made go, the last first: the ones before them keep their numbers)
                let mut temps = temps;
                temps.sort_unstable();
                for id in temps.into_iter().rev() {
                    signonDelete(id);
                }
            }
        }
        return;
    }
    let waited_from = std::time::Instant::now();
    if !rb.started {
        if !ts_sdk::net::wait_heard(std::time::Duration::from_secs(90)) {
            ts_sdk::net::lost("The other players did not answer, so the network game ends.");
            return;
        }
        ts_rt::journal::enable(KEEP);
        rb.started = true;
        rb.tick = 0;
        rb.verified = 0;
    }
    let k = rb.tick;
    ts_sdk::net::set_tick(k);
    // test aid: TS_SNDTEST=<sound>,<tick> starts that sound at the player on this machine at that tick
    if let Some((id, at)) = std::env::var("TS_SNDTEST").ok().and_then(|v| v.split_once(',').map(|(a, b)| (a.parse::<i32>().unwrap_or(0), b.parse::<u32>().unwrap_or(0)))) {
        if k == at {
            let me = lw(g::players) + ts_sdk::net::local_player() * 1820;
            if std::env::var_os("TS_SNDTEST_DELAY").is_some() {
                // (a voice, which the game loads when it is wanted and plays some ticks later)
                let slot = soundDelayStartEx(id, me + 152, 20, 0);
                eprintln!("[sndtest] delayed start of {id}: slot {slot}");
            } else if std::env::var_os("TS_SNDTEST_OTHER").is_some() {
                // (at the other player's character)
                let other = lw(g::players) + (1 - ts_sdk::net::local_player()) * 1820;
                let prop = lw(other + 384);
                eprintln!("[sndtest] listener {:.1} {:.1} {:.1} source {:.1} {:.1} {:.1}", lwc1(me + 152), lwc1(me + 156), lwc1(me + 160), lwc1(prop + 48), lwc1(prop + 52), lwc1(prop + 56));
                soundStart(id, prop + 48);
            } else {
                soundStart(id, me + 152);
            }
        }
    }
    // (what is stored from here is the record of this tick: going back to it undoes the pads written below too)
    ts_rt::journal::begin_tick();
    // (a test aid, TS_REPLAYCHECK=1: the tick before is played again from the memory before it, and what differs from the first time is told, which finds what the game reads
    // that is not in its memory)
    if std::env::var_os("TS_REPLAYCHECK").is_some() && k >= 40 && k % 25 == 0 && ts_rt::journal::depth() >= 3 {
        let a = memory_blocks();
        let before: Vec<i32> = (0..(0x8000 / 4)).map(|i| lw(0x32c000 + 4 * i)).collect();
        go_back(&mut rb, k - 1, k);
        let b = memory_blocks();
        let changed: Vec<String> = (0..before.len()).filter(|&i| lw(0x32c000 + 4 * i as i32) != before[i]).take(24).map(|i| format!("{:x}: {:x}->{:x}", 0x32c000 + 4 * i, before[i], lw(0x32c000 + 4 * i as i32))).collect();
        eprintln!("[replay] words of 32c000..334000 that changed: {changed:?}");
        let diff: Vec<String> = (0..a.len()).filter(|&i| a[i] != b[i]).take(30).map(|i| format!("{:x}", block_address(i))).collect();
        eprintln!("[replay] tick {} played again: {} blocks differ{}", k - 1, (0..a.len()).filter(|&i| a[i] != b[i]).count(), if diff.is_empty() { String::new() } else { format!(", the first at {diff:?}") });
    }
    loop {
        if !ts_sdk::net::poll(waited_from.elapsed()) {
            ts_sdk::net::lost("The other players have stopped answering, so the network game ends.");
            return;
        }
        let from = rb.verified;
        if let Some(e) = first_wrong(&rb, from, k) {
            go_back(&mut rb, e, k);
            continue;
        }
        while rb.verified < k && (0..ts_sdk::net::num_players() as usize).all(|p| ts_sdk::net::pad(p, rb.verified).is_some()) {
            rb.verified += 1;
        }
        // (the game does not run on for ever with guessed pads)
        if k < ts_sdk::net::confirmed().saturating_add(ts_sdk::net::MAX_AHEAD) {
            break;
        }
        if !ts_sdk::net::wait_known(k.saturating_sub(ts_sdk::net::MAX_AHEAD), std::time::Duration::from_secs(10)) {
            ts_sdk::net::lost("The other players have stopped answering, so the network game ends.");
            return;
        }
    }
    // the checksum of the state after the last tick
    if k >= 1 && (k - 1) % ts_sdk::net::SUM_EVERY == 0 {
        let sum = state_sum();
        if std::env::var("TS_NETDUMP3").ok().and_then(|v| v.parse::<u32>().ok()) == Some(k - 1) {
            // test aid: a checksum of every 256 bytes of the game's memory, to find where two machines differ (and the words of those places with TS_NETDUMP3_WORDS)
            let words = std::env::var_os("TS_NETDUMP3_WORDS").is_some();
            for page in 0..(0x1000_0000 / 256) {
                let at = page * 256;
                let mut h: u32 = 0x811c_9dc5;
                for w in 0..64 {
                    h = (h ^ lw(at + 4 * w) as u32).wrapping_mul(0x0100_0193);
                }
                if !words {
                    eprintln!("[pg] {at:08x} {h:08x}");
                }
                let from = std::env::var("TS_NETDUMP3_FROM").ok().and_then(|v| i32::from_str_radix(&v, 16).ok()).unwrap_or(0);
                let to = std::env::var("TS_NETDUMP3_TO").ok().and_then(|v| i32::from_str_radix(&v, 16).ok()).unwrap_or(0x200_0000);
                if words && at >= from && at < to {
                    for w in 0..64 {
                        eprintln!("[pw] {:08x} {:08x}", at + 4 * w, lw(at + 4 * w));
                    }
                }
            }
        }
        if std::env::var("TS_NETDUMP").ok().and_then(|v| v.parse::<u32>().ok()) == Some(k - 1) {
            // test aid: the words of the state that the checksum covers, at this tick, to find which differ between machines
            for i in 0..lw(g::lvnumplayers) {
                for w in 0..455 {
                    eprintln!("[dump] P{i} +{} {:08x}", w * 4, lw(lw(g::players) + i * 1820 + w * 4));
                }
            }
            eprintln!("[dump] R seed {:016x} count {} frame {}", ld(g::newrndseedval), lw(g::rndcount), lw(g::gameframe));
            let count = lw(g::lvnumplayers) + lw(g::numactivechars);
            for i in 0..count {
                for w in 0..(0x1210 / 4) {
                    eprintln!("[dump] C{i} +{} {:08x}", w * 4, lw(lw(g::chrinfo) + i * 0x1210 + w * 4));
                }
            }
            for w in 0..(14 * 1368 / 4) {
                eprintln!("[dump] M +{} {:08x}", w * 4, lw(g::matchstats + 4 * w));
            }
            for w in 0..28 {
                eprintln!("[dump] G +{} {:08x}", w * 4, lw(g::curgameparms + 4 * w));
            }
        }
        rb.pending.insert(k - 1, sum);
    }
    let certain = ts_sdk::net::confirmed();
    let ready: Vec<u32> = rb.pending.keys().copied().filter(|&t| t < certain && t < rb.verified).collect();
    for t in ready {
        if let Some(sum) = rb.pending.remove(&t) {
            ts_sdk::net::submit_sum(t, sum);
            if std::env::var_os("TS_NETLOG").is_some() {
                eprintln!("[net] tick {t} sum {sum:08x}{}", where_everybody());
            }
        }
    }
    check_out_of_step();
    // this machine's pad: read, and sent (it is used a few ticks on)
    joyTick();
    ts_sdk::net::submit(k, local_pad());
    let (pads, mask) = best_pads(k);
    apply_pads(&pads, mask);
    rb.used.insert(k, (pads, mask));
    rb.tick = k + 1;
    if k % 64 == 0 {
        ts_sdk::net::prune_before(k.saturating_sub(48));
        let keep_from = k.saturating_sub(48);
        rb.used = rb.used.split_off(&keep_from);
    }
    ts_sdk::net::note_boundary(k, waited_from.elapsed());
}

/// The options of the match the lobby set (called where the mods' presets are written, after the game made the records of the options).
pub fn netlink_options() {
    let Some(s) = ts_sdk::net::setup() else { return };
    if !s.block.is_empty() {
        return;
    }
    let rec = 0x1fc2c98 + (s.mode << 5);
    sw(rec + 12, s.time);
    match s.mode {
        3 => sw(rec + 16, s.limit),
        2 | 5 => sw(rec + 20, s.limit),
        _ => {}
    }
    sw(rec + 4, s.friendly as i32);
}

/// The mode and the bots of the match the lobby set (called when the game's parameters have been given their starts).
pub fn netlink_parms() {
    let Some(s) = ts_sdk::net::setup() else { return };
    if !s.block.is_empty() {
        return;
    }
    sw(g::curgameparms + 72, s.mode);
}

/// The character of a player the lobby set (at the start of a player's restart).
pub fn netlink_player_char(player: i32) {
    let Some(s) = ts_sdk::net::setup() else { return };
    let Some(&chr) = s.chars.get(player as usize) else { return };
    let id = signonGet(player);
    if id >= 0 && (0..128).contains(&chr) {
        sw(lw(g::signonarray) + id * 2936 + 32 + 84, chr);
    }
}

/// The bots the lobby set (at the start of the bots' restart).
pub fn netlink_bots() {
    let Some(s) = ts_sdk::net::setup() else { return };
    if !s.block.is_empty() {
        return;
    }
    let parms = g::curgameparms;
    let players = lw(g::lvnumplayers).max(1);
    let n = s.bots.len().min(10);
    sw(parms + 8, n as i32);
    for (i, b) in s.bots.iter().take(n).enumerate() {
        sb(parms + 21 + players + i as i32, *b);
    }
}

/// The characters of the game: (number, name), those with a name in the menus.
pub fn netlink_characters() -> Vec<(i32, String)> {
    let mut v = Vec::new();
    if std::env::var_os("TS_CHRDUMP").is_some() {
        for chr in 0..12 {
            let words: Vec<i32> = (0..11).map(|k| lw(CHRS_TABLE + 44 * chr + 4 * k)).collect();
            eprintln!("[chr] {chr} {:?}", words);
            eprintln!("[chr] {chr} flinch sound {}", lw(g::flinchsfx + 4 * chr));
        }
    }
    for chr in 0..128 {
        let id = lhu(CHRS_TABLE + 44 * chr);
        if id == 0 {
            continue;
        }
        let mut at = lw(lw(g::languages + 4 * lw(g::cur_language)) + 4 * id);
        let mut name = String::new();
        while at != 0 && lbu(at) != 0 && name.len() < 30 {
            name.push(lbu(at) as u8 as char);
            at += 1;
        }
        if !name.is_empty() {
            v.push((chr, name));
        }
    }
    v
}

/// The weapon sets the lobby offers: (the set's number, its five weapons): the custom ones that have weapons (from the file of the sets, the first from the
/// game's own when there is no file), then the nine presets.
pub fn netlink_weapon_sets() -> Vec<(i32, [i32; 5])> {
    let mut v = Vec::new();
    let text = std::fs::read_to_string(ts_sdk::save_root().join("weapon_sets.txt")).unwrap_or_default();
    for (i, line) in text.lines().take(5).enumerate() {
        let nums: Vec<i32> = line.split_whitespace().filter_map(|x| x.parse().ok()).collect();
        if nums.len() == 5 && nums[0] >= 0 {
            v.push((if i == 0 { 0 } else { 9 + i as i32 }, [nums[0], nums[1], nums[2], nums[3], nums[4]]));
        }
    }
    if v.is_empty() {
        let mut a = [0; 5];
        for (k, w) in a.iter_mut().enumerate() {
            *w = lw(g::gunzpack + 4 * k as i32);
        }
        v.push((0, a));
    }
    for p in 1..10 {
        v.push((p, [0; 5]));
    }
    v
}

/// The names of the levels the game has, in the order of its table (the lobby offers them).
pub fn netlink_level_names() -> Vec<String> {
    let mut v = Vec::new();
    let language = lw(g::cur_language);
    let mut entry = LVINFO;
    for _ in 0..200 {
        let text = lw((lh(entry) << 2) + lw((language << 2) + g::languages));
        if text == 0 {
            break;
        }
        let name = ts_rt::cstr(text);
        if lw(entry + 4) < 100 && !name.is_empty() {
            v.push(name);
        }
        entry += 24;
    }
    v
}

/// The number of players on this machine: in a network game the one.
pub fn netlink_local_players(all: i32) -> i32 {
    if ts_sdk::net::active() {
        1
    } else {
        all
    }
}

/// The number of the player on this machine.
pub fn netlink_start_player() -> i32 {
    // (only in a level: the front end and the loading before it have their players' records of the first player only)
    if ts_sdk::net::active() && matches!(lw(g::gamestate), 3 | 4) {
        ts_sdk::net::local_player()
    } else {
        0
    }
}

/// The controller slot of a player: in a network game each player has the slot of the number.
pub fn netlink_slot(player: i32, own: i32) -> i32 {
    if ts_sdk::net::active() {
        player
    } else {
        own
    }
}
