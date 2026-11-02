//! Room for up to eight players in a match.  The game was made for four: its per-player tables (the sign-on of each player, the in-game menu of each, the team of each, the
//! parameters of each one's HUD) are arrays of four at fixed places, with other data right after them.  They are moved here, to places of the port's own, with room for eight;
//! the game's code names them through these constants.

/// the port's memory above the console's
const BASE: i32 = 0x240_0000;
// (a place the port's other tables do not use: they are below BASE + 0xa_0000)
/// the most players of a match
pub const MAX_MATCH_PLAYERS: i32 = 8;
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
pub const FRONT_BUFFERS_EXT: i32 = BASE + 0x10_2000;
/// the parameters of each player's HUD: records of 352 bytes (the game's were 4 at 0x36f838)
pub const HUDPARMS: i32 = BASE + 0x10_0500;
pub const HUDPARMS_BYTES: i32 = 352 * MAX_MATCH_PLAYERS;

/// the target each player hit last (words; the game's were 4 at 0x1fe99b0)
pub const PLAYER_HIT_AT: i32 = BASE + 0x10_3200;
