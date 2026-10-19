//! Controllers (`joy...`), sound buffers, level teardown, and more short functions.

use crate::*;

/// The controller slot of a player: in a level other than 102 players beyond the local ones count from the end of the four slots (the same
/// arithmetic as `GetControllerNumber`).
pub(super) fn controller_slot(player: i32) -> i32 {
    let mut slot = player;
    if lw(g::lvlevelnum) != 102 && slot >= ilinkGetNumLocalPlayers() {
        slot -= ilinkGetNumLocalPlayers() - 4;
    }
    slot
}

/// Address of a player's controller record (384 bytes each).
pub(super) fn controller(player: i32) -> i32 {
    lw((controller_slot(player) << 2) + g::playercontrollers) * 384 + g::controllers
}

/// Buttons newly pressed on a player's controller (@ 0x2038e8).
pub fn joyNewBut(player: i32) -> i32 {
    lw(controller(player) + 292)
}

/// Buttons newly released (@ 0x203968).
pub fn joyReleaseBut(player: i32) -> i32 {
    lw(controller(player) + 296)
}

/// Strength of the left rumble motor (@ 0x203d70).
pub fn joyGetShockLeft(player: i32) -> i32 {
    lbu(controller(player) + 321)
}

/// Strength of the right rumble motor (@ 0x203df0).
pub fn joyGetShockRight(player: i32) -> i32 {
    lbu(controller(player) + 320)
}

/// Set the dead zone of a player's sticks (@ 0x203b30).
pub fn joySetCentreZone(player: i32, zone: f32) {
    swc1(controller(player) + 316, zone);
}

/// Copy the display x setting into the settings that are saved (@ 0x241580).
pub fn menusToSettings() {
    sw(settingsGet(), lw(g::gv_dispx));
}

/// Make sound buffer `n` (16 bytes) cover `start` to `end` and rewind it (@ 0x2044a0).
pub fn soundCreateBuffer(n: i32, start: i32, end: i32) {
    let rec = (n << 4) + 0x1fb1790;
    sw(rec + 4, end - start);
    sw(rec, start);
    soundResetBuffer(n);
}

/// Queue a sound: up to 200 entries of 8 bytes (two halfwords and a word) (@ 0x204908).
pub fn soundBatchAdd(a: i32, b: i32, c: i32) {
    let n = lw(g::soundbatchlen);
    if n < 200 {
        let entry = (n << 3) + 0x1fb19c0 | 0x20000000;
        sw(entry + 4, c);
        sh(entry, a);
        sh(entry + 2, b);
        sw(g::soundbatchlen, n + 1);
    }
}

/// What was possibly hit last becomes what was hit (@ 0x2636f0).
pub fn obConfirmLastHitInst() {
    let part = lw(g::s_PossibleLastHitPart_3aeaa4);
    let chunk = lw(g::s_PossibleLastHitChunk);
    let poly = lw(g::s_PossibleLastHitPoly);
    sw(g::s_pLastHitInst, lw(g::s_pPossibleLastHitInst));
    sw(g::s_LastHitPart_3aeaa0, part);
    sw(g::s_LastHitChunk, chunk);
    sw(g::s_LastHitPoly, poly);
}

/// Walked a list of particles and printed nothing (@ 0x296aa8).
pub fn particleprintlist(_list: i32) {}

/// Walked the map maker's objects and printed nothing (@ 0x248580).
pub fn mmDebugObjects() {}

/// Walked a list and did nothing (@ 0x264020).
pub fn testspecialfillets(_list: i32) {}

/// Compared two boxes and returned nothing; nothing calls it (@ 0x2095b8).
pub fn hittestBoxBox(_a: i32, _b: i32, _c: i32, _d: i32) {}

/// Forget the level's prop lists, unless the level is being restarted (@ 0x267168).
pub fn propEnd() {
    if lw(g::restart) == 0 {
        sw(g::levelprops, 0);
        sw(g::leveldoors, 0);
        sw(g::levelcars, 0);
        sw(g::levelremotes, 0);
        sw(g::levelchrremotes, 0);
        sw(g::levelheads, 0);
        sw(g::levelpickups, 0);
        sw(g::levelgfxpointers, 0);
    }
}

/// Attach prop `prop` to `parent` at the offsets `a` and `b`: the prop takes the parent's room (@ 0x2702d8).
pub fn propSetAttached(prop: i32, parent: i32, a: i32, b: i32) {
    let flags = lw(prop + 16) & -33 | 0x40000000;
    sw(prop + 140, 3);
    sw(prop + 16, flags);
    sw(prop + 192, a);
    sw(prop + 196, b);
    let model = lw(parent + 244);
    sw(prop + 188, parent);
    sw(prop + 12, lw(model + 12));
}

/// Make a prop roll: the roll speed, the axis `axis` (three floats) and the mode 4 (@ 0x270318).
pub fn propSetMtxRoll(prop: i32, _unused: i32, axis: i32, speed: f32) {
    swc1(prop + 96, speed);
    sw(prop + 92, 0);
    let ob = lw(lw(prop + 32));
    swc1(prop + 60, lwc1(axis));
    sw(prop + 140, 4);
    let y = lwc1(axis + 4);
    sw(ob + 4, 1);
    swc1(prop + 64, y);
    swc1(prop + 68, lwc1(axis + 8));
}

/// The prop record (592 bytes) of prop number `num`, or 0 (@ 0x2703f8).
pub fn propGetNum(num: i32) -> i32 {
    let mut left = lw(g::num_props);
    let mut at = lw(g::levelprops);
    while left > 0 {
        if lw(at + 4) == num {
            break;
        }
        at += 592;
        left -= 1;
    }
    if left != 0 { at } else { 0 }
}

/// Clear the used count of every block in the wall memory list (@ 0x266d60).
pub fn propRestartWallMem() {
    let mut block = 0x1fd38c0;
    while block != 0 {
        sw(block + 4, 0);
        block = lw(block);
    }
}

/// Offset of the pivot of a remote prop: at the part's +72, plus 12 for most (@ 0x2694c0).
pub fn remoteGetPivoteOffset(prop: i32) -> i32 {
    let ob = lw(lw(prop + 32));
    let part = ob - lw(ob) * 80;
    if lw(prop + 4) != 425 && lb(part + 7) != 1 {
        lw(part + 72) + 12
    } else {
        lw(part + 72)
    }
}

/// Whether program argument `argv[i + 1]` (where `i` is the position found last) does not start with `-` (@ 0x200140).
pub fn bossIsNextParm() -> i32 {
    let i = lw(g::_fbss);
    if i > 0 && i + 1 < lw(g::myargc) {
        (lb(lw((i << 2) + lw(g::myargv) + 4)) != 45) as i32
    } else {
        0
    }
}

/// Close in-game menu `n` (@ 0x212a90).
pub fn ingamemenuClear(n: i32) {
    sw(frontGet(n) + 20, 0);
    sw(frontGet(n) + 12, 0);
    let menu = n * 92 + 0x1fb3240;
    sh(menu + 8, lhu(menu + 8) & 0xfffc);
}

/// Whether the level has an overseer (levels 2 and 23) (@ 0x2298b8).
pub fn lvGetOverseerState() -> i32 {
    let level = lw(g::lvlevelnum);
    (level == 2 || level == 23) as i32
}

/// Read a word (little-endian) from the loading buffer: `slot` 16 to 27 is a buffer of 2 KB, `offset` into it; -1 outside (@ 0x2b3920).
pub fn makeInt(slot: i32, offset: i32) -> i32 {
    let s = slot - 16;
    if (s as u32) < 12 {
        let at = offset + (s << 11) + g::loadbuf;
        lbu(at) + (lbu(at + 3) << 24) + ((lbu(at + 2) << 16) + (lbu(at + 1) << 8))
    } else {
        -1
    }
}

/// Vibration option (bit 16) (@ 0x20f838).
pub fn vibration_onchange(item: i32) {
    let settings = lw(lw(g::currentplayer) + 24);
    if lw(item) == 0 {
        sh(settings + 2, lhu(settings + 2) & 0xffef);
    } else {
        sh(settings + 2, lhu(settings + 2) | 16);
    }
}

/// Count a glow (up to 72 a slot) when one was added (@ 0x2a6e28).
pub fn zbtestEndAddGlow() {
    let slot = (lw(0x3b0170) << 2) + 0x1fea0a0;
    let count = lw(slot);
    let added = lw(g::CurrentAddIndex);
    if count < 72 && added > 0 {
        sw(slot, count + 1);
    }
}

/// @ 0x2a6608
pub fn zbtestRestart() {
    let slots = 0x1fea0a0;
    sw(0x3b0174, 2);
    sw(slots, 0);
    sw(slots + 8, 0);
    sw(0x3b0170, 2);
    sw(0x3b0178, 0);
    sw(slots + 4, 0);
    sw(0x3b017c, 0);
    sw(0x3b0180, 0);
    sw(0x3b0184, 0);
}

/// Forget the level's rooms and portals, unless it is being restarted (@ 0x2567f8).
pub fn bgEnd() {
    if lw(g::restart) == 0 {
        sw(g::roombb, 0);
        let data = g::roomdata;
        sw(data, 0);
        sw(data + 12, 0);
        sw(data + 4, 0);
        sw(data + 8, 0);
        sw(0x3b0084, 0);
        sw(g::portaldata, 0);
        sw(g::roomprops, 0);
    }
}

/// Show a score on the HUD of player `n` (352 bytes each): the value at +280, and the change at +276 (@ 0x2b0f70).
pub fn hudShowScore(n: i32, score: i32) {
    let hud = n * 352 + g::hudparms;
    let shown = lw(hud + 276);
    sw(hud + 280, score);
    if shown != 0 {
        if shown >= score - 32 {
            sw(hud + 276, score - shown);
        }
        return;
    }
    sw(hud + 276, 1);
}

/// Whether a memory card file is intact: its first word is `header` and the CRC after the `len` bytes of data matches (@ 0x208f30).
pub fn memcardFileCheck(buf: i32, header: i32, len: i32) -> i32 {
    if lw(buf) == header {
        crcCheck(buf + 4, len, lw(len + buf + 4))
    } else {
        0
    }
}

/// Move the map maker's window by (dx, dy) (@ 0x249b58).
pub fn mmPanWindow(dx: i32, dy: i32) {
    let x = lw(g::m_x) + dx;
    let h = lw(g::m_h);
    let y = lw(g::m_y) + dy;
    let x2 = x + lw(g::m_w);
    sw(g::m_x, x);
    sw(g::m_x2, x2);
    sw(g::m_y2, y + h);
    sw(g::m_y, y);
}

/// Stop all 20 timing slots (@ 0x2b5fe8).
pub fn profileRestart() {
    for i in 0..20 {
        let slot = g::profdata + 28 * i;
        sw(slot, 0);
        sw(slot + 20, 0);
    }
}

/// The statistics record of player `n`'s profile: in the sign-on array at +132, or a default one (@ 0x21ffb8).
pub fn statsGetPlayer(n: i32) -> i32 {
    let profile = signonGet(n);
    if lw(g::curgameparms + 80) & 0x2000 == 0 && profile >= 0 {
        lw(g::signonarray) + profile * 2936 + 132
    } else {
        0x1fbc168
    }
}

/// The weather with no effects: scales of 9, the default weather (@ 0x2a8038).
pub fn weatherSet() {
    let w = g::weather;
    swc1(w + 56, 9.0);
    sw(w + 4, g::defaultweather);
    sw(w + 60, 0);
    sw(w + 12, 0);
    sw(w + 24, 0);
    sw(w + 36, 0);
    swc1(w + 48, 9.0);
    swc1(w + 52, 9.0);
}

/// The main window: the whole 640 x 224 field and its centre (@ 0x20c328).
pub fn windowMake() {
    let w = g::window;
    sw(w + 76, 112);
    sw(w + 44, 639);
    sw(w + 52, 640);
    sw(w + 56, 320);
    sw(w + 64, 223);
    sw(w + 72, 224);
    sw(w + 40, 0);
    sw(w + 48, 320);
    sw(w + 60, 0);
    sw(w + 68, 112);
}

/// Clear two words of each of the four front end definitions (@ 0x22ce78).
pub fn frontEnd() {
    for i in 0..4 {
        let def = g::frontdefs + 20 + 44 * i;
        sw(def + 4, 0);
        sw(def, 0);
    }
}

/// Same as `frontEnd` (@ 0x22ca70).
pub fn frontRestart() {
    for i in 0..4 {
        let def = g::frontdefs + 24 + 44 * i;
        sw(def - 4, 0);
        sw(def, 0);
    }
}

/// Whether the escort target (the first escort, an enemy of type 8) has reached the destination pad (@ 0x21de28).
pub fn gmEscorteeReachedDest() -> i32 {
    let dest = lw(0x3afef8);
    if dest >= 0 {
        let escort = lw(g::escort);
        if escort != 0 && lw(escort + 8) == 8 {
            return (lw(lw(escort + 352) + 680) == dest) as i32;
        }
    }
    0
}

/// Whether the left gun of the player's character is shown (@ 0x2943e0).
pub fn gunIsLeftGunVisible(flag: i32) -> i32 {
    let chr = lw(lw(lw(g::currentplayer) + 384) + 352);
    if gunLeftHandAble(chr) != 0 && lw(flag) != 0 {
        (lw(chr + 376) == lh((lw(chr + 260) << 3) + (GUNCOMBOS + 4))) as i32
    } else {
        0
    }
}

/// Whether a room is in view of the first player: its entry (20 bytes) has a last row (+12) not negative, and a first row (+4) below 640 and above it (@ 0x253f80).
pub fn isroomvisible(room: i32) -> i32 {
    let entry = room * 20 + lw((lw(lw(g::currentplayer)) << 2) + g::roomdata);
    let last = lw(entry + 12);
    if last >= 0 {
        let first = lw(entry + 4);
        if first < 640 {
            return (first < last) as i32;
        }
    }
    0
}
