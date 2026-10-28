//! Small functions of the game's systems that wait for a better home.

use crate::*;

/// Whether bullet type `t` belongs to group 1: types 14 to 16 (@ 0x289490).
pub fn bulletGetGroupType(t: i32) -> i32 {
    (14 <= t && t < 17) as i32
}

/// Whether a bullet type explodes: types 8 to 18 (@ 0x2894b0).
pub fn bulletTypeExplosive(t: i32) -> i32 {
    (8 <= t && t < 19) as i32
}

/// Size scale of a character (@ 0x2891e0).
pub fn chrGetScale(chr: i32) -> f32 {
    lwc1(chr * 44 + (g::chrs + 40))
}

/// @ 0x2c7ed8
pub fn enemyGetBase(n: i32) -> i32 {
    lw((n << 3) + (g::bases + 4))
}

/// An animation by number (@ 0x214998).
pub fn getAnimation(n: i32) -> i32 {
    lw((n << 2) + g::pAnimations)
}

/// The type of bullet a gun fires; a gun has its table entry (400 bytes) behind two pointers at +172 (@ 0x2906a0).
pub fn gunGetBulletType(gun: i32) -> i32 {
    lw(lw(lw(gun + 172)) * 400 + (GUNS_TABLE + 52))
}

/// Set the transparency scale of decal `n` (@ 0x2a4bc0).
pub fn decalSetAlphaScale(n: i32, scale: f32) {
    swc1(n * 240 + lw(g::s_Decals) + 220, scale);
}

/// Forget the decals unless the level restarts (@ 0x2a26f8).
pub fn decalEnd() {
    if lw(g::restart) == 0 {
        sw(g::s_Decals, 0);
    }
}

/// Empty in the build (@ 0x215d30).
pub fn animRestart() {}

/// Address of the current model-view matrix on the animation matrix stack, 64 bytes each (@ 0x215800).
pub fn anim_GetModelview() -> i32 {
    ((lw(g::animMTX_stackpos) + lw(g::animMTX_offset)) << 6) + 0x1fb37b0
}

/// Pop the matrix stack (@ 0x2157e8).
pub fn anim_PopModelview() {
    let top = lw(g::animMTX_stackpos);
    sw(g::animMTX_offset, 0);
    sw(g::animMTX_stackpos, top - 1);
}

/// @ 0x215718
pub fn anim_SetMatrixArray(array: i32) {
    sw(0x3afe7c, array);
}

/// Width of a graphics buffer in 64-pixel blocks: a buffer whose size (+4) is at most 64 is 1 block, or 2 if its flag (bit 0 of the byte at +36) is set (@ 0x2b7010).
pub fn GSBufferWidth(buf: i32) -> i32 {
    let size = lw(buf + 4);
    if (size as u32) < 65 {
        if lbu(buf + 36) & 1 != 0 { 2 } else { 1 }
    } else {
        ((size as u32) >> 6) as i32
    }
}

/// Dither setting for see-through drawing: 1 on level 23, else 2 (@ 0x226df8).
pub fn lvGetDitherForXLU() -> i32 {
    if lw(g::lvlevelnum) == 23 { 1 } else { 2 }
}

/// How far away a zoom tile of size `n` is seen, by how many of its two sizes are below 2, 3 and 4: 10, 15, 19 or 50 (@ 0x250600).
pub fn zoomtileSizeToDist(n: i32) -> f32 {
    let tile = |level: i32| {
        let t = lw(n * 4 + g::tiles);
        lw(t) < level && lw(t + 4) < level
    };
    if tile(2) {
        10.0
    } else if tile(3) {
        15.0
    } else if tile(4) {
        19.0
    } else {
        50.0
    }
}
