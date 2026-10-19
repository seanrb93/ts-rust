//! Finding the key frames of an animation around a time.

use crate::*;

/// The two key frames (written to `first` and `second`) that surround the time at +4 of the animation state `state` (the animation number is at +0): the
/// frame times of an animation (its count at +4, a table of times in 60ths of a second at +20, the end time at +16) (@ 0x214c50).
pub fn getKeyFrames(_unused: i32, state: i32, first: i32, second: i32) {
    let time = lwc1(state + 4);
    let anim = lw((lw(state) << 2) + g::pAnimations);
    let end = lwc1(anim + 16);
    if !(time == 0.0) {
        let count = lw(anim + 4);
        if count != 1 {
            if !(time == end) {
                sw(second, 0);
                sw(first, 0);
                let mut i = lw(anim + 4) - 2;
                if i >= 0 {
                    let mut frame_time = lw((i << 2) + lw(anim + 20)) as f32 / 60.0;
                    loop {
                        if frame_time < time {
                            sw(first, i);
                            sw(second, i + 1);
                            return;
                        }
                        if time == frame_time {
                            sw(second, i);
                            sw(first, i);
                            return;
                        }
                        i -= 1;
                        if i < 0 {
                            return;
                        }
                        frame_time = lw((i << 2) + lw(anim + 20)) as f32 / 60.0;
                    }
                }
                return;
            }
            sw(second, count - 1);
            sw(first, count - 1);
            return;
        }
    }
    sw(second, 0);
    sw(first, 0);
}
