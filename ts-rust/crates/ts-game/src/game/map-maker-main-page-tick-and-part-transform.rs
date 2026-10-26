//! The tick of the map maker's main page and the move of the points of a part into the space of its bones.

use crate::*;

/// The tick of the map maker's main page `page`: with button 16 the dialogs are left; else the page margins are set, the edit mode (0 to 2) is changed by buttons 0x2000 (next) and
/// 0x8000 (back) and the tile page by button 32; in mode 0 the tile under the cursor (the page's item number less 200) is shown zoomed; a push of button 64 on the item takes the
/// light palette (2), the next tile page (5), a mode (10 to 12), a new tile (200 up, which also leaves the dialogs), a colour (64 to 79; the colour that is already the current one
/// goes to the colour page) or a light type (100 to 110); at the end the dialog is made (@ 0x252c50).
pub fn mapmakermain_pageTick(page: i32) {
    if (joyNewBut(0) & 16) != 0 {
        exitDialogs();
        return;
    }
    sw(g::pagemargins, 0x1b0000);
    sw(g::pagemargins + 12, 0x24d0000);
    sw(g::pagemargins + 4, 0xc40000);
    sw(g::pagemargins + 8, g::sounds + 7808);
    let pad = joyNewBut(0);
    let before = lw(g::editmode);
    let mut mode;
    if (pad & 0x2000) != 0 {
        mode = (before + 1).wrapping_rem(3);
        sw(g::editmode, mode);
    } else {
        let pad = joyNewBut(0);
        let before = lw(g::editmode);
        if (pad & 0x8000) != 0 {
            mode = before - 1;
            sw(g::editmode, mode);
            if mode < 0 {
                mode = 2;
                sw(g::editmode, mode);
            }
        } else {
            let pad = joyNewBut(0);
            mode = lw(g::editmode);
            if (pad & 32) != 0 {
                cycleTilePage();
                mode = lw(g::editmode);
            }
        }
    }
    if mode != 0 {
        sw(g::zoomtilenum, -1);
    } else {
        let front = lw(g::front);
        let tile = lhu((lw(front + 20) << 4) + lw(front + 4) - 12) as i32 - 200;
        let mut shown = lw(g::zoomtilenum);
        if (tile as u32) < 21 {
            let distance = zoomtileSizeToDist(tile);
            swc1(g::zoomtiledist, distance);
            sw(g::zoomtileflags, 4);
            rotateMiniTile();
            shown = lw(g::zoomtilenum);
        }
        if tile != shown {
            sw(g::zoomtilenum, tile);
            zoomtileWindow(449, 103, 282);
            zoomtileWindowDest(449, 103, 387, 7);
        }
        sw(g::zoomtileobj, 0);
        zoomtileWindowTick();
        zoomtileTick();
    }
    if (joyNewBut(0) & 64) != 0 {
        let item = lhu(page + 4) as i32;
        if item == 2 {
            for k in 0..8 {
                sd(0x1fc4040 + k * 8, ld(g::lightpalette + k * 8));
            }
        } else if item == 5 {
            cycleTilePage();
        } else {
            let mode_item = item - 10;
            let tiles = lw(g::numtiles);
            if (mode_item as u32) >= 3 {
                let number = item - 200;
                if (number as u32) < (tiles as u32) {
                    let tile = lw((number << 2) + g::tiles);
                    let y = 0 - (lw(tile + 4) << 20);
                    let x = 0 - (lw(tile) << 20);
                    let made = mmNewTile(number, x, y, 0, 0);
                    sw(g::usedobj, made);
                    exitDialogs();
                    return;
                }
                let colour = item - 64;
                let current = lw(g::curlightcol);
                if (colour as u32) >= 16 {
                    let light = item - 100;
                    if (light as u32) < 11 {
                        sb(g::lighttype, lbu((light << 2) + (g::lighttypes + 2)));
                    }
                } else {
                    if current == colour {
                        frontPageSet(1, g::mapmakercolour_page);
                        joyClearAll(0);
                        return;
                    }
                    mmSetLightFromRGB(lw((colour << 2) + 0x1fc4040));
                    sw(g::curlightcol, lhu(page + 4) as i32 - 64);
                }
            } else {
                sw(g::editmode, mode_item);
            }
        }
    }
    let made = makedialog(page, lw(page + 8));
    sw(page + 12, made);
}

/// Move the points of the lists of a part `part` of the model `model` into the space of the bone that moves them: `flag` 0 takes the list at +8 (with the normals at +20 of the list
/// of the part), `flag` 1 the one at +12 (+44); each point is made relative to the matrix of the bone (the point less the matrix's place, turned by the matrix's transpose) and the
/// normals are turned. A part of kind 1 to 3 has two bones (+2 and +3) and each point is taken by whichever bone the table at +16 of the list names, the table updated to 0, 1 or
/// 2 (@ 0x263110).
pub fn transformPart(model: i32, part: i32, flag: i32) {
    let frame = Frame::new(448);
    let sp = frame.sp();
    let mut list;
    if flag != 0 {
        let found = lw(part + 12);
        if found == 0 {
            return;
        }
        list = found;
        sw(sp + 280, part + 44);
    } else {
        let found = lw(part + 8);
        list = found;
        if found == 0 {
            return;
        }
        sw(sp + 280, part + 20);
    }
    if ((lbu(part) as i32 - 1) as u32) >= 3 {
        anim_LoadMatrix(lb(part + 1));
        let view = anim_GetModelview();
        sceVu0CopyMatrix(sp, view);
        if lw(list + 20) >= 0 {
            let mut first = lw(list + 12);
            loop {
                let lists = lw(sp + 280);
                let mut i = 0;
                let offset = first * 12;
                let normals = lw(lists + 20);
                let mut point = lw(lists + 4) + offset;
                let mut normal = if normals != 0 { normals + offset } else { 0 };
                if lw(list + 16) == 0 {
                    list += 24;
                } else {
                    let next = list + 24;
                    loop {
                        sceVu0TransposeMatrix(sp + 192, sp);
                        let y = lwc1(sp + 52);
                        let z = lwc1(sp + 56);
                        swc1(sp + 256, ps2float::sub(lwc1(point), lwc1(sp + 48)));
                        swc1(sp + 260, ps2float::sub(lwc1(point + 4), y));
                        swc1(sp + 264, ps2float::sub(lwc1(point + 8), z));
                        matrixVecRotAligned(&[(sp + 192) as i64, (sp + 256) as i64]);
                        swc1(point, lwc1(sp + 256));
                        swc1(point + 4, lwc1(sp + 260));
                        swc1(point + 8, lwc1(sp + 264));
                        point += 12;
                        if normal != 0 {
                            matrixVecRot(sp + 192, normal);
                            normal += 12;
                        }
                        i += 1;
                        if (i as u32) >= (lw(list + 16) as u32) {
                            break;
                        }
                    }
                    list = next;
                }
                if lw(list + 20) < 0 {
                    return;
                }
                first = lw(list + 12);
            }
        }
    } else {
        let skeleton = lw(model);
        let bones = skeleton - lw(skeleton) * 80;
        let bone_a = bones + lb(part + 2) * 80;
        let bone_b = bones + lb(part + 3) * 80;
        sw(sp + 272, bone_a);
        sw(sp + 276, bone_b);
        let matrix_a = lb(bone_a + 1);
        let mut bone_now = lw(sp + 276);
        if matrix_a >= 0 {
            anim_LoadMatrix(matrix_a);
            let view = anim_GetModelview();
            sceVu0CopyMatrix(sp + 64, view);
            bone_now = lw(sp + 276);
        }
        let matrix_b = lb(bone_now + 1);
        let matrix_c;
        if matrix_b < 0 {
            matrix_c = lb(part + 1);
        } else {
            anim_LoadMatrix(matrix_b);
            let view = anim_GetModelview();
            sceVu0CopyMatrix(sp + 128, view);
            matrix_c = lb(part + 1);
        }
        let left;
        if matrix_c < 0 {
            left = lw(list + 20);
        } else {
            anim_LoadMatrix(matrix_c);
            let view = anim_GetModelview();
            sceVu0CopyMatrix(sp, view);
            left = lw(list + 20);
        }
        if left >= 0 {
            let mut first = lw(list + 12);
            loop {
                let lists = lw(sp + 280);
                let offset = first * 12;
                let normals = lw(lists + 20);
                let mut i = 0;
                let mut point = lw(lists + 4) + offset;
                let mut table = lw(lists + 16) + (first << 2);
                let mut normal = if normals != 0 { normals + offset } else { 0 };
                if lw(list + 16) == 0 {
                    list += 24;
                } else {
                    let next = list + 24;
                    loop {
                        let which = lbu(table) as i32;
                        let bone_now = lw(sp + 276);
                        let matrix;
                        if which != lb(lw(sp + 272) + 1) as i32 {
                            let mark;
                            if which != lb(bone_now + 1) as i32 {
                                mark = 2;
                                matrix = sp;
                            } else {
                                mark = 1;
                                matrix = sp + 128;
                            }
                            sb(table, mark);
                        } else {
                            sb(table, 0);
                            matrix = sp + 64;
                        }
                        sceVu0TransposeMatrix(sp + 192, matrix);
                        swc1(sp + 256, ps2float::sub(lwc1(point), lwc1(matrix + 48)));
                        swc1(sp + 260, ps2float::sub(lwc1(point + 4), lwc1(matrix + 52)));
                        swc1(sp + 264, ps2float::sub(lwc1(point + 8), lwc1(matrix + 56)));
                        matrixVecRotAligned(&[(sp + 192) as i64, (sp + 256) as i64]);
                        swc1(point, lwc1(sp + 256));
                        swc1(point + 4, lwc1(sp + 260));
                        swc1(point + 8, lwc1(sp + 264));
                        point += 12;
                        if normal != 0 {
                            matrixVecRot(sp + 192, normal);
                            normal += 12;
                        }
                        i += 1;
                        table += 4;
                        if (i as u32) >= (lw(list + 16) as u32) {
                            break;
                        }
                    }
                    list = next;
                }
                if lw(list + 20) < 0 {
                    return;
                }
                first = lw(list + 12);
            }
        }
    }
}
