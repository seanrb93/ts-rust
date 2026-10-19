//! Walking the bones of a skeleton (80-byte bones: kind at +0, child at +3, sibling at +4).

use crate::*;

/// Add `offset` to the texture offsets of the two lists of a bone (pointers at +8 and +12 to records of 24 bytes ended by a negative word at +20; the first
/// word of a record is changed unless it is negative), and of the bones below it (@ 0x260128).
pub fn partSetTextureOffset(skeleton: i32, bone: i32, offset: i32) {
    let mut bone = bone;
    loop {
        let bones = skeleton - lw(skeleton) * 80;
        let this = bones + bone * 80;
        for at in [8, 12] {
            let mut record = lw(this + at);
            if record != 0 && lw(record + 20) >= 0 {
                loop {
                    let v = lw(record);
                    if !(v < 0) {
                        sw(record, v + offset);
                    }
                    record += 24;
                    if lw(record + 20) < 0 {
                        break;
                    }
                }
            }
        }
        let child = lb(this + 3);
        if child >= 0 && lb(this) != 3 {
            partSetTextureOffset(skeleton, child, offset);
        }
        bone = lb(this + 4);
        if bone < 0 {
            break;
        }
    }
}

/// Copy the translation of bone `from` to the bone `to` that mirrors it, for the bones of kind 2, and walk the skeleton (80-byte bones; kind
/// at +0, matrix number at +1, child at +3, sibling at +4) (@ 0x214b58).
pub fn calDoubleFilletTrans(skeleton: i32, bone: i32) {
    let mut bone = bone;
    loop {
        let base = lw(skeleton);
        let bones = base - lw(base) * 80;
        let this = bones + bone * 80;
        let mut child = lb(this + 3);
        if lb(this) == 2 {
            let other = child * 80 + bones;
            for at in [48, 52, 56] {
                let matrices = lw(skeleton + 4);
                swc1((lb(this + 1) << 6) + matrices + at, lwc1((lb(other + 1) << 6) + matrices + at));
            }
            child = lb(this + 3);
        }
        if child >= 0 && lb(this) != 3 {
            calDoubleFilletTrans(skeleton, child);
        }
        bone = lb(this + 4);
        if bone < 0 {
            break;
        }
    }
}
