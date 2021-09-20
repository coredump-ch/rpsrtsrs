//! Network protocol.
//!
//! Everything related to the network protocol between the sever and the
//! clients.

use crate::common::Vec2;
use crate::state::{ClientId, GameState, UnitId, WorldState};

/// Commands alter the game state.
///
/// A command is sent from the client to the server. Examples include the
/// movement of a unit or the decision to attack another unit.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Command {
    /// Move command with unit ID and target
    Move(UnitId, Vec2),
    /// Let the unit shoot
    Shoot(UnitId),
}

/// Primary message type sent between server and client.
///
/// This includes connection buildup and game state transfer.
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum Message {
    Error,
    ClientHello,
    ClientReconnect(ClientId),
    ServerHello(ClientId, WorldState),
    UpdateGamestate(GameState),
    Command(Command),
}
