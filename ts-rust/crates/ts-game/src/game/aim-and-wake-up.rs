//! Testing the aim against every prop, waking an enemy up, and resetting the map maker dialogs.

use crate::*;

/// Test the current player's aim line against the box (half a unit round the position) of every prop of the level; the answer is not used (@ 0x267240).
pub fn propGetAimProp() {
    let frame = Frame::new(128);
    let sp = frame.sp();
    let mut i = 0;
    let half = 0.5_f32;
    if lw(g::num_props) > 0 {
        loop {
            let player = lw(g::currentplayer);
            let line_end = sp + 16;
            let center = player + 152;
            let prop = i * 592 + lw(g::levelprops);
            let sizes = lw(lw(player + 384) + 352) + 2976;
            swc1(sp, lwc1(prop + 48) - half);
            swc1(sp + 4, lwc1(prop + 52) - half);
            swc1(sp + 8, lwc1(prop + 56) - half);
            swc1(sp + 16, lwc1(prop + 48) + half);
            swc1(sp + 20, lwc1(prop + 52) + half);
            swc1(sp + 24, lwc1(prop + 56) + half);
            hittestLineBox(center, sizes, sp, line_end);
            i += 1;
            if i >= lw(g::num_props) {
                break;
            }
        }
    }
}

/// Wake a sleeping enemy: in the story with flag 32 and the mode 24 it goes to state 8 (the old state kept at +2720) aiming at the first player's character; otherwise it
/// takes a target by `suggestTarget`, loses flag 128, and goes to the state held at +2724 (when it holds any of 0x69c000) or else state 0x20000 (@ 0x2cb308).
pub fn enemyFromSleepToAwake(chr: i32) {
    let state = lw(chr + 352);
    let params = g::curgameparms;
    let players;
    if lw(params + 80) & 32 == 0 {
        players = lw(g::players);
    } else {
        players = lw(g::players);
        if lw(params + 68) == 24 {
            let old = lw(state + 2716);
            let target = lw(lw(g::chrinfo) + 3020);
            sw(state + 2716, 8);
            sw(state + 2788, target);
            sw(state + 2720, old);
            sw(state + 2892, 0);
            return;
        }
    }
    let target = suggestTarget(chr, lw(players + 384), 0);
    sw(chr + 16, lw(chr + 16) & -129);
    sw(state + 2788, target);
    let pending = lw(state + 2724);
    if pending & 0x69c000 == 0 {
        let old = lw(state + 2716);
        sw(state + 2716, 0x20000);
        sw(state + 2720, old);
    } else {
        let old = lw(state + 2716);
        sw(state + 2716, pending);
        sw(state + 2720, old);
        sw(state + 2724, 0);
    }
    sw(state + 2892, 0);
}

/// Reset the dialogs of the map maker: the light is the colour kept at 0x1fc4050, light 4 is the current one, type and phase are 0 (@ 0x253530).
pub fn mmDialogsReset() {
    let colour = lw(0x1fc4050);
    sw(g::curlightcol, 4);
    sw(0x3b005c, 0);
    sb(g::lighttype, 0);
    sb(g::lightphase, 0);
    mmSetLightFromRGB(colour);
    sw(0x3b0060, 0);
}
