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
/// the port a host listens on
pub const PORT: u16 = 7777;
const MAX_PLAYERS: usize = 4;

/// What the lobby decided.
#[derive(Clone, Debug)]
pub struct Outcome {
    pub level: String,
    pub players: usize,
    pub local: usize,
    pub bind: SocketAddr,
    pub peers: Vec<SocketAddr>,
    pub seed: i32,
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
    /// (a test aid) the lobby answers itself: `host,<players>,<level>` or `join,<address>`
    auto: Option<Auto>,
}

#[derive(Clone)]
enum Auto {
    Host(usize),
    Join,
}

fn now_seed() -> i32 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(1, |d| (d.subsec_nanos() ^ d.as_secs() as u32) as i32 & 0x7fff_ffff)
}

impl Lobby {
    pub fn new(levels: Vec<String>) -> Lobby {
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
            host: None,
            my_index: 0,
            roster: Vec::new(),
            n_players: 1,
            seed: 0,
            starting: None,
            last_send: Instant::now() - Duration::from_secs(1),
            done: None,
            client_start: false,
            auto: None,
        };
        // the test aid
        if let Ok(a) = std::env::var("TS_LOBBY_AUTO") {
            let parts: Vec<&str> = a.split(',').collect();
            match parts.as_slice() {
                ["host", n, level] => {
                    if let Some(i) = l.levels.iter().position(|x| x.eq_ignore_ascii_case(level)) {
                        l.level = i;
                    }
                    l.auto = Some(Auto::Host(n.parse().unwrap_or(2)));
                    l.begin_host();
                }
                ["join", addr] => {
                    l.addr_text = addr.to_string();
                    l.auto = Some(Auto::Join);
                    l.begin_join();
                }
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
                        self.status = "Type the host's address and press Enter".into();
                    }
                    _ => self.done = Some(Outcome { level: String::new(), players: 0, local: 0, bind: "0.0.0.0:0".parse().unwrap(), peers: Vec::new(), seed: 0 }),
                },
                Key::Back => self.done = Some(Outcome { level: String::new(), players: 0, local: 0, bind: "0.0.0.0:0".parse().unwrap(), peers: Vec::new(), seed: 0 }),
                _ => {}
            },
            Screen::Host => match k {
                Key::Left if self.starting.is_none() && !self.levels.is_empty() => self.level = (self.level + self.levels.len() - 1) % self.levels.len(),
                Key::Right if self.starting.is_none() && !self.levels.is_empty() => self.level = (self.level + 1) % self.levels.len(),
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
                    if k == Key::Back {
                        self.leave();
                    }
                    return;
                }
                match k {
                    Key::Char(c) if c.is_ascii_digit() || c == '.' || c == ':' => self.addr_text.push(c),
                    Key::Backspace => {
                        self.addr_text.pop();
                    }
                    Key::Enter => self.begin_join(),
                    Key::Back => {
                        self.screen = Screen::Main;
                        self.status.clear();
                    }
                    _ => {}
                }
            }
        }
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
        b
    }

    /// Once a frame or so: talk to the others and look at what came.
    pub fn tick(&mut self) {
        if self.done.is_some() {
            return;
        }
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
                        T_JOIN => {
                            if !self.others.contains(&from) && self.others.len() + 1 < MAX_PLAYERS && self.starting.is_none() {
                                self.others.push(from);
                            }
                        }
                        T_LEAVE => self.others.retain(|a| *a != from),
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
                            peers,
                            seed: self.seed,
                        });
                    }
                }
            }
            Screen::Join if self.sock.is_some() => {
                let host = self.host.unwrap();
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
                    if starting && self.roster.len() == self.n_players {
                        self.client_start = true;
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
                    });
                    self.sock = None;
                    return;
                }
                if self.last_send.elapsed() > Duration::from_millis(400) {
                    self.last_send = Instant::now();
                    let mut b = MAGIC.to_le_bytes().to_vec();
                    b.push(T_JOIN);
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
                s += &format!(" Level:  < {} >   (Left / Right)\n\n", self.levels.get(self.level).map_or("?", |l| l.as_str()));
                s += &format!("   1  you (host)\n");
                for (i, a) in self.others.iter().enumerate() {
                    s += &format!("   {}  {}\n", i + 2, a);
                }
                s += &format!("\n {}\n\n Enter: start the game      Esc: cancel\n", self.status);
            }
            Screen::Join => {
                if self.sock.is_none() {
                    s += &format!(" Host address:  {}_\n\n {}\n\n Enter: connect      Esc: back\n", self.addr_text, self.status);
                } else {
                    s += &format!(" Level: {}\n\n {}\n\n Esc: leave\n", self.levels.get(self.level).map_or("?", |l| l.as_str()), self.status);
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
