//! Game controllers: Xbox (XInput), PlayStation 4 / 5 (DualShock 4, DualSense), Nintendo Switch / Switch 2 (Pro Controller and
//! the like) and anything else SDL knows, through SDL's gamepad layer (it has the controller database and the drivers that
//! start the PlayStation and Nintendo pads' full reports).  Each controller becomes a virtual DualShock 2.

use sdl3::gamepad::{Axis, Button, Gamepad};
use sdl3::{EventPump, GamepadSubsystem, Sdl};
use ts_sdk::pad::{btn, PadState};

pub struct Pads {
    // (kept alive: SDL shuts down when the last handle goes)
    _sdl: Sdl,
    sub: GamepadSubsystem,
    pump: EventPump,
    open: Vec<(sdl3::joystick::JoystickId, Gamepad)>,
}

impl Pads {
    pub fn new() -> Result<Pads, String> {
        // a game window of the port's own is not an SDL window: let the controllers report while it has the focus, and let the
        // drivers use the controllers' own protocols (PlayStation and Nintendo pads only send simple reports otherwise).  The Xbox
        // pads are left to SDL's Windows drivers (XInput, Windows.Gaming.Input): its HIDAPI driver for them is off there on purpose.
        for (k, v) in [
            ("SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS", "1"),
            ("SDL_JOYSTICK_HIDAPI", "1"),
            ("SDL_JOYSTICK_HIDAPI_PS4", "1"),
            ("SDL_JOYSTICK_HIDAPI_PS5", "1"),
            ("SDL_JOYSTICK_HIDAPI_SWITCH", "1"),
            ("SDL_JOYSTICK_HIDAPI_SWITCH2", "1"),
            // (the pads' lights and rumble are not used; leave the player-number light alone)
            ("SDL_JOYSTICK_HIDAPI_SWITCH_PLAYER_LED", "0"),
        ] {
            sdl3::hint::set(k, v);
        }
        let sdl = sdl3::init().map_err(|e| e.to_string())?;
        let sub = sdl.gamepad().map_err(|e| e.to_string())?;
        let pump = sdl.event_pump().map_err(|e| e.to_string())?;
        let mut pads = Pads { _sdl: sdl, sub, pump, open: Vec::new() };
        pads.refresh();
        Ok(pads)
    }

    /// Open the controllers that appeared and let go of those that went.
    fn refresh(&mut self) {
        let Ok(ids) = self.sub.gamepads() else { return };
        self.open.retain(|(id, g)| ids.contains(id) && g.connected());
        for id in ids {
            if self.open.iter().any(|(o, _)| *o == id) {
                continue;
            }
            match self.sub.open(id) {
                Ok(g) => self.open.push((id, g)),
                Err(e) => eprintln!("[pad] cannot open controller {id:?}: {e}"),
            }
        }
    }

    /// The state of every connected controller, in the order they were connected.
    pub fn poll(&mut self) -> Vec<PadState> {
        // (events carry the plug and unplug; the pump also reads the controllers)
        let mut changed = false;
        for ev in self.pump.poll_iter() {
            if matches!(ev, sdl3::event::Event::GamepadAdded { .. } | sdl3::event::Event::GamepadRemoved { .. }) {
                changed = true;
            }
        }
        if changed {
            self.refresh();
        }
        let cfg = crate::config::get();
        self.open.iter().map(|(_, g)| state(g, cfg.stick_deadzone, cfg.stick_sensitivity)).collect()
    }
}

/// One stick axis as PCSX2 treats it: dead zone, then the sensitivity (the range is stretched and cut off at the ends).
pub fn stick_value(v: f32, deadzone: f32, sensitivity: f32) -> f32 {
    let v = if v.abs() < deadzone { 0.0 } else { v };
    (v * sensitivity).clamp(-1.0, 1.0)
}

/// A stick value (-1 to 1) as the byte of a DualShock 2 (0x80 = centre).
pub fn stick_to_byte(v: f32) -> u8 {
    (((v + 1.0) * 0.5 * 255.0).round() as i32).clamp(0, 255) as u8
}

fn stick_byte(raw: i16, deadzone: f32, sensitivity: f32) -> u8 {
    stick_to_byte(stick_value(raw as f32 / 32767.0, deadzone, sensitivity))
}

fn state(g: &Gamepad, deadzone: f32, sensitivity: f32) -> PadState {
    let mut s = PadState::idle(true);
    // (the buttons by position: the one at the bottom is the cross, whatever it is called on the pad)
    let map = [
        (Button::South, btn::CROSS),
        (Button::East, btn::CIRCLE),
        (Button::West, btn::SQUARE),
        (Button::North, btn::TRIANGLE),
        (Button::LeftShoulder, btn::L1),
        (Button::RightShoulder, btn::R1),
        (Button::Back, btn::SELECT),
        (Button::Start, btn::START),
        (Button::LeftStick, btn::L3),
        (Button::RightStick, btn::R3),
        (Button::DPadUp, btn::UP),
        (Button::DPadDown, btn::DOWN),
        (Button::DPadLeft, btn::LEFT),
        (Button::DPadRight, btn::RIGHT),
    ];
    for (b, bit) in map {
        if g.button(b) {
            s.buttons |= bit;
        }
    }
    // the DualShock 2's L2 and R2 are buttons: the triggers count when they are pressed a quarter of the way
    if g.axis(Axis::TriggerLeft) > 8000 {
        s.buttons |= btn::L2;
    }
    if g.axis(Axis::TriggerRight) > 8000 {
        s.buttons |= btn::R2;
    }
    s.sticks = [
        stick_byte(g.axis(Axis::RightX), deadzone, sensitivity),
        stick_byte(g.axis(Axis::RightY), deadzone, sensitivity),
        stick_byte(g.axis(Axis::LeftX), deadzone, sensitivity),
        stick_byte(g.axis(Axis::LeftY), deadzone, sensitivity),
    ];
    s
}
