//! Small functions that forward to another function, with a constant or a global for it.

use crate::*;

/// The tick of the link to the other console: the pads are polled (@ 0x2cddd0).
pub fn ilinkTick() {
    joyTick();
}

/// Preload for the in-game menu: the user interface's (@ 0x2124a0).
pub fn ingamemenuPreload() {
    guiPreload();
}

/// Reset the level's sounds, which restarts them (@ 0x226be0).
pub fn lvSoundsReset() {
    lvSoundsRestart();
}

/// Reset the music, which restarts it (@ 0x205ce8).
pub fn musicReset() {
    musicRestart();
}

/// The music volume of the menu changed: set the front end's volume (@ 0x231288).
pub fn musicvolume_onchange() {
    musicSetFrontEndVolume();
}

/// End the game: the state becomes 2 and the level ends (@ 0x21f038).
pub fn gameEnd() {
    sw(g::gamestate, 2);
    lvEnd();
}

/// The tick of the special effects: the spawning ones and the groups (@ 0x2adb50).
pub fn specialfxTick() {
    spawnfxTick();
    specialfxGroupsTick();
}

/// The tick of the credits: the time moves on by the frames times a rate, then the background and the text tick (@ 0x2ae698).
pub fn creditsTick() {
    swc1(g::creditstime, ps2float::add(lwc1(g::creditstime), ps2float::mul(lwc1(g::updateframesf), lwc1(0x3ad750))));
    creditsBgTick();
    creditsTextTick();
}

/// Draw the game type page: only the background (@ 0x23dd68).
pub fn gametype_pageDraw(a0: i32) {
    pagebackgroundGfx_230688(a0);
}

/// Draw the number of players page: only the background (@ 0x23fd68).
pub fn nplayers_pageGfx(a0: i32) {
    pagebackgroundGfx_230688(a0);
}

/// Menu entry that sets the cheat 1 (@ 0x203688).
pub fn setcheat1() {
    set_cheat(1);
}

/// Menu entry that sets the cheat 2 (@ 0x203690).
pub fn setcheat2() {
    set_cheat(2);
}

/// Menu entry that sets the cheat 3 (@ 0x203698).
pub fn setcheat3() {
    set_cheat(3);
}

/// Menu entry that sets the cheat 4 (@ 0x2036a0).
pub fn setcheat4() {
    set_cheat(4);
}

/// Menu entry that sets the cheat 5 (@ 0x2036a8).
pub fn setcheat5() {
    set_cheat(5);
}

/// Open the debug menu with the pad mode 1 (@ 0x20db48).
pub fn debugmenuStart() {
    sw(g::debugmenujoymode, 1);
    menuStart(g::debugmenu);
}

/// Make a new gun prop: the same as a prop (@ 0x269b10).
pub fn gunNew(a0: i32, f12: f32, f13: f32, f14: f32, f15: f32) -> i32 {
    propNew(a0, f12, f13, f14, f15)
}

/// Preload for the map maker: the tile zoom, the zoom box, the interface and the built-in maps (@ 0x2483e0).
pub fn mapmakerPreload() {
    zoomtilePreload();
    zoomboxPreload();
    guiPreload();
    mmfilePreloadBuiltinMaps();
}

/// End the music: stop and close it when a tune is playing (@ 0x205cf0).
pub fn musicEnd() {
    if lw(g::tunenum) == 0 {
        return;
    }
    musicStopClose();
}

/// Preload for the pickup position marker: its texture (@ 0x272a78).
pub fn pickupposPreload() {
    preloadAddTexName(0x3a8028, 0);
}

/// Start a looping sound with the default range 50 (@ 0x2058c8).
pub fn soundStartLoop(a0: i32, a1: i32) {
    soundStartLoopRange(a0, a1, 5e+01_f32);
}

/// Preload for the zoom box: the texture file (@ 0x22ece0).
pub fn zoomboxPreload() {
    preloadAddTexName(g::texturefile, 0);
}

/// Start loading a save from the sign-on page: the cards are marked as not checked, then the load tries (@ 0x2317b0).
pub fn signonLoadStart(a0: i32) {
    mcseqUnvalidateCards();
    signonLoadRetry(a0);
}

/// Make the number that names a saved file: the clock time in the low 26 bits and the frame count modulo 60 above (@ 0x220270).
pub fn signonMakeFileId() -> i32 {
    let frames = lw(g::tickframenum).wrapping_rem(60);
    let time = rtcGetTime();
    time & 0x3ffffff | frames << 26
}

/// Leave the level through a deferred reset: 101 for the first game kind, 102 for the others (@ 0x20d718).
pub fn exitlevel() {
    let frame = Frame::new(128);
    let sp = frame.sp();
    clear_reset_block(sp);
    sw(sp + 64, if (lw(g::lvgamenum) ^ 1) == 0 { 101 } else { 102 });
    gameDeferedReset(sp);
}

/// Set the volume of the music stream (`a1` 0 for the left channel pair, otherwise the other) to `a0` in both halves (@ 0x205bd8).
pub fn musicSetVol(a0: i32, a1: i32) {
    let command = if a1 == 0 { 176 } else { 192 };
    stream_RPC(command, a0 << 16 | a0);
}

/// Add the model of the prop `a0` to what is preloaded (@ 0x267220).
pub fn propAddPreLoad(a0: i32) {
    preloadAddOb(lw(a0 * 28 + (PROPTABLE + 12)), 0);
}
