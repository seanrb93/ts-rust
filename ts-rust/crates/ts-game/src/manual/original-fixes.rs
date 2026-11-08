//! Fixes of bugs of the original game, each switched on by a mod (`fixes()` of a script: a list of the names below) and off otherwise.
//!
//! The game's code asks `fix_on(FIX_...)` where the original goes wrong and does the fixed thing when it is on.  In a build without `extensions`
//! `fix_on` is always false, so the default build is the original.  The machines of a network game have the same mods with the same settings,
//! so they have the same fixes.  `docs/original-bugs.md` says what each bug does.

use std::sync::atomic::{AtomicU32, Ordering};

/// A sleeping enemy that has been tested for seeing the player is passed, so that the next sleeper is tested next time (`aiTickBefore`).
pub const FIX_SLEEPING_ENEMIES: u32 = 1 << 0;
/// A match on a map of the map maker (level 100) does not add to a word past the 30 levels of the statistics (`gameStatusUpdated`).
pub const FIX_CUSTOM_MAP_STATS: u32 = 1 << 1;
/// "Last seen by" is not recorded for a dead player, by the player of the pair tested (`playerAutoAimTick`).
pub const FIX_SEEN_BY: u32 = 1 << 2;
/// The statistic 68 of a kill goes to the killer (`propDamage`).
pub const FIX_KILL_STAT: u32 = 1 << 3;
/// The pickup of a weapon reads the flags of its gun, not of the gun with the number of the combination (`pickupPropTick`).
pub const FIX_PICKUP_GUN: u32 = 1 << 4;
/// The mean of statistic 46 for the medals starts from 0 (`medals`).
pub const FIX_MEDAL_MEAN: u32 = 1 << 5;
/// A full list of bag pads or bases takes no more (`enemyAddToBagList`, `enemyAddToPropList`).
pub const FIX_SPAWN_LISTS: u32 = 1 << 6;
/// The level is not ended twice in one frame (time out and the mode in the same frame).
pub const FIX_DOUBLE_END: u32 = 1 << 7;
/// A defender keeps the pad it chose (`enemyDefend`).
pub const FIX_DEFEND_PAD: u32 = 1 << 8;
/// An attacker plans a new route when its target has gone to another pad (`pad_attack`).
pub const FIX_ROUTE_RESET: u32 = 1 << 9;
/// A health or armour pickup the enemy decided against is not left as its target (`enemyCheckHealth`).
pub const FIX_REJECTED_PICKUP: u32 = 1 << 10;
/// The wall test of the following camera starts in the room of the eye (`camTick`).
pub const FIX_CAMERA_ROOM: u32 = 1 << 11;
/// The direction of a hit on a limb is square to the limb (`hittestLineCylinder`).
pub const FIX_LIMB_NORMAL: u32 = 1 << 12;
/// The map maker's list comes back on the map that was saved (`mmfileTick`).
pub const FIX_MAP_CURSOR: u32 = 1 << 13;
/// The list of maps of the arcade page comes back on the map that was loaded.
pub const FIX_ARCADE_MAP_CURSOR: u32 = 1 << 14;
/// A name of a sign-on takes 29 letters, so that its end does not blank the next name.
pub const FIX_NAME_LENGTH: u32 = 1 << 15;
/// A paste of the map maker that fails puts the map back as it was.
pub const FIX_PASTE: u32 = 1 << 16;
/// The camping time of a player grows while the player stays near one place (`playerCampingDetection`).
pub const FIX_CAMPING: u32 = 1 << 17;
/// The push of a player (mode 1) takes off its own part of each axis.
pub const FIX_PUSHED: u32 = 1 << 18;

/// The names a mod gives for each fix.
pub const FIX_NAMES: [(&str, u32); 19] = [
    ("sleeping_enemies", FIX_SLEEPING_ENEMIES),
    ("custom_map_stats", FIX_CUSTOM_MAP_STATS),
    ("seen_by", FIX_SEEN_BY),
    ("kill_stat", FIX_KILL_STAT),
    ("pickup_gun", FIX_PICKUP_GUN),
    ("medal_mean", FIX_MEDAL_MEAN),
    ("spawn_lists", FIX_SPAWN_LISTS),
    ("double_end", FIX_DOUBLE_END),
    ("defend_pad", FIX_DEFEND_PAD),
    ("route_reset", FIX_ROUTE_RESET),
    ("rejected_pickup", FIX_REJECTED_PICKUP),
    ("camera_room", FIX_CAMERA_ROOM),
    ("limb_normal", FIX_LIMB_NORMAL),
    ("map_cursor", FIX_MAP_CURSOR),
    ("arcade_map_cursor", FIX_ARCADE_MAP_CURSOR),
    ("name_length", FIX_NAME_LENGTH),
    ("paste", FIX_PASTE),
    ("camping", FIX_CAMPING),
    ("pushed", FIX_PUSHED),
];

static MASK: AtomicU32 = AtomicU32::new(0);

/// Is the fix on?
#[inline]
pub fn fix_on(fix: u32) -> bool {
    #[cfg(feature = "extensions")]
    {
        MASK.load(Ordering::Relaxed) & fix != 0
    }
    #[cfg(not(feature = "extensions"))]
    {
        let _ = fix;
        false
    }
}

/// The fixes the scripts of the mods asked for, by name (unknown names are told in the log once).
pub fn fixes_set(names: &[String]) {
    let mut mask = 0;
    for n in names {
        match FIX_NAMES.iter().find(|(name, _)| name.eq_ignore_ascii_case(n.trim())) {
            Some((_, bit)) => mask |= bit,
            None => {
                static TOLD: AtomicU32 = AtomicU32::new(0);
                if TOLD.swap(1, Ordering::Relaxed) == 0 {
                    eprintln!("[fixes] no fix is called {n:?}");
                }
            }
        }
    }
    if MASK.swap(mask, Ordering::Relaxed) != mask {
        eprintln!("[fixes] on: {}", FIX_NAMES.iter().filter(|(_, b)| mask & b != 0).map(|(n, _)| *n).collect::<Vec<_>>().join(", "));
    }
}
