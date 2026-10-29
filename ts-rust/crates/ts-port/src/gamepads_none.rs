//! No controller support (Android until SDL's activity glue is in): the same interface as `gamepads.rs`, with no controllers.

use ts_sdk::pad::PadState;

pub struct Pads;

impl Pads {
    pub fn new() -> Result<Pads, String> {
        Err("controllers are not supported on this platform yet".into())
    }
    pub fn styles(&self) -> Vec<u8> {
        Vec::new()
    }
    pub fn poll(&mut self) -> Vec<PadState> {
        Vec::new()
    }
}

pub fn stick_value(v: f32, deadzone: f32, sensitivity: f32) -> f32 {
    let v = if v.abs() < deadzone { 0.0 } else { v };
    (v * sensitivity).clamp(-1.0, 1.0)
}

pub fn stick_to_byte(v: f32) -> u8 {
    (((v + 1.0) * 0.5 * 255.0).round() as i32).clamp(0, 255) as u8
}
