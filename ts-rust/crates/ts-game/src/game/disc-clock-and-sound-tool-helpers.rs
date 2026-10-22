//! Functions: reading sectors of the disc, the clock of the console, the start of the sound tool, and the end of the sound.

use crate::*;

/// Read `a1` sectors of the disc from the sector `a0` and wait (@ 0x2b35a0).
pub fn cd_read(a0: i32, a1: i32) {
    sb(g::CDmode, 20);
    sb(g::CDmode + 1, 1);
    sb(g::CDmode + 2, 0);
    sceCdRead(&[((a1 as i64) << 32) >> 32, ((a0 as i64) << 32) >> 32, 0, g::CDmode as i64]);
    sceCdSync(&[0]);
}

/// The time of the clock of the console as a number (@ 0x2b3c50), or -1 when it cannot be read.
pub fn rtcGetTime() -> i32 {
    let frame = Frame::new(32);
    let sp = frame.sp();
    if sceCdReadClock(&[sp as i64]) == 0 {
        return -1;
    }
    sceScfGetLocalTimefromRTC(&[sp as i64]);
    clock2u32(sp)
}

/// Start the page of the sound tool (@ 0x207790).
pub fn soundtoolStart() {
    calcPitches();
    let page = lw(g::front);
    sw(page + 12, 0);
    sw(page + 20, 0);
    frontPageSet(0, g::main_page_329710);
    let page = lw(g::front);
    sw(page + 12, 0);
    sw(page + 40, 0);
    setcursound(lw(g::snd));
}

/// Stop the sound: key off the voices of the sound chip (@ 0x2048b8).
pub fn soundEnd() {
    sceSdRemote(&[1, 0x8030, 0x1600, 0xffffff]);
    sceSdRemote(&[1, 0x8030, 0x1601, 0xffffff]);
}
