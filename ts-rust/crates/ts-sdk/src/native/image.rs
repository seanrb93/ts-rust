//! The game's disc, read from an image of it: a cue sheet and its BIN file, or a CHD (`chdman createcd`, or `createdvd`).  The files of the disc are
//! found through its ISO 9660 directory, as the console finds them.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

pub const SECTOR: usize = 2048;

/// Where sector `n` of the data track is: at `base + n * stride + skip` of the BIN file, or of the CHD's data.
struct Layout {
    base: u64,
    stride: u64,
    skip: u64,
}

enum Source {
    Bin(File),
    Chd { chd: chd::Chd<BufReader<File>>, hunk_bytes: u64, at: Option<u32>, hunk: Vec<u8>, packed: Vec<u8> },
}

struct Image {
    source: Source,
    layout: Layout,
    /// every file of the disc: its path in capitals with `/` between folders (`PAK/BG.PAK`) -> its first sector and size
    files: HashMap<String, (u32, u32)>,
}

static IMAGE: Mutex<Option<Image>> = Mutex::new(None);

impl Image {
    fn read_bytes(&mut self, mut at: u64, out: &mut [u8]) -> std::io::Result<()> {
        match &mut self.source {
            Source::Bin(f) => {
                f.seek(SeekFrom::Start(at))?;
                f.read_exact(out)
            }
            Source::Chd { chd, hunk_bytes, at: loaded, hunk, packed } => {
                let mut done = 0;
                while done < out.len() {
                    let n = (at / *hunk_bytes) as u32;
                    if *loaded != Some(n) {
                        *loaded = None;
                        chd.hunk(n).and_then(|mut h| h.read_hunk_in(packed, hunk)).map_err(|e| std::io::Error::other(format!("CHD hunk {n}: {e}")))?;
                        *loaded = Some(n);
                    }
                    let within = (at % *hunk_bytes) as usize;
                    let k = (hunk.len() - within).min(out.len() - done);
                    out[done..done + k].copy_from_slice(&hunk[within..within + k]);
                    done += k;
                    at += k as u64;
                }
                Ok(())
            }
        }
    }

    fn read_sector(&mut self, lba: u32, out: &mut [u8]) -> std::io::Result<()> {
        let at = self.layout.base + lba as u64 * self.layout.stride + self.layout.skip;
        self.read_bytes(at, &mut out[..SECTOR])
    }

    /// Read the ISO 9660 directory: the primary volume descriptor at sector 16, then every folder from the root.
    fn read_directory(&mut self) -> Result<(), String> {
        let mut s = vec![0u8; SECTOR];
        self.read_sector(16, &mut s).map_err(|e| e.to_string())?;
        if s[0] != 1 || &s[1..6] != b"CD001" {
            return Err("the image holds no ISO 9660 disc (no volume descriptor at sector 16)".into());
        }
        let root = &s[156..190];
        let (lba, size) = (u32::from_le_bytes(root[2..6].try_into().unwrap()), u32::from_le_bytes(root[10..14].try_into().unwrap()));
        self.read_folder(lba, size, "", 0)
    }

    fn read_folder(&mut self, lba: u32, size: u32, path: &str, depth: u32) -> Result<(), String> {
        if depth > 16 {
            return Ok(());
        }
        let mut data = vec![0u8; (size as usize).div_ceil(SECTOR) * SECTOR];
        for (k, chunk) in data.chunks_mut(SECTOR).enumerate() {
            self.read_sector(lba + k as u32, chunk).map_err(|e| e.to_string())?;
        }
        let mut folders = Vec::new();
        let mut at = 0;
        while at < size as usize {
            let len = data[at] as usize;
            // (a record does not cross a sector: a 0 ends the records of this one)
            if len == 0 {
                at = (at / SECTOR + 1) * SECTOR;
                continue;
            }
            let r = &data[at..(at + len).min(data.len())];
            at += len;
            if r.len() < 34 {
                break;
            }
            let name = &r[33..(33 + r[32] as usize).min(r.len())];
            if name == [0] || name == [1] {
                continue;
            }
            let name = String::from_utf8_lossy(name);
            let name = name.split(';').next().unwrap_or("").trim_end_matches('.').to_ascii_uppercase();
            let full = if path.is_empty() { name } else { format!("{path}/{name}") };
            let (first, bytes) = (u32::from_le_bytes(r[2..6].try_into().unwrap()), u32::from_le_bytes(r[10..14].try_into().unwrap()));
            if r[25] & 2 != 0 {
                folders.push((first, bytes, full));
            } else {
                self.files.insert(full, (first, bytes));
            }
        }
        for (first, bytes, full) in folders {
            self.read_folder(first, bytes, &full, depth + 1)?;
        }
        Ok(())
    }
}

/// Where a track's sectors keep their 2048 bytes of data: the size of a sector of the image and the bytes before the data.
fn track_kind(kind: &str) -> Option<(u64, u64)> {
    match kind.to_ascii_uppercase().as_str() {
        "MODE1/2048" | "MODE1" | "MODE2_FORM1" => Some((2048, 0)),
        "MODE1/2352" | "MODE1_RAW" => Some((2352, 16)),
        "MODE2/2352" | "MODE2_RAW" => Some((2352, 24)),
        "MODE2/2336" | "MODE2" | "MODE2_FORM_MIX" => Some((2336, 8)),
        _ => None,
    }
}

/// A cue sheet: the BIN file of its first data track and where that track starts in it.
fn open_cue(cue: &Path) -> Result<(Source, Layout), String> {
    let text = std::fs::read_to_string(cue).map_err(|e| format!("{}: {e}", cue.display()))?;
    let mut file: Option<String> = None;
    let mut kind: Option<(u64, u64)> = None;
    for line in text.lines() {
        let line = line.trim();
        let word = line.split_whitespace().next().unwrap_or("").to_ascii_uppercase();
        match word.as_str() {
            "FILE" if kind.is_none() => {
                // FILE "name.bin" BINARY (the name in quotes, or a word)
                let rest = line[4..].trim();
                file = Some(match rest.strip_prefix('"') {
                    Some(q) => q.split('"').next().unwrap_or("").to_string(),
                    None => rest.split_whitespace().next().unwrap_or("").to_string(),
                });
            }
            "TRACK" if kind.is_none() => kind = line.split_whitespace().nth(2).and_then(track_kind),
            "INDEX" if kind.is_some() && line.split_whitespace().nth(1) == Some("01") => {
                let msf: Vec<u64> = line.split_whitespace().nth(2).unwrap_or("").split(':').filter_map(|v| v.parse().ok()).collect();
                let frames = if msf.len() == 3 { (msf[0] * 60 + msf[1]) * 75 + msf[2] } else { 0 };
                let (stride, skip) = kind.unwrap();
                let name = file.ok_or("the cue sheet names no BIN file")?;
                let bin = cue.parent().unwrap_or(Path::new(".")).join(&name);
                let f = File::open(&bin).map_err(|e| format!("{}: {e}", bin.display()))?;
                return Ok((Source::Bin(f), Layout { base: frames * stride, stride, skip }));
            }
            _ => {}
        }
    }
    Err(format!("{}: no data track in the cue sheet", cue.display()))
}

/// A CHD: of a CD (frames of 2352 bytes and 96 of subcode, tracks in the metadata) or of a DVD (sectors of 2048 bytes).
fn open_chd(path: &Path) -> Result<(Source, Layout), String> {
    let f = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut chd = chd::Chd::open(BufReader::new(f), None).map_err(|e| format!("{}: {e} (a CHD that needs a parent is not read)", path.display()))?;
    let refs: Vec<_> = chd.metadata_refs().filter_map(|r| r.ok()).collect();
    let mut tracks = Vec::new();
    for r in refs {
        let Ok(m) = r.read(chd.inner()) else { continue };
        if [*b"CHT2", *b"CHTR"].iter().any(|t| m.metatag == u32::from_be_bytes(*t)) {
            tracks.push(String::from_utf8_lossy(&m.value).trim_end_matches('\0').to_string());
        }
    }
    let field = |t: &str, k: &str| t.split_whitespace().find_map(|f| f.strip_prefix(k).and_then(|v| v.strip_prefix(':')).map(str::to_string));
    let mut layout = Layout { base: 0, stride: 2048, skip: 0 };
    if !tracks.is_empty() {
        const FRAME: u64 = 2448;
        tracks.sort_by_key(|t| field(t, "TRACK").and_then(|v| v.parse::<u32>().ok()).unwrap_or(0));
        let mut frame = 0u64;
        let mut found = false;
        for t in &tracks {
            let frames: u64 = field(t, "FRAMES").and_then(|v| v.parse().ok()).unwrap_or(0);
            // (a pregap kept in the CHD comes before the track's own frames)
            let stored = if field(t, "PGTYPE").is_some_and(|v| v.starts_with('V')) { field(t, "PREGAP").and_then(|v| v.parse().ok()).unwrap_or(0) } else { 0 };
            // (every frame of a CD in a CHD is 2448 bytes; the data is where the track's kind of sector has it)
            if let Some((_, skip)) = field(t, "TYPE").and_then(|k| track_kind(&k)) {
                layout = Layout { base: (frame + stored) * FRAME, stride: FRAME, skip };
                found = true;
                break;
            }
            // (each track is padded to a whole number of 4 frames)
            frame += frames + (4 - frames % 4) % 4;
        }
        if !found {
            return Err(format!("{}: no data track in the CHD", path.display()));
        }
    }
    let hunk_bytes = chd.header().hunk_size() as u64;
    let hunk = chd.get_hunksized_buffer();
    Ok((Source::Chd { chd, hunk_bytes, at: None, hunk, packed: Vec::new() }, layout))
}

/// The image of the disc in the folder `dir`: a `.chd`, or else a `.cue` (the first by name when there are several).
pub fn find_image(dir: &Path) -> Option<PathBuf> {
    let mut found: Vec<PathBuf> = std::fs::read_dir(dir).ok()?.flatten().map(|e| e.path()).filter(|p| p.is_file()).collect();
    found.sort();
    let with = |ext: &str| found.iter().find(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case(ext))).cloned();
    with("chd").or_else(|| with("cue"))
}

/// Open the image `path` (a `.chd` or a `.cue`) and read its directory: the disc the game reads from then on.
pub fn open_image(path: &Path) -> Result<(), String> {
    let is_chd = path.extension().is_some_and(|e| e.eq_ignore_ascii_case("chd"));
    let (source, layout) = if is_chd { open_chd(path)? } else { open_cue(path)? };
    let mut image = Image { source, layout, files: HashMap::new() };
    image.read_directory().map_err(|e| format!("{}: {e}", path.display()))?;
    if !image.files.contains_key("PAK/BG.PAK") {
        return Err(format!("{}: this is not a disc of TimeSplitters (it has no PAK/BG.PAK)", path.display()));
    }
    *IMAGE.lock().unwrap() = Some(image);
    Ok(())
}

fn key(name: &str) -> String {
    let name = name.strip_prefix("cdrom0:").unwrap_or(name);
    let name = name.split(';').next().unwrap_or(name);
    name.split(['\\', '/']).filter(|c| !c.is_empty()).collect::<Vec<_>>().join("/").to_ascii_uppercase()
}

/// A file of the disc (`\PAK\BG.PAK;1`, `PAK/BG.PAK`, ...): its first sector and its size.
pub fn image_file(name: &str) -> Option<(u32, u32)> {
    IMAGE.lock().unwrap().as_ref()?.files.get(&key(name)).copied()
}

/// The files of the disc in the folder `folder` (`PAK`): name and size.
pub fn image_folder(folder: &str) -> Vec<(String, u32)> {
    let prefix = format!("{}/", key(folder));
    let g = IMAGE.lock().unwrap();
    let Some(img) = g.as_ref() else { return Vec::new() };
    img.files.iter().filter_map(|(k, (_, size))| k.strip_prefix(&prefix).filter(|n| !n.contains('/')).map(|n| (n.to_string(), *size))).collect()
}

/// Every file of the disc: its path (`PAK/BG.PAK`) and size.
pub fn image_files() -> Vec<(String, u32)> {
    IMAGE.lock().unwrap().as_ref().map_or(Vec::new(), |img| img.files.iter().map(|(k, (_, size))| (k.clone(), *size)).collect())
}

/// Read `out.len() / 2048` sectors from the sector `lba` on.
pub fn image_read(lba: u32, out: &mut [u8]) -> bool {
    let mut g = IMAGE.lock().unwrap();
    let Some(img) = g.as_mut() else { return false };
    for (k, chunk) in out.chunks_mut(SECTOR).enumerate() {
        if chunk.len() < SECTOR {
            let mut s = [0u8; SECTOR];
            if img.read_sector(lba + k as u32, &mut s).is_err() {
                return false;
            }
            let n = chunk.len();
            chunk.copy_from_slice(&s[..n]);
        } else if img.read_sector(lba + k as u32, chunk).is_err() {
            return false;
        }
    }
    true
}

/// A whole file of the disc (the game's own, whatever the mods change).
pub fn image_read_file(name: &str) -> Option<Vec<u8>> {
    let (lba, size) = image_file(name)?;
    let mut data = vec![0u8; size as usize];
    image_read(lba, &mut data).then_some(data)
}

/// A file of the disc to read in turn, as a file of the host is read.
pub struct ImageReader {
    lba: u32,
    size: u64,
    pos: u64,
}

impl ImageReader {
    pub fn open(name: &str) -> Option<ImageReader> {
        let (lba, size) = image_file(name)?;
        Some(ImageReader { lba, size: size as u64, pos: 0 })
    }
}

impl Read for ImageReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let left = self.size.saturating_sub(self.pos);
        if left == 0 || buf.is_empty() {
            return Ok(0);
        }
        // (whole sectors from where it is, at most 64 at a time)
        let first = self.pos / SECTOR as u64;
        let within = (self.pos % SECTOR as u64) as usize;
        let want = (buf.len() as u64).min(left) as usize;
        let sectors = (within + want).div_ceil(SECTOR).min(64);
        let mut tmp = vec![0u8; sectors * SECTOR];
        if !image_read(self.lba + first as u32, &mut tmp) {
            return Err(std::io::Error::other("the disc image could not be read"));
        }
        let n = want.min(tmp.len() - within);
        buf[..n].copy_from_slice(&tmp[within..within + n]);
        self.pos += n as u64;
        Ok(n)
    }
}

impl Seek for ImageReader {
    fn seek(&mut self, to: SeekFrom) -> std::io::Result<u64> {
        let p = match to {
            SeekFrom::Start(p) => p as i64,
            SeekFrom::Current(d) => self.pos as i64 + d,
            SeekFrom::End(d) => self.size as i64 + d,
        };
        if p < 0 {
            return Err(std::io::Error::other("seek before the start of the file"));
        }
        self.pos = p as u64;
        Ok(self.pos)
    }
}
