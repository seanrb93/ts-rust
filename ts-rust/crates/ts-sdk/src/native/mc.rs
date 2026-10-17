//! Memory card services (`sceMc*`): a virtual PS2 memory card kept in a host directory.
//!
//! Port 0 / slot 0 always holds a formatted card whose files live under `<save root>/memcard0/`; every other slot
//! reports "no card".  The original API is asynchronous (issue a command, poll `sceMcSync`); commands complete
//! immediately here and `sceMcSync` just reports the stored result once.
#![allow(non_snake_case)]

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;
use ts_rt::*;

const RES_OK: i32 = 0;
const RES_CHANGED: i32 = -1;
const RES_NO_ENTRY: i32 = -4;
const RES_NO_CARD: i32 = -10;

const CARD_CLUSTERS: i64 = 8000; // 1 KB each

struct Mc {
    root: PathBuf,
    files: Vec<Option<File>>,
    pending: Option<(i32, i32)>,
    cwd: String,
    seen_card: bool,
    dir_cursor: usize,
}

static MC: Mutex<Option<Mc>> = Mutex::new(None);

fn new_mc(root: &Path) -> Mc {
    let root = root.join("memcard0");
    let _ = fs::create_dir_all(&root);
    Mc { root, files: Vec::new(), pending: None, cwd: String::new(), seen_card: false, dir_cursor: 0 }
}

/// Choose the host directory that holds the virtual memory card(s).
pub fn set_save_root(root: &Path) {
    *MC.lock().unwrap() = Some(new_mc(root));
}

/// The host directory that holds the virtual memory card(s) (and whatever else the port keeps of its own beside them).
pub fn save_root() -> PathBuf {
    with(|mc| mc.root.parent().map(Path::to_path_buf).unwrap_or_default())
}

fn with<R>(f: impl FnOnce(&mut Mc) -> R) -> R {
    let mut g = MC.lock().unwrap();
    let mc = g.get_or_insert_with(|| new_mc(Path::new("saves")));
    f(mc)
}

impl Mc {
    fn finish(&mut self, cmd: i32, res: i32) -> i32 {
        if std::env::var_os("TS_MCLOG").is_some() {
            eprintln!("[mc] cmd {cmd} -> {res}");
        }
        self.pending = Some((cmd, res));
        0
    }

    /// Card path (`/DIR/FILE`, or relative to the current directory) -> host path; None if it escapes the card.
    fn host_path(&self, name: &str) -> Option<PathBuf> {
        let full = if name.starts_with('/') || name.starts_with('\\') { name.to_string() } else { format!("{}/{}", self.cwd, name) };
        let mut p = self.root.clone();
        for c in full.split(['/', '\\']).filter(|c| !c.is_empty() && *c != ".") {
            if c == ".." {
                return None;
            }
            p.push(c);
        }
        Some(p)
    }

    fn used_clusters(&self) -> i64 {
        fn walk(p: &Path) -> i64 {
            let mut n = 0;
            if let Ok(rd) = fs::read_dir(p) {
                for e in rd.flatten() {
                    if let Ok(m) = e.metadata() {
                        if m.is_dir() {
                            n += 2 + walk(&e.path());
                        } else {
                            n += (m.len() as i64 + 1023) / 1024 + 1;
                        }
                    }
                }
            }
            n
        }
        walk(&self.root)
    }
}

pub fn sceMcInit(_args: &[i64]) -> i32 {
    0
}

/// sceMcGetInfo(port, slot, *type, *free, *format)
pub fn sceMcGetInfo(args: &[i64]) -> i32 {
    let (port, slot) = (args[0] as i32, args[1] as i32);
    let (ty, free, fmt) = (args[2] as i32, args[3] as i32, args[4] as i32);
    with(|mc| {
        if port != 0 || slot != 0 {
            return mc.finish(1, RES_NO_CARD);
        }
        if ty != 0 {
            sw(ty, 2);
        }
        if free != 0 {
            sw(free, (CARD_CLUSTERS - mc.used_clusters()).max(0) as i32);
        }
        if fmt != 0 {
            sw(fmt, 1);
        }
        let r = if mc.seen_card { RES_OK } else { RES_CHANGED };
        mc.seen_card = true;
        mc.finish(1, r)
    })
}

/// sceMcOpen(port, slot, name, flags) -> result = file descriptor
pub fn sceMcOpen(args: &[i64]) -> i32 {
    let (port, slot, name, flags) = (args[0] as i32, args[1] as i32, cstr(args[2] as i32), args[3] as i32);
    if std::env::var_os("TS_MCLOG").is_some() {
        eprintln!("[mc] open {name} flags {flags:#x}");
    }
    with(|mc| {
        if port != 0 || slot != 0 {
            return mc.finish(2, RES_NO_CARD);
        }
        let Some(path) = mc.host_path(&name) else { return mc.finish(2, RES_NO_ENTRY) };
        let mut o = OpenOptions::new();
        o.read(true);
        if flags & 2 != 0 {
            o.write(true);
        }
        if flags & 0x200 != 0 {
            o.create(true);
        }
        if flags & 0x400 != 0 {
            o.truncate(true);
        }
        match o.open(&path) {
            Ok(f) => {
                let idx = mc.files.iter().position(|x| x.is_none()).unwrap_or_else(|| {
                    mc.files.push(None);
                    mc.files.len() - 1
                });
                mc.files[idx] = Some(f);
                mc.finish(2, idx as i32)
            }
            Err(_) => mc.finish(2, RES_NO_ENTRY),
        }
    })
}

pub fn sceMcClose(args: &[i64]) -> i32 {
    let fd = args[0] as usize;
    with(|mc| {
        if let Some(s) = mc.files.get_mut(fd) {
            *s = None;
        }
        mc.finish(3, RES_OK)
    })
}

/// sceMcSeek(fd, offset, origin)
pub fn sceMcSeek(args: &[i64]) -> i32 {
    let (fd, off, origin) = (args[0] as usize, args[1] as i32, args[2] as i32);
    with(|mc| {
        let r = match mc.files.get_mut(fd).and_then(|s| s.as_mut()) {
            Some(f) => {
                let pos = match origin {
                    1 => SeekFrom::Current(off as i64),
                    2 => SeekFrom::End(off as i64),
                    _ => SeekFrom::Start(off.max(0) as u64),
                };
                f.seek(pos).map(|p| p as i32).unwrap_or(RES_NO_ENTRY)
            }
            None => RES_NO_ENTRY,
        };
        mc.finish(4, r)
    })
}

/// sceMcRead(fd, buf, size) -> result = bytes read
pub fn sceMcRead(args: &[i64]) -> i32 {
    let (fd, buf, size) = (args[0] as usize, args[1] as i32, args[2] as usize);
    with(|mc| {
        let r = match mc.files.get_mut(fd).and_then(|s| s.as_mut()) {
            Some(f) => {
                let dst = host_slice(buf, size);
                let mut n = 0;
                while n < size {
                    match f.read(&mut dst[n..]) {
                        Ok(0) | Err(_) => break,
                        Ok(k) => n += k,
                    }
                }
                n as i32
            }
            None => RES_NO_ENTRY,
        };
        mc.finish(5, r)
    })
}

/// sceMcWrite(fd, buf, size) -> result = bytes written
pub fn sceMcWrite(args: &[i64]) -> i32 {
    let (fd, buf, size) = (args[0] as usize, args[1] as i32, args[2] as usize);
    if std::env::var_os("TS_MCLOG").is_some() {
        eprintln!("[mc] write fd {fd} size {size}");
    }
    with(|mc| {
        let r = match mc.files.get_mut(fd).and_then(|s| s.as_mut()) {
            Some(f) => match f.write_all(host_slice(buf, size)) {
                Ok(()) => size as i32,
                Err(_) => -3,
            },
            None => RES_NO_ENTRY,
        };
        mc.finish(6, r)
    })
}

pub fn sceMcFlush(args: &[i64]) -> i32 {
    let fd = args[0] as usize;
    with(|mc| {
        if let Some(Some(f)) = mc.files.get_mut(fd) {
            let _ = f.flush();
        }
        mc.finish(10, RES_OK)
    })
}

/// sceMcMkdir(port, slot, name)
pub fn sceMcMkdir(args: &[i64]) -> i32 {
    let (port, slot, name) = (args[0] as i32, args[1] as i32, cstr(args[2] as i32));
    with(|mc| {
        if port != 0 || slot != 0 {
            return mc.finish(7, RES_NO_CARD);
        }
        let r = match mc.host_path(&name) {
            Some(p) if p.is_dir() => RES_OK,
            Some(p) => fs::create_dir(&p).map(|_| RES_OK).unwrap_or(RES_NO_ENTRY),
            None => RES_NO_ENTRY,
        };
        mc.finish(7, r)
    })
}

/// sceMcChdir(port, slot, name, *current)
pub fn sceMcChdir(args: &[i64]) -> i32 {
    let (port, slot, name) = (args[0] as i32, args[1] as i32, cstr(args[2] as i32));
    with(|mc| {
        if port != 0 || slot != 0 {
            return mc.finish(8, RES_NO_CARD);
        }
        match mc.host_path(&name) {
            Some(p) if p.is_dir() => {
                mc.cwd = if name.starts_with('/') { name.clone() } else { format!("{}/{}", mc.cwd, name) };
                mc.finish(8, RES_OK)
            }
            _ => mc.finish(8, RES_NO_ENTRY),
        }
    })
}

pub fn sceMcDelete(args: &[i64]) -> i32 {
    let (port, slot, name) = (args[0] as i32, args[1] as i32, cstr(args[2] as i32));
    with(|mc| {
        if port != 0 || slot != 0 {
            return mc.finish(9, RES_NO_CARD);
        }
        let r = match mc.host_path(&name) {
            Some(p) if p.is_dir() => fs::remove_dir(&p).map(|_| RES_OK).unwrap_or(-6),
            Some(p) => fs::remove_file(&p).map(|_| RES_OK).unwrap_or(RES_NO_ENTRY),
            None => RES_NO_ENTRY,
        };
        mc.finish(9, r)
    })
}

pub fn sceMcRename(args: &[i64]) -> i32 {
    let (port, slot, old, new) = (args[0] as i32, args[1] as i32, cstr(args[2] as i32), cstr(args[3] as i32));
    with(|mc| {
        if port != 0 || slot != 0 {
            return mc.finish(14, RES_NO_CARD);
        }
        let r = match (mc.host_path(&old), mc.host_path(&new)) {
            (Some(a), Some(b)) => fs::rename(a, b).map(|_| RES_OK).unwrap_or(RES_NO_ENTRY),
            _ => RES_NO_ENTRY,
        };
        mc.finish(14, r)
    })
}

pub fn sceMcSetFileInfo(_args: &[i64]) -> i32 {
    with(|mc| mc.finish(11, RES_OK))
}

pub fn sceMcFormat(_args: &[i64]) -> i32 {
    with(|mc| mc.finish(12, RES_OK))
}

pub fn sceMcUnformat(_args: &[i64]) -> i32 {
    with(|mc| {
        let _ = fs::remove_dir_all(&mc.root);
        let _ = fs::create_dir_all(&mc.root);
        mc.finish(13, RES_OK)
    })
}

/// sceMcGetEntSpace(port, slot, path): how many more entries fit.
pub fn sceMcGetEntSpace(_args: &[i64]) -> i32 {
    with(|mc| mc.finish(15, 1000))
}

fn wildcard(pat: &str, name: &str) -> bool {
    fn m(p: &[u8], n: &[u8]) -> bool {
        match p.first() {
            None => n.is_empty(),
            Some(b'*') => (0..=n.len()).any(|k| m(&p[1..], &n[k..])),
            Some(b'?') => !n.is_empty() && m(&p[1..], &n[1..]),
            Some(&c) => n.first().map_or(false, |&d| d == c) && m(&p[1..], &n[1..]),
        }
    }
    m(pat.as_bytes(), name.as_bytes())
}

/// sceMcStDateTime bytes: Resv2, Sec, Min, Hour, Day, Month, Year (u16 LE).
fn civil(t: SystemTime) -> [u8; 8] {
    let secs = t.duration_since(SystemTime::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) as i64;
    let (days, rem) = (secs.div_euclid(86400), secs.rem_euclid(86400));
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    [0, (rem % 60) as u8, (rem / 60 % 60) as u8, (rem / 3600) as u8, d as u8, m as u8, (y & 0xff) as u8, (y >> 8) as u8]
}

/// sceMcGetDir(port, slot, pattern, mode, maxent, *table) -> result = number of entries stored
pub fn sceMcGetDir(args: &[i64]) -> i32 {
    let (port, slot, pat, mode, maxent, table) =
        (args[0] as i32, args[1] as i32, cstr(args[2] as i32), args[3] as i32, args[4] as usize, args[5] as i32);
    with(|mc| {
        if port != 0 || slot != 0 {
            return mc.finish(16, RES_NO_CARD);
        }
        let (dir, leaf) = match pat.rfind(['/', '\\']) {
            Some(i) => (pat[..=i].to_string(), pat[i + 1..].to_string()),
            None => (String::new(), pat.clone()),
        };
        let Some(dpath) = mc.host_path(&dir) else { return mc.finish(16, RES_NO_ENTRY) };
        let mut entries: Vec<(String, bool, u64, SystemTime)> = Vec::new();
        if dpath != mc.root && wildcard(&leaf, ".") {
            entries.push((".".into(), true, 0, SystemTime::UNIX_EPOCH));
            entries.push(("..".into(), true, 0, SystemTime::UNIX_EPOCH));
        }
        if let Ok(rd) = fs::read_dir(&dpath) {
            let mut v: Vec<_> = rd.flatten().collect();
            v.sort_by_key(|e| e.file_name());
            for e in v {
                let name = e.file_name().to_string_lossy().to_string();
                if !wildcard(&leaf, &name) {
                    continue;
                }
                if let Ok(m) = e.metadata() {
                    entries.push((name, m.is_dir(), m.len(), m.modified().unwrap_or(SystemTime::UNIX_EPOCH)));
                }
            }
        }
        let start = if mode == 0 { 0 } else { mc.dir_cursor.min(entries.len()) };
        let n = (entries.len() - start).min(maxent);
        for (i, (name, is_dir, size, mtime)) in entries[start..start + n].iter().enumerate() {
            let e = table + 64 * i as i32;
            let dt = civil(*mtime);
            for k in 0..8 {
                sb(e + k, dt[k as usize] as i32);
                sb(e + 8 + k, dt[k as usize] as i32);
            }
            sw(e + 16, if *is_dir { 0 } else { *size as i32 });
            sh(e + 20, if *is_dir { 0x8427 } else { 0x8497 });
            sh(e + 22, 0);
            sw(e + 24, 0);
            sw(e + 28, 0);
            for k in 0..32 {
                sb(e + 32 + k, 0);
            }
            for (k, c) in name.bytes().take(31).enumerate() {
                sb(e + 32 + k as i32, c as i32);
            }
        }
        mc.dir_cursor = start + n;
        mc.finish(16, n as i32)
    })
}

/// sceMcSync(mode, *cmd, *result): 1 = finished (outputs written), -1 = nothing pending
pub fn sceMcSync(args: &[i64]) -> i32 {
    let (cmd, res) = (args[1] as i32, args[2] as i32);
    with(|mc| match mc.pending.take() {
        Some((c, r)) => {
            if cmd != 0 {
                sw(cmd, c);
            }
            if res != 0 {
                sw(res, r);
            }
            1
        }
        None => -1,
    })
}
