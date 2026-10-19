//! Loading the map maker's tiles, stepping a character along its path, and the animation test.

use crate::*;

/// Set up the 21 tile sets (records of 116 bytes at `tileset1`): each is listed in `tiles`, the offsets of its layers (from +12) are made from its width and height, and the
/// three turned copies of it are made (which add tiles of their own to the count) (@ 0x249278).
pub fn mmLoadTiles() {
    sw(g::numtiles, 0);
    let mut number = 0;
    loop {
        let tile = number * 116 + g::tileset1;
        let layers = lw(tile + 8);
        sw((number << 2) + g::tiles, tile);
        let mut layer = 1;
        if layer < layers {
            let width = lw(tile);
            let layer_offsets = tile + 12;
            let height = lw(tile + 4);
            loop {
                let at = layer_offsets + (layer << 2);
                let previous = lw(layer_offsets + ((layer - 1) << 2));
                layer += 1;
                sw(at, previous + width * height);
                if layer >= layers {
                    break;
                }
            }
        }
        mmMakeTileRotations(tile);
        number = lw(g::numtiles) + 1;
        sw(g::numtiles, number);
        if number >= 21 {
            break;
        }
    }
}

/// Take the next step of a character's path (the record `path`: its current pad at +0, its goal pad at +4, the links of a route inside a hall from +8 (their count at +1008), the
/// halls of a route between halls from +1012 (their count at +2012)): within one hall the route is made at once by `routeCalc`, between halls `hallrouteCalc` and then a route
/// inside the next hall. Returns the pad to go to next, -1 when it is already at the goal or the goal is not a pad, -2 when no route is found (@ 0x2bf680).
pub fn findNextPad(path: i32) -> i32 {
    let goal = lw(path + 4);
    let mut current = lw(path);
    if current == goal {
        sw(path + 2012, 0);
    } else {
        if goal >= 0 {
            if current != -1 {
                let mut length = lw(path + 1008);
                let list = path + 8;
                let links;
                if length == 0 {
                    let pads = lw(g::levelpads);
                    if lw(path + 2012) == 0 {
                        if lh(current * 28 + pads + 8) == lh(goal * 28 + pads + 8) {
                            let route = routeCalc(current, goal, list, 250);
                            current = lw(path);
                            length = route;
                            sw(path + 1008, route);
                        } else {
                            let halls = hallrouteCalc(current, goal, path + 1012, 250);
                            sw(path + 2012, halls);
                            if halls == 0 {
                                return -2;
                            }
                            current = lw(path);
                            length = lw(path + 1008);
                        }
                    }
                    links = lw(g::levellinks);
                    if length == 0 {
                        let pads = lw(g::levelpads);
                        let halls_left = lw(path + 2012) - 1;
                        let hall = lh(current * 28 + pads + 8);
                        sw(path + 2012, halls_left);
                        let link_number = lw(path + (halls_left << 2) + 1012);
                        sw(path + 8, link_number);
                        let link = link_number * 20 + links;
                        let mut other = lw(link + 4);
                        if lh(other * 28 + pads + 8) != hall {
                            other = lw(link + 8);
                        }
                        if current == other {
                            length = 1;
                            sw(path + 1008, 1);
                        } else {
                            let route = routeCalc(current, other, path + 12, 99);
                            let count = route + 1;
                            current = lw(path);
                            length = count;
                            sw(path + 1008, count);
                        }
                    }
                } else {
                    links = lw(g::levellinks);
                }
                let link = lw(list + ((length - 1) << 2)) * 20 + links;
                let mut next = lw(link + 4);
                if next == current {
                    next = lw(link + 8);
                }
                return next;
            }
            sw(path + 2012, 0);
            sw(path + 1008, 0);
            return -2;
        }
        sw(path + 2012, 0);
    }
    sw(path + 1008, 0);
    -1
}

/// Step through the animations of the first character by the buttons, to look at them: the first button moves on the number of the animation (223 to 442), the second the
/// character's kind (0 to 9); the animation is started when it is not the one playing, and the character's two height values follow the player's (@ 0x283e90).
pub fn testanims() {
    let player = lw(g::currentplayer);
    let chr = lw(player + 384);
    let buttons = joyNewBut(lw(player));
    let player_now = lw(g::currentplayer);
    let playing = lw(g::playanim);
    if (buttons & lw(player_now + 36)) != 0 {
        let next = playing + 1;
        sw(g::playanim, next);
        if next >= 443 {
            sw(g::playanim, 223);
        }
    }
    let buttons = joyNewBut(lw(player_now));
    let mut anim = lw(g::playanim);
    if (buttons & lw(lw(g::currentplayer) + 40)) != 0 {
        let kind = lw(chr + 344) + 1;
        sw(chr + 344, kind);
        if kind >= 10 {
            sw(chr + 344, 0);
        }
        anim = lw(g::playanim);
    }
    let current = curAnim(chr, anim);
    let mut player_after = lw(g::currentplayer);
    if current == 0 {
        setAnim(chr, lw(g::playanim), 0.0_f32, 1.0_f32);
        player_after = lw(g::currentplayer);
    }
    swc1(chr + 80, lwc1(player_after + 276));
    swc1(chr + 84, lwc1(player_after + 276));
}
