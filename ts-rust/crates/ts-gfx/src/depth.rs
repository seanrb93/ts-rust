//! The triangles of a frame that write depth, for the game's light glows (`ts_rt::depth`).

use crate::scene::{Frame, Topology};
use ts_rt::depth::DepthScene;

/// Fill `scene` with the frame's triangles (it is used again, so that a frame needs no new memory).
pub fn fill_scene(frame: &Frame, scene: &mut DepthScene) {
    scene.clear();
    for d in &frame.draws {
        if d.state.zmsk || d.topology != Topology::Triangles {
            continue;
        }
        let ztst = ((d.state.test >> 17) & 3) as u8;
        let zte = (d.state.test >> 16) & 1 != 0;
        let verts = &frame.vertices[d.first as usize..(d.first + d.count) as usize];
        scene.triangles.extend(verts.chunks_exact(3).map(|t| [t[0].pos, t[1].pos, t[2].pos]));
        scene.runs.push((scene.triangles.len() as u32, zte, ztst));
    }
}
