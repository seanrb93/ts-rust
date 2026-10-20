//! The props of the space level that bob and turn: the big ship, the three smaller ships and the wheels of the station.

use crate::*;

/// The big ship: its height (+52) is its rest height (+40) and a sine of the game's time, scaled (@ 0x27af08).
pub fn bigshipPropTick(prop: i32) {
    let time = lw(g::gameframe) as f32;
    swc1(prop + 52, lwc1(prop + 40) + lwc1((((time * lwc1(0x3acf5c) * lwc1(0x3acf60) + 0.5_f32) as i32 & 2047) << 2) + g::g_SinTable) * lwc1(0x3acf64));
}

/// The third ship: like the big ship with one constant for the rate and the size (@ 0x27aeb0).
pub fn spaceship3PropTick(prop: i32) {
    let scale = lwc1(0x3acf54);
    let time = lw(g::gameframe) as f32;
    swc1(prop + 52, lwc1(prop + 40) + lwc1((((time * scale * lwc1(0x3acf58) + 0.5_f32) as i32 & 2047) << 2) + g::g_SinTable) * scale);
}

/// The outer wheels: the height is the rest height, a constant and a sine (@ 0x27aa18).
pub fn outerwheelsPropTick(prop: i32) {
    let time = lw(g::gameframe) as f32;
    swc1(prop + 52, lwc1(prop + 40) + lwc1(0x3acf10) + lwc1((((time * lwc1(0x3acf08) * lwc1(0x3acf0c) + 0.5_f32) as i32 & 2047) << 2) + g::g_SinTable) * lwc1(0x3acf14));
}

/// The inner wheels: a sine of the rest height and the time, whose sign follows the sign of the rest height (@ 0x27aa80).
pub fn innerwheelsPropTick(prop: i32) {
    let rest = lwc1(prop + 40);
    let mut size = rest;
    let mut sign = 1.0_f32;
    let time = lw(g::gameframe) as f32 * 2.5_f32;
    if rest < 0.0_f32 {
        size = -rest;
        sign = -1.0_f32;
    }
    swc1(prop + 52, rest + lwc1(((((size * 3.5_f32 + time * lwc1(0x3acf18)) * lwc1(0x3acf20) + 0.5_f32) as i32 & 2047) << 2) + g::g_SinTable) * lwc1(0x3acf1c) * sign);
}

/// The second ship: it moves in a Lissajous curve round its rest place (+36, +40, +44) by three sines of the game's time (@ 0x27add8).
pub fn spaceship2PropTick(prop: i32) {
    let time = lw(g::gameframe) as f32;
    let scale = lwc1(0x3acf48);
    let table = g::g_SinTable;
    let half = 0.5_f32;
    let rest_y = lwc1(prop + 40);
    let rest_z = lwc1(prop + 44);
    let y_at = (((time * lwc1(0x3acf4c) * scale + half) as i32 & 2047) << 2) + table;
    let z_at = (((time * lwc1(0x3acf50) * scale + half) as i32 & 2047) << 2) + table;
    swc1(prop + 48, lwc1(prop + 36) + lwc1((((time * lwc1(0x3acf44) * scale + half) as i32 & 2047) << 2) + table) * 3.0_f32);
    swc1(prop + 52, rest_y + lwc1(y_at));
    swc1(prop + 56, rest_z + lwc1(z_at) * 10.0_f32);
}
