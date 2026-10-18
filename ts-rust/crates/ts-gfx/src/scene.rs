//! Output of the render front-end: a backend-independent list of draw calls for one frame.

/// Pixel formats of images in (virtual) GS memory.
pub mod psm {
    pub const CT32: u32 = 0;
    pub const CT24: u32 = 1;
    pub const CT16: u32 = 2;
    pub const CT16S: u32 = 10;
    pub const T8: u32 = 19;
    pub const T4: u32 = 20;
    pub const T8H: u32 = 27;
    pub const T4HL: u32 = 36;
    pub const T4HH: u32 = 44;
}

/// Identifies the texture a draw samples (names image + palette in "GS memory").
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TexKey {
    pub tbp0: u32,
    pub tbw: u32,
    pub psm: u32,
    pub tw: u32,
    pub th: u32,
    pub tcc: u32,
    pub tfx: u32,
    pub cbp: u32,
    pub cpsm: u32,
    pub csa: u32,
    pub csm: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub struct Blend {
    pub enabled: bool,
    /// raw ALPHA_1 register: A,B,C,D select fields and FIX
    pub alpha: u64,
}

/// Fixed-function state captured at the time of a draw.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DrawState {
    pub tex: Option<TexKey>,
    pub blend: Blend,
    /// TEST_1 raw register
    pub test: u64,
    /// ZBUF_1: z write mask in bit 32 (ZMSK)
    pub zmsk: bool,
    /// FRAME_1 FBMSK (bits 32..63): masked colour channels
    pub fbmsk: u32,
    pub clamp: u64,
    /// TEX1_1 raw (filtering)
    pub tex1: u64,
    pub scissor: [i32; 4],
    /// true when the primitive uses perspective S,T,Q coordinates; otherwise UV texel units
    pub use_stq: bool,
    pub fog: bool,
    pub gouraud: bool,
    /// frame buffer address (FRAME_1 FBP): identifies the render target (screen vs. off-screen copy)
    pub fbp: u32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Vertex {
    /// screen-space position: x,y in GS pixels relative to the draw offset, z 24-bit depth value
    pub pos: [f32; 3],
    /// RGBA, 0..255 (128 = 1.0 for texture modulation)
    pub color: [f32; 4],
    /// S,T,Q (perspective) or U,V in texels with q = 1
    pub st: [f32; 3],
    pub fog: f32,
    /// movement on screen since the previous game frame (field pixels); set for frame interpolation
    pub vel: [f32; 2],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Topology {
    Triangles,
    Lines,
    Points,
}

#[derive(Clone, Debug)]
pub struct Draw {
    pub state: DrawState,
    pub topology: Topology,
    /// range into `Frame::vertices`
    pub first: u32,
    pub count: u32,
    /// entry address of the VU1 microprogram that produced the draw (identifies what kind of geometry it is)
    pub prog: u32,
    /// drawn by a function that places sprites by projecting world positions (glows, flares, name tags)
    pub world: bool,
    /// number of the VU1 kick (one mesh batch) the draw came from
    pub kick: u32,
}

/// An image transfer into texture memory that happened while the list was processed.
#[derive(Clone, Debug)]
pub struct ImageWrite {
    pub dbp: u32,
    pub dbw: u32,
    pub psm: u32,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub data: Vec<u8>,
}

/// A texture decoded from texture memory as it was when the frame was built.
#[derive(Debug)]
pub struct DecodedTex {
    /// the size of the picture in pixels
    pub w: u32,
    pub h: u32,
    /// the size the game's coordinates address (the console's texture; a replacement picture of a mod can be of another size)
    pub lw: u32,
    pub lh: u32,
    pub rgba: Vec<u8>,
    pub sig: u64,
}

#[derive(Default, Debug, Clone)]
pub struct Frame {
    /// Snapshot of every texture the draws use.  Texture memory keeps changing while later frames are built and the
    /// drawing thread may show this frame (or blends of it) after that, so the pixels must travel with the frame.
    pub textures: std::collections::HashMap<TexKey, std::sync::Arc<DecodedTex>>,
    /// the same vertex ranges split at every VU1 kick (mesh batch): what frame interpolation pairs up
    pub segs: Vec<Draw>,
    /// the level is being played (as opposed to the front-end menus)
    pub in_game: bool,
    /// a front-end page (menu, pop-up panel) is open on top of whatever is drawn
    pub page_open: bool,
    /// the number of players of the level being played (the panes of the split screen); 0 or 1 outside a level
    pub players: u8,
    /// a menu of the running level is open (the developer menu)
    pub menu_open: bool,
    /// a player's score table (the select button) is shown over the level
    pub scores_open: bool,
    /// an attract-mode demo is playing: it runs with the original 4:3 view and is shown that way
    pub demo: bool,
    /// the world is being drawn with no menu on top: in-between frames may be blended
    pub smooth: bool,
    pub vertices: Vec<Vertex>,
    pub draws: Vec<Draw>,
    pub uploads: Vec<ImageWrite>,
    /// statistics for diagnostics
    pub unknown_programs: Vec<u32>,
}

/// Frame buffer page (FBP, in 32-block units: 0 or 40) the game is currently drawing into.  The game's display-list
/// setup runs outside the list, so the host publishes it here for the renderer's screen-copy logic.
pub static CURRENT_FBP: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
