//! Functions: the memory card state machine and the directory helpers it uses.
//!
//! The requests (see `memory-card-file-requests.rs`) set the state `mcardstate`, and `mcardTick` moves it along, one call of the memory card library for each state and one state to wait for it to finish. The states are 1 to 5 check the cards, 6 and 7 format, 8 and 9 get the free space, 10 to 13 check the directory, 14 to 31 make the directory of the game with its files, 32 to 39 read a file, 40 to 47 write a file, 48 and 49 delete a file. The result of the request goes to `mcardresult`: 1 good, 3 an error, 4 the card is not formatted or not good, 5 the file is not found, 6 the card was taken out, 7 not enough space, 2 the file is not the same size as asked.

use crate::*;

/// Ask to check the directory `a0` (0 for the game, else the sign on files) of the memory card, which also makes the icon file for it.
pub fn mcardCheckDir(a0: i32) {
    let name = if a0 != 0 { 0x39a0a8 } else { 0x39a090 };
    makeIconSys(a0);
    strcpy(0x1fb2940, name);
    sw(0x3afe4c, a0);
    sw(g::mcardstate, 16);
    sw(0x3afe68, 0);
    sw(g::mcardresult, 0);
}

/// Start the memory card state machine over: the names of the directories and the files of the icon are set, and the two icon files are loaded.
pub fn mcardReset() {
    sw(0x3afe50, 0x399c20);
    sw(0x3afe54, 0x399c68);
    let first = fileLoad(0x399c78, 1, 0x3afe60);
    sw(0x3afe58, first);
    let second = fileLoad(0x399c90, 1, 0x3afe64);
    sw(0x3afe5c, second);
    sw(g::mcardstate, 0);
}

/// Ask to make the directory `a0` (0 for the game, else the sign on files) on the memory card, with its icon file. `a1` says if a directory that is there can be used, and `a2` if the files are to be copy protected.
pub fn mcardMkdir(a0: i32, a1: i32, a2: i32) {
    let name = if a0 != 0 { 0x39a0a8 } else { 0x39a090 };
    makeIconSys(a0);
    strcpy(0x1fb2940, name);
    sw(0x3afe4c, a0);
    sw(g::mkdir_overwriteok, a1);
    sw(g::mkdir_copyprotect, a2);
    sw(g::mcardstate, 14);
    sw(0x3afe68, 0);
    sw(g::mcardresult, 0);
}

/// Read the details of the root directory of the disc: where its entries are (at `a0`, a sector) and how many sectors it takes (at `a1`). The sectors 11 to 26 are read the first time.
pub fn GetRootDirDetails(a0: i32, a1: i32) {
    if lw(g::readrootdir_6) == 0 {
        cd_read(11, 16);
        sw(g::readrootdir_6, 1);
    }
    sw(a0, makeInt(16, 158));
    let size = makeInt(16, 166);
    sw(a1, (if -1 < size { size } else { size + 2047 }) >> 11);
}

/// Look for the file named `a2` in the directory at the sector `*a0` of `*a1` sectors that was read to the load buffer. If it is there, its sector goes to `*a0` and its sector count to `*a1`, and the size is given (sign extended); if not, 0.
pub fn IsFileInDir(a0: i32, a1: i32, a2: i32) -> i64 {
    let frame = Frame::new(176);
    let sp = frame.sp();
    let mut sector_index = 0;
    while sector_index < lw(a1) {
        let sector = lw(a0) + sector_index;
        let block = ((sector - 16) << 11) + g::loadbuf;
        let mut at = 0;
        if lbu(block) != 0 {
            let mut entry = block;
            loop {
                // The name of the entry: its length is at +32 and the bytes follow.
                let length = lbu(entry + 32);
                let mut copied = 0;
                for from in (at + 33)..(at + 33 + length) {
                    sb(sp + copied, lbu(block + from));
                    copied += 1;
                }
                sb(sp + copied, 0);
                if strcmp(sp, a2) == 0 {
                    let first = makeInt(sector, at + 2);
                    let second = makeInt(sector, at + 10);
                    sw(a0, first);
                    sw(a1, ((second as i64) << 21 >> 32) as i32);
                    return second as i64;
                }
                at += lbu(entry);
                entry = block + at;
                if lbu(entry) == 0 {
                    break;
                }
            }
        }
        sector_index += 1;
    }
    0
}

/// Copy `count` doublewords from `from` to `to`.
fn copy_doublewords(to: i32, from: i32, count: i32) {
    for i in 0..count {
        sd(to + i * 8, ld(from + i * 8));
    }
}

/// Make the icon file (964 bytes) at 0x1fb29c0 for the directory `a0` (0 for the game, else the sign on files): the header, the title, the colours and lights of the icon, and the names of its three models.
pub fn makeIconSys(a0: i32) {
    let (title, models) = if a0 != 0 { (0x399c30, 0x399c68) } else { (0x399be0, 0x399c20) };
    let file = 0x1fb29c0;
    memset(file, 0, 964);
    let magic = 0x3ae428;
    sw(file, lw(magic));
    sb(file + 4, lb(magic + 4));
    strcpy(file + 192, title);
    sh(file + 6, 26);
    sw(file + 12, 48);
    copy_doublewords(file + 16, g::bgcolor_75, 8);
    copy_doublewords(file + 80, g::lightdir_76, 6);
    copy_doublewords(file + 128, g::lightcol_77, 6);
    copy_doublewords(file + 176, g::ambient_78, 2);
    strcpy(file + 260, models);
    strcpy(file + 324, models);
    strcpy(file + 388, models);
}

/// Set the result of the request and go back to the state 0.
fn finish(result: i32) {
    sw(g::mcardresult, result);
    sw(g::mcardstate, 0);
}

/// Go to the state `state`.
fn goto(state: i32) {
    sw(g::mcardstate, state);
}

/// Ask the memory card if it is done: gives the result (kept at `sp`) if it is, or `None` if it is still working. `arg` is the second argument of the call.
fn done(sp: i32, arg: i64) -> Option<i32> {
    if sceMcSync(&[1, arg, sp as i64]) != 1 {
        None
    } else {
        Some(lw(sp))
    }
}

/// Go on from the card at the port `port` and the slot `slot` to the next one (in the order of the ports and the slots) that is asked to be checked (it is 1 in the list at 0x3afe38). If there is none the check is done.
fn next_card_to_check(cards: i32, mut port: i32, mut slot: i32) {
    loop {
        sw(g::checkcards_slot, slot);
        if slot == 4 {
            port += 1;
            slot = 0;
            sw(g::checkcards_slot, 0);
            sw(g::checkcards_port, port);
            if port == 2 {
                break;
            }
        }
        let asked = lw((((port << 2) + slot) << 2) + cards);
        slot += 1;
        if asked == 1 {
            goto(3);
            return;
        }
    }
    finish(1);
}

/// The directory entries that were read (at 0x1fb2e20, each 64 bytes, up to 7): note in the flags at 0x3afe68 whether the icon file (1, and 16 if its size is not 964) and the file of the game (2, and 32 if its size is not the one expected) are there.
fn note_directory_entries(sp: i32, count: i32) {
    let entries = 0x1fb2e20;
    let mut i = 0;
    if count > 0 {
        loop {
            let entry = entries + (i << 6);
            let directory = lw(0x3afe4c);
            if strcmp(entry, 0x399ca8) != 0 {
                if strcmp(entry, lw((directory << 2) + 0x3afe50)) == 0 {
                    let flags = lw(0x3afe68);
                    let size = lw(entry - 16);
                    let expected = lw((lw(0x3afe4c) << 2) + 0x3afe60);
                    sw(0x3afe68, flags | 2);
                    if size != expected {
                        sw(0x3afe68, flags | 34);
                    }
                }
            } else {
                let flags = lw(0x3afe68);
                let size = lw(entry - 16);
                sw(0x3afe68, flags | 1);
                if size != 964 {
                    sw(0x3afe68, flags | 17);
                }
            }
            i += 1;
            if i >= 7 || i >= lw(sp) {
                break;
            }
        }
    }
}

/// The tick of the memory card state machine: one step of the state `mcardstate`.
pub fn mcardTick() {
    let frame = Frame::new(128);
    let sp = frame.sp();
    let state = lw(g::mcardstate);
    if (state as u32) >= 50 {
        return;
    }
    match state {
        0 => {}
        1 => {
            let cards = lw(0x3afe38);
            sw(g::checkcards_port, 0);
            let first = lw(cards);
            sw(g::checkcards_slot, 0);
            if first == 1 {
                goto(3);
                return;
            }
            next_card_to_check(cards, 0, 1);
        }
        2 => {
            if sceMcSync(&[1, 0, sp as i64]) == 1 {
                goto(3);
            }
        }
        3 => {
            let free = if lw(0x3afe3c) == 0 { 0 } else { g::checkcards_free };
            let result = sceMcGetInfo(&[
                lw(g::checkcards_port) as i64,
                lw(g::checkcards_slot) as i64,
                g::checkcards_type as i64,
                free as i64,
                g::checkcards_format as i64,
            ]);
            if result != 0 {
                if result <= 0 {
                    finish(3);
                } else {
                    goto(2);
                }
                return;
            }
            goto(4);
        }
        4 => {
            let Some(code) = done(sp, 0) else {
                return;
            };
            // What the card is, for the list at 0x3afe40 (the kind of card) and 0x3afe38 (its state).
            let kinds = lw(0x3afe40);
            if kinds != 0 {
                let at = kinds + (((lw(g::checkcards_port) << 2) + lw(g::checkcards_slot)) << 2);
                let kind = match code {
                    -1 => 1,
                    0 => 0,
                    -2 => 2,
                    _ => 3,
                };
                sw(at, kind);
            }
            let code = lw(sp);
            let port = lw(g::checkcards_port);
            let value;
            if (-2..=0).contains(&code) {
                if lw(g::checkcards_type) != 2 {
                    value = 3;
                } else if lw(g::checkcards_format) == 0 {
                    value = 5;
                } else {
                    let at = ((port << 2) + lw(g::checkcards_slot)) << 2;
                    let free = lw(0x3afe3c);
                    sw(at + lw(0x3afe38), 4);
                    if free != 0 {
                        sw(at + free, lw(g::checkcards_free));
                    }
                    goto(5);
                    return;
                }
            } else {
                value = 1;
            }
            sw((((port << 2) + lw(g::checkcards_slot)) << 2) + lw(0x3afe38), value);
            goto(5);
        }
        5 => {
            let port = lw(g::checkcards_port);
            let cards = lw(0x3afe38);
            next_card_to_check(cards, port, lw(g::checkcards_slot) + 1);
        }
        6 => {
            if sceMcFormat(&[lw(g::port) as i64, lw(g::slot) as i64]) != 0 {
                finish(3);
                return;
            }
            goto(7);
        }
        7 => {
            let Some(code) = done(sp, 0) else {
                return;
            };
            if code == 0 {
                finish(1);
            } else {
                finish(3);
            }
        }
        8 => {
            if sceMcGetEntSpace(&[lw(g::port) as i64, lw(g::slot) as i64, lw(0x3afe44) as i64]) != 0 {
                finish(3);
                return;
            }
            goto(9);
        }
        9 => {
            let Some(code) = done(sp, 0) else {
                return;
            };
            let target = lw(0x3afe48);
            if code < 0 {
                finish(3);
            } else {
                sw(target, code);
                finish(1);
            }
        }
        10 => {
            if sceMcOpen(&[lw(g::port) as i64, lw(g::slot) as i64, 0x1fb3140, 1]) != 0 {
                finish(3);
                return;
            }
            goto(11);
        }
        11 => {
            let Some(code) = done(sp, 0) else {
                return;
            };
            if code < 0 {
                match code {
                    -4 => finish(5),
                    -2 => finish(6),
                    _ => finish(if code >= -9 { 3 } else { 4 }),
                }
            } else {
                sw(g::mkdir_fd, code);
                goto(12);
            }
        }
        12 => {
            if sceMcClose(&[lw(g::mkdir_fd) as i64]) != 0 {
                finish(3);
                return;
            }
            goto(13);
        }
        13 => {
            let Some(code) = done(sp, 0) else {
                return;
            };
            match code {
                -2 | -1 => finish(6),
                0 => finish(1),
                _ => finish(if code >= -9 { 3 } else { 4 }),
            }
        }
        14 => {
            if sceMcMkdir(&[lw(g::port) as i64, lw(g::slot) as i64, 0x1fb2940]) != 0 {
                finish(3);
                return;
            }
            goto(15);
        }
        15 => {
            let Some(code) = done(sp, 0) else {
                return;
            };
            match code {
                -4 => {
                    if lw(g::mkdir_overwriteok) == 0 {
                        finish(3);
                    } else {
                        goto(16);
                    }
                }
                -3 => finish(7),
                -2 => finish(6),
                0 => {
                    let flags = lw(0x3afe68);
                    goto(18);
                    sw(0x3afe68, flags | 256);
                }
                _ => finish(3),
            }
        }
        16 => {
            sprintf(&[g::mkdir_tmpbuf as i64, 0x3ae430, 0x1fb2940]);
            if sceMcGetDir(&[lw(g::port) as i64, lw(g::slot) as i64, g::mkdir_tmpbuf as i64, 0, 7, 0x1fb2e00]) != 0 {
                finish(3);
                return;
            }
            goto(17);
        }
        17 => {
            let Some(code) = done(sp, 0) else {
                return;
            };
            if code >= 0 {
                note_directory_entries(sp, code);
                goto(18);
            } else if code == -2 {
                finish(6);
            } else {
                finish(3);
            }
        }
        18 => {
            if lw(0x3afe68) & 17 == 1 {
                goto(24);
                return;
            }
            sprintf(&[g::mkdir_tmpbuf as i64, 0x3ae438, 0x1fb2940, 0x399ca8]);
            let mode = if lw(0x3afe68) & 1 == 0 { 514 } else { 2 };
            if sceMcOpen(&[lw(g::port) as i64, lw(g::slot) as i64, g::mkdir_tmpbuf as i64, mode]) != 0 {
                finish(3);
                return;
            }
            goto(19);
        }
        19 => {
            let Some(code) = done(sp, 0) else {
                return;
            };
            if code >= 0 {
                sw(g::mkdir_fd, code);
                goto(20);
            } else {
                match code {
                    -3 => finish(7),
                    -2 => finish(6),
                    _ => finish(3),
                }
            }
        }
        20 => {
            if sceMcWrite(&[lw(g::mkdir_fd) as i64, 0x1fb29c0, 964]) != 0 {
                finish(3);
                return;
            }
            goto(21);
        }
        21 => {
            let Some(code) = done(sp, 0) else {
                return;
            };
            if code == 964 {
                goto(22);
            } else if code == -3 {
                finish(7);
            } else {
                finish(3);
            }
        }
        22 => {
            if sceMcClose(&[lw(g::mkdir_fd) as i64]) != 0 {
                finish(3);
                return;
            }
            goto(23);
        }
        23 => {
            let Some(code) = done(sp, 0) else {
                return;
            };
            if code == 0 {
                goto(24);
            } else {
                finish(3);
            }
        }
        24 => {
            if lw(0x3afe68) & 34 == 2 {
                goto(30);
                return;
            }
            sprintf(&[g::mkdir_tmpbuf as i64, 0x3ae438, 0x1fb2940, 0x1fb2ac4]);
            let mode = if lw(0x3afe68) & 2 != 0 { 2 } else { 514 };
            if sceMcOpen(&[lw(g::port) as i64, lw(g::slot) as i64, g::mkdir_tmpbuf as i64, mode]) != 0 {
                finish(3);
                return;
            }
            goto(25);
        }
        25 => {
            let Some(code) = done(sp, (sp + 4) as i64) else {
                return;
            };
            if code >= 0 {
                sw(g::mkdir_fd, code);
                goto(26);
            } else if code == -3 {
                finish(7);
            } else {
                finish(3);
            }
        }
        26 => {
            let directory = lw(0x3afe4c) << 2;
            let length = lw(directory + 0x3afe60);
            let data = lw(directory + 0x3afe58);
            if sceMcWrite(&[lw(g::mkdir_fd) as i64, data as i64, length as i64]) != 0 {
                finish(3);
                return;
            }
            goto(27);
        }
        27 => {
            let Some(code) = done(sp, 0) else {
                return;
            };
            if code == lw((lw(0x3afe4c) << 2) + 0x3afe60) {
                goto(28);
            } else if code == -3 {
                finish(7);
            } else {
                finish(3);
            }
        }
        28 => {
            if sceMcClose(&[lw(g::mkdir_fd) as i64]) != 0 {
                finish(3);
                return;
            }
            goto(29);
        }
        29 => {
            let Some(code) = done(sp, 0) else {
                return;
            };
            if code == 0 {
                goto(30);
            } else {
                finish(3);
            }
        }
        30 => {
            let protect = lw(g::mkdir_copyprotect);
            if lw(0x3afe68) & 256 != 0 && protect != 0 {
                let info = 0x1fb2dc0;
                memset(info, 0, 64);
                sh(info + 20, 47);
                if sceMcSetFileInfo(&[lw(g::port) as i64, lw(g::slot) as i64, 0x1fb2940, info as i64, 4]) != 0 {
                    finish(3);
                    return;
                }
                goto(31);
                return;
            }
            finish(1);
        }
        31 => {
            let Some(code) = done(sp, 0) else {
                return;
            };
            if code == 0 {
                finish(1);
            } else if code == -2 {
                finish(6);
            } else {
                finish(3);
            }
        }
        32 => {
            if sceMcOpen(&[lw(g::port) as i64, lw(g::slot) as i64, 0x1fb2fc0, 1]) != 0 {
                finish(3);
                return;
            }
            goto(33);
        }
        33 => {
            let Some(code) = done(sp, 0) else {
                return;
            };
            if code < 0 {
                match code {
                    -2 => finish(6),
                    -4 => finish(5),
                    _ => finish(if code >= -9 { 3 } else { 4 }),
                }
            } else {
                let wanted = lw(0x3afe70);
                sw(g::read_fd, code);
                if wanted <= 0 {
                    goto(36);
                } else {
                    goto(34);
                }
            }
        }
        34 => {
            if sceMcSeek(&[lw(g::read_fd) as i64, lw(0x3afe70) as i64, 0]) != 0 {
                finish(3);
                return;
            }
            goto(35);
        }
        35 => {
            let Some(code) = done(sp, 0) else {
                return;
            };
            if code == lw(0x3afe70) {
                goto(36);
            } else if code == -2 {
                finish(6);
            } else {
                finish(3);
            }
        }
        36 => {
            if sceMcRead(&[lw(g::read_fd) as i64, lw(g::read_buffer) as i64, lw(g::request_read_len) as i64]) != 0 {
                finish(3);
                return;
            }
            goto(37);
        }
        37 => {
            let Some(code) = done(sp, 0) else {
                return;
            };
            if code < 0 {
                match code {
                    -2 | -1 => finish(6),
                    _ => finish(if code < -9 { 4 } else { 3 }),
                }
            } else {
                sw(0x3afe6c, code);
                goto(38);
            }
        }
        38 => {
            if sceMcClose(&[lw(g::read_fd) as i64]) != 0 {
                finish(3);
                return;
            }
            goto(39);
        }
        39 => {
            let Some(code) = done(sp, 0) else {
                return;
            };
            match code {
                -2 | -1 => finish(6),
                0 => {
                    let read = lw(0x3afe6c);
                    let asked = lw(g::request_read_len);
                    sw(g::mcardstate, 0);
                    sw(g::mcardresult, if read ^ asked == 0 { 1 } else { 2 });
                }
                _ => finish(if code < -9 { 4 } else { 3 }),
            }
        }
        40 => {
            let mode = if lw(g::write_overwriteok) == 0 { 514 } else { 2 };
            if sceMcOpen(&[lw(g::port) as i64, lw(g::slot) as i64, 0x1fb3040, mode]) != 0 {
                finish(3);
                return;
            }
            goto(41);
        }
        41 => {
            let Some(code) = done(sp, 0) else {
                return;
            };
            let offset = lw(0x3afe74);
            if code < 0 {
                match code {
                    -10 => finish(4),
                    -3 => finish(7),
                    -2 => finish(6),
                    _ => finish(3),
                }
            } else {
                sw(g::write_fd, code);
                if offset > 0 {
                    goto(42);
                } else {
                    goto(44);
                }
            }
        }
        42 => {
            if sceMcSeek(&[lw(g::write_fd) as i64, lw(0x3afe74) as i64, 0]) != 0 {
                finish(3);
                return;
            }
            goto(43);
        }
        43 => {
            let Some(code) = done(sp, 0) else {
                return;
            };
            if code == lw(0x3afe74) {
                goto(44);
            } else if code == -2 {
                finish(6);
            } else {
                finish(3);
            }
        }
        44 => {
            if sceMcWrite(&[lw(g::write_fd) as i64, lw(g::write_buffer) as i64, lw(g::write_len) as i64]) != 0 {
                finish(3);
                return;
            }
            goto(45);
        }
        45 => {
            let Some(code) = done(sp, 0) else {
                return;
            };
            if code == lw(g::write_len) {
                goto(46);
                return;
            }
            match code {
                -10 => finish(4),
                -3 => finish(7),
                -2 => finish(6),
                _ => finish(3),
            }
        }
        46 => {
            if sceMcClose(&[lw(g::write_fd) as i64]) != 0 {
                finish(3);
                return;
            }
            goto(47);
        }
        47 => {
            let Some(code) = done(sp, 0) else {
                return;
            };
            match code {
                -2 | -1 => finish(6),
                0 => finish(1),
                _ => finish(3),
            }
        }
        48 => {
            if sceMcDelete(&[lw(g::port) as i64, lw(g::slot) as i64, 0x1fb30c0]) != 0 {
                finish(3);
                return;
            }
            goto(49);
        }
        _ => {
            let Some(code) = done(sp, 0) else {
                return;
            };
            match code {
                -4 => finish(5),
                0 => finish(1),
                _ => finish(3),
            }
        }
    }
}
