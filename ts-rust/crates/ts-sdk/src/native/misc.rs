//! Small SDK services that need no real hardware behind them.
#![allow(non_snake_case)]

/// `sceScfGetLocalTimefromRTC(clock)`: convert the RTC's JST value to local time in place.  `sceCdReadClock` already
/// reports the host's local-clock value, so no conversion is needed.
pub fn sceScfGetLocalTimefromRTC(_args: &[i64]) -> i32 {
    0
}

/// `sceGsSetDefDBuff(db, psm, w, h, ztest, zpsm, clear)`: fills a libgs double-buffer descriptor with GS register
/// values.  Those registers only matter to GS hardware; the port renders from the display-list contents instead, and
/// the game overwrites the fields it cares about right after this call.
pub fn sceGsSetDefDBuff(_args: &[i64]) -> i32 {
    0
}
