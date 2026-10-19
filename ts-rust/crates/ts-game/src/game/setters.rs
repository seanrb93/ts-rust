//! One-line setters, getters and resets of the game's global state.  A few of the words they touch have no name in the symbol table yet;
//! they are given by address.

use crate::*;

/// Start of the scratch allocator at the top of the scratchpad address range (@ 0x21a7a0).
pub fn AllocReset() {
    sw(0x3afeb4, 0x70000000);
}

/// A hit animation's blend value is set from a constant of the game (@ 0x215b40).
pub fn animHitReact(anim: i32) {
    swc1(anim + 200, lwc1(0x3ac844));
}

/// The camera goes back to the game view (@ 0x27e2f8).
pub fn camRestart() {
    sw(g::cammode, 0);
}

/// @ 0x21f310
pub fn challengeRestart() {
    sw(0x3aff00, 0);
}

/// The part that might have been hit becomes the part that was hit (@ 0x285050).
pub fn chrConfirmLastHitPart() {
    sw(g::s_LastHitPart_3afa2c, lw(g::s_PossibleLastHitPart_3afa30));
}

/// @ 0x2ae690
pub fn creditsEnd() {
    sw(g::currenttunneldata, 0);
}

/// @ 0x20db20
pub fn debugmenuSetJoyMode(mode: i32) {
    sw(g::debugmenujoymode, mode);
}

/// @ 0x20db28
pub fn debugmenuGetJoyMode() -> i32 {
    lw(g::debugmenujoymode)
}

/// The demo is over: back to the front end (game state 5) (@ 0x2257d8).
pub fn demoEnd() {
    sw(g::gamestate, 5);
}

/// Colour that fixed-colour drawing uses (@ 0x2b7ea0).
pub fn dlSetFixedRGBA(rgba: i32) {
    sw(0x3b01a4, rgba);
}

/// Pick texture buffer `n` (records of 0x1a18 bytes) (@ 0x2b6aa8).
pub fn emSelectTexBuffer(n: i32) {
    sw(g::thistexbuf, n * 0x1a18 + g::texbuf);
}

/// Select a front end definition (44 bytes each) (@ 0x22cb20).
pub fn frontSet(n: i32) {
    sw(g::front, n * 44 + g::frontdefs);
}

/// Address of a front end definition (@ 0x22cb40).
pub fn frontGet(n: i32) -> i32 {
    n * 44 + g::frontdefs
}

/// @ 0x28d820
pub fn gunReset() {
    gunRestart();
}

/// @ 0x244cf0
pub fn mcseqController(controller: i32) {
    sw(0x3affdc, controller);
}

/// Ask for the free memory to be printed (@ 0x201f98).
pub fn memdbPrintFree() {
    sw(g::printfree, 1);
}

/// Set the current light colour of the map maker dialogs (16 slots) (@ 0x24fac0).
pub fn mmDialogsSetCurLightCol(rgba: i32) {
    sw(((lw(g::curlightcol) & 15) << 2) + 0x1fc4040, rgba);
}

/// Whether models are clipped while loaded (@ 0x25eb28).
pub fn obLoadSetClip(clip: i32) {
    sw(g::doclip, clip);
}

/// @ 0x230140
pub fn pageRoot() {
    sw(0x3aff2c, 0);
}

/// Profile slot `n` (28 bytes) is no longer being timed (@ 0x2b6300).
pub fn profileFinish(n: i32) {
    sw(n * 28 + g::profdata, 0);
}

/// Set the word at +12 of an animation (@ 0x214978).
pub fn setAnimProp(anim: i32, value: i32) {
    sw(lw((anim << 2) + g::pAnimations) + 12, value);
}

/// @ 0x220208
pub fn settingsNew(settings: i32) {
    sw(settings, 0);
}

/// @ 0x220930
pub fn signonSet(n: i32, value: i32) {
    sw((n << 2) + 0x1fbc158, value);
}

/// @ 0x2049b8
pub fn soundBatchReset() {
    sw(g::soundbatchlen, 0);
}

/// Pick sound buffer `n` (16 bytes each) (@ 0x204460).
pub fn soundSelectBuffer(n: i32) {
    sw(g::thissoundbuf, (n << 4) + 0x1fb1790);
}

/// Clear message `n` (260 bytes each) (@ 0x230ae0).
pub fn splitMessageClear(n: i32) {
    sw(n * 260 + 0x1fc1488, 0);
}

/// Select a text box (60 bytes each) for editing (@ 0x22fed8).
pub fn stringeditSet(n: i32) {
    sw(g::textbox, n * 60 + 0x1fc09a8);
}

/// @ 0x2cdaf8
pub fn timerRestart() {
    sw(g::numtimers, 0);
}

/// The weapon change option was changed: copy the menu item's value into the current player's settings (+88) (@ 0x20f820).
pub fn wpchange_onchange(item: i32) {
    sw(lw(lw(g::currentplayer) + 24) + 88, lw(item));
}

/// Always 0 in the build (@ 0x21ccc8).
pub fn UnlockCombo() -> i32 {
    0
}

/// @ 0x25a718
pub fn bglightReset() {
    sw(g::numbglights, 0);
    bglightRestart();
}

/// Address of challenge definition `n` (64 bytes each) (@ 0x21f048).
pub fn challengeGetSetupNum(n: i32) -> i32 {
    (n << 6) + g::challengedefs
}

/// Width of the score on the HUD: 100 in mode 1, else 54 (@ 0x2b1598).
pub fn hudScoreWidth() -> i32 {
    if lw(g::curgameparms + 72) == 1 { 100 } else { 54 }
}

/// Whether `addr` is above the end of the game's code, so that it is text rather than a function pointer (@ 0x20ca90).
pub fn istext(addr: i32) -> i32 {
    (0x2cde08_u32 < addr as u32) as i32
}

/// How many slots the multitap on port `n` has (@ 0x203610).
pub fn joyMtapGetSlots(n: i32) -> i32 {
    lw((n << 2) + g::max_slots)
}

/// The buttons newly pressed on a player's controller (controllers are 384 bytes, state at +292) (@ 0x203e70).
pub fn joyNewButLOCAL(player: i32) -> i32 {
    lw(lw((player << 2) + g::playercontrollers) * 384 + (g::controllers + 292))
}

/// @ 0x22a0b0
pub fn loadingReset() {
    sw(0x3aff0c, 0);
    sw(0x3aff08, 2);
}

/// Whether the level creation flags have the challenge bit (512) (@ 0x228e60).
pub fn lvCheckCreateFlagsChallenge(flags: i32) -> i32 {
    (flags & 512 != 0) as i32
}

/// Start formatting the memory card (@ 0x208c48).
pub fn mcardFormat() {
    sw(g::mcardresult, 0);
    sw(g::mcardstate, 6);
}

/// @ 0x208eb0
pub fn mcardGetResult() -> i32 {
    lw(g::mcardresult)
}

/// Number of 1 KB clusters an icon takes: its size rounded up to 1 KB, plus one cluster (@ 0x207a30).
pub fn mcardIconClusters(n: i32) -> i32 {
    ((lw((n << 2) + 0x3afe60) + 1023 & -1024) + 1024) >> 10
}

/// @ 0x208c38
pub fn mcardSelectCard(port: i32, slot: i32) {
    sw(g::port, port);
    sw(g::slot, slot);
}

/// @ 0x244da0
pub fn mcseqGetCurCard() -> i32 {
    lw(0x3b0008)
}

/// @ 0x244a80
pub fn mcseqGetRequiredSpace() -> i32 {
    lw(0x3b0010)
}

/// @ 0x244a88
pub fn mcseqGetResult() -> i32 {
    lw(0x3afff8)
}
