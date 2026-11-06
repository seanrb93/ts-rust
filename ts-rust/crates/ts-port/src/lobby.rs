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
/// a piece of the Map Maker map the host sends to a machine that does not have it, and that machine's word that it has it
const T_MAP: u8 = 7;
const T_MAPOK: u8 = 8;
/// the host has closed its game
const T_CLOSED: u8 = 9;
/// a player the host has not heard from for this long is gone
const GONE_AFTER: Duration = Duration::from_secs(8);
/// a map as the game keeps it: 8 pieces of 1024 bytes
const MAP_PIECES: usize = 8;
const PIECE: usize = 1024;
/// the port the announcements go to (a joining machine listens on it; it is not the host's own port, so a host and a joiner can share a PC)
const BEACON_PORT: u16 = 7778;
/// the port a host listens on
pub const PORT: u16 = 7777;
const MAX_PLAYERS: usize = 10;
/// the rows of the host's screen: the level, the mode, the time limit, the score limit, the bots, friendly fire and the character
const HOST_ROWS: usize = 9;
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

/// What the lobby is showing, for the game's own menu pages.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum View {
    /// Host a game / Join a game
    Main,
    /// the host's settings and the players
    Host,
    /// looking for a game to join (the list and the address)
    Find,
    /// joined: waiting for the host to start
    Joined,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UiKind {
    /// something that is done when it is chosen
    Action,
    /// a line of text
    Info,
}

/// A row of a menu page of the lobby.
#[derive(Clone, Debug)]
pub struct UiRow {
    pub label: String,
    pub kind: UiKind,
    pub values: Vec<String>,
    pub index: usize,
}

fn info(label: impl Into<String>) -> UiRow {
    UiRow { label: label.into(), kind: UiKind::Info, values: vec![String::new()], index: 0 }
}

fn action(label: impl Into<String>) -> UiRow {
    UiRow { label: label.into(), kind: UiKind::Action, values: vec![String::new()], index: 0 }
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
    /// the control settings of the other machines' players (host) and of every player (client)
    prefs: std::collections::HashMap<SocketAddr, Vec<u8>>,
    roster_prefs: Vec<Vec<u8>>,
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
    /// the parts of `compat`: the program, the game's packs, the mods
    compat_parts: [u32; 3],
    /// the character each player chose (this machine's own at `my_index`; the host keeps its clients' here)
    picks: Vec<usize>,
    /// host: the socket the announcements go out of; client: the one they come in on
    beacon: Option<UdpSocket>,
    last_beacon: Instant,
    /// the hosts heard on the network: (address, when last heard, their compat hash, players there, level, the host's name, the mode)
    found: Vec<(SocketAddr, Instant, u32, u8, String, String, String)>,
    found_sel: usize,
    /// (a test aid) the lobby answers itself: `host,<players>,<level>` or `join,<address>`
    auto: Option<Auto>,
    /// the host's setup in the game's own terms (the pages of a local arcade game): the levels that are open (number, name, place in the arcade list), the modes of the
    /// level, the rows of the mode's options, the weapon sets and the bot sets
    alevels: Vec<(i32, String, i32)>,
    amodes: Vec<(i32, String)>,
    gvrows: Vec<ts_game::NetGvRow>,
    wrow: Option<ts_game::NetGvRow>,
    brow: Option<ts_game::NetGvRow>,
    /// the game variables the host set (a client has them from the host), and the parameters of the match once the host has made them
    gvs: Vec<(i32, i32)>,
    block: Vec<i32>,
    /// the mods that are on here (their titles), and the host's (a client has them from the roster or from the host's refusal)
    mods: Vec<String>,
    host_mods: Vec<String>,
    /// the name of the level shown (the host's choice; a client has it from the roster)
    level_name: String,
    /// a Map Maker map: the host's own (read from the game's buffer for maps) or the one a client has put together, its checksum, and what a client is still to get
    map: Vec<u8>,
    map_crc: u32,
    map_wanted: Option<u32>,
    map_pieces: Vec<Option<Vec<u8>>>,
    /// host: the machines that have said they have the map
    map_have: std::collections::HashSet<SocketAddr>,
    /// host: a line for the page (the map is missing, the map is being sent)
    notice: String,
    /// shown by the game's pages (see `use_pages`)
    pages: bool,
    /// host: the players who have chosen their characters, and when each was last heard
    ready: std::collections::HashSet<SocketAddr>,
    heard: std::collections::HashMap<SocketAddr, Instant>,
    /// a player: on the character select page; the host has closed its game
    in_chr: bool,
    host_gone: bool,
}

/// A checksum of a map's bytes (the same on every machine).
fn map_sum(b: &[u8]) -> u32 {
    let mut h: u32 = 0x811c_9dc5;
    for x in b {
        h = (h ^ *x as u32).wrapping_mul(0x0100_0193);
    }
    h
}

/// A list of mods as a line of text.
fn mods_text(mods: &[String]) -> String {
    if mods.is_empty() {
        return "none".into();
    }
    let s = mods.join(", ");
    if s.chars().count() > 42 {
        format!("{}...", s.chars().take(39).collect::<String>())
    } else {
        s
    }
}

/// The names of mods as they go in a message: a count, then each with its length (at most eight, twenty-four letters each).
fn put_mods(b: &mut Vec<u8>, mods: &[String]) {
    let n = mods.len().min(8);
    b.push(n as u8);
    for m in mods.iter().take(n) {
        let m: String = m.chars().filter(|c| c.is_ascii() && !c.is_ascii_control()).take(24).collect();
        b.push(m.len() as u8);
        b.extend_from_slice(m.as_bytes());
    }
}

/// The names of mods from a message at `at`.
fn get_mods(b: &[u8], mut at: usize) -> (Vec<String>, usize) {
    let mut v = Vec::new();
    let Some(&n) = b.get(at) else { return (v, at) };
    at += 1;
    for _ in 0..n {
        let Some(&len) = b.get(at) else { break };
        let Some(m) = b.get(at + 1..at + 1 + len as usize) else { break };
        v.push(String::from_utf8_lossy(m).to_string());
        at += 1 + len as usize;
    }
    (v, at)
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

/// The name this machine's player has: the name of the profile signed in to the game, or else the user's name on the computer.
fn default_name() -> String {
    if let Some(n) = ts_game::netsetup_profile_name() {
        return clean_name(&n);
    }
    clean_name(&std::env::var("USERNAME").or_else(|_| std::env::var("USER")).unwrap_or_default())
}

fn now_seed() -> i32 {
    // (a test aid: the seed of the match)
    if let Some(v) = std::env::var("TS_LOBBY_SEED").ok().and_then(|v| v.parse::<i32>().ok()) {
        return v;
    }
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
            prefs: std::collections::HashMap::new(),
            roster_prefs: Vec::new(),
            pages: false,
            ready: std::collections::HashSet::new(),
            heard: std::collections::HashMap::new(),
            in_chr: false,
            host_gone: false,
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
            compat_parts: [0; 3],
            compat: compat ^ std::env::var("TS_LOBBY_COMPAT_XOR").ok().and_then(|v| v.parse::<u32>().ok()).unwrap_or(0),
            picks: vec![0; MAX_PLAYERS],
            alevels: Vec::new(),
            amodes: Vec::new(),
            gvrows: Vec::new(),
            wrow: None,
            brow: None,
            gvs: Vec::new(),
            block: Vec::new(),
            mods: Vec::new(),
            host_mods: Vec::new(),
            level_name: String::new(),
            map: Vec::new(),
            map_crc: 0,
            map_wanted: None,
            map_pieces: Vec::new(),
            map_have: std::collections::HashSet::new(),
            notice: String::new(),
            beacon: None,
            last_beacon: Instant::now() - Duration::from_secs(5),
            found: Vec::new(),
            found_sel: 0,
            auto: None,
        };
        l.apply_wset();
        if std::env::var_os("TS_LEVELS").is_some() {
            eprintln!("[lobby] levels: {:?}", l.levels);
        }
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
                            Some(("weapons", v)) => {
                                // (the weapon set by its number: 1 to 9 a preset)
                                let n = v.parse::<i32>().unwrap_or(0);
                                if let Some(i) = l.wsets.iter().position(|w| w.0 == n) {
                                    l.wset = i;
                                    l.apply_wset();
                                }
                            }
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

    pub fn begin_host(&mut self) {
        match UdpSocket::bind(("0.0.0.0", PORT)) {
            Ok(s) => {
                let _ = s.set_nonblocking(true);
                self.sock = Some(s);
                self.screen = Screen::Host;
                self.others.clear();
                self.starting = None;
                self.status = format!("Waiting for players on port {PORT}");
                self.refresh_setup(true);
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
        // (a host tells the players that its game has closed)
        if let (Some(s), Screen::Host) = (&self.sock, self.screen) {
            let mut b = MAGIC.to_le_bytes().to_vec();
            b.push(T_CLOSED);
            for a in &self.others {
                let _ = s.send_to(&b, a);
            }
        }
        self.ready.clear();
        self.heard.clear();
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
                    self.make_block();
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

    /// The control settings of the players as the host has them (this machine's first).
    fn host_prefs(&self) -> Vec<Vec<u8>> {
        let mut v = vec![ts_game::netlink_my_prefs()];
        for a in &self.others {
            v.push(self.prefs.get(a).cloned().unwrap_or_default());
        }
        v
    }

    /// The match as it stands: the options, the bots and the character of each player.
    fn match_setup(&self, players: usize) -> ts_sdk::net::Setup {
        let mut s = self.setup.clone();
        let chr = |i: usize| self.chars.get(i % self.chars.len().max(1)).map_or(0, |c| c.0);
        s.chars = (0..players).map(|p| chr(self.picks.get(p).copied().unwrap_or(0))).collect();
        s.names = if self.screen == Screen::Host { self.host_names() } else { self.roster_names.clone() };
        s.prefs = if self.screen == Screen::Host { self.host_prefs() } else { self.roster_prefs.clone() };
        // (the bots are characters the players are not: the next ones of the list)
        // (the statistics of a match have 14 places, for the players and the bots together)
        s.bots = (0..self.bots.min(14usize.saturating_sub(players))).map(|b| chr(8 + 3 * b)).collect();
        s.block = self.block.clone();
        s.gvs = self.gvs.clone();
        s.map = if self.map.len() == MAP_PIECES * PIECE && (self.map_wanted.is_some() || self.is_map_level()) { self.map.clone() } else { Vec::new() };
        s
    }

    fn roster_packet(&self, to_index: usize, starting: bool) -> Vec<u8> {
        let mut b = MAGIC.to_le_bytes().to_vec();
        b.push(T_ROSTER);
        b.push(starting as u8);
        b.push(to_index as u8);
        b.push((1 + self.others.len()) as u8);
        b.extend_from_slice(&self.seed.to_le_bytes());
        let name = if self.level_name.is_empty() { self.levels.get(self.level).cloned().unwrap_or_default() } else { self.level_name.clone() };
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
        // the players' control settings
        for p in self.host_prefs() {
            b.push(p.len() as u8);
            b.extend_from_slice(&p);
        }
        // the game variables the host set, and the parameters of the match once they are made
        let n = self.gvs.len().min(16);
        b.push(n as u8);
        for &(gv, value) in self.gvs.iter().take(n) {
            b.extend_from_slice(&gv.to_le_bytes());
            b.extend_from_slice(&value.to_le_bytes());
        }
        b.push((self.block.len() == 28) as u8);
        if self.block.len() == 28 {
            for w in &self.block {
                b.extend_from_slice(&w.to_le_bytes());
            }
        }
        // the host's mods
        put_mods(&mut b, &self.mods);
        // the map, when the level is the Map Maker's
        if self.is_map_level() && !self.map.is_empty() {
            b.push(1);
            b.extend_from_slice(&self.map_crc.to_le_bytes());
        } else {
            b.push(0);
        }
        b
    }

    /// Is the level that is chosen the Map Maker's?
    fn is_map_level(&self) -> bool {
        self.screen == Screen::Host && !self.alevels.is_empty() && ts_game::netsetup_level_id() == ts_game::MAPMAKER
    }

    /// A piece of the map for a machine that does not have it (the pieces are sent over and over until it says it has them all).
    fn send_map(&self, to: SocketAddr) {
        let Some(s) = &self.sock else { return };
        for (i, piece) in self.map.chunks(PIECE).enumerate() {
            let mut b = MAGIC.to_le_bytes().to_vec();
            b.push(T_MAP);
            b.extend_from_slice(&self.map_crc.to_le_bytes());
            b.push(i as u8);
            b.push(MAP_PIECES as u8);
            b.extend_from_slice(piece);
            let _ = s.send_to(&b, to);
        }
    }

    /// The map is read from the game again (after the player chose one), and the machines that had the old one are asked to get the new.
    fn read_map(&mut self) {
        if !self.is_map_level() {
            self.map.clear();
            return;
        }
        // (the arcade page itself takes the flag that a map was loaded, so it is left alone here on the game's pages)
        if !self.pages && ts_game::netsetup_map_loaded_flag() {
            self.notice.clear();
        }
        if ts_game::netsetup_map_ready() {
            let bytes = ts_game::netsetup_map_bytes();
            let crc = map_sum(&bytes);
            if crc != self.map_crc || self.map.is_empty() {
                self.map = bytes;
                self.map_crc = crc;
                self.map_have.clear();
            }
        } else {
            self.map.clear();
        }
    }

    /// The titles of the mods that are on here are given to the lobby (it tells the others, and shows the host's).
    pub fn set_compat_parts(&mut self, parts: [u32; 3]) {
        self.compat_parts = parts;
    }

    pub fn set_mods(&mut self, mods: Vec<String>) {
        self.mods = mods;
    }

    /// The host's choices as the game's pages have them are read again (after anything changed), and what the roster's older fields tell a client is kept in step.
    fn refresh_setup(&mut self, begin: bool) {
        if begin {
            self.alevels = ts_game::netsetup_levels();
            ts_game::netsetup_begin();
        }
        if self.alevels.is_empty() {
            return;
        }
        self.amodes = ts_game::netsetup_modes(2);
        if std::env::var_os("TS_NETDUMP2").is_some() {
            eprintln!("[lobby] level {} modes {:?} map ready {}", ts_game::netsetup_level_id(), self.amodes, ts_game::netsetup_map_ready());
            ts_game::netsetup_map_dump();
        }
        if !self.amodes.iter().any(|m| m.0 == ts_game::netsetup_mode()) {
            if let Some(m) = self.amodes.first() {
                ts_game::netsetup_select_mode(m.0);
            }
        }
        self.gvrows = ts_game::netsetup_mode_rows();
        self.wrow = Some(ts_game::netsetup_weapon_row());
        self.brow = Some(ts_game::netsetup_botset_row());
        self.gvs = ts_game::netsetup_gv_values();
        self.bots = ts_game::netsetup_bots() as usize;
        self.setup.mode = ts_game::netsetup_mode();
        // the weapon set: the custom ones are the host's own
        // (a custom set goes as its five weapons, as the host has them now; a preset as its number)
        let w = self.wrow.as_ref().and_then(|r| r.choices.get(r.index)).map_or(1, |c| c.value);
        self.setup.weapons = Some(if w == 0 || w >= 10 { (0, ts_game::netsetup_weapon_pack()) } else { (w, [0; 5]) });
        if let Some(i) = self.alevels.iter().position(|l| l.0 == ts_game::netsetup_level_id()) {
            self.level = self.levels.iter().position(|n| *n == self.alevels[i].1).unwrap_or(self.level);
            self.level_name = self.alevels[i].1.clone();
        }
        self.read_map();
    }

    /// The lines that list the players: one each with their characters, or two to a line with ten players' worth of names (a page has room for 24 rows).
    fn player_lines(&self, names: &[String]) -> Vec<String> {
        let chr = |i: usize| self.chars.get(self.picks[i.min(MAX_PLAYERS - 1)]).map_or("?", |c| c.1.as_str());
        if names.len() <= 6 {
            return names.iter().enumerate().map(|(i, n)| format!("Player {}: {}   {}", i + 1, n, chr(i))).collect();
        }
        names.chunks(2).enumerate().map(|(k, pair)| pair.iter().enumerate().map(|(j, n)| format!("{}: {} ({})", 2 * k + j + 1, n, chr(2 * k + j))).collect::<Vec<_>>().join("    ")).collect()
    }

    /// Why the game cannot start yet: a Map Maker level needs its map here and on every machine.
    fn cannot_start(&self) -> Option<String> {
        if !self.is_map_level() {
            return None;
        }
        if self.map.is_empty() {
            return Some("Choose a map first".into());
        }
        let waiting = self.others.iter().filter(|a| !self.map_have.contains(a)).count();
        (waiting > 0).then(|| format!("Sending the map to {waiting} player{}", if waiting == 1 { "" } else { "s" }))
    }

    /// The parameters of the match are made (by the game's own start function) for the players that are here.
    fn make_block(&mut self) {
        if self.alevels.is_empty() {
            return;
        }
        self.block = ts_game::netsetup_block(1 + self.others.len() as i32).unwrap_or_default();
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
                    let level = if self.level_name.is_empty() { self.levels.get(self.level).cloned().unwrap_or_default() } else { self.level_name.clone() };
                    let mut b = MAGIC.to_le_bytes().to_vec();
                    b.push(T_BEACON);
                    b.extend_from_slice(&self.compat.to_le_bytes());
                    b.push(players as u8);
                    b.push(level.len().min(60) as u8);
                    b.extend_from_slice(&level.as_bytes()[..level.len().min(60)]);
                    let mode = if self.pages { ts_game::netsetup_mode_name(self.setup.mode) } else { self.mode_name().to_string() };
                    for text in [&self.my_name, &mode] {
                        b.push(text.len().min(40) as u8);
                        b.extend_from_slice(&text.as_bytes()[..text.len().min(40)]);
                    }
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
                    // (the host's name and the mode follow the level)
                    let mut at = 11 + len;
                    let mut texts = Vec::new();
                    for _ in 0..2 {
                        let size = b.get(at).copied().unwrap_or(0) as usize;
                        texts.push(b.get(at + 1..at + 1 + size).map_or(String::new(), |t| String::from_utf8_lossy(t).to_string()));
                        at += 1 + size;
                    }
                    let entry = (SocketAddr::new(from.ip(), PORT), Instant::now(), compat, b[9], String::from_utf8_lossy(name).to_string(), texts[0].clone(), texts[1].clone());
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
        // (the character is the one chosen on the game's own page)
        if self.pages {
            let at = ts_game::netflow_my_char();
            if let Some(i) = self.chars.iter().position(|c| c.0 == at) {
                self.picks[if self.screen == Screen::Host { 0 } else { self.my_index.min(MAX_PLAYERS - 1) }] = i;
            }
        }
        // (the weapons or the bots of a custom set may have been chosen on the game's own pages: the host's rows are read again now and then)
        if self.screen == Screen::Host && !self.alevels.is_empty() && self.starting.is_none() {
            static N: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
            if N.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % 20 == 0 {
                self.refresh_setup(false);
            }
        }
        let mut buf = [0u8; 2048];
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
                            put_mods(&mut r, &self.mods);
                            for part in self.compat_parts {
                                r.extend_from_slice(&part.to_le_bytes());
                            }
                            if let Some(s) = &self.sock {
                                let _ = s.send_to(&r, from);
                            }
                        }
                        T_JOIN => {
                            if let Some(&len) = b.get(9) {
                                if let Some(n) = b.get(10..10 + len as usize) {
                                    self.names.insert(from, clean_name(&String::from_utf8_lossy(n)));
                                }
                                // (the player's control settings come after the name)
                                let at = 10 + len as usize;
                                if let Some(&size) = b.get(at) {
                                    if let Some(p) = b.get(at + 1..at + 1 + size as usize) {
                                        self.prefs.insert(from, p.to_vec());
                                    }
                                }
                            }
                            if !self.others.contains(&from) && self.others.len() + 1 < MAX_PLAYERS && self.starting.is_none() {
                                self.others.push(from);
                            }
                            self.heard.insert(from, Instant::now());
                            // (after the settings: the player's character, and whether it is chosen)
                            let tail = b.get(9).map(|&len| 10 + len as usize).and_then(|at| b.get(at).map(|&size| at + 1 + size as usize));
                            if let (Some(t), Some(i)) = (tail, self.others.iter().position(|a| *a == from)) {
                                if let (Some(&pick), Some(&ready)) = (b.get(t), b.get(t + 1)) {
                                    self.picks[i + 1] = pick as usize % self.chars.len().max(1);
                                    if ready != 0 {
                                        self.ready.insert(from);
                                    } else {
                                        self.ready.remove(&from);
                                    }
                                }
                            }
                        }
                        T_LEAVE => {
                            self.others.retain(|a| *a != from);
                            self.ready.remove(&from);
                        }
                        T_MAPOK if b.len() >= 9 && u32::from_le_bytes([b[5], b[6], b[7], b[8]]) == self.map_crc => {
                            self.map_have.insert(from);
                        }
                        T_CHAR if b.len() >= 6 => {
                            if let Some(i) = self.others.iter().position(|a| *a == from) {
                                self.picks[i + 1] = b[5] as usize % self.chars.len().max(1);
                            }
                        }
                        _ => {}
                    }
                }
                // (a player whose game has gone does not hold up the others)
                if self.pages && self.starting.is_none() {
                    let heard = &self.heard;
                    self.others.retain(|a| heard.get(a).map_or(false, |t| t.elapsed() < GONE_AFTER));
                    let others = &self.others;
                    self.ready.retain(|a| others.contains(a));
                }
                if self.last_send.elapsed() > Duration::from_millis(200) {
                    self.last_send = Instant::now();
                    if let Some(s) = &self.sock {
                        for (i, a) in self.others.iter().enumerate() {
                            let _ = s.send_to(&self.roster_packet(i + 1, self.starting.is_some()), a);
                        }
                    }
                    if self.is_map_level() && !self.map.is_empty() {
                        for a in self.others.clone() {
                            if !self.map_have.contains(&a) {
                                self.send_map(a);
                            }
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
                        let (theirs, at) = get_mods(b, 5);
                        // (what differs, when the host says)
                        let part = |i: usize| b.get(at + 4 * i..at + 4 * i + 4).map(|p| u32::from_le_bytes([p[0], p[1], p[2], p[3]]));
                        let mine = self.compat_parts;
                        let mut what = Vec::new();
                        if part(0) != Some(mine[0]) {
                            what.push("another version of the program");
                        }
                        if part(1).is_some() && part(1) != Some(mine[1]) {
                            what.push("other game files");
                        }
                        if part(2).is_some() && part(2) != Some(mine[2]) {
                            what.push("other mods, or other versions of them");
                        }
                        self.status = format!("The host has {}. Host mods: {}. Yours: {}", what.join(" and "), mods_text(&theirs), mods_text(&self.mods));
                        return;
                    }
                }
                if got.iter().any(|(b, from)| b.len() >= 5 && u32::from_le_bytes([b[0], b[1], b[2], b[3]]) == MAGIC && b[4] == T_CLOSED && from.ip() == host.ip()) {
                    self.sock = None;
                    self.host = None;
                    self.roster.clear();
                    self.host_gone = true;
                    self.addr_text.clear();
                    self.status = "The host closed the game".into();
                    return;
                }
                for (b, from) in &got {
                    if b.len() > 10 && u32::from_le_bytes([b[0], b[1], b[2], b[3]]) == MAGIC && b[4] == T_MAP && from.ip() == host.ip() {
                        let crc = u32::from_le_bytes([b[5], b[6], b[7], b[8]]);
                        let (i, total) = (b[9] as usize, b[10] as usize);
                        if self.map_wanted == Some(crc) && total == MAP_PIECES && i < MAP_PIECES && b.len() == 11 + PIECE {
                            if self.map_pieces.len() != MAP_PIECES {
                                self.map_pieces = vec![None; MAP_PIECES];
                            }
                            self.map_pieces[i] = Some(b[11..].to_vec());
                            if self.map_pieces.iter().all(|p| p.is_some()) {
                                let all: Vec<u8> = self.map_pieces.iter().flatten().flatten().copied().collect();
                                if map_sum(&all) == crc {
                                    self.map = all;
                                    self.map_crc = crc;
                                } else {
                                    self.map_pieces = vec![None; MAP_PIECES];
                                }
                            }
                        }
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
                    self.level_name = level;
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
                        // the players' control settings
                        let mut prefs = Vec::new();
                        for _ in 0..self.n_players {
                            let Some(&len) = b.get(o) else { break };
                            let Some(p) = b.get(o + 1..o + 1 + len as usize) else { break };
                            prefs.push(p.to_vec());
                            o += 1 + len as usize;
                        }
                        self.roster_prefs = prefs;
                        // the host's variables and, once it is starting, the parameters of the match
                        if let Some(&n) = b.get(o) {
                            let mut gvs = Vec::new();
                            let mut p = o + 1;
                            for _ in 0..n {
                                let Some(e) = b.get(p..p + 8) else { break };
                                gvs.push((i32::from_le_bytes([e[0], e[1], e[2], e[3]]), i32::from_le_bytes([e[4], e[5], e[6], e[7]])));
                                p += 8;
                            }
                            if gvs.len() == n as usize {
                                self.gvs = gvs;
                                let mut block = Vec::new();
                                if b.get(p) == Some(&1) {
                                    for k in 0..28 {
                                        let Some(e) = b.get(p + 1 + 4 * k..p + 5 + 4 * k) else { break };
                                        block.push(i32::from_le_bytes([e[0], e[1], e[2], e[3]]));
                                    }
                                }
                                let had = block.len() == 28;
                                self.block = if had { block } else { Vec::new() };
                                let (mods, end) = get_mods(&b, p + 1 + if had { 112 } else { 0 });
                                self.host_mods = mods;
                                // the map of the match, when it is a Map Maker map: its checksum
                                if b.get(end) == Some(&1) {
                                    if let Some(c) = b.get(end + 1..end + 5) {
                                        let crc = u32::from_le_bytes([c[0], c[1], c[2], c[3]]);
                                        if self.map_wanted != Some(crc) {
                                            self.map_wanted = Some(crc);
                                            self.map_pieces = vec![None; MAP_PIECES];
                                            if self.map_crc != crc {
                                                self.map.clear();
                                            }
                                        }
                                    }
                                } else {
                                    self.map_wanted = None;
                                }
                            }
                        }
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
                    let prefs = ts_game::netlink_my_prefs();
                    b.push(prefs.len() as u8);
                    b.extend_from_slice(&prefs);
                    b.push(self.picks[self.my_index.min(MAX_PLAYERS - 1)] as u8);
                    b.push((self.in_chr && ts_game::netflow_chr_ready()) as u8);
                    if let Some(s) = &self.sock {
                        let _ = s.send_to(&b, host);
                        // (the map is here)
                        if self.map_wanted.is_some() && self.map_wanted == Some(self.map_crc) && self.map.len() == MAP_PIECES * PIECE {
                            let mut m = MAGIC.to_le_bytes().to_vec();
                            m.push(T_MAPOK);
                            m.extend_from_slice(&self.map_crc.to_le_bytes());
                            let _ = s.send_to(&m, host);
                        }
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

    // ---- the game's own menu pages -------------------------------------------------------------------------------------------------

    pub fn view(&self) -> View {
        match self.screen {
            Screen::Main => View::Main,
            Screen::Host => View::Host,
            Screen::Join if self.sock.is_none() => View::Find,
            Screen::Join => View::Joined,
        }
    }

    /// The title of the page.
    pub fn title(&self) -> &'static str {
        match self.view() {
            View::Main => "Network Game",
            View::Host => "Host Game",
            View::Find => "Join Game",
            View::Joined => "Game Details",
        }
    }

    /// The line at the bottom of the page.
    pub fn hint(&self, _editing: bool) -> String {
        match self.view() {
            View::Find => "^2 Join".into(),
            _ => "^2 Select".into(),
        }
    }

    fn mode_name(&self) -> &'static str {
        MODES.iter().find(|m| m.0 == self.setup.mode).map_or("?", |m| m.1)
    }

    /// The rows of the page for what the lobby is showing.
    pub fn rows(&self) -> Vec<UiRow> {
        match self.view() {
            View::Main => {
                let mut v = vec![action("Join Game"), action("Host Game")];
                if !self.status.is_empty() {
                    v.push(info(self.status.clone()));
                }
                v
            }
            // (the host is on the game's own pages: this page is only passed through)
            View::Host => vec![info("")],
            View::Find => {
                // (the games first, one row each, so that the cursor stays on a game while others turn up; then the address of a host on another network)
                let mut v = Vec::new();
                if self.found.is_empty() {
                    v.push(info("Searching for games ..."));
                }
                for f in &self.found {
                    let players = format!("{} player{}", f.3, if f.3 == 1 { "" } else { "s" });
                    let other = if f.2 == self.compat { "" } else { "   (cannot join)" };
                    v.push(action(format!("{}   {}   {}   {}{}", f.5, f.4, f.6, players, other)));
                }
                v.push(action(format!("Join by address: {}_", self.addr_text)));
                if !self.status.is_empty() {
                    v.push(info(self.status.clone()));
                }
                v
            }
            View::Joined => {
                if self.roster.is_empty() {
                    return vec![info(format!("Connecting to {} ...", self.host.map_or(String::new(), |h| h.ip().to_string())))];
                }
                let mut v = vec![action("Select Character")];
                v.push(info(format!("Level: {}", self.level_name)));
                if self.map_wanted.is_some() {
                    let got = self.map_pieces.iter().filter(|p| p.is_some()).count();
                    v.push(info(if self.map.len() == MAP_PIECES * PIECE { format!("Map: {}", ts_game::netsetup_map_name(&self.map)) } else { format!("Map: receiving {got} of {MAP_PIECES}") }));
                }
                v.push(info(format!("Game Mode: {}", ts_game::netsetup_mode_name(self.setup.mode))));
                // (the players, then the options; those of teams only when the game has teams, as on the host's page)
                for line in self.player_lines(&self.roster_names) {
                    v.push(info(line));
                }
                let teams = self.gvs.iter().any(|g| g.0 == ts_game::g::gv_teamplay && g.1 != 0);
                for &(gv, val) in self.gvs.iter().filter(|g| ts_game::netsetup_mode_has_gv(self.setup.mode, g.0)) {
                    if !teams && (gv == ts_game::g::gv_useteamstarts || gv == ts_game::g::gv_friendfire) {
                        continue;
                    }
                    let (label, text) = ts_game::netsetup_gv_describe(gv, val);
                    if label != "?" {
                        v.push(info(format!("{label}: {text}")));
                    }
                }
                if !self.host_mods.is_empty() {
                    v.push(info(format!("Mods: {}", mods_text(&self.host_mods))));
                }
                v
            }
        }
    }

    /// The player chose the action row `row`.
    pub fn act(&mut self, row: usize) {
        match self.view() {
            View::Main => match row {
                0 => {
                    self.screen = Screen::Join;
                    self.sock = None;
                    self.host = None;
                    self.beacon = None;
                    self.found.clear();
                    self.found_sel = 0;
                    self.status.clear();
                }
                1 => {
                    self.begin_host();
                    if self.screen == Screen::Host {
                        ts_game::netsetup_open_page(1);
                    }
                }
                _ => {}
            },
            View::Find => {
                // (the rows: the games, or one line when there are none; the address)
                let n = self.found.len();
                if row < n {
                    if let Some(f) = self.found.get(row) {
                        self.addr_text = f.0.to_string();
                        self.begin_join();
                    }
                } else if row == n.max(1) && !self.addr_text.trim().is_empty() {
                    self.begin_join();
                }
            }
            View::Joined => {
                if row == 0 && !self.roster.is_empty() {
                    self.in_chr = true;
                    ts_game::netsetup_open_page(2);
                }
            }
            View::Host => {}
        }
    }

    /// The triangle button: a step back inside the lobby; false when the lobby is at its first page (the page of the game closes then).
    pub fn back(&mut self) -> bool {
        match self.view() {
            View::Main => false,
            View::Host | View::Joined => {
                let joined = self.view() == View::Joined;
                self.leave();
                if joined {
                    self.screen = Screen::Join;
                }
                true
            }
            View::Find => {
                self.beacon = None;
                self.screen = Screen::Main;
                self.status.clear();
                true
            }
        }
    }

    /// The lobby is shown by the game's pages (not the text screen of `--lobby`): the character is the one of the game's character select, and the host starts when all are ready.
    pub fn use_pages(&mut self) {
        self.pages = true;
    }

    /// The Network page is on the screen again: a host has stepped back out of the arcade pages and closes its game, a player is back from the character select.
    pub fn page_shown(&mut self) {
        if self.screen == Screen::Host {
            self.leave();
        }
        self.in_chr = false;
    }

    /// Join the host of the last match again.
    pub fn join_address(&mut self, host: SocketAddr) {
        self.addr_text = host.to_string();
        self.begin_join();
        self.addr_text.clear();
    }

    /// This machine's player is on the character select page.
    pub fn choosing_character(&self) -> bool {
        self.in_chr || self.screen == Screen::Host
    }

    /// The host closed its game while this player was on the character select page (once).
    pub fn take_host_gone(&mut self) -> bool {
        let gone = self.host_gone && self.in_chr;
        if gone {
            self.in_chr = false;
        }
        self.host_gone = false;
        gone
    }

    /// What the character select page says once the character is chosen, and whether this player can start the match now.
    pub fn ready_line(&self) -> (String, bool) {
        if self.screen != Screen::Host {
            return ("Waiting for the host to start".into(), false);
        }
        if self.starting.is_some() {
            return ("Starting ...".into(), false);
        }
        if self.others.is_empty() {
            return (format!("Waiting for players: {}", local_ip_hint()), false);
        }
        let waiting: Vec<&str> = self.others.iter().filter(|a| !self.ready.contains(a)).map(|a| self.names.get(a).map_or("a player", |n| n.as_str())).collect();
        if !waiting.is_empty() {
            return (format!("Waiting for {}", waiting.join(", ")), false);
        }
        if let Some(why) = self.cannot_start() {
            return (why, false);
        }
        (format!("Start Game: {} players", 1 + self.others.len()), true)
    }

    /// The host starts the match, if it can.
    pub fn start_now(&mut self) {
        if self.ready_line().1 {
            self.notice.clear();
            self.make_block();
            self.seed = now_seed();
            self.starting = Some(Instant::now());
        }
    }

    /// A character typed (the address of the host).
    pub fn type_char(&mut self, c: char) {
        if self.view() == View::Find && (c.is_ascii_digit() || c == '.' || c == ':') && self.addr_text.len() < 24 {
            self.addr_text.push(c);
        }
    }

    pub fn backspace(&mut self) {
        if self.view() == View::Find {
            self.addr_text.pop();
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
                s += &format!(" Hosting.  Players should join {}:{PORT}\n", local_ip_hint());
                // (the players first, so that they are on the screen whatever its size)
                s += &format!(" {}\n", self.status);
                let chr0 = self.chars.get(self.picks[0]).map_or("?", |c| c.1.as_str());
                s += &format!("   1  {} (you, the host)  {}\n", self.my_name, chr0);
                for (i, a) in self.others.iter().enumerate() {
                    let name = self.names.get(a).map_or("", |n| n.as_str());
                    s += &format!("   {}  {}  {}  {}\n", i + 2, name, a, self.chars.get(self.picks[i + 1]).map_or("?", |c| c.1.as_str()));
                }
                s += "\n";
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
                s += &format!(" {} Input delay      < {} ticks, {} ms >\n", cur(7), self.setup.delay, self.setup.delay * 1000 / 60);
                s += &format!(" {} Your character   < {} >\n\n", cur(8), chr);
                s += " Up/Down, Left/Right change a row.  Enter: start the game      Esc: cancel\n";
            }
            Screen::Join => {
                if self.sock.is_none() {
                    s += " Games found on this network:\n";
                    if self.found.is_empty() {
                        s += "   (none yet)\n";
                    }
                    for (i, f) in self.found.iter().enumerate() {
                        let same = if f.2 == self.compat { "" } else { "   (another version, other game files or mods)" };
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
