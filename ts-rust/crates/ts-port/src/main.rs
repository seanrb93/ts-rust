//! TimeSplitters native PC port launcher.
//!
//! usage: ts-port [--game-dir <dir containing PAK/, MUSIC/ and SLUS_200.90; default: the program's folder>] [--level NAME] [--players N]
//!                 [--headless] [--frames N] [--screenshot FRAME file.png]
//!
//! Opens a window by default; `--headless` (implied by `--screenshot`) renders offscreen only.

mod audio;
mod config;
mod gamepads;
mod menu;
mod mods;
mod dldump;
mod window;
mod gfx_worker;

use std::path::PathBuf;
use std::sync::Mutex;

static DUMP_FRAMES: Mutex<Vec<u64>> = Mutex::new(Vec::new());
static GFX: Mutex<Option<ts_gfx::Gfx>> = Mutex::new(None);
static PROFILE_WINDOW: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(u64::MAX);
static PROFILE_END: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static SHOTS: Mutex<Vec<(u64, String)>> = Mutex::new(Vec::new());
/// Scripted pad input for headless testing: (first frame, frame count, buttons, sticks [rx, ry, lx, ly]).
static PAD_SCRIPT: Mutex<Vec<(u64, u64, u16, [u8; 4], usize)>> = Mutex::new(Vec::new());
/// Test clock (`TS_TICKCLOCK=1`): while a level is running the "frame" numbers of `--press`, `--screenshot`, `--profile` and
/// `--frames` count game ticks (starting at 1000000), so that runs at different frame rates line up.
static CLOCK: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
static MAX: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(u64::MAX);
static WINDOWED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// Image uploads made by the game, with the count of frames presented before them.
static PENDING_UPLOADS: Mutex<std::collections::VecDeque<(u64, ts_gfx::vram::Image)>> = Mutex::new(std::collections::VecDeque::new());

/// Apply the uploads that were made while frame `n` was being built (and before).
fn apply_uploads(n: u64) {
    let mut q = PENDING_UPLOADS.lock().unwrap();
    if q.front().is_some_and(|u| u.0 < n) {
        ts_gfx::vram::with_vram(|v| {
            while q.front().is_some_and(|u| u.0 < n) {
                let (_, img) = q.pop_front().unwrap();
                v.write(img);
            }
        });
    }
}

static PREV_FRAME: Mutex<Option<ts_gfx::scene::Frame>> = Mutex::new(None);
static SHOT_RENDERER: Mutex<Option<ts_render::Renderer>> = Mutex::new(None);

/// The folder of the program: the game's files (`PAK`, `MUSIC`, `SLUS_200.90`) and the saves are kept there.
fn exe_dir() -> PathBuf {
    std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf())).unwrap_or_else(|| PathBuf::from("."))
}

/// Tell the player what is wrong and stop (the program has no console window to show it in).
fn fatal(msg: &str) -> ! {
    eprintln!("[port] {msg}");
    #[cfg(windows)]
    if WINDOWED.load(std::sync::atomic::Ordering::Relaxed) {
        #[link(name = "user32")]
        extern "system" {
            fn MessageBoxW(hwnd: *mut std::ffi::c_void, text: *const u16, caption: *const u16, kind: u32) -> i32;
        }
        let wide = |s: &str| s.encode_utf16().chain(std::iter::once(0)).collect::<Vec<u16>>();
        unsafe {
            MessageBoxW(std::ptr::null_mut(), wide(msg).as_ptr(), wide("TimeSplitters").as_ptr(), 0x10);
        }
    }
    std::process::exit(1);
}

/// Periodic status lines (a console write can stall a frame for 100 ms, so they are opt-in).
fn verbose() -> bool {
    static V: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *V.get_or_init(|| std::env::var_os("TS_VERBOSE").is_some())
}

/// Second half of a frame: depth reconstruction for the game's occlusion probes, then drawing / presenting.  Runs on its
/// own thread when windowed so that it overlaps the translation of the next display list.
pub(crate) fn finish_frame(n: u64, frame: ts_gfx::scene::Frame, fbp: u32) {
    prepare_frame(n, &frame, fbp);
    show_frame(&frame);
}

/// Work that belongs to a new game frame (not to every picture shown): depth reconstruction, benchmarks.
pub(crate) fn prepare_frame(n: u64, frame: &ts_gfx::scene::Frame, fbp: u32) {
    ts_gfx::scene::CURRENT_FBP.store(fbp, std::sync::atomic::Ordering::Relaxed);
    // The game asks for the depth buffer on every frame (light glows fade in and out depending on it) and pairs it with the
    // glow positions of the same frame: it is rebuilt for every frame it was asked for (`depth_request` waits for it).
    if ts_sdk::depth_wanted() {
        let t_depth = std::time::Instant::now();
        ts_sdk::depth_store(ts_gfx::depth::rasterize(frame));
        ts_sdk::perf::add_depth(t_depth.elapsed());
    }
    if std::env::var_os("TS_ZLOG").is_some() && n % 100 == 0 {
        if let Some(d) = ts_sdk::depth_request() {
            use ts_game::g;
            for i in 0..3 {
                let cnt = ts_rt::lw(0x1fea0a0 + 4 * i) * 9;
                let base = ts_rt::lw(g::zbval + 4 * i);
                for k in 0..cnt.min(18) {
                    let e = base + k * 24;
                    let off = ts_rt::lw(e + 12) as usize / 2;
                    eprintln!("[zlog] f{n} set{i} #{k} expected={} stored_read={} mydepth={}", ts_rt::lhu(e + 16), ts_rt::lhu(e + 18), d.get(off).copied().unwrap_or(0));
                }
            }
        }
    }
    if std::env::var_os("TS_BENCH").is_some() && !WINDOWED.load(std::sync::atomic::Ordering::Relaxed) {
        let t_render = std::time::Instant::now();
        let mut g = SHOT_RENDERER.lock().unwrap();
        let r = g.get_or_insert_with(|| {
            let instance = wgpu::Instance::default();
            ts_render::Renderer::new(&instance, None, wgpu::TextureFormat::Rgba8Unorm)
        });
        r.settings = config::get().render_settings();
        r.bench_render(frame, 1280, 960);
        ts_sdk::perf::add_render(t_render.elapsed());
        if n % 120 == 0 {
            eprintln!(
                "[perf] {} | unpack {:.2} stream {:.2} tex {:.2} gif {:.2} ms/frame | vu {:.1} ms/frame, {:.0} runs/frame",
                ts_sdk::perf::report().replace('\n', " | "),
                ts_gfx::programs::UNPACK_NS.swap(0, std::sync::atomic::Ordering::Relaxed) as f64 / 1e6 / 120.0,
                ts_gfx::programs::STREAM_NS.swap(0, std::sync::atomic::Ordering::Relaxed) as f64 / 1e6 / 120.0,
                ts_gfx::programs::TEX_NS.swap(0, std::sync::atomic::Ordering::Relaxed) as f64 / 1e6 / 120.0,
                ts_gfx::programs::GIF_NS.swap(0, std::sync::atomic::Ordering::Relaxed) as f64 / 1e6 / 120.0,
                ts_gfx::programs::VU_NS.swap(0, std::sync::atomic::Ordering::Relaxed) as f64 / 1e6 / 120.0,
                ts_gfx::programs::VU_RUNS.swap(0, std::sync::atomic::Ordering::Relaxed) as f64 / 120.0
            );
        }
    }
}

pub(crate) fn window_present(frame: Option<&ts_gfx::scene::Frame>, update_last: bool, t: f32, warp: bool) {
    window::present_inner(frame, update_last, t, warp);
}

/// Draw / present one picture.
pub(crate) fn show_frame(frame: &ts_gfx::scene::Frame) {
    if WINDOWED.load(std::sync::atomic::Ordering::Relaxed) {
        let t_render = std::time::Instant::now();
        window::present(frame);
        ts_sdk::perf::add_render(t_render.elapsed());
    } else {
        audio::wav_frame();
    }
}

/// Windowed runs translate and draw frames on worker threads (`TS_SYNC_GFX=1` forces everything onto the game thread).
pub(crate) fn pipelined() -> bool {
    static P: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *P.get_or_init(|| WINDOWED.load(std::sync::atomic::Ordering::Relaxed) && std::env::var_os("TS_SYNC_GFX").is_none())
}

fn in_game_now(f: &ts_gfx::scene::Frame) -> bool {
    f.smooth
}


fn process_frame(n: u64, list: i32, world: Vec<(i32, i32)>) {
    if n % 120 == 0 && std::env::var_os("TS_PERFLOG").is_some() {
        eprintln!("[perf] {}", ts_sdk::perf::report().replace('\n', " | "));
    }
    let dumping = DUMP_FRAMES.lock().unwrap().contains(&n);
    ts_gfx::trace::enable(dumping);
    ts_gfx::trace::enable_vu(std::env::var("TS_VUTRACE").is_ok());
    apply_uploads(n);
    let t_gfx = std::time::Instant::now();
    let mut frame = GFX.lock().unwrap().get_or_insert_with(ts_gfx::Gfx::new).run_list(list, &world);
    frame.in_game = ts_rt::lw(ts_game::g::gamestate) == 3 && ts_rt::lw(ts_rt::lw(ts_game::g::front) + 20) == 0;
    frame.smooth = frame.in_game;
    frame.page_open = ts_rt::lw(ts_rt::lw(ts_game::g::front) + 20) != 0;
    frame.players = if matches!(ts_rt::lw(ts_game::g::gamestate), 3..=6) { ts_rt::lw(ts_game::g::lvnumplayers).clamp(0, 4) as u8 } else { 0 };
    frame.menu_open = frame.in_game && ts_rt::lw(ts_game::g::curmenu) != 0;
    frame.scores_open = frame.in_game && (0..4).any(|p| ts_rt::lhu(0x1fb3240 + 92 * p + 8) & 1 != 0);
    if std::env::var_os("TS_MMDBG").is_some() && n % 50 == 0 {
        let f = ts_rt::lw(ts_game::g::front);
        eprintln!("[mmdbg] frame {n} gamestate {} front depth {} pages {}", ts_rt::lw(ts_game::g::gamestate), ts_rt::lw(f + 20), ts_rt::lw(f + 4));
    }
    let first_ms = t_gfx.elapsed().as_secs_f64() * 1e3;
    ts_sdk::perf::add_gfx(t_gfx.elapsed());
    if std::env::var_os("TS_REPEAT").is_some() && frame.vertices.len() > 8000 && n % 7 == 0 {
        // deterministic micro-benchmark: translate the same draw list repeatedly
        let mut best = f64::MAX;
        let mut best_vu = f64::MAX;
        let mut best_gif = f64::MAX;
        for _ in 0..8 {
            ts_gfx::programs::VU_NS.store(0, std::sync::atomic::Ordering::Relaxed);
            ts_gfx::programs::GIF_NS.store(0, std::sync::atomic::Ordering::Relaxed);
            let t = std::time::Instant::now();
            let f2 = GFX.lock().unwrap().get_or_insert_with(ts_gfx::Gfx::new).run_list(list, &[]);
            best = best.min(t.elapsed().as_secs_f64() * 1e3);
            best_gif = best_gif.min(ts_gfx::programs::GIF_NS.load(std::sync::atomic::Ordering::Relaxed) as f64 / 1e6);
            best_vu = best_vu.min(ts_gfx::programs::VU_NS.load(std::sync::atomic::Ordering::Relaxed) as f64 / 1e6);
            drop(f2);
        }
        eprintln!("[rep] frame {n} verts {} best {:.2} ms (vu {:.2} ms incl. gif {:.2} ms) first {:.2} ms", frame.vertices.len(), best, best_vu, best_gif, first_ms);
    }
    ts_sdk::perf::set_counts(frame.draws.len(), frame.vertices.len());
    if dumping {
        let path = format!("gfxtrace_{n}.txt");
        std::fs::write(&path, ts_gfx::trace::take()).ok();
        eprintln!("[gfx] wrote {path}");
    }
    let shot = {
        // a screenshot is taken by the first picture at or after its frame (tick clock: ticks are shared by several frames)
        // (`TS_SHOTRAW=1`: screenshot frame numbers are raw frame counts, whatever the clock of the presses)
        let clk = if std::env::var_os("TS_SHOTRAW").is_some() { n } else { CLOCK.load(std::sync::atomic::Ordering::Relaxed) };
        let mut shots = SHOTS.lock().unwrap();
        shots.iter().position(|(f, _)| *f <= clk).map(|i| shots.remove(i).1)
    };
    if let Some(path) = shot {
        let mut g = SHOT_RENDERER.lock().unwrap();
        let r = g.get_or_insert_with(|| {
            let instance = wgpu::Instance::default();
            ts_render::Renderer::new(&instance, None, wgpu::TextureFormat::Rgba8Unorm)
        });
        r.settings = config::get().render_settings();
        let (w, h) = (std::env::var("TS_SHOTW").ok().and_then(|v| v.parse().ok()).unwrap_or(1280u32), std::env::var("TS_SHOTH").ok().and_then(|v| v.parse().ok()).unwrap_or(960u32));
        // debugging aid: TS_FROM=<n> renders only draws from index n on, TS_SKIP=a,b skips those indices
        let mut shot_warp: Option<f32> = None;
        let mut shot_frame = ts_gfx::scene::Frame { vertices: frame.vertices.clone(), draws: frame.draws.clone(), uploads: Vec::new(), unknown_programs: Vec::new(), in_game: frame.in_game, page_open: frame.page_open, players: frame.players, menu_open: frame.menu_open, scores_open: frame.scores_open, demo: frame.demo, smooth: frame.smooth, textures: frame.textures.clone(), segs: frame.segs.clone() };
        // debugging aid: TS_INTERPT=<0..1> shows the blend of the previous frame and this one
        if let Some(t) = std::env::var("TS_INTERPT").ok().and_then(|v| v.parse::<f32>().ok()) {
            if let Some(prev) = PREV_FRAME.lock().unwrap().as_ref() {
                shot_frame = ts_render::interpolate(prev, &frame, t);
                shot_warp = Some(t);
            }
        }
        if let Ok(f) = std::env::var("TS_FROM") {
            let n: usize = f.parse().unwrap_or(0);
            shot_frame.draws.drain(..n.min(shot_frame.draws.len()));
        }
        if let Ok(on) = std::env::var("TS_ONLY") {
            let only: Vec<usize> = on.split(',').filter_map(|x| x.parse().ok()).collect();
            let mut i = 0;
            shot_frame.draws.retain(|_| {
                i += 1;
                only.contains(&(i - 1))
            });
        }
        if let Ok(sk) = std::env::var("TS_SKIP") {
            let skip: Vec<usize> = sk.split(',').filter_map(|x| x.parse().ok()).collect();
            let mut i = 0;
            shot_frame.draws.retain(|_| {
                i += 1;
                !skip.contains(&(i - 1))
            });
        }
        r.overlay_text = std::env::var("TS_OVERLAY").unwrap_or_default().replace("\n", "
");
        if std::env::var_os("TS_NOSHIFT").is_some() {
            r.no_shift = true;
            let px = r.screenshot(&shot_frame, w, h);
            std::fs::write(format!("{path}.noshift.png"), ts_gfx::png::encode_rgba(w, h, &px)).ok();
            r.no_shift = false;
        }
        if std::env::var_os("TS_LEGACYALPHA").is_some() {
            r.legacy_alpha = true;
            let px = r.screenshot_t(&shot_frame, w, h, shot_warp.unwrap_or(0.0), shot_warp.is_some());
            std::fs::write(format!("{path}.legacy.png"), ts_gfx::png::encode_rgba(w, h, &px)).ok();
            r.legacy_alpha = false;
        }
        let px = r.screenshot_t(&shot_frame, w, h, shot_warp.unwrap_or(0.0), shot_warp.is_some());
        std::fs::write(&path, ts_gfx::png::encode_rgba(w, h, &px)).ok();
        eprintln!("[gfx] screenshot {path} ({} draws, {:?})", frame.draws.len(), r.stats);
        if std::env::var_os("TS_INTERPMULTI").is_some() {
            if let Some(prev) = PREV_FRAME.lock().unwrap().as_ref() {
                ts_render::interp::FIT.store(false, std::sync::atomic::Ordering::Relaxed);
                let f0 = ts_render::interpolate(prev, &frame, 1.0);
                {
                    let mut sx = 0.0f64; let mut sy = 0.0f64; let mut n = 0u32; let mut nz = 0u32;
                    for v in &f0.vertices { if v.pos[2] < 4.0e9 { n += 1; if v.vel != [0.0, 0.0] { nz += 1; sx += v.vel[0] as f64; sy += v.vel[1] as f64; } } }
                    eprintln!("[interp] 3D vertices {n}, with vector {nz}, mean vector ({:.2}, {:.2})", sx / nz.max(1) as f64, sy / nz.max(1) as f64);
                }
                let px = r.screenshot_t(&f0, w, h, 1.0, true);
                std::fs::write(format!("{path}.nofit.png"), ts_gfx::png::encode_rgba(w, h, &px)).ok();
                ts_render::interp::FIT.store(true, std::sync::atomic::Ordering::Relaxed);
                let px = r.screenshot_t(prev, w, h, 0.0, false);
                std::fs::write(format!("{path}.prev.png"), ts_gfx::png::encode_rgba(w, h, &px)).ok();
                for (i, t) in [0.0f32, 0.25, 0.5, 0.75, 1.0].into_iter().enumerate() {
                    let f = ts_render::interpolate(prev, &frame, t);
                    let px = r.screenshot_t(&f, w, h, t, true);
                    std::fs::write(format!("{path}.t{i}.png"), ts_gfx::png::encode_rgba(w, h, &px)).ok();
                }
            }
        }
        if let Ok(dn) = std::env::var("TS_DUMP_DRAW") {
            let dn: usize = dn.parse().unwrap_or(0);
            if let Some(d) = frame.draws.get(dn) {
                for v in &frame.vertices[d.first as usize..(d.first + d.count) as usize] {
                    eprintln!("  v pos=({:.2},{:.2},{}) col={:?} st={:?}", v.pos[0], v.pos[1], v.pos[2], v.color, v.st);
                }
            }
        }
        if std::env::var("TS_DUMP_HUD").is_ok() {
            for (i, d) in frame.draws.iter().enumerate() {
                let vs = &frame.vertices[d.first as usize..(d.first + d.count) as usize];
                if d.state.tex.is_none() && d.count > 3 && vs.iter().all(|v| v.pos[1] < 60.0 && v.pos[1] > 0.0 && v.pos[0] > 20.0) {
                    eprintln!("hud draw {i}: blend={:?} stq={}", d.state.blend.enabled.then_some(d.state.blend.alpha), d.state.use_stq);
                    for v in vs {
                        eprintln!("  v pos=({:.3},{:.3}) col={:?}", v.pos[0], v.pos[1], v.color);
                    }
                }
            }
        }
        if std::env::var("TS_BBOX").is_ok() {
            for (i, d) in frame.draws.iter().enumerate() {
                let vs = &frame.vertices[d.first as usize..(d.first + d.count) as usize];
                let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
                for v in vs {
                    x0 = x0.min(v.pos[0]); y0 = y0.min(v.pos[1]); x1 = x1.max(v.pos[0]); y1 = y1.max(v.pos[1]);
                }
                let (zmin, zmax) = vs.iter().fold((f32::MAX, f32::MIN), |a, v| (a.0.min(v.pos[2]), a.1.max(v.pos[2])));
                eprintln!("  draw {i}: n={} bbox=({x0:.0},{y0:.0})-({x1:.0},{y1:.0}) z=({zmin:.0},{zmax:.0}) tex={:?} test={:#x} zmsk={} blend={} prog={:#x}", d.count, d.state.tex.map(|t| (t.tbp0, t.tw, t.th)), d.state.test, d.state.zmsk, d.state.blend.enabled, d.prog);
            }
        }
        if std::env::var("TS_DRAWS").is_ok() {
            let (mut lo, mut hi) = (f32::MAX, f32::MIN);
            for v in frame.vertices.iter().filter(|v| v.pos[2] < 4.0e9 && v.pos[0] > -2000.0 && v.pos[0] < 3000.0) {
                lo = lo.min(v.pos[0]);
                hi = hi.max(v.pos[0]);
            }
            eprintln!("  3D vertex x range {lo}..{hi}, in_game {} page_open {}", frame.in_game, frame.page_open);
            for (i, d) in frame.draws.iter().enumerate().take(200) {
                let v = &frame.vertices[d.first as usize];
                let s = &d.state;
                eprintln!(
                    "  draw {i}: {:?} n={} tex={:?} blend={:?} test={:#x} zmsk={} fbmsk={:#x} scissor={:?} fbp={} stq={} first pos={:?} col={:?} st={:?}",
                    d.topology, d.count, s.tex.map(|t| (t.tbp0, t.psm, t.tw, t.th, t.cbp, t.tfx, t.tcc)), s.blend.enabled.then_some(s.blend.alpha),
                    s.test, s.zmsk, s.fbmsk, s.scissor, s.fbp, s.use_stq, v.pos, v.color, v.st
                );
            }
        }
    }
    if DUMP_FRAMES.lock().unwrap().contains(&n) {
        let mut seen = std::collections::HashSet::new();
        for d in &frame.draws {
            if let Some(k) = d.state.tex {
                if seen.insert(k) {
                    let r = ts_gfx::vram::with_vram(|v| ts_gfx::texture::decode(&k, v));
                    match r {
                        Some((w, h, rgba, _)) => {
                            let path = format!("tex_f{n}_{:04x}_{:04x}.png", k.tbp0, k.cbp);
                            std::fs::write(&path, ts_gfx::png::encode_rgba(w, h, &rgba)).ok();
                            eprintln!("[gfx] texture {path}: {w}x{h} psm={} cpsm={} csa={}", k.psm, k.cpsm, k.csa);
                        }
                        None => eprintln!("[gfx] texture tbp0={:#x} not found in vram", k.tbp0),
                    }
                }
            }
        }
    }
    if std::env::var_os("TS_STATELOG2").is_some() && (5400..5425).contains(&n) {
        eprintln!("[state2] frame {n}: gamestate={} replaymode={} parms80={:#x} replayend={}", ts_rt::lw(ts_game::g::gamestate), ts_rt::lw(ts_game::g::replaymode), ts_rt::lw(ts_game::g::curgameparms + 80), ts_rt::lw(ts_game::g::replayend));
    }
    #[cfg(feature = "calltrace")]
    {
        ts_rt::calltrace::set_frame(n as u32);
        if std::env::var("TS_RNGLOG").ok().and_then(|v| v.parse::<u64>().ok()) == Some(n) {
            ts_rt::calltrace::rng_log(true);
        }
    }
    if std::env::var_os("TS_DEMODUMP").is_some() && ts_rt::lw(ts_game::g::replaymode) == 1 && ts_rt::lw(ts_game::g::replayend) == 0 {
        static DONE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        if !DONE.swap(true, std::sync::atomic::Ordering::Relaxed) {
            let (b, l) = (ts_rt::lw(ts_game::g::replaybuf), ts_rt::lw(ts_game::g::replaybuflen));
            std::fs::write("demo.bin", &*ts_rt::host_slice(b, l as usize)).ok();
            eprintln!("[demo] wrote demo.bin ({l} bytes) cursor {:#x} buf {:#x}", ts_rt::lw(0x3afe1c), b);
        }
    }
    if std::env::var_os("TS_STATELOG").is_some() {
        static LAST: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(-1);
        let gs = ts_rt::lw(ts_game::g::gamestate);
        if LAST.swap(gs, std::sync::atomic::Ordering::Relaxed) != gs {
            eprintln!("[state] replay cursor={:#x} buf={:#x} len={:#x}", ts_rt::lw(0x3afe1c), ts_rt::lw(ts_game::g::replaybuf), ts_rt::lw(ts_game::g::replaybuflen));
            eprintln!("[state] frame {n}: gamestate={gs} level={} replaymode={} replayend={}", ts_rt::lw(ts_game::g::lvlevelnum), ts_rt::lw(ts_game::g::replaymode), ts_rt::lw(ts_game::g::replayend));
        }
    }
    if n % 300 == 0 && verbose() {
        let mut keys = std::collections::HashSet::new();
        for d in &frame.draws {
            if let Some(k) = d.state.tex {
                keys.insert((k.tbp0, k.psm, k.tw, k.th, k.cbp));
            }
        }
        use ts_game::g;
        eprintln!(
            "[game] frame {n}: gamestate={} level={} front={:#x} restart={} lvend={} framenum={} tickframenum={}",
            ts_rt::lw(g::gamestate), ts_rt::lw(g::lvlevelnum), ts_rt::lw(g::front), ts_rt::lw(g::restart),
            ts_rt::lw(g::lvend), ts_rt::lw(g::framenum), ts_rt::lw(g::tickframenum)
        );
        let pl = ts_rt::lw(g::players);
        let fl = |a: i32| ts_rt::lwc1(a);
        eprintln!(
            "[game] players={:#x} bgroomsdrawn={:#x} numplayers={} p0 floats@0..16: {:?}",
            pl, ts_rt::lw(g::bgroomsdrawn), ts_rt::lw(g::lvnumplayers),
            (0..16).map(|i| fl(pl + 4 * i)).collect::<Vec<_>>()
        );
        let nimg = ts_gfx::vram::with_vram(|v| v.images.len());
        eprintln!("[gfx] frame {n}: vram images {nimg}, textures used (tbp0,psm,tw,th,cbp) {keys:x?}");
    }
    if n % 60 == 0 && verbose() {
        eprintln!(
            "[gfx] frame {n}: {} draws, {} vertices, {} image uploads, unknown VU1 programs {:x?}",
            frame.draws.len(),
            frame.vertices.len(),
            frame.uploads.len(),
            frame.unknown_programs
        );
    }
    if DUMP_FRAMES.lock().unwrap().contains(&n) {
        let txt = dldump::dump_list(list, 20000);
        let path = format!("displaylist_{n}.txt");
        std::fs::write(&path, txt).ok();
        eprintln!("[port] wrote {path}");
    }
    #[cfg(feature = "shadow")]
    if n % 300 == 0 {
        let s = ts_verify::shadow::summary();
        if !s.is_empty() {
            eprintln!("[shadow] summary at frame {n}:
{s}");
        }
    }
    if n % 60 == 0 && verbose() {
        eprintln!("[port] frame {n}");
    }
    let fbp = if ts_rt::lw(ts_game::g::lastfrbuf) & 1 != 0 { 0 } else { 40 };
    if let Ok(range) = std::env::var("TS_INTERPCHECK") {
        // crack detector: pixels that are dark in the blended picture although neither neighbouring frame has a dark pixel there
        let mut g = PREV_FRAME.lock().unwrap();
        let (a, b) = range.split_once("..").map(|(a, b)| (a.parse::<u64>().unwrap_or(0), b.parse::<u64>().unwrap_or(0))).unwrap_or((0, 0));
        if let (Some(prev), true) = (g.as_ref(), n >= a && n < b && frame.smooth) {
            let mut sg = SHOT_RENDERER.lock().unwrap();
            let r = sg.get_or_insert_with(|| {
                let instance = wgpu::Instance::default();
                ts_render::Renderer::new(&instance, None, wgpu::TextureFormat::Rgba8Unorm)
            });
            r.settings = config::get().render_settings();
            let (w, h) = (640u32, 360u32);
            let blend = ts_render::interpolate(prev, &frame, 0.5);
            let (pa, pb, pc) = (r.screenshot(prev, w, h), r.screenshot(&frame, w, h), r.screenshot_t(&blend, w, h, 0.5, true));
            let pd = pc.clone();
            let dark = |px: &[u8], i: usize| (px[i * 4] as u32 + px[i * 4 + 1] as u32 + px[i * 4 + 2] as u32) < 9;
            let mut bad = 0u32;
            let mut mask = vec![false; (w * h) as usize];
            for y in 3..h as usize - 3 {
                for x in 3..w as usize - 3 {
                    let i = y * w as usize + x;
                    if dark(&pc, i) {
                        // dark in neither neighbour within a 3 pixel radius
                        let mut near = false;
                        'o: for dy in -3i32..=3 {
                            for dx in -3i32..=3 {
                                let j = ((y as i32 + dy) as usize) * w as usize + (x as i32 + dx) as usize;
                                if dark(&pa, j) || dark(&pb, j) {
                                    near = true;
                                    break 'o;
                                }
                            }
                        }
                        if !near {
                            bad += 1;
                            mask[i] = true;
                        }
                    }
                }
            }
            let mut bad_nb = 0u32;
            for y in 3..h as usize - 3 {
                for x in 3..w as usize - 3 {
                    let i = y * w as usize + x;
                    if dark(&pd, i) && !dark(&pb, i) && !dark(&pa, i) {
                        bad_nb += 1;
                    }
                }
            }
            if bad > 40 || bad_nb > 40 {
                eprintln!("[check] frame {n}: {bad} crack pixels with backdrop, {bad_nb} without");
                if std::env::var_os("TS_INTERPCHECK_SAVE").is_some() {
                    std::fs::write(format!("crack_{n}.png"), ts_gfx::png::encode_rgba(w, h, &pc)).ok();
                    std::fs::write(format!("crack_{n}_nb.png"), ts_gfx::png::encode_rgba(w, h, &pd)).ok();
                    std::fs::write(format!("crack_{n}_cur.png"), ts_gfx::png::encode_rgba(w, h, &pb)).ok();
                    
                }
            }
        }
        *g = Some(frame.clone());
    }
    if std::env::var_os("TS_VUPROF").is_some() {
        use std::sync::atomic::Ordering::Relaxed;
        ts_gfx::programs::PROF_ON.store(true, Relaxed);
        if n % 300 == 0 {
            let mut v: Vec<(usize, u64, u64)> = ts_gfx::programs::PROF.iter().enumerate().map(|(i, p)| (i, p.0.swap(0, Relaxed), p.1.swap(0, Relaxed))).filter(|x| x.2 > 0).collect();
            v.sort_by_key(|x| std::cmp::Reverse(x.1));
            let tot: u64 = v.iter().map(|x| x.1).sum();
            eprintln!("[vuprof] frame {n}: total {:.1} ms over 300 frames, runs by entry (ns/run): {}", tot as f64 / 1e6, v.iter().take(6).map(|x| format!("{:#x}: {:.0}% {} runs {:.0}ns", x.0, x.1 as f64 * 100.0 / tot.max(1) as f64, x.2, x.1 as f64 / x.2 as f64)).collect::<Vec<_>>().join(" | "));
        }
    }
    if std::env::var_os("TS_INTERPSTAT").is_some() {
        let mut g = PREV_FRAME.lock().unwrap();
        if let Some(prev) = g.as_ref() {
            let t0 = std::time::Instant::now();
            let f2 = ts_render::interpolate(prev, &frame, 0.5);
            let ms = t0.elapsed().as_secs_f64() * 1e3;
            if f2.vertices.len() > 40000 && n % 20 == 0 {
                eprintln!("[interp] {} verts -> {} verts, {:.2} ms", frame.vertices.len(), f2.vertices.len(), ms);
            }
        }
        if n % 120 == 0 && in_game_now(&frame) {
            let (t, b) = (ts_render::interp_total(), ts_render::interp_blended());
            eprintln!("[interp] frame {n}: world blended in {b} of {t} pictures; failures (unpaired, too far, incoherent): {:?}", ts_render::interp_fails());
            {
                use std::sync::atomic::Ordering::Relaxed;
                eprintln!("[interp] world vertices: {} paired as meshes, {} matched by texture position, {} in total", ts_render::interp::VERTS_PAIRED.swap(0, Relaxed), ts_render::interp::VERTS_MATCHED.swap(0, Relaxed), ts_render::interp::VERTS_TOTAL.swap(0, Relaxed));
            }
        }
        *g = Some(frame.clone());
    }
    if std::env::var_os("TS_INTERPT").is_some() {
        *PREV_FRAME.lock().unwrap() = Some(frame.clone());
    }
    gfx_worker::finish(n, frame, fbp);
    if CLOCK.load(std::sync::atomic::Ordering::Relaxed) >= MAX.load(std::sync::atomic::Ordering::Relaxed) {
        eprintln!("[port] reached frame limit, exiting");
        audio::finish_wav();
        std::process::exit(0);
    }
}

fn main() {
    ts_game::set_av_options(Box::new(menu::PortAv));
    // the translation's own timing counters (clock reads in its inner loops) only run when a report asks for them
    if ["TS_BENCH", "TS_PERFLOG", "TS_VUPROF"].iter().any(|k| std::env::var_os(k).is_some()) {
        ts_gfx::programs::TIMING.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    std::panic::set_hook(Box::new(|info| {
        eprintln!("[port] PANIC: {info}");
        // also keep it in a file: the exe is usually started without a console
        if let Ok(exe) = std::env::current_exe() {
            use std::io::Write;
            let bt = std::backtrace::Backtrace::force_capture();
            if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(exe.with_file_name("ts-port-crash.log")) {
                let _ = writeln!(f, "[PANIC] {info}
{bt}
");
            }
        }
        #[cfg(feature = "calltrace")]
        ts_rt::calltrace::dump_now();
    }));
    let mut game_dir: Option<PathBuf> = None;
    let mut max_frames: Option<u64> = None;
    let mut level: Option<String> = None;
    let mut players: Option<String> = None;
    let mut headless = false;
    let mut save_dir: Option<PathBuf> = None;
    let mut force_window = false;
    let mut overrides: Vec<(String, String)> = Vec::new();
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--game-dir" => game_dir = it.next().map(Into::into),
            "--frames" => max_frames = it.next().and_then(|v| v.parse().ok()),
            "--screenshot" => {
                let n: u64 = it.next().and_then(|v| v.parse().ok()).unwrap_or(0);
                let f = it.next().unwrap_or_else(|| format!("shot_{n}.png"));
                SHOTS.lock().unwrap().push((n, f));
            }
            "--profile" => {
                // --profile <start-frame> <end-frame>: list the functions entered in between
                let a: u64 = it.next().and_then(|v| v.parse().ok()).unwrap_or(0);
                let b: u64 = it.next().and_then(|v| v.parse().ok()).unwrap_or(a + 1);
                PROFILE_WINDOW.store(a, std::sync::atomic::Ordering::Relaxed);
                PROFILE_END.store(b, std::sync::atomic::Ordering::Relaxed);
            }
            "--press" => {
                // --press FRAME:NAME[+NAME..][:DURATION[:PAD]]   e.g. --press 300:start  --press 400:cross+up:5  --press 400:start:3:2
                // (PAD 0..3: with more than one the other controllers are connected; three or more are on a multitap)
                let spec = it.next().unwrap_or_default();
                let mut parts = spec.split(':');
                let at: u64 = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0);
                let names = parts.next().unwrap_or("").to_string();
                let dur: u64 = parts.next().and_then(|v| v.parse().ok()).unwrap_or(4);
                let pad: usize = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0).min(3);
                let (mut b, mut st) = (0u16, [0x80u8; 4]);
                for n in names.split('+') {
                    use ts_sdk::pad::btn::*;
                    match n {
                        "cross" => b |= CROSS, "circle" => b |= CIRCLE, "square" => b |= SQUARE, "triangle" => b |= TRIANGLE,
                        "start" => b |= START, "select" => b |= SELECT, "up" => b |= UP, "down" => b |= DOWN,
                        "left" => b |= LEFT, "right" => b |= RIGHT, "l1" => b |= L1, "r1" => b |= R1, "l2" => b |= L2, "r2" => b |= R2,
                        "lup" => st[3] = 0, "ldown" => st[3] = 255, "lleft" => st[2] = 0, "lright" => st[2] = 255,
                        "rup" => st[1] = 0, "rdown" => st[1] = 255, "rleft" => st[0] = 0, "rright" => st[0] = 255,
                        other => eprintln!("unknown button {other}"),
                    }
                }
                PAD_SCRIPT.lock().unwrap().push((at, dur, b, st, pad));
                if pad >= 2 {
                    // (the multitap is there from the start, as the game looks for it when it starts)
                    ts_sdk::pad::set_multitap(Some([ts_sdk::pad::PadState::idle(true); 3]));
                }
                window::SCRIPTED.store(true, std::sync::atomic::Ordering::Relaxed);
            }
            "--save-dir" => save_dir = it.next().map(Into::into),
            "--wav" => {
                if let Some(p) = it.next() {
                    audio::enable_wav(&p);
                }
            }
            "--headless" => headless = true,
            "--window" => force_window = true,
            "--set" => {
                if let Some((k, v)) = it.next().and_then(|a| a.split_once('=').map(|(k, v)| (k.to_string(), v.to_string()))) {
                    overrides.push((k, v));
                }
            }
            "--level" => level = it.next(),
            "--players" => players = it.next(),
            "--dump-frames" => {
                let v: Vec<u64> = it.next().unwrap_or_default().split(',').filter_map(|x| x.parse().ok()).collect();
                *DUMP_FRAMES.lock().unwrap() = v;
            }
            other => eprintln!("ignoring unknown argument {other}"),
        }
    }
    let windowed = force_window || !(headless || !SHOTS.lock().unwrap().is_empty());
    WINDOWED.store(windowed, std::sync::atomic::Ordering::Relaxed);
    // the game's files have to be in the program's own folder (`--game-dir` is for testing)
    let game_dir = game_dir.unwrap_or_else(exe_dir);
    if !game_dir.join("PAK").is_dir() || !game_dir.join("SLUS_200.90").is_file() {
        fatal(&format!("The game's files are missing.

Put the PAK and MUSIC folders and SLUS_200.90 of the game in the same folder as the program:
{}", game_dir.display()));
    }
    eprintln!("[port] game directory: {}", game_dir.display());

    ts_rt::init_memory();
    // The game's initialised data (strings, tables, defaults) is read from the original executable of the game.
    let exe = game_dir.join("SLUS_200.90");
    if let Err(e) = ts_rt::loader::load_elf_data(&exe) {
        fatal(&format!("The game's executable cannot be read.

SLUS_200.90 has to be in the same folder as the program:
{}

({e})", exe.display()));
    }
    #[cfg(feature = "shadow")]
    ts_verify::shadow::install(&game_dir.join("SLUS_200.90"));
    config::load(&overrides);
    ts_sdk::set_disc_root(&game_dir);
    mods::load(&exe_dir(), &exe);
    ts_game::weapon_table_init();
    ts_game::native_text_init();
    ts_game::sound_mix_init();
    // test aid: TS_BUTTONS=xbox|switch|keyboard names the buttons of that device in the texts (headless runs have no device)
    match std::env::var("TS_BUTTONS").as_deref() {
        Ok("xbox") => ts_sdk::pad::set_input_style(1),
        Ok("switch") => ts_sdk::pad::set_input_style(2),
        Ok("keyboard") => ts_sdk::pad::set_input_style(3),
        _ => {}
    }
    // the saves (the virtual memory card) are in the program's folder too
    let save_dir = save_dir.unwrap_or_else(|| exe_dir().join("saves"));
    eprintln!("[port] save directory: {}", save_dir.display());
    ts_sdk::set_save_root(&save_dir);

    MAX.store(max_frames.unwrap_or(u64::MAX), std::sync::atomic::Ordering::Relaxed);
    ts_sdk::set_upload_hook(|u| {
        static N: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let k = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if k < 12 {
            eprintln!("[gfx] upload #{k}: {u:?}");
        }
        let bits = match u.psm {
            0 | 48 => 32,
            1 | 49 => 24,
            2 | 10 | 50 | 58 => 16,
            19 | 27 => 8,
            20 | 36 | 44 => 4,
            _ => 32,
        };
        let bytes = (u.w as usize * u.h as usize * bits).div_ceil(8);
        if std::env::var_os("TS_UPLOGLOG").is_some() {
            eprintln!("[upload] frame {} dbp {:#x} dbw {} psm {} size {}x{} bytes {}", ts_sdk::frame_count(), u.dbp, u.dbw, u.psm, u.w, u.h, bytes);
        }
        let data = ts_rt::host_slice(u.data, bytes).to_vec();
        // Not written to texture memory yet: the drawing threads may still be working on earlier frames.  The upload is
        // queued with the number of the frame being built and applied when that frame is translated.
        PENDING_UPLOADS.lock().unwrap().push_back((ts_sdk::frame_count(), ts_gfx::vram::Image { dbp: u.dbp, dbw: u.dbw, psm: u.psm, w: u.w, h: u.h, data, generation: 0 }));
    });
    ts_sdk::set_sync_hook(gfx_worker::sync);
    ts_sdk::set_present_hook(|list| {
        let n = ts_sdk::frame_count();
        let clk = match ts_game::level_tick() {
            Some(t) if std::env::var_os("TS_TICKCLOCK").is_some() => 1_000_000 + t as u64,
            // outside levels the clock counts whole ticks since the start (the same as frames at 60 Hz)
            _ if std::env::var_os("TS_TICKCLOCK").is_some() => ts_rt::lw(ts_game::g::tickframenum) as u64,
            _ => n,
        };
        CLOCK.store(clk, std::sync::atomic::Ordering::Relaxed);
        #[cfg(feature = "calltrace")]
        {
            let (a, b) = (PROFILE_WINDOW.load(std::sync::atomic::Ordering::Relaxed), PROFILE_END.load(std::sync::atomic::Ordering::Relaxed));
            static STARTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
            if clk >= a && !STARTED.swap(true, std::sync::atomic::Ordering::Relaxed) {
                ts_rt::calltrace::profile(true);
            }
            if clk >= b && STARTED.load(std::sync::atomic::Ordering::Relaxed) && PROFILE_END.swap(0, std::sync::atomic::Ordering::Relaxed) != 0 {
                ts_rt::calltrace::profile(false);
                ts_rt::calltrace::profile_dump(3000);
            }
        }
        {
            let script = PAD_SCRIPT.lock().unwrap();
            if !script.is_empty() {
                let npads = script.iter().map(|e| e.4 + 1).max().unwrap_or(1);
                let mut pads = [ts_sdk::pad::PadState::idle(true), ts_sdk::pad::PadState::idle(npads > 1), ts_sdk::pad::PadState::idle(npads > 2), ts_sdk::pad::PadState::idle(npads > 3)];
                for &(at, dur, b, st, pad) in script.iter() {
                    if clk >= at && clk < at + dur {
                        let s = &mut pads[pad];
                        s.buttons |= b;
                        for i in 0..4 {
                            if st[i] != 0x80 {
                                s.sticks[i] = st[i];
                            }
                        }
                    }
                }
                if npads >= 3 {
                    ts_sdk::pad::set_multitap(Some([pads[1], pads[2], pads[3]]));
                    ts_sdk::set_pad_state(0, pads[0]);
                    ts_sdk::set_pad_state(1, ts_sdk::pad::PadState::idle(false));
                } else {
                    ts_sdk::pad::set_multitap(None);
                    ts_sdk::set_pad_state(0, pads[0]);
                    ts_sdk::set_pad_state(1, pads[1]);
                }
            }
        }
        ts_sdk::perf::frame();
        let world = ts_sdk::take_world_ranges();
        if pipelined() {
            gfx_worker::submit(n, list, world);
        } else {
            process_frame(n, list, world);
        }
    });

    // Watchdog: if no frame is presented for a while, or the game stays on its loading state (6), write what the game was waiting
    // for to `ts-port-hang.txt` next to the program (and ask the game thread for its call stack, needs `calltrace`).
    std::thread::spawn(|| {
        use ts_game::g;
        let mut last = u64::MAX;
        let mut loading = 0u32;
        loop {
            std::thread::sleep(std::time::Duration::from_secs(2));
            let n = ts_sdk::frame_count();
            let stuck_frames = n == last;
            last = n;
            let state = ts_rt::lw(g::gamestate);
            loading = if state == 6 { loading + 1 } else { 0 };
            if stuck_frames || loading >= 4 {
                let report = format!(
                    "{}: frame {n} stuck_frames {stuck_frames} gamestate {state} level {} lvend {} music_wait {} tunenum {} musicstate {} musicopened {} music_playing {} players {} page_depth {} front_state {}
",
                    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs()),
                    ts_rt::lw(g::lvlevelnum), ts_rt::lw(g::lvend), ts_rt::lw(0x3afefc), ts_rt::lw(g::tunenum), ts_rt::lw(g::musicstate), ts_rt::lw(g::musicopened),
                    ts_audio::music_playing(), ts_rt::lw(g::lvnumplayers), ts_rt::lw(0x3aff2c), ts_rt::lw(g::frontstate),
                );
                eprint!("[port] {report}");
                if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(exe_dir().join("ts-port-hang.txt")) {
                    use std::io::Write;
                    let _ = f.write_all(report.as_bytes());
                }
                #[cfg(feature = "calltrace")]
                ts_rt::calltrace::request_dump();
            }
            // last resort: the game leaves its loading state when the music has stopped (five checks in a row); if that does not
            // happen the music is closed and the state is left by hand
            if loading >= 4 && ts_rt::lw(g::lvend) != 666 {
                eprintln!("[port] loading state does not end: closing the music and ending the level by hand");
                ts_audio::stream_command(ts_audio::MusicCmd::Close);
                ts_rt::sw(g::lvend, 666);
            }
        }
    });

    // The decompiled game keeps its working state in the game address space; run it on a dedicated big-stack thread.
    // argv as the original `main(argc, argv)` expects: [program, level name, players]; no arguments = front end.
    let mut argv_strings = vec!["timesplitters".to_string()];
    if let Some(l) = &level {
        argv_strings.push(l.clone());
        argv_strings.push(players.clone().unwrap_or_else(|| "1".into()));
    }
    let argc = if level.is_some() { argv_strings.len() as i32 } else { 0 };
    let argv = {
        let table = ts_sdk::malloc(4 * (argv_strings.len() as i32 + 1));
        for (i, s) in argv_strings.iter().enumerate() {
            let p = ts_sdk::malloc(s.len() as i32 + 1);
            for (k, b) in s.bytes().enumerate() {
                ts_rt::sb(p + k as i32, b as i32);
            }
            ts_rt::sb(p + s.len() as i32, 0);
            ts_rt::sw(table + 4 * i as i32, p);
        }
        table
    };
    if windowed {
        audio::start_device();
    }
    let run_game = move || {
        let h = std::thread::Builder::new()
            .name("game".into())
            .stack_size(256 << 20)
            .spawn(move || ts_game::main_(argc, argv))
            .unwrap();
        if windowed_exit_watch(h) {
            std::process::exit(1);
        }
        std::process::exit(0);
    };
    if windowed {
        // the OS event loop must own the main thread; the game thread is started once the window exists
        window::run(move || {
            std::thread::spawn(run_game);
        });
    } else {
        run_game();
    }
}

/// Wait for the game thread; returns true if it panicked.
fn windowed_exit_watch(h: std::thread::JoinHandle<()>) -> bool {
    if let Err(e) = h.join() {
        let msg = e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()));
        eprintln!("[port] game thread panicked: {}", msg.unwrap_or_default());
        return true;
    }
    false
}
