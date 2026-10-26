//! The moving background and the glow of the goodies in the front end.

use crate::*;

/// The tick of the front end's background: the two scroll places in y (`frontbgtranslatey1` and `2`) and the time move by the game's steps, the places going back by 1 once they
/// reach 1, the place in x is set and the third one in x moves left by a double step, to not less than -1 (@ 0x22b600).
pub fn frontfxBgTick() {
    let first = ps2float::add(lwc1(g::frontbgtranslatey1), menu_scale(lwc1(0x3ac938)));
    let one = 1.0_f32;
    let time = ps2float::add(lwc1(g::frontbgtime), menu_scale(lwc1(0x3ac93c)));
    swc1(g::frontbgtranslatey1, first);
    swc1(g::frontbgtime, time);
    if one <= first {
        swc1(g::frontbgtranslatey1, ps2float::sub(first, one));
    }
    let second = ps2float::add(lwc1(g::frontbgtranslatey2), menu_scale(lwc1(0x3ac940)));
    swc1(g::frontbgtranslatey2, second);
    if one <= second {
        swc1(g::frontbgtranslatey2, ps2float::sub(second, one));
    }
    let third = lwc1(g::frontbgtranslatex3);
    swc1(g::frontbgtranslatex1, lwc1(0x3ac944));
    let left = dpsub(fptodp(third), menu_dp_scale(ld(0x3a18c0)));
    let moved = dptofp(left);
    let limit = -1.0_f32;
    swc1(g::frontbgtranslatex3, moved);
    if moved <= limit {
        swc1(g::frontbgtranslatex3, limit);
    }
}

/// The tick of the goodies' glow in the front end: the time goes on, the place of the glow (`frontfxlpos`) moves right by 3 and goes back to -64 at 596, and the alpha of each of
/// the 512 columns is 80 times the square of how close it is to the glow (nothing past 35 away) (@ 0x22be98).
pub fn frontfxGoodiesTick() {
    let time = ps2float::add(lwc1(g::frontfxgoodiestime), menu_scale(lwc1(0x3ac968)));
    let mut place = lw(g::frontfxlpos) + menu_steps_x(3);
    sw(g::frontfxlpos, place);
    swc1(g::frontfxgoodiestime, time);
    if place >= 596 {
        place = -64;
        sw(g::frontfxlpos, place);
    }
    let mut column = 0;
    loop {
        let d = column - place;
        let distance = ((d * d) as f32).abs().psqrt();
        let glow = if 35.0_f32 < distance {
            0.0_f32
        } else {
            let t = (ps2float::sub(35.0_f32, distance)).pdiv(35.0_f32);
            ps2float::mul(t, t)
        };
        sb(column + g::frontfxalpha, ps2float::add(ps2float::mul(glow, 8e+01_f32), 0.0_f32) as i32);
        column += 1;
        if column >= 512 {
            break;
        }
    }
}
