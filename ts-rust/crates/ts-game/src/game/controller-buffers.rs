//! Putting a controller's state into a byte buffer and taking it out again (the replay and the link between machines use it).

use crate::*;

/// The controller record (384 bytes) of a player, with the player's number made to fit the four slots (the same arithmetic as `GetControllerNumber`).
fn controller_of(player: i32) -> i32 {
    let mut slot = player;
    if slot >= ilinkGetNumLocalPlayers() {
        slot -= ilinkGetNumLocalPlayers() - 4;
    }
    lw((slot << 2) + g::playercontrollers) * 384 + g::controllers
}

/// Add a player's controller state to the buffer `buffer` (the address at which writing continues; a cursor of its own is made): the
/// four button words (+288, +340, +292, +296), four stick bytes (+300 to +312) and the dead zone (+316) (@ 0x2041d0).
pub fn PutControllerInBuff(player: i32, buffer: i32) {
    let frame = Frame::new(16);
    let cursor = frame.sp();
    sw(cursor, buffer);
    let c = controller_of(player);
    BuffAddu16(cursor, lhu(c + 288));
    BuffAddu16(cursor, lhu(c + 340));
    BuffAddu16(cursor, lhu(c + 292));
    BuffAddu16(cursor, lhu(c + 296));
    BuffAddu8(cursor, lbu(c + 300));
    BuffAddu8(cursor, lbu(c + 304));
    BuffAddu8(cursor, lbu(c + 308));
    BuffAddu8(cursor, lbu(c + 312));
    BuffAddfloat(cursor, lwc1(c + 316));
}

/// Read a player's controller state back from the buffer in the same order (@ 0x2042b0).
pub fn GetControllerFromBuff(player: i32, buffer: i32) {
    let frame = Frame::new(16);
    let cursor = frame.sp();
    sw(cursor, buffer);
    let mut slot = player;
    if slot >= ilinkGetNumLocalPlayers() {
        slot -= ilinkGetNumLocalPlayers() - 4;
    }
    let number = lw((slot << 2) + g::playercontrollers);
    let c = number * 384 + g::controllers;
    sw(c + 288, BuffRemoveu16(cursor));
    sw(c + 340, BuffRemoveu16(cursor));
    sw(c + 292, BuffRemoveu16(cursor));
    sw(c + 296, BuffRemoveu16(cursor));
    sw(c + 300, BuffRemoveu8(cursor));
    sw(c + 304, BuffRemoveu8(cursor));
    sw(c + 308, BuffRemoveu8(cursor));
    sw(c + 312, BuffRemoveu8(cursor));
    swc1(c + 316, BuffRemovefloat(cursor));
}
