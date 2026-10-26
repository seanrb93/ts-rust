//! Functions: making a prop, drawing the hud, starting a multiplayer game from the front end settings and testing the move of a character against the props of a room.

use super::particle_spawners::random_as_float;
use crate::*;

/// Make a new prop of the kind `a0` at the position (`f12`, `f13`, `f14`) with the heading `f15` (@ 0x2686f8). The slot comes from the prop list: a deleted one is
/// reused when there is one, else the list grows. The prop's object is loaded when it is first used, the instance and the handlers are set up, the data block of the kind
/// (door, car, remote, head) is taken, the walls, floors and rooms are calculated and the damaged-prop data is initialised.
pub fn propNew(a0: i32, f12: f32, f13: f32, f14: f32, f15: f32) -> i32 {
    let mut slot = 0;
    let deleted = lw(g::num_deleted_props);
    'slot: {
        if deleted != 0 {
            let count = lw(g::num_props);
            let reused;
            'found: {
                let mut i = 0;
                if count > 0 {
                    let props = lw(g::levelprops);
                    loop {
                        let prop = i * 592 + props;
                        slot = if (lw(prop + 16) & -0x80000000) != 0 { prop } else { slot };
                        i += 1;
                        if slot != 0 {
                            reused = deleted - 1;
                            break 'found;
                        }
                        if i >= count {
                            break;
                        }
                    }
                }
                reused = deleted - 1;
                if slot == 0 {
                    let props = lw(g::levelprops);
                    sw(g::num_props, count + 1);
                    sw(g::num_deleted_props, 0);
                    slot = props + count * 592;
                    break 'slot;
                }
            }
            sw(g::num_deleted_props, reused);
            sw(slot + 16, 0);
        } else {
            let count = lw(g::num_props);
            let props = lw(g::levelprops);
            sw(g::num_props, count + 1);
            slot = props + count * 592;
        }
    }
    let def = a0 * 28 + g::proptable;
    if lw(def + 4) == 0 {
        if lw(def) == 16 {
            obLoadSetClip(0);
        }
        let object = obLoad(lw(def + 12));
        let kind = lw(def);
        sw(def + 4, object);
        if kind == 16 {
            obLoadSetClip(1);
        }
        if propNeedsBoundsCalculating(a0) != 0 {
            let object = lw(def + 4);
            if a0 != 201 {
                obCalcBounds(object, 0);
            } else {
                obCalcBounds(object, 1);
            }
        }
        if propNeedsWallsFloorsCalculating(a0) == 0 {
            sw(def + 20, 0);
        } else {
            propCalculateWallsFloorsOriginal(lw(def), a0);
            sw(def + 20, 0);
        }
    }
    sw(slot + 4, a0);
    sw(slot, -1);
    sw(slot + 8, lw(def));
    let instance = obInstNew(slot, def);
    sw(slot + 32, instance);
    propPutPropInCollisionList(slot);
    let mut flags = 289;
    if lw(slot + 8) == 2 {
        flags = 33;
    }
    sw(slot + 16, flags);
    let draw_flags = getPropViewportDrawFlags();
    let flags = lw(slot + 16);
    swc1(slot + 112, f12);
    let combined = flags | draw_flags;
    swc1(slot + 116, f13);
    sw(slot + 16, combined);
    swc1(slot + 120, f14);
    let def_flags = lw(def + 24);
    sw(slot + 140, 1);
    sw(slot + 136, 1);
    swc1(slot + 128, f15);
    sw(slot + 16, if (def_flags & 32) == 0 { combined } else { combined | 2048 });
    sw(slot + 24, 0);
    sw(slot + 28, 0);
    sw(slot + 488, 0);
    swc1(slot + 36, f12);
    swc1(slot + 40, f13);
    swc1(slot + 44, f14);
    swc1(slot + 48, f12);
    swc1(slot + 52, f13);
    swc1(slot + 56, f14);
    sw(slot + 72, 0);
    swc1(slot + 76, f15);
    swc1(slot + 84, f15);
    swc1(slot + 80, f15);
    sw(slot + 88, 0);
    sw(slot + 124, 0);
    sw(slot + 132, 0);
    sw(slot + 92, 0);
    sw(slot + 96, 0);
    sw(slot + 532, 0);
    sw(slot + 536, 0);
    sw(slot + 576, -1);
    sw(slot + 184, 0);
    sw(slot + 188, 0);
    sw(slot + 192, -1);
    sw(slot + 196, 0);
    sceVu0UnitMatrix(slot + 208);
    sceVu0UnitMatrix(slot + 272);
    sw(slot + 336, -1);
    sw(slot + 340, -1);
    let none = -1;
    let options = lw(def + 16);
    sh(slot + 546, -1);
    sw(slot + 356, 0);
    let value = if (options ^ 1) != 0 { 0 } else { 20 };
    sh(slot + 544, -1);
    sw(slot + 348, value);
    swc1(slot + 520, 1.0_f32);
    sw(slot + 548, none);
    sw(slot + 344, value);
    sw(slot + 540, none);
    propSetHandler(slot, 0, 0, 0);
    propSetHandler(slot, 1, 0, 0);
    propSetHandler(slot, 2, 0, 0);
    propSetHandler(slot, 3, 0, 0);
    propSetHandler(slot, 4, 0, 0);
    propSetHandler(slot, 5, 0, 0);
    propSetHandler(slot, 6, 0, 0);
    propSetHandler(slot, 7, 0, 0);
    propSetHandler(slot, 8, 0, 0);
    propSetHandler(slot, 9, 0, 0);
    let kind = lw(slot + 8);
    sw(slot + 352, 0);
    if kind != 64 {
        if kind >= 65 {
            if kind == 512 {
                let number = lw(g::numremotes);
                let remotes = lw(g::levelremotes);
                let chr_remotes = lw(g::levelchrremotes);
                sw(g::numremotes, number + 1);
                let remote = remotes + number * 140;
                sw(remote + 100, chr_remotes + number * 108);
                sw(slot + 352, remote);
            } else if kind >= 513 {
                if kind == 2048 {
                    let head = getfreehead();
                    sw(slot + 352, head);
                }
            } else if kind != 128 {
                if kind == 256 {
                    let number = lw(g::numcars);
                    sw(slot + 352, lw(g::levelcars) + number * 112);
                    sw(g::numcars, number + 1);
                }
            }
        } else if kind == 4 {
            let number = lw(g::numdoors);
            sw(slot + 352, lw(g::leveldoors) + number * 40);
            sw(g::numdoors, number + 1);
        }
    }
    propAllocateWallsFloors(slot);
    propCalculateWalls(slot, 0);
    propCalculateFloors(slot, 0);
    propCalculateCentrePos(slot);
    let kind;
    if ((lw(g::lvlevelnum) - 102) as u32) >= (2 as u32) {
        propCalcRooms(slot);
        kind = lw(slot + 8);
        let room = if kind == 2 { lw(slot + 4) - 512 } else { lw(slot + 144) };
        sw(slot + 12, room);
        if lw(slot + 12) == 0 {
            sw(slot + 12, lw(slot + 144));
        }
    } else {
        sw(slot + 12, 0);
        kind = lw(slot + 8);
    }
    if kind == 8 {
        let model = lw(slot + 32);
        let first = lw(model);
        let parts_count = lw(first);
        let parts = first - parts_count * 80;
        if parts_count > 0 {
            let skeleton = lw(model + 92);
            let mut kinds = lw(skeleton + 32);
            let mut i = 0;
            loop {
                let part_kind = lw((i << 2) + kinds) - 8;
                let at = i * 80 + parts;
                let mut value = lhu(at + 68) | 0x4000;
                if (part_kind as u32) < (7 as u32) {
                    if part_kind <= 2 {
                        value = lhu(at + 68) | 0x8000;
                    }
                }
                sh(at + 68, value);
                i += 1;
                if i >= parts_count {
                    break;
                }
                kinds = lw(skeleton + 32);
            }
        }
    }
    initDamagedProps(slot);
    slot
}

/// Draw the hud of the current player (@ 0x2b2d68) while the game is running: the pain flash, the time left of the challenge, the score and rank bars, the bag icon, the
/// message, the health and armour meters, the radar, the ammo of the two hands, the mines, the escort or base game mode additions and, when a demo is playing, the
/// blinking demo text.
pub fn hudGfx() {
    let frame = Frame::new(192);
    let sp = frame.sp();
    let hud = lw(lw(g::currentplayer)) * 352 + g::hudparms;
    if lw(g::lvlevelnum) == 6 {
        return;
    }
    if lw(g::gamestate) != 3 {
        return;
    }
    'meters: {
        'reset: {
            let count = ilinkGetNumLocalPlayers();
            let old_font = textSelect(((count < 3) as i32) ^ 1);
            sw(sp + 16, old_font);
            let old_zb = dlSetZB(0);
            sw(sp + 20, old_zb);
            dlSetBlend(1);
            let mut pain = lw(hud + 268);
            if lw(lw(g::front) + 20) == 0 {
                if pain == 0 {
                    let player = lw(g::currentplayer);
                    sw(hud + 260, -1);
                    let body = lw(player + 384);
                    sw(hud + 264, pain);
                    swc1(hud + 256, lwc1(body + 520));
                    break 'meters;
                }
                sw(sp, 127);
                let level = if pain < 301 { pain } else { 300 };
                sw(sp + 8, 10);
                sw(sp + 4, 20);
                if pain == 600 {
                    sw(sp + 8, 20);
                    sw(sp, 63);
                }
                let rgb = lw(sp + 8) << 8 | lw(sp + 4) << 16;
                let alpha = (level * 160).wrapping_div(300) & 255;
                let high = lw(sp) << 24 | alpha;
                sw(sp + 12, alpha);
                hudPainGfx(high | rgb);
                pain = lw(hud + 268);
            }
            if pain != 0 {
                if lw(hud + 264) >= pain {
                    let fade = lw(hud + 260);
                    if fade <= 0 {
                        break 'meters;
                    }
                    sw(hud + 260, fade - 1);
                    break 'meters;
                }
                sw(hud + 264, pain);
                sw(hud + 260, 127);
                break 'meters;
            }
            let player = lw(g::currentplayer);
            let _ = player;
            break 'reset;
        }
        let player = lw(g::currentplayer);
        sw(hud + 260, -1);
        let body = lw(player + 384);
        sw(hud + 264, 0);
        swc1(hud + 256, lwc1(body + 520));
    }
    let flags = lw(g::curgameparms + 80);
    let player = lw(g::currentplayer);
    if (flags & 0x2000) == 0 {
        if lw(lw(g::front) + 20) == 0 {
            let window_top = lw(g::window + 20);
            let mut y = if lw(g::window + 92) != 0 { window_top + 14 } else { window_top + 26 };
            if (flags & 32) != 0 {
                if GetChallengeTimeLeft() != 0 {
                    hudChallengeTimeLeftGfx(y);
                    y = y + 22 + (2 >> (lw(g::window + 92) & 31));
                }
            }
            if gameModeDisplayRank() != 0 {
                let score = lw(hud + 276);
                let mut width = 0;
                let step = hudScoreWidth() + 4;
                if score != 0 {
                    if score >= 32 {
                        let total = lw(hud + 280);
                        width = step;
                        if score >= total - 32 {
                            let product = (total - score) * step;
                            width = (if -1 < product { product } else { product + 31 }) >> 5;
                        }
                    } else {
                        let product = score * step;
                        width = (if -1 < product { product } else { product + 31 }) >> 5;
                    }
                }
                let player = lw(g::currentplayer);
                if width > 0 {
                    hudScoreGfx(lw(lw(player + 384) + 352), y, width - step);
                }
                let player = lw(g::currentplayer);
                if lw(g::curgameparms + 72) != 6 {
                    hudRankGfx(lw(lw(player + 384) + 352), y, width);
                } else {
                    hudLSScoreGfx(lw(lw(player + 384) + 352), y, width);
                }
                y = y + 22 + (2 >> (lw(g::window + 92) & 31));
            }
            if (lw(g::curgameparms + 80) & 32) != 0 {
                if hudChallengeScoreGfx(y) != 0 {
                    y = y + 22 + (2 >> (lw(g::window + 92) & 31));
                }
            }
            hudBagIconGfx(y);
            let mut player = lw(g::currentplayer);
            if lw(hud + 272) != 0 {
                let dither = dlSetDitherMatrix(2);
                let alpha = (lw(hud + 272) << 7).wrapping_div(180) & 255;
                let count = ilinkGetNumLocalPlayers();
                let font = textSelect(if count < 3 { 2 } else { 1 });
                let width = textWidth(hud);
                let height = lw(g::window + 32);
                let outline = alpha | 0x5000;
                let colour = alpha | 0x407f7f00;
                let x = lw(g::window + 8) - ((((width as u32) >> 31) as i32 + width) >> 1);
                let row = lw(g::window + 24) - ((if -1 < height { height } else { height + 3 }) >> 2);
                textOutline(x, row, hud, colour, outline);
                textSelect(font);
                dlSetDitherMatrix(dither);
                player = lw(g::currentplayer);
            }
            hudMessageDraw(lw(player));
            let player = lw(g::currentplayer);
            let mut radar_row = 0;
            if lw(player + 296) != 4 {
                let health = lwc1(lw(lw(player + 384) + 352) + 12);
                let window_top = lw(g::window + 20);
                let bar_height = 8 >> (lw(g::window + 92) & 31);
                let shift = lw(g::window + 88);
                let bar_width = 160 >> (shift & 31);
                let bar_x = lw(g::window) + (32 >> (shift & 31));
                let pain = lw(hud + 268);
                let bar_y = window_top + bar_height;
                radar_row = window_top;
                sb(sp + 2, 100);
                sb(sp + 3, 96);
                sb(sp + 1, (health * 5e+01_f32) as i32 + 50);
                sb(sp, 0);
                if pain != 0 {
                    sb(sp + 3, 159);
                }
                hudMeterGfx(bar_x, bar_y, bar_width, bar_height, lbu(sp) << 24 | lbu(sp + 3) | (lbu(sp + 2) << 8 | lbu(sp + 1) << 16), 127, health, health);
                let armour = lwc1(lw(lw(g::currentplayer) + 384) + 520);
                if !(armour < 0.5_f32) {
                    sb(sp + 1, 127);
                    sb(sp, (127.0_f32 - armour * 127.0_f32) as i32);
                } else {
                    sb(sp, 127);
                    sb(sp + 1, (armour * 127.0_f32) as i32);
                }
                let hurt = lw(hud + 264);
                sb(sp + 3, 96);
                sb(sp + 2, (3e+01_f32 - armour * 3e+01_f32) as i32);
                if hurt != 0 {
                    sb(sp + 3, 127);
                }
                hudMeterGfx(bar_x, bar_y + 1 + bar_height, bar_width, bar_height, lbu(sp) << 24 | lbu(sp + 3) | (lbu(sp + 2) << 8 | lbu(sp + 1) << 16), lb(hud + 260), armour, lwc1(hud + 256));
                let mut player = lw(g::currentplayer);
                if (lw(g::curgameparms + 80) & 2) != 0 {
                    let height = hudRadarGfx();
                    radar_row = window_top + height;
                    player = lw(g::currentplayer);
                }
                let left_visible = gunIsLeftGunVisible(player + 408);
                let player = lw(g::currentplayer);
                if left_visible != 0 {
                    let window_left = lw(g::window);
                    let x = if lw(g::window + 88) != 0 { window_left + 60 } else { window_left + 80 };
                    hudAmmoGfx(lw(lw(player + 384) + 352) + 268, player + 408, x);
                }
                let mut player = lw(g::currentplayer);
                if lw(player + 600) != 0 {
                    let body = lw(player + 384);
                    let window_right = lw(g::window + 4);
                    let x = if lw(g::window + 88) != 0 { window_right - 60 } else { window_right - 80 };
                    hudAmmoGfx(lw(body + 352) + 376, player + 600, x);
                    player = lw(g::currentplayer);
                }
                hudMinesGfx(lw(player + 384), radar_row + 3);
            }
            let mode = lw(g::curgameparms + 72);
            if mode == 4 {
                let players = lw(g::lvnumplayers);
                if lw(g::escort) != 0 {
                    let x;
                    'x: {
                        'left: {
                            let window = lw(g::window);
                            if players >= 3 {
                                if window == 0 {
                                    break 'left;
                                }
                            } else if (window + 1) == 0 {
                                break 'left;
                            }
                            x = lw(g::window + 8) - 70;
                            break 'x;
                        }
                        x = (32 >> (lw(g::window + 88) & 31)) + 62;
                    }
                    let top = lw(g::window + 20);
                    let row = if lw(g::window + 92) != 0 { top + 14 } else { top + 26 };
                    let title = lw(lw((lw(g::cur_language) << 2) + g::languages) + 3232);
                    let colour = lw(g::teamdata + 4);
                    drawTitleBox(title, x, row, -1, 9, colour);
                    hudAmountBarGfx(x, row + 9, 140, 8, lwc1(lw(g::escort) + 520));
                }
            } else if mode == 6 {
                hudBaseGfx();
            }
        }
    } else if lw(player) == 0 {
        let font = textSelect(2);
        let count = lw(g::flashcount_167);
        let frames = lw(g::updateframesi);
        let x = 32 >> (lw(g::window + 88) & 31);
        sw(g::flashcount_167, (count + frames).wrapping_rem(60));
        if (count + frames).wrapping_rem(60) < 30 {
            textOutline(x, 22, lw(lw((lw(g::cur_language) << 2) + g::languages) + 3260), 0x40787880, 0x5080);
        }
        textSelect(font);
    }
    textSelect(lw(sp + 16));
    dlSetZB(lw(sp + 20));
}

/// Start a game from the front end settings (@ 0x2415a8): the game parameters (at 0x1fc1898) are filled in from the multiplayer settings (the arcade game mode's rules and
/// limits and the players' teams and characters, with the bots added), from the challenge that was picked or from the story level, then the game is reset with them.
pub fn frontmpgStartGame() {
    let frame = Frame::new(224);
    let sp = frame.sp();
    let params = 0x1fc1898;
    let teams = params + 16;
    let chrs = params + 35;
    let names = params + 21;
    frontmpgStoreGoodies();
    let players = lw(0x3aff20);
    sw(params + 4, players);
    sw(0x1fc1898, players);
    sw(params + 8, 0);
    sw(params + 12, 0);
    memset(teams, 0, 5);
    memset(chrs, 0, 14);
    memset(names, 0, 14);
    let kind = lhu(g::gametype_page + 8);
    if kind == 2 {
        let state = 0x1fc11f8;
        let radar = if lw(g::gv_dispradar) == 0 { 0 } else { kind };
        let mode = lw(state + 48);
        let flags = if lw(g::gv_spawngun) == 0 { radar } else { radar | 512 };
        let handicaps = lw(g::gv_handicaps);
        sw(params + 64, lw(state + 40));
        let mut flags_now = flags;
        sw(params + 76, 1);
        sw(params + 72, mode);
        sw(params + 80, flags);
        if handicaps != 0 {
            flags_now |= 0x10000;
        }
        'rules: {
            let index = mode - 1;
            sw(params + 80, flags_now);
            sw(params + 88, 0);
            sw(params + 92, 0);
            sw(params + 96, 0);
            if (index as u32) < (6 as u32) {
                let value;
                match index {
                    0 => {
                        let teamplay = lw(g::gv_teamplay);
                        let flags = lw(params + 80) | (((0 as u32) < (teamplay as u32)) as i32);
                        sw(params + 80, flags);
                        if teamplay != 0 {
                            if lw(g::gv_useteamstarts) == 0 {
                                sw(params + 80, flags | 0x20000);
                            }
                        }
                        if lw(g::gv_bagshoot) == 0 {
                            sw(params + 80, lw(params + 80) | 2048);
                        }
                        sw(params + 88, lw(g::gv_timelimit) * 3600);
                        break 'rules;
                    }
                    1 | 4 => {
                        let time = lw(g::gv_timelimit) * 3600;
                        let score = lw(g::gv_scorelimit);
                        sw(params + 80, lw(params + 80) | 1);
                        sw(params + 96, score);
                        sw(params + 88, time);
                        break 'rules;
                    }
                    2 => {
                        let teamplay = lw(g::gv_teamplay);
                        let flags = lw(params + 80) | (((0 as u32) < (teamplay as u32)) as i32);
                        sw(params + 80, flags);
                        if teamplay != 0 {
                            if lw(g::gv_useteamstarts) == 0 {
                                sw(params + 80, flags | 0x20000);
                            }
                        }
                        let current = lw(params + 80);
                        let kills = lw(g::gv_killlimit);
                        let with_one_shot = if lw(g::gv_oneshotkill) == 0 { current } else { current | 4 };
                        let method = lw(g::gv_scoremethod);
                        sw(params + 88, lw(g::gv_timelimit) * 3600);
                        sw(params + 92, kills);
                        sw(params + 80, with_one_shot);
                        if method != 0 {
                            break 'rules;
                        }
                        value = with_one_shot | 16;
                    }
                    3 => {
                        let current = lw(params + 80);
                        sw(params + 96, 1);
                        value = current | 1;
                    }
                    _ => {
                        sw(params + 80, (lw(params + 80) | 1) & -513);
                        break 'rules;
                    }
                }
                sw(params + 80, value);
            }
        }
        let flags = lw(params + 80);
        if (flags & 1) != 0 {
            sw(params + 80, if lw(g::gv_friendfire) == 0 { flags } else { flags | 8 });
        }
        memset(sp + 32, 0, 20);
        let players = lw(0x3aff20);
        let mut count;
        let mut i;
        if (lw(params + 80) & 1) == 0 {
            count = lw(0x3aff20);
            i = 0;
        } else {
            let mut j = 0;
            'used: {
                if players > 0 {
                    loop {
                        let team = lw((j << 2) + 0x1fc2c88);
                        j += 1;
                        sw(sp + 32 + (team << 2), 1);
                        if j >= players {
                            break 'used;
                        }
                    }
                }
            }
            let mut k = 0;
            let mut next = k + 1;
            loop {
                if countBots(next) != 0 {
                    sw(sp + 32 + (k << 2), 1);
                }
                k = next;
                next = k + 1;
                if k >= 4 {
                    break;
                }
            }
            count = lw(0x3aff20);
            let mut t = 0;
            loop {
                let team = t + 1;
                if lw(sp + 32 + (t << 2)) != 0 {
                    let at = lw(params + 12);
                    sb(teams + at, team);
                    sw(params + 12, at + 1);
                }
                t = team;
                if t >= 5 {
                    break;
                }
            }
            i = 0;
        }
        'players: {
            if count > 0 {
                let mut flags = lw(params + 80);
                loop {
                    let value = if (flags & 1) == 0 { 0 } else { lbu((i << 2) + 0x1fc2c88) + 1 };
                    sb(chrs + i, value);
                    let signon = signonGet(i);
                    let name_at = names + i;
                    let extra_at = params + 49 + i;
                    i += 1;
                    sb(name_at, lbu(signon * 2936 + lw(g::signonarray) + 116));
                    sb(extra_at, 0);
                    count = lw(0x3aff20);
                    if i >= count {
                        break 'players;
                    }
                    flags = lw(params + 80);
                }
            }
        }
        let mut bot = 0x1fc2a88;
        let end = bot + 160;
        let mut added = count;
        let mut head = lw(bot);
        loop {
            'bot: {
                if head < 0 {
                    bot += 16;
                } else if lw(bot + 12) == 0 {
                    bot += 16;
                } else {
                    let team_play = lw(params + 80) & 1;
                    let at = chrs + added;
                    let value;
                    'value: {
                        if team_play != 0 {
                            if lbu(lw(bot + 4) + 0x1fc1234) == 0 {
                                bot += 16;
                                break 'bot;
                            }
                            value = lbu(bot + 4) + 1;
                            break 'value;
                        }
                        value = 0;
                    }
                    let name = lbu(bot);
                    sb(at, value);
                    let extra = lbu(bot + 8);
                    sb(names + added, name);
                    sb(params + 49 + added, extra);
                    added += 1;
                    bot += 16;
                }
            }
            if bot >= end {
                break;
            }
            head = lw(bot);
        }
        sw(0x1fc11f8, 3);
        sw(params + 8, added - count);
    } else if kind >= 3 {
        if kind == 3 {
            let state = 0x1fc11f8;
            let setup = challengeGetSetupNum(lw(state + 44));
            sw(0x1fc1898, 1);
            sw(params + 4, 1);
            sw(params + 68, lw(state + 44));
            sw(params + 64, lw(setup + 4));
            sw(params + 76, 1);
            sw(params + 72, lw(setup + 8));
            let challenge_flags = lw(setup + 12);
            sw(params + 88, 0);
            sw(params + 92, 0);
            sw(params + 80, challenge_flags | 32);
            sw(params + 96, 0);
            memset(sp, 0, 24);
            let list = lw(setup + 48);
            let mut count = 0;
            if lw(list) >= 0 {
                let mut offset = 0;
                loop {
                    sb(names + count, lbu(offset + list));
                    sb(chrs + count, lbu(offset + lw(setup + 48) + 12));
                    sb(params + 49 + count, 2);
                    if (lw(setup + 12) & 1) != 0 {
                        sw(sp + (lw(offset + lw(setup + 48) + 12) << 2), 1);
                    }
                    count += 1;
                    let next = count << 4;
                    if count >= 11 {
                        break;
                    }
                    let list = lw(setup + 48);
                    offset = next;
                    if lw(offset + list) < 0 {
                        break;
                    }
                }
            }
            sw(params + 12, 0);
            sw(params + 8, count - 1);
            if (lw(setup + 12) & 1) != 0 {
                let mut team = 1;
                let mut at = sp + 4;
                loop {
                    if lw(at) == 0 {
                        team += 1;
                    } else {
                        let used = lw(params + 12);
                        sb(teams + used, team);
                        sw(params + 12, used + 1);
                        team += 1;
                    }
                    at += 4;
                    if team >= 6 {
                        break;
                    }
                }
            }
            sw(0x1fc11f8, 4);
        }
    } else if kind == 1 {
        let level = lw(0x1fc1218);
        sw(0x1fc11f8, 2);
        sw(params + 72, 7);
        sw(params + 80, 8);
        let id = lh((level << 3) + (g::storydata + 2));
        sw(params + 88, 0);
        sw(params + 64, id);
        sw(params + 92, 0);
        sw(params + 96, 0);
    }
    gameDeferedReset(params);
}

/// Test the move of a character `a0` in the room `a1` from the position `a2` to `a3` against the props of the room (@ 0x270e08): `a4` and `a5`... the parameters name the
/// move, with the radius `f12`. Doors and cars are tested against their walls (`MoveTestPropWalls`), remote props with a circle, and characters (of the kind 8) as circles
/// whose size depends on whether they are players. The nearest hit gives the fraction of the move that is possible, the position and direction of the hit and the
/// character that was hit; the results go to the pointers passed in (and, on the stack, to the ninth and tenth arguments).
pub fn propMoveTestRoom(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32, a6: i32, a7: i32, f12: f32, f13: f32) -> i32 {
    let frame = Frame::new(320);
    let sp = frame.sp();
    let mut chrinfo = lw(g::chrinfo);
    let mut fraction_limit = f12;
    let mut hit_chr = -1;
    let list_selector = lw(sp + 328);
    let own = lw(a0 * 0x1210 + chrinfo + 3020);
    sw(sp + 68, a5);
    sw(sp + 88, own);
    sw(sp + 72, a6);
    sw(sp + 76, a7);
    let mut best = 1.0_f32;
    sw(sp + 84, 0);
    let count;
    let list;
    if list_selector == 0 {
        count = lw(0x3b00e8);
        list = 0x1fd2f60;
    } else {
        count = lw(0x3b00ec);
        list = 0x1fd3410;
    }
    sw(sp + 80, list);
    let hidden = lw(a0 * 0x1210 + chrinfo + 0x11b4);
    if hidden != 0 {
        let flags = lw(hidden + 16);
        sw(sp + 84, flags);
        sw(hidden + 16, flags & -33);
    }
    let mut index = 0;
    if count > 0 {
        loop {
            'prop: {
                let prop = lw((index << 2) + lw(sp + 80));
                let next = index + 1;
                if prop == 0 {
                    break 'prop;
                }
                let kind;
                let flags;
                let in_room;
                'room: {
                    kind = lw(prop + 8);
                    if kind != 256 {
                        if kind != 4 {
                            flags = lw(prop + 16);
                            in_room = (((lw(prop + 12) ^ a1) as u32) < (1 as u32)) as i32;
                            break 'room;
                        }
                    }
                    let rooms = lw(prop + 184);
                    flags = lw(prop + 16);
                    if rooms <= 0 {
                        in_room = 0;
                    } else if lw(prop + 144) != a1 {
                        let mut k = 1;
                        let mut found = 0;
                        loop {
                            let offset = k << 2;
                            if k >= rooms {
                                break;
                            }
                            k += 1;
                            if lw(prop + offset + 144) == a1 {
                                found = 1;
                                break;
                            }
                        }
                        in_room = found;
                    } else {
                        in_room = 1;
                    }
                }
                if (flags & 32) == 0 {
                    break 'prop;
                }
                if in_room == 0 {
                    break 'prop;
                }
                let hit_x;
                if kind != 8 {
                    let prop_kind = lw(prop + 4);
                    if prop_kind != 425 {
                        if kind == 4 {
                            propCalculateWalls(prop, 1);
                        }
                        let walls = lw(prop + 484);
                        if walls == 0 {
                            break 'prop;
                        }
                        let fraction = MoveTestPropWalls(a0, prop, a2, a3, a4, sp + 32, sp + 64, f12, f13);
                        let reach = lwc1(sp + 64);
                        let x = lwc1(sp + 32);
                        if reach < fraction_limit {
                            fraction_limit = reach;
                            let y = lwc1(sp + 36);
                            swc1(sp + 16, x);
                            swc1(sp + 20, y);
                        }
                        let x = lwc1(sp + 32);
                        if !(fraction < best) {
                            break 'prop;
                        }
                        best = fraction;
                        hit_x = x;
                    } else {
                        let height = lwc1(prop + 52);
                        if !(height < lwc1(a2 + 16)) {
                            break 'prop;
                        }
                        if !(lwc1(a2 + 4) < height + lwc1(0x3acd1c)) {
                            break 'prop;
                        }
                        remoteGetPivotePos(prop, sp + 48);
                        let margin = lwc1(0x3acd20);
                        let reach = fraction_limit + margin;
                        let dx = lwc1(sp + 48) - lwc1(a3);
                        let dz = lwc1(sp + 56) - lwc1(a3 + 8);
                        let distance = dx * dx + dz * dz;
                        let radius_x;
                        if !(distance < reach * reach) {
                            radius_x = lwc1(a4);
                        } else {
                            let length = distance.abs().psqrt();
                            fraction_limit = length - margin;
                            if !(0.0_f32 < length) {
                                swc1(sp + 16, 0.0_f32);
                                swc1(sp + 20, 0.0_f32);
                            } else {
                                swc1(sp + 20, (dz).pdiv(length));
                                swc1(sp + 16, (dx).pdiv(length));
                            }
                            radius_x = lwc1(a4);
                        }
                        if !(radius_x == 0.0_f32) {
                        } else if lwc1(a4 + 4) == 0.0_f32 {
                            break 'prop;
                        }
                        let fraction = hittestMoveCircleToCircle(sp + 32, lwc1(a3), lwc1(a3 + 8), f12, radius_x, lwc1(a4 + 4), lwc1(sp + 48), lwc1(sp + 56), lwc1(0x3acd24));
                        let x = lwc1(sp + 32);
                        if !(fraction < best) {
                            break 'prop;
                        }
                        best = fraction;
                        hit_x = x;
                    }
                    let y = lwc1(sp + 36);
                    hit_chr = -1;
                    swc1(sp, hit_x);
                    swc1(sp + 4, y);
                } else {
                    let height = lwc1(prop + 52);
                    if !(height < lwc1(a2 + 16)) {
                        break 'prop;
                    }
                    let own = lw(sp + 88);
                    if !(lwc1(a2 + 4) < height + f13) {
                        break 'prop;
                    }
                    if own == prop {
                        break 'prop;
                    }
                    let scale;
                    let centre_x;
                    'scale: {
                        if lw(own + 356) == 0 {
                            if lw(prop + 356) == 0 {
                                centre_x = lwc1(prop + 48);
                                scale = 0.5_f32;
                                break 'scale;
                            }
                        }
                        scale = 1.0_f32;
                        centre_x = lwc1(prop + 48);
                    }
                    let reach = fraction_limit + f12;
                    let dx = lwc1(a3) - centre_x;
                    let dz = lwc1(a3 + 8) - lwc1(prop + 56);
                    let distance = ((dx * dx + dz * dz)).pdiv((scale * scale));
                    let radius_x;
                    if !(distance < reach * reach) {
                        radius_x = lwc1(a4);
                    } else {
                        'direction: {
                            'random: {
                                let length = distance.abs().psqrt();
                                fraction_limit = length - f12;
                                if 0.0_f32 < length {
                                    if dx == 0.0_f32 {
                                        if dz == 0.0_f32 {
                                            break 'random;
                                        }
                                    }
                                    swc1(sp + 20, (dz).pdiv(length));
                                    swc1(sp + 16, (dx).pdiv(length));
                                    break 'direction;
                                }
                            }
                            let first = random_as_float() * 2.3283064e-10_f32;
                            swc1(sp + 16, first + first - 1.0_f32);
                            let second = random_as_float();
                            let rx = lwc1(sp + 16);
                            let second = second * 2.3283064e-10_f32;
                            let ry = second + second - 1.0_f32;
                            let inverse = (1.0_f32).pdiv((rx * rx + ry * ry));
                            swc1(sp + 20, ry * inverse);
                            swc1(sp + 16, rx * inverse);
                        }
                        radius_x = lwc1(a4);
                    }
                    if !(radius_x == 0.0_f32) {
                    } else if lwc1(a4 + 4) == 0.0_f32 {
                        break 'prop;
                    }
                    let radius = f12 * scale;
                    let fraction = hittestMoveCircleToCircle(sp + 32, lwc1(a3), lwc1(a3 + 8), radius, radius_x, lwc1(a4 + 4), lwc1(prop + 48), lwc1(prop + 56), radius);
                    let y = lwc1(sp + 36);
                    if fraction < best {
                        best = fraction;
                        let data = lw(prop + 352);
                        swc1(sp, lwc1(sp + 32));
                        swc1(sp + 4, y);
                        hit_chr = lw(data);
                    }
                }
            }
            index += 1;
            if index >= count {
                break;
            }
        }
        chrinfo = lw(g::chrinfo);
    }
    let hidden = lw(a0 * 0x1210 + chrinfo + 0x11b4);
    let flags = lw(sp + 84);
    if hidden != 0 {
        sw(hidden + 16, flags);
    }
    let out = lw(sp + 68);
    if out != 0 {
        swc1(out, best);
    }
    let out = lw(sp + 72);
    let x = lwc1(sp);
    if out != 0 {
        let y = lwc1(sp + 4);
        swc1(out, x);
        swc1(out + 4, y);
    }
    let out = lw(sp + 76);
    if out != 0 {
        swc1(out, fraction_limit);
    }
    let out = lw(sp + 320);
    let nx = lwc1(sp + 16);
    if out != 0 {
        let ny = lwc1(sp + 20);
        swc1(out, nx);
        swc1(out + 4, ny);
    }
    hit_chr
}
