//! Network play (lockstep): every machine runs the whole game, and what travels is each player's pad, one record of 16 bytes for each tick.
//!
//! A tick of the game waits until the pads of every player are there for it; the pad of the player on this machine is read `delay` ticks
//! before it is used (by everybody, this machine included), which gives the packets that long to arrive.  Packets are UDP: each carries
//! the pads of the sender from the first tick the receiver still needs, so a lost packet is made good by the next, and says which tick of
//! the receiver's it wants next.  Every `SUM_EVERY` ticks the machines also compare a checksum of the game's state (`submit_sum`): a
//! difference is a desync and is reported.
//!
//! The game calls (through `ts-game`'s `netlink.rs`): `active`, `local_player`, `num_players`, `seed`, and each tick `step`.

use std::collections::BTreeMap;
use std::net::{SocketAddr, UdpSocket};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// The size of a pad record.
pub const PAD_BYTES: usize = 16;
/// The checksum is compared every this many ticks.
pub const SUM_EVERY: u32 = 30;

const MAGIC: u32 = 0x5453_4e50; // "TSNP"
/// the most pads a packet carries
const MAX_PADS: u32 = 24;
/// how long a tick waits for a pad before the link is given up
const TIMEOUT: Duration = Duration::from_secs(30);

/// What the lobby decided about the match, besides the level: the same on every machine.
#[derive(Clone, Debug, PartialEq)]
pub struct Setup {
    /// the game mode: 1 BagTag, 2 Capture the Bag, 3 Deathmatch, 4 Escort, 5 KnockOut, 6 Last Stand
    pub mode: i32,
    /// the time limit in minutes (0: none)
    pub time: i32,
    /// the score limit (the kills of Deathmatch, the goal of Capture the Bag and KnockOut)
    pub limit: i32,
    /// friendly fire in team games
    pub friendly: bool,
    /// the characters of the bots
    pub bots: Vec<i32>,
    /// the character of each player
    pub chars: Vec<i32>,
}

impl Default for Setup {
    fn default() -> Setup {
        Setup { mode: 3, time: 0, limit: 20, friendly: true, bots: Vec::new(), chars: Vec::new() }
    }
}

static SETUP: Mutex<Option<Setup>> = Mutex::new(None);

pub fn set_setup(s: Setup) {
    *SETUP.lock().unwrap() = Some(s);
}

pub fn setup() -> Option<Setup> {
    SETUP.lock().unwrap().clone()
}

/// How to start a session.
pub struct Config {
    /// this machine's player (0 to `peers.len() - 1`)
    pub local: usize,
    /// the address of every player's machine (the others use these to send to this machine too: this machine's own is where they find it)
    pub peers: Vec<SocketAddr>,
    /// where this machine listens
    pub bind: SocketAddr,
    /// the random seed of the game, the same on every machine
    pub seed: i32,
    /// ticks between a pad being read and being used
    pub delay: u32,
}

struct Session {
    sock: UdpSocket,
    local: usize,
    peers: Vec<SocketAddr>,
    delay: u32,
    seed: i32,
    /// the pads of every player: tick -> pad
    pads: Vec<BTreeMap<u32, [u8; PAD_BYTES]>>,
    /// for each player: the first tick of this machine's pads that player does not have yet
    acked: Vec<u32>,
    /// for each player: the first tick of theirs this machine does not have yet
    have: Vec<u32>,
    /// the checksums: of every player, tick -> sum
    sums: Vec<BTreeMap<u32, u32>>,
    /// a packet of each other player has been seen (the game starts when all have)
    heard: Vec<bool>,
    /// the tick the game is at
    tick: u32,
    /// desyncs found
    desyncs: u32,
    last_heard: Instant,
}

static SESSION: Mutex<Option<Session>> = Mutex::new(None);

/// Start network play.  Panics are not wanted here: a failure is reported and the game stays a local one.
pub fn start(cfg: Config) -> Result<(), String> {
    let n = cfg.peers.len();
    if !(2..=4).contains(&n) || cfg.local >= n {
        return Err(format!("{} players and this one number {}: a network game has 2 to 4 players", n, cfg.local));
    }
    let sock = UdpSocket::bind(cfg.bind).map_err(|e| format!("cannot listen on {}: {e}", cfg.bind))?;
    sock.set_nonblocking(true).map_err(|e| e.to_string())?;
    let mut pads = vec![BTreeMap::new(); n];
    // (the first ticks have nothing to wait for: nobody has pressed anything yet)
    for p in pads.iter_mut() {
        for t in 0..cfg.delay {
            p.insert(t, neutral());
        }
    }
    let mut heard = vec![false; n];
    heard[cfg.local] = true;
    *SESSION.lock().unwrap() = Some(Session {
        sock,
        local: cfg.local,
        peers: cfg.peers,
        delay: cfg.delay,
        seed: cfg.seed,
        pads,
        acked: vec![0; n],
        have: vec![0; n],
        sums: vec![BTreeMap::new(); n],
        heard,
        tick: 0,
        desyncs: 0,
        last_heard: Instant::now(),
    });
    Ok(())
}

/// A pad record of nothing pressed (the buttons clear, the sticks in the middle).
pub fn neutral() -> [u8; PAD_BYTES] {
    let mut b = [0u8; PAD_BYTES];
    b[8..12].fill(128);
    b
}

pub fn active() -> bool {
    SESSION.lock().unwrap().is_some()
}

pub fn local_player() -> i32 {
    SESSION.lock().unwrap().as_ref().map_or(0, |s| s.local as i32)
}

pub fn num_players() -> i32 {
    SESSION.lock().unwrap().as_ref().map_or(0, |s| s.peers.len() as i32)
}

pub fn seed() -> Option<i32> {
    SESSION.lock().unwrap().as_ref().map(|s| s.seed)
}

pub fn desyncs() -> u32 {
    SESSION.lock().unwrap().as_ref().map_or(0, |s| s.desyncs)
}

impl Session {
    fn n(&self) -> usize {
        self.peers.len()
    }

    /// Send what the others still need, and take in what arrived.
    fn pump(&mut self) {
        let mine = self.local;
        let have_to = self.pads[mine].keys().next_back().map_or(0, |t| t + 1);
        for p in 0..self.n() {
            if p == mine {
                continue;
            }
            let first = self.acked[p];
            let last = have_to.min(first + MAX_PADS);
            let count = last.saturating_sub(first);
            let mut b = Vec::with_capacity(32 + count as usize * PAD_BYTES);
            b.extend_from_slice(&MAGIC.to_le_bytes());
            b.push(mine as u8);
            // the first tick of the sender's that this machine still needs of the receiver
            b.extend_from_slice(&self.next_needed(p).to_le_bytes());
            b.extend_from_slice(&first.to_le_bytes());
            b.push(count as u8);
            for t in first..last {
                b.extend_from_slice(self.pads[mine].get(&t).unwrap_or(&neutral()));
            }
            // the recent checksums of this machine
            let recent: Vec<(u32, u32)> = self.sums[mine].iter().rev().take(4).map(|(t, s)| (*t, *s)).collect();
            b.push(recent.len() as u8);
            for (t, s) in recent {
                b.extend_from_slice(&t.to_le_bytes());
                b.extend_from_slice(&s.to_le_bytes());
            }
            let _ = self.sock.send_to(&b, self.peers[p]);
        }
        let mut buf = [0u8; 2048];
        while let Ok((len, from)) = self.sock.recv_from(&mut buf) {
            self.take(&buf[..len], from);
        }
    }

    /// The first tick of player `p`'s pads that this machine does not have.
    fn next_needed(&self, p: usize) -> u32 {
        self.have[p]
    }

    fn take(&mut self, b: &[u8], from: SocketAddr) {
        let u32_at = |i: usize| b.get(i..i + 4).map(|x| u32::from_le_bytes([x[0], x[1], x[2], x[3]]));
        if b.len() < 14 || u32_at(0) != Some(MAGIC) {
            return;
        }
        let who = b[4] as usize;
        if who >= self.n() || who == self.local {
            return;
        }
        self.heard[who] = true;
        self.last_heard = Instant::now();
        let (Some(wants), Some(first)) = (u32_at(5), u32_at(9)) else { return };
        let count = b[13] as usize;
        self.acked[who] = self.acked[who].max(wants);
        let mut at = 14;
        for i in 0..count {
            let Some(rec) = b.get(at..at + PAD_BYTES) else { return };
            let mut r = [0u8; PAD_BYTES];
            r.copy_from_slice(rec);
            self.pads[who].insert(first + i as u32, r);
            at += PAD_BYTES;
        }
        while self.pads[who].contains_key(&self.have[who]) {
            self.have[who] += 1;
        }
        let Some(&k) = b.get(at) else { return };
        at += 1;
        for _ in 0..k {
            let (Some(t), Some(s)) = (u32_at(at), u32_at(at + 4)) else { return };
            self.sums[who].insert(t, s);
            at += 8;
        }
    }

    /// Compare the checksums of the tick `t`, when everyone's are there.
    fn compare(&mut self, t: u32) {
        let Some(&mine) = self.sums[self.local].get(&t) else { return };
        for p in 0..self.n() {
            if p == self.local {
                continue;
            }
            if let Some(&theirs) = self.sums[p].get(&t) {
                if theirs != mine {
                    self.desyncs += 1;
                    eprintln!("[net] DESYNC at tick {t}: this machine {mine:08x}, player {p} {theirs:08x}");
                }
            }
        }
    }

    fn prune(&mut self, below: u32) {
        // (what the others still need of this machine's pads stays)
        let keep = (0..self.n()).filter(|&p| p != self.local).map(|p| self.acked[p]).min().unwrap_or(below).min(below).saturating_sub(2);
        for m in self.pads.iter_mut() {
            *m = m.split_off(&keep);
        }
        for m in self.sums.iter_mut() {
            *m = m.split_off(&below.saturating_sub(120));
        }
    }
}

/// One tick of the game: this machine's pad is given (it is used `delay` ticks on), the call waits until everybody's pad for this tick is
/// there and returns them, in the order of the players.  `None`: the link is lost.
pub fn step(local_pad: [u8; PAD_BYTES]) -> Option<Vec<[u8; PAD_BYTES]>> {
    let started = Instant::now();
    {
        let mut g = SESSION.lock().unwrap();
        let s = g.as_mut()?;
        let t = s.tick;
        let at = t + s.delay;
        let mine = s.local;
        s.pads[mine].insert(at, local_pad);
        s.have[mine] = at + 1;
    }
    loop {
        {
            let mut g = SESSION.lock().unwrap();
            let s = g.as_mut()?;
            s.pump();
            let t = s.tick;
            let everyone = s.heard.iter().all(|&h| h) && (0..s.n()).all(|p| s.pads[p].contains_key(&t));
            if everyone {
                let out: Vec<[u8; PAD_BYTES]> = (0..s.n()).map(|p| s.pads[p][&t]).collect();
                s.compare(t.saturating_sub(SUM_EVERY));
                s.tick += 1;
                let below = s.tick;
                if s.tick % 64 == 0 {
                    s.prune(below);
                }
                return Some(out);
            }
            if started.elapsed() > Duration::from_secs(3) && started.elapsed().as_millis() % 2000 < 3 {
                eprintln!("[net] waiting at tick {} heard {:?} have {:?} acked {:?}", s.tick, s.heard, s.have, s.acked);
            }
            if s.last_heard.elapsed() > TIMEOUT && started.elapsed() > TIMEOUT {
                eprintln!("[net] nothing has been heard of the other players for {} seconds: the link is lost", TIMEOUT.as_secs());
                return None;
            }
        }
        std::thread::sleep(Duration::from_micros(400));
    }
}

/// The tick the game is at (the number of ticks done).
pub fn tick() -> u32 {
    SESSION.lock().unwrap().as_ref().map_or(0, |s| s.tick)
}

/// Give the checksum of the game's state after the tick `t` (it is compared with the others' a few ticks later).
pub fn submit_sum(t: u32, sum: u32) {
    if let Some(s) = SESSION.lock().unwrap().as_mut() {
        let mine = s.local;
        s.sums[mine].insert(t, sum);
    }
}
