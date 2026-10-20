//! Choosing the next gun combination, the gun a player starts with, and the props of the front end that are freed.

use crate::*;

/// Restart the HUD (@ 0x2b0988).
pub fn hudReset() {
    hudRestart();
}

/// Set the page `page` and its title `title` for the page that is current in the menu (the page number at 0x3aff2c; the pages are at 0x1fc130c, 8 bytes each) (@ 0x230218).
pub fn pageSet(title: i32, page: i32) {
    sw((lw(0x3aff2c) << 3) + 0x1fc130c, page);
    pageSetTitle(title);
}

/// When the current combination of the player `player` does not stay without ammo, change it: to none if the most powerful combination is the one it has, else to that one (@ 0x28dac8).
pub fn changeToComboWithAmmo(player: i32) {
    let current = lw(player + 260);
    if gunShouldKeepWithNoAmmo(player, current) == 0 {
        if getmostpowerfulcombo(player) == lw(player + 260) {
            sw(player + 260, 0);
        } else {
            let best = getmostpowerfulcombo(player);
            sw(player + 260, best);
        }
    }
}

/// Whether the combination `combo` is one of the five of the level's list (@ 0x227960).
pub fn lvIsValidGunCombo(combo: i32) -> i32 {
    let list = lvGetGunComboNums();
    let mut at = list;
    let mut i = 0;
    while lw(at) != combo {
        i += 1;
        at += 4;
        if i >= 5 {
            return 0;
        }
    }
    1
}

/// Give the player `player` the gun it starts with: the challenge's (+56 of its set-up) when the game flag 32 is set; else, with flag 512, the first of the level's list (or 1
/// when it is less); else none. A combination from 1 to 32 is owned, made current and its ammo filled (@ 0x27f098).
pub fn playerSetStartGun(player: i32) {
    let mut combo = 0;
    let flags = lw(g::curgameparms + 80);
    let record = lw(lw(player + 384) + 352);
    let index;
    if (flags & 32) == 0 {
        if (flags & 512) == 0 {
            return;
        }
        let list = lvGetGunComboNums();
        let first = lw(list);
        combo = if first >= 1 { first } else { 1 };
        index = combo - 1;
    } else {
        let setup = challengeGetSetup();
        combo = lw(setup + 56);
        index = combo - 1;
    }
    if (index as u32) >= 32 {
        return;
    }
    sw(record + (combo << 2) + 488, 1);
    sw(record + 260, combo);
    gunSetAmmoToMax(record, combo);
}

/// Go back to the page of the game mode that was last chosen (the mode at 0x1fc11f8, its flag 0x10000 at +4 cleared): 3 the arcade options, 4 the challenge, 6 the game type, 2
/// the story; page number 4, 3, 2 and 3 are kept at 0x3aff2c (@ 0x23ff50).
pub fn setlastgamemodepage() {
    let mode = lw(0x1fc11f8);
    sw(0x1fc11fc, lw(0x1fc11fc) & -0x10001);
    if mode == 3 {
        sw(0x3aff2c, 4);
        frontPageSet(0, g::arcadeoptions_page);
        return;
    }
    if mode >= 4 {
        if mode == 4 {
            sw(0x3aff2c, 3);
            frontPageSet(0, g::challenge_page_354eb8);
            return;
        }
        if mode == 6 {
            sw(0x3aff2c, 2);
            frontPageSet(0, g::gametype_page);
            return;
        }
        return;
    }
    if mode == 2 {
        sw(0x3aff2c, 3);
        frontPageSet(0, g::story_page_354e08);
    }
}

/// Whether the player `player`, who has just picked up the combination `combo`, should switch to it: with no weapon in the hand when the combination ranks above the current one
/// (the first halfword of its record is not above); with a weapon in the state 0, or in the state 2 when the new combination ranks higher, when `gunShouldSwitchTo` says so;
/// never in the state 1 (@ 0x294f68).
pub fn gunShouldSwitchToOnPickup(player: i32, combo: i32) -> i32 {
    let weapon = lw(lw(player + 3020) + 356);
    let mut state = 0;
    if weapon == 0 {
        let table = GUNCOMBOS;
        return (lh((lw(player + 260) << 3) + table) < lh((combo << 3) + table)) as i32 ^ 1;
    }
    let holder = lw(weapon + 24);
    if holder != 0 {
        state = lw(holder + 88);
    }
    if state == 1 {
        return 0;
    }
    if state >= 2 {
        if state != 2 {
            return 0;
        }
        let table = GUNCOMBOS;
        if lh((lw(player + 260) << 3) + table) < lh((combo << 3) + table) {
            return 0;
        }
    } else if state != 0 {
        return 0;
    }
    gunShouldSwitchTo(player, combo)
}

/// The index of the player's current combination in the table of combinations by number (`GUNSINV`), or -1.
fn current_in_table(player: i32) -> i32 {
    let current = lw(player + 260);
    let mut at = GUNSINV;
    let mut i = 0;
    let mut found = -1;
    loop {
        let value = lw(at);
        at += 4;
        found = if (value ^ current) == 0 { i } else { found };
        i += 1;
        if i >= 33 {
            break;
        }
    }
    found
}

/// The first index to try in the table of combinations: 1 for a challenge game of a player with a life count, else 0.
fn first_in_table(player: i32) -> i32 {
    let parms = g::curgameparms;
    let mut first = 0;
    if (lw(parms + 80) & 32) != 0 && lw(parms + 68) == 0 {
        first = (0 < lw(player + 632)) as i32;
    }
    first
}

/// The next combination (the one after the current one in the table, going round) that the player `player` should switch to, or the current one when there is none (@ 0x28d828).
pub fn getnextcombo(player: i32) -> i32 {
    let first = first_in_table(player);
    let found = current_in_table(player);
    let mut next = found + 1;
    let mut index;
    let mut entry;
    loop {
        index = if next >= 33 { first } else { next };
        entry = (index << 2) + GUNSINV;
        if gunShouldSwitchTo(player, lw(entry)) != 0 {
            break;
        }
        if found == index {
            break;
        }
        next = index + 1;
    }
    if found == index {
        lw(player + 260)
    } else {
        lw(entry)
    }
}

/// The previous combination (the one before the current one in the table, going round) that the player `player` should switch to, or the current one when there is none (@ 0x28d9b8).
pub fn getprevcombo(player: i32) -> i32 {
    let first = first_in_table(player);
    let found = current_in_table(player);
    let previous = found - 1;
    let mut index = if previous < first { 32 } else { previous };
    let mut entry;
    loop {
        entry = (index << 2) + GUNSINV;
        if gunShouldSwitchTo(player, lw(entry)) != 0 {
            break;
        }
        if found == index {
            break;
        }
        let before = index - 1;
        index = if before < first { 32 } else { before };
    }
    if found == index {
        lw(player + 260)
    } else {
        lw(entry)
    }
}

/// Free the effects group `group` (records of 92 bytes at `fxgroups`, -1 at +0 for free): its four props (from +56; the kind at +8 of a prop decides whether it is deleted as a
/// special prop, a corpse or a prop) and its two particle groups (from +72) (@ 0x2aaed8).
pub fn specialfxFree(group: i32) {
    let mut left = 3;
    let mut at = group * 92 + (g::fxgroups + 56);
    loop {
        let prop = lw(at);
        if prop != 0 {
            let kind = lw(prop + 8);
            if kind == 128 {
                specialPropDelete(prop);
                sw(at, 0);
            } else if kind == 0x1000 {
                propDeleteCorpse(prop);
                sw(at, 0);
            } else {
                propDelete(prop);
                sw(at, 0);
            }
        }
        left -= 1;
        at += 4;
        if left < 0 {
            break;
        }
    }
    let mut left = 1;
    let mut at = group * 92 + (g::fxgroups + 72);
    loop {
        let particles = lw(at);
        left -= 1;
        if particles != 0 {
            particleFree(particles);
            sw(at, 0);
        }
        at += 4;
        if left < 0 {
            break;
        }
    }
    sw(group * 92 + g::fxgroups, -1);
}

/// Delete the four props of the front end's characters (at 0x1fc1238), freeing the texture buffer of a model that only one of them used, and reset the texture buffers (@ 0x237510).
pub fn resetchrprops() {
    let base = 0x1fc1238;
    let mut i = 0;
    loop {
        let slot = i << 2;
        let next = i + 1;
        if lw(slot + base) != 0 {
            let mut same = 0;
            let mut at = base;
            let mut left = 3;
            loop {
                let prop = lw(at);
                let more = same + 1;
                if prop != 0 {
                    same = if (lw(lw(prop + 32)) ^ lw(lw(lw(slot + base) + 32))) == 0 { more } else { same };
                }
                left -= 1;
                at += 4;
                if left < 0 {
                    break;
                }
            }
            if same == 1 {
                obFreeTextures(lw(lw(lw(slot + base) + 32)));
            }
            propDelete(lw(slot + base));
            sw(slot + base, 0);
        }
        i = next;
        if i >= 4 {
            break;
        }
    }
    obResetTextures();
}
