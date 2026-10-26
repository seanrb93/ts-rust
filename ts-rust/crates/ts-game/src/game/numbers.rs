//! Numbers: angles, fixed point, bit tricks and checksums.

use crate::*;

/// The shortest turn from angle `from` to angle `to`, in degrees: the difference wrapped into -180 to 180 (@ 0x284e38).
pub fn anglediff(from: f32, to: f32) -> f32 {
    let d = to - from;
    if d < -180.0 {
        d + 360.0
    } else if 180.0 < d {
        d - 360.0
    } else {
        d
    }
}

/// The smallest `n` with 2^n >= `x`, for x > 0 (@ 0x2b48c0).  Loops for ever on 0, as the original does.
pub fn PowerOf2(x: i32) -> i32 {
    let mut rest = x - 1;
    let mut n = 0;
    if rest != 0 {
        loop {
            rest >>= 1;
            n += 1;
            if rest == 0 {
                break;
            }
        }
    }
    n
}

/// Fixed point multiply: `a` * `b` in 16.16, as a signed 64-bit number; the top 16 bits of the 64-bit product are lost (@ 0x24b978).
pub fn FMul(a: i32, b: i32) -> i64 {
    ((a as i64) * (b as i64)) << 16 >> 32
}

/// CRC-32 of `len` bytes at `data`, with the game's table `crc_32_tab` (@ 0x2b3d00).  A length of 0 gives 0.
pub fn crcCalc(data: i32, len: i32) -> i32 {
    let mut crc = -1_i32;
    let (mut at, mut left) = (data, len);
    if left != 0 {
        loop {
            let byte = lbu(at);
            at += 1;
            left -= 1;
            crc = lw((((crc ^ byte) & 255) << 2) + g::crc_32_tab) ^ ((crc as u32) >> 8) as i32;
            if left == 0 {
                break;
            }
        }
    }
    !crc
}

/// Whether `crcCalc` of the data gives `expected` (@ 0x2b3d50).
pub fn crcCheck(data: i32, len: i32, expected: i32) -> i32 {
    (crcCalc(data, len) == expected) as i32
}

/// The position in the table of 575 note periods (u16 at 0x1fb20a8, falling) where `period` fits: the first entry it is not below (@ 0x205f98).
/// 288 if it is below them all.
pub fn getNote(period: i32) -> i32 {
    const TABLE: i32 = 0x1fb20a8;
    let p = period & 0xffff;
    let mut i = 0;
    loop {
        let next = i + 1;
        if p < lhu(i * 2 + TABLE) {
            i = next;
        } else {
            if p < lhu(next * 2 + TABLE) {
                return i;
            }
            i = next;
        }
        if i >= 575 {
            return 288;
        }
    }
}

/// A stick axis (0..255, centre 128) as -1..1 with a dead zone of `dead` (0 for none) (@ 0x203088).  Beyond the dead zone the value
/// is stretched so that it starts at 0 and still reaches 1.
pub fn joyFixVal(raw: i32, dead: f32) -> f32 {
    let off = raw - 128;
    let v = if raw >= 128 { (off as f32).pdiv(127.0) } else { off as f32 * 0.0078125 };
    if !(0.0 < dead) {
        return v;
    }
    if v < -dead {
        ((v + dead)).pdiv((1.0 - dead))
    } else if dead < v {
        ((v - dead)).pdiv((1.0 - dead))
    } else {
        0.0
    }
}
