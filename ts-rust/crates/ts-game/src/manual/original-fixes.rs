//! Fixes of bugs of the original game, each switched on by a mod (`fixes()` of a script: a list of the names below) and off otherwise.
//!
//! The game's code asks `fix_on(FIX_...)` where the original goes wrong and does the fixed thing when it is on.  In a build without `extensions`
//! `fix_on` is always false, so the default build is the original.  The machines of a network game have the same mods with the same settings,
//! so they have the same fixes.  `docs/original-bugs.md` says what each bug does.

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

/// A sleeping enemy that has been tested for seeing the player is passed, so that the next sleeper is tested next time (`aiTickBefore`).
pub const FIX_SLEEPING_ENEMIES: u64 = 1 << 0;
/// A match on a map of the map maker (level 100) does not add to a word past the 30 levels of the statistics (`gameStatusUpdated`).
pub const FIX_CUSTOM_MAP_STATS: u64 = 1 << 1;
/// "Last seen by" is not recorded for a dead player, by the player of the pair tested (`playerAutoAimTick`).
pub const FIX_SEEN_BY: u64 = 1 << 2;
/// The statistic 68 of a kill goes to the killer (`propDamage`).
pub const FIX_KILL_STAT: u64 = 1 << 3;
/// The pickup of a weapon reads the flags of its gun, not of the gun with the number of the combination (`pickupPropTick`).
pub const FIX_PICKUP_GUN: u64 = 1 << 4;
/// The mean of statistic 46 for the medals starts from 0 (`medals`).
pub const FIX_MEDAL_MEAN: u64 = 1 << 5;
/// A full list of bag pads or bases takes no more (`enemyAddToBagList`, `enemyAddToPropList`).
pub const FIX_SPAWN_LISTS: u64 = 1 << 6;
/// The level is not ended twice in one frame (time out and the mode in the same frame).
pub const FIX_DOUBLE_END: u64 = 1 << 7;
/// A defender keeps the pad it chose (`enemyDefend`).
pub const FIX_DEFEND_PAD: u64 = 1 << 8;
/// An attacker plans a new route when its target has gone to another pad (`pad_attack`).
pub const FIX_ROUTE_RESET: u64 = 1 << 9;
/// A health or armour pickup the enemy decided against is not left as its target (`enemyCheckHealth`).
pub const FIX_REJECTED_PICKUP: u64 = 1 << 10;
/// The wall test of the following camera starts in the room of the eye (`camTick`).
pub const FIX_CAMERA_ROOM: u64 = 1 << 11;
/// The direction of a hit on a limb is square to the limb (`hittestLineCylinder`).
pub const FIX_LIMB_NORMAL: u64 = 1 << 12;
/// The map maker's list comes back on the map that was saved (`mmfileTick`).
pub const FIX_MAP_CURSOR: u64 = 1 << 13;
/// The list of maps of the arcade page comes back on the map that was loaded.
pub const FIX_ARCADE_MAP_CURSOR: u64 = 1 << 14;
/// A name of a sign-on takes 29 letters, so that its end does not blank the next name.
pub const FIX_NAME_LENGTH: u64 = 1 << 15;
/// A paste of the map maker that fails puts the map back as it was.
pub const FIX_PASTE: u64 = 1 << 16;
/// The camping time of a player grows while the player stays near one place (`playerCampingDetection`).
pub const FIX_CAMPING: u64 = 1 << 17;
/// The push of a player (mode 1) takes off its own part of each axis.
pub const FIX_PUSHED: u64 = 1 << 18;
/// Story level 5 does not go on to turn off the link of level 9, which it does not have (link 0 of the level went instead) (`lvInitStoryState`).
pub const FIX_STORY_LINK: u64 = 1 << 19;
/// The shards of a mirror fade from 1 instead of jumping to 1.9 (`particleGlassTick`).
pub const FIX_MIRROR_GLASS: u64 = 1 << 20;
/// The arrows of a choice of a menu stay at the widest choice instead of moving with the current one (`draw_menuitem`).
pub const FIX_OPTION_ARROWS: u64 = 1 << 21;
/// A rate under 1% on the statistics comparison page is shown as a percentage (`statistics_pageDrawInfo`).
pub const FIX_STATS_PERCENT: u64 = 1 << 22;
/// Every player of a team sees the team's row of the rankings marked, not only the first (`drawRankingsTable`).
pub const FIX_TEAM_RANKINGS: u64 = 1 << 23;
/// Colour 3 of the glow of a pickup that comes back is green, not the orange of colour 0 (`specialfxCreate` kind 9).
pub const FIX_SPAWN_GLOW: u64 = 1 << 24;
/// An idle enemy outside the story lets animation 3 finish before it picks again (`enemySetIdleAnimation`).
pub const FIX_IDLE_ANIMATION: u64 = 1 << 25;
/// A move that meets a wall at a slant goes up to the wall, not short of it (`hittestMoveCircleToLine`).
pub const FIX_WALL_SLANT: u64 = 1 << 26;
/// The third pass of the defocus effect is one down and one to the right, not the first again (`ipDefocus`).
pub const FIX_DEFOCUS: u64 = 1 << 27;
/// The animation blending in is held at its own end, not at the length of the other animation (`calMatrices`).
pub const FIX_BLEND_LENGTH: u64 = 1 << 28;
/// A looping animation blending in takes off a whole loop's movement when it wraps (`calMatrices`).
pub const FIX_LOOP_ROOT: u64 = 1 << 29;
/// A pad with no room under it keeps its own height, not the floor of the pad before (`padSetRoomNums`).
pub const FIX_PAD_FLOOR: u64 = 1 << 30;
/// A plain file of the preload list is put in its place on the disc, not by an old entry's sector (`preloadAddFile`).
pub const FIX_PRELOAD_ORDER: u64 = 1 << 31;
/// A texture of more than one block takes only slots that are all free (`emGetNextPos`).
pub const FIX_TEXTURE_SLOTS: u64 = 1 << 32;
/// Taking the last decal out of room 0 clears the room's list (`decalRemoveFromDrawList`).
pub const FIX_DECAL_ROOM0: u64 = 1 << 33;
/// The electricity tick leaves the chunk that is not a line alone, instead of reading and writing before its tables (`electricityPropTick`).
pub const FIX_ELECTRICITY: u64 = 1 << 34;
/// A sound of no size lets go of the lock of the sound transfers, so later sounds are not held up (`soundTick`).
pub const FIX_SOUND_LOCK: u64 = 1 << 35;
/// Only the slots a port has are asked for a pad (`controllerTick`).
pub const FIX_MULTITAP_SLOT: u64 = 1 << 36;
/// A pad with no motors is not moved on to a state nothing reads (`step_pad_state`).
pub const FIX_PAD_NO_MOTORS: u64 = 1 << 37;
/// The pitch of an enemy's aim at a target of another kind is towards the target, not from stack words nothing wrote (`rotateChr`).
pub const FIX_AIM_PITCH: u64 = 1 << 38;
/// A chaser plans its route again when its target's pad changes, not only when that happens on its own turn (`followTarget`, `trackTarget`).
pub const FIX_CHASE_ROUTE: u64 = 1 << 39;
/// The killing spree starts again at a kill more than two seconds after the last (`StatsKillMade`).
pub const FIX_KILL_SPREE: u64 = 1 << 40;
/// The test for a pad taken by another character looks at every bot, and finds character 0 (`enemyOnPad`).
pub const FIX_ON_PAD: u64 = 1 << 41;
/// A zombie's turn animation is not started again while it plays (`setZombieAnimation`).
pub const FIX_ZOMBIE_TURN: u64 = 1 << 42;
/// A flickering room light keeps its colour on a frame with no update and does not take the strength of the room before it (`roomlightTick`).
pub const FIX_ROOM_LIGHT: u64 = 1 << 43;
/// The sound of a gun picked up is that gun's, not the one in hand (`pickupPropTick`).
pub const FIX_PICKUP_SOUND: u64 = 1 << 44;
/// A character nearest pad 0 is put on it (`enemyCheckValidPad`).
pub const FIX_PAD_ZERO: u64 = 1 << 45;
/// The map maker's pad cursor compares depth as a fraction, so the nearer of two close pads wins (`findHighlightPad`).
pub const FIX_HIGHLIGHT_DEPTH: u64 = 1 << 46;
/// The message for a dropped bag goes to a player only (`propDropBagOnDrop`).
pub const FIX_BAG_MESSAGE: u64 = 1 << 47;
/// A sixth escort is not written past the escort list (`enemyAddToEscortList`).
pub const FIX_ESCORT_LIST: u64 = 1 << 48;

/// The names a mod gives for each fix.
pub const FIX_NAMES: [(&str, u64); 49] = [
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
    ("story_link", FIX_STORY_LINK),
    ("mirror_glass", FIX_MIRROR_GLASS),
    ("option_arrows", FIX_OPTION_ARROWS),
    ("stats_percent", FIX_STATS_PERCENT),
    ("team_rankings", FIX_TEAM_RANKINGS),
    ("spawn_glow", FIX_SPAWN_GLOW),
    ("idle_animation", FIX_IDLE_ANIMATION),
    ("wall_slant", FIX_WALL_SLANT),
    ("defocus", FIX_DEFOCUS),
    ("blend_length", FIX_BLEND_LENGTH),
    ("loop_root", FIX_LOOP_ROOT),
    ("pad_floor", FIX_PAD_FLOOR),
    ("preload_order", FIX_PRELOAD_ORDER),
    ("texture_slots", FIX_TEXTURE_SLOTS),
    ("decal_room0", FIX_DECAL_ROOM0),
    ("electricity", FIX_ELECTRICITY),
    ("sound_lock", FIX_SOUND_LOCK),
    ("multitap_slot", FIX_MULTITAP_SLOT),
    ("pad_no_motors", FIX_PAD_NO_MOTORS),
    ("aim_pitch", FIX_AIM_PITCH),
    ("chase_route", FIX_CHASE_ROUTE),
    ("kill_spree", FIX_KILL_SPREE),
    ("on_pad", FIX_ON_PAD),
    ("zombie_turn", FIX_ZOMBIE_TURN),
    ("room_light", FIX_ROOM_LIGHT),
    ("pickup_sound", FIX_PICKUP_SOUND),
    ("pad_zero", FIX_PAD_ZERO),
    ("highlight_depth", FIX_HIGHLIGHT_DEPTH),
    ("bag_message", FIX_BAG_MESSAGE),
    ("escort_list", FIX_ESCORT_LIST),
];

static MASK: AtomicU64 = AtomicU64::new(0);

/// Is the fix on?
#[inline]
pub fn fix_on(fix: u64) -> bool {
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
