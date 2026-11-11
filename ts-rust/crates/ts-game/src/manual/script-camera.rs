//! The cameras the scripts of mods give the players' views, and what they read of the sticks and of where a player looks (extensions).
//! `camera_set(p, ...)` puts the view of the player `p` at a place looking at another, until `camera_clear(p)` or the match starts again.  The camera
//! is put in place where the game's camera tick (`camTick`) works out the view, as the game's own debug cameras that fly about are (the room of the
//! eye is found by its floor, so the level is drawn from there); while it is set the player's own body and the guns in its hands are drawn in its
//! view as in the game's third person camera, and the gun of the first person view is not.

use crate::*;
use rhai::Engine;
use std::sync::Mutex;

/// The eye and the place looked at, of each of the eight players.
static CAMERAS: Mutex<[Option<[f32; 6]>; 8]> = Mutex::new([None; 8]);

/// How wide a place `room_at` looks for a floor under.
const ROOM_RADIUS: f32 = 0.25;
/// The centre of a stick, as the controller has it (`joyFixVal`).
const STICK_CENTRE: i32 = 128;

pub(super) fn clear_all() {
    *CAMERAS.lock().unwrap() = [None; 8];
}

fn camera_of(number: i32) -> Option<[f32; 6]> {
    if !(0..8).contains(&number) || lw(g::gamestate) != 3 {
        return None;
    }
    CAMERAS.lock().unwrap()[number as usize]
}

/// The view of the player being drawn is a script's camera (its own body is drawn, the gun of the first person view is not).
pub fn script_camera_on() -> bool {
    let current = lw(g::currentplayer);
    current != 0 && camera_of(lw(current)).is_some()
}

/// The camera tick of the player `player` (`camTick`): when a script set its camera, the view is made from it and true is answered; `up` is the
/// camera tick's up direction.
pub fn script_camera_view(player: i32, up: i32) -> bool {
    let Some(c) = camera_of(lw(player)) else { return false };
    let frame = Frame::new(32);
    let sp = frame.sp();
    for k in 0..3 {
        swc1(player + 884 + 4 * k, c[k as usize]);
        swc1(sp + 4 * k, c[3 + k as usize]);
    }
    let old = lw(player + 896);
    if old != 0 {
        sw(player + 900, old);
    }
    // (an eye with no floor under it, outside the level, keeps the room it was in)
    let found = moveFindRoom(player + 884, 1.0, lwc1(player + 132));
    sw(player + 896, if found != 0 { found } else if old != 0 { old } else { lw(player + 796) });
    matrixLookAt(lw(player + 1764), player + 884, sp, up);
    true
}

fn valid_player(p: i64) -> bool {
    p >= 0 && p < 8 && (p as i32) < lw(g::lvnumplayers) && lw(g::players) != 0
}

fn player_record(p: i64) -> i32 {
    lw(g::players) + (p as i32) * 1820
}

pub(super) fn register(engine: &mut Engine) {
    // the camera of a player's view: the eye, and the place it looks at
    engine.register_fn("camera_set", |p: i64, ex: f64, ey: f64, ez: f64, tx: f64, ty: f64, tz: f64| {
        let same = ex == tx && ey == ty && ez == tz;
        if valid_player(p) && !same {
            CAMERAS.lock().unwrap()[p as usize] = Some([ex as f32, ey as f32, ez as f32, tx as f32, ty as f32, tz as f32]);
        }
    });
    engine.register_fn("camera_clear", |p: i64| {
        if (0..8).contains(&p) {
            CAMERAS.lock().unwrap()[p as usize] = None;
        }
    });
    // the room of the level a place is in (found by the floor under it, as the game finds the room of a camera); 0 outside the level
    engine.register_fn("room_at", |x: f64, y: f64, z: f64| -> i64 {
        if lw(g::gamestate) != 3 {
            return 0;
        }
        let frame = Frame::new(16);
        let at = frame.sp();
        swc1(at, x as f32);
        swc1(at + 4, y as f32);
        swc1(at + 8, z as f32);
        moveFindRoom(at, 1.0, ROOM_RADIUS) as i64
    });
    // nothing of the level is in the way of a line from one place to another (its walls, floors and the props that stop shots), as the game tests a line
    engine.register_fn("line_clear_between", |x1: f64, y1: f64, z1: f64, x2: f64, y2: f64, z2: f64| -> bool {
        if lw(g::gamestate) != 3 {
            return false;
        }
        let frame = Frame::new(80);
        let sp = frame.sp();
        let (from, to) = ([x1 as f32, y1 as f32, z1 as f32], [x2 as f32, y2 as f32, z2 as f32]);
        for k in 0..3 {
            swc1(sp + 48 + 4 * k as i32, from[k]);
            swc1(sp + 16 + 4 * k as i32, to[k] - from[k]);
        }
        let room = moveFindRoom(sp + 48, 1.0, ROOM_RADIUS);
        if room == 0 {
            return false;
        }
        // (the arguments the line test takes on the stack)
        sw(sp, 0);
        sw(sp + 8, 0);
        bgLineTest(room, sp + 48, sp + 16, 0, 0, 0, 0, 112) == 0
    });
    engine.register_fn("camera_on", |p: i64| (0..8).contains(&p) && CAMERAS.lock().unwrap()[p as usize].is_some());
    // the sticks of a player's controller (0 left across, 1 left up, 2 right across, 3 right up; -1 to 1), and taking them from the game
    engine.register_fn("stick", |p: i64, axis: i64| -> f64 {
        if valid_player(p) && (0..4).contains(&axis) {
            joyGetAxis(p as i32, axis as i32) as f64
        } else {
            0.0
        }
    });
    engine.register_fn("consume_sticks", |p: i64| {
        if valid_player(p) {
            let c = crate::game::controllers::controller(p as i32);
            for off in [300, 304, 308, 312] {
                sw(c + off, STICK_CENTRE);
            }
        }
    });
    // where a player looks: the turn about the up axis and up or down, in degrees, as the game keeps them (the mouse turns them too)
    engine.register_fn("look_yaw", |p: i64| if valid_player(p) { lwc1(player_record(p) + 276) as f64 } else { 0.0 });
    engine.register_fn("look_pitch", |p: i64| if valid_player(p) { lwc1(player_record(p) + 284) as f64 } else { 0.0 });
    engine.register_fn("set_look", |p: i64, yaw: f64, pitch: f64| {
        if valid_player(p) {
            swc1(player_record(p) + 276, yaw as f32);
            swc1(player_record(p) + 284, pitch as f32);
        }
    });
}
