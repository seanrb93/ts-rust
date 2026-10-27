//! The tick of the bullets in flight.
//!
//! The bullets are an array of 50 records of 276 bytes (the pointer to it is at 0x3b0138). A record has the kind of the bullet +0 (0 for one that is not there; the kinds are 2 to 4 shots that leave a tracer, 5 to 7 shots with sprite sparks, 8 and 9 rockets, 10 to 13 grenades that explode after 3 ticks, 14 to 16 mines that stick to what they hit, 17 and 18 thrown things that bounce, and 19 a thing that sits a while), the position +24, the velocity +48 (per tick), the spin of the prop +60 to +68, the age in ticks of 60 +72, the damage +76, the prop of the bullet +96, the owner +104 (the kind: 1 and 4 a player, 2 a character) and +108 (its number), the particles of the tracer and the trails +112, +116, +204 and +208, the chance to ricochet +152, the number of bounces +212, and the character the bullet is stuck to +224 with the place on it +228 to +252.

use super::particle_spawners::random_as_float;
use crate::*;

/// What a bullet's tick ends with.
enum Exit {
    /// Nothing more.
    Done,
    /// The bullet is gone.
    Free,
    /// The bullet moves by its step, nothing hit it.
    Drift,
    /// The bullet starts over (its age is 0).
    ResetAge,
    /// The owner (a character of that number) is told where the bullet is.
    Record(i32),
}

/// The time of the frame, as the game counts it.
fn frames() -> f32 {
    lwc1(g::updateframesf)
}

/// A random number from 0 to 1.
fn uniform() -> f32 {
    ps2float::mul(random_as_float(), 2.3283064e-10_f32)
}

/// The number of one of the 6 sounds of an explosion.
fn explosion_sound() -> i32 {
    ((newrnd() as i32 as u32) % 6) as i32 + 98
}

/// Free the particles that the bullet kept as its tracer (+112) and the second one (+116).
fn free_tracers(b: i32) {
    let first = lw(b + 112);
    if first != 0 {
        particleFree(first);
        sw(b + 112, 0);
    }
    let second = lw(b + 116);
    if second != 0 {
        particleFree(second);
        sw(b + 116, 0);
    }
}

/// The direction the bullet was going, bounced off the wall it hit: its velocity (divided by its speed squared) is put at `sp+16` and the bounce of it at `sp+32`; the normal of the wall is at `sp+64`. Gives the speed squared.
fn reflect_velocity(sp: i32, b: i32) -> f32 {
    let vx = lwc1(b + 48);
    let vy = lwc1(b + 52);
    let vz = lwc1(b + 56);
    let nx = lwc1(sp + 64);
    let ny = lwc1(sp + 68);
    let nz = lwc1(sp + 72);
    let speed_squared = ps2float::add(ps2float::add(ps2float::mul(vx, vx), ps2float::mul(vy, vy)), ps2float::mul(vz, vz));
    let inverse = (1.0_f32).pdiv(speed_squared);
    let dx = ps2float::mul(vx, inverse);
    swc1(sp + 16, dx);
    let dy = ps2float::mul(lwc1(b + 52), inverse);
    swc1(sp + 20, dy);
    let dz = ps2float::mul(lwc1(b + 56), inverse);
    swc1(sp + 24, dz);
    let dot = ps2float::add(ps2float::add(ps2float::mul(dx, nx), ps2float::mul(dy, ny)), ps2float::mul(dz, nz));
    let twice = ps2float::add(dot, dot);
    swc1(sp + 40, ps2float::sub(dz, ps2float::mul(twice, nz)));
    swc1(sp + 32, ps2float::sub(dx, ps2float::mul(twice, nx)));
    swc1(sp + 36, ps2float::sub(dy, ps2float::mul(twice, ny)));
    speed_squared
}

/// The number of the texture of the model that was hit last (for the sound of the ricochet), or 0.
fn last_hit_texture(sp: i32) -> i32 {
    let mut texture = 0;
    let inst = obGetLastHitInst(sp + 212, sp + 216, 0);
    if inst != 0 {
        let model = lw(inst);
        let list = lw(model - lw(model) * 80 + lw(sp + 212) * 80 + 8);
        let part = lw(sp + 216);
        if list != 0 {
            let index = lw(part * 24 + list);
            if index != -1 {
                texture = lbu(tex_kind_addr(lw(g::thistexbuf), lw((index << 4) + lw(model + 12))));
            }
        }
    }
    texture
}

/// The kinds 2 to 4: a shot that goes straight, leaves a tracer, and bounces off a wall some of the times.
fn tracer_bullet(sp: i32, b: i32, owner_prop: i32) -> Exit {
    if frames_gate() <= 0.0_f32 {
        return Exit::Done;
    }
    if 2.0_f32 <= lwc1(b + 72) {
        return Exit::Free;
    }
    let hit = bulletStraightLineTest(b, sp, sp + 48, sp + 64, sp + 208);
    let glass = lw(sp + 208);
    if glass != 0 {
        bulletProcessGlasshits(b, glass);
    }
    if hit == 0 {
        return Exit::Drift;
    }
    let hit_kind = lw(hit + 8);
    let speed_squared = reflect_velocity(sp, b);
    if hit_kind != 8 {
        let texture = last_hit_texture(sp);
        ricochetsfx(texture, sp + 48);
        particleNewSparks(0, sp + 48, sp + 32);
        particleNewSmoke(0, sp + 48);
        decalNewDecal(lw(b), lw(b + 100), hit, sp + 48, sp + 64);
    } else {
        soundStart(404, sp + 48);
    }
    let damage = lwc1(b + 76);
    propDamage(hit, owner_prop, sp + 48, lw(b), b + 24, b + 48, b, 0, damage);
    bulletExplodeMines(sp + 48, lwc1(0x3ad1f0));
    free_tracers(b);
    let chance = lwc1(b + 152);
    if !(chance < uniform()) && hit_kind != 8 {
        // It bounces: the bullet goes on from the wall, and the chance of the next bounce is smaller.
        sw(b + 156, 1);
        swc1(b + 152, ps2float::mul(chance, chance));
        let away = lwc1(0x3ad1f4);
        swc1(b + 24, ps2float::add(lwc1(sp + 48), ps2float::mul(lwc1(sp + 64), away)));
        swc1(b + 28, ps2float::add(lwc1(sp + 52), ps2float::mul(lwc1(sp + 68), away)));
        swc1(b + 32, ps2float::add(lwc1(sp + 56), ps2float::mul(lwc1(sp + 72), away)));
        swc1(b + 48, ps2float::mul(lwc1(sp + 32), speed_squared));
        swc1(b + 52, ps2float::mul(lwc1(sp + 36), speed_squared));
        swc1(b + 56, ps2float::mul(lwc1(sp + 40), speed_squared));
        let tracer = particleNewTracer(b + 112, b + 24, b + 48, 15);
        sw(b + 112, tracer);
        bulletInitialiseStraightLineTest(b);
        sw(b + 272, 1);
        return Exit::ResetAge;
    }
    Exit::Free
}

/// The kinds 5 to 7: a shot that goes straight, with sparks of sprites on a wall and a pulse after a bounce.
fn spark_bullet(sp: i32, b: i32, owner_prop: i32) -> Exit {
    if frames_gate() <= 0.0_f32 {
        return Exit::Done;
    }
    if 2.0_f32 <= lwc1(b + 72) {
        return Exit::Free;
    }
    let hit = bulletStraightLineTest(b, sp, sp + 48, sp + 64, sp + 208);
    let glass = lw(sp + 208);
    if glass != 0 {
        bulletProcessGlasshits(b, glass);
    }
    let mut sparks = -1;
    if hit == 0 {
        return Exit::Drift;
    }
    let first = lw(b + 112);
    let hit_kind = lw(hit + 8);
    let speed_squared = reflect_velocity(sp, b);
    if first != 0 {
        sparks = lw(first + 928);
        particleFree(first);
        sw(b + 112, 0);
    }
    let second = lw(b + 116);
    if second != 0 {
        sparks = lw(second + 928);
        particleFree(second);
        sw(b + 116, 0);
    }
    if hit_kind != 8 {
        let sound = (newrnd() & 3) as i32 + 174;
        soundDelayStartEx(sound, sp + 48, 20, 0);
        particleNewSpriteSparks(0, sparks, sp + 48, sp + 32);
        decalNewDecal(lw(b), lw(b + 100), hit, sp + 48, sp + 64);
    } else {
        soundStart(404, sp + 48);
    }
    let damage = lwc1(b + 76);
    propDamage(hit, owner_prop, sp + 48, lw(b), b + 24, b + 48, b, 0, damage);
    bulletExplodeMines(sp + 48, lwc1(0x3ad1f8));
    if !(speed_squared < 0.5_f32) {
        let random = uniform();
        if !(lwc1(b + 152) < random) && hit_kind != 8 {
            let away = lwc1(0x3ad1fc);
            swc1(b + 24, ps2float::add(lwc1(sp + 48), ps2float::mul(lwc1(sp + 64), away)));
            swc1(b + 28, ps2float::add(lwc1(sp + 52), ps2float::mul(lwc1(sp + 68), away)));
            swc1(b + 32, ps2float::add(lwc1(sp + 56), ps2float::mul(lwc1(sp + 72), away)));
            swc1(b + 48, ps2float::mul(lwc1(sp + 32), speed_squared));
            swc1(b + 52, ps2float::mul(lwc1(sp + 36), speed_squared));
            swc1(b + 56, ps2float::mul(lwc1(sp + 40), speed_squared));
            if sparks >= 0 {
                let pulse = particleNewPulse(b + 112, sparks, b + 24, b + 48, 15);
                sw(b + 112, pulse);
            }
            bulletInitialiseStraightLineTest(b);
            return Exit::ResetAge;
        }
    }
    Exit::Free
}

/// The kinds 8 and 9: a rocket; it flies with the pull down, and explodes where it hits.
fn rocket_bullet(sp: i32, b: i32, owner_prop: i32) -> Exit {
    if frames_gate() <= 0.0_f32 {
        return Exit::Done;
    }
    if 3.0_f32 <= lwc1(b + 72) {
        return Exit::Free;
    }
    let hit = bulletNonStraightLineTest(b, sp, sp + 48, sp + 64, sp + 208);
    let glass = lw(sp + 208);
    if glass != 0 {
        bulletProcessGlasshits(b, glass);
    }
    if hit == 0 {
        let step_x = lwc1(sp);
        let y = lwc1(b + 28);
        let x = ps2float::add(lwc1(b + 24), step_x);
        let z = lwc1(b + 32);
        let ticks = lw(g::updateframesi);
        swc1(b + 24, x);
        swc1(b + 28, ps2float::add(y, lwc1(sp + 4)));
        swc1(b + 32, ps2float::add(z, lwc1(sp + 8)));
        let trail = lw(b + 204);
        let prop = lw(b + 96);
        if ticks > 0 {
            let drag = lwc1(0x3ad200);
            let mut count = 0;
            loop {
                count += 1;
                let vz = ps2float::mul(lwc1(b + 56), drag);
                let vy = ps2float::add(ps2float::mul(lwc1(b + 52), drag), lwc1(0x3ad204));
                swc1(b + 48, ps2float::mul(lwc1(b + 48), drag));
                swc1(b + 56, vz);
                swc1(b + 52, vy);
                if count >= ticks {
                    break;
                }
            }
        }
        swc1(prop + 48, x);
        swc1(prop + 52, lwc1(b + 28));
        swc1(prop + 56, lwc1(b + 32));
        if trail != 0 {
            if lw(trail + 24) < 100 {
                particleUpdatePos(trail, prop + 48);
                return Exit::Done;
            }
            particleUpdateRate(trail, 0.0_f32);
            sw(lw(b + 204) + 20, 0);
        }
        let new_trail = particleNewTrail(b + 204, 1, b + 24, 60);
        sw(b + 204, new_trail);
        return Exit::Done;
    }
    soundStart(explosion_sound(), sp + 48);
    particleNewExplosion(0, 0, sp + 48, lw(b), b + 24, b + 48, owner_prop, hit);
    particleNewBurst(0, 13, b + 24, 0, 0.0_f32);
    let trail = lw(b + 204);
    if trail != 0 {
        particleUpdateRate(trail, 0.0_f32);
    }
    decalNewDecal(lw(b), lw(b + 100), hit, sp + 48, sp + 64);
    propDamage(hit, owner_prop, sp + 48, lw(b), b + 24, b + 48, b, 0, 0.0_f32);
    Exit::Free
}

/// Turn the prop `prop` to look along `(x, y, z)`, with the doubles at `scale_at` and `divisor_at` (degrees from the angles in radians; `offset_at`, if it is given, is added to the angle round): +72 gets the angle round, +88 the angle up (as a minus) and +76 is cleared.
fn point_prop_along(prop: i32, x: f32, y: f32, z: f32, scale_at: i32, divisor_at: i32, offset_at: Option<i32>) {
    let scale = ld(scale_at);
    let divisor = ld(divisor_at);
    let round = dpdiv(dpmul(atan2(fptodp(z), fptodp(y)), scale), divisor);
    let round = match offset_at {
        Some(at) => dpadd(round, ld(at)),
        None => round,
    };
    swc1(prop + 72, dptofp(round));
    let across = ps2float::add(ps2float::mul(y, y), ps2float::mul(z, z)).abs().psqrt();
    let up = dpdiv(dpmul(atan2(fptodp(x), fptodp(across)), scale), divisor);
    swc1(prop + 88, dptofp(dpsub(0, up)));
    sw(prop + 76, 0);
}

/// The sound of a bounce by the number of bounces so far (1 to 4 have one).
fn bounce_sound(b: i32, bounces: i32) {
    let number = match bounces {
        1 => 179,
        2 => 180,
        3 => 181,
        4 => 182,
        _ => return,
    };
    soundStart(number, b + 24);
}

/// Keep the trail of the bullet (the particle at `slot` of the bullet): the particle follows the prop at `follow`, or if it is gone (or too old) a new one of the kind `kind` and `life` is made.
fn keep_trail(b: i32, slot: i32, follow: i32, kind: i32, life: i32) {
    let trail = lw(b + slot);
    if trail != 0 {
        if lw(trail + 24) < 100 {
            particleUpdatePos(trail, follow);
            return;
        }
        particleUpdateRate(trail, 0.0_f32);
        sw(lw(b + slot) + 20, 0);
    }
    let new_trail = particleNewTrail(b + slot, kind, b + 24, life);
    sw(b + slot, new_trail);
}

/// The kind of trail a grenade leaves (4 to 7 by its kind).
fn grenade_trail_kind(kind: i32) -> i32 {
    if kind == 11 {
        5
    } else if kind >= 12 {
        if kind == 12 { 6 } else { 7 }
    } else if kind == 10 {
        4
    } else {
        7
    }
}

/// The spin of the prop about a random axis: three random numbers (of 0 to 1) make the axis at +100 of the prop, which is then made a unit vector in doubles.
fn spin_prop_randomly(b: i32) {
    let x = random_as_float();
    swc1(lw(b + 96) + 100, ps2float::mul(x, 2.3283064e-10_f32));
    let y = random_as_float();
    swc1(lw(b + 96) + 104, ps2float::mul(y, 2.3283064e-10_f32));
    let z = random_as_float();
    let axis = lw(b + 96);
    let axis_y = lwc1(axis + 104);
    let axis_x = lwc1(axis + 100);
    let axis_z = ps2float::mul(z, 2.3283064e-10_f32);
    swc1(axis + 108, axis_z);
    let length = dptofp(dpdiv(0x3ff0000000000000_i64, sqrt(fptodp(ps2float::add(ps2float::add(ps2float::mul(axis_x, axis_x), ps2float::mul(axis_y, axis_y)), ps2float::mul(axis_z, axis_z))))));
    let prop = lw(b + 96);
    let new_y = ps2float::mul(lwc1(prop + 104), length);
    let new_z = ps2float::mul(lwc1(prop + 108), length);
    swc1(prop + 100, ps2float::mul(lwc1(prop + 100), length));
    swc1(prop + 104, new_y);
    swc1(prop + 108, new_z);
}

/// A grenade that is moving and was not hit by anything on its way: it falls and spins, or bounces off what it hits when it is tested again with the room not given. Gives the prop.
fn grenade_moves(sp: i32, b: i32) -> i32 {
    let bounce_hit = bulletNonStraightLineTest(b, sp, sp + 48, sp + 64, 0);
    let normal_x = lwc1(sp + 64);
    if bounce_hit == 0 {
        let dt = frames();
        let x = ps2float::add(lwc1(b + 24), lwc1(sp));
        let y = lwc1(b + 28);
        let gravity = lwc1(0x3ad218);
        let vy = lwc1(b + 52);
        swc1(b + 24, x);
        let z = lwc1(b + 32);
        let vx = lwc1(b + 48);
        let vz = lwc1(b + 56);
        let drag = lwc1(0x3ad21c);
        let body = lw(b + 96);
        swc1(b + 28, ps2float::add(y, lwc1(sp + 4)));
        let step_z = lwc1(sp + 8);
        swc1(b + 56, ps2float::mul(vz, drag));
        swc1(b + 48, ps2float::mul(vx, drag));
        swc1(b + 52, ps2float::mul(ps2float::add(vy, ps2float::mul(dt, gravity)), drag));
        swc1(b + 32, ps2float::add(z, step_z));
        swc1(body + 48, x);
        swc1(body + 52, lwc1(b + 28));
        swc1(body + 56, lwc1(b + 32));
        spin_prop_randomly(b);
    } else {
        let away = lwc1(0x3ad208);
        let vx = lwc1(b + 48);
        let vy = lwc1(b + 52);
        let vz = lwc1(b + 56);
        let keep = lwc1(0x3ad20c);
        let slow = lwc1(0x3ad210);
        swc1(b + 24, ps2float::add(lwc1(sp + 48), ps2float::mul(normal_x, away)));
        swc1(b + 28, ps2float::add(lwc1(sp + 52), ps2float::mul(lwc1(sp + 68), away)));
        swc1(b + 32, ps2float::add(lwc1(sp + 56), ps2float::mul(lwc1(sp + 72), away)));
        let nx = lwc1(sp + 64);
        let twice = ps2float::mul(ps2float::add(ps2float::add(ps2float::mul(vx, nx), ps2float::mul(vy, lwc1(sp + 68))), ps2float::mul(vz, lwc1(sp + 72))), -2.0_f32);
        let new_x = ps2float::mul(ps2float::add(vx, ps2float::mul(nx, twice)), keep);
        swc1(b + 48, new_x);
        let new_y = ps2float::mul(ps2float::add(vy, ps2float::mul(lwc1(sp + 68), twice)), keep);
        swc1(b + 52, new_y);
        let new_z = ps2float::mul(ps2float::add(vz, ps2float::mul(lwc1(sp + 72), twice)), keep);
        swc1(b + 56, new_z);
        if !(ps2float::add(ps2float::add(ps2float::mul(new_x, new_x), ps2float::mul(new_y, new_y)), ps2float::mul(new_z, new_z)) < slow) {
            let dt = frames();
            if frames_gate() > 0.0_f32 {
                let lift = ps2float::mul(dt, lwc1(0x3ad214));
                let bounces = lw(b + 212) + 1;
                sw(b + 212, bounces);
                swc1(b + 52, ps2float::add(new_y, lift));
                bounce_sound(b, bounces);
            }
        } else {
            sw(b + 48, 0);
            sw(b + 52, 0);
            sw(b + 56, 0);
        }
        let body = lw(b + 96);
        swc1(body + 48, lwc1(b + 24));
        swc1(body + 52, lwc1(b + 28));
        swc1(body + 56, lwc1(b + 32));
    }
    let prop = lw(b + 96);
    keep_trail(b, 204, prop + 48, grenade_trail_kind(lw(b)), 20);
    lw(b + 96)
}

/// The kinds 10 to 13: a grenade; it bounces off what it hits (making a sound the first 4 times) and explodes after 3 ticks (the kinds 10 and 13 explode only by their age: what they hit makes them bounce).
fn grenade_bullet(sp: i32, b: i32, owner_prop: i32) -> Exit {
    if frames_gate() > 0.0_f32 {
        let mut hit = bulletNonStraightLineTest(b, sp, sp + 48, sp + 64, sp + 208);
        let kind = lw(b);
        if kind == 10 || kind == 13 {
            hit = 0;
        }
        let glass = lw(sp + 208);
        if glass != 0 {
            bulletProcessGlasshits(b, glass);
        }
        let prop;
        if lwc1(b + 72) < 3.0_f32 && hit == 0 {
            // (the fall of the step is looked at with the step itself)
            let still = lwc1(sp) == 0.0_f32 && lwc1(sp + 4) == 0.0_f32 && lwc1(sp + 8) == 0.0_f32;
            if still {
                prop = lw(b + 96);
            } else {
                prop = grenade_moves(sp, b);
            }
        } else {
            // It explodes: after 3 ticks, or where it hit.
            let kind = lw(b);
            let explosion = if kind == 10 {
                4
            } else if kind >= 10 && kind == 11 {
                2
            } else if kind >= 10 && kind == 12 {
                3
            } else {
                0
            };
            let at = b + 24;
            soundStart(explosion_sound(), at);
            particleNewExplosion(0, explosion, at, lw(b), at, b + 48, owner_prop, hit);
            let trail = lw(b + 204);
            if trail != 0 {
                particleUpdateRate(trail, 0.0_f32);
            }
            particleNewBurst(0, 13, at, 0, 0.0_f32);
            let place = if hit == 0 { at } else { sp + 48 };
            decalNewDecal(lw(b), lw(b + 100), hit, place, sp + 64);
            bulletFree(b);
            prop = lw(b + 96);
        }
        // The prop of the grenade turns about, slower each time.
        if prop != 0 {
            let spin = lwc1(prop + 96);
            if 0.0_f32 < spin {
                let slowed = ps2float::mul(spin, lwc1(0x3ad220));
                let angle = ps2float::add(lwc1(prop + 92), slowed);
                swc1(prop + 96, slowed);
                swc1(prop + 92, angle);
                if 360.0_f32 < angle {
                    loop {
                        let wrapped = ps2float::sub(lwc1(prop + 92), 360.0_f32);
                        swc1(prop + 92, wrapped);
                        if !(360.0_f32 < wrapped) {
                            break;
                        }
                    }
                }
            }
        }
    }
    let owner = lw(b + 108);
    if owner < 0 {
        return Exit::Done;
    }
    Exit::Record(owner)
}

/// The mine `b` is stuck to a character or a prop that moves: follow it (the place on it is kept at +232 to +252).
fn mine_follows(sp: i32, b: i32, stuck: i32) {
    if lw(stuck + 8) != 8 {
        // (the place on a character that is not alive is the same)
    }
    swc1(sp + 144, lwc1(b + 232));
    swc1(sp + 148, lwc1(b + 236));
    swc1(sp + 152, lwc1(b + 240));
    swc1(sp + 160, lwc1(b + 244));
    swc1(sp + 164, lwc1(b + 248));
    swc1(sp + 168, lwc1(b + 252));
    let part = lw(b + 228);
    if part >= 0 {
        sceVu0CopyMatrix(sp + 80, lw(lw(stuck + 32) + 4) + (part << 6));
        matrixVecMulAligned(&[(sp + 80) as i64, (sp + 144) as i64]);
        sw(sp + 128, 0);
        sw(sp + 132, 0);
        sw(sp + 136, 0);
        matrixVecMulAligned(&[(sp + 80) as i64, (sp + 160) as i64]);
    }
    if lw(stuck + 140) != 2 {
        let angle = ps2float::mul(lwc1(stuck + 76), lwc1(0x3ad290)).pdiv(1.8e+02_f32);
        matrixTransRotY(sp + 80, lwc1(stuck + 48), lwc1(stuck + 52), lwc1(stuck + 56), angle);
    } else {
        let scale = lwc1(0x3ad28c);
        let a = ps2float::mul(lwc1(stuck + 72), scale).pdiv(1.8e+02_f32);
        let b2 = ps2float::mul(lwc1(stuck + 76), scale).pdiv(1.8e+02_f32);
        let c = ps2float::mul(lwc1(stuck + 88), scale).pdiv(1.8e+02_f32);
        matrixTransRotZXY(sp + 80, lwc1(stuck + 48), lwc1(stuck + 52), lwc1(stuck + 56), a, b2, c);
    }
    matrixVecMulAligned(&[(sp + 80) as i64, (sp + 144) as i64]);
    sw(sp + 128, 0);
    sw(sp + 132, 0);
    sw(sp + 136, 0);
    matrixVecMulAligned(&[(sp + 80) as i64, (sp + 160) as i64]);
    let prop = lw(b + 96);
    let x = lwc1(sp + 144);
    swc1(b + 24, x);
    swc1(prop + 48, x);
    let y = lwc1(sp + 148);
    swc1(b + 28, y);
    swc1(prop + 52, y);
    let z = lwc1(sp + 152);
    swc1(b + 32, z);
    swc1(prop + 56, z);
    swc1(b + 128, lwc1(sp + 160));
    swc1(b + 132, lwc1(sp + 164));
    swc1(b + 136, lwc1(sp + 168));
    point_prop_along(lw(b + 96), lwc1(sp + 160), lwc1(sp + 164), lwc1(sp + 168), 0x3a8b00, 0x3a8b08, None);
}

/// The mine `b` hit `hit` (a character or a prop, `hit_kind` says which): it sticks there, with the place on it and the angles of the prop made from the normal.
fn mine_sticks(sp: i32, b: i32, hit: i32) {
    soundStart(194, b + 24);
    let hit_kind = lw(hit + 8);
    let part;
    if hit_kind != 2 {
        let record = lw(hit + 32);
        let row;
        if hit_kind != 8 {
            let model = lw(record);
            let base = model - lw(model) * 80;
            obGetLastHitInst(sp + 220, 0, 0);
            let index = lw(sp + 220);
            sw(b + 224, hit);
            row = index * 80 + base;
            part = Some(lb(row + 1));
        } else {
            let model = lw(record);
            let base = model - lw(model) * 80;
            let vx = lwc1(sp);
            let vy = lwc1(sp + 4);
            let vz = lwc1(sp + 8);
            let length = (1.0_f32).pdiv(ps2float::add(ps2float::add(ps2float::mul(vx, vx), ps2float::mul(vy, vy)), ps2float::mul(vz, vz)).abs().psqrt());
            swc1(sp + 8, ps2float::mul(vz, length));
            swc1(sp, ps2float::mul(vx, length));
            swc1(sp + 4, ps2float::mul(vy, length));
            chrLineTestChr(hit, b + 24, sp, 0, 1, sp + 48, sp + 64, 2);
            sw(b + 224, hit);
            let first = chrGetLastHitPart();
            let found = lb(first * 80 + base + 1);
            sw(b + 228, found);
            if found >= 0 {
                part = None;
            } else {
                let second = chrGetLastHitPart();
                let mut done = false;
                if lb(second * 80 + base + 2) >= 0 {
                    let third = chrGetLastHitPart();
                    sw(b + 228, lb(lb(third * 80 + base + 2) * 80 + base + 1));
                }
                if lw(b + 228) >= 0 {
                    done = true;
                }
                if done {
                    part = None;
                } else {
                    let fourth = chrGetLastHitPart();
                    if lb(fourth * 80 + base + 3) < 0 {
                        part = None;
                    } else {
                        let fifth = chrGetLastHitPart();
                        row = lb(fifth * 80 + base + 3) * 80 + base;
                        part = Some(lb(row + 1));
                    }
                }
            }
        }
    } else {
        part = Some(-1);
        sw(b + 224, 0);
    }
    if let Some(part) = part {
        sw(b + 228, part);
    }
    point_prop_along(lw(b + 96), lwc1(sp + 64), lwc1(sp + 68), lwc1(sp + 72), 0x3a8af0, 0x3a8af8, None);
    let prop = lw(b + 96);
    if lw(b + 224) != 0 {
        // The place is made one on the character it is stuck to.
        if lw(hit + 140) != 2 {
            matrixRotY(sp + 80, ps2float::mul(lwc1(hit + 76), lwc1(0x3ad27c)).pdiv(1.8e+02_f32));
        } else {
            let scale = lwc1(0x3ad278);
            matrixRotZXY(sp + 80, ps2float::mul(lwc1(hit + 72), scale).pdiv(1.8e+02_f32), ps2float::mul(lwc1(hit + 76), scale).pdiv(1.8e+02_f32), ps2float::mul(lwc1(hit + 88), scale).pdiv(1.8e+02_f32));
        }
        let x0 = lwc1(hit + 48);
        let y1 = lwc1(sp + 52);
        let z1 = lwc1(sp + 56);
        swc1(sp + 48, ps2float::sub(lwc1(sp + 48), x0));
        swc1(sp + 52, ps2float::sub(y1, lwc1(hit + 52)));
        swc1(sp + 56, ps2float::sub(z1, lwc1(hit + 56)));
        sceVu0InversMatrix(sp + 144, sp + 80);
        matrixVecMul(sp + 144, sp + 48);
        matrixVecMul(sp + 144, sp + 64);
        let part = lw(b + 228);
        if part >= 0 {
            sceVu0CopyMatrix(sp + 80, lw(lw(hit + 32) + 4) + (part << 6));
            let y = ps2float::sub(lwc1(sp + 52), lwc1(sp + 132));
            let z = ps2float::sub(lwc1(sp + 56), lwc1(sp + 136));
            swc1(sp + 48, ps2float::sub(lwc1(sp + 48), lwc1(sp + 128)));
            sw(sp + 128, 0);
            swc1(sp + 52, y);
            sw(sp + 132, 0);
            swc1(sp + 56, z);
            sw(sp + 136, 0);
            sceVu0InversMatrix(sp + 144, sp + 80);
            matrixVecMul(sp + 144, sp + 48);
            matrixVecMul(sp + 144, sp + 64);
        }
        let nx = lwc1(sp + 64);
        let ny = lwc1(sp + 68);
        let nz = lwc1(sp + 72);
        let x = lwc1(sp + 48);
        let length = (1.0_f32).pdiv(ps2float::add(ps2float::add(ps2float::mul(nx, nx), ps2float::mul(ny, ny)), ps2float::mul(nz, nz)).abs().psqrt());
        swc1(sp + 72, ps2float::mul(nz, length));
        swc1(sp + 64, ps2float::mul(nx, length));
        swc1(sp + 68, ps2float::mul(ny, length));
        swc1(b + 232, x);
        swc1(b + 236, lwc1(sp + 52));
        swc1(b + 240, lwc1(sp + 56));
        swc1(b + 244, lwc1(sp + 64));
        swc1(b + 248, lwc1(sp + 68));
        swc1(b + 252, lwc1(sp + 72));
    }
    let away = lwc1(0x3ad280);
    let x = ps2float::add(lwc1(sp + 48), ps2float::mul(lwc1(sp + 64), away));
    swc1(b + 24, x);
    swc1(b + 28, ps2float::add(lwc1(sp + 52), ps2float::mul(lwc1(sp + 68), away)));
    swc1(b + 32, ps2float::add(lwc1(sp + 56), ps2float::mul(lwc1(sp + 72), away)));
    swc1(b + 128, lwc1(sp + 64));
    swc1(b + 132, lwc1(sp + 68));
    let z = lwc1(sp + 72);
    sw(b + 48, 0);
    swc1(b + 136, z);
    sw(b + 52, 0);
    sw(b + 56, 0);
    swc1(prop + 48, x);
    swc1(prop + 52, lwc1(b + 28));
    swc1(prop + 56, lwc1(b + 32));
}

/// The kinds 14 to 16: a mine; it flies and sticks to what it hits (a character it follows, until the character is hidden); the kind 15 spins and explodes after 6 ticks, and the kind 16 makes a sound each tick and tells the props that are near.
fn mine_bullet(sp: i32, b: i32, owner_prop: i32) -> Exit {
    let mut kind = lw(b);
    if frames_gate() > 0.0_f32 {
        'work: {
            if kind == 15 && 6.0_f32 <= lwc1(b + 72) {
                let at = b + 24;
                soundStart(explosion_sound(), at);
                particleNewExplosion(0, 0, at, lw(b), at, b + 48, owner_prop, 0);
                particleNewSmoke(0, at);
                decalNewDecal(lw(b), lw(b + 100), 0, at, 0);
                bulletFree(b);
                kind = lw(b);
                break 'work;
            }
            let still = lwc1(sp) == 0.0_f32 && lwc1(sp + 4) == 0.0_f32 && lwc1(sp + 8) == 0.0_f32;
            if still {
                let stuck = lw(b + 224);
                if stuck == 0 {
                    break 'work;
                }
                if lw(stuck + 8) == 8 && lw(lw(stuck + 352) + 2708) & 32 != 0 {
                    // The character is hidden: the mine drops.
                    let dt = frames();
                    sw(b + 224, 0);
                    swc1(b + 52, ps2float::mul(dt, lwc1(0x3ad288)));
                    break 'work;
                }
                mine_follows(sp, b, stuck);
                kind = lw(b);
                break 'work;
            }
            let hit = bulletNonStraightLineTest(b, sp, sp + 48, sp + 64, sp + 208);
            let glass = lw(sp + 208);
            if glass != 0 {
                bulletProcessGlasshits(b, glass);
            }
            if hit == 0 {
                let dt = frames();
                let x = ps2float::add(lwc1(b + 24), lwc1(sp));
                let y = lwc1(b + 28);
                let gravity = lwc1(0x3ad284);
                let vy = lwc1(b + 52);
                swc1(b + 24, x);
                let z = lwc1(b + 32);
                let prop = lw(b + 96);
                kind = lw(b);
                swc1(b + 28, ps2float::add(y, lwc1(sp + 4)));
                let step_z = lwc1(sp + 8);
                swc1(b + 52, ps2float::add(vy, ps2float::mul(dt, gravity)));
                swc1(b + 32, ps2float::add(z, step_z));
                swc1(prop + 48, x);
                let spin = ps2float::add(lwc1(prop + 76), 12.0_f32);
                swc1(prop + 52, lwc1(b + 28));
                let z = lwc1(b + 32);
                swc1(prop + 76, spin);
                swc1(prop + 56, z);
            } else {
                mine_sticks(sp, b, hit);
                kind = lw(b);
            }
        }
        // Every way out of the work above comes here, with the kind as it was last read.
        if kind == 16 {
            updateProximityProps(b, owner_prop);
            kind = lw(b);
        }
    }
    if kind == 15 {
        // The mine turns, and its prop with it, and it makes a sound for each tick of its time.
        let dt = frames();
        let angle = lwc1(b + 124);
        let speed = ps2float::add(lwc1(b + 64), ps2float::mul(dt, lwc1(0x3ad294)));
        swc1(b + 64, speed);
        let turned = ps2float::add(angle, ps2float::mul(speed, dt));
        swc1(b + 124, turned);
        if 360.0_f32 < turned {
            swc1(b + 124, ps2float::sub(turned, 360.0_f32));
        }
        obInstSetAnim(lw(lw(b + 96) + 32), 1, 0.0_f32);
        let prop = lw(b + 96);
        let age;
        if lw(prop + 16) & 256 != 0 {
            age = lwc1(b + 72);
        } else {
            sceVu0UnitMatrix(lw(lw(prop + 32) + 4));
            sceVu0UnitMatrix(lw(lw(lw(b + 96) + 32) + 4) + 128);
            let rotation = ps2float::mul(lwc1(b + 124), lwc1(0x3ad298)).pdiv(1.8e+02_f32);
            matrixRotY(lw(lw(lw(b + 96) + 32) + 4) + 64, rotation);
            age = lwc1(b + 72);
        }
        let ticks = lw(b + 260);
        let next = ticks + 1;
        if ticks < ps2float::sub(age + age, 1.0_f32) as i32 {
            sw(b + 260, next);
            if next < 10 {
                soundStart(191, b + 24);
            }
            if lw(b + 260) == 10 {
                soundStart(192, b + 24);
            }
        }
        kind = lw(b);
    }
    if kind == 16 {
        if lw(b + 260) < ps2float::mul(lwc1(b + 72), 0.5_f32) as i32 {
            soundStart(193, b + 24);
            sw(b + 260, lw(b + 260) + 1);
        }
    }
    Exit::Done
}

/// Turn the prop `prop` by the spin of the bullet `b` (+60 to +68), each angle kept from 0 to 360.
fn spin_prop(prop: i32, b: i32) {
    for (at, spin) in [(72, 60), (76, 64), (88, 68)] {
        let angle = ps2float::add(lwc1(prop + at), lwc1(b + spin));
        swc1(prop + at, angle);
        if angle < 0.0_f32 {
            swc1(prop + at, ps2float::add(angle, 360.0_f32));
        }
        let angle = lwc1(prop + at);
        if 360.0_f32 <= angle {
            swc1(prop + at, ps2float::sub(angle, 360.0_f32));
        }
    }
}

/// The kinds 17 and 18: a thing that is thrown and bounces; at rest it turns the way it lies, and when its time (1 tick, for the kind 17 the constant at 0x3ad224) is over it explodes.
fn thrown_bullet(sp: i32, b: i32, owner_prop: i32) -> Exit {
    let kind = lw(b);
    let limit = if kind == 17 { lwc1(0x3ad224) } else { 1.0_f32 };
    let age = lwc1(b + 72);
    let step_x = lwc1(sp);
    let owner;
    if !(limit <= age) {
        let at_rest = step_x == 0.0_f32 && lwc1(sp + 4) == 0.0_f32 && lwc1(sp + 8) == 0.0_f32;
        let mut first_trail = true;
        if at_rest {
            let prop = lw(b + 96);
            let rest = lwc1(0x3ad248);
            swc1(prop + 72, ps2float::add(lwc1(b + 60), 90.0_f32));
            swc1(prop + 76, lwc1(b + 64));
            swc1(prop + 88, lwc1(b + 68));
            let spin_y = ps2float::mul(lwc1(b + 64), rest);
            let spin_z = ps2float::mul(lwc1(b + 68), rest);
            swc1(b + 60, ps2float::mul(lwc1(b + 60), rest));
            swc1(b + 64, spin_y);
            swc1(b + 68, spin_z);
            first_trail = false;
        }
        if first_trail {
            let hit = bulletNonStraightLineTest(b, sp, sp + 48, sp + 64, sp + 208);
            let glass = lw(sp + 208);
            if glass != 0 {
                bulletProcessGlasshits(b, glass);
            }
            let normal_x = lwc1(sp + 64);
            let prop;
            if hit == 0 {
                let dt = frames();
                let x = ps2float::add(lwc1(b + 24), lwc1(sp));
                let y = lwc1(b + 28);
                let gravity = lwc1(0x3ad240);
                let vy = lwc1(b + 52);
                swc1(b + 24, x);
                let z = lwc1(b + 32);
                let vx = lwc1(b + 48);
                let vz = lwc1(b + 56);
                let drag = lwc1(0x3ad244);
                prop = lw(b + 96);
                swc1(b + 28, ps2float::add(y, lwc1(sp + 4)));
                let step_z = lwc1(sp + 8);
                swc1(b + 56, ps2float::mul(vz, drag));
                swc1(b + 48, ps2float::mul(vx, drag));
                swc1(b + 52, ps2float::mul(ps2float::add(vy, ps2float::mul(dt, gravity)), drag));
                swc1(b + 32, ps2float::add(z, step_z));
                swc1(prop + 48, x);
                swc1(prop + 52, lwc1(b + 28));
                swc1(prop + 56, lwc1(b + 32));
                spin_prop(prop, b);
            } else {
                let away = lwc1(0x3ad228);
                let vx = lwc1(b + 48);
                let vy = lwc1(b + 52);
                let vz = lwc1(b + 56);
                let keep = lwc1(0x3ad22c);
                let slow = lwc1(0x3ad230);
                swc1(b + 24, ps2float::add(lwc1(sp + 48), ps2float::mul(normal_x, away)));
                swc1(b + 28, ps2float::add(lwc1(sp + 52), ps2float::mul(lwc1(sp + 68), away)));
                swc1(b + 32, ps2float::add(lwc1(sp + 56), ps2float::mul(lwc1(sp + 72), away)));
                let nx = lwc1(sp + 64);
                let twice = ps2float::mul(ps2float::add(ps2float::add(ps2float::mul(vx, nx), ps2float::mul(vy, lwc1(sp + 68))), ps2float::mul(vz, lwc1(sp + 72))), -2.0_f32);
                let new_x = ps2float::mul(ps2float::add(vx, ps2float::mul(nx, twice)), keep);
                swc1(b + 48, new_x);
                let new_y = ps2float::mul(ps2float::add(vy, ps2float::mul(lwc1(sp + 68), twice)), keep);
                swc1(b + 52, new_y);
                let new_z = ps2float::mul(ps2float::add(vz, ps2float::mul(lwc1(sp + 72), twice)), keep);
                swc1(b + 56, new_z);
                if !(ps2float::add(ps2float::add(ps2float::mul(new_x, new_x), ps2float::mul(new_y, new_y)), ps2float::mul(new_z, new_z)) < slow) {
                    let spin_z = ps2float::mul(lwc1(b + 68), lwc1(0x3ad238));
                    let spin_x = ps2float::mul(lwc1(b + 60), lwc1(0x3ad23c));
                    let spin_y = ps2float::mul(lwc1(b + 64), lwc1(0x3ad238));
                    let bounces = lw(b + 212) + 1;
                    swc1(b + 52, ps2float::add(new_y, ps2float::mul(frames(), lwc1(0x3ad234))));
                    swc1(b + 60, spin_x);
                    swc1(b + 64, spin_y);
                    swc1(b + 68, spin_z);
                    sw(b + 212, bounces);
                    bounce_sound(b, bounces);
                    prop = lw(b + 96);
                } else {
                    sw(b + 48, 0);
                    sw(b + 52, 0);
                    sw(b + 56, 0);
                    prop = lw(b + 96);
                    swc1(b + 60, lwc1(prop + 72));
                    swc1(b + 64, lwc1(prop + 76));
                    swc1(b + 68, lwc1(prop + 88));
                }
                swc1(prop + 48, lwc1(b + 24));
                swc1(prop + 52, lwc1(b + 28));
                swc1(prop + 56, lwc1(b + 32));
            }
            let follow = prop + 48;
            let trail = lw(b + 204);
            if trail != 0 && lw(trail + 24) < 100 {
                particleUpdatePos(trail, follow);
            } else {
                if trail != 0 {
                    particleUpdateRate(trail, 0.0_f32);
                    sw(lw(b + 204) + 20, 0);
                }
                let new_trail = particleNewTrail(b + 204, 2, b + 24, 10);
                sw(b + 204, new_trail);
            }
        }
        let second = lw(b + 208);
        if second != 0 && lw(second + 24) < 100 {
            particleUpdatePos(second, lw(b + 96) + 48);
        } else {
            if second != 0 {
                particleUpdateRate(second, 0.0_f32);
                sw(lw(b + 208) + 20, 0);
            }
            let new_trail = particleNewTrail(b + 208, 3, b + 24, 80);
            sw(b + 208, new_trail);
        }
        owner = lw(b + 108);
    } else {
        let at = b + 24;
        soundStart(explosion_sound(), at);
        particleNewExplosion(0, 4, at, lw(b), at, b + 48, owner_prop, 0);
        let trail = lw(b + 204);
        if trail != 0 {
            particleUpdateRate(trail, 0.0_f32);
        }
        let second = lw(b + 208);
        if second != 0 {
            particleUpdateRate(second, 0.0_f32);
        }
        particleNewBurst(0, 13, at, 0, 0.0_f32);
        decalNewDecal(lw(b), lw(b + 100), 0, at, 0);
        bulletFree(b);
        owner = lw(b + 108);
    }
    if owner >= 0 {
        return Exit::Record(owner);
    }
    Exit::Done
}

/// The kind 19: a thing that lands and sits; when it is at rest its prop fades out and the bullet goes. The owner is told its place in all the cases it is not gone.
fn sitting_bullet(sp: i32, b: i32, owner_prop: i32) -> Exit {
    if frames_gate() > 0.0_f32 {
        let step_x = lwc1(sp);
        let at_rest = step_x == 0.0_f32 && lwc1(sp + 4) == 0.0_f32 && lwc1(sp + 8) == 0.0_f32;
        if at_rest {
            let fade = ps2float::mul(lwc1(b + 264), 127.0_f32) as i32 & 255;
            obInstSetOverrideStates(lw(lw(b + 96) + 32), 3, -1, -1, fade);
            let left = ps2float::sub(lwc1(b + 264), ps2float::mul(frames(), lwc1(0x3ad274)));
            swc1(b + 264, left);
            if left <= 0.0_f32 {
                bulletFree(b);
                return Exit::Done;
            }
        } else {
            let hit = bulletNonStraightLineTest(b, sp, sp + 48, sp + 64, sp + 208);
            let glass = lw(sp + 208);
            if glass != 0 {
                bulletProcessGlasshits(b, glass);
            }
            if hit != 0 {
                let normal_x = lwc1(sp + 64);
                if lw(hit + 8) == 8 {
                    propDamage(hit, owner_prop, sp + 48, lw(b), b + 24, b + 48, b, 0, lwc1(b + 76));
                }
                let normal_x = if lw(hit + 8) == 8 { lwc1(sp + 64) } else { normal_x };
                let away = lwc1(0x3ad24c);
                let lift = lwc1(0x3ad250);
                let vx = lwc1(b + 48);
                let vy = lwc1(b + 52);
                let vz = lwc1(b + 56);
                let keep = lwc1(0x3ad254);
                let slow = lwc1(0x3ad258);
                swc1(b + 24, ps2float::add(lwc1(sp + 48), ps2float::mul(normal_x, away)));
                swc1(b + 28, ps2float::add(ps2float::add(lwc1(sp + 52), ps2float::mul(lwc1(sp + 68), away)), lift));
                swc1(b + 32, ps2float::add(lwc1(sp + 56), ps2float::mul(lwc1(sp + 72), away)));
                let nx = lwc1(sp + 64);
                let twice = ps2float::mul(ps2float::add(ps2float::add(ps2float::mul(vx, nx), ps2float::mul(vy, lwc1(sp + 68))), ps2float::mul(vz, lwc1(sp + 72))), -2.0_f32);
                let new_x = ps2float::mul(ps2float::add(vx, ps2float::mul(nx, twice)), keep);
                swc1(b + 48, new_x);
                let new_y = ps2float::mul(ps2float::add(vy, ps2float::mul(lwc1(sp + 68), twice)), 0.5_f32);
                swc1(b + 52, new_y);
                let new_z = ps2float::mul(ps2float::add(vz, ps2float::mul(lwc1(sp + 72), twice)), keep);
                swc1(b + 56, new_z);
                let mut bounce = true;
                if ps2float::add(ps2float::add(ps2float::mul(new_x, new_x), ps2float::mul(new_y, new_y)), ps2float::mul(new_z, new_z)) < slow {
                    let up = lwc1(sp + 68);
                    if lw(hit + 8) != 8 && lwc1(0x3ad25c) < up {
                        // It lies down on the floor.
                        point_prop_along(lw(b + 96), lwc1(sp + 64), lwc1(sp + 68), lwc1(sp + 72), 0x3a8ad8, 0x3a8ae0, Some(0x3a8ae8));
                        swc1(b + 264, 1.0_f32);
                        sw(b + 48, 0);
                        sw(b + 52, 0);
                        sw(b + 56, 0);
                        bounce = false;
                    }
                }
                if bounce {
                    let dt = frames();
                    let spin_y = lwc1(b + 64);
                    let damping = lwc1(0x3ad264);
                    let bounces = lw(b + 212);
                    let spin_z = ps2float::mul(lwc1(b + 68), damping);
                    let spin_x = ps2float::mul(lwc1(b + 60), lwc1(0x3ad268));
                    swc1(b + 52, ps2float::add(lwc1(b + 52), ps2float::mul(dt, lwc1(0x3ad260))));
                    swc1(b + 68, spin_z);
                    swc1(b + 60, spin_x);
                    swc1(b + 64, ps2float::mul(spin_y, damping));
                    sw(b + 212, bounces + 1);
                    soundStart(136, b + 24);
                }
                let prop = lw(b + 96);
                swc1(prop + 48, lwc1(b + 24));
                swc1(prop + 52, lwc1(b + 28));
                swc1(prop + 56, lwc1(b + 32));
            } else {
                let dt = frames();
                let x = ps2float::add(lwc1(b + 24), lwc1(sp));
                let y = lwc1(b + 28);
                let gravity = lwc1(0x3ad26c);
                let vy = lwc1(b + 52);
                swc1(b + 24, x);
                let z = lwc1(b + 32);
                let vx = lwc1(b + 48);
                let vz = lwc1(b + 56);
                let drag = lwc1(0x3ad270);
                let prop = lw(b + 96);
                swc1(b + 28, ps2float::add(y, lwc1(sp + 4)));
                let step_z = lwc1(sp + 8);
                swc1(b + 56, ps2float::mul(vz, drag));
                swc1(b + 48, ps2float::mul(vx, drag));
                swc1(b + 52, ps2float::mul(ps2float::add(vy, ps2float::mul(dt, gravity)), drag));
                swc1(b + 32, ps2float::add(z, step_z));
                swc1(prop + 48, x);
                swc1(prop + 52, lwc1(b + 28));
                swc1(prop + 56, lwc1(b + 32));
                spin_prop(prop, b);
            }
        }
    }
    Exit::Record(lw(b + 108))
}

/// The tick of the bullets in flight (@ 0x28a8e8): for every bullet of the array move it by its kind, test what it hits, and do what the hit does (damage, sparks, smoke, decals, explosions, ricochets) or let it die.
pub fn bulletTick() {
    let frame = Frame::new(448);
    let sp = frame.sp();
    let mut index = 0;
    loop {
        let dt = frames();
        let b = index * 276 + lw(0x3b0138);
        let mut kind = lw(b);
        if kind != 0 {
            // The prop of the owner is hidden from the line tests while the bullet is young (and is shown again at the end).
            let mut owner_prop = 0;
            let owner = lw(b + 108);
            if owner >= 0 {
                match lw(b + 104) {
                    1 | 4 => {
                        owner_prop = lw(owner * 1820 + lw(g::players) + 384);
                        if lwc1(b + 72) <= lwc1(0x3ad1ec) {
                            sw(owner_prop + 16, lw(owner_prop + 16) & -33);
                        }
                        kind = lw(b);
                    }
                    2 => {
                        owner_prop = lw(owner * 0x1210 + lw(g::chrinfo) + 3020);
                        sw(owner_prop + 16, lw(owner_prop + 16) & -33);
                        kind = lw(b);
                    }
                    _ => {}
                }
            }
            // The step of this frame.
            swc1(sp, ps2float::mul(lwc1(b + 48), dt));
            swc1(sp + 4, ps2float::mul(lwc1(b + 52), dt));
            swc1(sp + 8, ps2float::mul(lwc1(b + 56), dt));
            swc1(b + 72, ps2float::add(lwc1(b + 72), (dt).pdiv(60.0_f32)));
            let exit = match kind {
                2..=4 => tracer_bullet(sp, b, owner_prop),
                5..=7 => spark_bullet(sp, b, owner_prop),
                8 | 9 => rocket_bullet(sp, b, owner_prop),
                10..=13 => grenade_bullet(sp, b, owner_prop),
                14..=16 => mine_bullet(sp, b, owner_prop),
                17 | 18 => thrown_bullet(sp, b, owner_prop),
                19 => sitting_bullet(sp, b, owner_prop),
                _ => Exit::Done,
            };
            match exit {
                Exit::Done => {}
                Exit::Free => bulletFree(b),
                Exit::Drift => {
                    let y = lwc1(b + 28);
                    let z = lwc1(b + 32);
                    swc1(b + 24, ps2float::add(lwc1(b + 24), lwc1(sp)));
                    swc1(b + 28, ps2float::add(y, lwc1(sp + 4)));
                    swc1(b + 32, ps2float::add(z, lwc1(sp + 8)));
                }
                Exit::ResetAge => sw(b + 72, 0),
                Exit::Record(owner) => {
                    let character = owner * 0x1210 + lw(g::chrinfo);
                    swc1(character + 2932, lwc1(b + 24));
                    sw(character + 2944, lw(b));
                    swc1(character + 2936, lwc1(b + 28));
                    swc1(character + 2940, lwc1(b + 32));
                }
            }
            if owner_prop != 0 {
                sw(owner_prop + 16, lw(owner_prop + 16) | 32);
            }
        }
        index += 1;
        if index >= 50 {
            break;
        }
    }
}
