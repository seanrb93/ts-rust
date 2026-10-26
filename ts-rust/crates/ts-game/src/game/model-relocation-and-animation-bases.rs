//! Relocating a model, allocating override chunks, the colour of the map maker lights, which animation family a character plays, and the announcer.

use super::overrides_teams_and_glass::last_chunk;
use crate::*;

/// A random number from 0 up to 1 (the 32 bits of `newrnd` as an unsigned number times 2^-32).
fn random_unit() -> f32 {
    ps2float::mul((newrnd() as i32 as u32) as f32, 2.3283064e-10_f32)
}

/// Add `delta` to the pointers of every bone (80 bytes) of a model that has been loaded at another address: the words at +8, +20 to +40, +12, +44, +48, +52 to +64, +16 and +72
/// when they are not 0; the word at +76 is cleared (@ 0x25e930).
pub fn obRelocate(skeleton: i32, delta: i32) {
    let n = lw(skeleton);
    let bones = skeleton - n * 80;
    for i in 0..n.max(0) {
        let bone = bones + 80 * i;
        for at in [8, 20, 24, 28, 32, 36, 40, 12, 44, 48, 52, 56, 60, 64, 16, 72] {
            let v = lw(bone + at);
            if v != 0 {
                sw(bone + at, v + delta);
            }
        }
        sw(bone + 76, 0);
    }
}

/// Allocate the arrays of an override record (48 bytes a bone) for a bone of a prop, big enough for the vertices of its last chunk: `flags` 1 a vertex array of 12 bytes
/// a vertex, 4 one of 4 bytes, 2 one of 16 bytes (rounded up to 16 bytes); for the first list of chunks (`which` 0) they go to +4, +12, +8, for the second (1) to +28, +36, +32 (@ 0x274650).
pub fn allocateOverrideChunk(prop: i32, bone: i32, which: i32, flags: i32) {
    let skeleton = lw(lw(prop + 32));
    let this = skeleton - lw(skeleton) * 80 + bone * 80;
    let list = if which == 0 {
        lw(this + 8)
    } else if which == 1 {
        lw(this + 12)
    } else {
        0
    };
    if list == 0 {
        return;
    }
    let chunk = last_chunk(list);
    let n = lw(chunk + 12) + lw(chunk + 16);
    if n == 0 {
        return;
    }
    let (at_12, at_4, at_16) = if which == 0 { (4, 12, 8) } else { (28, 36, 32) };
    if flags & 1 != 0 {
        let array = memdbAlloc(n * 12 + 15 & -16);
        sw(bone * 48 + lw(lw(prop + 32) + 248) + at_12, array);
    }
    if flags & 4 != 0 {
        let array = memdbAlloc((n << 2) + 15 & -16);
        sw(bone * 48 + lw(lw(prop + 32) + 248) + at_4, array);
    }
    if flags & 2 != 0 {
        let array = memdbAlloc(n << 4);
        sw(bone * 48 + lw(lw(prop + 32) + 248) + at_16, array);
    }
}

/// Take the light the map maker dialog chose (0xrrggbb00) apart into volume (the largest channel), saturation and hue in degrees (`lightvolume`, `lightsaturation`, `lighthue`) (@ 0x246318).
pub fn mmSetLightFromRGB(rgb: i32) {
    let channel = |shift: u32, extra: i32| (((rgb as u32) >> shift) as i32 & extra) as f32;
    let r = ps2float::mul(channel(23, 254), 0.00390625_f32);
    let g_ = ps2float::mul(channel(15, 254), 0.00390625_f32);
    let b = ps2float::mul(channel(7, 254), 0.00390625_f32);
    let mut high = r;
    if r < g_ {
        high = g_;
    }
    let mut maximum = high;
    if high < b {
        maximum = b;
    }
    let mut low = r;
    if g_ < r {
        low = g_;
    }
    let mut minimum = low;
    if b < low {
        minimum = b;
    }
    let mut saturation = 0.0_f32;
    swc1(g::lightvolume, maximum);
    if !(maximum == saturation) {
        saturation = (ps2float::sub(maximum, minimum)).pdiv(maximum);
    }
    swc1(g::lightsaturation, saturation);
    if !(lwc1(g::lightsaturation) == 0.0) {
        let spread = ps2float::sub(maximum, minimum);
        if maximum == r {
            swc1(g::lighthue, (ps2float::sub(g_, b)).pdiv(spread));
        } else if maximum == g_ {
            swc1(g::lighthue, ps2float::add((ps2float::sub(b, r)).pdiv(spread), 2.0));
        } else if maximum == b {
            swc1(g::lighthue, ps2float::add((ps2float::sub(r, g_)).pdiv(spread), 4.0));
        }
        let hue = ps2float::mul(lwc1(g::lighthue), 60.0);
        swc1(g::lighthue, hue);
        if hue < 0.0 {
            swc1(g::lighthue, ps2float::add(hue, 360.0));
        }
        return;
    }
    swc1(g::lighthue, 0.0);
}

/// Whether a character is playing the animation of the same kind as `n` in the family of animations (3 to 13 and 113 to 123 are the families of a character of
/// a kind; its two animation slots at +96 and +160 of its state say which it plays), or else plays animation `m` (moved by 38 for some) (@ 0x2158b8).
pub fn curAnimBases(chr: i32, n: i32, _unused_a: i32, _unused_b: i32, m: i32) -> i32 {
    let model = lw(chr + 352);
    let state = lw(chr + 32);
    let mut a4 = n;
    let mut range = 0;
    let mut tail_n = true;
    if model != 0 && lw(model + 8) == 0 {
        let near = n - 3;
        let far = n - 113;
        if (near as u32) >= 11 && (far as u32) >= 11 {
            // not in a family: the animation itself
        } else {
            let first_family = n < 113;
            let low = n < 14;
            if !first_family {
                let c = lw(state + 96) - 113;
                if (c as u32) < 330 {
                    return (c.wrapping_rem(11) == far) as i32;
                }
            }
            if low {
                let c = lw(state + 96);
                if c >= 3 && c < 333 {
                    return ((c - 3).wrapping_rem(11) == near) as i32;
                }
            }
            if !first_family {
                let c = lw(state + 160) - 113;
                if (c as u32) < 330 {
                    return (c.wrapping_rem(11) == far) as i32;
                }
            }
            tail_n = false;
            a4 = m;
            range = m - 443;
            if low {
                let c = lw(state + 160);
                if c >= 3 {
                    if c < 333 {
                        return ((c - 3).wrapping_rem(11) == near) as i32;
                    }
                }
            }
        }
    }
    if tail_n {
        a4 = n;
        range = a4 - 443;
    }
    if (range as u32) < 32 && lw(lw(state + 88) + 12) == 1 {
        a4 += 38;
    }
    (lw(state + 96) == a4 || lw(state + 160) == a4) as i32
}

/// The announcer: a random wait of 20 to 40 seconds (a countdown in ticks) and then, in the levels that have one, a random announcement different from the last
/// (a sound number from a range of the level), started as a delayed sound (@ 0x204c78).
pub fn announceTick() {
    let mut countdown = lw(g::announcecountdown);
    let mut announce = lw(g::lastannounce);
    if countdown < 0 {
        countdown = ps2float::mul(ps2float::add(random_unit(), 0.5_f32), 40.0_f32) as i32 * 60;
        sw(g::announcecountdown, countdown);
    }
    let now = countdown - lw(g::updateframesi);
    sw(g::announcecountdown, now);
    if now <= 0 {
        let pick = |base: i32, modulo: Option<u32>, mask: i64| -> i32 {
            let r = newrnd();
            match modulo {
                Some(m) => ((r as i32 as u32) % m) as i32 + base,
                None => (r & mask) as i32 + base,
            }
        };
        let level = lw(g::lvlevelnum);
        let (base, modulo, mask) = match level {
            27 => (302, Some(26), 0),
            18 => (328, Some(13), 0),
            16 => (341, None, 15),
            12 => (357, Some(24), 0),
            10 => (381, None, 1),
            11 => (383, None, 1),
            _ => (-1, None, 0),
        };
        if level == 27 || level == 18 || level == 16 || level == 12 || level == 10 || level == 11 {
            if announce == lw(g::lastannounce) {
                loop {
                    announce = pick(base, modulo, mask);
                    if announce != lw(g::lastannounce) {
                        break;
                    }
                }
            }
        } else {
            announce = -1;
        }
        if announce >= 0 {
            soundDelayStartEx(announce, 0, 0, 0);
        }
        sw(g::lastannounce, announce);
    }
}

/// Work out which of the plugs of a map maker object's tile (5 bytes each: kind 1 to 4 at +0, the position at +1 to +3) have nothing to join them: a bit for each
/// plug that is open (the cell next to the plug in the direction of its kind is empty or has no plug that fits) (@ 0x247858).
pub fn mmMakeSeals(object: i32) -> i32 {
    let x = lw(object + 12);
    let y = lw(object + 16);
    let tile = lw(object + (lw(object + 24) << 2) + 36);
    let mut seals = 0;
    let mut bit = 1;
    let mut i = 0;
    if lw(tile + 28) > 0 {
        let mut plug = tile + 32;
        loop {
            let dx = lbu(plug + 1);
            let dy = lbu(plug + 2);
            sw(g::editlevel, lw(object + 20) + lbu(plug + 3));
            let px = x + (dx << 21);
            let kind = lbu(plug);
            let py = y + (dy << 21);
            let cell = match kind {
                2 => Some(mmCellBits(px, py) & 8),
                3 => Some(mmCellBits(px, py) & 1),
                4 => Some(mmCellBits(px - 0x200000, py) & 2),
                1 => Some(mmCellBits(px, py - 0x200000) & 4),
                _ => None,
            };
            if let Some(cell) = cell {
                let mut joined = false;
                if cell != 0 {
                    let level = lw(object + 20) + lbu(plug + 3);
                    joined = mmFindPlug(lw(g::mmcellobject), plug, px >> 21, py >> 21, level) != 0;
                }
                if !joined {
                    seals |= bit;
                }
            }
            i += 1;
            bit = bit << 1 & 0xfffe;
            plug += 5;
            if i >= lw(tile + 28) {
                break;
            }
        }
    }
    seals
}

/// Get a particle group of kind `kind` (its type info, 16 bytes in `particletypeinfos`: the limit at +12, the kind of its child group at +4): one from the free list, or, when the kind
/// is at its limit or none is free, one taken from the groups in use (the oldest that is not locked) and freed; it is put at the head of the list of its kind (@ 0x296c90).
pub fn getgroup(kind: i32) -> i32 {
    let info = kind << 4;
    let limit = lw(info + (g::particletypeinfos + 12));
    let slot = kind << 2;
    let mut free = lw(0x3b0150);
    if limit != -1 && lw(slot + 0x1fe9a00) >= limit {
        particleFree(getlastinusedlist(kind));
        free = lw(0x3b0150);
    } else if free == 0 {
        let mut other = 0;
        let victim = loop {
            let mut candidate = getlastinusedlist(other);
            other += 1;
            if candidate != 0 && lw(candidate + 56) != 0 {
                candidate = lw(candidate + 64);
                loop {
                    if candidate == 0 || lw(candidate + 56) == 0 {
                        break;
                    }
                    candidate = lw(candidate + 64);
                }
            }
            if candidate != 0 {
                break candidate;
            }
        };
        particleFree(victim);
        free = lw(0x3b0150);
    }
    let group = free;
    let next_free = lw(group + 68);
    sw(0x3b0150, next_free);
    if next_free != 0 {
        sw(next_free + 64, 0);
    }
    let head_at = slot + 0x1fe99c0;
    let head = lw(head_at);
    sw(group + 68, head);
    if head != 0 {
        sw(head + 64, group);
    }
    let count_at = slot + 0x1fe9a00;
    let count = lw(count_at);
    sw(head_at, group);
    sw(count_at, count + 1);
    sw(group, kind);
    sw(group + 56, 1);
    sw(group + 48, 0);
    sw(group + 64, 0);
    let child_kind = lw(info + (g::particletypeinfos + 4));
    sw(group + 60, 0);
    sw(group + 20, 0);
    if child_kind != -1 {
        sw(group + 60, getgroup(child_kind));
    }
    sw(group + 56, 0);
    group
}
