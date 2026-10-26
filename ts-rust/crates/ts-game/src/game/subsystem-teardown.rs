//! More small functions: ends of subsystems that free their state, particle and instance setters, fan props, memory card sequence state.

use crate::*;

/// @ 0x244d48
pub fn mcseqPollCount() -> i32 {
    lw(0x3affcc)
}

/// Size a memory card file takes: its data plus an 8-byte header (@ 0x208eb8).
pub fn memcardFileSize(size: i32) -> i32 {
    size + 8
}

/// Clear the on-screen message (@ 0x20d448).
pub fn menuEndMessage() {
    sw(g::in_message, 0);
    sw(g::in_message_ticker, 0);
}

/// State of message `n` (records of 1040 bytes) (@ 0x2312c0).
pub fn messageGetState_2312c0(n: i32) -> i32 {
    lw(n * 1040 + 0x1fc1dcc)
}

/// @ 0x24c908
pub fn messageGetState_24c908() -> i32 {
    lw(0x3b0030)
}

/// Current light colour of the map maker dialogs (@ 0x24fae0).
pub fn mmDialogsGetCurLightColRGB() -> i32 {
    lw(((lw(g::curlightcol) & 15) << 2) + 0x1fc4040)
}

/// No object instances at the start of a level (@ 0x25d188).
pub fn obRestart() {
    sw(g::num_instances, 0);
    sw(g::num_deleted_instances, 0);
}

/// @ 0x266d58
pub fn propControlPanelActivated() -> i32 {
    lw(g::controlPanelActivated)
}

/// Make sound `n` the current one: its number and its record (36 bytes each, from `sounds` + 8) (@ 0x206000).
pub fn setcursound(n: i32) {
    sw(g::snd, n);
    sw(g::snd + 4, n * 36 + (g::sounds + 8));
}

/// @ 0x220a30
pub fn signonGet(n: i32) -> i32 {
    lw((n << 2) + 0x1fbc158)
}

/// @ 0x2048f8
pub fn soundSetChannel(core: i32, voice: i32) {
    sw(g::sound_core, core);
    sw(g::sound_voice, voice);
}

// The end of a subsystem forgets its table, unless the level is being restarted (the data then stays for the next run).

/// @ 0x285038
pub fn chrEnd() {
    if lw(g::restart) == 0 {
        sw(g::chrinfo, 0);
    }
}

/// @ 0x296a90
pub fn particleEnd() {
    if lw(g::restart) == 0 {
        sw(g::pgroups, 0);
    }
}

/// @ 0x2a7a38
pub fn roomlightEnd() {
    if lw(g::restart) == 0 {
        sw(g::roomlights, 0);
    }
}

/// @ 0x277be0
pub fn specialEnd() {
    if lw(g::restart) == 0 {
        sw(g::levelspecials, 0);
    }
}

/// @ 0x2645a0
pub fn padEnd() {
    if lw(g::restart) == 0 {
        sw(g::levelpads, 0);
        sw(g::levellinks, 0);
    }
}

/// Forget the level and local memory blocks (@ 0x201c10).
pub fn memEnd() {
    if lw(g::restart) == 0 {
        sw(g::mem_level_at, 0);
        sw(g::mem_level_end, 0);
        sw(g::mem_level_free, 0);
        sw(g::mem_local_at, 0);
    }
}

/// Mark a character's gun muzzle as flashing: flag 1 of the halfword at +292 of its model data (@ 0x28e348).
pub fn chrMuzzleFlash(chr: i32) {
    let data = lw(lw(chr + 64) + 32);
    sh(data + 292, lhu(data + 292) | 1);
}

/// @ 0x241540
pub fn frontmpgSetFlags(flags: i32) {
    let front = 0x1fc11f8;
    sw(front + 4, lw(front + 4) | flags);
}

/// Set the light of a map maker tile: two bytes at +28 / +29 and a colour at +32 (@ 0x246308).
pub fn mmSetTileLight(tile: i32, a: i32, b: i32, colour: i32) {
    sb(tile + 29, b);
    sb(tile + 28, a);
    sw(tile + 32, colour);
}

/// @ 0x296e40
pub fn particleSetViewmask(particle: i32, mask: i32) {
    if particle != 0 {
        sw(particle + 48, mask);
    }
}

/// @ 0x296eb0
pub fn particleUpdateGroundHeight(particle: i32, height: f32) {
    if particle != 0 {
        swc1(particle + 40, height);
    }
}

/// Move a particle to the point `pos` (three floats) (@ 0x296e88).
pub fn particleUpdatePos(particle: i32, pos: i32) {
    if particle != 0 {
        swc1(particle + 28, lwc1(pos));
        swc1(particle + 32, lwc1(pos + 4));
        swc1(particle + 36, lwc1(pos + 8));
    }
}

/// @ 0x280770
pub fn playerAutoAimRestart() {
    sw(0x3b0134, 0);
    sw(g::nexttestfrom, 0);
    sw(g::nexttestto, 0);
}

/// Give a special effect spawn back (mark it free with -1) (@ 0x2ae1c0).
pub fn specialfxFreeSpawn(spawn: i32) {
    if spawn != 0 {
        sw(spawn, -1);
    }
}

/// The depth tests cycle through 3 slots: advance the add pointer (@ 0x2a7008).
pub fn zbtestAdvanceAddPtr() {
    sw(0x3b0170, (lw(0x3b0170) + 1) % 3);
}

/// Advance the test pointer (@ 0x2a7030).
pub fn zbtestAdvanceTestPtr() {
    sw(0x3b0174, (lw(0x3b0174) + 1) % 3);
}

/// Set the speed of the current animation, at +140 (@ 0x215b20).  (The original first clears a negative speed, then stores the new one.)
pub fn curAnimSpeed(anim: i32, speed: f32) {
    if lwc1(anim + 140) < 0.0 {
        swc1(anim + 140, 0.0);
    }
    swc1(anim + 140, speed);
}

/// Push a display list tag: the current one is kept as the pushed one (@ 0x2b9018).
pub fn dlPushDmaTag(tag: i32) {
    let current = lw(g::gfxtag);
    sw(g::gfxtag, tag);
    sw(g::pushgfxtag, current);
}

/// Set the zoom limits of a player from gun `gun`'s table entry (@ 0x2922b0).
pub fn gunSetPlayerZoom(player: i32, gun: i32) {
    let entry = gun * 400 + g::guns;
    swc1(player + 268, lwc1(entry + 44));
    swc1(player + 272, lwc1(entry + 48));
}

/// The buttons held on a player's controller that are not in the mask at +340 (@ 0x203ea0).
pub fn joyButLOCAL(player: i32) -> i32 {
    let c = lw((player << 2) + g::playercontrollers) * 384 + g::controllers;
    lw(c + 288) & !lw(c + 340)
}

/// Ask the memory card driver for the free space of an entry (@ 0x208c58).
pub fn mcardGetEntSpace(a: i32, b: i32) {
    sw(0x3afe44, a);
    sw(0x3afe48, b);
    sw(g::mcardstate, 8);
    sw(g::mcardresult, 0);
}

/// Add an option to the memory card dialog's list (@ 0x244cf8).
pub fn mcseqAddOption(option: i32) {
    let n = lw(0x3affd4);
    sw((n << 2) + 0x3affd8, option);
    sw(0x3affd4, n + 1);
}

/// Ask the memory card driver to check the cards (@ 0x208c18).
pub fn mcardCheckCards(a: i32, b: i32, c: i32) {
    sw(0x3afe38, a);
    sw(0x3afe3c, b);
    sw(0x3afe40, c);
    sw(g::mcardstate, 1);
    sw(g::mcardresult, 0);
}

/// Pick card `n` in the memory card dialog (@ 0x244da8).
pub fn mcseqSetCurCard(n: i32) {
    let flags = lw(g::mcseq_flags);
    sw((n << 2) + 0x1fc3710, 1);
    sw(g::mcseq_flags, flags & -7);
    sw(0x3b0008, n);
}

/// Close the memory card dialog's window (@ 0x244a60).
pub fn mcseqWindowClose() {
    let front = lw(g::front);
    if lbu(0x3affb8) != 0 {
        sw(front + 20, lw(front + 20) - 1);
    }
    sb(0x3affb8, 0);
}

/// Largest size of a map maker file: the directory and 0xc8000 bytes (@ 0x2538f0).
pub fn mmfileMaxSize() -> i32 {
    mmfileDirSize() + 0xc8000
}

/// Offset of map maker file `n` in the card file: after the directory, 8 KB each (@ 0x253998).
pub fn mmfileOffset(n: i32) -> i32 {
    mmfileDirSize() + (n << 13)
}

/// Take `size` bytes from the debug allocator (@ 0x201f78).
pub fn memdbAlloc(size: i32) -> i32 {
    let at = lw(0x3afe14);
    let total = lw(g::dballoc) + size;
    sw(0x3afe14, at + size);
    sw(g::dballoc, total);
    at
}

/// Write a chunk header (two words) into a map maker file buffer at `offset`; returns the offset after it (@ 0x247840).
pub fn mmWriteChunk(buffer: i32, offset: i32, kind: i32, size: i32) -> i32 {
    let at = buffer + offset;
    sw(at + 4, size);
    sw(at, kind);
    offset + 8
}

/// Override the colour of an object instance (@ 0x264008).
pub fn obInstSetOverrideCol(inst: i32, a: i32, b: i32, c: i32, amount: f32) {
    sw(inst + 324, c);
    swc1(inst + 312, amount);
    sw(inst + 316, a);
    sw(inst + 320, b);
}

/// Override the draw states of an object instance (@ 0x263ff0).
pub fn obInstSetOverrideStates(inst: i32, a: i32, b: i32, c: i32, d: i32) {
    sw(inst + 296, a);
    sw(inst + 308, d & 255);
    sw(inst + 300, b);
    sw(inst + 304, c);
}

/// Remember what was hit last: the instance, part, chunk and polygon (@ 0x263718).
pub fn obSetLastHitInst(inst: i32, part: i32, chunk: i32, poly: i32) {
    sw(g::s_pLastHitInst, inst);
    sw(g::s_LastHitPart_3aeaa0, part);
    sw(g::s_LastHitChunk, chunk);
    sw(g::s_LastHitPoly, poly);
}

/// Delete an object instance: clear its fields and count it as deleted (@ 0x260a58).
pub fn obInstDelete(inst: i32) {
    let n = lw(g::num_deleted_instances);
    sw(inst + 28, 0);
    sw(inst, 0);
    sw(g::num_deleted_instances, n + 1);
    sw(inst + 24, 0);
}

/// Begin deleting a sign-on file in the memory card front end (@ 0x2328c0).
pub fn signonDeleteStart() {
    sb(0x3aff4b, 0);
    let front = 0x1fc11f8;
    sw(front + 4, (lw(front + 4) | 1) & -9);
}

/// Time at a position for the weather: the position divided by the weather's scale on each axis, summed (@ 0x2a8078).
pub fn weatherCalcPositionalTime(pos: i32) -> f32 {
    let w = g::weather;
    (lwc1(pos)).pdiv(lwc1(w + 48)) + (lwc1(pos + 4)).pdiv(lwc1(w + 52)) + (lwc1(pos + 8)).pdiv(lwc1(w + 56))
}

/// Turn a character to the angle `angle`: the angle and its record's copy at +2972, and no turn speed (@ 0x236988).
pub fn rotateChar(chr: i32, angle: f32) {
    let data = lw(chr + 352);
    swc1(chr + 84, angle);
    swc1(data + 2972, angle);
    sw(chr + 80, 0);
}

/// Select font `n` (24 bytes each) and return the number of the one that was selected (@ 0x202270).
pub fn textSelect(n: i32) -> i32 {
    let fonts = g::fonts;
    let previous = (lw(g::curfont) - fonts).wrapping_mul(-0x55555555) >> 3;
    sw(g::curfont, n * 24 + fonts);
    previous
}

/// Set the zoom tile window: x, y and width; the height follows the 640x224 field's proportion (@ 0x250500).
pub fn zoomtileWindow(x: i32, y: i32, width: i32) {
    sw(0x3b0064, x);
    sw(0x3b0068, y);
    sw(0x3b006c, width);
    sw(0x3b0074, 0);
    sw(0x3b0070, (width * 224).wrapping_div(640));
}

/// The display offset setting changed: write it into the screen registers' x position (@ 0x231240).
pub fn displayoffsetx_onchange(item: i32) {
    let db = g::db;
    let keep = ld(db + 64) & -0x1000_i64;
    sd(db + 24, ld(db + 24) & -0x1000_i64 | ((lw(item) << 2) + 676 & 4092) as i64);
    sd(db + 64, keep | (lw(db + 24) & 4095) as i64);
}

/// Put a clip tag (a DMA tag that sends packet `n` of 48 bytes) into the display list (@ 0x2b7c98).
pub fn dlSetClipTag(n: i32) {
    sb(lw(g::gfxtag) + 3, 48);
    let tag = lw(g::gfxtag);
    sw(tag + 4, n * 48 + g::cliptagvifpackets);
    sh(tag, 3);
    sw(g::gfxtag, tag + 16);
}

/// Turn a fan prop's angle (at +`at`) by `speed` degrees a tick, wrapped; `inclusive` is whether an angle of exactly 0 wraps too.
fn spin(prop: i32, at: i32, speed: f32, inclusive: bool) {
    let angle = lwc1(prop + at) - lwc1(g::updateframesf) * speed;
    swc1(prop + at, angle);
    if angle < 0.0 || (inclusive && angle == 0.0) {
        swc1(prop + at, angle + 360.0);
    }
}

/// @ 0x279168
pub fn fanChinesePropTick(prop: i32) {
    spin(prop, 76, 7.0, true);
}

/// @ 0x279120
pub fn fanscifiPropTick(prop: i32) {
    spin(prop, 72, 2.5, false);
}

/// @ 0x2791b0
pub fn mallfan1PropTick(prop: i32) {
    spin(prop, 88, 4.0, true);
}

/// @ 0x2791f8
pub fn mallfan2PropTick(prop: i32) {
    spin(prop, 88, 10.0, true);
}
