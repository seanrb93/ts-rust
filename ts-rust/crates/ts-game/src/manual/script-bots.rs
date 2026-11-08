//! What the scripts of mods can ask about the computer characters and the things around them, and the places in the game's own code where a script may change what a computer
//! character does.
//!
//! A script can ask what a character can see, its health and armour, its guns and ammunition, the shots in flight and the pickups of the level; it can switch a computer
//! character's gun, send it running to a place, hold its fire, choose which of its gun's two shots it fires, and scale its aim and the time it takes to react.  The game's code asks
//! the scripts (with the `extensions` feature) whom a computer character goes after (`on_bot_target`), where it aims (`on_bot_aim`), whether a shot no player sees hits
//! (`on_bot_unseen_hit`) and which weapon it fetches (`on_bot_weapon`).  None of it costs anything while no script is running.  `docs/scripting.md` lists it all.

use super::*;
use rhai::{Array, Dynamic, Map};

/// The bullets of the game: an array of 50 records of 276 bytes (+0 the kind, 0 for a free one; +24 the place; +48 the velocity in units a frame; +72 the age in seconds;
/// +104 the kind of owner, 1 and 4 a player and 2 a computer character, and +108 its number).
const BULLETS: i32 = 0x3b0138;
const BULLET_COUNT: i32 = 50;
const BULLET_BYTES: i32 = 276;

fn float_map(pairs: &[(&str, f64)]) -> Map {
    let mut m = Map::new();
    for (k, v) in pairs {
        m.insert((*k).into(), Dynamic::from_float(*v));
    }
    m
}

/// The place a character looks from (its record keeps it at +3008: computer characters and players alike).
fn eye(i: i64) -> [f32; 3] {
    let r = record(i);
    [lwc1(r + 3008), lwc1(r + 3012), lwc1(r + 3016)]
}

/// Is nothing in the way from the eyes of the character `i` to the place `to`?  The test the game makes for a computer character and its target (`aiTickBefore`), with the
/// character itself left out of it.  `target`: the prop that counts as reached when it is hit (0: none).
fn line_clear(i: i64, to: [f32; 3], target: i32) -> bool {
    let p = prop_of(i);
    if p == 0 || lw(p + 12) == 0 {
        return false;
    }
    let from = eye(i);
    let frame = Frame::new(64);
    let sp = frame.sp();
    for k in 0..3 {
        swc1(sp + 48 + 4 * k as i32, from[k]);
        swc1(sp + 16 + 4 * k as i32, to[k] - from[k]);
    }
    // (the arguments the line test takes on the stack)
    sw(sp, (target != 0) as i32);
    sw(sp + 8, 0);
    let flags = lw(p + 16);
    sw(p + 16, flags & -33);
    let hit = bgLineTest(lw(p + 12), sp + 48, sp + 16, 0, 0, 0, 0, 112);
    sw(p + 16, flags);
    hit == 0 || (target != 0 && hit == target)
}

/// Can the character `i` see the character `j` (nothing in the way from its eyes to the middle of the other's body)?
fn sees(i: i64, j: i64) -> bool {
    let (a, b) = (prop_of(i), prop_of(j));
    if a == 0 || b == 0 || i == j || !is_alive(j) {
        return false;
    }
    // (the height the game tests: the other's eyes, a little lower)
    let at = [lwc1(b + 48), lwc1(record(j) + 3012) - lwc1(0x3ad7e4), lwc1(b + 56)];
    line_clear(i, at, b)
}

fn ammo_type(gun: i32) -> i32 {
    if (0..gun_count()).contains(&gun) {
        lw(gun * 400 + GUNS_TABLE + 16)
    } else {
        -1
    }
}

pub(super) fn register(engine: &mut rhai::Engine) {
    // the game's mode (1 BagTag, 2 Capture the Bag, 3 Deathmatch, 4 Escort, 5 KnockOut, 6 Last Stand, 7 the story), and whether the script of a mode of a mod is playing it
    engine.register_fn("game_mode", || lw(g::curgameparms + 72) as i64);
    engine.register_fn("mode_script", || MODE_SCRIPT.load(Ordering::Relaxed));
    // health and armour, each a part of 1 (the health is +520 of the character's prop, the armour +12 of its record)
    engine.register_fn("health", |i: i64| match prop_of(i) { 0 => 0.0, p => lwc1(p + 520) as f64 });
    engine.register_fn("armour", |i: i64| if prop_of(i) == 0 { 0.0 } else { lwc1(record(i) + 12) as f64 });
    // the skill the match gives a computer character: 0 (the easiest) to 4
    engine.register_fn("bot_skill", |i: i64| if prop_of(i) == 0 || is_player_index(i) { 0 } else { lbu(g::curgameparms + 49 + i as i32) as i64 });
    // what a character sees
    engine.register_fn("can_see", |i: i64, j: i64| sees(i, j));
    engine.register_fn("line_clear", |i: i64, x: f64, y: f64, z: f64| line_clear(i, [x as f32, y as f32, z as f32], 0));
    engine.register_fn("eye_y", |i: i64| if prop_of(i) == 0 { 0.0 } else { lwc1(record(i) + 3012) as f64 });

    // guns: a weapon (a "combination") is one gun or a gun in each hand; a gun has two shots
    engine.register_fn("weapon", |i: i64| if prop_of(i) == 0 { 0 } else { lw(record(i) + 260) as i64 });
    engine.register_fn("owns", |i: i64, combo: i64| prop_of(i) != 0 && (0..combo_count() as i64).contains(&combo) && own_get(record(i), combo as i32) != 0);
    engine.register_fn("bot_switch", |i: i64, combo: i64| {
        if prop_of(i) != 0 && !is_player_index(i) && (0..combo_count() as i64).contains(&combo) && own_get(record(i), combo as i32) != 0 {
            sw(record(i) + 260, combo as i32);
        }
    });
    // the gun in the right hand of a weapon (-1 for none: the fists)
    engine.register_fn("weapon_gun", |combo: i64| if (0..combo_count() as i64).contains(&combo) { lh(GUNCOMBOS + 8 * combo as i32 + 4) as i64 } else { -1 });
    engine.register_fn("weapon_rank", |combo: i64| if (0..combo_count() as i64).contains(&combo) { lh(GUNCOMBOS + 8 * combo as i32) as i64 } else { 0 });
    // a shot of a gun (`second` for its second shot): its kind of bullet, speed (units a frame), damage (a part of a character's health) and spread
    engine.register_fn("shot_kind", |gun: i64, second: bool| shot_field(gun, second, 0, true));
    engine.register_fn("shot_speed", |gun: i64, second: bool| shot_field(gun, second, 28, false));
    engine.register_fn("shot_damage", |gun: i64, second: bool| shot_field(gun, second, 20, false));
    engine.register_fn("shot_spread", |gun: i64, second: bool| shot_field(gun, second, 32, false));
    // the frames a computer character waits between two shots of a gun
    engine.register_fn("gun_wait", |gun: i64| if (0..gun_count() as i64).contains(&gun) { lw(gun as i32 * 400 + GUNS_TABLE + 60) as i64 } else { 0 });
    // the ammunition a character has for a gun (not counting what is in the gun), the most it can have, and setting it
    engine.register_fn("ammo", |i: i64, gun: i64| match ammo_type(gun as i32) {
        t if t >= 0 && prop_of(i) != 0 => lw(record(i) + 620 + 4 * t) as i64,
        _ => 0,
    });
    engine.register_fn("ammo_max", |gun: i64| match ammo_type(gun as i32) {
        t if t >= 0 => lw(g::ammoinfo + 8 * t) as i64,
        _ => 0,
    });
    engine.register_fn("set_ammo", |i: i64, gun: i64, n: i64| {
        let t = ammo_type(gun as i32);
        if t >= 0 && prop_of(i) != 0 {
            sw(record(i) + 620 + 4 * t, (n.max(0) as i32).min(lw(g::ammoinfo + 8 * t)));
        }
    });

    // the shots in flight: a list of maps (kind, x, y, z, vx, vy, vz in units a frame, age in seconds, owner the character that fired it or -1)
    engine.register_fn("shots", || {
        let mut out = Array::new();
        let base = lw(BULLETS);
        if base == 0 {
            return out;
        }
        for k in 0..BULLET_COUNT {
            let b = base + k * BULLET_BYTES;
            let kind = lw(b);
            if kind == 0 {
                continue;
            }
            let owner = match lw(b + 104) {
                1 | 2 | 4 => lw(b + 108) as i64,
                _ => -1,
            };
            let mut m = float_map(&[
                ("x", lwc1(b + 24) as f64),
                ("y", lwc1(b + 28) as f64),
                ("z", lwc1(b + 32) as f64),
                ("vx", lwc1(b + 48) as f64),
                ("vy", lwc1(b + 52) as f64),
                ("vz", lwc1(b + 56) as f64),
                ("age", lwc1(b + 72) as f64),
            ]);
            m.insert("kind".into(), Dynamic::from_int(kind as i64));
            m.insert("owner".into(), Dynamic::from_int(owner));
            out.push(Dynamic::from_map(m));
        }
        out
    });

    // the pickups of the level: a list of maps (id, kind "weapon" / "health" / "armour" / "ammo", weapon for a weapon, pad, x, y, z, and here, false while it is taken)
    engine.register_fn("pickups", || {
        let mut out = Array::new();
        for (list, count) in [(g::weaponspawn, g::numPickupWeapons), (g::healthspawn, g::numPickupHealth)] {
            for k in 0..lw(count).clamp(0, 64) {
                let pad = lw(list + 8 * k);
                let prop = lw(list + 8 * k + 4);
                if prop == 0 || lw(prop + 352) == 0 {
                    continue;
                }
                let data = lw(prop + 352);
                // (the kinds of pickup: 1 ammunition, 2 a weapon, 4 health, 8 armour)
                let kind = match lw(data) {
                    1 => "ammo",
                    2 => "weapon",
                    4 => "health",
                    8 => "armour",
                    _ => continue,
                };
                let mut m = float_map(&[("x", lwc1(prop + 48) as f64), ("y", lwc1(prop + 52) as f64), ("z", lwc1(prop + 56) as f64)]);
                m.insert("id".into(), Dynamic::from_int(prop as i64));
                m.insert("kind".into(), Dynamic::from(kind.to_string()));
                m.insert("weapon".into(), Dynamic::from_int(if kind == "weapon" { lw(data + 12) as i64 } else { 0 }));
                // (the pad a computer character goes to for it: the weapons keep theirs at +36 of their record, the health and armour in the list)
                let pad = if kind == "weapon" { lw(data + 36) } else { pad };
                m.insert("pad".into(), Dynamic::from_int(pad as i64));
                m.insert("here".into(), Dynamic::from_bool(lw(prop + 16) & 1 != 0));
                out.push(Dynamic::from_map(m));
            }
        }
        out
    });

    // a computer character: fire or not, which shot, its aim, how quickly it reacts, running to a place, and whether it uses up ammunition
    engine.register_fn("bot_hold_fire", |i: i64, on: bool| {
        fx_of(i, |f| f.hold_fire = on);
    });
    engine.register_fn("bot_fire_mode", |i: i64, mode: i64| {
        fx_of(i, |f| f.fire_mode = mode.clamp(-1, 1) as i32);
    });
    engine.register_fn("bot_spread", |i: i64, part: f64| {
        fx_of(i, |f| f.spread = part.max(0.0) as f32);
    });
    engine.register_fn("bot_reaction", |i: i64, part: f64| {
        fx_of(i, |f| f.react = part.max(0.0) as f32);
    });
    engine.register_fn("bot_run_to", |i: i64, x: f64, y: f64, z: f64| {
        fx_of(i, |f| f.run_to = Some([x as f32, y as f32, z as f32]));
    });
    engine.register_fn("bot_run_clear", |i: i64| {
        let was = fx_of(i, |f| f.run_to.take().is_some()).unwrap_or(false);
        // (it chooses what to do again)
        if was && prop_of(i) != 0 && !is_player_index(i) && lw(record(i) + 2716) == 8 {
            sw(record(i) + 2716, 128);
        }
    });
    engine.register_fn("bot_uses_ammo", |i: i64, on: bool| {
        fx_of(i, |f| f.uses_ammo = on);
    });
}

fn shot_field(gun: i64, second: bool, at: i32, int: bool) -> Dynamic {
    if !(0..gun_count() as i64).contains(&gun) {
        return if int { Dynamic::from_int(0) } else { Dynamic::from_float(0.0) };
    }
    let shot = gun as i32 * 400 + GUNS_TABLE + if second { 188 } else { 52 };
    if int {
        Dynamic::from_int(lw(shot + at) as i64)
    } else {
        Dynamic::from_float(lwc1(shot + at) as f64)
    }
}

/// The number of the computer character whose prop is `chr`, when a script is running and asks something of it.
fn bot_number(chr: i32) -> Option<i32> {
    if !ACTIVE.load(Ordering::Relaxed) {
        return None;
    }
    let mind = lw(chr + 352);
    if mind == 0 {
        return None;
    }
    let n = lw(mind);
    (n >= lw(g::lvnumplayers)).then_some(n)
}

/// The number of the character whose prop is `prop` (-1 when it is not a character).
fn character_of(prop: i32) -> i64 {
    if prop == 0 || lw(prop + 8) != 8 || lw(prop + 352) == 0 {
        -1
    } else {
        lw(lw(prop + 352)) as i64
    }
}

/// `gunChrFire`: the computer character holds its fire.
pub fn script_bot_holds_fire(chr: i32) -> bool {
    bot_number(chr).and_then(|n| fx_read(n, |f| f.hold_fire)).unwrap_or(false)
}

/// `gunChrFire`: which shot of its gun the computer character fires (-1: the game's choice).
pub fn script_bot_fire_mode(chr: i32) -> i32 {
    bot_number(chr).and_then(|n| fx_read(n, |f| f.fire_mode)).unwrap_or(-1)
}

/// `gunChrFire`: the spread of a computer character's own aim, `own` in the game.
pub fn script_bot_spread(chr: i32, own: f32) -> f32 {
    match bot_number(chr).and_then(|n| fx_read(n, |f| f.spread)) {
        Some(s) if s != 1.0 => own * s,
        _ => own,
    }
}

/// `enemyReactTime`: the time a computer character takes to react, `time` in the game.
pub fn script_bot_react(chr: i32, time: f32) -> f32 {
    match bot_number(chr).and_then(|n| fx_read(n, |f| f.react)) {
        Some(s) if s != 1.0 => time * s,
        _ => time,
    }
}

/// `gunChrFire`, before a shot: a computer character that uses ammunition and has none left does not fire; one that has takes one.
pub fn script_bot_take_ammo(chr: i32, gun: i32) -> bool {
    let Some(n) = bot_number(chr) else { return true };
    if !fx_read(n, |f| f.uses_ammo).unwrap_or(false) {
        return true;
    }
    let t = ammo_type(gun);
    if t < 0 {
        return true;
    }
    let at = lw(chr + 352) + 620 + 4 * t;
    let left = lw(at);
    if left <= 0 {
        return false;
    }
    sw(at, left - 1);
    true
}

/// `selectTarget`, once the game has chosen whom a computer character goes after: a script may choose another (`on_bot_target(i)` gives the number of a character, or -1 for the
/// game's choice).  Gives the prop of the character chosen.
pub fn script_bot_target(chr: i32) -> Option<i32> {
    let n = bot_number(chr)?;
    if !has("on_bot_target") {
        return None;
    }
    let answer = call_all::<_, i64>("on_bot_target", (n as i64,)).into_iter().find(|&t| t >= 0)?;
    if answer == n as i64 || !is_alive(answer) {
        return None;
    }
    let p = prop_of(answer);
    (p != 0).then_some(p)
}

/// `gunChrFire`: the place a computer character aims at (three floats at `at`), which a script may move (`on_bot_aim(i, t, x, y, z)` answers `[x, y, z]`, or anything else to
/// leave it).
pub fn script_bot_aim(chr: i32, at: i32) {
    let Some(n) = bot_number(chr) else { return };
    if !has("on_bot_aim") {
        return;
    }
    let t = character_of(lw(lw(chr + 352) + 2788));
    if t < 0 {
        return;
    }
    let (x, y, z) = (lwc1(at) as f64, lwc1(at + 4) as f64, lwc1(at + 8) as f64);
    for answer in call_all::<_, Dynamic>("on_bot_aim", (n as i64, t, x, y, z)) {
        if let Some(a) = answer.try_cast::<Array>() {
            let v: Vec<f32> = a.iter().filter_map(|d| d.as_float().ok().map(|f| f as f32).or_else(|| d.as_int().ok().map(|i| i as f32))).collect();
            if v.len() == 3 && v.iter().all(|f| f.is_finite()) {
                swc1(at, v[0]);
                swc1(at + 4, v[1]);
                swc1(at + 8, v[2]);
                return;
            }
        }
    }
}

/// `gunChrFire`, a shot at a character that no player sees, which the game makes hit at once: a script may say it misses (`on_bot_unseen_hit(i, t)` answers false).
pub fn script_bot_unseen_hit(chr: i32) -> bool {
    let Some(n) = bot_number(chr) else { return true };
    if !has("on_bot_unseen_hit") {
        return true;
    }
    let t = character_of(lw(lw(chr + 352) + 2788));
    call_all::<_, bool>("on_bot_unseen_hit", (n as i64, t)).into_iter().all(|b| b)
}

/// `selectWeaponPad`: the weapon a computer character with none goes to fetch, which a script may choose (`on_bot_weapon(i)` gives the `id` of a pickup of `pickups()`, or -1).
/// Gives the prop of the pickup when it is a weapon that is there.
pub fn script_bot_weapon(chr: i32) -> Option<i32> {
    let n = bot_number(chr)?;
    if !has("on_bot_weapon") {
        return None;
    }
    let id = call_all::<_, i64>("on_bot_weapon", (n as i64,)).into_iter().find(|&t| t > 0)? as i32;
    let ok = (0..lw(g::numPickupWeapons).clamp(0, 64)).any(|k| lw(g::weaponspawn + 8 * k + 4) == id);
    (ok && lw(id + 16) & 1 != 0 && lw(id + 352) != 0).then_some(id)
}
