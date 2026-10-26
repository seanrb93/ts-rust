//! The two streams of the Timesplitter effects (hands and radius), and the handlers of a sleeping enemy and of the controller options.

use super::particle_spawners::random_as_float;
use crate::*;

/// The tick of an effect of a Timesplitter (records of 92 bytes in `fxgroups`: its time at +48, two points at +24 and +36, the two streams started kept at +72 and +76): at
/// a time from 1.5 to 2 it starts two streams of the type `kind` (the count kind `count_kind`) if none runs; while they run they follow the points (a quarter higher), and
/// from 2.5 on their rate is set to 0; with frames passing a random number from 0 to 1 is kept as the time at +52.
fn timesplitter_tick(index: i32, kind: i32, count_kind: i32) {
    let frame = Frame::new(160);
    let sp = frame.sp();
    let fx = g::fxgroups;
    let time = lwc1(fx + index * 92 + 48);
    if 1.5_f32 <= time && time < 2.0_f32 {
        let first = index * 92 + (fx + 72);
        if lw(first) == 0 {
            let second = index * 92 + (fx + 76);
            if lw(second) == 0 {
                let stream = particleNewStream(first, kind, index * 92 + (fx + 24), 0, count_kind, 0.0_f32);
                sw(first, stream);
                let stream = particleNewStream(second, kind, index * 92 + (fx + 36), 0, count_kind, 0.0_f32);
                sw(second, stream);
            }
        }
    }
    let fx = g::fxgroups;
    let first = index * 92 + (fx + 72);
    let group = lw(first);
    let rec = fx + index * 92;
    if group != 0 {
        let lifted = ps2float::add(lwc1(rec + 28), 0.25_f32);
        let z = lwc1(rec + 32);
        swc1(sp, lwc1(rec + 24));
        swc1(sp + 8, z);
        swc1(sp + 4, lifted);
        particleUpdatePos(group, sp);
        if 2.5_f32 < lwc1(fx + index * 92 + 48) {
            particleUpdateRate(lw(first), 0.0_f32);
        }
    }
    let fx = g::fxgroups;
    let second = index * 92 + (fx + 76);
    let group = lw(second);
    let rec = fx + index * 92;
    let mut frames;
    if group != 0 {
        let lifted = ps2float::add(lwc1(rec + 40), 0.25_f32);
        let z = lwc1(rec + 44);
        swc1(sp + 16, lwc1(rec + 36));
        swc1(sp + 24, z);
        swc1(sp + 20, lifted);
        particleUpdatePos(group, sp + 16);
        frames = lw(g::updateframesi);
        if 2.5_f32 < lwc1(fx + index * 92 + 48) {
            particleUpdateRate(lw(second), 0.0_f32);
            frames = lw(g::updateframesi);
        }
    } else {
        frames = lw(g::updateframesi);
    }
    if frames > 0 {
        let r = random_as_float();
        swc1(index * 92 + (g::fxgroups + 52), ps2float::mul(r, 2.3283064e-10_f32));
    }
}

/// The hands of a Timesplitter (streams of kind 20, counts of kind 40) (@ 0x2ad788).
pub fn specialfxTimesplitterHandsTick(index: i32) {
    timesplitter_tick(index, 20, 40);
}

/// The radius of a Timesplitter (streams of kind 21, counts of kind 20) (@ 0x2ad520).
pub fn specialfxTimesplitterRadiusTick(index: i32) {
    timesplitter_tick(index, 21, 20);
}

/// A computer character goes to sleep: its move mode is set to 8 (and to 4 as well if it is in state 0x80000); with any of the flags 0xf000000 it wakes up again (@ 0x2cb290).
pub fn enemySleep(chr: i32) {
    if lw(lw(chr + 352) + 2724) != 0x80000 {
        enemySetMoveMode(chr, 8);
    } else {
        enemySetMoveMode(chr, 8);
        enemySetMoveMode(chr, 4);
    }
    if lw(chr + 16) & 0xf000000 == 0 {
        return;
    }
    enemyFromSleepToAwake(chr);
}

/// An option of the controller changed: the new kind (a halfword at `option`) is kept in the controller record of the current player, and the player's buttons are set again (@ 0x20f558).
pub fn config_onchange(option: i32) {
    let player = lw(g::currentplayer);
    sh(lw(player + 24), lhu(option));
    playerSetControls(player);
}
