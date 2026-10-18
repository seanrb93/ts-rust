//! Which file each texture of the game was loaded from: the game's loader (`emLoadTexture`) notes the name of the file and the block address
//! its picture was given in texture memory, and the renderer asks which file a texture is when it decodes it (to show a replacement picture
//! of a mod instead).

use std::collections::HashMap;
use std::sync::Mutex;

static NAMES: Mutex<Option<HashMap<u32, String>>> = Mutex::new(None);

/// A texture file's name as the mods spell it: lower case, no `host0:` or `cdrom0:`, forward slashes, no `.qpm` (`textures/misc/ammo_big`).
pub fn normalise(name: &str) -> String {
    let n = name.trim().to_ascii_lowercase().replace('\\', "/");
    let n = n.trim_start_matches("host0:").trim_start_matches("cdrom0:").trim_start_matches('/');
    n.strip_suffix(".qpm").unwrap_or(n).to_string()
}

/// The texture at block address `tbp0` of texture memory is the file `name`.
pub fn note_texture(tbp0: u32, name: &str) {
    NAMES.lock().unwrap().get_or_insert_with(HashMap::new).insert(tbp0, normalise(name));
}

/// The file the texture at `tbp0` was loaded from, as `normalise` spells it.
pub fn texture_name(tbp0: u32) -> Option<String> {
    NAMES.lock().unwrap().as_ref()?.get(&tbp0).cloned()
}
