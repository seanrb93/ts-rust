//! Separate volumes for kinds of sound effects (Audio Options, second page): weapons, impacts, voices, ambience and the rest.
//!
//! The game plays every effect from one sample memory and has a single Sound volume.  The mixer (`ts-audio`) asks, when a voice
//! starts, which group the sample at its address belongs to; the answer is found from the game's table of sounds (`g::sounds`: 36
//! bytes per sound, the file's name at +4 and the sample's address at +24), by the words in the file's name (the sounds are named
//! after what they are: `bullet_hit_metal01_22`, `fem_helen22_die05`, `mall_roomtone22_01_blank`).

use crate::*;
use std::collections::HashMap;
use std::sync::Mutex;

pub const WEAPONS: u8 = 0;
pub const IMPACTS: u8 = 1;
pub const VOICES: u8 = 2;
pub const AMBIENCE: u8 = 3;
pub const EFFECTS: u8 = 4;

/// number of sounds in the game's table
const SOUNDS: i32 = 0xaadc / 36;

fn has(name: &str, words: &[&str]) -> bool {
    words.iter().any(|w| name.contains(w))
}

/// The group of a sound, from the name of its file.
pub fn group_of(path: &str) -> u8 {
    let name = path.rsplit(['/', ':']).next().unwrap_or(path).to_ascii_lowercase();
    let name = name.trim_end_matches(".vag");
    if has(name, &["bullet_hit", "bullet_impact", "ricochet", "spacerico", "bodyfall", "bonecrack", "bounce", "brickhit", "dyingfall", "smashglass"]) {
        IMPACTS
    } else if name.starts_with("fem_")
        || name.starts_with("male")
        || name.starts_with("female")
        || has(name, &["die", "flinch", "announce", "tannoy", "guard_lee", "splitter_roar", "pillar22_b", "mummie_gn", "jenny", "laugh", "_sg16", "avi_", "cyborg_", "red_alien", "zombie2_", "greenie", "floaty", "robofish_lr", "mka22", "time22", "bonus", "mutant_insect", "scream", "zombie_gn", "zombie_les", "zombie_lr", "zombie_fem"])
    {
        VOICES
    } else if has(name, &["pickup", "getgun", "drop_gun", "bag_", "respawn", "level_", "countdown", "timeup", "headsup", "map0", "map1", "map2", "front/", "blank01", "placeholder", "timed_mine_count"]) {
        EFFECTS
    } else if has(
        name,
        &["roomtone", "_hum", "fan", "wind", "waterfall", "lorry", "plane", "jet", "clock", "chime", "gong", "neonsign", "pub_sign", "door", "chain_fence", "camera_alarm", "chopper", "hovering", "steam", "pulsing", "wolf", "feathers", "gunshots", "launch_engine", "tone", "crow", "elecloop", "lab_spark", "roomhum"],
    ) {
        AMBIENCE
    } else if has(
        name,
        &["gun", "colt", "uzi", "m16", "walther", "sniper", "shotgun", "laser", "rocket", "grenade", "hybrid", "turok", "handgun", "mine", "tnt", "zippo", "brick", "throw", "blast", "fireball", "compound", "reload", "cock", "swipe", "punch", "_me_", "scifi_sniper", "withbullet", "with_bullet"],
    ) {
        WEAPONS
    } else {
        EFFECTS
    }
}

fn read_name(mut at: i32) -> String {
    let mut s = String::new();
    while lbu(at) != 0 && s.len() < 120 {
        s.push(lbu(at) as u8 as char);
        at += 1;
    }
    s
}

/// Hand the mixer the way to tell the group of a sample (called once at the start).
pub fn sound_mix_init() {
    // (sample address -> index in the table, kept while the entry still has that address; the table is filled again for every level)
    static CACHE: Mutex<Option<HashMap<u32, (i32, u8)>>> = Mutex::new(None);
    ts_audio::set_classifier(|ssa| {
        let mut guard = CACHE.lock().unwrap();
        let cache = guard.get_or_insert_with(HashMap::new);
        if let Some(&(i, group)) = cache.get(&ssa) {
            if (lw(g::sounds + 36 * i + 24) as u32 & 0x1f_ffff) == ssa {
                return group;
            }
        }
        for i in 0..SOUNDS {
            let at = g::sounds + 36 * i;
            if lw(at + 24) as u32 & 0x1f_ffff == ssa && lw(at + 24) != 0 {
                let p = lw(at + 4);
                if p == 0 {
                    continue;
                }
                let group = group_of(&read_name(p));
                cache.insert(ssa, (i, group));
                return group;
            }
        }
        u8::MAX
    });
}
