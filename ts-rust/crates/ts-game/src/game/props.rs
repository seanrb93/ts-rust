//! Props (the game's interactive objects: doors, fans, crates, ...) and the table that describes each kind.
//!
//! The prop table has a 28-byte entry per prop number (`PROPTABLE`, the port's relocated copy of the game's `proptable`): the type at +0, the
//! object (model) at +4, flags at +24.

use crate::*;

/// Flags of a prop table entry.
const NEEDS_BOUNDS: i32 = 1;
const NEEDS_WALLS_FLOORS: i32 = 6;
const COLLIDES_CAN_PUSH: i32 = 8;
const COLLIDES_CAN_NOT_PUSH: i32 = 16;

fn entry(prop: i32) -> i32 {
    prop * 28 + PROPTABLE
}

/// The prop's type (@ 0x267200).
pub fn getPropType(prop: i32) -> i32 {
    lw(entry(prop))
}

/// Give a prop its object (@ 0x2671c0).
pub fn propSetOb(prop: i32, ob: i32) {
    sw(entry(prop) + 4, ob);
}

/// Whether the prop's bounds have to be worked out (@ 0x2684d8).
pub fn propNeedsBoundsCalculating(prop: i32) -> i32 {
    lw(entry(prop) + 24) & NEEDS_BOUNDS
}

/// Whether walls and floors have to be worked out for the prop (@ 0x2684b8).
pub fn propNeedsWallsFloorsCalculating(prop: i32) -> i32 {
    lw(entry(prop) + 24) & NEEDS_WALLS_FLOORS
}

/// Non-zero if the prop collides and can push what it hits (@ 0x268518).
pub fn propDoCollisionsCanPush(prop: i32) -> i32 {
    lw(entry(prop) + 24) & COLLIDES_CAN_PUSH
}

/// Non-zero if the prop collides and cannot push (@ 0x2684f8).
pub fn propDoCollisionsCanNotPush(prop: i32) -> i32 {
    lw(entry(prop) + 24) & COLLIDES_CAN_NOT_PUSH
}

/// Set the handlers of a prop instance: slot `slot` of its table at +360 (12 bytes a slot) takes two words (@ 0x2671e0).
pub fn propSetHandler(inst: i32, slot: i32, handler: i32, data: i32) {
    let s = inst + slot * 12;
    sw(s + 368, data);
    sw(s + 360, handler);
}

/// Empty in the build (@ 0x27c178).
pub fn propEditable() {}

/// Mark a door prop as activated (bit 0 of the flags at +28) (@ 0x269960).
pub fn doorActivate(prop: i32) {
    sw(prop + 28, lw(prop + 28) | 1);
}

/// The data of door `n`: the list of door props at 0x1fd0da0 holds the instance, whose +352 is its door record.
fn door(n: i32) -> i32 {
    lw(lw((n << 2) + 0x1fd0da0) + 352)
}

/// Lock door `n` (@ 0x26c138).
pub fn doorLock(n: i32) {
    sw(door(n) + 20, 1);
}

/// Unlock door `n` (@ 0x26c160).
pub fn doorUnlock(n: i32) {
    sw(door(n) + 20, 0);
}

/// Open door `n` (@ 0x26c180).
pub fn doorOpen(n: i32) {
    sw(door(n) + 24, 1);
}

/// Close door `n` (@ 0x26c1a8).
pub fn doorClose(n: i32) {
    sw(door(n) + 24, 0);
}
