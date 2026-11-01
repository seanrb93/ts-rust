//! The lobby of a network game: a screen shown before the game starts, where one player hosts and the others join, and the host picks the level.
//! It ends with what `ts_sdk::net::start` and the game's start-up need (the level, the players, this machine's player, the seed and the addresses).
//!
//! The lobby has its own UDP messages (the game's, in `ts-sdk/src/native/net.rs`, start when the lobby ends): a joining machine sends JOIN to the
//! host every 400 ms; the host answers each with the ROSTER (the player's number, the players' addresses, the level and the seed) and, when it
//! starts the game, sends the ROSTER marked "starting" for a while so that every machine gets one.

use std::net::{SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

const MAGIC: u32 = 0x4c42_5354; // "TSBL"
const T_JOIN: u8 = 1;
const T_LEAVE: u8 = 2;
const T_ROSTER: u8 = 3;
const T_CHAR: u8 = 4;
const T_REJECT: u8 = 5;
/// a host's announcement on the local network, sent to the broadcast address
const T_BEACON: u8 = 6;
/// the port the announcements go to (a joining machine listens on it; it is not the host's own port, so a host and a joiner can share a PC)
const BEACON_PORT: u16 = 7778;
/// the port a host listens on
pub const PORT: u16 = 7777;
const MAX_PLAYERS: usize = 4;
/// the rows of the host's screen: the level, the mode, the time limit, the score limit, the bots, friendly fire and the character
const HOST_ROWS: usize = 8;
/// the modes the lobby offers: (number, name)
const MODES: [(i32, &str); 4] = [(3, "Deathmatch"), (1, "Bag Tag"), (2, "Capture the Bag"), (5, "Knockout")];

/// What the lobby decided.
#[derive(Clone, Debug)]
pub struct Outcome {
    pub level: String,
    pub players: usize,
    pub local: usize,
    pub bind: SocketAddr,
    pub peers: Vec<SocketAddr>,
    pub seed: i32,
    pub setup: ts_sdk::net::Setup,
}

/// A key or a button of the lobby.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
    Enter,
    Back,
    Backspace,
    Char(char),
}

#[derive(Clone, Copy, PartialEq)]
enum Screen {
    Main,
    Host,
    Join,
}

pub struct Lobby {
    screen: Screen,
    sel: usize,
    levels: Vec<String>,
    level: usize,
    addr_text: String,
    status: String,
    sock: Option<UdpSocket>,
    // host: the other players' addresses (index 0 of the players is this machine)
    others: Vec<SocketAddr>,
    /// this machine's player's name, the names of the other machines' players (host) and of every player (client)
    my_name: String,
    names: std::collections::HashMap<SocketAddr, String>,
    roster_names: Vec<String>,
    // client: the host, this machine's player and the roster as last heard
    host: Option<SocketAddr>,
    my_index: usize,
    roster: Vec<SocketAddr>,
    n_players: usize,
    seed: i32,
    /// the host has started: when (it keeps announcing for a moment before it goes)
    starting: Option<Instant>,
    last_send: Instant,
    done: Option<Outcome>,
    /// a client has seen the start
    client_start: bool,
    /// the match: what the host chose (a client sees it as the host sends it)
    setup: ts_sdk::net::Setup,
    /// the characters of the game: (number, name)
    chars: Vec<(i32, String)>,
    /// the row of the screen the cursor is on
    row: usize,
    /// the number of bots
    bots: usize,
    /// the weapon sets that can be chosen: (number, the five weapons of a custom one) and the one chosen
    wsets: Vec<(i32, [i32; 5])>,
    wset: usize,
    /// what the machines must have in common (the build, the game's files, the mods)
    compat: u32,
    /// the character each player chose (this machine's own at `my_index`; the host keeps its clients' here)
    picks: Vec<usize>,
    /// host: the socket the announcements go out of; client: the one they come in on
    beacon: Option<UdpSocket>,
    last_beacon: Instant,
    /// the hosts heard on the network: (address, when last heard, their compat hash, players there, level)
    found: Vec<(SocketAddr, Instant, u32, u8, String)>,
    found_sel: usize,
    /// (a test aid) the lobby answers itself: `host,<players>,<level>` or `join,<address>`
    auto: Option<Auto>,
}

#[derive(Clone)]
enum Auto {
    Host(usize),
    Join,
    /// look for a game on the network and join the first one found
    Find,
}

/// A name as the game can show it: letters, digits and spaces, at most 12.
fn clean_name(s: &str) -> String {
    let n: String = s.chars().filter(|c| c.is_ascii_alphanumeric() || *c == ' ').take(12).collect();
    let n = n.trim().to_string();
    if n.is_empty() {
        "Player".into()
    } else {
        n
    }
}

/// The name this machine's player has: the user's name on the computer.
fn default_name() -> String {
    clean_name(&std::env::var("USERNAME").or_else(|_| std::env::var("USER")).unwrap_or_default())
}

fn now_seed() -> i32 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(1, |d| (d.subsec_nanos() ^ d.as_secs() as u32) as i32 & 0x7fff_ffff)
}

impl Lobby {
    pub fn new(levels: Vec<String>, chars: Vec<(i32, String)>, compat: u32, wsets: Vec<(i32, [i32; 5])>) -> Lobby {
        let level = levels.iter().position(|l| l.eq_ignore_ascii_case("Prison")).unwrap_or(0);
        let mut l = Lobby {
            screen: Screen::Main,
            sel: 0,
            levels,
            level,
            addr_text: std::env::var("TS_LOBBY_ADDR").unwrap_or_default(),
            status: String::new(),
            sock: None,
            others: Vec::new(),
            my_name: default_name(),
            names: std::collections::HashMap::new(),
            roster_names: Vec::new(),
            host: None,
            my_index: 0,
            roster: Vec::new(),
            n_players: 1,
            seed: 0,
            starting: None,
            last_send: Instant::now() - Duration::from_secs(1),
            done: None,
            client_start: false,
            setup: ts_sdk::net::Setup::default(),
            chars,
            row: 0,
            bots: 0,
            wsets,
            wset: 0,
            // (a test aid: a machine that is not like the others)
            compat: compat ^ std::env::var("TS_LOBBY_COMPAT_XOR").ok().and_then(|v| v.parse::<u32>().ok()).unwrap_or(0),
            picks: vec![0; MAX_PLAYERS],
            beacon: None,
            last_beacon: Instant::now() - Duration::from_secs(5),
            found: Vec::new(),
            found_sel: 0,
            auto: None,
        };
        l.apply_wset();
        // the test aid
        if let Ok(a) = std::env::var("TS_LOBBY_AUTO") {
            let parts: Vec<&str> = a.split(',').collect();
            match parts.as_slice() {
                ["host", n, level] => {
                    if let Some(i) = l.levels.iter().position(|x| x.eq_ignore_ascii_case(level)) {
                        l.level = i;
                    }
                    l.auto = Some(Auto::Host(n.parse().unwrap_or(2)));
                    // (the match, for a test: mode=<n>,time=<minutes>,limit=<points>,bots=<count>,char=<index>)
                    for part in std::env::var("TS_LOBBY_SETUP").unwrap_or_default().split(',') {
                        match part.split_once('=') {
                            Some(("mode", v)) => l.setup.mode = v.parse().unwrap_or(3),
                            Some(("time", v)) => l.setup.time = v.parse().unwrap_or(0),
                            Some(("limit", v)) => l.setup.limit = v.parse().unwrap_or(20),
                            Some(("bots", v)) => l.bots = v.parse().unwrap_or(0),
                            Some(("char", v)) => l.picks[0] = v.parse().unwrap_or(0),
                            _ => {}
                        }
                    }
                    l.begin_host();
                }
                ["join", addr] => {
                    l.addr_text = addr.to_string();
                    l.auto = Some(Auto::Join);
                    l.begin_join();
                }
                ["find"] => l.auto = Some(Auto::Find),
                _ => {}
            }
        }
        l
    }

    pub fn finished(&self) -> Option<Outcome> {
        self.done.clone()
    }

    fn begin_host(&mut self) {
        match UdpSocket::bind(("0.0.0.0", PORT)) {
            Ok(s) => {
                let _ = s.set_nonblocking(true);
                self.sock = Some(s);
                self.screen = Screen::Host;
                self.others.clear();
                self.starting = None;
                self.status = format!("Waiting for players on port {PORT}");
            }
            Err(e) => self.status = format!("Cannot listen on port {PORT}: {e}"),
        }
    }

    fn begin_join(&mut self) {
        let text = self.addr_text.trim();
        let addr: Option<SocketAddr> = text.parse().ok().or_else(|| format!("{text}:{PORT}").parse().ok());
        let Some(addr) = addr else {
            self.status = "That is not an address (like 192.168.1.20 or 192.168.1.20:7777)".into();
            return;
        };
        match UdpSocket::bind("0.0.0.0:0") {
            Ok(s) => {
                let _ = s.set_nonblocking(true);
                self.sock = Some(s);
                self.beacon = None;
                self.host = Some(addr);
                self.screen = Screen::Join;
                self.client_start = false;
                self.roster.clear();
                self.status = format!("Asking {addr} ...");
            }
            Err(e) => self.status = format!("Cannot open a socket: {e}"),
        }
    }

    fn leave(&mut self) {
        if let (Some(s), Some(h)) = (&self.sock, self.host) {
            let mut b = MAGIC.to_le_bytes().to_vec();
            b.push(T_LEAVE);
            let _ = s.send_to(&b, h);
        }
        self.sock = None;
        self.host = None;
        self.beacon = None;
        self.others.clear();
        self.roster.clear();
        self.starting = None;
        self.screen = Screen::Main;
        self.status.clear();
    }

    pub fn key(&mut self, k: Key) {
        match self.screen {
            Screen::Main => match k {
                Key::Up => self.sel = (self.sel + 2) % 3,
                Key::Down => self.sel = (self.sel + 1) % 3,
                Key::Enter => match self.sel {
                    0 => self.begin_host(),
                    1 => {
                        self.screen = Screen::Join;
                        self.sock = None;
                        self.host = None;
                        self.beacon = None;
                        self.found.clear();
                        self.found_sel = 0;
                        self.status = "Pick a game that was found, or type the host's address, and press Enter".into();
                    }
                    _ => self.done = Some(Outcome { level: String::new(), players: 0, local: 0, bind: "0.0.0.0:0".parse().unwrap(), peers: Vec::new(), seed: 0, setup: Default::default() }),
                },
                Key::Back => self.done = Some(Outcome { level: String::new(), players: 0, local: 0, bind: "0.0.0.0:0".parse().unwrap(), peers: Vec::new(), seed: 0, setup: Default::default() }),
                _ => {}
            },
            Screen::Host => match k {
                Key::Up if self.starting.is_none() => self.row = (self.row + HOST_ROWS - 1) % HOST_ROWS,
                Key::Down if self.starting.is_none() => self.row = (self.row + 1) % HOST_ROWS,
                Key::Left if self.starting.is_none() => self.change(-1),
                Key::Right if self.starting.is_none() => self.change(1),
                Key::Enter if self.starting.is_none() && !self.others.is_empty() => {
                    self.seed = now_seed();
                    self.starting = Some(Instant::now());
                    self.status = "Starting ...".into();
                }
                Key::Enter if self.others.is_empty() => self.status = "Nobody has joined yet".into(),
                Key::Back => self.leave(),
                _ => {}
            },
            Screen::Join => {
                if self.sock.is_some() {
                    match k {
                        Key::Back => self.leave(),
                        Key::Left => self.change_char(-1),
                        Key::Right => self.change_char(1),
                        _ => {}
                    }
                    return;
                }
                match k {
                    Key::Char(c) if c.is_ascii_digit() || c == '.' || c == ':' => self.addr_text.push(c),
                    Key::Backspace => {
                        self.addr_text.pop();
                    }
                    Key::Up if !self.found.is_empty() => self.found_sel = (self.found_sel + self.found.len() - 1) % self.found.len(),
                    Key::Down if !self.found.is_empty() => self.found_sel = (self.found_sel + 1) % self.found.len(),
                    Key::Enter => {
                        if self.addr_text.trim().is_empty() {
                            if let Some(f) = self.found.get(self.found_sel) {
                                self.addr_text = f.0.to_string();
                            }
                        }
                        self.begin_join();
                    }
                    Key::Back => {
                        self.beacon = None;
                        self.screen = Screen::Main;
                        self.status.clear();
                    }
                    _ => {}
                }
            }
        }
    }

    /// Change the value of the row the host's cursor is on.
    fn change(&mut self, d: i32) {
        let step = |v: i32, lo: i32, hi: i32| (v + d).clamp(lo, hi);
        let n = self.levels.len().max(1) as i32;
        match self.row {
            0 => self.level = ((self.level as i32 + d).rem_euclid(n)) as usize,
            1 => {
                let at = MODES.iter().position(|m| m.0 == self.setup.mode).unwrap_or(0) as i32;
                self.setup.mode = MODES[(at + d).rem_euclid(MODES.len() as i32) as usize].0;
            }
            2 => self.setup.time = step(self.setup.time, 0, 60),
            3 => self.setup.limit = step(self.setup.limit, 1, 100),
            4 => self.bots = step(self.bots as i32, 0, 6) as usize,
            5 => self.setup.friendly = !self.setup.friendly,
            6 => {
                let n = self.wsets.len().max(1) as i32;
                self.wset = (self.wset as i32 + d).rem_euclid(n) as usize;
                self.apply_wset();
            }
            _ => self.change_char(d),
        }
    }

    /// The weapon set chosen goes into the match.
    fn apply_wset(&mut self) {
        self.setup.weapons = self.wsets.get(self.wset).map(|w| (if w.0 == 0 || w.0 >= 10 { 0 } else { w.0 }, w.1));
    }

    /// The name of a weapon set.
    fn wset_name(&self, i: usize) -> String {
        match self.wsets.get(i) {
            Some(&(v, _)) if v == 0 => "Custom 1".into(),
            Some(&(v, _)) if v >= 10 => format!("Custom {}", v - 8),
            Some(&(v, _)) => format!("Preset {v}"),
            None => "?".into(),
        }
    }

    /// Change the character this machine's player plays.
    fn change_char(&mut self, d: i32) {
        if self.chars.is_empty() {
            return;
        }
        let n = self.chars.len() as i32;
        let at = (self.picks[self.my_index.min(MAX_PLAYERS - 1)] as i32 + d).rem_euclid(n) as usize;
        self.picks[self.my_index.min(MAX_PLAYERS - 1)] = at;
        // (a client tells the host)
        if self.screen == Screen::Join {
            if let (Some(s), Some(h)) = (&self.sock, self.host) {
                let mut b = MAGIC.to_le_bytes().to_vec();
                b.push(T_CHAR);
                b.push(at as u8);
                let _ = s.send_to(&b, h);
            }
        }
    }

    /// The names of the players as the host has them (this machine's first).
    fn host_names(&self) -> Vec<String> {
        let mut v = vec![self.my_name.clone()];
        for a in &self.others {
            v.push(self.names.get(a).cloned().unwrap_or_else(|| format!("Player {}", v.len() + 1)));
        }
        v
    }

    /// The match as it stands: the options, the bots and the character of each player.
    fn match_setup(&self, players: usize) -> ts_sdk::net::Setup {
        let mut s = self.setup.clone();
        let chr = |i: usize| self.chars.get(i % self.chars.len().max(1)).map_or(0, |c| c.0);
        s.chars = (0..players).map(|p| chr(self.picks.get(p).copied().unwrap_or(0))).collect();
        s.names = if self.screen == Screen::Host { self.host_names() } else { self.roster_names.clone() };
        // (the bots are characters the players are not: the next ones of the list)
        s.bots = (0..self.bots).map(|b| chr(8 + 3 * b)).collect();
        s
    }

    fn roster_packet(&self, to_index: usize, starting: bool) -> Vec<u8> {
        let mut b = MAGIC.to_le_bytes().to_vec();
        b.push(T_ROSTER);
        b.push(starting as u8);
        b.push(to_index as u8);
        b.push((1 + self.others.len()) as u8);
        b.extend_from_slice(&self.seed.to_le_bytes());
        let name = self.levels.get(self.level).cloned().unwrap_or_default();
        b.push(name.len().min(60) as u8);
        b.extend_from_slice(&name.as_bytes()[..name.len().min(60)]);
        // the players' addresses: the host (player 0) is known to the clients, the others as the host sees them
        b.extend_from_slice(&[0; 6]);
        for a in &self.others {
            match a {
                SocketAddr::V4(v) => {
                    b.extend_from_slice(&v.ip().octets());
                    b.extend_from_slice(&v.port().to_le_bytes());
                }
                _ => b.extend_from_slice(&[0; 6]),
            }
        }
        // the match: the mode, the time limit, the score limit, friendly fire, the bots, the characters picked by the players (the list's numbers)
        b.push(self.setup.mode as u8);
        b.push(self.setup.time as u8);
        b.push(self.setup.limit as u8);
        b.push(self.setup.friendly as u8);
        b.push(self.bots as u8);
        for p in 0..1 + self.others.len() {
            b.push(self.picks[p] as u8);
        }
        // the weapon set and the five weapons of the custom one
        let (set, pack) = self.setup.weapons.unwrap_or((0, [-1; 5]));
        b.push(set as u8);
        for w in pack {
            b.extend_from_slice(&(w as i16).to_le_bytes());
        }
        // the players' names
        for n in self.host_names() {
            b.push(n.len() as u8);
            b.extend_from_slice(n.as_bytes());
        }
        b
    }

    /// The hosts announce themselves on the local network once a second; a machine looking for a game listens and keeps the list.
    fn discover(&mut self) {
        match self.screen {
            Screen::Host if self.starting.is_none() && self.sock.is_some() => {
                if self.beacon.is_none() {
                    self.beacon = UdpSocket::bind("0.0.0.0:0").ok().inspect(|s| {
                        let _ = s.set_broadcast(true);
                    });
                }
                let players = 1 + self.others.len();
                if self.last_beacon.elapsed() > Duration::from_secs(1) && players < MAX_PLAYERS {
                    self.last_beacon = Instant::now();
                    let level = self.levels.get(self.level).cloned().unwrap_or_default();
                    let mut b = MAGIC.to_le_bytes().to_vec();
                    b.push(T_BEACON);
                    b.extend_from_slice(&self.compat.to_le_bytes());
                    b.push(players as u8);
                    b.push(level.len().min(60) as u8);
                    b.extend_from_slice(&level.as_bytes()[..level.len().min(60)]);
                    if let Some(s) = &self.beacon {
                        let _ = s.send_to(&b, ("255.255.255.255", BEACON_PORT));
                    }
                }
            }
            Screen::Join if self.sock.is_none() => {
                if self.beacon.is_none() {
                    self.beacon = UdpSocket::bind(("0.0.0.0", BEACON_PORT)).ok().inspect(|s| {
                        let _ = s.set_nonblocking(true);
                    });
                }
                let mut buf = [0u8; 256];
                while let Some((n, from)) = self.beacon.as_ref().and_then(|s| s.recv_from(&mut buf).ok()) {
                    let b = &buf[..n];
                    if b.len() < 11 || u32::from_le_bytes([b[0], b[1], b[2], b[3]]) != MAGIC || b[4] != T_BEACON {
                        continue;
                    }
                    let compat = u32::from_le_bytes([b[5], b[6], b[7], b[8]]);
                    let len = b[10] as usize;
                    let Some(name) = b.get(11..11 + len) else { continue };
                    let entry = (SocketAddr::new(from.ip(), PORT), Instant::now(), compat, b[9], String::from_utf8_lossy(name).to_string());
                    match self.found.iter_mut().find(|f| f.0 == entry.0) {
                        Some(f) => *f = entry,
                        None => self.found.push(entry),
                    }
                }
                self.found.retain(|f| f.1.elapsed() < Duration::from_secs(4));
                self.found_sel = self.found_sel.min(self.found.len().saturating_sub(1));
            }
            _ => {}
        }
    }

    /// Once a frame or so: talk to the others and look at what came.
    pub fn tick(&mut self) {
        if self.done.is_some() {
            return;
        }
        self.discover();
        let mut buf = [0u8; 512];
        let mut got: Vec<(Vec<u8>, SocketAddr)> = Vec::new();
        if let Some(s) = &self.sock {
            while let Ok((n, from)) = s.recv_from(&mut buf) {
                got.push((buf[..n].to_vec(), from));
            }
        }
        match self.screen {
            Screen::Host => {
                for (b, from) in got {
                    if b.len() < 5 || u32::from_le_bytes([b[0], b[1], b[2], b[3]]) != MAGIC {
                        continue;
                    }
                    match b[4] {
                        T_JOIN if b.len() >= 9 && u32::from_le_bytes([b[5], b[6], b[7], b[8]]) != self.compat => {
                            // (a machine with other game files or other mods would not stay in step)
                            let mut r = MAGIC.to_le_bytes().to_vec();
                            r.push(T_REJECT);
                            if let Some(s) = &self.sock {
                                let _ = s.send_to(&r, from);
                            }
                        }
                        T_JOIN => {
                            if let Some(&len) = b.get(9) {
                                if let Some(n) = b.get(10..10 + len as usize) {
                                    self.names.insert(from, clean_name(&String::from_utf8_lossy(n)));
                                }
                            }
                            if !self.others.contains(&from) && self.others.len() + 1 < MAX_PLAYERS && self.starting.is_none() {
                                self.others.push(from);
                            }
                        }
                        T_LEAVE => self.others.retain(|a| *a != from),
                        T_CHAR if b.len() >= 6 => {
                            if let Some(i) = self.others.iter().position(|a| *a == from) {
                                self.picks[i + 1] = b[5] as usize % self.chars.len().max(1);
                            }
                        }
                        _ => {}
                    }
                }
                if self.last_send.elapsed() > Duration::from_millis(200) {
                    self.last_send = Instant::now();
                    if let Some(s) = &self.sock {
                        for (i, a) in self.others.iter().enumerate() {
                            let _ = s.send_to(&self.roster_packet(i + 1, self.starting.is_some()), a);
                        }
                    }
                }
                self.status = match self.starting {
                    Some(_) => "Starting ...".into(),
                    None => format!("{} player{} here; Enter starts the game", 1 + self.others.len(), if self.others.is_empty() { "" } else { "s" }),
                };
                // the test aid: start when there are enough
                if let Some(Auto::Host(n)) = &self.auto {
                    if self.starting.is_none() && 1 + self.others.len() >= *n {
                        self.key(Key::Enter);
                    }
                }
                if let Some(t) = self.starting {
                    if t.elapsed() > Duration::from_millis(700) {
                        let mut peers = vec!["127.0.0.1:0".parse().unwrap()];
                        peers.extend(self.others.iter().copied());
                        self.done = Some(Outcome {
                            level: self.levels.get(self.level).cloned().unwrap_or_default(),
                            players: peers.len(),
                            local: 0,
                            bind: SocketAddr::from(([0, 0, 0, 0], PORT)),
                            setup: self.match_setup(1 + self.others.len()),
                            peers,
                            seed: self.seed,
                        });
                    }
                }
            }
            Screen::Join if self.sock.is_some() => {
                let host = self.host.unwrap();
                for (b, from) in &got {
                    if b.len() >= 5 && u32::from_le_bytes([b[0], b[1], b[2], b[3]]) == MAGIC && b[4] == T_REJECT && from.ip() == host.ip() {
                        self.sock = None;
                        self.status = "The host has other game files or other mods than this machine (the mods that are on must be the same)".into();
                        return;
                    }
                }
                for (b, from) in got {
                    if b.len() < 14 || u32::from_le_bytes([b[0], b[1], b[2], b[3]]) != MAGIC || b[4] != T_ROSTER || from.ip() != host.ip() {
                        continue;
                    }
                    let starting = b[5] != 0;
                    self.my_index = b[6] as usize;
                    self.n_players = b[7] as usize;
                    self.seed = i32::from_le_bytes([b[8], b[9], b[10], b[11]]);
                    let len = b[12] as usize;
                    let Some(name) = b.get(13..13 + len) else { continue };
                    let level = String::from_utf8_lossy(name).to_string();
                    if let Some(i) = self.levels.iter().position(|l| *l == level) {
                        self.level = i;
                    }
                    let mut at = 13 + len;
                    let mut roster = Vec::new();
                    for i in 0..self.n_players {
                        let Some(e) = b.get(at..at + 6) else { break };
                        at += 6;
                        let ip = std::net::Ipv4Addr::new(e[0], e[1], e[2], e[3]);
                        let port = u16::from_le_bytes([e[4], e[5]]);
                        roster.push(if i == 0 { host } else { SocketAddr::from((ip, port)) });
                    }
                    if roster.len() == self.n_players {
                        self.roster = roster;
                    }
                    // the players' names
                    let mut names = Vec::new();
                    let mut o = at + 5 + self.n_players + 11;
                    for _ in 0..self.n_players {
                        let Some(&len) = b.get(o) else { break };
                        let Some(n) = b.get(o + 1..o + 1 + len as usize) else { break };
                        names.push(clean_name(&String::from_utf8_lossy(n)));
                        o += 1 + len as usize;
                    }
                    if names.len() == self.n_players {
                        self.roster_names = names;
                    }
                    // the match the host set
                    if let Some(w) = b.get(at + 5 + self.n_players..at + 5 + self.n_players + 11) {
                        let mut pack = [0i32; 5];
                        for (k, p) in pack.iter_mut().enumerate() {
                            *p = i16::from_le_bytes([w[1 + 2 * k], w[2 + 2 * k]]) as i32;
                        }
                        self.setup.weapons = Some((w[0] as i32, pack));
                    }
                    if let Some(m) = b.get(at..at + 5 + self.n_players) {
                        self.setup.mode = m[0] as i32;
                        self.setup.time = m[1] as i32;
                        self.setup.limit = m[2] as i32;
                        self.setup.friendly = m[3] != 0;
                        self.bots = m[4] as usize;
                        for p in 0..self.n_players.min(MAX_PLAYERS) {
                            if p != self.my_index {
                                self.picks[p] = m[5 + p] as usize;
                            }
                        }
                    }
                    if starting && self.roster.len() == self.n_players {
                        self.client_start = true;
                    }
                }
                // (the test aid: a character picked at the start)
                if let (Some(Auto::Join | Auto::Find), Ok(v), true) = (&self.auto, std::env::var("TS_LOBBY_CHAR"), !self.roster.is_empty()) {
                    if let Ok(i) = v.parse::<usize>() {
                        if self.picks[self.my_index.min(MAX_PLAYERS - 1)] != i {
                            self.picks[self.my_index.min(MAX_PLAYERS - 1)] = 0;
                            self.change_char(i as i32);
                        }
                    }
                }
                if self.client_start {
                    let local = self.sock.as_ref().and_then(|s| s.local_addr().ok()).map_or(0, |a| a.port());
                    let mut peers = self.roster.clone();
                    let bind = SocketAddr::from(([0, 0, 0, 0], local));
                    if self.my_index < peers.len() {
                        peers[self.my_index] = bind;
                    }
                    self.done = Some(Outcome {
                        level: self.levels.get(self.level).cloned().unwrap_or_default(),
                        players: self.n_players,
                        local: self.my_index,
                        bind,
                        peers,
                        seed: self.seed,
                        setup: self.match_setup(self.n_players),
                    });
                    self.sock = None;
                    return;
                }
                if self.last_send.elapsed() > Duration::from_millis(400) {
                    self.last_send = Instant::now();
                    let mut b = MAGIC.to_le_bytes().to_vec();
                    b.push(T_JOIN);
                    b.extend_from_slice(&self.compat.to_le_bytes());
                    b.push(self.my_name.len() as u8);
                    b.extend_from_slice(self.my_name.as_bytes());
                    if let Some(s) = &self.sock {
                        let _ = s.send_to(&b, host);
                    }
                }
                self.status = if self.roster.is_empty() {
                    format!("Asking {host} ...")
                } else {
                    format!("Joined as player {} of {}; the host starts the game", self.my_index + 1, self.n_players)
                };
            }
            _ => {}
        }
        // the test aid: a client that has an address goes straight to it
        if matches!(self.auto, Some(Auto::Find)) {
            if self.screen == Screen::Main {
                self.sel = 1;
                self.key(Key::Enter);
            } else if self.screen == Screen::Join && self.sock.is_none() && !self.found.is_empty() {
                self.key(Key::Enter);
            }
        }
        if matches!(self.auto, Some(Auto::Join)) && self.screen == Screen::Main {
            self.begin_join();
        }
    }

    /// The text of the screen.
    pub fn text(&self) -> String {
        let mut s = String::from(" TIMESPLITTERS  -  NETWORK GAME\n\n");
        match self.screen {
            Screen::Main => {
                for (i, label) in ["Host a game", "Join a game", "Back"].iter().enumerate() {
                    s += &format!(" {} {}\n", if i == self.sel { ">" } else { " " }, label);
                }
                s += "\n Up/Down and Enter (or the D-pad and X)\n";
            }
            Screen::Host => {
                s += &format!(" Hosting.  Players should join {}:{PORT}\n\n", local_ip_hint());
                let cur = |r: usize| if r == self.row { ">" } else { " " };
                let level = self.levels.get(self.level).map_or("?", |l| l.as_str());
                let mode = MODES.iter().find(|m| m.0 == self.setup.mode).map_or("?", |m| m.1);
                let time = if self.setup.time == 0 { "none".to_string() } else { format!("{} min", self.setup.time) };
                let chr = self.chars.get(self.picks[0]).map_or("?", |c| c.1.as_str());
                s += &format!(" {} Level            < {} >\n", cur(0), level);
                s += &format!(" {} Mode             < {} >\n", cur(1), mode);
                s += &format!(" {} Time limit       < {} >\n", cur(2), time);
                s += &format!(" {} Score limit      < {} >\n", cur(3), self.setup.limit);
                s += &format!(" {} Bots             < {} >\n", cur(4), self.bots);
                s += &format!(" {} Friendly fire    < {} >\n", cur(5), if self.setup.friendly { "yes" } else { "no" });
                s += &format!(" {} Weapons          < {} >\n", cur(6), self.wset_name(self.wset));
                s += &format!(" {} Your character   < {} >\n\n", cur(7), chr);
                s += &format!("   1  {} (you, the host)  {}\n", self.my_name, chr);
                for (i, a) in self.others.iter().enumerate() {
                    let name = self.names.get(a).map_or("", |n| n.as_str());
                    s += &format!("   {}  {}  {}  {}\n", i + 2, name, a, self.chars.get(self.picks[i + 1]).map_or("?", |c| c.1.as_str()));
                }
                s += &format!("\n {}\n\n Up/Down, Left/Right change a row.  Enter: start the game      Esc: cancel\n", self.status);
            }
            Screen::Join => {
                if self.sock.is_none() {
                    s += " Games found on this network:\n";
                    if self.found.is_empty() {
                        s += "   (none yet)\n";
                    }
                    for (i, f) in self.found.iter().enumerate() {
                        let same = if f.2 == self.compat { "" } else { "   (other game files or mods)" };
                        s += &format!(" {} {}   {} player{}   {}{}\n", if i == self.found_sel { ">" } else { " " }, f.0.ip(), f.3, if f.3 == 1 { "" } else { "s" }, f.4, same);
                    }
                    s += &format!("\n Host address:  {}_\n\n {}\n\n Up/Down: pick a game   Enter: connect      Esc: back\n", self.addr_text, self.status);
                } else {
                    let mode = MODES.iter().find(|m| m.0 == self.setup.mode).map_or("?", |m| m.1);
                    let time = if self.setup.time == 0 { "none".to_string() } else { format!("{} min", self.setup.time) };
                    let chr = self.chars.get(self.picks[self.my_index.min(MAX_PLAYERS - 1)]).map_or("?", |c| c.1.as_str());
                    let weapons = match self.setup.weapons {
                        Some((0, _)) => "Custom".to_string(),
                        Some((v, _)) => format!("Preset {v}"),
                        None => "?".to_string(),
                    };
                    s += &format!(" Level: {}   Mode: {}   Time: {}   Score limit: {}   Bots: {}   Weapons: {}\n\n", self.levels.get(self.level).map_or("?", |l| l.as_str()), mode, time, self.setup.limit, self.bots, weapons);
                    s += &format!(" Your character  < {} >   (Left / Right)\n\n {}\n\n Esc: leave\n", chr, self.status);
                }
            }
        }
        s
    }
}

/// An address of this machine on its network, to tell the others (the one a UDP socket would leave by).
fn local_ip_hint() -> String {
    UdpSocket::bind("0.0.0.0:0")
        .and_then(|s| {
            s.connect("8.8.8.8:80")?;
            s.local_addr()
        })
        .map(|a| a.ip().to_string())
        .unwrap_or_else(|_| "<this machine's address>".into())
}
