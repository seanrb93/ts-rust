//! Room for up to eight players in a match.  The game was made for four: its per-player tables (the sign-on of each player, the in-game menu of each, the team of each, the
//! parameters of each one's HUD) are arrays of four at fixed places, with other data right after them.  They are moved here, to places of the port's own, with room for eight;
//! the game's code names them through these constants.

/// the port's memory above the console's
const BASE: i32 = 0x240_0000;
// (a place the port's other tables do not use: they are below BASE + 0xa_0000)
/// the most players of a match
pub const MAX_MATCH_PLAYERS: i32 = 10;
/// the in-game menu of each player: records of 92 bytes (the game's were 4 at 0x1fb3240)
pub const IGMENU: i32 = BASE + 0x10_0000;
pub const IGMENU_BYTES: i32 = 92 * MAX_MATCH_PLAYERS;
/// the team of each player: words (the game's were 4 at 0x1fc2c88)
pub const TEAMS: i32 = BASE + 0x10_0400;
/// the sign-on of each player: words (the game's were 4 at 0x1fbc158)
pub const SIGNONS: i32 = BASE + 0x10_0440;
/// the definitions of the front end (the in-game menu of each player): 44 bytes each (the game's were 4 at 0x353f50), their tables of pages (64 bytes each) and boxes (3072
/// bytes each) for the players after the fourth (the first four have the game's own)
pub const FRONTDEFS: i32 = BASE + 0x10_1000;
pub const FRONT_TABLES_EXT: i32 = BASE + 0x10_1800;
pub const FRONT_BUFFERS_EXT: i32 = BASE + 0x10_a000;
/// the parameters of each player's HUD: records of 352 bytes (the game's were 4 at 0x36f838)
pub const HUDPARMS: i32 = BASE + 0x10_9000;
pub const HUDPARMS_BYTES: i32 = 352 * MAX_MATCH_PLAYERS;

/// the target each player hit last (words; the game's were 4 at 0x1fe99b0)
pub const PLAYER_HIT_AT: i32 = BASE + 0x10_6200;

/// The rooms of a player's own props (the view of his gun, and his body): the game has numbers for four players, 201 to 204 and 205 to 208 (209 to 212 are for everybody), so the
/// players after the fourth have numbers after them.
pub fn gun_room(player: i32) -> i32 {
    if player < 4 { 201 + player } else { 213 + 2 * (player - 4) }
}

/// The bit of a player in the flags of a prop (from bit 24, the viewers that may see it): the flags have room for four players (the bits after are other flags), so the players
/// after the fourth have all four, and the drawing tells them apart (`prop_hidden_from`).
pub fn view_bit(player: i32) -> i32 {
    if player < 4 { 0x1000000 << player } else { 0xf000000 }
}

/// Whether the prop `prop` is not drawn for the player `player` (a record of the players): the flags say so for the first four players, and a player after the fourth does
/// not see his own body when his camera is in his head.
pub fn prop_hidden_from(player: i32, prop: i32) -> bool {
    let flags = crate::lw(prop + 16);
    if player < 4 {
        return flags & view_bit(player) == 0;
    }
    if flags & 0xf000000 == 0 {
        return true;
    }
    crate::lw(crate::lw(crate::dec::g::currentplayer) + 384) == prop && crate::camGetMode() != 2 && crate::lw(crate::dec::g::lvlevelnum) != 6
}

pub fn body_room(player: i32) -> i32 {
    if player < 4 { 205 + player } else { 214 + 2 * (player - 4) }
}
