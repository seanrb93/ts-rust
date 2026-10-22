//! Functions: loading, saving and copying files and the pack file.
//!
//! The game keeps up to 8 pack files at 0x1fb4c40 (their count is at 0x3afea8). A pack has a header of its own and a table of entries of 60 bytes: the name (48 bytes), the offset on the disc (+48), the size (+52) and the compressed size (+56, or 0 if the file is not compressed).

use crate::*;

/// Write `a2` bytes from `a1` to the file named `a0`; the game ends if it cannot be opened.
pub fn fileSave(a0: i32, a1: i32, a2: i32) {
    let fd = sceOpen(&[a0 as i64, 1538, a2 as i64]);
    if fd < 0 {
        exit(0);
    }
    sceWrite(&[fd as i64, a1 as i64, a2 as i64]);
    sceClose(&[fd as i64]);
}

/// Load the file named `a0` into new memory of the group `a1`; the size is put in `a2` if it is not 0. Gives the memory, or 0 if the file is empty.
pub fn fileLoad(a0: i32, a1: i32, a2: i32) -> i32 {
    let mut memory = 0;
    let size = fileSize(a0);
    if size > 0 {
        memory = memAllocAlign64(size + 15 & -16, a1);
        fileLoadTo(a0, memory, size);
    }
    if a2 != 0 {
        sw(a2, size);
    }
    memory
}

/// Copy the file named `a0` to a file named `a1`, through a buffer of 4 MB that is made the first time.
pub fn fileCopy(a0: i32, a1: i32) {
    let size = fileSize(a0);
    if lw(0x3afe80) == 0 {
        sw(0x3afe80, malloc(0x400000));
    }
    if size <= 0 {
        return;
    }
    fileLoadTo(a0, lw(0x3afe80), size);
    fileSave(a1, lw(0x3afe80), size);
}

/// The size of the file named `a0`, from the pack files if it is in one (a compressed file has 256 more), or else from the host; -1 if it does not exist.
pub fn fileSize(a0: i32) -> i32 {
    if lw(0x3afea8) != 0 {
        let entry = packfileSeek(a0);
        if entry != 0 {
            let size = lw(entry + 52);
            return if lw(entry + 56) != 0 { size + 256 } else { size };
        }
    }
    if lw(g::displayallfileops) != 0 {
        pause();
    }
    let fd = sceOpen(&[a0 as i64, 1]);
    if fd < 0 {
        return -1;
    }
    let size = sceLseek(&[fd as i64, 0, 2]);
    if lw(g::displayallfileops) != 0 {
        pause();
    }
    sceClose(&[fd as i64]);
    size
}

/// Load the file named `a0` to the address `a1`, `a2` is the room there. Gives the size of the file, or 0 if it is not in a pack file.
pub fn fileLoadTo(a0: i32, a1: i32, a2: i32) -> i32 {
    if lw(0x3afea8) == 0 {
        return 0;
    }
    let entry = packfileSeek(a0);
    if entry == 0 {
        return 0;
    }
    let compressed = lw(entry + 56);
    let size = lw(entry + 52);
    let sector = lw(lw(0x3afeb0) + 4) + (lw(entry + 48) >> 11);
    if compressed != 0 {
        // The compressed data goes to the end of the room and is inflated from there.
        let at = a1 + (a2 & -16) - (compressed + 15 & -16);
        fileLoadCD(sector, at, compressed);
        if lw(g::displayallfileops) != 0 {
            pause();
        }
        return fileInflate(at, lw(entry + 56), a1);
    }
    fileLoadCD(sector, a1, size);
    if lw(g::displayallfileops) != 0 {
        pause();
    }
    size
}

/// The address in memory of the file named `a0` if its pack file is all in memory and the file is not compressed and starts on 16 bytes, or else 0.
pub fn fileGetMem(a0: i32) -> i32 {
    if lw(0x3afea8) == 0 {
        return 0;
    }
    let entry = packfileSeek(a0);
    let pack = lw(0x3afeb0);
    if entry == 0 {
        return 0;
    }
    let base = lw(pack + 16);
    if base == 0 || lw(entry + 56) != 0 {
        return 0;
    }
    let offset = lw(entry + 48);
    if offset & 15 != 0 {
        return 0;
    }
    base + offset
}

/// Wait until the disc is done with what it was told, and try it again if it gave an error. The original also spins for a while between the two checks of the disc.
fn wait_for_disc() {
    while sceCdSync(&[0]) != 0 {
        for _ in 0..0x100000 {
            std::hint::black_box(());
        }
    }
}

/// Read `count` sectors of the disc from `sector` to `dest`, and ask again until the disc does it without an error. `retry_message` is the text for an error of the read.
fn read_sectors(sector: i32, count: i32, dest: i32, retry_message: i32) {
    loop {
        if sceCdRead(&[sector as i64, count as i64, dest as i64, 0x3afe90]) == 0 {
            let error = sceCdGetError(&[]);
            scePrintf(&[0x3a06c8, error as i64]);
            continue;
        }
        wait_for_disc();
        if sceCdGetError(&[]) != 0 {
            scePrintf(&[retry_message as i64]);
            continue;
        }
        break;
    }
}

/// Read the bytes `a2` from the sector `a0` of the disc to `a1`. Whole sectors are read straight to the destination in blocks of 0x2800, and the last part of a sector through a buffer at 0x1fb4440.
pub fn fileLoadCD(a0: i32, a1: i32, a2: i32) {
    let remainder = a2 & 2047;
    let mut sector = a0;
    let mut dest = a1;
    let mut left = a2 >> 11;
    sb(0x3afe91, 1);
    sb(0x3afe90, 0);
    sb(0x3afe92, 0);
    FlushCache(0);
    while left != 0 {
        let count = if left < 0x2801 { left } else { 0x2800 };
        read_sectors(sector, count, dest, 0x3a06e8);
        dest += count << 11;
        left -= count;
        sector += count;
    }
    if remainder == 0 {
        return;
    }
    read_sectors(sector, 1, 0x1fb4440, 0x3a06f8);
    bcopy(0x1fb4440, dest, remainder);
    sceSifWriteBackDCache(&[dest as i64, remainder as i64]);
}

/// Load the pack files of the start: the first four are only opened, not kept in memory.
pub fn packfileMake() {
    sw(0x3afea8, 0);
    packfileLoad(0x3a0708, 0, 2);
    packfileLoad(0x3a0720, 0, 2);
    packfileLoad(0x3a0738, 0, 2);
    packfileLoad(0x3a0750, 0, 2);
}

/// Whether the name of the entry `entry` is the one that was put at `name` (a copy of 48 bytes), compared a word at a time.
fn entry_has_name(entry: i32, name: i32) -> bool {
    let first = lw(entry);
    if first == 0 {
        return true;
    }
    if first != lw(name) {
        return false;
    }
    for word in 1..12 {
        let w = lw(entry + (word << 2));
        if w == 0 {
            return true;
        }
        if w != lw(name + (word << 2)) {
            return false;
        }
    }
    true
}

/// Find the entry of the file named `a0` in the pack files, or 0 if it is in none. A name that starts with the text at 0x3ae550 is compared without its first 6 bytes. The entry found is kept, and its pack file becomes the one in use.
pub fn packfileSeek(a0: i32) -> i32 {
    let frame = Frame::new(80);
    let sp = frame.sp();
    let name = if strcmp(a0, 0x3ae550) >= 0 { a0 + 6 } else { a0 };
    let remembered = lw(g::remember);
    if remembered != 0 && strcmp(name, remembered) == 0 {
        return lw(g::remember);
    }
    strncpy(sp, name, 48);
    for i in 0..lw(0x3afea8) {
        let pack = lw(0x1fb4c40 + (i << 2));
        let mut entry = lw(pack + 20);
        for _ in 0..lw(pack + 12) {
            if entry_has_name(entry, sp) {
                sw(g::remember, entry);
                sw(0x3afeb0, lw(0x1fb4c40 + (i << 2)));
                return entry;
            }
            entry += 60;
        }
    }
    0
}

/// Add the pack file named `a0` to the list, from the disc. If `a1` is not 0 the whole file is read to memory of the group `a2` (and `a2` 1 means a level pack that is counted at 0x3afeac), or else only its header is read and the data is loaded when it is needed. A file that is not a pack, or too big, or a list that is full, is ignored. The pack of the disc whose name is not found is searched for again until it is.
pub fn packfileLoad(a0: i32, a1: i32, a2: i32) {
    let frame = Frame::new(160);
    let sp = frame.sp();
    let mut sector = -1;
    let mut length = -1;
    let mut found = false;
    let name = a0 + 7;
    if !(a1 != 0 && a2 != 2) && lw(0x3afeac) != 0 {
        exit(-1);
    }
    while !found {
        if SCEECdSearchFile(0x3afea0, name) == 1 {
            sector = lw(0x3afea0);
            found = true;
            length = lw(0x3afea4);
        }
    }
    if lw(0x3afea8) >= 8 {
        return;
    }
    let mut buffer = 0;
    let mut file_size = 0;
    if a1 == 0 {
        fileLoadCD(sector, sp, 12);
    } else {
        file_size = lw(0x3afea4);
        if file_size <= 0 {
            return;
        }
        buffer = memAllocAlign64(file_size + 15 & -16, a2);
        fileLoadCD(sector, buffer, file_size);
        let tail = lw(buffer + 8);
        sd(sp, ld(buffer));
        sw(sp + 8, tail);
    }
    if lw(sp) != 0x4b433450 || (0x3c000_u32) < (lw(sp + 8) as u32) {
        return;
    }
    let pack = memAlloc(32, a2);
    if buffer == 0 {
        let data_size = lw(sp + 8);
        let data = memAllocAlign64(data_size + 15 & -16, 2);
        sw(pack + 20, data);
        fileLoadCD(sector + ((length - data_size) >> 11), data, data_size);
        sw(pack + 4, sector);
        sw(pack + 8, length);
        // The original stores the register s7 of its caller here, which the function never sets.
        sw(pack, 0);
        sw(pack + 16, 0);
    } else {
        let data_size = lw(sp + 8);
        sw(pack + 4, sector);
        sw(pack + 8, length);
        sw(pack + 20, buffer + file_size - data_size);
        sw(pack, -1);
        sw(pack + 16, buffer);
    }
    let count = lw(0x3afea8);
    let entries = ((lw(sp + 8) as u32) / 60) as i32;
    sw(0x1fb4c40 + (count << 2), pack);
    sw(0x3afea8, count + 1);
    sw(pack + 12, entries);
    if a1 != 0 && a2 == 1 {
        sw(0x3afeac, lw(0x3afeac) + 1);
    }
}

/// Open the picture file `a0` into the image `a1`, by its kind; the game ends if it is not a kind it can read.
pub fn ImageLoad(a0: i32, a1: i32) {
    let kind = ImageGetDetails(a0);
    sw(a1, 0);
    sb(a1 + 37, 1);
    match kind {
        1 => OpenRGB(a0, a1),
        2 | 4 | 5 | 6 | 7 => OpenXPM(a0, a1, kind),
        _ => {
            exit(1);
        }
    }
}

/// Open the picture in the download buffer into the image `a1`, by its kind; the game ends if it is not a kind it can read.
pub fn ImageLoadFromMem(a0: i32, a1: i32) {
    let kind = ImageGetDetailsFromMem(a0);
    sw(a1, 0);
    sb(a1 + 37, 1);
    match kind {
        1 => OpenRGB(g::downloadbuffer + 4, a1),
        2 | 4 | 5 | 6 | 7 => OpenXPM(g::downloadbuffer + 4, a1, kind),
        _ => {
            exit(1);
        }
    }
}
