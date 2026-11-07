//! The Network Game pages of the game's front end: the lobby (`lobby.rs`) shown as the game's own menu pages.
//!
//! The lobby keeps the network (the games found, the players that have joined, the match's settings); this file gives its rows to the pages (`AvOptions::net_*`), takes
//! what the player chooses back to it, and when the match is decided it starts the session and the level.

use crate::lobby::{Lobby, UiKind, View};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

static LOBBY: Mutex<Option<Lobby>> = Mutex::new(None);

/// A lobby for the pages: the levels and the characters of the game, the weapon sets, and what the machines must have in common.
fn new_lobby() -> Lobby {
    let mut l = crate::new_lobby();
    l.use_pages();
    l
}

fn note(what: &str) {
    if std::env::var_os("TS_NETLOG").is_some() {
        eprintln!("[netmenu] {what}");
    }
}

/// A match was played: where this machine goes back to (the host to its settings, a player to the game's details).
#[derive(Clone, Copy)]
enum Resume {
    Host,
    Client(std::net::SocketAddr),
}

static RESUME: Mutex<Option<Resume>> = Mutex::new(None);
/// The host is to go on to the match settings when the page is first seen.
static HOP: AtomicBool = AtomicBool::new(false);
/// The front end's number of players while the Network pages are open (they are for this machine's one player).
static FRONT_PLAYERS: Mutex<Option<i32>> = Mutex::new(None);

pub fn open() {
    note("open");
    // (a test aid: the front end has the number of players of a local game, as if 2up, 3up or 4up was chosen before)
    if let Some(n) = std::env::var("TS_FRONTPLAYERS").ok().and_then(|v| v.parse::<i32>().ok()) {
        ts_rt::sw(0x3aff20, n);
    }
    FRONT_PLAYERS.lock().unwrap().get_or_insert_with(|| ts_rt::lw(0x3aff20));
    ts_rt::sw(0x3aff20, 1);
    ts_game::netsetup_dump();
    let mut l = new_lobby();
    match RESUME.lock().unwrap().take() {
        Some(Resume::Host) => {
            l.begin_host();
            HOP.store(l.view() == View::Host, Ordering::Relaxed);
        }
        Some(Resume::Client(host)) => l.join_address(host),
        None => {}
    }
    *LOBBY.lock().unwrap() = Some(l);
}

pub fn close() {
    note("close");
    *LOBBY.lock().unwrap() = None;
    if let Some(n) = FRONT_PLAYERS.lock().unwrap().take() {
        ts_rt::sw(0x3aff20, n);
    }
}

/// Once a tick of the Network page itself: the player is back on it from the game's pages.
pub fn tick() {
    {
        let mut g = LOBBY.lock().unwrap();
        let l = g.get_or_insert_with(new_lobby);
        if HOP.swap(false, Ordering::Relaxed) {
            ts_game::netsetup_open_page(3);
        } else {
            l.page_shown();
        }
    }
    background();
}

/// Once a tick of the front end on any page: the network is looked at; when the match is decided the session and the level are started.
pub fn background() {
    // (a test aid: `TS_NETPAGES=1` opens the lobby of the pages once the front end runs, so that `TS_LOBBY_AUTO` starts a match as the Network pages do, with the bots and the
    // teams of the game's own pages)
    static AUTO_OPENED: AtomicBool = AtomicBool::new(false);
    if std::env::var_os("TS_NETPAGES").is_some() && LOBBY.lock().unwrap().is_none() && !AUTO_OPENED.swap(true, Ordering::Relaxed) {
        open();
    }
    let mut rejoin = None;
    let (outcome, back_out) = {
        let mut g = LOBBY.lock().unwrap();
        let Some(l) = g.as_mut() else { return };
        // (a test aid: limits of kills and of score, so that a match ends soon)
        if let Some(n) = std::env::var("TS_KILLLIMIT").ok().and_then(|v| v.parse::<i32>().ok()) {
            ts_game::netsetup_gv_set(ts_game::g::gv_killlimit, n);
            ts_game::netsetup_gv_set(ts_game::g::gv_scorelimit, n);
        }
        // (and of the minutes)
        if let Some(n) = std::env::var("TS_TIMELIMIT").ok().and_then(|v| v.parse::<i32>().ok()) {
            ts_game::netsetup_gv_set(ts_game::g::gv_timelimit, n);
        }
        l.tick();
        let back_out = l.take_host_gone();
        match l.finished() {
            Some(o) if o.players >= 2 => {
                note("start");
                rejoin = l.rejoin_data();
                if l.choosing_character() {
                    ts_game::netflow_leave_chr();
                }
                // (the sockets of the lobby are closed before the session binds the same ports)
                *g = None;
                (Some(o), false)
            }
            Some(_) => {
                note("finished with fewer than two players");
                *g = Some(new_lobby());
                (None, false)
            }
            None => (None, back_out),
        }
    };
    if back_out {
        ts_game::netflow_back_out();
    }
    if let Some(o) = outcome {
        start(o);
        if let Some(r) = rejoin {
            ts_sdk::net::set_rejoin(r);
        }
    }
}

pub fn ready_line() -> (String, bool) {
    LOBBY.lock().unwrap().as_ref().map_or((String::new(), false), |l| l.ready_line())
}

pub fn start_now() {
    if let Some(l) = LOBBY.lock().unwrap().as_mut() {
        l.start_now();
    }
}

fn start(o: crate::lobby::Outcome) {
    let cfg = ts_sdk::net::Config { rejoin: o.rejoin, local: o.local, peers: o.peers.clone(), bind: o.bind, seed: o.seed, delay: o.setup.delay };
    if let Err(e) = ts_sdk::net::start(cfg) {
        eprintln!("[net] The network game cannot start: {e}");
        open();
        return;
    }
    *RESUME.lock().unwrap() = Some(if o.local == 0 { Resume::Host } else { Resume::Client(o.peers[0]) });
    note(&format!("setup {:?} level {} players {} local {} seed {}", o.setup, o.level, o.players, o.local, o.seed));
    ts_sdk::net::set_setup(o.setup.clone());
    ts_sdk::net::set_rollback(std::env::var_os("TS_LOCKSTEP").is_none());
    #[cfg(feature = "extensions")]
    ts_game::netlink_start_match(&o.level, o.players);
}

pub fn rows() -> Vec<ts_game::NetRow> {
    let g = LOBBY.lock().unwrap();
    let Some(l) = g.as_ref() else { return Vec::new() };
    l.rows()
        .into_iter()
        .map(|r| ts_game::NetRow {
            row: ts_game::AvRow { label: r.label, values: r.values, index: r.index },
            kind: match r.kind {
                UiKind::Action => ts_game::NetKind::Action,
                UiKind::Info => ts_game::NetKind::Info,
            },
        })
        .collect()
}

pub fn action(row: usize) {
    if let Some(l) = LOBBY.lock().unwrap().as_mut() {
        l.act(row);
    }
}

pub fn back() -> bool {
    LOBBY.lock().unwrap().as_mut().map_or(false, |l| l.back())
}

pub fn title() -> String {
    LOBBY.lock().unwrap().as_ref().map_or("Network Game", |l| l.title()).to_string()
}

pub fn hint(editing: bool) -> String {
    LOBBY.lock().unwrap().as_ref().map_or(String::new(), |l| l.hint(editing))
}

pub fn typed(c: char) {
    if let Some(l) = LOBBY.lock().unwrap().as_mut() {
        l.type_char(c);
    }
}

pub fn delete() {
    if let Some(l) = LOBBY.lock().unwrap().as_mut() {
        l.backspace();
    }
}

#[allow(dead_code)]
pub fn view() -> Option<View> {
    LOBBY.lock().unwrap().as_ref().map(|l| l.view())
}
