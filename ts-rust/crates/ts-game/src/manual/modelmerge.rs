//! Adding triangles to a model when it is loaded: a tool for the scripts of mods (`make_model`, `model_add`: docs/scripting.md).  The model is one of the pack of objects as the game will read it, whichever mod it comes from.  Nothing
//! here knows what the triangles are.

use crate::*;
use std::collections::HashMap;
use std::sync::Mutex;

type V3 = [f64; 3];

/// (at most so many vertices, strips and `3 * vertices + strips` in a chunk of a part with normals: what the vector unit's memory holds)
const CHUNK_VERTICES: usize = 48;
const CHUNK_STRIPS: usize = 24;
const CHUNK_OUTPUT: usize = 160;
/// The values of a triangle in what a script gives: its texture's number, the point its faces turn away from, and three corners of a place, s, t and a colour.
const TRIANGLE_VALUES: usize = 4 + 3 * 8;

static PACK: Mutex<Option<HashMap<String, (u64, usize)>>> = Mutex::new(None);

/// A triangle being added.
struct Placed {
    number: i32,
    positions: [V3; 3],
    normals: [V3; 3],
    st: [[f32; 2]; 3],
    colours: [[u8; 3]; 3],
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

/// The triangles a script gave.  The normals are the average of the faces that meet at a point, so that the shape is lit as a round thing and not as the flat pieces it is made of, and they point away from the points given.
fn triangles_of(values: &[f64]) -> Vec<Placed> {
    let mut out = Vec::new();
    let mut centres = Vec::new();
    for t in values.chunks_exact(TRIANGLE_VALUES) {
        let corner = |k: usize| &t[4 + 8 * k..4 + 8 * k + 8];
        let at = |k: usize| [corner(k)[0], corner(k)[1], corner(k)[2]];
        let st = |k: usize| [corner(k)[3] as f32, corner(k)[4] as f32];
        let colour = |k: usize| [corner(k)[5] as u8, corner(k)[6] as u8, corner(k)[7] as u8];
        out.push(Placed { number: t[0] as i32, positions: [at(0), at(1), at(2)], normals: [[0.0; 3]; 3], st: [st(0), st(1), st(2)], colours: [colour(0), colour(1), colour(2)] });
        centres.push([t[1], t[2], t[3]]);
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
            let length = dot(n, n).sqrt();
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

/// The model `f` with the triangles `added` added to its part `part`.  The part's arrays are written again after the file's other data, with the part's own in front, and the old ones taken out, so that the file does not hold them twice.
fn add_to_part(f: &[u8], part: usize, added: &[Placed]) -> Option<Vec<u8>> {
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
    // the table of textures, with the textures of what is added
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
    for t in added {
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
        let of_number: Vec<&Placed> = added.iter().filter(|t| t.number == number).collect();
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
    let triangles = u16::from_le_bytes(out[nb + 70..nb + 72].try_into().unwrap()) as usize + added.len();
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

fn read_model(file: &str) -> Option<Vec<u8>> {
    use std::io::{Read, Seek, SeekFrom};
    let mut pack = PACK.lock().unwrap();
    let (place, length) = *pack.get_or_insert_with(read_pack).get(file)?;
    let mut f = std::fs::File::open(ts_sdk::disc_host_path(r"\PAK\OB.PAK;1")?).ok()?;
    f.seek(SeekFrom::Start(place)).ok()?;
    let mut data = vec![0u8; length];
    f.read_exact(&mut data).ok()?;
    Some(data)
}

/// The model `file` of the pack with what a script gave added: for each call of `model_add`, a part and its triangles.
pub fn model_with(file: &str, additions: &[(usize, Vec<f64>)]) -> Option<Vec<u8>> {
    let mut data = read_model(file)?;
    let mut parts: Vec<usize> = additions.iter().map(|a| a.0).collect();
    parts.sort_unstable();
    parts.dedup();
    for part in parts {
        // (the normals of each call apart: what is added in one call is one shape)
        let added: Vec<Placed> = additions.iter().filter(|a| a.0 == part).flat_map(|a| triangles_of(&a.1)).collect();
        data = add_to_part(&data, part, &added)?;
    }
    Some(data)
}

/// Copy a model into the game's memory.
pub fn model_copy(model: &[u8], to: i32) {
    for (k, b) in model.iter().enumerate() {
        sb(to + k as i32, *b as i32);
    }
}
