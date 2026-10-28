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
pub const CHR_COUNT: i32 = 96;

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
pub fn parse(which: &str, text: &str) -> (Vec<Op>, Vec<String>) {
    let (fields, base, size, count): (Vec<Field>, i32, i32, i32) = if which == "weapon" { (weapon_fields(), GUNS, GUN_SIZE, GUN_COUNT) } else { (character_fields(), CHRS, CHR_SIZE, CHR_COUNT) };
    let mut ops = Vec::new();
    let mut errors = Vec::new();
    let mut current: Option<i32> = None;
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
            current = match words.get(1).and_then(|w| parse_int(w)) {
                Some(v) if (0..count as i64).contains(&v) => Some(v as i32),
                Some(v) => {
                    errors.push(format!("line {n}: {which} {v} does not exist (0 to {})", count - 1));
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
        let entry = base + item * size;
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
    (ops, errors)
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

/// Read and apply the file `path` of the mod `mod_name`; the complaints go to the log.
pub fn load_file(mod_name: &str, which: &str, path: &Path) {
    let Ok(text) = std::fs::read_to_string(path) else { return };
    let (ops, errors) = parse(which, &text);
    let file = if which == "weapon" { "weapons.txt" } else { "characters.txt" };
    for e in &errors {
        eprintln!("[mods] {mod_name}/{file} {e}");
    }
    apply(&ops);
    eprintln!("[mods] {mod_name}/{file}: {} change(s){}", ops.len(), if errors.is_empty() { String::new() } else { format!(", {} line(s) left out", errors.len()) });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_weapons() {
        let (ops, errors) = parse("weapon", "weapon 0\n clip 15\n max_ammo 250 # most\n raw +60 w 7\n");
        assert!(errors.is_empty(), "{errors:?}");
        assert_eq!(ops[0], Op::Put { addr: GUNS + 20, kind: Kind::W, value: Value::Int(15) });
        assert_eq!(ops[1], Op::MaxAmmo { weapon: 0, value: 250 });
        assert_eq!(ops[2], Op::Put { addr: GUNS + 60, kind: Kind::W, value: Value::Int(7) });
    }

    #[test]
    fn complains() {
        let (ops, errors) = parse("weapon", "clip 5\nweapon 99\nweapon 1\n clip 100000\n colour 3\n raw +399 w 1\n");
        assert!(ops.is_empty());
        assert_eq!(errors.len(), 5, "{errors:?}");
        assert!(errors[0].starts_with("line 1"));
    }

    #[test]
    fn reads_shot_fields() {
        let (ops, errors) = parse("weapon", "weapon 2
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
    fn reads_characters() {
        let (ops, errors) = parse("character", "character 3\n model 120\n scale 1.5\n");
        assert!(errors.is_empty(), "{errors:?}");
        assert_eq!(ops[0], Op::Put { addr: CHRS + 3 * 44 + 8, kind: Kind::W, value: Value::Int(120) });
        assert_eq!(ops[1], Op::Put { addr: CHRS + 3 * 44 + 40, kind: Kind::F, value: Value::Float(1.5) });
    }
}
