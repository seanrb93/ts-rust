//! Touch play (Android; harmless elsewhere): the left part of the screen is the left stick (walk), the right part the right stick
//! (look), a held corner fires, and a short tap presses cross (the menus).  A proof of concept: nothing is drawn on the screen.

use std::collections::HashMap;
use std::time::{Duration, Instant};
use ts_sdk::pad::{btn, PadState};

struct Finger {
    start: (f64, f64),
    now: (f64, f64),
    since: Instant,
    zone: Zone,
}

#[derive(Clone, Copy, PartialEq)]
enum Zone {
    Move,
    Look,
    Fire,
    Aim,
    Start,
}

#[derive(Default)]
pub struct Touch {
    fingers: HashMap<u64, Finger>,
    /// taps that were let go: (button, until)
    taps: Vec<(u16, Instant)>,
    size: (f64, f64),
}

impl Touch {
    pub fn set_size(&mut self, w: f64, h: f64) {
        self.size = (w, h);
    }

    fn zone(&self, x: f64, y: f64) -> Zone {
        let (w, h) = (self.size.0.max(1.0), self.size.1.max(1.0));
        let (fx, fy) = (x / w, y / h);
        if fx > 0.8 && fy > 0.75 {
            Zone::Fire
        } else if fx > 0.8 && fy > 0.5 {
            Zone::Aim
        } else if fx > 0.4 && fx < 0.6 && fy < 0.12 {
            Zone::Start
        } else if fx < 0.45 {
            Zone::Move
        } else {
            Zone::Look
        }
    }

    pub fn event(&mut self, id: u64, phase: winit::event::TouchPhase, x: f64, y: f64) {
        use winit::event::TouchPhase::*;
        match phase {
            Started => {
                let zone = self.zone(x, y);
                self.fingers.insert(id, Finger { start: (x, y), now: (x, y), since: Instant::now(), zone });
            }
            Moved => {
                if let Some(f) = self.fingers.get_mut(&id) {
                    f.now = (x, y);
                }
            }
            Ended | Cancelled => {
                if let Some(f) = self.fingers.remove(&id) {
                    let moved = (f.now.0 - f.start.0).hypot(f.now.1 - f.start.1);
                    if phase == Ended && f.since.elapsed() < Duration::from_millis(300) && moved < 30.0 {
                        let b = if f.zone == Zone::Start { btn::START } else { btn::CROSS };
                        self.taps.push((b, Instant::now() + Duration::from_millis(90)));
                    }
                }
            }
        }
    }

    pub fn active(&self) -> bool {
        !self.fingers.is_empty() || !self.taps.is_empty()
    }

    pub fn state(&mut self) -> PadState {
        let now = Instant::now();
        self.taps.retain(|t| t.1 > now);
        let mut s = PadState::idle(true);
        let reach = (self.size.1 * 0.12).max(40.0);
        let axis = |d: f64| (0x80 as f64 + (d / reach).clamp(-1.0, 1.0) * 127.0) as u8;
        for f in self.fingers.values() {
            let (dx, dy) = (f.now.0 - f.start.0, f.now.1 - f.start.1);
            match f.zone {
                Zone::Move => {
                    s.sticks[2] = axis(dx);
                    s.sticks[3] = axis(dy);
                }
                Zone::Look => {
                    s.sticks[0] = axis(dx);
                    s.sticks[1] = axis(dy);
                }
                Zone::Fire => s.buttons |= btn::R2,
                Zone::Aim => s.buttons |= btn::L2,
                Zone::Start => {}
            }
        }
        for t in &self.taps {
            s.buttons |= t.0;
        }
        s
    }
}
