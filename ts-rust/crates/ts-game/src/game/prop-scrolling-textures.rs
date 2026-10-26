//! The props whose surface moves: the torch of the tomb and the waterfall.

use crate::*;

/// Copy the vertices of a bone's chunk (16 bytes each) to the override array `out`, the second float (the height) moved on by the prop's scroll value at +40 of its state.
fn copy_scrolled(source: i32, mut out: i32, mut count: i32, scroll_state: i32) {
    let mut source = source;
    loop {
        count -= 1;
        swc1(out, lwc1(source));
        swc1(out + 4, ps2float::add(lwc1(source + 4), lwc1(scroll_state + 40)));
        swc1(out + 8, lwc1(source + 8));
        let w = lwc1(source + 12);
        source += 16;
        swc1(out + 12, w);
        out += 16;
        if count == 0 {
            break;
        }
    }
}

/// The torch of the tomb: the vertices of the second bone's second chunk list are copied scrolled by the state's value, which then falls with the frame time and wraps by 1;
/// the prop is turned on a little (@ 0x27beb0).
pub fn tombtorchPropTick(prop: i32) {
    let frame = Frame::new(80);
    let sp = frame.sp();
    let state = lw(prop + 352);
    let skeleton = lw(lw(prop + 32));
    let bone = skeleton - lw(skeleton) * 80 + 80;
    allocateOverrideChunk(prop, 1, 1, 2);
    let out = getOverrideChunk(prop, 1, 1, 2, sp);
    let mut step = lwc1(g::updateframesf);
    if out != 0 {
        let count = lw(sp);
        let source = lw(bone + 52);
        if count > 0 {
            copy_scrolled(source, out, count, state);
            step = lwc1(g::updateframesf);
        }
    }
    let scroll = ps2float::sub(lwc1(state + 40), ps2float::mul(step, lwc1(0x3acf9c)));
    swc1(state + 40, scroll);
    if scroll < 0.0_f32 {
        swc1(state + 40, ps2float::add(scroll, 1.0_f32));
    }
    swc1(prop + 76, ps2float::add(lwc1(prop + 76), 0.5_f32));
}

/// The waterfall: the same for the first bone; the scroll value rises with the frame time (at one of two speeds by `slow`) and wraps by 1 (@ 0x27b0d0).
pub fn waterfallPropTick(prop: i32, slow: i32) {
    let frame = Frame::new(96);
    let sp = frame.sp();
    let state = lw(prop + 352);
    let skeleton = lw(lw(prop + 32));
    let bone = skeleton - lw(skeleton) * 80;
    allocateOverrideChunk(prop, 0, 1, 2);
    let out = getOverrideChunk(prop, 0, 1, 2, sp);
    if out != 0 {
        let count = lw(sp);
        let source = lw(bone + 52);
        if count > 0 {
            copy_scrolled(source, out, count, state);
        }
    }
    let mut speed = lwc1(0x3acf78);
    let step = lwc1(g::updateframesf);
    if slow == 0 {
        speed = lwc1(0x3acf7c);
    }
    let one = 1.0_f32;
    let scroll = ps2float::add(lwc1(state + 40), ps2float::mul(step, speed));
    swc1(state + 40, scroll);
    if one < scroll {
        swc1(state + 40, ps2float::sub(scroll, one));
    }
}
