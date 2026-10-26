//! Functions: updating the statistics when the game ends, drawing the sign on players page, testing a swipe against a prop, making the corpse of a prop, the special props'
//! dispatcher, resetting the multiplayer front end, drawing the page background and the tick of a level.

use crate::*;

/// Update the statistics for the end of the game (@ 0x21ccd8): calculate the statistics and medals and, unless a cheat is on or the game is a demo, add the played level to
/// the players' records (games played and time, first place and best times in the story levels, challenge results, kills, losses and the totals of the life) before the players'
/// statistics are combined. Returns 1 when the statistics were updated.
pub fn gameStatusUpdated() -> i32 {
    StatsCalc();
    StatsCalcMedals();
    let mut result = 0;
    if (lw(g::allowedcheats) & lw(g::selectedcheats) & lw(g::unlockedcheats)) == 0 {
        let params = g::curgameparms;
        if (lw(params + 80) & 0x2000) == 0 {
            let mut now = g::curgameparms;
            if lw(params + 72) == 7 {
                let mut players = lw(g::lvnumplayers);
                if (lw(g::lvlevelnum) as u32) < (30 as u32) {
                    let mut i = 0;
                    let frame = lw(g::gameframe);
                    if players > 0 {
                        loop {
                            let record = statsGetPlayer(i) + (lw(g::lvlevelnum) * 36 + 320);
                            let time = lw(record + 4) + frame;
                            sw(record, lw(record) + 1);
                            sw(record + 4, time);
                            players = lw(g::lvnumplayers);
                            if (lw(params + 80) & 0x10000000) != 0 {
                                let wins = statsGetPlayer(i);
                                sw(wins + 1956, lw(wins + 1956) + 1);
                                players = lw(g::lvnumplayers);
                            }
                            i += 1;
                            if i >= players {
                                break;
                            }
                        }
                    }
                    now = g::curgameparms;
                    if (lw(g::curgameparms + 80) & 0x30000000) == 0x10000000 {
                        let mut i = 0;
                        if players > 0 {
                            let mut offset = 0;
                            loop {
                                if lw(offset + lw(g::players) + 296) == 4 {
                                    i += 1;
                                } else {
                                    let record = statsGetPlayer(i) + (lw(g::lvlevelnum) * 36 + 320);
                                    let current = g::curgameparms;
                                    players = lw(g::lvnumplayers);
                                    let best = (lw(current + 76) << 2) + (if (players ^ 1) == 0 { record + 8 } else { record + 20 });
                                    let old = lw(best);
                                    if old != 0 && (old as u32) < (frame as u32) {
                                        i += 1;
                                    } else {
                                        sw(current + 80, lw(current + 80) | 0x40000000 << (i & 31));
                                        sw(best, frame);
                                        i += 1;
                                    }
                                }
                                offset += 1820;
                                if i >= players {
                                    break;
                                }
                            }
                        }
                        now = g::curgameparms;
                    }
                }
            }
            let mut players = lw(g::lvnumplayers);
            if (lw(now + 80) & 32) != 0 {
                let record = statsGetPlayer(0) + ((lw(now + 68) << 3) + 1400);
                if (lw(now + 80) & 0x10000000) != 0 {
                    if challengeIsBestScore() != 0 {
                        sw(now + 80, lw(now + 80) | 0x40000000);
                        let score = challengeGetScore();
                        sw(record + 4, score);
                    }
                    sw(record, lw(record) + 1);
                }
                players = lw(g::lvnumplayers);
            }
            let mut i = 0;
            if players > 0 {
                let mut offset = 0;
                loop {
                    let owner = lw(lw(offset + lw(g::players) + 384) + 352);
                    if (lhu(owner + 4) as u32) >= (80 as u32) {
                        i += 1;
                    } else {
                        let slot = statsGetPlayer(i) + (lh(owner + 4) << 2);
                        sw(slot, lw(slot) + 1);
                        players = lw(g::lvnumplayers);
                        i += 1;
                    }
                    offset += 1820;
                    if i >= players {
                        break;
                    }
                }
            }
            let mut i = 0;
            if players > 0 {
                let params = g::curgameparms;
                let mut flags = lw(params + 80);
                loop {
                    let count;
                    if (flags & 32) == 0 {
                        if lw(params + 72) != 7 {
                            let record = statsGetPlayer(i) + (lw(g::lvlevelnum) * 36 + 320);
                            sw(record + 32, lw(record + 32) + 1);
                            let life = statsGetPlayer(i);
                            sw(life + 2012, lw(life + 2012) + 1);
                            let life = statsGetPlayer(i);
                            StatsAddToLife(life + 2012, i);
                            count = lw(g::lvnumplayers);
                        } else {
                            let life = statsGetPlayer(i);
                            sw(life + 1616, lw(life + 1616) + 1);
                            let life = statsGetPlayer(i);
                            StatsAddToLife(life + 1616, i);
                            count = lw(g::lvnumplayers);
                        }
                    } else {
                        let life = statsGetPlayer(i);
                        sw(life + 2408, lw(life + 2408) + 1);
                        let life = statsGetPlayer(i);
                        StatsAddToLife(life + 2408, i);
                        count = lw(g::lvnumplayers);
                    }
                    i += 1;
                    if i >= count {
                        break;
                    }
                    flags = lw(params + 80);
                }
            }
            statsCombine(lw(g::lvnumplayers));
            result = 1;
        }
    }
    result
}

/// Draw the sign on players page (@ 0x23e4b0): the name of the player or the number of the player waiting for a card, the sign on slots with their colours while the
/// player picks a slot and the string edit while the name is typed.
pub fn signonplr_pageDraw(a0: i32) {
    let frame = Frame::new(176);
    let sp = frame.sp();
    let first = lbu(0x3aff48);
    sw(sp + 8, 0);
    'edit: {
        if first == 0 {
            if lbu(0x3aff4a) != 0 {
                break 'edit;
            }
            if lbu(0x3aff40) != 0 {
                break 'edit;
            }
            if lbu(0x3aff94) != 0 {
                break 'edit;
            }
            sw(sp + 8, ((lw(0x1fc1488) as u32) < (1 as u32)) as i32);
        }
    }
    let front = lw(g::front);
    if lw(front) > 0 {
        menuitemHighlight(a0);
    }
    let old_font = textSelect(2);
    sw(sp + 12, old_font);
    let mut front = lw(g::front);
    if lbu(lw(front) + 0x3aff28) != 1 {
        let (bg, fg) = if lw(sp + 8) != 0 { (0x5080, 0x40787880) } else { (128, 0x28282880) };
        let player = lw(front);
        let language = lw(g::cur_language);
        let x;
        let y;
        let text;
        if lbu(player + 0x3aff28) != 5 {
            let format = lw(lw((language << 2) + g::languages) + 1304);
            text = vafmt(&[0x3ae890, format as i64, (player + 1) as i64]);
            let list = g::signonplr_list;
            let window = g::window;
            y = lw(window + 20) + (lh(list + 6) >> (lw(window + 92) & 31)) - 15;
            x = lw(g::window) + (lh(list + 4) >> (lw(window + 88) & 31));
        } else {
            let signon = signonGet(player);
            let name = lw(g::signonarray) + signon * 2936;
            if textWidth(name) >= 301 {
                textSelect(1);
            }
            let width = textWidth(name);
            let window = g::window;
            y = lw(window + 20) + (lh(g::signon_ready_waiting + 6) >> (lw(window + 92) & 31)) - 15;
            text = name;
            x = lw(window + 8) - ((((width as u32) >> 31) as i32 + width) >> 1);
        }
        textOutline(x, y, text, fg, bg);
        front = lw(g::front);
    }
    if lw(sp + 8) != 0 {
        if (lw((lw(front) << 2) + 0x1fc1200) & 8) != 0 {
            let list = g::signonplr_list;
            let window = g::window;
            let y = lw(window + 20) + 5 + (lhu(list + 2) * 11 + (lh(list + 6) >> (lw(window + 92) & 31)));
            let fg = lw(g::menuitem_fgcol);
            sw(sp, lw(g::window) + (lh(list + 4) >> (lw(window + 88) & 31)));
            sw(sp + 4, y);
            textSelect(1);
            textPrint(sp, sp | 4, lw(lw((lw(g::cur_language) << 2) + g::languages) + 1308), fg);
            front = lw(g::front);
        }
    }
    if lbu(lw(front) + 0x3aff28) == 2 {
        let slots = g::somi;
        textSelect(1);
        let window = g::window;
        let header = slots - 16;
        let top = lw(window + 20) + (lh(header + 6) >> (lw(window + 92) & 31));
        let left = lw(g::window) + (lh(header + 4) >> (lw(window + 88) & 31));
        let row_height = if (lbu(g::fontflags) & 1) == 0 { lw(lw(g::curfont) + 12) - 1 } else { (lw(lw(g::curfont) + 12) >> 1) - 1 };
        let mut i = 0;
        if lhu(g::signonplr_list + 2) != 0 {
            let mut front = lw(g::front);
            let mut list = g::signonplr_list;
            loop {
                let slot = lhu(((lbu(list + lw(front) + 12) + i) << 5) + slots + 8) - 1;
                let count = signonCount();
                let mut colour = 0;
                if slot < count {
                    dlSetBlend(2);
                    colour = lw((lw((lbu(slot + 0x1fc3310) << 3) + g::signoninfo) << 2) + g::mcseq_slotcolours);
                }
                let offset = i * row_height;
                i += 1;
                let y = offset + top + 1;
                dlFillRectangle(left, y, left + 173, y + row_height - 2, colour);
                dlSetBlend(1);
                list = g::signonplr_list;
                front = lw(g::front);
                if i >= lhu(list + 2) {
                    break;
                }
            }
        }
        if lw(sp + 8) != 0 {
            let found = frontGetBox(a0, lhu(a0 + 4));
            if found != 0 {
                let alpha = lbu(g::flashalpha);
                let y = (lw(g::window + 20) + lhu(found + 2) + 2) as f32;
                let x = (lw(g::window) + lhu(found) - 24) as f32;
                let colour = ((((alpha as u32) >> 1) as i32) << 16) | alpha << 8 | 128;
                guiDrawImage(21, colour, x, y);
            }
        }
    }
    textSelect(lw(sp + 12));
    let player = lw(lw(g::front));
    if lbu(player + 0x3aff28) == 4 {
        stringeditDraw(player, a0, lw(sp + 8));
    }
}

/// Test a swipe from the position `a1` to the targets `a2` and `a3` against the prop `a0` (@ 0x271650): the swipe starts at the position of the prop's owner (the player's
/// eye or the character's head), crosses the portals from the prop's room to find the room it ends in, clamps the target to the box between the two points, tests the glass
/// of the rooms the box overlaps and finally does a line test; the hit position and normal go to `a4` and `a5` when they are given. Returns the line test's result.
pub fn propSwipeTest(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32) -> i32 {
    let frame = Frame::new(416);
    let sp = frame.sp();
    let mut room = 0;
    let owner = lw(a0 + 356);
    sw(sp + 228, a3);
    sw(sp + 224, a2);
    sw(sp + 232, a4);
    sw(sp + 236, a5);
    let mut current = lw(a0 + 12);
    let z;
    if owner == 0 {
        let player = lw(a0 + 352);
        swc1(sp + 192, lwc1(player + 2808));
        swc1(sp + 196, lwc1(player + 2812));
        z = lwc1(player + 2816);
    } else {
        swc1(sp + 192, lwc1(owner + 152));
        swc1(sp + 196, lwc1(owner + 156));
        z = lwc1(owner + 160);
    }
    swc1(sp + 200, z);
    let target_z = lwc1(a1 + 8);
    let start_z = lwc1(sp + 200);
    let dy = ps2float::sub(lwc1(a1 + 4), lwc1(sp + 196));
    swc1(sp + 208, ps2float::sub(lwc1(a1), lwc1(sp + 192)));
    swc1(sp + 212, dy);
    swc1(sp + 216, ps2float::sub(target_z, start_z));
    let mut previous = 0;
    sw(sp + 248, sp + 176);
    sw(sp + 240, sp + 144);
    sw(sp + 244, sp + 160);
    if current != previous {
        loop {
            room = bgPortalCrossed(sp + 192, sp + 208, current, previous, 0);
            previous = current;
            current = room;
            if room == previous {
                break;
            }
        }
    }
    let mut high = sp + 28;
    let mut upper = lw(sp + 228);
    let mut lower = lw(sp + 224);
    let mut low = sp + 16;
    let high_end = high + 12;
    let mut point = a1;
    loop {
        'low: {
            let value = lwc1(point);
            let clamped;
            if !(value < lwc1(lower)) {
                clamped = lwc1(lower);
            } else {
                if value < lwc1(upper) {
                    swc1(low, value);
                    break 'low;
                }
                clamped = lwc1(lower);
            }
            let top = lwc1(upper);
            if !(clamped < top) {
                swc1(low, top);
            } else {
                swc1(low, clamped);
            }
        }
        'high: {
            let value = lwc1(point);
            let clamped;
            if !(lwc1(lower) < value) {
                clamped = lwc1(lower);
            } else {
                if lwc1(upper) < value {
                    swc1(high, value);
                    break 'high;
                }
                clamped = lwc1(lower);
            }
            let top = lwc1(upper);
            if !(top < clamped) {
                swc1(high, top);
            } else {
                swc1(high, clamped);
            }
        }
        high += 4;
        upper += 4;
        lower += 4;
        point += 4;
        low += 4;
        if high >= high_end {
            break;
        }
    }
    let rooms = bgFloorBBIntersectionXZ(sp + 16, sp + 48, 20);
    if rooms > 0 {
        let mut list = sp + 48;
        let mut left = rooms;
        let mut glass_room = lw(list);
        loop {
            let third = lw(sp + 224);
            let fourth = lw(sp + 228);
            list += 4;
            left -= 1;
            bgGlassSwipeTestRoom(glass_room, a0, a1, third, fourth);
            if left == 0 {
                break;
            }
            glass_room = lw(list);
        }
    }
    let target = lw(sp + 228);
    let z = lwc1(target + 8);
    let x = ps2float::sub(lwc1(target), lwc1(a1));
    let y = ps2float::sub(lwc1(target + 4), lwc1(a1 + 4));
    let flags = lw(a0 + 16);
    let z = ps2float::sub(z, lwc1(a1 + 8));
    let line = lw(sp + 248);
    swc1(sp + 176, x);
    let hit_a = lw(sp + 240);
    swc1(sp + 180, y);
    let hit_b = lw(sp + 244);
    swc1(sp + 184, z);
    sw(a0 + 16, flags & -33);
    sw(sp, 0);
    sw(sp + 8, 0);
    let result = bgLineTest(room, a1, line, 0, hit_a, hit_b, 0, 0);
    sw(a0 + 16, lw(a0 + 16) | 32);
    if result != 0 {
        let out_position = lw(sp + 232);
        let x = lwc1(sp + 144);
        if out_position != 0 {
            let y = lwc1(sp + 148);
            let z = lwc1(sp + 152);
            swc1(out_position, x);
            swc1(out_position + 8, z);
            swc1(out_position + 4, y);
        }
        let out_normal = lw(sp + 236);
        let x = lwc1(sp + 160);
        if out_normal != 0 {
            let y = lwc1(sp + 164);
            let z = lwc1(sp + 168);
            swc1(out_normal, x);
            swc1(out_normal + 8, z);
            swc1(out_normal + 4, y);
        }
    }
    result
}

/// Copy 32 bytes the way the compiled code does: three doublewords are read first, the first one is read and written, then the other three are written.
fn copy_32_bytes(dst: i32, src: i32) {
    let second = ld(src + 8);
    let third = ld(src + 16);
    let fourth = ld(src + 24);
    sd(dst, ld(src));
    sd(dst + 8, second);
    sd(dst + 16, third);
    sd(dst + 24, fourth);
}

/// Make the corpse of the prop `a0` (@ 0x273658): a new prop of the same kind is made with the same position, heading, matrices, definition and animation state, and takes one
/// of the 30 slots of corpse data (the slot index and the owner's position are kept). When no slot is free the new prop is deleted and 0 is returned.
pub fn propNewCorpse(a0: i32) -> i32 {
    let owner = lw(a0 + 352);
    let corpse = propNew(lw(a0 + 4), lwc1(a0 + 48), lwc1(a0 + 52), lwc1(a0 + 56), lwc1(a0 + 76));
    if (lw(a0 + 16) & 0x10000000) != 0 {
        sw(corpse + 16, lw(corpse + 16) | 0x10000000);
    }
    swc1(corpse + 36, lwc1(a0 + 36));
    swc1(corpse + 40, lwc1(a0 + 40));
    swc1(corpse + 44, lwc1(a0 + 44));
    swc1(corpse + 60, lwc1(a0 + 60));
    swc1(corpse + 64, lwc1(a0 + 64));
    swc1(corpse + 68, lwc1(a0 + 68));
    swc1(corpse + 72, lwc1(a0 + 72));
    swc1(corpse + 76, lwc1(a0 + 76));
    swc1(corpse + 88, lwc1(a0 + 88));
    swc1(corpse + 80, lwc1(a0 + 80));
    swc1(corpse + 84, lwc1(a0 + 84));
    swc1(corpse + 88, lwc1(a0 + 88));
    swc1(corpse + 92, lwc1(a0 + 92));
    swc1(corpse + 96, lwc1(a0 + 96));
    swc1(corpse + 100, lwc1(a0 + 100));
    swc1(corpse + 104, lwc1(a0 + 104));
    swc1(corpse + 108, lwc1(a0 + 108));
    let word1 = lw(a0 + 136);
    swc1(corpse + 112, lwc1(a0 + 112));
    sw(corpse + 136, word1);
    let word2 = lw(a0 + 140);
    swc1(corpse + 116, lwc1(a0 + 116));
    sw(corpse + 140, word2);
    swc1(corpse + 120, lwc1(a0 + 120));
    swc1(corpse + 124, lwc1(a0 + 124));
    swc1(corpse + 128, lwc1(a0 + 128));
    swc1(corpse + 132, lwc1(a0 + 132));
    sceVu0CopyMatrix(corpse + 208, a0 + 208);
    sceVu0CopyMatrix(corpse + 272, a0 + 272);
    let value = lwc1(a0 + 552);
    sw(corpse + 336, lw(a0 + 336));
    swc1(corpse + 552, value);
    let half1 = lhu(a0 + 544);
    sw(corpse + 340, lw(a0 + 340));
    let value2 = lwc1(a0 + 556);
    let word3 = lw(a0 + 344);
    sh(corpse + 544, half1);
    sw(corpse + 344, word3);
    swc1(corpse + 556, value2);
    let half2 = lhu(a0 + 546);
    sw(corpse + 348, lw(a0 + 348));
    let value3 = lwc1(a0 + 560);
    let new_def = lw(corpse + 32);
    sw(corpse + 540, lw(a0 + 540));
    let def = lw(a0 + 32);
    let word4 = lw(a0 + 548);
    sh(corpse + 546, half2);
    sw(corpse + 548, word4);
    swc1(corpse + 560, value3);
    copy_32_bytes(new_def + 96, def + 96);
    copy_32_bytes(new_def + 128, def + 128);
    let new_def = lw(corpse + 32);
    let def = lw(a0 + 32);
    copy_32_bytes(new_def + 160, def + 160);
    copy_32_bytes(new_def + 192, def + 192);
    let def = lw(a0 + 32);
    let new_def = lw(corpse + 32);
    sh(new_def + 292, lhu(def + 292));
    let word5 = lw(def + 236);
    swc1(new_def + 224, lwc1(def + 224));
    let word6 = lw(def + 32);
    let value4 = lwc1(def + 228);
    sw(new_def + 236, word5);
    swc1(new_def + 228, value4);
    sw(new_def + 32, word6);
    let word7 = lw(def + 240);
    let word8 = lw(def + 36);
    swc1(new_def + 232, lwc1(def + 232));
    sw(new_def + 240, word7);
    sw(new_def + 36, word8);
    let mut row = 0;
    let mut offset = row * 12;
    loop {
        let next = row + 1;
        let mut count = 2;
        let mut from = offset + (def + 64);
        let mut to = offset + (new_def + 64);
        loop {
            count -= 1;
            swc1(to - 24, lwc1(from - 24));
            let value = lwc1(from);
            from += 4;
            swc1(to, value);
            to += 4;
            if count < 0 {
                break;
            }
        }
        row = next;
        offset = row * 12;
        if row >= 2 {
            break;
        }
    }
    sw(corpse + 8, 0x1000);
    let mut slot = 0;
    let mut index = 0;
    let mut full = false;
    if lw(g::corpseDataUsed) != 0 {
        slot += 1;
        loop {
            if slot >= 30 {
                full = true;
                break;
            }
            let used = (slot << 2) + g::corpseDataUsed;
            if lw(used) == 0 {
                index = slot;
                sw(used, 1);
                sw(corpse + 352, (index << 4) + g::corpseData);
                break;
            }
            slot += 1;
        }
    } else {
        sw(g::corpseDataUsed, 1);
        index = 0;
        sw(corpse + 352, g::corpseData);
    }
    let _ = full;
    if slot != 30 {
        let data = lw(corpse + 352);
        let x = lwc1(owner + 2968);
        sw(data, index);
        swc1(data + 4, x);
        let flags = lw(corpse + 16) & -2;
        swc1(data + 8, lwc1(owner + 2972));
        sw(corpse + 16, flags);
        swc1(data + 12, lwc1(owner + 2840));
        corpse
    } else {
        propDelete(corpse);
        0
    }
}

/// Tick of the special props (@ 0x27bfc8): the kind of the prop picks its own tick function (scanner, clock, signs, gong, fans, lanterns, electricity, wheels, space ships,
/// waterfalls, lift, spawners and the reflections); kinds without a tick do nothing.
pub fn specialPropTick(a0: i32) {
    let kind = lw(a0 + 4) - 246;
    if (kind as u32) < (74 as u32) {
        match kind {
            0 => scannerPropTick(a0),
            1 => clockPropTick(a0),
            2 => pubsignPropTick(a0),
            3 => boatsignPropTick(a0),
            4 => gongPropTick(a0),
            5 => fanPropTick(a0),
            6 => fanshadowPropTick(a0),
            7 => fanscifiPropTick(a0),
            8 => bannerPropTick(a0),
            9 => streamersPropTick(a0),
            10 => lanternPropTick(a0),
            12 => electricityPropTick(a0),
            13 => outerwheelsPropTick(a0),
            14 | 15 | 16 | 17 => innerwheelsPropTick(a0),
            18 => polePropTick(a0),
            19 => spaceship1PropTick(a0),
            20 => spaceship2PropTick(a0),
            21 => spaceship3PropTick(a0),
            22 => greenthingPropTick(a0),
            23 => bigshipPropTick(a0),
            24 => chandelierPropTick(a0),
            25 => mallclockPropTick(a0),
            27 => fanChinesePropTick(a0),
            28 => hologramPropTick(a0),
            29 => laserPropTick(a0),
            30 => waterfallPropTick(a0, 0),
            31 => waterfallPropTick(a0, 1),
            32 => liftPropTick(a0),
            37 => tombtorchPropTick(a0),
            43 => mallfan1PropTick(a0),
            44 => mallfan2PropTick(a0),
            48 => planetspawnPropTick(a0),
            49 => planetspawn2PropTick(),
            50 | 51 | 52 | 53 => rotatespawnPropTick(a0),
            54 | 55 | 56 | 57 => blastspawnPropTick(a0),
            58 | 59 | 60 | 61 => ringspawnPropTick(a0),
            62 => splitterdomePropTick(a0),
            73 => reflectionsPropTick(a0),
            _ => {}
        }
    }
}

/// Reset the multiplayer front end (@ 0x241160): the front end effects, gui and zoom boxes are reset, the menu colours and fonts are set, the names of the arcade character
/// and gun sets and of the three player slots are filled in, a memory card file buffer is allocated and the first page is picked by the game mode, then the draw flags
/// of the guns, props and stage are cleared.
pub fn frontmpgReset() {
    frontfxReset();
    guiReset();
    zoomboxReset();
    sw(g::menuitem_fgcol, 0x64646480);
    sw(g::menuitem_bgcol, 0x14141480);
    sw(g::menuitem_normalfont, 1);
    sw(g::menuitem_bigfont, 2);
    textSelect(1);
    let mut language = lw(g::cur_language);
    sw(0x3aff70, 0);
    let mut name_ids = g::arcadechrsetnames;
    let strings = lw((language << 2) + g::languages);
    let mut entry = 0x1fc1960;
    let mut i = 0;
    loop {
        let name = lh(name_ids);
        sw(entry, i);
        name_ids += 2;
        i += 1;
        sw(entry + 4, lw((name << 2) + strings));
        entry += 8;
        if i >= 8 {
            break;
        }
    }
    let mut slot = 0;
    let table = 0x1fc1960;
    let names = table + 4;
    let languages = g::languages;
    loop {
        let index = slot + 8;
        let offset = index << 3;
        let text = (slot << 5) + 0x1fc0c98;
        let strings = lw((language << 2) + languages);
        let entry = offset + table;
        sw(offset + names, text);
        slot += 1;
        let format = lw(strings + 1408);
        sw(entry, index);
        let name = vafmt(&[0x3ae890, format as i64, slot as i64, entry as i64, index as i64]);
        strcpy(text, name);
        if slot >= 3 {
            break;
        }
        language = lw(g::cur_language);
    }
    let gun_table = 0x1fc1908;
    let strings = lw((lw(g::cur_language) << 2) + languages);
    sw(((slot + 8) << 3) + names, 0);
    let mut gun_ids = g::arcadegunsetnames;
    sw(0x1fc1908, 0);
    let mut entry = gun_table + 8;
    let mut i = 1;
    sw(gun_table + 4, lw(strings + 1412));
    loop {
        let name = lh(gun_ids);
        sw(entry, i);
        gun_ids += 2;
        i += 1;
        sw(entry + 4, lw((name << 2) + strings));
        entry += 8;
        if i >= 10 {
            break;
        }
    }
    let flags = 0x1fc11f8;
    'page: {
        sw(0x1fc195c, 0);
        memset(0x1fc1238, 0, 16);
        sw(flags + 4, lw(flags + 4) & -0x10000);
        let buffer = memcardFileAlloc(0xb790, 0);
        let word = lw(flags + 4);
        let mode = lw(0x1fc11f8);
        sw(0x3aff3c, buffer);
        sw(flags + 4, word & -2);
        if (mode as u32) < (7 as u32) {
            match mode {
                0 | 1 => {
                    frontPageSet(0, g::nplayers_page);
                    break 'page;
                }
                2 | 3 | 4 | 6 => {
                    if frontmpgNewGoodies() == 0 {
                        if (lw(0x1fc11fc) & 0x10000) != 0 {
                            if setStatsUpdatePage() != 0 {
                                break 'page;
                            }
                        }
                        setlastgamemodepage();
                        break 'page;
                    }
                    frontmpgSetGoodiesPage();
                    break 'page;
                }
                _ => {
                    sw(flags + 4, lw(flags + 4) | 128);
                }
            }
        }
        sw(0x3aff2c, 3);
        frontPageSet(0, g::arcade_page);
    }
    let front = lw(g::front);
    let word = lw(front + 12);
    sw(front + 40, 0);
    sw(front + 12, word & -4);
    mmfileReset();
    let mut gun2_draw = g::gundrawflag2;
    let mut prop_draw = g::propdrawflag;
    let mut gun_draw = g::gundrawflag;
    let mut stage_prop = g::stageprop;
    let mut gun2 = g::gun2;
    let mut gun = g::gun;
    let mut stage = g::stage;
    let mut left = 3;
    loop {
        sw(stage, 0);
        left -= 1;
        sw(gun, 0);
        stage += 4;
        sw(gun_draw, 0);
        gun += 4;
        sw(gun2, 0);
        gun_draw += 4;
        sw(gun2_draw, 0);
        gun2 += 4;
        sw(stage_prop, 0);
        gun2_draw += 4;
        sw(prop_draw, 0);
        stage_prop += 4;
        prop_draw += 4;
        if left < 0 {
            break;
        }
    }
    let mut left = 3;
    let mut at = 0x1fc1304;
    loop {
        sw(at, -1);
        left -= 1;
        at -= 4;
        if left < 0 {
            break;
        }
    }
    sw(0x1fc122c, 0x1518);
    memset(0x1fc1898, 0, 112);
}

/// Draw the background of a front end page (@ 0x230688): the effects behind the page and, once they have finished, the highlight bar behind the picked item and the page's
/// title and cheat notice.
pub fn pagebackgroundGfx_230688(a0: i32) {
    let frame = Frame::new(160);
    let sp = frame.sp();
    let old_font = textSelect(4);
    dlSetBlend(1);
    let window = g::window;
    dlSetClip(lw(g::window), lw(window + 20), lw(window + 4), lw(window + 24));
    let left = lw(sp);
    if (lw(sp + 8) - left) < 240 {
        sw(sp + 8, left + 240);
    }
    let page = lw(a0);
    let mut effect = if (page ^ g::gametype_page) == 0 { 3 } else { (((page ^ g::nplayers_page) as u32) < (1 as u32)) as i32 };
    if page == g::characterselect_page || page == g::storychr_page || page == g::challengechr_page {
        effect = 2;
    }
    let effect = if (page ^ g::goodies_page) != 0 { effect } else { 4 };
    frontfxGfx(effect);
    if frontfxFinished(0) != 0 {
        dlSetDitherMatrix(2);
        let mut page = lw(a0);
        if page != g::characterselect_page {
            'bar: {
                if page != g::storychr_page {
                    if page != g::signon_page {
                        if page == g::nplayers_page {
                            break 'bar;
                        }
                        let found = frontGetBox(a0, lhu(a0 + 4));
                        if found == 0 {
                            page = lw(a0);
                        } else {
                            let item = lw(found + 16);
                            if item == 0 {
                                page = lw(a0);
                            } else {
                                let flags = lhu(item + 10);
                                if (flags & 0x8000) != 0 {
                                    page = lw(a0);
                                } else {
                                    let top = lhu(found + 2);
                                    let window = g::window;
                                    let window_top = lw(window + 20);
                                    let x = lhu(found);
                                    let window_left = lw(g::window);
                                    let right = window_left + x + lhu(found + 4);
                                    sw(sp, window_left + x);
                                    sw(sp + 4, window_top + top + 1);
                                    sw(sp + 8, right);
                                    sw(sp + 12, window_top + top + (if (flags & 256) != 0 { 15 } else { 11 }) - 1);
                                    dlSetBlend(2);
                                    let middle = lw(sp) + ((lhu(found + 4) as u32) >> 1) as i32;
                                    guiHGradient(lw(g::window), lw(sp + 4), middle, lw(sp + 12), 0x28285040, 0, 128);
                                    let middle = lw(sp) + ((lhu(found + 4) as u32) >> 1) as i32;
                                    guiHGradient(middle, lw(sp + 4), lw(window + 4), lw(sp + 12), 0x28285040, 128, 0);
                                    dlSetBlend(1);
                                    page = lw(a0);
                                }
                            }
                        }
                    }
                }
            }
            if page != g::characterselect_page {
                if page != g::storychr_page {
                    if page != g::nplayers_page {
                        let flags = lbu(g::fontflags);
                        if lb(0x1fc1388) != 0 {
                            let font = lw(g::curfont);
                            let bottom = if (flags & 1) == 0 { lw(lw(g::curfont) + 12) + 12 } else { (lw(font + 12) >> 1) + 12 };
                            guiHGradient(0, 8, 192, bottom, 0xa1480, 128, 0);
                            textOutline(32, 10, 0x1fc1388, 0x40787880, 0x5080);
                        }
                        if lw(0x3aff2c) >= 3 {
                            let mut cheated = 0;
                            'cheat: {
                                let mode = 0x1fc11f8;
                                if lw(mode + 28) == 2 {
                                    let available = cheatsStory();
                                    if (available & lw(g::selectedcheats)) != 0 {
                                        cheated = 1;
                                        break 'cheat;
                                    }
                                }
                                if lw(mode + 28) == 3 {
                                    let available = cheatsArcade();
                                    cheated = ((0 as u32) < ((available & lw(g::selectedcheats)) as u32)) as i32;
                                }
                            }
                            let font_flags = lbu(g::fontflags);
                            if cheated != 0 {
                                let blink = lw(g::tickframenum) & 32;
                                sb(g::fontflags, font_flags | 1);
                                textSelect(1);
                                let centre = lw(g::window + 4) - 111;
                                let text = lw(lw((lw(g::cur_language) << 2) + g::languages) + 1628);
                                let width = textWidth(text);
                                let colour = if blink != 0 { 0x7f7f7f80 } else { 0x40787880 };
                                textOutline(centre - ((((width as u32) >> 31) as i32 + width) >> 1), 26, text, colour, 0x5080);
                                sb(g::fontflags, lbu(g::fontflags) & 254);
                            }
                        }
                    }
                }
            }
        }
        menuitemHighlight(a0);
        textSelect(old_font);
    }
}

/// Tick of a level (@ 0x21e3c0): advance the game frame, rank the teams and characters by score, test the challenge, count down the time limit (with a warning message and
/// sound at the minutes and in the last ten seconds, ending the level when it reaches zero) and tick the game mode.
pub fn gameLevelTick() {
    let frame = Frame::new(240);
    let sp = frame.sp();
    let mut i = 0;
    sw(g::gameframe, lw(g::gameframe) + lw(g::updateframesi));
    memset(sp, 0, 109);
    let mut players = lw(g::lvnumplayers);
    let mut chars = lw(g::numactivechars);
    sw(g::gameranksizeteam, 0);
    let teams;
    if (players + chars) <= 0 {
        teams = lw(g::gameranksizeteam);
    } else {
        loop {
            let team = lw(i * 0x1210 + lw(g::chrinfo) + 24);
            let seen = sp + team;
            if lbu(seen) != 0 {
                i += 1;
            } else {
                sb(seen, 1);
                let score = teamGetScore(team);
                let size = lw(g::gameranksizeteam);
                players = lw(g::lvnumplayers);
                let offset = size << 3;
                sw(offset + 0x1fbba28, score);
                sw(offset + 0x1fbba2c, team);
                sw(g::gameranksizeteam, size + 1);
                chars = lw(g::numactivechars);
                i += 1;
            }
            if i >= players + chars {
                break;
            }
        }
        teams = lw(g::gameranksizeteam);
    }
    bubbleSort(&[0x1fbba28, g::gameranksteam as i64, teams as i64, players as i64]);
    let mut i = 0;
    let mut chars = 0;
    let total = lw(g::lvnumplayers) + lw(g::numactivechars);
    sw(g::gameranksizechr, 0);
    if total > 0 {
        loop {
            let score = chrGetScore(i);
            let size = lw(g::gameranksizechr);
            let count = lw(g::lvnumplayers);
            let offset = size << 3;
            chars = lw(g::numactivechars);
            sw(offset + 0x1fbbd88, score);
            sw(offset + 0x1fbbd8c, i);
            i += 1;
            sw(g::gameranksizechr, size + 1);
            if i >= count + chars {
                break;
            }
        }
    }
    let ranked = lw(g::gameranksizechr);
    bubbleSort(&[0x1fbbd88, g::gamerankschr as i64, ranked as i64, chars as i64]);
    let flags = lw(g::curgameparms + 80);
    let mut timed = flags & 0x4000;
    if (flags & 32) != 0 {
        challengeTick();
        let flags = lw(g::curgameparms + 80);
        if (flags & 0x30000000) != 0 {
            return;
        }
        timed = flags & 0x4000;
    }
    if timed != 0 {
        let frames = lw(g::curgameparms + 88);
        if frames > 0 {
            let mut message = 0;
            'warning: {
                let seconds = frames.wrapping_div(60);
                if seconds < 60 {
                    let warned = lw(g::curgameparms + 84);
                    if (warned & 1) == 0 {
                        let language = lw(g::cur_language) << 2;
                        sw(g::curgameparms + 84, warned | 1);
                        message = lw(lw(language + g::languages) + 1764);
                        soundStart(199, 0);
                        break 'warning;
                    }
                }
                let mut minute = 9;
                let mut next = minute + 1;
                loop {
                    let bit = 2 << (minute & 31);
                    if seconds >= next {
                        minute -= 1;
                    } else {
                        if (lw(g::curgameparms + 84) & bit) == 0 {
                            break;
                        }
                        minute -= 1;
                    }
                    next = minute + 1;
                    if minute < 0 {
                        break 'warning;
                    }
                }
                let language = lw(g::cur_language);
                if next != 10 {
                    message = vafmt(&[0x3ae590, next as i64, seconds as i64, 10, g::languages as i64]);
                } else {
                    message = lw(lw((language << 2) + g::languages) + 1768);
                }
                sw(g::curgameparms + 84, lw(g::curgameparms + 84) | 2 << (minute & 31));
                soundStart(200, 0);
            }
            'show: {
                let count = lw(g::lvnumplayers);
                if message != 0 {
                    let mut player = 0;
                    if count > 0 {
                        loop {
                            hudMessage(player, message);
                            player += 1;
                            if player >= lw(g::lvnumplayers) {
                                break 'show;
                            }
                        }
                    }
                }
            }
            let left = lw(g::curgameparms + 88) - lw(g::updateframesi);
            sw(g::curgameparms + 88, left);
            if left <= 0 {
                gameLevelCompleted();
                soundStart(198, 0);
            }
        }
    }
    let mode = lw(g::curgameparms + 72) - 1;
    if (mode as u32) < (7 as u32) {
        match mode {
            0 => gmBagTagTick(),
            1 => gmCtfTick(),
            2 => gmDeathmatchTick(),
            3 => gmEscortTick(),
            4 => gmKnockOutTick(),
            5 => gmLastStandTick(),
            _ => gmStoryTick(),
        }
    }
}
