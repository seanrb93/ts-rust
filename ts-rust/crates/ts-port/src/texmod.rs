//! Textures of mods as PNG files: a PNG is made into the game's texture format (`.qpm`, `Q8`: a palette of 256 colours with alpha and one
//! palette index per pixel) when the mods are loaded.

use std::path::Path;

/// The most pixels a texture can have (the game decodes a texture file into a buffer of 66432 bytes behind the 66688 bytes it is read into).
pub const MAX_PIXELS: usize = 65536;

/// The picture of a PNG file as RGBA8: (width, height, pixels).
pub fn read_png(path: &Path) -> Result<(usize, usize, Vec<[u8; 4]>), String> {
    let f = std::fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    read_png_from(std::io::BufReader::new(f), &path.display().to_string())
}

/// The picture of PNG data (`what` names it in the messages).
pub fn read_png_from<R: std::io::Read>(source: R, what: &str) -> Result<(usize, usize, Vec<[u8; 4]>), String> {
    let mut dec = png::Decoder::new(source);
    dec.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut r = dec.read_info().map_err(|e| format!("{what}: {e}"))?;
    let mut buf = vec![0u8; r.output_buffer_size()];
    let info = r.next_frame(&mut buf).map_err(|e| format!("{what}: {e}"))?;
    let (w, h) = (info.width as usize, info.height as usize);
    let px: Vec<[u8; 4]> = match info.color_type {
        png::ColorType::Rgba => buf.chunks_exact(4).map(|p| [p[0], p[1], p[2], p[3]]).collect(),
        png::ColorType::Rgb => buf.chunks_exact(3).map(|p| [p[0], p[1], p[2], 255]).collect(),
        png::ColorType::GrayscaleAlpha => buf.chunks_exact(2).map(|p| [p[0], p[0], p[0], p[1]]).collect(),
        png::ColorType::Grayscale => buf.iter().map(|&g| [g, g, g, 255]).collect(),
        png::ColorType::Indexed => return Err(format!("{what}: a PNG with a palette was not expanded")),
    };
    if px.len() < w * h {
        return Err(format!("{what}: the picture is incomplete"));
    }
    Ok((w, h, px))
}

/// A picture of a mod as the renderer takes it: RGBA8 with the console's alpha (0 to 128, 128 = opaque), whatever its size.
pub fn load_hd(path: &Path) -> Option<(u32, u32, Vec<u8>)> {
    match read_png(path) {
        Ok((w, h, px)) => finish_hd(w, h, px),
        Err(e) => {
            eprintln!("[mods] {e}");
            None
        }
    }
}

/// The same for a picture the program carries (PNG data in memory).
pub fn load_hd_bytes(bytes: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    match read_png_from(bytes, "(a picture of the program)") {
        Ok((w, h, px)) => finish_hd(w, h, px),
        Err(e) => {
            eprintln!("[icons] {e}");
            None
        }
    }
}

fn finish_hd(w: usize, h: usize, px: Vec<[u8; 4]>) -> Option<(u32, u32, Vec<u8>)> {
    let mut rgba = Vec::with_capacity(w * h * 4);
    for p in &px[..w * h] {
        rgba.extend_from_slice(&[p[0], p[1], p[2], ((p[3] as u32 * 128 + 127) / 255) as u8]);
    }
    // (a picture longer than the limit of the settings is halved until it fits)
    let limit = crate::config::get().hd_texture_max as usize;
    let (mut w, mut h) = (w, h);
    while w.max(h) > limit && w > 1 && h > 1 {
        let (nw, nh) = (w / 2, h / 2);
        let mut next = vec![0u8; nw * nh * 4];
        for y in 0..nh {
            for x in 0..nw {
                for c in 0..4 {
                    let at = |dx: usize, dy: usize| rgba[((2 * y + dy) * w + 2 * x + dx) * 4 + c] as u32;
                    next[(y * nw + x) * 4 + c] = ((at(0, 0) + at(1, 0) + at(0, 1) + at(1, 1) + 2) / 4) as u8;
                }
            }
        }
        rgba = next;
        w = nw;
        h = nh;
    }
    Some((w as u32, h as u32, rgba))
}

/// Reduce the pixels to at most 256 colours (median cut on red, green, blue and alpha; exact when the picture has no more colours).  Returns the
/// palette and the index of each pixel.
fn quantize(px: &[[u8; 4]]) -> (Vec<[u8; 4]>, Vec<u8>) {
    // a clear pixel's colour does not matter: they are all the same colour
    let px: Vec<[u8; 4]> = px.iter().map(|p| if p[3] == 0 { [0, 0, 0, 0] } else { *p }).collect();
    let mut unique: std::collections::HashMap<[u8; 4], usize> = std::collections::HashMap::new();
    for p in &px {
        *unique.entry(*p).or_insert(0) += 1;
    }
    let mut colours: Vec<([u8; 4], usize)> = unique.into_iter().collect();
    colours.sort();
    let palette: Vec<[u8; 4]> = if colours.len() <= 256 {
        colours.iter().map(|c| c.0).collect()
    } else {
        // median cut: split the box with the widest spread at the median (by the number of pixels)
        let mut boxes: Vec<Vec<([u8; 4], usize)>> = vec![colours];
        while boxes.len() < 256 {
            // the box with the most spread (weighted by its pixels) that can still be split
            let mut pick: Option<(usize, usize, u64)> = None;
            for (i, b) in boxes.iter().enumerate() {
                if b.len() < 2 {
                    continue;
                }
                for ch in 0..4 {
                    let (lo, hi) = b.iter().fold((255u8, 0u8), |(l, h), c| (l.min(c.0[ch]), h.max(c.0[ch])));
                    let spread = (hi - lo) as u64 * if ch == 3 { 2 } else { 1 } * b.iter().map(|c| c.1 as u64).sum::<u64>().max(1).ilog2().max(1) as u64;
                    if pick.map_or(true, |p| spread > p.2) {
                        pick = Some((i, ch, spread));
                    }
                }
            }
            let Some((i, ch, _)) = pick else { break };
            let mut b = boxes.swap_remove(i);
            b.sort_by_key(|c| c.0[ch]);
            let total: usize = b.iter().map(|c| c.1).sum();
            let mut acc = 0;
            let mut cut = b.len() / 2;
            for (k, c) in b.iter().enumerate() {
                acc += c.1;
                if acc * 2 >= total {
                    cut = (k + 1).clamp(1, b.len() - 1);
                    break;
                }
            }
            let second = b.split_off(cut);
            boxes.push(b);
            boxes.push(second);
        }
        boxes
            .iter()
            .map(|b| {
                let total: u64 = b.iter().map(|c| c.1 as u64).sum::<u64>().max(1);
                let mut m = [0u64; 4];
                for c in b {
                    for ch in 0..4 {
                        m[ch] += c.0[ch] as u64 * c.1 as u64;
                    }
                }
                [(m[0] / total) as u8, (m[1] / total) as u8, (m[2] / total) as u8, (m[3] / total) as u8]
            })
            .collect()
    };
    // the nearest palette entry of every colour of the picture
    let mut nearest: std::collections::HashMap<[u8; 4], u8> = std::collections::HashMap::new();
    let mut out = Vec::with_capacity(px.len());
    for p in &px {
        let i = *nearest.entry(*p).or_insert_with(|| {
            palette
                .iter()
                .enumerate()
                .min_by_key(|(_, q)| (0..4).map(|c| (q[c] as i32 - p[c] as i32).pow(2) * if c == 3 { 3 } else { 1 }).sum::<i32>())
                .map_or(0, |x| x.0 as u8)
        });
        out.push(i);
    }
    (palette, out)
}

/// A PNG as the game's `Q8` texture file.
pub fn png_to_qpm(path: &Path) -> Result<(Vec<u8>, usize, usize), String> {
    let (w, h, px) = read_png(path)?;
    if w == 0 || h == 0 || w * h > MAX_PIXELS {
        return Err(format!("{}: {w}x{h} is {} pixels; the game can load at most {MAX_PIXELS} (256x256, 512x128, ...)", path.display(), w * h));
    }
    let (pal, idx) = quantize(&px[..w * h]);
    let third = "255 v 256";
    let mut head = format!("Q8\n{w} {h}\n");
    let mut line = third.to_string();
    while (head.len() + line.len() + 1) % 4 != 0 {
        line.push(' ');
    }
    head.push_str(&line);
    head.push('\n');
    let mut out = head.into_bytes();
    for i in 0..256 {
        let c = pal.get(i).copied().unwrap_or([0, 0, 0, 0]);
        // (the game's alpha is 0..127, 127 = opaque)
        out.extend_from_slice(&[c[0], c[1], c[2], (c[3] as u32 * 127 / 255) as u8]);
    }
    out.extend_from_slice(&idx);
    Ok((out, w, h))
}
