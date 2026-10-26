//! User settings: `ts-port.ini` next to the executable (created with defaults on first run), overridable with
//! `--set key=value` on the command line.  Everything is read once at start-up; the F-keys in `window.rs` change a few
//! options live.

use std::path::PathBuf;
use std::sync::Mutex;

/// How the game's window is shown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowMode {
    /// an ordinary window of `width` x `height`
    Windowed,
    /// a window without borders that covers the desktop (the desktop's resolution)
    Borderless,
    /// the monitor is switched to `fullscreen_width` x `fullscreen_height`
    Fullscreen,
}

impl WindowMode {
    pub fn name(self) -> &'static str {
        match self {
            WindowMode::Windowed => "windowed",
            WindowMode::Borderless => "borderless",
            WindowMode::Fullscreen => "fullscreen",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    /// window size in windowed mode
    pub width: u32,
    pub height: u32,
    pub window_mode: WindowMode,
    /// resolution in fullscreen mode (0 = the desktop's)
    pub fs_width: u32,
    pub fs_height: u32,
    pub vsync: bool,
    /// "auto" (window shape), or a ratio such as 4:3 / 16:9 / 1.78
    pub aspect: Option<f32>,
    pub render_scale: f32,
    /// internal resolution as a height (0 = the window's size times the render scale)
    pub internal_height: u32,
    /// the internal image is scaled up with sharp pixels
    pub upscale_sharp: bool,
    /// the longest side of a picture of a mod that replaces a texture (a bigger one is made smaller when it is read)
    pub hd_texture_max: u32,
    pub msaa: u32,
    pub filter: ts_render::Filter,
    pub aniso: u16,
    /// horizontal field of view in degrees at the chosen aspect; 0 = the game's own
    pub fov: f32,
    /// placement of HUD and menus in a picture wider than 4:3
    pub hud: ts_render::HudMode,
    /// frame rate of the game itself while a level is played: 60 = the original, higher = the game logic really runs at
    /// that rate, 0 = the refresh rate of the monitor
    pub framerate: u32,
    /// gamepad sticks as PCSX2 treats them by default: dead zone (0 = none) and sensitivity (1.33 = the stick's range is stretched by a third)
    pub stick_deadzone: f32,
    pub stick_sensitivity: f32,
    /// turning with the mouse: 1 = 0.12 degrees for a count of the mouse (at the game's normal field of view)
    pub mouse_sensitivity: f32,
    /// everything the game keeps locked is open
    pub unlock_all: bool,
    /// F2 opens the developer menu in a level
    pub debug_menu: bool,
    /// the dual versions of the weapons (and the two fists) are on
    pub dual_all: bool,
    /// bots without a weapon chase and punch
    pub bot_punch: bool,
    /// the frame rate and timing figures are drawn over the picture
    pub show_fps: bool,
    /// turning speed of the right stick as a factor
    pub look_sensitivity: f32,
    /// the MapMaker has no limit of what a map holds
    pub mapmaker_unlimited: bool,
    /// 0 controller, 1 keyboard and mouse
    pub input_method: u8,
    /// the keys of each button of the pad, as the names of the ini (see `keys::ACTIONS`)
    pub keys: Vec<Vec<String>>,
    /// volume of the groups of sound effects in percent (weapons, voices, environment), 100 = as the game has it
    pub sound_mix: [u8; 3],
    /// the music of the levels: 0 = the level's own, 1 = a random track each time, n + 1 = track n (1 to 27); the same numbers as the row of the arcade Options page
    pub level_music: i32,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            width: 1280,
            height: 960,
            window_mode: WindowMode::Windowed,
            fs_width: 0,
            fs_height: 0,
            vsync: false,
            aspect: None,
            render_scale: 1.0,
            internal_height: 0,
            upscale_sharp: false,
            hd_texture_max: 4096,
            msaa: 1,
            filter: ts_render::Filter::Bilinear,
            aniso: 1,
            fov: 0.0,
            hud: ts_render::HudMode::Edges,
            framerate: 60,
            stick_deadzone: 0.0,
            stick_sensitivity: 1.33,
            mouse_sensitivity: 1.0,
            unlock_all: false,
            debug_menu: false,
            dual_all: true,
            bot_punch: true,
            show_fps: false,
            look_sensitivity: 1.0,
            mapmaker_unlimited: false,
            input_method: 1,
            keys: (0..14).map(crate::keys::default_keys).collect(),
            sound_mix: [100; 3],
            level_music: 0,
        }
    }
}

const TEMPLATE: &str = "\
# TimeSplitters native port - settings.  Delete this file to get the defaults back.
# Command line: --set key=value (e.g. --set msaa=4) overrides the file for one run.

[display]
# window size in pixels in windowed mode
width = 1280
height = 960
# windowed, borderless (a window without borders over the whole desktop) or fullscreen (the monitor is switched to the
# resolution below)
window_mode = windowed
# resolution in fullscreen mode (0 = the desktop's)
fullscreen_width = 0
fullscreen_height = 0
# aspect ratio of the picture: auto (follows the window), 4:3, 16:9, 16:10, 21:9, or a number such as 1.78
# anything wider than 4:3 shows more of the world to the sides (the original had no widescreen mode)
aspect = auto
# horizontal field of view in degrees at the chosen aspect; 0 = the game's own value
fov = 0
# HUD and menus in a widescreen picture: edges (original proportions, in-game HUD sticks to the screen edges),
# center (original proportions, centred) or stretch (the original layout stretched with the picture)
hud = edges
# wait for the monitor refresh before showing each frame (stops tearing)
vsync = false

# frame rate of the game while a level is played: 60 (the original), a higher number (the game logic itself then runs at
# that rate, so movement, animation and aiming are as smooth as the monitor allows; menus and loading stay at 60), or
# display (the refresh rate of the monitor).  A fast PC is needed: every frame costs as much as one at 60.
framerate = 60

# gamepad sticks, with the defaults of PCSX2 (the console game was made for a pad with a nearly square gate, and the emulator's
# stretch of 1.33 makes a modern pad reach the same values): dead zone 0..1 (0 = none; the game has its own) and sensitivity
# (1 = as the pad reports)
stick_deadzone = 0
stick_sensitivity = 1.33

# mouse look: 1 = 0.12 degrees for a count of the mouse (a zoomed view turns less); the mouse also follows look_sensitivity below
mouse_sensitivity = 1

# everything the game keeps locked (levels, characters, bot sets, challenges, cheats) is open; also a row of the Audio / Video Options page
unlock_all = false

# the developer menu (F2 in a level: camera views, invincible, invisible, noclip); also a row of the Game Options page
debug_menu = false

# the dual (x2) versions of the weapons in the weapon sets, and the left fist when unarmed; also a row of the Game Options page
dual_wield_all = true

# bots that have no weapon (Always start with gun: No, and none to pick up) go after their target and punch it, instead of standing still; also a row of the Game Options page
bots_punch = true

# the frame rate drawn in a corner of the picture; also a row of the Game Options page
show_fps = false

# turning speed of the right stick and the mouse (1 = as the game has it); also a row of the Controls page
look_sensitivity = 1

# which device plays a level: controller or keyboard (and mouse); also a row of the Controls page
input_method = keyboard

# the keys of the buttons of the virtual pad: names separated by commas (a letter, a digit, F1..F12, Enter, Space, Backspace, Tab, Esc, Ctrl, Shift, Alt,
# Up, Down, Left, Right, Insert, Delete, Home, End, PageUp, PageDown, Comma, Period, Slash, Semicolon, Quote, LeftBracket, RightBracket, Backslash,
# Minus, Equals, Grave).  The first key of a button is the one the game's texts name, as in '(Enter) Ok'.  W, A, S, D walk; the mouse looks, its buttons fire
# (left: R2, right: R1) and the wheel changes the weapon.  Cross accepts and uses, triangle goes back and reloads, left and right change the weapon.
key_cross = Enter, Space, F
key_circle = C
key_triangle = Backspace, R
key_square = X
key_l1 = LeftCtrl, RightCtrl
key_l2 = Z
key_r1 = V
key_r2 = G
key_start = Esc, P
key_select = Tab
key_up = Up
key_down = Down
key_left = Left, Q
key_right = Right, E

# volume of the kinds of sound effects, 0 to 100: weapons (and what they hit), voices, the environment (everything else); the game's own Sound volume applies
# to all of them; also sliders of the Audio Options page, second page
volume_weapons = 100
volume_voices = 100
volume_environment = 100

# the music of the levels: own (the level's), random (a new track each time a level starts) or a track number, 1 to 27 (MUSIC/100N.MSC, or the .opus beside it);
# also a row of the arcade Options page (the page before GO!)
level_music = own

# the MapMaker has no limit of what a map holds (the FREE bar is not shown); also a row of the MapMaker's Options page
mapmaker_unlimited = false

[graphics]
# internal resolution relative to the window (1.0 = native, 2.0 = supersampling, 0.5 = half)
render_scale = 1.0

# the height of the internal picture in pixels (the width follows the aspect ratio): 0 = as large as the window times render_scale, or 480, 720, 1080, 1440, 2160
# (480 on a 4:3 picture is the console's 640x480, shown at any window size); also a row of the Video Options page
internal_resolution = 0

# the internal picture is scaled up to the window with sharp pixels (true) or smoothed (false); also a row of the Video Options page
upscale_sharp = false
# multisample anti-aliasing: 1 (off), 2, 4, 8
msaa = 1
# texture filtering: nearest, bilinear, trilinear
texture_filter = bilinear
# anisotropic filtering 1 (off) .. 16 (needs bilinear or trilinear; adds mip maps)
anisotropy = 1
# the longest side, in pixels, of a picture of a mod that replaces a texture; a bigger picture is made smaller when it is read (a 4096x4096 picture takes about
# 85 MB of video memory with its mip maps)
hd_texture_max = 4096
";

fn parse_bool(v: &str) -> Option<bool> {
    match v.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn parse_aspect(v: &str) -> Option<Option<f32>> {
    let v = v.trim().to_ascii_lowercase();
    if v == "auto" {
        return Some(None);
    }
    if let Some((a, b)) = v.split_once(':').or_else(|| v.split_once('/')) {
        let (a, b): (f32, f32) = (a.trim().parse().ok()?, b.trim().parse().ok()?);
        return (a > 0.0 && b > 0.0).then_some(Some(a / b));
    }
    let r: f32 = v.parse().ok()?;
    (r > 0.1).then_some(Some(r))
}

impl Config {
    pub fn set(&mut self, key: &str, value: &str) -> Result<(), String> {
        let bad = || format!("bad value '{value}' for {key}");
        match key.trim().to_ascii_lowercase().as_str() {
            "width" => self.width = value.trim().parse().map_err(|_| bad())?,
            "height" => self.height = value.trim().parse().map_err(|_| bad())?,
            "window_mode" | "window" => {
                self.window_mode = match value.trim().to_ascii_lowercase().as_str() {
                    "windowed" | "window" => WindowMode::Windowed,
                    "borderless" => WindowMode::Borderless,
                    "fullscreen" | "exclusive" => WindowMode::Fullscreen,
                    _ => return Err(bad()),
                }
            }
            // the old on / off setting: a window over the desktop
            "fullscreen" => self.window_mode = if parse_bool(value).ok_or_else(bad)? { WindowMode::Borderless } else { WindowMode::Windowed },
            "fullscreen_width" => self.fs_width = value.trim().parse().map_err(|_| bad())?,
            "fullscreen_height" => self.fs_height = value.trim().parse().map_err(|_| bad())?,
            "vsync" => self.vsync = parse_bool(value).ok_or_else(bad)?,
            "aspect" => self.aspect = parse_aspect(value).ok_or_else(bad)?,
            "stick_deadzone" => self.stick_deadzone = value.trim().parse::<f32>().map_err(|_| bad())?.clamp(0.0, 0.95),
            "debug_menu" => self.debug_menu = parse_bool(value).ok_or_else(bad)?,
            "unlock_all" => self.unlock_all = parse_bool(value).ok_or_else(bad)?,
            "dual_wield_all" => self.dual_all = parse_bool(value).ok_or_else(bad)?,
            "bots_punch" => self.bot_punch = parse_bool(value).ok_or_else(bad)?,
            "show_fps" => self.show_fps = parse_bool(value).ok_or_else(bad)?,
            "mapmaker_unlimited" => self.mapmaker_unlimited = parse_bool(value).ok_or_else(bad)?,
            "look_sensitivity" => self.look_sensitivity = value.trim().parse::<f32>().map_err(|_| bad())?.clamp(0.1, 5.0),
            "input_method" => {
                self.input_method = match value.trim().to_ascii_lowercase().as_str() {
                    "controller" | "gamepad" => 0,
                    "keyboard" | "mouse" | "keyboard_mouse" | "auto" => 1,
                    _ => return Err(bad()),
                }
            }
            "volume_weapons" | "volume_voices" | "volume_environment" => {
                let k = ["volume_weapons", "volume_voices", "volume_environment"].iter().position(|n| *n == key).unwrap_or(0);
                self.sound_mix[k] = value.trim().parse::<f32>().map_err(|_| bad())?.clamp(0.0, 100.0).round() as u8;
            }
            "level_music" => {
                let v = value.trim().to_ascii_lowercase();
                self.level_music = match v.as_str() {
                    "own" | "level" | "off" | "default" => 0,
                    "random" => 1,
                    n => n.parse::<i32>().ok().filter(|t| (1..=27).contains(t)).map(|t| t + 1).ok_or_else(bad)?,
                };
            }
            // (the first version had five kinds)
            "volume_impacts" | "volume_ambience" | "volume_effects" => {}
            k if k.starts_with("key_") => {
                let name = &k[4..];
                let i = crate::keys::ACTIONS.iter().position(|a| a.0 == name).ok_or_else(bad)?;
                let list = crate::keys::parse_list(value);
                self.keys[i] = if list.is_empty() { crate::keys::default_keys(i) } else { list };
            }
            "mouse_sensitivity" => self.mouse_sensitivity = value.trim().parse::<f32>().map_err(|_| bad())?.clamp(0.05, 10.0),
            "stick_sensitivity" | "stick_gain" => self.stick_sensitivity = value.trim().parse::<f32>().map_err(|_| bad())?.clamp(0.5, 3.0),
            "fov" => self.fov = value.trim().parse().map_err(|_| bad())?,
            // settings of the removed frame interpolation (the game itself runs at the higher rate now)
            "interpolation" | "interpolation_method" | "fps_cap" => {}
            "framerate" | "fps" => {
                self.framerate = match value.trim().to_ascii_lowercase().as_str() {
                    "display" | "monitor" | "auto" => 0,
                    v => v.parse::<u32>().map_err(|_| bad())?.clamp(60, 1000),
                }
            }
            "hud" => {
                self.hud = match value.trim().to_ascii_lowercase().as_str() {
                    "edges" | "edge" => ts_render::HudMode::Edges,
                    "center" | "centre" => ts_render::HudMode::Center,
                    "stretch" => ts_render::HudMode::Stretch,
                    _ => return Err(bad()),
                }
            }
            "internal_resolution" => {
                let v = value.trim().to_ascii_lowercase();
                self.internal_height = if v == "auto" || v == "window" { 0 } else { v.trim_end_matches('p').parse::<u32>().map_err(|_| bad())?.clamp(0, 8192) };
            }
            "upscale_sharp" => self.upscale_sharp = parse_bool(value).ok_or_else(bad)?,
            "hd_texture_max" => self.hd_texture_max = value.trim().parse::<u32>().map_err(|_| bad())?.clamp(256, 16384),
            "render_scale" => self.render_scale = value.trim().parse::<f32>().map_err(|_| bad())?.clamp(0.25, 8.0),
            "msaa" => {
                let n: u32 = value.trim().parse().map_err(|_| bad())?;
                self.msaa = if matches!(n, 1 | 2 | 4 | 8) { n } else { return Err(bad()) };
            }
            "texture_filter" => {
                self.filter = match value.trim().to_ascii_lowercase().as_str() {
                    "nearest" | "point" => ts_render::Filter::Nearest,
                    "bilinear" | "linear" => ts_render::Filter::Bilinear,
                    "trilinear" => ts_render::Filter::Trilinear,
                    _ => return Err(bad()),
                }
            }
            "anisotropy" | "aniso" => self.aniso = value.trim().parse::<u16>().map_err(|_| bad())?.clamp(1, 16),
            // (the removed setting: the picture is always filtered when it is scaled)
            "smooth_output" => {}
            other => return Err(format!("unknown setting '{other}'")),
        }
        Ok(())
    }

    fn parse(&mut self, text: &str) {
        for line in text.lines() {
            let line = line.split('#').next().unwrap_or("").trim();
            if line.is_empty() || line.starts_with('[') {
                continue;
            }
            if let Some((k, v)) = line.split_once('=') {
                if let Err(e) = self.set(k, v) {
                    eprintln!("[config] {e}");
                }
            }
        }
    }

    /// The renderer's share of the settings.
    pub fn render_settings(&self) -> ts_render::Settings {
        // (without `extensions` the picture is the console's own: 640x448, 4:3 whatever the window's shape, the filtering the game asks
        // for, no anti-aliasing and no widescreen layout of the HUD and menus)
        #[cfg(not(feature = "extensions"))]
        {
            ts_render::Settings { aspect: Some(4.0 / 3.0), native: true, ..ts_render::Settings::default() }
        }
        #[cfg(feature = "extensions")]
        {
            ts_render::Settings { render_scale: self.render_scale, internal_height: self.internal_height, sharp: self.upscale_sharp, msaa: self.msaa, filter: self.filter, aniso: self.aniso, aspect: self.aspect, hud: self.hud, native: false }
        }
    }
}

fn ini_path() -> PathBuf {
    std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.join("ts-port.ini"))).unwrap_or_else(|| PathBuf::from("ts-port.ini"))
}

pub static CONFIG: Mutex<Option<Config>> = Mutex::new(None);

/// Load the settings (file, then command-line overrides).  Call once at start.
pub fn load(overrides: &[(String, String)]) {
    let mut c = Config::default();
    let path = ini_path();
    match std::fs::read_to_string(&path) {
        Ok(t) => c.parse(&t),
        Err(_) => {
            std::fs::write(&path, TEMPLATE).ok();
        }
    }
    for (k, v) in overrides {
        if let Err(e) = c.set(k, v) {
            eprintln!("[config] {e}");
        }
    }
    ts_sdk::set_view(c.aspect.unwrap_or(4.0 / 3.0), c.fov);
    ts_sdk::set_logic_hz(c.logic_hz(None));
    ts_sdk::set_unlock_all(c.unlock_all);
    ts_sdk::set_dual_all(c.dual_all);
    ts_sdk::set_bots_punch(c.bot_punch);
    ts_sdk::pad::set_look_sensitivity(c.look_sensitivity);
    ts_game::set_mm_unlimited(c.mapmaker_unlimited);
    apply_sound_mix(&c);
    apply_keys(&c);
    *CONFIG.lock().unwrap() = Some(c);
}

pub fn get() -> Config {
    CONFIG.lock().unwrap().clone().unwrap_or_default()
}

/// Change a setting while the game runs (hotkeys).
pub fn update(f: impl FnOnce(&mut Config)) {
    if let Some(c) = CONFIG.lock().unwrap().as_mut() {
        f(c);
        ts_sdk::set_unlock_all(c.unlock_all);
        ts_sdk::set_dual_all(c.dual_all);
        ts_sdk::set_bots_punch(c.bot_punch);
        ts_sdk::pad::set_look_sensitivity(c.look_sensitivity);
        apply_sound_mix(c);
        apply_keys(c);
    }
}

fn apply_keys(c: &Config) {
    let labels = c.keys.iter().map(|k| k.iter().filter_map(|n| crate::keys::parse_key(n)).map(crate::keys::label).collect()).collect();
    ts_sdk::pad::set_key_labels(labels);
}

fn apply_sound_mix(c: &Config) {
    for (group, &tenths) in c.sound_mix.iter().enumerate() {
        ts_audio::set_group_volume(group, tenths as f32 / 100.0);
    }
}

fn aspect_name(a: Option<f32>) -> String {
    match a {
        None => "auto".into(),
        Some(r) if (r - 4.0 / 3.0).abs() < 0.005 => "4:3".into(),
        Some(r) if (r - 16.0 / 9.0).abs() < 0.005 => "16:9".into(),
        Some(r) if (r - 16.0 / 10.0).abs() < 0.005 => "16:10".into(),
        Some(r) if (r - 21.0 / 9.0).abs() < 0.005 => "21:9".into(),
        Some(r) => format!("{r:.3}"),
    }
}

pub fn filter_name(f: ts_render::Filter) -> &'static str {
    match f {
        ts_render::Filter::Nearest => "nearest",
        ts_render::Filter::Bilinear => "bilinear",
        ts_render::Filter::Trilinear => "trilinear",
    }
}

pub fn hud_name(h: ts_render::HudMode) -> &'static str {
    match h {
        ts_render::HudMode::Edges => "edges",
        ts_render::HudMode::Center => "center",
        ts_render::HudMode::Stretch => "stretch",
    }
}

impl Config {
    /// the MSAA level that will actually be used on a GPU supporting up to `max` samples
    pub fn msaa_eff(&self, max: u32) -> u32 {
        self.render_settings().msaa_for(max)
    }

    /// the frame rate to run the game at, given the refresh rate of the monitor (if known)
    pub fn logic_hz(&self, monitor_hz: Option<f32>) -> f32 {
        // (without `high-framerate` the game runs at its own 60 Hz whatever `framerate` says: the logic is made to match the
        // original exactly, and the changes that let it run at other rates are not in it)
        #[cfg(not(feature = "high-framerate"))]
        {
            let _ = monitor_hz;
            60.0
        }
        #[cfg(feature = "high-framerate")]
        {
            if self.framerate == 0 {
                let hz = monitor_hz.unwrap_or(60.0).max(60.0);
                // with vsync the monitor sets the pace (the game waits for the picture before it, and its frame time is the
                // measured one); the game's own timer must not compete with it
                if self.vsync {
                    hz * 1.03
                } else {
                    hz
                }
            } else {
                self.framerate as f32
            }
        }
    }

    pub fn aspect_name(&self) -> String {
        aspect_name(self.aspect)
    }

    /// Write the current values back into the settings file, keeping the template's comments.
    pub fn save(&self) {
        let vals: [(&str, String); 46] = [
            ("width", self.width.to_string()),
            ("height", self.height.to_string()),
            ("window_mode", self.window_mode.name().to_string()),
            ("fullscreen_width", self.fs_width.to_string()),
            ("fullscreen_height", self.fs_height.to_string()),
            ("aspect", aspect_name(self.aspect)),
            ("fov", format!("{}", self.fov)),
            ("hud", hud_name(self.hud).into()),
            ("vsync", self.vsync.to_string()),
            ("framerate", if self.framerate == 0 { "display".to_string() } else { self.framerate.to_string() }),
            ("render_scale", format!("{}", self.render_scale)),
            ("internal_resolution", self.internal_height.to_string()),
            ("upscale_sharp", self.upscale_sharp.to_string()),
            ("hd_texture_max", self.hd_texture_max.to_string()),
            ("msaa", self.msaa.to_string()),
            ("texture_filter", filter_name(self.filter).into()),
            ("anisotropy", self.aniso.to_string()),
            ("stick_deadzone", format!("{}", self.stick_deadzone)),
            ("stick_sensitivity", format!("{}", self.stick_sensitivity)),
            ("mouse_sensitivity", format!("{}", self.mouse_sensitivity)),
            ("unlock_all", self.unlock_all.to_string()),
            ("debug_menu", self.debug_menu.to_string()),
            ("dual_wield_all", self.dual_all.to_string()),
            ("bots_punch", self.bot_punch.to_string()),
            ("show_fps", self.show_fps.to_string()),
            ("look_sensitivity", format!("{}", self.look_sensitivity)),
            ("mapmaker_unlimited", self.mapmaker_unlimited.to_string()),
            ("input_method", ["controller", "keyboard"][self.input_method.min(1) as usize].to_string()),
            ("key_cross", self.keys[0].join(", ")),
            ("key_circle", self.keys[1].join(", ")),
            ("key_triangle", self.keys[2].join(", ")),
            ("key_square", self.keys[3].join(", ")),
            ("key_l1", self.keys[4].join(", ")),
            ("key_l2", self.keys[5].join(", ")),
            ("key_r1", self.keys[6].join(", ")),
            ("key_r2", self.keys[7].join(", ")),
            ("key_start", self.keys[8].join(", ")),
            ("key_select", self.keys[9].join(", ")),
            ("key_up", self.keys[10].join(", ")),
            ("key_down", self.keys[11].join(", ")),
            ("key_left", self.keys[12].join(", ")),
            ("key_right", self.keys[13].join(", ")),
            ("volume_weapons", self.sound_mix[0].to_string()),
            ("volume_voices", self.sound_mix[1].to_string()),
            ("volume_environment", self.sound_mix[2].to_string()),
            ("level_music", match self.level_music {
                0 => "own".to_string(),
                1 => "random".to_string(),
                n => (n - 1).to_string(),
            }),
        ];
        let mut out = String::new();
        for line in TEMPLATE.lines() {
            let key = line.split('=').next().unwrap_or("").trim();
            let hit = if line.starts_with('#') || !line.contains('=') { None } else { vals.iter().find(|(k, _)| *k == key) };
            match hit {
                Some((k, v)) => out.push_str(&format!("{k} = {v}\n")),
                None => {
                    out.push_str(line);
                    out.push('\n');
                }
            }
        }
        std::fs::write(ini_path(), out).ok();
    }
}
