//! Differential tester for the decompiler.
//!
//! For each decompiled function: run the ORIGINAL machine code (tiny interpreter, dev-only) and the DECOMPILED Rust on
//! identical randomised inputs and memory, then compare return values and every memory write.
//!
//! usage: ts-verify [--n N] [--elf PATH] [--list] [name ...]


use ts_verify::cpu::*;
use ts_verify::interp::*;
use std::collections::HashMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use ts_game::{call_any, FUNC_INFO};
use ts_rt::loader::{load_elf_data, read_symbols};
use ts_rt::*;


struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
    fn float(&mut self) -> f32 {
        match self.below(8) {
            0 => 0.0,
            1 => 1.0,
            2 => -1.0,
            3 => (self.below(2000) as f32 - 1000.0) / 4.0,
            // large values too (times, angles in the wind and the sines of the weather change little over small ones), except when the small numbers are asked for
            4 if std::env::var_os("TS_VERIFY_SMALL").is_none() => self.below(2_000_000) as f32 - 1_000_000.0,
            _ => (self.below(200_000) as f32 - 100_000.0) / 10_000.0,
        }
    }
}


fn run_native(addr: u32, ints: &[i64; 8], floats: &[f32; 8]) -> Outcome {
    trace::start();
    let res = silent(|| call_any(addr, *ints, *floats));
    let log = trace::stop();
    let writes = changed(&log);
    trace::rollback(&log);
    let log: Vec<trace::Store> = log.into_iter().filter(|s| !s.bulk && s.addr < STACK_LIMIT).collect();
    match res {
        Err(p) if p.contains("not yet implemented") => Outcome::Skip(format!("stub: {p}")),
        Err(p) if p.starts_with("trap:") => Outcome::Trap,
        Err(p) => Outcome::Panic(p),
        Ok((ret, fret)) => Outcome::Ok { ret, fret, writes, stores: log },
    }
}

/// Fill the bullet table with random bullets (see `TS_VERIFY_PERTURB=bullets`): any kind, near the first player, with plausible owners, ages and speeds.
fn perturb_bullets(rng: &mut Rng) {
    use ts_game::g;
    let p = lw(g::players);
    let table = lw(0x3b0138);
    let spread = |rng: &mut Rng, low: f32, high: f32| low + (high - low) * (rng.below(10001) as f32 / 10000.0);
    let lone = rng.below(50) as i32;
    for i in 0..50 {
        let b = table + i * 276;
        if std::env::var_os("TS_VERIFY_LONE").is_some() && i != lone || rng.below(3) == 0 {
            sw(b, 0);
            continue;
        }
        for word in 0..69 {
            sw(b + word * 4, 0);
        }
        let (kind_low, kind_high) = std::env::var("TS_VERIFY_KINDS").ok().and_then(|v| { let (a, b) = v.split_once('-')?; Some((a.parse::<u64>().ok()?, b.parse::<u64>().ok()?)) }).unwrap_or((2, 19));
        sw(b, (kind_low + rng.below(kind_high - kind_low + 1)) as i32);
        let show = std::env::var_os("TS_VERIFY_LONE").is_some() && i == lone && std::env::var_os("TS_VERIFY_SHOWBULLET").is_some();
        for axis in 0..3 {
            swc1(b + 24 + axis * 4, lwc1(p + 48 + axis * 4) + spread(rng, -6.0, 6.0));
            swc1(b + 48 + axis * 4, spread(rng, -8.0, 8.0));
            swc1(b + 128 + axis * 4, spread(rng, -1.0, 1.0));
        }
        // Half of the bullets are shot at a character (or the player), from near it, so that the hits on characters and props are tested.
        if rng.below(2) == 0 {
            let count = lw(g::numactivechars).clamp(0, 32);
            let target = if count > 0 && rng.below(3) != 0 {
                lw(lw(g::chrinfo) + rng.below(count as u64) as i32 * 0x1210 + 3020)
            } else {
                lw(p + 384)
            };
            if target != 0 && lw(target + 12) >= 0 && lw(target + 12) < 4096 {
                sw(b + 100, lw(target + 12));
                for axis in 0..3 {
                    let at = lwc1(target + 48 + axis * 4) + spread(rng, -1.0, 2.0);
                    let from = at + spread(rng, -3.0, 3.0);
                    swc1(b + 24 + axis * 4, from);
                    swc1(b + 48 + axis * 4, (at - from) * spread(rng, 0.2, 1.2));
                }
                // a mine that is stuck to the character (it does not move)
                if rng.below(3) == 0 {
                    sw(b + 224, target);
                    sw(b + 228, rng.below(20) as i32 - 1);
                    for axis in 0..3 {
                        swc1(b + 232 + axis * 4, spread(rng, -1.0, 1.0));
                        swc1(b + 244 + axis * 4, spread(rng, -1.0, 1.0));
                        if rng.below(4) != 0 {
                            sw(b + 48 + axis * 4, 0);
                        }
                    }
                }
                // the prop that an earlier line test of the bullet found (the bullet is accepted as hitting it when it is heading into it)
                if rng.below(2) == 0 {
                    sw(b + 184, target);
                    sw(b + 188, lw(target + 32));
                    sw(b + 220, lw(p + 384));
                    for axis in 0..3 {
                        swc1(b + 160 + axis * 4, lwc1(target + 48 + axis * 4) + spread(rng, -0.5, 0.5));
                        swc1(b + 172 + axis * 4, spread(rng, -1.0, 1.0));
                    }
                    sw(b + 192, rng.below(8) as i32);
                    sw(b + 196, rng.below(4) as i32);
                    sw(b + 200, rng.below(4) as i32);
                }
            }
        }
        swc1(b + 72, spread(rng, 0.0, 4.0));
        if rng.below(2) == 0 {
            sw(b + 96, lw(p + 384));
        }
        sw(b + 104, [0, 1, 2, 4, 3][rng.below(5) as usize]);
        sw(b + 108, if rng.below(8) == 0 { -1 } else { 0 });
        for off in if std::env::var_os("TS_VERIFY_NOFIELDS").is_some() { vec![] } else { vec![76, 80, 84, 88, 92, 100, 112, 116, 120, 124, 140, 144, 148, 152, 156, 160, 164, 168, 172, 176, 180, 184, 188, 192, 196, 200, 204, 208, 212, 216, 220, 224, 228] } {
            match rng.below(4) {
                0 => sw(b + off, 0),
                1 => sw(b + off, rng.below(6) as i32),
                2 => swc1(b + off, spread(rng, -3.0, 3.0)),
                _ => {}
            }
        }
        if show {
            let words: Vec<String> = (0..69).map(|w| format!("{:x}", lw(b + w * 4))).collect();
            println!("bullet {i}: {}", words.join(" "));
        }
    }
}

/// With snapshots, the functions that walk a model get a real one (the model of a character of the level) as their first argument, and the bone 0 where they take one: a random pointer makes them recurse for ever.
fn use_real_model(rng: &mut Rng, name: &str, ints: &mut [i64; 8]) {
    use ts_game::g;
    let takes_bone = matches!(name, "partColour" | "partTransform" | "partSetTextureOffset");
    if !takes_bone && !matches!(name, "obColour" | "obTransform" | "obSetTexList") {
        return;
    }
    let count = lw(g::numactivechars).clamp(0, 32);
    let prop = if count > 0 && rng.below(4) != 0 {
        lw(lw(g::chrinfo) + rng.below(count as u64) as i32 * 0x1210 + 3020)
    } else {
        lw(lw(g::players) + 384)
    };
    let model = lw(lw(prop + 32));
    if model != 0 {
        ints[0] = model as i64;
        if takes_bone {
            ints[1] = 0;
        }
    }
}

/// Apply the perturbations named in `TS_VERIFY_PERTURB` (separated by commas: `1` for the first player, `bullets`, `world`).
fn apply_perturb(rng: &mut Rng) {
    if let Ok(modes) = std::env::var("TS_VERIFY_PERTURB") {
        for mode in modes.split(',') {
            match mode {
                "bullets" => perturb_bullets(rng),
                "world" => perturb_world(rng),
                _ => perturb_player(rng),
            }
        }
    }
}

/// Change words of the players' and characters' records at random (`TS_VERIFY_PERTURB=world`): modes, flags, counters and floats, but not what looks like a pointer, so that the branches that real play seldom reaches are run without wild pointers.
fn perturb_world(rng: &mut Rng) {
    use ts_game::g;
    let density = std::env::var("TS_VERIFY_DENSITY").ok().and_then(|v| v.parse::<u64>().ok()).unwrap_or(40);
    let players = lw(g::lvnumplayers).clamp(0, 4);
    let chars = lw(g::numactivechars).clamp(0, 64);
    let ranges = [(lw(g::players), players * 1820), (lw(g::chrinfo), (players + chars) * 0x1210)];
    for (start, len) in ranges {
        let mut at = start;
        while at < start + len {
            if rng.below(density) == 0 {
                let v = lw(at);
                let f = f32::from_bits(v as u32);
                let looks_like_pointer = (0x10_0000..0x200_0000).contains(&v);
                if looks_like_pointer {
                    // left alone
                } else if v == 0 {
                    match rng.below(3) {
                        0 => sw(at, rng.below(8) as i32),
                        1 => swc1(at, (rng.below(2001) as f32 - 1000.0) / 100.0),
                        _ => {}
                    }
                } else if (0..4096).contains(&v) {
                    match rng.below(3) {
                        0 => sw(at, v ^ (1 << rng.below(12))),
                        1 => sw(at, rng.below(64) as i32),
                        _ => sw(at, 0),
                    }
                } else if f.is_finite() && f.abs() > 1e-3 && f.abs() < 1e6 {
                    match rng.below(3) {
                        0 => swc1(at, f * (rng.below(300) as f32 / 100.0)),
                        1 => swc1(at, -f),
                        _ => swc1(at, f + (rng.below(2001) as f32 - 1000.0) / 100.0),
                    }
                } else {
                    sw(at, v ^ (1 << rng.below(32)));
                }
            }
            at += 4;
        }
    }
}

/// Change the state of the first player and of its pad at random (see `TS_VERIFY_PERTURB`).
fn perturb_player(rng: &mut Rng) {
    use ts_game::g;
    let p = lw(g::players);
    let pad = lw(p);
    let ctrl = g::controllers + (pad & 7) * 384;
    let mut buttons = 0;
    for bit in 0..16 {
        if rng.below(4) == 0 {
            buttons |= 1 << bit;
        }
    }
    let before = {
        let mut m = 0;
        for bit in 0..16 {
            if rng.below(3) == 0 {
                m |= 1 << bit;
            }
        }
        m
    };
    sw(ctrl + 284, before);
    sw(ctrl + 288, buttons);
    sw(ctrl + 292, buttons & !before);
    sw(ctrl + 296, before & !buttons);
    for axis in 0..4 {
        let value = match rng.below(4) {
            0 | 1 => 128,
            2 => [0, 255, 64, 192][rng.below(4) as usize],
            _ => rng.below(256) as i32,
        };
        sw(ctrl + 300 + axis * 4, value);
    }
    if rng.below(5) < 3 {
        sw(p + 296, rng.below(8) as i32);
    }
    let mut spread = |rng: &mut Rng, low: f32, high: f32| low + (high - low) * (rng.below(10001) as f32 / 10000.0);
    if rng.below(2) == 0 {
        swc1(p + 128, spread(rng, -0.5, 0.5));
    }
    if rng.below(2) == 0 {
        swc1(p + 124, spread(rng, -1.0, 0.0));
    }
    for offset in [300, 304, 308] {
        if rng.below(2) == 0 {
            swc1(p + offset, spread(rng, -3.0, 3.0));
        }
    }
    for offset in [856, 864] {
        if rng.below(2) == 0 {
            swc1(p + offset, spread(rng, -0.6, 0.6));
        }
    }
    let character = lw(lw(p + 384) + 352);
    if character != 0 {
        if rng.below(2) == 0 {
            swc1(character + 2864, spread(rng, -1.0, 1.0));
        }
        if rng.below(2) == 0 {
            swc1(character + 2956, lwc1(p + 144) + spread(rng, -2.0, 14.0));
        }
    }
    // the characters the player sees, with the weights of the aim
    for offset in [260, 264, 268, 272] {
        if rng.below(2) == 0 {
            swc1(p + offset, spread(rng, 20.0, 60.0));
        }
    }
    for (offset, low, high) in [(276, 0.0, 360.0), (284, -60.0, 60.0), (312, 0.0, 4.0), (320, 0.0, 1.5), (324, 0.0, 360.0), (328, 0.0, 360.0)] {
        if rng.below(2) == 0 {
            swc1(p + offset, spread(rng, low, high));
        }
    }
    if rng.below(3) == 0 {
        sw(p + 792, rng.below(70) as i32);
    }
    let records = lw(p + 820);
    if records != 0 {
        for i in 0..(lw(g::lvnumplayers) + lw(g::numactivechars)).clamp(0, 64) {
            if rng.below(2) == 0 {
                sw(records + i * 12, 1 + rng.below(2) as i32);
                swc1(records + i * 12 + 4, spread(rng, 0.0, 3.0));
            }
        }
    }
    let ground = lw(p + 824);
    if ground != 0 && rng.below(3) == 0 {
        sh(ground + 6, lhu(ground + 6) ^ (1 << rng.below(3)) as i32);
    }
    // the polygon of the ground gets slopes
    if ground != 0 && rng.below(2) == 0 {
        for k in 0..9 {
            swc1(ground + 8 + 4 * k, lwc1(ground + 8 + 4 * k) + spread(rng, -2.0, 2.0));
        }
    }
    let body = lw(p + 384);
    if rng.below(2) == 0 {
        sw(p + 388, body);
    }
    if rng.below(3) == 0 {
        sw(p + 904, body);
    }
    if rng.below(3) == 0 {
        swc1(body + 520, spread(rng, -2.0, 2.0));
    }
    if rng.below(3) == 0 {
        swc1(p + 1776, spread(rng, 0.0, 1.5));
        swc1(p + 1780, spread(rng, 0.0, 1.0));
    }
    if character != 0 {
        if rng.below(3) == 0 {
            sw(character + 0x11b0, rng.below(2) as i32);
        }
        if rng.below(3) == 0 {
            sw(character + 0x11b4, if rng.below(2) == 0 { body } else { 0 });
        }
    }
    if rng.below(3) == 0 {
        sw(g::curgameparms + 72, [0, 6, 7][rng.below(3) as usize]);
    }
    if rng.below(3) == 0 {
        sw(g::curgameparms + 80, lw(g::curgameparms + 80) ^ 32);
        sw(g::curgameparms + 68, [17, 0][rng.below(2) as usize]);
    }
    if rng.below(3) == 0 {
        sw(p + 20, (lw(p + 20) & !3) | rng.below(4) as i32);
    }
    if rng.below(3) == 0 {
        sw(p + 288, rng.below(3) as i32);
    }
    if rng.below(4) == 0 {
        sw(p + 612, rng.below(2) as i32);
    }
    if rng.below(4) == 0 {
        sw(p + 420, rng.below(2) as i32);
    }
    let gun = lw(p + 24);
    if gun != 0 {
        let mut flags = lhu(gun + 2);
        for bit in [0, 1, 2, 3, 4, 5, 7] {
            if rng.below(3) == 0 {
                flags ^= 1 << bit;
            }
        }
        sh(gun + 2, flags);
    }
    let ticks = 1 + rng.below(3) as i32;
    sw(g::updateframesi, ticks);
    swc1(g::updateframesf, ticks as f32);
}

fn gen_inputs(rng: &mut Rng, ni: u8, nf: u8) -> ([i64; 8], [f32; 8]) {
    let mut ints = [0i64; 8];
    let mut floats = [0f32; 8];
    for i in 0..ni as usize {
        ints[i] = match rng.below(6) {
            0 => 0,
            1 => rng.below(16) as i64,
            2 => rng.below(8) as i64 - 4,
            3 | 4 => (SCRATCH + 16 * rng.below(2048) as u32) as i64,
            // TS_VERIFY_SMALL=1: no huge counts and offsets, so that loops over the data finish
            _ if std::env::var_os("TS_VERIFY_SMALL").is_some() => rng.below(32) as i64,
            _ => (rng.next() as i32 as i64) >> 12,
        };
    }
    for i in 0..nf as usize {
        floats[i] = rng.float();
    }
    (ints, floats)
}

fn random_word(rng: &mut Rng) -> u32 {
    match rng.below(7) {
        0 => rng.float().to_bits(),
        1 => rng.below(16) as u32,
        2 => rng.float().to_bits(),
        // a pointer into the random area, so that chains of pointers lead to data and not to zeros
        3 | 4 => SCRATCH + 4 * rng.below(SCRATCH_LEN as u64 / 4) as u32,
        // TS_VERIFY_SMALL=1: no huge counts, so that loops over the data finish
        _ if std::env::var_os("TS_VERIFY_SMALL").is_some() => rng.below(64) as u32,
        _ => (rng.next() as u32) & 0x00ff_ffff,
    }
}

fn fill_scratch(rng: &mut Rng) {
    for a in (0..SCRATCH_LEN).step_by(4) {
        sw((SCRATCH + a) as i32, random_word(rng) as i32);
    }
    // the game's variables (the start of the zero-filled data and the structures high in it) are random too: otherwise a function that works from them
    // finds counts of 0 everywhere and its work is never tested. `TS_VERIFY_ZERO_GLOBALS=1` leaves them as they are.
    if std::env::var_os("TS_VERIFY_ZERO_GLOBALS").is_none() {
        for &(from, to) in GLOBAL_RANGES {
            for a in (from..to).step_by(4) {
                sw(a as i32, random_word(rng) as i32);
            }
        }
    }
}

/// The ranges of the zero-filled data where the game keeps its variables and tables.
const GLOBAL_RANGES: &[(u32, u32)] = &[(0x3afe00, 0x420000), (0x1fb0000, 0x1ff5390)];


fn same_f(a: f32, b: f32) -> bool {
    a.to_bits() == b.to_bits() || (a.is_nan() && b.is_nan()) || (a - b).abs() <= 1e-5 * a.abs().max(b.abs())
}

/// The bytes that differ are all in 4-byte words that both runs last stored as a NaN (x86 and the console's NaN differ in sign and payload; the store logs tell what word was stored).
fn changes_equal_mod_nan(w1: &[(u32, u8)], w2: &[(u32, u8)], s1: &[trace::Store], s2: &[trace::Store]) -> bool {
    let m1: HashMap<_, _> = w1.iter().cloned().collect();
    let m2: HashMap<_, _> = w2.iter().cloned().collect();
    let lane = |ss: &[trace::Store], a: u32| -> Option<u32> {
        let s = ss.iter().rev().find(|x| a >= x.addr && a < x.addr + x.size as u32)?;
        let base = a & !3;
        if base < s.addr || base + 4 > s.addr + s.size as u32 {
            return None;
        }
        let o = (base - s.addr) as usize;
        Some(u32::from_le_bytes([s.new[o], s.new[o + 1], s.new[o + 2], s.new[o + 3]]))
    };
    let is_nan = |w: u32| (w >> 23) & 0xff == 0xff && w & 0x7f_ffff != 0;
    let mut keys: Vec<u32> = m1.keys().chain(m2.keys()).cloned().collect();
    keys.sort_unstable();
    keys.dedup();
    keys.iter().filter(|k| m1.get(k) != m2.get(k)).all(|&k| match (lane(s1, k), lane(s2, k)) {
        (Some(a), Some(b)) => is_nan(a) && is_nan(b),
        _ => false,
    })
}

/// The calls of one frame of the game in order (see `bossMainLoop`); the players' ticks each with its player current.
fn frame_calls(syms: &[ts_rt::loader::Symbol]) -> Vec<(u32, Option<i32>)> {
    let addr = |n: &str| syms.iter().find(|s| s.name == n).unwrap().addr;
    let mut calls = vec![(addr("gameTick"), None), (addr("lvTickBefore"), None)];
    let players = lw(ts_game::g::lvnumplayers).clamp(0, 4);
    for index in 0..players {
        calls.push((addr("lvTickPlayer"), Some(index)));
    }
    calls.push((addr("lvTickAfter"), None));
    calls
}

fn run_frames(ctx: &Ctx, syms: &[ts_rt::loader::Symbol], snapshots: &[Vec<u8>], n: usize) {
    use ts_game::g;
    let frames: usize = std::env::var("TS_VERIFY_FRAMES").ok().and_then(|v| v.parse().ok()).unwrap_or(3);
    let from: usize = std::env::var("TS_VERIFY_FROM").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    let (mut ok, mut bad, mut skipped) = (0, 0, 0);
    for t in from..n {
        let mut rng = Rng(0x51_7C_C1B7_2722_0A95 ^ (t as u64 + 1) * 0x1234567);
        let image = &snapshots[t % snapshots.len()];
        host_slice(0, image.len()).copy_from_slice(image);
        sw(g::currentplayer, lw(g::players));
        apply_perturb(&mut rng);
        let calls = frame_calls(syms);
        let set_player = |index: Option<i32>| {
            if let Some(i) = index {
                sw(g::currentplayer, lw(g::players) + i * 1820);
            }
        };
        let zeros = ([0i64; 8], [0f32; 8]);
        // the original
        trace::start();
        let mut note: Option<String> = None;
        'interp: for _ in 0..frames {
            for &(addr, player) in &calls {
                set_player(player);
                let ran = exec_interp(ctx, addr, &zeros.0, &zeros.1);
                if let Some(u) = ran.unsupported {
                    note = Some(u);
                    break 'interp;
                }
                if ran.exited || !matches!(ran.res, Ok(Ok(()))) {
                    note = Some(format!("interpreter stopped: {:?}", ran.res.as_ref().err()));
                    break 'interp;
                }
            }
        }
        let log1 = trace::stop();
        let w1 = changed(&log1);
        trace::rollback(&log1);
        let s1: Vec<trace::Store> = log1.into_iter().filter(|s| !s.bulk && s.addr < STACK_LIMIT).collect();
        if let Some(why) = note {
            skipped += 1;
            if verbose_frames() {
                println!("run {t}: skipped ({why})");
            }
            continue;
        }
        // the port
        sw(g::currentplayer, lw(g::players));
        trace::start();
        let mut fault: Option<String> = None;
        'native: for _ in 0..frames {
            for &(addr, player) in &calls {
                set_player(player);
                if let Err(p) = silent(|| call_any(addr, zeros.0, zeros.1)) {
                    let at = syms.iter().find(|s| s.addr == addr).map_or("?", |s| s.name.as_str());
                    fault = Some(format!("{p}, in {at}"));
                    break 'native;
                }
            }
        }
        let log2 = trace::stop();
        let w2 = changed(&log2);
        trace::rollback(&log2);
        let s2: Vec<trace::Store> = log2.into_iter().filter(|s| !s.bulk && s.addr < STACK_LIMIT).collect();
        // words the original takes from stack garbage (see `TS_VERIFY_IGNORE`): the last hit that `bulletInitialiseStraightLineTest` puts back, and the wall normal of `moveStep`
        let ignored = |a: u32| a < 0x10000 || (0x3aea90..0x3aeab8).contains(&a) || (0x3b0118..0x3b0124).contains(&a);
        let w1: Vec<(u32, u8)> = w1.into_iter().filter(|(a, _)| !ignored(*a)).collect();
        let w2: Vec<(u32, u8)> = w2.into_iter().filter(|(a, _)| !ignored(*a)).collect();
        if let Some(p) = fault {
            // the interpreter wraps addresses past the end of the port's RAM and the port faults on them: a garbage pointer made by the perturbation, not a difference
            let wild = p.contains("unmapped game address") && p.split("0x").nth(1).and_then(|h| u32::from_str_radix(h.split(|c: char| !c.is_ascii_hexdigit()).next().unwrap_or(""), 16).ok()).map_or(false, |a| a >= 0x0800_0000);
            if wild {
                skipped += 1;
                continue;
            }
            bad += 1;
            println!("FAIL run {t}: the port faulted ({p}) where the original ran to the end");
            continue;
        }
        if w1 != w2 && !stores_equal_mod_nan(&s1, &s2) && !changes_equal_mod_nan(&w1, &w2, &s1, &s2) {
            bad += 1;
            let m1: HashMap<_, _> = w1.iter().cloned().collect();
            let m2: HashMap<_, _> = w2.iter().cloned().collect();
            let mut keys: Vec<_> = m1.keys().chain(m2.keys()).cloned().filter(|k| m1.get(k) != m2.get(k)).collect();
            keys.sort_unstable();
            keys.dedup();
            let shown: Vec<String> = keys.iter().take(6).map(|k| format!("[{k:#x}: original {:?} port {:?}]", m1.get(k), m2.get(k))).collect();
            println!("FAIL run {t}: {} bytes differ {}", keys.len(), shown.join(" "));
            if let Some(&k) = keys.first() {
                for (label, ss) in [("original", &s1), ("port", &s2)] {
                    for x in ss.iter().filter(|x| k >= x.addr && k < x.addr + x.size as u32) {
                        println!("  {label} store {:#x} +{} = {:02x?} pc {:#x}", x.addr, x.size, &x.new[..x.size as usize], x.tag);
                    }
                }
            }
        } else {
            ok += 1;
        }
    }
    if std::env::var("TS_VERIFY_COVERAGE").map_or(false, |v| v == "all") {
        print_all_coverage(syms);
    }
    println!("
== frames: {ok} ok, {bad} differ, {skipped} skipped ==");
}

/// Every function the runs went through: how much of it ran (the functions below the driver are checked as part of it).
fn print_all_coverage(syms: &[ts_rt::loader::Symbol]) {
    let covered = COVERAGE.lock().unwrap().take().unwrap_or_default();
    let mut rows: Vec<(String, u32, u32)> = Vec::new();
    for sym in syms.iter().filter(|x| x.is_func && x.size >= 4) {
        let total = sym.size / 4;
        let hit = (0..total).filter(|k| covered.contains(&(sym.addr + k * 4))).count() as u32;
        if hit > 0 {
            rows.push((sym.name.clone(), hit, total));
        }
    }
    rows.sort_by(|a, b| (b.2 - b.1).cmp(&(a.2 - a.1)));
    for (n, h, t) in &rows {
        println!("cover {n}: {h} of {t}");
    }
}

/// Run the function of a capture in the original code and in the port, on the captured memory, and compare the memory they change (above the stack in use).
fn run_capture(ctx: &Ctx, syms: &[ts_rt::loader::Symbol], path: &str) {
    let Some((name, sp, ints_list, floats_list)) = ts_rt::capture::load(path) else {
        println!("cannot read the capture {path}");
        return;
    };
    let Some(&(addr, _, _, _, rk)) = FUNC_INFO.iter().find(|f| f.1 == name) else {
        println!("no function {name}");
        return;
    };
    let mut ints = [0i64; 8];
    let mut floats = [0f32; 8];
    for (i, v) in ints_list.iter().enumerate().take(8) {
        ints[i] = *v;
    }
    for (i, v) in floats_list.iter().enumerate().take(8) {
        floats[i] = *v;
    }
    // the stack of the original call is in use above the captured stack pointer: run below it
    let sp = sp - 0x1000;
    INTERP_SP.store(sp, std::sync::atomic::Ordering::Relaxed);
    ts_rt::frame::set_sp(sp);
    let a = run_interp(ctx, addr, &ints, &floats);
    if std::env::var_os("TS_VERIFY_INTERP_ONLY").is_some() {
        println!("original: {:?}", match &a { Outcome::Ok { ret, writes, .. } => format!("finished, returned {ret:#x}, {} bytes changed", writes.len()), other => format!("{other:?}") });
        return;
    }
    let b = run_native(addr, &ints, &floats);
    match (&a, &b) {
        (Outcome::Ok { ret: r1, writes: w1, stores: s1, .. }, Outcome::Ok { ret: r2, writes: w2, stores: s2, .. }) => {
            let same_ret = rk != 1 || (*r1 as i32) == (*r2 as i32);
            if !same_ret {
                println!("FAIL {name}: returned {r1:#x} in the original, {r2:#x} in the port");
            } else if w1 != w2 && !stores_equal_mod_nan(s1, s2) && !changes_equal_mod_nan(w1, w2, s1, s2) {
                let m1: HashMap<_, _> = w1.iter().cloned().collect();
                let m2: HashMap<_, _> = w2.iter().cloned().collect();
                let mut keys: Vec<_> = m1.keys().chain(m2.keys()).cloned().filter(|k| m1.get(k) != m2.get(k)).collect();
                keys.sort_unstable();
                keys.dedup();
                let shown: Vec<String> = keys.iter().take(8).map(|k| format!("[{k:#x}: original {:?} port {:?}]", m1.get(k), m2.get(k))).collect();
                println!("FAIL {name}: {} bytes differ {}", keys.len(), shown.join(" "));
            } else {
                println!("ok   {name}: {} bytes changed, returned {r1:#x}", w1.len());
            }
        }
        _ => println!("{name}: original {a:?} / port {b:?}"),
    }
    if std::env::var("TS_VERIFY_COVERAGE").is_ok() {
        if let Some(sym) = syms.iter().find(|x| x.addr == addr && x.is_func) {
            let covered = COVERAGE.lock().unwrap().take().unwrap_or_default();
            let total = sym.size / 4;
            let hit = (0..total).filter(|k| covered.contains(&(addr + k * 4))).count();
            println!("coverage {name}: {hit} of {total} instructions");
        }
    }
}

fn verbose_frames() -> bool {
    std::env::var_os("TS_VERIFY_FAULTS").is_some()
}

fn find_root() -> PathBuf {
    let mut d = std::env::current_dir().unwrap();
    loop {
        if d.join("Game Files").is_dir() {
            return d;
        }
        if !d.pop() {
            panic!("could not find the 'Game Files' directory above the working directory");
        }
    }
}

fn main() {
    if std::env::var_os("TS_VERIFY_BACKTRACE").is_some() {
        // print where the port faults (the last fault of a run is the one that matters)
        std::panic::set_hook(Box::new(|info| {
            let bt = std::backtrace::Backtrace::force_capture().to_string();
            let frames: Vec<&str> = bt.lines().filter(|l| l.contains("ts_game::game")).take(8).collect();
            println!("panic: {info}
{}", frames.join("
"));
        }));
    } else {
        std::panic::set_hook(Box::new(|_| {}));
    }
    ts_rt::VERIFYING.store(true, std::sync::atomic::Ordering::Relaxed);
    let mut n = 12usize;
    let mut elf: Option<PathBuf> = None;
    let mut names = Vec::new();
    let mut verbose = false;
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--n" => n = it.next().unwrap().parse().unwrap(),
            "--elf" => elf = Some(it.next().unwrap().into()),
            "-v" => verbose = true,
            _ => names.push(a),
        }
    }
    let elf = elf.unwrap_or_else(|| find_root().join("Game Files").join("SLUS_200.90"));
    init_memory();
    load_elf_data(&elf).expect("cannot load executable");
    #[cfg(feature = "shadow-hook")]
    ts_verify::shadow::install(&elf);
    let syms = read_symbols(&elf).unwrap();
    let gp = syms.iter().find(|s| s.name == "_gp").unwrap().addr as u64;
    let split = syms.iter().find(|s| s.name == "text_data_split").unwrap().addr;
    let externs: HashMap<u32, String> =
        syms.iter().filter(|s| s.is_func && s.addr >= split).map(|s| (s.addr, s.name.clone())).collect();
    let ctx = Ctx { externs, gp };
    // the table of sines is made by the game's start-up code (it is not in the executable): without it every sine and cosine reads 0 and the drawing functions' results hide their mistakes
    if let Some(table) = syms.iter().find(|s| s.name == "g_SinTable") {
        for i in 0..2048u32 {
            let value = (i as f64 * std::f64::consts::TAU / 2048.0).sin() as f32;
            sw((table.addr + i * 4) as i32, value.to_bits() as i32);
        }
    }
    // the memory areas of the allocator: room for the usual small allocations (a function that calls `memAlloc` would otherwise end every run in `exit`)
    let seeds: Vec<(u32, i32)> = [
        ("mem_level_free", 0x2_8000),
        ("mem_level_at", (SCRATCH + 0x8000) as i32),
        ("mem_level_end", (SCRATCH + 0x3_fff0) as i32),
        ("mem_local_at", (SCRATCH + 0x3_fff0) as i32),
        ("mem_eternal_free", 0x4000),
        ("mem_eternal_at", (SCRATCH + 0x1_8000) as i32),
        // the display list being built and the debug memory the drawing functions take their packets from
        ("gfxtag", (SCRATCH + 0x3_8000) as i32),
        // the map maker's view of the map and the window
        ("w_w", 640),
        ("w_h", 224),
        ("m_w", 0x200_0000),
        ("m_h", 0x100_0000),
        ("scale_mtowx", 0x1_0000),
        ("scale_mtowy", 0x1_0000),
    ]
    .iter()
    .filter_map(|(n, v)| syms.iter().find(|s| s.name == *n).map(|s| (s.addr, *v)))
    .chain(std::iter::once((0x3afe14u32, (SCRATCH + 0x3_0000) as i32)))
    .collect();

    // `TS_VERIFY_SNAPSHOT=a.bin,b.bin`: instead of random memory, test on snapshots of the game's RAM (written by `TS_SNAP` of the port while a level is played),
    // one for each run in turn; the functions that work from the whole game's state (a player's tick, a bullet's tick) get real data this way
    let snapshots: Vec<Vec<u8>> = std::env::var("TS_VERIFY_SNAPSHOT")
        .map(|v| v.split(',').filter_map(|f| std::fs::read(f.trim()).ok()).collect())
        .unwrap_or_default();

    // `frames`: run whole frames of the game's tick (as `bossMainLoop` does) one after the other on each snapshot, in the interpreter and in the port, and compare all the memory they changed
    // (`TS_VERIFY_FRAMES=k` frames a run, 3 by default); a difference that only shows after the state has moved on (a bullet that flies for a few frames) is found this way
    if names.len() == 1 && names[0] == "frames" {
        run_frames(&ctx, &syms, &snapshots, n);
        return;
    }
    // `TS_VERIFY_CAPTURE=file.cap` (written by the port, see `ts_rt::capture`): run the function of the capture on the memory and the arguments of that call
    if let Ok(path) = std::env::var("TS_VERIFY_CAPTURE") {
        run_capture(&ctx, &syms, &path);
        return;
    }
    let (mut pass, mut fail, mut skip, mut vacuous) = (0, 0, 0, 0);
    let mut failures = Vec::new();
    for &(addr, name, ni, nf, _rk) in FUNC_INFO {
        if !names.is_empty() && !names.iter().any(|x| x == name) {
            continue;
        }
        // hand-written platform replacements (no original behaviour to compare against)
        if matches!(name, "SCEECdSearchFile") {
            skip += 1;
            continue;
        }
        let mut real = 0;
        let mut bad: Option<String> = None;
        let mut skipped: Option<String> = None;
        let from: usize = std::env::var("TS_VERIFY_FROM").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
        for t in from..n {
            let mut rng = Rng(0x9E37_79B9_7F4A_7C15 ^ (addr as u64) << 20 ^ (t as u64 + 1) * 0x1234567);
            if snapshots.is_empty() {
                fill_scratch(&mut rng);
                for &(a, v) in &seeds {
                    sw(a as i32, v);
                }
            } else {
                let image = &snapshots[t % snapshots.len()];
                host_slice(0, image.len()).copy_from_slice(image);
                sw(ts_game::g::currentplayer, lw(ts_game::g::players));
                // `TS_VERIFY_PERTURB=1`: change the player's modes, flags, buttons and sticks too (from the same random numbers each time), so that the branches of the player's tick that the snapshots do not reach are tested
                apply_perturb(&mut rng);
            }
            let (mut ints, floats) = gen_inputs(&mut rng, ni, nf);
            // `TS_VERIFY_ARGS=name=hex,hex,...` (with `TS_VERIFY_SNAPSHOT` of a memory image captured in the port during a call, and `TS_VERIFY_SP` below the stack that was in use): run the function on the arguments it was called with
            if let Ok(spec) = std::env::var("TS_VERIFY_ARGS") {
                if let Some((n, list)) = spec.split_once('=') {
                    if n == name {
                        for (i, v) in list.split(',').enumerate().take(8) {
                            ints[i] = i64::from_str_radix(v.trim().trim_start_matches("0x"), 16).unwrap_or(0) as i32 as i64;
                        }
                    }
                }
            }
            if let Some(sp) = std::env::var("TS_VERIFY_SP").ok().and_then(|v| u32::from_str_radix(v.trim_start_matches("0x"), 16).ok()) {
                INTERP_SP.store(sp, std::sync::atomic::Ordering::Relaxed);
                ts_rt::frame::set_sp(sp);
            }
            if !snapshots.is_empty() {
                use_real_model(&mut rng, name, &mut ints);
            }
            let a = run_interp(&ctx, addr, &ints, &floats);
            if std::env::var_os("TS_VERIFY_INTERP_ONLY").is_some() {
                match &a {
                    Outcome::Ok { ret, writes, stores, .. } => println!("interpreter: finished, returned {ret:#x}, {} bytes changed, {} stores", writes.len(), stores.len()),
                    Outcome::Skip(s) => println!("interpreter: {s}"),
                    Outcome::Panic(p) => println!("interpreter: fault {p}"),
                    Outcome::Trap => println!("interpreter: trap"),
                }
                continue;
            }
            if let Outcome::Skip(s) = &a {
                // TS_VERIFY_SMALL: a run whose loops do not finish is left out, the others still count
                // (a huge count also lets the original write past the end of RAM, where the interpreter wraps round over its own code: that run is left out too)
                if (s == "timeout" || s.starts_with("syscall")) && std::env::var_os("TS_VERIFY_SMALL").is_some() {
                    continue;
                }
                skipped = Some(s.clone());
                break;
            }
            let b = run_native(addr, &ints, &floats);
            if let Outcome::Skip(s) = &b {
                skipped = Some(s.clone());
                break;
            }
            if std::env::var_os("TS_VERIFY_FAULTS").is_some() {
                if let Outcome::Panic(m) = &a { println!("run {t}: interpreter fault: {m}"); }
                if let Outcome::Panic(m) = &b { println!("run {t}: native fault: {m}"); }
            }
            match (&a, &b) {
                (Outcome::Ok { ret: r1, fret: f1, writes: w1, stores: s1 }, Outcome::Ok { ret: r2, fret: f2, writes: w2, stores: s2 }) => {
                    real += 1;
                    // `TS_VERIFY_IGNORE=low-high` (hex): leave a range of addresses out of the comparison (words that the original gets from stack garbage, such as the last hit that `bulletInitialiseStraightLineTest` puts back when nothing was hit)
                    let ignore: Option<(u32, u32)> = std::env::var("TS_VERIFY_IGNORE").ok().and_then(|v| {
                        let (lo, hi) = v.split_once('-')?;
                        Some((u32::from_str_radix(lo.trim_start_matches("0x"), 16).ok()?, u32::from_str_radix(hi.trim_start_matches("0x"), 16).ok()?))
                    });
                    let keep = |w: &Vec<(u32, u8)>| -> Vec<(u32, u8)> { w.iter().cloned().filter(|(a, _)| ignore.map_or(true, |(lo, hi)| *a < lo || *a >= hi)).collect() };
                    let (w1, w2) = (&keep(w1), &keep(w2));
                    if _rk == 1 && (*r1 as i32) != (*r2 as i32) && (*r1 != *r2) {
                        bad = Some(format!("test {t}: int return differs: interp {r1:#x} native {r2:#x} (ints {ints:x?})"));
                    } else if !same_f(*f1, *f2) && _rk == 2 {
                        bad = Some(format!("test {t}: float return differs: interp {f1} native {f2} (ints {ints:x?} floats {floats:?})"));
                    } else if w1 != w2 && !stores_equal_mod_nan(s1, s2) && !changes_equal_mod_nan(&w1, &w2, s1, s2) {
                        let mut d = String::new();
                        let m1: HashMap<_, _> = w1.iter().cloned().collect();
                        let m2: HashMap<_, _> = w2.iter().cloned().collect();
                        let mut keys: Vec<_> = m1.keys().chain(m2.keys()).cloned().collect();
                        keys.sort_unstable();
                        keys.dedup();
                        let mut shown = 0;
                        for k in keys {
                            if m1.get(&k) != m2.get(&k) && shown < 4 {
                                d += &format!(" [{k:#x}: interp {:?} native {:?}]", m1.get(&k), m2.get(&k));
                                shown += 1;
                            }
                        }
                        let mut first = String::new();
                        let cover = |ss: &[trace::Store], a: u32| ss.iter().rev().find(|x| a >= x.addr && a < x.addr + x.size as u32).copied();
                        let mut dk: Vec<u32> = m1.keys().chain(m2.keys()).cloned().collect();
                        dk.sort_unstable();
                        dk.dedup();
                        if let Some(&k) = dk.iter().find(|k| m1.get(k) != m2.get(k)) {
                            let f = |s: Option<trace::Store>| match s {
                                Some(s) => format!("[{:#x} +{} = {:02x?} pc {:#x}]", s.addr, s.size, &s.new[..s.size as usize], s.tag),
                                None => "[no store]".to_string(),
                            };
                            first = format!(" | byte {k:#x}: last interp store {} / last native store {}", f(cover(s1, k)), f(cover(s2, k)));
                        }
                        if let Some(&k) = dk.iter().find(|k| m1.get(k) != m2.get(k)) {
                            if std::env::var_os("TS_VERIFY_HISTORY").is_some() {
                                for (label, ss) in [("interp", s1), ("native", s2)] {
                                    for x in ss.iter().filter(|x| k >= x.addr && k < x.addr + x.size as u32) {
                                        println!("  {label} store {:#x} +{} = {:02x?} pc {:#x}", x.addr, x.size, &x.new[..x.size as usize], x.tag);
                                    }
                                }
                            }
                        }
                        bad = Some(format!("test {t}: memory writes differ ({} vs {} bytes){d}{first} (ints {ints:x?} floats {floats:?})", w1.len(), w2.len()));
                    }
                }
                (Outcome::Panic(_), Outcome::Panic(_)) | (Outcome::Trap, Outcome::Trap) => {}
                (Outcome::Panic(_), Outcome::Ok { .. }) => {} // dead loads removed by the decompiler never fault
                // the interpreter wraps addresses past the end of the port's RAM (it was made for the console's 32 MB), the port faults on them: a garbage pointer, not a difference
                (Outcome::Ok { .. }, Outcome::Panic(p)) if p.contains("unmapped game address") && p.rsplit("0x").next().and_then(|h| u32::from_str_radix(h.trim(), 16).ok()).map_or(false, |a| a >= 0x0800_0000) => {}
                (Outcome::Trap, Outcome::Panic(_)) | (Outcome::Panic(_), Outcome::Trap) => {}
                (x, y) => {
                    bad = Some(format!("test {t}: outcome differs: interp {x:?} native {y:?} (ints {ints:x?} floats {floats:?})"));
                }
            }
            if bad.is_some() {
                break;
            }
        }
        if std::env::var("TS_VERIFY_COVERAGE").map_or(false, |v| v == "all") {
            print_all_coverage(&syms);
        } else if std::env::var_os("TS_VERIFY_COVERAGE").is_some() {
            if let Some(sym) = syms.iter().find(|x| x.addr == addr && x.is_func) {
                let covered = COVERAGE.lock().unwrap().take().unwrap_or_default();
                let total = sym.size / 4;
                let hit = (0..total).filter(|k| covered.contains(&(addr + k * 4))).count();
                println!("coverage {name}: {hit} of {total} instructions");
                let mut run_start: Option<u32> = None;
                for k in 0..=total {
                    let pc = addr + k * 4;
                    let missed = k < total && !covered.contains(&pc);
                    match (missed, run_start) {
                        (true, None) => run_start = Some(pc),
                        (false, Some(start)) => {
                            println!("  not run: {start:#x} to {:#x} ({} instructions)", pc - 4, (pc - start) / 4);
                            run_start = None;
                        }
                        _ => {}
                    }
                }
            }
        }
        if let Some(b) = bad {
            fail += 1;
            println!("FAIL {name} @ {addr:#x}: {b}");
            failures.push(name);
        } else if let Some(s) = skipped {
            skip += 1;
            if verbose {
                println!("skip {name}: {s}");
            }
        } else if real == 0 {
            vacuous += 1;
            if verbose {
                println!("vacuous {name} (every run faulted)");
            }
        } else {
            pass += 1;
            if verbose {
                println!("ok   {name} ({real} runs)");
            }
        }
    }
    println!("\n== pass {pass}, fail {fail}, skipped {skip}, vacuous {vacuous} ==");
}
