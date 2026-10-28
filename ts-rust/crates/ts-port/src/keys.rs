//! The keyboard's keys for the buttons of the virtual pad: the names for `ts-rust.ini` (`key_cross = Enter, Space, F`), the keys that stand
//! for each button by default, and the names the game's texts use for them (the legend "(Enter) Ok" shows the first key of the button).

use ts_sdk::pad::btn;
use winit::keyboard::KeyCode;

/// The buttons that have keys: (ini name, button, default keys).  The order is the one of `ts_sdk::pad::key_label`.
pub const ACTIONS: [(&str, u16, &[&str]); 14] = [
    ("cross", btn::CROSS, &["Enter", "Space", "F"]),
    ("circle", btn::CIRCLE, &["C"]),
    ("triangle", btn::TRIANGLE, &["Backspace", "R"]),
    ("square", btn::SQUARE, &["X"]),
    ("l1", btn::L1, &["LeftCtrl", "RightCtrl"]),
    ("l2", btn::L2, &["Z"]),
    ("r1", btn::R1, &["V"]),
    ("r2", btn::R2, &["G"]),
    ("start", btn::START, &["Esc", "P"]),
    ("select", btn::SELECT, &["Tab"]),
    ("up", btn::UP, &["Up"]),
    ("down", btn::DOWN, &["Down"]),
    ("left", btn::LEFT, &["Left", "Q"]),
    ("right", btn::RIGHT, &["Right", "E"]),
];

/// The default keys of a button as the strings of the ini.
pub fn default_keys(action: usize) -> Vec<String> {
    ACTIONS[action].2.iter().map(|s| s.to_string()).collect()
}

/// The key of a name (a letter, a digit, F1..F12, or Enter, Space, Backspace, Tab, Esc, Ctrl, Shift, Alt, Up, Down, Left, Right, Insert, Delete,
/// Home, End, PageUp, PageDown, and the punctuation keys by their position: Comma, Period, Slash, Semicolon, Quote, LeftBracket, RightBracket,
/// Backslash, Minus, Equals, Grave; `LeftCtrl` / `RightCtrl` and the like to tell the sides apart).
pub fn parse_key(name: &str) -> Option<KeyCode> {
    use KeyCode::*;
    let n = name.trim().to_ascii_lowercase().replace([' ', '_', '-'], "");
    let letters = [KeyA, KeyB, KeyC, KeyD, KeyE, KeyF, KeyG, KeyH, KeyI, KeyJ, KeyK, KeyL, KeyM, KeyN, KeyO, KeyP, KeyQ, KeyR, KeyS, KeyT, KeyU, KeyV, KeyW, KeyX, KeyY, KeyZ];
    let digits = [Digit0, Digit1, Digit2, Digit3, Digit4, Digit5, Digit6, Digit7, Digit8, Digit9];
    let fkeys = [F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12];
    if n.len() == 1 {
        let c = n.as_bytes()[0];
        if c.is_ascii_lowercase() {
            return Some(letters[(c - b'a') as usize]);
        }
        if c.is_ascii_digit() {
            return Some(digits[(c - b'0') as usize]);
        }
    }
    if let Some(rest) = n.strip_prefix("key") {
        if rest.len() == 1 && rest.as_bytes()[0].is_ascii_lowercase() {
            return Some(letters[(rest.as_bytes()[0] - b'a') as usize]);
        }
    }
    if let Some(rest) = n.strip_prefix("digit") {
        if let Ok(d) = rest.parse::<usize>() {
            return digits.get(d).copied();
        }
    }
    if let Some(rest) = n.strip_prefix('f') {
        if let Ok(k) = rest.parse::<usize>() {
            return (1..=12).contains(&k).then(|| fkeys[k - 1]);
        }
    }
    Some(match n.as_str() {
        "enter" | "return" => Enter,
        "space" | "spacebar" => Space,
        "backspace" | "bksp" => Backspace,
        "tab" => Tab,
        "esc" | "escape" => Escape,
        "ctrl" | "control" | "leftctrl" | "leftcontrol" | "lctrl" => ControlLeft,
        "rightctrl" | "rightcontrol" | "rctrl" => ControlRight,
        "shift" | "leftshift" | "lshift" => ShiftLeft,
        "rightshift" | "rshift" => ShiftRight,
        "alt" | "leftalt" | "lalt" => AltLeft,
        "rightalt" | "ralt" => AltRight,
        "up" | "uparrow" | "arrowup" => ArrowUp,
        "down" | "downarrow" | "arrowdown" => ArrowDown,
        "left" | "leftarrow" | "arrowleft" => ArrowLeft,
        "right" | "rightarrow" | "arrowright" => ArrowRight,
        "insert" => Insert,
        "delete" | "del" => Delete,
        "home" => Home,
        "end" => End,
        "pageup" => PageUp,
        "pagedown" => PageDown,
        "comma" => Comma,
        "period" | "dot" => Period,
        "slash" => Slash,
        "semicolon" => Semicolon,
        "quote" | "apostrophe" => Quote,
        "leftbracket" | "bracketleft" => BracketLeft,
        "rightbracket" | "bracketright" => BracketRight,
        "backslash" => Backslash,
        "minus" => Minus,
        "equals" | "equal" | "plus" => Equal,
        "grave" | "backquote" | "tilde" => Backquote,
        _ => return None,
    })
}

/// The name of a key in the game's texts.
pub fn label(code: KeyCode) -> String {
    use KeyCode::*;
    let s = match code {
        Enter => "Enter",
        Space => "Space",
        Backspace => "Bksp",
        Tab => "Tab",
        Escape => "Esc",
        ControlLeft | ControlRight => "Ctrl",
        ShiftLeft | ShiftRight => "Shift",
        AltLeft | AltRight => "Alt",
        ArrowUp => "Up Arrow",
        ArrowDown => "Down Arrow",
        ArrowLeft => "Left Arrow",
        ArrowRight => "Right Arrow",
        Insert => "Ins",
        Delete => "Del",
        Home => "Home",
        End => "End",
        PageUp => "PgUp",
        PageDown => "PgDn",
        Comma => ",",
        Period => ".",
        Slash => "/",
        Semicolon => ";",
        Quote => "'",
        BracketLeft => "[",
        BracketRight => "]",
        Backslash => "\\",
        Minus => "-",
        Equal => "=",
        Backquote => "`",
        other => {
            let d = format!("{other:?}");
            return if let Some(l) = d.strip_prefix("Key") { l.to_string() } else if let Some(n) = d.strip_prefix("Digit") { n.to_string() } else { d };
        }
    };
    s.to_string()
}

/// The keys of one button as they are written in the ini (names that are not keys are dropped).
pub fn parse_list(text: &str) -> Vec<String> {
    text.split(',').map(str::trim).filter(|n| parse_key(n).is_some()).map(str::to_string).collect()
}
