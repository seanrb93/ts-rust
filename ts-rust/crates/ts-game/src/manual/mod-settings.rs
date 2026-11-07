//! The settings of a mod: the `[settings]` section of its `mod.ini`, of named values a player can change without knowing the game's tables.
//!
//! ```text
//! # mod.ini
//! name = Weapon Balance
//! enabled = true
//!
//! [settings]
//! # How strong explosions are: 1 is the game's, 0.5 half as strong.
//! explosion_strength = 0.8
//! # Balance the shotguns too (yes or no).
//! balance_shotguns = yes
//! ```
//!
//! The mod's data files (`weapons.txt`, `characters.txt`, `modes.txt`, `patches.txt`) use them:
//!
//! ```text
//! weapon 8
//!   blast {explosion_strength}              # a value in braces: a setting, a number, or sums of them with + - * / and brackets
//!   alt_blast {explosion_strength * 0.6}
//! if balance_shotguns                       # the lines up to `else` or `end` are read only when the setting is yes (or a number not 0)
//! weapon 24
//!   damage 0.19
//! end
//! ```
//!
//! `if` takes a setting (`if name`, `if not name`) or a comparison of it (`if name = word`, `!=`, `<`, `<=`, `>`, `>=`), and the blocks can be inside each other.
//! A script reads a setting with `setting("name")` (`docs/scripting.md`).  Lines that are left out stay as blank lines, so the line numbers of complaints are the file's.
//! A `settings.ini` in the mod's folder, as the first mods with settings had, is read too (the `mod.ini` wins where both have a setting).

use std::path::Path;

/// The settings of one mod, in the order of the file (the names are kept in lower case).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModSettings {
    pub values: Vec<(String, String)>,
}

/// A value as a number: yes / on / true are 1, no / off / false are 0.
pub fn setting_number(value: &str) -> Option<f64> {
    match value.trim().to_ascii_lowercase().as_str() {
        "yes" | "on" | "true" => Some(1.0),
        "no" | "off" | "false" => Some(0.0),
        v => v.parse::<f64>().ok().filter(|f| f.is_finite()),
    }
}

impl ModSettings {
    /// The value of the setting `name`, as it is written.
    pub fn get(&self, name: &str) -> Option<&str> {
        let name = name.to_ascii_lowercase();
        self.values.iter().find(|(k, _)| *k == name).map(|(_, v)| v.as_str())
    }

    /// Read the text of a `settings.ini`: `name = value` lines, `#` starting a note.  The complaints have line numbers.
    pub fn parse(text: &str) -> (ModSettings, Vec<String>) {
        Self::parse_lines(text, false)
    }

    /// Read the `[settings]` section of the text of a `mod.ini` (the lines from `[settings]` to the next `[...]`).
    pub fn parse_mod_ini(text: &str) -> (ModSettings, Vec<String>) {
        Self::parse_lines(text, true)
    }

    fn parse_lines(text: &str, in_section_only: bool) -> (ModSettings, Vec<String>) {
        let mut s = ModSettings::default();
        let mut errors = Vec::new();
        let mut reading = !in_section_only;
        for (n, line) in text.lines().enumerate() {
            let line = line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            if line.starts_with('[') {
                reading = !in_section_only || line.trim_matches(|c| c == '[' || c == ']').trim().eq_ignore_ascii_case("settings");
                continue;
            }
            if !reading {
                continue;
            }
            let Some((k, v)) = line.split_once('=') else {
                errors.push(format!("line {}: expected `name = value`", n + 1));
                continue;
            };
            let k = k.trim().to_ascii_lowercase();
            let v = v.trim().trim_matches('"').to_string();
            if k.is_empty() || !k.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') || k.starts_with(|c: char| c.is_ascii_digit()) {
                errors.push(format!("line {}: `{k}` is not a name (letters, digits and _)", n + 1));
                continue;
            }
            match s.values.iter_mut().find(|(name, _)| *name == k) {
                Some(e) => e.1 = v,
                None => s.values.push((k, v)),
            }
        }
        (s, errors)
    }

    /// The settings of the mod folder `dir`: the `[settings]` of its `mod.ini`, and a `settings.ini` if it has one (none: no settings).  The complaints
    /// start with the name of the file.
    pub fn read(dir: &Path) -> (ModSettings, Vec<String>) {
        let mut s = ModSettings::default();
        let mut errors = Vec::new();
        if let Ok(text) = std::fs::read_to_string(dir.join("settings.ini")) {
            let (old, e) = ModSettings::parse(&text);
            s = old;
            errors.extend(e.into_iter().map(|e| format!("settings.ini {e}")));
        }
        if let Ok(text) = std::fs::read_to_string(dir.join("mod.ini")) {
            let (new, e) = ModSettings::parse_mod_ini(&text);
            for (k, v) in new.values {
                match s.values.iter_mut().find(|(name, _)| *name == k) {
                    Some(e) => e.1 = v,
                    None => s.values.push((k, v)),
                }
            }
            errors.extend(e.into_iter().map(|e| format!("mod.ini {e}")));
        }
        (s, errors)
    }

    /// Is a setting yes (a number not 0, or a word other than no / off / false)?
    fn truthy(&self, name: &str) -> Result<bool, String> {
        let v = self.get(name).ok_or_else(|| format!("the setting `{name}` is not in [settings] of mod.ini"))?;
        Ok(match setting_number(v) {
            Some(f) => f != 0.0,
            None => !v.is_empty(),
        })
    }

    /// The condition of an `if` line.
    fn condition(&self, words: &str) -> Result<bool, String> {
        let words = words.trim();
        if let Some(rest) = words.strip_prefix("not ") {
            return self.condition(rest).map(|b| !b);
        }
        for op in ["!=", "<=", ">=", "=", "<", ">"] {
            if let Some((name, want)) = words.split_once(op) {
                let (name, want) = (name.trim(), want.trim().trim_matches('"'));
                let have = self.get(name).ok_or_else(|| format!("the setting `{name}` is not in [settings] of mod.ini"))?;
                return match op {
                    "=" | "!=" => {
                        let same = match (setting_number(have), setting_number(want)) {
                            (Some(a), Some(b)) => a == b,
                            _ => have.eq_ignore_ascii_case(want),
                        };
                        Ok(same == (op == "="))
                    }
                    _ => {
                        let (Some(a), Some(b)) = (setting_number(have), self.number_or_setting(want)) else {
                            return Err(format!("`{op}` needs numbers (`{name}` is {have})"));
                        };
                        Ok(match op {
                            "<" => a < b,
                            "<=" => a <= b,
                            ">" => a > b,
                            _ => a >= b,
                        })
                    }
                };
            }
        }
        if words.is_empty() {
            return Err("`if` needs a setting".into());
        }
        self.truthy(words)
    }

    fn number_or_setting(&self, w: &str) -> Option<f64> {
        setting_number(w).or_else(|| self.get(w).and_then(setting_number))
    }

    /// The value of a `{...}`: a setting that is a word stays a word; anything else is worked out as a number.
    fn value_of(&self, expr: &str) -> Result<String, String> {
        let e = expr.trim();
        if let Some(v) = self.get(e) {
            if setting_number(v).is_none() {
                return Ok(v.to_string());
            }
        }
        let tokens = tokens(e)?;
        let mut at = 0;
        let v = self.sum(&tokens, &mut at)?;
        if at != tokens.len() {
            return Err(format!("`{{{e}}}` is not a sum that can be worked out"));
        }
        if !v.is_finite() {
            return Err(format!("`{{{e}}}` divides by 0"));
        }
        Ok(number_text(v))
    }

    fn sum(&self, t: &[Tok], at: &mut usize) -> Result<f64, String> {
        let mut v = self.product(t, at)?;
        while let Some(Tok::Op(c @ ('+' | '-'))) = t.get(*at) {
            *at += 1;
            let r = self.product(t, at)?;
            v = if *c == '+' { v + r } else { v - r };
        }
        Ok(v)
    }

    fn product(&self, t: &[Tok], at: &mut usize) -> Result<f64, String> {
        let mut v = self.unit(t, at)?;
        while let Some(Tok::Op(c @ ('*' | '/'))) = t.get(*at) {
            *at += 1;
            let r = self.unit(t, at)?;
            v = if *c == '*' { v * r } else { v / r };
        }
        Ok(v)
    }

    fn unit(&self, t: &[Tok], at: &mut usize) -> Result<f64, String> {
        let tok = t.get(*at).cloned().ok_or("a sum ends too soon")?;
        *at += 1;
        match tok {
            Tok::Num(v) => Ok(v),
            Tok::Name(n) => {
                let v = self.get(&n).ok_or_else(|| format!("the setting `{n}` is not in [settings] of mod.ini"))?;
                setting_number(v).ok_or_else(|| format!("the setting `{n}` is {v}, not a number"))
            }
            Tok::Op('-') => Ok(-self.unit(t, at)?),
            Tok::Op('(') => {
                let v = self.sum(t, at)?;
                match t.get(*at) {
                    Some(Tok::Op(')')) => {
                        *at += 1;
                        Ok(v)
                    }
                    _ => Err("a bracket is not closed".into()),
                }
            }
            Tok::Op(c) => Err(format!("`{c}` is out of place")),
        }
    }
}

#[derive(Clone, Debug)]
enum Tok {
    Num(f64),
    Name(String),
    Op(char),
}

fn tokens(e: &str) -> Result<Vec<Tok>, String> {
    let mut out = Vec::new();
    let c: Vec<char> = e.chars().collect();
    let mut i = 0;
    while i < c.len() {
        let ch = c[i];
        if ch.is_whitespace() {
            i += 1;
        } else if "+-*/()".contains(ch) {
            out.push(Tok::Op(ch));
            i += 1;
        } else if ch.is_ascii_digit() || ch == '.' {
            let start = i;
            while i < c.len() && (c[i].is_ascii_digit() || c[i] == '.') {
                i += 1;
            }
            let s: String = c[start..i].iter().collect();
            out.push(Tok::Num(s.parse().map_err(|_| format!("`{s}` is not a number"))?));
        } else if ch.is_ascii_alphabetic() || ch == '_' {
            let start = i;
            while i < c.len() && (c[i].is_ascii_alphanumeric() || c[i] == '_') {
                i += 1;
            }
            out.push(Tok::Name(c[start..i].iter().collect()));
        } else {
            return Err(format!("`{ch}` cannot be in a sum"));
        }
    }
    Ok(out)
}

/// A number as it goes into a data file: a whole number without a point (so it can be a count), else up to 6 places.
fn number_text(v: f64) -> String {
    if (v - v.round()).abs() < 1e-9 && v.abs() < 1e15 {
        format!("{}", v.round() as i64)
    } else {
        let s = format!("{v:.6}");
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

/// The text of a data file with the settings put in: `if` / `else` / `end` blocks chosen and `{...}` worked out.  A line that cannot be worked out
/// is left out (and told in the complaints, with its line number), so a wrong setting never writes a wrong value.
pub fn expand_settings(text: &str, s: &ModSettings) -> (String, Vec<String>) {
    let mut out = String::with_capacity(text.len());
    let mut errors = Vec::new();
    // (for each open `if`: is its block being read, has a branch been taken, was the block around it being read)
    let mut stack: Vec<(bool, bool)> = Vec::new();
    let reading = |stack: &[(bool, bool)]| stack.iter().all(|b| b.0);
    for (n, line) in text.lines().enumerate() {
        let n = n + 1;
        let (body, note) = match line.find('#') {
            Some(k) => (&line[..k], &line[k..]),
            None => (line, ""),
        };
        let trimmed = body.trim();
        let first = trimmed.split_whitespace().next().unwrap_or("").to_ascii_lowercase();
        match first.as_str() {
            "if" => {
                let outer = reading(&stack);
                let yes = if outer {
                    s.condition(&trimmed[2..]).unwrap_or_else(|e| {
                        errors.push(format!("line {n}: {e}"));
                        false
                    })
                } else {
                    false
                };
                stack.push((yes, yes));
                out.push('\n');
                continue;
            }
            "else" => {
                match stack.last_mut() {
                    Some(b) => {
                        b.0 = !b.1;
                        b.1 = true;
                    }
                    None => errors.push(format!("line {n}: `else` with no `if`")),
                }
                out.push('\n');
                continue;
            }
            "end" => {
                if stack.pop().is_none() {
                    errors.push(format!("line {n}: `end` with no `if`"));
                }
                out.push('\n');
                continue;
            }
            _ => {}
        }
        if !reading(&stack) {
            out.push('\n');
            continue;
        }
        if !body.contains('{') {
            out.push_str(line);
            out.push('\n');
            continue;
        }
        // the values in braces
        let mut done = String::new();
        let mut rest = body;
        let mut bad = None;
        while let Some(k) = rest.find('{') {
            done.push_str(&rest[..k]);
            let Some(end) = rest[k..].find('}') else {
                bad = Some("a `{` is not closed".to_string());
                break;
            };
            match s.value_of(&rest[k + 1..k + end]) {
                Ok(v) => done.push_str(&v),
                Err(e) => {
                    bad = Some(e);
                    break;
                }
            }
            rest = &rest[k + end + 1..];
        }
        match bad {
            Some(e) => {
                errors.push(format!("line {n}: {e}"));
                out.push('\n');
            }
            None => {
                done.push_str(rest);
                done.push_str(note);
                out.push_str(&done);
                out.push('\n');
            }
        }
    }
    if !stack.is_empty() {
        errors.push(format!("{} `if` with no `end`", stack.len()));
    }
    (out, errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings() -> ModSettings {
        ModSettings::parse("# a note\nexplosion_strength = 0.8\nshotguns = yes\nstyle = classic\nclip = 12\n").0
    }

    #[test]
    fn reads_the_file() {
        let (s, e) = ModSettings::parse("a = 1\nB = two # note\nbad line\n9x = 1\n");
        assert_eq!(s.get("a"), Some("1"));
        assert_eq!(s.get("b"), Some("two"));
        assert_eq!(e.len(), 2);
    }

    #[test]
    fn reads_the_section_of_mod_ini() {
        let (s, e) = ModSettings::parse_mod_ini("name = A Mod\nenabled = true\nscript = a.rhai\n\n[settings]\n# a note\nstrength = 2\n[other]\nx = 1\n");
        assert!(e.is_empty(), "{e:?}");
        assert_eq!(s.values, vec![("strength".to_string(), "2".to_string())]);
    }

    #[test]
    fn works_out_values() {
        let s = settings();
        let (t, e) = expand_settings("blast {explosion_strength}\nalt_blast {explosion_strength * 0.5}  # half\nclip {clip + 3}\nx {-(1 + 2) / 4}\nname {style}\n", &s);
        assert!(e.is_empty(), "{e:?}");
        assert_eq!(t, "blast 0.8\nalt_blast 0.4  # half\nclip 15\nx -0.75\nname classic\n");
    }

    #[test]
    fn chooses_blocks() {
        let s = settings();
        let text = "a\nif shotguns\nb\nif style = modern\nc\nelse\nd\nend\nend\nif not shotguns\ne\nend\nif clip >= 12\nf\nend\n";
        let (t, e) = expand_settings(text, &s);
        assert!(e.is_empty(), "{e:?}");
        let kept: Vec<&str> = t.lines().filter(|l| !l.is_empty()).collect();
        assert_eq!(kept, ["a", "b", "d", "f"]);
        assert_eq!(t.lines().count(), text.lines().count());
    }

    #[test]
    fn leaves_out_bad_lines() {
        let s = settings();
        let (t, e) = expand_settings("damage {missing * 2}\nclip {style + 1}\nok 1\nif nothing\nx\nend\n", &s);
        assert_eq!(t.lines().filter(|l| !l.is_empty()).collect::<Vec<_>>(), ["ok 1"]);
        assert_eq!(e.len(), 3, "{e:?}");
        assert!(e[0].starts_with("line 1:"));
    }
}
