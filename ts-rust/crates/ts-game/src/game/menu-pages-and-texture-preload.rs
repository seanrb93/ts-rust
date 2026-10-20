//! Map maker and message pages, built-in maps, titles, and preloading the textures of models and levels.

use crate::*;

/// Show the display setting as the value of the game variable `gv_dispx` (@ 0x241558).
pub fn settingsToMenus() {
    let settings = settingsGet();
    gvSetValue(g::gv_dispx, lw(settings));
}

/// Open the main dialog of the map maker: flag 128 of the edit flags, the front end's main map maker page, flags 3 of the front end, the cursor position kept, and the zoom
/// tile's state reset (speed from the game's constant, no tile, tilted 25, not turned) (@ 0x252fa0).
pub fn mmStartMainDialog() {
    sw(g::editflags, lw(g::editflags) | 128);
    frontPageSet(0, g::mapmakermain_page);
    let front = lw(g::front);
    sw(front + 12, lw(front + 12) | 3);
    sw(0x3b0060, 0);
    mmPushCursorPos();
    swc1(0x3b007c, lwc1(0x3acaa4));
    swc1(0x3b0080, 1.0_f32);
    sw(g::zoomtilenum, -1);
    swc1(g::zoomtiletilt, 25.0_f32);
    sw(g::zoomtileroty, 0);
}

/// The tick of a message page: the page's own function (at 0x3afe2c) when there is one, else the default; when neither takes it over, the zoom box ticks, otherwise the page's
/// counter goes down (@ 0x206b90).
pub fn message_pageTick_206b90() {
    let handler = lw(0x3afe2c);
    let mut counts_down = false;
    if handler != 0 {
        let result = call_indirect(handler, [0; 8], [0.0; 8]).0;
        if result != 0 {
            counts_down = true;
        }
    }
    if !counts_down && defaultmessagetick_206b68() == 0 {
        zoomboxTick(0);
        return;
    }
    let front = lw(g::front);
    sw(front + 20, lw(front + 20) - 1);
}

/// Put the names of the ten built-in maps (records of 40 bytes from +12 of the directory, 31 characters) in the directory of map files (@ 0x253568).
pub fn mmfileDirBuiltinMaps() {
    let mut directory = lw(g::mapdir1);
    let mut i = 0;
    sw(directory + 8, 10);
    loop {
        let offset = i * 40;
        let word = lh((i << 3) + g::builtinmaps) << 2;
        i += 1;
        let name = lw(word + lw((lw(g::cur_language) << 2) + g::languages));
        strncpy(directory + (offset + 12), name, 31);
        sb(lw(g::mapdir1) + offset + 43, 0);
        if i >= 10 {
            break;
        }
        directory = lw(g::mapdir1);
    }
}

/// Make the title of the menu from the titles of the pages on the way (entries of 8 bytes from 0x1fc1308, up to the depth at 0x3aff2c) in the buffer at 0x1fc1388 (@ 0x2300b8).
pub fn makemtreeTitle() {
    let depth = lw(0x3aff2c);
    sb(0x1fc1388, 0);
    let mut limit = depth;
    let mut at = (depth << 3) + 0x1fc1308;
    let mut title = lw(at);
    let mut i = depth;
    loop {
        i += 1;
        at += 8;
        if title != 0 {
            strcat(0x1fc1388, title);
            limit = lw(0x3aff2c);
        }
        if limit < i {
            break;
        }
        title = lw(at);
    }
}

/// The text shown for a player's sign-on message: `text`, with two players or more the player's number (`player` + 1) in front, in the language's format (@ 0x231638).
pub fn GetPlayerSignOnMessage(player: i32, text: i32) -> i32 {
    let frame = Frame::new(304);
    let sp = frame.sp();
    strcpy(sp, text);
    let buffer = 0x1fc0b98;
    let one_player = lw(0x3aff20) < 2;
    sb(buffer, 0);
    if !one_player {
        let format = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1420);
        let prefix = vafmt(&[0x3a1950, format as i64, (player + 1) as i64]);
        strcpy(buffer, prefix);
    }
    strcat(buffer, sp);
    buffer
}

/// Set up the split screen pages of the players after the first: for each its message cleared, its flag halfword (from 0x3aff30) cleared, the front end moved to it, the page `page` set and
/// flags 3 of the front end cleared; then the front end goes back to the first (@ 0x2311a8).
pub fn splitpageInit(page: i32) {
    let mut player = 1;
    if player < lw(0x3aff20) {
        loop {
            splitMessageClear(player);
            let flag_at = (player << 1) + 0x3aff30;
            let current = player;
            player += 1;
            sh(flag_at, 0);
            frontSet(current);
            frontPageSet(0, page);
            let front = lw(g::front);
            let more = player < lw(0x3aff20);
            sw(front + 12, lw(front + 12) & -4);
            if !more {
                break;
            }
        }
    }
    frontSet(0);
}

/// Preload the textures of the model called `name`: those of its list of textures (16 bytes each, ended by -1) are passed to the second pass, unless the model has flag 0x10000000, and
/// the model is marked as done (+36 of its second header) (@ 0x25ea78).
pub fn obPreloadTextures(name: i32) {
    let entry = preloadGetOb(name);
    if entry != 0 {
        let base = lw(entry + 16);
        let textures = lw(base) + base;
        let header = lw(base + 4) + base;
        if (lw(textures + 12) & 0x10000000) == 0 {
            let mut i = 0;
            if lw(textures) != -1 {
                let mut at = textures;
                loop {
                    let number = lw(at);
                    i += 1;
                    preloadPass2AddTexNum(number, at, 0);
                    if lw((i << 4) + textures) == -1 {
                        break;
                    }
                    at = (i << 4) + textures;
                }
            }
            sw(header + 36, 1);
        }
    }
}

/// Preload the textures of the level (the file of the level's textures, named by the level number, except for the levels 100 to 105): its list of 16-byte records (ended by a
/// negative number) is passed to the second pass (@ 0x2564a8).
pub fn bgPreloadTextures() {
    let frame = Frame::new(96);
    let sp = frame.sp();
    let level = lw(g::lvlevelnum);
    if ((level - 100) as u32) >= 6 {
        sprintf(&[sp as i64, 0x3a2640, level as i64, level as i64]);
        let entry = preloadGetFile(sp);
        let base = lw(entry + 16);
        let textures = lw(base) + base;
        let first = lw(textures);
        sw(g::leveltextures, textures);
        if first >= 0 {
            let mut at = textures;
            let mut i = 1;
            loop {
                preloadPass2AddTexNum(lw(at), at, at);
                at = (i << 4) + lw(g::leveltextures);
                i += 1;
                if lw(at) < 0 {
                    break;
                }
            }
        }
    }
}

/// Load the built-in map number `index` into the map buffer (0x1ffc bytes) from its preloaded file; 1 if it has the right header (0x1324626 and 114), else the buffer is marked empty and 0 (@ 0x253678).
pub fn mmfileLoadBuiltinMap(index: i32) -> i32 {
    memset(lw(g::mmtempbuffer), 0, 0x1ffc);
    let entry = preloadGetFile(lw((index << 3) + (g::builtinmaps + 4)));
    if entry == 0 {
        return 0;
    }
    memcpy(lw(g::mmtempbuffer), lw(entry + 16), lw(entry + 20));
    let buffer = lw(g::mmtempbuffer);
    if lw(buffer) != 0x1324626 {
        sw(buffer, 0);
    } else {
        if lw(buffer + 4) == 114 {
            return 1;
        }
        sw(buffer, 0);
    }
    0
}
