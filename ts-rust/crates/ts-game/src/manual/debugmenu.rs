//! The game's developer menu, opened with F2 while a level is played.
//!
//! The retail code still contains the menu (its tables, `debugmenuStart`, the pad handling in `menuTick`) but nothing opens it,
//! and many of its entries were compiled out: the debug drawing (portals, volumes, walls, floors, positions, speed, performance
//! numbers, editors), the memory statistics and the turtle (slow-down) switches do nothing.  The menu is trimmed to the entries
//! that work before it is first shown.  The settings are put back to normal whenever a level is loaded.

use crate::*;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

static REQUEST: AtomicBool = AtomicBool::new(false);
static TRIMMED: AtomicBool = AtomicBool::new(false);
static LAST_STATE: AtomicI32 = AtomicI32::new(0);

/// F2 was pressed.
pub fn request_debug_menu() {
    REQUEST.store(true, Ordering::Relaxed);
}

/// One menu table: header of 16 bytes (title, number of items, the item with the focus), then items of 32 bytes.  An item has its
/// text (or a function that returns it), its place relative to the item before it (or to the item it describes: the On / Off
/// items to the right), flags (4 = runs a function, 0x40 = opens a menu, 0x80 = goes back, 2 = display only) and the function or menu.
const ITEM: i32 = 32;

fn item(menu: i32, i: i32) -> i32 {
    menu + 16 + ITEM * i
}

/// What an entry shows after its name: whether a setting is on.
#[derive(Clone, Copy)]
enum State {
    None,
    Flag(i32),
    /// on while the flag is clear
    FlagClear(i32),
    /// on while the camera mode is one of these
    Camera(&'static [i32]),
}

/// (item address, name, state) of the entries whose text shows their state
static LABELS: std::sync::Mutex<Vec<(i32, String, State)>> = std::sync::Mutex::new(Vec::new());
const LABEL_AT: i32 = 0x240_0000 + 0x7000;

/// Keep only some entries of a menu (by their number).  The first kept entry takes the place of the menu's first one and every
/// other entry goes below the one before it.  Must run before the menu is first started (starting converts the relative places to
/// absolute ones).  The On / Off displays of the original are dropped: their functions returned nothing in the translated code
/// and they were laid out for the entries that are gone; the entries show their state in their own text instead (`update_labels`).
fn trim(menu: i32, keep: &[(i32, State, &str)]) {
    let old: Vec<[i32; 8]> = (0..lbu(menu + 4) as i32).map(|i| std::array::from_fn(|k| lw(item(menu, i) + 4 * k as i32))).collect();
    let mut labels = LABELS.lock().unwrap();
    for (n, &(a, state, name)) in keep.iter().enumerate() {
        let mut it = old[a as usize];
        if n == 0 {
            it[1] = old[0][1];
            it[3] = old[0][3];
            it[2] = 0;
            it[4] = 0;
        } else {
            it[1] = old[1][1];
            it[3] = old[1][3];
            it[2] = item(menu, n as i32 - 1);
            it[4] = item(menu, n as i32 - 1);
        }
        for (k, w) in it.iter().enumerate() {
            sw(item(menu, n as i32) + 4 * k as i32, *w);
        }
        let mut text = String::new();
        let mut at = it[0];
        while lbu(at) != 0 && text.len() < 40 {
            text.push(lbu(at) as u8 as char);
            at += 1;
        }
        let text = if name.is_empty() { text.trim().to_string() } else { name.to_string() };
        labels.push((item(menu, n as i32), text, state));
    }
    sb(menu + 4, keep.len() as i32);
    sw(menu + 8, 0);
}

fn trim_menus() {
    if TRIMMED.swap(true, Ordering::Relaxed) {
        return;
    }
    use State::*;
    // Move View, Game View, 3rd Person View, Prop View, View Items, Cheats, Exit Level
    trim(g::debugmenu, &[(0, Camera(&[3]), ""), (1, Camera(&[0]), ""), (2, Camera(&[1, 2]), ""), (3, Camera(&[4, 5, 6, 9]), ""), (6, None, ""), (8, None, ""), (10, None, "")]);
    // View Guns, View Background
    trim(g::debugmenu4, &[(5, Flag(g::viewguns), ""), (6, Flag(g::viewbg), "")]);
    // (the first entry goes where the Cheats menu's first one is: the original place is for a longer menu)
    sw(item(g::debugmenu4, 0) + 4, lw(item(g::debugmenu2, 0) + 4));
    sw(item(g::debugmenu4, 0) + 12, lw(item(g::debugmenu2, 0) + 12));
    // Invincible, Invisible, Noclip (the unused "Collisions" switch, which clears `collisions`), Exit
    trim(g::debugmenu2, &[(0, Flag(g::invincible), ""), (1, Flag(g::invisible), ""), (2, FlagClear(g::collisions), "Noclip"), (5, None, "")]);
}

/// The text of the entries that show a setting: "name: On" / "name: Off".
fn update_labels() {
    let labels = LABELS.lock().unwrap();
    for (n, (at, name, state)) in labels.iter().enumerate() {
        let on = match *state {
            State::None => continue,
            State::Flag(a) => lw(a) != 0,
            State::FlagClear(a) => lw(a) == 0,
            State::Camera(modes) => modes.contains(&camGetMode()),
        };
        let text = format!("{name}: {}", if on { "On" } else { "Off" });
        let start = LABEL_AT + 64 * n as i32;
        for (i, b) in text.bytes().enumerate() {
            sb(start + i as i32, b as i32);
        }
        sb(start + text.len() as i32, 0);
        sw(*at, start);
    }
}

/// Everything the menu can change goes back to normal.
fn reset_settings() {
    sw(g::viewguns, 1);
    sw(g::viewbg, 1);
    sw(g::invincible, 0);
    sw(g::invisible, 0);
    sw(g::collisions, 1);
    sw(g::padlinkedit, 0);
    sw(g::propedit, 0);
    camSetMode(0);
}

/// Noclip: while `collisions` is clear the player flies through the level.  `moveTestDoSlide` (patched) calls this first: when it is the
/// player's own move (from `from` to `target`) the test against the level is skipped; `target` is the position the game wants to move to; it is replaced by the position the
/// stick asks for, along the direction the player looks (up and down too) and to the sides.  Returns true when it did so.
const FLY_SPEED: f32 = 14.0; // units per second at full stick

pub fn noclip_slide(from: i32, target: i32) -> bool {
    if lw(g::collisions) != 0 || lw(g::gamestate) != 3 {
        return false;
    }
    let p = lw(g::currentplayer);
    if p == 0 {
        return false;
    }
    let prop = lw(p + 384);
    // only the player's own move (from its position)
    if from != p + 140 && from != prop + 48 {
        return false;
    }
    let pad = lw(p);
    let forward = joyGetAxis(pad, lw(p + 68));
    let strafe = joyGetAxis(pad, lw(p + 72));
    let dt = lwc1(g::updateframesf) / 60.0;
    // circle goes up and cross goes down; the player counts as airborne so that the game does not follow the floor
    let buttons = joyBut(pad);
    let rise = (buttons & 32 != 0) as i32 as f32 - (buttons & 64 != 0) as i32 as f32;
    sw(p + 296, 3);
    let mut pos = [lwc1(from), lwc1(from + 4), lwc1(from + 8)];
    // the direction the player looks (a unit vector) and the one to its right
    let f = [lwc1(p + 188), lwc1(p + 192), lwc1(p + 196)];
    let flat = (f[0] * f[0] + f[2] * f[2]).sqrt().max(1e-4);
    let r = [-f[2] / flat, 0.0, f[0] / flat];
    pos[1] += rise * FLY_SPEED * dt;
    for k in 0..3 {
        pos[k] += (f[k] * forward + r[k] * strafe) * FLY_SPEED * dt;
        swc1(target + 4 * k as i32, pos[k]);
    }
    true
}

/// Switching noclip on closes the menu, which takes the sticks.
fn noclip_menu_tick() {
    static WAS_ON: AtomicBool = AtomicBool::new(false);
    let on = lw(g::collisions) == 0;
    if on && !WAS_ON.swap(true, Ordering::Relaxed) && debugmenuGetJoyMode() == 1 {
        close_menu();
    } else if !on {
        WAS_ON.store(false, Ordering::Relaxed);
    }
}

/// Close the menu with all its pages.
fn close_menu() {
    let mut guard = 0;
    while lw(g::menu_stackpos) > 0 && guard < 16 {
        menuEnd();
        guard += 1;
    }
}

/// Called once per frame by `present_frame`.
pub fn debug_menu_tick() {
    // test aid: TS_DEBUGMENU_AT=<frame>[,<frame>..] presses F2 at those frames
    static TEST_AT: std::sync::OnceLock<Vec<u64>> = std::sync::OnceLock::new();
    let at = TEST_AT.get_or_init(|| std::env::var("TS_DEBUGMENU_AT").map(|v| v.split(',').filter_map(|x| x.trim().parse().ok()).collect()).unwrap_or_default());
    if at.contains(&ts_sdk::frame_count()) {
        request_debug_menu();
    }
    noclip_menu_tick();
    let state = lw(g::gamestate);
    let prev = LAST_STATE.swap(state, Ordering::Relaxed);
    if state != 3 && debugmenuGetJoyMode() == 1 {
        // the level was left with the menu open
        close_menu();
    }
    if state == 6 && prev != 6 {
        // a level is being loaded
        reset_settings();
    }
    if state == 3 && lw(g::curmenu) != 0 && TRIMMED.load(Ordering::Relaxed) {
        update_labels();
    }
    if !REQUEST.swap(false, Ordering::Relaxed) || state != 3 {
        return;
    }
    if debugmenuGetJoyMode() == 1 {
        close_menu();
    } else if lw(g::curmenu) == 0 {
        trim_menus();
        debugmenuStart();
    }
}
