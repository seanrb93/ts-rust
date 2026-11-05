//! The pictures of the buttons for the device in use.
//!
//! The game marks a button in its texts as `^1` to `^4` (circle, cross, triangle, square) and draws a picture of the button where the marker is: the textures `mm_button01` to
//! `mm_button04`, which are PlayStation buttons.  For an Xbox or Switch controller, or the keyboard, the renderer is told to show another picture in their place
//! (`ts_gfx::hd::register_variant`, which depends on the device in use): the buttons of that controller, or the cap of the key the player gave the button.  The pictures are files of a
//! mod, in its folder `button_icons` (the Universal Controller Icons mod; `tools/make-button-icons.py` draws them).  With no picture for a button the texts name it in words.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;

/// The pictures of the mods, by name (`xbox/a`).  They stay for as long as the program runs: the renderer keeps them.
static ICONS: Mutex<Option<HashMap<String, &'static [u8]>>> = Mutex::new(None);

/// The folder `button_icons` of a mod: a folder for each device (`xbox/a.png`).  A mod after it can give a picture of the same name.
pub fn add_folder(dir: &Path) {
    let Ok(groups) = std::fs::read_dir(dir) else { return };
    let mut icons = ICONS.lock().unwrap();
    let map = icons.get_or_insert_with(HashMap::new);
    for group in groups.flatten() {
        let Ok(files) = std::fs::read_dir(group.path()) else { continue };
        for file in files.flatten() {
            let path = file.path();
            if !path.extension().map_or(false, |e| e.eq_ignore_ascii_case("png")) {
                continue;
            }
            let (Some(g), Some(stem)) = (group.file_name().to_str().map(str::to_ascii_lowercase), path.file_stem().and_then(|n| n.to_str()).map(str::to_ascii_lowercase)) else { continue };
            if let Ok(bytes) = std::fs::read(&path) {
                map.insert(format!("{g}/{stem}"), Box::leak(bytes.into_boxed_slice()));
            }
        }
    }
}

/// The texture of the button picture of each marker (`^1` to `^4`).
const TEXTURES: [&str; 4] = ["textures/misc/mm_button01", "textures/misc/mm_button02", "textures/misc/mm_button03", "textures/misc/mm_button04"];

/// The textures of the shoulder button pictures of the MapMaker (L1 and L2).
const PILLS: [&str; 2] = ["textures/misc/mm_cursor13", "textures/misc/mm_cursor14"];

fn icon(name: &str) -> Option<&'static [u8]> {
    ICONS.lock().unwrap().as_ref()?.get(name).copied()
}

/// The pictures of the Xbox and Switch buttons (circle, cross, triangle and square are B, A, Y, X on an Xbox and A, B, X, Y on a Switch), with the shoulder buttons of the MapMaker.
const DEVICES: [(u8, [&str; 4], [&str; 2]); 2] = [(1u8, ["xbox/b", "xbox/a", "xbox/y", "xbox/x"], ["pills/xbox_lb", "pills/xbox_lt"]), (2u8, ["switch/a", "switch/b", "switch/x", "switch/y"], ["pills/switch_l", "pills/switch_zl"])];

/// Start up: the picture reader.
pub fn init() {
    ts_gfx::hd::set_bytes_loader(crate::texmod::load_hd_bytes);
}

/// The mods are loaded: the pictures they have for the Xbox and Switch buttons replace the game's while that device is in use.
pub fn register() {
    for (style, names, pills) in DEVICES {
        for (code, name) in names.iter().enumerate() {
            if let Some(bytes) = icon(name) {
                ts_gfx::hd::register_variant(TEXTURES[code], style, bytes);
            }
        }
        for (i, name) in pills.iter().enumerate() {
            if let Some(bytes) = icon(name) {
                ts_gfx::hd::register_variant(PILLS[i], style, bytes);
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

/// The pill of a key (the shoulder buttons of the MapMaker's pictures).
fn key_pill(label: &str) -> Option<&'static [u8]> {
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
        other => other,
    };
    icon(&format!("pills/key_{name}"))
}

static KEYS_SEEN: Mutex<Option<(u8, [String; 6])>> = Mutex::new(None);

/// Every frame: the device in use is passed on to the renderer, the keyboard's caps follow the keys the player chose, and the texts are told which buttons have a picture.
pub fn update(style: u8) {
    ts_gfx::hd::set_style(style);
    // (the slots of the settings: cross 0, circle 1, triangle 2, square 3; the markers: circle ^1, cross ^2, triangle ^3, square ^4)
    // (and the shoulder buttons L1 and L2, slots 4 and 5, that the MapMaker draws as pills)
    let labels: [String; 6] = [1usize, 0, 2, 3, 4, 5].map(ts_sdk::pad::key_short);
    let mut seen = KEYS_SEEN.lock().unwrap();
    let changed = seen.as_ref().map_or(true, |(s, l)| *s != style || *l != labels);
    let mut mask = 0u8;
    match style {
        1 | 2 => {
            if let Some((_, names, _)) = DEVICES.iter().find(|d| d.0 == style) {
                for (code, name) in names.iter().enumerate() {
                    if icon(name).is_some() {
                        mask |= 1 << code;
                    }
                }
            }
        }
        3 => {
            if changed {
                ts_gfx::hd::clear_variants(3);
                for (code, label) in labels.iter().take(4).enumerate() {
                    if let Some(bytes) = key_icon(label) {
                        ts_gfx::hd::register_variant(TEXTURES[code], 3, bytes);
                    }
                }
                for (i, label) in labels.iter().skip(4).enumerate() {
                    if let Some(bytes) = key_pill(label) {
                        ts_gfx::hd::register_variant(PILLS[i], 3, bytes);
                    }
                }
            }
            for (code, label) in labels.iter().take(4).enumerate() {
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
