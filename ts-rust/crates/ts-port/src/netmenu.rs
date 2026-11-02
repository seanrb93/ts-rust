//! The Network Game pages of the game's front end: the lobby (`lobby.rs`) shown as the game's own menu pages.
//!
//! The lobby keeps the network (the games found, the players that have joined, the match's settings); this file gives its rows to the pages (`AvOptions::net_*`), takes
//! what the player chooses back to it, and when the match is decided it starts the session and the level.

use crate::lobby::{Lobby, UiKind, View};
use std::sync::Mutex;

static LOBBY: Mutex<Option<Lobby>> = Mutex::new(None);

/// A lobby for the pages: the levels and the characters of the game, the weapon sets, and what the machines must have in common.
fn new_lobby() -> Lobby {
    crate::new_lobby()
}

fn note(what: &str) {
    if std::env::var_os("TS_NETLOG").is_some() {
        eprintln!("[netmenu] {what}");
    }
}

pub fn open() {
    note("open");
    ts_game::netsetup_dump();
    *LOBBY.lock().unwrap() = Some(new_lobby());
}

pub fn close() {
    note("close");
    *LOBBY.lock().unwrap() = None;
}

/// Once a tick of the page: the network is looked at; when the match is decided the session and the level are started.
pub fn tick() {
    let outcome = {
        let mut g = LOBBY.lock().unwrap();
        let l = g.get_or_insert_with(new_lobby);
        l.tick();
        match l.finished() {
            Some(o) if o.players >= 2 => {
                note("start");
                // (the sockets of the lobby are closed before the session binds the same ports)
                *g = None;
                Some(o)
            }
            Some(_) => {
                note("finished with fewer than two players");
                *g = Some(new_lobby());
                None
            }
            None => None,
        }
    };
    if let Some(o) = outcome {
        start(o);
    }
}

fn start(o: crate::lobby::Outcome) {
    let cfg = ts_sdk::net::Config { local: o.local, peers: o.peers.clone(), bind: o.bind, seed: o.seed, delay: o.setup.delay };
    if let Err(e) = ts_sdk::net::start(cfg) {
        eprintln!("[net] The network game cannot start: {e}");
        open();
        return;
    }
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
                UiKind::Value => ts_game::NetKind::Value,
                UiKind::Action => ts_game::NetKind::Action,
                UiKind::Info => ts_game::NetKind::Info,
            },
        })
        .collect()
}

pub fn set(row: usize, index: usize) {
    if let Some(l) = LOBBY.lock().unwrap().as_mut() {
        l.set_row(row, index);
    }
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
