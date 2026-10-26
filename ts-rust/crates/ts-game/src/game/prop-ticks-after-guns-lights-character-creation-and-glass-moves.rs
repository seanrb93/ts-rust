//! Functions: the tick of the props after the rest, the statistics page drawing, the gun tick, the light of an object instance, the sound select page, making a character, the
//! end of level sound and moving things through glass.

use crate::*;

/// Light the model of a prop for the current player if it is visible to it, otherwise clear its light fields (+24 and +28) (@ the body of `propTickAfter`).
fn update_prop_light(prop: i32) {
    let model = lw(prop + 32);
    if lw(lw(model) + 8) != 0 {
        let flags = lw(lw(model + 244) + 16);
        let mut lit = 0;
        if (flags & 0xf000000) != 0 {
            let player = lw(g::currentplayer);
            if (flags & 256) == 0 {
                if lw(player + 1764) != 0 {
                    obInstLightUpdate(model);
                    lit = 1;
                }
            }
        }
        if lit == 0 {
            let model = lw(prop + 32);
            sw(model + 28, 0);
            sw(model + 24, 0);
        }
    }
}

/// Run the props' tick after the rest: first the props that are not attached are run through their own tick function (at +420 with the argument at +428), lit and updated, and
/// the characters get their after tick; then the attached ones are updated (and the heads ticked), lit and updated, and unattached when flagged (@ 0x26f9e8).
pub fn propTickAfter() {
    let mut count = lw(g::num_props);
    let mut i = 0;
    if count > 0 {
        let mut offset = 0;
        loop {
            let prop = lw(g::levelprops) + offset;
            if (lw(prop + 16) & -0x7fffff00) != 0 {
                i += 1;
            } else if lw(prop + 188) != 0 {
                i += 1;
            } else {
                let tick = lw(prop + 420);
                if tick != 0 {
                    call_indirect(tick, [prop, lw(prop + 428), 0, 0, 0, 0, 0, 0], [0.0; 8]);
                }
                update_prop_light(prop);
                propUpdate(prop);
                count = lw(g::num_props);
                if lw(prop + 8) == 8 {
                    chrPropTickAfter(prop);
                    count = lw(g::num_props);
                }
                i += 1;
            }
            offset += 592;
            if i >= count {
                break;
            }
        }
    }
    let mut i = 0;
    if count > 0 {
        let mut offset = 0;
        loop {
            let prop = lw(g::levelprops) + offset;
            if (lw(prop + 16) & -0x7fffff00) != 0 {
                i += 1;
            } else if lw(prop + 188) == 0 {
                i += 1;
            } else {
                propAttachUpdate(prop);
                if lw(prop + 8) == 2048 {
                    headPropTick(prop);
                }
                let tick = lw(prop + 420);
                if tick != 0 {
                    call_indirect(tick, [prop, lw(prop + 428), 0, 0, 0, 0, 0, 0], [0.0; 8]);
                }
                update_prop_light(prop);
                propUpdate(prop);
                count = lw(g::num_props);
                if (lw(prop + 28) & 4) != 0 {
                    propUnattach(prop);
                    count = lw(g::num_props);
                }
                i += 1;
            }
            offset += 592;
            if i >= count {
                break;
            }
        }
    }
}

/// Draw the statistics page: the heading, then the character the player has used most (the highest count among the 80), and the lines of information of the chosen kind
/// (the sum of the three kinds for 0) (@ 0x23c9c8).
pub fn statistics_pageDraw(a0: i32) {
    let frame = Frame::new(592);
    let sp = frame.sp();
    let mut stats = 0;
    let mut best = -1;
    pagebackgroundGfx_230688(a0);
    let kind = lw(g::gv_statstype);
    let colour = lw(g::menuitem_fgcol);
    let player = lw(g::gv_statsplayer);
    sw(sp + 408, lw(g::gv_statscat));
    'b1: {
        if kind == 1 {
            stats = statsGetPlayer(player) + 1616;
        } else if kind >= 2 {
            if kind == 2 {
                stats = statsGetPlayer(player) + 2012;
            } else {
                if kind != 3 {
                    break 'b1;
                }
                stats = statsGetPlayer(player) + 2408;
            }
        } else {
            if kind != 0 {
                break 'b1;
            }
            let first = statsGetPlayer(player);
            let second = statsGetPlayer(player);
            let third = statsGetPlayer(player);
            StatsGetOverallLifestats(sp, first + 1616, second + 2012, third + 2408);
            stats = sp;
        }
    }
    textSelect(1);
    let mut best_value = 1;
    let language = lw(g::cur_language);
    sw(sp + 412, sp + 400);
    sw(sp + 404, 49);
    sw(sp + 416, sp + 404);
    sw(sp + 400, 65);
    let mut i = 0;
    textPrint(lw(sp + 412), lw(sp + 416), lw(lw((language << 2) + g::languages) + 1064), colour);
    loop {
        let offset = i << 2;
        let record = statsGetPlayer(player);
        if (lw(record + offset) as u32) < (best_value as u32) {
            i += 1;
        } else {
            best = i;
            let record = statsGetPlayer(player);
            best_value = lw(record + offset);
            i += 1;
        }
        if i >= 80 {
            break;
        }
    }
    let name = if best >= 0 {
        lw((lh(best * 44 + g::chrs) << 2) + lw((lw(g::cur_language) << 2) + g::languages))
    } else {
        lw(lw((lw(g::cur_language) << 2) + g::languages) + 1068)
    };
    let width = textWidth(name);
    sw(sp + 400, 570 - width);
    textPrint(lw(sp + 412), lw(sp + 416), name, colour);
    statistics_pageDrawInfo(65, 570, colour, stats, kind, lw(sp + 408), player);
}

/// The tick of the guns of the current player: the buttons change the gun combo (not while editing, dead or in the modes 6 and 7), the walk bob is worked out from the
/// player's speed, the two hands' guns update and are made when they changed, and the timer at +2912 runs (@ 0x2946c0).
pub fn gunTick() {
    let mut player = lw(g::currentplayer);
    let record = lw(lw(player + 384) + 352);
    let speed;
    'b1: {
        'b2: {
            if lw(g::padlinkedit) == 0 {
                if lw(g::propedit) == 0 {
                    let mode = lw(player + 296);
                    if mode == 4 {
                        break 'b2;
                    }
                    if mode == 6 {
                        break 'b2;
                    }
                    if mode == 7 {
                        speed = lw(player + 340);
                        break 'b1;
                    }
                    let pressed = joyNewBut(lw(player));
                    let now = lw(g::currentplayer);
                    if (pressed & lw(now + 60)) != 0 {
                        StatsAdd(60, lw(record), 1.0_f32);
                        let next = getnextcombo(record);
                        sw(record + 260, next);
                    }
                    let now = lw(g::currentplayer);
                    let pressed = joyNewBut(lw(now));
                    player = lw(g::currentplayer);
                    if (pressed & lw(player + 64)) == 0 {
                        speed = lw(player + 340);
                        break 'b1;
                    }
                    StatsAdd(60, lw(record), 1.0_f32);
                    let previous = getprevcombo(record);
                    sw(record + 260, previous);
                    player = lw(g::currentplayer);
                    break 'b2;
                }
            }
            sw(record + 260, 0);
        }
        speed = lw(player + 340);
    }
    let frames = lwc1(g::updateframesf);
    if speed > 0 {
        swc1(player + 392, lwc1(player + 392) + (lwc1(player + 380) * lwc1(0x3ad450)).pdiv(speed as f32) * frames);
    }
    let scale = lwc1(0x3ad454);
    let phase = lwc1(player + 392) * 3.6e+02_f32;
    let half = 1.8e+02_f32;
    let angle = (phase * scale).pdiv(half);
    let amplitude = (lwc1(player + 380) * lwc1(0x3ad458)).pdiv(lw(player + 340) as f32);
    let first = sinf(angle);
    let now = lw(g::currentplayer);
    swc1(player + 396, amplitude * first);
    let second_angle = ((phase + phase + 45.0_f32) * scale).pdiv(half);
    let second = sinf(second_angle);
    let now_again = lw(g::currentplayer);
    let right_gun = lw(record + 376);
    sw(g::righthandprevammo, 0);
    swc1(now + 400, amplitude * second);
    sw(now_again + 404, 0);
    if right_gun != -1 {
        sw(g::righthandprevammo, lw(record + (lw(right_gun * 400 + (g::guns + 16)) << 2) + 380));
    }
    gunUpdate(now_again + 600, 0);
    gunUpdate(lw(g::currentplayer) + 408, 1);
    let mut player_now = lw(g::currentplayer);
    let left_state;
    if lw(player_now + 600) != 4 {
        left_state = lw(player_now + 408);
    } else {
        gunCreatePlayerGun(player_now + 600, record + 376, 0);
        player_now = lw(g::currentplayer);
        left_state = lw(player_now + 408);
    }
    let mut frames = lwc1(g::updateframesf);
    if left_state == 4 {
        gunCreatePlayerGun(player_now + 408, record + 268, 1);
        frames = lwc1(g::updateframesf);
    }
    let timer = lwc1(record + 2912) + (frames).pdiv(6e+01_f32);
    let over = lwc1(0x3ad45c) < timer;
    swc1(record + 2912, timer);
    if over {
        sw(record + 2940, 0);
        sw(record + 2932, 0);
        sw(record + 2936, 0);
    }
}

/// Light the object instance `a0` from the light record `a1`: the colours of the vertex lists of both sides of each part are worked out into the buffer that is not being
/// drawn (the instance's flip at +288) (@ 0x2a7ab0).
pub fn obInstLight(a0: i32, a1: i32) {
    let object = lw(a0);
    let bones = object - lw(object) * 80;
    let mut count = lw(object);
    if lw(a0 + 252) == 0 {
        obPrecalcDbufGfxData(a0);
        count = lw(lw(a0));
    }
    sw(a0 + 288, 1 - lw(a0 + 288));
    let mut i = 0;
    if count > 0 {
        loop {
            let part = i * 80 + bones;
            let first = lw(part + 8);
            let next = i + 1;
            if first != 0 {
                let mut entry = first;
                if lw(entry + 20) >= 0 {
                    if lw(entry + 44) >= 0 {
                        loop {
                            entry += 24;
                            if lw(entry + 44) < 0 {
                                break;
                            }
                        }
                    }
                }
                let flip = lw(a0 + 288);
                let base = lw(a0 + 248);
                let slot = i << 2;
                let table = (a0 + 268) + (flip << 3);
                let colour_at = lw(a1 + 24);
                let size = lw(entry + 16);
                let target = i * 48 + base;
                let offset = lw(entry + 12);
                let colours = lw(slot + lw(table));
                let sum = offset + size;
                let step = lw(a1 + 12);
                sw(target + 12, colours);
                vtxlistcolour(&[lw(part + 32) as i64, colours as i64, sum as i64, step as i64, colour_at as i64, target as i64]);
            }
            let second = lw(part + 12);
            if second != 0 {
                let mut entry = second;
                if lw(entry + 20) >= 0 {
                    if lw(entry + 44) >= 0 {
                        loop {
                            entry += 24;
                            if lw(entry + 44) < 0 {
                                break;
                            }
                        }
                    }
                }
                let flip = lw(a0 + 288);
                let base = lw(a0 + 248);
                let slot = i << 2;
                let table = (a0 + 272) + (flip << 3);
                let colour_at = lw(a1 + 24);
                let size = lw(entry + 16);
                let target = i * 48 + base;
                let offset = lw(entry + 12);
                let colours = lw(slot + lw(table));
                let sum = offset + size;
                let step = lw(a1 + 12);
                sw(target + 36, colours);
                vtxlistcolour(&[lw(part + 56) as i64, colours as i64, sum as i64, step as i64, colour_at as i64, target as i64]);
            }
            i = next;
            if i >= lw(lw(a0)) {
                break;
            }
        }
    }
}

/// The tick of the sound select page: pages of ten sounds are stepped by the pad, the button 2048 or 64 plays or chooses the sound at the cursor, and the ten menu items are
/// made with the sound names (@ 0x206808).
pub fn soundselect_pageTick(a0: i32) {
    if lw(g::first_sound) == i32::MIN {
        let current = lw(g::snd);
        sh(a0 + 4, current.wrapping_rem(10) + 1);
        sw(g::first_sound, current - current.wrapping_rem(10));
    }
    let pressed = joyNewBut(0);
    let first = lw(g::first_sound);
    if (pressed & 2048) != 0 {
        let sound = lhu(a0 + 4) + (first - 1);
        if sound < 1215 {
            soundStart(sound, 0);
        }
    }
    // 0 nothing, 1 the previous page, 2 the next page
    let mut action = 0;
    if (joyNewBut(0) & 0x8000) == 0 {
        let other = joyNewBut(0);
        let front = lw(g::front);
        if (other & 0x2000) == 0 {
            let flags = lw(front + 12);
            let page = lw(g::first_sound);
            if (flags & 8) == 0 {
                if (flags & 4) != 0 {
                    if lhu(a0 + 4) == 1 {
                        sh(a0 + 4, 10);
                        sw(front + 12, flags & -5);
                        action = 1;
                    }
                }
            } else if (page + 10) >= 1215 {
                if (page + lhu(a0 + 4)) >= 1215 {
                    sw(g::first_sound, 0);
                    sw(front + 12, flags & -9);
                    sh(a0 + 4, 1);
                }
            } else if lhu(a0 + 4) == 10 {
                sh(a0 + 4, 1);
                sw(front + 12, flags & -9);
                action = 2;
            }
        } else {
            action = 2;
        }
    } else {
        action = 1;
    }
    if action == 2 {
        let page = lw(g::first_sound) + 10;
        sw(g::first_sound, page);
        if page < 1215 {
            if (page + lhu(a0 + 4) - 1) >= 1215 {
                sh(a0 + 4, 6);
            }
        } else {
            sw(g::first_sound, 0);
        }
    } else if action == 1 {
        let page = lw(g::first_sound) - 10;
        sw(g::first_sound, page);
        if page < 0 {
            sw(g::first_sound, 1210);
            sh(a0 + 4, 6);
        } else if (page + lhu(a0 + 4) - 1) >= 1215 {
            sh(a0 + 4, 6);
        }
    }
    let pressed = joyNewBut(0);
    let page = lw(g::first_sound);
    if (pressed & 64) != 0 {
        let sound = lhu(a0 + 4) + (page - 1);
        if sound < 1215 {
            setcursound(sound);
            let front = lw(g::front);
            sw(front + 20, lw(front + 20) - 1);
            return;
        }
    }
    menutick_2062c0(a0, g::soundselect_menu);
    let mut i = 0;
    let mut page = lw(g::first_sound);
    loop {
        let sound = page + i;
        let next;
        if sound >= 1215 {
            next = i + 1;
            sw(((i + 11) << 5) + g::soundselect_menuitems, 0);
            sw((next << 5) + g::soundselect_menuitems, 0);
        } else {
            let text = (i << 2) + 0x1fb2068;
            sw(((i + 11) << 5) + g::soundselect_menuitems, lw(sound * 36 + g::sounds));
            sprintf(&[text as i64, 0x3ae358_i64, sound as i64]);
            next = i + 1;
            sw((next << 5) + g::soundselect_menuitems, text);
        }
        i = next;
        page = lw(g::first_sound);
        if i >= 10 {
            break;
        }
    }
    let made = std_menumake(lw(a0 + 8), g::soundselect_menu);
    sw(a0 + 12, made);
}

/// Make a character of the type `a0`/`a1` (`a2` picks the second model) in the slot `a3`: the prop is made at the pad `a4` or at the given position, the record is set up (the
/// team and colour by the game kind, the role flag at +8, the name), cleared, respawned and (with `a5`) its bounding box data is made (@ 0x287cc0).
pub fn chrNew(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32, f12: f32, f13: f32, f14: f32, f15: f32) -> i32 {
    sw(g::chrnew_lastchrnum, a0);
    let model = if a2 == 0 { lw(a0 * 44 + (g::chrs + 8)) } else { lw(a0 * 44 + (g::chrs + 12)) };
    let prop;
    if a4 >= 0 {
        let pad = a4 * 28 + lw(g::levelpads);
        let angle = lwc1(pad + 24);
        let x = lwc1(pad + 12);
        let y = lwc1(pad + 16);
        let z = lwc1(pad + 20);
        prop = propNew(model, x, y, z, angle);
    } else {
        prop = propNew(model, f12, f13, f14, f15);
    }
    let instance = lw(prop + 32);
    let info = lw(g::chrinfo);
    let model_flags = lhu(instance + 292);
    sw(prop + 16, 289);
    let record = info + a3 * 0x1210;
    let angle = lwc1(prop + 76);
    sw(record, a3);
    sw(record + 8, 0);
    sh(instance + 292, model_flags | 0xc000);
    let kind = lw(g::curgameparms + 72);
    swc1(prop + 80, angle);
    sw(prop + 352, record);
    sw(record + 3020, prop);
    sh(record + 4, a0);
    sh(record + 6, a1);
    let flags = lw(g::curgameparms + 80);
    if kind == 7 || (flags & 256) != 0 || (kind == 6 && lw(g::lvlevelnum) == 8) {
        if ((a1 - 40) as u32) < 5 || a1 == 60 || a1 == 61 || a1 == 64 {
            if !(lw(record) < lw(g::lvnumplayers)) {
                sw(record + 8, 1);
            }
        }
    }
    if !(a1 < 79) {
        sw(record + 8, 3);
    }
    let mut applies = true;
    if lw(g::curgameparms + 72) != 7 {
        if (lw(g::curgameparms + 80) & 1024) == 0 {
            applies = false;
        }
    }
    if applies {
        if ((a1 - 69) as u32) < 4 {
            sw(record + 8, 2);
        }
    }
    if (lw(g::curgameparms + 80) & 1) == 0 {
        sw(record + 24, a3 + 1);
        sw(record + 20, lw((a3.wrapping_rem(14) << 2) + g::charcolours));
    } else {
        let team_kind = lw(g::curgameparms + 72);
        let team = if team_kind == 7 && lw(record) < lw(g::lvnumplayers) { 1 } else { 5 };
        sw(record + 24, team);
        sw(record + 20, lw((lw(record + 24) << 3) + (g::storydata + 68)));
    }
    sw(record + 16, lw((lh(a0 * 44 + (g::chrs + 2)) << 2) + lw((lw(g::cur_language) << 2) + g::languages)));
    memset(record + 32, 0, 216);
    sw(record + 248, 0);
    sw(record + 440, 0);
    sw(record + 332, 0);
    sw(record + 484, 0);
    chrRespawn(record);
    swc1(record + 2840, lwc1(a0 * 44 + (g::chrs + 40)));
    if a5 != 0 {
        InitialiseChrBoundingBoxData(record);
    }
    prop
}

/// Play the sound for the end of a level of the game kinds 4, 6, 7 (or with the flag 32): the level (2 to 27) chooses a pair of sounds and the flag 0x10000000 picks the
/// second; none gives the plain sound 222, otherwise the music is faded and the sound plays after 5 (@ 0x21d098).
pub fn gameEndLevelSound() {
    let parms = g::curgameparms;
    let kind = lw(parms + 72);
    let mut sound = -1;
    'b1: {
        if kind != 7 {
            if (lw(parms + 80) & 32) == 0 {
                if kind != 4 {
                    if kind != 6 {
                        break 'b1;
                    }
                }
            }
        }
        let level = lw(g::lvlevelnum);
        let index = level - 2;
        if (index as u32) < 26 {
            let (plain, special) = match index {
                0 => (48, 47),
                3 => (50, 49),
                6 => (52, 51),
                7 => (54, 53),
                8 => (56, 55),
                9 => (58, 57),
                10 => (60, 59),
                11 => (62, 61),
                12 => (64, 63),
                13 => (66, 65),
                14 => (68, 67),
                15 => (70, 69),
                16 => (72, 71),
                19 => (74, 73),
                20 => (76, 75),
                21 => (78, 77),
                22 => (80, 79),
                24 => (84, 83),
                25 => (86, 85),
                _ => break 'b1,
            };
            let flags = lw(g::curgameparms + 80);
            sound = if (flags & 0x10000000) != 0 { special } else { plain };
        }
    }
    if sound < 0 {
        soundStart(222, 0);
        return;
    }
    musicSetVol(0, 1);
    soundDelayStartEx(sound, 0, 5, 0);
}

/// Test a moving thing (the segment from `a2` to `a3`, lifted by `f12` and with the height `f13` taken off) of the character `a0` against the panes of glass of the room `a1`:
/// a pane that is hit is smashed by the speed (and the hit counts for the statistics, the part that was hit is recorded and the character takes the hit as damage) (@ 0x259bf0).
pub fn bgGlassMoveTestRoom(a0: i32, a1: i32, a2: i32, a3: i32, f12: f32, f13: f32) {
    let frame = Frame::new(240);
    let sp = frame.sp();
    let list = lw(a1 * 44 + lw(g::levelrooms) + 16);
    if list != 0 {
        let y0 = lwc1(a2 + 4);
        let mut i = 0;
        let x0 = lwc1(a2);
        let dx = lwc1(a3) - x0;
        let z0 = lwc1(a2 + 8);
        let dy = lwc1(a3 + 4) - y0;
        let first = lw(list);
        let dz = lwc1(a3 + 8) - z0;
        let drop = lwc1(0x3acac4);
        swc1(sp + 16, x0);
        swc1(sp + 20, y0 + f12);
        let height = f13 - drop;
        swc1(sp + 24, z0);
        swc1(sp, dx);
        swc1(sp + 4, dy);
        swc1(sp + 8, dz);
        if first != 0 {
            loop {
                'b3: {
                    let glass = lw((i << 2) + list);
                    if 0.0_f32 <= lwc1(glass) {
                        let distance = dx * dx + dy * dy + dz * dz + height * height;
                        let hit = bgHittestDistGlass(sp + 16, sp, glass + 16, glass + 56, glass + 96, glass + 136, sp + 32, sp + 48, distance);
                        if hit != 0 {
                            let source = 0x3a2670;
                            let tail = lw(source + 8);
                            sd(sp + 64, ld(source));
                            sw(sp + 72, tail);
                            let mx = lwc1(sp);
                            let my = lwc1(sp + 4);
                            let mz = lwc1(sp + 8);
                            let speed = ((mx * mx + my * my + mz * mz).abs().psqrt()).pdiv(lwc1(g::updateframesf));
                            let parts = lw(lw(lw(lw(a0 * 0x1210 + lw(g::chrinfo) + 3020) + 32)));
                            StatsAdd(32, a0, 1.0_f32);
                            bgSmashGlass(glass, sp + 32, sp + 48, sp, sp + 64, speed * 1e+01_f32);
                            let mut part = 0;
                            let mut info = lw(g::chrinfo);
                            let next = i + 1;
                            'b7: {
                                loop {
                                    if part >= parts {
                                        break 'b7;
                                    }
                                    if lw((part << 2) + lw(lw(lw(lw(a0 * 0x1210 + info + 3020) + 32) + 92) + 32)) == 0 {
                                        break;
                                    }
                                    part += 1;
                                }
                                chrSetLastHitPart(part);
                                info = lw(g::chrinfo);
                            }
                            propDamage(lw(a0 * 0x1210 + info + 3020), 0, sp + 32, -1, 0, 0, 0, 0, speed + lwc1(0x3acac8));
                            i = next;
                            break 'b3;
                        }
                    }
                    i += 1;
                }
                if lw((i << 2) + list) == 0 {
                    break;
                }
            }
        }
    }
}
