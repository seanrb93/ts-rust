//! Mod loader.  Every folder in `mods/` next to the program is a mod.  What a mod can hold:
//!
//! * `PACK.PAK/<path inside the pack>`: a file that replaces the one of that name in the game's pack `PACK.PAK` (`ANIM.PAK`, `BG.PAK`,
//!   `FRONTEND.PAK`, `MISC.PAK`, `OB.PAK`, `SFX.PAK`), or is added to it (textures, models, animations, sounds, level files: whatever
//!   the game reads by name from a pack).  A path that does not start with a pack's name replaces the file of that name in whichever
//!   pack has it.
//! * `MUSIC/<name>.MSC`: replaces a music stream.
//! * `patches.txt`: changes to the game's data in memory when it starts (tables such as the weapons or characters).  One per line:
//!   `<symbol or 0xADDRESS>[+offset] <b|h|w|f> <value>[ <value>...]` (byte, halfword, word, float; `#` starts a comment) or
//!   `<symbol or 0xADDRESS>[+offset] s <text>` (the text and a zero byte; it must fit where the old text was).
//! * `mod.ini` with `enabled = false` to switch the mod off.
//!
//! The packs are not changed: the changed packs are built in `mods/.cache/` when the mods change, and the game reads those.  Mods are
//! applied in alphabetical order of their folders; a later mod wins.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

const PACKS: [&str; 6] = ["ANIM.PAK", "BG.PAK", "FRONTEND.PAK", "MISC.PAK", "OB.PAK", "SFX.PAK"];

struct Mod {
    name: String,
    dir: PathBuf,
}

/// Is the mod switched off by its `mod.ini` (`enabled = false`, `0` or `no`)?
fn switched_off(ini: &str) -> bool {
    ini.lines().any(|l| {
        let l = l.replace(' ', "").to_ascii_lowercase();
        l == "enabled=false" || l == "enabled=0" || l == "enabled=no"
    })
}

/// The folders of `dir` that are mods, switched on or off, in the order they are applied.
fn all_mods_in(dir: &Path) -> Vec<(Mod, bool)> {
    let mut v: Vec<(Mod, bool)> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') || name.starts_with('_') {
                return None;
            }
            let ini = std::fs::read_to_string(e.path().join("mod.ini")).unwrap_or_default();
            Some((Mod { name, dir: e.path() }, !switched_off(&ini)))
        })
        .collect();
    v.sort_by(|a, b| a.0.name.to_ascii_lowercase().cmp(&b.0.name.to_ascii_lowercase()));
    v
}

fn mods_in(dir: &Path) -> Vec<Mod> {
    all_mods_in(dir).into_iter().filter(|m| m.1).map(|m| m.0).collect()
}

/// The `mods` folder, and the mods that were loaded when the program started.
static MODS_DIR: std::sync::Mutex<Option<PathBuf>> = std::sync::Mutex::new(None);
static LOADED: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

/// A mod folder for the Mods page.
pub struct Listed {
    pub name: String,
    /// switched on in its `mod.ini` now
    pub enabled: bool,
    /// was loaded when the program started (a change of `enabled` takes effect at the next start)
    pub loaded: bool,
}

/// The mods that are installed.
pub fn list() -> Vec<Listed> {
    let Some(dir) = MODS_DIR.lock().unwrap().clone() else { return Vec::new() };
    let loaded = LOADED.lock().unwrap().clone();
    all_mods_in(&dir).into_iter().map(|(m, enabled)| Listed { loaded: loaded.contains(&m.name), name: m.name, enabled }).collect()
}

/// Switch a mod on or off in its `mod.ini` (the other lines of the file stay as they are).
pub fn set_enabled(name: &str, on: bool) {
    let Some(dir) = MODS_DIR.lock().unwrap().clone() else { return };
    let path = dir.join(name).join("mod.ini");
    let old = std::fs::read_to_string(&path).unwrap_or_default();
    let line = format!("enabled = {on}");
    let mut done = false;
    let mut out: Vec<String> = old
        .lines()
        .map(|l| {
            if l.replace(' ', "").to_ascii_lowercase().starts_with("enabled=") {
                done = true;
                line.clone()
            } else {
                l.to_string()
            }
        })
        .collect();
    if !done {
        out.push(line);
    }
    let _ = std::fs::write(&path, out.join("
") + "
");
}

fn files_under(dir: &Path, prefix: &str, out: &mut Vec<(String, PathBuf)>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    let mut entries: Vec<_> = rd.flatten().collect();
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let n = e.file_name().to_string_lossy().into_owned();
        let rel = if prefix.is_empty() { n.clone() } else { format!("{prefix}/{n}") };
        if e.path().is_dir() {
            files_under(&e.path(), &rel, out);
        } else {
            out.push((rel, e.path()));
        }
    }
}

/// One entry of a pack's directory: 48 bytes of name, then the offset and size of the file in the pack, then a word of zero.
struct Entry {
    name: Vec<u8>,
    offset: u32,
    size: u32,
}

fn read_pack(path: &Path) -> Option<(Vec<u8>, Vec<Entry>)> {
    let d = std::fs::read(path).ok()?;
    if d.len() < 16 || &d[0..4] != b"P4CK" {
        return None;
    }
    let dir_at = u32::from_le_bytes(d[4..8].try_into().ok()?) as usize;
    let dir_len = u32::from_le_bytes(d[8..12].try_into().ok()?) as usize;
    if dir_at + dir_len > d.len() {
        return None;
    }
    let entries = d[dir_at..dir_at + dir_len]
        .chunks_exact(60)
        .map(|e| {
            let n = e[..48].iter().position(|&c| c == 0).unwrap_or(48);
            Entry { name: e[..n].to_vec(), offset: u32::from_le_bytes(e[48..52].try_into().unwrap()), size: u32::from_le_bytes(e[52..56].try_into().unwrap()) }
        })
        .collect();
    Some((d, entries))
}

/// The names of the files of the packs the game will read (the packs the mods changed, or the game's own), given to the game so that a mod's model file that is in none of them is noticed
/// before the game asks for it.  Only the directories of the packs are read.
pub fn publish_pack_names() {
    use std::io::{Read, Seek, SeekFrom};
    let mut names = std::collections::HashSet::new();
    for pack in PACKS {
        let Some(path) = ts_sdk::disc_host_path(&format!(r"\PAK\{pack};1")) else { continue };
        let Ok(mut f) = std::fs::File::open(&path) else { continue };
        let mut head = [0u8; 12];
        if f.read_exact(&mut head).is_err() || &head[0..4] != b"P4CK" {
            continue;
        }
        let dir_at = u32::from_le_bytes(head[4..8].try_into().unwrap()) as u64;
        let dir_len = u32::from_le_bytes(head[8..12].try_into().unwrap()) as usize;
        let mut dir = vec![0u8; dir_len];
        if f.seek(SeekFrom::Start(dir_at)).is_err() || f.read_exact(&mut dir).is_err() {
            continue;
        }
        for e in dir.chunks_exact(60) {
            let n = e[..48].iter().position(|&c| c == 0).unwrap_or(48);
            names.insert(String::from_utf8_lossy(&e[..n]).to_ascii_lowercase());
        }
    }
    ts_game::set_pack_names(names);
}

/// Build `out` from the pack `src` with the files of `changes` (name in the pack -> host file) replaced or added.
fn build_pack(src: &Path, changes: &BTreeMap<String, PathBuf>, out: &Path) -> Result<(), String> {
    let (d, entries) = read_pack(src).ok_or_else(|| format!("{} is not a pack", src.display()))?;
    let dir_at = u32::from_le_bytes(d[4..8].try_into().unwrap()) as usize;
    let mut data = d[..dir_at].to_vec();
    let mut entries: Vec<(Vec<u8>, u32, u32)> = entries.into_iter().map(|e| (e.name, e.offset, e.size)).collect();
    for (name, host) in changes {
        let bytes = std::fs::read(host).map_err(|e| format!("{}: {e}", host.display()))?;
        if name.len() > 47 {
            return Err(format!("{name}: a name in a pack has at most 47 characters"));
        }
        while data.len() % 2048 != 0 {
            data.push(0);
        }
        let offset = data.len() as u32;
        data.extend_from_slice(&bytes);
        match entries.iter_mut().find(|e| e.0 == name.as_bytes()) {
            Some(e) => {
                e.1 = offset;
                e.2 = bytes.len() as u32;
            }
            None => entries.push((name.as_bytes().to_vec(), offset, bytes.len() as u32)),
        }
    }
    while data.len() % 2048 != 0 {
        data.push(0);
    }
    let new_dir_at = data.len() as u32;
    for (name, offset, size) in &entries {
        let mut e = [0u8; 60];
        e[..name.len()].copy_from_slice(name);
        e[48..52].copy_from_slice(&offset.to_le_bytes());
        e[52..56].copy_from_slice(&size.to_le_bytes());
        data.extend_from_slice(&e);
    }
    data[4..8].copy_from_slice(&new_dir_at.to_le_bytes());
    data[8..12].copy_from_slice(&((entries.len() * 60) as u32).to_le_bytes());
    std::fs::write(out, data).map_err(|e| format!("{}: {e}", out.display()))
}

/// What of an original sound decides how a replacement is made: its sample rate (the game's table says how fast to play it, not the file) and the
/// flags of its first and last block (a sound that loops has 4 and 3).
fn original_sound(packs: &mut BTreeMap<&'static str, Option<(Vec<u8>, Vec<Entry>)>>, game_dir: &Path, name: &str) -> Option<(u32, u8, u8)> {
    for pack in ["SFX.PAK", "FRONTEND.PAK"] {
        let (d, entries) = packs.entry(pack).or_insert_with(|| read_pack(&game_dir.join("PAK").join(pack))).as_ref()?;
        if let Some(e) = entries.iter().find(|e| e.name == name.as_bytes()) {
            let b = d.get(e.offset as usize..(e.offset + e.size) as usize)?;
            if b.len() < 0x60 || &b[..4] != b"VAGp" {
                return None;
            }
            let rate = u32::from_be_bytes(b[0x10..0x14].try_into().ok()?);
            return Some((rate, b[0x41], b[b.len() - 15]));
        }
    }
    None
}

/// The largest sound file the game can load (it reads a file into a buffer of 133120 bytes).
const MAX_SOUND: usize = 133_000;

/// A sound given as Opus or WAV is made into the game's `.vag` in `mods/.cache/converted/`: mono, at the sample rate of the sound it
/// replaces (22050 Hz for a new one), with the same looping.  Returns the name of the file in the pack and the converted file.
fn convert_sound(game_dir: &Path, cache: &Path, mod_name: &str, rel: &str, host: &Path, packs: &mut BTreeMap<&'static str, Option<(Vec<u8>, Vec<Entry>)>>) -> Option<(String, PathBuf)> {
    let (stem, _) = rel.rsplit_once('.')?;
    let first = stem.split('/').next().unwrap_or("");
    let inner_stem = if PACKS.iter().any(|p| p.eq_ignore_ascii_case(first)) && stem.contains('/') { &stem[first.len() + 1..] } else { stem };
    let name = format!("{inner_stem}.vag");
    let out_dir = cache.join("converted").join(mod_name);
    let _ = std::fs::create_dir_all(&out_dir);
    let out = out_dir.join(name.replace('/', "__"));
    let newer = |a: &Path, b: &Path| match (std::fs::metadata(a).and_then(|m| m.modified()), std::fs::metadata(b).and_then(|m| m.modified())) {
        (Ok(x), Ok(y)) => x >= y,
        _ => false,
    };
    let rel_name = if PACKS.iter().any(|p| p.eq_ignore_ascii_case(first)) && stem.contains('/') { format!("{}/{}", first, name) } else { name.clone() };
    if out.is_file() && newer(&out, host) {
        return Some((rel_name, out));
    }
    let (rate, f_first, f_last) = original_sound(packs, game_dir, &name).unwrap_or((22050, 0, 1));
    let (pcm, from) = match ts_audio::codec::read_audio(host) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("[mods] {mod_name}: {e}");
            return None;
        }
    };
    let mut mono = ts_audio::codec::mono_resampled(&pcm, from, rate);
    let room = (MAX_SOUND - 64) / 16 * 28;
    if mono.len() > room {
        eprintln!("[mods] {mod_name}: {rel} is longer than the game can load ({:.1} s at {rate} Hz): cut", room as f32 / rate as f32);
        mono.truncate(room);
    }
    let blocks = ts_audio::codec::adpcm_encode(&mono, f_first, f_last);
    let stem_name = inner_stem.rsplit('/').next().unwrap_or(inner_stem);
    if let Err(e) = std::fs::write(&out, ts_audio::codec::vag_file(stem_name, rate, &blocks)) {
        eprintln!("[mods] {mod_name}: {}: {e}", out.display());
        return None;
    }
    eprintln!("[mods] {mod_name}: {rel} -> {name} ({:.1} s at {rate} Hz{})", mono.len() as f32 / rate as f32, if f_first & 4 != 0 { ", looping" } else { "" });
    Some((rel_name, out))
}

/// The size of a texture of the game (its `.qpm` header), to tell a mod that its picture has another size.
fn original_texture_size(packs: &mut BTreeMap<&'static str, Option<(Vec<u8>, Vec<Entry>)>>, game_dir: &Path, name: &str) -> Option<(usize, usize)> {
    let (d, entries) = packs.entry("MISC.PAK").or_insert_with(|| read_pack(&game_dir.join("PAK").join("MISC.PAK"))).as_ref()?;
    let e = entries.iter().find(|e| e.name == name.as_bytes())?;
    let b = d.get(e.offset as usize..(e.offset + e.size.min(64)) as usize)?;
    let text = String::from_utf8_lossy(b);
    let mut lines = text.lines();
    lines.next()?;
    let mut dims = lines.next()?.split_whitespace().filter_map(|x| x.parse::<usize>().ok());
    Some((dims.next()?, dims.next()?))
}

/// A picture given as PNG replaces the texture of that name when it is drawn, whatever its size (the renderer shows it in place of the game's own
/// picture over the same surface; `ts_gfx::hd`).  The file is read when the texture is first used.
fn register_picture(game_dir: &Path, mod_name: &str, rel: &str, host: &Path, packs: &mut BTreeMap<&'static str, Option<(Vec<u8>, Vec<Entry>)>>) {
    let Some((stem, _)) = rel.rsplit_once('.') else { return };
    let first = stem.split('/').next().unwrap_or("");
    let inner = if PACKS.iter().any(|p| p.eq_ignore_ascii_case(first)) && stem.contains('/') { &stem[first.len() + 1..] } else { stem };
    let name = format!("{inner}.qpm");
    // (a texture of the mod's own: the mod has a .qpm of that name beside the picture, a small one the game's loader takes; the picture is shown in its place at any size)
    let own = host.with_extension("qpm").is_file();
    if original_texture_size(packs, game_dir, &name).is_none() && !own {
        eprintln!("[mods] {mod_name}: {name} is not a texture of the game: {rel} is not used");
        return;
    }
    ts_gfx::hd::register(inner, host.to_path_buf());
    eprintln!("[mods] {mod_name}: {rel} is shown in place of {name}");
}

/// A picture given as PNG is made into the game's texture format in `mods/.cache/converted/`.  Returns the name of the texture in the pack and the
/// converted file.
fn convert_texture(game_dir: &Path, cache: &Path, mod_name: &str, rel: &str, host: &Path, packs: &mut BTreeMap<&'static str, Option<(Vec<u8>, Vec<Entry>)>>) -> Option<(String, PathBuf)> {
    let (stem, _) = rel.rsplit_once('.')?;
    let first = stem.split('/').next().unwrap_or("");
    let has_pack = PACKS.iter().any(|p| p.eq_ignore_ascii_case(first)) && stem.contains('/');
    let inner = if has_pack { &stem[first.len() + 1..] } else { stem };
    let name = format!("{inner}.qpm");
    let out_dir = cache.join("converted").join(mod_name);
    let _ = std::fs::create_dir_all(&out_dir);
    let out = out_dir.join(name.replace('/', "__"));
    let rel_name = if has_pack { format!("{first}/{name}") } else { name.clone() };
    let fresh = match (std::fs::metadata(&out).and_then(|m| m.modified()), std::fs::metadata(host).and_then(|m| m.modified())) {
        (Ok(a), Ok(b)) => a >= b,
        _ => false,
    };
    if out.is_file() && fresh {
        return Some((rel_name, out));
    }
    let (bytes, w, h) = match crate::texmod::png_to_qpm(host) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("[mods] {mod_name}: {e}");
            return None;
        }
    };
    match original_texture_size(packs, game_dir, &name) {
        Some((ow, oh)) if (ow, oh) != (w, h) => eprintln!("[mods] {mod_name}: {rel} is {w}x{h}, the texture it replaces is {ow}x{oh} (on a model the picture is stretched over the same surface; a picture the game draws as a sprite, such as the HUD's, is drawn at its new size)"),
        Some(_) => {}
        None => eprintln!("[mods] {mod_name}: {name} is not a texture of the game: it is added (a model has to ask for it by name)"),
    }
    if !w.is_power_of_two() || !h.is_power_of_two() {
        eprintln!("[mods] {mod_name}: {rel} is {w}x{h}: a side that is not a power of two (64, 128, 256, ...) does not display properly in the game; make it a power of two");
    }
    if let Err(e) = std::fs::write(&out, bytes) {
        eprintln!("[mods] {mod_name}: {}: {e}", out.display());
        return None;
    }
    eprintln!("[mods] {mod_name}: {rel} -> {name} ({w}x{h}, 256 colours)");
    Some((rel_name, out))
}

fn stamp(files: &BTreeMap<String, PathBuf>) -> String {
    let mut s = String::new();
    for (name, p) in files {
        let m = std::fs::metadata(p).ok();
        let t = m.as_ref().and_then(|m| m.modified().ok()).and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map_or(0, |d| d.as_nanos());
        s.push_str(&format!("{name} {} {} {t}\n", p.display(), m.map_or(0, |m| m.len())));
    }
    s
}

/// Load the mods: set up the file overlay of the virtual disc and apply the memory patches.
pub fn load(game_dir: &Path, exe: &Path) {
    let root = game_dir.join("mods");
    *MODS_DIR.lock().unwrap() = Some(root.clone());
    let mods = mods_in(&root);
    *LOADED.lock().unwrap() = mods.iter().map(|m| m.name.clone()).collect();
    if mods.is_empty() {
        return;
    }
    let cache = root.join(".cache");
    let _ = std::fs::create_dir_all(&cache);
    let mut pack_changes: BTreeMap<&str, BTreeMap<String, PathBuf>> = BTreeMap::new();
    let mut overlay: Vec<(String, PathBuf)> = Vec::new();
    let mut patches: Vec<(String, String)> = Vec::new();
    let mut data_files: Vec<(String, &str, PathBuf)> = Vec::new();
    let mut level_lists: Vec<(String, PathBuf)> = Vec::new();
    // the packs' contents, to find where a file without a pack name belongs
    let contents: Vec<(&str, Vec<Vec<u8>>)> = PACKS
        .iter()
        .filter_map(|p| read_pack(&game_dir.join("PAK").join(p)).map(|(_, e)| (*p, e.into_iter().map(|e| e.name).collect())))
        .collect();
    let mut original_packs: BTreeMap<&'static str, Option<(Vec<u8>, Vec<Entry>)>> = BTreeMap::new();
    for m in &mods {
        let mut files = Vec::new();
        files_under(&m.dir, "", &mut files);
        let mut count = 0;
        for (rel, host) in files {
            let lower = rel.to_ascii_lowercase();
            let first = lower.split('/').next().unwrap_or("");
            // (what a mod author keeps next to the files: notes, and the sources of the pictures and sounds in a folder that starts with _ or is called art, source or docs)
            if matches!(first, "art" | "source" | "src" | "docs") || first.starts_with('_') || first.starts_with('.') {
                if lower.contains('/') {
                    continue;
                }
            }
            if lower == "levels.txt" {
                level_lists.push((m.name.clone(), host));
                continue;
            }
            if lower == "mod.ini" || lower == "patches.txt" || lower.ends_with(".md") || lower == "readme.txt" || (lower.ends_with(".txt") && !lower.contains('/')) {
                if lower == "patches.txt" {
                    patches.push((m.name.clone(), host.to_string_lossy().into_owned()));
                } else if lower == "weapons.txt" {
                    data_files.push((m.name.clone(), "weapon", host.clone()));
                } else if lower == "characters.txt" {
                    data_files.push((m.name.clone(), "character", host.clone()));
                } else if lower == "modes.txt" {
                    data_files.push((m.name.clone(), "mode", host.clone()));
                }
                continue;
            }
            let audio = lower.ends_with(".opus") || lower.ends_with(".wav");
            if first == "music" && lower.contains('/') {
                // (an Opus file takes the place of the .MSC stream of that name)
                let key = if lower.ends_with(".opus") { format!("{}.msc", &lower[..lower.len() - 5]) } else { lower };
                overlay.push((key, host));
                count += 1;
            } else if lower.ends_with(".png") && std::env::var_os("TS_PNG_QPM").is_none() {
                register_picture(game_dir, &m.name, &lower, &host, &mut original_packs);
                count += 1;
            } else if lower.ends_with(".png") {
                if let Some((name, file)) = convert_texture(game_dir, &cache, &m.name, &lower, &host, &mut original_packs) {
                    let (pack, inner) = match name.split_once('/').and_then(|(f, rest)| PACKS.iter().find(|p| p.eq_ignore_ascii_case(f)).map(|p| (*p, rest.to_string()))) {
                        Some(x) => x,
                        None => ("MISC.PAK", name.clone()),
                    };
                    pack_changes.entry(pack).or_default().insert(inner, file);
                    count += 1;
                }
            } else if audio {
                if let Some((name, file)) = convert_sound(game_dir, &cache, &m.name, &lower, &host, &mut original_packs) {
                    let (pack, inner) = match name.split_once('/').and_then(|(f, rest)| PACKS.iter().find(|p| p.eq_ignore_ascii_case(f)).map(|p| (*p, rest.to_string()))) {
                        Some(x) => x,
                        None => match contents.iter().find(|(_, names)| names.iter().any(|n| *n == name.as_bytes())) {
                            Some((pack, _)) => (*pack, name.clone()),
                            None => ("SFX.PAK", name.clone()),
                        },
                    };
                    pack_changes.entry(pack).or_default().insert(inner, file);
                    count += 1;
                }
            } else if let Some(pack) = PACKS.iter().find(|p| p.eq_ignore_ascii_case(first)) {
                let inner = lower[first.len() + 1..].to_string();
                pack_changes.entry(pack).or_default().insert(inner, host);
                count += 1;
            } else {
                match contents.iter().find(|(_, names)| names.iter().any(|n| *n == lower.as_bytes())) {
                    Some((pack, _)) => {
                        pack_changes.entry(pack).or_default().insert(lower, host);
                        count += 1;
                    }
                    None => eprintln!("[mods] {}: {rel} is not in any pack (put it in a folder named after the pack, e.g. OB.PAK/{rel})", m.name),
                }
            }
        }
        eprintln!("[mods] {}: {count} file(s)", m.name);
    }
    add_levels(game_dir, &cache, &level_lists, &mut pack_changes, &mut original_packs);
    for (pack, changes) in &pack_changes {
        let out = cache.join(pack);
        let key = cache.join(format!("{pack}.key"));
        let want = stamp(changes);
        if std::fs::read_to_string(&key).ok().as_deref() != Some(want.as_str()) || !out.is_file() {
            eprintln!("[mods] building {pack} ({} file(s))...", changes.len());
            if let Err(e) = build_pack(&game_dir.join("PAK").join(pack), changes, &out) {
                eprintln!("[mods] {pack}: {e}");
                continue;
            }
            let _ = std::fs::write(&key, want);
        }
        overlay.push((format!("pak/{}", pack.to_ascii_lowercase()), out));
    }
    ts_sdk::set_disc_overlay(overlay);
    apply_patches(exe, &patches);
    for (name, which, path) in &data_files {
        crate::moddata::load_file(name, which, path);
    }
}

/// The levels of the mods (`levels.txt`, one per line: `level <number 31..99> "<name>" like <level> [tune <track>] [modes dm,bagtag,ctb,escort,ko,ls]`):
/// they are told to the game, and a level that has no files of the mod's own gets copies of the files of the level it is like.
fn add_levels(game_dir: &Path, cache: &Path, lists: &[(String, PathBuf)], pack_changes: &mut BTreeMap<&str, BTreeMap<String, PathBuf>>, packs: &mut BTreeMap<&'static str, Option<(Vec<u8>, Vec<Entry>)>>) {
    let mut levels: Vec<ts_sdk::CustomLevel> = Vec::new();
    for (mod_name, file) in lists {
        let text = std::fs::read_to_string(file).unwrap_or_default();
        for (n, line) in text.lines().enumerate() {
            let line = line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let bad = |why: &str| eprintln!("[mods] {mod_name}/levels.txt line {}: {why}", n + 1);
            // (words, with a name in quotes as one)
            let mut words: Vec<String> = Vec::new();
            let mut cur = String::new();
            let mut quoted = false;
            for c in line.chars() {
                match c {
                    '"' => quoted = !quoted,
                    c if c.is_whitespace() && !quoted => {
                        if !cur.is_empty() {
                            words.push(std::mem::take(&mut cur));
                        }
                    }
                    c => cur.push(c),
                }
            }
            if !cur.is_empty() {
                words.push(cur);
            }
            if words.len() < 4 || words[0] != "level" {
                bad("expected `level <number> \"<name>\" like <level> [tune <track>] [modes ...]`");
                continue;
            }
            let Ok(number) = words[1].parse::<i32>() else { bad("the number is not a number"); continue };
            if !(31..=99).contains(&number) || levels.iter().any(|l| l.number == number) {
                bad("the number has to be from 31 to 99, and not used by another level");
                continue;
            }
            let mut lv = ts_sdk::CustomLevel { number, name: words[2].clone(), like: 0, tune: 0, modes: None, setup: None };
            let mut i = 3;
            while i + 1 < words.len() {
                match words[i].as_str() {
                    "like" => lv.like = words[i + 1].parse().unwrap_or(0),
                    "tune" => lv.tune = words[i + 1].parse::<i32>().unwrap_or(0).clamp(0, 99),
                    "modes" => {
                        let mut mask = 0;
                        for m in words[i + 1].split(',') {
                            mask |= match m.to_ascii_lowercase().as_str() {
                                "dm" | "deathmatch" => 1,
                                "bagtag" => 2,
                                "ctb" | "capture" => 4,
                                "escort" => 8,
                                "ko" | "knockout" => 16,
                                "ls" | "laststand" => 32,
                                _ => 0,
                            };
                        }
                        lv.modes = Some(mask);
                    }
                    "setup" => {
                        let path = file.parent().unwrap_or(Path::new(".")).join(&words[i + 1]);
                        match read_setup(&path) {
                            Ok(s) => lv.setup = Some(s),
                            Err(e) => bad(&format!("setup {}: {e}", words[i + 1])),
                        }
                    }
                    _ => {}
                }
                i += 2;
            }
            if !(1..=30).contains(&lv.like) {
                bad("`like` has to be one of the game's levels (1 to 30)");
                continue;
            }
            // (the files of the level: the mod's own, or copies of those of the level it is like)
            let names = [
                ("BG.PAK", format!("bg/level{}/level{}.raw", number, number), format!("bg/level{}/level{}.raw", lv.like, lv.like)),
                ("MISC.PAK", format!("pad/data/level{}.raw", number), format!("pad/data/level{}.raw", lv.like)),
                ("MISC.PAK", format!("pad/data/level{}_reduced.raw", number), format!("pad/data/level{}_reduced.raw", lv.like)),
                ("MISC.PAK", format!("propsetup/data/level{}.raw", number), format!("propsetup/data/level{}.raw", lv.like)),
            ];
            for (pack, new, old) in names {
                let pack: &'static str = PACKS.iter().find(|p| **p == pack).unwrap();
                if pack_changes.get(pack).map_or(false, |c| c.contains_key(&new)) {
                    continue;
                }
                let Some((d, entries)) = packs.entry(pack).or_insert_with(|| read_pack(&game_dir.join("PAK").join(pack))).as_ref() else { continue };
                let Some(e) = entries.iter().find(|e| e.name == old.as_bytes()) else { continue };
                let Some(bytes) = d.get(e.offset as usize..(e.offset + e.size) as usize) else { continue };
                let dir = cache.join("levels");
                let _ = std::fs::create_dir_all(&dir);
                let out = dir.join(new.replace('/', "__"));
                if std::fs::write(&out, bytes).is_ok() {
                    pack_changes.entry(pack).or_default().insert(new, out);
                }
            }
            eprintln!("[mods] {mod_name}: level {number} {:?}, like level {}", lv.name, lv.like);
            levels.push(lv);
        }
    }
    levels.sort_by_key(|l| l.number);
    ts_sdk::set_custom_levels(levels);
}

/// The set-up file of a level (`setup <file>` of `levels.txt`; `tools/obj2level.py` writes one): one entry to a line, `modes dm,bagtag,..`,
/// `spawn <pad>` where players and bots start, `weapon <slot 1..5> <pad>` where a weapon of the weapon set's slot lies, `pickup <type> <pad>`
/// where a pickup of the game's number lies.  The pads are the ids of the level's pad file (1001 and up).
fn read_setup(path: &Path) -> Result<ts_sdk::CustomSetup, String> {
    let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut s = ts_sdk::CustomSetup::default();
    for (n, line) in text.lines().enumerate() {
        let line = line.split('#').next().unwrap_or("").trim();
        let w: Vec<&str> = line.split_whitespace().collect();
        let num = |i: usize| w.get(i).and_then(|x| x.parse::<i32>().ok()).ok_or_else(|| format!("line {}: a number is missing", n + 1));
        match w.first().copied() {
            None => {}
            Some("modes") => {
                let mut mask = 0;
                for m in w.get(1).unwrap_or(&"").split(',') {
                    mask |= match m.to_ascii_lowercase().as_str() {
                        "dm" | "deathmatch" => 1,
                        "bagtag" => 2,
                        "ctb" | "capture" => 4,
                        "escort" => 8,
                        "ko" | "knockout" => 16,
                        "ls" | "laststand" => 32,
                        _ => 0,
                    };
                }
                s.modes = Some(mask);
            }
            Some("spawn") => s.spawns.push(num(1)?),
            Some("weapon") => {
                let slot = num(1)?;
                if !(1..=5).contains(&slot) {
                    return Err(format!("line {}: the weapon slot has to be 1 to 5", n + 1));
                }
                s.pickups.push((-slot, num(2)?));
            }
            Some("pickup") => s.pickups.push((num(1)?, num(2)?)),
            Some(other) => return Err(format!("line {}: `{other}` is not known", n + 1)),
        }
    }
    Ok(s)
}

fn apply_patches(exe: &Path, patches: &[(String, String)]) {
    if patches.is_empty() {
        return;
    }
    let symbols = ts_rt::loader::read_symbols(exe).unwrap_or_default();
    for (name, file) in patches {
        let text = std::fs::read_to_string(file).unwrap_or_default();
        for (n, line) in text.lines().enumerate() {
            let line = line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let mut it = line.split_whitespace();
            let (Some(target), Some(kind)) = (it.next(), it.next()) else {
                eprintln!("[mods] {name}/patches.txt line {}: expected `<address> <b|h|w|f> <values>`", n + 1);
                continue;
            };
            let (base, off) = match target.split_once('+') {
                Some((b, o)) => (b, parse_num(o).unwrap_or(0)),
                None => (target, 0),
            };
            let addr = parse_num(base).or_else(|| symbols.iter().find(|s| s.name == base).map(|s| s.addr as i64));
            let Some(mut addr) = addr.map(|a| a + off) else {
                eprintln!("[mods] {name}/patches.txt line {}: unknown symbol {base}", n + 1);
                continue;
            };
            if kind == "s" {
                // `<address> s <text>`: the text (as it is written after the `s`) and a zero byte
                let text = line.splitn(3, char::is_whitespace).nth(2).unwrap_or("").trim();
                for c in text.bytes().chain(std::iter::once(0)) {
                    ts_rt::sb(addr as i32, c as i32);
                    addr += 1;
                }
                continue;
            }
            for v in it {
                use ts_rt::*;
                match kind {
                    "b" => {
                        sb(addr as i32, parse_num(v).unwrap_or(0) as i32);
                        addr += 1;
                    }
                    "h" => {
                        sh(addr as i32, parse_num(v).unwrap_or(0) as i32);
                        addr += 2;
                    }
                    "f" => {
                        swc1(addr as i32, v.parse::<f32>().unwrap_or(0.0));
                        addr += 4;
                    }
                    _ => {
                        sw(addr as i32, parse_num(v).unwrap_or(0) as i32);
                        addr += 4;
                    }
                }
            }
        }
        eprintln!("[mods] {name}: patches applied");
    }
}

fn parse_num(s: &str) -> Option<i64> {
    let s = s.trim();
    if let Some(h) = s.strip_prefix("0x") {
        i64::from_str_radix(h, 16).ok()
    } else if let Some(h) = s.strip_prefix("-0x") {
        i64::from_str_radix(h, 16).ok().map(|v| -v)
    } else {
        s.parse().ok()
    }
}
