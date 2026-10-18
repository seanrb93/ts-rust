//! The game's random numbers.

use crate::*;

/// The next random number, a shift-and-xor generator on the 64-bit `newrndseedval` (@ 0x2b6888).  Returns the low 32 bits,
/// sign-extended.
pub fn newrnd2() -> i64 {
    let x = ld(g::newrndseedval) as u64;
    let t = ((x << 63) >> 31 | (x << 31) >> 32) ^ (x << 44) >> 32;
    let next = ((t >> 20) & 4095) ^ t;
    sd(g::newrndseedval, next as i64);
    next as i32 as i64
}

/// A random number that is also counted: `rndcount` is how many have been asked for since the seed was set (@ 0x2b68d0).
pub fn newrnd() -> i64 {
    sw(g::rndcount, lw(g::rndcount) + 1);
    newrnd2()
}

/// Set the seed and restart the count (@ 0x2b68f8).
pub fn newrndseed(seed: i32) {
    sw(g::rndcount, 0);
    sd(g::newrndseedval, seed as u32 as i64 + 1);
}
