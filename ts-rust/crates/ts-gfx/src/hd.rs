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

/// A replacement: its file (read the first time the texture is used) and a number that changes when the file does.
pub struct Entry {
    pub id: u64,
    path: PathBuf,
    image: OnceLock<Option<Arc<HdImage>>>,
}

impl Entry {
    pub fn image(&self) -> Option<Arc<HdImage>> {
        self.image
            .get_or_init(|| {
                let t = std::time::Instant::now();
                let (w, h, rgba) = LOADER.get().and_then(|f| f(&self.path))?;
                eprintln!("[hd] {} read: {w}x{h} ({:.0} ms)", self.path.display(), t.elapsed().as_secs_f64() * 1e3);
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
    FILES.lock().unwrap().get_or_insert_with(HashMap::new).insert(ts_rt::texnames::normalise(name), Arc::new(Entry { id, path, image: OnceLock::new() }));
    ANY.store(true, Ordering::Relaxed);
}

/// The replacement of the texture at block address `tbp0`, if the mods have one.
pub fn for_texture(tbp0: u32) -> Option<Arc<Entry>> {
    if !ANY.load(Ordering::Relaxed) {
        return None;
    }
    let name = ts_rt::texnames::texture_name(tbp0)?;
    let e = FILES.lock().unwrap().as_ref()?.get(&name).cloned();
    if e.is_some() && std::env::var_os("TS_HDLOG").is_some() {
        static SEEN: Mutex<Option<std::collections::HashSet<(u32, String)>>> = Mutex::new(None);
        if SEEN.lock().unwrap().get_or_insert_with(Default::default).insert((tbp0, name.clone())) {
            eprintln!("[hd] {name} is at {tbp0:#x}");
        }
    }
    e
}
