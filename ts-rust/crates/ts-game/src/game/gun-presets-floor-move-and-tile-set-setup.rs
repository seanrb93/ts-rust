//! Which gun combinations a level gives, a character moved by the thing it stands on, and the set-up of a tile set.

use crate::*;

/// The table of five gun combinations to start with: for the modes other than 7, mode 6 gives the preset of the level (5, 8, 9 or 12 their own, others the common one), else the
/// players' own choices (when the challenge does not name any: the front end's choices of five, put in a table at 0x1fbcc60), or the challenge's list at +24 (a preset, or the table
/// at +28 of the challenge when that is below 0); in mode 7 a cheat of all-ones (64) gives five of combination 11, a cheat 128 gives one preset, and else the level's set-up gives
/// its list (or the front end's choices when there is no set-up) (@ 0x227790).
pub fn lvGetGunComboNums() -> i32 {
    let parms = g::curgameparms;
    let mode = lw(parms + 72);
    if mode == 7 {
        let allowed = lw(g::allowedcheats);
        let both = lw(g::unlockedcheats) & lw(g::selectedcheats);
        if (allowed & 64 & both) != 0 {
            let mut left = 4;
            let mut at = 0x1fbcc70;
            loop {
                sw(at, 11);
                left -= 1;
                at -= 4;
                if left < 0 {
                    break;
                }
            }
            return 0x1fbcc60;
        }
        if (allowed & 128 & both) != 0 {
            return g::gunpresets + 340;
        }
        let setup = lvGetLevelSetup(lw(g::lvlevelnum));
        if setup == 0 {
            return front_gun_choices();
        }
        return 20 * lw(setup + 4) + g::gunpresets;
    }
    if mode == 6 {
        let level = lw(g::lvlevelnum);
        if level == 8 {
            return g::gunpresets + 40;
        }
        if level >= 9 {
            if level == 9 || level == 12 {
                return g::gunpresets + 140;
            }
        } else if level == 5 {
            return g::gunpresets + 80;
        }
        return g::gunpresets + 240;
    }
    let flags = lw(parms + 80);
    if (flags & 32) == 0 {
        return front_gun_choices();
    }
    let challenge = challengeGetSetup();
    let list = lw(challenge + 24);
    if list < 0 {
        return challenge + 28;
    }
    list * 20 + g::gunpresets
}

/// Put the front end's choice of gun combination for each of the five players in the table at 0x1fbcc60 and return its address.
fn front_gun_choices() -> i32 {
    let mut i = 0;
    let mut at = 0x1fbcc60;
    loop {
        let choice = frontmpgGetGunComboNum(i);
        i += 1;
        sw(at, choice);
        at += 4;
        if i >= 5 {
            break;
        }
    }
    0x1fbcc60
}

/// Move the position `pos` (x at +0, z at +8) and the angle `*angle` (degrees) of the character `index` by what the thing it stands on (+0x11a0 of its record, `plat`) moved
/// since the last tick; a platform of kind 310 (a lift that turns) whose model is not the first one, instead keeps the character at the same place on it: the character's
/// offset along and across the platform's line (from the two points at +16 and +20 of its state, now and before) is worked out and put on the new line, with the change of the
/// line's angle added to the character's (@ 0x285070).
pub fn chrAddFloorMoveToPos(index: i32, pos: i32, angle: i32) {
    let frame = Frame::new(112);
    let sp = frame.sp();
    let record = lw(g::chrinfo) + index * 0x1210;
    let plat = lw(record + 0x11a0);
    if plat == 0 {
        return;
    }
    let held = lw(record + 0x119c);
    let before_x;
    if held == 0 || lw(plat + 4) != 310 || held == lw(lw(plat + 480)) {
        before_x = lwc1(plat + 112);
    } else {
        let state = lw(plat + 352);
        let now = lw(state + 20);
        let old = lw(state + 16);
        let f2 = lwc1(now);
        swc1(sp, f2);
        let f3 = lwc1(now + 8);
        swc1(sp + 4, f3);
        let f0 = lwc1(now + 12);
        swc1(sp + 16, f0);
        let line_x = f2 - f0;
        let f1 = lwc1(now + 20);
        let line_z = f3 - f1;
        swc1(sp + 20, f1);
        swc1(sp + 48, lwc1(old));
        swc1(sp + 52, lwc1(old + 8));
        let length = (line_x * line_x + line_z * line_z).abs().sqrt();
        swc1(sp + 64, lwc1(old + 12));
        let old_z = lwc1(old + 20);
        swc1(sp + 32, line_x);
        swc1(sp + 68, old_z);
        swc1(sp + 36, line_z);
        let unit_x = lwc1(sp + 32) / length;
        let x = lwc1(pos);
        let unit_z = lwc1(sp + 36) / length;
        let z = lwc1(pos + 8);
        let start_x = lwc1(sp + 16);
        let start_z = lwc1(sp + 20);
        let rel_x = x - start_x;
        let old_start_x = lwc1(sp + 48);
        let rel_z = z - start_z;
        let old_start_z = lwc1(sp + 64);
        swc1(sp + 32, unit_x);
        swc1(sp + 36, unit_z);
        let old_end_z = lwc1(sp + 68);
        let along = rel_x * unit_x + rel_z * unit_z;
        let old_unit_z = (lwc1(sp + 52) - old_end_z) / length;
        let across_x = rel_x - along * unit_x;
        let across_z = rel_z - along * unit_z;
        swc1(sp + 80, (old_start_x - old_start_z) / length);
        swc1(sp + 96, across_x);
        swc1(sp + 100, across_z);
        let mut across = (across_x * across_x + across_z * across_z).abs().sqrt();
        swc1(sp + 84, old_unit_z);
        if (lwc1(sp + 36) * (x - start_x) - lwc1(sp + 32) * (z - start_z)) < 0.0_f32 {
            across = -across;
        }
        let new_unit_x = lwc1(sp + 80);
        let new_unit_z = lwc1(sp + 84);
        swc1(pos + 8, old_end_z + along * new_unit_z - across * new_unit_x);
        swc1(pos, old_start_z + along * new_unit_x + across * new_unit_z);
        let new_angle = atan2f(new_unit_x, new_unit_z);
        let line_angle = atan2f(lwc1(sp + 32), lwc1(sp + 36));
        swc1(angle, lwc1(angle) + (new_angle - line_angle) * 1.8e+02_f32 / lwc1(0x3ad198));
        return;
    }
    let z = lwc1(pos + 8);
    swc1(pos, lwc1(pos) + (lwc1(plat + 48) - before_x));
    swc1(pos + 8, z + (lwc1(plat + 56) - lwc1(plat + 120)));
    swc1(angle, lwc1(angle) + (lwc1(plat + 76) - lwc1(plat + 128)));
}

/// Find the counts a tile set needs in the set-up `setup` (a record of 112 bytes, cleared first) by reading the chunks of the map maker's temporary buffer (8-byte headers, the
/// kind and the size, from +8; the buffer holds 0x1ffc bytes): the chunk 0x1324627 gives the number of records, each record of chunk 0x1324628 (36 bytes and a list of 4-byte
/// entries, `lhu` +10 of the record the count of entries) adds 1 to the count at +36 for an entry of kind below 5 and 1 to the count at +20 for kinds 5 to 19; chunk 0x1324629
/// ends. Then the memory of the +20 count of 48-byte records is taken for +16 and of the +36 count of 16-byte records for +32 (@ 0x226fd0).
pub fn lvAllocTileSetSetup(setup: i32) {
    memset(setup, 0, 112);
    let buffer = lw(g::mmtempbuffer);
    let limit = buffer + 0x1ffc;
    let mut chunk = buffer + 8;
    let mut count = 0;
    'scan: {
        if (chunk as u32) < (limit as u32) {
            let mut tag = lw(buffer + 8);
            let mut data = buffer + 16;
            if tag != 0x1324629 {
                let mut entries = lw(setup + 20);
                loop {
                    'record: {
                        if tag != 0x1324627 {
                            if tag != 0x1324628 {
                                break 'record;
                            }
                            let mut at = data;
                            let mut done = 0;
                            if count > 0 {
                                let mut size = lhu(at + 10);
                                loop {
                                    let mut k = 0;
                                    at += 36;
                                    if size == 0 {
                                        done += 1;
                                    } else {
                                        let next = done + 1;
                                        loop {
                                            let kind = lhu(at);
                                            if (kind as u32) >= 5 {
                                                let more = entries + 1;
                                                if (((kind - 5) & 0xffff) as u32) < 15 {
                                                    sw(setup + 20, more);
                                                    entries = more;
                                                }
                                            } else {
                                                sw(setup + 36, lw(setup + 36) + 1);
                                            }
                                            k += 1;
                                            at += 4;
                                            if k >= size {
                                                break;
                                            }
                                        }
                                        done = next;
                                    }
                                    if done >= count {
                                        break 'record;
                                    }
                                    size = lhu(at + 10);
                                }
                            }
                        } else {
                            count = lw(data + 4);
                        }
                    }
                    let size = lw(chunk + 4);
                    let next = data + size;
                    chunk = next;
                    if (next as u32) >= (limit as u32) {
                        break 'scan;
                    }
                    tag = lw(chunk);
                    data = chunk + 8;
                    if tag == 0x1324629 {
                        break 'scan;
                    }
                }
            }
        }
    }
    let records = lw(setup + 20);
    if records > 0 {
        let memory = memAlloc(records * 48, 1);
        sw(setup + 16, memory);
    }
    let small = lw(setup + 36);
    if small > 0 {
        let memory = memAlloc(small << 4, 1);
        sw(setup + 32, memory);
    }
}
