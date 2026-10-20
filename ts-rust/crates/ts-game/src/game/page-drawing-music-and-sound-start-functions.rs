//! Small functions for the menu pages, the music stream and starting sounds.

use crate::*;

/// Draw the arcade options page: the background and the bottom bar at (364, 206) (@ 0x239060).
pub fn arcadeoptions_pageDraw(a0: i32) {
    pagebackgroundGfx_230688(a0);
    drawBotSet(364, 206);
}

/// Draw the character select page: the background and the split screen part (@ 0x237e68).
pub fn characterselect_pageDraw(a0: i32) {
    pagebackgroundGfx_230688(a0);
    splitpageDraw(a0, g::chrplayer_page);
}

/// Toggle the first cheat and play the sound that says whether it is on (142) or off (140) (@ 0x203660).
pub fn cheat1() {
    let value = 1 - lw(g::val1);
    let sound = if value == 0 { 142 } else { 140 };
    sw(g::val1, value);
    soundStart(sound, 0);
}

/// The long name of the memory card save slot being worked on, or 0 when none is (@ 0x243428).
pub fn mcseqSlotGetLongName() -> i32 {
    let slot = lw(0x3b0008);
    if slot != -1 {
        mcseqSlotMakeName(slot)
    } else {
        0
    }
}

/// Whether the music stream has stopped (the answer of its state call is 0) (@ 0x205b68).
pub fn musicDefinitelyStopped() -> i32 {
    let state = stream_RPC(0x80e0, 0);
    ((state as u32) < 1) as i32
}

/// Start the sound `a0` at the position `a1` with the range 50 and the flags 1024 (@ 0x205518).
pub fn soundStart(a0: i32, a1: i32) -> i32 {
    soundStartRange(a0, a1, 1024, 5e+01_f32)
}

/// Draw the story character page: the background and the split screen part (@ 0x23ba08).
pub fn storychr_pageDraw(a0: i32) {
    pagebackgroundGfx_230688(a0);
    splitpageDraw(a0, g::storychrsel_page);
}

/// Close the music stream if it is open, and forget the tune (@ 0x205aa0).
pub fn musicClose() {
    if lw(g::musicopened) != 0 {
        sw(g::musicopened, 0);
        stream_RPC(48, 0);
    }
    sw(g::tunenum, 0);
}

/// The wind at a position: the weather time shifted for the position, the wind written to `a2` (@ 0x2a8450).
pub fn weatherGetWindAtPosition(a0: i32, _a1: i32, a2: i32) {
    let shift = weatherCalcPositionalTime(a0);
    weatherCalcWindAtTime(a2, lwc1(g::weather + 60) + shift);
}

/// Draw the map name page: its background and the string editor (@ 0x24d1f0).
pub fn mapnameedit_pageDraw(a0: i32) {
    pagebackgroundGfx_24c670(a0);
    stringeditDraw(0, a0, 1);
}

/// The length of the card save slot's short name: the length of the language's text for the slot plus one added to the name made for it (@ 0x243458).
pub fn mcseqSlotGetShortName() -> i32 {
    let prefix = strlen(lw(lw((lw(g::cur_language) << 2) + g::languages) + 1460));
    let name = mcseqSlotMakeName(lw(0x3b0008));
    name + (prefix + 1)
}

/// Start the flash of the character select box `a0` and play its sound (@ 0x238e00).
pub fn fxSelectCharFlashInit(a0: i32) {
    let at = a0 << 2;
    sw(at + 0x1fc12b8, 1);
    sw(at + 0x1fc12c8, 0);
    soundStart(146, 0);
}

/// Set the volume of the music for the front end from the music volume setting (0 when the music is off, half in the level 102) (@ 0x205bf8).
pub fn musicSetFrontEndVolume() {
    let volume = lw(g::gv_musicvolume);
    let scaled = if lw(g::gv_playmusic) == 0 { 0 } else { ((volume << 14) - volume) >> 4 };
    let level = if (lw(g::lvlevelnum) ^ 102) == 0 { scaled >> 1 } else { scaled };
    musicSetVol(level << 16 | level, 0);
}

/// Draw the sign-on page's background and, unless the flag 512 is set, the split screen part (@ 0x23f140).
pub fn signon_pageDrawBefore(a0: i32) {
    pagebackgroundGfx_230688(a0);
    if (lw(0x1fc11fc) & 512) != 0 {
        return;
    }
    splitpageDraw(a0, g::signonplr_page);
}

/// Draw the statistics update page: the background and the message of the player when it has one (@ 0x2400b8).
pub fn statsupdate_pageDraw(a0: i32) {
    pagebackgroundGfx_230688(a0);
    let player = lw(0x3aff38);
    if lw(player * 260 + 0x1fc1488) == 0 {
        return;
    }
    splitMessageDraw(player);
}

/// Take the page out of the stack of pages of the menu `a0`: it is recorded at `a1`, the sound 142 plays and the depth drops (@ 0x20e1e8).
pub fn outPage(a0: i32, a1: i32) {
    let menu = a0 * 92 + 0x1fb3240;
    sw(menu + (lhu(menu + 4) << 2) + 52, a1);
    soundStart(142, 0);
    sh(menu + 2, lhu(menu + 2) - 1);
}

/// Toggle the cheat `a0` and play the sound that says whether it is on (142) or off (140) (@ 0x203628).
pub fn set_cheat(a0: i32) {
    let at = a0 + g::pcarray;
    let value = 1 - lbu(at);
    sb(at, value);
    let sound = if (value << 24) == 0 { 142 } else { 140 };
    soundStart(sound, 0);
}

/// Delete the save entry `a1` from the list of 15 entries of 2936 bytes at `a0`: those after it move up, and the flag at +0xac08 is cleared (@ 0x220738).
pub fn signonFileDelete(a0: i32, a1: i32) {
    if a1 < 15 {
        let entry = a1 * 2936 + a0;
        memcpy(entry, entry + 2936, (15 - a1) * 2936);
    }
    sb(a0 + 0xac08, 0);
}

/// Press a control panel prop `a0`: the first time, the sound 9 plays and the model's flag 4 is replaced by 8 (@ 0x275170).
pub fn controlActivate(a0: i32) {
    if lw(g::controlPanelActivated) == 0 {
        sw(g::controlPanelActivated, 1);
        soundStart(9, a0 + 48);
        let model = lw(a0 + 32);
        sh(model + 292, lhu(model + 292) & 0xfffb | 8);
    }
}

/// Show or hide the goodie message: with a message the sound 145 plays and it is kept to be shown, with 0 it is hidden (@ 0x240868).
pub fn goodieMessage(a0: i32) {
    if a0 != 0 {
        soundStart(145, 0);
        sw(0x3affb4, a0);
        sw(0x3affb0, 1);
    } else {
        sw(0x3affb0, 0);
    }
}

/// Draw the map description entry page: its background and, when its flag is 1, the string editor (@ 0x24cc68).
pub fn mmenterdesc_pageDraw(a0: i32) {
    pagebackgroundGfx_24c670(a0);
    if lbu(0x3b0038) != 1 {
        return;
    }
    stringeditDraw(0, a0, 1);
}

/// Preload the star field's two textures in the level 17 (@ 0x2a8628).
pub fn starfieldPreload() {
    if lw(g::lvlevelnum) != 17 {
        return;
    }
    preloadAddTexName(0x3a9608, 0);
    preloadAddTexName(0x3a9630, 0);
}

/// The strength of the wind at a position along the direction `a2`: the wind vector at the time for the position is dotted with it (@ 0x2a8488).
pub fn weatherCalcWindStrengthInDir(a0: i32, _a1: i32, a2: i32) -> f32 {
    let frame = Frame::new(48);
    let sp = frame.sp();
    let shift = weatherCalcPositionalTime(a0);
    weatherCalcWindAtTime(sp, lwc1(g::weather + 60) + shift);
    lwc1(sp) * lwc1(a2) + lwc1(sp + 4) * lwc1(a2 + 4) + lwc1(sp + 8) * lwc1(a2 + 8)
}

/// Leave the map maker: the map is backed up and the game is reset to the menu (102) (@ 0x24ee10).
pub fn exitmapmaker() {
    let frame = Frame::new(128);
    let sp = frame.sp();
    mmBackupMap();
    memset(sp, 0, 112);
    sw(sp + 64, 102);
    gameDeferedReset(sp);
}

/// Put the page `a1` into the stack of pages of the menu `a0`: it is recorded, the sound 140 plays and the depth rises (@ 0x20e190).
pub fn inPage(a0: i32, a1: i32) {
    let menu = a0 * 92 + 0x1fb3240;
    sw(menu + (lhu(menu + 4) << 2) + 52, a1);
    soundStart(140, 0);
    sh(menu + 2, lhu(menu + 2) + 1);
}

/// The tick of the main set-up page: the in-game menu ticks with the main set-up menu, and the count of items grows by the items made (@ 0x210fc8).
pub fn mainsetup_pageTick(a0: i32) {
    let menu = g::mainsetup_menu;
    ingame_menutick(a0, menu);
    let made = std_menumake(lw(a0 + 8), menu);
    sw(a0 + 12, lw(a0 + 12) + made);
}

/// Start the music: resume it if it was paused (state 1), otherwise play it; the state becomes 2 (@ 0x205ad8).
pub fn musicStart() {
    if lw(g::musicstate) != 1 {
        stream_RPC(0x8050, 0);
    } else {
        stream_RPC(0x8060, 0);
    }
    sw(g::musicstate, 2);
}

/// Pause the music when it is playing (the state call answers 0x5000): the state becomes 1 (@ 0x205b20).
pub fn musicStop() {
    let state = stream_RPC(0x80e0, 0);
    if state == 0x5000 {
        stream_RPC(0x8070, 0);
        sw(g::musicstate, 1);
    }
}

/// Load the texture `a0` named by `a1` into the buffer at `a2`: the last texture slot of the current texture buffer is taken first (@ 0x2b78d8).
pub fn emLoadTextureAddr(a0: i32, a1: i32, a2: i32) {
    let buffer = lw(g::thistexbuf);
    let slot = lw(buffer + 0x1a08);
    sw(a2, slot);
    sw(buffer + 0x1a08, slot - 1);
    emLoadTexture(lw(a2), a0, a1);
}
