//! Choosing a pad to fetch a weapon from, the pad the mouse is over in the map maker, and taking away what is not unlocked.

use crate::*;

/// Choose a prop to go to (the props at the places of the extra data of the character's pad, `levelpadsextra`, 5 of them from +8 in records of 28 bytes) for the character
/// `chr`: of those that carry flag 1 and whose own pad (+36 of their state) is not this pad and is not one of the camping pads, one at random (kept on the stack); with none, the
/// last prop of the pad with flag 1 that is on another pad, or the last on another pad, or 0 (@ 0x2c4f98).
pub fn selectWeaponPad(chr: i32) -> i32 {
    let frame = Frame::new(144);
    let sp = frame.sp();
    let mut count = 0;
    let mut store = sp;
    let mut slot = 4;
    let state = lw(chr + 352);
    let mut pad = lw(state + 680);
    loop {
        let offset = slot << 2;
        let prop = lw(pad * 28 + lw(g::levelpadsextra) + offset + 8);
        if prop == 0 {
            slot -= 1;
        } else if (lw(prop + 16) & 1) == 0 {
            slot -= 1;
        } else {
            let other_pad = lw(lw(prop + 352) + 36);
            slot -= 1;
            if pad != other_pad && campingPad(other_pad) == 0 {
                count += 1;
                sw(store, lw(lw(state + 680) * 28 + lw(g::levelpadsextra) + offset + 8));
                store += 4;
            }
        }
        if slot < 0 {
            break;
        }
        pad = lw(state + 680);
    }
    if count == 1 {
        return lw(sp);
    }
    if count < 2 {
        let pad = lw(state + 680);
        let extra = lw(g::levelpadsextra);
        let mut left = 4;
        let mut at = pad * 28 + extra + 24;
        loop {
            let prop = lw(at);
            left -= 1;
            if prop != 0 && (lw(prop + 16) & 1) != 0 && pad != lw(lw(prop + 352) + 36) {
                return prop;
            }
            at -= 4;
            if left < 0 {
                break;
            }
        }
        let mut left = 4;
        let mut at = pad * 28 + extra + 24;
        loop {
            let prop = lw(at);
            left -= 1;
            if prop != 0 && pad != lw(lw(prop + 352) + 36) {
                return prop;
            }
            at -= 4;
            if left < 0 {
                break;
            }
        }
        return 0;
    }
    let random = newrnd();
    if count == 0 {
        trap("break");
    }
    let pick = ((random as i32 as u32) % (count as u32)) as i32;
    lw(sp + (pick << 2))
}

/// Find the pad of the map maker (records of 24 bytes at 0x1fc4108, their number at 0x3b0078) that the mouse is nearest to on the screen, while in a mode that shows them (flags 1
/// and 2 of `zoomtileflags`): each pad's place goes through the player's two matrices to the screen, and the nearest in front of the camera (depth under 1; of two within 126 of each
/// other in distance the one with the smaller depth) is kept in `aimpadinfo` (@ 0x250b38).
pub fn findHighlightPad() {
    let frame = Frame::new(208);
    let sp = frame.sp();
    sw(g::aimpadinfo, 0);
    if lw(g::zoomtileflags) & 3 == 0 {
        return;
    }
    let front = lw(g::front);
    let window_x = lw(g::window);
    let mut nearest = 0x7fffffff;
    let mut i = 0;
    let mouse_y = lh(front + 38) + lw(g::window + 20);
    let mut best_depth = 1.1920929e-07_f32;
    let mouse_x = lh(front + 34) + window_x;
    let mut count = lw(0x3b0078);
    if count > 0 {
        loop {
            let offset = i * 24;
            let pads = 0x1fc4108;
            let matrix = lw(lw(g::currentplayer) + 1764);
            let entry = pads + i * 24;
            let y = lwc1(entry + 16);
            let z = lwc1(entry + 20);
            swc1(sp, lwc1(entry + 12));
            swc1(sp + 4, y);
            swc1(sp + 8, z);
            matrixVecMulAligned(&[matrix as i64, sp as i64, entry as i64, window_x as i64]);
            let projection = lw(lw(g::currentplayer) + 1760);
            matrixVec3Mul4Aligned(&[projection as i64, sp as i64, (sp + 16) as i64]);
            let w = lwc1(sp + 28);
            let window = g::window;
            let depth = ((lwc1(sp + 24)).pdiv(w)) as i32;
            let x_screen = ps2float::add(lw(window + 8) as f32, ps2float::mul(lw(window + 16) as f32, ((lwc1(sp + 16)).pdiv(w)))) as i32;
            let y_screen = ps2float::sub(lw(window + 28) as f32, ps2float::mul(lw(window + 36) as f32, ((lwc1(sp + 20)).pdiv(w)))) as i32;
            count = lw(0x3b0078);
            if lw(g::window) < x_screen && x_screen < lw(window + 4) && lw(window + 20) < y_screen && y_screen < lw(window + 24) {
                count = lw(0x3b0078);
                let depth_f = depth as f32;
                if depth_f < 1.0_f32 {
                    let dx = x_screen - mouse_x;
                    let dy = (y_screen - mouse_y) << 1;
                    let distance = dx * dx + dy * dy;
                    let gap = nearest - distance;
                    if distance < nearest {
                        if gap < 126 && !(best_depth < depth_f) {
                            count = lw(0x3b0078);
                        } else {
                            nearest = distance;
                            sw(g::aimpadinfo, offset + pads);
                            best_depth = depth_f;
                        }
                    }
                    count = lw(0x3b0078);
                }
            }
            i += 1;
            if i >= count {
                break;
            }
        }
    }
}

/// Take away what is not unlocked in the choices of the game set-up: the weapon set 8 is changed to 3 when its set is not available (the weapon list gives up the entries of the
/// kind 32), a bot set that is not available is changed to 0, and the characters of the three teams' rows (10 bytes each at 0x1fc2b28) that are not available are cleared to -1 (@ 0x235210).
pub fn checkUnavailableThings() {
    sw(0x3aff68, 0);
    sw(0x3aff64, 0);
    sw(g::gunzlistrow, 0);
    sw(g::gunzlistrowtop, 0);
    if weaponsetAvail(8) == 0 {
        if (lw(g::gv_weaponset) - 1) == 8 {
            sw(g::gv_weaponset, 3);
        }
        let pack = g::gunzpack;
        let mut i = 0;
        loop {
            let next = i + 1;
            if lw((i << 2) + pack) == 32 {
                let mut k = i;
                if i < 4 {
                    loop {
                        let after = k + 1;
                        sw((k << 2) + pack, lw((after << 2) + pack));
                        k = after;
                        if k >= 4 {
                            break;
                        }
                    }
                }
                sw((k << 2) + pack, -1);
            }
            i = next;
            if i >= 5 {
                break;
            }
        }
    }
    let bot_set = lw(g::gv_chrset);
    if bot_set < 8 && botsetAvail(bot_set) == 0 {
        sw(g::gv_chrset, 0);
    }
    let mut team = 0;
    loop {
        let next = team + 1;
        let mut left = 9;
        let mut at = team * 10 + 0x1fc2b28;
        loop {
            let chr = lb(at);
            left -= 1;
            if chr >= 0 && chrAvail(chr) == 0 {
                sb(at, -1);
            }
            at += 1;
            if left < 0 {
                break;
            }
        }
        team = next;
        if team >= 3 {
            break;
        }
    }
}
