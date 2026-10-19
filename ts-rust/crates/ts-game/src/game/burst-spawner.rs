//! Starting a burst of particles.

use super::particle_spawners::random_as_float;
use crate::*;

/// Start a burst (a group of kind 4) of the particle type `kind` at the point `at` with the speed `speed`: for each particle of the game's number, a position (16 bytes a
/// particle from +928), a random spread in each axis from the info (104 bytes each, the spread sizes at +68 to +76, centred) and a random time (@ 0x2983e0).
pub fn particleNewBurst(owner: i32, kind: i32, at: i32, flag: i32, speed: f32) {
    let frame = Frame::new(224);
    let _sp = frame.sp();
    let infos = g::particleinfos;
    let group = getgroup(4);
    sw(group + 12, 0);
    sw(group + 8, (flag != 0) as i32);
    sw(group + 4, kind);
    let mut i = 0;
    sw(group + 44, 0);
    let rec = infos + kind * 104;
    sw(group + 20, owner);
    let count_kind = lw(rec + 8);
    swc1(group + 16, lwc1(rec + 12));
    let count = particleGetNum(count_kind);
    let half = 0.5_f32;
    let offset = lw(group + 4) * 104;
    sw(group + 24, count);
    let info = infos + offset;
    swc1(group + 52, lwc1(info + 100));
    swc1(group + 28, lwc1(at));
    swc1(group + 32, lwc1(at + 4));
    let z = lwc1(at + 8);
    swc1(group + 40, speed);
    swc1(group + 36, z);
    let spread_x = lwc1(info + 68);
    let spread_y = lwc1(info + 72);
    let spread_z = lwc1(info + 76);
    let (centre_x, centre_y, centre_z) = (spread_x * half, spread_y * half, spread_z * half);
    if count > 0 {
        loop {
            let place = i << 4;
            swc1(group + 928 + place, lwc1(at));
            swc1(group + 932 + place, lwc1(at + 4));
            swc1(group + 936 + place, lwc1(at + 8));
            let r = random_as_float();
            let base = lw(group + 4) * 104 + (g::particleinfos + 56);
            let to = i * 12 + (group + 2528);
            swc1(to, lwc1(base) + (r * 2.3283064e-10_f32 * spread_x - centre_x));
            let r = random_as_float();
            let base = lw(group + 4) * 104 + (g::particleinfos + 60);
            let to = i * 12 + (group + 2532);
            swc1(to, lwc1(base) + (r * 2.3283064e-10_f32 * spread_y - centre_y));
            let r = random_as_float();
            let base = lw(group + 4) * 104 + (g::particleinfos + 64);
            let to = i * 12 + (group + 2536);
            swc1(to, lwc1(base) + (r * 2.3283064e-10_f32 * spread_z - centre_z));
            let r = random_as_float();
            let to = group + 3728 + (i << 2);
            let kind_now = lw(group + 4);
            i += 1;
            swc1(to, (r * 2.3283064e-10_f32 - 0.5_f32) * lwc1(kind_now * 104 + (g::particleinfos + 36)));
            if i >= lw(group + 24) {
                break;
            }
        }
    }
}
