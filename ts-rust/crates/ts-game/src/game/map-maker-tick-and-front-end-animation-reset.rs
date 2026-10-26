//! Functions: the tick of the map maker and the loading of the animations of the front end level.

use crate::*;

/// The tick of the map maker (@ 0x246ae8): the buttons of the pad pick the edit mode and the level of editing, drop, copy, rotate and delete objects, start the dialogs, lasso
/// objects, and the sticks pan and zoom the window.
pub fn mapmakerTick() {
    let frame = Frame::new(224);
    let sp = frame.sp();
    let held = joyBut(0);
    let pressed = joyNewBut(0);
    let flags_before = lw(g::editflags);
    let mut flags = flags_before & -1025;
    sw(g::editflags, flags);
    if (flags_before & 0x4008) != 0 {
        return;
    }
    let front = lw(g::front);
    if (pressed & 2304) != 0 {
        sw(front + 12, 0);
        sw(front + 20, 0);
        mmStartMainMenu();
        return;
    }
    'b2: {
        'b3: {
            let page = lw(g::front);
            if lw(page + 20) == 0 {
                let dialog;
                if lw(g::usedobj) != 0 {
                    dialog = lw(page + 20);
                } else {
                    let mode = lw(g::editmode);
                    if (pressed & 0x2000) == 0 {
                        let mode_now = lw(g::editmode);
                        if (pressed & 0x8000) != 0 {
                            let previous = mode_now - 1;
                            sw(g::editmode, previous);
                            if previous < 0 {
                                sw(g::editmode, 2);
                            }
                        }
                    } else {
                        sw(g::editmode, (mode + 1).wrapping_rem(3));
                    }
                    dialog = lw(page + 20);
                }
                if dialog == 0 {
                    'b6: {
                        mmUpdateMapCost();
                        let mut used = lw(g::usedobj);
                        if (lw(g::editflags) & 64) != 0 {
                            mmFindLassoed();
                            flags = lw(g::editflags);
                            sw(g::highlightobj, 0);
                            if (flags & 64) != 0 {
                                break 'b6;
                            }
                            used = lw(g::usedobj);
                        }
                        if used != 0 {
                            sw(g::highlightobj, 0);
                        } else {
                            let under = mmObjectUnderCursor();
                            sw(g::highlightobj, under);
                            used = lw(g::usedobj);
                        }
                        let new_flags;
                        'b9: {
                            let now = lw(g::editflags);
                            if used != 0 {
                                let ok = mmCheckPlacement();
                                let now = lw(g::editflags);
                                if ok != 0 {
                                    new_flags = now | 4;
                                    break 'b9;
                                }
                                new_flags = now & -5;
                                break 'b9;
                            }
                            new_flags = now & -5;
                        }
                        flags = new_flags;
                        sw(g::editflags, flags);
                    }
                    let level = lw(g::editlevel);
                    if (pressed & 4) == 0 {
                        let level_now = lw(g::editlevel);
                        if (pressed & 1) != 0 {
                            let lower = if level_now >= -2 { level_now - 1 } else { -3 };
                            sw(g::editlevel, lower);
                            sw(g::otherlevel, if lower >= -2 { lower - 1 } else { -3 });
                        }
                    } else {
                        let higher = if 2 >= level { level + 1 } else { 3 };
                        sw(g::editlevel, higher);
                        sw(g::otherlevel, higher - 1);
                    }
                    'b12: {
                        let used = lw(g::usedobj);
                        if (pressed & 16) == 0 {
                            if (pressed & 32) == 0 {
                                let mut button64 = pressed & 64;
                                'b16: {
                                    'b17: {
                                        if (held & 64) == 0 {
                                            let before = flags;
                                            if (flags & 64) == 0 {
                                                button64 = pressed & 64;
                                            } else {
                                                let cleared = flags & -65;
                                                sw(g::editflags, cleared);
                                                if (before & 32) != 0 {
                                                    sw(g::editflags, cleared & -33);
                                                    return;
                                                }
                                                if lw(g::editmode) != 2 {
                                                    mmDoCopy(1);
                                                    if lw(g::usedobj) != 0 {
                                                        break 'b12;
                                                    }
                                                    mmStartMainDialog();
                                                    break 'b12;
                                                }
                                                let colour = mmDialogsGetCurLightColRGB();
                                                let under = mmObjectUnderCursor();
                                                sw(g::highlightobj, under);
                                                if under == 0 {
                                                    mmStartMainDialog();
                                                    break 'b12;
                                                }
                                                mmBackupMap();
                                                let object = lw(g::highlightobj);
                                                let mut list = lw(g::mapobjects);
                                                if (lw(object + 4) & 1) == 0 {
                                                    mmSetTileLight(object, lbu(g::lighttype), lbu(g::lightphase), colour);
                                                    break 'b12;
                                                }
                                                if list == 0 {
                                                    break 'b12;
                                                }
                                                let mut marked = lw(list + 4);
                                                loop {
                                                    let kind = lbu(g::lighttype);
                                                    if (marked & 1) != 0 {
                                                        mmSetTileLight(list, kind, lbu(g::lightphase), colour);
                                                    }
                                                    list = lw(list + 56);
                                                    if list == 0 {
                                                        break 'b12;
                                                    }
                                                    marked = lw(list + 4);
                                                }
                                            }
                                        }
                                        let mode = lw(g::editmode);
                                        if button64 == 0 {
                                            if (pressed & 128) != 0 {
                                                if mode != 1 {
                                                    let used = lw(g::usedobj);
                                                    if mode != 2 {
                                                        if used == 0 {
                                                            mmDoCopy(0);
                                                            mmClearLassoed();
                                                            flags = lw(g::editflags);
                                                        }
                                                    } else {
                                                        let highlighted = lw(g::highlightobj);
                                                        if highlighted != 0 {
                                                            if lw(highlighted) != 1 {
                                                                break 'b16;
                                                            }
                                                            sb(g::lighttype, lbu(highlighted + 28));
                                                            sb(g::lightphase, lbu(highlighted + 29));
                                                            mmSetLightFromRGB(lw(highlighted + 32));
                                                            mmDialogsSetCurLightCol(lw(lw(g::highlightobj) + 32));
                                                            return;
                                                        }
                                                    }
                                                }
                                            }
                                        } else {
                                            let mut used_now = lw(g::usedobj);
                                            if mode == 1 {
                                                let highlighted = lw(g::highlightobj);
                                                if highlighted != 0 {
                                                    used_now = lw(g::usedobj);
                                                    if lw(highlighted) == mode {
                                                        mmStartZoomTileView(highlighted);
                                                        return;
                                                    }
                                                }
                                            }
                                            if used_now == 0 {
                                                let scale_x = lw(g::scale_wtomx);
                                                let x = lh(lw(g::front) + 34) - lw(g::w_x);
                                                sw(g::editflags, flags | 64);
                                                let moved = FMul(x << 16, scale_x);
                                                let scale_y = lw(g::scale_wtomy);
                                                let y = lh(lw(g::front) + 38) - lw(g::w_y);
                                                sw(g::lasso_x, lw(g::m_x) + moved as i32);
                                                let moved = FMul(y << 16, scale_y);
                                                flags = lw(g::editflags);
                                                sw(g::lasso_y, lw(g::m_y) + moved as i32);
                                            } else {
                                                if (flags & 0x400004) != 4 {
                                                    soundStart(142, 0);
                                                } else {
                                                    memset(sp + 16, 0, 40);
                                                    let mut list = lw(g::mapobjects);
                                                    if list != 0 {
                                                        let mut group = lw(list + 52);
                                                        loop {
                                                            mmItemGroupCount(group, sp + 16);
                                                            list = lw(list + 56);
                                                            if list == 0 {
                                                                break;
                                                            }
                                                            group = lw(list + 52);
                                                        }
                                                    }
                                                    let mut held_list = lw(g::usedobj);
                                                    'b23: {
                                                        if held_list != 0 {
                                                            let mut group = lw(held_list + 52);
                                                            loop {
                                                                mmItemGroupCount(group, sp + 16);
                                                                held_list = lw(held_list + 56);
                                                                if held_list == 0 {
                                                                    break 'b23;
                                                                }
                                                                group = lw(held_list + 52);
                                                            }
                                                        }
                                                    }
                                                    let mut counts = sp + 16;
                                                    let mut limits = g::itemgrouplimits;
                                                    let mut n = 0;
                                                    let mut limit_at = limits;
                                                    'b27: {
                                                        loop {
                                                            let over = (lw(limits) < lw(counts)) as i32;
                                                            limits += 4;
                                                            if over != 0 {
                                                                break;
                                                            }
                                                            n += 1;
                                                            counts += 4;
                                                            limit_at += 4;
                                                            if n >= 10 {
                                                                break 'b27;
                                                            }
                                                        }
                                                        let strings = lw((lw(g::cur_language) << 2) + g::languages);
                                                        let name = mmItemGroupName(n);
                                                        let text = vafmt(&[
                                                            0x3a20a8,
                                                            lw(strings + 3488) as i64,
                                                            lw(limit_at) as i64,
                                                            name as i64,
                                                            lw(lw((lw(g::cur_language) << 2) + g::languages) + 3660) as i64,
                                                        ]);
                                                        let front_now = lw(g::front);
                                                        let mouse_x = lw(front_now + 32);
                                                        sw(g::savemouse_y, lw(front_now + 36));
                                                        sw(g::savemouse_x, mouse_x);
                                                        mmStartMessage(text, 0x6464647f, 0x246a40);
                                                    }
                                                    flags = lw(g::editflags);
                                                    if n != 10 {
                                                        break 'b17;
                                                    }
                                                    let pasted = mmDoPaste();
                                                    flags = lw(g::editflags);
                                                    if pasted != 0 {
                                                        break 'b17;
                                                    }
                                                    mmStartMessage(lw(lw((lw(g::cur_language) << 2) + g::languages) + 3484), 0x7f7f3c7f, 0);
                                                }
                                                flags = lw(g::editflags);
                                            }
                                        }
                                    }
                                }
                                let page = lw(g::front);
                                if lw(page + 20) == 0 {
                                    let top = (lw(g::w_y) << 16) + 0x200000;
                                    let left = (lw(g::w_x) << 16) + 0x400000;
                                    let right = left + (lw(g::w_w) << 16) - 0x800000;
                                    sw(sp + 4, top + (lw(g::w_h) << 16) - 0x400000);
                                    sw(sp + 12, right);
                                    sw(sp, top);
                                    sw(sp + 8, left);
                                    sw(g::editflags, flags & -4);
                                    flags &= -4;
                                    break 'b2;
                                }
                                break 'b3;
                            }
                            // the button 32: turn the held object, or the lights of the thing under the cursor
                            if used != 0 {
                                if lw(used) != 1 {
                                    return;
                                }
                                let mut object = used;
                                let mut phase = lw(object + 24);
                                loop {
                                    let a = lw(object + 12);
                                    let next_phase = (phase + 1) & 3;
                                    let b = 0 - lw(object + 16);
                                    sw(object + 24, next_phase);
                                    let tile = lw(lw(object + (next_phase << 2) + 36));
                                    sw(object + 16, a);
                                    sw(object + 12, b - (tile << 21));
                                    object = lw(object + 56);
                                    if object == 0 {
                                        return;
                                    }
                                    phase = lw(object + 24);
                                }
                            } else {
                                let highlighted = lw(g::highlightobj);
                                let mode = lw(g::editmode);
                                if highlighted == 0 {
                                    return;
                                }
                                if mode == 1 {
                                    return;
                                }
                                let marked = lw(highlighted + 4);
                                if mode != 2 {
                                    let toggled = marked ^ 1;
                                    sw(highlighted + 4, toggled);
                                    let count = lw(g::lassocount);
                                    if (toggled & 1) == 0 {
                                        sw(g::lassocount, count - 1);
                                        return;
                                    }
                                    sw(g::lassocount, count + 1);
                                    return;
                                }
                                if (marked & 1) == 0 {
                                    let next = lbu(highlighted + 29) + 1;
                                    sb(highlighted + 29, next);
                                    if ((next & 255) as u32) < (12 as u32) {
                                        return;
                                    }
                                    sb(lw(g::highlightobj) + 29, 0);
                                    return;
                                }
                                let mut list = lw(g::mapobjects);
                                if list == 0 {
                                    return;
                                }
                                let mut flag = lw(list + 4);
                                loop {
                                    if (flag & 1) == 0 {
                                        list = lw(list + 56);
                                    } else {
                                        let next = lbu(list + 29) + 1;
                                        sb(list + 29, next);
                                        if ((next & 255) as u32) >= (12 as u32) {
                                            sb(list + 29, 0);
                                        }
                                        list = lw(list + 56);
                                    }
                                    if list == 0 {
                                        return;
                                    }
                                    flag = lw(list + 4);
                                }
                            }
                        } else {
                            // the button 16: delete
                            if used != 0 {
                                mmFreeObject(used);
                                sw(g::usedobj, 0);
                                return;
                            }
                            let highlighted = lw(g::highlightobj);
                            let mode;
                            if highlighted != 0 {
                                mode = lw(g::editmode);
                            } else {
                                mode = lw(g::editmode);
                                if lw(g::lassocount) != 0 {
                                    break 'b12;
                                }
                            }
                            if mode == 1 {
                                return;
                            }
                            if mode != 2 {
                                if highlighted == 0 {
                                    return;
                                }
                                if (lw(highlighted + 4) & 1) == 0 {
                                    mmBackupMap();
                                    mmFreeSingleObject(lw(g::highlightobj));
                                    return;
                                }
                                mmDeleteAllMarked();
                                return;
                            }
                            if highlighted == 0 {
                                return;
                            }
                            let mut list = lw(g::mapobjects);
                            if (lw(highlighted + 4) & 1) == 0 {
                                sb(highlighted + 28, 0);
                                let object = lw(g::highlightobj);
                                sw(highlighted + 32, 0x7f7f7f80);
                                sb(object + 29, 0);
                                return;
                            }
                            if list == 0 {
                                return;
                            }
                            let mut marked = lw(list + 4);
                            loop {
                                if (marked & 1) == 0 {
                                    list = lw(list + 56);
                                } else {
                                    sw(list + 32, 0x7f7f7f80);
                                    sb(list + 28, 0);
                                    sb(list + 29, 0);
                                    list = lw(list + 56);
                                }
                                if list == 0 {
                                    return;
                                }
                                marked = lw(list + 4);
                            }
                        }
                    }
                    mmClearLassoed();
                    return;
                }
            }
            // the margins of the page are kept
            let margins = ld(g::pagemargins + 8);
            sd(sp, ld(g::pagemargins));
            sd(sp + 8, margins);
        }
    }
    let page = lw(g::front);
    let in_page = lw(page + 20);
    sw(g::editflags, flags & -4);
    let mut zoom_step = 0;
    if in_page == 0 {
        let stick_x;
        let stick_y;
        'b36: {
            let x = lw(page + 32);
            let right = lw(sp + 12);
            if x >= lw(sp + 8) {
                let top = lw(sp);
                if right >= x {
                    let y = lw(page + 36);
                    let bottom = lw(sp + 4);
                    if y >= top {
                        if bottom >= y {
                            joySetCentreZone(0, 0.5_f32);
                            let first = joyGetAxis(0, 2);
                            let second = joyGetAxis(0, 3);
                            joySetCentreZone(0, 0.25_f32);
                            stick_x = ps2float::add(first, first);
                            stick_y = ps2float::add(second, second);
                            break 'b36;
                        }
                    }
                }
            }
            stick_x = ps2float::mul(joyGetAxis(0, 0), 1.5_f32);
            stick_y = ps2float::mul(joyGetAxis(0, 1), 1.5_f32);
        }
        let pan_scale = lwc1(0x3aca54);
        let pan_x = ps2float::mul(stick_x, pan_scale) as i32;
        let pan_y = ps2float::mul(stick_y, pan_scale) as i32;
        let page_now;
        'b38: {
            if pan_x == 0 {
                if pan_y == 0 {
                    page_now = lw(g::front);
                    break 'b38;
                }
            }
            'b40: {
                mmPanWindow(pan_x, 0 - pan_y);
                sw(g::editflags, lw(g::editflags) | 2);
                let direction;
                if pan_x >= 0 {
                    let mut value = 1;
                    if pan_x <= 0 {
                        value = 3;
                        if pan_y >= 0 {
                            value = 4;
                            if pan_y <= 0 {
                                break 'b40;
                            }
                        }
                    }
                    direction = value;
                } else {
                    direction = 2;
                }
                sb(g::panning, direction);
            }
            page_now = lw(g::front);
        }
        if lw(page_now + 20) == 0 {
            if (held & 2) == 0 {
                zoom_step = if (held & 8) == 0 { 0 } else { 0xfae1 };
            } else {
                zoom_step = 0x1051e;
            }
        }
    }
    mmClampMouse(sp);
    let page_now = lw(g::front);
    if zoom_step != 0 {
        let scale_x = lw(g::scale_wtomx);
        let x = lw(page_now + 32) - (lw(g::w_x) << 16);
        sw(g::editflags, lw(g::editflags) | 1);
        let moved_x = FMul(x, scale_x);
        let scale_y = lw(g::scale_wtomy);
        let y = lw(lw(g::front) + 36) - (lw(g::w_y) << 16);
        let target_x = lw(g::m_x) + moved_x as i32;
        let moved_y = FMul(y, scale_y);
        let map_y = lw(g::m_y);
        let target_y = map_y + moved_y as i32;
        let centre_y = map_y + (lw(g::m_h) >> 1);
        let centre_x = lw(g::m_x) + (lw(g::m_w) >> 1);
        let window_w = lw(g::w_w);
        sw(g::m_x, centre_x + ((target_x - centre_x) >> 4));
        let wide = window_w << 16;
        sw(g::m_y, centre_y + ((target_y - centre_y) >> 4));
        let min_zoom = FDiv(wide, 0x7800000);
        let max_zoom = FDiv(lw(g::w_w) << 16, 0x1000000);
        let new_x = FMul(lw(g::scale_mtowx), zoom_step);
        let old_y = lw(g::scale_mtowy);
        sw(g::scale_mtowx, new_x as i32);
        let new_y = FMul(old_y, zoom_step);
        let window_h = lw(g::w_h);
        let window_w = lw(g::w_w);
        sw(g::scale_mtowy, new_y as i32);
        let ratio = FDiv(window_h << 17, window_w << 16);
        let current = lw(g::scale_mtowx);
        let height = lw(g::w_h);
        'b43: {
            let limit;
            if (current as i64) >= min_zoom {
                if max_zoom >= current as i64 {
                    break 'b43;
                }
                limit = 0x1000000;
                sw(g::scale_mtowx, max_zoom as i32);
            } else {
                limit = 0x7800000;
                sw(g::scale_mtowx, min_zoom as i32);
            }
            let product = FMul(limit, ratio as i32);
            let new_y = FDiv(height << 16, product as i32);
            sw(g::scale_mtowy, new_y as i32);
        }
        let scale_x = lw(g::scale_mtowx);
        let inverse_x = FDiv(0x10000, scale_x);
        let scale_y = lw(g::scale_mtowy);
        sw(g::scale_wtomx, inverse_x as i32);
        let inverse_y = FDiv(0x10000, scale_y);
        let window_w = lw(g::w_w);
        let wtomx = lw(g::scale_wtomx);
        sw(g::scale_wtomy, inverse_y as i32);
        let map_w = FMul(window_w << 16, wtomx);
        let window_h = lw(g::w_h);
        let wtomy = lw(g::scale_wtomy);
        sw(g::m_w, map_w as i32);
        let map_h = FMul(window_h << 16, wtomy);
        let width = lw(g::m_w);
        let new_top = lw(g::m_y) - ((map_h as i32 + ((map_h as i32 as u32) >> 31) as i32) >> 1);
        let new_left = lw(g::m_x) - ((width + ((width as u32) >> 31) as i32) >> 1);
        let mtowx = lw(g::scale_mtowx);
        let from_left = target_x - new_left;
        sw(g::m_x2, new_left + width);
        sw(g::m_y2, new_top + map_h as i32);
        sw(g::m_h, map_h as i32);
        sw(g::m_x, new_left);
        sw(g::m_y, new_top);
        let screen_x = FMul(from_left, mtowx);
        let top_now = lw(g::m_y);
        let mtowy = lw(g::scale_mtowy);
        sw(lw(g::front) + 32, (lw(g::w_x) << 16) + screen_x as i32);
        let from_top = target_y - top_now;
        let screen_y = FMul(from_top, mtowy);
        sw(lw(g::front) + 36, (lw(g::w_y) << 16) + screen_y as i32);
    }
    mmClampView();
    let mut page_last = lw(g::front);
    if lw(page_last + 20) == 0 {
        let highlighted = lw(g::highlightobj);
        if highlighted != 0 {
            if lw(highlighted) == 1 {
                let mut angle = ps2float::sub(0.0_f32, lwc1((lw(highlighted + 24) << 2) + g::tilerotang));
                if angle < 0.0_f32 {
                    angle = ps2float::add(angle, 3.6e+02_f32);
                }
                zoomtileWindow(544, 190, 172);
                let number = ((lw(lw(g::highlightobj) + 36) - g::tileset1).wrapping_mul(0x4f72c235)) >> 2;
                sw(g::zoomtilenum, number);
                let distance = zoomtileSizeToDist(number);
                let object = lw(g::highlightobj);
                swc1(g::zoomtileroty, angle);
                swc1(g::zoomtiledist, ps2float::add(distance, 0.0_f32));
                swc1(g::zoomtiletilt, 47.0_f32);
                sw(g::zoomtileflags, 8);
                sw(g::zoomtileobj, object);
                zoomtileTick();
                page_last = lw(g::front);
            }
        }
    }
    sw(g::button_text, 0);
    let in_page_last = lw(page_last + 20);
    sw(g::button_text + 4, 0);
    sw(g::mmcursor, 16);
    sw(g::button_text + 12, 0);
    sw(g::button_text + 8, 0);
    if in_page_last == 0 {
        mmMapViewHelp();
    }
}

/// The animations loaded for the front end level: their places in memory and their numbers (@ 0x215d38).
const FRONT_END_ANIMATIONS: [(i32, i32); 96] = [
    (0x39a4c0, 617),
    (0x39a4e0, 618),
    (0x39a500, 620),
    (0x39a530, 622),
    (0x39a558, 623),
    (0x39a580, 624),
    (0x39a5b0, 625),
    (0x39a5d8, 626),
    (0x39a608, 627),
    (0x39a630, 629),
    (0x39a658, 631),
    (0x39a680, 633),
    (0x39a6b0, 634),
    (0x39a6d8, 654),
    (0x39a700, 635),
    (0x39a728, 636),
    (0x39a758, 637),
    (0x39a780, 638),
    (0x39a7a8, 639),
    (0x39a7d8, 640),
    (0x39a800, 642),
    (0x39a828, 644),
    (0x39a850, 621),
    (0x39a880, 645),
    (0x39a8a8, 646),
    (0x39a8d8, 647),
    (0x39a908, 648),
    (0x39a938, 649),
    (0x39a960, 650),
    (0x39a988, 651),
    (0x39a9b8, 652),
    (0x39a9e8, 653),
    (0x39aa18, 655),
    (0x39aa40, 656),
    (0x39aa68, 657),
    (0x39aa90, 658),
    (0x39aab8, 659),
    (0x39aae8, 661),
    (0x39ab10, 662),
    (0x39ab38, 663),
    (0x39ab60, 665),
    (0x39ab88, 666),
    (0x39abb0, 667),
    (0x39abd8, 668),
    (0x39ac00, 669),
    (0x39ac30, 670),
    (0x39ac58, 672),
    (0x39ac88, 674),
    (0x39acb0, 675),
    (0x39acd8, 676),
    (0x39ad08, 677),
    (0x39ad38, 679),
    (0x39ad68, 681),
    (0x39ad98, 682),
    (0x39adc0, 683),
    (0x39adf0, 684),
    (0x39ae20, 685),
    (0x39ae50, 687),
    (0x39ae78, 688),
    (0x39aea8, 689),
    (0x39aed8, 690),
    (0x39af08, 691),
    (0x39af30, 692),
    (0x39af58, 693),
    (0x39af88, 695),
    (0x39afb8, 696),
    (0x39afe8, 697),
    (0x39b018, 698),
    (0x39b048, 699),
    (0x39b078, 700),
    (0x39b0a8, 701),
    (0x39b0d0, 702),
    (0x39b0f8, 703),
    (0x39b128, 704),
    (0x39b158, 705),
    (0x39b188, 706),
    (0x39b1b8, 707),
    (0x39b1e8, 708),
    (0x39b218, 709),
    (0x39b240, 710),
    (0x39b278, 694),
    (0x39b2a8, 711),
    (0x39b2d8, 671),
    (0x39b308, 678),
    (0x39b338, 673),
    (0x39b368, 686),
    (0x39b398, 680),
    (0x39b3c8, 712),
    (0x39b400, 628),
    (0x39b428, 630),
    (0x39b458, 664),
    (0x39b488, 632),
    (0x39b4b8, 641),
    (0x39b4e8, 660),
    (0x39b518, 713),
    (0x39b548, 714),
];

/// The kinds set for the front end animations: their numbers and kinds.
const FRONT_END_ANIMATION_PROPS: [(i32, i32); 104] = [
    (617, 3),
    (618, 3),
    (620, 2),
    (622, 2),
    (623, 2),
    (624, 2),
    (625, 2),
    (626, 2),
    (627, 2),
    (628, 6),
    (629, 2),
    (630, 2),
    (631, 2),
    (632, 2),
    (633, 2),
    (634, 2),
    (635, 2),
    (636, 2),
    (637, 2),
    (638, 2),
    (639, 2),
    (640, 2),
    (642, 2),
    (644, 3),
    (621, 3),
    (645, 3),
    (646, 3),
    (647, 3),
    (648, 3),
    (649, 3),
    (650, 3),
    (651, 3),
    (652, 3),
    (653, 3),
    (654, 3),
    (655, 3),
    (657, 3),
    (658, 3),
    (659, 3),
    (661, 3),
    (662, 3),
    (663, 3),
    (664, 3),
    (665, 3),
    (666, 2),
    (667, 2),
    (668, 2),
    (669, 2),
    (670, 2),
    (671, 2),
    (672, 2),
    (673, 2),
    (674, 2),
    (675, 3),
    (676, 3),
    (677, 3),
    (678, 3),
    (680, 3),
    (681, 3),
    (682, 3),
    (683, 3),
    (684, 3),
    (687, 3),
    (688, 2),
    (689, 3),
    (690, 3),
    (691, 2),
    (692, 2),
    (693, 3),
    (695, 2),
    (696, 3),
    (697, 3),
    (698, 2),
    (699, 3),
    (700, 3),
    (701, 2),
    (702, 3),
    (703, 3),
    (704, 2),
    (705, 3),
    (706, 3),
    (707, 3),
    (709, 3),
    (710, 3),
    (679, 3),
    (685, 3),
    (694, 2),
    (711, 3),
    (671, 2),
    (678, 3),
    (673, 2),
    (686, 3),
    (680, 3),
    (712, 2),
    (628, 2),
    (630, 2),
    (664, 3),
    (632, 2),
    (641, 2),
    (660, 3),
    (713, 2),
    (656, 3),
    (714, 2),
    (708, 3),
];

/// Load the animations of the front end level (the level 102) and set the kind of each (@ 0x215d38).
pub fn animReset() {
    if lw(g::lvlevelnum) != 102 {
        return;
    }
    for (at, number) in FRONT_END_ANIMATIONS {
        loadAnimation(at, number);
    }
    for (number, kind) in FRONT_END_ANIMATION_PROPS {
        setAnimProp(number, kind);
    }
}
