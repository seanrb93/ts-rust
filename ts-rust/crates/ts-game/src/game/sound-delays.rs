//! Sounds that start after a delay.

use crate::*;

/// Start a sound after `delay` ticks: take a free slot of the 10 (32 bytes each from 0x1fb1870) for sound `sound` at the position `pos` (three
/// floats, 0 for none) and return its number, or -1 if all are used (@ 0x205808).
pub fn soundDelayStartEx(sound: i32, pos: i32, a: i32, delay: i32) -> i32 {
    for i in 0..10 {
        let slot = (i << 5) + 0x1fb1870;
        if lw(slot) == 0 && lhu(slot + 10) == 0 {
            sw(slot + 4, sound);
            sw(slot + 24, a);
            sw(slot + 28, -1);
            sh(slot + 10, delay);
            sw(slot, 1);
            if pos == 0 {
                sh(slot + 8, 0);
            } else {
                let x = lwc1(pos);
                sh(slot + 8, 1);
                swc1(slot + 12, x);
                swc1(slot + 16, lwc1(pos + 4));
                swc1(slot + 20, lwc1(pos + 8));
            }
            return i;
        }
    }
    -1
}
