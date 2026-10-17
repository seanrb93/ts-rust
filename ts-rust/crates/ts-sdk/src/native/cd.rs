//! CD/DVD drive replacement: a *virtual disc* that serves the game's files from a directory on the host.
//!
//! The original game located files through an ISO9660 directory walk and then read raw 2048-byte sectors by logical
//! sector number (LSN).  On PC the directory is served from the file system: every file the game asks for gets a
//! synthetic sector range, and `sceCdRead` maps sector numbers back to host files.
#![allow(non_snake_case)]

use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use ts_rt::*;

const SECTOR: u64 = 2048;

struct DiscFile {
    lsn: u32,
    sectors: u32,
    path: PathBuf,
    size: u64,
}

struct Disc {
    root: PathBuf,
    files: Vec<DiscFile>,
    next_lsn: u32,
}

static DISC: Mutex<Option<Disc>> = Mutex::new(None);

/// Files that stand in for disc files (the mod loader): lower case disc path without the version (`pak/ob.pak`) -> host file.
static OVERLAY: Mutex<Vec<(String, PathBuf)>> = Mutex::new(Vec::new());

pub fn set_disc_overlay(files: Vec<(String, PathBuf)>) {
    *OVERLAY.lock().unwrap() = files;
}

/// Point the virtual disc at the directory that holds `SLUS_200.90`, `PAK/`, `MUSIC/`, ...
pub fn set_disc_root(root: &Path) {
    *DISC.lock().unwrap() = Some(Disc { root: root.to_path_buf(), files: Vec::new(), next_lsn: 0x400 });
}

/// Resolve a disc path (`\PAK\ANIM.PAK;1`, optionally prefixed by `cdrom0:`) case-insensitively to a host file.
fn resolve(root: &Path, name: &str) -> Option<PathBuf> {
    let name = name.strip_prefix("cdrom0:").unwrap_or(name);
    let name = name.split(';').next().unwrap_or(name);
    let key = name.split(['\\', '/']).filter(|c| !c.is_empty()).collect::<Vec<_>>().join("/").to_ascii_lowercase();
    if let Some((_, p)) = OVERLAY.lock().unwrap().iter().find(|(k, _)| *k == key) {
        return Some(p.clone());
    }
    let mut cur = root.to_path_buf();
    for comp in name.split(['\\', '/']).filter(|c| !c.is_empty()) {
        let mut found = None;
        for e in std::fs::read_dir(&cur).ok()? {
            let e = e.ok()?;
            if e.file_name().to_string_lossy().eq_ignore_ascii_case(comp) {
                found = Some(e.path());
                break;
            }
        }
        cur = found?;
    }
    cur.is_file().then_some(cur)
}

/// Host path of a disc file (for streaming readers that bypass the sector interface).
pub fn disc_host_path(name: &str) -> Option<PathBuf> {
    let g = DISC.lock().unwrap();
    resolve(&g.as_ref()?.root, name)
}

/// Find a file on the virtual disc.  Returns (first LSN, size in bytes).
pub fn disc_find(name: &str) -> Option<(u32, u32)> {
    let mut g = DISC.lock().unwrap();
    let d = g.as_mut()?;
    if std::env::var_os("TS_CDLOG").is_some() {
        eprintln!("[cd] find {name}");
    }
    let path = resolve(&d.root, name)?;
    if let Some(f) = d.files.iter().find(|f| f.path == path) {
        return Some((f.lsn, f.size as u32));
    }
    let size = std::fs::metadata(&path).ok()?.len();
    let sectors = size.div_ceil(SECTOR) as u32;
    let lsn = d.next_lsn;
    d.next_lsn += sectors + 16;
    d.files.push(DiscFile { lsn, sectors, path, size });
    Some((lsn, size as u32))
}

/// Read `count` sectors starting at `lsn` into game memory at `dst`.
pub fn disc_read(lsn: u32, count: u32, dst: i32) -> bool {
    let g = DISC.lock().unwrap();
    let Some(d) = g.as_ref() else { return false };
    let Some(f) = d.files.iter().find(|f| lsn >= f.lsn && lsn < f.lsn + f.sectors.max(1)) else {
        eprintln!("[cd] read of unmapped LSN {lsn:#x} x{count}");
        return false;
    };
    let Ok(mut fh) = std::fs::File::open(&f.path) else { return false };
    let off = (lsn - f.lsn) as u64 * SECTOR;
    if fh.seek(SeekFrom::Start(off)).is_err() {
        return false;
    }
    let want = count as usize * SECTOR as usize;
    let buf = host_slice(dst, want);
    let mut got = 0;
    while got < want {
        match fh.read(&mut buf[got..]) {
            Ok(0) | Err(_) => break,
            Ok(n) => got += n,
        }
    }
    buf[got..].fill(0);
    true
}

/// `sceCdRead(lsn, sectors, buf, mode*)`: performed synchronously.
pub fn sceCdRead(args: &[i64]) -> i32 {
    disc_read(args[0] as u32, args[1] as u32, args[2] as i32) as i32
}
pub fn sceCdSync(_args: &[i64]) -> i32 {
    0
}
pub fn sceCdInit(_args: &[i64]) -> i32 {
    1
}
pub fn sceCdMmode(_args: &[i64]) -> i32 {
    1
}
pub fn sceCdDiskReady(_args: &[i64]) -> i32 {
    2
}
pub fn sceCdGetError(_args: &[i64]) -> i32 {
    0
}

/// The host's local date and time (year, month, day, hours, minutes, seconds).  The console's clock is whatever its owner set,
/// and the game shows it as it is (the clocks of the Mall and Bank levels), so this is the local time of the PC.
#[cfg(windows)]
fn local_time() -> (i64, i64, i64, i64, i64, i64) {
    #[repr(C)]
    #[derive(Default)]
    struct SystemTime {
        year: u16,
        month: u16,
        day_of_week: u16,
        day: u16,
        hour: u16,
        minute: u16,
        second: u16,
        milliseconds: u16,
    }
    extern "system" {
        fn GetLocalTime(t: *mut SystemTime);
    }
    let mut t = SystemTime::default();
    unsafe { GetLocalTime(&mut t) };
    (t.year as i64, t.month as i64, t.day as i64, t.hour as i64, t.minute as i64, t.second as i64)
}

#[cfg(not(windows))]
fn local_time() -> (i64, i64, i64, i64, i64, i64) {
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    let (days, rem) = (t / 86400, t % 86400);
    // civil-from-days (Howard Hinnant), UTC
    let z = days as i64 + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d, (rem / 3600) as i64, ((rem / 60) % 60) as i64, (rem % 60) as i64)
}

/// `sceCdReadClock(clock*)`: fills an 8-byte BCD clock with the local time of the host.
pub fn sceCdReadClock(args: &[i64]) -> i32 {
    let p = args[0] as i32;
    let (y, m, d, hh, mm, ss) = local_time();
    let bcd = |v: i64| (((v / 10) << 4) | (v % 10)) as i32;
    sb(p, 0);
    sb(p + 1, bcd(ss)); // seconds
    sb(p + 2, bcd(mm));
    sb(p + 3, bcd(hh));
    sb(p + 4, 0);
    sb(p + 5, bcd(d));
    sb(p + 6, bcd(m));
    sb(p + 7, bcd(y % 100));
    1
}
