//! Resetting the portal visibility data of a frame, and choosing the room of a prop.

use crate::*;

/// Reset the portals (records of 48 bytes after the first) and the first player's room data for a new frame (@ 0x257680).
pub fn bgPortalCalcInit() {
    let player = lw(g::currentplayer);
    let portals = lw(g::portaldata);
    for i in 0..lw(g::bgNumPortals) {
        let rec = i * 48 + portals + 48;
        sw(rec + 36, -1);
        sw(rec + 40, 1);
        sw(rec + 24, 640);
        sw(rec + 28, 224);
        sw(rec + 44, 0);
        sw(rec + 32, -1);
    }
    let rooms = lw(g::bgNumRooms);
    if rooms > 0 {
        let seen = lw(0x3b0084);
        for i in 0..rooms {
            let entry = i * 20 + lw((lw(player) << 2) + g::roomdata);
            sw(entry + 20, -1);
            sw(i * 20 + lw((lw(player) << 2) + g::roomdata) + 32, -1);
            sw((i << 2) + seen + 4, 0);
        }
    }
    sw((lw(player) << 2) + g::maxlevel, -1);
}

/// Choose the room a prop is in when it touches several (its list of rooms from +144, the count at +184): the visible one with the lowest
/// level.  A prop with flag 0x40000000 is attached to another and keeps its room (@ 0x269c88).
pub fn propSetRoom(prop: i32) {
    if lw(prop + 16) & 0x40000000 != 0 {
        return;
    }
    if lw(prop + 184) >= 2 {
        let mut best = 999;
        let mut best_room = 0;
        let mut i = 0;
        while i < lw(prop + 184) {
            let room = lw(prop + 144 + (i << 2));
            let level = bgRoomGetLevel(room);
            if level < best && level >= 0 && isroomvisible(room) != 0 {
                best = level;
                best_room = room;
            }
            i += 1;
        }
        if best_room > 0 {
            sw(prop + 12, best_room);
        }
    } else {
        sw(prop + 12, lw(prop + 144));
    }
}
