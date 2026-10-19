//! Resets, option changes, prop ticks and other short functions.

use crate::*;

/// @ 0x27e300
pub fn camReset() {
    camRestart();
}

/// @ 0x2cdb00
pub fn timerReset() {
    timerRestart();
}

/// @ 0x2b6ac8
pub fn emRestart() {
    emSelectTexBuffer(0);
}

/// Size of a sign-on (profile) file on the memory card (@ 0x220790).
pub fn signonFileSize() -> i32 {
    memcardFileSize(0xb790)
}

/// Address of the settings inside the first sign-on record (records are 2936 bytes, settings at +128) (@ 0x2201d8).
pub fn settingsGet() -> i32 {
    lw(g::signonarray) + signonGet(0) * 2936 + 128
}

/// A sign-on update is starting in the front end: clear flags 2, 4 and 32 and set 1 of the page flags (@ 0x231d18).
pub fn signonUpdateStart() {
    let front = 0x1fc11f8;
    let flags = lw(front + 4);
    sb(0x3aff4b, 0);
    sw(front + 4, (flags | 1) & -5 & -9 & -33);
}

/// The original floors and walls of a prop (its table `PWFTAB` has 16 bytes a prop: walls and count, then floors and count): the
/// walls (@ 0x266e48) ...
pub fn propGetOriginalWalls(prop: i32, walls: i32, count: i32) {
    let at = prop << 4;
    let n = lw(PWFTAB + at + 4);
    sw(walls, lw(at + PWFTAB));
    sw(count, n);
}

/// ... and the floors (@ 0x266e70).
pub fn propGetOriginalFloors(prop: i32, floors: i32, count: i32) {
    let at = (prop << 4) + PWFTAB;
    let list = lw(at + 8);
    sw(count, lw(at + 12));
    sw(floors, list);
}

/// Start sound buffer `n` again: its second pair of words gets the first pair (@ 0x204478).
pub fn soundResetBuffer(n: i32) {
    let rec = (n << 4) + 0x1fb1790;
    let v = lw(rec + 4);
    sw(rec + 8, lw(rec));
    sw(rec + 12, v);
}

/// Give looping sound `n` (24 bytes) the position `pos` (three floats) (@ 0x2059d0).
pub fn soundSetLoopPos(n: i32, pos: i32) {
    let rec = n * 24 + 0x1fb1768;
    swc1(rec + 8, lwc1(pos));
    sw(rec + 4, 1);
    swc1(rec + 12, lwc1(pos + 4));
    swc1(rec + 16, lwc1(pos + 8));
}

/// Set where the zoom tile window goes (@ 0x250538).
pub fn zoomtileWindowDest(a: i32, b: i32, c: i32, d: i32) {
    sw(0x3b0074, d);
    let dest = 0x1fc40f8;
    sw(dest, a);
    sw(dest + 8, c);
    sw(dest + 4, b);
}

/// The dot product of two unit vectors if it is more than `limit` (they point the same way within a cone), else -1 (@ 0x2c6b00).
pub fn checkInFrontOfDir(a: i32, b: i32, limit: f32) -> f32 {
    let dot = lwc1(a) * lwc1(b) + lwc1(a + 4) * lwc1(b + 4) + lwc1(a + 8) * lwc1(b + 8);
    if limit < dot { dot } else { -1.0 }
}

/// A corpse prop is drawn (bit 0 of its flags set) unless bit 8 is set (@ 0x26ecb8).
pub fn corpsePropTick(prop: i32) {
    let flags = lw(prop + 16);
    if flags & 256 != 0 {
        sw(prop + 16, flags & -2);
    } else {
        sw(prop + 16, flags | 1);
    }
}

/// Whether the current animation has completed: the word at +216 when it plays backwards (+160 set), else the one at +152 (@ 0x215b00).
pub fn curAnimComplete(anim: i32) -> i32 {
    if lw(anim + 160) != 0 { lw(anim + 216) } else { lw(anim + 152) }
}

/// The room of a decal's prop: the room that moving props are in if it is prop 256 (@ 0x2a30c0).
pub fn decalGetRoomFromProp(prop: i32) -> i32 {
    if lw(prop + 8) == 256 { lw(g::s_MovingPropRoom) } else { lw(prop + 12) }
}

/// Whether the left hand holds a gun in the player's current combination (@ 0x2950f0).
pub fn gunLeftHandActive(player: i32) -> i32 {
    (gunLeftHandAble(player) != 0 && lh((lw(player + 260) << 3) + (GUNCOMBOS + 2)) != -1) as i32
}

/// Whether the right hand holds a gun in the player's current combination (@ 0x295140).
pub fn gunRightHandActive(player: i32) -> i32 {
    (gunRightHandAble(player) != 0 && lh((lw(player + 260) << 3) + (GUNCOMBOS + 4)) != -1) as i32
}

/// Whether both hands hold a gun (@ 0x295190).
pub fn gunBothHandsActive(player: i32) -> i32 {
    let left = gunLeftHandActive(player);
    let right = gunRightHandActive(player);
    (left != 0 && right != 0) as i32
}

/// Switch muzzle flash off (the two low flag bits at +292 of the model data) for a character's right (`hand` non-zero) or left gun (@ 0x2944f0).
pub fn gunChrMuzzleOff(chr: i32, hand: i32) {
    let data = lw(chr + 352);
    let gun = lw((if hand != 0 { data + 268 } else { data + 376 }) + 64);
    if gun != 0 {
        let model = lw(gun + 32);
        sh(model + 292, lhu(model + 292) & 0xfffc);
    }
}

/// Flip bits of a pad record's flags (@ 0x264cd8).
pub fn padTogglePadFlag(pad: i32, flag: i32) {
    let flags = lw(pad + 4);
    if flags & flag == 0 {
        sw(pad + 4, flags | flag);
    } else {
        sw(pad + 4, flags & !flag);
    }
}

/// Fill a room light (28 bytes): colour words, two floats (@ 0x2a7fe8).
pub fn roomlightSet(n: i32, a: i32, b: i32, c: i32, f1: f32, f2: f32) {
    let light = lw(g::roomlights) + n * 28;
    sw(light + 20, c);
    sw(light, a);
    swc1(light + 4, f1);
    swc1(light + 8, f2);
    sw(light + 16, b);
}

/// Move on to the next of the 3 depth test slots for drawing, clearing the counters of the one taken (@ 0x2a7058).
pub fn zbtestAdvanceDrawPtr() {
    let slot = lw(0x3b0178);
    sw(0x3b017c, 0);
    sw(0x3b0180, 0);
    sw((slot << 2) + 0x1fea0a0, 0);
    sw(0x3b0184, 0);
    sw(0x3b0178, (slot + 1) % 3);
}

/// @ 0x2a6790
pub fn zbtestEnd() {
    if lw(g::restart) == 0 {
        sw(g::zbuffer, 0);
        let val = g::zbval;
        sw(val, 0);
        sw(val + 8, 0);
        sw(val + 4, 0);
    }
}

/// @ 0x2aff10
pub fn creditsTextReset() {
    let pos = g::creditscampos;
    sw(pos, 0);
    swc1(pos + 8, -10.0);
    sw(g::creditspagetime, 0);
    sw(g::creditsTextIndex, 0);
    sw(pos + 4, 0);
    sw(g::creditscamrotz, 0);
}

/// Turn a prop's angle at +76 by `speed` degrees a tick, wrapping at 360 (@ 0x2790a0 and 0x27a4d8).
fn spin_up(prop: i32, speed: f32) {
    let angle = lwc1(prop + 76) + lwc1(g::updateframesf) * speed;
    swc1(prop + 76, angle);
    if 360.0 <= angle {
        swc1(prop + 76, angle - 360.0);
    }
}

/// @ 0x2790a0
pub fn fanPropTick(prop: i32) {
    spin_up(prop, 5.0);
}

/// @ 0x27a4d8
pub fn polePropTick(prop: i32) {
    spin_up(prop, 1.5);
}

/// Empty loop in the build (@ 0x28d000).
pub fn bulletGfx() {}

/// Empty loop in the build (@ 0x202ff0).
pub fn joyNum() {}

/// Empty loop in the build (@ 0x296ad8).
pub fn particleprintsizes() {}

/// Give the memory of the local block back to the level's free count (@ 0x201e50).
pub fn memLocalFree() {
    let end = lw(g::mem_level_end);
    let free = lw(g::mem_level_free);
    let released = end - lw(g::mem_local_at);
    sw(g::mem_local_at, end);
    sw(g::mem_level_free, free + released);
}

/// Put a CRC after the data of a memory card file: `header` word, then `len` bytes, then the CRC of those (@ 0x208ee8).
pub fn memcardFileMakeCRC(buf: i32, header: i32, len: i32) {
    sw(buf, header);
    let crc = crcCalc(buf + 4, len);
    sw(len + buf + 4, crc);
}

/// Clear the overrides of an object instance (@ 0x260918).
pub fn obInstResetOverride(inst: i32) {
    sw(inst + 324, 0);
    sw(inst + 304, -1);
    sw(inst + 296, -1);
    sw(inst + 300, -1);
    sw(inst + 308, 0);
    sw(inst + 312, 0);
    sw(inst + 316, 0);
    sw(inst + 320, 0);
}

/// @ 0x21a728
pub fn packfileEnd() {
    let used = lw(0x3afea8);
    if lw(g::restart) == 0 {
        let base = lw(0x3afeac);
        sw(g::remember, 0);
        sw(0x3afea8, used - base);
    }
}

/// Set up a zoom box (40 bytes) from a 16-byte rectangle (@ 0x22ef10).
pub fn zoomboxInit(n: i32, rect: i32, value: i32) {
    let b = n * 40 + g::zoomboxes;
    let second = ld(rect + 8);
    sd(b, ld(rect));
    sd(b + 8, second);
    sw(b + 16, value);
    sw(b + 36, 10);
}

/// Works out a controller number and does nothing with it (@ 0x204390).
pub fn GetControllerNumber(_n: i32) {}

/// Cycle the tile page of the map maker: below 16 goes to 16, 16 goes to 0 (@ 0x252c28).
pub fn cycleTilePage() {
    let page = lw(0x3b005c);
    if page >= 16 {
        if page == 16 {
            sw(0x3b005c, 0);
        }
        return;
    }
    sw(0x3b005c, 16);
}

/// Pop a display list tag: the tag becomes an empty one (type 96) and the pushed one is current again (@ 0x2b9028).
pub fn dlPopDmaTag() {
    sb(lw(g::gfxtag) + 3, 96);
    let tag = lw(g::gfxtag);
    let pushed = lw(g::pushgfxtag);
    sh(tag, 0);
    sw(g::gfxtag, pushed);
    sw(tag + 4, 0);
    sw(g::pushgfxtag, 0);
}

/// The memory card dialog polls: with no card chosen (-1) wait for the next poll, else go to state 23 for 60 ticks (@ 0x244d18).
pub fn mcseqPollLoop() {
    let card = lw(0x3b0008);
    sw(0x3affcc, 0);
    if card == -1 {
        sw(0x3affcc, 1);
        return;
    }
    sw(g::mcseq_state, 23);
    sw(0x3affd0, 60);
}

/// Start timing profile slot `n` (28 bytes) (@ 0x2b62c0).
pub fn profileStart(n: i32, a: i32, b: i32) {
    let data = g::profdata;
    let slot = data + n * 28;
    sw(n * 28 + data, 1);
    sw(slot + 4, 0);
    sw(slot + 24, b);
    sw(slot + 20, a);
    sw(slot + 8, 0);
}

/// Set (`on`) or clear a bit of the current player's option flags (the halfword at +2 of its settings).
fn set_player_option(bit: i32, on: bool) {
    let settings = lw(lw(g::currentplayer) + 24);
    let flags = lhu(settings + 2) & !bit;
    sh(settings + 2, flags);
    if on {
        sh(settings + 2, flags | bit);
    }
}

/// Aim mode option (bit 2, set when the menu value is 0) (@ 0x20f498).
pub fn aimmode_onchange(item: i32) {
    set_player_option(2, lw(item) == 0);
}

/// Auto-aim option (bit 128) (@ 0x20f7f0).
pub fn autoaim_onchange(item: i32) {
    set_player_option(128, lw(item) != 0);
}

/// Crouch mode option (bit 4, set when the menu value is 0) (@ 0x20f4c8).
pub fn crouchmode_onchange(item: i32) {
    set_player_option(4, lw(item) == 0);
}

/// Inverted look option (bit 1) (@ 0x20f528).
pub fn invlook_onchange(item: i32) {
    set_player_option(1, lw(item) != 0);
}

/// Look-ahead option (bit 8) (@ 0x20f4f8).
pub fn lookahead_onchange(item: i32) {
    set_player_option(8, lw(item) != 0);
}

/// The debug memory tracker's frame step: shift its three counters on and note whether the free memory was asked for (@ 0x201f50).
pub fn memdbTick() {
    let a = lw(0x3afe0c);
    if lw(g::printfree) != 0 {
        sw(g::printfree, 0);
    }
    let b = lw(0x3afe10);
    sw(0x3afe10, a);
    sw(0x3afe14, b);
    sw(0x3afe0c, b);
}

/// A fan shadow turns like the fan (@ 0x2790e0).
pub fn fanshadowPropTick(prop: i32) {
    spin_up(prop, 5.0);
}
