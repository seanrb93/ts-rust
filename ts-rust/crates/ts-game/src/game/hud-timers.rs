//! The HUD timers.

use crate::*;

/// Count down the HUD timers of the current player: the ring angle (0x3b0190, kept below 360, only for player 0), the counter at +268 (reset to 600 in
/// state 4), the one at +272 and the one at +276 that runs up to the limit at +280 and then clears; then the HUD message (@ 0x2b0e80).
pub fn hudTick() {
    let player = lw(g::currentplayer);
    let number = lw(player);
    let hud = number * 352 + g::hudparms;
    if number == 0 {
        let mut angle = ps2float::add(lwc1(0x3b0190), per_frame(lwc1(0x3ad780)));
        swc1(0x3b0190, angle);
        while 360.0 <= angle {
            angle = ps2float::sub(angle, 360.0);
            swc1(0x3b0190, angle);
        }
    }
    let frames = lw(g::updateframesi);
    let timer_a = lw(hud + 272);
    let timer_b = lw(hud + 276);
    let state = lw(player + 296);
    if state != 4 {
        let left = lw(hud + 268) - frames;
        sw(hud + 268, left);
        if left < 0 {
            sw(hud + 268, 0);
        }
    } else {
        sw(hud + 268, 600);
    }
    let left = timer_a - frames;
    sw(hud + 272, left);
    if left < 0 {
        sw(hud + 272, 0);
    }
    if timer_b != 0 {
        let limit = lw(hud + 280);
        let now = timer_b + frames;
        if timer_b < limit {
            sw(hud + 276, now);
            if now < limit {
                hudMessageTick(lw(player));
                return;
            }
        }
        sw(hud + 276, 0);
    }
    hudMessageTick(lw(player));
}
