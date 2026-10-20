//! The story end page, the stored goodies, the character select draw, the special effect draw loop, stopping the stage animations and relocating a part.

use crate::*;

/// Draw the story end page: the completed screen when the game's option flag 0x10000000 is set and the current player's kind (+296) is not 4, else the not completed one (@ 0x211990).
pub fn story_pageGfx() {
    let player = lw(g::currentplayer);
    if (lw(g::curgameparms + 80) & 0x10000000) != 0 {
        if lw(player + 296) != 4 {
            drawStoryCompleted();
            return;
        }
    }
    drawStoryNotCompleted();
}

/// Store what the goodies page shows: the unlocked cheats, the levels, the bot sets and the misc flag, and the lists of available and playable characters (@ 0x240510).
pub fn frontmpgStoreGoodies() {
    sw(0x3affa0, lw(g::unlockedcheats));
    let levels = goodiesGetLevels();
    sw(0x3affa4, levels);
    let bots = goodiesGetBotSets();
    sw(0x3affa8, bots);
    let misc = goodiesGetMisc();
    sb(0x3affac, misc);
    goodiesGetAvailableChrs(0x1fc33b0);
    goodiesGetPlayableChrs(0x1fc33c0);
}

/// Draw what comes after the character select page: the window over the whole screen with its clip, and the little characters of the teams (@ 0x237e98).
pub fn characterselect_pageDrawAfter() {
    windowSet(0, 0, 639, 223);
    let window = g::window;
    dlSetClip(lw(g::window), lw(window + 20), lw(window + 4), lw(window + 24));
    windowGfx();
    drawLittleGuys();
}

/// Draw each of the 56 special effect groups (92 bytes from `fxgroups`) by its kind: kind 10 the time splitter's hands, kind 11 its ring, -1 and the others nothing (@ 0x2abdf8).
pub fn specialfxGfx() {
    let mut group = g::fxgroups;
    let mut i = 0;
    loop {
        let kind = lw(group);
        if kind == -1 {
            i += 1;
        } else if kind == 10 {
            specialfxTimeSplitterHandsGfx(i);
            i += 1;
        } else if kind == 11 {
            specialfxTimeSplitterRadiusGfx(i);
            i += 1;
        } else {
            i += 1;
        }
        group += 92;
        if i >= 56 {
            break;
        }
    }
}

/// Bring the animations of the stage props to their end at once: for each of the players' things listed from 0x1fc1238 (the count at 0x3aff20) the animation time at +140 of the
/// model is cleared, and so is the stage prop's, whose matrices are then made with the scale constant at 0x3ac9f0 (0x3ac9f4 when the model type at +12 is 0) (@ 0x241c68).
pub fn animStopImmediately() {
    let mut count = lw(0x3aff20);
    let mut i = 0;
    if count > 0 {
        let mut offset = i << 2;
        loop {
            let thing = lw(offset + 0x1fc1238);
            let stage = lw(offset + g::stageprop);
            let model = lw(thing + 32);
            sw(model + 140, 0);
            if stage != 0 {
                let stage_model = lw(stage + 32);
                let kind = lw(lw(model + 88) + 12);
                let mut scale = lwc1(0x3ac9f0);
                sw(stage_model + 140, 0);
                if kind == 0 {
                    scale = lwc1(0x3ac9f4);
                }
                calPropMatrices(stage_model, scale);
                count = lw(0x3aff20);
            }
            i += 1;
            offset = i << 2;
            if i >= count {
                break;
            }
        }
    }
}

/// Relocate the part `part` of the model `model` and the parts that follow it: both of its point lists are transformed (`transformPart`), a child (+3) is relocated unless the part is of
/// kind 3, and the next part (+4) is taken until there is none (@ 0x263078).
pub fn partRelocate(model: i32, part: i32) {
    let mut current = part;
    loop {
        let skeleton = lw(model);
        let record = skeleton - lw(skeleton) * 80 + current * 80;
        transformPart(model, record, 0);
        transformPart(model, record, 1);
        let child = lb(record + 3);
        if child >= 0 {
            if lb(record) == 3 {
                current = lb(record + 4);
                if current < 0 {
                    break;
                }
                continue;
            }
            partRelocate(model, child);
        }
        current = lb(record + 4);
        if current < 0 {
            break;
        }
    }
}
