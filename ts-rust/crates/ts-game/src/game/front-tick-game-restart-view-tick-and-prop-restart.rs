//! The tick of the front end, restarting a game, the free camera of the view test, and restarting the props.

use crate::*;

/// The tick of the front end (`front` is one of the four definitions in `frontdefs`, 44 bytes each; its flags are at +12, the pages at +4, the boxes at +8, the page count at +20,
/// a page waiting to be added at +24 and its place at +28; a page is 16 bytes: definition, current box +4, boxes +8, box count +12): add the waiting page; for the first definition
/// run the mouse; then run the controls and call the tick function of each page from the top one down (after one the controls are cleared and flags 61 of the front end taken off), and
/// give the focus tick to the top page when flag 1 of the front end was set at the start of the tick and still is, and the page has a current box (@ 0x22cb58).
pub fn frontTick() {
    let mut front = lw(g::front);
    let index = (front - g::frontdefs).wrapping_mul(-0x45d1745d) >> 2;
    let waiting = lw(front + 24);
    let flag_clear = (lw(front + 12) ^ 1) & 1;
    if waiting != 0 {
        let pages = lw(front + 4);
        let page = (lw(front + 28) << 4) + pages;
        sw(page + 12, 0);
        sw(page, waiting);
        let place = lw(front + 28);
        let count = lw(front + 20);
        let start = lhu(lw(front + 24) + 8);
        sw(page + 8, 0);
        sh((place << 4) + pages + 4, start);
        if place == count {
            sw(front + 20, place + 1);
        }
        sw(front + 24, 0);
    }
    let count;
    if index != 0 {
        count = lw(front + 20);
    } else {
        frontMouseTick();
        count = lw(lw(g::front) + 20);
    }
    if count == 0 {
        return;
    }
    frontControlsTick(index);
    front = lw(g::front);
    let mut page_number = lw(front + 20) - 1;
    let mut boxes_before = 0;
    let pages_now;
    if page_number < 0 {
        pages_now = lw(front + 20);
    } else {
        loop {
            let pages = lw(front + 4);
            let offset = boxes_before * 24;
            let boxes = lw(front + 8);
            let page = pages + (page_number << 4);
            let definition = lw(page);
            sw(page + 12, 0);
            let tick = lw(definition);
            sw(page + 8, boxes + boxes_before * 24);
            if tick != 0 {
                call_indirect(tick, [page, pages, offset, 0, front, 0, 0, 0], [0.0; 8]);
                joyClearAll(index);
                front = lw(g::front);
                sw(front + 12, lw(front + 12) & -61);
            }
            page_number -= 1;
            if lw(front + 20) == 0 {
                return;
            }
            boxes_before += lw(page + 12);
            if page_number < 0 {
                break;
            }
        }
        pages_now = lw(front + 20);
    }
    if pages_now > 0 && flag_clear == 0 {
        let last = pages_now << 4;
        if (lw(front + 12) & 1) != 0 {
            let pages = lw(front + 4);
            if lw(last + pages - 4) != 0 {
                frontFocusTick(pages + (last - 16));
            }
        }
    }
}

/// Restart a game: the game state is 2 and the frame 0, the time unpaused; the game flags (+80 of the game parameters) keep only their low 20 bits, the 5 bytes at +100 are marked
/// for the players listed at +16 (the count at +12), and the count of the players' bytes at +49 of the team sizes (+8 words from the first) is put at +108 (@ 0x21e8c8).
/// (The original also counts down through the player and team counts in three empty loops, which do nothing and are left out.)
pub fn gameRestart() {
    let parms = g::curgameparms;
    sw(g::gamestate, 2);
    let marks = parms + 100;
    sw(g::gameframe, 0);
    timePause(0);
    let flags = lw(parms + 80);
    sw(parms + 84, 0);
    sw(parms + 80, flags & 0xfffff);
    memset(marks, 0, 5);
    let mut i = 0;
    if lw(parms + 12) > 0 {
        let players = parms + 16;
        loop {
            sb(lbu(i + players) + marks - 1, 1);
            i += 1;
            if i >= lw(parms + 12) {
                break;
            }
        }
    }
    let base = g::curgameparms;
    let mut team = lw(g::curgameparms);
    let end = team + lw(base + 8);
    sw(base + 108, 0);
    if team < end {
        loop {
            let size = lbu(team + (base + 49));
            team += 1;
            sw(base + 108, lw(base + 108) + size);
            if team >= end {
                break;
            }
        }
    }
}

/// The free camera of the view test: the left stick turns the view (angle in x, and in y by -4 times the stick), the right stick moves the camera along the view's direction and
/// up/down by the buttons 64 and 16; the view matrix (64 bytes of the debug memory) is made by looking from the camera's place to a point one view-direction on (@ 0x20c100).
pub fn viewTick() {
    let frame = Frame::new(112);
    let sp = frame.sp();
    memset(sp, 0, 12);
    let direction = g::view_camdir;
    let position = g::view_campos;
    let to_radians = lwc1(0x3ac810);
    sd(sp + 16, ld(0x39a0c8));
    sw(sp + 24, lw(0x39a0c8 + 8));
    let buttons = joyBut(0);
    let half_turn = 1.8e+02_f32;
    sw(g::viewmatrix, memdbAlloc(64));
    swc1(g::view_angx, lwc1(g::view_angx) + joyGetAxis(0, 0) * 4.0_f32);
    let stick_y = joyGetAxis(0, 3);
    let angle_y = lwc1(g::view_angy);
    sw(g::view_camdir, 0);
    swc1(direction + 8, 1.0_f32);
    let angle_y = angle_y + stick_y * -4.0_f32;
    sw(direction + 4, 0);
    swc1(g::view_angy, angle_y);
    matrixRotX(sp + 32, (angle_y * to_radians).pdiv(half_turn));
    matrixVecMulAligned(&[(sp + 32) as i64, direction as i64]);
    matrixRotY(sp + 32, (lwc1(g::view_angx) * to_radians).pdiv(half_turn));
    matrixVecMulAligned(&[(sp + 32) as i64, direction as i64]);
    let forward = joyGetAxis(0, 1);
    swc1(g::view_campos, lwc1(g::view_campos) + lwc1(g::view_camdir) * forward);
    let forward = joyGetAxis(0, 1);
    let dz = lwc1(direction + 8);
    swc1(position + 8, lwc1(position + 8) + dz * forward);
    if (buttons & 64) != 0 {
        swc1(position + 4, lwc1(position + 4) + lwc1(0x3ac814));
    }
    if (buttons & 16) != 0 {
        swc1(position + 4, lwc1(position + 4) - lwc1(0x3ac818));
    }
    let dy = lwc1(direction + 4);
    let target_z = lwc1(position + 8) + dz;
    let matrix = lw(g::viewmatrix);
    let target_x = lwc1(g::view_campos) + lwc1(g::view_camdir);
    swc1(sp + 4, lwc1(position + 4) + dy);
    swc1(sp + 8, target_z);
    swc1(sp, target_x);
    matrixLookAt(matrix, position, sp, sp + 16);
}

/// Restart the props: all the counts are cleared, and the tables of the pick-ups, corpses, props, doors, cars, remotes, character remotes, heads, pick-up props, the heads' use and
/// the graphics pointers are cleared; the last reaction frame is cleared and the prop wall memory restarted (@ 0x266ec0).
pub fn propRestart() {
    sw(g::num_props, 0);
    sw(g::num_deleted_props, 0);
    sw(g::numdoors, 0);
    sw(g::numcars, 0);
    sw(g::numremotes, 0);
    sw(g::numpickups, 0);
    sw(g::numgfxpointers, 0);
    sw(0x3b00e8, 0);
    sw(0x3b00ec, 0);
    sw(0x3b00f0, 0);
    sw(0x3b00f4, 0);
    memset(g::tmpPickupData, 0, 2400);
    memset(g::tmpPickupDataUsed, 0, 80);
    memset(g::tmpPickupDataProp, 0, 80);
    memset(g::tmpPickupDataFramenum, 0, 80);
    memset(g::corpseData, 0, 480);
    memset(g::corpseDataUsed, 0, 120);
    memset(lw(g::levelprops), 0, lw(g::max_props) * 592);
    memset(lw(g::leveldoors), 0, 608);
    memset(lw(g::levelcars), 0, 560);
    memset(lw(g::levelremotes), 0, 3504);
    memset(lw(g::levelchrremotes), 0, 2704);
    memset(lw(g::levelheads), 0, 1680);
    memset(lw(g::levelpickups), 0, 0x3840);
    memset(g::headused, 0, 60);
    memset(lw(g::levelgfxpointers), 0, 0x12c0);
    sw(g::lastreactionframenum, 0);
    propRestartWallMem();
}
