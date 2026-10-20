//! Setting the window of a split screen, keeping an enemy in its room, finding the nearest bullet or explosion for the lights, and choosing a last stand NPC.

use crate::*;

/// Set the window of the screen for the layout `mode` and the parts `parts` and, with `draw` set, draw the window and the clip: mode 1 is the whole screen, 2 two halves of the
/// screen side by side (with `draw` it also makes the perspective of the first player and puts its matrix into the display list), 3 and 4 the four-way split (the part number in
/// `parts`: bit 0 the right, bit 1 the lower half); a screen of more than one player marks the window split (+88, +92). The top is moved down by 32 when the front end flag 256
/// is set (@ 0x230ca8).
pub fn SetWindow(parts: i32, mode: i32, draw: i32) {
    let frame = Frame::new(32);
    let sp = frame.sp();
    let top = if (lw(0x1fc11fc) & 256) == 0 { 0 } else { 32 };
    let mut players = 0;
    'done: {
        if mode == 2 {
            if draw != 0 {
                let left = if parts == 0 { -240 } else { 80 };
                sw(sp + 4, -28);
                sw(sp + 8, left + 800);
                sw(sp + 12, 252);
                sw(sp, left);
                windowSet(left, -28, left + 800, 252);
                windowGfx();
                let projection = lw(lw(g::currentplayer) + 1760);
                matrixPerspective(projection, lwc1(0x3ac98c), lwc1(0x3ac990), lwc1(0x3ac994), 1e+03_f32);
                let player = lw(g::currentplayer);
                sceVu0MulMatrix(lw(player + 1768), lw(player + 1760), lw(player + 1764));
                let packet = memdbAlloc(80);
                sb(lw(g::gfxtag) + 3, 48);
                sw(packet, 0x6c048000);
                let tag = lw(g::gfxtag);
                let matrix = lw(lw(g::currentplayer) + 1760);
                sw(tag + 4, packet);
                let first = lw(matrix);
                let mut at = packet + 4;
                sh(tag, 5);
                sw(at, first);
                at += 4;
                sw(g::gfxtag, tag + 16);
                for k in 1..16 {
                    sw(at, lw(matrix + k * 4));
                    at += 4;
                }
                sw(at, 0x1400079d);
                at += 4;
                sw(at, 0x11000000);
                sw(at + 4, 0);
            }
            let left = if (parts & 1) == 0 { 0 } else { 322 };
            let right = left + 320;
            sw(sp + 12, 223);
            sw(sp, left);
            sw(sp + 4, top);
            sw(sp + 8, right);
            windowSet(left, top, right, 223);
            players = lw(0x3aff20);
        } else if mode >= 3 {
            if mode < 5 {
                let side = parts & 1;
                let half = (224 - top) >> 1;
                sw(sp, if side == 0 { 0 } else { 322 });
                let from = if (parts & 2) == 0 { top } else { top + 1 + half };
                let left = lw(sp);
                let right = if side != 0 { left + 319 } else { left + 317 };
                let bottom = from + half - ((parts >> 1 ^ 1) & 1);
                sw(sp + 4, from);
                sw(sp + 8, right);
                sw(sp + 12, bottom);
                windowSet(left, from, right, bottom);
                players = lw(0x3aff20);
                if draw == 0 {
                    break 'done;
                }
                windowGfx();
            }
            players = lw(0x3aff20);
        } else {
            players = lw(0x3aff20);
            if mode == 1 {
                sw(sp + 8, 639);
                sw(sp, 0);
                sw(sp + 4, top);
                sw(sp + 12, 223);
                windowSet(0, top, 639, 223);
                players = lw(0x3aff20);
                if draw != 0 {
                    windowGfx();
                    players = lw(0x3aff20);
                }
            }
        }
    }
    let window = g::window;
    let split = ((players < 2) as i32) ^ 1;
    sw(window + 92, split);
    sw(window + 88, split);
    if draw != 0 {
        let left = lw(g::window);
        if mode == 2 {
            dlSetClip(lw(sp), top, lw(sp + 8), 223);
        } else {
            dlSetClip(left, lw(window + 20), lw(window + 4), lw(window + 24));
        }
    }
}

/// Put the enemy `enemy` in its room: an enemy that is not in the flying or lifted states (flags 0xf000000 of its prop) and has a pad (+680 of its record) takes the pad's room and
/// moves its kept place (+2808) through the portals from the pad to where the enemy is, room by room (`bgPortalCrossed`); else its rooms are worked out (`propCalcRooms`, or with
/// `find` set `moveFindRoom`) and the kept place followed through the portals the same way, and one with the flags put on the floor (`moveFindFloor`) and in its room or on its pad;
/// then the place of the target point of its body (+3008: the bone's place, or above it by a constant, higher for the kind 2) is kept (@ 0x2c7f20).
pub fn enemySetRoom(enemy: i32, find: i32) {
    let frame = Frame::new(176);
    let sp = frame.sp();
    let mut flags = lw(enemy + 16);
    let record = lw(enemy + 352);
    'rooms: {
        if (flags & 0xf000000) == 0 {
            let pad = lw(record + 680);
            if pad != -1 {
                let pads = lw(g::levelpads);
                let raise = lwc1(0x3ad9b8);
                let mut last = 0;
                let mut room_now = 0;
                let room = lh(pad * 28 + pads + 10);
                swc1(record + 2808, lwc1(enemy + 48));
                sw(enemy + 12, room);
                let mut room = room;
                let pad_at = lw(record + 680) * 28;
                swc1(record + 2812, lwc1(enemy + 52) + raise);
                let at = pad_at + pads;
                swc1(record + 2816, lwc1(enemy + 56));
                let px = lwc1(at + 12);
                swc1(sp, px);
                let py = lwc1(at + 16) + raise;
                swc1(sp + 4, py);
                let pz = lwc1(at + 20);
                swc1(sp + 8, pz);
                swc1(sp + 16, lwc1(record + 2808) - px);
                swc1(sp + 20, lwc1(record + 2812) - py);
                swc1(sp + 24, lwc1(record + 2816) - pz);
                if room != 0 {
                    loop {
                        room_now = bgPortalCrossed(sp, sp + 16, room, last, 0);
                        last = room;
                        room = room_now;
                        if room_now == last {
                            break;
                        }
                    }
                    flags = lw(enemy + 16);
                }
                sw(enemy + 12, room_now);
                if room_now == 0 {
                    sw(enemy + 12, lh(lw(record + 680) * 28 + lw(g::levelpads) + 10));
                }
                break 'rooms;
            }
        }
        propCalcRooms(enemy);
        let player = lw(g::currentplayer);
        if find != 0 {
            let room = moveFindRoom(enemy + 48, lwc1(0x3ad9bc), lwc1(player + 132));
            let x = lwc1(enemy + 48);
            sw(enemy + 12, room);
            swc1(record + 2808, x);
            swc1(record + 2812, lwc1(enemy + 52));
            swc1(record + 2816, lwc1(enemy + 56));
        }
        let mut room_now = 0;
        'follow: {
            let kept_x = lwc1(record + 2808);
            let mut last = 0;
            let y = lwc1(enemy + 52);
            swc1(sp, kept_x);
            let kept_y = lwc1(record + 2812);
            let x = lwc1(enemy + 48);
            let raised = y + lwc1(0x3ad9c0);
            swc1(sp + 4, kept_y);
            let dx = x - kept_x;
            let kept_z = lwc1(record + 2816);
            let mut room = lw(enemy + 12);
            let dz = lwc1(enemy + 56) - kept_z;
            swc1(sp + 8, kept_z);
            swc1(sp + 20, raised - kept_y);
            swc1(sp + 16, dx);
            swc1(sp + 24, dz);
            swc1(record + 2808, kept_x + dx);
            swc1(record + 2812, lwc1(sp + 4) + lwc1(sp + 20));
            swc1(record + 2816, lwc1(sp + 8) + lwc1(sp + 24));
            if room != 0 {
                loop {
                    room_now = bgPortalCrossed(sp, sp + 16, room, last, 0);
                    last = room;
                    room = room_now;
                    if room_now == last {
                        break 'follow;
                    }
                }
            }
        }
        sw(enemy + 12, room_now);
        flags = lw(enemy + 16);
        if room_now == 0 && (flags & 0xf000000) != 0 {
            let y = lwc1(enemy + 52) + lwc1(0x3ad9c4);
            let z = lwc1(enemy + 56);
            swc1(sp + 160, lwc1(enemy + 48));
            swc1(sp + 168, z);
            swc1(sp + 164, y);
            let height = lwc1(0x3ad9c8);
            let floor = moveFindFloor(sp + 160, 0, 0, height, lwc1(lw(g::currentplayer) + 132));
            swc1(enemy + 52, floor);
            let player = lw(g::currentplayer);
            swc1(sp + 164, floor);
            let room = moveFindRoom(enemy + 48, height, lwc1(player + 132));
            sw(enemy + 12, room);
            if room == 0 {
                let at = lw(record + 680) * 28 + lw(g::levelpads);
                swc1(enemy + 48, lwc1(at + 12));
                swc1(enemy + 52, lwc1(at + 16));
                swc1(enemy + 56, lwc1(at + 20));
                sw(enemy + 12, lh(at + 10));
            }
            flags = lw(enemy + 16);
            swc1(record + 2808, lwc1(enemy + 48));
            swc1(record + 2812, lwc1(enemy + 52));
            swc1(record + 2816, lwc1(enemy + 56));
        }
    }
    let lift;
    let base_x;
    'body: {
        if (flags & 128) == 0 && (flags & 0xf000000) != 0 {
            let turn = lwc1(enemy + 76);
            matrixTransRotY(sp + 32, lwc1(enemy + 48), lwc1(enemy + 52), lwc1(enemy + 56), turn * lwc1(0x3ad9d0) / 1.8e+02_f32);
            let model = lw(enemy + 32);
            let bone = lw(model + 4) + (lw(lw(model + 92) + 20) << 6);
            sceVu0MulMatrix(sp + 96, sp + 32, bone);
            let drop = lwc1(0x3ad9d4);
            swc1(record + 3008, lwc1(sp + 144));
            let kind = lw(record + 8);
            swc1(record + 3012, lwc1(sp + 148) - drop);
            swc1(record + 3016, lwc1(sp + 152));
            if kind != 2 {
                return;
            }
            base_x = lwc1(enemy + 48);
            lift = 1.5_f32;
            break 'body;
        }
        lift = lwc1(0x3ad9cc);
        base_x = lwc1(enemy + 48);
    }
    swc1(record + 3008, base_x);
    swc1(record + 3012, lwc1(enemy + 52) + lift);
    swc1(record + 3016, lwc1(enemy + 56));
}

/// Find the nearest bullet or explosion that lights the object instance `instance` at the place `at`, and put its kind's colour and place into the record at 0x1fc5e00 (the colour
/// at +12 to +20, the place at +24 to +32) and its strength (1 close up, less the farther) into `*strength`; the answer is that record, or what `bulletGetClosest` found. The
/// bullet in the hand of the player the instance belongs to (a prop number 201 to 204) is used first; the explosion is looked for only every fourth tick (by the instance's
/// number), and its colour, place and radius are kept at +548 to +560 of the instance's record between (@ 0x25bc88).
pub fn bgBulletGetClosest(instance: i32, at: i32, strength: i32) -> i32 {
    let frame = Frame::new(64);
    let sp = frame.sp();
    let record = lw(instance + 244);
    let ticks = lw(g::updateframesi);
    let owner = lw(record + 12);
    let mut explosion_strength = 0.0_f32;
    let mut frame_number = lw(g::gameframe);
    let number = ((instance - lw(g::obinstances)).wrapping_mul(-0x3e7063e7)) >> 3;
    if ticks > 0 {
        if ticks == 0 {
            trap("break");
        }
        frame_number = frame_number.wrapping_div(ticks);
    }
    let mut search = true;
    'closest: {
        if ((owner - 201) as u32) < 4 {
            let player = lw(g::players) + (owner * 1820 - 0x594fc);
            let right = lw(player + 420);
            let mut right_visible = 0;
            if right != 0 {
                right_visible = lhu(lw(right + 32) + 292) & 1;
            }
            let left = lw(player + 612);
            let left_visible = if left == 0 { 0 } else { lhu(lw(left + 32) + 292) & 1 };
            let mut hand = player + 408;
            if right_visible == 0 {
                hand = if left_visible == 0 { 0 } else { player + 600 };
            }
            if hand != 0 {
                let bullet = gunGetBulletType(hand);
                if bullet != 0 {
                    bulletGetBulletColour(bullet, sp + 32);
                    let green = lwc1(sp + 36);
                    let blue = lwc1(sp + 40);
                    swc1(0x1fc5e0c, lwc1(sp + 32));
                    swc1(0x1fc5e10, green);
                    swc1(0x1fc5e14, blue);
                    let x = lwc1(hand + 16);
                    swc1(0x1fc5e18, x);
                    let y = lwc1(hand + 20);
                    swc1(0x1fc5e1c, y);
                    let z = lwc1(hand + 24);
                    swc1(0x1fc5e20, z);
                    swc1(0x1fc5e18, x + lwc1(player + 228) * 0.5_f32);
                    swc1(0x1fc5e1c, y + lwc1(player + 232) * 0.5_f32);
                    swc1(0x1fc5e20, z + lwc1(player + 236) * 0.5_f32);
                    swc1(strength, 1.0_f32);
                    return 0x1fc5e00;
                }
            }
        }
        break 'closest;
    }
    let found = bulletGetClosest(at, sp + 48);
    let distance = lwc1(sp + 48);
    if found != 0 {
        if distance <= 3.0_f32 {
            swc1(strength, 1.0_f32);
        } else {
            swc1(strength, 1.0_f32 / (distance - 2.0_f32));
        }
    }
    if (number & 3) != frame_number & 3 {
        let packed = lw(record + 548);
        search = false;
        if packed != -1 {
            let dx = lwc1(at) - lwc1(record + 552);
            let dy = lwc1(at + 4) - lwc1(record + 556);
            let dz = lwc1(at + 8) - lwc1(record + 560);
            swc1(sp + 16, (((packed as u32) >> 24) & 255) as f32 * lwc1(0x3acb2c));
            swc1(sp + 20, (((packed as u32) >> 16) & 255) as f32 * lwc1(0x3acb30));
            let squared = dy * dy;
            swc1(sp + 24, (((packed as u32) >> 8) & 255) as f32 * lwc1(0x3acb34));
            swc1(sp, lwc1(record + 552));
            let radius = (dx * dx + squared + dz * dz).abs().sqrt();
            swc1(sp + 4, lwc1(record + 556));
            swc1(sp + 8, lwc1(record + 560));
            let half = radius * 0.5_f32;
            explosion_strength = 1.0_f32;
            swc1(sp + 52, half);
            if !(half <= 2.0_f32) {
                explosion_strength = explosion_strength / (half - explosion_strength);
            }
        }
    }
    if search {
        let hit = particleGetClosestExplosion(at, sp, sp + 52, sp + 16);
        let red = lwc1(sp + 16);
        if hit == 0 {
            sw(record + 548, -1);
        } else {
            explosion_strength = 1.0_f32;
            let r = fptoui(red * 255.0_f32);
            let g = fptoui(lwc1(sp + 20) * 255.0_f32);
            let b = fptoui(lwc1(sp + 24) * 255.0_f32);
            swc1(record + 552, lwc1(sp));
            let y = lwc1(sp + 4);
            sw(record + 548, r << 24 | g << 16 | b << 8);
            swc1(record + 556, y);
            swc1(record + 560, lwc1(sp + 8));
            let half = lwc1(sp + 52) * 0.5_f32;
            swc1(sp + 52, half);
            if !(half <= 2.0_f32) {
                explosion_strength = explosion_strength / (half - explosion_strength);
            }
        }
    }
    if lw(record + 548) == -1 {
        return found;
    }
    let red = lwc1(sp + 16);
    if found != 0 && !(lwc1(strength) < explosion_strength) {
        return found;
    }
    let green = lwc1(sp + 20);
    let blue = lwc1(sp + 24);
    let x = lwc1(sp);
    let y = lwc1(sp + 4);
    let z = lwc1(sp + 8);
    swc1(strength, explosion_strength);
    swc1(0x1fc5e0c, red);
    swc1(0x1fc5e10, green);
    swc1(0x1fc5e14, blue);
    swc1(0x1fc5e18, x);
    swc1(0x1fc5e1c, y);
    swc1(0x1fc5e20, z);
    0x1fc5e00
}

/// Choose at random a record (24 bytes, the wave at +20 equal to `wave`) of the NPCs of the last stand in the level's set-up (the records from +40, the count at +44) whose pad
/// no active character is on, going on to the next such record if its pad is taken; 0 when there is none or all are taken (@ 0x227f00).
pub fn lvGetRandomLastStandNPC(wave: i32) -> i32 {
    let setup = lvGetLevelSetup(lw(g::lvlevelnum));
    if setup == 0 {
        return 0;
    }
    let count = lw(setup + 44);
    if count == 0 {
        return 0;
    }
    let mut matching = 0;
    if count > 0 {
        let records = lw(setup + 40);
        let mut i = 0;
        loop {
            let offset = i * 24;
            i += 1;
            if (lw(offset + records + 20) ^ wave) == 0 {
                matching += 1;
            }
            if i >= count {
                break;
            }
        }
    }
    if matching == 0 {
        return 0;
    }
    let mut found = 0;
    let list = 0x1fbcc78;
    if count > 0 {
        let mut left = count;
        let mut offset = 0;
        let mut put = list;
        loop {
            let record = offset + lw(setup + 40);
            left -= 1;
            if lw(record + 20) == wave {
                sw(put, record);
                found += 1;
                put += 4;
            }
            offset += 24;
            if left == 0 {
                break;
            }
        }
    }
    let random = newrnd();
    if found == 0 {
        trap("break");
    }
    let start = (random as i32 as u32 % found as u32) as i32;
    let mut at = start;
    loop {
        let record = lw((at << 2) + list);
        let pad = padNumFromExtref(lw(record));
        let characters = lw(g::lvnumplayers) + lw(g::numactivechars);
        let mut taken = false;
        if characters > 0 {
            let base = lw(g::chrinfo);
            let mut j = 0;
            loop {
                let character = j * 0x1210 + base;
                if lw(character + 2716) != 0 && lw(character + 680) == pad {
                    taken = true;
                    break;
                }
                j += 1;
                if j >= characters {
                    break;
                }
            }
        }
        if !taken {
            return record;
        }
        let following = at + 1;
        at = if (following ^ found) == 0 { 0 } else { following };
        if at == start {
            return 0;
        }
    }
}
