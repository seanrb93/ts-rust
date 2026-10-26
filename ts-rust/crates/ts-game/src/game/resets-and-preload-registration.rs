//! Resetting subsystems at the start of a level, naming files, registering preloads, and making a new sign-on.

use crate::*;

/// Reset the cartridges (@ 0x295280).
pub fn cartridgesReset() {
    cartridgesRestart();
}

/// Reset the weather (@ 0x2a8030).
pub fn weatherReset() {
    weatherRestart();
}

/// Find the texture of the zoom box frame (the number kept at +20 of the preload entry of the texture file) (@ 0x22ecf0).
pub fn zoomboxReset() {
    let entry = preloadGetTexName(g::texturefile);
    sw(0x3aff18, lw(entry + 20));
}

/// No pickup positions; find the texture of their marks (@ 0x272a88).
pub fn pickupposRestart() {
    let entry = preloadGetTexName(0x3a8028);
    let texture = lw(entry + 20);
    sw(g::num_pickuppos, 0);
    sw(g::pickuppos_texnum, texture);
}

/// Copy a loaded map file directory (from `source`) over the directory (@ 0x2539c8).
pub fn mmfileDirLoaded(source: i32) {
    let directory = lw(g::mapdir1);
    let size = mmfileDirSize();
    memcpy(directory, source, size);
}

/// The name of the file the pads of this level are written to (@ 0x2640e8).
pub fn padFileOutName() -> i32 {
    let level = lw(g::lvlevelnum);
    sprintf(&[0x1fc6450, 0x3a2778, level as i64]);
    0x1fc6450
}

/// The name of the file the pads of this level are read from: another format for level 27 in the game modes 2, 3, 4, 5 and 6 (@ 0x264058).
pub fn padFileInName() -> i32 {
    if lw(g::lvlevelnum) == 27 {
        let mode = lw(g::curgameparms + 72);
        if ((mode - 1) as u32) < 3 || mode == 5 || mode == 6 {
            let level = lw(g::lvlevelnum);
            sprintf(&[0x1fc6410, 0x3a2730, level as i64]);
            return 0x1fc6410;
        }
    }
    let level = lw(g::lvlevelnum);
    sprintf(&[0x1fc6410, 0x3a2758, level as i64]);
    0x1fc6410
}

/// Start the split screen message number `number` (records of 260 bytes at 0x1fc1488: a word and up to 256 characters): `kind` and the text `text` (@ 0x230a88).
pub fn splitMessageStart(number: i32, kind: i32, text: i32) {
    let record = number * 260 + 0x1fc1488;
    strncpy(record + 4, text, 256);
    sw(record, kind);
    sb(record + 259, 0);
}

/// The weapon combination at place `place` of the chosen weapon set (the game's own list for set 0, else the preset's, 20 bytes a set from `gunpresets` + 160); -1 gives 0 (@ 0x232d48).
pub fn frontmpgGetGunComboNum(place: i32) -> i32 {
    let set = weaponset_read();
    let list = if set == 0 { g::gunzpack } else { set * 20 + (g::gunpresets + 160) };
    let combo = lw((place << 2) + list);
    dual_combo_or(place, if !(0 | combo) == 0 { 0 } else { combo })
}

/// Add the file `name` (with the size or place `info`) to the preload list when it is not in it (@ 0x21c2d8).
pub fn preloadAddFile(name: i32, info: i32) {
    if preloadGetFile(name) != 0 {
        return;
    }
    let count = lw(g::numpreload);
    sw(g::numpreload, count + 1);
    let entry = count * 24 + 0x1fb4c60;
    sw(entry + 12, name);
    sw(entry + 16, info);
    sw(entry, 0);
    sw(entry + 4, 0);
    preloadInsert(entry);
}

/// Set the game variable of the tune to the place of the tune `tune` in the list of tunes that are shown (@ 0x24c890).
pub fn mmSetTuneNum(tune: i32) {
    let mut place = 0;
    loop {
        let shown = lbu(place + g::disptunenums);
        let this = place;
        place += 1;
        if shown == tune {
            gvSetValue(g::gv_tune, this);
        }
        if (place as u32) >= 24 {
            break;
        }
    }
}

/// Set the root pages of the in-game menu of player `player` (records of 92 bytes at 0x1fb3240) and clear the rest of it (@ 0x212890).
pub fn setRootPages(player: i32) {
    let menu = player * 92 + 0x1fb3240;
    sw(menu + 28, g::story_page_32aaf0);
    sw(menu + 32, g::challenge_page_32aa50);
    sw(menu + 36, g::pause_page);
    sw(menu + 24, g::scores_page);
    sw(menu + 40, g::mainsetup_page);
    sw(menu + 44, g::endgame_page);
    sw(menu + 48, 0);
    sw(menu + 12, 0);
    sw(menu + 16, 0);
    sw(menu + 20, 0);
    memset(menu + 52, 0, 40);
    sh(menu + 2, 0);
}

/// Find the textures of the six flares and restart the flare test (@ 0x2a6730).
pub fn zbtestResetAfter() {
    let mut target = g::flareTextureNum;
    let mut name_at = g::flareTexturenames_36a290;
    let mut left = 6;
    loop {
        let name = lw(name_at);
        left -= 1;
        name_at += 4;
        let entry = preloadGetTexName(name);
        sw(target, lw(entry + 20));
        target += 4;
        if left < 0 {
            break;
        }
    }
    zbtestRestartAfter();
}

/// Clear texture buffer `n` (records of 0x1a18 bytes at `texbuf`): the 40-byte texture records, the slots and the table, and the last slot is the number minus 1 (@ 0x2b69a8).
pub fn emResetTexBuffer(n: i32) {
    let buffer = n * 0x1a18 + g::texbuf;
    memset(lw(buffer + 4), 0, lw(buffer + 0x1a14) * 40);
    memset(buffer + 8, 0, 0x1800);
    memset(buffer + 0x1808, 0, 512);
    sw(buffer + 0x1a08, lw(buffer + 0x1a14) - 1);
}

/// Find where the files of the sounds were loaded (records of 36 bytes at `sounds`: with flag 128 the place and size from the preload entry of the file, else 0), and restart (@ 0x204750).
pub fn soundResetAfter() {
    let mut sound = g::sounds;
    let end = sound + 0xaadc;
    let mut flags = lhu(sound + 22);
    loop {
        if (flags & 128) == 0 {
            sw(sound + 28, 0);
            sw(sound + 32, 0);
        } else {
            let entry = preloadGetFile(lw(sound + 4));
            sw(sound + 28, lw(entry + 16));
            sw(sound + 32, lw(entry + 20));
        }
        sound += 36;
        if sound >= end {
            break;
        }
        flags = lhu(sound + 22);
    }
    soundRestartAfter();
}

/// Make a new sign-on called `name` in a free place (records of 2936 bytes): its preferences, settings and statistics are new; returns its number or -1 (@ 0x2202c0).
pub fn signonNew(name: i32) -> i32 {
    let number = signonGetFree();
    if number < 0 {
        return -1;
    }
    let record = lw(g::signonarray) + number * 2936;
    strcpy(record, name);
    playerprefsNew(record + 32);
    settingsNew(record + 128);
    statsNew(record + 132);
    let slot = number << 3;
    let info = g::signoninfo;
    sw(info + slot + 4, 0);
    sw(slot + info, -1);
    number
}
