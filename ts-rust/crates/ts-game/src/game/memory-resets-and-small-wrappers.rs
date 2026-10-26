//! The start-up of the memory areas and of the objects that take memory from them, the deleting of props that keep a slot, and small functions that call one other.

use crate::*;

/// Restart the table of pick-up positions (@ 0x272ab8).
pub fn pickupposReset() {
    pickupposRestart();
}

/// Delete the prop `prop` of the map maker (@ 0x27c1c0).
pub fn propEditDelete(prop: i32) {
    propDelete(prop);
}

/// Set the title of the page that is current in the menu (the page number at 0x3aff2c; the titles are at 0x1fc1308, 8 bytes each) and make the title of the menu tree (@ 0x2301f8).
pub fn pageSetTitle(title: i32) {
    sw((lw(0x3aff2c) << 3) + 0x1fc1308, title);
    makemtreeTitle();
}

/// The time of the profiler: the game time (@ 0x2b62a0).
pub fn profileTimeGet() -> f32 {
    timeGet()
}

/// Free the texture buffer of the object model `model`: its texture number (+36) is cleared and the buffer given at +40 is reset and taken from the buffer table (@ 0x25edb8).
pub fn obFreeTextures(model: i32) {
    sw(model + 36, 0);
    emResetTexBuffer(lw(model + 40));
    sw(((lw(model + 40) - 1) << 2) + g::tbuffers, 0);
}

/// Get memory for the bullets (0x35f0 bytes of the level area, kept at 0x3b0138) and restart them (@ 0x2892a8).
pub fn bulletReset() {
    let memory = memAlloc(0x35f0, 1);
    sw(0x3b0138, memory);
    bulletRestart();
}

/// Get memory for the characters (4 records of 0x1210 bytes on level 102, else 108) and restart them (@ 0x284ff8).
pub fn chrReset() {
    let size = (if (lw(g::lvlevelnum) ^ 102) == 0 { 4 } else { 108 }) * 0x1210;
    let memory = memAlloc(size, 1);
    sw(g::chrinfo, memory);
    chrRestart();
}

/// The height of the text `text` in the current font (@ 0x202a60).
pub fn textHeight(text: i32) -> i32 {
    let frame = Frame::new(16);
    let sp = frame.sp();
    textSize(sp, sp | 4, text);
    lw(sp + 4)
}

/// The width of the text `text` in the current font (@ 0x202a38).
pub fn textWidth(text: i32) -> i32 {
    let frame = Frame::new(16);
    let sp = frame.sp();
    textSize(sp, sp | 4, text);
    lw(sp)
}

/// Delete the pick-up prop `prop` (0 does nothing): the slot of its data (+92 of its state) is freed in `tmpPickupDataUsed` (@ 0x273618).
pub fn pickupPropDelete(prop: i32) {
    if prop == 0 {
        return;
    }
    let state = lw(prop + 352);
    if state != 0 {
        let slot = lw(state + 92);
        if slot != -1 {
            sw((slot << 2) + g::tmpPickupDataUsed, 0);
        }
    }
    propDelete(prop);
}

/// Delete the corpse prop `prop` (0 does nothing): the slot of its data (+0 of its state) is freed in `corpseDataUsed` (@ 0x273a80).
pub fn propDeleteCorpse(prop: i32) {
    if prop == 0 {
        return;
    }
    let state = lw(prop + 352);
    if state != 0 {
        let slot = lw(state);
        if slot != -1 {
            sw((slot << 2) + g::corpseDataUsed, 0);
        }
    }
    propDelete(prop);
}

/// Delete the special prop `prop` (0 does nothing): the slot of its data (+0 of its state) is freed in `tmpSpecialDataUsed` (@ 0x2783c0).
pub fn specialPropDelete(prop: i32) {
    if prop == 0 {
        return;
    }
    let state = lw(prop + 352);
    if state != 0 {
        let slot = lw(state);
        if slot != -1 {
            sw((slot << 2) + g::tmpSpecialDataUsed, 0);
        }
    }
    propDelete(prop);
}

/// Get the buffer of the memory debugger of kind `kind` (1 or 2: a record of 0x40000 or, for 2, 0x200000 bytes, with twice that of memory) and restart it (@ 0x201ef8).
pub fn memdbReset(kind: i32) {
    if ((kind - 1) as u32) < 2 {
        let length = if (kind ^ 2) == 0 { 0x200000 } else { 0x40000 };
        sw(g::buflen, length);
        let memory = memAlloc(length << 1, 1);
        sw(g::memdbbuffer, memory);
    }
    memdbRestart(kind);
}

/// Get and clear the memory for the tiles of the map maker (0x2610 bytes, kept at 0x3b0020) and clear the count at 0x3b0024 (@ 0x248eb0).
pub fn mmResetTiles() {
    let memory = memAlloc(0x2610, 1);
    sw(0x3b0020, memory);
    memset(memory, 0, 0x2610);
    sw(0x3b0024, 0);
}

/// Get and clear the memory of the level special props (3712 bytes) and restart them (@ 0x277ba8).
pub fn specialReset() {
    let memory = memAlloc(3712, 1);
    sw(g::levelspecials, memory);
    memset(memory, 0, 3712);
    specialRestart();
}

/// Get the memory for the directory of the map files (the largest size on level 103, else the size of the directory) and reset the directory (@ 0x253950).
pub fn mmfileReset() {
    let size = if lw(g::lvlevelnum) != 103 { mmfileDirSize() } else { mmfileMaxSize() };
    let memory = memAlloc(size, 1);
    sw(g::mapdir1, memory);
    mmfileDirReset();
}

/// Add the file of the sound `sound` (records of 36 bytes at `sounds`, the file name at +4) to the files to preload and mark the sound (flag 128 of its halfword at +22) (@ 0x205a08).
pub fn soundAddPreLoad(sound: i32) {
    let record = sound * 36 + g::sounds;
    preloadAddFile(lw(record + 4), 0);
    sh(record + 22, lhu(record + 22) | 128);
}

/// Clear the first texture buffer record and reset the four texture buffers (@ 0x25eb98).
pub fn obResetTextures() {
    let buffers = g::tbuffers;
    sw(g::tbuffers, 0);
    sw(buffers + 12, 0);
    sw(buffers + 4, 0);
    sw(buffers + 8, 0);
    emResetTexBuffer(1);
    emResetTexBuffer(2);
    emResetTexBuffer(3);
    emResetTexBuffer(4);
}

/// Get and clear the memory of the object instances (328 bytes each: 20 on level 102, else 300) and restart them (@ 0x25d198).
pub fn obReset() {
    let count = if (lw(g::lvlevelnum) ^ 102) == 0 { 20 } else { 300 };
    sw(g::max_instances, count);
    let memory = memAlloc(count * 328, 1);
    let count = lw(g::max_instances);
    sw(g::obinstances, memory);
    memset(memory, 0, count * 328 + 15 & -16);
    obRestart();
}

/// Set up the memory areas: the level area is the buffer after its first 0x72b000 bytes (all of it free), cleared, with the local area growing down from its end; the memory
/// marks are put and the allocator restarted (@ 0x201bb0).
pub fn memReset() {
    let buffer = lw(g::memBuffer);
    let size = lw(g::memBufferSize);
    let skip = 0x72b000;
    let start = buffer + skip;
    let end = buffer + size;
    sw(g::mem_level_free, end - start);
    sw(g::mem_level_at, start);
    sw(g::mem_level_end, end);
    memset(start, 0, size - skip);
    let end = lw(g::mem_level_end);
    let at = lw(g::mem_level_at);
    sw(g::mem_local_at, end);
    memMark(at, end);
    memRestart();
}

/// Clear the four front-end definitions (11 words each, at `frontdefs`) and give each its number, a table at 0x1fc01d0 + 64 each and a buffer at 0x1fbd1d0 + 3072 each; then
/// select the first and restart (@ 0x22caa8).
pub fn frontReset() {
    let mut def = g::frontdefs;
    memset(def, 0, 176);
    let mut buffer = 0x1fbd1d0;
    let mut table = 0x1fc01d0;
    let mut i = 0;
    loop {
        sw(def, i);
        sw(def + 4, table);
        i += 1;
        sw(def + 8, buffer);
        table += 64;
        buffer += 3072;
        def += 44;
        if i >= 4 {
            break;
        }
    }
    frontSet(0);
    frontRestart();
}

/// Allocate `size` bytes of area `area` with the start put on a multiple of 64 (the free space of the area goes down by the bytes skipped) (@ 0x201d38).
pub fn memAllocAlign64(size: i32, area: i32) -> i32 {
    if area == 1 {
        let at = lw(g::mem_level_at);
        let free = lw(g::mem_level_free);
        let aligned = at + 63 & -64;
        sw(g::mem_level_at, aligned);
        sw(g::mem_level_free, free - (aligned - at));
    } else if area == 2 {
        let at = lw(g::mem_eternal_at);
        let free = lw(g::mem_eternal_free);
        let aligned = at + 63 & -64;
        sw(g::mem_eternal_at, aligned);
        sw(g::mem_eternal_free, free - (aligned - at));
    }
    memAlloc(size, area)
}

/// Get the memory of the room lights (28 bytes for each room and one more) and of the overrides (48 bytes for each bone of the models of the rooms) and restart the lights (@ 0x2a7998).
pub fn roomlightReset() {
    let memory = memAlloc((lw(g::bgNumRooms) + 1) * 28 + 15 & -16, 1);
    let rooms = lw(g::bgNumRooms);
    let mut bones = 0;
    sw(g::roomlights, memory);
    let mut i = 0;
    if rooms > 0 {
        let props = lw(g::roomprops);
        loop {
            let at = (i << 2) + props;
            i += 1;
            bones += lw(lw(lw(lw(at + 4) + 32)));
            if i >= rooms {
                break;
            }
        }
    }
    let overrides = memAlloc(bones * 48, 1);
    sw(g::overrides, overrides);
    roomlightRestart();
}

/// Find the texture numbers of the fonts (in the `fonts` table: 40, 64, 88 and, but not on level 104, 16 and 112 bytes in), make the texture number kept at +88 current (at 0x3afe18), clear the font
/// flags, select font 1 and make the first font current (@ 0x202010).
pub fn textRestart() {
    let fonts = g::fonts;
    let font07 = preloadGetTexName(lw(g::s_font07));
    let news_name = lw(g::s_newsfont);
    sw(fonts + 40, lw(font07 + 20));
    let news = preloadGetTexName(news_name);
    let square_name = lw(g::s_squarefont);
    sw(fonts + 64, lw(news + 20));
    let square = preloadGetTexName(square_name);
    let level = lw(g::lvlevelnum);
    sw(fonts + 88, lw(square + 20));
    if level != 104 {
        let font09 = preloadGetTexName(lw(g::s_font09));
        let itcka_name = lw(g::s_itckabelfont);
        sw(fonts + 16, lw(font09 + 20));
        let itcka = preloadGetTexName(itcka_name);
        sw(fonts + 112, lw(itcka + 20));
    }
    let texture = lw(fonts + 88);
    sb(g::fontflags, 0);
    sw(0x3afe18, texture);
    textSelect(1);
    sw(g::curfont, fonts);
}

/// Get `size` bytes for the wall data of a prop from the chain of pages (0x1010 bytes each from the level area, the first at 0x1fd38c0; a page holds 0x1000 and its used count is
/// at +4): the first page with room is used, a new one is added at the end of the chain when there is none; 0 for a size past 0x1000 (@ 0x266da8).
pub fn propAllocWallMem(size: i32) -> i32 {
    let mut page = 0x1fd38c0;
    if size < 0x1001 {
        let mut used;
        loop {
            used = lw(page + 4);
            if (0x1000 - used) >= size {
                break;
            }
            let next = lw(page);
            if next == 0 {
                let memory = memAlloc(0x1010, 1);
                sw(page, memory);
                page = memory;
                sw(page, 0);
                sw(page + 4, 0);
            } else {
                page = next;
            }
        }
        let at = page + (used + 8);
        sw(page + 4, used + size);
        at
    } else {
        0
    }
}

/// The combination (of the 33) with the lowest rank (the first halfword of its record) among those the player `player` may switch to; 0 when none ranks below 100 (@ 0x28d920).
pub fn getmostpowerfulcombo(player: i32) -> i32 {
    let mut best = 0;
    let mut rank = 100;
    let mut entry = GUNCOMBOS;
    let mut i = 0;
    loop {
        if lh(entry) < rank && gunShouldSwitchTo(player, i) != 0 {
            rank = lh(entry);
            best = i;
        }
        i += 1;
        entry += 8;
        if i >= combo_count() {
            break;
        }
    }
    best
}

/// The set-up of level `level`: none for levels 3, 4 and 7, the prison's for 1, and else the story's when the game mode is 7 and the arcade's if not (@ 0x227b90).
pub fn lvGetLevelSetup(level: i32) -> i32 {
    if level >= 5 {
        if level == 7 {
            return 0;
        }
    } else {
        if level >= 3 {
            return 0;
        }
        if level == 1 {
            return g::setup_prison;
        }
    }
    if lw(g::curgameparms + 72) != 7 {
        lvGetARCADESetup(level)
    } else {
        lvGetSTORYSetup(level)
    }
}
