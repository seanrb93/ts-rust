//! Desktop window, presentation and input.
//!
//! The decompiled game runs on its own thread and calls `present` once per finished frame; that thread renders the
//! frame straight into the window's swap chain.  The main thread only runs the OS event loop: it translates keyboard
//! and gamepad input into the virtual DualShock 2 the game's pad code reads (`ts_sdk::set_pad_state`).

use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use ts_sdk::pad::{btn, PadState};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use crate::config::WindowMode;
use winit::monitor::MonitorHandle;
use winit::window::{Fullscreen, Window, WindowId};

struct Presenter {
    window: Arc<Window>,
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    renderer: ts_render::Renderer,
    /// present mode used when vsync is off
    mailbox: wgpu::PresentMode,
    /// refresh rate of the monitor the window is on, in Hz
    monitor_hz: Option<f32>,
    presents: u32,
}

static PRESENTER: Mutex<Option<Presenter>> = Mutex::new(None);

/// What the monitor the window is on offers (for the resolution choices of the options page).
#[derive(Clone, Default)]
pub struct MonitorInfo {
    /// the desktop's resolution
    pub desktop: (u32, u32),
    /// video modes (width, height) without repeats, ascending
    pub modes: Vec<(u32, u32)>,
}

static MONITOR: Mutex<MonitorInfo> = Mutex::new(MonitorInfo { desktop: (0, 0), modes: Vec::new() });

/// The style of the controller that was used last (see `ts_sdk::pad::input_style`)
static LAST_STYLE: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

/// The device the player chose the other one against, on the Controls page (1 keyboard and mouse, 0 controller, -1 none): it does not
/// take the method back until the chosen device has been used.
static IGNORED_DEVICE: std::sync::atomic::AtomicI8 = std::sync::atomic::AtomicI8::new(-1);

/// The control method was set by hand: `from` is the one that was in use (the device the player is holding now).
pub fn method_chosen(from: u8, to: u8) {
    IGNORED_DEVICE.store(if from != to { from as i8 } else { -1 }, std::sync::atomic::Ordering::Relaxed);
}

/// The keyboard and mouse (`true`) or a controller was used: the control method follows (Controls page, Control Method).
fn note_device(keyboard: bool) {
    let want = keyboard as u8;
    match IGNORED_DEVICE.load(std::sync::atomic::Ordering::Relaxed) {
        d if d == want as i8 => return,
        d if d >= 0 => IGNORED_DEVICE.store(-1, std::sync::atomic::Ordering::Relaxed),
        _ => {}
    }
    if crate::config::get().input_method != want {
        crate::config::update(|c| c.input_method = want);
    }
}

pub fn monitor_info() -> MonitorInfo {
    MONITOR.lock().unwrap().clone()
}

fn read_monitor(m: &MonitorHandle) {
    let size = m.size();
    let mut modes: Vec<(u32, u32)> = m.video_modes().map(|v| (v.size().width, v.size().height)).filter(|&(w, h)| w >= 640 && h >= 480).collect();
    modes.sort_by_key(|&(w, h)| (w * h, w));
    modes.dedup();
    *MONITOR.lock().unwrap() = MonitorInfo { desktop: (size.width, size.height), modes };
}

/// The fullscreen state to ask for: `None` for a window, a borderless window over the monitor, or the monitor's video mode of the
/// chosen resolution (the one with the highest refresh rate; the nearest size when there is no such mode).
fn fullscreen_for(monitor: Option<MonitorHandle>, cfg: &crate::config::Config) -> Option<Fullscreen> {
    match cfg.window_mode {
        WindowMode::Windowed => None,
        WindowMode::Borderless => Some(Fullscreen::Borderless(monitor)),
        WindowMode::Fullscreen => {
            let Some(m) = monitor else { return Some(Fullscreen::Borderless(None)) };
            let want = if cfg.fs_width == 0 || cfg.fs_height == 0 { (m.size().width, m.size().height) } else { (cfg.fs_width, cfg.fs_height) };
            let dist = |s: winit::dpi::PhysicalSize<u32>| (s.width as i64 - want.0 as i64).abs() + (s.height as i64 - want.1 as i64).abs();
            let best = m.video_modes().min_by_key(|v| (dist(v.size()), -(v.refresh_rate_millihertz() as i64), -(v.bit_depth() as i64)));
            Some(match best {
                Some(v) => Fullscreen::Exclusive(v),
                None => Fullscreen::Borderless(Some(m)),
            })
        }
    }
}

/// The part of the settings that decides the window state (the other sizes do not matter in a mode).
type WindowWant = (WindowMode, (u32, u32), (u32, u32));

fn window_want(c: &crate::config::Config) -> WindowWant {
    match c.window_mode {
        WindowMode::Windowed => (c.window_mode, (c.width, c.height), (0, 0)),
        WindowMode::Borderless => (c.window_mode, (0, 0), (0, 0)),
        WindowMode::Fullscreen => (c.window_mode, (0, 0), (c.fs_width, c.fs_height)),
    }
}
/// Set when a `--press` script drives the pad: keyboard / gamepad state is then ignored.
pub static SCRIPTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Render one frame into the window (called from the game thread).
pub fn present(frame: &ts_gfx::scene::Frame) {
    present_ex(frame, true)
}

/// `update_last`: this picture counts as the game's "previous screen" for blending effects.
pub fn present_ex(frame: &ts_gfx::scene::Frame, update_last: bool) {
    present_inner(Some(frame), update_last, 0.0, false)
}

/// Draw `frame` (if given) and show the picture `t` of the way back towards the previous game frame (`warp`), or the
/// frame as it is.  `None` re-shows the scene drawn last.
pub fn present_inner(frame: Option<&ts_gfx::scene::Frame>, update_last: bool, t: f32, warp: bool) {
    let mut g = PRESENTER.lock().unwrap();
    let Some(p) = g.as_mut() else { return };
    let size = p.window.inner_size();
    if size.width == 0 || size.height == 0 {
        return;
    }
    if (size.width, size.height) != (p.config.width, p.config.height) {
        p.config.width = size.width;
        p.config.height = size.height;
        p.surface.configure(p.renderer.device(), &p.config);
    }
    let cfg = crate::config::get();
    ts_sdk::set_view(cfg.aspect.unwrap_or(size.width as f32 / size.height as f32), cfg.fov);
    if p.presents % 120 == 0 {
        p.monitor_hz = p.window.current_monitor().and_then(|m| m.refresh_rate_millihertz()).map(|m| m as f32 / 1000.0);
    }
    if p.presents % 120 == 1 {
        if let Some(m) = p.window.current_monitor() {
            read_monitor(&m);
        }
    }
    p.presents = p.presents.wrapping_add(1);
    ts_sdk::set_logic_hz(cfg.logic_hz(p.monitor_hz));
    p.renderer.settings = cfg.render_settings();
    p.renderer.update_last = update_last;
    crate::menu::MAX_MSAA.store(p.renderer.max_msaa, std::sync::atomic::Ordering::Relaxed);
    let want_mode = if cfg.vsync { wgpu::PresentMode::AutoVsync } else { p.mailbox };
    if p.config.present_mode != want_mode {
        p.config.present_mode = want_mode;
        p.surface.configure(p.renderer.device(), &p.config);
    }
    // (the FPS / timing figures: the Game Options row, F3 too)
    p.renderer.overlay_text = if cfg.show_fps {
        format!("{}
{}", ts_sdk::perf::report(), p.renderer.gpu_name)
    } else {
        String::new()
    };
    if let Some(f) = frame {
        p.renderer.render_scene((size.width, size.height), f);
        ts_sdk::perf::set_gpu(p.renderer.gpu_ms);
    }
    match p.surface.get_current_texture() {
        Ok(tex) => {
            p.renderer.present_scene(&tex.texture, (size.width, size.height), t, warp);
            tex.present();
            ts_sdk::perf::display_frame();
        }
        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => p.surface.configure(p.renderer.device(), &p.config),
        Err(e) => eprintln!("[window] surface error: {e:?}"),
    }
}

/// The keyboard and the mouse as a DualShock 2 (port 0).  The buttons are those of the game's own default set-up: fire R2, alternate
/// fire R1, aim L2, crouch L1, activate cross, manual reload triangle, zoom and weapon change on the directional buttons.
struct KbMouse {
    left: bool,
    right: bool,
    /// the aim is on (a click of the third mouse button turns it on and off)
    aiming: bool,
    /// buttons held for a moment by the mouse wheel and the like: (button, until)
    pulses: Vec<(u16, Instant)>,
}

impl KbMouse {
    fn new() -> KbMouse {
        KbMouse { left: false, right: false, aiming: false, pulses: Vec::new() }
    }

    fn pulse(&mut self, button: u16) {
        self.pulses.push((button, Instant::now() + Duration::from_millis(90)));
    }

    fn state(&mut self, keys: &HashSet<KeyCode>) -> PadState {
        let now = Instant::now();
        self.pulses.retain(|p| p.1 > now);
        let mut s = PadState::idle(true);
        let map = [
            (KeyCode::KeyE, btn::CROSS),
            (KeyCode::Enter, btn::CROSS),
            (KeyCode::KeyQ, btn::LEFT),
            (KeyCode::KeyF, btn::RIGHT),
            (KeyCode::Backspace, btn::TRIANGLE),
            (KeyCode::KeyR, btn::TRIANGLE),
            (KeyCode::Escape, btn::START),
            (KeyCode::Tab, btn::SELECT),
            (KeyCode::ControlLeft, btn::L1),
            (KeyCode::ControlRight, btn::L1),
            (KeyCode::ArrowUp, btn::UP),
            (KeyCode::ArrowDown, btn::DOWN),
            (KeyCode::ArrowLeft, btn::LEFT),
            (KeyCode::ArrowRight, btn::RIGHT),
        ];
        for (k, b) in map {
            if keys.contains(&k) {
                s.buttons |= b;
            }
        }
        if self.left {
            s.buttons |= btn::R2;
        }
        if self.right {
            s.buttons |= btn::R1;
        }
        // (with the game's Aim Mode on Hold the aim is held for as long as it is on; with Toggle the game turns it on and off by
        // itself at each press, so a click only presses the button for a moment)
        if self.aiming && !ts_game::aim_mode_toggle() {
            s.buttons |= btn::L2;
        }
        for (b, _) in &self.pulses {
            s.buttons |= *b;
        }
        // movement: the left stick.  Diagonals are not faster than straight ahead, and Shift walks.
        let dir = |neg: KeyCode, pos: KeyCode| (keys.contains(&pos) as i32 - keys.contains(&neg) as i32) as f32;
        let (mut x, mut y) = (dir(KeyCode::KeyA, KeyCode::KeyD), dir(KeyCode::KeyW, KeyCode::KeyS));
        let len = x.hypot(y);
        if len > 1.0 {
            x /= len;
            y /= len;
        }
        // the keys are a stick pushed all the way: it gets the same dead zone and sensitivity as a controller's, and Shift holds
        // it back to a walk
        let cfg = crate::config::get();
        let walk = if keys.contains(&KeyCode::ShiftLeft) || keys.contains(&KeyCode::ShiftRight) { 0.65 } else { 1.0 };
        let byte = |v: f32| crate::gamepads::stick_to_byte(crate::gamepads::stick_value(v, cfg.stick_deadzone, cfg.stick_sensitivity) * walk);
        // sticks: right x, right y, left x, left y (y: 0 = up)
        s.sticks = [0x80, 0x80, byte(x), byte(y)];
        s
    }
}

/// Combine keyboard and gamepad: buttons are OR-ed, the stick further from centre wins.
fn merge(a: PadState, b: PadState) -> PadState {
    let mut s = a;
    s.buttons |= b.buttons;
    for i in 0..4 {
        let da = (a.sticks[i] as i32 - 0x80).abs();
        let db = (b.sticks[i] as i32 - 0x80).abs();
        s.sticks[i] = if db > da { b.sticks[i] } else { a.sticks[i] };
    }
    s
}

struct App {
    window: Option<Arc<Window>>,
    start_game: Option<Box<dyn FnOnce() + Send>>,
    keys: HashSet<KeyCode>,
    kbm: KbMouse,
    /// the mouse pointer is captured (hidden and held in the window)
    grabbed: bool,
    focused: bool,
    pads: Option<crate::gamepads::Pads>,
    /// window state asked for last, and when
    applied: Option<WindowWant>,
    applied_at: Instant,
}

impl App {
    /// Bring the window to the mode and size in the settings (they can be changed by the options page at any time).
    fn apply_window(&mut self) {
        let Some(w) = self.window.clone() else { return };
        let cfg = crate::config::get();
        let want = window_want(&cfg);
        if self.applied == Some(want) {
            return;
        }
        let monitor = w.current_monitor().or_else(|| w.primary_monitor());
        if let Some(m) = &monitor {
            read_monitor(m);
        }
        let fs = fullscreen_for(monitor, &cfg);
        let windowed = fs.is_none();
        w.set_fullscreen(fs);
        if windowed {
            w.set_maximized(false);
            let _ = w.request_inner_size(winit::dpi::PhysicalSize::new(cfg.width.max(320), cfg.height.max(240)));
        }
        eprintln!("[window] {} {:?}", cfg.window_mode.name(), want);
        self.applied = Some(want);
        self.applied_at = Instant::now();
    }

    /// The pointer belongs to the game while a level is played (hidden and held in the window); the menus have it back.
    fn update_mouse_capture(&mut self) {
        let Some(w) = self.window.clone() else { return };
        let want = self.focused && ts_game::mouse_look_active();
        if !want {
            self.kbm.aiming = false;
            self.kbm.left = false;
            self.kbm.right = false;
        }
        if want == self.grabbed {
            return;
        }
        use winit::window::CursorGrabMode;
        if want {
            if w.set_cursor_grab(CursorGrabMode::Locked).is_err() {
                let _ = w.set_cursor_grab(CursorGrabMode::Confined);
            }
        } else {
            let _ = w.set_cursor_grab(CursorGrabMode::None);
        }
        w.set_cursor_visible(!want);
        self.grabbed = want;
    }

    fn update_pads(&mut self) {
        if SCRIPTED.load(std::sync::atomic::Ordering::Relaxed) {
            return;
        }
        // (both devices always work; the method is the one used last, and shows in the Controls page, where it can be set too)
        let mut pads = [Some(self.kbm.state(&self.keys)), None];
        if let Some(gp) = self.pads.as_mut() {
            let states = gp.poll();
            let styles = gp.styles();
            if let Some(&style) = styles.first() {
                LAST_STYLE.store(style, std::sync::atomic::Ordering::Relaxed);
            }
            for (slot, state) in states.iter().copied().take(2).enumerate() {
                // a controller that is being used (a button or a stick out of the middle) takes over from the keyboard and mouse
                let used = state.buttons != 0 || state.sticks.iter().any(|&v| (v as i32 - 0x80).abs() > 40);
                if used {
                    LAST_STYLE.store(styles.get(slot).copied().unwrap_or(1), std::sync::atomic::Ordering::Relaxed);
                    if slot == 0 || ts_game::single_player_level() {
                        note_device(false);
                        // (the aim the mouse turned on is let go: it would otherwise stay pressed alongside the controller)
                        self.kbm.aiming = false;
                    }
                }
                pads[slot] = Some(match pads[slot] {
                    Some(k) => merge(k, state),
                    None => state,
                });
            }
            // in a one-player level any controller is the player's (a pad that was connected after another one is the second player
            // otherwise, and does nothing there); the menus and the split-screen levels keep each pad to its own player
            if ts_game::single_player_level() {
                for state in states.iter().skip(1) {
                    pads[0] = Some(merge(pads[0].unwrap_or(PadState::idle(true)), *state));
                }
            }
            // three or more controllers: a multitap on port 0 holds them all (the game has room for four players)
            if states.len() >= 3 {
                let tap = |i: usize| states.get(i).copied().unwrap_or(PadState::idle(false));
                ts_sdk::pad::set_multitap(Some([tap(1), tap(2), tap(3)]));
                pads[1] = None;
            } else {
                ts_sdk::pad::set_multitap(None);
            }
        }
        for (port, p) in pads.iter().enumerate() {
            ts_sdk::set_pad_state(port, p.unwrap_or(PadState::idle(false)));
        }
        // what the buttons are called in the game's texts, and whether the mouse is the player's
        let keyboard_method = crate::config::get().input_method == 1;
        ts_sdk::pad::set_mkb_active(keyboard_method);
        ts_sdk::pad::set_input_style(if keyboard_method { 3 } else { LAST_STYLE.load(std::sync::atomic::Ordering::Relaxed) });
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let cfg = crate::config::get();
        let mut attrs = Window::default_attributes().with_title("TimeSplitters").with_inner_size(winit::dpi::PhysicalSize::new(cfg.width.max(320), cfg.height.max(240)));
        let monitor = el.primary_monitor().or_else(|| el.available_monitors().next());
        if let Some(m) = &monitor {
            read_monitor(m);
        }
        attrs = attrs.with_fullscreen(fullscreen_for(monitor, &cfg));
        self.applied = Some(window_want(&cfg));
        self.applied_at = Instant::now();
        let window = Arc::new(el.create_window(attrs).expect("cannot create window"));
        eprintln!("[window] {} -> {}x{}, fullscreen {}", cfg.window_mode.name(), window.inner_size().width, window.inner_size().height, window.fullscreen().is_some());
        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(window.clone()).expect("cannot create surface");
        let caps_adapter = pollster_adapter_caps(&instance, &surface);
        let format = caps_adapter.formats.iter().copied().find(|f| !f.is_srgb()).unwrap_or(caps_adapter.formats[0]);
        let renderer = ts_render::Renderer::new(&instance, Some(&surface), format);
        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: if caps_adapter.usages.contains(wgpu::TextureUsages::COPY_SRC) {
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC
            } else {
                wgpu::TextureUsages::RENDER_ATTACHMENT
            },
            format,
            width: size.width.max(1),
            height: size.height.max(1),
            // the game paces itself to 60 Hz (wait_vsync); don't stack a second vsync wait on top
            present_mode: if cfg.vsync {
                wgpu::PresentMode::AutoVsync
            } else if caps_adapter.present_modes.contains(&wgpu::PresentMode::Mailbox) {
                wgpu::PresentMode::Mailbox
            } else {
                wgpu::PresentMode::AutoNoVsync
            },
            desired_maximum_frame_latency: 2,
            alpha_mode: caps_adapter.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(renderer.device(), &config);
        self.window = Some(window.clone());
        let mailbox = if caps_adapter.present_modes.contains(&wgpu::PresentMode::Mailbox) { wgpu::PresentMode::Mailbox } else { wgpu::PresentMode::AutoNoVsync };
        *PRESENTER.lock().unwrap() = Some(Presenter { window, surface, config, renderer, mailbox, monitor_hz: None, presents: 0 });
        if let Some(f) = self.start_game.take() {
            f();
        }
        el.set_control_flow(ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(8)));
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                std::process::exit(0);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key {
                    match event.state {
                        ElementState::Pressed => {
                            if !event.repeat {
                                if code == KeyCode::F11 {
                                    // a window <-> a window over the desktop
                                    crate::config::update(|c| c.window_mode = if c.window_mode == WindowMode::Windowed { WindowMode::Borderless } else { WindowMode::Windowed });
                                    crate::config::get().save();
                                    self.apply_window();
                                }
                                if code == KeyCode::F2 && crate::config::get().debug_menu {
                                    ts_game::request_debug_menu();
                                }
                                if code == KeyCode::F3 {
                                    crate::config::update(|c| c.show_fps = !c.show_fps);
                                    crate::config::get().save();
                                }
                                if code == KeyCode::F10 {
                                    std::process::exit(0);
                                }
                            }
                            self.keys.insert(code);
                            if !matches!(code, KeyCode::F1 | KeyCode::F2 | KeyCode::F3 | KeyCode::F10 | KeyCode::F11) {
                                note_device(true);
                            }
                        }
                        ElementState::Released => {
                            self.keys.remove(&code);
                        }
                    }
                    self.update_pads();
                }
            }
            WindowEvent::Resized(size) => {
                // a window the player resized by dragging its edge is the new windowed size
                if let Some(w) = &self.window {
                    let windowed = w.fullscreen().is_none() && matches!(self.applied, Some((WindowMode::Windowed, ..)));
                    if windowed && size.width >= 320 && size.height >= 240 && self.applied_at.elapsed() > Duration::from_millis(700) {
                        crate::config::update(|c| (c.width, c.height) = (size.width, size.height));
                        self.applied = Some(window_want(&crate::config::get()));
                    }
                }
            }
            WindowEvent::Focused(f) => {
                self.focused = f;
                if !f {
                    self.keys.clear();
                    self.kbm.left = false;
                    self.kbm.right = false;
                    self.update_pads();
                }
            }
            WindowEvent::MouseInput { state, button, .. } => {
                use winit::event::MouseButton;
                let down = state == ElementState::Pressed;
                // (only a level takes the mouse's buttons: in the menus a click does nothing)
                let live = ts_game::mouse_look_active() && self.focused;
                if live {
                    note_device(true);
                }
                match button {
                    MouseButton::Left => self.kbm.left = down && live,
                    MouseButton::Right => self.kbm.right = down && live,
                    MouseButton::Middle if down && live => {
                        self.kbm.aiming = !self.kbm.aiming;
                        if ts_game::aim_mode_toggle() {
                            self.kbm.pulse(btn::L2);
                        }
                    }
                    _ => {}
                }
                self.update_pads();
            }
            WindowEvent::MouseWheel { delta, .. } => {
                use winit::event::MouseScrollDelta;
                let up = match delta {
                    MouseScrollDelta::LineDelta(_, y) => y,
                    MouseScrollDelta::PixelDelta(p) => p.y as f32,
                };
                if up != 0.0 && ts_game::mouse_look_active() {
                    // aiming with a weapon that zooms: zoom in and out; otherwise: the previous and the next weapon
                    let b = match (self.kbm.aiming && ts_game::weapon_zooms(), up > 0.0) {
                        (true, true) => btn::UP,
                        (true, false) => btn::DOWN,
                        (false, true) => btn::LEFT,
                        (false, false) => btn::RIGHT,
                    };
                    self.kbm.pulse(b);
                    self.update_pads();
                }
            }
            _ => {}
        }
        let _ = el;
    }

    fn device_event(&mut self, _el: &ActiveEventLoop, _id: winit::event::DeviceId, event: winit::event::DeviceEvent) {
        if let winit::event::DeviceEvent::MouseMotion { delta } = event {
            if self.grabbed {
                if delta.0 != 0.0 || delta.1 != 0.0 {
                    note_device(true);
                }
                let k = crate::config::get().mouse_sensitivity as f64;
                ts_sdk::pad::mouse_move(delta.0 * k, delta.1 * k);
            }
        }
    }

    fn about_to_wait(&mut self, el: &ActiveEventLoop) {
        self.update_mouse_capture();
        self.update_pads();
        // the setting can also be changed by the game's own options page
        self.apply_window();
        el.set_control_flow(ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(8)));
    }
}

fn pollster_adapter_caps(instance: &wgpu::Instance, surface: &wgpu::Surface) -> wgpu::SurfaceCapabilities {
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: Some(surface),
        force_fallback_adapter: false,
    }))
    .expect("no suitable GPU adapter");
    surface.get_capabilities(&adapter)
}

/// Open the window and run the event loop on the calling (main) thread.  `start_game` is invoked once the window and
/// swap chain exist.  Never returns.
pub fn run(start_game: impl FnOnce() + Send + 'static) -> ! {
    let event_loop = EventLoop::new().expect("cannot create event loop");
    let pads = match crate::gamepads::Pads::new() {
        Ok(p) => Some(p),
        Err(e) => {
            eprintln!("[window] gamepad support unavailable: {e}");
            None
        }
    };
    let mut app = App { window: None, start_game: Some(Box::new(start_game)), keys: HashSet::new(), kbm: KbMouse::new(), grabbed: false, focused: true, pads, applied: None, applied_at: Instant::now() };
    event_loop.run_app(&mut app).expect("event loop failed");
    std::process::exit(0);
}
