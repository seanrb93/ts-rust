//! Audio files of mods: Ogg Opus (and plain WAV) are read, and sounds are turned into the SPU's ADPCM ("VAG") for the game.

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// Opus always decodes at 48 kHz here.
pub const OPUS_RATE: u32 = 48000;

/// An Ogg Opus file read packet by packet.
pub struct OpusReader {
    packets: ogg::PacketReader<BufReader<File>>,
    dec: opus_decoder::OpusDecoder,
    channels: usize,
    /// samples (per channel) still to drop at the start
    skip: usize,
    gain: f32,
    /// samples decoded so far (with the ones dropped at the start)
    done: u64,
    finished: bool,
}

impl OpusReader {
    pub fn open(path: &Path) -> Result<OpusReader, String> {
        let f = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut packets = ogg::PacketReader::new(BufReader::new(f));
        let head = packets.read_packet().map_err(|e| format!("{}: {e}", path.display()))?.ok_or("empty file")?;
        let h = &head.data;
        if h.len() < 19 || &h[..8] != b"OpusHead" {
            return Err(format!("{}: not an Ogg Opus file", path.display()));
        }
        let channels = h[9] as usize;
        let pre_skip = u16::from_le_bytes([h[10], h[11]]) as usize;
        let gain_q8 = i16::from_le_bytes([h[16], h[17]]);
        let family = h[18];
        if !(1..=2).contains(&channels) || family != 0 {
            return Err(format!("{}: only mono and stereo Opus is supported ({channels} channels, mapping {family})", path.display()));
        }
        // the tags packet
        let _ = packets.read_packet().map_err(|e| e.to_string())?;
        let dec = opus_decoder::OpusDecoder::new(OPUS_RATE, channels).map_err(|e| format!("{}: {e:?}", path.display()))?;
        Ok(OpusReader { packets, dec, channels, skip: pre_skip, gain: 10f32.powf(gain_q8 as f32 / 256.0 / 20.0), done: 0, finished: false })
    }

    /// The next block of the file as interleaved stereo (a mono file is doubled), None at the end.
    pub fn next_block(&mut self) -> Option<Vec<f32>> {
        while !self.finished {
            let p = match self.packets.read_packet() {
                Ok(Some(p)) => p,
                _ => {
                    self.finished = true;
                    return None;
                }
            };
            // (the granule position of the last page counts the samples of the stream, with the ones dropped at the start: what follows it is
            // the encoder's padding)
            let total = if p.last_in_stream() {
                self.finished = true;
                Some(p.absgp_page())
            } else {
                None
            };
            let mut pcm = vec![0f32; 5760 * self.channels];
            let Ok(n) = self.dec.decode_float(&p.data, &mut pcm, false) else {
                continue;
            };
            let start = self.skip.min(n);
            self.skip -= start;
            let mut end = n;
            self.done += n as u64;
            if let Some(total) = total {
                end = n.saturating_sub(self.done.saturating_sub(total) as usize);
            }
            let mut out = Vec::with_capacity(n * 2);
            for i in start..end.max(start) {
                if self.channels == 1 {
                    let v = pcm[i] * self.gain;
                    out.push(v);
                    out.push(v);
                } else {
                    out.push(pcm[2 * i] * self.gain);
                    out.push(pcm[2 * i + 1] * self.gain);
                }
            }
            if !out.is_empty() {
                return Some(out);
            }
        }
        None
    }
}

/// A whole sound file as interleaved stereo f32 and its sample rate (`.opus` or `.wav`).
pub fn read_audio(path: &Path) -> Result<(Vec<f32>, u32), String> {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "opus" | "ogg" => {
            let mut r = OpusReader::open(path)?;
            let mut all = Vec::new();
            while let Some(b) = r.next_block() {
                all.extend_from_slice(&b);
            }
            Ok((all, OPUS_RATE))
        }
        "wav" => read_wav(path),
        _ => Err(format!("{}: not a supported sound file (.opus or .wav)", path.display())),
    }
}

fn read_wav(path: &Path) -> Result<(Vec<f32>, u32), String> {
    let mut d = Vec::new();
    File::open(path).and_then(|mut f| f.read_to_end(&mut d)).map_err(|e| format!("{}: {e}", path.display()))?;
    if d.len() < 44 || &d[..4] != b"RIFF" || &d[8..12] != b"WAVE" {
        return Err(format!("{}: not a WAV file", path.display()));
    }
    let (mut fmt, mut rate, mut ch, mut bits) = (0u16, 0u32, 0usize, 0u16);
    let mut data: &[u8] = &[];
    let mut at = 12;
    while at + 8 <= d.len() {
        let id = &d[at..at + 4];
        let len = u32::from_le_bytes(d[at + 4..at + 8].try_into().unwrap()) as usize;
        let body = &d[at + 8..(at + 8 + len).min(d.len())];
        if id == b"fmt " && body.len() >= 16 {
            fmt = u16::from_le_bytes([body[0], body[1]]);
            ch = u16::from_le_bytes([body[2], body[3]]) as usize;
            rate = u32::from_le_bytes(body[4..8].try_into().unwrap());
            bits = u16::from_le_bytes([body[14], body[15]]);
        } else if id == b"data" {
            data = body;
        }
        at += 8 + len + (len & 1);
    }
    if !(1..=2).contains(&ch) || rate == 0 {
        return Err(format!("{}: only mono and stereo WAV files are supported", path.display()));
    }
    let mut out = Vec::new();
    let bytes = bits as usize / 8;
    for f in data.chunks_exact((ch * bytes).max(1)) {
        let mut s = [0f32; 2];
        for c in 0..ch {
            let b = &f[c * bytes..];
            s[c] = match (fmt, bits) {
                (1, 16) => i16::from_le_bytes([b[0], b[1]]) as f32 / 32768.0,
                (1, 8) => (b[0] as f32 - 128.0) / 128.0,
                (1, 24) => (i32::from_le_bytes([0, b[0], b[1], b[2]]) >> 8) as f32 / 8388608.0,
                (3, 32) => f32::from_le_bytes([b[0], b[1], b[2], b[3]]),
                _ => return Err(format!("{}: unsupported WAV format {fmt} with {bits} bits", path.display())),
            };
        }
        out.push(s[0]);
        out.push(if ch == 2 { s[1] } else { s[0] });
    }
    Ok((out, rate))
}

/// Mix interleaved stereo to mono and resample it from `from` to `to` Hz (a windowed sinc, so that nothing above the new rate's half is left).
pub fn mono_resampled(stereo: &[f32], from: u32, to: u32) -> Vec<f32> {
    let mono: Vec<f32> = stereo.chunks_exact(2).map(|f| (f[0] + f[1]) * 0.5).collect();
    if from == to {
        return mono;
    }
    let ratio = from as f64 / to as f64;
    let cutoff = (to as f64 / from as f64).min(1.0) * 0.94;
    let taps = 24f64;
    let reach = (taps / cutoff).ceil() as i64;
    let n_out = (mono.len() as f64 / ratio) as usize;
    let mut out = Vec::with_capacity(n_out);
    for n in 0..n_out {
        let t = n as f64 * ratio;
        let c = t.floor() as i64;
        let (mut acc, mut wsum) = (0f64, 0f64);
        for k in (c - reach + 1)..=(c + reach) {
            if k < 0 || k as usize >= mono.len() {
                continue;
            }
            let x = (k as f64 - t) * cutoff;
            let sinc = if x.abs() < 1e-9 { 1.0 } else { (std::f64::consts::PI * x).sin() / (std::f64::consts::PI * x) };
            let wpos = (k as f64 - t) / reach as f64;
            let win = 0.5 + 0.5 * (std::f64::consts::PI * wpos).cos();
            let w = sinc * win * cutoff;
            acc += mono[k as usize] as f64 * w;
            wsum += w;
        }
        out.push(if wsum.abs() > 1e-9 { (acc / wsum) as f32 } else { 0.0 });
    }
    out
}

const COEF: [(i32, i32); 5] = [(0, 0), (60, 0), (115, -52), (98, -55), (122, -60)];

/// Encode mono samples as SPU-ADPCM blocks (16 bytes for 28 samples).  `first` / `last` are the flags of the first and last block (the
/// middle ones have 0): a sound that loops has 4 and 3, one that plays once 0 and 1.
pub fn adpcm_encode(samples: &[f32], first: u8, last: u8) -> Vec<u8> {
    let pcm: Vec<i32> = samples.iter().map(|&s| (s * 32767.0).round().clamp(-32768.0, 32767.0) as i32).collect();
    let blocks = pcm.len().div_ceil(28).max(1);
    let mut out = Vec::with_capacity(blocks * 16);
    let (mut h1, mut h2) = (0i32, 0i32);
    for b in 0..blocks {
        let mut chunk = [0i32; 28];
        for (i, c) in chunk.iter_mut().enumerate() {
            *c = pcm.get(b * 28 + i).copied().unwrap_or(0);
        }
        // the filter and shift with the least error (each tried against the history the decoder will have)
        let mut best: Option<(i64, usize, u32, [u8; 14], i32, i32)> = None;
        for (f, &(c0, c1)) in COEF.iter().enumerate() {
            for shift in 0..=12u32 {
                let (mut a1, mut a2) = (h1, h2);
                let mut err = 0i64;
                let mut nibbles = [0u8; 14];
                for (i, &s) in chunk.iter().enumerate() {
                    let pred = (a1 * c0 + a2 * c1 + 32) >> 6;
                    let step = 1i32 << (12 - shift);
                    let nib = (((s - pred) as f64) / step as f64).round().clamp(-8.0, 7.0) as i32;
                    let rec = (((nib << 12) >> shift) + pred).clamp(-32768, 32767);
                    err += ((s - rec) as i64).pow(2);
                    a2 = a1;
                    a1 = rec;
                    let nb = (nib & 15) as u8;
                    if i % 2 == 0 {
                        nibbles[i / 2] = nb;
                    } else {
                        nibbles[i / 2] |= nb << 4;
                    }
                }
                if best.as_ref().map_or(true, |b| err < b.0) {
                    best = Some((err, f, shift, nibbles, a1, a2));
                }
            }
        }
        let (_, f, shift, nibbles, a1, a2) = best.unwrap();
        h1 = a1;
        h2 = a2;
        out.push(shift as u8 | ((f as u8) << 4));
        out.push(if b == 0 && b == blocks - 1 {
            first | last
        } else if b == 0 {
            first
        } else if b == blocks - 1 {
            last
        } else {
            0
        });
        out.extend_from_slice(&nibbles);
    }
    out
}

/// A `.vag` file of the game: the 48 byte header, a silent block, the blocks.
pub fn vag_file(name: &str, rate: u32, blocks: &[u8]) -> Vec<u8> {
    let mut v = Vec::with_capacity(64 + blocks.len());
    v.extend_from_slice(b"VAGp");
    v.extend_from_slice(&0x20u32.to_be_bytes());
    v.extend_from_slice(&0u32.to_be_bytes());
    v.extend_from_slice(&((blocks.len() + 16) as u32).to_be_bytes());
    v.extend_from_slice(&rate.to_be_bytes());
    v.extend_from_slice(&[0u8; 12]);
    let mut n = [0u8; 16];
    for (i, b) in name.bytes().take(15).enumerate() {
        n[i] = b;
    }
    v.extend_from_slice(&n);
    v.extend_from_slice(&[0u8; 16]);
    v.extend_from_slice(blocks);
    v
}
