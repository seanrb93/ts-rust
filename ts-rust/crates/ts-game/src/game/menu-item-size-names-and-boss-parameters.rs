//! The size of a menu item, the names in the character select, the level list of the red front end, a free block of the map files, and the program's parameters.

use crate::*;

/// Work out the size of the menu item `item` into `size` (the width at +0, the height at +4): its text is the string number (below 1246, taken from the current language) or
/// the address at +0 of the item, or the text a function makes from it when it is not a text (@ 0x20cac0).
pub fn menuitemCalcSize(item: i32, size: i32) {
    let mut text = lw(item);
    let is_text = istext(text);
    let mut numbered = (text < 1246) as i32;
    if is_text == 0 {
        text = gettextfromfunction(text, item);
        numbered = (text < 1246) as i32;
    }
    let language = lw(g::cur_language);
    if numbered != 0 {
        text = lw((text << 2) + lw((language << 2) + g::languages));
    }
    textSize(size, size + 4, text);
}

/// The index of the program parameter `name` (compared without case; the first is the program's name), kept at `_fbss`, or 0 when there is none (@ 0x2000c0).
pub fn bossCheckParm(name: i32) -> i32 {
    let mut i = 1;
    if i < lw(g::myargc) {
        let mut argv = lw(g::myargv);
        loop {
            let compared = strcasecmp(name, lw((i << 2) + argv));
            let count = lw(g::myargc);
            if compared == 0 {
                sw(g::_fbss, i);
                return i;
            }
            i += 1;
            argv = lw(g::myargv);
            if i >= count {
                break;
            }
        }
    }
    sw(g::_fbss, 0);
    0
}

/// Draw the name `text` of the character select of the box `box_number` of the page `page` (the box's y at +2 of its record, from the window's top) in the middle of the window,
/// with an outline: in the menu's colours, or, for the selected one (`selected` not 0), with the arrows at its sides and the colours flashing every 32 ticks (@ 0x236f40).
pub fn drawCharName(page: i32, box_number: i32, text: i32, selected: i32) {
    let found = frontGetBox(page, box_number & 0xffff);
    if found == 0 {
        return;
    }
    let y = lw(g::window + 20) + lhu(found + 2);
    let (colour, outline);
    if selected == 0 {
        colour = lw(g::menuitem_fgcol);
        outline = lw(g::menuitem_bgcol);
    } else {
        drawSelectCharArrows(y, text);
        colour = if (lw(g::tickframenum) & 32) != 0 { 0x40787880 } else { 0x7f7f7f80 };
        outline = 0x5080;
    }
    let width = textWidth(text);
    let x = lw(g::window + 8) - ((((width as u32) >> 31) as i32 + width) >> 1);
    textOutline(x, y, text, colour, outline);
}

/// Draw the level list of the red front end: the title from `curselect` + 4, then the name of each of the levels (records of 8 bytes at `levels`, the count at 0x3aff1c) in the
/// same place (the drawing moves it), white for the selected one and dim for the others (@ 0x22ffd8).
pub fn frontredGfx() {
    let frame = Frame::new(16);
    let sp = frame.sp();
    dlSetBlend(1);
    let y = lw(g::window + 20) + 50;
    sw(sp, lw(g::window) + 50);
    sw(sp + 4, y);
    textPrint(sp, sp + 4, g::curselect + 4, 0x407f7f7f);
    if lw(0x3aff1c) > 0 {
        let mut i = 0;
        let mut selected = lw(g::curselect);
        loop {
            let name = lw((i << 3) + g::levels);
            let colour = if (i ^ selected) != 0 { 0x4040407f } else { 0x7f7f7f7f };
            i += 1;
            textPrint(sp, sp + 4, name, colour);
            if i >= lw(0x3aff1c) {
                break;
            }
            selected = lw(g::curselect);
        }
    }
}

/// The number of the first free block of the map file directory (the blocks used are the halfwords at +48 of the 40-byte records after the directory's header; the count of them at
/// +8; 100 blocks are looked at) (@ 0x253a08).
pub fn mmfileGetFreeBlock() -> i32 {
    let frame = Frame::new(112);
    let sp = frame.sp();
    memset(sp, 0, 100);
    let directory = lw(g::mapdir1);
    let mut i = 0;
    if lw(directory + 8) > 0 {
        let mut offset = 0;
        loop {
            i += 1;
            sb(sp + lhu(directory + offset + 48), 1);
            offset += 40;
            if i >= lw(directory + 8) {
                break;
            }
        }
    }
    let mut block = 0;
    if lbu(sp) != 0 {
        block += 1;
        loop {
            if block >= 100 {
                break;
            }
            if lbu(sp + block) == 0 {
                break;
            }
            block += 1;
        }
    }
    block
}
