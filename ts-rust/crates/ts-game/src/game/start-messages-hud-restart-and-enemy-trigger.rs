//! Starting the message boxes, restarting the HUD, taking the guns of a player, and the test of whether an enemy is triggered.

use crate::*;

/// Copy the text `text` into the buffer `buffer` (at most 1023 characters) and end it with a zero.
fn copy_message(buffer: i32, text: i32) {
    let mut i = 0;
    if lbu(text) != 0 {
        loop {
            sb(buffer + i, lbu(text + i));
            i += 1;
            let more = i < 1023;
            if lbu(text + i) == 0 {
                break;
            }
            if !more {
                break;
            }
        }
    }
    sb(buffer + i, 0);
}

/// Half of a number, rounded towards 0.
fn half(value: i32) -> i32 {
    (value + ((value as u32) >> 31) as i32) >> 1
}

/// Start a message: keep the text in the buffer at 0x1fb2528 (kept `kind` at 0x3afe2c and `action` at 0x3afe30), size it in the current font, and open a zoom box (kind 0, mode
/// 0x2880) of the text's size with 16 more in x and 6 in y in the middle of the window, with the message page (@ 0x206be8).
pub fn startMessage(text: i32, action: i32, kind: i32) {
    let frame = Frame::new(32);
    let sp = frame.sp();
    copy_message(0x1fb2528, text);
    sw(0x3afe2c, kind);
    sw(0x3afe30, action);
    textSize(sp + 16, sp + 20, text);
    let width = lw(sp + 16);
    let window = g::window;
    let height = lw(sp + 20);
    let left = lw(window + 8) - half(width);
    let top = lw(window + 28) - half(height);
    sw(sp + 8, left + width + 16);
    sw(sp + 12, top + height + 6);
    sw(sp, left);
    sw(sp + 4, top);
    zoomboxInit(0, sp, 0x2880);
    frontPageSet(lw(lw(g::front) + 20), g::message_page_329538);
}

/// Start a message of the front end: keep the text in the buffer of the front end's current definition (1040 bytes each from 0x1fc19c0; at +1024 `action`, +1028 `kind`, +1032
/// `data`), size it, and open a zoom box (mode 0x2880) of the text's size with 32 more in x and 16 in y in the middle of the window, with the message page (@ 0x231458).
pub fn frontStartMessage(data: i32, text: i32, kind: i32, action: i32) {
    let frame = Frame::new(32);
    let sp = frame.sp();
    let buffer = lw(lw(g::front)) * 1040 + 0x1fc19c0;
    copy_message(buffer, text);
    sw(buffer + 1032, data);
    sw(buffer + 1024, action);
    sw(buffer + 1028, kind);
    textSize(sp + 16, sp + 20, text);
    let window = g::window;
    let width = lw(sp + 16) + 32;
    let height = lw(sp + 20) + 16;
    let left = lw(window + 8) - half(width);
    let top = lw(window + 28) - half(height);
    sw(sp + 8, left + width);
    sw(sp + 16, width);
    sw(sp + 12, top + height);
    sw(sp + 20, height);
    sw(sp, left);
    sw(sp + 4, top);
    zoomboxInit(lw(lw(g::front)), sp, 0x2880);
    frontPageSet(lw(lw(g::front) + 20), g::message_page_353a80);
}

/// Start a message in the map maker: like `frontStartMessage` but the text goes in the buffer at 0x1fc3a00 (`kind` at 0x3b0028, `action` at 0x3b002c, 0 at 0x3b0030, the front
/// end's +12 at 0x3b0034), and the front end gets flag 2 at +12, the map maker's cursor 16 and the edit flags 0x4000 (@ 0x24ca80).
pub fn mmStartMessage(text: i32, action: i32, kind: i32) {
    let frame = Frame::new(32);
    let sp = frame.sp();
    let front = lw(g::front);
    copy_message(0x1fc3a00, text);
    sw(0x3b0028, kind);
    sw(0x3b002c, action);
    let state = lw(front + 12);
    sw(0x3b0030, 0);
    sw(0x3b0034, state);
    textSize(sp + 16, sp + 20, text);
    let window = g::window;
    let width = lw(sp + 16) + 32;
    let height = lw(sp + 20) + 16;
    let left = lw(window + 8) - half(width);
    let top = lw(window + 28) - half(height);
    sw(sp + 8, left + width);
    sw(sp + 12, top + height);
    sw(sp + 16, width);
    sw(sp + 20, height);
    sw(sp, left);
    sw(sp + 4, top);
    zoomboxInit(0, sp, 0x2880);
    frontPageSet(lw(lw(g::front) + 20), g::message_page_356828);
    let front = lw(g::front);
    let flags = lw(g::editflags) | 0x4000;
    sw(front + 12, lw(front + 12) | 2);
    sw(g::mmcursor, 16);
    sw(g::editflags, flags);
}

/// Restart the HUD: clear its parameters (1408 bytes), find the texture numbers of the two HUD pictures (kept at 0x3b0198 and 0x3b0194), of the bag icon (0x3b019c; -1 when the
/// number at 0x3b01a0 is below 0) and of the 15 pictures named in the table at 0x1feaf80 (the numbers go to 0x1feaf40); and give the three at +28 to +36 the number at +24 (@ 0x2b0890).
pub fn hudRestart() {
    memset(g::hudparms, 0, 1408);
    let first = preloadGetTexName(0x3a9920);
    sw(0x3b0198, lw(first + 20));
    let second = preloadGetTexName(0x3a9940);
    let bag = lw(0x3b01a0);
    sw(0x3b0194, lw(second + 20));
    if bag < 0 {
        sw(0x3b019c, -1);
    } else {
        let name = hudGetBagIconFileName(bag);
        let texture = preloadGetTexName(name);
        sw(0x3b019c, lw(texture + 20));
    }
    let mut names = 0x1feaf80;
    let mut numbers = 0x1feaf40;
    let mut left = 14;
    loop {
        let name = lw(names);
        if name != 0 {
            let texture = preloadGetTexName(name);
            sw(numbers, lw(texture + 20));
        }
        numbers += 4;
        left -= 1;
        names += 4;
        if left < 0 {
            break;
        }
    }
    let table = 0x1feaf40;
    let number = lw(table + 24);
    sw(0x3b0190, 0);
    sw(table + 36, number);
    sw(table + 28, number);
    sw(table + 32, number);
}

/// Take the guns of the player `player`: the right-hand gun's props (+420, with the two at +548 and +552) and the left-hand gun's (+612, with +740 and +744) are deleted, the
/// hand flags (+408, +600) cleared, and the character's current combination (+260 of its record) and its two clip tables (60 bytes at +272 and +380) cleared (@ 0x280018).
pub fn playerClearGuns(player: i32) {
    let record = lw(lw(player + 384) + 352);
    if lw(player + 420) != 0 {
        let first = lw(player + 548);
        if first != 0 {
            propDelete(first);
            sw(player + 548, 0);
        }
        let second = lw(player + 552);
        if second != 0 {
            propDelete(second);
            sw(player + 552, 0);
        }
        propDelete(lw(player + 420));
        sw(player + 420, 0);
    }
    if lw(player + 612) == 0 {
        sw(player + 408, 0);
    } else {
        let first = lw(player + 740);
        if first != 0 {
            propDelete(first);
            sw(player + 740, 0);
        }
        let second = lw(player + 744);
        if second != 0 {
            propDelete(second);
            sw(player + 744, 0);
        }
        propDelete(lw(player + 612));
        sw(player + 612, 0);
        sw(player + 408, 0);
    }
    sw(player + 600, 0);
    sw(record + 260, 0);
    memset(record + 272, 0, 60);
    memset(record + 380, 0, 60);
}

/// Whether the enemy `enemy` is triggered: the level's set-up is found (and kept as `setup`); the enemy is triggered at once when a player is within 4 of it; or else, when the
/// enemy's team (+2696 of its record) is the team of a player's character it suggests that player's character as its target (kind 4) and gets flag 0x1000 at +2704; 1 for triggered
/// and 0 when no player is there (@ 0x2c9a68).
pub fn enemyTriggered(enemy: i32) -> i32 {
    let level = lw(g::lvlevelnum);
    let record = lw(enemy + 352);
    sw(g::setup, lvGetLevelSetup(level));
    let players = lw(g::lvnumplayers);
    let mut i = 0;
    if players > 0 {
        let characters = lw(g::chrinfo);
        let team = lw(record + 2696);
        let mut character;
        loop {
            character = i * 0x1210 + characters;
            if lw(character + 680) == team {
                break;
            }
            let player = i * 1820 + lw(g::players);
            let dx = lwc1(player + 140) - lwc1(enemy + 48);
            let dy = lwc1(player + 144) - lwc1(enemy + 52);
            let dz = lwc1(player + 148) - lwc1(enemy + 56);
            if (dx * dx + dy * dy + dz * dz) < 16.0_f32 {
                return 1;
            }
            i += 1;
            if i >= players {
                return 0;
            }
        }
        let target = suggestTarget(enemy, lw(character + 3020), 4);
        let flags = lw(record + 2704);
        sw(record + 2788, target);
        sw(record + 2704, flags | 0x1000);
        return 1;
    }
    0
}

/// Move the character `chr` a little (adding the doubles at 0x3a1cb8 to x and 0x3a1cc0 to z for the characters of kinds 2, 27, 28 and 62 [the kind is the halfword at +4 of the
/// record less 2], 0 to x and the double at 0x3a1cc8 to z for 40, 46 and 60) (@ 0x2365b8).
pub fn tlhAdjustChrPosition(chr: i32) {
    let kind = ((lhu(lw(chr + 352) + 4) - 2) << 16) >> 16;
    if (kind as u32) < 61 {
        let z_add;
        match kind {
            0 | 25 | 26 | 60 => {
                let x = dptofp(dpadd(fptodp(lwc1(chr + 48)), ld(0x3a1cb8)));
                let z = lwc1(chr + 56);
                let y = lwc1(chr + 52) + 0.0_f32;
                swc1(chr + 48, x);
                swc1(chr + 52, y);
                z_add = (fptodp(z), ld(0x3a1cc0));
            }
            38 | 44 | 58 => {
                let x = dptofp(dpadd(fptodp(lwc1(chr + 48)), 0_i64));
                let z = lwc1(chr + 56);
                let y = lwc1(chr + 52) + 0.0_f32;
                swc1(chr + 48, x);
                swc1(chr + 52, y);
                z_add = (fptodp(z), ld(0x3a1cc8));
            }
            _ => return,
        }
        swc1(chr + 56, dptofp(dpadd(z_add.0, z_add.1)));
    }
}
