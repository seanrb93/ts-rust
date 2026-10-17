//! SPU ADPCM ("VAG") block decoding: 16-byte blocks -> 28 samples.

const COEF: [(i32, i32); 5] = [(0, 0), (60, 0), (115, -52), (98, -55), (122, -60)];

/// Decoder history (the two previous output samples).
#[derive(Clone, Copy, Default)]
pub struct State {
    pub s1: i32,
    pub s2: i32,
}

/// Decode one 16-byte block (`block[0]` = shift/filter, `block[1]` = flags) into 28 samples.
pub fn decode_block(block: &[u8], st: &mut State, out: &mut [i16; 28]) {
    let shift = (block[0] & 15) as u32;
    let filter = ((block[0] >> 4) & 7).min(4) as usize;
    let (c0, c1) = COEF[filter];
    let mut n = 0;
    for &b in &block[2..16] {
        for nib in [(b & 15) as i32, (b >> 4) as i32] {
            let nib = if nib >= 8 { nib - 16 } else { nib };
            let mut s = (nib << 12) >> shift.min(15);
            s += (st.s1 * c0 + st.s2 * c1 + 32) >> 6;
            s = s.clamp(-32768, 32767);
            out[n] = s as i16;
            n += 1;
            st.s2 = st.s1;
            st.s1 = s;
        }
    }
}
