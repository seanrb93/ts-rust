//! The port's settings as rows of the game's own option pages (Video Options, Game Options, Controls).

use crate::config::{self, Config, WindowMode};
use std::sync::atomic::{AtomicU32, Ordering};

/// MSAA sample counts the GPU offers, as a bit mask (set by the window once the renderer exists)
pub static MAX_MSAA: AtomicU32 = AtomicU32::new(1 << 1);

/// aspect ratios x 1000 (0 = follow the window)
const ASPECTS: [i64; 5] = [0, 1333, 1600, 1778, 2333];
const FOVS: [f32; 8] = [0.0, 70.0, 80.0, 90.0, 100.0, 110.0, 120.0, 130.0];
const AF: [u16; 5] = [1, 2, 4, 8, 16];
/// The game frame rates to choose from: 60 (the original), the refresh rates the monitor has video modes for and above 60 (any of them is a rate it can show), and 0 for the
/// monitor's own current rate.
fn rates() -> Vec<u32> {
    let mut list = vec![60];
    list.extend(crate::window::monitor_info().rates.into_iter().filter(|&r| r > 60));
    list.push(0);
    list
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

/// the values of the sensitivity rows of the Controls page (1 = as the game has it)
const SENS: [f32; 10] = [0.25, 0.5, 0.75, 1.0, 1.25, 1.5, 2.0, 2.5, 3.0, 4.0];

impl ts_game::AvOptions for PortAv {
    fn rows(&self) -> Vec<ts_game::AvRow> {
        use ts_game::AvRow;
        let c = config::get();
        let max = MAX_MSAA.load(Ordering::Relaxed);
        let aspect_cur = c.aspect.map(|v| (v * 1000.0).round() as i64).unwrap_or(0);
        let aspect_idx = ASPECTS.iter().enumerate().min_by_key(|(_, a)| (**a - aspect_cur).abs()).map(|x| x.0).unwrap_or(0);
        let rates = rates();
        // (a rate that the monitor does not have, from an old settings file, is shown as the nearest of those it has)
        let rate_idx = rates.iter().enumerate().filter(|(_, v)| **v != 0).min_by_key(|(_, v)| (**v as i64 - c.framerate as i64).abs()).map(|x| x.0).unwrap_or(0);
        let rate_idx = if c.framerate == 0 { rates.len() - 1 } else { rate_idx };
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
            row("Frame Rate", names(&rates, |r| if *r == 0 { "Monitor".into() } else if *r == 60 { "60 (Original)".into() } else { r.to_string() }), rate_idx),
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
            ts_game::AvRow { label: "Debug Menu".into(), values: onoff(), index: c.debug_menu as usize },
            ts_game::AvRow { label: "FPS Counter".into(), values: onoff(), index: c.show_fps as usize },
        ]
    }

    fn control_rows(&self) -> Vec<ts_game::AvRow> {
        let c = config::get();
        let pct = |k: f32| SENS.iter().position(|&s| (s - k).abs() < 0.001).unwrap_or_else(|| SENS.iter().enumerate().min_by(|a, b| (a.1 - k).abs().partial_cmp(&(b.1 - k).abs()).unwrap()).map_or(3, |x| x.0));
        let values = || SENS.iter().map(|s| format!("{}x", s)).collect::<Vec<_>>();
        vec![
            ts_game::AvRow { label: "Look / Mouse Sensitivity".into(), values: values(), index: pct(c.look_sensitivity) },
            ts_game::AvRow { label: "Control Method".into(), values: vec!["Controller".into(), "Keyboard & Mouse".into()], index: c.input_method.min(1) as usize },
        ]
    }

    fn set_control(&self, row: usize, index: usize) {
        config::update(|c| match row {
            0 => c.look_sensitivity = SENS[index.min(SENS.len() - 1)],
            1 => {
                crate::window::method_chosen(c.input_method, index.min(1) as u8);
                c.input_method = index.min(1) as u8;
            }
            _ => {}
        });
        config::get().save();
    }

    fn audio_rows(&self) -> Vec<ts_game::AvRow> {
        // (sliders: the value is the volume in percent)
        let c = config::get();
        ["Weapon Volume", "Voice Volume", "Environment Volume"]
            .iter()
            .enumerate()
            .map(|(i, label)| ts_game::AvRow { label: (*label).into(), values: Vec::new(), index: c.sound_mix[i].min(100) as usize })
            .collect()
    }

    fn set_audio(&self, row: usize, percent: usize) {
        config::update(|c| {
            if let Some(v) = c.sound_mix.get_mut(row) {
                *v = percent.min(100) as u8;
            }
        });
        config::get().save();
    }

    fn level_music(&self) -> i32 {
        config::get().level_music
    }

    fn set_level_music(&self, choice: i32) {
        config::update(|c| c.level_music = choice.clamp(0, 28));
        config::get().save();
    }

    fn mod_rows(&self) -> Vec<ts_game::AvRow> {
        mod_cache(false)
            .iter()
            .map(|m| {
                // (a change shows as pending: the mods are loaded when the program starts)
                let pending = if m.enabled != m.loaded { " (after restart)" } else { "" };
                let (on, off) = if m.enabled { (format!("On{pending}"), "Off".to_string()) } else { ("On".to_string(), format!("Off{pending}")) };
                let label: String = m.title.chars().take(28).collect();
                ts_game::AvRow { label, values: vec![on, off], index: if m.enabled { 0 } else { 1 } }
            })
            .collect()
    }

    fn set_mod(&self, row: usize, index: usize) {
        let list = mod_cache(false);
        if let Some(m) = list.get(row) {
            if m.enabled != (index == 0) {
                crate::mods::set_enabled(&m.name, index == 0);
                mod_cache(true);
            }
        }
    }

    fn set_mapmaker_unlimited(&self, on: bool) {
        config::update(|c| c.mapmaker_unlimited = on);
        config::get().save();
    }

    fn set_game(&self, row: usize, index: usize) {
        config::update(|c| {
            match row {
                0 => c.debug_menu = index != 0,
                1 => c.show_fps = index != 0,
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
            6 => {
                let list = rates();
                c.framerate = list[index.min(list.len() - 1)];
            }
            7 => {
                let opts = msaa_options();
                c.msaa = opts[index.min(opts.len() - 1)];
            }
            8 => {
                use ts_render::Filter::*;
                c.filter = [Nearest, Bilinear, Trilinear][index.min(2)];
            }
            9 => c.aniso = AF[index.min(AF.len() - 1)],
            _ => {}
        });
        config::get().save();
    }
}

/// The installed mods for the Mods page, read again after a second (or when `refresh` says a change was made).
fn mod_cache(refresh: bool) -> Vec<crate::mods::Listed> {
    use std::sync::Mutex;
    static CACHE: Mutex<Option<(std::time::Instant, Vec<crate::mods::Listed>)>> = Mutex::new(None);
    let mut g = CACHE.lock().unwrap();
    let stale = g.as_ref().map_or(true, |(t, _)| refresh || t.elapsed().as_millis() > 1000);
    if stale {
        let v = crate::mods::list();
        *g = Some((std::time::Instant::now(), v));
    }
    g.as_ref().map(|(_, v)| v.clone()).unwrap_or_default()
}
