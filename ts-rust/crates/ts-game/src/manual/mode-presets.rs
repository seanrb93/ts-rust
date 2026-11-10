//! Presets of the game modes that mods give (`modes.txt`): the options a mode starts with and its name in the menus.
//!
//! The game keeps the options of each mode in a record of 32 bytes (`0x1fc2c98 + 32 * mode`, the mode being 1 BagTag, 2 Capture the Bag, 3 Deathmatch, 4 Escort,
//! 5 KnockOut, 6 Last Stand): team play, friendly fire, radar, time limit, kill limit, score limit, scoring method and one shot kills, one word each.  `gameoptionsMake` fills
//! them when a game starts and the menus keep what the player changes in them (`storeGameModeOptions`, `getGameModeOptions`), so a preset is written into the record
//! right after `gameoptionsMake`, and a player can still change the options on the Game Options page.
//!
//! The name of a mode is the text number in `gamemodedata` (12 bytes a mode); a mod's name is a new text of the port's tables (`customlevels::register_text`), so it needs
//! a build with `extended-tables`.

use crate::*;
use std::sync::Mutex;

const RECORDS: i32 = 0x1fc2c98;

/// What a mod says about a mode: the changes of its record (the place in it, the value) and its name.
#[derive(Clone, Debug)]
pub struct ModePreset {
    pub kind: i32,
    pub name: String,
    pub fields: Vec<(i32, i32)>,
}

static PRESETS: Mutex<Vec<ModePreset>> = Mutex::new(Vec::new());

/// A way to play a mode that a mod offers on the Options page of the arcade match (the row "Rules"): a name and the options it sets, the others staying as the mode has them.
#[derive(Clone, Debug)]
pub struct ModeVariant {
    pub kind: i32,
    pub name: String,
    pub fields: Vec<(i32, i32)>,
    /// rules the tick applies while this variant is played: (the rule, the value)
    pub rules: Vec<(Rule, i32)>,
    /// the options that the Options page leaves out for this variant (the addresses of their values, see `hidden_option`)
    pub hidden: Vec<i32>,
    /// the script of the variant (`script <file>` in `modes.txt`): the name of the file, and its text once the loader has read it
    pub script_file: String,
    pub script_source: String,
    /// the settings of the mod it comes from (`[settings]` of its `mod.ini`), which its script reads with `setting`
    pub script_settings: super::ModSettings,
    /// the options of its own that the Options page shows for it (`option` in `modes.txt`), which its script reads with `setting` too
    pub options: Vec<ModeOption>,
}

/// An option a variant adds to the Options page (`option <name> "<label>" <least> <most> <default> [<step>]` in `modes.txt`): a row of whole
/// numbers from `least` to `most`, `step` apart, which the variant's script reads with `setting(name, ...)`.  `value` is what it is set to now
/// (the default, or the setting of that name in the mod's `mod.ini`, until a player changes it on the page).
#[derive(Clone, Debug)]
pub struct ModeOption {
    pub name: String,
    pub label: String,
    pub least: i32,
    pub most: i32,
    pub step: i32,
    pub value: i32,
}

impl ModeOption {
    /// The value nearest `value` that the option can have (in its range, a whole number of steps from `least`).
    pub fn fit(&self, value: i32) -> i32 {
        let step = self.step.max(1);
        let k = ((value.clamp(self.least, self.most) - self.least) + step / 2) / step;
        (self.least + k * step).min(self.most)
    }
}

/// The option of the Options page that `hide` names in `modes.txt`: the address of its value in the game.
pub fn hidden_option(name: &str) -> Option<i32> {
    Some(match name {
        "team_play" => g::gv_teamplay,
        "team_starts" => g::gv_useteamstarts,
        "friendly_fire" => g::gv_friendfire,
        "radar" => g::gv_dispradar,
        "time_limit" => g::gv_timelimit,
        "kill_limit" => g::gv_killlimit,
        "score_method" => g::gv_scoremethod,
        "score_limit" => g::gv_scorelimit,
        "one_shot_kill" => g::gv_oneshotkill,
        "handicaps" => g::gv_handicaps,
        "start_with_gun" => g::gv_spawngun,
        _ => return None,
    })
}

/// A rule of a variant (`rule <name> <value>` in `modes.txt`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Rule {
    /// the level is completed when a team's score reaches the value (the score the mode keeps: the kills in Deathmatch, else the character's score field; BagTag counts ticks of 1/60 second)
    WinScore,
    /// the same for BagTag, with the value in seconds of holding the bag
    WinScoreSeconds,
    /// the level is completed when the best team's score is ahead of the second best by the value (and is above 0)
    WinLead,
}

static VARIANTS: Mutex<Vec<ModeVariant>> = Mutex::new(Vec::new());
/// the records as the game and the presets left them when the game started: what "Standard" stands for
static BASE: Mutex<[[i32; 8]; 7]> = Mutex::new([[0; 8]; 7]);

pub fn register_mode_variant(v: ModeVariant) {
    // (the list of the Rules row has room for 70 names with the strings: 30 for a mode is plenty)
    let mut all = VARIANTS.lock().unwrap();
    if (1..=6).contains(&v.kind) && all.iter().filter(|o| o.kind == v.kind).count() < 30 {
        all.push(v);
    } else {
        eprintln!("[mods] variant {:?}: left out (at most 30 variants for a mode)", v.name);
    }
}

/// The rules of the variant that is being played, if the mode is the one it was chosen for.
pub fn active_rules(mode: i32) -> Vec<(Rule, i32)> {
    let choice = super::arcadeopts::variant_choice(mode);
    if choice == 0 {
        return Vec::new();
    }
    VARIANTS.lock().unwrap().iter().filter(|v| v.kind == mode).nth(choice - 1).map_or(Vec::new(), |v| v.rules.clone())
}

/// Every variant of every mod, in the order the mods gave them: (its mode, its number among the variants of that mode (1 and on), its name).
pub fn all_variants() -> Vec<(i32, usize, String)> {
    let all = VARIANTS.lock().unwrap();
    all.iter().enumerate().map(|(i, v)| (v.kind, all[..=i].iter().filter(|o| o.kind == v.kind).count(), v.name.clone())).collect()
}

/// The options the Options page leaves out for the variant that is being played (the addresses of their values).
pub fn active_hidden(mode: i32) -> Vec<i32> {
    let choice = super::arcadeopts::variant_choice(mode);
    if choice == 0 {
        return Vec::new();
    }
    VARIANTS.lock().unwrap().iter().filter(|v| v.kind == mode).nth(choice - 1).map_or(Vec::new(), |v| v.hidden.clone())
}

/// The script of the variant that is being played: its file name, its text and the settings of its mod, with the values of the variant's own
/// options as the Options page left them in place of the settings of the same names.
pub fn active_script(mode: i32) -> Option<(String, String, super::ModSettings)> {
    let choice = super::arcadeopts::variant_choice(mode);
    if choice == 0 {
        return None;
    }
    super::arcademodes::option_rows_keep();
    let all = VARIANTS.lock().unwrap();
    let v = all.iter().filter(|v| v.kind == mode).nth(choice - 1)?;
    if v.script_source.is_empty() {
        return None;
    }
    let mut settings = v.script_settings.clone();
    for o in &v.options {
        let name = o.name.to_ascii_lowercase();
        settings.values.retain(|(k, _)| *k != name);
        settings.values.insert(0, (name, o.value.to_string()));
    }
    Some((v.script_file.clone(), v.script_source.clone(), settings))
}

/// The options of the variant `choice` (1 and on) of the mode `kind`.
pub fn variant_options(kind: i32, choice: usize) -> Vec<ModeOption> {
    if choice == 0 {
        return Vec::new();
    }
    VARIANTS.lock().unwrap().iter().filter(|v| v.kind == kind).nth(choice - 1).map_or(Vec::new(), |v| v.options.clone())
}

/// The name an option is kept under in the settings: the variant's name and the option's, in lower case with `_` for anything but letters and digits
/// (`elimination.lives`).
pub fn mode_option_key(variant: &str, option: &str) -> String {
    let clean = |s: &str| s.chars().map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '_' }).collect::<String>();
    format!("{}.{}", clean(variant), clean(option))
}

/// The values of options kept from the last time (`<mode>.<option>`, the value) are set, where a variant of a mod has that option and the value is one it can
/// have (called once the mods are read, before the menus).
pub fn mode_options_restore(saved: &[(String, i32)]) {
    let mut all = VARIANTS.lock().unwrap();
    for v in all.iter_mut() {
        let name = v.name.clone();
        for o in v.options.iter_mut() {
            let key = mode_option_key(&name, &o.name);
            if let Some(&(_, value)) = saved.iter().find(|(k, _)| *k == key) {
                if o.fit(value) == value {
                    o.value = value;
                }
            }
        }
    }
}

/// A player changed the option `index` of the variant `choice` of the mode `kind` on the Options page: it is set, and kept for the next start when it changed.
pub fn variant_option_choose(kind: i32, choice: usize, index: usize, value: i32) {
    if choice == 0 {
        return;
    }
    let changed = {
        let mut all = VARIANTS.lock().unwrap();
        let Some(v) = all.iter_mut().filter(|v| v.kind == kind).nth(choice - 1) else { return };
        let name = v.name.clone();
        let Some(o) = v.options.get_mut(index) else { return };
        let value = o.fit(value);
        (o.value != value).then(|| {
            o.value = value;
            (mode_option_key(&name, &o.name), value)
        })
    };
    if let (Some((key, value)), Some(p)) = (changed, super::avmenu::provider()) {
        p.set_mode_option(&key, value);
    }
}

/// A player set the option `index` of the variant `choice` of the mode `kind` to `value` (on the Options page).
pub fn variant_option_set(kind: i32, choice: usize, index: usize, value: i32) {
    if choice == 0 {
        return;
    }
    if let Some(v) = VARIANTS.lock().unwrap().iter_mut().filter(|v| v.kind == kind).nth(choice - 1) {
        if let Some(o) = v.options.get_mut(index) {
            o.value = o.fit(value);
        }
    }
}

/// The names of the variants of a mode.
pub fn variants_of(kind: i32) -> Vec<String> {
    VARIANTS.lock().unwrap().iter().filter(|v| v.kind == kind).map(|v| v.name.clone()).collect()
}

/// Set the options of the menus for the variant `index` of the mode `kind` (0: the mode as it starts, 1 and on: the variants in the order of the mods).
pub fn apply_variant(kind: i32, index: usize) {
    if !(1..=6).contains(&kind) {
        return;
    }
    let mut words = BASE.lock().unwrap()[kind as usize];
    if index > 0 {
        if let Some(v) = VARIANTS.lock().unwrap().iter().filter(|v| v.kind == kind).nth(index - 1) {
            for &(at, value) in &v.fields {
                words[(at / 4) as usize] = value;
            }
        }
    }
    // (the order of `getGameModeOptions`)
    let targets = [g::gv_teamplay, g::gv_friendfire, g::gv_dispradar, g::gv_timelimit, g::gv_killlimit, g::gv_scorelimit, g::gv_scoremethod, g::gv_oneshotkill];
    for (t, w) in targets.iter().zip(words) {
        gvSetValue(*t, w);
    }
}

/// A mod gives a preset (before the game starts); a later one wins for the fields it sets.
pub fn register_mode_preset(p: ModePreset) {
    if !(1..=6).contains(&p.kind) {
        return;
    }
    if !p.name.is_empty() {
        #[cfg(feature = "extended-tables")]
        {
            let text = super::customlevels::register_text(&p.name);
            if text >= 0 {
                sh(g::gamemodedata + 12 * p.kind, text);
            }
        }
        #[cfg(not(feature = "extended-tables"))]
        eprintln!("[mods] the name of mode {} needs a build with extended-tables", p.kind);
    }
    PRESETS.lock().unwrap().push(p);
}

/// Write the presets into the records of the options (`gameoptionsMake` has just filled them).
pub fn mode_presets_apply() {
    let mut presets = PRESETS.lock().unwrap().clone();
    presets.retain(|p| !p.fields.is_empty());
    for p in presets.iter() {
        let record = RECORDS + (p.kind << 5);
        for &(at, value) in &p.fields {
            sw(record + at, value);
        }
        if std::env::var_os("TS_COMBOLOG").is_some() {
            eprintln!("[mode] {} record: {:?}", p.kind, (0..8).map(|i| lw(record + 4 * i)).collect::<Vec<_>>());
        }
    }
    mode_base_save();
}

/// Remember the records as the game starts with them (after the presets), for "Standard" (`gameoptionsMake` calls this through `mode_presets_x`).
pub fn mode_base_save() {
    let mut base = BASE.lock().unwrap();
    for kind in 1..=6usize {
        for i in 0..8 {
            base[kind][i] = lw(RECORDS + ((kind as i32) << 5) + 4 * i as i32);
        }
    }
}

/// The rules of the variant that is being played, applied after the mode's own tick (`game-status-and-level-tick.rs` calls this through `custom_rules_x`).
pub fn custom_rules_tick() {
    // (a level that is completed is not completed again: that would open the menus again)
    if lw(g::gamestate) == 4 {
        return;
    }
    let mode = lw(g::curgameparms + 72);
    if std::env::var_os("TS_COMBOLOG").is_some() && ts_sdk::frame_count() % 600 == 1 {
        eprintln!("[rules] frame {} mode {mode}", ts_sdk::frame_count());
    }
    // (the script of the variant)
    #[cfg(feature = "extensions")]
    script_tick();
    for (rule, value) in active_rules(mode) {
        if rule == Rule::WinLead {
            let mut scores: Vec<i32> = (1..=5).map(teamGetScore).collect();
            scores.sort_unstable_by(|a, b| b.cmp(a));
            if scores[0] > 0 && scores[0].saturating_sub(scores[1]) >= value {
                if std::env::var_os("TS_COMBOLOG").is_some() {
                    eprintln!("[rules] {rule:?} {value} reached in mode {mode}");
                }
                gameLevelCompleted();
                return;
            }
            continue;
        }
        let goal = match rule {
            Rule::WinScore => value,
            Rule::WinScoreSeconds => value.saturating_mul(60),
            Rule::WinLead => continue,
        };
        // (a test aid: TS_RULEGOAL=-1 makes the goal reached at once)
        let goal = std::env::var("TS_RULEGOAL").ok().and_then(|v| v.parse().ok()).unwrap_or(goal);
        if goal == 0 {
            continue;
        }
        if (1..=5).any(|team| teamGetScore(team) >= goal) {
            if std::env::var_os("TS_COMBOLOG").is_some() {
                eprintln!("[rules] {rule:?} {value} reached in mode {mode}");
            }
            gameLevelCompleted();
            return;
        }
    }
}
