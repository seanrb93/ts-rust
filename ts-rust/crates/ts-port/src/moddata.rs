//! `weapons.txt` and `characters.txt` of a mod: the game's tables of weapons and characters set by the names of their fields instead of addresses.
//!
//! ```text
//! # weapons.txt
//! weapon 0                # the pistol (a number 0 to 29: the order of the game's table `guns`)
//!   clip 15
//!   ammo_kind 1
//!   max_ammo 250          # the most of its kind of ammo that can be carried (it is kept per kind of ammo, so it changes the other weapons of the kind too)
//!   bullet_type 3
//!   flags 0x4
//!   raw +60 w 7           # a field that has no name yet: offset in the entry of 400 bytes, b h w or f, the value
//!
//! # characters.txt
//! character 3             # a number 0 to 95: the order of the game's table `chrs`
//!   model 120             # the number of the body model (a prop of the game's table of props)
//!   front_model 121
//!   name_text 5000        # numbers of the texts of the name (see chrnames.rs)
//!   flags 1
//!   scale 1.1
//! ```
//!
//! The text is read when the mods are loaded and the values are written into the game's tables before the game starts, as `patches.txt` does, so the checks are the file's:
//! a line that names a weapon or a field that does not exist, or a value out of its range, is reported with its line number and left out.

use std::path::Path;

pub const GUNS: i32 = 0x366218;
pub const GUN_SIZE: i32 = 400;
pub const GUN_COUNT: i32 = 30;
pub const AMMOINFO: i32 = 0x3661a0;
pub const AMMO_KINDS: i32 = 15;
pub const CHRS: i32 = 0x3650e0;
pub const CHR_SIZE: i32 = 44;
pub const CHR_COUNT: i32 = 80;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    B,
    H,
    W,
    F,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f32),
}

/// One write into the game's memory (or, for `max_ammo`, into the table of ammo of the weapon's kind of ammo, found when the writes are made).
#[derive(Clone, Debug, PartialEq)]
pub enum Op {
    Put { addr: i32, kind: Kind, value: Value },
    MaxAmmo { weapon: i32, value: i64 },
}

/// A field with a name: its place in the entry, how it is stored, the lowest and highest value.
struct Field {
    name: String,
    at: i32,
    kind: Kind,
    min: f64,
    max: f64,
}

fn field(name: &str, at: i32, kind: Kind, min: f64, max: f64) -> Field {
    Field { name: name.to_string(), at, kind, min, max }
}

/// The weapon's fields that are named.  Seen in the code: how the game reads them (the entry is `guns + 400 * weapon`).  A weapon has a second shot (the alternate
/// fire) at +188 that is laid out as the first at +52, so the fields of the shot are listed once and named again with `alt_` in front.
fn weapon_fields() -> Vec<Field> {
    let mut v = vec![
        field("ammo_kind", 16, Kind::W, 0.0, AMMO_KINDS as f64 - 1.0),   // the kind of ammo (an index of `ammoinfo`)
        field("clip", 20, Kind::W, -1.0, 9999.0),                         // rounds in a clip (-1: no clip)
        field("ammo_reserve", 24, Kind::W, 0.0, 9999.0),                  // what a player starts with or is given of the ammo (read by the level setup and by what a dead enemy drops)
        field("flags", 28, Kind::W, 0.0, 0xffff as f64),                  // 1, 2, 8 change the animations a character uses and what an enemy does with the weapon
        field("view_x", 32, Kind::F, -10.0, 10.0),                        // where the gun is held on the screen: across (mirrored for the left hand), up, forward
        field("view_y", 36, Kind::F, -10.0, 10.0),
        field("view_z", 40, Kind::F, -10.0, 10.0),
        field("fov_a", 44, Kind::F, 1.0, 179.0),                          // the fields of view the zoom uses (player + 268, + 272)
        field("fov_b", 48, Kind::F, 1.0, 179.0),
        field("ai_burst", 60, Kind::W, 0.0, 9999.0),                          // compared with an enemy's shot counter: how long its burst lasts
        field("muzzle_bone", 364, Kind::W, -1.0, 255.0),                  // the bones of the model that the shot, the muzzle flash and the sparks come from
        field("flash_bone", 340, Kind::W, -1.0, 255.0),
        field("spark_bone", 360, Kind::W, -1.0, 255.0),
    ];
    for (prefix, base) in [("", 52), ("alt_", 188)] {
        v.push(field(&format!("{prefix}bullet_type"), base, Kind::W, 0.0, 255.0));        // what the shot makes (the table of bullets)
        v.push(field(&format!("{prefix}damage"), base + 20, Kind::F, 0.0, 1000.0));        // what a bullet takes from what it hits (a character's health is about 1)
        v.push(field(&format!("{prefix}ricochet"), base + 16, Kind::F, 0.0, 1.0));        // the chance that a bullet bounces off a wall
        v.push(field(&format!("{prefix}bullet_speed"), base + 28, Kind::F, 0.0, 1000.0)); // scales the direction of a bullet
        v.push(field(&format!("{prefix}spread"), base + 32, Kind::F, 0.0, 10.0));         // how far a bullet can leave the aim
        v.push(field(&format!("{prefix}recoil"), base + 56, Kind::F, -100.0, 100.0));     // the gun's kick back
        v.push(field(&format!("{prefix}kick"), base + 60, Kind::F, -100.0, 100.0));       // multiplies the recoil
        v.push(field(&format!("{prefix}pull"), base + 72, Kind::F, -100.0, 100.0));       // how far the view is pulled by a shot
    }
    v
}

fn character_fields() -> Vec<Field> {
    vec![
        field("name_text", 0, Kind::H, 0.0, 65535.0),
        field("name_text_2", 2, Kind::H, 0.0, 65535.0),
        field("model", 8, Kind::W, 0.0, 1100.0),
        field("front_model", 12, Kind::W, 0.0, 1100.0),
        field("flags", 16, Kind::W, -2147483648.0, 2147483647.0),
        field("scale", 40, Kind::F, 0.01, 100.0),
    ]
}

fn parse_int(s: &str) -> Option<i64> {
    let s = s.trim();
    if let Some(h) = s.strip_prefix("0x") {
        i64::from_str_radix(h, 16).ok()
    } else if let Some(h) = s.strip_prefix("-0x") {
        i64::from_str_radix(h, 16).ok().map(|v| -v)
    } else {
        s.parse().ok()
    }
}

/// Read the text of a `weapons.txt` (`which` = "weapon") or `characters.txt` (`which` = "character"); the writes it asks for, and the complaints (with line numbers).
pub fn parse(which: &str, text: &str) -> (Vec<Op>, Vec<ts_game::NewWeapon>, Vec<String>) {
    let (fields, base, size, count): (Vec<Field>, i32, i32, i32) = if which == "weapon" { (weapon_fields(), GUNS, GUN_SIZE, GUN_COUNT) } else { (character_fields(), CHRS, CHR_SIZE, CHR_COUNT) };
    let mut ops = Vec::new();
    let mut errors = Vec::new();
    let mut current: Option<i32> = None;
    // (which weapon the writes after each `weapon` line belong to: the added weapons take theirs out of `ops` at the end)
    let mut segments: Vec<(i32, usize)> = Vec::new();
    let mut new_weapons: std::collections::BTreeMap<i32, ts_game::NewWeapon> = Default::default();
    let names = |f: &[Field]| f.iter().map(|f| f.name.as_str()).collect::<Vec<_>>().join(", ");
    for (n, line) in text.lines().enumerate() {
        let n = n + 1;
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let words: Vec<&str> = line.split_whitespace().collect();
        let key = words[0];
        if key == which {
            let limit = if which == "weapon" { ts_game::MAX_GUNS } else { count };
            current = match words.get(1).and_then(|w| parse_int(w)) {
                Some(v) if (0..limit as i64).contains(&v) => {
                    segments.push((v as i32, ops.len()));
                    if which == "weapon" && v as i32 >= GUN_COUNT {
                        new_weapons.entry(v as i32).or_insert_with(|| ts_game::NewWeapon { number: v as i32, copy_of: -1, name: String::new(), puts: Vec::new(), max_ammo: None, view_model_file: String::new(), model_file: String::new(), ammo_model_file: String::new() });
                    }
                    Some(v as i32)
                }
                Some(v) => {
                    errors.push(format!("line {n}: {which} {v} does not exist (0 to {})", limit - 1));
                    None
                }
                None => {
                    errors.push(format!("line {n}: `{which}` needs a number"));
                    None
                }
            };
            continue;
        }
        let Some(item) = current else {
            errors.push(format!("line {n}: `{line}` comes before any `{which} <number>` line (or after one that was wrong)"));
            continue;
        };
        let is_new = which == "weapon" && item >= GUN_COUNT;
        let entry = if is_new { 0 } else { base + item * size };
        if which == "weapon" && key == "copy" {
            match words.get(1).and_then(|w| parse_int(w)) {
                Some(v) if is_new && (0..item as i64).contains(&v) => {
                    new_weapons.get_mut(&item).unwrap().copy_of = v as i32;
                }
                _ if !is_new => errors.push(format!("line {n}: `copy` is only for added weapons (numbers from {GUN_COUNT})")),
                _ => errors.push(format!("line {n}: `copy` needs the number of an earlier weapon")),
            }
            continue;
        }
        if which == "weapon" && matches!(key, "view_model_file" | "model_file" | "ammo_model_file") {
            match words.get(1) {
                Some(p) if is_new && !p.contains("..") => {
                    let w = new_weapons.get_mut(&item).unwrap();
                    let p = p.trim_matches('"').trim_start_matches('/').to_string();
                    match key {
                        "view_model_file" => w.view_model_file = p,
                        "model_file" => w.model_file = p,
                        _ => w.ammo_model_file = p,
                    }
                }
                _ => errors.push(format!("line {n}: `{key}` is for added weapons (numbers from {GUN_COUNT}) and needs a path in the pack, such as ob/chrgun90/chrgun90.raw")),
            }
            continue;
        }
        if which == "weapon" && key == "name" {
            let name = line.splitn(2, char::is_whitespace).nth(1).unwrap_or("").trim().trim_matches('"').to_string();
            if !is_new {
                errors.push(format!("line {n}: `name` is only for added weapons (numbers from {GUN_COUNT})"));
            } else if name.is_empty() {
                errors.push(format!("line {n}: `name` needs a name"));
            } else {
                new_weapons.get_mut(&item).unwrap().name = name;
            }
            continue;
        }
        if key == "raw" {
            // raw +<offset> <b|h|w|f> <value>
            let (Some(off), Some(kind), Some(val)) = (words.get(1).and_then(|w| parse_int(w.trim_start_matches('+'))), words.get(2), words.get(3)) else {
                errors.push(format!("line {n}: expected `raw +<offset> <b|h|w|f> <value>`"));
                continue;
            };
            if !(0..size as i64).contains(&off) {
                errors.push(format!("line {n}: offset {off} is outside the entry ({size} bytes)"));
                continue;
            }
            let (kind, width) = match *kind {
                "b" => (Kind::B, 1),
                "h" => (Kind::H, 2),
                "w" => (Kind::W, 4),
                "f" => (Kind::F, 4),
                other => {
                    errors.push(format!("line {n}: `{other}` is not b, h, w or f"));
                    continue;
                }
            };
            if off + width > size as i64 {
                errors.push(format!("line {n}: the value does not fit in the entry"));
                continue;
            }
            let value = if kind == Kind::F { val.parse::<f32>().ok().map(Value::Float) } else { parse_int(val).map(Value::Int) };
            match value {
                Some(value) => ops.push(Op::Put { addr: entry + off as i32, kind, value }),
                None => errors.push(format!("line {n}: `{val}` is not a number")),
            }
            continue;
        }
        if which == "weapon" && key == "max_ammo" {
            match words.get(1).and_then(|w| parse_int(w)) {
                Some(v) if (0..=65535).contains(&v) => ops.push(Op::MaxAmmo { weapon: item, value: v }),
                _ => errors.push(format!("line {n}: `max_ammo` needs a number 0 to 65535")),
            }
            continue;
        }
        let Some(field) = fields.iter().find(|f| f.name == key) else {
            let extra = if which == "weapon" { ", max_ammo" } else { "" };
            errors.push(format!("line {n}: unknown field `{key}` (the fields are {}{extra}, raw)", names(&fields)));
            continue;
        };
        if field.kind == Kind::F {
            match words.get(1).and_then(|w| w.parse::<f32>().ok()) {
                Some(v) if (field.min..=field.max).contains(&(v as f64)) => ops.push(Op::Put { addr: entry + field.at, kind: field.kind, value: Value::Float(v) }),
                Some(v) => errors.push(format!("line {n}: {key} {v} is out of range ({} to {})", field.min, field.max)),
                None => errors.push(format!("line {n}: `{key}` needs a number")),
            }
            continue;
        }
        match words.get(1).and_then(|w| parse_int(w)) {
            Some(v) if (field.min..=field.max).contains(&(v as f64)) => ops.push(Op::Put { addr: entry + field.at, kind: field.kind, value: Value::Int(v) }),
            Some(v) => errors.push(format!("line {n}: {key} {v} is out of range ({} to {})", field.min, field.max)),
            None => errors.push(format!("line {n}: `{key}` needs a number")),
        }
    }
    // the writes of the added weapons become changes of the copy, by their place in the entry
    let mut kept = Vec::new();
    let mut owner: Vec<Option<i32>> = vec![None; ops.len()];
    for (k, &(item, start)) in segments.iter().enumerate() {
        let end = segments.get(k + 1).map_or(ops.len(), |s| s.1);
        for o in owner.iter_mut().take(end).skip(start) {
            *o = Some(item);
        }
    }
    for (op, who) in ops.into_iter().zip(owner) {
        match (who, op) {
            (Some(item), Op::Put { addr, kind, value }) if item >= GUN_COUNT && which == "weapon" => {
                let (width, bits) = match (kind, value) {
                    (Kind::B, Value::Int(v)) => (1, v as u32),
                    (Kind::H, Value::Int(v)) => (2, v as u32),
                    (Kind::F, Value::Float(v)) => (4, v.to_bits()),
                    (_, Value::Int(v)) => (4, v as u32),
                    (_, Value::Float(v)) => (4, v.to_bits()),
                };
                new_weapons.get_mut(&item).unwrap().puts.push((addr, width, bits));
            }
            (Some(item), Op::MaxAmmo { value, .. }) if item >= GUN_COUNT => new_weapons.get_mut(&item).unwrap().max_ammo = Some(value as i32),
            (_, op) => kept.push(op),
        }
    }
    for w in new_weapons.values() {
        if w.copy_of < 0 {
            errors.push(format!("weapon {} has no `copy <weapon>` line", w.number));
        }
    }
    (kept, new_weapons.into_values().filter(|w| w.copy_of >= 0).collect(), errors)
}

/// Make the writes in the game's memory.
pub fn apply(ops: &[Op]) {
    use ts_rt::*;
    for op in ops {
        match *op {
            Op::Put { addr, kind, value } => match (kind, value) {
                (Kind::B, Value::Int(v)) => sb(addr, v as i32),
                (Kind::H, Value::Int(v)) => sh(addr, v as i32),
                (Kind::W, Value::Int(v)) => sw(addr, v as i32),
                (Kind::F, Value::Float(v)) => swc1(addr, v),
                _ => {}
            },
            Op::MaxAmmo { weapon, value } => {
                let kind = lw(GUNS + weapon * GUN_SIZE + 16);
                if (0..AMMO_KINDS).contains(&kind) {
                    sw(AMMOINFO + 8 * kind, value as i32);
                }
            }
        }
    }
}

/// Take the characters a mod adds (the numbers from the game's 80 on) out of a `characters.txt`: the text of the rest (the game's own characters, which `parse` reads), the added characters and
/// the complaints.  An added character has `copy <character>`, `name "text"`, `model_file <path in the pack>` (the model file the mod holds, such as `ob/chrs/chr90.raw`) and the fields of a character.
pub fn split_new_characters(text: &str) -> (String, Vec<ts_game::NewCharacter>, Vec<String>) {
    let fields = character_fields();
    let mut rest = String::new();
    let mut chars: Vec<ts_game::NewCharacter> = Vec::new();
    let mut errors = Vec::new();
    let mut current_new = false;
    for (n, line) in text.lines().enumerate() {
        let n = n + 1;
        let body = line.split('#').next().unwrap_or("").trim();
        let words: Vec<&str> = body.split_whitespace().collect();
        if words.first() == Some(&"character") {
            current_new = false;
            if let Some(v) = words.get(1).and_then(|w| parse_int(w)) {
                if (CHR_COUNT as i64..ts_game::MAX_CHARACTERS as i64).contains(&v) {
                    current_new = true;
                    chars.push(ts_game::NewCharacter { number: v as i32, copy_of: -1, name: String::new(), model_file: String::new(), puts: Vec::new() });
                    continue;
                }
            }
        }
        if !current_new || body.is_empty() {
            rest.push_str(line);
            rest.push('\n');
            continue;
        }
        let c = chars.last_mut().unwrap();
        let key = words[0];
        let arg = || words.get(1).copied();
        match key {
            "copy" => match arg().and_then(parse_int) {
                Some(v) if (0..c.number as i64).contains(&v) => c.copy_of = v as i32,
                _ => errors.push(format!("line {n}: `copy` needs the number of an earlier character")),
            },
            "name" => {
                let name = body.splitn(2, char::is_whitespace).nth(1).unwrap_or("").trim().trim_matches('"').to_string();
                if name.is_empty() {
                    errors.push(format!("line {n}: `name` needs a name"));
                } else {
                    c.name = name;
                }
            }
            "model_file" => match arg() {
                Some(p) if !p.contains("..") => c.model_file = p.trim_matches('"').trim_start_matches('/').to_string(),
                _ => errors.push(format!("line {n}: `model_file` needs a path in the pack, such as ob/chrs/chr90.raw")),
            },
            "raw" => {
                let (Some(off), Some(kind), Some(val)) = (words.get(1).and_then(|w| parse_int(w.trim_start_matches('+'))), words.get(2), words.get(3)) else {
                    errors.push(format!("line {n}: expected `raw +<offset> <b|h|w|f> <value>`"));
                    continue;
                };
                let (width, bits) = match (*kind, if *kind == "f" { val.parse::<f32>().ok().map(|f| f.to_bits() as i64) } else { parse_int(val) }) {
                    ("b", Some(v)) => (1, v as u32),
                    ("h", Some(v)) => (2, v as u32),
                    ("w", Some(v)) => (4, v as u32),
                    ("f", Some(v)) => (4, v as u32),
                    _ => {
                        errors.push(format!("line {n}: expected `raw +<offset> <b|h|w|f> <value>`"));
                        continue;
                    }
                };
                if (0..CHR_SIZE as i64).contains(&off) && off + width as i64 <= CHR_SIZE as i64 {
                    c.puts.push((off as i32, width, bits));
                } else {
                    errors.push(format!("line {n}: offset {off} is outside the entry ({CHR_SIZE} bytes)"));
                }
            }
            _ => match fields.iter().find(|f| f.name == key) {
                Some(f) => {
                    let value = if f.kind == Kind::F { arg().and_then(|w| w.parse::<f32>().ok()).filter(|v| (f.min..=f.max).contains(&(*v as f64))).map(|v| v.to_bits()) } else { arg().and_then(parse_int).filter(|v| (f.min..=f.max).contains(&(*v as f64))).map(|v| v as u32) };
                    match value {
                        Some(bits) => c.puts.push((f.at, match f.kind { Kind::B => 1, Kind::H => 2, _ => 4 }, bits)),
                        None => errors.push(format!("line {n}: `{key}` needs a number in its range")),
                    }
                }
                None => errors.push(format!("line {n}: unknown field `{key}` of an added character (copy, name, model_file, raw and the fields of a character)")),
            },
        }
    }
    for c in &chars {
        if c.copy_of < 0 {
            errors.push(format!("character {} has no `copy <character>` line", c.number));
        }
    }
    chars.retain(|c| c.copy_of >= 0);
    (rest, chars, errors)
}

/// The game modes by name: the number of the mode in the game's records of options.
fn mode_number(word: &str) -> Option<i32> {
    match word.to_ascii_lowercase().as_str() {
        "bagtag" | "bag_tag" | "1" => Some(1),
        "capturebag" | "capture_the_bag" | "ctb" | "2" => Some(2),
        "deathmatch" | "dm" | "3" => Some(3),
        "escort" | "4" => Some(4),
        "knockout" | "knock_out" | "ko" | "5" => Some(5),
        "laststand" | "last_stand" | "ls" | "6" => Some(6),
        _ => None,
    }
}

/// The options of a mode that a preset can set: the name, the place in the record of the game and the largest value (1 for a switch, written on or off).
const MODE_FIELDS: &[(&str, i32, i32)] = &[
    ("team_play", 0, 1),
    ("friendly_fire", 4, 1),
    ("radar", 8, 1),
    ("time_limit", 12, 999),
    ("kill_limit", 16, 999),
    ("score_limit", 20, 9999),
    ("score_method", 24, 15),
    ("one_shot_kill", 28, 1),
];

/// Read a `modes.txt`: `mode <name or number>` and then the options of that mode, and `name "text"` for its name in the menus.
pub fn parse_modes(text: &str) -> (Vec<ts_game::ModePreset>, Vec<ts_game::ModeVariant>, Vec<String>) {
    let mut presets: Vec<ts_game::ModePreset> = Vec::new();
    let mut variants: Vec<ts_game::ModeVariant> = Vec::new();
    // (the options that follow go to the last variant while one is open)
    let mut in_variant = false;
    let mut errors = Vec::new();
    let mut current: Option<usize> = None;
    for (n, line) in text.lines().enumerate() {
        let n = n + 1;
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let words: Vec<&str> = line.split_whitespace().collect();
        if words[0] == "mode" {
            current = match words.get(1).and_then(|w| mode_number(w)) {
                Some(kind) => {
                    in_variant = false;
                    presets.push(ts_game::ModePreset { kind, name: String::new(), fields: Vec::new() });
                    Some(presets.len() - 1)
                }
                None => {
                    errors.push(format!("line {n}: `mode` needs bagtag, capturebag, deathmatch, escort, knockout or laststand"));
                    None
                }
            };
            continue;
        }
        let Some(k) = current else {
            errors.push(format!("line {n}: `{line}` comes before any `mode <name>` line"));
            continue;
        };
        if words[0] == "variant" {
            let name = line.splitn(2, char::is_whitespace).nth(1).unwrap_or("").trim().trim_matches('"').to_string();
            if name.is_empty() {
                errors.push(format!("line {n}: `variant` needs a name"));
            } else {
                variants.push(ts_game::ModeVariant { kind: presets[k].kind, name: name.chars().take(20).collect(), fields: Vec::new(), rules: Vec::new() });
                in_variant = true;
            }
            continue;
        }
        if words[0] == "rule" {
            let rule = match words.get(1).copied() {
                Some("win_score") => Some(ts_game::Rule::WinScore),
                Some("win_score_seconds") => Some(ts_game::Rule::WinScoreSeconds),
                _ => None,
            };
            match (variants.last_mut().filter(|_| in_variant), rule, words.get(2).and_then(|w| parse_int(w))) {
                (None, _, _) => errors.push(format!("line {n}: a `rule` belongs to a `variant`")),
                (_, None, _) => errors.push(format!("line {n}: unknown rule (the rules are win_score, win_score_seconds)")),
                (Some(v), Some(r), Some(val)) if (1..=100000).contains(&val) => v.rules.push((r, val as i32)),
                _ => errors.push(format!("line {n}: a rule needs a number from 1 to 100000")),
            }
            continue;
        }
        if words[0] == "name" {
            let name = line.splitn(2, char::is_whitespace).nth(1).unwrap_or("").trim().trim_matches('"').to_string();
            if name.is_empty() {
                errors.push(format!("line {n}: `name` needs a name"));
            } else if in_variant {
                errors.push(format!("line {n}: a variant has its name on the `variant` line"));
            } else {
                presets[k].name = name;
            }
            continue;
        }
        let Some(&(_, at, max)) = MODE_FIELDS.iter().find(|f| f.0 == words[0]) else {
            let names = MODE_FIELDS.iter().map(|f| f.0).collect::<Vec<_>>().join(", ");
            errors.push(format!("line {n}: unknown option `{}` (the options are {names}, name)", words[0]));
            continue;
        };
        let value = match words.get(1).map(|w| w.to_ascii_lowercase()).as_deref() {
            Some("on") | Some("yes") | Some("true") if max == 1 => Some(1),
            Some("off") | Some("no") | Some("false") if max == 1 => Some(0),
            Some(w) => parse_int(w),
            None => None,
        };
        match value {
            Some(v) if (0..=max as i64).contains(&v) => match variants.last_mut().filter(|_| in_variant) {
                Some(var) => var.fields.push((at, v as i32)),
                None => presets[k].fields.push((at, v as i32)),
            },
            Some(v) => errors.push(format!("line {n}: {} {v} is out of range (0 to {max})", words[0])),
            None => errors.push(format!("line {n}: `{}` needs a number{}", words[0], if max == 1 { " or on / off" } else { "" })),
        }
    }
    (presets, variants, errors)
}

/// Read and apply the file `path` of the mod `mod_name`; the complaints go to the log.
pub fn load_file(mod_name: &str, which: &str, path: &Path) {
    let Ok(text) = std::fs::read_to_string(path) else { return };
    let (text, new_characters) = if which == "character" {
        let (rest, chars, errors) = split_new_characters(&text);
        for e in &errors {
            eprintln!("[mods] {mod_name}/characters.txt {e}");
        }
        (rest, chars)
    } else {
        (text, Vec::new())
    };
    for c in new_characters {
        ts_game::register_character(c);
    }
    if which == "mode" {
        let (presets, variants, errors) = parse_modes(&text);
        for e in &errors {
            eprintln!("[mods] {mod_name}/modes.txt {e}");
        }
        let (n, nv) = (presets.len(), variants.len());
        for p in presets {
            ts_game::register_mode_preset(p);
        }
        for v in variants {
            ts_game::register_mode_variant(v);
        }
        eprintln!("[mods] {mod_name}/modes.txt: {n} mode(s), {nv} variant(s){}", if errors.is_empty() { String::new() } else { format!(", {} line(s) left out", errors.len()) });
        return;
    }
    let (ops, added, errors) = parse(which, &text);
    let file = if which == "weapon" { "weapons.txt" } else { "characters.txt" };
    for e in &errors {
        eprintln!("[mods] {mod_name}/{file} {e}");
    }
    apply(&ops);
    let n_added = added.len();
    for w in added {
        ts_game::register_weapon(w);
    }
    eprintln!("[mods] {mod_name}/{file}: {} change(s), {n_added} added weapon(s){}", ops.len(), if errors.is_empty() { String::new() } else { format!(", {} line(s) left out", errors.len()) });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_weapons() {
        let (ops, _added, errors) = parse("weapon", "weapon 0\n clip 15\n max_ammo 250 # most\n raw +60 w 7\n");
        assert!(errors.is_empty(), "{errors:?}");
        assert_eq!(ops[0], Op::Put { addr: GUNS + 20, kind: Kind::W, value: Value::Int(15) });
        assert_eq!(ops[1], Op::MaxAmmo { weapon: 0, value: 250 });
        assert_eq!(ops[2], Op::Put { addr: GUNS + 60, kind: Kind::W, value: Value::Int(7) });
    }

    #[test]
    fn complains() {
        let (ops, _added, errors) = parse("weapon", "clip 5\nweapon 99\nweapon 1\n clip 100000\n colour 3\n raw +399 w 1\n");
        assert!(ops.is_empty());
        assert_eq!(errors.len(), 5, "{errors:?}");
        assert!(errors[0].starts_with("line 1"));
    }

    #[test]
    fn reads_shot_fields() {
        let (ops, _added, errors) = parse("weapon", "weapon 2
 spread 0.02
 alt_bullet_type 5
 view_x 0.3
");
        assert!(errors.is_empty(), "{errors:?}");
        assert_eq!(ops[0], Op::Put { addr: GUNS + 2 * 400 + 84, kind: Kind::F, value: Value::Float(0.02) });
        assert_eq!(ops[1], Op::Put { addr: GUNS + 2 * 400 + 188, kind: Kind::W, value: Value::Int(5) });
        assert_eq!(ops[2], Op::Put { addr: GUNS + 2 * 400 + 32, kind: Kind::F, value: Value::Float(0.3) });
    }

    #[test]
    fn adds_weapons() {
        let (ops, added, errors) = parse("weapon", "weapon 30
 copy 0
 name \"Super Pistol\"
 damage 0.9
 max_ammo 100
weapon 0
 clip 12
");
        assert!(errors.is_empty(), "{errors:?}");
        assert_eq!(ops.len(), 1, "{ops:?}");
        assert_eq!(added.len(), 1);
        assert_eq!(added[0].copy_of, 0);
        assert_eq!(added[0].name, "Super Pistol");
        assert_eq!(added[0].puts, vec![(72, 4, 0.9f32.to_bits())]);
        assert_eq!(added[0].max_ammo, Some(100));
        let (_, _, errors) = parse("weapon", "weapon 31
 damage 1
");
        assert_eq!(errors.len(), 1, "{errors:?}");
    }

    #[test]
    fn adds_characters() {
        let (rest, chars, errors) = split_new_characters("character 3
 scale 1.1
character 80
 copy 1
 name \"Robo\"
 model_file ob/chrs/chr90.raw
 scale 1.2
character 81
 scale 2
");
        assert_eq!(chars.len(), 1);
        assert_eq!(chars[0].number, 80);
        assert_eq!(chars[0].copy_of, 1);
        assert_eq!(chars[0].model_file, "ob/chrs/chr90.raw");
        assert_eq!(chars[0].puts, vec![(40, 4, 1.2f32.to_bits())]);
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert!(rest.contains("character 3") && rest.contains("scale 1.1") && !rest.contains("Robo"));
        let (ops, _, e2) = parse("character", &rest);
        assert!(e2.is_empty() && ops.len() == 1, "{e2:?}");
    }

    #[test]
    fn reads_modes() {
        let (p, vs, errors) = parse_modes("mode deathmatch
 name \"Quick Match\"
 kill_limit 10
 friendly_fire off
mode ko
 score_limit 5
");
        assert!(errors.is_empty(), "{errors:?}");
        assert_eq!(p.len(), 2);
        assert_eq!(p[0].kind, 3);
        assert_eq!(p[0].name, "Quick Match");
        assert_eq!(p[0].fields, vec![(16, 10), (4, 0)]);
        assert_eq!(p[1].kind, 5);
        assert!(vs.is_empty());
        let (p, vs, errors) = parse_modes("mode dm
 kill_limit 20
 variant \"Short\"
 kill_limit 5
 radar off
 variant \"Long\"
 kill_limit 50
");
        assert!(errors.is_empty(), "{errors:?}");
        assert_eq!(p[0].fields, vec![(16, 20)]);
        assert_eq!(vs.len(), 2);
        assert_eq!(vs[0].name, "Short");
        assert_eq!(vs[0].fields, vec![(16, 5), (8, 0)]);
        assert_eq!(vs[1].fields, vec![(16, 50)]);
        let (_, vs, errors) = parse_modes("mode bagtag
 variant \"First to 30\"
 rule win_score_seconds 30
 rule win_score 9
");
        assert!(errors.is_empty(), "{errors:?}");
        assert_eq!(vs[0].rules, vec![(ts_game::Rule::WinScoreSeconds, 30), (ts_game::Rule::WinScore, 9)]);
        let (_, _, errors) = parse_modes("mode bagtag
 rule win_score 5
 variant \"A\"
 rule chess 1
 rule win_score 0
");
        assert_eq!(errors.len(), 3, "{errors:?}");
        let (_, _, errors) = parse_modes("kill_limit 3
mode chess
mode ls
 time_limit 5000
 radar maybe
");
        assert_eq!(errors.len(), 4, "{errors:?}");
    }

    #[test]
    fn reads_characters() {
        let (ops, _added, errors) = parse("character", "character 3\n model 120\n scale 1.5\n");
        assert!(errors.is_empty(), "{errors:?}");
        assert_eq!(ops[0], Op::Put { addr: CHRS + 3 * 44 + 8, kind: Kind::W, value: Value::Int(120) });
        assert_eq!(ops[1], Op::Put { addr: CHRS + 3 * 44 + 40, kind: Kind::F, value: Value::Float(1.5) });
    }
}
