//! Mines attached to props.

use crate::*;

/// Show the mines attached to prop `prop` and hide the others: of the nodes 20 to 49 of the bullet pool that are in use, flag 1 (hidden) of the three
/// objects of the node (+96, +140, +144) is cleared when the node's prop (+224) is `prop` and set otherwise (@ 0x28d2d8).
pub fn bulletMakeAttachedMinesInvisible(prop: i32) {
    let nodes = lw(0x3b0138);
    for i in 20..50 {
        let node = nodes + 276 * i;
        if lw(node) == 0 {
            continue;
        }
        let attached = lw(node + 224) == prop;
        for at in [96, 140, 144] {
            let object = lw(node + at);
            if object != 0 {
                let flags = lw(object + 16);
                sw(object + 16, if attached { flags & -2 } else { flags | 1 });
            }
        }
    }
}
