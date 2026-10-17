//! F1 settings menu: drawn with the debug text overlay, driven by the keyboard (arrows, Enter, Esc / F1 to close).
//! Changes apply immediately; closing the menu writes them to `ts-port.ini`.

use crate::config::{self, Config, WindowMode};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;
use winit::keyboard::KeyCode;

pub struct Menu {
    pub open: bool,
    pub sel: usize,
}

pub static MENU: Mutex<Menu> = Mutex::new(Menu { open: false, sel: 0 });
/// MSAA sample counts the GPU offers, as a bit mask (set by the window once the renderer exists)
pub static MAX_MSAA: AtomicU32 = AtomicU32::new(1 << 1);

/// aspect ratios x 1000 (0 = follow the window)
const ASPECTS: [i64; 5] = [0, 1333, 1600, 1778, 2333];
const FOVS: [f32; 8] = [0.0, 70.0, 80.0, 90.0, 100.0, 110.0, 120.0, 130.0];
const SCALES: [f32; 7] = [0.5, 0.75, 1.0, 1.5, 2.0, 3.0, 4.0];
const AF: [u16; 5] = [1, 2, 4, 8, 16];
const ITEMS: usize = 11;
/// game frame rates (0 = the monitor's refresh rate)
const RATES: [u32; 8] = [60, 90, 120, 144, 165, 240, 360, 0];

fn step<T: PartialEq + Copy>(list: &[T], cur: T, dir: i32) -> T {
    let i = list.iter().position(|x| *x == cur).unwrap_or(0) as i32;
    list[(i + dir).rem_euclid(list.len() as i32) as usize]
}

fn nearest_f32(list: &[f32], cur: f32) -> f32 {
    *list.iter().min_by(|a, b| (**a - cur).abs().partial_cmp(&(**b - cur).abs()).unwrap()).unwrap()
}

fn nearest_i64(list: &[i64], cur: i64) -> i64 {
    *list.iter().min_by_key(|a| (**a - cur).abs()).unwrap()
}

/// Window sizes offered in windowed mode (those larger than the desktop are left out).
const WINDOW_SIZES: [(u32, u32); 12] = [
    (640, 480),
    (800, 600),
    (1024, 768),
    (1280, 720),
    (1280, 960),
    (1366, 768),
    (1600, 900),
    (1600, 1200),
    (1920, 1080),
    (1920, 1440),
    (2560, 1440),
    (3840, 2160),
];

/// The resolutions to choose from in the current window mode (none in borderless mode: it is the desktop's) and the one in use.
fn resolutions(c: &Config) -> (Vec<(u32, u32)>, (u32, u32)) {
    let mon = crate::window::monitor_info();
    let desktop = if mon.desktop.0 > 0 { mon.desktop } else { (1920, 1080) };
    let (mut list, current) = match c.window_mode {
        WindowMode::Borderless => return (Vec::new(), desktop),
        WindowMode::Windowed => (WINDOW_SIZES.iter().copied().filter(|&(w, h)| w <= desktop.0 && h <= desktop.1).collect::<Vec<_>>(), (c.width, c.height)),
        WindowMode::Fullscreen => {
            let modes = if mon.modes.is_empty() { WINDOW_SIZES.iter().copied().filter(|&(w, h)| w <= desktop.0 && h <= desktop.1).collect() } else { mon.modes.clone() };
            (modes, if c.fs_width == 0 || c.fs_height == 0 { desktop } else { (c.fs_width, c.fs_height) })
        }
    };
    if !list.contains(&current) {
        list.push(current);
    }
    list.sort_by_key(|&(w, h)| (w * h, w));
    (list, current)
}

fn set_resolution(c: &mut Config, size: (u32, u32)) {
    match c.window_mode {
        WindowMode::Windowed => (c.width, c.height) = size,
        WindowMode::Fullscreen => (c.fs_width, c.fs_height) = size,
        WindowMode::Borderless => {}
    }
}

fn resolution_name(size: (u32, u32)) -> String {
    if size == (640, 480) {
        format!("{}x{} (Original)", size.0, size.1)
    } else {
        format!("{}x{}", size.0, size.1)
    }
}

fn step_resolution(c: &mut Config, dir: i32) {
    let (list, cur) = resolutions(c);
    if list.is_empty() {
        return;
    }
    let i = list.iter().position(|&r| r == cur).unwrap_or(0) as i32;
    set_resolution(c, list[(i + dir).rem_euclid(list.len() as i32) as usize]);
}

fn step_window_mode(c: &mut Config, dir: i32) {
    use WindowMode::*;
    let all = [Windowed, Borderless, Fullscreen];
    let i = all.iter().position(|&m| m == c.window_mode).unwrap_or(0) as i32;
    c.window_mode = all[(i + dir).rem_euclid(3) as usize];
}

fn on_off(b: bool) -> &'static str {
    if b {
        "on"
    } else {
        "off"
    }
}

pub fn text(c: &Config, sel: usize) -> String {
    let max = MAX_MSAA.load(Ordering::Relaxed);
    let fov = if c.fov > 1.0 { format!("{:.0}", c.fov) } else { "game default".into() };
    let aa = if c.msaa_eff(max) <= 1 { "off".to_string() } else { format!("{}x", c.msaa_eff(max)) };
    let rows: [(&str, String); ITEMS] = [
        ("Aspect ratio", c.aspect_name()),
        ("Field of view", fov),
        ("HUD placement", config::hud_name(c.hud).into()),
        ("Window mode", c.window_mode.name().into()),
        ("Resolution", if c.window_mode == WindowMode::Borderless { "desktop".into() } else { resolution_name(resolutions(c).1) }),
        ("VSync", on_off(c.vsync).into()),
        ("Game frame rate", if c.framerate == 0 { "monitor".into() } else if c.framerate == 60 { "60 (original)".into() } else { c.framerate.to_string() }),
        ("Render scale", format!("{}x", c.render_scale)),
        ("Anti-aliasing", aa),
        ("Texture filter", config::filter_name(c.filter).into()),
        ("Anisotropic", if c.aniso > 1 { format!("{}x", c.aniso) } else { "off".into() }),
    ];
    let mut s = String::from("SETTINGS    up/down select, left/right change, F1 or Esc closes\n");
    for (i, (k, v)) in rows.iter().enumerate() {
        s.push_str(&format!("{} {:<16}{}\n", if i == sel { '>' } else { ' ' }, k, v));
    }
    s
}

fn adjust(c: &mut Config, idx: usize, dir: i32) {
    let max = MAX_MSAA.load(Ordering::Relaxed);
    match idx {
        0 => {
            let cur = c.aspect.map(|v| (v * 1000.0).round() as i64).unwrap_or(0);
            let next = step(&ASPECTS, nearest_i64(&ASPECTS, cur), dir);
            c.aspect = if next == 0 { None } else { Some(next as f32 / 1000.0) };
        }
        1 => c.fov = step(&FOVS, nearest_f32(&FOVS, c.fov), dir),
        2 => {
            use ts_render::HudMode::*;
            c.hud = match (c.hud, dir > 0) {
                (Edges, true) | (Stretch, false) => Center,
                (Center, true) | (Edges, false) => Stretch,
                _ => Edges,
            };
        }
        3 => step_window_mode(c, dir),
        4 => step_resolution(c, dir),
        5 => c.vsync = !c.vsync,
        6 => c.framerate = step(&RATES, RATES.iter().copied().min_by_key(|v| (*v as i64 - c.framerate as i64).abs()).unwrap_or(60), dir),
        7 => c.render_scale = step(&SCALES, nearest_f32(&SCALES, c.render_scale), dir),
        8 => {
            let opts: Vec<u32> = [1u32, 2, 4, 8].into_iter().filter(|&n| (max >> n) & 1 != 0).collect();
            c.msaa = step(&opts, c.msaa_eff(max), dir);
        }
        9 => {
            use ts_render::Filter::*;
            c.filter = step(&[Nearest, Bilinear, Trilinear], c.filter, dir);
        }
        10 => c.aniso = step(&AF, c.aniso, dir),
        _ => {}
    }
}

/// Handle a key press; returns true when the menu consumed it.
pub fn key(code: KeyCode) -> bool {
    let mut m = MENU.lock().unwrap();
    if !m.open {
        if code == KeyCode::F1 {
            m.open = true;
            return true;
        }
        return false;
    }
    match code {
        KeyCode::F1 | KeyCode::Escape => {
            m.open = false;
            drop(m);
            config::get().save();
        }
        KeyCode::ArrowUp => m.sel = (m.sel + ITEMS - 1) % ITEMS,
        KeyCode::ArrowDown => m.sel = (m.sel + 1) % ITEMS,
        KeyCode::ArrowLeft | KeyCode::ArrowRight | KeyCode::Enter => {
            let (sel, dir) = (m.sel, if code == KeyCode::ArrowLeft { -1 } else { 1 });
            drop(m);
            config::update(|c| adjust(c, sel, dir));
        }
        _ => {}
    }
    true
}

pub fn is_open() -> bool {
    MENU.lock().unwrap().open
}

pub fn current_text() -> String {
    let sel = MENU.lock().unwrap().sel;
    text(&config::get(), sel)
}

// ---- the same settings as rows of the game's Audio / Video Options page --------------------------------------------------

/// Supplies the graphics settings to the game's own options page (see `ts_game::av_menu`).
pub struct PortAv;

/// The resolution chosen on the options page but not accepted yet.
static PENDING_RES: std::sync::Mutex<Option<(u32, u32)>> = std::sync::Mutex::new(None);

const ASPECT_NAMES: [&str; 5] = ["Auto", "4:3", "16:10", "16:9", "21:9"];
const HUD_NAMES: [&str; 3] = ["Edges", "Centre", "Stretch"];

fn msaa_options() -> Vec<u32> {
    let max = MAX_MSAA.load(Ordering::Relaxed);
    [1u32, 2, 4, 8].into_iter().filter(|&n| (max >> n) & 1 != 0 || n == 1).collect()
}

fn nearest_idx_f32(list: &[f32], cur: f32) -> usize {
    list.iter().enumerate().min_by(|a, b| (*a.1 - cur).abs().partial_cmp(&(*b.1 - cur).abs()).unwrap()).map(|x| x.0).unwrap_or(0)
}

fn names<T>(list: &[T], f: impl Fn(&T) -> String) -> Vec<String> {
    list.iter().map(f).collect()
}

impl ts_game::AvOptions for PortAv {
    fn rows(&self) -> Vec<ts_game::AvRow> {
        use ts_game::AvRow;
        let c = config::get();
        let max = MAX_MSAA.load(Ordering::Relaxed);
        let aspect_cur = c.aspect.map(|v| (v * 1000.0).round() as i64).unwrap_or(0);
        let aspect_idx = ASPECTS.iter().enumerate().min_by_key(|(_, a)| (**a - aspect_cur).abs()).map(|x| x.0).unwrap_or(0);
        let rate_idx = RATES.iter().enumerate().min_by_key(|(_, v)| (**v as i64 - c.framerate as i64).abs()).map(|x| x.0).unwrap_or(0);
        let aa = msaa_options();
        let aa_idx = aa.iter().position(|&n| n == c.msaa_eff(max)).unwrap_or(0);
        let aniso_idx = AF.iter().position(|&n| n == c.aniso).unwrap_or(0);
        let onoff = || vec!["Off".to_string(), "On".to_string()];
        let row = |label: &str, values: Vec<String>, index: usize| AvRow { label: label.into(), values, index };
        let (res, res_cur) = resolutions(&c);
        // (a resolution being chosen is shown, but only used once it is accepted)
        let res_shown = PENDING_RES.lock().unwrap().filter(|p| res.contains(p)).unwrap_or(res_cur);
        let res_row = if res.is_empty() {
            row("Resolution", vec!["Desktop".into()], 0)
        } else {
            row("Resolution", names(&res, |r| resolution_name(*r)), res.iter().position(|&r| r == res_shown).unwrap_or(0))
        };
        vec![
            row("Window Mode", vec!["Windowed".into(), "Borderless".into(), "Fullscreen".into()], match c.window_mode {
                WindowMode::Windowed => 0,
                WindowMode::Borderless => 1,
                WindowMode::Fullscreen => 2,
            }),
            res_row,
            row("Aspect Ratio", ASPECT_NAMES.iter().map(|s| s.to_string()).collect(), aspect_idx),
            row("Field of View", names(&FOVS, |f| if *f > 1.0 { format!("{f:.0}") } else { "Default".into() }), nearest_idx_f32(&FOVS, c.fov)),
            row("HUD Placement", HUD_NAMES.iter().map(|s| s.to_string()).collect(), match c.hud {
                ts_render::HudMode::Edges => 0,
                ts_render::HudMode::Center => 1,
                ts_render::HudMode::Stretch => 2,
            }),
            row("VSync", onoff(), c.vsync as usize),
            row("Frame Rate", names(&RATES, |r| if *r == 0 { "Monitor".into() } else if *r == 60 { "60 (Original)".into() } else { r.to_string() }), rate_idx),
            row("Render Scale", names(&SCALES, |s| format!("{s}x")), nearest_idx_f32(&SCALES, c.render_scale)),
            row("Anti-Aliasing", names(&aa, |n| if *n <= 1 { "Off".into() } else { format!("{n}x") }), aa_idx),
            row("Texture Filter", vec!["Nearest".into(), "Bilinear".into(), "Trilinear".into()], match c.filter {
                ts_render::Filter::Nearest => 0,
                ts_render::Filter::Bilinear => 1,
                ts_render::Filter::Trilinear => 2,
            }),
            row("Anisotropic", names(&AF, |n| if *n <= 1 { "Off".into() } else { format!("{n}x") }), aniso_idx),
        ]
    }

    fn game_rows(&self) -> Vec<ts_game::AvRow> {
        let c = config::get();
        let onoff = || vec!["Off".to_string(), "On".to_string()];
        vec![
            ts_game::AvRow { label: "Unlock Everything".into(), values: onoff(), index: c.unlock_all as usize },
            ts_game::AvRow { label: "Debug Menu".into(), values: onoff(), index: c.debug_menu as usize },
            ts_game::AvRow { label: "Dual Wield Everything".into(), values: onoff(), index: c.dual_all as usize },
            ts_game::AvRow { label: "Bots Punch".into(), values: onoff(), index: c.bot_punch as usize },
        ]
    }

    fn set_game(&self, row: usize, index: usize) {
        config::update(|c| {
            match row {
                0 => c.unlock_all = index != 0,
                1 => c.debug_menu = index != 0,
                2 => c.dual_all = index != 0,
                3 => c.bot_punch = index != 0,
                _ => {}
            }
        });
        config::get().save();
    }

    /// The player accepted the value of a row (X).
    fn commit(&self, row: usize) {
        if row == 1 {
            if let Some(size) = PENDING_RES.lock().unwrap().take() {
                config::update(|c| set_resolution(c, size));
                config::get().save();
            }
        }
    }

    /// The player gave up the change of a row (triangle).
    fn cancel(&self, row: usize) {
        if row == 1 {
            *PENDING_RES.lock().unwrap() = None;
        }
    }

    fn set(&self, row: usize, index: usize) {
        config::update(|c| match row {
            0 => {
                *PENDING_RES.lock().unwrap() = None;
                c.window_mode = [WindowMode::Windowed, WindowMode::Borderless, WindowMode::Fullscreen][index.min(2)]
            }
            1 => {
                // not applied yet: switching the picture's size every step of the choice blacks out the screen
                let (list, _) = resolutions(c);
                *PENDING_RES.lock().unwrap() = list.get(index).copied();
            }
            2 => c.aspect = if ASPECTS[index.min(4)] == 0 { None } else { Some(ASPECTS[index.min(4)] as f32 / 1000.0) },
            3 => c.fov = FOVS[index.min(FOVS.len() - 1)],
            4 => {
                use ts_render::HudMode::*;
                c.hud = [Edges, Center, Stretch][index.min(2)];
            }
            5 => c.vsync = index != 0,
            6 => c.framerate = RATES[index.min(RATES.len() - 1)],
            7 => c.render_scale = SCALES[index.min(SCALES.len() - 1)],
            8 => {
                let opts = msaa_options();
                c.msaa = opts[index.min(opts.len() - 1)];
            }
            9 => {
                use ts_render::Filter::*;
                c.filter = [Nearest, Bilinear, Trilinear][index.min(2)];
            }
            10 => c.aniso = AF[index.min(AF.len() - 1)],
            _ => {}
        });
        config::get().save();
    }
}
