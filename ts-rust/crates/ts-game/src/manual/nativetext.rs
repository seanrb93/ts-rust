//! The game's texts for a PC: the console's memory card and the PlayStation are not mentioned (the saves are the program's own data), and
//! the buttons in the legends are named for the device in use.
//!
//! The strings are replaced in memory when the program starts (`native_text_init`: the table of English texts is patched, the executable
//! is not).  The game's texts show a button as `^` and a digit (^1 circle, ^2 cross, ^3 triangle, ^4 square, drawn as a picture of the
//! font); `button_text` (called by `textPrint` and `textSize`, so the text is drawn and measured the same) puts the name of the button
//! of the device in its place when that is not a PlayStation controller: (A), (B), (X), (Y) for an Xbox controller, the same places with
//! the Switch's letters, the keys of the keyboard.

use crate::*;
use std::sync::Mutex;

const TEXT_AT: i32 = 0x240_0000 + 0x20000;
const TEXT_END: i32 = TEXT_AT + 0x10000;

fn read(mut at: i32) -> String {
    let mut s = String::new();
    while lbu(at) != 0 && s.len() < 600 {
        s.push(lbu(at) as u8 as char);
        at += 1;
    }
    s
}

fn write(at: &mut i32, s: &str) -> i32 {
    let start = *at;
    for b in s.bytes() {
        sb(*at, b as i32);
        *at += 1;
    }
    sb(*at, 0);
    *at += 1;
    *at = (*at + 3) & !3;
    start
}

/// The memory card and the console are not in the texts: what is left of them names the saved data.
fn native(s: &str) -> Option<String> {
    let mut t = s.to_string();
    // (the legends that offered a change of card: nothing to change)
    for legend in ["   ^4 Change Memory Card (PS2)", "^4 Check Memory Card (PS2)(s)\n"] {
        t = t.replace(legend, "");
    }
    for (from, to) in [
        // (the longer texts first: the shorter ones are parts of them)
        ("No Memory Card (PS2) inserted\nPlease insert Memory Card (PS2)", "No saved data found"),
        ("No Memory Card (PS2) inserted.", "No saved data found."),
        ("No Memory Card (PS2) inserted\n", "No saved data found\n"),
        ("No Memory Card (PS2) inserted", "No saved data found"),
        ("free space on the Memory Card (PS2).", "free space for saving."),
        ("already exists on Memory Card (PS2)", "already exists in the saved data"),
        ("from Memory Card (PS2) in", "from the saved data in"),
        ("on Memory Card (PS2) in", "in the saved data in"),
        ("Memory Card (PS2) in", "Saved data in"),
        ("DO NOT REMOVE MEMORY CARD (PS2)!", "SAVING - DO NOT CLOSE THE GAME!"),
        ("Insert Memory Card (PS2) and press ^4.", "The saved data could not be found."),
        ("Please insert Memory Card (PS2) containing", "Please restore the saved data containing"),
        ("There is no Memory Card (PS2)", "There is no saved data"),
        ("There is no MEMORY CARD", "There is no saved data"),
        ("Select Memory Card (PS2)", "Select Saved Data"),
        ("MEMORY CARD (PS2)", "SAVED DATA"),
        ("MEMORY CARD slot", "SAVE SLOT"),
        ("Memory Card (PS2)", "Saved data"),
        ("MEMORY CARD", "SAVED DATA"),
    ] {
        t = t.replace(from, to);
    }
    // (what the console's cards asked: a card that is not formatted, taken out and put in again)
    t = match t.as_str() {
        "is unformatted." => "has no saved data.".to_string(),
        "Do you want to format ?" => "Do you want to create it ?".to_string(),
        "Formatting" => "Preparing".to_string(),
        "from memory ?" => "from the saved data ?".to_string(),
        "has been removed." => "Can't be found".to_string(),
        "Control Pad" => "Controller".to_string(),
        _ => t,
    };
    (t != s).then_some(t)
}

/// Replace the texts of the memory card and the console in the table of English texts.
pub fn native_text_init() {
    let table = lw(g::languages);
    let mut at = TEXT_AT;
    for i in 0..1246 {
        let p = lw(table + 4 * i);
        if !(0x100000..0x2000000).contains(&p) {
            continue;
        }
        let s = read(p);
        if std::env::var_os("TS_TEXTDUMP").is_some() {
            eprintln!("[textdump] {i}: {:?}", native(&s).unwrap_or_else(|| s.clone()));
        }
        if let Some(t) = native(&s) {
            if std::env::var_os("TS_TEXTLOG").is_some() {
                eprintln!("[text] {i}: {s:?} -> {t:?}");
            }
            if at + t.len() as i32 + 8 < TEXT_END {
                sw(table + 4 * i, write(&mut at, &t));
            }
        }
    }
}

/// The name of the button `code` (1 circle, 2 cross, 3 triangle, 4 square) for the device in use, if it is not a PlayStation controller.
fn name(code: u8) -> Option<String> {
    let names: [String; 4] = match ts_sdk::pad::input_style() {
        1 => ["B", "A", "Y", "X"].map(String::from),
        2 => ["A", "B", "X", "Y"].map(String::from),
        // (the keyboard: the first key of each button, as set in the port's settings: cross, circle, triangle, square are 0, 1, 2, 3)
        3 => [1, 0, 2, 3].map(ts_sdk::pad::key_short),
        _ => return None,
    };
    names.get((code as usize).wrapping_sub(1)).cloned().filter(|n| !n.is_empty())
}

/// The words for the buttons of a controller or the keyboard in the names of the controls (L1, Cross, Left Analog Stick ...).
pub fn style_words(s: &str) -> String {
    let style = ts_sdk::pad::input_style();
    if style == 0 {
        return s.to_string();
    }
    let key = |a: usize, mouse: &str| {
        let k = ts_sdk::pad::key_label(a);
        if mouse.is_empty() {
            k
        } else {
            format!("{mouse} / {k}")
        }
    };
    let (cross, circle, square, triangle) = match style {
        1 => ("A".to_string(), "B".to_string(), "X".to_string(), "Y".to_string()),
        2 => ("B".to_string(), "A".to_string(), "Y".to_string(), "X".to_string()),
        _ => (key(0, ""), key(1, ""), key(3, ""), key(2, "")),
    };
    let (l1, r1, l2, r2, start, select) = match style {
        1 => ("LB".to_string(), "RB".to_string(), "LT".to_string(), "RT".to_string(), "Menu".to_string(), "View".to_string()),
        2 => ("L".to_string(), "R".to_string(), "ZL".to_string(), "ZR".to_string(), "+".to_string(), "-".to_string()),
        // (the mouse's buttons are the pad's too: the right one is R1, the left one R2, the middle one aims)
        _ => (key(4, ""), key(6, "Right Mouse"), key(5, "Middle Mouse"), key(7, "Left Mouse"), key(8, ""), key(9, "")),
    };
    let mut t = s.to_string();
    if style == 3 {
        t = t
            .replace("Left Analog Stick", "WASD")
            .replace("Right Analog Stick", "Mouse")
            .replace("Up directional button", &key(10, ""))
            .replace("Down directional button", &key(11, ""))
            .replace("Left directional button", &key(12, ""))
            .replace("Right directional button", &key(13, ""));
    } else {
        t = t.replace("Left Analog Stick", "Left Stick").replace("Right Analog Stick", "Right Stick");
    }
    // (whole words only: "Select Weapon" is a control of its own)
    let whole = t.clone();
    t.split(' ')
        .map(|w| match w {
            "Triangle" => triangle.as_str(),
            "Circle" => circle.as_str(),
            "Square" => square.as_str(),
            "Cross" => cross.as_str(),
            "L1" => l1.as_str(),
            "R1" => r1.as_str(),
            "L2" => l2.as_str(),
            "R2" => r2.as_str(),
            "Start" => start.as_str(),
            "Select" if !whole.contains("Select ") => select.as_str(),
            other => other,
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// (source text, translated copy, style): the copies live in a ring of places of memory
static CACHE: Mutex<(Vec<(i32, i32, u8)>, i32)> = Mutex::new((Vec::new(), 0));
const RING_AT: i32 = 0x240_0000 + 0x30000;
const RING_SLOTS: i32 = 64;
const SLOT_SIZE: i32 = 0x400;

/// The text with the pictures of the buttons (^ and a digit) written as names, for the device in use; the text itself when it has no
/// buttons or the device is a PlayStation controller.
pub fn button_text(text: i32) -> i32 {
    let style = ts_sdk::pad::input_style();
    if style == 0 || text == 0 {
        return text;
    }
    // (a quick look for a ^, or for the texts that name the controller's buttons: the help of the MapMaker, each line of it)
    let src = read(text);
    let has = src.contains('^');
    let help = src.contains("analog") || src.contains(" button - ") || src.contains("PRESS START");
    if !has && !help {
        return text;
    }
    if std::env::var_os("TS_BTNLOG").is_some() {
        eprintln!("[btn] {:?}", src);
    }
    let mut cache = CACHE.lock().unwrap();
    // (the same text as one of the copies that are still there)
    let mut out = String::new();
    let mut at = text;
    while lbu(at) != 0 && at - text < 600 {
        let c = lbu(at);
        let next = lbu(at + 1);
        if c == 94 && (b'1' as i32..=b'4' as i32).contains(&next) {
            out.push('(');
            out.push_str(&name((next - b'0' as i32) as u8).unwrap_or_else(|| "?".to_string()));
            out.push(')');
            at += 2;
        } else {
            out.push(c as u8 as char);
            at += 1;
        }
    }
    if help {
        // ("left analog stick - Move cursor", "R1 / R2 button - Zoom Out / In": the names of the device's buttons)
        let (l1, r1, l2, r2) = match style {
            1 => ("LB".to_string(), "RB".to_string(), "LT".to_string(), "RT".to_string()),
            2 => ("L".to_string(), "R".to_string(), "ZL".to_string(), "ZR".to_string()),
            _ => (ts_sdk::pad::key_short(4), ts_sdk::pad::key_short(6), ts_sdk::pad::key_short(5), ts_sdk::pad::key_short(7)),
        };
        out = out.replace("analog stick", "stick").replace("L1", &l1).replace("L2", &l2).replace("R1", &r1).replace("R2", &r2);
        // (the title screen: the start button)
        let start = match style {
            1 => "START".to_string(),
            2 => "+".to_string(),
            _ => ts_sdk::pad::key_label(8).to_ascii_uppercase(),
        };
        out = out.replace("PRESS START BUTTON", &if style == 3 { format!("PRESS {start}") } else { format!("PRESS {start} BUTTON") });
    }
    if let Some(&(_, dst, _)) = cache.0.iter().find(|e| e.0 == text && e.2 == style) {
        if read(dst) == out {
            return dst;
        }
    }
    let slot = cache.1;
    cache.1 = (slot + 1) % RING_SLOTS;
    let dst = RING_AT + slot * SLOT_SIZE;
    let mut p = dst;
    write(&mut p, &out[..out.len().min(SLOT_SIZE as usize - 4)]);
    cache.0.retain(|e| e.1 != dst);
    cache.0.push((text, dst, style));
    dst
}
