//! Choosing characters by cheat and team, and freeing lists of map maker objects.

use crate::*;

/// Free a map maker object and all those after it in its list (through +56) (@ 0x248798).
pub fn mmFreeObject(first: i32) {
    let mut object = first;
    while object != 0 {
        let next = lw(object + 56);
        mmFreeSingleObject(object);
        object = next;
    }
}

/// The team (1 to 4) when exactly one of them can be chosen, else 0 (@ 0x235710).
pub fn onlyoneselectableteam() -> i32 {
    let mut count = 0;
    let mut team_before = 0;
    let mut previous = 0;
    loop {
        let team = previous + 1;
        if selectableTeam(team) != 0 {
            team_before = previous;
            count += 1;
        }
        previous = team;
        if previous >= 4 {
            break;
        }
    }
    if count != 1 { 0 } else { team_before + 1 }
}

/// The character types for setup `setup` of the level: `first_out` gets the one the player plays (in the story mode a cheat, if it is selected, unlocked
/// and allowed, picks one: 256 -> 21, 512 -> 23, 1024 -> 22, 2048 -> 20, 4096 -> 24; else the level's), `second_out` the level's (@ 0x227648).
pub fn lvGetChrTypeNumFromSetup(setup: i32, first_out: i32, second_out: i32) {
    sw(first_out, -1);
    if lw(g::curgameparms + 72) == 7 {
        let active = || lw(g::allowedcheats) & lw(g::unlockedcheats) & lw(g::selectedcheats);
        for (bit, character) in [(256, 21), (512, 23), (1024, 22), (2048, 20), (0x1000, 24)] {
            if active() & bit != 0 {
                sw(first_out, character);
            }
        }
    }
    let index = (if setup < 11 { setup } else { 0 }) << 2;
    if lw(first_out) == -1 {
        sw(first_out, lw(index + lvGetLevelChrTypeNums(lw(g::lvlevelnum))));
    }
    sw(second_out, lw(index + lvGetLevelChrTypeNums(lw(g::lvlevelnum))));
}
