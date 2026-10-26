//! Characters, enemies and the lists of what they carry; pads (the bots' map points) and their searches.

use crate::*;

/// Size of one character's record in `chrinfo`.
const CHRINFO_SIZE: i32 = 0x1210;

/// Number of bots (characters of the match with a team that is allowed), or, when `team` is not 0, of those of team `team` (@ 0x235590).
/// The 10 character slots are 16 bytes from 0x1fc2a88: a flag at +12, a rank at +0 (negative: out), the team index at +4.
pub fn countBots(team: i32) -> i32 {
    let mut count = 0;
    for i in 0..10 {
        let slot = 0x1fc2a88 + 16 * i;
        if lw(slot + 12) == 0 || lw(slot) < 0 {
            continue;
        }
        let t = lw(slot + 4);
        if lbu(t + 0x1fc1234) == 0 {
            continue;
        }
        if team != 0 {
            if t + 1 == team {
                count += 1;
            }
        } else {
            count += 1;
        }
    }
    count
}

/// How often the character `chr` (its record's first word is its number) was lost to the characters in play: the sum of a table of 16-bit counts (@ 0x289028).
pub fn chrCountLosses(chr: i32) -> i32 {
    let n = lw(g::lvnumplayers) + lw(g::numactivechars);
    let mut sum = 0;
    if n <= 0 {
        return 0;
    }
    let who = lw(chr);
    let info = lw(g::chrinfo);
    for i in 0..n {
        sum += lh(i * CHRINFO_SIZE + info + (who << 1) + 32);
    }
    sum
}

/// The place of `key` in a ranking: `size` pointers to records whose word at +4 is the character or team; 0 if it is not there.
fn rank_of(key: i32, size: i32, table: i32) -> i32 {
    for i in 0..size {
        if lw(lw((i << 2) + table) + 4) == key {
            return i;
        }
    }
    0
}

/// Place of a character in the game's ranking (@ 0x21e370).
pub fn gameGetRankChr(chr: i32) -> i32 {
    rank_of(chr, lw(g::gameranksizechr), g::gamerankschr)
}

/// Place of a team in the game's ranking (@ 0x21e320).
pub fn gameGetRankTeam(team: i32) -> i32 {
    rank_of(team, lw(g::gameranksizeteam), g::gameranksteam)
}

/// Remember an enemy that escorts someone, up to 5 (@ 0x2c7ef0).
pub fn enemyAddToEscortList(enemy: i32) {
    let n = lw(g::numEscorts);
    sw((n << 2) + g::escort, enemy);
    if n < 5 {
        sw(g::numEscorts, n + 1);
    }
}

/// Add to a list of 8-byte entries (`value` at +4, `pad` at +0) at `list`, counted by the word `count_at`; the count stops at `cap`, and the last entry is overwritten after that.
fn add_spawn(count_at: i32, list: i32, cap: i32, value: i32, pad: i32) {
    let n = lw(count_at);
    sw(list + (n << 3) + 4, value);
    sw((n << 3) + list, pad);
    if n < cap {
        sw(count_at, n + 1);
    }
}

/// Note a place where a bag is picked up (up to 6) (@ 0x2c7db0).
pub fn enemyAddToBagList(value: i32, pad: i32) {
    add_spawn(g::numPickupBags, g::bagspawn, 6, value, pad);
}

/// Note a place where an enemy drops something (up to 20) (@ 0x2c7e20).
pub fn enemyAddToDropList(value: i32, pad: i32) {
    add_spawn(g::numPickupDrops, g::dropspawn, 20, value, pad);
}

/// Note a place with a health pickup (up to 25) (@ 0x2c7de8).
pub fn enemyAddToHealthList(value: i32, pad: i32) {
    add_spawn(g::numPickupHealth, g::healthspawn, 25, value, pad);
}

/// Note a place with a weapon pickup (up to 35) (@ 0x2c7d78).
pub fn enemyAddToWeaponList(value: i32, pad: i32) {
    add_spawn(g::numPickupWeapons, g::weaponspawn, 35, value, pad);
}

/// Empty in the build (@ 0x2cd708).
pub fn enemyAddToExplosionList() {}

/// Empty in the build (@ 0x2cd710).
pub fn enemyRemoveFromExplosionList() {}

/// The drop-list entry (address) at pad `pad`, or 0 (@ 0x2cd9e0).
pub fn enemyGetPadProp(pad: i32) -> i32 {
    for i in 0..lw(g::numPickupDrops) {
        let entry = (i << 3) + g::dropspawn;
        if lw(entry) == pad {
            return entry;
        }
    }
    0
}

/// The number of the other character that stands on pad `pad`, or 0 (@ 0x2c0e50).  `enemy`'s own number is the word its record points to at +352.
pub fn enemyOnPad(enemy: i32, pad: i32) -> i32 {
    let n = lw(g::numactivechars);
    let me = if n > 0 { lw(lw(enemy + 352)) } else { 0 };
    for i in 0..n {
        if i != me && lw(i * CHRINFO_SIZE + lw(g::chrinfo) + 684) == pad {
            return i;
        }
    }
    0
}

/// Whether a pad record can be used: it exists, bit 0 of its flags is set (+4), and its first word is not -1 (@ 0x264ac8).
pub fn padvalid(pad: i32) -> i32 {
    (pad != 0 && lw(pad + 4) & 1 != 0 && lw(pad) != -1) as i32
}

/// The number of the pad (28-byte records) with external reference `extref`, or -1 (@ 0x264e00).
pub fn padNumFromExtref(extref: i32) -> i32 {
    let pads = lw(g::levelpads);
    for i in 0..lw(g::numpads) {
        if lw(i * 28 + pads) == extref {
            return i;
        }
    }
    -1
}

/// Counted the pads of a hall once, and does nothing with the count (@ 0x264be8).
pub fn counthallpads(_hall: i32) {}

/// Of `count` pad numbers at `list`, the one nearest to the point `pos` (three floats); 0 if none is nearer than the distance at 0x3aeb34 (@ 0x266c58).
pub fn closestpadinlist(pos: i32, count: i32, list: i32) -> i32 {
    let mut best = 0;
    let mut best_dist = lwc1(0x3aeb34);
    for i in 0..count.max(0) {
        let pad = lw(list + 4 * i);
        let at = pad * 28 + lw(g::levelpads);
        let dx = lwc1(at + 12) - lwc1(pos);
        let dy = lwc1(at + 16) - lwc1(pos + 4);
        let dz = lwc1(at + 20) - lwc1(pos + 8);
        let dist = (dx * dx + dy * dy + dz * dz).abs().psqrt();
        if dist < best_dist {
            best_dist = dist;
            best = pad;
        }
    }
    best
}
