//! The Virus mode (from the later games): a way to play Deathmatch, `rule virus 1` in a variant of `modes.txt`.
//!
//! A free for all: everyone is a survivor but one, who starts infected.  An infected character that touches a survivor infects him; he is on the infected side from then on and respawns as
//! one.  The last survivor wins.  The survivors run from the infected and never shoot; the infected go after the nearest survivor on foot.  The sides are kept in the team word of a character's
//! record (+24, 1 survivors, 2 infected: the game uses it to know whom a computer character goes after; the teams of the Options page stay off), and an infected character has the colour of
//! the green team.

use crate::*;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::Mutex;

/// the order in which the characters were infected (the first infected is 0), for the table of results; -1 for one who is not
static ORDER: Mutex<Vec<i32>> = Mutex::new(Vec::new());
/// the game frame of the last tick (a smaller one means the level started again)
static LAST_GAME_FRAME: AtomicI32 = AtomicI32::new(0);


const SURVIVORS: i32 = 1;
const INFECTED: i32 = 2;

static SETUP: AtomicBool = AtomicBool::new(false);
/// the game frame at which the teams were made
static SETUP_FRAME: AtomicI32 = AtomicI32::new(0);

/// Show a message to a player of this machine (the text is kept in the port's tables).
fn tell(player: i32, text: &str) {
    if !(0..lw(g::lvnumplayers)).contains(&player) {
        return;
    }
    let at = 0x240_0000 + 0x17_2c00;
    for (i, b) in text.bytes().take(60).enumerate() {
        sb(at + i as i32, b as i32);
    }
    sb(at + text.len().min(60) as i32, 0);
    hudMessage(player, at);
}

/// A game starts: the teams are made again when the level has its characters.
pub fn virus_reset() {
    SETUP.store(false, Ordering::Relaxed);
}

fn virus_on() -> bool {
    active_rules(lw(g::curgameparms + 72)).iter().any(|r| r.0 == Rule::Virus)
}

fn characters() -> (i32, i32) {
    (lw(g::chrinfo), lw(g::lvnumplayers) + lw(g::numactivechars))
}

/// Every game frame while the Virus rule is on (`custom_rules_tick`): the teams are made when the level has started, and the game is over when there is no survivor.
pub fn virus_tick(infected_at_start: i32) {
    if lw(g::gamestate) != 3 {
        return;
    }
    let (info, total) = characters();
    if info == 0 || total < 2 {
        return;
    }
    let frame = ts_sdk::frame_count() as i32;
    // (the level was started again from the menu of the game or the results: the sides are made again)
    let game_frame = lw(g::gameframe);
    if game_frame < LAST_GAME_FRAME.swap(game_frame, Ordering::Relaxed) {
        SETUP.store(false, Ordering::Relaxed);
    }
    if std::env::var_os("TS_VIRUSLOG").is_some() && frame % 300 == 0 {
        let at = |i: i32| {
            let prop = lw(i * 0x1210 + info + 3020);
            if prop == 0 { (0, 0, 0) } else { (lwc1(prop + 48) as i32, lwc1(prop + 56) as i32, lw(i * 0x1210 + info + 2716)) }
        };
        eprintln!("[virus] f{frame} players {} teams {:?} (x, z, state) {:?}", lw(g::lvnumplayers), (0..total).map(|i| lw(i * 0x1210 + info + 24)).collect::<Vec<_>>(), (0..total).map(at).collect::<Vec<_>>());
    }
    if !SETUP.load(Ordering::Relaxed) {
        // (a few frames after the level starts, when all the characters are made)
        if lw(g::lvnumplayers) <= 0 {
            return;
        }
        let present: Vec<i32> = (0..total).collect();
        if present.len() < 2 {
            return;
        }
        for &i in &present {
            sw(i * 0x1210 + info + 24, SURVIVORS);
        }
        *ORDER.lock().unwrap() = vec![-1; total as usize];
        let mut pool = present.clone();
        for _ in 0..infected_at_start.clamp(1, (present.len() as i32 - 1).max(1)) {
            // (chosen by the game's own random numbers)
            let k = std::env::var("TS_VIRUSZERO").ok().and_then(|v| v.parse::<usize>().ok()).unwrap_or(((newrnd() as u32) >> 8) as usize) % pool.len();
            let zero = pool.remove(k);
            sw(zero * 0x1210 + info + 24, INFECTED);
            ORDER.lock().unwrap()[zero as usize] = 0;
            tell(zero, "You are infected! Infect them all");
            hurt_sound(info, zero);
            if std::env::var_os("TS_VIRUSLOG").is_some() {
                eprintln!("[virus] f{frame} character {zero} starts infected");
            }
        }
        SETUP.store(true, Ordering::Relaxed);
        SETUP_FRAME.store(frame, Ordering::Relaxed);
        return;
    }
    let mut survivors = Vec::new();
    let mut infected = Vec::new();
    let green = lw((4 << 3) + g::storydata + 68);
    for i in 0..total {
        let record = info + i * 0x1210;
        match lw(record + 24) {
            SURVIVORS => survivors.push(i),
            INFECTED => {
                infected.push(i);
                sw(record + 20, green);
                // (an infected character groans now and then)
                if alive(info, i) && (newrnd() as u32) % 480 == 0 {
                    hurt_sound(info, i);
                }
            }
            _ => {}
        }
    }
    // (the touch of an infected character infects a survivor)
    for &a in &infected {
        for &b in &survivors {
            if lw(info + b * 0x1210 + 24) != SURVIVORS || !alive(info, a) || !alive(info, b) {
                continue;
            }
            let (pa, pb) = (lw(info + a * 0x1210 + 3020), lw(info + b * 0x1210 + 3020));
            if pa == 0 || pb == 0 {
                continue;
            }
            let (dx, dy, dz) = (lwc1(pa + 48) - lwc1(pb + 48), lwc1(pa + 52) - lwc1(pb + 52), lwc1(pa + 56) - lwc1(pb + 56));
            if dx * dx + dz * dz < TOUCH * TOUCH && dy.abs() < 1.6 {
                infect(info, a, b, frame);
            }
        }
    }
    // (the game is over when one survivor is left: he wins)
    let left: Vec<i32> = (0..total).filter(|&i| lw(i * 0x1210 + info + 24) == SURVIVORS).collect();
    if left.len() <= 1 && lw(g::gamestate) != 4 && frame > SETUP_FRAME.load(Ordering::Relaxed) + 60 {
        if std::env::var_os("TS_VIRUSLOG").is_some() {
            eprintln!("[virus] f{frame} {}", if left.is_empty() { "nobody is left uninfected".to_string() } else { format!("character {} is the last survivor", left[0]) });
        }
        if let Some(&winner) = left.first() {
            // (he is first on the scores)
            let best = (0..total).map(|i| lw(i * 0x1210 + info + 248)).max().unwrap_or(0);
            sw(winner * 0x1210 + info + 248, best + 1);
            tell(winner, "You are the last survivor!");
        }
        gameLevelCompleted();
    }
}

/// How close an infected character has to be to a survivor to infect him (world units).
const TOUCH: f32 = 1.3;

/// Is the character `i` on the field (a computer character that is dying or spawning is not, nor a player who is dead)?
fn alive(info: i32, i: i32) -> bool {
    let player = lw(g::players) + i * 1820;
    if i < lw(g::lvnumplayers) {
        lw(player + 296) != 4
    } else {
        lw(info + i * 0x1210 + 2716) & (16 | 32) == 0
    }
}

/// The infected character `a` touches the survivor `b`.
fn infect(info: i32, a: i32, b: i32, frame: i32) {
    sw(info + b * 0x1210 + 24, INFECTED);
    {
        let mut order = ORDER.lock().unwrap();
        let next = order.iter().max().copied().unwrap_or(0) + 1;
        if (b as usize) < order.len() {
            order[b as usize] = next;
        }
    }
    tell(b, "You are infected!");
    hurt_sound(info, b);
    if std::env::var_os("TS_VIRUSLOG").is_some() {
        eprintln!("[virus] f{frame} character {a} infects character {b}");
    }
}

/// An infected computer character runs at the survivor it goes after when it is near (`enemyTick`, after the state of the character ran): its pad is not where the survivor is.
pub fn virus_chase(chr: i32) {
    if !SETUP.load(Ordering::Relaxed) || !virus_on() {
        return;
    }
    let mind = lw(chr + 352);
    if mind == 0 || lw(mind) < lw(g::lvnumplayers) || lw(mind + 24) != INFECTED || lw(mind + 2716) != 8 {
        return;
    }
    let (info, total) = characters();
    let me = [lwc1(chr + 48), lwc1(chr + 52), lwc1(chr + 56)];
    let mut best: Option<(f32, [f32; 3])> = None;
    for i in 0..total {
        let record = info + i * 0x1210;
        let prop = lw(record + 3020);
        if prop == 0 || lw(record + 24) != SURVIVORS || !alive(info, i) {
            continue;
        }
        let at = [lwc1(prop + 48), lwc1(prop + 52), lwc1(prop + 56)];
        let d = (at[0] - me[0]).powi(2) + (at[2] - me[2]).powi(2);
        if (at[1] - me[1]).abs() < 2.0 && best.map_or(true, |b| d < b.0) {
            best = Some((d, at));
        }
    }
    if let Some((d, at)) = best {
        if d < 15.0 * 15.0 {
            swc1(mind + 2796, at[0]);
            swc1(mind + 2800, at[1]);
            swc1(mind + 2804, at[2]);
        }
    }
}

/// Is `chr` (a character's prop) a computer character of a game of Virus?  None of them fights: they run (the survivors) or go after the survivors on foot (the infected).
pub fn virus_bot(chr: i32) -> bool {
    if !SETUP.load(Ordering::Relaxed) || !virus_on() {
        return false;
    }
    let mind = lw(chr + 352);
    mind != 0 && lw(mind) >= lw(g::lvnumplayers)
}

/// Is `chr` (a character's prop) a computer survivor of a game of Virus?  Survivors do not fight: they run from the infected.
pub fn virus_survivor_bot(chr: i32) -> bool {
    if !SETUP.load(Ordering::Relaxed) || !virus_on() {
        return false;
    }
    let mind = lw(chr + 352);
    mind != 0 && lw(mind) >= lw(g::lvnumplayers) && lw(mind + 24) == SURVIVORS
}

/// (the pad a survivor runs to, the frame it is chosen again) per character number
static FLEE: Mutex<Vec<(i32, i32)>> = Mutex::new(Vec::new());

/// `enemyGetTargetPad`: the pad a survivor bot heads for, one far from the nearest infected character and not far from the bot, chosen again every second or so.  None for anyone else.
pub fn virus_flee_pad(chr: i32) -> Option<i32> {
    if !virus_survivor_bot(chr) {
        return None;
    }
    let mind = lw(chr + 352);
    let number = lw(mind) as usize;
    let frame = ts_sdk::frame_count() as i32;
    let mut cache = FLEE.lock().unwrap();
    if cache.len() <= number {
        cache.resize(number + 1, (-1, 0));
    }
    if cache[number].0 >= 0 && frame < cache[number].1 {
        return Some(cache[number].0);
    }
    let (info, total) = characters();
    let me = [lwc1(chr + 48), lwc1(chr + 52), lwc1(chr + 56)];
    let mut infected: Vec<[f32; 3]> = Vec::new();
    for i in 0..total {
        let record = info + i * 0x1210;
        let prop = lw(record + 3020);
        if prop != 0 && lw(record + 24) == INFECTED && lw(record + 2716) & (16 | 32) == 0 {
            infected.push([lwc1(prop + 48), lwc1(prop + 52), lwc1(prop + 56)]);
        }
    }
    if infected.is_empty() {
        return None;
    }
    let dist = |a: [f32; 3], x: f32, y: f32, z: f32| ((a[0] - x).powi(2) + (a[1] - y).powi(2) + (a[2] - z).powi(2)).sqrt();
    let pads = lw(g::levelpads);
    let mut best = (-1, f32::MIN);
    for k in 0..lw(g::numpads) {
        let pad = pads + k * 28;
        let (x, y, z) = (lwc1(pad + 12), lwc1(pad + 16), lwc1(pad + 20));
        let away = infected.iter().map(|&p| dist(p, x, y, z)).fold(f32::MAX, f32::min);
        let score = away - 0.5 * dist(me, x, y, z);
        if score > best.1 {
            best = (k, score);
        }
    }
    if best.0 < 0 {
        return None;
    }
    cache[number] = (best.0, frame + 45);
    Some(best.0)
}

/// The sound of a character in pain (the game's sounds 207 to 210), where the character `i` is.
fn hurt_sound(info: i32, i: i32) {
    let prop = lw(info + i * 0x1210 + 3020);
    if prop != 0 {
        soundStart(207 + (newrnd() as u32 % 4) as i32, prop + 48);
    }
}

/// The score the table of results ranks the character of the record `record` by (`score` is the game's): in a game of Virus the survivors come first, then the infected, the last infected first.
pub fn virus_rank_score(record: i32, score: i32) -> i32 {
    if !SETUP.load(Ordering::Relaxed) || !virus_on() {
        return score;
    }
    let info = lw(g::chrinfo);
    let i = ((record - info) / 0x1210) as usize;
    if lw(record + 24) == SURVIVORS {
        return 100000;
    }
    1000 + ORDER.lock().unwrap().get(i).copied().unwrap_or(0).max(0)
}

/// The text the table of results shows in place of the score for the character of the record `record` in a game of Virus: its status.
pub fn virus_status_text(record: i32) -> Option<i32> {
    if !SETUP.load(Ordering::Relaxed) || !virus_on() {
        return None;
    }
    let mut at = 0x240_0000 + 0x17_3a00;
    let survivor = lw(record + 24) == SURVIVORS;
    Some(super::avmenu::put_str(&mut at, if survivor { "Survivor" } else { "Infected" }))
}

/// The edge of the screen of an infected player is green (`hudGfx`, where the game draws the ring that flashes red when a character is hurt).
pub fn virus_hud() {
    if !SETUP.load(Ordering::Relaxed) || !virus_on() {
        return;
    }
    let number = lw(lw(g::currentplayer));
    let info = lw(g::chrinfo);
    if info == 0 || lw(info + number * 0x1210 + 24) != INFECTED {
        return;
    }
    // (a slow pulse; the word is red, green, blue and alpha, from the top byte)
    let t = (ts_sdk::frame_count() as f32) * 0.05;
    let alpha = (85.0 + 30.0 * t.sin()) as i32;
    hudPainGfx(20 << 24 | 127 << 16 | 30 << 8 | alpha);
}

/// The colours that light the model `chr` (`partGfx`): the game's, or for a character that is infected in a game of Virus a green light.
pub fn virus_lighting(chr: i32) -> i32 {
    let own = lw(chr + 28);
    if own == 0 || !SETUP.load(Ordering::Relaxed) || !virus_on() {
        return own;
    }
    let owner = lw(chr + 244);
    if owner == 0 || lw(owner + 8) != 8 {
        return own;
    }
    let record = lw(owner + 352);
    if record == 0 || lw(record + 24) != INFECTED {
        return own;
    }
    let at = 0x240_0000 + 0x17_3c00;
    for k in 0..4 {
        let (r, g, b, a) = (lwc1(own + 16 * k), lwc1(own + 16 * k + 4), lwc1(own + 16 * k + 8), lwc1(own + 16 * k + 12));
        let top = r.max(g).max(b);
        swc1(at + 16 * k, r * 0.35);
        swc1(at + 16 * k + 4, g * 1.1 + top * 0.4);
        swc1(at + 16 * k + 8, b * 0.35);
        swc1(at + 16 * k + 12, a);
    }
    at
}

/// The kind of the table of results (`scores_pageGfx`): in a game of Virus every character has a row, not every team.
pub fn virus_table_kind(kind: i32) -> i32 {
    if SETUP.load(Ordering::Relaxed) && virus_on() {
        2
    } else {
        kind
    }
}
