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
    name: &'static str,
    at: i32,
    kind: Kind,
    min: i64,
    max: i64,
}

const WEAPON_FIELDS: &[Field] = &[
    Field { name: "ammo_kind", at: 16, kind: Kind::W, min: 0, max: AMMO_KINDS as i64 - 1 },
    Field { name: "clip", at: 20, kind: Kind::W, min: 0, max: 9999 },
    Field { name: "flags", at: 28, kind: Kind::W, min: 0, max: 0xffff },
    Field { name: "bullet_type", at: 52, kind: Kind::W, min: 0, max: 255 },
];

const CHARACTER_FIELDS: &[Field] = &[
    Field { name: "name_text", at: 0, kind: Kind::H, min: 0, max: 0xffff },
    Field { name: "name_text_2", at: 2, kind: Kind::H, min: 0, max: 0xffff },
    Field { name: "model", at: 8, kind: Kind::W, min: 0, max: 1100 },
    Field { name: "front_model", at: 12, kind: Kind::W, min: 0, max: 1100 },
    Field { name: "flags", at: 16, kind: Kind::W, min: -0x8000_0000, max: 0x7fff_ffff },
];

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
    let (fields, base, size, count): (&[Field], i32, i32, i32) = if which == "weapon" { (WEAPON_FIELDS, GUNS, GUN_SIZE, GUN_COUNT) } else { (CHARACTER_FIELDS, CHRS, CHR_SIZE, CHR_COUNT) };
    let mut ops = Vec::new();
    let mut errors = Vec::new();
    let mut current: Option<i32> = None;
    let names = |f: &[Field]| f.iter().map(|f| f.name).collect::<Vec<_>>().join(", ");
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
        if which == "character" && key == "scale" {
            match words.get(1).and_then(|w| w.parse::<f32>().ok()) {
                Some(v) if v > 0.0 && v < 100.0 => ops.push(Op::Put { addr: entry + 40, kind: Kind::F, value: Value::Float(v) }),
                _ => errors.push(format!("line {n}: `scale` needs a number above 0")),
            }
            continue;
        }
        let Some(field) = fields.iter().find(|f| f.name == key) else {
            let extra = if which == "weapon" { ", max_ammo" } else { ", scale" };
            errors.push(format!("line {n}: unknown field `{key}` (the fields are {}{extra}, raw)", names(fields)));
            continue;
        };
        match words.get(1).and_then(|w| parse_int(w)) {
            Some(v) if (field.min..=field.max).contains(&v) => ops.push(Op::Put { addr: entry + field.at, kind: field.kind, value: Value::Int(v) }),
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
    fn reads_characters() {
        let (ops, errors) = parse("character", "character 3\n model 120\n scale 1.5\n");
        assert!(errors.is_empty(), "{errors:?}");
        assert_eq!(ops[0], Op::Put { addr: CHRS + 3 * 44 + 8, kind: Kind::W, value: Value::Int(120) });
        assert_eq!(ops[1], Op::Put { addr: CHRS + 3 * 44 + 40, kind: Kind::F, value: Value::Float(1.5) });
    }
}
