//! The scripts of mods: a file of a mod in the Rhai language that runs while the game is played.  There are two kinds:
//!
//! * the script of a mode (`script virus.rhai` in a `variant` of `modes.txt`): it runs while that mode is played;
//! * the script of a mod (`script = bots_punch.rhai` in `mod.ini`): it runs all the time the mod is on, in every match and (`on_frame`) in the menus too.
//!
//! The engine has no knowledge of what a script does: it gives it a small set of things to ask and do (the characters and where they are, their sides, messages and sounds, how the computer
//! characters move, damage, a flame, a tint, a glow at the edge of the screen, what the table of results says, the words of the game's memory) and calls it at a few places.  A script is read
//! when the game starts (a mod's) or when the match starts (a mode's); one that has an error is told in the log and stops.
//!
//! The functions a script may define (all are optional): `on_frame()`, `on_start()`, `on_tick()`, `on_hud(player)`, `on_bot_idle(i)`, `file_prefixes()` with `on_file(name)`, `on_gun_model(name, left)`,
//! `lists_pair(single)`, `allows_pair(pair, single)`, `alternate_fists()`, `results_per_character()`, `result_key(i)` and `result_text(i)`.  `docs/scripting.md` lists them and what the script can call.

use crate::*;
use rhai::{Engine, Scope, AST};
use std::cell::RefCell;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

const BASE: i32 = 0x240_0000;
const MESSAGE_AT: i32 = BASE + 0x17_2c00;
const TEXT_AT: i32 = BASE + 0x17_3a00;
const LIGHT_AT: i32 = BASE + 0x17_3c00;
const FLAME_AT: i32 = BASE + 0x17_2d00;
/// the place, source and direction of a blow (three floats each, 16 bytes apart)
const HIT_AT: i32 = BASE + 0x17_3d00;

/// What a script has asked for, for each character.
#[derive(Clone)]
struct Fx {
    flame: bool,
    /// (red, green and blue as parts of the game's light, and a part of the strongest colour added to green)
    tint: Option<[f32; 4]>,
    /// the character does not attack (a computer character)
    passive: bool,
    /// a part of the character's own top speed (a computer character that goes to a pad)
    speed: f32,
    /// the pad a computer character goes to (-1: the game's choice)
    pad: i32,
    /// a place a computer character runs at when it is going to a pad
    steer: Option<[f32; 3]>,
    /// the group of particles of the flame
    flame_group: i32,
    /// the colour of the character's dot on the radar (the game's colour word of a team)
    radar: Option<i32>,
}

impl Default for Fx {
    fn default() -> Self {
        Fx { flame: false, tint: None, passive: false, speed: 1.0, pad: -1, steer: None, flame_group: 0, radar: None }
    }
}

struct State {
    fx: Vec<Fx>,
    started: bool,
    last_game_frame: i32,
    in_hud: bool,
    dt: f32,
    /// the score and the rank are left off the screen
    hide_score: bool,
}

struct Host {
    engine: Engine,
    ast: AST,
    scope: Scope<'static>,
    functions: HashSet<String>,
    name: String,
    /// what the script keeps between calls: a map, which its functions see as `this`
    state: rhai::Dynamic,
    /// the script of a mode (put away when a game starts), not of a mod
    of_mode: bool,
    /// the start of the names of the files that `on_file` is asked about (`file_prefixes()`)
    prefixes: Vec<String>,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State { fx: Vec::new(), started: false, last_game_frame: 0, in_hud: false, dt: 1.0, hide_score: false });
    static HOSTS: RefCell<Vec<Host>> = RefCell::new(Vec::new());
}

/// The scripts that mods have given (the mod, the file, the text), read by the loader when the game starts.
static MOD_SCRIPTS: Mutex<Vec<(String, String, String)>> = Mutex::new(Vec::new());

/// A mod has a script (`script = file` in its `mod.ini`).
pub fn register_script(mod_name: &str, file: &str, source: String) {
    MOD_SCRIPTS.lock().unwrap().push((mod_name.to_string(), file.to_string(), source));
}

/// A script is running (set while one is loaded, so that the hooks in the game's code cost nothing otherwise).
static ACTIVE: AtomicBool = AtomicBool::new(false);
/// How much a script has scaled the height of the view of each character (`chr_view_ratio`; 0: not at all).
static VIEW_RATIOS: [std::sync::atomic::AtomicU32; 128] = [const { std::sync::atomic::AtomicU32::new(0) }; 128];
/// The script of the mode chosen was looked for in this game, and those of the mods in this run.
static TRIED: AtomicBool = AtomicBool::new(false);
static MODS_TRIED: AtomicBool = AtomicBool::new(false);

fn log_on() -> bool {
    std::env::var_os("TS_SCRIPTLOG").is_some()
}

fn characters() -> (i32, i32) {
    (lw(g::chrinfo), lw(g::lvnumplayers) + lw(g::numactivechars))
}

fn record(i: i64) -> i32 {
    lw(g::chrinfo) + (i as i32) * 0x1210
}

fn prop_of(i: i64) -> i32 {
    let (info, total) = characters();
    if info == 0 || i < 0 || i as i32 >= total {
        0
    } else {
        lw(record(i) + 3020)
    }
}

fn fx_of<R>(i: i64, f: impl FnOnce(&mut Fx) -> R) -> Option<R> {
    if i < 0 || i > 64 {
        return None;
    }
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        if s.fx.len() <= i as usize {
            s.fx.resize(i as usize + 1, Fx::default());
        }
        Some(f(&mut s.fx[i as usize]))
    })
}

fn fx_read<R>(i: i32, f: impl FnOnce(&Fx) -> R) -> Option<R> {
    if i < 0 {
        return None;
    }
    STATE.with(|s| s.borrow().fx.get(i as usize).map(f))
}

fn is_alive(i: i64) -> bool {
    let (info, total) = characters();
    if info == 0 || i < 0 || i as i32 >= total {
        return false;
    }
    if (i as i32) < lw(g::lvnumplayers) {
        lw(lw(g::players) + (i as i32) * 1820 + 296) != 4
    } else {
        lw(record(i) + 2716) & (16 | 32) == 0
    }
}

fn is_player_index(i: i64) -> bool {
    i >= 0 && (i as i32) < lw(g::lvnumplayers)
}

fn put_text(at: i32, text: &str, most: usize) {
    for (k, b) in text.bytes().take(most).enumerate() {
        sb(at + k as i32, b as i32);
    }
    sb(at + text.len().min(most) as i32, 0);
}

fn register(engine: &mut Engine) {
    // the characters: numbered from 0, the players first
    engine.register_fn("chars", || characters().1 as i64);
    engine.register_fn("players", || lw(g::lvnumplayers) as i64);
    engine.register_fn("is_player", |i: i64| is_player_index(i));
    engine.register_fn("alive", |i: i64| is_alive(i));
    engine.register_fn("team", |i: i64| if prop_of(i) == 0 { 0 } else { lw(record(i) + 24) as i64 });
    engine.register_fn("set_team", |i: i64, t: i64| {
        if prop_of(i) != 0 {
            sw(record(i) + 24, t as i32);
        }
    });
    engine.register_fn("x", |i: i64| match prop_of(i) { 0 => 0.0, p => lwc1(p + 48) as f64 });
    engine.register_fn("y", |i: i64| match prop_of(i) { 0 => 0.0, p => lwc1(p + 52) as f64 });
    engine.register_fn("z", |i: i64| match prop_of(i) { 0 => 0.0, p => lwc1(p + 56) as f64 });
    engine.register_fn("score", |i: i64| if prop_of(i) == 0 { 0 } else { lw(record(i) + 248) as i64 });
    engine.register_fn("set_score", |i: i64, v: i64| {
        if prop_of(i) != 0 {
            sw(record(i) + 248, v as i32);
        }
    });
    // a computer character's state in the game's own machine of states (8: going to a pad, 64: looking for a weapon, 128: choosing a target)
    engine.register_fn("bot_state", |i: i64| if prop_of(i) == 0 || is_player_index(i) { 0 } else { lw(record(i) + 2716) as i64 });
    engine.register_fn("bot_set_state", |i: i64, state: i64| {
        if prop_of(i) != 0 && !is_player_index(i) {
            let r = record(i);
            sw(r + 2720, lw(r + 2716));
            sw(r + 2716, state as i32);
            sw(r + 2892, 0);
        }
    });
    // a computer character with no weapon in either hand
    engine.register_fn("bot_unarmed", |i: i64| prop_of(i) != 0 && !is_player_index(i) && lw(record(i) + 8) == 0 && lw(record(i) + 376) == -1 && lw(record(i) + 260) == 0);
    // the character a computer character goes after (-1: none)
    engine.register_fn("bot_target", |i: i64| {
        if prop_of(i) == 0 || is_player_index(i) {
            return -1;
        }
        let target = lw(record(i) + 2788);
        if target == 0 || lw(target + 8) != 8 {
            return -1;
        }
        let rec = lw(target + 352);
        if rec == 0 {
            -1
        } else {
            lw(rec) as i64
        }
    });
    // the character of the player `p` (the number in the game's table of characters; -1 before the player has chosen)
    engine.register_fn("player_character", |p: i64| {
        let id = signonGet(p as i32);
        if id < 0 {
            return -1;
        }
        lw(lw(g::signonarray) + id * 2936 + 32 + 84) as i64
    });
    // time and chance
    engine.register_fn("dt", || STATE.with(|s| s.borrow().dt) as f64);
    engine.register_fn("rand_int", |n: i64| if n <= 0 { 0 } else { ((newrnd() as u32) as i64) % n });
    engine.register_fn("env_int", |name: &str, default: i64| std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default));
    engine.register_fn("log", |text: &str| {
        if log_on() {
            eprintln!("[script] f{} {text}", ts_sdk::frame_count());
        }
    });
    // the words of the game's memory (for what the other functions do not reach)
    engine.register_fn("peek", |at: i64| lw(at as i32) as i64);
    engine.register_fn("poke", |at: i64, value: i64| sw(at as i32, value as i32));
    engine.register_fn("peekf", |at: i64| lwc1(at as i32) as f64);
    engine.register_fn("pokef", |at: i64, value: f64| swc1(at as i32, value as f32));
    // the entry of a character in the game's table of characters (44 bytes): +20 the height of its head standing, +24 crouching, +28 the floor, +32 and +36 sizes of its body, +40 its scale
    engine.register_fn("chr_entry", |c: i64| (CHRS_TABLE + 44 * (c as i32)) as i64);
    // (the script has scaled the heights of the view of the character by so much: the arm of the fists stays where it is in the view)
    engine.register_fn("chr_view_ratio", |c: i64, ratio: f64| {
        if let Some(r) = usize::try_from(c).ok().and_then(|c| VIEW_RATIOS.get(c)) {
            r.store((ratio as f32).to_bits(), Ordering::Relaxed);
        }
    });
    // the pads of the level (the places the computer characters go between)
    engine.register_fn("pad_count", || lw(g::numpads) as i64);
    engine.register_fn("pad_x", |k: i64| lwc1(lw(g::levelpads) + (k as i32) * 28 + 12) as f64);
    engine.register_fn("pad_y", |k: i64| lwc1(lw(g::levelpads) + (k as i32) * 28 + 16) as f64);
    engine.register_fn("pad_z", |k: i64| lwc1(lw(g::levelpads) + (k as i32) * 28 + 20) as f64);
    // messages, sounds, blows and the end of the match
    engine.register_fn("message", |i: i64, text: &str| {
        if (0..lw(g::lvnumplayers) as i64).contains(&i) {
            put_text(MESSAGE_AT, text, 60);
            hudMessage(i as i32, MESSAGE_AT);
        }
    });
    engine.register_fn("sound", |id: i64, i: i64| {
        let p = prop_of(i);
        if p != 0 {
            soundStart(id as i32, p + 48);
        }
    });
    engine.register_fn("pain_voice", |i: i64| pain_voice(i));
    // a blow of the character `attacker` on the character `target` (of the game's damage, a fraction of a character's health: a pistol shot is 0.3)
    engine.register_fn("damage", |target: i64, attacker: i64, amount: f64| {
        let (t, a) = (prop_of(target), prop_of(attacker));
        if t == 0 || a == 0 {
            return;
        }
        let d = [lwc1(t + 48) - lwc1(a + 48), lwc1(t + 52) - lwc1(a + 52), lwc1(t + 56) - lwc1(a + 56)];
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1.0e-3);
        for k in 0..3 {
            swc1(HIT_AT + 4 * k, lwc1(t + 48 + 4 * k)); // where it hits
            swc1(HIT_AT + 16 + 4 * k, lwc1(a + 48 + 4 * k)); // where it comes from
            swc1(HIT_AT + 32 + 4 * k, d[k as usize] / len); // the way it goes
        }
        propDamage(t, a, HIT_AT, 1, HIT_AT + 16, HIT_AT + 32, 0, 0, amount as f32);
    });
    engine.register_fn("finish", || {
        if lw(g::gamestate) != 4 {
            gameLevelCompleted();
        }
    });
    // how computer characters behave
    engine.register_fn("bot_passive", |i: i64, on: bool| {
        fx_of(i, |f| f.passive = on);
    });
    engine.register_fn("bot_speed", |i: i64, s: f64| {
        fx_of(i, |f| f.speed = s as f32);
    });
    engine.register_fn("bot_pad", |i: i64, k: i64| {
        fx_of(i, |f| f.pad = k as i32);
    });
    engine.register_fn("bot_steer", |i: i64, x: f64, y: f64, z: f64| {
        fx_of(i, |f| f.steer = Some([x as f32, y as f32, z as f32]));
    });
    engine.register_fn("bot_steer_clear", |i: i64| {
        fx_of(i, |f| f.steer = None);
    });
    // how characters look
    engine.register_fn("flame", |i: i64, on: bool| {
        fx_of(i, |f| f.flame = on);
    });
    engine.register_fn("tint", |i: i64, r: f64, g: f64, b: f64, add: f64| {
        fx_of(i, |f| f.tint = Some([r as f32, g as f32, b as f32, add as f32]));
    });
    engine.register_fn("tint_clear", |i: i64| {
        fx_of(i, |f| f.tint = None);
    });
    // the colour a character has on the radar and in the table of results: that of a team of the game (1 red, 2 blue, 3 yellow, 4 green)
    engine.register_fn("set_colour", |i: i64, team: i64| {
        if prop_of(i) != 0 {
            sw(record(i) + 20, lw(((team as i32) << 3) + g::storydata + 68));
        }
    });
    // the colour of a character's dot on the radar, that of a team of the game (1 red, 2 blue, 3 yellow, 4 green); the game uses one colour for every character when it is not a team game
    engine.register_fn("radar_colour", |i: i64, team: i64| {
        let colour = lw(((team as i32) << 3) + g::storydata + 68);
        fx_of(i, |f| f.radar = Some(colour));
    });
    engine.register_fn("radar_colour_clear", |i: i64| {
        fx_of(i, |f| f.radar = None);
    });
    // the score and the rank of the player are not shown on the screen (a mode with no use for them)
    engine.register_fn("hide_score", |on: bool| STATE.with(|s| s.borrow_mut().hide_score = on));
    // the glow at the edge of the screen (only in `on_hud`): red, green, blue and alpha of 0 to 127
    engine.register_fn("hud_glow", |r: i64, g: i64, b: i64, a: i64| {
        if STATE.with(|s| s.borrow().in_hud) {
            hudPainGfx((r as i32 & 255) << 24 | (g as i32 & 255) << 16 | (b as i32 & 255) << 8 | (a as i32 & 255));
        }
    });
}

/// A character says one of its lines of being hit: the lines the game gives each character in its table of characters (the ones it plays when it is hit), where the character is.
fn pain_voice(i: i64) {
    let p = prop_of(i);
    if p == 0 {
        return;
    }
    let number = lh(record(i) + 4);
    let list = lw(CHRS_TABLE + 44 * number + 4);
    if list == 0 {
        return;
    }
    let mut sounds = Vec::new();
    for k in 0..18 {
        let id = lhu(list + 2 * k);
        if id == 0 {
            break;
        }
        sounds.push(id);
    }
    if sounds.is_empty() {
        return;
    }
    // (as the game plays it when a character is hit: the voice is loaded when it is wanted and starts a few ticks later)
    soundDelayStartEx(sounds[(newrnd() as u32 as usize) % sounds.len()], p + 48, 20, 0);
}

fn compile(name: &str, source: &str, of_mode: bool) -> Option<Host> {
    let mut engine = Engine::new();
    engine.set_max_operations(5_000_000);
    engine.set_max_expr_depths(256, 128);
    register(&mut engine);
    match engine.compile(source) {
        Ok(ast) => {
            let functions = ast.iter_functions().map(|f| f.name.to_string()).collect();
            let mut host = Host { engine, ast, scope: Scope::new(), functions, name: name.to_string(), state: rhai::Dynamic::from_map(rhai::Map::new()), of_mode, prefixes: Vec::new() };
            if host.functions.contains("file_prefixes") {
                let Host { engine, ast, scope, state, .. } = &mut host;
                let options = rhai::CallFnOptions::new().eval_ast(false).rewind_scope(false).bind_this_ptr(state);
                let list = engine.call_fn_with_options::<rhai::Array>(options, scope, ast, "file_prefixes", ());
                match list {
                    Ok(list) => host.prefixes = list.into_iter().filter_map(|d| d.into_string().ok()).collect(),
                    Err(e) => eprintln!("[script] {name}: file_prefixes: {e}"),
                }
            }
            Some(host)
        }
        Err(e) => {
            eprintln!("[script] {name}: {e}");
            None
        }
    }
}

/// A new game starts: the script of the last mode and what the scripts asked for are put away.
pub fn script_reset() {
    HOSTS.with(|h| h.borrow_mut().retain(|host| !host.of_mode));
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        s.fx.clear();
        s.started = false;
        s.last_game_frame = 0;
        s.hide_score = false;
    });
    TRIED.store(false, Ordering::Relaxed);
    refresh_active();
}

fn refresh_active() {
    ACTIVE.store(HOSTS.with(|h| !h.borrow().is_empty()), Ordering::Relaxed);
}

/// Read the scripts of the mods (once) and the script of the mode being played (once for each game).
fn load() {
    if !MODS_TRIED.swap(true, Ordering::Relaxed) {
        let scripts = MOD_SCRIPTS.lock().unwrap().clone();
        for (m, file, source) in scripts {
            if let Some(host) = compile(&format!("{m}/{file}"), &source, false) {
                eprintln!("[script] {m}/{file}");
                HOSTS.with(|h| h.borrow_mut().push(host));
            }
        }
    }
    if lw(g::gamestate) == 3 && !TRIED.swap(true, Ordering::Relaxed) {
        if let Some((name, source)) = active_script(lw(g::curgameparms + 72)) {
            if let Some(host) = compile(&name, &source, true) {
                eprintln!("[script] a script is running");
                HOSTS.with(|h| h.borrow_mut().push(host));
            }
        }
    }
    refresh_active();
}

/// Call the function `name` of every script that has it: the answers come back in a list.  A script that fails is put away.
fn call_all<A: rhai::FuncArgs + Clone, T: Clone + std::any::Any>(name: &str, args: A) -> Vec<T> {
    let mut out = Vec::new();
    let mut failed: Vec<String> = Vec::new();
    HOSTS.with(|h| {
        // (a script that is running and makes the game call it again, by a file it loads, is not asked twice)
        let Ok(mut hosts) = h.try_borrow_mut() else {
            return;
        };
        for host in hosts.iter_mut() {
            if !host.functions.contains(name) {
                continue;
            }
            let Host { engine, ast, scope, name: script, state, .. } = host;
            let options = rhai::CallFnOptions::new().eval_ast(false).rewind_scope(false).bind_this_ptr(state);
            match engine.call_fn_with_options::<rhai::Dynamic>(options, scope, ast, name, args.clone()) {
                Ok(v) => match v.try_cast::<T>() {
                    Some(v) => out.push(v),
                    None => eprintln!("[script] {script}: {name}: the answer is not of the kind the game asks for"),
                },
                Err(e) => {
                    eprintln!("[script] {script}: {name}: {e}");
                    failed.push(script.clone());
                }
            }
        }
    });
    if !failed.is_empty() {
        HOSTS.with(|h| h.borrow_mut().retain(|host| !failed.contains(&host.name)));
        refresh_active();
    }
    out
}

fn has(name: &str) -> bool {
    HOSTS.with(|h| h.borrow().iter().any(|host| host.functions.contains(name)))
}

/// Every frame the game is run, in the menus too (`on_frame` of the scripts).
pub fn script_frame() {
    load();
    if ACTIVE.load(Ordering::Relaxed) {
        let _: Vec<()> = call_all("on_frame", ());
        FISTS.store(call_all::<_, bool>("alternate_fists", ()).into_iter().any(|b| b), Ordering::Relaxed);
    }
}

/// Every game frame of a match (`custom_rules_tick`).
pub fn script_tick() {
    if lw(g::gamestate) != 3 {
        return;
    }
    load();
    if !ACTIVE.load(Ordering::Relaxed) {
        return;
    }
    let (info, total) = characters();
    if info == 0 || total < 2 || lw(g::lvnumplayers) <= 0 {
        return;
    }
    // (the level was started again from the menu of the game or the results: the scripts start again)
    let game_frame = lw(g::gameframe);
    STATE.with(|s| {
        let mut s = s.borrow_mut();
        if game_frame < s.last_game_frame {
            s.fx.clear();
            s.started = false;
        }
        s.last_game_frame = game_frame;
        s.dt = lwc1(g::updateframesf).clamp(0.0, 4.0);
    });
    let started = STATE.with(|s| s.borrow().started);
    if !started {
        STATE.with(|s| s.borrow_mut().started = true);
        let _: Vec<()> = call_all("on_start", ());
        return;
    }
    let _: Vec<()> = call_all("on_tick", ());
    flames(info, total);
}

/// The game's hook of the screen of the player being drawn (`hudGfx`): a script may draw a glow.
pub fn script_hud() {
    if !ACTIVE.load(Ordering::Relaxed) || !has("on_hud") {
        return;
    }
    let number = lw(lw(g::currentplayer)) as i64;
    STATE.with(|s| s.borrow_mut().in_hud = true);
    let _: Vec<()> = call_all("on_hud", (number,));
    STATE.with(|s| s.borrow_mut().in_hud = false);
}

/// `findweapon` found nothing for the computer character of the record `record` to pick up: a script may give it something else to do (it says so by answering true).
pub fn script_bot_idle(record: i32) -> bool {
    if !ACTIVE.load(Ordering::Relaxed) || !has("on_bot_idle") {
        return false;
    }
    let i = ((record - lw(g::chrinfo)) / 0x1210) as i64;
    call_all::<_, bool>("on_bot_idle", (i,)).into_iter().any(|b| b)
}

/// The weapon page may list the dual (x2) version of the weapon `single` (a script says so with `lists_pair(single)`; with none the pairs the game does not have are not listed).
pub fn script_pair_listed(single: i32) -> bool {
    ACTIVE.load(Ordering::Relaxed) && call_all::<_, bool>("lists_pair", (single as i64,)).into_iter().any(|b| b)
}

/// A set of weapons may have the pair `pair` of the weapon `single` (`allows_pair(pair, single)` of a script; otherwise the set has the single weapon).
pub fn script_pair_allowed(pair: i32, single: i32) -> bool {
    ACTIVE.load(Ordering::Relaxed) && call_all::<_, bool>("allows_pair", (pair as i64, single as i64)).into_iter().any(|b| b)
}

/// The height of the eyes that the arm of the fists is placed from, for a character whose view a script has moved (`chr_view_ratio`): the height it has without the script, so that the
/// arm is where it always is in the view.
pub fn script_arm_eye(character: i32, eye: f32) -> f32 {
    let Some(ratio) = usize::try_from(character).ok().and_then(|c| VIEW_RATIOS.get(c)).map(|r| f32::from_bits(r.load(Ordering::Relaxed))) else {
        return eye;
    };
    if ratio <= 0.0 {
        return eye;
    }
    let floor = lwc1(CHRS_TABLE + 44 * character + 28);
    floor + (eye - floor) / ratio
}

/// An unarmed player punches with the left fist and the right in turn (`alternate_fists()` of a script).
pub fn script_alternate_fists() -> bool {
    ACTIVE.load(Ordering::Relaxed) && FISTS.load(Ordering::Relaxed)
}

/// What `alternate_fists()` said at the start of the frame (the picture of a fist asks every frame).
static FISTS: AtomicBool = AtomicBool::new(false);

/// The colour of the dot of the character `i` on the radar (`hudRadarGfx`), if a script gave one.
pub fn script_radar_colour(i: i32) -> Option<i32> {
    if !ACTIVE.load(Ordering::Relaxed) {
        return None;
    }
    fx_read(i, |f| f.radar).flatten()
}

/// The screen leaves out the score and the rank (`hudGfx`).
pub fn script_hides_score() -> bool {
    ACTIVE.load(Ordering::Relaxed) && STATE.with(|s| s.borrow().hide_score)
}

/// A computer character of the game being scripted does not attack.
pub fn script_bot_passive(chr: i32) -> bool {
    if !ACTIVE.load(Ordering::Relaxed) {
        return false;
    }
    let mind = lw(chr + 352);
    mind != 0 && fx_read(lw(mind), |f| f.passive).unwrap_or(false)
}

/// A computer character that a script holds back (`enemyTick`, before its state runs): whatever it was going to do (attack, look for a weapon or health, defend) it picks its way again, so that it only goes to the pads it is sent to.
pub fn script_bot_hold(chr: i32) {
    if !script_bot_passive(chr) {
        return;
    }
    let mind = lw(chr + 352);
    let state = lw(mind + 2716);
    // (0: not in play, 8: going to a pad, 16: dying, 32: spawning, 128: choosing a target)
    if !matches!(state, 0 | 8 | 16 | 32 | 128) {
        sw(mind + 2716, 128);
    }
}

/// `enemyGetTargetPad`: the pad a script sends a computer character to.
pub fn script_bot_pad(chr: i32) -> Option<i32> {
    if !ACTIVE.load(Ordering::Relaxed) {
        return None;
    }
    let mind = lw(chr + 352);
    if mind == 0 {
        return None;
    }
    fx_read(lw(mind), |f| f.pad).filter(|&p| p >= 0)
}

/// After the state of a computer character ran (`enemyTick`): its speed, and the place a script has it run at, when it is going to a pad.
pub fn script_bot_post(chr: i32) {
    if !ACTIVE.load(Ordering::Relaxed) {
        return;
    }
    let mind = lw(chr + 352);
    if mind == 0 || lw(mind) < lw(g::lvnumplayers) || lw(mind + 2716) != 8 {
        return;
    }
    let Some((speed, steer)) = fx_read(lw(mind), |f| (f.speed, f.steer)) else {
        return;
    };
    // (the top speed of a character that is going to a pad is set from its base speed each frame)
    if (speed - 1.0).abs() > 1.0e-3 {
        swc1(mind + 2872, lwc1(mind + 2856) * speed);
    }
    if let Some(at) = steer {
        swc1(mind + 2796, at[0]);
        swc1(mind + 2800, at[1]);
        swc1(mind + 2804, at[2]);
    }
}

/// The colours that light the model `chr` (`partGfx`): the game's, or a tint that a script gave its character.
pub fn script_lighting(chr: i32) -> i32 {
    let own = lw(chr + 28);
    if own == 0 || !ACTIVE.load(Ordering::Relaxed) {
        return own;
    }
    let owner = lw(chr + 244);
    if owner == 0 || lw(owner + 8) != 8 {
        return own;
    }
    let rec = lw(owner + 352);
    if rec == 0 {
        return own;
    }
    let Some(tint) = fx_read(lw(rec), |f| f.tint).flatten() else {
        return own;
    };
    for k in 0..4 {
        let (r, g, b, a) = (lwc1(own + 16 * k), lwc1(own + 16 * k + 4), lwc1(own + 16 * k + 8), lwc1(own + 16 * k + 12));
        let top = r.max(g).max(b);
        swc1(LIGHT_AT + 16 * k, r * tint[0]);
        swc1(LIGHT_AT + 16 * k + 4, g * tint[1] + top * tint[3]);
        swc1(LIGHT_AT + 16 * k + 8, b * tint[2]);
        swc1(LIGHT_AT + 16 * k + 12, a);
    }
    LIGHT_AT
}

/// The kind of the table of results (`scores_pageGfx`): a script can ask for a row for every character.
pub fn script_table_kind(kind: i32) -> i32 {
    if ACTIVE.load(Ordering::Relaxed) && call_all::<_, bool>("results_per_character", ()).into_iter().any(|b| b) {
        2
    } else {
        kind
    }
}

/// The number the table of results ranks the character of the record `record` by (`score` is the game's).
pub fn script_result_key(record: i32, score: i32) -> i32 {
    if !ACTIVE.load(Ordering::Relaxed) || !has("result_key") {
        return score;
    }
    let i = ((record - lw(g::chrinfo)) / 0x1210) as i64;
    call_all::<_, i64>("result_key", (i,)).first().map_or(score, |&v| v as i32)
}

/// What the table of results says in place of the score of the character of the record `record`, if a script has a text for it.
pub fn script_result_text(record: i32) -> Option<i32> {
    if !ACTIVE.load(Ordering::Relaxed) || !has("result_text") {
        return None;
    }
    let i = ((record - lw(g::chrinfo)) / 0x1210) as i64;
    let text = call_all::<_, String>("result_text", (i,)).into_iter().next()?;
    put_text(TEXT_AT, &text, 60);
    Some(TEXT_AT)
}

/// The flames of the characters a script has set alight.
fn flames(info: i32, total: i32) {
    let frame = ts_sdk::frame_count() as i32;
    for i in 0..total {
        let wanted = fx_read(i, |f| f.flame).unwrap_or(false);
        let prop = lw(info + i * 0x1210 + 3020);
        if wanted && prop != 0 && is_alive(i as i64) {
            flame(i, prop, frame);
        }
    }
}

/// the particle type of the game that is drawn as a flame (a soft sprite, added to the picture): the points of a group of it are put by hand each frame
const FLAME: i32 = 10;

/// Keep a green flame on the character `i` whose prop is `prop`: a group of manual particles whose points are put by hand each frame, rising from the feet.
fn flame(i: i32, prop: i32, frame: i32) {
    let mut group = fx_read(i, |f| f.flame_group).unwrap_or(0);
    if group == 0 || lw(group) != 11 {
        for k in 0..3 {
            swc1(FLAME_AT + 4 * k, 0.0);
        }
        group = particleNewManual(0, FLAME, FLAME_AT);
        fx_of(i as i64, |f| f.flame_group = group);
        for r in 0..4 {
            for c in 0..4 {
                swc1(group + 928 + 16 * r + 4 * c, if r == c { 1.0 } else { 0.0 });
            }
        }
    }
    let info = g::particleinfos + FLAME * 104;
    // (the age is kept where the particles are fully lit)
    let life = lwc1(info + 12);
    let age = (lwc1(info + 24) + 0.1).min(life * 0.5);
    swc1(group + 12, age);
    swc1(group + 52, 0.0);
    let n = lw(group + 24).clamp(0, 40);
    let t = frame as f32 / 60.0;
    let (cx, cy, cz) = (lwc1(prop + 48), lwc1(prop + 52), lwc1(prop + 56));
    for k in 0..n {
        let phase = (k as f32 / n as f32 + t * 0.8).fract();
        let radius = 0.3 * (1.0 - phase).powf(0.7);
        let angle = k as f32 * 2.4 + t * 3.0;
        let place = group + 992 + 12 * k;
        swc1(place, cx + radius * angle.cos());
        swc1(place + 4, cy + 0.1 + phase * 1.8);
        swc1(place + 8, cz + radius * angle.sin());
        let g = (255.0 * (1.0 - 0.55 * phase)) as i32;
        let r = (g as f32 * (0.15 + 0.35 * (1.0 - phase))) as i32;
        let colour = (r << 16) | (g << 8) | r;
        swc1(group + 2192 + 4 * k, colour as f32);
        swc1(group + 2592 + 4 * k, 20.0 * (k % 5) as f32);
    }
}

/// The names of the files that scripts want to give another name (`packfileSeek`): the name of the file at `name` (in the game's memory, with or without `host0:`) goes to the scripts that asked
/// for names that start so (`file_prefixes()`), and each of their answers (`on_file(name)`, a list of names) is put in the game's memory; the file with the first of them that the packs have is
/// the one that is used.
pub fn script_file_candidates(name: i32) -> Vec<i32> {
    if !ACTIVE.load(Ordering::Relaxed) {
        return Vec::new();
    }
    // (the name, up to 48 bytes)
    let mut text = Vec::new();
    for i in 0..48 {
        let b = lbu(name + i) as u8;
        if b == 0 {
            break;
        }
        text.push(b);
    }
    let full = String::from_utf8_lossy(&text).to_string();
    let (prefix, plain) = match full.strip_prefix("host0:") {
        Some(rest) => ("host0:", rest.to_string()),
        None => ("", full.clone()),
    };
    let wanted = HOSTS.with(|h| h.try_borrow().map_or(false, |hosts| hosts.iter().any(|host| host.prefixes.iter().any(|p| plain.starts_with(p.as_str())))));
    if !wanted {
        return Vec::new();
    }
    let answers = call_all::<_, rhai::Array>("on_file", (plain,));
    let mut out = Vec::new();
    let mut at = BASE + 0x17_2a00;
    for list in answers {
        for d in list {
            let Ok(s) = d.into_string() else { continue };
            let s = format!("{prefix}{s}");
            if s.len() > 120 || out.len() >= 4 {
                continue;
            }
            put_text(at, &s, 120);
            out.push(at);
            at += 0x80;
        }
    }
    out
}

/// The name of a model file, from the game's memory.
fn read_name(at: i32) -> String {
    let mut text = Vec::new();
    for i in 0..48 {
        let b = lbu(at + i) as u8;
        if b == 0 {
            break;
        }
        text.push(b);
    }
    String::from_utf8_lossy(&text).to_string()
}

/// `gunNew` for a gun held in both hands: a script may give another model for it (`on_gun_model(file name of the gun's model, left hand)`: the name of a file of the packs, or ""), which is loaded
/// the first time and kept as long as the model of the plain gun is the one it was loaded beside; the table of props keeps the plain model for the next gun.
pub fn script_gun_new(prop: i32, left_hand: bool) -> i32 {
    thread_local! {
        static MODELS: RefCell<Vec<(i32, i32, i32, bool)>> = RefCell::new(Vec::new());
    }
    let plain_view = || gunNew(prop, 0.0_f32, -1e+02_f32, 0.0_f32, 0.0_f32);
    if !ACTIVE.load(Ordering::Relaxed) || !has("on_gun_model") {
        return plain_view();
    }
    let def = prop * 28 + PROPTABLE;
    if lw(def + 4) == 0 {
        // (the plain model is loaded by making a prop of it)
        let view = plain_view();
        propDelete(view);
    }
    let plain = lw(def + 4);
    if plain == 0 {
        return plain_view();
    }
    let known = MODELS.with(|m| m.borrow().iter().find(|e| e.0 == prop && e.1 == plain && e.3 == left_hand).map(|e| e.2));
    let other = match known {
        Some(other) => other,
        None => {
            let full = read_name(lw(def + 12));
            let (prefix, name) = match full.strip_prefix("host0:") {
                Some(rest) => ("host0:", rest.to_string()),
                None => ("", full.clone()),
            };
            let answer = call_all::<_, String>("on_gun_model", (name, left_hand)).into_iter().find(|s| !s.is_empty()).map(|s| format!("{prefix}{s}"));
            let other = match answer {
                Some(s) if s.len() < 100 => {
                    let at = BASE + 0x17_2a00 + 0x200;
                    put_text(at, &s, 120);
                    if packfileSeekPlain(at) != 0 {
                        obLoad(at)
                    } else {
                        0
                    }
                }
                _ => 0,
            };
            MODELS.with(|m| {
                let mut m = m.borrow_mut();
                m.retain(|e| !(e.0 == prop && e.3 == left_hand));
                m.push((prop, plain, other, left_hand));
            });
            other
        }
    };
    if other == 0 {
        return plain_view();
    }
    sw(def + 4, other);
    let view = plain_view();
    sw(def + 4, plain);
    view
}
