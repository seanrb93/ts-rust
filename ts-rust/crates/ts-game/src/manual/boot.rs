//! Hand-written platform glue for the boot / frame loop.
//!
//! Original structure: `main` created a CPU thread (game logic, builds display lists) and a GS thread (kicks the
//! lists to the graphics hardware), then slept while vertical-blank interrupts swapped buffers.  Here the frame loop
//! (`cpuMain`, ported in `boot_gen.rs`) runs directly and hands each finished list to the renderer.

use crate::*;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

static TIMEBASE: AtomicU32 = AtomicU32::new(0);

/// Platform initialisation formerly done in `bossMake`: IOP reboot, module loading, CD init, DMA / GS reset and the
/// semaphores + interrupt handlers of the GS thread.  None of that exists on PC; only the double-buffer
/// descriptor setup the rest of the game still reads is kept.
pub fn bossMake() {
    sjeGsSetDefDBuff(2, 640, 224, 2, 50, 0, 1);
    sd(g::db + 432, 1);
    sd(g::db + 192, 1);
}

/// Called once per finished display list (replaces the GS thread + vblank interrupt).
/// After "Load Sign Ons" the game stores the new profile's index in the *whole* signon table as the sign-on page's
/// cursor, but the menu numbers only the profiles stored on a card (the in-memory default player is hidden), so the
/// cursor can name an item that does not exist and the page ignores all input.  Move such a cursor onto the last
/// listed profile.
fn fix_signon_cursor() {
    let page = 0x1fc01d0; // sign-on player page of player 0 (cursor index at +4)
    let idx = lhu(page + 4);
    // only while the sign-on page itself is showing (page-stack level 1 or 2 of front 0 is `signon_page`)
    let front = g::frontdefs;
    if lw(front + 20) < 1 || lw(lw(front + 4)) != g::signon_page {
        return;
    }
    if lbu(0x3aff28) != 2 || idx == 0 || idx >= 2000 || menuitem(g::signonplr_menu, idx) != 0 {
        return;
    }
    for id in (1..idx).rev() {
        if menuitem(g::signonplr_menu, id) != 0 {
            sh(page + 4, id);
            return;
        }
    }
}

/// Debugging aid (`TS_WATCH=<address>`, `TS_WATCHFROM=<tick>`; needs the `calltrace` build): report the functions that
/// store to a game address once the level clock passes `TS_WATCHFROM`.
fn fake_goodies() {
    // Debugging aid (`TS_FAKEGOODIES=<frame>`): open the "new playable character" page, to check its layout.
    static AT: std::sync::OnceLock<Option<i32>> = std::sync::OnceLock::new();
    if let Some(at) = *AT.get_or_init(|| std::env::var("TS_FAKEGOODIES").ok().and_then(|v| v.parse().ok())) {
        if lw(g::framenum) == at {
            sw(0x3affa0, lw(g::unlockedcheats) ^ 1);
            let mut chrs = [0i32; 3];
            for (i, c) in chrs.iter_mut().enumerate() {
                *c = lw(0x1fc33b0 + 4 * i as i32);
            }
            sw(0x1fc33b0, chrs[0] ^ 1);
            frontmpgSetGoodiesPage();
        }
    }
}

fn watch_tick() {
    fake_goodies();
    #[cfg(feature = "calltrace")]
    {
        // TS_FPROF=<from>,<to>: profile the float stores between two level ticks (see ts_rt::calltrace::fprof)
        static RANGE: std::sync::OnceLock<Option<(f32, f32)>> = std::sync::OnceLock::new();
        static STATE: AtomicU32 = AtomicU32::new(0);
        let r = *RANGE.get_or_init(|| {
            let v = std::env::var("TS_FPROF").ok()?;
            let (a, b) = v.split_once(',')?;
            Some((a.parse().ok()?, b.parse().ok()?))
        });
        if let (Some((from, to)), Some(t)) = (r, level_tick()) {
            let st = STATE.load(Ordering::Relaxed);
            if st == 0 && t >= from {
                ts_rt::calltrace::fprof(true);
                STATE.store(1, Ordering::Relaxed);
            } else if st == 1 && t >= to {
                ts_rt::calltrace::fprof(false);
                ts_rt::calltrace::fprof_dump();
                STATE.store(2, Ordering::Relaxed);
            }
        }
    }
    #[cfg(any(feature = "calltrace", feature = "watch"))]
    {
        static ADDR: std::sync::OnceLock<Option<u32>> = std::sync::OnceLock::new();
        let a = *ADDR.get_or_init(|| std::env::var("TS_WATCH").ok().and_then(|v| u32::from_str_radix(v.trim_start_matches("0x"), 16).ok()));
        if let Some(a) = a {
            let from: f32 = std::env::var("TS_WATCHFROM").ok().and_then(|v| v.parse().ok()).unwrap_or(0.0);
            let on = if let Some(f) = std::env::var("TS_WATCHFRAME").ok().and_then(|v| v.parse::<u64>().ok()) {
                ts_sdk::frame_count() >= f && ts_sdk::frame_count() < f + 12
            } else {
                level_tick().is_some_and(|t| t >= from && t < from + 3.0)
            };
            ts_rt::watch::set(if on { a } else { 0 });
        }
    }
}

fn page_log() {
    #[cfg(any(feature = "calltrace", feature = "watch"))]
    if let Some(w) = std::env::var("TS_WATCHPAGE").ok().and_then(|v| v.parse::<i32>().ok()) {
        // watch the cursor of the bottom page of front `w`
        let stack = lw(g::frontdefs + w * 44 + 4);
        ts_rt::watch::set(if stack != 0 { stack as u32 + 4 } else { 0 });
    }
    if std::env::var_os("TS_PAGELOG").is_none() {
        return;
    }
    let mut line = format!("[page] f={} lvl={}", lw(g::framenum), lw(0x3aff2c));
    for fi in 0..4 {
        let front = g::frontdefs + fi * 44;
        let (stack, n) = (lw(front + 4), lw(front + 20));
        line += &format!(" || F{fi} n={n}");
        for i in 0..n.clamp(0, 8) {
            let e = stack + i * 16;
            let d = lw(e);
            line += &format!(" | d={:#x} cur={} (d+8)={}", d, lhu(e + 4), lhu(d + 8));
        }
    }
    static LAST: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());
    let mut l = LAST.lock().unwrap();
    let key = line.split_once(" ||").map(|x| x.1.to_string()).unwrap_or_default();
    if *l != key {
        *l = key;
        eprintln!("{line}");
    }
}

/// Widescreen / field-of-view: the camera setup (`lvCamera`-side code) multiplies the window's aspect by a constant at
/// 0x3acfc8 and converts the player's field of view (degrees) to radians with a constant at 0x3acfcc.  Rewriting those two
/// words each frame widens the picture (and the culling frustum, which is derived from the same numbers) and scales the view.
fn apply_view() {
    static BASE: std::sync::OnceLock<(f32, f32)> = std::sync::OnceLock::new();
    let (b20, b22) = *BASE.get_or_init(|| {
        let b = (lwc1(0x3acfc8), lwc1(0x3acfcc));
        if std::env::var_os("TS_VIEWLOG").is_some() {
            eprintln!("[view] base aspect constant {} degree constant {}", b.0, b.1);
        }
        b
    });
    let (aspect, hfov) = ts_sdk::view();
    let k = aspect / (4.0 / 3.0);
    let mut deg_scale = 1.0f32;
    // only the level camera is scaled: the front-end's 3D previews (character select) keep the game's own field of view
    // (3 = playing, 4 = level finished / game over / retry-or-quit screen, 5 = leaving the level: the level is still shown)
    if hfov > 1.0 && matches!(lw(g::gamestate), 3..=5) {
        // the game's vertical field of view is fixed per player; derive the scale from the unzoomed value of player 0
        let p = lw(g::players);
        let v0 = lwc1(p + 260);
        let known = FOV_BASE.with(|f| f.get()) > 0.0;
        if std::env::var_os("TS_VIEWLOG").is_some() {
            eprintln!("[view] player fov {v0}");
        }
        if known || (v0 > 1.0 && v0 < 170.0) {
            // `hfov` is the field of view of the 4:3 picture; wider pictures show more to the sides at the same height
            let target_v = 2.0 * ((hfov.to_radians() * 0.5).tan() / (4.0 / 3.0)).atan().to_degrees();
            deg_scale = target_v / FOV_BASE.with(|f| {
                if f.get() == 0.0 {
                    f.set(v0);
                }
                f.get()
            });
        }
    }
    swc1(0x3acfc8, b20 * k);
    swc1(0x3acfcc, b22 * deg_scale);
}

/// A demo (attract mode) is being played back inside a level.
pub fn demo_playing() -> bool {
    let state = lw(g::gamestate);
    // the demo's level is being loaded: its parameters (flag 0x2000) wait in the deferred-reset block at 0x1fbc0e8
    (lw(g::replaymode) == 1 && matches!(state, 2..=5)) || (state == 6 && lw(0x1fbc0e8 + 80) & 0x2000 != 0)
}

thread_local! {
    static FOV_BASE: std::cell::Cell<f32> = const { std::cell::Cell::new(0.0) };
}

/// A demo stores the game's random-number state on every frame and ends as soon as the running game disagrees with it.
/// The port's arithmetic is not bit-identical to the console's, so after a while the state drifts (a bullet that hit
/// there misses here); keep the demo going by handing the game the recorded state before the next frame reads it.
fn demo_resync() {
    if lw(g::replaymode) != 1 || lw(g::replayend) != 0 || lw(g::gamestate) != 3 {
        return;
    }
    let cursor = lw(0x3afe1c);
    if (cursor as u32) + 26 > (lw(g::replaybuf) + lw(g::replaybuflen)) as u32 {
        return;
    }
    let recorded = lw(cursor + 18);
    if lw(g::newrndseedval) != recorded {
        if std::env::var_os("TS_DEMOLOG").is_some() {
            eprintln!("[demo] resync random state {:#x} -> {:#x}", lw(g::newrndseedval), recorded);
        }
        sw(g::newrndseedval, recorded);
    }
}

pub fn present_frame() {
    ts_sdk::present_sync();
    chr_names_once();
    mouse_look_tick();
    debug_menu_tick();
    fx_stress();
    av_frame_end();
    apply_view();
    demo_resync();
    page_log();
    fix_signon_cursor();
    let list = lw(lw(g::curcpudmabuf) * 4 + g::dmalists);
    // The GS thread probed the z-buffer of the frame on screen before sending the next list: copy it and evaluate the
    // glow visibility tests registered for it (see `zbtest*`).
    zbtestCopyZB();
    zbtestDoTest();
    zbtestAdvanceTestPtr();
    ts_sdk::present(list);
    if std::env::var_os("TS_PLAYERLOG").is_some() {
        let p = lw(g::players);
        let f = |o: i32| lwc1(p + o);
        eprintln!(
            "[pl] frame {} gameframe {} +{:.3} tickframenum {} dt {:.3}/{} pos {:.2} {:.2} {:.2} | {:.2} {:.2} {:.2} | ang {:.2} {:.2} | spd {:.3} {:.3} {:.3} | st {} {} {} but {:#x} | bob {:.3} {:.3} {:.3} {:.2} {:.2} {:.4} {:.4} {:.4}",
            ts_sdk::frame_count(),
            lw(g::gameframe),
            f32::from_bits(TICK_ACC.load(Ordering::Relaxed)),
            lw(g::tickframenum),
            lwc1(g::updateframesf),
            lw(g::updateframesi),
            f(140), f(144), f(148), f(116), f(120), f(124), f(276), f(284), f(212), f(216), f(220), lw(p + 20), lw(p + 288), lw(p + 296), joyBut(0), f(312), f(316), f(320), f(324), f(328), f(376), f(396), f(400)
        );
    }
    snapshot();
    watch_tick();
    if std::env::var_os("TS_MENULOG").is_some() {
        eprintln!("[menu] f{} state {} tick {} skip {} pc {:#x} front {} players {:#x} cur {:#x} nplayers {} fxstate {} fronttime {:.3} scale {:.3} {:.3} replay {}", ts_sdk::frame_count(), lw(g::gamestate), lw(g::tickframenum), menu_skip(), lw(g::playercontrollers), lw(lw(g::front) + 20), lw(g::players), lw(g::currentplayer), lw(g::lvnumplayers), lw(g::frontstate), lwc1(g::fronttime), lwc1(g::globalscalex), lwc1(g::globalscaley), lw(g::replaymode));
    }
    // "Unlock Everything": the game's own switch for it (what its cheat code sets) is kept on while the option is on
    static UNLOCKED: AtomicBool = AtomicBool::new(false);
    if ts_sdk::unlock_all() {
        sw(g::val1, 1);
        UNLOCKED.store(true, Ordering::Relaxed);
    } else if UNLOCKED.swap(false, Ordering::Relaxed) {
        sw(g::val1, 0);
    }
    // test aid: TS_SCORE=<frame> keeps every player's score box shown from then on
    if let Ok(v) = std::env::var("TS_SCORE") {
        if v.parse::<u64>().map_or(false, |f| ts_sdk::frame_count() as u64 >= f) && lw(g::gamestate) == 3 {
            for p in 0..4 {
                sw(g::hudparms + p * 352 + 276, 40);
                sw(g::hudparms + p * 352 + 280, 80);
            }
        }
    }
    if std::env::var_os("TS_INGAMELOG").is_some() && lw(g::gamestate) == 3 && ts_sdk::frame_count() % 20 == 0 {
        let v: Vec<String> = (0..4).map(|p| { let b = 0x1fb3240 + 92 * p; format!("{}:{:#x}/{}/{:#x}", p, lhu(b + 8), lhu(b + 2), lhu(b + 4)) }).collect();
        eprintln!("[ig] f{} {} front20 {}", ts_sdk::frame_count(), v.join(" "), lw(lw(g::front) + 20));
    }
    // test aid: TS_NOGUN=1 sets "Always start with gun" to No (outside levels)
    if std::env::var_os("TS_NOGUN").is_some() && lw(g::gamestate) != 3 {
        sw(g::gv_spawngun, 0);
    }
    // test aid: TS_BOTSET=a,b,c uses these characters as custom bot set 1 (outside levels)
    if let (Ok(v), true) = (std::env::var("TS_BOTSET"), lw(g::gamestate) != 3) {
        let ids: Vec<i32> = v.split(',').filter_map(|x| x.trim().parse().ok()).collect();
        for k in 0..10 {
            sb(0x1fc2b28 + k, ids.get(k as usize).copied().unwrap_or(-1));
            sw(0x1fc2a88 + 16 * k, ids.get(k as usize).copied().unwrap_or(-1));
            sw(0x1fc2a88 + 16 * k + 12, 1);
        }
        sw(g::gv_chrset, 8);
    }
    if std::env::var_os("TS_CHRORDER").is_some() && ts_sdk::frame_count() == 50 {
        let v: Vec<i32> = (0..64).map(|i| lbu(g::characterorder + i)).collect();
        eprintln!("[chrorder] {:?}", v);
    }
    // test aid: TS_WSET=a,b,c,d,e uses these weapon combinations as the custom weapon set (outside levels)
    if let (Ok(v), true) = (std::env::var("TS_WSET"), lw(g::gamestate) != 3) {
        let ids: Vec<i32> = v.split(',').filter_map(|x| x.trim().parse().ok()).collect();
        // (the sets are set up on the first ask: that must not come after the ids are written)
        let _ = weaponset_effective();
        for k in 0..5 {
            sw(g::gunzpack + 4 * k, ids.get(k as usize).copied().unwrap_or(-1));
        }
        sw(g::gv_weaponset, 0);
        if std::env::var_os("TS_COMBOLOG").is_some() && ts_sdk::frame_count() % 300 == 0 {
            eprintln!("[combo] TS_WSET applied at f{} state {} ids {:?}", ts_sdk::frame_count(), lw(g::gamestate), ids);
        }
    }
    if std::env::var_os("TS_BOTLOG").is_some() && lw(g::gamestate) == 3 && ts_sdk::frame_count() % 120 == 0 {
        let b = lw(g::chrinfo) + 0x1210;
        eprintln!("[bot] chars {} f{} state {:#x} prev {:#x} flags {:#x} {:#x} mode {} target {:#x} gun {} combo {} pos {:.1},{:.1}", lw(g::numactivechars), ts_sdk::frame_count(), lw(b + 2716), lw(b + 2720), lw(b + 2704), lw(b + 2708), lw(b + 2724), lw(b + 2788), lw(lw(lw(b + 3020) + 352) + 376), lw(lw(lw(b + 3020) + 352) + 260), lwc1(lw(b + 3020) + 48), lwc1(lw(b + 3020) + 56));
    }
    if std::env::var_os("TS_COMBOLOG").is_some() && lw(g::gamestate) == 3 && ts_sdk::frame_count() % 120 == 0 {
        // test aid: the weapon combination and what player 1 owns
        let s = lw(lw(lw(g::players) + 384) + 352);
        let owned: Vec<i32> = (0..combo_count()).filter(|&c| own_get(s, c) != 0).collect();
        eprintln!("[combo] f{} combo {} next {} owned {:?} left {} right {}", ts_sdk::frame_count(), lw(s + 260), lw(s + 264), owned, lw(s + 376), lw(s + 268));
    }
    // test aid: TS_GIVE=<frame>:<combo> gives player 1 the weapon at that frame of a level
    if let (Ok(v), true) = (std::env::var("TS_GIVE"), lw(g::gamestate) == 3) {
        if let Some((f, c)) = v.split_once(':') {
            if f.parse::<u64>().ok() == Some(ts_sdk::frame_count() as u64) {
                let s = lw(lw(lw(g::players) + 384) + 352);
                let c: i32 = c.parse().unwrap_or(0);
                own_set(s, c);
                gunSetAmmoToMax(s, c);
            }
        }
    }
    if std::env::var_os("TS_TEAMPLAY").is_some() && lw(g::gamestate) != 3 {
        sw(g::gv_teamplay, 1);
    }
    // test aid: TS_QUIT=<frame> leaves the level then (as Quit on the end screen does)
    if let Ok(v) = std::env::var("TS_QUIT") {
        if v.parse::<u64>().ok() == Some(ts_sdk::frame_count() as u64) {
            gameLevelQuit();
        }
    }
    // test aid: TS_ENDGAME=<frame> ends the match then (as a time limit does)
    if let Ok(v) = std::env::var("TS_ENDGAME") {
        if v.parse::<u64>().ok() == Some(ts_sdk::frame_count() as u64) && lw(g::gamestate) == 3 {
            gameLevelCompleted();
        }
    }
    // test aid: TS_TELEPORT=<frame>,<x>,<y>,<z> puts the first player there when the frame is reached
    if let Ok(v) = std::env::var("TS_TELEPORT") {
        let n: Vec<f32> = v.split(',').filter_map(|x| x.trim().parse().ok()).collect();
        if n.len() == 4 && ts_sdk::frame_count() as f32 == n[0] && lw(g::gamestate) == 3 && lw(g::players) != 0 {
            let p = lw(g::players);
            for k in 0..3 {
                swc1(p + 140 + 4 * k, n[1 + k as usize]);
            }
        }
    }
    if std::env::var_os("TS_PAINTEST").is_some() && lw(g::gamestate) == 3 {
        // test aid: every player is in pain (the red vignette of the HUD)
        for i in 0..4 {
            sw(g::hudparms + i * 352 + 268, 250);
        }
    }
    if std::env::var_os("TS_WINLOG").is_some() && ts_sdk::frame_count() % 20 == 0 {
        eprintln!("[win] f{} state {} win {} {} {} {} {} {} {} {} pl {}", ts_sdk::frame_count(), lw(g::gamestate), lw(g::window), lw(g::window + 4), lw(g::window + 8), lw(g::window + 12), lw(g::window + 16), lw(g::window + 20), lw(g::window + 32), lw(g::window + 36), lw(g::lvnumplayers));
    }
    let hz = ts_sdk::logic_hz();
    if hz > 60.5 && (hirate_ok() || menu_hirate()) {
        // A level (or the front end) at more than 60 frames per second: the next frame stands for 60/hz game ticks.
        let ticks = (ts_sdk::wait_frame(hz as f64) * 60.0) as f32;
        PENDING_DT.store(ticks.to_bits(), Ordering::Relaxed);
        // the frame counter keeps following the 60 Hz clock
        let mut acc = f32::from_bits(VBL_ACC.load(Ordering::Relaxed)) + ticks;
        while acc >= 1.0 {
            timeTickRetrace();
            acc -= 1.0;
        }
        VBL_ACC.store(acc.to_bits(), Ordering::Relaxed);
    } else {
        // The vertical-blank interrupt drove the game clock; run it once per elapsed 60 Hz tick.
        for _ in 0..ts_sdk::wait_vsync() {
            timeTickRetrace();
        }
    }
}

/// Debugging aid (`TS_STRESSFX=<period>`): every `period` frames of a level, set off an explosion in front of the player
/// (`TS_STRESSKIND` picks the kind, default 5), to measure how the drawing copes with particle-heavy scenes.
fn fx_stress() {
    static PERIOD: std::sync::OnceLock<u64> = std::sync::OnceLock::new();
    let period = *PERIOD.get_or_init(|| std::env::var("TS_STRESSFX").ok().and_then(|v| v.parse().ok()).unwrap_or(0));
    if period == 0 || lw(g::gamestate) != 3 || lw(g::lvfirstframe_130) != 0 || lw(g::gamepaused) != 0 || ts_sdk::frame_count() % period != 0 {
        return;
    }
    let kind: i32 = std::env::var("TS_STRESSKIND").ok().and_then(|v| v.parse().ok()).unwrap_or(5);
    let frame = ts_rt::Frame::new(64);
    let sp = frame.sp();
    let p = lw(g::players);
    let f = |o: i32| lwc1(p + o);
    let dist: f32 = std::env::var("TS_STRESSDIST").ok().and_then(|v| v.parse().ok()).unwrap_or(5.0);
    let count: i32 = std::env::var("TS_STRESSN").ok().and_then(|v| v.parse().ok()).unwrap_or(1);
    for n in 0..count {
        // a spread of explosions in front of the player
        let side = (n % 5 - 2) as f32 * 1.5;
        for i in 0..3 {
            let right = [f(196), 0.0, -f(188)][i as usize];
            swc1(sp + 4 * i, f(176 + 4 * i) + (dist + (n / 5) as f32 * 2.0) * f(188 + 4 * i) + side * right);
        }
        particleNewExplosion(0, if count > 1 { [5, 2, 0, 1, 3][(n % 5) as usize] } else { kind }, sp, 0, 0, 0, 0, 0);
    }
}

/// Debugging aid (`TS_SNAP=300,600 TS_SNAPFILE=prefix`): write the game's RAM to `prefix_<tick>.bin` once the game clock
/// passes each tick count.  Runs at different frame rates can then be compared word by word.
fn snapshot() {
    static TARGETS: std::sync::OnceLock<Vec<f32>> = std::sync::OnceLock::new();
    static DONE: AtomicU32 = AtomicU32::new(0);
    let t = TARGETS.get_or_init(|| std::env::var("TS_SNAP").ok().map(|v| v.split(',').filter_map(|x| x.parse().ok()).collect()).unwrap_or_default());
    if let Some(v) = std::env::var_os("TS_FRAMESNAP") {
        // `TS_FRAMESNAP=frame:file,...`: dump RAM after the given frame (frame-exact, for finding state a redraw changes)
        let fc = ts_sdk::frame_count();
        for item in v.to_string_lossy().split(',') {
            if let Some((f, file)) = item.split_once(':') {
                if f.parse::<u64>().ok() == Some(fc as u64) {
                    std::fs::write(file, host_slice(0, 0x200_0000)).ok();
                }
            }
        }
    }
    let i = DONE.load(Ordering::Relaxed) as usize;
    if i < t.len() && lw(g::gamestate) == 3 && gameframe_f() >= t[i] {
        DONE.store(i as u32 + 1, Ordering::Relaxed);
        let prefix = std::env::var("TS_SNAPFILE").unwrap_or_else(|_| "snap".into());
        std::fs::write(format!("{prefix}_{}.bin", t[i]), host_slice(0, 0x200_0000)).ok();
    }
}

// ---- frame rates above 60 Hz ----------------------------------------------------------------------------------------
//
// The game measures the time a frame stands for in 60 Hz ticks: `updateframesf` (float, scales movement) and
// `updateframesi` (whole ticks, drives timers and counters).  The original never saw less than one tick per frame; at a
// higher rate a frame stands for a fraction of a tick, `updateframesf` holds that fraction and the whole ticks are
// handed out as they are completed (`updateframesi` is 0 on most frames).

static PENDING_DT: AtomicU32 = AtomicU32::new(0);
static TICK_ACC: AtomicU32 = AtomicU32::new(0);
static VBL_ACC: AtomicU32 = AtomicU32::new(0);

/// A level is being played with nothing from the front end (menus, pause, retry screens) on top of it: the only
/// situation in which the game runs above 60 Hz.  Everything else is per-frame logic written for exactly 60 Hz.
pub fn hirate_ok() -> bool {
    lw(g::gamestate) == 3
        && lw(lw(g::front) + 20) == 0
        && lw(g::gamepaused) == 0
        && lw(g::replaymode) == 0
        && lw(g::lvfirstframe_130) == 0
        && ilinkIsILinkGame() == 0
}

static MENU_SKIP: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static MENU_DT: AtomicU32 = AtomicU32::new(0x3f80_0000); // 1.0
static MENU_FX: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(-1);
static MENU_STEP_ACC: AtomicU32 = AtomicU32::new(0);

/// The front end (menus) is showing and the game runs above 60 Hz.
pub fn menu_hirate() -> bool {
    // (not in the first seconds: the pad and page structures are set up by the first few ticks of logic)
    ts_sdk::logic_hz() > 60.5 && lw(g::gamestate) == 1 && lw(g::tickframenum) > 180 && lw(g::replaymode) == 0 && ilinkIsILinkGame() == 0
}

/// This frame completes no whole tick of the front end: its logic and pad sampling are skipped (patched into `bossMainLoop`
/// and `joyTick`); the picture is built again from the unchanged state.
pub fn menu_skip() -> bool {
    MENU_SKIP.load(Ordering::Relaxed)
}

/// Time of this front-end frame in ticks (1.0 whenever the front end runs at 60 Hz): scales the steps of the animated
/// background.
pub fn menu_dt() -> f32 {
    f32::from_bits(MENU_DT.load(Ordering::Relaxed))
}

/// A whole-number step of `base` per tick, handed out in the fractions that fit the frame time.
pub fn menu_steps(base: i32) -> i32 {
    let dt = menu_dt();
    if dt >= 0.999 {
        return base;
    }
    let acc = f32::from_bits(MENU_STEP_ACC.load(Ordering::Relaxed)) + base as f32 * dt;
    let n = acc.floor();
    MENU_STEP_ACC.store((acc - n).to_bits(), Ordering::Relaxed);
    n as i32
}

/// `frontfxTick` remembers the page state it was last called with, so that the frames without front-end logic can keep the
/// background moving.
pub fn menu_note_fx(a0: i32) {
    MENU_FX.store(a0, Ordering::Relaxed);
}

// The per-frame arena (`memdbTick` flips between two halves) holds what the logic of a tick built for drawing: the packets of
// the 3D props of the menus (character select, MapMaker previews).  The frames in between draw the same packets again, so
// they must not hand out that memory a second time: a frame that lands in the same half as the last tick continues after the
// logic's allocations instead of starting over.
static MARK_BASE: AtomicU32 = AtomicU32::new(0);
static MARK_PTR: AtomicU32 = AtomicU32::new(0);

/// `memdbTick` for a front-end frame (see above).
pub fn menu_memdb_tick() {
    memdbTick();
    if menu_skip() && lw(0x3afe0c) as u32 == MARK_BASE.load(Ordering::Relaxed) {
        sw(0x3afe14, MARK_PTR.load(Ordering::Relaxed) as i32);
    }
}

/// The logic of a tick is done: remember how much of the arena it used.
pub fn menu_memdb_mark() {
    MARK_BASE.store(lw(0x3afe0c) as u32, Ordering::Relaxed);
    MARK_PTR.store(lw(0x3afe14) as u32, Ordering::Relaxed);
}

pub fn menu_effects() {
    let a = MENU_FX.load(Ordering::Relaxed);
    if a >= 0 {
        frontfxTick(a);
    }
}

/// The game's tick counter as a float, including the part of the current tick that has elapsed.
pub fn gameframe_f() -> f32 {
    lw(g::gameframe) as f32 + f32::from_bits(TICK_ACC.load(Ordering::Relaxed))
}

/// Game time in ticks while a level is running (tests use it as a clock that does not depend on the frame rate).
pub fn level_tick() -> Option<f32> {
    if lw(g::gamestate) == 3 && lw(g::lvfirstframe_130) == 0 && lw(g::gamepaused) == 0 && lw(lw(g::front) + 20) == 0 {
        Some(gameframe_f())
    } else {
        None
    }
}

/// Whole ticks of this frame for code that only asks "is the game running?": at least 1 whenever time passes, 0 while paused.
pub fn running_i() -> i32 {
    if lwc1(g::updateframesf) > 0.0 {
        lw(g::updateframesi).max(1)
    } else {
        0
    }
}

/// The tick counter used for animation (`tickframenum`) as a float, with the elapsed part of the current tick.
pub fn tickframenum_f() -> f32 {
    lw(g::tickframenum) as f32 + f32::from_bits(TICK_ACC.load(Ordering::Relaxed))
}

pub fn timeTickStart() {
    let dt = f32::from_bits(PENDING_DT.swap(0, Ordering::Relaxed));
    MENU_SKIP.store(false, Ordering::Relaxed);
    if dt > 0.0 && menu_hirate() {
        // The front end: its logic is written for exactly one frame per tick.  It runs once per whole tick, as it always did;
        // the frames in between only redraw and advance the animated background by `dt` (see `menu_skip`, `menu_effects`).
        swc1(g::lastcpustarttime, lwc1(g::cpustarttime));
        swc1(g::lastcpuendtime, lwc1(g::cpuendtime));
        swc1(g::cpustarttime, timeGet());
        let mut acc = f32::from_bits(TICK_ACC.load(Ordering::Relaxed)) + dt;
        let whole = acc.floor();
        acc -= whole;
        TICK_ACC.store(acc.to_bits(), Ordering::Relaxed);
        MENU_DT.store(dt.to_bits(), Ordering::Relaxed);
        let whole = whole as i32;
        if whole >= 1 {
            sw(g::tickframesi, whole);
            sw(g::tickframenum, lw(g::tickframenum) + whole);
            swc1(g::tickframesf, whole as f32);
            sw(g::updateframesi, whole);
            swc1(g::updateframesf, whole as f32);
        } else {
            MENU_SKIP.store(true, Ordering::Relaxed);
        }
        return;
    }
    MENU_DT.store(1.0f32.to_bits(), Ordering::Relaxed);
    if dt > 0.0 && hirate_ok() {
        swc1(g::lastcpustarttime, lwc1(g::cpustarttime));
        swc1(g::lastcpuendtime, lwc1(g::cpuendtime));
        swc1(g::cpustarttime, timeGet());
        let mut acc = f32::from_bits(TICK_ACC.load(Ordering::Relaxed)) + dt;
        let whole = acc.floor();
        acc -= whole;
        TICK_ACC.store(acc.to_bits(), Ordering::Relaxed);
        let whole = whole as i32;
        sw(g::tickframesi, whole);
        sw(g::tickframenum, lw(g::tickframenum) + whole);
        swc1(g::tickframesf, dt);
        sw(g::updateframesi, whole);
        swc1(g::updateframesf, dt);
    } else {
        TICK_ACC.store(0, Ordering::Relaxed);
        timeTickStart_60();
    }
}

/// `timeTickStart` as the original had it: the elapsed time in whole ticks, at least 1 and at most 5.
fn timeTickStart_60() {
    let f1 = lwc1(g::cpuendtime);
    swc1(g::lastcpustarttime, lwc1(g::cpustarttime));
    swc1(g::lastcpuendtime, f1);
    let f0 = timeGet();
    let elapsed = f0 - lwc1(g::lastcpustarttime);
    swc1(g::cpustarttime, f0);
    let mut ticks = (elapsed + 0.5).floor() as i32;
    while ticks < 0 {
        ticks += 128;
    }
    if ticks >= 6 {
        ticks = 5;
    }
    if ticks == 0 {
        ticks = 1;
    }
    let paused = lw(g::gamepaused);
    let tickframenum = lw(g::tickframenum);
    let update;
    if paused == -1 {
        ticks = 1;
        update = 1;
        sw(g::gamepaused, 0);
    } else if paused == 0 {
        update = ticks;
    } else {
        update = 0;
    }
    sw(g::tickframesi, ticks);
    sw(g::updateframesi, update);
    sw(g::tickframenum, tickframenum + ticks);
    swc1(g::updateframesf, update as f32);
    swc1(g::tickframesf, ticks as f32);
    if ilinkIsILinkGame() != 0 {
        ilinkSetFramesi();
        sw(g::updateframesi, 1);
        swc1(g::updateframesf, 1.0);
    }
}

/// Replacement for the game's ISO9660 directory walk: look the file up on the virtual disc.
/// `file` receives { lsn, size }; returns 1 when found.
pub fn SCEECdSearchFile(file: i32, name: i32) -> i32 {
    let n = cstr(name);
    match ts_sdk::disc_find(&n) {
        Some((lsn, size)) => {
            sw(file, lsn as i32);
            sw(file + 4, size as i32);
            1
        }
        None => {
            eprintln!("[disc] file not found: {n}");
            std::process::exit(2);
        }
    }
}

/// EE timer 1 replacement: elapsed time in 60 Hz frames since the last reset (resets every 128 vblanks).
pub fn timeGet() -> f32 {
    let t = ts_sdk::now_frames();
    t - f32::from_bits(TIMEBASE.load(Ordering::Relaxed))
}

pub fn timeTickRetrace() {
    let v = lw(g::framenum) + 1;
    sw(g::framenum, v);
    if v & 127 == 0 {
        TIMEBASE.store(ts_sdk::now_frames().to_bits(), Ordering::Relaxed);
    }
}

/// Test aid: `TS_FORCEPLAYERS=<n>` starts the match with n players, whatever the front end set up (called by `gameReset`).
pub fn test_players(n: i32) -> i32 {
    // (not for the front end, which is set up the same way: its "levels" are 100 and up)
    if lw(g::curgameparms + 64) >= 100 {
        return n;
    }
    std::env::var("TS_FORCEPLAYERS").ok().and_then(|v| v.parse().ok()).unwrap_or(n)
}
