//! Pictures that replace the game's textures (from mods), of any size and with full colour.  The game still loads its own texture; when the
//! renderer decodes a texture it asks which file it came from (`ts_rt::texnames`) and shows the picture of that name instead, over the same
//! surface: the picture's texels are addressed by the same coordinates as the original's, so it is stretched to fit.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

/// A picture as RGBA8 with the console's alpha (0 to 128, 128 = opaque).
pub struct HdImage {
    pub w: u32,
    pub h: u32,
    pub rgba: Vec<u8>,
}

/// Reads a picture file (the port has the PNG reader): (width, height, RGBA8 with alpha 0..128).
pub type Loader = fn(&std::path::Path) -> Option<(u32, u32, Vec<u8>)>;

static LOADER: OnceLock<Loader> = OnceLock::new();

pub fn set_loader(f: Loader) {
    let _ = LOADER.set(f);
}

/// Reads a picture from memory (the same, for the pictures the program carries itself).
pub type BytesLoader = fn(&[u8]) -> Option<(u32, u32, Vec<u8>)>;

static BYTES_LOADER: OnceLock<BytesLoader> = OnceLock::new();

pub fn set_bytes_loader(f: BytesLoader) {
    let _ = BYTES_LOADER.set(f);
}

/// Where a replacement comes from: a file of a mod, or a picture in the program.
enum Source {
    Path(PathBuf),
    Bytes(&'static [u8]),
}

/// A replacement: its file (read the first time the texture is used) and a number that changes when the file does.
pub struct Entry {
    pub id: u64,
    source: Source,
    image: OnceLock<Option<Arc<HdImage>>>,
}

impl Entry {
    pub fn image(&self) -> Option<Arc<HdImage>> {
        self.image
            .get_or_init(|| {
                let t = std::time::Instant::now();
                let (w, h, rgba, from) = match &self.source {
                    Source::Path(path) => {
                        let (w, h, rgba) = LOADER.get().and_then(|f| f(path))?;
                        (w, h, rgba, path.display().to_string())
                    }
                    Source::Bytes(bytes) => {
                        let (w, h, rgba) = BYTES_LOADER.get().and_then(|f| f(bytes))?;
                        (w, h, rgba, "(a picture of the program)".to_string())
                    }
                };
                eprintln!("[hd] {from} read: {w}x{h} ({:.0} ms)", t.elapsed().as_secs_f64() * 1e3);
                Some(Arc::new(HdImage { w, h, rgba }))
            })
            .clone()
    }
}

static FILES: Mutex<Option<HashMap<String, Arc<Entry>>>> = Mutex::new(None);
static ANY: AtomicBool = AtomicBool::new(false);
static NEXT: AtomicU64 = AtomicU64::new(1);

/// The picture `path` replaces the texture of this name (`textures/misc/ammo_big`).
pub fn register(name: &str, path: PathBuf) {
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    FILES.lock().unwrap().get_or_insert_with(HashMap::new).insert(ts_rt::texnames::normalise(name), Arc::new(Entry { id, source: Source::Path(path), image: OnceLock::new() }));
    ANY.store(true, Ordering::Relaxed);
}

// ---- the pictures that depend on the device in use (the buttons of an Xbox or Switch controller, the keys) -------------------------------------------------

static STYLE: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);
static VARIANTS: Mutex<Option<HashMap<(String, u8), Arc<Entry>>>> = Mutex::new(None);

/// The device in use (0 the game's own, 1 Xbox, 2 Switch, 3 keyboard): the pictures made for it replace the texture; a texture with none for it is as it was.
pub fn set_style(style: u8) {
    STYLE.store(style, Ordering::Relaxed);
}

/// A picture of the program that replaces the texture `name` while the device in use is `style`.  Registering the name again with another picture makes the texture be drawn from it.
pub fn register_variant(name: &str, style: u8, bytes: &'static [u8]) {
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    VARIANTS.lock().unwrap().get_or_insert_with(HashMap::new).insert((ts_rt::texnames::normalise(name), style), Arc::new(Entry { id, source: Source::Bytes(bytes), image: OnceLock::new() }));
    ANY.store(true, Ordering::Relaxed);
}

/// Take the pictures of a device back (the keyboard's change with the keys the player chose).
pub fn clear_variants(style: u8) {
    if let Some(m) = VARIANTS.lock().unwrap().as_mut() {
        m.retain(|(_, s), _| *s != style);
    }
}

/// The replacement of the texture at block address `tbp0`, if the mods have one.
pub fn for_texture(tbp0: u32) -> Option<Arc<Entry>> {
    if !ANY.load(Ordering::Relaxed) {
        return None;
    }
    let name = ts_rt::texnames::texture_name(tbp0)?;
    let style = STYLE.load(Ordering::Relaxed);
    if style != 0 {
        if let Some(e) = VARIANTS.lock().unwrap().as_ref().and_then(|m| m.get(&(name.clone(), style)).cloned()) {
            return Some(e);
        }
    }
    let e = FILES.lock().unwrap().as_ref()?.get(&name).cloned();
    if e.is_some() && std::env::var_os("TS_HDLOG").is_some() {
        static SEEN: Mutex<Option<std::collections::HashSet<(u32, String)>>> = Mutex::new(None);
        if SEEN.lock().unwrap().get_or_insert_with(Default::default).insert((tbp0, name.clone())) {
            eprintln!("[hd] {name} is at {tbp0:#x}");
        }
    }
    e
}
