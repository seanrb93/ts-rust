//! Small game functions: the preload list, the demo, the level restart after, the map maker's backup, the character select page and the like.

use crate::*;

/// Load the textures of the second pass of the preload: the list of records (texture, two arguments and the next) from the head at 0x3afee4 (@ 0x21c908).
pub fn preloadPass2Exec() {
    let mut record = lw(0x3afee4);
    if record != 0 {
        loop {
            emLoadTextureNum(lw(record), lw(record + 4), lw(record + 8));
            record = lw(record + 12);
            if record == 0 {
                break;
            }
        }
    }
}

/// Update the guns of all the non-player characters (@ 0x294970).
pub fn chrgunTick() {
    let mut i = lw(g::lvnumplayers);
    if i < i + lw(g::numactivechars) {
        let mut at = i * 0x1210;
        let mut info = lw(g::chrinfo);
        loop {
            i += 1;
            let chr = lw(at + info + 3020);
            at += 0x1210;
            enemyGunUpdate(chr);
            info = lw(g::chrinfo);
            if i >= lw(g::lvnumplayers) + lw(g::numactivechars) {
                break;
            }
        }
    }
}

/// Delete an enemy that was spawned: it has no behaviour, its prop is made dead (flags 128 instead of 32 and 2), its guns are cleared and the spawner counts one less (@ 0x2cc498).
pub fn enemyDeleteNPC(a0: i32) {
    let record = lw(a0 + 352);
    sw(record + 2716, 0);
    let prop = lw(record + 3020);
    sw(prop + 16, lw(prop + 16) & -34 | 128);
    chrClearGuns(record);
    let spawner = lw(record + 3032);
    sw(record + 680, -1);
    if spawner != 0 {
        let count = lw(spawner + 44);
        let total = lw(g::totactivespawnedchars) - 1;
        sw(record + 3032, 0);
        sw(g::totactivespawnedchars, total);
        sw(spawner + 44, count - 1);
    }
}

/// Kill a character prop: its health is set to 0 and a damage of the kind 2 is dealt to it at its position (@ 0x2728d0).
pub fn propKill(a0: i32) {
    if lw(a0 + 8) != 8 {
        return;
    }
    let damage = lwc1(0x3af948);
    sw(lw(a0 + 352) + 12, 0);
    propDamage(a0, 0, a0 + 48, -1, 0, 0, 0, 2, damage);
}

/// Draw the challenge character page: the background, the select effect and, once the flag of the front end is 2, the name of the character of the challenge set (@ 0x23c308).
pub fn challengechr_pageDraw(a0: i32) {
    let setup = challengeGetSetupNum(lw(0x1fc1224));
    let character = lw(lw(setup + 48));
    pagebackgroundGfx_230688(a0);
    fxSelectCharGfx();
    if lbu(lw(lw(g::front)) + 0x3aff28) != 2 {
        return;
    }
    let name = lw((lh(character * 44 + g::chrs) << 2) + lw((lw(g::cur_language) << 2) + g::languages));
    drawCharName(a0, 1, name, 0);
}

/// Load the demo `a0`: the file is read and the replay is reset with its header (@ 0x225740).
pub fn demoLoad(a0: i32) {
    let frame = Frame::new(32);
    let sp = frame.sp();
    let loaded = fileLoad(demoFileName(a0), 1, sp);
    let size = lw(sp);
    sw(g::demoheader, loaded);
    replayReset(1, lw(loaded), loaded, size);
}

/// Start the character select effect of the player `a0` with the character `a1`: the first time the character is selected, otherwise the box moves in the direction the front
/// end says (@ 0x2384d8).
pub fn fxSelectCharInit(a0: i32, a1: i32) {
    let at = a0 << 2;
    let slot = at + 0x1fc1288;
    if lw(slot) == -1 {
        sw(slot, a1);
        selectChar(a0, a1);
    } else if (lw(lw(g::front) + 12) & 16) == 0 {
        sw(at + 0x1fc1268, 1);
    } else {
        sw(at + 0x1fc1268, -1);
    }
    sw(at + 0x1fc1298, a1);
    swc1(at + 0x1fc12e8, 1.0_f32);
    sw(at + 0x1fc12d8, 0);
    actingsoundStop(a0);
}

/// Delete all the marked objects of the map: the map is backed up first and the lasso is cleared (@ 0x245c80).
pub fn mmDeleteAllMarked() {
    mmBackupMap();
    let mut object = lw(g::mapobjects);
    if object != 0 {
        let mut marked = lw(object + 4);
        loop {
            let next = lw(object + 56);
            if marked != 0 {
                mmFreeSingleObject(object);
            }
            object = next;
            if object == 0 {
                break;
            }
            marked = lw(object + 4);
        }
    }
    sw(g::lassocount, 0);
}

/// Restart the music for the level: the tune of the level is looked up (none when the music is off), opened, set to volume and started (@ 0x205c78).
pub fn musicRestart() {
    let tune = av_level_music_or(lvGetTuneNum());
    sw(g::tunenum, tune);
    if lw(g::gv_playmusic) == 0 {
        sw(g::tunenum, 0);
    }
    let tune = lw(g::tunenum);
    if tune == 0 {
        return;
    }
    musicOpen(tune);
    stream_RPC(128, 0);
    stream_RPC(129, 1);
    musicSetFrontEndVolume();
    musicStart();
}

/// Add the model `a0` to the preload list (kind 1) with the argument `a1` unless it is already there: the record of 24 bytes gets the file's sector (@ 0x21c350).
pub fn preloadAddOb(a0: i32, a1: i32) {
    if preloadGetOb(a0) != 0 {
        return;
    }
    let number = lw(g::numpreload);
    sw(g::numpreload, number + 1);
    let record = number * 24 + preload_table();
    sw(record, 1);
    let sector = fileGetSector(a0);
    sw(record + 12, a0);
    sw(record + 16, a1);
    sw(record + 8, sector);
    sw(record + 4, 0);
    preloadInsert(record);
}

/// Add the texture named `a0` to the preload list (kind 2) with the argument `a1` unless it is already there (@ 0x21c3e8).
pub fn preloadAddTexName(a0: i32, a1: i32) {
    if preloadGetTexName(a0) != 0 {
        return;
    }
    let number = lw(g::numpreload);
    sw(g::numpreload, number + 1);
    let record = number * 24 + preload_table();
    sw(record, 2);
    let sector = fileGetSector(a0);
    sw(record + 12, a0);
    sw(record + 16, a1);
    sw(record + 8, sector);
    sw(record + 4, 0);
    preloadInsert(record);
}

/// Create the texture buffer `a0` of 0x1a18 bytes: it holds `a3` textures of 128 bytes of data and 40 bytes of records, with the sizes `a1` and `a2` (@ 0x2b6a20).
pub fn emCreateTexBuffer(a0: i32, a1: i32, a2: i32, a3: i32) {
    let buffer = a0 * 0x1a18 + g::texbuf;
    sw(buffer + 0x1a0c, a1);
    sw(buffer + 0x1a10, tex_slot_count(a2));
    sw(buffer + 0x1a14, tex_count(a3));
    let data = tex_memory(a0, 0, a3 << 7);
    let count = lw(buffer + 0x1a14);
    sw(buffer, data);
    let records = tex_memory(a0, 1, count * 40 + 15 & -16);
    sw(buffer + 4, records);
    tex_slots_create(a0, buffer);
    emResetTexBuffer(a0);
}

/// Restart everything after a level restart, in the order the game does (@ 0x225e98).
pub fn lvRestartAfter() {
    StatsRestart();
    textRestart();
    soundRestartAfter();
    pickupposRestart();
    particleRestart();
    weatherRestart();
    zbtestRestartAfter();
    starfieldRestart();
    specialRestart();
    specialfxRestart();
    memdbRestart(2);
    ingamemenuRestart();
    bgRestart();
    roomlightRestart();
    padRestart();
    hudRestart();
    gunRestart();
    bulletRestart();
    cartridgesRestart();
    playerRestart();
    musicRestart();
    menuEndMessage();
    decalRestart();
    lvSoundsRestart();
}

/// Back up the map maker's map into the temporary buffer: the map is saved and the edit flags follow its size (0 clears the buffer and the flag 256, up to 0x1ffc sets the flags
/// 0x1100, more than that drops the flag 256 only) (@ 0x2482e8).
pub fn mmBackupMap() {
    let size = mmSaveMapFile(lw(g::mmtempbuffer), 1);
    let flags = lw(g::editflags);
    if size < 0x1ffd {
        let flags_again = lw(g::editflags);
        if size <= 0 {
            memset(lw(g::mmtempbuffer), 0, 0x1ffc);
            sw(g::editflags, lw(g::editflags) & -257);
        } else {
            sw(g::editflags, flags_again | 0x1100);
        }
    } else {
        sw(g::editflags, flags & -257);
    }
}

/// Put the players back at their start position and mark their positions as dirty (@ 0x280100).
pub fn playerDirtyPosReset() {
    let mut i = 0;
    if lw(g::lvnumplayers) > 0 {
        let mut offset = 0;
        loop {
            i += 1;
            let player = lw(g::players) + offset;
            offset += 1820;
            playerSetStartPos(player);
            let count = lw(g::lvnumplayers);
            store_one_tick(player + 340);
            sw(player + 292, 1);
            if i >= count {
                break;
            }
        }
    }
}

/// Draw the story character select page: the highlight of the menu, the select effect and, once the front end's flag is 2, the name of the chosen character (@ 0x23b550).
pub fn storychrsel_pageDraw(a0: i32) {
    let player = signonGet(lw(lw(g::front)));
    let entry = lw(g::signonarray) + player * 2936 + 32;
    if lw(lw(g::front)) > 0 {
        menuitemHighlight(a0);
    }
    fxSelectCharGfx();
    if lbu(lw(lw(g::front)) + 0x3aff28) != 2 {
        return;
    }
    let name = lw((lh(lw(entry + 84) * 44 + g::chrs) << 2) + lw((lw(g::cur_language) << 2) + g::languages));
    drawCharName(a0, 1, name, 1);
}

/// The tick of the demo playback: any button restarts the countdown (0x1518); when it runs out the next of the demos 1 to 7 starts (@ 0x240ea0).
pub fn demoplaybackTick() {
    if joyBut(0) != 0 {
        sw(0x1fc122c, 0x1518);
    }
    let left = lw(0x1fc122c) - 1;
    sw(0x1fc122c, left);
    if left == -1 {
        demoStart(lbu(g::curdemo));
        let next = lbu(g::curdemo) + 1;
        sw(0x1fc11f8, 1);
        sw(0x1fc122c, 0x1518);
        sb(g::curdemo, next);
        if ((next & 255) as u32) >= 8 {
            sb(g::curdemo, 1);
        }
    }
}

/// Preload the three pictures of the loading screen: their names are made from the numbers 1 to 3 into 100 byte slots (@ 0x22a0c0).
pub fn loadingPreload() {
    let mut i = 0;
    loop {
        let name = i * 100 + 0x1fbd078;
        i += 1;
        sprintf(&[name as i64, 0x3a1698_i64, i as i64]);
        preloadAddTexName(name, 0);
        if i >= 3 {
            break;
        }
    }
}

/// The tick of the map maker's settings page: a press of the buttons 2304 leaves the menu; otherwise the menu ticks and its items are made (@ 0x24cbf0).
pub fn settings_pageTick(a0: i32) {
    mm_options_tick_hook(a0);
        if (joyNewBut(0) & 2304) == 0 {
        let menu = mm_options_menu_or(g::settings_menu);
        menutick_24c3a8(a0, menu);
        let made = menumake(lw(a0 + 8), menu);
        sw(a0 + 12, made);
        return;
    }
    mmExitMenu(a0);
}

/// The behaviour of the mansion level's enemies in the story game kinds 2 and 3: when two or more spawned enemies are active the state becomes 1; when fewer than two are,
/// the state becomes 2, the door 14 opens and a sound plays (@ 0x2cbfb0).
pub fn enemyAIMansionHandler1() {
    if ((lw(g::curgameparms + 76) - 1) as u32) < 2 {
        let state = lw(g::story_state1);
        if state == 1 {
            if lw(g::totactivespawnedchars) < 2 {
                sw(g::story_state1, 2);
                doorOpen(14);
                if frames_gate() > 0.0_f32 {
                    soundStart(43, 0);
                    return;
                }
            }
        } else if state < 2 {
            let active = lw(g::totactivespawnedchars);
            if state == 0 {
                if active >= 2 {
                    sw(g::story_state1, 1);
                }
            }
        }
    }
}
