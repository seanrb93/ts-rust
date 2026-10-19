//! Putting the numbers of a tile (a part of a level) into the level: its textures and its rooms.

use crate::*;

/// Give the textures of the level to each room of the tile (its objects, 44 bytes each after the first) and move the texture numbers in the lists of their
/// glass (a list of pointers ended by 0, whose entries have the number at +4) by the tile's offset (+40) (@ 0x254d88).
pub fn bgTileSetTextureNums(tile: i32) {
    let mut i = 0;
    while i < lw(tile + 16) {
        let room = lw(tile) + (i * 44 + 44);
        obSetTexList(lw(room), lw(g::leveltextures), lw(tile + 40));
        let list = lw(room + 16);
        i += 1;
        if list != 0 && lw(list) != 0 {
            let mut k = 0;
            loop {
                let entry = lw(list + 4 * k);
                k += 1;
                let next = lw((k << 2) + list);
                sw(entry + 4, lw(entry + 4) + lw(tile + 40));
                if next == 0 {
                    break;
                }
            }
        }
    }
}

/// Move the room numbers of a tile by its offset (+32): those in its walls' lists (the first two words of each, unless negative), those in its portals (44 bytes
/// each), and give each room a prop (number 513 up) and a halfword in the table at 0x1fc4828 (@ 0x254bd0).
pub fn bgTileSetRoomNums(tile: i32) {
    if lw(tile + 20) > 0 {
        let list = lw(tile + 4);
        let mut i = 0;
        loop {
            let item = lw((i << 2) + list + 4);
            let first = lw(item);
            if first >= 0 {
                sw(item, first + lw(tile + 32));
            }
            let second = lw(item + 4);
            if second >= 0 {
                sw(item + 4, second + lw(tile + 32));
            }
            i += 1;
            if i >= lw(tile + 20) {
                break;
            }
        }
    }
    if lw(tile + 28) > 0 {
        let portals = lw(tile + 12);
        let mut i = 0;
        loop {
            let rec = i * 44 + portals;
            i += 1;
            sw(rec + 4, lw(rec + 4) + lw(tile + 32));
            if i >= lw(tile + 28) {
                break;
            }
        }
    }
    if lw(tile + 16) > 0 {
        let mut i = 0;
        loop {
            let offset = lw(tile + 32);
            propSetOb(offset + (i + 513), lw(i * 44 + lw(tile) + 44));
            let offset = lw(tile + 32);
            let at = ((offset + i) << 1) + 0x1fc4828;
            i += 1;
            sh(at, lhu(tile + 52));
            if i >= lw(tile + 16) {
                break;
            }
        }
    }
}
