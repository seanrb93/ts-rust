//! Controller services (`scePad*`, `sceMtap*`) backed by a virtual DualShock 2 per port.
//!
//! The port (keyboard / gamepad layer) calls `set_pad_state`; `scePadRead` then reports it in the original libpad
//! wire format (active-low button bitmask, 0x80-centred sticks, pressure bytes), which is what the decompiled input
//! code expects.
#![allow(non_snake_case)]

use std::sync::Mutex;
use ts_rt::*;

/// libpad button bits (active when set here; they are inverted when written to the game).
pub mod btn {
    pub const SELECT: u16 = 0x0100;
    pub const L3: u16 = 0x0200;
    pub const R3: u16 = 0x0400;
    pub const START: u16 = 0x0800;
    pub const UP: u16 = 0x1000;
    pub const RIGHT: u16 = 0x2000;
    pub const DOWN: u16 = 0x4000;
    pub const LEFT: u16 = 0x8000;
    pub const L2: u16 = 0x0001;
    pub const R2: u16 = 0x0002;
    pub const L1: u16 = 0x0004;
    pub const R1: u16 = 0x0008;
    pub const TRIANGLE: u16 = 0x0010;
    pub const CIRCLE: u16 = 0x0020;
    pub const CROSS: u16 = 0x0040;
    pub const SQUARE: u16 = 0x0080;
}

#[derive(Clone, Copy)]
pub struct PadState {
    pub connected: bool,
    pub buttons: u16,
    /// right x, right y, left x, left y; 0x80 = centre
    pub sticks: [u8; 4],
}

impl PadState {
    pub const fn idle(connected: bool) -> PadState {
        PadState { connected, buttons: 0, sticks: [0x80; 4] }
    }
}

static PADS: Mutex<[PadState; 2]> = Mutex::new([PadState::idle(true), PadState::idle(false)]);

/// The multitap on port 0 (three or more controllers): the controllers in its slots 1 to 3 (slot 0 is the port's own pad).
static TAP: Mutex<(bool, [PadState; 3])> = Mutex::new((false, [PadState::idle(false); 3]));

pub fn set_pad_state(port: usize, s: PadState) {
    if let Some(p) = PADS.lock().unwrap().get_mut(port) {
        *p = s;
    }
}

/// Turn the multitap of port 0 on (with the controllers of its slots 1 to 3) or off.  The game asks how many slots the port has
/// every couple of seconds and finds the controllers there, so they can come and go.
pub fn set_multitap(slots: Option<[PadState; 3]>) {
    *TAP.lock().unwrap() = match slots {
        Some(s) => (true, s),
        None => (false, [PadState::idle(false); 3]),
    };
}

fn pad(port: i64, slot: i64) -> PadState {
    if slot == 0 {
        PADS.lock().unwrap().get(port as usize).copied().unwrap_or(PadState::idle(false))
    } else if port == 0 && (1..=3).contains(&slot) {
        let t = TAP.lock().unwrap();
        if t.0 { t.1[slot as usize - 1] } else { PadState::idle(false) }
    } else {
        PadState::idle(false)
    }
}

pub fn scePadInit(_args: &[i64]) -> i32 {
    1
}
pub fn scePadPortOpen(_args: &[i64]) -> i32 {
    1
}
pub fn sceMtapInit(_args: &[i64]) -> i32 {
    1
}
pub fn sceMtapPortOpen(_args: &[i64]) -> i32 {
    1
}
/// A multitap is on port 0 when there are more than two controllers.
pub fn sceMtapGetConnection(args: &[i64]) -> i32 {
    (args[0] == 0 && TAP.lock().unwrap().0) as i32
}
pub fn scePadGetSlotMax(args: &[i64]) -> i32 {
    if args[0] == 0 && TAP.lock().unwrap().0 { 4 } else { 1 }
}
/// scePadStateStable = 6, scePadStateDiscon = 0
pub fn scePadGetState(args: &[i64]) -> i32 {
    if pad(args[0], args[1]).connected { 6 } else { 0 }
}
/// term 1 = current mode id (7 = DualShock 2 analog), term 2 = extended id, term 3 = offset
pub fn scePadInfoMode(args: &[i64]) -> i32 {
    match args[2] {
        1 => 7,
        2 => 7,
        3 => 0,
        _ => 0,
    }
}
pub fn scePadSetMainMode(_args: &[i64]) -> i32 {
    1
}
/// actuator count when actno == -1
pub fn scePadInfoAct(args: &[i64]) -> i32 {
    if args[2] == -1 { 2 } else { 1 }
}
pub fn scePadSetActAlign(_args: &[i64]) -> i32 {
    1
}
pub fn scePadSetActDirect(_args: &[i64]) -> i32 {
    1
}
/// scePadReqStateComplete = 0
pub fn scePadGetReqState(_args: &[i64]) -> i32 {
    0
}

/// Fill the 32-byte libpad receive buffer; returns its length.
pub fn scePadRead(args: &[i64]) -> i32 {
    let p = pad(args[0], args[1]);
    let buf = args[2] as i32;
    for i in 0..32 {
        sb(buf + i, 0);
    }
    if !p.connected {
        return 0;
    }
    let raw = !p.buttons;
    sb(buf, 0); // success
    sb(buf + 1, 0x79); // DualShock 2 analog, 9 half-words
    sb(buf + 2, (raw >> 8) as i32);
    sb(buf + 3, (raw & 0xff) as i32);
    for (i, s) in p.sticks.iter().enumerate() {
        sb(buf + 4 + i as i32, *s as i32);
    }
    // pressure sensitive buttons: right, left, up, down, triangle, circle, cross, square, L1, R1, L2, R2
    const ORDER: [u16; 12] = [
        btn::RIGHT, btn::LEFT, btn::UP, btn::DOWN, btn::TRIANGLE, btn::CIRCLE, btn::CROSS, btn::SQUARE, btn::L1,
        btn::R1, btn::L2, btn::R2,
    ];
    for (i, m) in ORDER.iter().enumerate() {
        sb(buf + 8 + i as i32, if p.buttons & m != 0 { 0xff } else { 0 });
    }
    32
}

// ---- the mouse ---------------------------------------------------------------------------------------------------------
//
// The mouse does not go through the virtual pad: its movement is collected here and the game side turns the player's view by it
// (`ts_game::mouse_look_tick`).

static MOUSE: Mutex<(f64, f64)> = Mutex::new((0.0, 0.0));

/// Movement of the mouse (in counts, x to the right, y down) since the last `mouse_take`.
pub fn mouse_move(dx: f64, dy: f64) {
    let mut m = MOUSE.lock().unwrap();
    m.0 += dx;
    m.1 += dy;
}

pub fn mouse_take() -> (f64, f64) {
    std::mem::take(&mut *MOUSE.lock().unwrap())
}

/// The keyboard and mouse were used last (not a controller): the game switches the aim assistance off then.
static MKB_ACTIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn set_mkb_active(on: bool) {
    MKB_ACTIVE.store(on, std::sync::atomic::Ordering::Relaxed);
}

pub fn mkb_active() -> bool {
    MKB_ACTIVE.load(std::sync::atomic::Ordering::Relaxed)
}
