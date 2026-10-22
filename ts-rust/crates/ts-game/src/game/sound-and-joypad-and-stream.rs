//! Functions: the sound loading and playing, the joypads, the controller and the streams.
//!
//! A sound is an entry of 36 bytes in the table `sounds`: the name (+4), the data on the IOP (+24), the sizes (+32) and the settings of the voice (+8 to +20), and the flags (+22). The commands for the sound chip are collected in a batch (`soundBatchAdd`) and sent to it in one go by `soundBatchExec`.

use crate::*;

/// Send the batch of commands for the sound chip to the IOP and run it there.
pub fn soundBatchExec() {
    let length = lw(g::soundbatchlen);
    if length <= 0 {
        return;
    }
    sceSdTransToIOP(&[0x1fb19c0, lw(0x3afe24) as i64, ((length << 3) + 15 & -16) as i64, 1]);
    sceSdRemote(&[0, 0x80b0, lw(0x3afe24) as i64, 0, lw(g::soundbatchlen) as i64]);
}

/// Start the sound: take the memory on the IOP for the sound data (0x40000 bytes) and for the batch (1600 bytes), and set the sound chip up. The game ends if the IOP has no memory.
pub fn soundMake() {
    sceSifInitIopHeap(&[]);
    let data = sceSifAllocIopHeap(&[0x40000]);
    sw(0x3afe20, data);
    if data < 0 {
        exit(-1);
    }
    let batch = sceSifAllocIopHeap(&[1600]);
    sw(0x3afe24, batch);
    if batch < 0 {
        exit(-1);
    }
    sceSdRemoteInit(&[]);
    sceSdRemote(&[1, 0x8000, 0]);
    sceSdRemote(&[1, 0x8070, 10, 2048]);
}

/// Load the sound `a0` from its file to the memory of the sound chip, if it is not loaded yet. The file is read to the scratch buffer, made to end on a block of 64 bytes (the last 16 bytes are repeated to fill it), and sent to the IOP.
pub fn soundLoad(a0: i32) {
    let frame = Frame::new(160);
    let sp = frame.sp();
    let entry = g::sounds + a0 * 36;
    if lw(entry + 24) != 0 {
        return;
    }
    let name_at = entry + 4;
    let size = fileSize(lw(name_at));
    let buffer = g::scratchbuffer;
    let padded = size + 63 & -64;
    let mut length = size - 64;
    sceSifWriteBackDCache(&[buffer as i64, padded as i64]);
    let data = buffer + 64;
    InvalidDCache(buffer, padded + buffer);
    fileLoadTo(lw(name_at), buffer, size);
    if length & 63 != 0 {
        let mut from = size - 80 + data;
        loop {
            bcopy(from, data + length, 16);
            length += 16;
            from += 16;
            if length & 63 == 0 {
                break;
            }
        }
    }
    sceSifWriteBackDCache(&[data as i64, length as i64]);
    sceSdTransToIOP(&[data as i64, lw(0x3afe20) as i64, length as i64, 1]);
    let buffers = lw(g::thissoundbuf);
    let iop = lw(0x3afe20);
    let slot = a0 * 36 + g::sounds + 24;
    sw(slot, lw(buffers + 8));
    sw(g::sounds + a0 * 36 + 32, length);
    let left = lw(buffers + 12) - length;
    sw(buffers + 8, lw(buffers + 8) + length);
    sw(buffers + 12, left);
    sceSdRemote(&[1, 0x80d0, 1, 0, iop as i64, lw(slot) as i64, length as i64, slot as i64]);
    sceSdRemote(&[1, 0x80f0, 1, 1]);
    // A pause for the IOP, as a loop on the stack.
    for count in 0..=0xf423f_i32 {
        std::hint::black_box(count.wrapping_mul(count));
    }
    let _ = sp;
}

/// Start the sound `a0` with the volume and pan `a1`, `a2` and the pitch `f12`, on a voice chosen by the sound chip (or on the voice kept in `sound_voice`), and give the number of the voice; -1 if it cannot be played: the sound is off, the same sound was started in this frame already, or it is not loaded.
pub fn soundStartRange(a0: i32, a1: i32, a2: i32, f12: f32) -> i32 {
    let frame = Frame::new(96);
    let sp = frame.sp();
    let sound = a0 * 36 + g::sounds;
    if lw(g::gv_playsound) == 0 {
        return -1;
    }
    let mut started = 0;
    let tick = lw(g::tickframenum);
    if tick == lw(g::lastsoundframenum_160) {
        started = lw(g::numsounds_161);
        for i in 0..started {
            if lw(0x1fb1600 + (i << 2)) == a0 {
                return -1;
            }
        }
    } else {
        sw(g::lastsoundframenum_160, tick);
        sw(g::numsounds_161, 0);
    }
    if started < 100 {
        sw(g::numsounds_161, started + 1);
        sw((started << 2) + 0x1fb1600, a0);
    }
    let voice = lw(g::sound_voice);
    if lw(sound + 24) == 0 {
        return -1;
    }
    let core = lw(g::sound_core);
    if voice < 0 {
        getNextChannel(sp, sp + 4);
    } else {
        sw(sp + 4, voice);
        sw(sp, core);
    }
    soundCalcVol(sound, a1, sp + 8, sp + 12, a2, f12);
    let channel = lhu(sp) | lhu(sp + 4) << 1;
    soundBatchAdd(1, channel & 0xffff, lw(sp + 8));
    soundBatchAdd(1, (channel | 256) & 0xffff, lw(sp + 12));
    soundBatchAdd(1, (channel | 512) & 0xffff, lhu(sound + 18));
    soundBatchAdd(
        1,
        (channel | 768) & 0xffff,
        (lhu(sound + 8) & 127) << 8 | lhu(sound + 14) & 15 | ((lhu(sound + 10) & 15) << 4 | 0x8000),
    );
    soundBatchAdd(1, (channel | 1024) & 0xffff, (lhu(sound + 12) & 127) << 6 | (lhu(sound + 16) & 31 | 0xc020));
    soundBatchAdd(3, (channel | 0x2040) & 0xffff, lw(sound + 24));
    soundBatchAdd(2, lhu(sp) | 0x1500, 1 << (lw(sp + 4) & 31));
    lw(sp) | lw(sp + 4) << 1
}

/// Reset the sound chip and the state of the sounds (after a load of a level, or the restart of the game): the 10 voices of the table at 0x1fb1870 are cleared, both cores are keyed off, set to the same volumes and effect and the memory taken is forgotten, and the sounds the level needs are queued for loading.
pub fn soundRestart() {
    let frame = Frame::new(96);
    let sp = frame.sp();
    let mut voice = 0x1fb1870;
    for _ in 0..10 {
        sw(voice, 0);
        sh(voice + 10, 0);
        voice += 32;
    }
    sw(g::transinprogress, 0);
    sw(0x3afe28, 0);
    for core in 0..2 {
        sceSdRemote(&[1, 0x8030, (core | 0x1600) as i64, 0xffffff]);
        sceSdRemote(&[1, 0x8050, (core | 0x1d00) as i64, if core != 0 { 0x1fffff } else { 0x1dffff }]);
        sceSdRemote(&[1, 0x8070, (core | 2) as i64, 1]);
        sceSdRemote(&[1, 0x8010, (core | 2944) as i64, 0]);
        sceSdRemote(&[1, 0x8010, (core | 3200) as i64, 0]);
        sceSdRemote(&[1, 0x8010, (core | 2432) as i64, 0x3fff]);
        sceSdRemote(&[1, 0x8010, (core | 2688) as i64, 0x3fff]);
        sh(sp + 8, 0);
        sh(sp + 10, 0);
        sw(sp + 4, 261);
        sceSdRemote(&[1, 0x8130, core as i64, sp as i64]);
    }
    for index in 0..1215 {
        let flags = lhu(g::sounds + 22 + index * 36);
        if flags & 16 != 0 {
            let level = lw(g::lvlevelnum);
            if flags & 8 != 0 {
                if ((flags as u32) >> 8) as i32 != level {
                    continue;
                }
                soundAddPreLoad(index);
            }
        }
    }
    sw(g::announcecountdown, -1);
    sw(g::lastannounce, -1);
}

/// Write the table of the sounds as the text of a file (@ 0x3998b0) for the sound tool: one line for each sound, in a format that depends on its flags. The old file is copied to a backup first.
pub fn sounddataSave() {
    let frame = Frame::new(1216);
    let sp = frame.sp();
    let path = 0x3998b0;
    let buffer = malloc(0x100000);
    if buffer == 0 {
        return;
    }
    sprintf(&[buffer as i64, 0x3998c8]);
    let mut at = buffer + strlen(buffer);
    for index in 0..1215 {
        let entry = index * 36 + g::sounds;
        let flags = lhu(entry + 22);
        if flags & 8 != 0 {
            sw(sp, lh(entry + 16));
            sw(sp + 8, lhu(entry + 18));
            sw(sp + 16, lhu(entry + 20));
            let voice = if flags & 16 == 0 { g::snd + 8 } else { 0x399980 };
            sw(sp + 24, voice);
            sw(sp + 32, (((flags & 0xffff) as u32) >> 8) as i32);
            sprintf(&[
                at as i64,
                0x399938,
                lw(entry) as i64,
                lw(entry + 4) as i64,
                lh(entry + 8) as i64,
                lh(entry + 10) as i64,
                lh(entry + 12) as i64,
                lh(entry + 14) as i64,
            ]);
        } else {
            let format = if flags & 16 != 0 {
                0x399998
            } else if flags & 4 != 0 {
                0x3999d8
            } else if flags & 1 != 0 {
                0x399a18
            } else if flags & 2 != 0 {
                0x399a58
            } else {
                0x399a98
            };
            sw(sp, lh(entry + 16));
            sw(sp + 8, lhu(entry + 18));
            sw(sp + 16, lhu(entry + 20));
            sprintf(&[
                at as i64,
                format,
                lw(entry) as i64,
                lw(entry + 4) as i64,
                lh(entry + 8) as i64,
                lh(entry + 10) as i64,
                lh(entry + 12) as i64,
                lh(entry + 14) as i64,
            ]);
        }
        at += strlen(at);
    }
    sprintf(&[at as i64, 0x3ae340]);
    let end = at + strlen(at);
    sprintf(&[(sp + 48) as i64, 0x3ae348, path as i64]);
    if fileSize(path) > 0 {
        fileCopy(path, sp + 48);
    }
    fileSave(path, buffer, end - buffer);
    free(buffer);
}

/// Ask a multitap how many pads it has, one of the two each time the counter of the polls gets to 0 or 60 (it counts to 120).
pub fn joyMtapTick() {
    let mut count = lw(g::mtappollcount);
    if count == 0 {
        sw(g::max_slots, scePadGetSlotMax(&[0]));
        count = lw(g::mtappollcount);
    } else if count == 60 {
        sw(g::max_slots + 4, scePadGetSlotMax(&[1]));
        count = lw(g::mtappollcount);
    }
    sw(g::mtappollcount, (count + 1).wrapping_rem(120));
}

/// The tick of all the 8 controllers, and then of the cheats and of the replay.
pub fn joyTick() {
    joyMtapTick();
    sw(g::joy_justdisconnected, 0);
    for index in 0..8 {
        let controller = g::controllers + index * 384;
        sw(controller + 340, 0);
        controllerTick(controller);
    }
    cheatTick();
    replayTick();
}

/// Start the pad library and the multitaps, open all the 8 controllers, and ask how many pads are on each of the two ports.
pub fn joyMake() {
    sceMtapInit(&[]);
    for port in 0..4 {
        sceMtapPortOpen(&[port]);
    }
    scePadInit(&[0]);
    for index in 0..8 {
        controllerOpen(g::controllers + index * 384);
    }
    if sceMtapGetConnection(&[0]) == 0 {
        sw(g::max_slots, 1);
    } else {
        sw(g::max_slots, scePadGetSlotMax(&[0]));
    }
    let mut slots = 1;
    if sceMtapGetConnection(&[1]) != 0 {
        slots = scePadGetSlotMax(&[1]);
    }
    sw(g::max_slots + 4, slots);
}

/// Open the pad of the controller `a0` and clear its state (the vibration bytes at +320 to +325); the game ends if the pad cannot be opened.
pub fn controllerOpen(a0: i32) {
    for offset in 0..6 {
        sb(a0 + 320 + offset, 0);
    }
    swc1(a0 + 316, 0.25_f32);
    if scePadPortOpen(&[lw(a0 + 256) as i64, lw(a0 + 260) as i64, a0 as i64, a0 as i64]) != 0 {
        return;
    }
    exit(0);
}

/// Go to the next state of the pad.
fn next_pad_state(pad: i32) {
    sw(pad + 272, lw(pad + 272) + 1);
}

/// Move the pad `pad` along the states of the start: 0 (not known) asks what kind of pad it is, 40 and 41 set the mode with the analogue sticks, 70 and 71 set the vibration motors, and 99 is a pad that works. 42 waits for the answer to the mode.
fn step_pad_state(pad: i32) {
    let port = lw(pad + 256) as i64;
    let slot = lw(pad + 260) as i64;
    match lw(pad + 272) {
        70 => {
            if scePadInfoAct(&[port, slot, -1, 0]) == 0 {
                sw(pad + 272, 99);
            }
            sb(pad + 326, 0);
            sb(pad + 327, 1);
            sb(pad + 331, 255);
            sb(pad + 328, 255);
            sb(pad + 329, 255);
            sb(pad + 330, 255);
            if scePadSetActAlign(&[port, slot, (pad + 326) as i64]) != 0 {
                next_pad_state(pad);
            }
        }
        71 => {
            if scePadGetReqState(&[port, slot, port]) == 1 {
                sw(pad + 272, 70);
            }
            if scePadGetReqState(&[port, slot]) == 0 {
                sw(pad + 272, 99);
            }
        }
        41 => {
            if scePadSetMainMode(&[port, slot, 1, 3]) == 1 {
                next_pad_state(pad);
            }
        }
        42 => {
            if scePadGetReqState(&[port, slot, port]) == 1 {
                sw(pad + 272, 41);
            }
            if scePadGetReqState(&[port, slot]) == 0 {
                sw(pad + 272, 0);
            }
        }
        0 => {
            let state = lw(pad + 264);
            if state != 6 && state != 2 {
                return;
            }
            let first = scePadInfoMode(&[port, slot, 1, 0]);
            if first == 0 {
                return;
            }
            let second = scePadInfoMode(&[port, slot, 2, 0]);
            let kind = if second >= 1 { second } else { first };
            if kind == 4 {
                sw(pad + 272, 40);
            } else if kind == 7 {
                sw(pad + 272, 70);
            }
        }
        40 => {
            if scePadInfoMode(&[port, slot, 2, 0]) != 0 {
                next_pad_state(pad);
            } else {
                sw(pad + 272, 0);
            }
        }
        _ => {}
    }
}

/// Read the pad in the state 99 into the controller `pad`: the buttons (+288, with the ones of the last read at +284), the sticks (+300 to +312) and the pressed (+292) and released (+296) buttons. It also keeps the last button presses in the buffer of the cheats, and sets the vibration of the motors.
fn read_pad(pad: i32, buffer: i32) {
    sw(pad + 284, lw(pad + 288));
    let result = scePadRead(&[lw(pad + 256) as i64, lw(pad + 260) as i64, buffer as i64]);
    let high = lbu(buffer + 2);
    let previous;
    if result <= 0 {
        sw(pad + 272, 0);
        previous = lw(pad + 284);
    } else {
        let mode = lbu(buffer + 1);
        let kept = lw(pad + 280);
        sw(pad + 288, (high << 8 | lbu(buffer + 3)) ^ 0xffff);
        sw(pad + 300, lbu(buffer + 6));
        sw(pad + 308, lbu(buffer + 7));
        sw(pad + 304, lbu(buffer + 4));
        sw(pad + 312, lbu(buffer + 5));
        sw(pad + 276, mode);
        if kept != 0 && mode != kept {
            sw(pad + 280, 0);
            sw(pad + 272, 0);
        }
        sw(pad + 280, lw(pad + 276));
        let motor = lbu(pad + 320);
        if motor != 0 {
            sb(pad + 320, motor + 128);
        }
        let motor = lbu(pad + 321);
        if motor != 0 {
            sb(pad + 321, motor + 128);
        }
        scePadSetActDirect(&[lw(pad + 256) as i64, lw(pad + 260) as i64, (pad + 320) as i64, kept as i64, mode as i64]);
        previous = lw(pad + 284);
    }
    let now = lw(pad + 288);
    let pressed = now & !previous;
    sw(pad + 296, previous & !now);
    sw(pad + 292, pressed);
    if pressed & 0xf0f0 != 0 {
        let position = lw(g::cycbufpos);
        sh((position << 1) + g::cycbuf, lhu(pad + 288) & 0xf0ff);
        sw(g::cycbufpos, (position + 1).wrapping_rem(20));
    }
}

/// The tick of the controller `a0`: ask the state of the pad and move it along the states of the start (see `step_pad_state`), and read it if it works. A pad that does not work has no buttons pressed and its sticks in the middle (128).
pub fn controllerTick(a0: i32) {
    let frame = Frame::new(96);
    let sp = frame.sp();
    let port = lw(a0 + 256);
    let slot = lw(a0 + 260);
    sw(a0 + 268, lw(a0 + 264));
    if lw((port << 2) + g::max_slots) >= slot {
        sw(a0 + 264, scePadGetState(&[port as i64, slot as i64, port as i64]));
    } else {
        sw(a0 + 264, 0);
    }
    if lw(a0 + 264) == 0 {
        if lw(a0 + 268) != 0 {
            sw(g::joy_justdisconnected, 1);
        }
        sw(a0 + 272, 0);
    }
    step_pad_state(a0);
    if lw(a0 + 272) == 99 {
        read_pad(a0, sp);
        if lw(a0 + 272) == 99 {
            return;
        }
    }
    sw(a0 + 292, 0);
    sw(a0 + 312, 128);
    sw(a0 + 296, 0);
    sw(a0 + 288, 0);
    sw(a0 + 300, 128);
    sw(a0 + 308, 128);
    sw(a0 + 304, 128);
}

/// Connect to the stream server on the IOP (the id 0x534a4521) and wait for it to be there. Gives 0, or -1 if the connection fails.
pub fn stream_RPCinit() -> i32 {
    loop {
        let client = 0x1fb2040;
        if sceSifBindRpc(&[client as i64, 0x534a4521, 0]) < 0 {
            return -1;
        }
        if lw(client + 36) != 0 {
            return 0;
        }
    }
}

/// Call the function `a0` of the stream server with the argument `a1`, and give the answer. A function of the kind 0x802x gets `a1` as the buffer of its argument (64 bytes); any other gets it as a word.
pub fn stream_RPC(a0: i32, a1: i32) -> i32 {
    let frame = Frame::new(48);
    let sp = frame.sp();
    if a0 & 0xfff0 != 0x8020 {
        sw(0x1fb2000, a1);
        sw(sp, 0);
        sceSifCallRpc(&[0x1fb2040, a0 as i64, 0, 0x1fb2000, 16, 0x1fb2000, 64, 0]);
    } else {
        sw(sp, 0);
        sceSifCallRpc(&[0x1fb2040, a0 as i64, 0, a1 as i64, 64, 0x1fb2000, 64, 0]);
    }
    lw(0x1fb2000)
}
