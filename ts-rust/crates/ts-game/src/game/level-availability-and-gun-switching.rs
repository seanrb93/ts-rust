//! Which levels are available, the arcade set-up of a level, and whether to switch to a gun.

use crate::*;

/// Whether level `level` is available: the story levels by `levelAvailSTORY`, the arcade levels by their unlock items (the compound, streets, future, site, bank, graveyard,
/// spaceship, mall and lab), the levels 26, 99 and 102 always, 105 by the items of the last part of the story, the rest not (@ 0x224dd0).
pub fn levelAvail(level: i32) -> i32 {
    if level_added_avail(level) {
        return 1;
    }
    let offset = level - 2;
    if (offset as u32) < 104 {
        match offset {
            0 | 3 | 6 | 7 | 12 | 19 | 21 | 22 | 25 => return levelAvailSTORY(level, 0),
            8 => return unlockedEx(g::unlock_compound, 0),
            9 => return unlockedEx(g::unlock_streets, 0),
            10 => return unlockedEx(g::unlock_future, 0),
            11 => return unlockedEx(g::unlock_site, 0),
            13 => return unlockedEx(g::unlock_bank, 0),
            14 => return unlockedEx(g::unlock_graveyard, 0),
            15 => return unlockedEx(g::unlock_spaceship, 0),
            16 => return unlockedEx(g::unlock_mall, 0),
            20 => return unlockedEx(g::unlock_lab, 0),
            24 | 98 | 101 => return 1,
            103 => return unlockedEx(g::unlock_story789, 0),
            _ => {}
        }
    }
    0
}

/// The set-up (the arcade modes and what they use) of level `level`, or 0 (@ 0x2279b8).
pub fn lvGetARCADESetup(level: i32) -> i32 {
    if let Some(setup) = custom_setup_x(level) {
        return setup;
    }
    let offset = level - 1;
    let mut setup = 0;
    if (offset as u32) < 105 {
        setup = match offset {
            0 => g::setup_prison,
            1 => g::setup_village_arcade,
            4 => g::setup_industrial_arcade,
            7 => g::setup_mansion_arcade,
            8 => g::setup_planet_arcade,
            9 => g::setup_compound_arcade,
            10 => g::setup_streets_arcade,
            11 => g::setup_future_arcade,
            12 => g::setup_site_dm,
            13 => g::setup_scifi_arcade,
            14 => g::setup_bank_arcade,
            15 => g::setup_graveyard_arcade,
            16 => g::setup_spaceship_arcade,
            17 => g::setup_mall_arcade,
            20 => g::setup_chinese_arcade,
            21 => g::setup_lab_arcade,
            23 => g::setup_docks_arcade,
            24 => g::setup_ron_ls,
            25 => g::setup_tomb_arcade,
            26 => g::setup_launch_arcade,
            27 => g::setup_tech_arcade,
            99 => g::setup_tileset,
            _ => 0,
        };
    }
    setup
}

/// Whether the player `player` should switch to the weapon combination `combo` (0 means to the empty hand, always yes): not when it is not owned; the ammo of the guns of the
/// combination (the guns' own clips added when a gun is also in the current combination's hand) must be there, else only when `gunShouldKeepWithNoAmmo`; and then only when
/// the player cannot use the left hand or the combination's right gun is not the current one's (@ 0x294c30).
pub fn gunShouldSwitchTo(player: i32, combo: i32) -> i32 {
    if combo == 0 {
        return 1;
    }
    if own_get(player, combo) == 0 {
        return 0;
    }
    let current = lw(player + 264);
    let mut right_ok = 1;
    let mut left_ok = 1;
    let mut left_clip = 0;
    let mut left_gun = 0;
    let mut right_clip = 0;
    let mut right_gun = 0;
    let right = lh((current << 3) + (GUNCOMBOS + 4));
    if right != -1 && (lw(g::currentplayer) + 600) != 0 && (player + 376) != 0 {
        let gun = lw(right * 400 + (GUNS_TABLE + 16));
        right_gun = gun;
        right_clip = lw(player + (gun << 2) + 380);
    }
    let left = lh((current << 3) + (GUNCOMBOS + 2));
    if left != -1 && (lw(g::currentplayer) + 408) != 0 && (player + 268) != 0 {
        let gun = lw(left * 400 + (GUNS_TABLE + 16));
        left_gun = gun;
        left_clip = lw(player + (gun << 2) + 272);
    }
    let base = GUNCOMBOS;
    let offset = combo << 3;
    let wanted_right = lh(base + offset + 4);
    if wanted_right != -1 {
        let gun = lw(wanted_right * 400 + (GUNS_TABLE + 16));
        let ammo = lw(player + (gun << 2) + 620);
        let with_right = if (gun ^ right_gun) == 0 { ammo + right_clip } else { ammo };
        right_ok = if (gun ^ left_gun) == 0 { with_right + left_clip } else { with_right };
    }
    let wanted_left = lh(base + offset + 2);
    if wanted_left != -1 {
        let gun = lw(wanted_left * 400 + (GUNS_TABLE + 16));
        let ammo = lw(player + (gun << 2) + 620);
        let with_right = if (gun ^ right_gun) == 0 { ammo + right_clip } else { ammo };
        left_ok = if (gun ^ left_gun) == 0 { with_right + left_clip } else { with_right };
    }
    if !(right_ok != 0 && left_ok != 0) && gunShouldKeepWithNoAmmo(player, combo) == 0 {
        return 0;
    }
    if gunLeftHandAble(player) == 0 {
        let at = GUNCOMBOS + 4;
        return (lh((lw(player + 260) << 3) + at) != lh(offset + at)) as i32;
    }
    1
}
