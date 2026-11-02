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

const MAGIC: u32 = 0x5453_4e50; // "TSNP"
/// the most pads a packet carries
const MAX_PADS: u32 = 24;
/// how long a tick waits for a pad before the link is given up (the first tick waits longer: the other machines may still be loading)
const TIMEOUT: Duration = Duration::from_secs(10);
const FIRST_TIMEOUT: Duration = Duration::from_secs(90);
/// how long a machine that has been heard from may stay silent, while this one waits for it, before it is given up
const DROP_AFTER: Duration = Duration::from_secs(4);
/// how many of a given-up player's last pads each packet passes on, so the machines that are left all end up with the same ones
const RELAY_PADS: usize = 16;
/// how long before the packets are sent again when nothing new has come up
const RESEND: Duration = Duration::from_millis(5);

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
    /// ticks between a pad being read and being used: more hides longer stalls of the network and makes the controls lag
    pub delay: u32,
}

impl Default for Setup {
    fn default() -> Setup {
        Setup { mode: 3, time: 0, limit: 20, friendly: true, bots: Vec::new(), chars: Vec::new(), weapons: None, names: Vec::new(), delay: 2 }
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
        last: vec![Instant::now(); n],
        suspect: vec![false; n],
        reports: vec![vec![None; n]; n],
        dropped: vec![None; n],
        told: vec![false; n],
        last_sent: Instant::now(),
        sent_to: 0,
        streak: 0,
        compared: std::collections::BTreeSet::new(),
        lag: Duration::from_millis(std::env::var("TS_NETLAG").ok().and_then(|v| v.parse().ok()).unwrap_or(0)),
        inbox: std::collections::VecDeque::new(),
    });
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
    *SESSION.lock().unwrap() = None;
    *SETUP.lock().unwrap() = None;
    *SOUNDS.lock().unwrap() = None;
    set_rollback(false);
    set_replaying(false);
    set_muted(false);
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
        let mine = self.local;
        let have_to = self.pads[mine].keys().next_back().map_or(0, |t| t + 1);
        // (this is called many times a second while a tick waits: a packet goes out when there is a new pad, and otherwise every few milliseconds in case one was lost)
        let fresh = have_to != self.sent_to || self.last_sent.elapsed() > RESEND;
        if fresh {
            self.sent_to = have_to;
            self.last_sent = Instant::now();
        }
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
            // the pads of the players given up on that this machine has, and which they are
            let gone: Vec<usize> = (0..self.n()).filter(|&q| q != mine && self.suspect[q]).collect();
            b.push(gone.len() as u8);
            for &q in &gone {
                let last: Vec<(&u32, &[u8; PAD_BYTES])> = self.pads[q].iter().rev().take(RELAY_PADS).collect();
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
            }
            let _ = self.sock.send_to(&b, self.peers[p]);
        }
        let mut buf = [0u8; 2048];
        while let Ok((len, from)) = self.sock.recv_from(&mut buf) {
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
            let (Some(&q), Some(have)) = (b.get(at), u32_at(at + 1)) else { return };
            at += 5;
            let q = q as usize;
            if q >= self.n() || q == self.local || q == who {
                continue;
            }
            self.reports[who][q].get_or_insert(have);
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
                if p != s.local && s.heard[p] && !s.suspect[p] && s.last[p].elapsed() > DROP_AFTER && started.elapsed() > DROP_AFTER {
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
        let line = format!(
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
    let mut g = SESSION.lock().unwrap();
    let Some(s) = g.as_mut() else { return false };
    s.pump();
    for p in 0..s.n() {
        if p != s.local && s.heard[p] && !s.suspect[p] && s.last[p].elapsed() > DROP_AFTER && waiting > DROP_AFTER {
            s.give_up(p);
        }
    }
    s.settle();
    s.check_sums();
    !(0..s.n()).all(|p| p == s.local || s.suspect[p])
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
    }
}
