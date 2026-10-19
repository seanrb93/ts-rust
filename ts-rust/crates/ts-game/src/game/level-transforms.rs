//! Moving the parts of a level (walls, floors, glass, volumes, portals) by a matrix, for the tiles that move or rotate.

use crate::*;

/// Transform a wall (two points of three floats at +4 and +20) and lift its two heights (+16, +32) by the height of the translation `trans` (@ 0x254360).
pub fn bgWallTransform(wall: i32, trans: i32, m: i32) {
    matrixVecMul(m, wall + 4);
    matrixVecMul(m, wall + 20);
    let top = lwc1(wall + 32);
    swc1(wall + 16, lwc1(wall + 16) + lwc1(trans + 8));
    swc1(wall + 32, top + lwc1(trans + 8));
}

/// Transform a volume: its position (+8), its axes (+20), and turn its angle (+36) by the rotation of the tile (`tilerotang`, by the word at +16 of `tile`) (@ 0x254260).
pub fn bgVolumeTransform(volume: i32, tile: i32, m: i32) {
    matrixVecMul(m, volume + 8);
    matrixVecRot(m, volume + 20);
    let angle = lwc1(volume + 36) + lwc1((lw(tile + 16) << 2) + g::tilerotang);
    swc1(volume + 36, angle);
    if 360.0 <= angle {
        swc1(volume + 36, angle - 360.0);
    }
}

/// Transform the points (12 bytes each from +8) of a floor; the count is at +0 (@ 0x2542f0).
pub fn bgFloorTransform(floor: i32, m: i32) {
    if lw(floor) != 0 {
        let mut i = 0;
        loop {
            matrixVecMul(m, floor + 8 + 12 * i);
            i += 1;
            if (i as u32) >= (lw(floor) as u32) {
                break;
            }
        }
    }
}

/// Transform the panes (40 bytes each from +16) of a room's glass; the count is at +12 (@ 0x2543d0).
pub fn bgGlassTransform(glass: i32, m: i32) {
    if lw(glass + 12) != 0 {
        let mut i = 0;
        loop {
            matrixVecMul(m, glass + 16 + 40 * i);
            i += 1;
            if (i as u32) >= (lw(glass + 12) as u32) {
                break;
            }
        }
    }
}

/// Transform the points of a portal (12 bytes each from +24, their number a halfword at +20) and its normal (+8) (@ 0x254610).
pub fn bgPortalTransform(portal: i32, m: i32) {
    let mut i = 0;
    if lh(portal + 20) > 0 {
        loop {
            matrixVecMul(m, portal + 24 + 12 * i);
            i += 1;
            if i >= lh(portal + 20) {
                break;
            }
        }
    }
    matrixVecRot(m, portal + 8);
}
