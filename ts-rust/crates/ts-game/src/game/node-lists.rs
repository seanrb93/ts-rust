//! Linked lists of the game: the node pool lists and the bots' pad links.

use crate::*;

/// Node pool of the doubly linked lists: `NODES` holds the address of an array of 276-byte nodes; a node has its `next` index at +4
/// and its `prev` index at +8 (-1 for none).  `HEADS` / `TAILS` hold the first / last node of each list.
const NODES: i32 = 0x3b0138;
const HEADS: i32 = 0x3b0140;
const TAILS: i32 = 0x3b0148;
const NODE_SIZE: i32 = 276;
const NEXT: i32 = 4;
const PREV: i32 = 8;

/// Put node `node` at the front of list `list` (@ 0x2894d0).
pub fn addhead(node: i32, list: i32) {
    let head = HEADS + (list << 2);
    let nodes = lw(NODES);
    sw(lw(head) * NODE_SIZE + nodes + PREV, node);
    let n = nodes + node * NODE_SIZE;
    let old = lw(head);
    sw(n + PREV, -1);
    sw(n + NEXT, old);
    sw(head, node);
}

/// Put node `node` at the end of list `list` (@ 0x289518).
pub fn addtail(node: i32, list: i32) {
    let tail = TAILS + (list << 2);
    let nodes = lw(NODES);
    sw(lw(tail) * NODE_SIZE + nodes + NEXT, node);
    let n = nodes + node * NODE_SIZE;
    sw(n + NEXT, -1);
    sw(n + PREV, lw(tail));
    sw(tail, node);
}

/// Remove a link between two pads of the bots' map (@ 0x264738).  A link is 20 bytes: flags at +0 (bit 0: in use), the pad numbers at +4
/// and +8.  Pads are 28 bytes, their hall at +8.  The two counters are of links inside a hall and links between halls.
pub fn deletelink(link: i32) {
    const SAME_HALL_LINKS: i32 = 0x3b00a4;
    const OTHER_HALL_LINKS: i32 = 0x3b00a8;
    let pads = lw(g::levelpads);
    let hall_of = |pad: i32| lh(pad * 28 + pads + 8);
    let hall_b = hall_of(lw(link + 8));
    let hall_a = hall_of(lw(link + 4));
    sw(link, lw(link) & !1);
    let counter = if hall_a != hall_b { OTHER_HALL_LINKS } else { SAME_HALL_LINKS };
    sw(counter, lw(counter) - 1);
}
