//! Data structures that can be shared between the server and the client.
//!
//! All these structures should be serializable, so that they can be
//! transferred from the server to the client over the network.
extern crate rand;

use std::convert::Into;
use std::fmt;


/// A unit identifier.
#[derive(RustcEncodable, RustcDecodable, PartialEq, Eq, Debug, Copy, Clone)]
pub struct UnitId(pub u32);

impl Into<UnitId> for u32 {
    fn into(self) -> UnitId {
        UnitId(self)
    }
}

impl fmt::Display for UnitId {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.0.fmt(f)
    }
}

/// A client/player identifier.
#[derive(RustcEncodable, RustcDecodable, PartialEq, Eq, Debug, Copy, Clone)]
pub struct ClientId(pub u32);

impl Into<ClientId> for u32 {
    fn into(self) -> ClientId {
        ClientId(self)
    }
}

impl fmt::Display for ClientId {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.0.fmt(f)
    }
}


/// The state of a single unit
#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug, Clone)]
pub struct Unit {
    /// The unit identifier
    pub id: UnitId,

    /// X/Y position in the world in mm
    pub position: [u64; 2],

    /// Angle of the unit in milli degrees
    pub angle: u32,

    /// Direction and speed of the movement. The angle of the movement may be different then the
    /// angle of the unit. The unit is mm per milli second.
    pub speed_vector: [u64; 2],

    /// Health of the unit
    pub health: u64,
}

impl Unit {
    pub fn new<T: Into<UnitId>>(id: T, position: [u64; 2]) -> Unit {
        println!("Create unit at {:?}", position);
        Unit {
            id: id.into(),
            position: position,
            angle: 0,
            speed_vector: [0,0],
            health: 100_0000,
        }
    }

    // Like `new`, but generate a random unit id.
    // TODO: Id conflicts are possible, so this needs to be changed.
    pub fn new_random(position: [u64; 2]) -> Unit {
        let id = rand::random::<u32>();
        Unit::new(id, position)
    }

    pub fn update(&mut self, dt_ms: u64) {
        self.position[0] += self.speed_vector[0] * dt_ms;
        self.position[1] += self.speed_vector[1] * dt_ms;
    }
}


/// A player has an ID and consists of 0..N `Unit`s
#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug, Clone)]
pub struct Player {
    pub id: ClientId,
    pub units: Vec<Unit>,
}

impl Player {
    pub fn new<T: Into<ClientId>>(id: T) -> Player {
        Player {
            id: id.into(),
            units: vec![],
        }
    }
}

/// Data related to the current game.
///
/// This needs to be transferred to the client every time the game state
/// changes.
#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug, Clone)]
pub struct GameState {
    pub players: Vec<Player>,
}

/// Data related to the entire world, like width and height.
///
/// This needs to be transferred to the client only once, on connecting.
#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug, Clone)]
pub struct WorldState {
    /// Width of the world in mm
    pub x: u64,
    /// Height of the world in mm
    pub y: u64,
    pub game: GameState,
}

impl WorldState {
    pub fn new(x: u64, y: u64) -> WorldState {
        WorldState {
            x: x,
            y: y,
            game: GameState {
                players: vec![],
            }
        }
    }
}

