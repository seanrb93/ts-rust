//! Graphics-synthesizer / display services.  The original handed finished display lists (DMA chains of GIF/VIF
//! packets) to the GS.  The port instead lets a renderer register a hook that is called with each finished list.
#![allow(non_snake_case)]

use std::sync::Mutex;
use std::time::{Duration, Instant};

pub type PresentFn = fn(list_addr: i32);

static PRESENT: Mutex<Option<PresentFn>> = Mutex::new(None);
static FRAMES: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static VSYNC: Mutex<Option<Instant>> = Mutex::new(None);

pub const VBLANK_HZ: f64 = 60.0;

/// Register the function that renders one finished display list.
pub fn set_present_hook(f: PresentFn) {
    *PRESENT.lock().unwrap() = Some(f);
}

/// Hand a finished display list to the renderer.
pub fn present(list_addr: i32) {
    FRAMES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let f = *PRESENT.lock().unwrap();
    if let Some(f) = f {
        f(list_addr);
    }
}

/// Hook run at the start of every `present_frame` (before the depth read-back): lets the port finish the previous
/// frame's asynchronous rendering.
static SYNC: Mutex<Option<fn()>> = Mutex::new(None);

pub fn set_sync_hook(f: fn()) {
    *SYNC.lock().unwrap() = Some(f);
}

pub fn present_sync() {
    let f = *SYNC.lock().unwrap();
    if let Some(f) = f {
        f();
    }
}

pub fn frame_count() -> u64 {
    FRAMES.load(std::sync::atomic::Ordering::Relaxed)
}

#[cfg(windows)]
#[link(name = "winmm")]
extern "system" {
    fn timeBeginPeriod(ms: u32) -> u32;
}

// ---- game frame rate ------------------------------------------------------------------------------------------------

static LOGIC_HZ: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0x4270_0000); // 60.0
/// true while the frame clock runs at the high rate (see `wait_frame`)
static HIRATE_CLOCK: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Frame rate the game logic is asked to run at while a level is being played (60 = the original; menus always run at 60).
pub fn set_logic_hz(hz: f32) {
    LOGIC_HZ.store(hz.clamp(60.0, 1000.0).to_bits(), std::sync::atomic::Ordering::Relaxed);
}

pub fn logic_hz() -> f32 {
    f32::from_bits(LOGIC_HZ.load(std::sync::atomic::Ordering::Relaxed))
}

fn sleep_until(target: Instant) {
    // Windows sleeps in ~15 ms steps: sleep most of the wait, then spin the rest for an exact deadline.
    let coarse = Duration::from_micros(2500);
    if let Some(d) = target.checked_duration_since(Instant::now()) {
        if d > coarse {
            std::thread::sleep(d - coarse);
        }
    }
    while Instant::now() < target {
        std::hint::spin_loop();
    }
}

/// Wait for the next frame of a game running at `hz` (> 60) and return the time that frame stands for, in seconds: one
/// period when the game keeps up, the real elapsed time when it does not (so the game clock never falls behind).
pub fn wait_frame(hz: f64) -> f64 {
    #[cfg(windows)]
    {
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| unsafe {
            timeBeginPeriod(1);
        });
    }
    let period = Duration::from_secs_f64(1.0 / hz);
    let mut g = VSYNC.lock().unwrap();
    // the frame grid of the 60 Hz clock does not fit: start a new one
    if !HIRATE_CLOCK.swap(true, std::sync::atomic::Ordering::Relaxed) {
        *g = Some(Instant::now());
    }
    let last = g.get_or_insert_with(Instant::now);
    let target = *last + period;
    let dt;
    if std::env::var_os("TS_NOPACE").is_some() {
        dt = period.as_secs_f64();
    } else if Instant::now() < target {
        let t = Instant::now();
        sleep_until(target);
        perf::add_wait(t.elapsed());
        *last = target;
        dt = period.as_secs_f64();
    } else {
        let now = Instant::now();
        dt = now.duration_since(*last).as_secs_f64();
        *last = now;
    }
    let dt = dt.min(5.0 / 60.0);
    perf::add_ticks_f(dt * VBLANK_HZ);
    dt
}

/// Block until the next vertical blank (60 Hz) and return how many blanks elapsed since the previous call (>= 1).
pub fn wait_vsync() -> u32 {
    #[cfg(windows)]
    {
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| unsafe {
            timeBeginPeriod(1);
        });
    }
    let period = Duration::from_secs_f64(1.0 / VBLANK_HZ);
    let mut g = VSYNC.lock().unwrap();
    if HIRATE_CLOCK.swap(false, std::sync::atomic::Ordering::Relaxed) {
        *g = Some(Instant::now());
    }
    let last = g.get_or_insert_with(Instant::now);
    let elapsed = last.elapsed();
    let mut ticks = (elapsed.as_secs_f64() * VBLANK_HZ) as u32;
    if ticks == 0 {
        let t = Instant::now();
        // Windows sleeps in ~15 ms steps: sleep most of the wait, then spin the rest for an exact vblank.
        let target = *last + period;
        let coarse = Duration::from_micros(2500);
        if let Some(d) = target.checked_duration_since(Instant::now()) {
            if d > coarse {
                std::thread::sleep(d - coarse);
            }
        }
        while Instant::now() < target {
            std::hint::spin_loop();
        }
        perf::add_wait(t.elapsed());
        ticks = 1;
    }
    perf::add_ticks(ticks);
    *last += period * ticks;
    // never accumulate a backlog of more than a few blanks (e.g. after a long load)
    if last.elapsed() > period * 4 {
        *last = Instant::now();
    }
    ticks
}

/// Seconds since start expressed in 60 Hz frames, wrapped by the caller.
pub fn now_frames() -> f32 {
    static START: Mutex<Option<Instant>> = Mutex::new(None);
    let mut g = START.lock().unwrap();
    let s = g.get_or_insert_with(Instant::now);
    (s.elapsed().as_secs_f64() * VBLANK_HZ) as f32
}

// ---- display-list ranges written by functions that draw projected world sprites -------------------------------------

static WORLD_RANGES: Mutex<Vec<(i32, i32)>> = Mutex::new(Vec::new());

/// A function that draws world-projected sprites wrote the display-list tags in `[start, end)`.
pub fn world_range(start: i32, end: i32) {
    if end > start {
        WORLD_RANGES.lock().unwrap().push((start, end));
    }
}

/// Ranges recorded since the last call (taken when a frame is presented).
pub fn take_world_ranges() -> Vec<(i32, i32)> {
    std::mem::take(&mut *WORLD_RANGES.lock().unwrap())
}

// ---- widescreen / field of view (set by the window, read by the game thread) ----------------------------------------

static VIEW_ASPECT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0x3faa_aaab); // 4/3
static VIEW_HFOV: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// Display aspect ratio (width / height) and the requested horizontal field of view in degrees (0 = the game's own).
pub fn set_view(aspect: f32, hfov: f32) {
    VIEW_ASPECT.store(aspect.to_bits(), std::sync::atomic::Ordering::Relaxed);
    VIEW_HFOV.store(hfov.to_bits(), std::sync::atomic::Ordering::Relaxed);
}

static UNLOCK_ALL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Everything the game keeps locked (levels, characters, bot sets, challenges, cheats) is open (the "Unlock Everything" option).
pub fn set_unlock_all(on: bool) {
    UNLOCK_ALL.store(on, std::sync::atomic::Ordering::Relaxed);
}

static DUAL_ALL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

/// The dual versions of the weapons and the two fists are on (the "Dual Wield Everything" option).
pub fn set_dual_all(on: bool) {
    DUAL_ALL.store(on, std::sync::atomic::Ordering::Relaxed);
}

pub fn dual_all() -> bool {
    DUAL_ALL.load(std::sync::atomic::Ordering::Relaxed)
}

static BOT_PUNCH: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

/// Bots without a weapon go after their target and punch it (the "Bots Punch" option).
pub fn set_bots_punch(on: bool) {
    BOT_PUNCH.store(on, std::sync::atomic::Ordering::Relaxed);
}

pub fn bots_punch() -> bool {
    BOT_PUNCH.load(std::sync::atomic::Ordering::Relaxed)
}

pub fn unlock_all() -> bool {
    UNLOCK_ALL.load(std::sync::atomic::Ordering::Relaxed)
}

pub fn view() -> (f32, f32) {
    (f32::from_bits(VIEW_ASPECT.load(std::sync::atomic::Ordering::Relaxed)), f32::from_bits(VIEW_HFOV.load(std::sync::atomic::Ordering::Relaxed)))
}

// ---- depth read-back (zbuffer probes for light glows) -------------------------------------------------------

/// Field size of the game's depth buffer copy.
pub const DEPTH_W: usize = 640;
pub const DEPTH_H: usize = 224;

static DEPTH: Mutex<Option<Vec<u16>>> = Mutex::new(None);
static DEPTH_WANTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// requests of the game so far (one per frame whose picture it wants the depth of) and depth buffers delivered
static DEPTH_REQUESTS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static DEPTH_DONE: (Mutex<u64>, std::sync::Condvar) = (Mutex::new(0), std::sync::Condvar::new());

/// The game asks for the depth buffer of the picture it sent last, and for the one of the picture it is about to send to be
/// kept.  Its light glows test their positions against the depth of the very frame they were drawn in, so the answer waits
/// until the translation / drawing threads (which may be a frame behind) have delivered exactly that picture's depth.
pub fn depth_request() -> Option<Vec<u16>> {
    if !DEPTH_WANTED.swap(true, std::sync::atomic::Ordering::AcqRel) {
        let before = DEPTH_REQUESTS.fetch_add(1, std::sync::atomic::Ordering::AcqRel);
        let mut done = DEPTH_DONE.0.lock().unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(40);
        while *done < before {
            let left = deadline.saturating_duration_since(std::time::Instant::now());
            if left.is_zero() {
                break;
            }
            done = DEPTH_DONE.1.wait_timeout(done, left).unwrap().0;
        }
    }
    DEPTH.lock().unwrap().clone()
}

/// Renderer side: true once after the game requested depth.
pub fn depth_wanted() -> bool {
    DEPTH_WANTED.swap(false, std::sync::atomic::Ordering::AcqRel)
}

pub fn depth_store(d: Vec<u16>) {
    *DEPTH.lock().unwrap() = Some(d);
    *DEPTH_DONE.0.lock().unwrap() += 1;
    DEPTH_DONE.1.notify_all();
}

/// Frame-time statistics for the on-screen FPS counter.
pub mod perf {
    use std::sync::Mutex;
    use std::time::{Duration, Instant};

    struct Acc {
        window_start: Option<Instant>,
        last_frame: Option<Instant>,
        frames: u32,
        ticks: f64,
        wait: f64,
        sync: f64,
        gfx: f64,
        render: f64,
        dframes: u32,
        gpu: f64,
        depth: f64,
        worker: f64,
        f_worker: f64,
        worst: f64,
        draws: usize,
        verts: usize,
        report: String,
        // since the previous frame (for spike logging)
        f_wait: f64,
        f_sync: f64,
        f_gfx: f64,
        f_render: f64,
        spike: String,
    }

    static ACC: Mutex<Acc> = Mutex::new(Acc {
        window_start: None,
        last_frame: None,
        frames: 0,
        ticks: 0.0,
        wait: 0.0,
        sync: 0.0,
        gfx: 0.0,
        render: 0.0,
        dframes: 0,
        gpu: 0.0,
        depth: 0.0,
        worker: 0.0,
        f_worker: 0.0,
        worst: 0.0,
        draws: 0,
        verts: 0,
        report: String::new(),
        f_wait: 0.0,
        f_sync: 0.0,
        f_gfx: 0.0,
        f_render: 0.0,
        spike: String::new(),
    });

    pub fn add_wait(d: Duration) {
        let mut a = ACC.lock().unwrap();
        a.wait += d.as_secs_f64();
        a.f_wait += d.as_secs_f64();
    }
    pub fn add_sync(d: Duration) {
        let mut a = ACC.lock().unwrap();
        a.sync += d.as_secs_f64();
        a.f_sync += d.as_secs_f64();
    }
    pub fn add_ticks(n: u32) {
        ACC.lock().unwrap().ticks += n as f64;
    }
    pub fn add_ticks_f(n: f64) {
        ACC.lock().unwrap().ticks += n;
    }
    pub fn add_gfx(d: Duration) {
        let mut a = ACC.lock().unwrap();
        a.gfx += d.as_secs_f64();
        a.f_gfx += d.as_secs_f64();
    }
    pub fn add_render(d: Duration) {
        let mut a = ACC.lock().unwrap();
        a.render += d.as_secs_f64();
        a.f_render += d.as_secs_f64();
    }
    /// One picture was presented (may be an in-between frame).
    pub fn display_frame() {
        ACC.lock().unwrap().dframes += 1;
    }
    /// GPU time left after the CPU finished submitting a scene (ms).
    pub fn set_gpu(ms: f32) {
        ACC.lock().unwrap().gpu = ms as f64;
    }
    pub fn add_depth(d: Duration) {
        ACC.lock().unwrap().depth += d.as_secs_f64();
    }
    /// Total time the gfx worker spent on one whole frame.
    pub fn add_worker(d: Duration) {
        let mut a = ACC.lock().unwrap();
        a.worker += d.as_secs_f64();
        a.f_worker += d.as_secs_f64();
    }
    pub fn set_counts(draws: usize, verts: usize) {
        let mut a = ACC.lock().unwrap();
        a.draws = draws;
        a.verts = verts;
    }

    /// Call once per presented frame.
    pub fn frame() {
        let mut a = ACC.lock().unwrap();
        let now = Instant::now();
        if let Some(l) = a.last_frame {
            let dt = (now - l).as_secs_f64();
            if dt > a.worst {
                a.worst = dt;
            }
            if dt > 0.026 {
                let line = format!(
                    "last spike {:.0} ms: vsync {:.0} gfx-wait {:.0} worker {:.0} lists {:.0} render {:.0}",
                    dt * 1e3, a.f_wait * 1e3, a.f_sync * 1e3, a.f_worker * 1e3, a.f_gfx * 1e3, a.f_render * 1e3
                );
                if std::env::var_os("TS_SPIKELOG").is_some() {
                    eprintln!("[spike] {line}");
                }
                a.spike = line;
            }
        }
        a.last_frame = Some(now);
        a.f_wait = 0.0;
        a.f_sync = 0.0;
        a.f_gfx = 0.0;
        a.f_render = 0.0;
        a.f_worker = 0.0;
        a.frames += 1;
        let start = *a.window_start.get_or_insert(now);
        let el = (now - start).as_secs_f64();
        if el >= 0.5 {
            let n = a.frames as f64;
            let ms = |v: f64| v / n * 1000.0;
            let logic = (el - a.wait - a.sync).max(0.0);
            a.report = format!(
                "{:.0} display FPS | {:.1} game FPS  (frame {:.1} ms, worst {:.1} ms)
game {:.1}  waiting for gfx {:.1}  idle {:.1} ms
gfx thread: total {:.1} (lists {:.1}  depth {:.1}  render {:.1}) ms  gpu wait {:.1} ms
ticks/s {:.0}  draws {}  verts {}
{}",
                a.dframes as f64 / el,
                n / el,
                el / n * 1000.0,
                a.worst * 1000.0,
                ms(logic),
                ms(a.sync),
                ms(a.wait),
                ms(a.worker),
                ms(a.gfx),
                ms(a.depth),
                ms(a.render),
                a.gpu,
                a.ticks / el,
                a.draws,
                a.verts,
                a.spike
            );
            a.window_start = Some(now);
            a.frames = 0;
            a.ticks = 0.0;
            a.wait = 0.0;
            a.sync = 0.0;
            a.gfx = 0.0;
            a.render = 0.0;
            a.depth = 0.0;
            a.dframes = 0;
            a.worker = 0.0;
            a.worst = 0.0;
        }
    }

    pub fn report() -> String {
        ACC.lock().unwrap().report.clone()
    }
}
