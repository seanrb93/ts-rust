//! Looking around with the mouse: the player's view is turned by the mouse's own movement, not by a stick that is held.
//!
//! The yaw (+276) and pitch (+284) of the player are in degrees and the game's own turning adds to them (`playerTick`), so the
//! mouse adds to them too, once per frame.  A zoomed view turns less, as the game's stick turning does (it scales by the field of
//! view).  The port's window collects the mouse (`ts_sdk::pad::mouse_move`) and asks here when the view is the mouse's
//! (`mouse_look_active`: a level is running and no menu is open) to capture the pointer.

use crate::*;

/// Degrees for one count of the mouse at a sensitivity of 1.
const DEGREES: f32 = 0.12;

/// A level is being played and nothing of the menus is on top of it.
pub fn mouse_look_active() -> bool {
    lw(g::gamestate) == 3 && lw(g::lvfirstframe_130) == 0 && lw(g::gamepaused) == 0 && lw(g::replaymode) == 0 && lw(lw(g::front) + 20) == 0 && lw(g::players) != 0
}

/// A level is being played (a pause or a menu on top of it does not matter).
pub fn level_running() -> bool {
    lw(g::gamestate) == 3
}

/// A one-player level is being played: any controller can be the player's then.
pub fn single_player_level() -> bool {
    lw(g::gamestate) == 3 && lw(g::lvnumplayers) <= 1
}

/// The game's Aim Mode is Toggle (not Hold): bit 1 of the player's profile flags.
pub fn aim_mode_toggle() -> bool {
    let p = lw(g::players);
    p != 0 && lw(p + 24) != 0 && (lhu(lw(p + 24) + 2) & 2) != 0
}

/// The weapon in the player's hands zooms (it has a range of fields of view).
pub fn weapon_zooms() -> bool {
    let p = lw(g::players);
    p != 0 && lwc1(p + 268) != lwc1(p + 272)
}

/// The aim assistance is off for the player who uses the keyboard and the mouse.
pub fn autoaim_off() -> bool {
    ts_sdk::pad::mkb_active() && lw(g::currentplayer) == lw(g::players)
}

/// Called once per frame.
pub fn mouse_look_tick() {
    let (mut dx, mut dy) = ts_sdk::pad::mouse_take();
    // test aid: TS_MOUSETEST=<x>,<y> moves the mouse by that much on every frame of a level
    static TEST: std::sync::OnceLock<(f64, f64)> = std::sync::OnceLock::new();
    let t = TEST.get_or_init(|| std::env::var("TS_MOUSETEST").ok().and_then(|v| v.split_once(',').map(|(a, b)| (a.parse().unwrap_or(0.0), b.parse().unwrap_or(0.0)))).unwrap_or((0.0, 0.0)));
    dx += t.0;
    dy += t.1;
    if !mouse_look_active() || (dx == 0.0 && dy == 0.0) {
        return;
    }
    let p = lw(g::players);
    let fov = (lwc1(p + 260) / 60.0).clamp(0.05, 2.0);
    let k = DEGREES * fov;
    // (the game's yaw grows to the left and its pitch upwards)
    let mut yaw = lwc1(p + 276) - dx as f32 * k;
    while yaw < 0.0 {
        yaw += 360.0;
    }
    while yaw >= 360.0 {
        yaw -= 360.0;
    }
    swc1(p + 276, yaw);
    swc1(p + 284, (lwc1(p + 284) - dy as f32 * k).clamp(-50.0, 50.0));
}
