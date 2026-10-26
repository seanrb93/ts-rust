//! Checking the sign-on of a memory card slot, the game state after a reset, making the items of a menu list, the HUD rank box, the size of a menu item, and the pad numbers of a level.

use crate::*;

/// Delete the sign-ons that are for the card `card` (the first word of each sign-on info record of 8 bytes at `signoninfo`) but no longer match the record `record` (its word at +4
/// against the second word of the info), and look again until none is deleted (@ 0x220448).
pub fn signonCheckSlot(record: i32, card: i32) {
    let info = g::signoninfo;
    loop {
        let mut clean = 1;
        let count = signonCount();
        let mut i = 0;
        let mut second = info + 4;
        let mut first = info;
        loop {
            if i >= count {
                break;
            }
            let id = lw(first);
            first += 8;
            if id != card {
                second += 8;
            } else {
                let other = lw(second);
                second += 8;
                if other != lw(record + 4) {
                    clean = 0;
                    signonDelete(i);
                    break;
                }
            }
            i += 1;
        }
        if clean != 0 {
            break;
        }
    }
}

/// Set the game state after a reset (1 for the levels 6 and 101 to 105, else 3) and set up the game mode: mode 1 clears the escort number at 0x3afee8, mode 4 sets up the escort,
/// mode 5 the knockout; a challenge game restarts the challenge (@ 0x21ede8).
pub fn gameResetAfter() {
    let level = lw(g::lvlevelnum);
    let state = if level == 6 || (level >= 101 && level < 106) { 1 } else { 3 };
    sw(g::gamestate, state);
    let mode = lw(g::curgameparms + 72) - 1;
    if (mode as u32) < 6 {
        match mode {
            0 => sw(0x3afee8, -1),
            3 => gmEscortSetup(),
            4 => knockoutSetup(),
            _ => {}
        }
    }
    if (lw(g::curgameparms + 80) & 32) == 0 {
        return;
    }
    challengeRestart();
}

/// Make the menu items of the page `page` into `menus` (records of 24 bytes): from the item the front end's scroll position (+12 of the page, a byte of the front end's) for as
/// many as the page's count at +2 allows (up to the total at +0), each placed 10 lower than the one before; the number made is returned (@ 0x24c168).
pub fn menulistmake(menus: i32, page: i32) -> i32 {
    let frame = Frame::new(16);
    let sp = frame.sp();
    let mut at = menus;
    let scroll = page + 12;
    let count = lhu(page + 2);
    let first = lbu(scroll + lw(lw(g::front)));
    let top = lhu(page + 6);
    let mut i = first;
    sh(sp, lhu(page + 4));
    sh(sp + 2, top);
    let mut item = page + 16 + (first << 5);
    if i < i + count {
        let total = lhu(page);
        loop {
            if i >= total {
                break;
            }
            i += 1;
            menuitemmake(at, sp, item);
            at += 24;
            let front = lw(lw(g::front));
            let shown = lhu(page + 2);
            sh(sp + 2, lhu(sp + 2) + 10);
            item += 32;
            if i >= lbu(scroll + front) + shown {
                break;
            }
            let _ = lhu(page);
        }
    }
    ((at - menus).wrapping_mul(-0x55555555)) >> 3
}

/// Draw the team's rank box at height `y` and `across` more to the right: the rank from `gameGetRankTeam` for the team `team` as the language's text "1st" to "3rd" or the
/// number with the language's suffix, in a box titled by the language's text at +3256 (@ 0x2b19b8).
pub fn hudRankGfx(team: i32, y: i32, across: i32) {
    let frame = Frame::new(32);
    let sp = frame.sp();
    let key = lw(team + 24);
    let x = lw(g::window) + (32 >> (lw(g::window + 88) & 31)) + across;
    let rank = gameGetRankTeam(key);
    let place = rank + 1;
    let language;
    let text;
    if place < 4 && place > 0 {
        language = lw(g::cur_language);
        text = lw((rank << 2) + (language << 4) + g::nth_144);
    } else {
        let suffix = lw((lw(g::cur_language) << 4) + (g::nth_144 + 12));
        text = vafmt(&[0x3afc10, place as i64, suffix as i64]);
        language = lw(g::cur_language);
    }
    let texts = lw((language << 2) + g::languages);
    let record = lw(lw(lw(g::currentplayer) + 384) + 352);
    sw(sp, 0);
    drawValueBox(lw(texts + 3256), text, x, y, 54, 22, lw(record + 20), 0x7f7f7f80);
}

/// Work out the sizes of the menu item `item` and of the two items it is joined to (at +8 and +16) and their places (`menuitemCalcPos`) (@ 0x20cc28).
pub fn menuitemInit(item: i32) {
    let frame = Frame::new(48);
    let sp = frame.sp();
    let mut extra = 0;
    menuitemCalcSize(item, sp);
    let before = lw(item + 8);
    if before != 0 {
        menuitemCalcSize(before, sp + 16);
        extra = lhu(before + 6);
    }
    let across = menuitemCalcPos(lbu(item + 4), lhu(item + 6), lw(sp), extra, extra + lw(sp + 16));
    let after = lw(item + 16);
    sh(item + 6, across);
    sb(item + 4, 0);
    if after != 0 {
        menuitemCalcSize(after, sp + 16);
        extra = lhu(after + 14);
    }
    let down = menuitemCalcPos(lbu(item + 12), lhu(item + 14), lw(sp + 4), extra, extra + lw(sp + 20));
    sh(item + 14, down);
    sb(item + 12, 0);
}

/// Put the pad numbers into the level's set-up: for each of its things (48 bytes each) with a prop number (not negative) the pad number of the reference at +24 goes to +44, and
/// for each of its pad references (16 bytes each) the number of the reference at +4 goes to +12 (@ 0x228c10).
pub fn lvLevelSetPadNumbers() {
    let setup = lvGetLevelSetup(lw(g::lvlevelnum));
    if setup == 0 {
        return;
    }
    let mut i = 0;
    if lw(setup + 20) > 0 {
        let mut offset = 0;
        loop {
            let record = lw(setup + 16) + offset;
            let number = lvGetPropNum(lw(record + 4));
            if number >= 0 {
                let pad = padNumFromExtref(lw(record + 24));
                sw(record + 44, pad);
            }
            let total = lw(setup + 20);
            i += 1;
            offset += 48;
            if i >= total {
                break;
            }
        }
    }
    let mut j = 0;
    if lw(setup + 36) > 0 {
        let mut records = lw(setup + 32);
        loop {
            let offset = j << 4;
            j += 1;
            let record = records + offset;
            let pad = padNumFromExtref(lw(record + 4));
            sw(record + 12, pad);
            if j >= lw(setup + 36) {
                break;
            }
            records = lw(setup + 32);
        }
    }
}
