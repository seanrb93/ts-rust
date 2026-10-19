//! Which room each prop is in, and the lists of the props of each room for drawing.

use crate::*;

/// Choose the room of every prop that is not attached to another (flags 0xc0000000 clear) (@ 0x269ef8).
pub fn propUpdateRooms() {
    let mut i = 0;
    while i < lw(g::num_props) {
        let prop = lw(g::levelprops) + 592 * i;
        i += 1;
        if (ld(prop + 16) & 0xc0000000_i64) == 0 {
            propSetRoom(prop);
        }
    }
}

/// Make the lists of the props of each room for `propGfxRoom`: clear the counts, then put each prop (not the sky, type 2, unless the
/// background is shown) on the list of its room (a count and 100 props, 400 bytes, for each room) (@ 0x26fd40).
pub fn propGfxRoomPreproc() {
    let count = lw(g::num_props);
    for i in 0..=ROOM_LAST {
        sw(ROOM_PROP_COUNTS + 4 * i, 0);
    }
    if count > 0 {
        let props = lw(g::levelprops);
        let show_background = lw(g::viewbg);
        for i in 0..count {
            let prop = i * 592 + props;
            if show_background == 0 && lw(prop + 8) == 2 {
                continue;
            }
            let room = lw(prop + 12);
            let at = (room << 2) + ROOM_PROP_COUNTS;
            let n = lw(at);
            if n < 100 && (room as u32) < (ROOM_LIMIT as u32) {
                sw(at, n + 1);
                sw((n << 2) + room * 400 + ROOM_PROP_LISTS, prop);
            }
        }
    }
}
