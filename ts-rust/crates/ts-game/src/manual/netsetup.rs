//! What a network game is set up with: the same choices as a local arcade game, read from the game's own tables (the levels of the arcade page, the modes each level
//! has, the option rows of each mode, the weapon sets and the bot sets) and written to the game's own variables, so that the match is set up exactly as the arcade pages
//! would set it up.  The parameters of the match are then made by the game's own function (`frontmpgStartGame`, which fills in a block of 112 bytes from those variables)
//! and given to every machine, which starts the level with them.

use crate::*;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

/// The front end's settings of the arcade game (the level's place in the arcade list, its number, the mode).
const STATE: i32 = 0x1fc11f8;
/// The block of parameters the game makes (`gameDeferedReset` copies it).
const PARAMS: i32 = 0x1fc1898;

/// A text of the game in its language (`n` is the number of the text).
pub fn netsetup_text(n: i32) -> String {
    let language = lw(g::cur_language);
    let table = lw((language << 2) + g::languages);
    read_cstr(lw(table + 4 * n))
}

fn read_cstr(mut at: i32) -> String {
    let mut s = String::new();
    if at == 0 {
        return s;
    }
    while lbu(at) != 0 && s.len() < 60 {
        s.push(lbu(at) as u8 as char);
        at += 1;
    }
    s
}

/// One of the values a game variable can have, and how the game writes it.
#[derive(Clone, Debug)]
pub struct NetChoice {
    pub value: i32,
    pub text: String,
}

/// A row of the setup: a label and the values of the game variable it is made from.
#[derive(Clone, Debug)]
pub struct NetGvRow {
    pub label: String,
    pub gv: i32,
    pub choices: Vec<NetChoice>,
    pub index: usize,
}

/// The values of a game variable: all of them when there are few, otherwise a handful from the range (the game's pages let a number run up by holding a button).
fn gv_choices(gv: i32) -> Vec<NetChoice> {
    let (min, max) = (lw(gv + 4), lw(gv + 8));
    let values: Vec<i32> = if max - min <= 120 {
        (min..=max).collect()
    } else {
        let mut v = vec![min];
        v.extend([1, 2, 3, 4, 5, 10, 15, 20, 25, 30, 40, 50, 75, 100, 150, 200, 250, 500, 1000].into_iter().filter(|&x| x > min && x <= max));
        v
    };
    let old = lw(gv);
    let out = values
        .into_iter()
        .map(|v| {
            sw(gv, v);
            NetChoice { value: v, text: read_cstr(gvGetStringValue(gv)) }
        })
        .collect();
    sw(gv, old);
    out
}

fn gv_row(label: String, gv: i32) -> NetGvRow {
    let choices = gv_choices(gv);
    let value = lw(gv);
    // (a value that is not among those offered is shown as the nearest one below it)
    let index = choices.iter().rposition(|c| c.value <= value).unwrap_or(0);
    NetGvRow { label, gv, choices, index }
}

/// A game variable is set to a value, as the game's own pages do.
pub fn netsetup_gv_set(gv: i32, value: i32) {
    gvSetValue(gv, value);
}

/// The name of a mode of the game by its kind.
pub fn netsetup_mode_name(kind: i32) -> String {
    netsetup_text(lh(kind * 12 + g::gamemodedata))
}

/// The name of a level by its number.
pub fn netsetup_level_name(id: i32) -> String {
    let language = lw(g::cur_language);
    let mut entry = LVINFO;
    for _ in 0..200 {
        let text = lw((lh(entry) << 2) + lw((language << 2) + g::languages));
        if text == 0 {
            break;
        }
        if lw(entry + 4) == id {
            return ts_rt::cstr(text);
        }
        entry += 24;
    }
    format!("Level {id}")
}

/// The levels of the arcade page that this machine has open: (the number of the level, its name, its place in the arcade list).  The Map Maker's level is not among them.
pub fn netsetup_levels() -> Vec<(i32, String, i32)> {
    let count = av_arcade_count_or(19);
    let list = av_arcade_levels_or(g::arcadelevels);
    let mut v = Vec::new();
    for i in 0..count {
        let id = lw(list + 4 * i);
        if id == 100 || id <= 0 || levelAvail(id) == 0 {
            continue;
        }
        v.push((id, netsetup_level_name(id), i));
    }
    v
}

/// The level of the match is chosen (its place in the arcade list).
pub fn netsetup_select_level(index: i32) {
    let list = av_arcade_levels_or(g::arcadelevels);
    sw(STATE + 36, index);
    sw(STATE + 40, lw(list + 4 * index));
    allow_teams();
}

/// The teams and the team starts a level can be played with (the arcade page works them out when a level is chosen; the pages of the bots warn "Team not set in map" without).
fn allow_teams() {
    memset(STATE + 60, 1, 4);
    checkAllowedTeams();
}

/// The place in the arcade list of the level now chosen.
pub fn netsetup_level_index() -> i32 {
    lw(STATE + 36)
}

pub fn netsetup_level_id() -> i32 {
    lw(STATE + 40)
}

/// The modes the chosen level has for this number of players: (the kind of the mode, its name).
pub fn netsetup_modes(players: i32) -> Vec<(i32, String)> {
    let setup = lvGetARCADESetup(lw(STATE + 40));
    let mut v = Vec::new();
    if setup == 0 {
        return v;
    }
    for k in 0..6 {
        let kind = lbu(g::gmtab + 2 * k);
        let mask = lbu(g::gmtab + 2 * k + 1);
        // (Escort and Last Stand are for the bots and a team: not here)
        if !matches!(kind, 1 | 2 | 3 | 5) || players < 0 {
            continue;
        }
        if lw(setup) & mask != 0 {
            v.push((kind, netsetup_text(lh(kind * 12 + g::gamemodedata))));
        }
    }
    v
}

pub fn netsetup_select_mode(kind: i32) {
    sw(STATE + 48, kind);
    allow_teams();
}

pub fn netsetup_mode() -> i32 {
    lw(STATE + 48)
}

/// The option rows of the chosen mode, as the arcade page has them.
pub fn netsetup_mode_rows() -> Vec<NetGvRow> {
    let (table, count) = match lw(STATE + 48) {
        1 => (g::bagtag_menuitems, 8),
        2 => (g::ctf_menuitems, 6),
        3 => (g::deathmatch_menuitems, 10),
        5 => (g::knockout_menuitems, 6),
        _ => return Vec::new(),
    };
    (0..count)
        .filter_map(|i| {
            let it = table + 32 * i;
            let gv = lw(it + 12);
            (gv > 0x10000).then(|| gv_row(netsetup_text(lw(it)), gv))
        })
        .collect()
}

/// The weapon sets that can be chosen (the presets that are open, and the custom sets), as the arcade page lists them.
pub fn netsetup_weapon_row() -> NetGvRow {
    // (the page's list of names is made on the first ask)
    let _ = weaponset_effective();
    let gv = g::gv_weaponset;
    let mut row = gv_row("Weapon Set".into(), gv);
    row.choices.retain(|c| c.value == 0 || c.value >= 10 || weaponsetAvail(c.value - 1) != 0);
    row.index = row.choices.iter().position(|c| c.value == lw(gv)).unwrap_or(0);
    row
}

/// The five weapons of the custom weapon set now chosen (the set is in the game's place for it once `weaponset_effective` has been asked).
pub fn netsetup_weapon_pack() -> [i32; 5] {
    let _ = weaponset_effective();
    let mut a = [0; 5];
    for (k, v) in a.iter_mut().enumerate() {
        *v = lw(g::gunzpack + 4 * k as i32);
    }
    a
}

pub fn netsetup_weapon_set(value: i32) {
    sw(g::gv_weaponset, value);
    let _ = weaponset_effective();
}

/// The bot sets that can be chosen.
pub fn netsetup_botset_row() -> NetGvRow {
    let _ = chrset_effective();
    let gv = g::gv_chrset;
    let mut row = gv_row("Bot Set".into(), gv);
    row.choices.retain(|c| c.value >= 8 || botsetAvail(c.value) != 0);
    row.index = row.choices.iter().position(|c| c.value == lw(gv)).unwrap_or(0);
    row
}

pub fn netsetup_bot_set(value: i32) {
    sw(g::gv_chrset, value);
    let _ = chrset_effective();
}

/// The number of bots of the match.
static BOTS: AtomicI32 = AtomicI32::new(0);

/// The bots that will play: those of the bot set that the Setup Bots page has ticked.
pub fn netsetup_bots() -> i32 {
    (0..10).filter(|i| lw(0x1fc2a88 + 16 * i) >= 0 && lw(0x1fc2a88 + 16 * i + 12) != 0).count() as i32
}

pub fn netsetup_set_bots(n: i32) {
    BOTS.store(n.clamp(0, 8), Ordering::Relaxed);
}

/// A new setup: the choices are those the game has now, the level is one that is open, the mode one the level has.
pub fn netsetup_begin() {
    let levels = netsetup_levels();
    if !levels.iter().any(|l| l.0 == lw(STATE + 40)) {
        if let Some(l) = levels.first() {
            netsetup_select_level(l.2);
        }
    }
    let modes = netsetup_modes(2);
    if !modes.iter().any(|m| m.0 == lw(STATE + 48)) {
        if let Some(m) = modes.iter().find(|m| m.0 == 3).or(modes.first()) {
            netsetup_select_mode(m.0);
        }
    }
    allow_teams();
    BOTS.store(0, Ordering::Relaxed);
}

// ---- the parameters of the match -----------------------------------------------------------------------------------------------------------------------

static CAPTURE: AtomicBool = AtomicBool::new(false);
static CAPTURED: Mutex<Option<Vec<i32>>> = Mutex::new(None);

/// `gameDeferedReset` was asked to start a game while the parameters are only wanted: they are kept, and the game does not start.
pub fn netsetup_capture(block: i32) -> bool {
    if !CAPTURE.load(Ordering::Relaxed) {
        return false;
    }
    *CAPTURED.lock().unwrap() = Some((0..28).map(|i| lw(block + 4 * i)).collect());
    true
}

/// The parameters of the match the game would start with the choices made, for this number of players (their profiles must be made: the game reads their characters).
pub fn netsetup_block(players: i32) -> Option<Vec<i32>> {
    let front_state = lw(STATE);
    // (the players that are not here yet have the first player's profile while the game reads them)
    let profiles: Vec<i32> = (0..4).map(signonGet).collect();
    for i in 1..4 {
        if i >= players || profiles[i as usize] < 0 {
            signonSet(i, profiles[0]);
        }
    }
    let kind = lhu(g::gametype_page + 8);
    sw(0x3aff20, players);
    sh(g::gametype_page + 8, 2);
    // (the teams of a team game: the players take turns, as do the bots; both teams are in use)
    for i in 0..4 {
        sw(0x1fc2c88 + 4 * i, i & 1);
    }
    sb(0x1fc1234, 1);
    sb(0x1fc1234 + 1, 1);
    // the bots: the characters of the set, the first of them as many as the match has
    botsetInit(chrset_read());
    // (which bots play, and their teams, are what the Setup Bots page has them: they stay as they are)
    // (the game stores what this machine has unlocked while it makes the parameters; the machines of a match must not differ in it, so it is given back)
    let kept: Vec<(i32, i32)> = (0..4).map(|i| 0x3affa0 + 4 * i).chain((0..8).map(|i| 0x1fc33b0 + 4 * i)).map(|a| (a, lw(a))).collect();
    if std::env::var_os("TS_NETLOG").is_some() {
        eprintln!("[setup] bots {} chrset {} slots {:?} teams {:?}", BOTS.load(Ordering::Relaxed), chrset_read(), (0..10).map(|i| (lw(0x1fc2a88 + 16 * i), lw(0x1fc2a88 + 16 * i + 4), lw(0x1fc2a88 + 16 * i + 12))).collect::<Vec<_>>(), (0..6).map(|i| lbu(0x1fc1234 + i)).collect::<Vec<_>>());
    }
    *CAPTURED.lock().unwrap() = None;
    CAPTURE.store(true, Ordering::Relaxed);
    frontmpgStartGame();
    CAPTURE.store(false, Ordering::Relaxed);
    for (a, v) in kept {
        sw(a, v);
    }
    sw(STATE, front_state);
    sh(g::gametype_page + 8, kind);
    for (i, id) in profiles.iter().enumerate() {
        signonSet(i as i32, *id);
    }
    let mut block = CAPTURED.lock().unwrap().take()?;
    // (the records of the mode's options, which the level reads, are made from the variables as they are made on the other machines)
    if (1..=6).contains(&block[18]) {
        storeGameModeOptions(block[18]);
    }
    // (the players of the block are all of them)
    block[0] = players;
    block[1] = players;
    Some(block)
}

/// The name of the profile signed in on this machine (the name the network game goes by).
pub fn netsetup_profile_name() -> Option<String> {
    let id = signonGet(0);
    if !(0..144).contains(&id) {
        return None;
    }
    let name = read_cstr(lw(g::signonarray) + id * 2936);
    (!name.is_empty()).then_some(name)
}

/// What a game variable is called on the pages, and how a value of it reads: (the label, the text of the value).
pub fn netsetup_gv_describe(gv: i32, value: i32) -> (String, String) {
    let label = if gv == g::gv_weaponset {
        "Weapon Set".to_string()
    } else if gv == g::gv_chrset {
        "Bot Set".to_string()
    } else {
        [(g::bagtag_menuitems, 8), (g::ctf_menuitems, 6), (g::deathmatch_menuitems, 10), (g::knockout_menuitems, 6)]
            .into_iter()
            .flat_map(|(t, n)| (0..n).map(move |i| t + 32 * i))
            .find(|&it| lw(it + 12) == gv)
            .map_or_else(|| "?".to_string(), |it| netsetup_text(lw(it)))
    };
    let old = lw(gv);
    sw(gv, value);
    let text = read_cstr(gvGetStringValue(gv));
    sw(gv, old);
    (label, text)
}

/// Every game variable the level's setup reads (the page's rows, and the ones a mode has no row for): all of them are the host's on every machine.
fn all_gvs() -> [i32; 14] {
    [
        g::gv_timelimit, g::gv_scorelimit, g::gv_killlimit, g::gv_scoremethod, g::gv_teamplay, g::gv_useteamstarts, g::gv_friendfire, g::gv_bagshoot, g::gv_dispradar,
        g::gv_oneshotkill, g::gv_handicaps, g::gv_spawngun, g::gv_weaponset, g::gv_chrset,
    ]
}

/// The game variables with their values: what every machine sets before it starts the match.
pub fn netsetup_gv_values() -> Vec<(i32, i32)> {
    all_gvs().into_iter().map(|gv| (gv, lw(gv))).collect()
}

/// The values the host chose are set in this machine's variables (before the parameters are used), and the records of the mode's options the level reads are made from them.
pub fn netsetup_apply(values: &[(i32, i32)], mode: i32) {
    for &(gv, value) in values {
        // (only the variables of the pages: an address from the network is not written to blindly)
        if all_gvs().contains(&gv) {
            sw(gv, value);
        }
    }
    if (1..=6).contains(&mode) {
        storeGameModeOptions(mode);
    }
}

/// A summary of a block of parameters, for the page of a game that was joined: (the level, the mode, the time limit in minutes, the limit of kills or of the score, the bots).
pub fn netsetup_block_summary(block: &[i32]) -> (String, String, i32, i32, i32) {
    if block.len() < 28 {
        return (String::new(), String::new(), 0, 0, 0);
    }
    let level = netsetup_level_name(block[16]);
    let mode = netsetup_text(lh(block[18] * 12 + g::gamemodedata));
    (level, mode, block[22] / 3600, block[23].max(block[24]), block[2])
}

/// A test aid (`TS_NETDUMP2=1`): the tables the pages are made from.
pub fn netsetup_dump() {
    if std::env::var_os("TS_NETDUMP2").is_none() {
        return;
    }
    eprintln!("[setup] levels {:?}", netsetup_levels());
    netsetup_begin();
    eprintln!("[setup] level {} mode {} modes {:?}", netsetup_level_id(), netsetup_mode(), netsetup_modes(2));
    for r in netsetup_mode_rows() {
        eprintln!("[setup] row {:?} {} choices {}: {:?}", r.label, r.index, r.choices.len(), r.choices.iter().take(6).map(|c| c.text.clone()).collect::<Vec<_>>());
    }
    for r in [netsetup_weapon_row(), netsetup_botset_row()] {
        eprintln!("[setup] row {:?} {} choices {:?}", r.label, r.index, r.choices.iter().map(|c| (c.value, c.text.clone())).collect::<Vec<_>>());
    }
}
