//! Reading back the depth tests of glows.

use crate::*;

/// The sum of the values (at +20 of each point, 24 bytes) of the points of the depth test (a, b, c) that passed: a test has 9 points; a point
/// passed when its depth difference (halfwords at +16 and +18) is below the square of the second one over 65536.  0 when there is no such test (@ 0x2a7188).
pub fn zbtestGetVisibleBias(a: i32, b: i32, c: i32) -> i32 {
    let slot = lw(0x3b0178);
    let tests_at = (slot << 2) + 0x1fea0a0;
    let mut total = 0;
    if lw(tests_at) <= 0 {
        return 0;
    }
    let mut i = 0;
    let mut point;
    loop {
        point = lw((slot << 2) + g::zbval) + i * 216;
        if lw(point + 4) == b && lw(point + 8) == c && lw(point) == a {
            break;
        }
        i += 1;
        if i >= lw(tests_at) {
            return 0;
        }
    }
    let mut depth = lhu(point + 18);
    let mut near = lhu(point + 16);
    let mut square = depth * depth;
    let mut k = 0;
    loop {
        k += 1;
        let scaled = (if -1 < square { square } else { square.wrapping_add(0xffff) }) >> 16;
        if (depth - near) < scaled {
            total += lw(point + 20);
        }
        point += 24;
        if k >= 9 || lw(point) != a {
            break;
        }
        depth = lhu(point + 18);
        near = lhu(point + 16);
        square = depth * depth;
    }
    total
}
