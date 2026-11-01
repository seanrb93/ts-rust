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
    h
}

/// Where the players are (a line of the log, to see two machines agree).
fn where_everybody() -> String {
    let mut s = String::new();
    for i in 0..lw(g::lvnumplayers) {
        let prop = lw(lw(g::players) + i * 1820 + 384);
        if prop != 0 {
            s += &format!(" p{} ({:.1} {:.1} {:.1})", i, lwc1(prop + 48), lwc1(prop + 52), lwc1(prop + 56));
        }
    }
    s
}

/// Called at the start of each frame (`ilinkTick`) of a network game once the pads have been read: in a level, this machine's pad is given
/// to the others, and the pads of every player for the tick are written into their controller records.
pub fn netlink_tick() {
    if !ts_sdk::net::active() || lw(g::gamestate) != 3 {
        return;
    }
    let local = pad_bytes(g::controllers + LOCAL_PAD * 384);
    let Some(pads) = ts_sdk::net::step(local) else {
        eprintln!("[net] the link is lost: the game ends");
        std::process::exit(3);
    };
    for (p, b) in pads.iter().enumerate() {
        let record = FIRST_NET_PAD + p as i32;
        put_pad(g::controllers + record * 384, b);
        sw(g::playercontrollers + 4 * p as i32, record);
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
    if ts_sdk::net::active() {
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
