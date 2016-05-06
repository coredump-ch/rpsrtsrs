//! Network protocol.
//!
//! Everything related to the network protocol between the sever and the
//! clients.

use state::{GameState, WorldState, UnitId, ClientId};

/// Commands alter the game state.
///
/// A command is sent from the client to the server. Examples include the
/// movement of a unit or the decision to attack another unit.
#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug)]
pub enum Command {
    /// Move command with unit ID and target
    Move(UnitId, [u64;2]),
}

/// Primary message type sent between server and client.
///
/// This includes connection buildup and game state transfer.
#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug)]
pub enum Message {
    Error,
    ClientHello,
    ClientReconnect(ClientId),
    ServerHello(ClientId, WorldState),
    UpdateGamestate(GameState),
    Command(Command),
}
