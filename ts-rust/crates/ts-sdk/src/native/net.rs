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
pub const PAD_BYTES: usize = 20;
/// The checksum is compared every this many ticks.
pub const SUM_EVERY: u32 = 30;

const MAGIC: u32 = 0x5453_4e39; // "TSN9"
/// The packets of a resync (see `Resync`), which go beside the others on the same socket.
const RS_MAGIC: u32 = 0x5453_5253; // "TSRS"
const RS_ANNOUNCE: u8 = 1;
const RS_ACK: u8 = 2;
const RS_LATE: u8 = 3;
const RS_CHUNK: u8 = 4;
const RS_HAVE: u8 = 5;
const RS_DONE: u8 = 6;
/// a machine that has lost the others, or has come back after it quit, says it is there
const RS_HELLO: u8 = 7;
/// how long a packet takes there and back: a machine sends the time, and the other sends it back
const RS_PING: u8 = 8;
const RS_PONG: u8 = 9;
/// a machine asking to come back, in the messages of the lobby
const LOBBY_MAGIC: u32 = 0x4c42_5354; // "TSBL"
const LOBBY_JOIN: u8 = 1;
/// The bytes of a resync in one packet.
const RS_PIECE: usize = 1100;
/// The form of the packets: machines that differ in it cannot play together.
pub const PROTOCOL: u32 = MAGIC;
/// the most pads a packet carries
const MAX_PADS: u32 = 24;
/// how long a tick waits for a pad before the link is given up (the first tick waits longer: the other machines may still be loading)
const TIMEOUT: Duration = Duration::from_secs(10);
const FIRST_TIMEOUT: Duration = Duration::from_secs(90);
/// how long a machine that has been heard from may stay silent, while this one waits for it, before it is given up
const DROP_AFTER_DEFAULT: Duration = Duration::from_secs(4);

/// The pad that the first pad of a packet is told as a difference from: nothing pressed, and the centre zone that the game uses.
fn base_pad() -> [u8; PAD_BYTES] {
    let mut b = neutral();
    b[12..16].copy_from_slice(&0.25f32.to_le_bytes());
    b
}

/// The fields of a pad as they change together: the buttons, the buttons newly pressed and let go, the sticks, the centre zone, the mouse.
const PAD_FIELDS: [(usize, usize); 7] = [(0, 2), (2, 4), (4, 6), (6, 8), (8, 12), (12, 16), (16, 20)];

/// A pad as it travels: a byte that says which fields differ from `before`, then those fields.  A held stick or button costs one byte, and a packet needs nothing from any other packet.
fn encode_pad(out: &mut Vec<u8>, pad: &[u8; PAD_BYTES], before: &[u8; PAD_BYTES]) {
    let mut mask = 0u8;
    for (i, &(a, b)) in PAD_FIELDS.iter().enumerate() {
        if pad[a..b] != before[a..b] {
            mask |= 1 << i;
        }
    }
    out.push(mask);
    for (i, &(a, b)) in PAD_FIELDS.iter().enumerate() {
        if mask & (1 << i) != 0 {
            out.extend_from_slice(&pad[a..b]);
        }
    }
}

/// The pad at `at` of `b` and where what follows it starts; `None` when the packet ends in it.
fn decode_pad(b: &[u8], mut at: usize, before: &[u8; PAD_BYTES]) -> Option<([u8; PAD_BYTES], usize)> {
    let mask = *b.get(at)?;
    at += 1;
    let mut pad = *before;
    for (i, &(x, y)) in PAD_FIELDS.iter().enumerate() {
        if mask & (1 << i) != 0 {
            pad[x..y].copy_from_slice(b.get(at..at + (y - x))?);
            at += y - x;
        }
    }
    Some((pad, at))
}

/// What is sent, for `net-stats.txt`: the bytes (with the 28 of the IP and UDP headers) and the packets.
static SENT_BYTES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static SENT_PACKETS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
/// A network game is on (`active` is asked at every turn of the game: this needs no lock).
static ACTIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// How many more rounds of packets carry the newest checksum: a checksum comes every half second, so it need not be in every packet, and a few tries get it there.
const SUM_SENDS: u32 = 12;

/// How long a machine may be silent before the others drop it (a test aid, `TS_NETDROP=<seconds>`, for tests of many machines on one PC).
fn drop_after() -> Duration {
    static D: std::sync::OnceLock<Duration> = std::sync::OnceLock::new();
    *D.get_or_init(|| std::env::var("TS_NETDROP").ok().and_then(|v| v.parse::<u64>().ok()).map_or(DROP_AFTER_DEFAULT, Duration::from_secs))
}
/// how many of a given-up player's last pads each packet passes on, so the machines that are left all end up with the same ones
const RELAY_PADS: usize = 16;
/// how long before the packets are sent again when nothing new has come up: more than a tick, so that a game that is running sends one packet a tick, with its new pad, and only a machine that waits sends the old ones again
const RESEND: Duration = Duration::from_millis(25);

/// What to do when the link is lost: told the reason (the port shows it and ends the game).
static ON_LOST: Mutex<Option<fn(&str)>> = Mutex::new(None);

pub fn set_on_lost(f: fn(&str)) {
    *ON_LOST.lock().unwrap() = Some(f);
}

/// The link is lost: say so (the game ends).
pub fn lost(why: &str) {
    eprintln!("[net] {why}");
    let f = *ON_LOST.lock().unwrap();
    if let Some(f) = f {
        f(why);
    }
    std::process::exit(3);
}

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
    /// the weapon set (0 custom, 1 to 9 a preset) and the five weapons of the custom one; `None`: as the game has it
    pub weapons: Option<(i32, [i32; 5])>,
    /// the name of each player
    pub names: Vec<String>,
    /// the control settings of each player's profile, as `netlink_my_prefs` tells them
    pub prefs: Vec<Vec<u8>>,
    /// the team of each player (0 to 3), as chosen on the character select pages
    pub teams: Vec<i32>,
    /// ticks between a pad being read and being used: more hides longer stalls of the network and makes the controls lag
    pub delay: u32,
    /// the parameters of the match as the game's own pages make them (28 words), when the host set the match up with them
    pub block: Vec<i32>,
    /// the game variables the host set (address, value): the other machines set them the same
    pub gvs: Vec<(i32, i32)>,
    /// the Map Maker map of the match (8192 bytes as the game keeps it), empty for the game's own levels
    pub map: Vec<u8>,
}

impl Default for Setup {
    fn default() -> Setup {
        Setup { mode: 3, time: 0, limit: 20, friendly: true, bots: Vec::new(), chars: Vec::new(), weapons: None, prefs: Vec::new(), teams: Vec::new(), names: Vec::new(), delay: 2, block: Vec::new(), gvs: Vec::new(), map: Vec::new() }
    }
}

static SETUP: Mutex<Option<Setup>> = Mutex::new(None);

pub fn set_setup(s: Setup) {
    *SETUP.lock().unwrap() = Some(s);
}

/// The weapon set of the match, when the lobby chose one.
pub fn weapons() -> Option<(i32, [i32; 5])> {
    SETUP.lock().unwrap().as_ref().and_then(|s| s.weapons)
}

pub fn setup() -> Option<Setup> {
    SETUP.lock().unwrap().clone()
}

/// How to start a session.
pub struct Config {
    /// this machine quit the match and is coming back into it
    pub rejoin: bool,
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
    /// when each player was last heard from
    last: Vec<Instant>,
    /// this machine has given the player up (what it has of their pads is then fixed, and it tells the others)
    suspect: Vec<bool>,
    /// `reports[r][q]`: how many of the pads of the given-up player `q` the machine `r` had when it gave `q` up
    reports: Vec<Vec<Option<u32>>>,
    /// a dropped player and the tick from which their pad is nothing pressed: the most pads of theirs that any machine has
    dropped: Vec<Option<u32>>,
    /// for each machine, how many pads of each player given up on it has said it has: what it has need not be passed on to it again
    relay_have: Vec<Vec<u32>>,
    /// the game has been told the player is out
    told: Vec<bool>,
    /// when packets were last sent, and how many of this machine's pads they had (a packet is sent when there is something new, or now and then)
    last_sent: Instant,
    sent_to: u32,
    /// the checksums compared one after the other that differed (a lone difference is something that only one machine's picture touches)
    streak: u32,
    /// the ticks whose checksums have been compared
    compared: std::collections::BTreeSet<u32>,
    lag: Duration,
    inbox: std::collections::VecDeque<(Instant, Vec<u8>, SocketAddr)>,
    /// a test aid, `TS_NETLOSS=<n>`: one in n of the packets that come is dropped
    loss: u32,
    received: u32,
    /// the rounds of packets that will still carry the newest checksum
    sum_sends: u32,
    /// the packet being made (kept, so that a packet needs no new memory)
    out: Vec<u8>,
    rs: Resync,
    /// the checksums of ticks before this one are not compared (they were of the state before a resync)
    sum_floor: u32,
    rejoin: Option<Rejoin>,
    last_beacon: Instant,
    /// nobody came back, and this machine has left the match
    stop_waiting: bool,
    /// when each player last came back
    welcomed: Vec<Option<Instant>>,
    /// how long a packet takes to each player and back (milliseconds, smoothed), and when this machine last asked
    pings: Vec<Option<f32>>,
    began: Instant,
    last_ping: Instant,
    /// a test aid, `TS_NETCUT=<tick>,<seconds>`: from that tick this machine neither sends nor hears anything for that long
    cut: Option<(u32, u32)>,
    cut_until: Option<Instant>,
}

/// Bringing the machines back into step: the machine in charge names a tick, every machine stops there with all the pads up to it known, and the machine in charge sends what its game's memory has.
#[derive(Default)]
struct Resync {
    /// the number of the resync (a later one replaces an earlier one), and the tick to stop at
    generation: u32,
    tick: Option<u32>,
    /// the last one this machine has finished
    finished: u32,
    /// (in charge) who has said they will stop there, who have said they were past it, who have it all, and what each has
    acked: Vec<bool>,
    late: bool,
    done: Vec<bool>,
    have: Vec<Vec<u8>>,
    pieces: Vec<Vec<u8>>,
    /// (the others) what has come
    got: Vec<Option<Vec<u8>>>,
    last_send: Option<Instant>,
    /// the machine that named the tick
    charge: usize,
    /// the player who comes back at the tick, and from where
    back: Option<(usize, SocketAddr)>,
    /// a machine that has said it is there and wants to come back
    wanting: Option<(usize, SocketAddr)>,
    last_hello: Option<Instant>,
}

/// What a machine that has quit needs to come back into the match: what the host's lobby told each player when the match began.
pub struct Rejoin {
    pub compat: u32,
    pub names: Vec<String>,
    /// for each player, the lobby's message that starts the match
    pub packets: Vec<Vec<u8>>,
    /// the lobby's announcement of a game
    pub beacon: Vec<u8>,
}

static SESSION: Mutex<Option<Session>> = Mutex::new(None);

/// Start network play.  Panics are not wanted here: a failure is reported and the game stays a local one.
pub fn start(cfg: Config) -> Result<(), String> {
    let n = cfg.peers.len();
    if !(2..=10).contains(&n) || cfg.local >= n {
        return Err(format!("{} players and this one number {}: a network game has 2 to 10 players", n, cfg.local));
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
        last: vec![Instant::now(); n],
        suspect: vec![false; n],
        reports: vec![vec![None; n]; n],
        dropped: vec![None; n],
        relay_have: vec![vec![0; n]; n],
        told: vec![false; n],
        last_sent: Instant::now(),
        sent_to: 0,
        streak: 0,
        compared: std::collections::BTreeSet::new(),
        lag: Duration::from_millis(std::env::var("TS_NETLAG").ok().and_then(|v| v.parse().ok()).unwrap_or(0)),
        inbox: std::collections::VecDeque::new(),
        loss: std::env::var("TS_NETLOSS").ok().and_then(|v| v.parse().ok()).unwrap_or(0),
        received: 0,
        sum_sends: 0,
        out: Vec::with_capacity(512),
        rs: Resync { acked: vec![false; n], done: vec![false; n], have: vec![Vec::new(); n], ..Default::default() },
        sum_floor: 0,
        rejoin: None,
        last_beacon: Instant::now(),
        stop_waiting: false,
        welcomed: vec![None; n],
        pings: vec![None; n],
        began: Instant::now(),
        last_ping: Instant::now(),
        cut: std::env::var("TS_NETCUT").ok().and_then(|v| v.split_once(',').and_then(|(a, b)| Some((a.parse().ok()?, b.parse().ok()?)))),
        cut_until: None,
    });
    // (a machine coming back knows of nobody yet, and asks)
    if cfg.rejoin {
        if let Some(s) = SESSION.lock().unwrap().as_mut() {
            for p in 0..n {
                if p != cfg.local {
                    s.suspect[p] = true;
                }
            }
        }
    }
    ACTIVE.store(true, std::sync::atomic::Ordering::Relaxed);
    Ok(())
}

/// A pad record of nothing pressed (the buttons clear, the sticks in the middle).
pub fn neutral() -> [u8; PAD_BYTES] {
    let mut b = [0u8; PAD_BYTES];
    b[8..12].fill(128);
    b
}

/// The network game is over: the session is closed (its socket too) and the game is a game of this machine again.
pub fn stop() {
    ACTIVE.store(false, std::sync::atomic::Ordering::Relaxed);
    *SESSION.lock().unwrap() = None;
    *SETUP.lock().unwrap() = None;
    *SOUNDS.lock().unwrap() = None;
    set_rollback(false);
    set_replaying(false);
    set_muted(false);
}

#[inline]
pub fn active() -> bool {
    ACTIVE.load(std::sync::atomic::Ordering::Relaxed)
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

/// The players whose pad became nothing pressed with the tick just done (each is given once; the same tick on every machine).
pub fn newly_gone() -> Vec<usize> {
    let mut g = SESSION.lock().unwrap();
    let Some(s) = g.as_mut() else { return Vec::new() };
    let done = s.tick.saturating_sub(1);
    let mut out = Vec::new();
    for p in 0..s.n() {
        if !s.told[p] && s.dropped[p].map_or(false, |v| done >= v) {
            s.told[p] = true;
            out.push(p);
        }
    }
    out
}

/// The checksums have differed three times running: the machines have parted.
pub fn out_of_step() -> bool {
    SESSION.lock().unwrap().as_ref().map_or(false, |s| s.streak >= 3)
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
        if let Some((t, secs)) = self.cut {
            if self.tick >= t {
                self.cut = None;
                self.cut_until = Some(Instant::now() + Duration::from_secs(secs as u64));
                eprintln!("[net] the test cuts this machine off for {secs} seconds");
            }
        }
        if let Some(until) = self.cut_until {
            if Instant::now() < until {
                let mut buf = [0u8; 2048];
                while self.sock.recv_from(&mut buf).is_ok() {}
                return;
            }
            self.cut_until = None;
        }
        let mine = self.local;
        let have_to = self.pads[mine].keys().next_back().map_or(0, |t| t + 1);
        // (this is called many times a second while a tick waits: a packet goes out when there is a new pad, and otherwise every few milliseconds in case one was lost)
        let fresh = have_to != self.sent_to || self.last_sent.elapsed() > RESEND;
        if fresh {
            self.sent_to = have_to;
            self.last_sent = Instant::now();
        }
        let newest_sum = if fresh && self.sum_sends > 0 { self.sums[mine].iter().next_back().map(|(t, s)| (*t, *s)) } else { None };
        if fresh {
            self.sum_sends = self.sum_sends.saturating_sub(1);
        }
        let mut b = std::mem::take(&mut self.out);
        for p in 0..self.n() {
            if !fresh {
                break;
            }
            if p == mine || self.suspect[p] {
                continue;
            }
            let first = self.acked[p];
            let last = have_to.min(first + MAX_PADS);
            let count = last.saturating_sub(first);
            b.clear();
            b.extend_from_slice(&MAGIC.to_le_bytes());
            b.push(mine as u8);
            // the first tick of the sender's that this machine still needs of the receiver
            b.extend_from_slice(&self.next_needed(p).to_le_bytes());
            b.extend_from_slice(&first.to_le_bytes());
            b.push(count as u8);
            let mut before = base_pad();
            for t in first..last {
                let pad = self.pads[mine].get(&t).copied().unwrap_or_else(neutral);
                encode_pad(&mut b, &pad, &before);
                before = pad;
            }
            // the newest checksum of this machine, while it is new
            b.push(newest_sum.is_some() as u8);
            if let Some((t, s)) = newest_sum {
                b.extend_from_slice(&t.to_le_bytes());
                b.extend_from_slice(&s.to_le_bytes());
            }
            // the pads of the players given up on that this machine has, and which they are
            let gone: Vec<usize> = (0..self.n()).filter(|&q| q != mine && self.suspect[q]).collect();
            b.push(gone.len() as u8);
            for &q in &gone {
                // (only what that machine lacks, and nothing once it has them all: a match goes on for a long time after a player has left)
                let told = self.reports[p][q];
                let seen = self.relay_have[p][q].max(told.unwrap_or(0));
                let last: Vec<(&u32, &[u8; PAD_BYTES])> = match (self.dropped[q], told) {
                    (Some(out), _) if seen >= out => Vec::new(),
                    (Some(out), Some(_)) => self.pads[q].range(seen..out).take(RELAY_PADS).collect(),
                    (None, Some(_)) => self.pads[q].range(seen..).take(RELAY_PADS).collect(),
                    _ => self.pads[q].iter().rev().take(RELAY_PADS).collect(),
                };
                b.push(q as u8);
                b.push(last.len() as u8);
                for (t, pad) in last {
                    b.extend_from_slice(&t.to_le_bytes());
                    b.extend_from_slice(pad);
                }
            }
            b.push(gone.len() as u8);
            for &q in &gone {
                b.push(q as u8);
                b.extend_from_slice(&self.reports[mine][q].unwrap_or(self.have[q]).to_le_bytes());
                b.extend_from_slice(&self.have[q].to_le_bytes());
            }
            let _ = self.sock.send_to(&b, self.peers[p]);
            SENT_BYTES.fetch_add(b.len() as u64 + 28, std::sync::atomic::Ordering::Relaxed);
            SENT_PACKETS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        self.out = b;
        self.rs_pump();
        let mut buf = [0u8; 2048];
        while let Ok((len, from)) = self.sock.recv_from(&mut buf) {
            // (not by turns: the packets of the others come in turn, and that would lose all those of one machine)
            self.received = self.received.wrapping_mul(1664525).wrapping_add(1013904223);
            if self.loss != 0 && (self.received >> 8) % self.loss == 0 {
                continue;
            }
            if self.lag.is_zero() {
                self.take(&buf[..len], from);
            } else {
                self.inbox.push_back((Instant::now() + self.lag, buf[..len].to_vec(), from));
            }
        }
        // (a test aid, `TS_NETLAG=<milliseconds>`: what comes is held back that long, as on a slow network)
        while self.inbox.front().map_or(false, |(t, _, _)| *t <= Instant::now()) {
            if let Some((_, b, from)) = self.inbox.pop_front() {
                self.take(&b, from);
            }
        }
    }

    /// The first tick of player `p`'s pads that this machine does not have.
    fn next_needed(&self, p: usize) -> u32 {
        self.have[p]
    }

    fn take(&mut self, b: &[u8], from: SocketAddr) {
        let u32_at = |i: usize| b.get(i..i + 4).map(|x| u32::from_le_bytes([x[0], x[1], x[2], x[3]]));
        if u32_at(0) == Some(RS_MAGIC) {
            self.rs_take(b, from);
            return;
        }
        if u32_at(0) == Some(LOBBY_MAGIC) {
            self.rejoin_take(b, from);
            return;
        }
        if b.len() < 14 || u32_at(0) != Some(MAGIC) {
            return;
        }
        let who = b[4] as usize;
        if who >= self.n() || who == self.local || self.suspect[who] {
            return;
        }
        self.heard[who] = true;
        self.last_heard = Instant::now();
        // (the longest time between two packets of the others: a gap is the network's, or the other machine's game not running)
        let gap = self.last[who].elapsed().as_secs_f64() * 1000.0;
        {
            let mut st = STATS.lock().unwrap();
            st.gap_max = st.gap_max.max(gap);
        }
        self.last[who] = Instant::now();
        let (Some(wants), Some(first)) = (u32_at(5), u32_at(9)) else { return };
        let count = b[13] as usize;
        self.acked[who] = self.acked[who].max(wants);
        let mut at = 14;
        let mut before = base_pad();
        for i in 0..count {
            let Some((r, next)) = decode_pad(b, at, &before) else { return };
            self.pads[who].insert(first + i as u32, r);
            before = r;
            at = next;
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
        // the pads of given-up players that this machine may not have
        let Some(&relays) = b.get(at) else { return };
        at += 1;
        for _ in 0..relays {
            let (Some(&q), Some(&count)) = (b.get(at), b.get(at + 1)) else { return };
            at += 2;
            for _ in 0..count {
                let Some(t) = u32_at(at) else { return };
                let Some(rec) = b.get(at + 4..at + 4 + PAD_BYTES) else { return };
                let q = q as usize;
                if q < self.n() && q != self.local {
                    let mut r = [0u8; PAD_BYTES];
                    r.copy_from_slice(rec);
                    self.pads[q].entry(t).or_insert(r);
                }
                at += 4 + PAD_BYTES;
            }
        }
        for q in 0..self.n() {
            while q != self.local && self.pads[q].contains_key(&self.have[q]) {
                self.have[q] += 1;
            }
        }
        // who the sender has given up on: this machine does too
        let Some(&reported) = b.get(at) else { return };
        at += 1;
        for _ in 0..reported {
            let (Some(&q), Some(have), Some(has_now)) = (b.get(at), u32_at(at + 1), u32_at(at + 5)) else { return };
            at += 9;
            let q = q as usize;
            if q >= self.n() || q == self.local || q == who {
                continue;
            }
            // (a machine that has not yet got to the tick a player came back at still says they are gone)
            if self.welcomed[q].map_or(false, |t| t.elapsed() < Duration::from_secs(3)) {
                continue;
            }
            self.reports[who][q].get_or_insert(have);
            self.relay_have[who][q] = self.relay_have[who][q].max(has_now);
            self.give_up(q);
        }
    }

    /// This machine gives the player `q` up: what it has of their pads is what it says it has.
    fn give_up(&mut self, q: usize) {
        if !self.suspect[q] {
            self.suspect[q] = true;
            self.reports[self.local][q] = Some(self.have[q]);
            eprintln!("[net] player {q} has stopped answering; dropping them");
        }
    }

    /// A given-up player is dropped once every machine still in the game has said how many of their pads it has: from the most of those, their pad is
    /// nothing pressed (no machine has used one beyond it, and the ones before it are passed on by the machines that have them).
    fn settle(&mut self) {
        for q in 0..self.n() {
            if q == self.local || !self.suspect[q] || self.dropped[q].is_some() {
                continue;
            }
            let all = (0..self.n()).filter(|&r| r != q && !self.suspect[r]).all(|r| self.reports[r][q].is_some());
            if all {
                let v = (0..self.n()).filter_map(|r| self.reports[r][q]).max().unwrap_or(0);
                self.dropped[q] = Some(v);
                eprintln!("[net] player {q} is out from tick {v}");
            }
        }
    }

    /// Everything for the tick `t` is there: the pads of the players still in, and the nothing pressed of those dropped.
    fn ready(&self, t: u32) -> bool {
        (0..self.n()).all(|p| self.pads[p].contains_key(&t) || self.dropped[p].map_or(false, |v| t >= v))
            && (0..self.n()).all(|p| self.heard[p] || self.suspect[p])
    }

    /// Compare the checksums that everyone has now (rollback: they come when the game is sure of the tick).
    fn check_sums(&mut self) {
        let mine = self.local;
        let ticks: Vec<u32> = self.sums[mine].keys().copied().filter(|t| !self.compared.contains(t)).collect();
        for t in ticks {
            if (0..self.n()).all(|p| p == mine || self.suspect[p] || self.sums[p].contains_key(&t)) {
                self.compared.insert(t);
                self.compare(t);
            }
        }
        while self.compared.len() > 64 {
            let first = *self.compared.iter().next().unwrap();
            self.compared.remove(&first);
        }
    }

    /// Compare the checksums of the tick `t`, when everyone's are there.
    fn compare(&mut self, t: u32) {
        if t < self.sum_floor {
            return;
        }
        let Some(&mine) = self.sums[self.local].get(&t) else { return };
        let before = self.desyncs;
        let mut compared = false;
        for p in 0..self.n() {
            if p == self.local || self.suspect[p] {
                continue;
            }
            if let Some(&theirs) = self.sums[p].get(&t) {
                compared = true;
                if theirs != mine {
                    self.desyncs += 1;
                    eprintln!("[net] DESYNC at tick {t}: this machine {mine:08x}, player {p} {theirs:08x}");
                }
            }
        }
        if compared {
            self.streak = if self.desyncs > before { self.streak + 1 } else { 0 };
        }
    }

    fn prune(&mut self, below: u32) {
        // (what the others still need of this machine's pads stays)
        let keep = (0..self.n()).filter(|&p| p != self.local && !self.suspect[p]).map(|p| self.acked[p]).min().unwrap_or(below).min(below).saturating_sub(2);
        for m in self.pads.iter_mut() {
            *m = m.split_off(&keep);
        }
        for m in self.sums.iter_mut() {
            *m = m.split_off(&below.saturating_sub(120));
        }
    }

    /// A machine that has quit asks to join: if a player is out, it is told how the match began, and starts the level as if it were beginning.
    fn rejoin_take(&mut self, b: &[u8], from: SocketAddr) {
        if b.len() < 10 || b[4] != LOBBY_JOIN {
            return;
        }
        let Some(r) = self.rejoin.as_ref() else { return };
        if u32::from_le_bytes([b[5], b[6], b[7], b[8]]) != r.compat {
            return;
        }
        let name = b.get(10..10 + b[9] as usize).map_or(String::new(), |n| String::from_utf8_lossy(n).to_string());
        let out: Vec<usize> = (0..self.n()).filter(|&p| p != self.local && self.dropped[p].is_some()).collect();
        let Some(&slot) = out.iter().find(|&&p| r.names.get(p) == Some(&name)).or(out.first()) else { return };
        if let Some(packet) = r.packets.get(slot) {
            let _ = self.sock.send_to(packet, from);
        }
    }

    /// All the other machines have been given up: this one can only wait for them.
    fn alone(&self) -> bool {
        (0..self.n()).all(|p| p == self.local || self.suspect[p])
    }

    /// The player comes back from the tick: on every machine their pads start again there, and the machine that comes back starts again from there with everyone's.
    fn rs_welcome(&mut self, p: usize, tick: u32) {
        let first = tick + self.delay;
        self.welcomed[p] = Some(Instant::now());
        for t in tick..first {
            self.pads[p].insert(t, neutral());
        }
        self.pads[p].retain(|t, _| *t < first);
        self.have[p] = first;
        self.suspect[p] = false;
        self.dropped[p] = None;
        self.told[p] = false;
        self.last[p] = Instant::now();
        self.heard[p] = true;
        for r in 0..self.n() {
            self.reports[r][p] = None;
            self.reports[p][r] = None;
            self.relay_have[r][p] = 0;
            self.relay_have[p][r] = 0;
        }
        if p == self.local {
            for q in 0..self.n() {
                if q == p {
                    continue;
                }
                // (this machine knows nothing of them now: a player who is really out is told of again by the others)
                self.acked[q] = first;
                self.suspect[q] = false;
                self.dropped[q] = None;
                self.told[q] = false;
                self.pads[q].retain(|t, _| *t < tick);
                self.have[q] = tick;
                self.last[q] = Instant::now();
                self.heard[q] = true;
                self.welcomed[q] = Some(Instant::now());
                for r in 0..self.n() {
                    self.reports[r][q] = None;
                    self.reports[q][r] = None;
                }
            }
            self.sent_to = 0;
        } else {
            self.acked[p] = tick;
        }
    }

    /// The machine that brings the others back into step: the first one still playing.
    fn in_charge(&self) -> usize {
        (0..self.n()).find(|&p| p == self.local || (!self.suspect[p] && self.dropped[p].is_none())).unwrap_or(self.local)
    }

    fn rs_head(&self, kind: u8) -> Vec<u8> {
        let mut b = RS_MAGIC.to_le_bytes().to_vec();
        b.push(self.local as u8);
        b.push(kind);
        b.extend_from_slice(&self.rs.generation.to_le_bytes());
        b
    }

    fn rs_to(&self, p: usize, b: &[u8]) {
        let _ = self.sock.send_to(b, self.peers[p]);
        SENT_BYTES.fetch_add(b.len() as u64, std::sync::atomic::Ordering::Relaxed);
        SENT_PACKETS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// What goes out for a resync: every 10 ms, what each machine is still to be told.
    fn rs_pump(&mut self) {
        // (the host tells the network that a place has come free in its game)
        if self.rejoin.is_some() && self.last_beacon.elapsed() > Duration::from_secs(1) && self.dropped.iter().any(|d| d.is_some()) {
            self.last_beacon = Instant::now();
            if let Some(r) = &self.rejoin {
                let _ = self.sock.set_broadcast(true);
                let _ = self.sock.send_to(&r.beacon, ("255.255.255.255", 7778));
            }
        }
        // (twice a second, how long a packet takes)
        if self.last_ping.elapsed() > Duration::from_millis(500) {
            self.last_ping = Instant::now();
            let mut b = self.rs_head(RS_PING);
            b.extend_from_slice(&(self.began.elapsed().as_millis() as u32).to_le_bytes());
            for p in 0..self.n() {
                if p != self.local && !self.suspect[p] {
                    self.rs_to(p, &b);
                }
            }
        }
        // (a machine on its own says that it is there, five times a second)
        if self.alone() && self.rs.tick.is_none() && self.rs.last_hello.map_or(true, |t| t.elapsed() > Duration::from_millis(200)) {
            self.rs.last_hello = Some(Instant::now());
            let b = self.rs_head(RS_HELLO);
            for p in 0..self.n() {
                if p != self.local {
                    self.rs_to(p, &b);
                }
            }
        }
        let Some(tick) = self.rs.tick else { return };
        if self.rs.last_send.map_or(false, |t| t.elapsed() < Duration::from_millis(10)) {
            return;
        }
        let charge = self.rs.charge;
        // (what has come is told ten times a second: more would have pieces on their way sent again)
        if charge != self.local && !self.rs.got.is_empty() && self.rs.last_send.map_or(false, |t| t.elapsed() < Duration::from_millis(100)) {
            return;
        }
        self.rs.last_send = Some(Instant::now());
        if charge != self.local {
            // (what has come, as one bit for each piece)
            let mut b = self.rs_head(if self.rs.got.is_empty() { RS_ACK } else { RS_HAVE });
            if !self.rs.got.is_empty() {
                let mut bits = vec![0u8; self.rs.got.len().div_ceil(8)];
                for (i, g) in self.rs.got.iter().enumerate() {
                    if g.is_some() {
                        bits[i / 8] |= 1 << (i % 8);
                    }
                }
                b.extend_from_slice(&bits);
            }
            self.rs_to(charge, &b);
            return;
        }
        for p in 0..self.n() {
            if p == self.local || !self.rs_counts(p) || self.rs.done[p] {
                continue;
            }
            if self.rs.pieces.is_empty() {
                let mut b = self.rs_head(RS_ANNOUNCE);
                b.extend_from_slice(&tick.to_le_bytes());
                // (who comes back, and where they are now)
                match self.rs.back {
                    Some((q, SocketAddr::V4(a))) => {
                        b.push(q as u8);
                        b.extend_from_slice(&a.ip().octets());
                        b.extend_from_slice(&a.port().to_le_bytes());
                    }
                    _ => b.push(255),
                }
                self.rs_to(p, &b);
                continue;
            }
            // (up to 16 pieces of what the machine lacks: 1.7 MB a second, which the bits that come back correct when the network loses some)
            let total = self.rs.pieces.len();
            let mut sent = 0;
            for i in 0..total {
                let has = self.rs.have[p].get(i / 8).map_or(false, |x| x & (1 << (i % 8)) != 0);
                if has {
                    continue;
                }
                let mut b = self.rs_head(RS_CHUNK);
                b.extend_from_slice(&tick.to_le_bytes());
                b.extend_from_slice(&(i as u32).to_le_bytes());
                b.extend_from_slice(&(total as u32).to_le_bytes());
                b.extend_from_slice(&self.rs.pieces[i]);
                self.rs_to(p, &b);
                // (it is taken as sent until the machine says otherwise, so that the next round sends the next pieces)
                if self.rs.have[p].len() <= i / 8 {
                    self.rs.have[p].resize(i / 8 + 1, 0);
                }
                self.rs.have[p][i / 8] |= 1 << (i % 8);
                sent += 1;
                if sent == 16 {
                    break;
                }
            }
        }
    }

    /// The machine is part of this resync: it is still playing, or it is the one coming back.
    fn rs_counts(&self, p: usize) -> bool {
        (!self.suspect[p] && self.dropped[p].is_none()) || self.rs.back.map_or(false, |b| b.0 == p)
    }

    fn rs_take(&mut self, b: &[u8], from: SocketAddr) {
        if b.len() < 10 {
            return;
        }
        let who = b[4] as usize;
        let kind = b[5];
        let generation = u32::from_le_bytes([b[6], b[7], b[8], b[9]]);
        if who >= self.n() || who == self.local {
            return;
        }
        self.last[who] = Instant::now();
        self.last_heard = Instant::now();
        let word = |at: usize| b.get(at..at + 4).map(|x| u32::from_le_bytes([x[0], x[1], x[2], x[3]]));
        match kind {
            RS_PING => {
                let mut r = self.rs_head(RS_PONG);
                r.extend_from_slice(b.get(10..14).unwrap_or(&[0; 4]));
                self.rs_to(who, &r);
            }
            RS_PONG => {
                if let Some(sent) = word(10) {
                    let ms = (self.began.elapsed().as_millis() as u32).wrapping_sub(sent) as f32;
                    self.pings[who] = Some(self.pings[who].map_or(ms, |old| old * 0.7 + ms * 0.3));
                }
            }
            RS_HELLO => {
                // (the numbers go on from the highest any machine has used)
                self.rs.generation = self.rs.generation.max(generation).max(self.rs.finished);
                // (of two machines that are each on their own, the first takes charge)
                if self.in_charge() == self.local && (!self.alone() || who > self.local) {
                    if !self.suspect[who] {
                        self.give_up(who);
                    }
                    self.settle();
                    if self.dropped[who].is_some() {
                        self.rs.wanting = Some((who, from));
                    }
                }
            }
            RS_ANNOUNCE | RS_CHUNK => {
                let Some(tick) = word(10) else { return };
                if generation <= self.rs.finished || (generation == self.rs.generation && self.rs.tick.is_some() && who != self.rs.charge) {
                    // (it did not hear that this machine has finished)
                    if generation == self.rs.finished && generation != 0 {
                        let mut r = RS_MAGIC.to_le_bytes().to_vec();
                        r.push(self.local as u8);
                        r.push(RS_DONE);
                        r.extend_from_slice(&generation.to_le_bytes());
                        self.rs_to(who, &r);
                    }
                    return;
                }
                if generation > self.rs.generation || self.rs.tick.is_none() {
                    self.rs.generation = generation;
                    self.rs.got.clear();
                    self.rs.charge = who;
                    self.peers[who] = from;
                    if kind == RS_ANNOUNCE {
                        self.rs.back = match (b.get(14), b.get(15..21)) {
                            (Some(&q), Some(a)) if (q as usize) < self.n() => Some((q as usize, SocketAddr::from(([a[0], a[1], a[2], a[3]], u16::from_le_bytes([a[4], a[5]]))))),
                            _ => None,
                        };
                        if let Some((q, a)) = self.rs.back {
                            if q != self.local {
                                self.peers[q] = a;
                            }
                        }
                    }
                    let coming_back = self.rs.back.map_or(false, |x| x.0 == self.local);
                    // (this machine is past that tick: it says so, and another is named)
                    if self.tick > tick && !coming_back {
                        self.rs.tick = None;
                        let r = self.rs_head(RS_LATE);
                        self.rs_to(who, &r);
                        return;
                    }
                    self.rs.tick = Some(tick);
                }
                if kind == RS_ANNOUNCE && generation == self.rs.generation {
                    return;
                }
                if kind == RS_CHUNK && generation == self.rs.generation {
                    let (Some(i), Some(total)) = (word(14), word(18)) else { return };
                    let (i, total) = (i as usize, total as usize);
                    if self.rs.got.len() != total {
                        self.rs.got = vec![None; total];
                    }
                    if i < total {
                        self.rs.got[i] = Some(b[22..].to_vec());
                    }
                }
            }
            _ if generation != self.rs.generation || self.rs.tick.is_none() => {}
            RS_ACK => self.rs.acked[who] = true,
            RS_LATE => self.rs.late = true,
            RS_HAVE => {
                self.rs.acked[who] = true;
                // (the bits of what has come: what was sent a moment ago may still be on its way, so a piece is sent again only after a while)
                if self.rs.last_send.is_some() {
                    self.rs.have[who] = b[10..].to_vec();
                }
            }
            RS_DONE => self.rs.done[who] = true,
            _ => {}
        }
    }
}

/// One tick of the game: this machine's pad is given (it is used `delay` ticks on), the call waits until everybody's pad for this tick is
/// there and returns them, in the order of the players.  `None`: the link is lost.
pub fn step(local_pad: [u8; PAD_BYTES]) -> Option<Vec<[u8; PAD_BYTES]>> {
    let started = Instant::now();
    note_frame(started);
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
            for p in 0..s.n() {
                if p != s.local && s.heard[p] && !s.suspect[p] && s.last[p].elapsed() > drop_after() && started.elapsed() > drop_after() {
                    s.give_up(p);
                }
            }
            s.settle();
            if (0..s.n()).all(|p| p == s.local || s.suspect[p]) {
                return None;
            }
            if s.ready(t) {
                let out: Vec<[u8; PAD_BYTES]> = (0..s.n()).map(|p| s.pads[p].get(&t).copied().unwrap_or_else(neutral)).collect();
                s.compare(t.saturating_sub(SUM_EVERY));
                s.tick += 1;
                let below = s.tick;
                if s.tick % 64 == 0 {
                    s.prune(below);
                }
                note_wait(started.elapsed(), s.tick);
                return Some(out);
            }
            if started.elapsed() > Duration::from_secs(3) && started.elapsed().as_millis() % 2000 < 3 {
                eprintln!("[net] waiting at tick {} heard {:?} have {:?} acked {:?}", s.tick, s.heard, s.have, s.acked);
            }
            let limit = if s.tick == 0 { FIRST_TIMEOUT } else { TIMEOUT };
            if s.last_heard.elapsed() > limit && started.elapsed() > limit {
                return None;
            }
        }
        std::thread::sleep(Duration::from_micros(400));
    }
}

/// How the ticks went, written to `net-stats.txt` in the saves folder every 300 ticks: the time a tick took (the game's own work and the wait together) and the
/// part of it spent waiting for the other machines' pads.  A machine whose own work is slow slows everybody; a long wait means the network or another machine.
struct Stats {
    last: Option<Instant>,
    frames: u32,
    frame_total: f64,
    frame_max: f64,
    wait_total: f64,
    wait_max: f64,
    waited: u32,
    rollbacks: u32,
    replayed: u32,
    gap_max: f64,
    subs: u32,
    /// the pictures wanted between the ticks, and the ticks that had none for the state of the game (a pause, a menu) or for the time
    wanted: u32,
    skipped_state: u32,
    skipped_time: u32,
    /// the commands for the sound chip: started in a tick, started in a tick played again (new), left out in one (already started)
    snd_live: u32,
    snd_new: u32,
    snd_skipped: u32,
}

static STATS: Mutex<Stats> = Mutex::new(Stats { last: None, frames: 0, frame_total: 0.0, frame_max: 0.0, wait_total: 0.0, wait_max: 0.0, waited: 0, rollbacks: 0, replayed: 0, gap_max: 0.0, subs: 0, wanted: 0, skipped_state: 0, skipped_time: 0, snd_live: 0, snd_new: 0, snd_skipped: 0 });

fn note_frame(now: Instant) {
    let mut st = STATS.lock().unwrap();
    if let Some(l) = st.last {
        let ms = (now - l).as_secs_f64() * 1000.0;
        st.frames += 1;
        st.frame_total += ms;
        st.frame_max = st.frame_max.max(ms);
    }
    st.last = Some(now);
}

fn note_wait(wait: Duration, tick: u32) {
    let ms = wait.as_secs_f64() * 1000.0;
    let mut st = STATS.lock().unwrap();
    st.wait_total += ms;
    st.wait_max = st.wait_max.max(ms);
    if ms > 2.0 {
        st.waited += 1;
    }
    if tick % 300 == 0 && st.frames > 0 {
        let (sent, packets) = (SENT_BYTES.swap(0, std::sync::atomic::Ordering::Relaxed), SENT_PACKETS.swap(0, std::sync::atomic::Ordering::Relaxed));
        let line = format!(
            "tick {tick}: sent {} bytes in {packets} packets; ",
            sent
        ) + &format!(
            "tick {tick}: {:.1} ms a tick (worst {:.0}), waiting for the others {:.1} ms a tick (worst {:.0}), {} of {} ticks waited more than 2 ms, {} rollbacks ({} ticks played again), the longest silence from the others {:.0} ms, {:.1} extra pictures a tick ({} wanted, {} ticks without for the state of the game, {} for lack of time), sound commands: {} in ticks, {} new and {} left out in ticks played again
",
            st.frame_total / st.frames as f64,
            st.frame_max,
            st.wait_total / st.frames as f64,
            st.wait_max,
            st.waited,
            st.frames,
            st.rollbacks,
            st.replayed,
            st.gap_max,
            st.subs as f64 / st.frames.max(1) as f64,
            st.wanted,
            st.skipped_state,
            st.skipped_time,
            st.snd_live,
            st.snd_new,
            st.snd_skipped
        );
        use std::io::Write;
        if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(crate::save_root().join("net-stats.txt")) {
            let _ = f.write_all(line.as_bytes());
        }
        *st = Stats { last: st.last, frames: 0, frame_total: 0.0, frame_max: 0.0, wait_total: 0.0, wait_max: 0.0, waited: 0, rollbacks: 0, replayed: 0, gap_max: 0.0, subs: 0, wanted: 0, skipped_state: 0, skipped_time: 0, snd_live: 0, snd_new: 0, snd_skipped: 0 };
    }
}

// ---- rollback: the game runs on without waiting and goes back when a guess about another player's pad was wrong ------------------------------------------

static ROLLBACK: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Play with rollback (the game does not wait for the others' pads: it guesses them and goes back when it guessed wrong) instead of in lockstep.
pub fn set_rollback(on: bool) {
    ROLLBACK.store(on, std::sync::atomic::Ordering::Relaxed);
}

pub fn rollback() -> bool {
    ROLLBACK.load(std::sync::atomic::Ordering::Relaxed) && active()
}

static REPLAYING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// The game is playing ticks again after going back: what only shows or sounds (the sounds) is left out.
pub fn set_replaying(on: bool) {
    REPLAYING.store(on, std::sync::atomic::Ordering::Relaxed);
}

pub fn replaying() -> bool {
    REPLAYING.load(std::sync::atomic::Ordering::Relaxed)
}

/// The sounds started in each tick, so that a tick played again after a rollback starts only the sounds that were not started the first time.
struct Sounds {
    seen: std::collections::BTreeMap<u32, std::collections::HashMap<[u32; 4], u32>>,
    used: std::collections::HashMap<[u32; 4], u32>,
}

static SOUNDS: Mutex<Option<Sounds>> = Mutex::new(None);

/// A tick is being played again: nothing of it has been played again yet.
pub fn replay_tick(t: u32) {
    set_tick(t);
    if let Some(so) = SOUNDS.lock().unwrap().as_mut() {
        so.used.clear();
    }
}

/// The game is about to start a sound (a command for the sound chip); true when it is to be done: the first time in a tick, or when a tick played again has a sound
/// that the first time did not have.
pub fn sound_new(key: [u32; 4]) -> bool {
    if muted() {
        return false;
    }
    if !rollback() {
        return true;
    }
    let tick = SESSION.lock().unwrap().as_ref().map_or(0, |s| s.tick);
    let mut g = SOUNDS.lock().unwrap();
    let so = g.get_or_insert_with(|| Sounds { seen: std::collections::BTreeMap::new(), used: std::collections::HashMap::new() });
    if !replaying() {
        STATS.lock().unwrap().snd_live += 1;
        *so.seen.entry(tick).or_default().entry(key).or_insert(0) += 1;
        while so.seen.len() > 96 {
            let first = *so.seen.keys().next().unwrap();
            so.seen.remove(&first);
        }
        return true;
    }
    let before = so.seen.get(&tick).and_then(|m| m.get(&key)).copied().unwrap_or(0);
    let used = so.used.entry(key).or_insert(0);
    *used += 1;
    if *used <= before {
        STATS.lock().unwrap().snd_skipped += 1;
        return false;
    }
    STATS.lock().unwrap().snd_new += 1;
    *so.seen.entry(tick).or_default().entry(key).or_insert(0) += 1;
    true
}

static MUTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// A picture in between the ticks is being made: it starts no sounds.
pub fn set_muted(on: bool) {
    MUTED.store(on, std::sync::atomic::Ordering::Relaxed);
}

pub fn muted() -> bool {
    MUTED.load(std::sync::atomic::Ordering::Relaxed)
}

/// The players whose pad becomes nothing pressed with this tick (they left: bit `p`).
pub fn gone_mask(t: u32) -> u8 {
    let g = SESSION.lock().unwrap();
    let Some(s) = g.as_ref() else { return 0 };
    (0..s.n()).filter(|&p| s.dropped[p] == Some(t)).fold(0u8, |m, p| m | (1 << p))
}

/// The most ticks the game may run on with a guessed pad.
pub const MAX_AHEAD: u32 = 12;

/// The ticks of the game are numbered by the game (rollback): this is the one it is at.
pub fn set_tick(t: u32) {
    if let Some(s) = SESSION.lock().unwrap().as_mut() {
        s.tick = t;
    }
}

pub fn delay() -> u32 {
    SESSION.lock().unwrap().as_ref().map_or(0, |s| s.delay)
}

/// This machine's pad for the tick `tick` is given; it is used `delay` ticks on (and sent at once).
pub fn submit(tick: u32, pad: [u8; PAD_BYTES]) {
    let mut g = SESSION.lock().unwrap();
    let Some(s) = g.as_mut() else { return };
    let at = tick + s.delay;
    let mine = s.local;
    s.pads[mine].insert(at, pad);
    s.have[mine] = at + 1;
    s.pump();
}

/// Take in what has come and send what is new; see who has stopped answering.  False: nobody else is left.
pub fn poll(waiting: Duration) -> bool {
    // (a game with rollback waits for the others to come back: see `rs_alone`)
    let keep_on = ROLLBACK.load(std::sync::atomic::Ordering::Relaxed);
    let mut g = SESSION.lock().unwrap();
    let Some(s) = g.as_mut() else { return false };
    s.pump();
    for p in 0..s.n() {
        if p != s.local && s.heard[p] && !s.suspect[p] && s.last[p].elapsed() > drop_after() && waiting > drop_after() {
            s.give_up(p);
        }
    }
    s.settle();
    s.check_sums();
    keep_on || !(0..s.n()).all(|p| p == s.local || s.suspect[p])
}

/// Every other machine has been heard from.
pub fn all_heard() -> bool {
    SESSION.lock().unwrap().as_ref().map_or(false, |s| (0..s.n()).all(|p| s.heard[p] || s.suspect[p]))
}

/// The pad of the player `p` for the tick, when it is known (the real one: from the machine, or nothing pressed for a player who has gone).
pub fn pad(p: usize, tick: u32) -> Option<[u8; PAD_BYTES]> {
    let g = SESSION.lock().unwrap();
    let s = g.as_ref()?;
    if let Some(r) = s.pads.get(p)?.get(&tick) {
        return Some(*r);
    }
    match s.dropped[p] {
        Some(v) if tick >= v => Some(neutral()),
        _ => None,
    }
}

/// The first tick for which a pad of some player is not known yet: the game up to there is certain.
pub fn confirmed() -> u32 {
    let g = SESSION.lock().unwrap();
    let Some(s) = g.as_ref() else { return 0 };
    (0..s.n()).map(|p| if s.dropped[p].is_some() { u32::MAX } else { s.have[p] }).min().unwrap_or(0)
}

/// Another player's pad is guessed as the last one known (the buttons held stay held; a button that was just pressed or let go is not pressed again).
pub fn guess(p: usize, tick: u32) -> [u8; PAD_BYTES] {
    let g = SESSION.lock().unwrap();
    let Some(s) = g.as_ref() else { return neutral() };
    let mut b = s.pads[p].range(..tick).next_back().map(|(_, r)| *r).unwrap_or_else(neutral);
    // (the words of the buttons newly pressed and newly released: bytes 4 to 8)
    b[4..8].fill(0);
    // (and no turning by the mouse)
    b[16..20].fill(0);
    b
}

/// What is older than the tick `t` is not wanted any more.
pub fn prune_before(t: u32) {
    if let Some(s) = SESSION.lock().unwrap().as_mut() {
        s.prune(t);
    }
}

/// Wait (polling) until the pads of every player are known for the tick `tick`; false when that does not come (the link is lost).
pub fn wait_known(tick: u32, limit: Duration) -> bool {
    let started = Instant::now();
    loop {
        if !poll(started.elapsed()) {
            return false;
        }
        if confirmed() > tick {
            return true;
        }
        if started.elapsed() > limit {
            return false;
        }
        std::thread::sleep(Duration::from_micros(300));
    }
}

/// Wait until every other machine has been heard from (the others may still be loading).
pub fn wait_heard(limit: Duration) -> bool {
    let started = Instant::now();
    loop {
        if !poll(Duration::ZERO) {
            return false;
        }
        if all_heard() {
            return true;
        }
        if started.elapsed() > limit {
            return false;
        }
        std::thread::sleep(Duration::from_micros(300));
    }
}

/// The pictures made between this tick and the next (a measure for the stats).
pub fn note_subframes(n: u32) {
    STATS.lock().unwrap().subs += n;
}

/// A tick's pictures in between: how many were wanted, and why there were none (`state`: a pause or a menu; or the time ran out).
pub fn note_subframes_wanted(wanted: u32, state_skip: bool, time_skip: bool) {
    let mut st = STATS.lock().unwrap();
    st.wanted += wanted;
    st.skipped_state += state_skip as u32;
    st.skipped_time += time_skip as u32;
}

/// The ticks that were played again (a measure for the stats): how many times the game went back and how many ticks that made.
pub fn note_rollback(frames: u32) {
    let mut st = STATS.lock().unwrap();
    st.rollbacks += 1;
    st.replayed += frames;
}

/// The frame boundary of a game with rollback (`tick` is the tick the game is at, `waited` how long it waited for the others).
pub fn note_boundary(tick: u32, waited: Duration) {
    note_frame(Instant::now());
    note_wait(waited, tick);
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
        s.sum_sends = SUM_SENDS;
    }
}

// ---- bringing the machines back into step (`netlink_boundary`) ------------------------------------------------------------------------------------------------

/// This machine is the one that sends its game to the others.
pub fn rs_in_charge() -> bool {
    SESSION.lock().unwrap().as_ref().map_or(false, |s| s.in_charge() == s.local && s.rs.charge_is_me(s.local))
}

/// Nobody came back: this machine stops waiting.
pub fn rs_give_up_waiting() {
    if let Some(s) = SESSION.lock().unwrap().as_mut() {
        s.stop_waiting = true;
    }
}

/// All the other machines have been given up: the game cannot go on, and this machine keeps saying that it is there.
pub fn rs_alone() -> bool {
    SESSION.lock().unwrap().as_ref().map_or(false, |s| ROLLBACK.load(std::sync::atomic::Ordering::Relaxed) && s.alone() && !s.stop_waiting)
}

/// This machine is the one coming back at the tick that has been named.
pub fn rs_coming_back() -> bool {
    SESSION.lock().unwrap().as_ref().map_or(false, |s| s.rs.tick.is_some() && s.rs.back.map_or(false, |b| b.0 == s.local))
}

/// (in charge) A machine has said it is there and wants to come back (once).
pub fn rs_take_wanting() -> Option<usize> {
    let mut g = SESSION.lock().unwrap();
    let s = g.as_mut()?;
    let (p, a) = s.rs.wanting.take()?;
    s.rs.back = Some((p, a));
    // (a machine that has started again has another port)
    s.peers[p] = a;
    Some(p)
}

/// (in charge) The machine that was coming back has not answered: the others go on without it.
pub fn rs_forget_back() {
    if let Some(s) = SESSION.lock().unwrap().as_mut() {
        s.rs.back = None;
    }
}

/// The player is coming back (for a test or a message).
pub fn rs_back() -> Option<usize> {
    SESSION.lock().unwrap().as_ref().and_then(|s| s.rs.back.map(|b| b.0))
}

/// (in charge) The machine coming back has it all, or has been waited for long enough.
pub fn rs_stop_waiting_for_back() {
    if let Some(s) = SESSION.lock().unwrap().as_mut() {
        if let Some((p, _)) = s.rs.back {
            s.rs.done[p] = true;
        }
    }
}

/// What a machine needs to come back after it quit: the host keeps it for the match.
pub fn set_rejoin(r: Rejoin) {
    if let Some(s) = SESSION.lock().unwrap().as_mut() {
        s.rejoin = Some(r);
    }
}

/// The machines are to stop at the tick (a later call replaces it).
pub fn rs_name_tick(tick: u32) {
    if let Some(s) = SESSION.lock().unwrap().as_mut() {
        s.rs.generation = s.rs.generation.max(s.rs.finished) + 1;
        s.rs.tick = Some(tick);
        s.rs.charge = s.local;
        s.rs.late = false;
        s.rs.pieces.clear();
        let n = s.n();
        s.rs.acked = vec![false; n];
        s.rs.done = vec![false; n];
        s.rs.have = vec![Vec::new(); n];
    }
}

/// The tick the machines are to stop at, when one has been named.
pub fn rs_tick() -> Option<u32> {
    SESSION.lock().unwrap().as_ref().and_then(|s| s.rs.tick)
}

/// (in charge) A machine was past the tick that was named (once).
pub fn rs_take_late() -> bool {
    SESSION.lock().unwrap().as_mut().map_or(false, |s| std::mem::take(&mut s.rs.late))
}

/// (in charge) Every machine still playing has said it will stop at the tick.
pub fn rs_all_acked() -> bool {
    SESSION.lock().unwrap().as_ref().map_or(true, |s| (0..s.n()).all(|p| p == s.local || !s.rs_counts(p) || s.rs.acked[p]))
}

/// (in charge) What is sent: `data` in pieces, packed.
pub fn rs_send(data: &[u8]) -> usize {
    let packed = miniz_oxide::deflate::compress_to_vec(data, 1);
    let mut g = SESSION.lock().unwrap();
    let Some(s) = g.as_mut() else { return 0 };
    s.rs.pieces = packed.chunks(RS_PIECE).map(|c| c.to_vec()).collect();
    packed.len()
}

/// (in charge) Every machine still playing has it all.  A piece the network lost is asked for again by the bits a machine keeps sending.
pub fn rs_all_done() -> bool {
    SESSION.lock().unwrap().as_ref().map_or(true, |s| (0..s.n()).all(|p| p == s.local || !s.rs_counts(p) || s.rs.done[p]))
}

/// (the others) What has been sent, once all of it has come.
pub fn rs_received() -> Option<Vec<u8>> {
    let g = SESSION.lock().unwrap();
    let s = g.as_ref()?;
    if s.rs.got.is_empty() || s.rs.got.iter().any(|p| p.is_none()) {
        return None;
    }
    let packed: Vec<u8> = s.rs.got.iter().flatten().flatten().copied().collect();
    miniz_oxide::inflate::decompress_to_vec(&packed).ok()
}

/// The resync is over on this machine: the checksums start again from `floor`.
pub fn rs_finish(floor: u32) {
    if let Some(s) = SESSION.lock().unwrap().as_mut() {
        let tick = s.rs.tick.unwrap_or(s.tick);
        if s.rs.charge != s.local {
            let mut r = s.rs_head(RS_DONE);
            r.truncate(10);
            let charge = s.rs.charge;
            for _ in 0..3 {
                s.rs_to(charge, &r);
            }
        }
        if let Some((p, a)) = s.rs.back.take() {
            if p != s.local {
                s.peers[p] = a;
            }
            s.rs_welcome(p, tick);
            s.tick = tick;
        }
        s.rs.finished = s.rs.generation;
        s.rs.tick = None;
        s.rs.pieces.clear();
        s.rs.got.clear();
        s.sum_floor = floor;
        s.streak = 0;
        s.compared.clear();
        for m in s.sums.iter_mut() {
            *m = m.split_off(&floor);
        }
    }
}

/// How long a packet takes to each player and back, in milliseconds, for those still playing.
pub fn pings() -> Vec<Option<u32>> {
    SESSION.lock().unwrap().as_ref().map_or(Vec::new(), |s| (0..s.n()).map(|p| if s.suspect[p] { None } else { s.pings[p].map(|m| m.round() as u32) }).collect())
}

impl Resync {
    /// This machine is not following another's resync.
    fn charge_is_me(&self, local: usize) -> bool {
        self.tick.is_none() || self.charge == local
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What is told goes back to the same pads, a packet that ends early is noticed, and pads that do not change cost a byte each.
    #[test]
    fn pads_travel() {
        let mut seed = 5u32;
        let mut r = move |m: u32| {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            (seed >> 8) % m
        };
        for _ in 0..2000 {
            let mut pads = vec![base_pad()];
            for _ in 0..r(24) {
                let mut p = *pads.last().unwrap();
                for _ in 0..r(4) {
                    let i = r(PAD_BYTES as u32) as usize;
                    p[i] = r(256) as u8;
                }
                pads.push(p);
            }
            let mut b = Vec::new();
            let mut before = base_pad();
            for p in &pads {
                encode_pad(&mut b, p, &before);
                before = *p;
            }
            let (mut at, mut before) = (0, base_pad());
            for p in &pads {
                let (got, next) = decode_pad(&b, at, &before).unwrap();
                assert_eq!(got, *p);
                before = got;
                at = next;
            }
            assert_eq!(at, b.len());
            if !b.is_empty() {
                let cut = &b[..b.len() - 1];
                let (mut at, mut before, mut whole) = (0, base_pad(), true);
                for _ in &pads {
                    match decode_pad(cut, at, &before) {
                        Some((got, next)) => {
                            before = got;
                            at = next;
                        }
                        None => {
                            whole = false;
                            break;
                        }
                    }
                }
                assert!(!whole, "a packet cut short went unnoticed");
            }
        }
        let mut b = Vec::new();
        encode_pad(&mut b, &base_pad(), &base_pad());
        assert_eq!(b.len(), 1);
    }
}
