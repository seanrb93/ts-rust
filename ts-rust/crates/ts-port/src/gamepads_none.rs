//! A game controller on Android (the built-in controls of a handheld such as the AYN Thor, or a Bluetooth controller): the buttons are the
//! keys Android sends (`key`, called by the window), the sticks, triggers and the hat are the axes the patched winit keeps
//! (`third_party/winit`).  Each is a virtual DualShock 2, laid out like an Xbox pad: A is cross, B circle, X square, Y triangle.

use std::sync::atomic::{AtomicU16, AtomicBool, Ordering};
use ts_sdk::pad::{btn, PadState};

/// the buttons held, by the keys
static HELD: AtomicU16 = AtomicU16::new(0);
/// a controller has been used (it is then the player's pad)
static SEEN: AtomicBool = AtomicBool::new(false);

/// A key of the Android codes of a controller (KEYCODE_BUTTON_A and the like, and the D-pad) went down or up.
pub fn key(code: u32, down: bool) {
    let bit = match code {
        96 => btn::CROSS,
        97 => btn::CIRCLE,
        99 => btn::SQUARE,
        100 => btn::TRIANGLE,
        102 => btn::L1,
        103 => btn::R1,
        104 => btn::L2,
        105 => btn::R2,
        106 => btn::L3,
        107 => btn::R3,
        108 => btn::START,
        109 => btn::SELECT,
        19 => btn::UP,
        20 => btn::DOWN,
        21 => btn::LEFT,
        22 => btn::RIGHT,
        _ => return,
    };
    if code < 96 {
        // (the arrow keys are the D-pad of a controller or the arrows of a keyboard: both are the D-pad)
    } else {
        SEEN.store(true, Ordering::Relaxed);
    }
    if down {
        HELD.fetch_or(bit, Ordering::Relaxed);
    } else {
        HELD.fetch_and(!bit, Ordering::Relaxed);
    }
}

pub struct Pads;

impl Pads {
    pub fn new() -> Result<Pads, String> {
        Ok(Pads)
    }

    /// the style of the pictures of the buttons: Xbox
    pub fn styles(&self) -> Vec<u8> {
        if SEEN.load(Ordering::Relaxed) {
            vec![1]
        } else {
            Vec::new()
        }
    }

    pub fn poll(&mut self) -> Vec<PadState> {
        #[cfg(target_os = "android")]
        let a = winit::platform::android::gamepad_axes();
        #[cfg(not(target_os = "android"))]
        let a = [0.0f32; 8];
        let cfg = crate::config::get();
        let mut buttons = HELD.load(Ordering::Relaxed);
        // the triggers (axes on most controllers), and the hat as the D-pad
        if a[6] > 0.5 {
            buttons |= btn::L2;
        }
        if a[7] > 0.5 {
            buttons |= btn::R2;
        }
        if a[4] < -0.5 {
            buttons |= btn::LEFT;
        }
        if a[4] > 0.5 {
            buttons |= btn::RIGHT;
        }
        if a[5] < -0.5 {
            buttons |= btn::UP;
        }
        if a[5] > 0.5 {
            buttons |= btn::DOWN;
        }
        let stick = |v: f32| stick_to_byte(stick_value(v, cfg.stick_deadzone, cfg.stick_sensitivity));
        // (right x, right y, left x, left y)
        let sticks = [stick(a[2]), stick(a[3]), stick(a[0]), stick(a[1])];
        if buttons != 0 || sticks.iter().any(|&v| (v as i32 - 0x80).abs() > 20) {
            SEEN.store(true, Ordering::Relaxed);
        }
        if !SEEN.load(Ordering::Relaxed) {
            return Vec::new();
        }
        let mut s = PadState::idle(true);
        s.buttons = buttons;
        s.sticks = sticks;
        vec![s]
    }
}

pub fn stick_value(v: f32, deadzone: f32, sensitivity: f32) -> f32 {
    let v = if v.abs() < deadzone { 0.0 } else { v };
    (v * sensitivity).clamp(-1.0, 1.0)
}

pub fn stick_to_byte(v: f32) -> u8 {
    (((v + 1.0) * 0.5 * 255.0).round() as i32).clamp(0, 255) as u8
}
