//! Texture "memory" as the game sees it: images uploaded at block addresses plus the palettes uploaded next to them.
//! This is an asset-binding table (address -> pixels), not a model of GS memory layout.

use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Clone, Debug)]
pub struct Image {
    pub dbp: u32,
    pub dbw: u32,
    pub psm: u32,
    pub w: u32,
    pub h: u32,
    /// raw pixel stream exactly as uploaded (raster order, `psm` packing)
    pub data: Vec<u8>,
    /// bumped every time the image is replaced (lets the renderer invalidate GPU copies)
    pub generation: u64,
}

#[derive(Default)]
pub struct Vram {
    pub images: HashMap<u32, Image>,
    next_gen: u64,
}

impl Vram {
    pub fn write(&mut self, mut img: Image) {
        self.next_gen += 1;
        img.generation = self.next_gen;
        // A partial update (x/y offset) is not used by the game for textures; whole-image replace is enough.
        self.images.insert(img.dbp, img);
    }

    /// Find the image that covers block address `bp` (exact start first, otherwise the image containing it).
    pub fn find(&self, bp: u32) -> Option<&Image> {
        if let Some(i) = self.images.get(&bp) {
            return Some(i);
        }
        self.images.values().find(|i| {
            let blocks = (i.data.len() as u32).div_ceil(256);
            bp >= i.dbp && bp < i.dbp + blocks
        })
    }
}

static VRAM: Mutex<Option<Vram>> = Mutex::new(None);

pub fn with_vram<R>(f: impl FnOnce(&mut Vram) -> R) -> R {
    let mut g = VRAM.lock().unwrap();
    f(g.get_or_insert_with(Vram::default))
}
