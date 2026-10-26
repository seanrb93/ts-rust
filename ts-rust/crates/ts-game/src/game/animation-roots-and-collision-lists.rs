//! The root movement of animations, and the lists of props that collide.

use crate::*;

/// Put the texture list `textures` on an object and add `offset` to the texture offsets of all its bones (@ 0x260220).
pub fn obSetTexList(skeleton: i32, textures: i32, offset: i32) {
    partSetTextureOffset(skeleton, 0, offset);
    sw(skeleton + 12, textures);
}

/// Put `prop` in the first free place of the list `list` (`count_at` holds how many places it has) or, when no place was freed (`cleared` is 0), at its end (@ 0x268538, both lists).
fn put_in_list(list: i32, count_at: i32, cleared_at: i32, cleared: i32, prop: i32) {
    if cleared != 0 {
        let count = lw(count_at);
        if count > 0 {
            if lw(list) != 0 {
                for i in 1..count {
                    let slot = (i << 2) + list;
                    if lw(slot) == 0 {
                        sw(slot, prop);
                        break;
                    }
                }
            } else {
                sw(list, prop);
            }
        }
        sw(cleared_at, cleared - 1);
    } else {
        let n = lw(count_at);
        sw(count_at, n + 1);
        sw((n << 2) + list, prop);
    }
}

/// Put a prop in the list of the props that push (0x1fd3410) or of those that do not (0x1fd2f60), reusing a place that was cleared (@ 0x268538).
pub fn propPutPropInCollisionList(prop: i32) {
    let can_push = propDoCollisionsCanPush(lw(prop + 4));
    let cleared_pushers = lw(0x3b00f4);
    if can_push == 0 {
        let can_not_push = propDoCollisionsCanNotPush(lw(prop + 4));
        let cleared = lw(0x3b00f0);
        if can_not_push != 0 {
            put_in_list(0x1fd2f60, 0x3b00e8, 0x3b00f0, cleared, prop);
        }
    } else {
        put_in_list(0x1fd3410, 0x3b00ec, 0x3b00f4, cleared_pushers, prop);
    }
}

/// The root movement of an animation: the values at the start (`first`) and end (`last`) for the axis `axis` (0, 1 or 2), read from the animation's data by its
/// kind (0, 2, 4 or 8), a value of the animation to `third`; the two are then scaled by the character's size (+2840 of its record), times the constant at
/// `factor_at` for some characters (the code of `extractAnimRootData` and `extractAnimRoot`).
fn anim_root(chr: i32, anim: i32, first: i32, last: i32, third: i32, axis: i32, factor_at: i32) {
    let mut scale = lwc1(lw(chr + 352) + 2840);
    let axis = match axis {
        0 => 0,
        1 => 1,
        2 => 2,
        _ => -1,
    };
    let data = lw(anim + 32);
    let kind = lw(data + 4);
    swc1(third, lw(anim) as f32);
    match kind {
        0 | 2 => {
            let v = lwc1((axis << 2) + lw(data + 20));
            swc1(last, v);
            swc1(first, v);
        }
        4 => {
            let list = lw(data + 20);
            swc1(first, lwc1((axis << 2) + list));
            swc1(last, lwc1(lw(anim + 4) * 12 + list + (axis << 2) + 4));
        }
        8 => {
            let list = lw(data + 20);
            swc1(first, lwc1((axis << 2) + list));
            swc1(last, lwc1((axis << 2) + (lw(anim + 4) * 28 + list) - 28));
        }
        _ => {}
    }
    if lw(lw(lw(chr + 32) + 88) + 12) == 1 {
        scale = ps2float::mul(scale, lwc1(factor_at));
    }
    swc1(first, ps2float::mul(lwc1(first), scale));
    swc1(last, ps2float::mul(lwc1(last), scale));
}

/// The root movement of animation `anim_number` for a character (@ 0x215590).
pub fn extractAnimRootData(chr: i32, anim_number: i32, first: i32, last: i32, third: i32, axis: i32) {
    anim_root(chr, lw((anim_number << 2) + g::pAnimations), first, last, third, axis, 0x3ac840);
}

/// The root movement of the animation a character is playing (its model's animation number is at +96) (@ 0x215420).
pub fn extractAnimRoot(chr: i32, first: i32, last: i32, third: i32, axis: i32) {
    let model = lw(chr + 32);
    anim_root(chr, lw((lw(model + 96) << 2) + g::pAnimations), first, last, third, axis, 0x3ac83c);
}
