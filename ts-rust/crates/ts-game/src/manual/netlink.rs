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
    if !ts_sdk::net::active() || lw(g::gamestate) != 3 {
        return;
    }
    let mut local = pad_bytes(g::controllers + LOCAL_PAD * 384);
    if lw(g::gamepaused) != 0 {
        // (the game goes on while this player's pause menu is open, and the menu reads the same pad: its buttons go on, the sticks do not move the character)
        local[8..12].fill(128);
    }
    let Some(pads) = ts_sdk::net::step(local) else {
        ts_sdk::net::lost("The other players have stopped answering, so the network game ends.");
        return;
    };
    for (p, b) in pads.iter().enumerate() {
        let record = FIRST_NET_PAD + p as i32;
        put_pad(g::controllers + record * 384, b);
        sw(g::playercontrollers + 4 * p as i32, record);
    }
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

/// The options of the match the lobby set (called where the mods' presets are written, after the game made the records of the options).
pub fn netlink_options() {
    let Some(s) = ts_sdk::net::setup() else { return };
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
