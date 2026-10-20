//! Functions: swiping glass, exploding mines, respawning a player, making a player's gun, loading a pad file, informing other enemies, selecting a character on the select
//! screen, the launch handler of the story, making an object instance and the last stand game's tick.

use crate::*;

/// Test a swipe (the line from `a2` to `a4`) against the panes of glass of the room `a0`: each pane that is still whole (the first float not negative) and hit within reach is
/// broken (`bgSmashGlass`) unless a wall is in the way, and the hit adds to the statistics of a character (@ 0x2599c0).
pub fn bgGlassSwipeTestRoom(a0: i32, a1: i32, a2: i32, _a3: i32, a4: i32) {
    let frame = Frame::new(256);
    let sp = frame.sp();
    let list = lw(a0 * 44 + lw(g::levelrooms) + 16);
    if list != 0 {
        let mut i = 0;
        let dx = lwc1(a4) - lwc1(a2);
        let end_z = lwc1(a4 + 8);
        let dy = lwc1(a4 + 4) - lwc1(a2 + 4);
        let start_z = lwc1(a2 + 8);
        let first = lw(list);
        swc1(sp + 16, dx);
        let dz = end_z - start_z;
        swc1(sp + 20, dy);
        swc1(sp + 24, dz);
        if first != 0 {
            loop {
                let glass = lw((i << 2) + list);
                if 0.0_f32 <= lwc1(glass) {
                    let distance = dx * dx + dy * dy + dz * dz;
                    let hit = bgHittestDistGlass(a2, sp + 16, glass + 16, glass + 56, glass + 96, glass + 136, sp + 32, sp + 48, distance);
                    let hit_x = lwc1(sp + 32);
                    if hit != 0 {
                        let hit_y = lwc1(sp + 36) - lwc1(a2 + 4);
                        let hit_z = lwc1(sp + 40) - lwc1(a2 + 8);
                        let room = lw(a1 + 12);
                        let flags = lw(a1 + 16) & -33;
                        swc1(sp + 64, hit_x - lwc1(a2));
                        swc1(sp + 68, hit_y);
                        swc1(sp + 72, hit_z);
                        sw(a1 + 16, flags);
                        sw(sp, 0);
                        sw(sp + 8, 0);
                        let blocked = bgLineTest(room, a2, sp + 64, 0, 0, 0, 0, 0);
                        let current;
                        if blocked != 0 {
                            current = lw(a1 + 16);
                        } else {
                            let kind = lw(a1 + 8);
                            let source = 0x3a2670;
                            let tail = lw(source + 8);
                            sd(sp + 80, ld(source));
                            sw(sp + 88, tail);
                            if kind == 8 {
                                StatsAdd(32, lw(lw(a1 + 352)), 1.0_f32);
                            }
                            bgSmashGlass(glass, sp + 32, sp + 48, sp + 16, sp + 80, 0.25_f32);
                            current = lw(a1 + 16);
                        }
                        sw(a1 + 16, current | 32);
                    }
                }
                i += 1;
                if lw((i << 2) + list) == 0 {
                    break;
                }
            }
        }
    }
}

/// Explode the mines (bullet kinds 14 to 16, among the bullets 20 to 49) within the radius `f12` of the position `a0` that have a clear line to it: a mine that is not of the
/// kind 16 explodes; a remote one of the kind 16 is armed to explode in 2 (@ 0x28a470).
pub fn bulletExplodeMines(a0: i32, f12: f32) {
    let frame = Frame::new(176);
    let sp = frame.sp();
    let mut i = 20;
    let mut bullets = lw(0x3b0138);
    let mut x = lwc1(a0);
    let mut y = lwc1(a0 + 4);
    let mut z = lwc1(a0 + 8);
    loop {
        let bullet = i * 276 + bullets;
        if ((lw(bullet) - 14) as u32) >= 3 {
            i += 1;
        } else {
            let dx = x - lwc1(bullet + 24);
            swc1(sp + 16, dx);
            let dy = y - lwc1(bullet + 28);
            swc1(sp + 20, dy);
            let dz = z - lwc1(bullet + 32);
            swc1(sp + 24, dz);
            if (dx * dx + dy * dy + dz * dz) < f12 * f12 {
                'b5: {
                    sw(sp, 0);
                    let position = bullet + 24;
                    sw(sp + 8, 0);
                    let hit = bgLineTest(lw(bullet + 100), position, sp + 16, 0, 0, 0, 0, 0);
                    bullets = lw(0x3b0138);
                    if hit == 0 {
                        if lw(bullet) != 16 {
                            let number = lw(bullet + 108);
                            let mut owner = 0;
                            if number >= 0 {
                                let kind = lw(bullet + 104);
                                if kind != 1 {
                                    if kind == 2 {
                                        owner = lw(number * 0x1210 + lw(g::chrinfo) + 3020);
                                    }
                                } else {
                                    owner = lw(number * 1820 + lw(g::players) + 384);
                                }
                            }
                            let roll = newrnd();
                            soundStart(((roll as i32 as u32) % 6) as i32 + 98, position);
                            particleNewExplosion(0, 0, position, lw(bullet), position, bullet + 48, owner, 0);
                            particleNewSmoke(0, position);
                            decalNewDecal(lw(bullet), lw(bullet + 100), 0, position, 0);
                            bulletFree(bullet);
                        } else {
                            if lw(bullet + 120) >= 2 {
                                x = lwc1(a0);
                                break 'b5;
                            }
                            sw(bullet + 120, 2);
                        }
                        bullets = lw(0x3b0138);
                    }
                    x = lwc1(a0);
                }
                y = lwc1(a0 + 4);
                z = lwc1(a0 + 8);
            }
            i += 1;
        }
        if i >= 50 {
            break;
        }
    }
}

/// Respawn the current player: the mode is 0, the hud values are cleared, the character respawns with its ammo reset, the camera mode is set (1 with a boss parameter), guns
/// are cleared, the player goes to its start position, the lists are cleared, the timers are set to 60 and the standing animation starts (@ 0x280178).
pub fn playerRespawn() {
    let player = lw(g::currentplayer);
    let hud = g::hudparms;
    sw(player + 296, 0);
    let record = lw(lw(player + 384) + 352);
    sw(lw(player) * 352 + hud + 268, 0);
    sw(lw(player) * 352 + hud + 272, 0);
    chrRespawn(record);
    let player_now = lw(g::currentplayer);
    let chr = lw(player_now + 384);
    sw(chr + 16, lw(chr + 16) | 289);
    sw(player_now + 20, lw(player_now + 20) & -4);
    swc1(player_now + 112, lwc1(lh(record + 4) * 44 + (g::chrs + 28)));
    chrResetAmmo(record);
    if bossCheckParm(0x3a87e0) != 0 {
        camSetMode(1);
    } else {
        camSetMode(0);
    }
    chrClearGuns(record);
    playerClearGuns(lw(g::currentplayer));
    playerSetStartPos(lw(g::currentplayer));
    let player_now = lw(g::currentplayer);
    let mut i = 0;
    let players = lw(g::lvnumplayers);
    let chars = lw(g::numactivechars);
    sw(player_now + 804, 0);
    let total = players + chars;
    sw(player_now + 808, 0);
    sw(player_now + 812, 0);
    sw(player_now + 816, 0);
    if total > 0 {
        loop {
            let offset = i * 12;
            i += 1;
            sw(offset + lw(player_now + 820), 0);
            if i >= total {
                break;
            }
        }
    }
    let sixty = 6e+01_f32;
    let chr = lw(player_now + 384);
    swc1(player_now + 272, sixty);
    swc1(player_now + 260, sixty);
    swc1(player_now + 264, sixty);
    swc1(player_now + 268, sixty);
    let two = 2.0_f32;
    let height = lwc1(chr + 52);
    setAnim(chr, 3, 0.0_f32, two);
    setAnim(lw(lw(g::currentplayer) + 384), 3, 0.0_f32, two);
    let player_now = lw(g::currentplayer);
    let kind = lw(g::curgameparms + 72);
    let chr = lw(player_now + 384);
    let model = lw(chr + 32);
    swc1(chr + 52, height);
    swc1(model + 128, 0.0_f32);
    swc1(model + 132, 0.0_f32);
    swc1(model + 136, 0.0_f32);
    swc1(model + 104, lwc1(player_now + 140));
    swc1(model + 108, lwc1(player_now + 144));
    swc1(model + 112, lwc1(player_now + 148));
    swc1(model + 116, lwc1(player_now + 140));
    swc1(model + 120, lwc1(player_now + 144));
    swc1(model + 124, lwc1(player_now + 148));
    swc1(player_now + 1776, 0.0_f32);
    swc1(player_now + 1780, 0.0_f32);
    swc1(player_now + 128, 0.0_f32);
    if kind == 7 {
        swc1(record + 2900, 0.0_f32);
    }
    let x = lwc1(player_now + 140);
    let y = lwc1(player_now + 144);
    let z = lwc1(player_now + 148);
    swc1(player_now + 880, 0.0_f32);
    swc1(player_now + 884, x);
    swc1(player_now + 888, y);
    swc1(player_now + 892, z);
}

/// Make the gun prop of the player's gun slot `a1` (the left hand when `a2` is set): the old prop is deleted, and unless the left gun is not wanted a gun prop and its drawn
/// model are made and attached to the hand bone, the slot is reset, the gun is loaded and the timers are set from the gun's values (@ 0x2941a8).
pub fn gunCreatePlayerGun(a0: i32, a1: i32, a2: i32) {
    let old = lw(a1 + 64);
    let record = lw(lw(lw(g::currentplayer) + 384) + 352);
    let gun = lw(a1) * 400 + g::guns;
    if old != 0 {
        propDelete(old);
        sw(a1 + 64, 0);
    }
    if a2 != 0 {
        if lw(record + 376) != lh((lw(record + 260) << 3) + (g::guncombos + 4)) {
            return;
        }
        if gunLeftHandAble(record) == 0 {
            sw(a0, 0);
            return;
        }
    }
    let view = gunNew(lw(gun), 0.0_f32, -1e+02_f32, 0.0_f32, 0.0_f32);
    let player = lw(g::currentplayer);
    sw(view + 136, 2);
    sw(view + 140, 2);
    let model = lw(view + 32);
    let room = lw(player) + 201;
    sw(view + 16, lw(view + 16) | 0x40000000);
    sw(view + 12, room);
    sw(a0 + 12, view);
    obInstSetAnim(model, 1, 0.0_f32);
    let prop = propNew(lw(gun + 4), lwc1(0x3ad448), 1.0_f32, lwc1(0x3ad44c), 0.0_f32);
    let player = lw(g::currentplayer);
    sw(a1 + 64, prop);
    let chr = lw(player + 384);
    let bone = if a2 == 0 { lw(lw(lw(chr + 32) + 92) + 28) } else { lw(lw(lw(chr + 32) + 92) + 24) };
    propSetAttached(lw(a1 + 64), lw(chr + 32), bone, if a2 == 0 { 1 } else { 2 });
    let attached = lw(a1 + 64);
    let owner = lw(lw(g::currentplayer) + 384);
    swc1(attached + 564, lwc1(owner + 48));
    swc1(attached + 568, lwc1(owner + 52));
    swc1(attached + 572, lwc1(owner + 56));
    resetPlayerGun(a0, a1);
    let player = lw(g::currentplayer);
    let mut at = a1 + 60;
    for _ in 0..15 {
        sw(at, 0);
        at -= 4;
    }
    gunReloadCore(lw(lw(player + 384) + 352), a1);
    resetPlayerGunReload(a0, a1);
    let player = lw(g::currentplayer);
    let first = lwc1(gun + 44);
    let flags = lw(player + 20);
    swc1(player + 268, first);
    swc1(player + 272, lwc1(gun + 48));
    if (flags & 2) == 0 {
        swc1(player + 268, lwc1(player + 264));
        swc1(player + 264, 6e+01_f32);
    } else {
        swc1(player + 264, first);
    }
}

/// Load the pad file `a0` (from the preload list or the file): the pads (28 bytes) and the links (20 bytes) are copied to the level's tables, the pads and links that name a
/// pad that is not there are dropped, and the counts of the kinds of pads and links are kept (@ 0x265558).
pub fn padLoad(a0: i32) {
    let frame = Frame::new(160);
    let sp = frame.sp();
    sw(g::numpads, 0);
    sw(g::numlinks, 0);
    sw(0x3b00a0, 0);
    sw(0x3b00a4, 0);
    sw(0x3b00a8, 0);
    let preloaded = preloadGetFile(a0);
    let file;
    if preloaded == 0 {
        file = fileLoad(a0, 1, sp);
    } else {
        file = lw(preloaded + 16);
        sw(sp, lw(preloaded + 20));
    }
    if lw(sp) > 0 {
        let pad_count = lw(file + 4);
        let pads = file + 12;
        let mut dropped = 0;
        sw(g::numlinks, lw(file + 8));
        let mut i = 0;
        sw(g::numpads, pad_count);
        if pad_count > 0 {
            let level_pads = lw(g::levelpads);
            loop {
                let source = i * 28 + pads;
                let dest = i * 28 + level_pads;
                let a = lw(source);
                swc1(dest + 12, lwc1(source + 12));
                sw(dest, a);
                let flags = lw(source + 4);
                swc1(dest + 16, lwc1(source + 16));
                let height = lwc1(source + 20);
                sh(dest + 8, lhu(source + 8));
                swc1(dest + 20, height);
                sw(dest + 4, flags);
                swc1(dest + 24, lwc1(source + 24));
                if (flags & 1) != 0 {
                    sw(0x3b00a0, lw(0x3b00a0) + 1);
                }
                i += 1;
                if i >= lw(g::numpads) {
                    break;
                }
            }
        }
        let links = lw(g::numlinks);
        let mut j = 0;
        if links > 0 {
            let mut link = pads + pad_count * 28;
            loop {
                'b7: {
                    let from = padNumFromExtref(lw(link + 4));
                    let mut count = lw(g::numlinks);
                    if from != -1 {
                        let to = padNumFromExtref(lw(link + 8));
                        let index = j - dropped;
                        if to != -1 {
                            sw(index * 20 + lw(g::levellinks), lw(link));
                            let from_pad = padNumFromExtref(lw(link + 4));
                            sw(index * 20 + lw(g::levellinks) + 4, from_pad);
                            let to_pad = padNumFromExtref(lw(link + 8));
                            let entry = index * 20 + lw(g::levellinks);
                            let kind = lw(entry);
                            sw(entry + 8, to_pad);
                            sw(entry + 12, 0);
                            if (kind & 1) == 0 {
                                break 'b7;
                            }
                            let level_pads = lw(g::levelpads);
                            let other = lw(0x3b00a8);
                            if lh(lw(entry + 4) * 28 + level_pads + 8) != lh(to_pad * 28 + level_pads + 8) {
                                sw(0x3b00a8, other + 1);
                                break 'b7;
                            }
                            sw(0x3b00a4, lw(0x3b00a4) + 1);
                            break 'b7;
                        }
                        count = lw(g::numlinks);
                    }
                    dropped += 1;
                    sw(g::numlinks, count - 1);
                }
                j += 1;
                link += 20;
                if j >= links {
                    break;
                }
            }
        }
    }
}

/// An enemy `a0` raised the alarm (the game kind 7): every other enemy that is awake and able to react, and is near it (inside its hearing distance) or in the same room
/// as the one a path of portals leads to, takes a target and goes into the fight state, and the activation sound plays (@ 0x2cb908).
pub fn enemyInformOthers(a0: i32) {
    let frame = Frame::new(192);
    let sp = frame.sp();
    let record = lw(a0 + 352);
    if lw(g::curgameparms + 72) == 7 {
        let mut i = 0;
        if lw(g::numactivechars) > 0 {
            let mut chrs = g::chr;
            loop {
                let index = i << 2;
                let mut flag = 0;
                let other = lw(index + chrs);
                let other_record = lw(other + 352);
                if other == a0 {
                    i += 1;
                } else if (lw(other_record + 2716) & 0x32000) == 0 {
                    i += 1;
                } else if (lw(other_record + 2704) & 256) == 0 {
                    i += 1;
                } else {
                    let x = lwc1(a0 + 48);
                    let y = lwc1(a0 + 52);
                    let dx = lwc1(other + 48) - x;
                    let dy = lwc1(other + 52) - y;
                    let z = lwc1(a0 + 56);
                    let dz = lwc1(other + 56) - z;
                    if (dx * dx + dy * dy + dz * dz) < lwc1(other_record + 2884) {
                        flag = 1;
                    }
                    let up = lwc1(0x3ada74);
                    swc1(sp, x);
                    swc1(sp + 8, z);
                    swc1(sp + 4, y + up);
                    swc1(sp + 16, lwc1(other + 48) - x);
                    swc1(sp + 20, lwc1(other + 52) - y);
                    swc1(sp + 24, lwc1(other + 56) - z);
                    let mut room = bgPortalCrossed(sp, sp + 16, lw(a0 + 12), 0, 0);
                    let mut previous = 0;
                    let mut last = 0;
                    if room == 0 {
                        i += 1;
                    } else {
                        i += 1;
                        loop {
                            let crossed = bgPortalCrossed(sp, sp + 16, room, previous, 0);
                            last = crossed;
                            previous = room;
                            room = crossed;
                            if last == previous {
                                break;
                            }
                        }
                    }
                    flag = if (last ^ lw(lw(index + g::chr) + 12)) == 0 { 1 } else { flag };
                }
                let mut count = lw(g::numactivechars);
                if flag != 0 {
                    let target = suggestTarget(a0, lw(record + 2788), 1);
                    let pending = lw(other_record + 2724);
                    sw(other_record + 2788, target);
                    if pending == 0 {
                        let state = lw(other_record + 2716);
                        sw(other_record + 2716, 8);
                        sw(other_record + 2720, state);
                    } else {
                        let state = lw(other_record + 2716);
                        sw(other_record + 2716, pending);
                        sw(other_record + 2720, state);
                    }
                    sw(other_record + 2892, 0);
                    enemyPlayActivationSound(a0);
                    count = lw(g::numactivechars);
                }
                chrs = g::chr;
                if !(i < count) {
                    break;
                }
            }
        }
    }
}

/// Select the character `a1` for the player `a0` on the select screen: the previous stage prop and guns are deleted, the previous character is deleted (its textures freed if
/// no other player shows the same model), the new character is made and posed (animation by its number), and put in front of the player's view (@ 0x2366b0).
pub fn selectChar(a0: i32, a1: i32) {
    let index = a0 << 2;
    let stage = index + g::stageprop;
    let prop = lw(stage);
    if prop != 0 {
        propDelete(prop);
        sw(stage, 0);
        sw(index + g::propdrawflag, 0);
    }
    let gun = index + g::gun;
    let prop = lw(gun);
    if prop != 0 {
        propDelete(prop);
        sw(gun, 0);
        sw(index + g::gundrawflag, 0);
    }
    let gun2 = index + g::gun2;
    let prop = lw(gun2);
    if prop != 0 {
        propDelete(prop);
        sw(gun2, 0);
        sw(index + g::gundrawflag2, 0);
    }
    let characters = 0x1fc1238;
    let slot = index + characters;
    let difference = a1 - 41;
    if lw(slot) != 0 {
        let mut same = 0;
        let mut at = characters;
        let mut left = 3;
        loop {
            let character = lw(at);
            let next = same + 1;
            if character != 0 {
                same = if (lw(lw(character + 32)) ^ lw(lw(lw(slot) + 32))) == 0 { next } else { same };
            }
            left -= 1;
            at += 4;
            if left < 0 {
                break;
            }
        }
        if same == 1 {
            obFreeTextures(lw(lw(lw(index + 0x1fc1238) + 32)));
        }
        let at = index + characters;
        propDelete(lw(at));
        sw(at, 0);
    }
    let chr = chrNew(a1, a1, 1, a0, -1, 1, 0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32);
    let flags = lw(chr + 16);
    sw(chr + 576, a0);
    sw(chr + 344, 0);
    sw(chr + 16, (flags | 0x1000000) & -0xe000001);
    if (difference as u32) >= 3 {
        if a1 != 29 {
            obInstSetAnim(lw(chr + 32), lw(a1 * 20 + g::fe_animation_list), 1.0_f32);
        } else {
            obInstSetAnim(lw(chr + 32), lw(g::fe_animation_list + 580), lwc1(0x3ac9bc));
        }
    } else {
        obInstSetAnim(lw(chr + 32), lw(a1 * 20 + g::fe_animation_list), 0.0_f32);
    }
    let player = lw(g::currentplayer);
    let distance = lwc1(0x3ac9c0);
    let x = lwc1(player + 152);
    let dx = lwc1(player + 176) * distance;
    sw(chr + 344, 0);
    swc1(chr + 48, x + dx);
    swc1(chr + 52, lwc1(player + 156) + lwc1(player + 180) * distance - 1.0_f32);
    swc1(chr + 56, lwc1(player + 160) + lwc1(player + 184) * distance);
    tlhAdjustChrPosition(chr);
    sw(index + 0x1fc1238, chr);
}

/// The launch level's story handler: when the player is at the pad for the current step (the state 0 to 3), two doors open, a sound plays, a link between pads is enabled and
/// the state moves to the next (@ 0x2cc210).
pub fn enemyAILaunchHandler1() {
    let state = lw(g::story_state1);
    if (state as u32) < 5 {
        let (pad, door_a, door_b, link_a, link_b, next) = match state {
            0 => (1747, 5, 6, 1742, 1018, 1),
            1 => (1027, 7, 8, 1029, 1030, 2),
            2 => (1059, 1, 2, 1065, 1067, 3),
            3 => (1064, 3, 4, 1070, 1570, 4),
            _ => return,
        };
        if enemyStoryPlayerAtPad(pad) == 0 {
            return;
        }
        doorOpen(door_a);
        doorOpen(door_b);
        if lw(g::updateframesi) > 0 {
            soundStart(45, 0);
        }
        let link = padGetLinkNum(link_a, link_b);
        linkEnable(link);
        sw(g::story_state1, next);
    }
}

/// Make an instance of the object `a1` for the prop `a0`: a free record of 328 bytes is found (a deleted one, or the next at the end), cleared and set up, with the texture
/// buffers' matrices for a character, the skeleton bound for the animated ones, and the light added for a light prop (@ 0x2606f0).
pub fn obInstNew(a0: i32, a1: i32) -> i32 {
    let mut model = lw(a1 + 4);
    let mut instance = 0;
    if lw(model + 36) == 0 {
        obLoadTextures(model);
        model = lw(a1 + 4);
    }
    let deleted = lw(g::num_deleted_instances);
    if deleted != 0 {
        let count = lw(g::num_instances);
        let mut i = 0;
        if count > 0 {
            let list = lw(g::obinstances);
            loop {
                let candidate = i * 328 + list;
                instance = if lw(candidate) == 0 { candidate } else { instance };
                i += 1;
                if instance != 0 {
                    break;
                }
                if i >= count {
                    break;
                }
            }
        }
        if instance == 0 {
            let list = lw(g::obinstances);
            sw(g::num_instances, count + 1);
            sw(g::num_deleted_instances, 0);
            instance = list + count * 328;
        } else {
            sw(g::num_deleted_instances, deleted - 1);
        }
    } else {
        let used = lw(g::num_instances);
        let list = lw(g::obinstances);
        sw(g::num_instances, used + 1);
        instance = list + used * 328;
    }
    'b7: {
        sw(instance + 244, a0);
        sw(instance + 96, 0);
        sw(instance + 100, 0);
        sw(instance + 140, 0);
        sw(instance + 152, 0);
        sw(instance + 160, 0);
        sw(instance + 164, 0);
        sw(instance + 204, 0);
        sw(instance + 216, 0);
        sw(instance + 224, 0);
        sw(instance + 228, 0);
        sw(instance + 232, 0);
        sw(instance + 236, 0);
        sw(instance + 240, 0);
        sw(instance + 252, 0);
        sw(instance + 256, 0);
        sw(instance + 260, 0);
        sw(instance + 264, 0);
        sw(instance + 268, 0);
        sw(instance + 272, 0);
        sw(instance + 276, 0);
        sw(instance + 280, 0);
        sw(instance + 284, 0);
        sw(instance + 288, 0);
        sw(instance, model);
        obInstResetOverride(instance);
        if lw(lw(a1 + 4) + 4) != 0 {
            if lw(lw(instance + 244) + 8) == 8 {
                let object = lw(instance);
                sw(instance + 24, 0);
                let parts = lw(object + 4);
                sw(instance + 28, 0);
                let first = (parts << 7) + 0x70000000;
                let step = parts << 6;
                sw(instance + 4, first);
                let second = first + step;
                sw(instance + 8, second);
                let third = second + step;
                sw(instance + 12, third);
                let fourth = third + step;
                sw(instance + 16, fourth);
                sw(instance + 20, fourth + step);
                break 'b7;
            }
        }
        sw(instance + 4, 0);
        sw(instance + 24, 0);
        sw(instance + 28, 0);
    }
    let kind;
    if (lw(a1 + 16) as u32) >= 2 {
        sw(instance + 88, 0);
        sw(instance + 92, 0);
        kind = lw(a0 + 8);
    } else {
        obBindSkeleton(instance, a1, ((lw(a1 + 20) as u32) < 1) as i32);
        kind = lw(a0 + 8);
    }
    if kind == 2 {
        bglightAdd(instance);
    }
    sh(instance + 292, 0);
    sw(instance + 248, 0);
    sw(instance + 32, 0);
    sw(instance + 36, 0);
    instance
}

/// The tick of the last stand game: the level completes (flag 0x20000000) when every base is destroyed or every player is dead; each living player inside the zone of the
/// level is counted down (a message each second) and killed when the time is up, and out of the zone the message is cleared (@ 0x21e018).
pub fn gmLastStandTick() {
    let info = getLastStandInfo(lw(g::lvlevelnum));
    let mut dead = 0;
    let bases = enemyGetNumBases();
    let mut players = lw(g::lvnumplayers);
    if bases > 0 {
        let destroyed = gmLastStandAllBasesDestroyed();
        players = lw(g::lvnumplayers);
        if destroyed != 0 {
            sw(g::curgameparms + 80, lw(g::curgameparms + 80) | 0x20000000);
            gameLevelCompleted();
            return;
        }
    } else {
        'b2: {
            let mut i = 0;
            if players > 0 {
                let first = lw(g::players);
                let mut at = first + 296;
                if lw(first + 296) == 4 {
                    dead = 1;
                    let count = players;
                    loop {
                        i += 1;
                        at += 1820;
                        if i >= count {
                            break 'b2;
                        }
                        if lw(at) != 4 {
                            break 'b2;
                        }
                        dead += 1;
                    }
                }
            }
        }
        if dead == players {
            sw(g::curgameparms + 80, lw(g::curgameparms + 80) | 0x20000000);
            gameLevelCompleted();
            return;
        }
    }
    let mut i = 0;
    if players > 0 {
        loop {
            let offset = i * 1820;
            let player = i * 1820 + lw(g::players);
            let chr = lw(player + 384);
            let dx = lwc1(chr + 48) - lwc1(info + 4);
            let dz = -(lwc1(chr + 56) - lwc1(info + 8));
            let distance_squared = dx * dx + dz * dz;
            let record = lw(chr + 352);
            if lw(player + 296) == 4 {
            } else if !(lwc1(info + 12) < distance_squared.abs().sqrt()) {
                if lw(record + 252) != 0 {
                    sw(record + 252, 0);
                    hudMessage(i, 0);
                }
            } else {
                let mut deadline = lw(record + 252);
                let mut now = lw(g::gameframe);
                if deadline == 0 {
                    let set = (lw(g::gameframe) as f32 + lwc1(info + 16) * 6e+01_f32) as i32;
                    sw(record + 252, set);
                    deadline = set;
                    now = lw(g::gameframe);
                }
                let left = deadline - now;
                if left > 0 {
                    let seconds = left.wrapping_div(60);
                    if left.wrapping_rem(60) == 0 {
                        let texts = lw((lw(g::cur_language) << 2) + g::languages);
                        let text = vafmt(&[0x3a0878_i64, lw(texts + 2856) as i64, seconds as i64, lw(texts + 2860) as i64, distance_squared.to_bits() as i64]);
                        hudMessage(i, text);
                        soundStart(200, 0);
                    }
                } else {
                    propKill(chr);
                    playerMessage(lw(g::players) + offset, lw(lw((lw(g::cur_language) << 2) + g::languages) + 2852));
                }
            }
            i += 1;
            if i >= lw(g::lvnumplayers) {
                break;
            }
        }
    }
}
