//! Starting an animation on a character.

use crate::*;

/// Start the animation `anim` on the character `chr`, blended in over `blend` (0 for at once) with `speed_param` kept at +204 or +140 of its animation state: an animation that
/// is locked (flag 8) and not yet finished is not interrupted unless the new one has flag 32; flinch animations make a sound; some animations are turned to the family of the
/// character's kind and its two slots (+156 and +220 of the state) say which side a step leads with; the state's old animation (the second slot at +160) is kept when a new one
/// is blended in, and the position of the root is moved by the offset of the first frame. Nothing is done while the frame time is 0 (@ 0x214d68).
pub fn setAnim(chr: i32, anim: i32, blend: f32, extra: f32) {
    let frame = Frame::new(128);
    let sp = frame.sp();
    let mut anim = anim;
    let mut blend = blend;
    let mut info = lw(chr + 352);
    let info_first = info;
    let scale = lwc1(info + 2840);
    if lw(g::updateframesi) == 0 {
        return;
    }
    let mut state = lw(chr + 32);
    let current = lw(state + 96);
    if current == 1 {
        return;
    }
    let anims = g::pAnimations;
    let current_for_flags;
    if (lw(lw((current << 2) + anims) + 12) & 8) == 0 {
        current_for_flags = lw(state + 96);
    } else if lw(state + 152) != 0 {
        current_for_flags = lw(state + 96);
    } else {
        if (lw(lw((anim << 2) + anims) + 12) & 32) == 0 {
            return;
        }
        current_for_flags = lw(state + 96);
    }
    let mut relative = anim - 460;
    if (lw(lw((current_for_flags << 2) + anims) + 12) & 16) != 0 {
        blend = 0.0_f32;
    }
    let flinch = if (relative as u32) < 5 {
        true
    } else {
        relative = anim - 443;
        ((anim - 444) as u32) < 3
    };
    if flinch {
        let sound = lw((lh(info_first + 4) << 2) + g::flinchsfx);
        soundDelayStartEx(sound, chr + 48, 20, 0);
        info = lw(chr + 352);
        state = lw(chr + 32);
        relative = anim - 443;
    }
    let turned = anim + 38;
    if (relative as u32) < 32 {
        anim = if (lw(lw(state + 88) + 12) ^ 1) == 0 { turned } else { anim };
    }
    let kind = lw(info + 8);
    let slot: i32;
    if kind == 1 {
        if !(blend == 0.0_f32) {
            if anim == 520 || anim == 522 {
                sw(state + 220, 0);
            } else if anim == 521 || anim == 523 {
                sw(state + 220, 1);
            } else {
                sw(state + 220, -1);
            }
        } else if anim == 520 || anim == 522 {
            sw(state + 156, 0);
        } else if anim == 521 || anim == 523 {
            sw(state + 156, 1);
        } else {
            sw(state + 156, -1);
        }
        slot = anim;
    } else if kind < 2 {
        if kind == 0 {
            let in_family = ((anim - 3) as u32) < 11 || ((anim - 113) as u32) < 11;
            slot = if in_family { lw(chr + 344) * 11 + anim } else { anim };
            let side = if anim == 12 || anim == 122 {
                0
            } else if anim == 13 || anim == 123 {
                1
            } else {
                -1
            };
            if !(blend == 0.0_f32) {
                sw(state + 220, side);
            } else {
                sw(state + 156, side);
            }
        } else {
            slot = -1;
        }
    } else if kind == 2 {
        if !(blend == 0.0_f32) {
            if anim == 560 {
                sw(state + 220, 0);
            } else if anim == 561 {
                sw(state + 220, 1);
            } else {
                sw(state + 220, -1);
            }
        } else if anim == 560 {
            sw(state + 156, 0);
        } else if anim == 561 {
            sw(state + 156, 1);
        } else {
            sw(state + 156, -1);
        }
        slot = anim;
    } else {
        slot = -1;
    }
    if ((anim - 566) as u32) < 41 {
        sw(info + 2968, 0);
        swc1(info + 2972, lwc1(chr + 80));
    }
    let data_at = slot << 2;
    let state_now;
    if !(blend == 0.0_f32) {
        let old = lw(state + 160);
        sw(state + 236, 1);
        let mut skip_copy = false;
        if old != 0 {
            if old == slot {
                skip_copy = true;
            } else {
                let b = ld(state + 168);
                let c = ld(state + 176);
                let d = ld(state + 184);
                sd(state + 96, ld(state + 160));
                sd(state + 104, b);
                sd(state + 112, c);
                sd(state + 120, d);
                let b = ld(state + 200);
                let c = ld(state + 208);
                let d = ld(state + 216);
                sd(state + 128, ld(state + 192));
                sd(state + 136, b);
                sd(state + 144, c);
                sd(state + 152, d);
                state = lw(chr + 32);
            }
        }
        let _ = skip_copy;
        sw(state + 160, slot);
        let data = lw(data_at + anims);
        swc1(state + 224, blend);
        swc1(state + 204, extra);
        sw(state + 232, 0);
        sw(state + 164, 0);
        sw(state + 212, 0);
        sw(state + 216, 0);
        extractAnimData(state, data, 0, 0, sp, scale);
        state_now = lw(chr + 32);
        sw(state_now + 132, 0);
        let x = lwc1(sp);
        swc1(state_now + 168, x);
        swc1(state_now + 180, x);
        swc1(state_now + 172, lwc1(sp + 4));
        let z = lwc1(sp + 8);
        swc1(state_now + 176, z);
        swc1(state_now + 188, z);
    } else {
        sw(state + 96, slot);
        let data = lw(data_at + anims);
        swc1(state + 224, blend);
        swc1(state + 140, extra);
        swc1(state + 148, 0.0_f32);
        swc1(state + 232, 0.0_f32);
        swc1(state + 100, 0.0_f32);
        sw(state + 152, 0);
        extractAnimData(state, data, 0, 0, sp, scale);
        state_now = lw(chr + 32);
        let y = lwc1(sp + 4);
        let old_y = lwc1(state_now + 120);
        sw(state_now + 160, 0);
        swc1(state_now + 132, y - old_y);
        let x = lwc1(sp);
        swc1(state_now + 104, x);
        swc1(state_now + 116, x);
        let y = lwc1(sp + 4);
        swc1(state_now + 108, y);
        let z = lwc1(sp + 8);
        swc1(state_now + 120, y);
        swc1(state_now + 112, z);
        swc1(state_now + 124, z);
    }
    swc1(chr + 52, lwc1(chr + 52) - lwc1(state_now + 132));
    let f2 = lwc1(state_now + 108);
    let f3 = lwc1(state_now + 112);
    let f4 = lwc1(state_now + 168);
    let f5 = lwc1(state_now + 172);
    let f0 = lwc1(state_now + 176);
    swc1(state_now + 116, lwc1(state_now + 104));
    swc1(state_now + 188, f0);
    swc1(state_now + 120, f2);
    swc1(state_now + 124, f3);
    swc1(state_now + 180, f4);
    swc1(state_now + 184, f5);
}
