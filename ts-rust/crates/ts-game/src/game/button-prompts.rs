//! Button prompts, pad route costs, replay and profile state, glow tests, and more short functions.

use super::controllers::controller;
use crate::*;

/// @ 0x2b6020
pub fn profileReset() {
    profileRestart();
}

/// @ 0x266d98
pub fn propResetWallMem() {
    sw(0x1fd38c0, 0);
    propRestartWallMem();
}

/// Whether a message was dismissed: cross (16) or circle (64) newly pressed on the first controller (the buttons' bits as the game numbers them) (@ 0x231388).
pub fn defaultmessagetick_231388(player: i32) -> i32 {
    (joyNewBut(player) & 80 != 0) as i32
}

/// @ 0x206b68
pub fn defaultmessagetick_206b68() -> i32 {
    (joyNewBut(0) & 80 != 0) as i32
}

/// @ 0x24c968
pub fn defaultmessagetick_24c968() -> i32 {
    (joyNewBut(0) & 80 != 0) as i32
}

/// Choice with circle = yes (bit 32 gives 1), triangle = no (shifted bit gives 2), else 0 (@ 0x24d2a0).
pub fn CircleYesTriangleNo() -> i64 {
    if joyNewBut(0) & 32 != 0 {
        return 1;
    }
    ((joyNewBut(0) as i64 as u64) >> 3) as i64 & 2
}

/// Choice with bit 64 = yes (1), bit 16 = no (2), else 0 (@ 0x24d268).
pub fn confirmyesno() -> i64 {
    if joyNewBut(0) & 64 != 0 {
        return 1;
    }
    ((joyNewBut(0) as i64 as u64) >> 3) as i64 & 2
}

/// Buttons held on a player's controller and not masked (@ 0x203860).
pub fn joyBut(player: i32) -> i32 {
    let c = controller(player);
    lw(c + 288) & !lw(c + 340)
}

/// Set a player's rumble: left and right strengths, at most 127 (@ 0x203cc0).
pub fn joySetShock(player: i32, left: i32, right: i32) {
    let c = controller(player);
    sb(c + 321, if left < 128 { left } else { 127 });
    sb(c + 320, if right < 128 { right } else { 127 });
}

/// Stop a sound voice: queue the command 2 with the voice's side and its bit (@ 0x2054a0).
pub fn soundStop(voice: i32) {
    soundBatchAdd(2, (voice & 1) | 0x1600, 1 << ((voice >> 1) & 31));
}

/// The colour and intensity of a room's light, if lights were set up: 1 and the two values written to `colour` and `intensity` (@ 0x2a7cc8).
pub fn roomlightGetRoomColourIntensity(room: i32, colour: i32, intensity: i32) -> i32 {
    let initial = lw(g::roomlightinitial);
    if initial == 0 {
        return 0;
    }
    let light = room * 28 + lw(g::roomlights);
    sw(colour, lw(room * 24 + initial - 4) & -256);
    sw(intensity, lbu(light + 24));
    1
}

/// A scanner prop turns at half a degree a tick (@ 0x279058).
pub fn scannerPropTick(prop: i32) {
    let turn = lwc1(g::updateframesf) * 0.5;
    sw(prop + 140, 2);
    let angle = lwc1(prop + 72) + turn;
    swc1(prop + 72, angle);
    if 360.0 <= angle {
        swc1(prop + 72, angle - 360.0);
    }
}

/// Load the player's option flags into the menu variables (@ 0x20f570).
pub fn setup_vars() {
    let settings = lw(lw(g::currentplayer) + 24);
    let flags = lhu(settings + 2);
    let config = lhu(settings);
    sw(g::gv_aimmode, ((flags >> 1) ^ 1) & 1);
    sw(g::gv_crouchmode, ((flags >> 2) ^ 1) & 1);
    sw(g::gv_lookahead, (flags >> 3) & 1);
    sw(g::gv_invlook, flags & 1);
    sw(g::gv_config, config);
}

/// Forget the special effects: clear the used table (50 words) and the count (@ 0x277b70).
pub fn specialRestart() {
    for i in 0..50 {
        sw(g::tmpSpecialDataUsed + 4 * i, 0);
    }
    sw(g::numspecials, 0);
}

/// Add a glow to the depth test list (60 at most, 28 bytes each): its position and values (@ 0x2a6ef8).
pub fn zbtestDrawGlow(a: i32, b: i32, size: i32, c: i32, x: f32, y: f32) {
    let n = lw(0x3b017c);
    if n < 60 {
        sw(0x3b017c, n + 1);
        let glow = n * 28 + 0x1fea0b0;
        sw(glow + 20, size);
        sw(glow + 24, c);
        sw(glow, a);
        sw(glow + 8, b);
        swc1(glow + 12, x);
        swc1(glow + 16, y);
    }
}

/// Add a lens flare to the depth test list (12 at most, 24 bytes each) (@ 0x2a6f50).
pub fn zbtestDrawFlareFx(a: i32, b: i32, pos: i32, c: i32) {
    let n = lw(0x3b0180);
    if n < 12 {
        sw(0x3b0180, n + 1);
        let flare = n * 24 + 0x1fea740;
        swc1(flare + 8, lwc1(pos));
        sw(flare + 20, c);
        let y = lwc1(pos + 4);
        sw(flare, a);
        swc1(flare + 12, y);
        sw(flare + 4, b);
        swc1(flare + 16, lwc1(pos + 8));
    }
}

/// An enemy's spawn effect (a record at +0x11a4 of the enemy) is freed once it has reached state 2 (@ 0x2bbca0).
pub fn enemyCheckSpawnfx(enemy: i32) {
    let fx = lw(enemy + 0x11a4);
    if fx != 0 && lw(fx + 4) == 2 {
        specialfxFreeSpawn(fx);
        sw(enemy + 0x11a4, 0);
    }
}

/// Clear the front end goodies effect's 512 alpha bytes (@ 0x22be58).
pub fn frontfxGoodiesReset() {
    for i in 0..512 {
        sb(g::frontfxalpha + i, 0);
    }
    sw(g::frontfxgoodiestime, 0);
    sw(g::frontfxlpos, -64);
}

/// The straight distance between two pads: the bots' estimate of the cost of a route (@ 0x266100).
pub fn hr_calcestcost(from: i32, to: i32) -> f32 {
    let pads = lw(g::levelpads);
    let a = from * 28 + pads;
    let b = to * 28 + pads;
    let dx = lwc1(b + 12) - lwc1(a + 12);
    let dy = lwc1(b + 16) - lwc1(a + 16);
    let dz = lwc1(b + 20) - lwc1(a + 20);
    (dx * dx + dy * dy + dz * dz).abs().psqrt()
}

/// The same estimate for the other route finder (@ 0x266768).
pub fn r_calcestcost(from: i32, to: i32) -> f32 {
    hr_calcestcost(from, to)
}

/// How long the overseer sleeps when you go back: 1.5 on level 2, 6 on level 23, else 0 (@ 0x2298e0).
pub fn lvGetOverseerBackSleepTime() -> f32 {
    match lw(g::lvlevelnum) {
        2 => 1.5,
        23 => 6.0,
        _ => 0.0,
    }
}

/// Un-lasso every map maker object (clear bit 0 of its flags) (@ 0x245b68).
pub fn mmClearLassoed() {
    let mut object = lw(g::mapobjects);
    while object != 0 {
        sw(object + 4, lw(object + 4) & -2);
        object = lw(object + 56);
    }
    sw(g::lassocount, 0);
}

/// Whether the prop number is of type 1 or 8 (those the prop editor can change) (@ 0x27c180).
pub fn propEditValidPropnum(prop: i32) -> i32 {
    let kind = lw(prop * 28 + PROPTABLE);
    (kind == 1 || kind == 8) as i32
}

/// Set up a replay: `mode` 0 off, 1 record (with the number of players, the buffer and its length); the random numbers restart from 1 (@ 0x202d58).
pub fn replayReset(mode: i32, players: i32, buffer: i32, len: i32) {
    sw(g::replaymode, mode);
    if mode == 1 {
        sw(g::replaybuflen, len);
        sw(0x3afe1c, buffer + 112);
        sw(g::replaynumplayers, players);
        sw(g::replaybuf, buffer);
        sw(g::replayend, 0);
        newrndseed(1);
    }
}

/// The next program argument if it is not an option, advancing past it; else 0 (@ 0x200180).
pub fn bossGetNextParm() -> i32 {
    let ok = bossIsNextParm();
    let i = lw(g::_fbss);
    if ok == 0 {
        0
    } else {
        sw(g::_fbss, i + 1);
        lw(((i + 1) << 2) + lw(g::myargv))
    }
}

/// Hand out the next sound channel as (core, voice): voices 0 to 23 of core 0, then core 1 starting at 11 (@ 0x205540).
pub fn getNextChannel(core: i32, voice: i32) {
    sw(core, lw(g::c_154));
    sw(voice, lw(g::v_155));
    let next = lw(g::v_155) + 1;
    sw(g::v_155, next);
    if next >= 24 {
        let mut c = lw(g::c_154) + 1;
        sw(g::c_154, c);
        if c >= 2 {
            sw(g::c_154, 0);
            c = 0;
        }
        sw(g::v_155, if c != 0 { 0 } else { 11 });
    }
}

/// The flashing value for menu highlights: a triangle wave of the frame number, and a quick on / off (@ 0x22e208).
pub fn guiFlashAlpha() {
    let phase = lbu(g::tickframenum) & 127;
    sb(g::flashalpha, phase);
    if phase as u32 >= 64 {
        sb(g::flashalpha, 128 - phase);
    }
    let quick = if lw(g::tickframenum) & 32 != 0 { 128 } else { 64 };
    sb(g::flashalpha, lbu(g::flashalpha) + 64);
    sb(g::quickflash, quick);
}

/// Start a new blast ring: the next one of the table (@ 0x2a7298).
pub fn blastringNew() {
    let ring = g::br;
    let n = lw(g::numrings) + 1;
    swc1(g::br, 1.0);
    sw(g::numrings, n);
    swc1(ring + 4, 2.0);
    swc1(ring + 16, 90.0);
    swc1(ring + 24, 1.0);
    sw(ring + 32, 0);
    swc1(ring + 8, 1.0);
    sw(ring + 12, 0);
    sw(ring + 20, 0);
    sw(ring + 28, 0);
}

/// Add the commands that select texture `n` and kick it to the display list (@ 0x2b7ad0).
pub fn dlSelectTextureKick(n: i32) {
    let index = if n <= -1 { 0 } else { n };
    let data = lw(index * 40 + lw(lw(g::thistexbuf) + 4) + 32);
    sb(lw(g::gfxtag) + 3, 48);
    let tag = lw(g::gfxtag);
    let next = tag + 16;
    sw(tag + 4, data);
    sw(g::gfxtag, next);
    sh(tag, 8);
    sb(next + 3, 48);
    let tag2 = lw(g::gfxtag);
    sw(tag2 + 4, g::texturekick);
    sh(tag2, 6);
    sw(g::gfxtag, tag2 + 16);
}

/// Whether a gun auto-aims: all except guns 6 and 17 (@ 0x295238).
pub fn gunDoesAutoAim(gun: i32) -> i32 {
    if gun != 0 {
        let kind = lw(gun + 260);
        if kind == 6 || kind == 17 {
            return 0;
        }
    }
    1
}

/// Searched a table for an index and did nothing with it (@ 0x22c378).
pub fn gvDispIndex(_table: i32, _value: i32) {}

/// Mark all 8 memory card slots as not valid and no card chosen (@ 0x244d50).
pub fn mcseqUnvalidateCards() {
    let flags = lw(g::mcseq_flags);
    for i in 0..8 {
        sw(0x1fc3710 + 4 * i, 1);
    }
    sw(0x3b0008, -1);
    sw(g::mcseq_flags, flags & -7);
}

/// Leave a map maker menu: restore its state and, with the last one closed, hand the front end back (@ 0x24ba08).
pub fn mmExitMenu(menu: i32) {
    let front = lw(g::front);
    let open = lw(front + 20) - 1;
    sh(lw(menu) + 8, lhu(menu + 4));
    sw(front + 20, open);
    if open == 0 {
        let flags = lw(front + 12);
        sw(g::editflags, lw(g::editflags) & -9);
        sw(front + 12, flags | 3);
        sw(front + 40, 0x24fe68);
    }
    if lw(g::editflags) & 8 != 0 {
        sw(front + 12, lw(front + 12) & -4);
    }
}

/// Particle size for the number of local players: half for 2, a quarter for 3 or 4 (@ 0x296838).
pub fn particleGetScale() -> f32 {
    match ilinkGetNumLocalPlayers() {
        2 => 0.5,
        3 | 4 => 0.25,
        _ => 1.0,
    }
}

/// Zero the running time of every timing slot that is in use (@ 0x2b6028).
pub fn profileTick() {
    for i in 0..20 {
        let slot = g::profdata + 28 * i;
        if lw(slot) != 0 {
            sw(slot + 8, 0);
        }
    }
}
