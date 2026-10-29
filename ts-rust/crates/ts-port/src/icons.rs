//! The pictures of the buttons for the device in use.
//!
//! The game marks a button in its texts as `^1` to `^4` (circle, cross, triangle, square) and draws a picture of the button where the marker is: the textures `mm_button01` to
//! `mm_button04`, which are PlayStation buttons.  For an Xbox or Switch controller, or the keyboard, the renderer is told to show another picture in their place
//! (`ts_gfx::hd::register_variant`, which depends on the device in use): the buttons of that controller, or the cap of the key the player gave the button.  The pictures are drawn
//! by `tools/make-button-icons.py` and carried in the program (`icons_data.rs`).

use std::sync::Mutex;

include!("icons_data.rs");

/// The texture of the button picture of each marker (`^1` to `^4`).
const TEXTURES: [&str; 4] = ["textures/misc/mm_button01", "textures/misc/mm_button02", "textures/misc/mm_button03", "textures/misc/mm_button04"];

fn icon(name: &str) -> Option<&'static [u8]> {
    ICONS.iter().find(|(n, _)| *n == name).map(|(_, b)| *b)
}

/// Start up: the picture reader, and the pictures of the Xbox and Switch buttons (circle, cross, triangle and square are B, A, Y, X on an Xbox and A, B, X, Y on a Switch).
pub fn init() {
    ts_gfx::hd::set_bytes_loader(crate::texmod::load_hd_bytes);
    for (style, names) in [(1u8, ["xbox/b", "xbox/a", "xbox/y", "xbox/x"]), (2u8, ["switch/a", "switch/b", "switch/x", "switch/y"])] {
        for (code, name) in names.iter().enumerate() {
            if let Some(bytes) = icon(name) {
                ts_gfx::hd::register_variant(TEXTURES[code], style, bytes);
            }
        }
    }
}

/// The name of the cap picture of a key as the settings name it ("F", "Enter", "LeftCtrl", "PageUp" ...).
fn key_icon(label: &str) -> Option<&'static [u8]> {
    let l = label.trim().to_ascii_lowercase();
    let name = match l.as_str() {
        "leftctrl" | "rightctrl" | "lctrl" | "rctrl" | "control" => "ctrl",
        "leftshift" | "rightshift" => "shift",
        "leftalt" | "rightalt" => "alt",
        "backspace" => "back",
        "escape" => "esc",
        "pageup" => "pgup",
        "pagedown" => "pgdn",
        "insert" => "ins",
        "delete" => "del",
        "leftbracket" => "lbracket",
        "rightbracket" => "rbracket",
        other => other,
    };
    icon(&format!("keys_sq/{name}"))
}

static KEYS_SEEN: Mutex<Option<(u8, [String; 4])>> = Mutex::new(None);

/// Every frame: the device in use is passed on to the renderer, the keyboard's caps follow the keys the player chose, and the texts are told which buttons have a picture.
pub fn update(style: u8) {
    ts_gfx::hd::set_style(style);
    // (the slots of the settings: cross 0, circle 1, triangle 2, square 3; the markers: circle ^1, cross ^2, triangle ^3, square ^4)
    let labels: [String; 4] = [1usize, 0, 2, 3].map(ts_sdk::pad::key_short);
    let mut seen = KEYS_SEEN.lock().unwrap();
    let changed = seen.as_ref().map_or(true, |(s, l)| *s != style || *l != labels);
    let mut mask = 0u8;
    match style {
        1 | 2 => mask = 0b1111,
        3 => {
            if changed {
                ts_gfx::hd::clear_variants(3);
                for (code, label) in labels.iter().enumerate() {
                    if let Some(bytes) = key_icon(label) {
                        ts_gfx::hd::register_variant(TEXTURES[code], 3, bytes);
                    }
                }
            }
            for (code, label) in labels.iter().enumerate() {
                if key_icon(label).is_some() {
                    mask |= 1 << code;
                }
            }
        }
        _ => {}
    }
    *seen = Some((style, labels));
    ts_sdk::pad::set_button_icons(mask);
}
