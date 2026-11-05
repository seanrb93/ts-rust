//! Arms on the first person guns, put there when a gun is loaded.
//!
//! A mod's folder `arms` holds the arms of each character (`c<NN>.arm`: the triangles of the arm that holds the grip and of the other, as they are in the character's model) and how each gun is held (`guns.txt`), both made by
//! `examples/first_person_hands_src/make-hands.py`.  When the game asks for `ob/gun<N>/gun<N>_c<NN>[_l|_m].raw` (a script says so: `on_file`, `on_gun_model`), that model is made here: the gun as the packs have it, whichever mod it comes
//! from, with the arms of the character, posed, added to the part of the gun that carries them.  No file is kept for it, so a set of guns for each character costs nothing on the disc.

use crate::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

type V3 = [f64; 3];

const CURL_LIMIT: f64 = 150.0;
const WRIST_BEND: f64 = 0.14;
/// (at most so many vertices, strips and `3 * vertices + strips` in a chunk of a part with normals: what the vector unit's memory holds)
const CHUNK_VERTICES: usize = 48;
const CHUNK_STRIPS: usize = 24;
const CHUNK_OUTPUT: usize = 160;

#[derive(Clone, Copy)]
struct Corner {
    pos: [f32; 3],
    st: [f32; 2],
    colour: [u8; 3],
}

struct ArmTriangle {
    slot: usize,
    corners: [Corner; 3],
}

struct Character {
    scale: f64,
    numbers: Vec<i32>,
    chains: [Vec<Vec<ArmTriangle>>; 2],
}

struct Hand {
    part: usize,
    chain: usize,
    mirror: bool,
    scale: f64,
    palm: bool,
    fingers: V3,
    thumb: V3,
    at: V3,
    width: f64,
    limit: f64,
    arm: V3,
}

#[derive(Default)]
struct Arms {
    folder: Option<PathBuf>,
    guns: HashMap<(String, String), Vec<Hand>>,
    characters: HashMap<i32, Option<Arc<Character>>>,
    pack: Option<HashMap<String, (u64, usize)>>,
    made: HashMap<String, Arc<Vec<u8>>>,
}

static ON: AtomicBool = AtomicBool::new(false);
static ARMS: Mutex<Option<Arms>> = Mutex::new(None);

/// A mod's folder `arms`: its guns are read now, the characters when they are wanted.
pub fn register_arms_folder(dir: &Path) {
    let Ok(text) = std::fs::read_to_string(dir.join("guns.txt")) else { return };
    let mut guns: HashMap<(String, String), Vec<Hand>> = HashMap::new();
    for line in text.lines() {
        let w: Vec<&str> = line.split_whitespace().collect();
        if w.len() != 22 {
            continue;
        }
        let n = |i: usize| w[i].parse::<f64>().unwrap_or(0.0);
        guns.entry((w[0].to_string(), if w[1] == "-" { String::new() } else { w[1].to_string() })).or_default().push(Hand {
            part: n(2) as usize,
            chain: n(3) as usize,
            mirror: w[4] == "1",
            scale: n(5),
            palm: w[6] == "palm",
            fingers: [n(7), n(8), n(9)],
            thumb: [n(10), n(11), n(12)],
            at: [n(13), n(14), n(15)],
            width: n(16),
            limit: n(17),
            arm: [n(18), n(19), n(20)],
        });
    }
    *ARMS.lock().unwrap() = Some(Arms { folder: Some(dir.to_path_buf()), guns, ..Default::default() });
    ON.store(true, Ordering::Relaxed);
}

/// A variant's parts: `ob/gun1/gun1_c54_l.raw` is the file `ob/gun1/gun1.raw`, the character 54 and the tag `_l`.
fn split_name(name: &str) -> Option<(String, i32, String)> {
    let stem = name.strip_suffix(".raw")?;
    let at = stem.rfind("_c")?;
    let rest = &stem[at + 2..];
    let (number, tag) = match rest.find('_') {
        Some(k) => (&rest[..k], &rest[k..]),
        None => (rest, ""),
    };
    if number.is_empty() || !number.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    Some((format!("{}.raw", &stem[..at]), number.parse().ok()?, tag.to_string()))
}

fn read_character(folder: &Path, number: i32) -> Option<Character> {
    let d = std::fs::read(folder.join(format!("c{number:02}.arm"))).ok()?;
    let mut at = 0usize;
    let word = |at: &mut usize| -> Option<u32> {
        let v = u32::from_le_bytes(d.get(*at..*at + 4)?.try_into().ok()?);
        *at += 4;
        Some(v)
    };
    let scale = f32::from_bits(word(&mut at)?) as f64;
    let count = word(&mut at)?;
    let mut numbers = Vec::new();
    for _ in 0..count {
        numbers.push(word(&mut at)? as i32);
    }
    let mut chains: [Vec<Vec<ArmTriangle>>; 2] = [Vec::new(), Vec::new()];
    for chain in chains.iter_mut() {
        for _ in 0..word(&mut at)? {
            let mut part = Vec::new();
            for _ in 0..word(&mut at)? {
                let slot = word(&mut at)? as usize;
                let mut corners = [Corner { pos: [0.0; 3], st: [0.0; 2], colour: [0; 3] }; 3];
                for c in corners.iter_mut() {
                    for k in 0..3 {
                        c.pos[k] = f32::from_bits(word(&mut at)?);
                    }
                    for k in 0..2 {
                        c.st[k] = f32::from_bits(word(&mut at)?);
                    }
                    c.colour.copy_from_slice(&word(&mut at)?.to_le_bytes()[..3]);
                }
                part.push(ArmTriangle { slot, corners });
            }
            chain.push(part);
        }
    }
    Some(Character { scale, numbers, chains })
}

fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn times(a: V3, s: f64) -> V3 {
    [a[0] * s, a[1] * s, a[2] * s]
}
fn dot(a: V3, b: V3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: V3, b: V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
fn size(a: V3) -> f64 {
    dot(a, a).sqrt()
}
fn unit(a: V3) -> V3 {
    times(a, 1.0 / size(a))
}
fn of(c: &Corner) -> V3 {
    [c.pos[0] as f64, c.pos[1] as f64, c.pos[2] as f64]
}
fn mean(points: &[V3]) -> V3 {
    let mut total = [0.0; 3];
    for p in points {
        total = add(total, *p);
    }
    times(total, 1.0 / points.len() as f64)
}

/// A turn as its three rows.
type Turn = [V3; 3];

fn apply(m: &Turn, v: V3) -> V3 {
    [dot(m[0], v), dot(m[1], v), dot(m[2], v)]
}
fn multiply(a: &Turn, b: &Turn) -> Turn {
    let column = |k: usize| [b[0][k], b[1][k], b[2][k]];
    let mut out = [[0.0; 3]; 3];
    for r in 0..3 {
        for k in 0..3 {
            out[r][k] = dot(a[r], column(k));
        }
    }
    out
}
/// The turn by `angle` about the unit `axis`.
fn about(axis: V3, angle: f64) -> Turn {
    let (s, c) = angle.sin_cos();
    let k = [[0.0, -axis[2], axis[1]], [axis[2], 0.0, -axis[0]], [-axis[1], axis[0], 0.0]];
    let kk = multiply(&k, &k);
    let mut out = [[0.0; 3]; 3];
    for r in 0..3 {
        for q in 0..3 {
            out[r][q] = (r == q) as i32 as f64 + s * k[r][q] + (1.0 - c) * kk[r][q];
        }
    }
    out
}
/// The turn that takes the direction `a` to the direction `b`.
fn turn_between(a: V3, b: V3) -> Turn {
    let (a, b) = (unit(a), unit(b));
    let v = cross(a, b);
    let c = dot(a, b);
    if c > 0.9999999 {
        return [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    }
    if c < -0.9999999 {
        return [[-1.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.0, 0.0, -1.0]];
    }
    let k = [[0.0, -v[2], v[1]], [v[2], 0.0, -v[0]], [-v[1], v[0], 0.0]];
    let kk = multiply(&k, &k);
    let mut out = [[0.0; 3]; 3];
    for r in 0..3 {
        for q in 0..3 {
            out[r][q] = (r == q) as i32 as f64 + k[r][q] + kk[r][q] / (1.0 + c);
        }
    }
    out
}

/// How the points of an arm go on a gun: the hand is turned to the gun with its fingers bent round what they hold, and the rest of the arm comes from where the elbow is, with a bend at the wrist.
struct Pose {
    whole: bool,
    mirror: bool,
    scale: f64,
    limit: f64,
    radius: f64,
    middle: V3,
    down: V3,
    palm_rest: V3,
    thumb_rest: V3,
    knuckle_u: f64,
    tip_u: f64,
    palm_face: f64,
    thumb_most: f64,
    wrist: V3,
    arm_rest: V3,
    turn: Turn,
    arm_turn: Turn,
    hand_at: V3,
    wrist_at: V3,
}

impl Pose {
    fn new(hand: &Hand, parts: &[Vec<ArmTriangle>], character_scale: f64) -> Option<Pose> {
        let corners = |part: &Vec<ArmTriangle>| -> Vec<V3> { part.iter().flat_map(|t| t.corners.iter().map(of)).collect() };
        let scale = hand.scale * character_scale;
        let whole = parts.len() == 1;
        let first = corners(parts.first()?);
        let (grip, elbow) = if whole {
            let mut by_y = first.clone();
            by_y.sort_by(|a, b| a[1].total_cmp(&b[1]));
            let k = (by_y.len() / 6).max(1);
            (by_y[..k].to_vec(), by_y[by_y.len() - k..].to_vec())
        } else {
            (first, corners(parts.last()?))
        };
        let (middle, far) = (mean(&grip), mean(&elbow));
        let down = unit(sub(middle, far));
        let thumb_rest = unit(sub([0.0, 0.0, 1.0], times(down, down[2])));
        let side = if middle[0] > 0.0 { 1.0 } else { -1.0 };
        let palm_rest = times(cross(down, thumb_rest), side);
        let along: Vec<f64> = grip.iter().map(|p| dot(sub(*p, middle), down)).collect();
        let wrist_u = along.iter().cloned().fold(f64::MAX, f64::min);
        let tip_u = along.iter().cloned().fold(f64::MIN, f64::max);
        let wrist = add(middle, times(down, wrist_u));
        let knuckle_u = wrist_u + 0.55 * (tip_u - wrist_u);
        let palm_face = grip.iter().zip(&along).filter(|(_, u)| **u < knuckle_u).map(|(p, _)| dot(sub(*p, middle), palm_rest)).fold(f64::MIN, f64::max);
        let thumb_most = grip.iter().map(|p| dot(sub(*p, middle), thumb_rest)).fold(f64::MIN, f64::max);
        let fingers = unit(hand.fingers);
        let thumb = unit(sub(hand.thumb, times(fingers, dot(hand.thumb, fingers))));
        // (a turn takes down, the thumb and the palm of the rest to the fingers, the thumb and what follows of them)
        let rest = [down, thumb_rest, cross(down, thumb_rest)];
        let goal = [fingers, thumb, cross(fingers, thumb)];
        let mut turn = [[0.0; 3]; 3];
        for r in 0..3 {
            for q in 0..3 {
                turn[r][q] = (0..3).map(|k| goal[k][r] * rest[k][q]).sum();
            }
        }
        let radius = (hand.width / 2.0).min(0.05).max(0.018) / scale.max(1.0e-6);
        let knuckle_rest = add(add(middle, times(down, knuckle_u)), times(palm_rest, palm_face));
        let hand_at = if hand.palm { sub(hand.at, times(apply(&turn, times(palm_rest, palm_face)), scale)) } else { sub(hand.at, times(apply(&turn, sub(knuckle_rest, middle)), scale)) };
        let mut arm_turn = turn_between(sub(far, wrist), hand.arm);
        // (the arm is rolled about itself so that its thumb side is the hand's: no twist at the wrist)
        let axis = unit(hand.arm);
        let got = apply(&arm_turn, thumb_rest);
        let got = sub(got, times(axis, dot(got, axis)));
        let want = sub(thumb, times(axis, dot(thumb, axis)));
        if size(got) > 1.0e-6 && size(want) > 1.0e-6 {
            arm_turn = multiply(&about(axis, dot(cross(got, want), axis).atan2(dot(got, want))), &arm_turn);
        }
        let wrist_at = add(hand_at, times(apply(&turn, sub(wrist, middle)), scale));
        let arm_rest = unit(sub(far, wrist));
        Some(Pose { whole, mirror: hand.mirror, scale, limit: hand.limit, radius, middle, down, palm_rest, thumb_rest, knuckle_u, tip_u, palm_face, thumb_most, wrist, arm_rest, turn, arm_turn, hand_at, wrist_at })
    }

    fn curl(&self, point: V3) -> V3 {
        let d = sub(point, self.middle);
        let (u, n, t) = (dot(d, self.down), dot(d, self.palm_rest), dot(d, self.thumb_rest));
        let (mut new_u, mut new_n) = (u, n);
        if u > self.knuckle_u {
            let side_of = ((t / self.thumb_most.max(1.0e-6) - 0.35) / 0.4).clamp(0.0, 1.0);
            let fade = 1.0 - side_of * side_of * (3.0 - 2.0 * side_of);
            let angle = (u - self.knuckle_u) / (self.tip_u - self.knuckle_u).max(1.0e-6) * self.limit.to_radians() * fade;
            let reach = self.palm_face + self.radius - n;
            new_u = self.knuckle_u + (u - self.knuckle_u) * (1.0 - fade) + reach * angle.sin();
            new_n = self.palm_face + self.radius - reach * angle.cos();
        }
        add(add(add(self.middle, times(self.down, new_u)), times(self.palm_rest, new_n)), times(self.thumb_rest, t))
    }

    fn place(&self, point: V3, of_hand: bool) -> V3 {
        let mut out = if self.whole {
            add(self.hand_at, times(apply(&self.arm_turn, sub(point, self.middle)), self.scale))
        } else if of_hand {
            add(self.hand_at, times(apply(&self.turn, sub(self.curl(point), self.middle)), self.scale))
        } else {
            let arm = add(self.wrist_at, times(apply(&self.arm_turn, sub(point, self.wrist)), self.scale));
            // (the arm bends over its first part: a stiff arm would stick out at the top of the wrist)
            let bend = (dot(sub(point, self.wrist), self.arm_rest) / WRIST_BEND).clamp(0.0, 1.0);
            let bend = bend * bend * (3.0 - 2.0 * bend);
            add(times(arm, bend), times(add(self.hand_at, times(apply(&self.turn, sub(point, self.middle)), self.scale)), 1.0 - bend))
        };
        if self.mirror {
            out[0] = -out[0];
        }
        out
    }
}

/// A triangle of an arm on a gun.
struct Placed {
    number: i32,
    positions: [V3; 3],
    normals: [V3; 3],
    st: [[f32; 2]; 3],
    colours: [[u8; 3]; 3],
}

/// The triangles of a hand on the gun.  The normals are the average of the faces that meet at a point, so the arm is lit as a round thing and not as the flat pieces it is made of.
fn place_hand(hand: &Hand, character: &Character) -> Vec<Placed> {
    let parts = &character.chains[hand.chain];
    let Some(pose) = Pose::new(hand, parts, character.scale) else { return Vec::new() };
    let mut out: Vec<Placed> = Vec::new();
    let mut centres = Vec::new();
    for (index, part) in parts.iter().enumerate() {
        let of_hand = index == 0;
        let all: Vec<V3> = part.iter().flat_map(|t| t.corners.iter().map(of)).collect();
        if all.is_empty() {
            continue;
        }
        let centre = pose.place(mean(&all), of_hand);
        for t in part {
            let order = if hand.mirror { [2, 1, 0] } else { [0, 1, 2] };
            let c = order.map(|k| t.corners[k]);
            out.push(Placed {
                number: character.numbers.get(t.slot).copied().unwrap_or(0),
                positions: c.map(|x| pose.place(of(&x), of_hand)),
                normals: [[0.0; 3]; 3],
                st: c.map(|x| x.st),
                colours: c.map(|x| x.colour),
            });
            centres.push(centre);
        }
    }
    let key = |p: V3| ((p[0] * 1000.0).round() as i64, (p[1] * 1000.0).round() as i64, (p[2] * 1000.0).round() as i64);
    let mut total: HashMap<(i64, i64, i64), V3> = HashMap::new();
    let mut flip = 0.0;
    for (t, centre) in out.iter().zip(&centres) {
        let [a, b, c] = t.positions;
        let n = cross(sub(b, a), sub(c, a));
        flip += dot(n, sub(times(add(add(a, b), c), 1.0 / 3.0), *centre));
        for p in t.positions {
            let e = total.entry(key(p)).or_insert([0.0; 3]);
            *e = add(*e, n);
        }
    }
    let sign = if flip < 0.0 { -1.0 } else { 1.0 };
    for t in out.iter_mut() {
        for k in 0..3 {
            let n = times(total[&key(t.positions[k])], sign);
            let length = size(n);
            t.normals[k] = if length > 1.0e-9 { times(n, 1.0 / length) } else { [0.0, 1.0, 0.0] };
        }
    }
    out
}

fn r32(f: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(f[at..at + 4].try_into().unwrap())
}
fn w32(f: &mut [u8], at: usize, v: u32) {
    f[at..at + 4].copy_from_slice(&v.to_le_bytes());
}
fn align16(f: &mut Vec<u8>) {
    while f.len() % 16 != 0 {
        f.push(0);
    }
}

/// The model `f` with the triangles `arms` added to its part `part`.  The part's arrays are written again after the file's other data, with the part's own in front, and the old ones taken out, so that the file does not hold them twice.
fn add_to_part(f: &[u8], part: usize, arms: &[&Placed]) -> Option<Vec<u8>> {
    let (tex, tab) = (r32(f, 0) as usize, r32(f, 4) as usize);
    let count = r32(f, tab) as usize;
    let bones_at = tab.checked_sub(count * 80)?;
    let b = bones_at + 80 * part;
    if part >= count {
        return None;
    }
    let list_at = r32(f, b + 8) as usize;
    let (strips_at, positions_at, coords_at, colours_at, normals_at) = (r32(f, b + 20) as usize, r32(f, b + 24) as usize, r32(f, b + 28) as usize, r32(f, b + 32) as usize, r32(f, b + 40) as usize);
    if list_at + 24 > f.len() || positions_at == 0 || colours_at == 0 || normals_at == 0 {
        return None;
    }
    // the part's chunks: (slot, first strip, strips, first vertex, vertices, kind)
    let mut chunks = Vec::new();
    let mut at = list_at;
    while at + 24 <= f.len() {
        let c: Vec<i32> = (0..6).map(|k| r32(f, at + 4 * k) as i32).collect();
        if c[5] < 0 {
            break;
        }
        chunks.push(c);
        at += 24;
    }
    if chunks.is_empty() || chunks.iter().any(|c| c[5] != 3) {
        return None;
    }
    let strip_count = chunks.iter().map(|c| (c[1] + c[2]) as usize).max()?;
    let vertex_count = chunks.iter().map(|c| (c[3] + c[4]) as usize).max()?;
    // the table of textures, with the textures of the arms
    let mut table: Vec<[u32; 4]> = Vec::new();
    let mut at = tex;
    while r32(f, at) != 0xffff_ffff {
        table.push([r32(f, at), r32(f, at + 4), r32(f, at + 8), r32(f, at + 12)]);
        at += 16;
    }
    let mut strips = f[strips_at..strips_at + 16 * strip_count].to_vec();
    let mut positions = f[positions_at..positions_at + 12 * vertex_count].to_vec();
    let mut coords = f[coords_at..coords_at + 16 * vertex_count].to_vec();
    let mut colours = f[colours_at..colours_at + 4 * vertex_count].to_vec();
    let mut normals = f[normals_at..normals_at + 12 * vertex_count].to_vec();
    let (mut strip_no, mut vertex_no) = (strip_count, vertex_count);
    let mut numbers: Vec<i32> = Vec::new();
    for t in arms {
        if !numbers.contains(&t.number) {
            numbers.push(t.number);
        }
    }
    for number in numbers {
        let slot = match table.iter().position(|e| e[0] == number as u32) {
            Some(s) => s,
            None => {
                table.push([number as u32, 0, 0, 0]);
                table.len() - 1
            }
        };
        let of_number: Vec<&&Placed> = arms.iter().filter(|t| t.number == number).collect();
        for group in of_number.chunks((CHUNK_VERTICES / 3).min(CHUNK_STRIPS).min((CHUNK_OUTPUT + 2) / 10)) {
            while vertex_no % 4 != 0 {
                positions.extend_from_slice(&[0; 12]);
                coords.extend_from_slice(&[0; 16]);
                colours.extend_from_slice(&[0; 4]);
                normals.extend_from_slice(&[0; 12]);
                vertex_no += 1;
            }
            let first_vertex = vertex_no;
            for (k, t) in group.iter().enumerate() {
                let word = 3u32 | if k + 1 == group.len() { 0x8000 } else { 0 };
                for v in [word, 0x302e_4000, 0x412, 0] {
                    strips.extend_from_slice(&v.to_le_bytes());
                }
                for c in 0..3 {
                    for v in t.positions[c] {
                        positions.extend_from_slice(&(v as f32).to_le_bytes());
                    }
                    for v in [t.st[c][0], t.st[c][1], 1.0, 1.0] {
                        coords.extend_from_slice(&v.to_le_bytes());
                    }
                    colours.extend_from_slice(&[t.colours[c][0], t.colours[c][1], t.colours[c][2], 0x7f]);
                    for v in t.normals[c] {
                        normals.extend_from_slice(&(v as f32).to_le_bytes());
                    }
                    vertex_no += 1;
                }
            }
            chunks.push(vec![slot as i32, strip_no as i32, group.len() as i32, first_vertex as i32, (vertex_no - first_vertex) as i32, 3]);
            strip_no += group.len();
        }
    }
    // the old data of the part comes out, when nothing else is in it
    const OWN: [usize; 7] = [8, 20, 24, 28, 32, 36, 40];
    let mut own = Vec::new();
    let mut others = vec![bones_at];
    for i in 0..count {
        for k in (8..80).step_by(4) {
            let v = r32(f, bones_at + 80 * i + k) as usize;
            if (128..bones_at).contains(&v) && v % 8 == 0 {
                if i == part && OWN.contains(&k) {
                    own.push(v);
                } else {
                    others.push(v);
                }
            }
        }
    }
    let lo = *own.iter().min()?;
    let hi = others.iter().copied().filter(|v| *v > lo).min()?;
    let compact = *own.iter().max()? < hi && !others.iter().any(|v| own.contains(v));
    let (keep_to, resume, delta) = if compact { (lo, hi, hi - lo) } else { (bones_at, bones_at, 0) };
    let mut out = f[..keep_to].to_vec();
    out.extend_from_slice(&f[resume..bones_at]);
    let mut place = |out: &mut Vec<u8>, data: &[u8]| -> u32 {
        align16(out);
        let at = out.len() as u32;
        out.extend_from_slice(data);
        at
    };
    let strips_place = place(&mut out, &strips);
    let positions_place = place(&mut out, &positions);
    let coords_place = place(&mut out, &coords);
    let colours_place = place(&mut out, &colours);
    let normals_place = place(&mut out, &normals);
    let mut table_bytes = Vec::new();
    for e in &table {
        for v in e {
            table_bytes.extend_from_slice(&v.to_le_bytes());
        }
    }
    for v in [0xffff_ffffu32, 0, 0, 0] {
        table_bytes.extend_from_slice(&v.to_le_bytes());
    }
    let table_place = place(&mut out, &table_bytes);
    let mut chunk_bytes = Vec::new();
    for c in &chunks {
        for v in c {
            chunk_bytes.extend_from_slice(&v.to_le_bytes());
        }
    }
    for v in [0i32, 0, 0, 0, 0, -1] {
        chunk_bytes.extend_from_slice(&v.to_le_bytes());
    }
    let chunks_place = place(&mut out, &chunk_bytes);
    align16(&mut out);
    // the bones and what follows: the places behind the data that came out are moved up, and the part has its new ones
    let new_bones = out.len();
    out.extend_from_slice(&f[bones_at..tab]);
    let new_tab = out.len();
    out.extend_from_slice(&f[tab..]);
    if delta != 0 {
        for i in 0..count {
            for k in (8..80).step_by(4) {
                let at = new_bones + 80 * i + k;
                let v = r32(&out, at) as usize;
                if (resume..bones_at).contains(&v) && v % 8 == 0 {
                    w32(&mut out, at, (v - delta) as u32);
                }
            }
        }
    }
    let nb = new_bones + 80 * part;
    w32(&mut out, nb + 8, chunks_place);
    w32(&mut out, nb + 20, strips_place);
    w32(&mut out, nb + 24, positions_place);
    w32(&mut out, nb + 28, coords_place);
    w32(&mut out, nb + 32, colours_place);
    w32(&mut out, nb + 36, 0);
    w32(&mut out, nb + 40, normals_place);
    let triangles = u16::from_le_bytes(out[nb + 70..nb + 72].try_into().unwrap()) as usize + arms.len();
    out[nb + 70..nb + 72].copy_from_slice(&(triangles.min(65535) as u16).to_le_bytes());
    w32(&mut out, 0, table_place);
    w32(&mut out, 4, new_tab as u32);
    Some(out)
}

/// The directory of the pack of objects as the game will read it: name -> (place, size).  A file that is compressed is not here.
fn read_pack() -> HashMap<String, (u64, usize)> {
    use std::io::{Read, Seek, SeekFrom};
    let mut out = HashMap::new();
    let Some(path) = ts_sdk::disc_host_path(r"\PAK\OB.PAK;1") else { return out };
    let Ok(mut file) = std::fs::File::open(path) else { return out };
    let mut head = [0u8; 12];
    if file.read_exact(&mut head).is_err() || &head[..4] != b"P4CK" {
        return out;
    }
    let mut dir = vec![0u8; r32(&head, 8) as usize];
    if file.seek(SeekFrom::Start(r32(&head, 4) as u64)).is_err() || file.read_exact(&mut dir).is_err() {
        return out;
    }
    for e in dir.chunks_exact(60) {
        let end = e[..48].iter().position(|b| *b == 0).unwrap_or(48);
        if r32(e, 56) == 0 {
            out.insert(String::from_utf8_lossy(&e[..end]).to_ascii_lowercase(), (r32(e, 48) as u64, r32(e, 52) as usize));
        }
    }
    out
}

fn read_gun(arms: &mut Arms, file: &str) -> Option<Vec<u8>> {
    use std::io::{Read, Seek, SeekFrom};
    let pack = arms.pack.get_or_insert_with(read_pack);
    let (place, length) = *pack.get(file)?;
    let mut f = std::fs::File::open(ts_sdk::disc_host_path(r"\PAK\OB.PAK;1")?).ok()?;
    f.seek(SeekFrom::Start(place)).ok()?;
    let mut data = vec![0u8; length];
    f.read_exact(&mut data).ok()?;
    Some(data)
}

fn character_of(arms: &mut Arms, number: i32) -> Option<Arc<Character>> {
    let folder = arms.folder.clone()?;
    arms.characters.entry(number).or_insert_with(|| read_character(&folder, number).map(Arc::new)).clone()
}

/// Is `name` a model that can be made here?
fn known(arms: &mut Arms, name: &str) -> bool {
    let Some((file, number, tag)) = split_name(name) else { return false };
    arms.guns.contains_key(&(file, tag)) && character_of(arms, number).is_some()
}

fn make(arms: &mut Arms, name: &str) -> Option<Arc<Vec<u8>>> {
    if let Some(done) = arms.made.get(name) {
        return Some(done.clone());
    }
    let (file, number, tag) = split_name(name)?;
    let character = character_of(arms, number)?;
    let mut data = read_gun(arms, &file)?;
    let hands = arms.guns.get(&(file, tag))?;
    let placed: Vec<(usize, Vec<Placed>)> = hands.iter().map(|h| (h.part, place_hand(h, &character))).collect();
    let mut parts: Vec<usize> = placed.iter().map(|p| p.0).collect();
    parts.dedup();
    parts.sort_unstable();
    parts.dedup();
    for part in parts {
        let on_part: Vec<&Placed> = placed.iter().filter(|p| p.0 == part).flat_map(|p| p.1.iter()).collect();
        data = add_to_part(&data, part, &on_part)?;
    }
    let made = Arc::new(data);
    // (a game's guns are made once each: only the last few are kept, for the size and then the file)
    if arms.made.len() >= 8 {
        arms.made.clear();
    }
    arms.made.insert(name.to_string(), made.clone());
    Some(made)
}

fn read_text(at: i32) -> String {
    let mut text = Vec::new();
    for i in 0..120 {
        let b = lbu(at + i) as u8;
        if b == 0 {
            break;
        }
        text.push(b);
    }
    String::from_utf8_lossy(&text).to_ascii_lowercase()
}

/// Can the model named at `at` be made here (the game's own name, or a script's)?
pub fn arm_model_known(at: i32) -> bool {
    if !ON.load(Ordering::Relaxed) {
        return false;
    }
    let text = read_text(at);
    let name = text.strip_prefix("host0:").unwrap_or(&text);
    let mut g = ARMS.lock().unwrap();
    g.as_mut().map_or(false, |arms| known(arms, name))
}

/// The file that is to be loaded in place of the one named at `name`, if it is a model with arms: the name itself, or what a script says instead (`on_file`).
pub fn arm_model_for(name: i32) -> Option<Arc<Vec<u8>>> {
    if !ON.load(Ordering::Relaxed) {
        return None;
    }
    let text = read_text(name);
    let plain = text.strip_prefix("host0:").unwrap_or(&text).to_string();
    if !plain.starts_with("ob/gun") {
        return None;
    }
    let wanted = {
        let mut g = ARMS.lock().unwrap();
        g.as_mut().map_or(false, |arms| known(arms, &plain))
    };
    let chosen = if wanted {
        plain
    } else {
        script_file_candidates(name).into_iter().find_map(|at| {
            let text = read_text(at);
            let s = text.strip_prefix("host0:").unwrap_or(&text).to_string();
            let mut g = ARMS.lock().unwrap();
            g.as_mut().and_then(|arms| known(arms, &s).then_some(s))
        })?
    };
    let mut g = ARMS.lock().unwrap();
    make(g.as_mut()?, &chosen)
}

/// Copy the model into the game's memory.
pub fn arm_model_copy(model: &[u8], to: i32) {
    for (k, b) in model.iter().enumerate() {
        sb(to + k as i32, *b as i32);
    }
}

/// Test aid: `TS_ARMDUMP=<folder>` writes every model that can be made there, as the files they would be.
pub fn arms_dump_if_asked() {
    let Ok(folder) = std::env::var("TS_ARMDUMP") else { return };
    let mut g = ARMS.lock().unwrap();
    let Some(arms) = g.as_mut() else { return };
    let variants: Vec<(String, String)> = arms.guns.keys().cloned().collect();
    let mut written = 0;
    for number in 0..128 {
        if character_of(arms, number).is_none() {
            continue;
        }
        for (file, tag) in &variants {
            let name = format!("{}_c{number:02}{tag}.raw", file.strip_suffix(".raw").unwrap_or(file));
            if let Some(data) = make(arms, &name) {
                let path = Path::new(&folder).join(&name);
                let _ = std::fs::create_dir_all(path.parent().unwrap());
                let _ = std::fs::write(path, data.as_slice());
                written += 1;
            } else {
                eprintln!("[arms] {name} could not be made");
            }
        }
    }
    eprintln!("[arms] {written} models written to {folder}");
}
