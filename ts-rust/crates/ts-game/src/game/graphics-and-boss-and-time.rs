//! Functions: the graphics synthesizer set up, the interrupt of the vertical blank, the main loop and the boss routines, the texture loading and the time text.

use crate::*;

/// Write the words `words` one after the other from `at`.
fn put_words(at: i32, words: &[i32]) {
    for (i, word) in words.iter().enumerate() {
        sw(at + (i as i32) * 4, *word);
    }
}

/// Add the packet `packet` of `count` quad words to the list of the display being built: the tag with the id byte `id` goes at the end of the list (the global `gfxtag`) and the pointer moves on.
fn link_packet(id: i32, packet: i32, count: i32) {
    sb(lw(g::gfxtag) + 3, id);
    let tag = lw(g::gfxtag);
    sw(tag + 4, packet);
    sh(tag, count);
    sw(g::gfxtag, tag + 16);
}

/// Set the double buffer of the graphics synthesizer to the defaults for a screen of the given sizes (`a6` is the clear colour), and set the clear colour of both of its environments.
pub fn sjeGsSetDefDBuff(a0: i32, a1: i32, a2: i32, a3: i32, a4: i32, a5: i32, a6: i32) {
    let db = g::db;
    let a7 = a3 << 16;
    sceGsSetDefDBuff(&[
        db as i64,
        ((a0 << 16) >> 16) as i64,
        ((a1 << 16) >> 16) as i64,
        ((a2 << 16) >> 16) as i64,
        (a7 >> 16) as i64,
        ((a4 << 16) >> 16) as i64,
        ((a5 << 16) >> 16) as i64,
        a7 as i64,
    ]);
    let other = ld(db + 64) & -0x1000_i64;
    sd(db + 24, ld(db + 24) & -0x1000_i64 | 676_i64);
    sd(db + 432, a6 as i64);
    let low = lw(db + 24);
    sd(db + 192, a6 as i64);
    sd(db + 64, other | (low & 4095) as i64);
}

/// Add a packet to the display list that sets the mask of the frame buffer to `a0`; the register FRAME is taken from the environment of the buffer that is not being shown.
pub fn dlSetFBMSK(a0: i32) {
    let env = if lw(g::lastfrbuf) == 0 { g::db + 96 } else { g::db + 336 };
    let frame_low = lw(env);
    let packet = memdbAlloc(48);
    sw(packet, 0x6c028000);
    link_packet(48, packet, 3);
    put_words(
        packet + 4,
        &[0x8001, 0x10000000, 14, 0, frame_low, a0, lw(env + 8), 0, 0x140007fc, 0x11000000, 0],
    );
}

/// The interrupt of the vertical blank. It swaps the double buffer when the picture on the graphics synthesizer is done, and starts the drawing of the next list of the display when the one for the other buffer is due.
pub fn vblIntHandler() {
    timeTickRetrace();
    if lw(g::gsdone) != 0 && iPollSema(lw(g::gsavail_sid)) != 0 {
        let shown = 1 - lw(g::lastfrbuf);
        let mut dropped = lw(g::framedropcount);
        sw(g::lastfrbuf, shown);
        if dropped != 0 {
            dropped -= 1;
            sw(g::framedropcount, dropped);
        }
        let half = if dropped != 0 { 0 } else { lw(g::framenum) & 1 };
        sjeGsSetHalfOffset((shown != 0) as i32, half);
        sjeGsSwapDBuff(lw(g::lastfrbuf));
        iSignalSema(lw(g::gsavail_sid));
        let info = g::dbinfo;
        // A list is due when its buffer is free and the frame it was drawn for is at least 2 frames past, counted from 128 before, modulo 128.
        let mut first = false;
        let mut second = false;
        if lw(info) == 0 {
            let age = lw(g::framenum) - (lw(info + 4) - 128);
            first = age % 128 >= 2;
        }
        if lw(info + 12) == 0 {
            let age = lw(g::framenum) - (lw(info + 16) - 128);
            second = age % 128 >= 2;
        }
        if first {
            if second {
                if lw(info + 4) >= lw(info + 16) {
                    second = false;
                } else {
                    first = false;
                }
            }
            if first {
                sw(g::curgsdmabuf, 0);
                iSignalSema(lw(g::gsstart_sid));
                return;
            }
        }
        if second {
            sw(g::curgsdmabuf, 1);
            iSignalSema(lw(g::gsstart_sid));
        }
        return;
    }
    sw(g::framedropcount, 30);
}

/// The main loop of the thread of the graphics synthesizer. Each time it is started it sends the list of the display that the interrupt of the vertical blank chose to the graphics synthesizer, and tests the depth buffer the first time, and notes the times of the frame.
pub fn gsMain() {
    loop {
        let start = lw(g::gsstart_sid);
        sw(g::gsdone, 1);
        WaitSema(start);
        let last_start = lwc1(0x3afe04);
        let last_end = lwc1(0x3afe08);
        sw(g::gsdone, 0);
        swc1(g::lastgfxstarttime, last_start);
        swc1(g::lastgfxendtime, last_end);
        let now = timeGet();
        sw(0x10000010, 131);
        sw(0x10000000, 0);
        let counts = g::dbinfo + 8;
        swc1(0x3afe04, now);
        let count_at = lw(g::curgsdmabuf) * 12 + counts;
        sw(count_at, lw(count_at) + 1);
        if lw(lw(g::curgsdmabuf) * 12 + counts) < 3 {
            if lw(lw(g::curgsdmabuf) * 12 + counts) == 1 {
                WaitSema(lw(g::gsavail_sid));
                zbtestCopyZB();
                SignalSema(lw(g::gsavail_sid));
                zbtestDoTest();
                zbtestAdvanceTestPtr();
            }
            // An alarm watches that the DMA finishes in time.
            let list = lw((lw(g::curgsdmabuf) << 2) + g::dmalists);
            let alarm = SetAlarm(&[3144, 0x200f10, list as i64]);
            sw(g::alarm_activated, 0);
            WaitSema(lw(g::gsavail_sid));
            FlushCache(0);
            sceDmaSend(&[lw(g::dmaVif1) as i64, lw((lw(g::curgsdmabuf) << 2) + g::dmalists) as i64]);
            WaitSema(lw(g::gsdone_sid));
            SignalSema(lw(g::gsavail_sid));
            if lw(g::alarm_activated) != 0 {
                scePrintf(&[0x3838d0]);
                sw(g::numdmafail, lw(g::numdmafail) + 1);
                sw(0x1000f590, 0);
                sw(0x10009000, 5);
                sw(0x10003c10, 1);
                sw(0x10003000, 1);
                sd(0x12001000, 256_i64);
                sw(0x1000f590, 1);
            } else {
                ReleaseAlarm(&[alarm as i64]);
            }
        }
        let mut end = timeGet();
        let start_time = lwc1(0x3afe04);
        swc1(0x3afe08, end);
        while end < start_time {
            end = ps2float::add(end, 128.0_f32);
            swc1(0x3afe08, end);
        }
        sw(0x10000810, 131);
        let cycles = lw(0x10000800);
        let cpu_done = lw(g::cpudone);
        sw(g::ucodecycles, cycles.wrapping_mul(0x112492));
        if cpu_done != 0 {
            SignalSema(lw(g::bufffree_sid));
        }
    }
}

/// Clear the video memory of the graphics synthesizer to the colour (`a0`, `a1`, `a2`, `a3` are red, green, blue and alpha), with a packet that is sent by DMA on the channel 2.
pub fn clearvram(a0: i32, a1: i32, a2: i32, a3: i32) {
    let frame = Frame::new(112);
    let sp = frame.sp();
    let channel = sceDmaGetChan(&[2, a1 as i64, a2 as i64, a3 as i64]);
    sceGifPkInit(&[sp as i64, g::base_ptr as i64]);
    sceGifPkReset(&[sp as i64]);
    let colour = (a0 & 255) as i64 | 0x3f80000000000000_i64 | (((a2 & 255) as i64) << 16 | ((a1 & 255) as i64) << 8) | ((a3 & 255) as i64) << 24;
    let data: [i64; 20] = [
        0x1000000000008009_i64,
        14,
        1,
        26,
        0x100000,
        76,
        0,
        24,
        0x400000004000000_i64,
        64,
        0,
        71,
        6,
        0,
        colour,
        1,
        0,
        4,
        0x40004000,
        4,
    ];
    for value in data {
        sceGifPkAddGsData(&[sp as i64, value]);
    }
    sceGifPkTerminate(&[sp as i64]);
    FlushCache(0);
    let address = lw(sp + 4) & 0xfffffff;
    let size = sceGifPkSize(&[sp as i64]);
    sceDmaSendN(&[channel as i64, address as i64, size as i64]);
    sceGsSyncPath(&[0, 0]);
}

/// Start all the parts of the game that are made once.
pub fn bossMakeAll() {
    bossMake();
    trigMake();
    memMake(0x3b0380, 0x1ae1c00);
    packfileMake();
    profileMake();
    windowMake();
    joyMake();
    soundMake();
    musicMake();
    mcardMake();
    mcseqMake();
    ilinkMake();
    cheatsMake();
    signonMake();
    frontmpgMake();
    mapmakerMake();
}

/// Fill the rectangle `x0`, `y0`, `x1`, `y1` of the screen around a window with the grey 128.
fn fill_border(x0: i32, y0: i32, x1: i32, y1: i32) {
    dlFillRectangle(x0, y0, x1, y1, 128);
}

/// Set the bit of the player `index` in the flags of the player of the current one, for the camera that can look at all of the players, or clear it.
fn mark_camera_bit(index: i32) {
    let mode = camGetMode();
    let level = lw(g::lvlevelnum);
    let bit = 0x1000000_i32 << (index & 31);
    let flags_at = lw(lw(g::currentplayer) + 384) + 16;
    if mode != 2 && level != 6 {
        sw(flags_at, lw(flags_at) & !bit);
    } else {
        sw(flags_at, lw(flags_at) | bit);
    }
}

/// Add the draw of one of the local players to the display list: its window, its camera and the world, the turtles of the tests, and the border.
fn draw_player_window(index: i32) {
    let player = lw(g::players) + index * 1820;
    sw(g::currentplayer, player);
    windowSet(lw(player + 4), lw(player + 8), lw(player + 12), lw(player + 16));
    let packet = memdbAlloc(80);
    sw(packet, 0x6c048000);
    let camera = lw(lw(g::currentplayer) + 1760);
    link_packet(48, packet, 5);
    for word in 0..16 {
        sw(packet + 4 + word * 4, lw(camera + word * 4));
    }
    put_words(packet + 68, &[0x1400079d, 0x11000000, 0]);
    lvGfx();
    if lw(g::turtlecpu) != 0 {
        bossTurtleCpu();
    }
    if lw(g::turtlegs) != 0 {
        bossTurtleGs();
    }
    dlSetClip(0, 0, 639, 223);
    dlSetBlend(0);
    let p = lw(g::currentplayer);
    fill_border(lw(p + 4) - 2, lw(p + 8) - 1, lw(p + 12) + 1, lw(p + 8));
    let p = lw(g::currentplayer);
    fill_border(lw(p + 4) - 2, lw(p + 16), lw(p + 12) + 1, lw(p + 16) + 1);
    let p = lw(g::currentplayer);
    let left = lw(p + 4);
    fill_border(left - 2, lw(p + 8) - 1, left + 1, lw(p + 16) + 1);
    let p = lw(g::currentplayer);
    let right = lw(p + 12);
    fill_border(right - 1, lw(p + 8) - 1, right + 2, lw(p + 16) + 1);
}

/// The work of the game for a frame, run by the main thread: the tick of the world and the players, and the building of the list of the display of everything.
pub fn bossMainLoop() {
    sw(g::currentplayer, lw(g::players));
    profileTick();
    debugmenuTick();
    windowSet(0, 0, 639, 223);
    ilinkTick();
    for pad in 0..4 {
        joySetShock(pad, 0, 0);
    }
    memdbTick();
    gameTick();
    lvTickBefore();
    let mut index = 0;
    if lw(g::lvnumplayers) > 0 {
        loop {
            let player = lw(g::players) + index * 1820;
            sw(g::currentplayer, player);
            if lw(player + 384) != 0 {
                mark_camera_bit(index);
            }
            index += 1;
            lvTickPlayer();
            if index >= lw(g::lvnumplayers) {
                break;
            }
        }
    }
    lvTickAfter();

    // The list starts with a jump to the memory of the vector unit.
    sw(g::gfxtag, lw((lw(g::curcpudmabuf) << 2) + g::dmalists));
    link_packet(48, 0x2eeff0, (0x2f2bc0 - 0x2eeff0) >> 4);
    let packet = memdbAlloc(48);
    sw(packet, 0x1000404);
    link_packet(48, packet, 3);
    put_words(packet + 4, &[0x6c028000, 0x8001, 0x10000000, 14, 0, 1, 0, 70, 0, 0x140007fc, 0x11000000]);
    dlSetClip(0, 0, 640, 224);
    dlSetZB(1);
    dlClearFB(0, 0, 640, 224, lvGetBgCol());
    dlSetZB(3);
    let mut local = ilinkGetStartLocalPlayerNum();
    while local < ilinkGetStartLocalPlayerNum() + ilinkGetNumLocalPlayers() {
        draw_player_window(local);
        local += 1;
    }
    // With 3 players the fourth window is cleared (unless the background is the black one).
    let local_players = ilinkGetNumLocalPlayers();
    if local_players == 3 {
        let background = lvGetBgCol();
        let first = lw(g::players);
        if background != 128 {
            let old = dlSetZB(1);
            sw(first + 0x1554, local_players);
            let window = first + 0x1554;
            playerSetWindow(window);
            windowSet(lw(window + 4), lw(window + 8), lw(window + 12), lw(window + 16));
            dlClearFB(lw(window + 4) - 2, lw(window + 8) - 1, lw(window + 12) + 2, lw(window + 16), 128);
            dlSetZB(old);
        }
    }
    zbtestAdvanceAddPtr();
    zbtestAdvanceDrawPtr();
    windowSet(0, 0, 639, 223);
    let window = g::window;
    dlSetClip(lw(g::window), lw(window + 20), lw(window + 4), lw(window + 24));
    if lw(g::gamestate) == 6 {
        guiLoadingGfx();
    }
    debugmenuGfx();
    profileGfx();
    dlFinish();
    link_packet(112, 0, 0);
}

/// Write a register of the graphics synthesizer to the packet at `at` as the quad word of a GIF A+D packet: the 64 bit value, the number of the register and 0. Gives the address after it.
fn put_register(at: i32, value: i64, register: i32) -> i32 {
    sw(at, value as i32);
    sw(at + 4, (value >> 32) as i32);
    sw(at + 8, register);
    sw(at + 12, 0);
    at + 16
}

/// Upload to the video memory the picture that `position` (a copy of the first 40 bytes of a texture) says is next, from the mip level `level`; the base pointer and the buffer width of the level are kept at sp+48 and sp+80.
fn upload_level(texture: i32, position: i32, level_at: i32, width_at: i32) {
    let target = g::pTex;
    let format = if lbu(texture + 36) & 1 == 0 { 0 } else { 19 };
    sceGsSetDefLoadImage(&[
        target as i64,
        lh(level_at) as i64,
        lh(width_at) as i64,
        format,
        0,
        0,
        lh(position + 4) as i64,
        lh(position + 2) as i64,
    ]);
    FlushCache(0);
    WaitSema(lw(g::gsavail_sid));
    sjeGsExecLoadImage(target, lw(position + 20));
    sceGsSyncPath(&[0, 0]);
    SignalSema(lw(g::gsavail_sid));
}

/// Load the picture `a1` (a file or, if `a2` says so, the download buffer) as the texture number `a0` of the textures being built: the picture is read, uploaded level by level to the video memory, and the GIF packet that selects it for drawing is made. `a2` is the info of the texture, or 0.
pub fn emLoadTexture(a0: i32, a1: i32, a2: i32) {
    let frame = Frame::new(272);
    let sp = frame.sp();
    let texture = lw(lw(g::thistexbuf) + 4) + a0 * 40;
    let info_flags = if a2 != 0 { lw(a2 + 12) } else { 0 };
    sw(texture + 8, if a2 != 0 { lw(a2 + 4) } else { 0 });
    sw(texture + 12, if a2 != 0 { lw(a2 + 8) } else { 0 });
    if a2 != 0 && info_flags & 0x10000000 != 0 {
        ImageLoadFromMem(a1, texture);
    } else {
        ImageLoad(a1, texture);
    }
    if lbu(texture + 36) & 1 != 0 {
        emGetNextPos(texture, 1);
    }
    for part in 0..5 {
        sd(sp + part * 8, ld(texture + part * 8));
    }
    let mut levels = 0;
    let slot = a0 << 7;
    if lbu(texture + 37) != 0 {
        loop {
            if emGetNextPos(sp, 0) == 0 {
                sw(sp + 16, 0);
            }
            let level_at = sp + 48 + (levels << 2);
            let width_at = sp + 80 + (levels << 2);
            sw(level_at, lw(sp + 16));
            sw(width_at, GSBufferWidth(sp));
            if lw(sp + 16) == 0 {
                break;
            }
            sceDmaGetChan(&[2]);
            let mut width = lw(sp + 4);
            let mut height = lhu(sp + 2);
            if width & 7 == 0 {
                if height & 3 == 0 {
                    upload_level(texture, sp, level_at, width_at);
                    width = lw(sp + 4);
                    height = lhu(sp + 2);
                }
            }
            let area = width.wrapping_mul(lhu(sp + 2));
            sw(sp + 4, ((width as u32) >> 1) as i32);
            let at = lw(sp + 20);
            let size = lhu(sp);
            levels += 1;
            sh(sp + 2, ((height as u32) >> 1) as i32);
            sh(sp, ((size as u32) >> 1) as i32);
            sw(sp + 20, at + if lbu(texture + 36) & 1 == 0 { area << 2 } else { area });
            if levels >= lbu(texture + 37) {
                break;
            }
        }
    }
    sb(texture + 37, levels);
    if lbu(texture + 36) & 1 != 0 {
        // The palette is uploaded as a picture of 16 by 16.
        let target = g::pTex;
        sceGsSetDefLoadImage(&[target as i64, lh(texture + 24) as i64, 1, 0, 0, 0, 16, 16]);
        FlushCache(0);
        WaitSema(lw(g::gsavail_sid));
        sjeGsExecLoadImage(target, lw(texture + 28));
        sceGsSyncPath(&[0, 0]);
        SignalSema(lw(g::gsavail_sid));
    }

    // The GIF packet that sets the texture: the registers TEX1, TEX0, MIPTBP1, MIPTBP2 and CLAMP.
    let packet = lw(lw(g::thistexbuf)) + slot;
    sw(texture + 32, packet);
    put_words(packet, &[0, 0, 0, 0x6c0780d8, 1, 0x60000000, 0xeeeeee, 0, 0, 0, 63, 0]);
    let mut at = packet + 48;
    let no_mips = info_flags & 0x20000000 != 0;
    let (tex1_high, tex1_low);
    if lbu(texture + 36) & 2 == 0 {
        let c = if a2 != 0 && no_mips { 0_i64 } else { 1_i64 };
        tex1_high = c << 6;
        tex1_low = c << 5;
    } else {
        let mut filter = 4_i64;
        if a2 != 0 {
            filter = if no_mips { 2 } else { filter };
        }
        let mipmap = if a2 != 0 && no_mips { 0_i64 } else { 1_i64 };
        tex1_high = filter << 6 | mipmap << 5;
        tex1_low = ((lbu(texture + 37) - 1) as i64) << 2 | -0x1e00000000_i64;
    }
    at = put_register(at, tex1_low | tex1_high, 20);
    let tex0;
    if lbu(texture + 36) & 1 == 0 {
        let width_log = PowerOf2(lhu(texture));
        let height_log = PowerOf2(lhu(texture + 2));
        tex0 = lwu(sp + 48) | 0x400000000_i64 | ((width_log as i64) << 26 | lwu(sp + 80) << 14) | (height_log as i64) << 30;
    } else {
        let width_log = PowerOf2(lhu(texture));
        let height_log = PowerOf2(lhu(texture + 2));
        tex0 = lwu(sp + 48)
            | -0x8000000000000000_i64
            | (lwu(sp + 80) << 14 | 0x400000000_i64)
            | ((width_log as i64) << 26 | 0x1300000_i64 | ((height_log as i64) << 30 | lwu(texture + 24) << 37));
    }
    at = put_register(at, tex0, 6);
    let miptbp1 = if lbu(texture + 36) & 2 == 0 {
        0_i64
    } else {
        lwu(sp + 52) | lwu(sp + 92) << 54 | (lwu(sp + 88) << 34 | lwu(sp + 56) << 20) | (lwu(sp + 84) << 14 | lwu(sp + 60) << 40)
    };
    at = put_register(at, miptbp1, 52);
    let miptbp2 = if lbu(texture + 36) & 2 == 0 || (lbu(texture + 37) as u32) < 4 {
        0_i64
    } else {
        lwu(sp + 64) | lwu(sp + 104) << 54 | (lwu(sp + 100) << 34 | lwu(sp + 68) << 20) | (lwu(sp + 96) << 14 | lwu(sp + 72) << 40)
    };
    at = put_register(at, miptbp2, 54);
    let wrap_x = ((lhu(texture) - 1) as i64) << 14;
    let wrap_y = ((lhu(texture + 2) - 1) as i64) << 34;
    let mode = if lw(texture + 8) == 0 { 0 } else { 2_i64 };
    let clamp = if lw(texture + 12) == 0 { mode | wrap_x | wrap_y } else { mode | wrap_y | (wrap_x | 8_i64) };
    put_register(at, clamp, 8);
    tex_loaded_hook(lw(sp + 48), a1, a2);
}

/// The text of a time of the clock of the console (kept in the buffer `rettime_96`): `a0` is the time as a number, or -1 for a time that is not known, which gives the text at 0x3afc38. Gives the buffer.
pub fn rtcFormatTime(a0: i32) -> i32 {
    let frame = Frame::new(32);
    let sp = frame.sp();
    if a0 == -1 {
        sprintf(&[lw(g::rettime_96) as i64, 0x3afc38]);
    } else {
        u322clock(sp, a0);
        let a2 = lbu(sp + 5);
        let a3 = lbu(sp + 6);
        let a4 = lbu(sp + 7);
        let a5 = lbu(sp + 3);
        let a6 = lbu(sp + 2);
        let a7 = lbu(sp + 1);
        sprintf(&[lw(g::rettime_96) as i64, 0x3a9a60, a2 as i64, a3 as i64, a4 as i64, a5 as i64, a6 as i64, a7 as i64]);
    }
    lw(g::rettime_96)
}
